use crate::crypto::{AesEncryptor, ChaChaEncryptor};
use crate::error::{LostLoveError, Result};
use zeroize::Zeroizing;

/// Hybrid Symmetric Encryption (HSE)
/// Combines ChaCha20-Poly1305 and AES-256-GCM for double encryption
/// Formula: HSE = ChaCha20(data) ⊕ AES256(data)
pub struct HSEEncryptor {
    chacha: ChaChaEncryptor,
    aes: AesEncryptor,
}

impl HSEEncryptor {
    /// Create new HSE encryptor with both keys
    pub fn new(chacha_key: &[u8; 32], aes_key: &[u8; 32]) -> Self {
        Self {
            chacha: ChaChaEncryptor::new(chacha_key),
            aes: AesEncryptor::new(aes_key),
        }
    }

    /// Encrypt data using hybrid encryption
    /// Process:
    /// 1. Encrypt with ChaCha20-Poly1305
    /// 2. Encrypt with AES-256-GCM
    /// 3. XOR the two ciphertexts together
    pub fn encrypt(&self, plaintext: &[u8], nonce: &[u8; 12]) -> Result<Vec<u8>> {
        // Encrypt with both algorithms
        let chacha_encrypted = self.chacha.encrypt(plaintext, nonce)?;
        let aes_encrypted = self.aes.encrypt(plaintext, nonce)?;

        // Ensure both ciphertexts are the same length
        if chacha_encrypted.len() != aes_encrypted.len() {
            return Err(LostLoveError::Crypto(
                "Ciphertext length mismatch in HSE".to_string(),
            ));
        }

        // XOR the two ciphertexts
        let mut result = Vec::with_capacity(chacha_encrypted.len());
        for (c1, c2) in chacha_encrypted.iter().zip(aes_encrypted.iter()) {
            result.push(c1 ^ c2);
        }

        Ok(result)
    }

    /// Decrypt data using hybrid decryption
    /// Process:
    /// 1. XOR with AES ciphertext to recover ChaCha ciphertext
    /// 2. XOR with ChaCha ciphertext to recover AES ciphertext
    /// 3. Decrypt both and verify they match
    pub fn decrypt(&self, ciphertext: &[u8], nonce: &[u8; 12]) -> Result<Vec<u8>> {
        // We need to try both algorithms separately since we have XORed data
        // C_combined = C_chacha ⊕ C_aes
        // To decrypt:
        // 1. Decrypt with ChaCha: D_chacha(C_combined ⊕ E_aes(plaintext))
        // 2. Decrypt with AES: D_aes(C_combined ⊕ E_chacha(plaintext))
        //
        // Since we don't know the plaintext, we need to use a different approach:
        // We'll use the property that both encryptions should produce the same plaintext

        // For now, we'll use a brute-force approach with length estimation
        // In a real implementation, we'd need to store metadata about the original length

        // Estimate plaintext length (ciphertext - auth tag overhead)
        // ChaCha20-Poly1305 adds 16 bytes, AES-GCM adds 16 bytes
        let estimated_plaintext_len = if ciphertext.len() > 32 {
            ciphertext.len() - 32
        } else {
            return Err(LostLoveError::Crypto(
                "HSE ciphertext too short".to_string(),
            ));
        };

        // Try different plaintext lengths around the estimate
        for plaintext_len in (estimated_plaintext_len.saturating_sub(10))
            ..=(estimated_plaintext_len + 10)
        {
            if let Ok(plaintext) = self.try_decrypt_with_length(ciphertext, nonce, plaintext_len)
            {
                return Ok(plaintext);
            }
        }

        Err(LostLoveError::Crypto(
            "HSE decryption failed: could not find valid plaintext".to_string(),
        ))
    }

    /// Try to decrypt with a specific plaintext length
    fn try_decrypt_with_length(
        &self,
        combined_ciphertext: &[u8],
        nonce: &[u8; 12],
        plaintext_len: usize,
    ) -> Result<Vec<u8>> {
        // Try to decrypt as if the combined ciphertext is valid
        // We'll create dummy plaintexts and check if they work

        // Create a test plaintext of the specified length
        let test_plaintext = vec![0u8; plaintext_len];

        // Encrypt test plaintext with both algorithms
        let chacha_test = self.chacha.encrypt(&test_plaintext, nonce)?;
        let aes_test = self.aes.encrypt(&test_plaintext, nonce)?;

        // Check if the lengths match
        if chacha_test.len() != combined_ciphertext.len()
            || aes_test.len() != combined_ciphertext.len()
        {
            return Err(LostLoveError::Crypto("Length mismatch".to_string()));
        }

        // XOR combined ciphertext with AES test to get ChaCha ciphertext
        let mut chacha_ciphertext = Vec::with_capacity(combined_ciphertext.len());
        for (combined, aes_byte) in combined_ciphertext.iter().zip(aes_test.iter()) {
            chacha_ciphertext.push(combined ^ aes_byte);
        }

        // Try to decrypt the ChaCha ciphertext
        if let Ok(plaintext1) = self.chacha.decrypt(&chacha_ciphertext, nonce) {
            // Now verify with AES
            let mut aes_ciphertext = Vec::with_capacity(combined_ciphertext.len());
            for (combined, chacha_byte) in combined_ciphertext.iter().zip(chacha_test.iter()) {
                aes_ciphertext.push(combined ^ chacha_byte);
            }

            if let Ok(plaintext2) = self.aes.decrypt(&aes_ciphertext, nonce) {
                // Both should produce the same plaintext
                if plaintext1 == plaintext2 {
                    return Ok(plaintext1);
                }
            }
        }

        Err(LostLoveError::Crypto("Decryption failed".to_string()))
    }

    /// Generate random keys for HSE
    pub fn generate_keys() -> (Zeroizing<[u8; 32]>, Zeroizing<[u8; 32]>) {
        (
            ChaChaEncryptor::generate_key(),
            AesEncryptor::generate_key(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_hse() -> HSEEncryptor {
        let chacha_key = [1u8; 32];
        let aes_key = [2u8; 32];
        HSEEncryptor::new(&chacha_key, &aes_key)
    }

    #[test]
    fn test_hse_encrypt_decrypt() {
        let hse = create_test_hse();
        let plaintext = b"Hello, LostLove Protocol!";
        let nonce = [0u8; 12];

        let ciphertext = hse.encrypt(plaintext, &nonce).unwrap();
        let decrypted = hse.decrypt(&ciphertext, &nonce).unwrap();

        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_hse_different_from_single_encryption() {
        let hse = create_test_hse();
        let plaintext = b"Test data";
        let nonce = [0u8; 12];

        // HSE encryption
        let hse_ciphertext = hse.encrypt(plaintext, &nonce).unwrap();

        // Single ChaCha encryption
        let chacha_key = [1u8; 32];
        let chacha = ChaChaEncryptor::new(&chacha_key);
        let chacha_ciphertext = chacha.encrypt(plaintext, &nonce).unwrap();

        // HSE should produce different output than single encryption
        assert_ne!(hse_ciphertext, chacha_ciphertext);
    }

    #[test]
    fn test_hse_with_various_sizes() {
        let hse = create_test_hse();
        let nonce = [0u8; 12];

        // Test various plaintext sizes
        for size in [1, 10, 100, 1000, 10000] {
            let plaintext = vec![42u8; size];
            let ciphertext = hse.encrypt(&plaintext, &nonce).unwrap();
            let decrypted = hse.decrypt(&ciphertext, &nonce).unwrap();

            assert_eq!(decrypted, plaintext, "Failed for size {}", size);
        }
    }

    #[test]
    fn test_hse_tampering_detection() {
        let hse = create_test_hse();
        let plaintext = b"Sensitive data";
        let nonce = [0u8; 12];

        let mut ciphertext = hse.encrypt(plaintext, &nonce).unwrap();

        // Tamper with the ciphertext
        if !ciphertext.is_empty() {
            ciphertext[0] ^= 1;
        }

        // Decryption should fail
        let result = hse.decrypt(&ciphertext, &nonce);
        assert!(result.is_err());
    }

    #[test]
    fn test_hse_wrong_nonce() {
        let hse = create_test_hse();
        let plaintext = b"Secret message";
        let nonce1 = [1u8; 12];
        let nonce2 = [2u8; 12];

        let ciphertext = hse.encrypt(plaintext, &nonce1).unwrap();

        // Try to decrypt with wrong nonce
        let result = hse.decrypt(&ciphertext, &nonce2);
        assert!(result.is_err());
    }

    #[test]
    fn test_hse_deterministic() {
        let hse = create_test_hse();
        let plaintext = b"Deterministic test";
        let nonce = [0u8; 12];

        let ciphertext1 = hse.encrypt(plaintext, &nonce).unwrap();
        let ciphertext2 = hse.encrypt(plaintext, &nonce).unwrap();

        // Same input should produce same output
        assert_eq!(ciphertext1, ciphertext2);
    }

    #[test]
    fn test_hse_different_keys_produce_different_output() {
        let hse1 = HSEEncryptor::new(&[1u8; 32], &[2u8; 32]);
        let hse2 = HSEEncryptor::new(&[3u8; 32], &[4u8; 32]);

        let plaintext = b"Test message";
        let nonce = [0u8; 12];

        let ciphertext1 = hse1.encrypt(plaintext, &nonce).unwrap();
        let ciphertext2 = hse2.encrypt(plaintext, &nonce).unwrap();

        assert_ne!(ciphertext1, ciphertext2);
    }

    #[test]
    fn test_generate_keys() {
        let (key1, key2) = HSEEncryptor::generate_keys();

        // Keys should be different
        assert_ne!(&*key1, &*key2);

        // Keys should be valid length
        assert_eq!(key1.len(), 32);
        assert_eq!(key2.len(), 32);
    }
}
