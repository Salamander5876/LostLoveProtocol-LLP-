use std::fmt;

/// Stream identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StreamId(pub u16);

impl StreamId {
    /// Control stream (reserved)
    pub const CONTROL: StreamId = StreamId(0);

    /// Create new stream ID
    pub fn new(id: u16) -> Self {
        StreamId(id)
    }

    /// Check if this is a control stream
    pub fn is_control(&self) -> bool {
        self.0 == 0
    }

    /// Get the raw ID value
    pub fn value(&self) -> u16 {
        self.0
    }
}

impl fmt::Display for StreamId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Stream({})", self.0)
    }
}

impl From<u16> for StreamId {
    fn from(id: u16) -> Self {
        StreamId(id)
    }
}

impl From<StreamId> for u16 {
    fn from(id: StreamId) -> Self {
        id.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stream_id() {
        let control = StreamId::CONTROL;
        assert!(control.is_control());
        assert_eq!(control.value(), 0);

        let stream = StreamId::new(42);
        assert!(!stream.is_control());
        assert_eq!(stream.value(), 42);
    }

    #[test]
    fn test_stream_id_conversion() {
        let id: StreamId = 100u16.into();
        assert_eq!(id.value(), 100);

        let raw: u16 = id.into();
        assert_eq!(raw, 100);
    }
}
