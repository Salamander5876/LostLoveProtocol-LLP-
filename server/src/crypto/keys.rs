use crate::crypto::kdf::{derive_session_keys, SessionKeys as DerivedSessionKeys};
use crate::crypto::HSEEncryptor;
use crate::error::Result;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use zeroize::Zeroizing;

pub use crate::crypto::kdf::SessionKeys;

/// Key rotation interval (30 minutes)
const KEY_ROTATION_INTERVAL: Duration = Duration::from_secs(30 * 60);

/// Manages cryptographic keys for a session with automatic rotation
pub struct KeyManager {
    /// Current session keys
    current_keys: Arc<RwLock<SessionKeys>>,
    /// Previous session keys (for graceful key rotation)
    previous_keys: Arc<RwLock<Option<SessionKeys>>>,
    /// Time when keys were last rotated
    last_rotation: Arc<RwLock<Instant>>,
    /// Shared secret for key derivation
    shared_secret: Zeroizing<Vec<u8>>,
    /// Client random value
    client_random: [u8; 32],
    /// Server random value
    server_random: [u8; 32],
    /// Enable automatic key rotation
    auto_rotation: bool,
}

impl KeyManager {
    /// Create a new key manager
    pub fn new(
        shared_secret: Vec<u8>,
        client_random: [u8; 32],
        server_random: [u8; 32],
        auto_rotation: bool,
    ) -> Result<Self> {
        let keys = derive_session_keys(&shared_secret, &client_random, &server_random)?;

        Ok(Self {
            current_keys: Arc::new(RwLock::new(keys)),
            previous_keys: Arc::new(RwLock::new(None)),
            last_rotation: Arc::new(RwLock::new(Instant::now())),
            shared_secret: Zeroizing::new(shared_secret),
            client_random,
            server_random,
            auto_rotation,
        })
    }

    /// Get current session keys
    pub async fn get_keys(&self) -> SessionKeys {
        let keys = self.current_keys.read().await;
        keys.clone()
    }

    /// Get current HSE encryptor
    pub async fn get_hse_encryptor(&self) -> HSEEncryptor {
        let keys = self.current_keys.read().await;
        HSEEncryptor::new(&keys.chacha_key, &keys.aes_key)
    }

    /// Check if keys need rotation and rotate if necessary
    pub async fn check_rotation(&self) -> Result<bool> {
        if !self.auto_rotation {
            return Ok(false);
        }

        let last_rotation = *self.last_rotation.read().await;
        let elapsed = last_rotation.elapsed();

        if elapsed >= KEY_ROTATION_INTERVAL {
            self.rotate_keys().await?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Force key rotation
    pub async fn rotate_keys(&self) -> Result<()> {
        // Derive new keys with updated info string
        let rotation_count = self.get_rotation_count().await;
        let info = format!("LLP-v1-rotation-{}", rotation_count);

        let new_keys = crate::crypto::kdf::derive_keys(
            &self.shared_secret,
            &[],
            info.as_bytes(),
            64,
        )?;

        // Derive ChaCha and AES keys from the rotated master secret
        let chacha_key = crate::crypto::kdf::derive_keys(
            &new_keys,
            &[],
            b"LLP-chacha20-key",
            32,
        )?;

        let aes_key = crate::crypto::kdf::derive_keys(
            &new_keys,
            &[],
            b"LLP-aes-key",
            32,
        )?;

        let chacha_key_array: [u8; 32] = chacha_key[..]
            .try_into()
            .map_err(|_| crate::error::LostLoveError::Connection("Invalid key length".to_string()))?;

        let aes_key_array: [u8; 32] = aes_key[..]
            .try_into()
            .map_err(|_| crate::error::LostLoveError::Connection("Invalid key length".to_string()))?;

        let master_secret_array: [u8; 64] = new_keys[..]
            .try_into()
            .map_err(|_| crate::error::LostLoveError::Connection("Invalid master secret length".to_string()))?;

        let rotated_keys = SessionKeys {
            chacha_key: Zeroizing::new(chacha_key_array),
            aes_key: Zeroizing::new(aes_key_array),
            master_secret: Zeroizing::new(master_secret_array),
        };

        // Store current keys as previous
        let current = self.current_keys.read().await.clone();
        *self.previous_keys.write().await = Some(current);

        // Update current keys
        *self.current_keys.write().await = rotated_keys;

        // Update rotation time
        *self.last_rotation.write().await = Instant::now();

        Ok(())
    }

    /// Get previous keys (for decrypting data encrypted with old keys during rotation)
    pub async fn get_previous_keys(&self) -> Option<SessionKeys> {
        self.previous_keys.read().await.clone()
    }

    /// Try to decrypt with current or previous keys
    pub async fn decrypt_with_fallback(
        &self,
        ciphertext: &[u8],
        nonce: &[u8; 12],
    ) -> Result<Vec<u8>> {
        // Try current keys first
        let current_hse = self.get_hse_encryptor().await;
        if let Ok(plaintext) = current_hse.decrypt(ciphertext, nonce) {
            return Ok(plaintext);
        }

        // Try previous keys if available
        if let Some(prev_keys) = self.get_previous_keys().await {
            let prev_hse = HSEEncryptor::new(&prev_keys.chacha_key, &prev_keys.aes_key);
            if let Ok(plaintext) = prev_hse.decrypt(ciphertext, nonce) {
                return Ok(plaintext);
            }
        }

        Err(crate::error::LostLoveError::Crypto(
            "Decryption failed with both current and previous keys".to_string(),
        ))
    }

    /// Get time until next key rotation
    pub async fn time_until_rotation(&self) -> Duration {
        if !self.auto_rotation {
            return Duration::from_secs(0);
        }

        let last_rotation = *self.last_rotation.read().await;
        let elapsed = last_rotation.elapsed();

        KEY_ROTATION_INTERVAL.saturating_sub(elapsed)
    }

    /// Get number of key rotations performed
    async fn get_rotation_count(&self) -> u64 {
        let last_rotation = *self.last_rotation.read().await;
        let total_time = last_rotation.elapsed();
        (total_time.as_secs() / KEY_ROTATION_INTERVAL.as_secs()) + 1
    }

    /// Clear all keys (called on disconnect)
    pub async fn clear_keys(&self) {
        *self.current_keys.write().await = SessionKeys::from_raw([0u8; 32], [0u8; 32]);
        *self.previous_keys.write().await = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_key_manager() -> KeyManager {
        let shared_secret = vec![1u8; 32];
        let client_random = [2u8; 32];
        let server_random = [3u8; 32];

        KeyManager::new(shared_secret, client_random, server_random, false).unwrap()
    }

    #[tokio::test]
    async fn test_key_manager_creation() {
        let km = create_test_key_manager();
        let keys = km.get_keys().await;

        assert_eq!(keys.chacha_key.len(), 32);
        assert_eq!(keys.aes_key.len(), 32);
        assert_eq!(keys.master_secret.len(), 64);
    }

    #[tokio::test]
    async fn test_get_hse_encryptor() {
        let km = create_test_key_manager();
        let hse = km.get_hse_encryptor().await;

        let plaintext = b"Test message";
        let nonce = [0u8; 12];

        let ciphertext = hse.encrypt(plaintext, &nonce).unwrap();
        let decrypted = hse.decrypt(&ciphertext, &nonce).unwrap();

        assert_eq!(decrypted, plaintext);
    }

    #[tokio::test]
    async fn test_key_rotation() {
        let shared_secret = vec![1u8; 32];
        let client_random = [2u8; 32];
        let server_random = [3u8; 32];

        let km = KeyManager::new(shared_secret, client_random, server_random, true).unwrap();

        // Get initial keys
        let keys_before = km.get_keys().await;

        // Rotate keys
        km.rotate_keys().await.unwrap();

        // Get new keys
        let keys_after = km.get_keys().await;

        // Keys should be different
        assert_ne!(&*keys_before.chacha_key, &*keys_after.chacha_key);
        assert_ne!(&*keys_before.aes_key, &*keys_after.aes_key);
    }

    #[tokio::test]
    async fn test_previous_keys_stored() {
        let km = create_test_key_manager();

        // Initially no previous keys
        assert!(km.get_previous_keys().await.is_none());

        // Get current keys
        let keys_before = km.get_keys().await;

        // Rotate
        km.rotate_keys().await.unwrap();

        // Previous keys should be stored
        let prev_keys = km.get_previous_keys().await.unwrap();
        assert_eq!(&*prev_keys.chacha_key, &*keys_before.chacha_key);
    }

    #[tokio::test]
    async fn test_decrypt_with_fallback() {
        let km = create_test_key_manager();

        // Encrypt with current keys
        let hse_before = km.get_hse_encryptor().await;
        let plaintext = b"Secret data";
        let nonce = [0u8; 12];
        let ciphertext = hse_before.encrypt(plaintext, &nonce).unwrap();

        // Rotate keys
        km.rotate_keys().await.unwrap();

        // Should still be able to decrypt with fallback
        let decrypted = km.decrypt_with_fallback(&ciphertext, &nonce).await.unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[tokio::test]
    async fn test_auto_rotation_disabled() {
        let shared_secret = vec![1u8; 32];
        let client_random = [2u8; 32];
        let server_random = [3u8; 32];

        let km = KeyManager::new(shared_secret, client_random, server_random, false).unwrap();

        // Check rotation should return false when disabled
        let rotated = km.check_rotation().await.unwrap();
        assert!(!rotated);
    }

    #[tokio::test]
    async fn test_time_until_rotation() {
        let shared_secret = vec![1u8; 32];
        let client_random = [2u8; 32];
        let server_random = [3u8; 32];

        let km = KeyManager::new(shared_secret, client_random, server_random, true).unwrap();

        let time_left = km.time_until_rotation().await;
        assert!(time_left <= KEY_ROTATION_INTERVAL);
    }

    #[tokio::test]
    async fn test_clear_keys() {
        let km = create_test_key_manager();

        // Clear keys
        km.clear_keys().await;

        let keys = km.get_keys().await;

        // Keys should be zeroed
        assert_eq!(&*keys.chacha_key, &[0u8; 32]);
        assert_eq!(&*keys.aes_key, &[0u8; 32]);
    }

    #[tokio::test]
    async fn test_multiple_rotations() {
        let km = create_test_key_manager();

        let mut previous_keys = km.get_keys().await;

        // Perform multiple rotations
        for _ in 0..5 {
            km.rotate_keys().await.unwrap();
            let current_keys = km.get_keys().await;

            // Each rotation should produce different keys
            assert_ne!(&*previous_keys.chacha_key, &*current_keys.chacha_key);
            assert_ne!(&*previous_keys.aes_key, &*current_keys.aes_key);

            previous_keys = current_keys;
        }
    }
}
