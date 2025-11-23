use chacha20poly1305::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    ChaCha20Poly1305, Key, Nonce,
};
use zeroize::Zeroizing;

use crate::error::{LostLoveError, Result};

/// ChaCha20-Poly1305 encryptor
pub struct ChaChaEncryptor {
    cipher: ChaCha20Poly1305,
}

impl ChaChaEncryptor {
    /// Create new encryptor with key
    pub fn new(key: &[u8; 32]) -> Self {
        let key = Key::from_slice(key);
        let cipher = ChaCha20Poly1305::new(key);

        Self { cipher }
    }

    /// Generate random key
    pub fn generate_key() -> Zeroizing<[u8; 32]> {
        let key = ChaCha20Poly1305::generate_key(&mut OsRng);
        Zeroizing::new(*key.as_ref())
    }

    /// Generate random nonce
    pub fn generate_nonce() -> [u8; 12] {
        let nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng);
        *nonce.as_ref()
    }

    /// Encrypt data
    pub fn encrypt(&self, plaintext: &[u8], nonce: &[u8; 12]) -> Result<Vec<u8>> {
        let nonce = Nonce::from_slice(nonce);

        self.cipher
            .encrypt(nonce, plaintext)
            .map_err(|e| LostLoveError::Connection(format!("ChaCha20 encryption failed: {}", e)))
    }

    /// Decrypt data
    pub fn decrypt(&self, ciphertext: &[u8], nonce: &[u8; 12]) -> Result<Vec<u8>> {
        let nonce = Nonce::from_slice(nonce);

        self.cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| LostLoveError::Connection(format!("ChaCha20 decryption failed: {}", e)))
    }

    /// Encrypt in-place (modifies the buffer)
    pub fn encrypt_in_place(&self, buffer: &mut Vec<u8>, nonce: &[u8; 12]) -> Result<()> {
        let nonce_obj = Nonce::from_slice(nonce);

        self.cipher
            .encrypt_in_place(nonce_obj, b"", buffer)
            .map_err(|e| LostLoveError::Connection(format!("ChaCha20 encryption failed: {}", e)))
    }

    /// Decrypt in-place (modifies the buffer)
    pub fn decrypt_in_place(&self, buffer: &mut Vec<u8>, nonce: &[u8; 12]) -> Result<()> {
        let nonce_obj = Nonce::from_slice(nonce);

        self.cipher
            .decrypt_in_place(nonce_obj, b"", buffer)
            .map_err(|e| LostLoveError::Connection(format!("ChaCha20 decryption failed: {}", e)))
    }

    /// Get key size
    pub const fn key_size() -> usize {
        32 // 256 bits
    }

    /// Get nonce size
    pub const fn nonce_size() -> usize {
        12 // 96 bits
    }

    /// Get auth tag size
    pub const fn tag_size() -> usize {
        16 // 128 bits
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt() {
        let key = ChaChaEncryptor::generate_key();
        let encryptor = ChaChaEncryptor::new(&key);

        let plaintext = b"Hello, LostLove Protocol!";
        let nonce = ChaChaEncryptor::generate_nonce();

        // Encrypt
        let ciphertext = encryptor.encrypt(plaintext, &nonce).unwrap();

        // Verify ciphertext is different
        assert_ne!(ciphertext, plaintext);

        // Decrypt
        let decrypted = encryptor.decrypt(&ciphertext, &nonce).unwrap();

        // Verify plaintext matches
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_encrypt_in_place() {
        let key = ChaChaEncryptor::generate_key();
        let encryptor = ChaChaEncryptor::new(&key);

        let plaintext = b"Hello, LostLove!";
        let nonce = ChaChaEncryptor::generate_nonce();

        let mut buffer = plaintext.to_vec();
        let original = buffer.clone();

        // Encrypt in place
        encryptor.encrypt_in_place(&mut buffer, &nonce).unwrap();

        // Verify encrypted
        assert_ne!(buffer, original);

        // Decrypt in place
        encryptor.decrypt_in_place(&mut buffer, &nonce).unwrap();

        // Verify decrypted
        assert_eq!(buffer, original);
    }

    #[test]
    fn test_wrong_nonce() {
        let key = ChaChaEncryptor::generate_key();
        let encryptor = ChaChaEncryptor::new(&key);

        let plaintext = b"Test data";
        let nonce1 = ChaChaEncryptor::generate_nonce();
        let nonce2 = ChaChaEncryptor::generate_nonce();

        let ciphertext = encryptor.encrypt(plaintext, &nonce1).unwrap();

        // Try to decrypt with wrong nonce - should fail
        let result = encryptor.decrypt(&ciphertext, &nonce2);
        assert!(result.is_err());
    }

    #[test]
    fn test_wrong_key() {
        let key1 = ChaChaEncryptor::generate_key();
        let key2 = ChaChaEncryptor::generate_key();

        let encryptor1 = ChaChaEncryptor::new(&key1);
        let encryptor2 = ChaChaEncryptor::new(&key2);

        let plaintext = b"Test data";
        let nonce = ChaChaEncryptor::generate_nonce();

        let ciphertext = encryptor1.encrypt(plaintext, &nonce).unwrap();

        // Try to decrypt with wrong key - should fail
        let result = encryptor2.decrypt(&ciphertext, &nonce);
        assert!(result.is_err());
    }

    #[test]
    fn test_tampering_detection() {
        let key = ChaChaEncryptor::generate_key();
        let encryptor = ChaChaEncryptor::new(&key);

        let plaintext = b"Important data";
        let nonce = ChaChaEncryptor::generate_nonce();

        let mut ciphertext = encryptor.encrypt(plaintext, &nonce).unwrap();

        // Tamper with ciphertext
        if !ciphertext.is_empty() {
            ciphertext[0] ^= 0xFF;
        }

        // Decryption should fail due to authentication
        let result = encryptor.decrypt(&ciphertext, &nonce);
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_data() {
        let key = ChaChaEncryptor::generate_key();
        let encryptor = ChaChaEncryptor::new(&key);

        let plaintext = b"";
        let nonce = ChaChaEncryptor::generate_nonce();

        let ciphertext = encryptor.encrypt(plaintext, &nonce).unwrap();
        let decrypted = encryptor.decrypt(&ciphertext, &nonce).unwrap();

        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_large_data() {
        let key = ChaChaEncryptor::generate_key();
        let encryptor = ChaChaEncryptor::new(&key);

        // 1 MB of data
        let plaintext = vec![0x42u8; 1024 * 1024];
        let nonce = ChaChaEncryptor::generate_nonce();

        let ciphertext = encryptor.encrypt(&plaintext, &nonce).unwrap();
        let decrypted = encryptor.decrypt(&ciphertext, &nonce).unwrap();

        assert_eq!(decrypted, plaintext);
    }
}
