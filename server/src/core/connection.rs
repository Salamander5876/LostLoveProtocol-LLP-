use dashmap::DashMap;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use crate::core::session::{Session, SessionId, SessionState};
use crate::error::{LostLoveError, Result};
use crate::protocol::{Handshake, HandshakeState};

/// Connection represents a single client connection
pub struct Connection {
    session: Arc<Session>,
    handshake: Arc<RwLock<Handshake>>,
    sequence_number: AtomicU64,
}

impl Connection {
    /// Create new connection
    pub fn new(peer_addr: SocketAddr) -> Self {
        Self {
            session: Arc::new(Session::new(peer_addr)),
            handshake: Arc::new(RwLock::new(Handshake::new_server())),
            sequence_number: AtomicU64::new(0),
        }
    }

    /// Get session
    pub fn session(&self) -> &Arc<Session> {
        &self.session
    }

    /// Get next sequence number
    pub fn next_sequence(&self) -> u64 {
        self.sequence_number.fetch_add(1, Ordering::SeqCst)
    }

    /// Get handshake
    pub fn handshake(&self) -> &Arc<RwLock<Handshake>> {
        &self.handshake
    }

    /// Check if handshake is completed
    pub async fn is_handshake_completed(&self) -> bool {
        self.handshake.read().await.is_completed()
    }

    /// Update activity
    pub async fn update_activity(&self) {
        self.session.update_activity().await;
    }
}

/// Connection Manager manages all active connections
pub struct ConnectionManager {
    connections: Arc<DashMap<SessionId, Arc<Connection>>>,
    max_connections: usize,
    active_count: AtomicUsize,
    total_connections: AtomicU64,
}

impl ConnectionManager {
    /// Create new connection manager
    pub fn new(max_connections: usize) -> Self {
        info!("Creating ConnectionManager with max {} connections", max_connections);

        Self {
            connections: Arc::new(DashMap::new()),
            max_connections,
            active_count: AtomicUsize::new(0),
            total_connections: AtomicU64::new(0),
        }
    }

    /// Create new connection
    pub fn create_connection(&self, peer_addr: SocketAddr) -> Result<Arc<Connection>> {
        let current = self.active_count.load(Ordering::Relaxed);

        if current >= self.max_connections {
            warn!(
                "Maximum connections reached: {}/{}",
                current, self.max_connections
            );
            return Err(LostLoveError::TooManyConnections);
        }

        let connection = Arc::new(Connection::new(peer_addr));
        let session_id = connection.session().id().clone();

        debug!("Creating new connection: {} from {}", session_id, peer_addr);

        self.connections.insert(session_id.clone(), connection.clone());
        self.active_count.fetch_add(1, Ordering::SeqCst);
        self.total_connections.fetch_add(1, Ordering::SeqCst);

        info!(
            "New connection established: {} (total: {})",
            session_id,
            self.active_count.load(Ordering::Relaxed)
        );

        Ok(connection)
    }

    /// Get connection by session ID
    pub fn get_connection(&self, session_id: &SessionId) -> Option<Arc<Connection>> {
        self.connections.get(session_id).map(|r| r.value().clone())
    }

    /// Remove connection
    pub fn remove_connection(&self, session_id: &SessionId) -> Option<Arc<Connection>> {
        debug!("Removing connection: {}", session_id);

        let result = self.connections.remove(session_id).map(|(_, conn)| conn);

        if result.is_some() {
            self.active_count.fetch_sub(1, Ordering::SeqCst);
            info!(
                "Connection removed: {} (remaining: {})",
                session_id,
                self.active_count.load(Ordering::Relaxed)
            );
        }

        result
    }

    /// Get active connections count
    pub fn active_count(&self) -> usize {
        self.active_count.load(Ordering::Relaxed)
    }

    /// Get total connections count (historical)
    pub fn total_count(&self) -> u64 {
        self.total_connections.load(Ordering::Relaxed)
    }

    /// Cleanup stale connections
    pub async fn cleanup_stale(&self, timeout: Duration) {
        let mut to_remove = Vec::new();

        for entry in self.connections.iter() {
            let session = entry.value().session();

            if session.should_timeout(timeout).await {
                warn!("Session {} timed out", entry.key());
                to_remove.push(entry.key().clone());
            }
        }

        for session_id in to_remove {
            self.remove_connection(&session_id);
        }
    }

    /// Get all session IDs
    pub fn get_all_sessions(&self) -> Vec<SessionId> {
        self.connections
            .iter()
            .map(|entry| entry.key().clone())
            .collect()
    }

    /// Get statistics
    pub async fn get_stats(&self) -> ConnectionManagerStats {
        let mut total_packets_sent = 0u64;
        let mut total_packets_received = 0u64;
        let mut total_bytes_sent = 0u64;
        let mut total_bytes_received = 0u64;
        let mut total_errors = 0u64;

        for entry in self.connections.iter() {
            let stats = entry.value().session().stats().await;
            total_packets_sent += stats.packets_sent;
            total_packets_received += stats.packets_received;
            total_bytes_sent += stats.bytes_sent;
            total_bytes_received += stats.bytes_received;
            total_errors += stats.errors;
        }

        ConnectionManagerStats {
            active_connections: self.active_count(),
            total_connections: self.total_count(),
            total_packets_sent,
            total_packets_received,
            total_bytes_sent,
            total_bytes_received,
            total_errors,
        }
    }
}

/// Connection manager statistics
#[derive(Debug, Clone)]
pub struct ConnectionManagerStats {
    pub active_connections: usize,
    pub total_connections: u64,
    pub total_packets_sent: u64,
    pub total_packets_received: u64,
    pub total_bytes_sent: u64,
    pub total_bytes_received: u64,
    pub total_errors: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr};

    #[tokio::test]
    async fn test_connection_creation() {
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
        let connection = Connection::new(addr);

        assert_eq!(connection.session().peer_address(), addr);
        assert!(!connection.is_handshake_completed().await);
    }

    #[tokio::test]
    async fn test_sequence_number() {
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
        let connection = Connection::new(addr);

        assert_eq!(connection.next_sequence(), 0);
        assert_eq!(connection.next_sequence(), 1);
        assert_eq!(connection.next_sequence(), 2);
    }

    #[tokio::test]
    async fn test_connection_manager() {
        let manager = ConnectionManager::new(10);
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);

        let conn = manager.create_connection(addr).unwrap();
        let session_id = conn.session().id().clone();

        assert_eq!(manager.active_count(), 1);
        assert!(manager.get_connection(&session_id).is_some());

        manager.remove_connection(&session_id);
        assert_eq!(manager.active_count(), 0);
        assert!(manager.get_connection(&session_id).is_none());
    }

    #[tokio::test]
    async fn test_max_connections() {
        let manager = ConnectionManager::new(2);
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);

        // Create 2 connections (max)
        let conn1 = manager.create_connection(addr).unwrap();
        let conn2 = manager.create_connection(addr).unwrap();

        // Try to create 3rd connection (should fail)
        let result = manager.create_connection(addr);
        assert!(result.is_err());
        assert_eq!(manager.active_count(), 2);
    }

    #[tokio::test]
    async fn test_connection_stats() {
        let manager = ConnectionManager::new(10);
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);

        let conn = manager.create_connection(addr).unwrap();

        // Record some activity
        conn.session().record_packet_sent(100).await;
        conn.session().record_packet_received(200).await;

        let stats = manager.get_stats().await;
        assert_eq!(stats.active_connections, 1);
        assert_eq!(stats.total_packets_sent, 1);
        assert_eq!(stats.total_bytes_sent, 100);
        assert_eq!(stats.total_bytes_received, 200);
    }
}
