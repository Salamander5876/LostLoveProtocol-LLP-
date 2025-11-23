use hkdf::Hkdf;
use sha2::Sha512;
use zeroize::Zeroizing;

use crate::error::{LostLoveError, Result};

/// Derive keys using HKDF-SHA512
pub fn derive_keys(
    secret: &[u8],
    salt: &[u8],
    info: &[u8],
    output_length: usize,
) -> Result<Zeroizing<Vec<u8>>> {
    let hk = Hkdf::<Sha512>::new(Some(salt), secret);

    let mut okm = Zeroizing::new(vec![0u8; output_length]);

    hk.expand(info, &mut okm)
        .map_err(|_| LostLoveError::Connection("HKDF key derivation failed".to_string()))?;

    Ok(okm)
}

/// Derive session keys from shared secret
pub fn derive_session_keys(
    shared_secret: &[u8],
    client_random: &[u8; 32],
    server_random: &[u8; 32],
) -> Result<SessionKeys> {
    // Create salt from random values
    let mut salt = Vec::with_capacity(64);
    salt.extend_from_slice(client_random);
    salt.extend_from_slice(server_random);

    // Derive master secret (64 bytes)
    let master_secret = derive_keys(
        shared_secret,
        &salt,
        b"LLP-v1-master-secret",
        64,
    )?;

    // Derive ChaCha20 key (32 bytes)
    let chacha_key = derive_keys(
        &master_secret,
        &[],
        b"LLP-chacha20-key",
        32,
    )?;

    // Derive AES key (32 bytes)
    let aes_key = derive_keys(
        &master_secret,
        &[],
        b"LLP-aes-key",
        32,
    )?;

    // Convert to fixed-size arrays
    let chacha_key_array: [u8; 32] = chacha_key[..]
        .try_into()
        .map_err(|_| LostLoveError::Connection("Invalid key length".to_string()))?;

    let aes_key_array: [u8; 32] = aes_key[..]
        .try_into()
        .map_err(|_| LostLoveError::Connection("Invalid key length".to_string()))?;

    let master_secret_array: [u8; 64] = master_secret[..]
        .try_into()
        .map_err(|_| LostLoveError::Connection("Invalid master secret length".to_string()))?;

    Ok(SessionKeys {
        chacha_key: Zeroizing::new(chacha_key_array),
        aes_key: Zeroizing::new(aes_key_array),
        master_secret: Zeroizing::new(master_secret_array),
    })
}

/// Session keys derived from handshake
#[derive(Clone)]
pub struct SessionKeys {
    pub chacha_key: Zeroizing<[u8; 32]>,
    pub aes_key: Zeroizing<[u8; 32]>,
    pub master_secret: Zeroizing<[u8; 64]>,
}

impl SessionKeys {
    /// Create from raw keys (for testing)
    pub fn from_raw(chacha_key: [u8; 32], aes_key: [u8; 32]) -> Self {
        Self {
            chacha_key: Zeroizing::new(chacha_key),
            aes_key: Zeroizing::new(aes_key),
            master_secret: Zeroizing::new([0u8; 64]),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kdf_deterministic() {
        let secret = b"test_secret";
        let salt = b"test_salt";
        let info = b"test_info";

        let key1 = derive_keys(secret, salt, info, 32).unwrap();
        let key2 = derive_keys(secret, salt, info, 32).unwrap();

        // Same inputs should produce same output
        assert_eq!(&*key1, &*key2);
    }

    #[test]
    fn test_kdf_different_info() {
        let secret = b"test_secret";
        let salt = b"test_salt";

        let key1 = derive_keys(secret, salt, b"info1", 32).unwrap();
        let key2 = derive_keys(secret, salt, b"info2", 32).unwrap();

        // Different info should produce different keys
        assert_ne!(&*key1, &*key2);
    }

    #[test]
    fn test_session_keys_derivation() {
        let shared_secret = b"shared_secret_from_key_exchange";
        let client_random = [1u8; 32];
        let server_random = [2u8; 32];

        let keys = derive_session_keys(shared_secret, &client_random, &server_random).unwrap();

        // Keys should be different
        assert_ne!(&*keys.chacha_key, &*keys.aes_key);

        // Keys should have correct length
        assert_eq!(keys.chacha_key.len(), 32);
        assert_eq!(keys.aes_key.len(), 32);
        assert_eq!(keys.master_secret.len(), 64);
    }

    #[test]
    fn test_session_keys_deterministic() {
        let shared_secret = b"shared_secret";
        let client_random = [1u8; 32];
        let server_random = [2u8; 32];

        let keys1 = derive_session_keys(shared_secret, &client_random, &server_random).unwrap();
        let keys2 = derive_session_keys(shared_secret, &client_random, &server_random).unwrap();

        // Same inputs should produce same keys
        assert_eq!(&*keys1.chacha_key, &*keys2.chacha_key);
        assert_eq!(&*keys1.aes_key, &*keys2.aes_key);
        assert_eq!(&*keys1.master_secret, &*keys2.master_secret);
    }

    #[test]
    fn test_different_random_produces_different_keys() {
        let shared_secret = b"shared_secret";
        let client_random1 = [1u8; 32];
        let client_random2 = [2u8; 32];
        let server_random = [3u8; 32];

        let keys1 = derive_session_keys(shared_secret, &client_random1, &server_random).unwrap();
        let keys2 = derive_session_keys(shared_secret, &client_random2, &server_random).unwrap();

        // Different random should produce different keys
        assert_ne!(&*keys1.chacha_key, &*keys2.chacha_key);
        assert_ne!(&*keys1.aes_key, &*keys2.aes_key);
    }

    #[test]
    fn test_kdf_various_lengths() {
        let secret = b"test_secret";
        let salt = b"test_salt";
        let info = b"test_info";

        // Test different output lengths
        for length in [16, 32, 64, 128] {
            let key = derive_keys(secret, salt, info, length).unwrap();
            assert_eq!(key.len(), length);
        }
    }
}
