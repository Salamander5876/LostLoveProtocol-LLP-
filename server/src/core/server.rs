use anyhow::Context;
use bytes::{Bytes, BytesMut};
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast;
use tokio::time;
use tracing::{debug, error, info, warn};

use crate::config::Config;
use crate::core::connection::ConnectionManager;
use crate::core::session::SessionState;
use crate::error::{LostLoveError, Result};
use crate::protocol::{HandshakeMessage, Packet, PacketType, HEADER_SIZE};

/// Server shutdown signal
type ShutdownSignal = broadcast::Receiver<()>;

/// LostLove Server
pub struct Server {
    config: Arc<Config>,
    connection_manager: Arc<ConnectionManager>,
    shutdown_tx: broadcast::Sender<()>,
}

impl Server {
    /// Create new server
    pub async fn new(config: Config) -> anyhow::Result<Self> {
        info!("Initializing LostLove Server v{}", env!("CARGO_PKG_VERSION"));

        let (shutdown_tx, _) = broadcast::channel(1);

        let connection_manager = Arc::new(ConnectionManager::new(config.server.max_connections));

        Ok(Self {
            config: Arc::new(config),
            connection_manager,
            shutdown_tx,
        })
    }

    /// Run the server
    pub async fn run(&self) -> anyhow::Result<()> {
        let addr = format!("{}:{}", self.config.server.bind_address, self.config.server.port);

        info!("Starting TCP listener on {}", addr);

        let listener = TcpListener::bind(&addr)
            .await
            .context(format!("Failed to bind to {}", addr))?;

        info!("Server listening on {}", addr);
        info!("Max connections: {}", self.config.server.max_connections);
        info!("Protocol: {}", self.config.server.protocol);

        // Start background tasks
        self.start_background_tasks();

        // Main accept loop
        loop {
            match listener.accept().await {
                Ok((stream, addr)) => {
                    debug!("New TCP connection from {}", addr);

                    let connection_manager = self.connection_manager.clone();
                    let config = self.config.clone();
                    let mut shutdown_rx = self.shutdown_tx.subscribe();

                    // Spawn connection handler
                    tokio::spawn(async move {
                        tokio::select! {
                            result = handle_connection(stream, addr, connection_manager, config) => {
                                if let Err(e) = result {
                                    error!("Connection error from {}: {}", addr, e);
                                }
                            }
                            _ = shutdown_rx.recv() => {
                                info!("Shutdown signal received, closing connection from {}", addr);
                            }
                        }
                    });
                }
                Err(e) => {
                    error!("Failed to accept connection: {}", e);
                }
            }
        }
    }

    /// Start background tasks
    fn start_background_tasks(&self) {
        let connection_manager = self.connection_manager.clone();
        let timeout = Duration::from_secs(self.config.limits.connection_timeout);

        // Cleanup task
        tokio::spawn(async move {
            let mut interval = time::interval(Duration::from_secs(60));

            loop {
                interval.tick().await;
                debug!("Running connection cleanup task");

                connection_manager.cleanup_stale(timeout).await;

                let stats = connection_manager.get_stats().await;
                info!(
                    "Server stats - Active: {}, Total: {}, Sent: {}, Received: {}",
                    stats.active_connections,
                    stats.total_connections,
                    stats.total_packets_sent,
                    stats.total_packets_received
                );
            }
        });
    }

    /// Shutdown the server
    pub fn shutdown(&self) {
        info!("Shutting down server...");
        let _ = self.shutdown_tx.send(());
    }
}

/// Handle a single connection
async fn handle_connection(
    mut stream: TcpStream,
    peer_addr: std::net::SocketAddr,
    connection_manager: Arc<ConnectionManager>,
    config: Arc<Config>,
) -> Result<()> {
    info!("Handling connection from {}", peer_addr);

    // Create connection
    let connection = connection_manager.create_connection(peer_addr)?;
    let session_id = connection.session().id().clone();

    info!("Session {} created for {}", session_id, peer_addr);

    // Perform handshake
    match perform_handshake(&mut stream, &connection).await {
        Ok(_) => {
            info!("Handshake completed for session {}", session_id);
            connection.session().set_state(SessionState::Active).await;
        }
        Err(e) => {
            error!("Handshake failed for session {}: {}", session_id, e);
            connection_manager.remove_connection(&session_id);
            return Err(e);
        }
    }

    // Main data loop
    let result = handle_data_loop(&mut stream, &connection).await;

    // Cleanup
    info!("Connection closed for session {}: {:?}", session_id, result);
    connection_manager.remove_connection(&session_id);

    result
}

/// Perform handshake with client
async fn perform_handshake(
    stream: &mut TcpStream,
    connection: &Arc<crate::core::connection::Connection>,
) -> Result<()> {
    debug!("Starting handshake for session {}", connection.session().id());

    // Read ClientHello packet
    let client_hello_packet = read_packet(stream).await?;

    if client_hello_packet.header.packet_type != PacketType::HandshakeInit {
        return Err(LostLoveError::HandshakeFailed(
            "Expected HandshakeInit packet".to_string(),
        ));
    }

    // Parse ClientHello message
    let client_hello = HandshakeMessage::from_bytes(&client_hello_packet.payload)?;

    // Process ClientHello and generate ServerHello
    let server_hello = {
        let mut handshake = connection.handshake().write().await;
        handshake.process_client_hello(&client_hello)?
    };

    // Send ServerHello
    let server_hello_bytes = server_hello.to_bytes()?;
    let response_packet = Packet::new(PacketType::HandshakeResponse, server_hello_bytes);

    write_packet(stream, &response_packet).await?;

    debug!("Handshake completed for session {}", connection.session().id());

    Ok(())
}

/// Handle data loop
async fn handle_data_loop(
    stream: &mut TcpStream,
    connection: &Arc<crate::core::connection::Connection>,
) -> Result<()> {
    let mut buffer = BytesMut::with_capacity(4096);

    loop {
        // Read packet header
        let header_bytes = match read_exact(stream, HEADER_SIZE).await {
            Ok(bytes) => bytes,
            Err(e) => {
                if e.kind() == std::io::ErrorKind::UnexpectedEof {
                    debug!("Client disconnected");
                    return Ok(());
                }
                return Err(LostLoveError::from(e));
            }
        };

        // Parse packet
        buffer.clear();
        buffer.extend_from_slice(&header_bytes);

        // For now, just echo back (in Phase 1 we don't have routing yet)
        let packet = match Packet::deserialize(&buffer[..]) {
            Ok(p) => p,
            Err(e) => {
                warn!("Failed to parse packet: {}", e);
                connection.session().record_error().await;
                continue;
            }
        };

        connection.session().record_packet_received(packet.size()).await;
        connection.update_activity().await;

        debug!(
            "Received packet: type={:?}, stream={}, seq={}",
            packet.header.packet_type, packet.header.stream_id, packet.header.sequence_number
        );

        match packet.header.packet_type {
            PacketType::Data => {
                // For Phase 1: just acknowledge
                let ack = Packet::new(PacketType::Ack, Bytes::new());
                write_packet(stream, &ack).await?;
                connection.session().record_packet_sent(ack.size()).await;
            }
            PacketType::KeepAlive => {
                // Respond to keepalive
                let response = Packet::new(PacketType::KeepAlive, Bytes::new());
                write_packet(stream, &response).await?;
                connection.session().record_packet_sent(response.size()).await;
            }
            PacketType::Disconnect => {
                info!("Client requested disconnect");
                return Ok(());
            }
            _ => {
                debug!("Unhandled packet type: {:?}", packet.header.packet_type);
            }
        }
    }
}

/// Read exact number of bytes from stream
async fn read_exact(stream: &mut TcpStream, len: usize) -> std::io::Result<Vec<u8>> {
    let mut buf = vec![0u8; len];
    stream.read_exact(&mut buf).await?;
    Ok(buf)
}

/// Read a complete packet from stream
async fn read_packet(stream: &mut TcpStream) -> Result<Packet> {
    // Read header
    let header_bytes = read_exact(stream, HEADER_SIZE).await?;

    // Parse header to get payload length (for now we read remaining data)
    // In a real implementation, we'd include length in the header
    let mut buf = BytesMut::from(&header_bytes[..]);

    // For Phase 1, we assume small payloads that fit in one read
    // Read up to 4KB of payload
    let mut payload_buf = vec![0u8; 4096];
    let n = stream.read(&mut payload_buf).await?;

    if n > 0 {
        buf.extend_from_slice(&payload_buf[..n]);
    }

    Packet::deserialize(buf)
}

/// Write packet to stream
async fn write_packet(stream: &mut TcpStream, packet: &Packet) -> Result<()> {
    let data = packet.serialize();
    stream.write_all(&data).await?;
    stream.flush().await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    #[tokio::test]
    async fn test_server_creation() {
        let config = Config::default_for_testing();
        let server = Server::new(config).await.unwrap();

        assert_eq!(server.connection_manager.active_count(), 0);
    }
}
