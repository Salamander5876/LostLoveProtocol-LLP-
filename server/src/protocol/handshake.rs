use bytes::Bytes;
use serde::{Deserialize, Serialize};
use crate::error::{LostLoveError, Result};

/// Handshake state machine
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HandshakeState {
    Init,
    ClientHelloSent,
    ServerHelloReceived,
    Completed,
    Failed,
}

/// Handshake message types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HandshakeMessage {
    ClientHello {
        client_random: [u8; 32],
        protocol_version: u8,
    },
    ServerHello {
        server_random: [u8; 32],
        session_id: String,
    },
    ClientFinish {
        verification_data: Vec<u8>,
    },
    ServerFinish {
        verification_data: Vec<u8>,
    },
}

impl HandshakeMessage {
    /// Serialize handshake message to bytes
    pub fn to_bytes(&self) -> Result<Bytes> {
        let json = serde_json::to_vec(self)
            .map_err(|e| LostLoveError::HandshakeFailed(format!("Serialization error: {}", e)))?;
        Ok(Bytes::from(json))
    }

    /// Deserialize handshake message from bytes
    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        serde_json::from_slice(data)
            .map_err(|e| LostLoveError::HandshakeFailed(format!("Deserialization error: {}", e)))
    }
}

/// Handshake handler
pub struct Handshake {
    state: HandshakeState,
    client_random: Option<[u8; 32]>,
    server_random: Option<[u8; 32]>,
    session_id: Option<String>,
}

impl Handshake {
    /// Create new handshake (server side)
    pub fn new_server() -> Self {
        Self {
            state: HandshakeState::Init,
            client_random: None,
            server_random: None,
            session_id: None,
        }
    }

    /// Create new handshake (client side)
    pub fn new_client() -> Self {
        Self {
            state: HandshakeState::Init,
            client_random: Some(generate_random()),
            server_random: None,
            session_id: None,
        }
    }

    /// Get current state
    pub fn state(&self) -> HandshakeState {
        self.state
    }

    /// Check if handshake is completed
    pub fn is_completed(&self) -> bool {
        self.state == HandshakeState::Completed
    }

    /// Generate ClientHello message
    pub fn generate_client_hello(&mut self) -> Result<HandshakeMessage> {
        if self.state != HandshakeState::Init {
            return Err(LostLoveError::HandshakeFailed(
                "Invalid state for ClientHello".to_string(),
            ));
        }

        let client_random = self.client_random.unwrap_or_else(generate_random);
        self.client_random = Some(client_random);
        self.state = HandshakeState::ClientHelloSent;

        Ok(HandshakeMessage::ClientHello {
            client_random,
            protocol_version: 1,
        })
    }

    /// Process ClientHello message (server side)
    pub fn process_client_hello(&mut self, msg: &HandshakeMessage) -> Result<HandshakeMessage> {
        if self.state != HandshakeState::Init {
            return Err(LostLoveError::HandshakeFailed(
                "Invalid state for processing ClientHello".to_string(),
            ));
        }

        if let HandshakeMessage::ClientHello {
            client_random,
            protocol_version,
        } = msg
        {
            if *protocol_version != 1 {
                return Err(LostLoveError::HandshakeFailed(format!(
                    "Unsupported protocol version: {}",
                    protocol_version
                )));
            }

            self.client_random = Some(*client_random);

            let server_random = generate_random();
            self.server_random = Some(server_random);

            let session_id = uuid::Uuid::new_v4().to_string();
            self.session_id = Some(session_id.clone());

            self.state = HandshakeState::ServerHelloReceived;

            Ok(HandshakeMessage::ServerHello {
                server_random,
                session_id,
            })
        } else {
            Err(LostLoveError::HandshakeFailed(
                "Expected ClientHello message".to_string(),
            ))
        }
    }

    /// Process ServerHello message (client side)
    pub fn process_server_hello(&mut self, msg: &HandshakeMessage) -> Result<()> {
        if self.state != HandshakeState::ClientHelloSent {
            return Err(LostLoveError::HandshakeFailed(
                "Invalid state for processing ServerHello".to_string(),
            ));
        }

        if let HandshakeMessage::ServerHello {
            server_random,
            session_id,
        } = msg
        {
            self.server_random = Some(*server_random);
            self.session_id = Some(session_id.clone());
            self.state = HandshakeState::Completed;

            Ok(())
        } else {
            Err(LostLoveError::HandshakeFailed(
                "Expected ServerHello message".to_string(),
            ))
        }
    }

    /// Get session ID
    pub fn session_id(&self) -> Option<&str> {
        self.session_id.as_deref()
    }

    /// Get client random
    pub fn client_random(&self) -> Option<[u8; 32]> {
        self.client_random
    }

    /// Get server random
    pub fn server_random(&self) -> Option<[u8; 32]> {
        self.server_random
    }
}

/// Generate random bytes
fn generate_random() -> [u8; 32] {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let mut bytes = [0u8; 32];
    rng.fill(&mut bytes);
    bytes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handshake_flow() {
        // Client side
        let mut client_handshake = Handshake::new_client();
        let client_hello = client_handshake.generate_client_hello().unwrap();

        // Server side
        let mut server_handshake = Handshake::new_server();
        let server_hello = server_handshake.process_client_hello(&client_hello).unwrap();

        // Client processes server hello
        client_handshake.process_server_hello(&server_hello).unwrap();

        assert!(client_handshake.is_completed());
        assert_eq!(
            server_handshake.state(),
            HandshakeState::ServerHelloReceived
        );
    }

    #[test]
    fn test_handshake_serialization() {
        let msg = HandshakeMessage::ClientHello {
            client_random: [0u8; 32],
            protocol_version: 1,
        };

        let bytes = msg.to_bytes().unwrap();
        let deserialized = HandshakeMessage::from_bytes(&bytes).unwrap();

        match deserialized {
            HandshakeMessage::ClientHello { protocol_version, .. } => {
                assert_eq!(protocol_version, 1);
            }
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_invalid_state_transition() {
        let mut handshake = Handshake::new_server();

        // Try to generate client hello from server side
        let result = handshake.generate_client_hello();
        assert!(result.is_err());
    }
}
