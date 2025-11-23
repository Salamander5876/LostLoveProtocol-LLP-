use std::fmt;
use std::sync::Arc;
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::Mutex;

/// Session identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SessionId(String);

impl SessionId {
    /// Create new session ID
    pub fn new() -> Self {
        SessionId(uuid::Uuid::new_v4().to_string())
    }

    /// Create from string
    pub fn from_string(s: String) -> Self {
        SessionId(s)
    }

    /// Get string representation
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for SessionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Default for SessionId {
    fn default() -> Self {
        Self::new()
    }
}

/// Session state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionState {
    Handshaking,
    Active,
    Disconnecting,
    Closed,
}

/// Session statistics
#[derive(Debug, Clone, Default)]
pub struct SessionStats {
    pub packets_sent: u64,
    pub packets_received: u64,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub errors: u64,
}

/// Session data
pub struct Session {
    id: SessionId,
    state: Arc<Mutex<SessionState>>,
    stats: Arc<Mutex<SessionStats>>,
    created_at: SystemTime,
    last_activity: Arc<Mutex<Instant>>,
    peer_address: std::net::SocketAddr,
}

impl Session {
    /// Create new session
    pub fn new(peer_address: std::net::SocketAddr) -> Self {
        Self {
            id: SessionId::new(),
            state: Arc::new(Mutex::new(SessionState::Handshaking)),
            stats: Arc::new(Mutex::new(SessionStats::default())),
            created_at: SystemTime::now(),
            last_activity: Arc::new(Mutex::new(Instant::now())),
            peer_address,
        }
    }

    /// Get session ID
    pub fn id(&self) -> &SessionId {
        &self.id
    }

    /// Get peer address
    pub fn peer_address(&self) -> std::net::SocketAddr {
        self.peer_address
    }

    /// Get current state
    pub async fn state(&self) -> SessionState {
        *self.state.lock().await
    }

    /// Set state
    pub async fn set_state(&self, new_state: SessionState) {
        *self.state.lock().await = new_state;
    }

    /// Update last activity timestamp
    pub async fn update_activity(&self) {
        *self.last_activity.lock().await = Instant::now();
    }

    /// Get time since last activity
    pub async fn time_since_activity(&self) -> std::time::Duration {
        self.last_activity.lock().await.elapsed()
    }

    /// Get session uptime
    pub fn uptime(&self) -> std::time::Duration {
        SystemTime::now()
            .duration_since(self.created_at)
            .unwrap_or_default()
    }

    /// Update statistics - packet sent
    pub async fn record_packet_sent(&self, size: usize) {
        let mut stats = self.stats.lock().await;
        stats.packets_sent += 1;
        stats.bytes_sent += size as u64;
    }

    /// Update statistics - packet received
    pub async fn record_packet_received(&self, size: usize) {
        let mut stats = self.stats.lock().await;
        stats.packets_received += 1;
        stats.bytes_received += size as u64;
    }

    /// Update statistics - error
    pub async fn record_error(&self) {
        let mut stats = self.stats.lock().await;
        stats.errors += 1;
    }

    /// Get statistics snapshot
    pub async fn stats(&self) -> SessionStats {
        self.stats.lock().await.clone()
    }

    /// Check if session is active
    pub async fn is_active(&self) -> bool {
        *self.state.lock().await == SessionState::Active
    }

    /// Check if session should timeout
    pub async fn should_timeout(&self, timeout_duration: std::time::Duration) -> bool {
        self.time_since_activity().await > timeout_duration
    }
}

impl fmt::Debug for Session {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Session")
            .field("id", &self.id)
            .field("peer_address", &self.peer_address)
            .field("created_at", &self.created_at)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};

    #[tokio::test]
    async fn test_session_creation() {
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
        let session = Session::new(addr);

        assert_eq!(session.state().await, SessionState::Handshaking);
        assert_eq!(session.peer_address(), addr);
    }

    #[tokio::test]
    async fn test_session_state_transition() {
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
        let session = Session::new(addr);

        session.set_state(SessionState::Active).await;
        assert_eq!(session.state().await, SessionState::Active);
        assert!(session.is_active().await);
    }

    #[tokio::test]
    async fn test_session_stats() {
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
        let session = Session::new(addr);

        session.record_packet_sent(100).await;
        session.record_packet_received(200).await;

        let stats = session.stats().await;
        assert_eq!(stats.packets_sent, 1);
        assert_eq!(stats.packets_received, 1);
        assert_eq!(stats.bytes_sent, 100);
        assert_eq!(stats.bytes_received, 200);
    }

    #[tokio::test]
    async fn test_session_activity() {
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
        let session = Session::new(addr);

        session.update_activity().await;

        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        let duration = session.time_since_activity().await;
        assert!(duration >= std::time::Duration::from_millis(100));
    }
}
