use bytes::{Buf, BufMut, Bytes, BytesMut};
use std::time::{SystemTime, UNIX_EPOCH};
use crate::error::{LostLoveError, Result};

/// Protocol identifier
pub const PROTOCOL_ID: u16 = 0x4C4C; // "LL" in hex (LostLove)

/// Header size in bytes
pub const HEADER_SIZE: usize = 24;

/// Packet types
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PacketType {
    Data = 0x01,
    Ack = 0x02,
    HandshakeInit = 0x03,
    HandshakeResponse = 0x04,
    KeepAlive = 0x05,
    Disconnect = 0x06,
}

impl PacketType {
    pub fn from_u8(value: u8) -> Result<Self> {
        match value {
            0x01 => Ok(PacketType::Data),
            0x02 => Ok(PacketType::Ack),
            0x03 => Ok(PacketType::HandshakeInit),
            0x04 => Ok(PacketType::HandshakeResponse),
            0x05 => Ok(PacketType::KeepAlive),
            0x06 => Ok(PacketType::Disconnect),
            _ => Err(LostLoveError::InvalidPacketType(value)),
        }
    }
}

/// Packet header structure
#[derive(Debug, Clone)]
pub struct PacketHeader {
    pub protocol_id: u16,
    pub packet_type: PacketType,
    pub stream_id: u16,
    pub sequence_number: u64,
    pub timestamp: u64,
    pub flags: u8,
    pub checksum: u16,
}

impl PacketHeader {
    /// Create a new packet header
    pub fn new(packet_type: PacketType) -> Self {
        Self {
            protocol_id: PROTOCOL_ID,
            packet_type,
            stream_id: 0,
            sequence_number: 0,
            timestamp: current_timestamp(),
            flags: 0,
            checksum: 0,
        }
    }

    /// Serialize header to bytes
    pub fn serialize(&self, buf: &mut BytesMut) {
        buf.put_u16(self.protocol_id);
        buf.put_u8(self.packet_type as u8);
        buf.put_u16(self.stream_id);
        buf.put_u64(self.sequence_number);
        buf.put_u64(self.timestamp);
        buf.put_u8(self.flags);
        buf.put_u16(self.checksum);
    }

    /// Deserialize header from bytes
    pub fn deserialize(buf: &mut impl Buf) -> Result<Self> {
        if buf.remaining() < HEADER_SIZE {
            return Err(LostLoveError::InsufficientData {
                expected: HEADER_SIZE,
                actual: buf.remaining(),
            });
        }

        let protocol_id = buf.get_u16();
        if protocol_id != PROTOCOL_ID {
            return Err(LostLoveError::InvalidProtocolId(protocol_id));
        }

        let packet_type = PacketType::from_u8(buf.get_u8())?;
        let stream_id = buf.get_u16();
        let sequence_number = buf.get_u64();
        let timestamp = buf.get_u64();
        let flags = buf.get_u8();
        let checksum = buf.get_u16();

        Ok(Self {
            protocol_id,
            packet_type,
            stream_id,
            sequence_number,
            timestamp,
            flags,
            checksum,
        })
    }

    /// Calculate CRC16 checksum
    pub fn calculate_checksum(&self, payload: &[u8]) -> u16 {
        let mut crc = 0xFFFFu16;

        // Hash header fields
        let mut data = Vec::new();
        data.extend_from_slice(&self.protocol_id.to_be_bytes());
        data.push(self.packet_type as u8);
        data.extend_from_slice(&self.stream_id.to_be_bytes());
        data.extend_from_slice(&self.sequence_number.to_be_bytes());
        data.extend_from_slice(&self.timestamp.to_be_bytes());
        data.push(self.flags);

        // CRC16-CCITT algorithm
        for byte in data.iter().chain(payload.iter()) {
            crc ^= (*byte as u16) << 8;
            for _ in 0..8 {
                if (crc & 0x8000) != 0 {
                    crc = (crc << 1) ^ 0x1021;
                } else {
                    crc <<= 1;
                }
            }
        }

        crc
    }

    /// Verify checksum
    pub fn verify_checksum(&self, payload: &[u8]) -> bool {
        let calculated = self.calculate_checksum(payload);
        calculated == self.checksum
    }
}

/// Complete packet structure
#[derive(Debug, Clone)]
pub struct Packet {
    pub header: PacketHeader,
    pub payload: Bytes,
}

impl Packet {
    /// Create a new packet
    pub fn new(packet_type: PacketType, payload: Bytes) -> Self {
        let mut header = PacketHeader::new(packet_type);
        header.checksum = header.calculate_checksum(&payload);

        Self { header, payload }
    }

    /// Create a packet with specific stream ID and sequence number
    pub fn new_with_metadata(
        packet_type: PacketType,
        stream_id: u16,
        sequence_number: u64,
        payload: Bytes,
    ) -> Self {
        let mut header = PacketHeader::new(packet_type);
        header.stream_id = stream_id;
        header.sequence_number = sequence_number;
        header.checksum = header.calculate_checksum(&payload);

        Self { header, payload }
    }

    /// Serialize packet to bytes
    pub fn serialize(&self) -> BytesMut {
        let mut buf = BytesMut::with_capacity(HEADER_SIZE + self.payload.len());
        self.header.serialize(&mut buf);
        buf.put_slice(&self.payload);
        buf
    }

    /// Deserialize packet from bytes
    pub fn deserialize(mut buf: impl Buf) -> Result<Self> {
        let header = PacketHeader::deserialize(&mut buf)?;
        let payload = buf.copy_to_bytes(buf.remaining());

        let packet = Self { header, payload };

        // Verify checksum
        if !packet.header.verify_checksum(&packet.payload) {
            return Err(LostLoveError::ChecksumMismatch {
                expected: packet.header.checksum,
                actual: packet.header.calculate_checksum(&packet.payload),
            });
        }

        Ok(packet)
    }

    /// Get packet total size
    pub fn size(&self) -> usize {
        HEADER_SIZE + self.payload.len()
    }

    /// Check if packet is a control packet
    pub fn is_control(&self) -> bool {
        matches!(
            self.header.packet_type,
            PacketType::HandshakeInit
                | PacketType::HandshakeResponse
                | PacketType::KeepAlive
                | PacketType::Disconnect
        )
    }
}

/// Get current timestamp in milliseconds
pub fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_packet_type_conversion() {
        assert_eq!(PacketType::from_u8(0x01).unwrap(), PacketType::Data);
        assert_eq!(PacketType::from_u8(0x05).unwrap(), PacketType::KeepAlive);
        assert!(PacketType::from_u8(0xFF).is_err());
    }

    #[test]
    fn test_packet_serialization() {
        let payload = Bytes::from("Hello, LostLove!");
        let packet = Packet::new(PacketType::Data, payload.clone());

        let serialized = packet.serialize();
        let deserialized = Packet::deserialize(serialized).unwrap();

        assert_eq!(deserialized.header.packet_type, PacketType::Data);
        assert_eq!(deserialized.payload, payload);
    }

    #[test]
    fn test_checksum_verification() {
        let payload = Bytes::from("test data");
        let packet = Packet::new(PacketType::Data, payload);

        assert!(packet.header.verify_checksum(&packet.payload));
    }

    #[test]
    fn test_invalid_checksum() {
        let payload = Bytes::from("test data");
        let mut packet = Packet::new(PacketType::Data, payload);

        // Corrupt checksum
        packet.header.checksum = 0xDEAD;

        let serialized = packet.serialize();
        let result = Packet::deserialize(serialized);

        assert!(result.is_err());
    }

    #[test]
    fn test_header_size() {
        let header = PacketHeader::new(PacketType::Data);
        let mut buf = BytesMut::new();
        header.serialize(&mut buf);

        assert_eq!(buf.len(), HEADER_SIZE);
    }
}
