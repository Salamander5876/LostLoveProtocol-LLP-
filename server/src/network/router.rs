use std::sync::Arc;
use tracing::{debug, warn};

use crate::core::connection::ConnectionManager;
use crate::core::session::SessionId;
use crate::error::Result;

/// Packet router for forwarding packets between TUN and connections
pub struct PacketRouter {
    connection_manager: Arc<ConnectionManager>,
}

impl PacketRouter {
    /// Create new packet router
    pub fn new(connection_manager: Arc<ConnectionManager>) -> Self {
        Self { connection_manager }
    }

    /// Route packet from TUN interface to client
    pub async fn route_from_tun(&self, packet: &[u8], session_id: &SessionId) -> Result<()> {
        debug!(
            "Routing {} bytes from TUN to session {}",
            packet.len(),
            session_id
        );

        // Get connection
        if let Some(connection) = self.connection_manager.get_connection(session_id) {
            // Check if connection is active
            if connection.session().is_active().await {
                // In Phase 1, we just log. Actual sending will be implemented later
                debug!("Would send packet to session {}", session_id);
                connection.session().record_packet_sent(packet.len()).await;
                Ok(())
            } else {
                warn!("Session {} is not active", session_id);
                Err(crate::error::LostLoveError::Connection(
                    "Session not active".to_string(),
                ))
            }
        } else {
            warn!("Session {} not found", session_id);
            Err(crate::error::LostLoveError::SessionNotFound(
                session_id.to_string(),
            ))
        }
    }

    /// Route packet from client to TUN interface
    pub async fn route_to_tun(&self, packet: &[u8], session_id: &SessionId) -> Result<Vec<u8>> {
        debug!(
            "Routing {} bytes from session {} to TUN",
            packet.len(),
            session_id
        );

        // Get connection and update stats
        if let Some(connection) = self.connection_manager.get_connection(session_id) {
            connection.session().record_packet_received(packet.len()).await;
            connection.update_activity().await;

            // In Phase 1, just return the packet as-is
            // Later this will extract the inner IP packet
            Ok(packet.to_vec())
        } else {
            warn!("Session {} not found", session_id);
            Err(crate::error::LostLoveError::SessionNotFound(
                session_id.to_string(),
            ))
        }
    }

    /// Route packet between two sessions (peer-to-peer)
    pub async fn route_p2p(
        &self,
        packet: &[u8],
        from_session: &SessionId,
        to_session: &SessionId,
    ) -> Result<()> {
        debug!(
            "Routing {} bytes from {} to {}",
            packet.len(),
            from_session,
            to_session
        );

        // Get both connections
        let from_conn = self
            .connection_manager
            .get_connection(from_session)
            .ok_or_else(|| {
                crate::error::LostLoveError::SessionNotFound(from_session.to_string())
            })?;

        let to_conn = self
            .connection_manager
            .get_connection(to_session)
            .ok_or_else(|| {
                crate::error::LostLoveError::SessionNotFound(to_session.to_string())
            })?;

        // Update stats
        from_conn.session().record_packet_sent(packet.len()).await;
        to_conn.session().record_packet_received(packet.len()).await;

        // In Phase 1, just log
        debug!("Would forward packet from {} to {}", from_session, to_session);

        Ok(())
    }

    /// Get active routes count
    pub fn active_routes(&self) -> usize {
        self.connection_manager.active_count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};

    #[tokio::test]
    async fn test_router_creation() {
        let manager = Arc::new(ConnectionManager::new(10));
        let router = PacketRouter::new(manager);

        assert_eq!(router.active_routes(), 0);
    }

    #[tokio::test]
    async fn test_route_to_nonexistent_session() {
        let manager = Arc::new(ConnectionManager::new(10));
        let router = PacketRouter::new(manager);

        let session_id = SessionId::new();
        let packet = vec![0u8; 100];

        let result = router.route_to_tun(&packet, &session_id).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_route_with_active_session() {
        let manager = Arc::new(ConnectionManager::new(10));
        let router = PacketRouter::new(manager.clone());

        // Create connection
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
        let conn = manager.create_connection(addr).unwrap();
        let session_id = conn.session().id().clone();

        // Set session as active
        conn.session()
            .set_state(crate::core::session::SessionState::Active)
            .await;

        // Route packet
        let packet = vec![0u8; 100];
        let result = router.route_from_tun(&packet, &session_id).await;
        assert!(result.is_ok());

        // Check stats
        let stats = conn.session().stats().await;
        assert_eq!(stats.packets_sent, 1);
        assert_eq!(stats.bytes_sent, 100);
    }
}
