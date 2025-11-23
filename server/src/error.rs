use thiserror::Error;

#[derive(Error, Debug)]
pub enum LostLoveError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid protocol ID: {0}")]
    InvalidProtocolId(u16),

    #[error("Invalid packet type: {0}")]
    InvalidPacketType(u8),

    #[error("Insufficient data: expected {expected}, got {actual}")]
    InsufficientData { expected: usize, actual: usize },

    #[error("Checksum mismatch: expected {expected:04x}, got {actual:04x}")]
    ChecksumMismatch { expected: u16, actual: u16 },

    #[error("Invalid sequence number: {0}")]
    InvalidSequence(u64),

    #[error("Timestamp too old: {0}")]
    TimestampTooOld(u64),

    #[error("Connection error: {0}")]
    Connection(String),

    #[error("Too many connections")]
    TooManyConnections,

    #[error("Session not found: {0}")]
    SessionNotFound(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Handshake failed: {0}")]
    HandshakeFailed(String),
}

pub type Result<T> = std::result::Result<T, LostLoveError>;
