# Phase 2: Cryptography Usage Guide

## Quick Reference

### Import Crypto Modules

```rust
use lostlove_server::crypto::{
    ChaChaEncryptor,
    AesEncryptor,
    HSEEncryptor,
    KeyManager,
    SessionKeys,
    derive_session_keys,
};
```

## Basic Usage

### 1. Single Algorithm Encryption

#### ChaCha20-Poly1305

```rust
// Generate a key
let key = ChaChaEncryptor::generate_key();

// Create encryptor
let chacha = ChaChaEncryptor::new(&key);

// Encrypt data
let nonce = [0u8; 12]; // In production, use unique nonce per message
let plaintext = b"Hello, LostLove!";
let ciphertext = chacha.encrypt(plaintext, &nonce)?;

// Decrypt data
let decrypted = chacha.decrypt(&ciphertext, &nonce)?;
assert_eq!(decrypted, plaintext);
```

#### AES-256-GCM

```rust
// Generate a key
let key = AesEncryptor::generate_key();

// Create encryptor
let aes = AesEncryptor::new(&key);

// Encrypt data
let nonce = [0u8; 12];
let plaintext = b"Secret message";
let ciphertext = aes.encrypt(plaintext, &nonce)?;

// Decrypt data
let decrypted = aes.decrypt(&ciphertext, &nonce)?;
assert_eq!(decrypted, plaintext);
```

### 2. Hybrid Symmetric Encryption (HSE)

```rust
// Generate both keys
let (chacha_key, aes_key) = HSEEncryptor::generate_keys();

// Create HSE encryptor
let hse = HSEEncryptor::new(&chacha_key, &aes_key);

// Encrypt with double encryption
let nonce = [0u8; 12];
let plaintext = b"Top secret data";
let ciphertext = hse.encrypt(plaintext, &nonce)?;

// Decrypt with automatic verification
let decrypted = hse.decrypt(&ciphertext, &nonce)?;
assert_eq!(decrypted, plaintext);
```

### 3. Key Derivation

```rust
// Derive session keys from handshake
let shared_secret = b"shared_secret_from_key_exchange";
let client_random = [1u8; 32];
let server_random = [2u8; 32];

let session_keys = derive_session_keys(
    shared_secret,
    &client_random,
    &server_random,
)?;

// Use derived keys
let hse = HSEEncryptor::new(
    &session_keys.chacha_key,
    &session_keys.aes_key,
);
```

### 4. Key Manager (Recommended)

```rust
// Create key manager with auto-rotation
let key_manager = KeyManager::new(
    shared_secret.to_vec(),
    client_random,
    server_random,
    true, // Enable automatic rotation
)?;

// Get HSE encryptor
let hse = key_manager.get_hse_encryptor().await;

// Encrypt data
let ciphertext = hse.encrypt(plaintext, &nonce)?;

// Decrypt with automatic fallback during rotation
let decrypted = key_manager
    .decrypt_with_fallback(&ciphertext, &nonce)
    .await?;

// Check if rotation is needed
if key_manager.check_rotation().await? {
    println!("Keys rotated successfully");
}

// Force immediate rotation
key_manager.rotate_keys().await?;

// Get time until next rotation
let time_left = key_manager.time_until_rotation().await;
println!("Next rotation in: {:?}", time_left);
```

## Integration with Protocol

### Session Establishment

```rust
// During handshake
async fn establish_session(
    shared_secret: Vec<u8>,
    client_random: [u8; 32],
    server_random: [u8; 32],
) -> Result<KeyManager> {
    // Create key manager
    let key_manager = KeyManager::new(
        shared_secret,
        client_random,
        server_random,
        true, // Enable auto-rotation
    )?;

    Ok(key_manager)
}
```

### Packet Encryption

```rust
// Encrypt outgoing packet
async fn encrypt_packet(
    key_manager: &KeyManager,
    payload: &[u8],
) -> Result<Vec<u8>> {
    // Generate unique nonce from sequence number or timestamp
    let nonce = generate_nonce();

    // Get HSE encryptor
    let hse = key_manager.get_hse_encryptor().await;

    // Encrypt payload
    let encrypted = hse.encrypt(payload, &nonce)?;

    Ok(encrypted)
}
```

### Packet Decryption

```rust
// Decrypt incoming packet
async fn decrypt_packet(
    key_manager: &KeyManager,
    ciphertext: &[u8],
    nonce: &[u8; 12],
) -> Result<Vec<u8>> {
    // Decrypt with automatic fallback to previous keys
    let plaintext = key_manager
        .decrypt_with_fallback(ciphertext, nonce)
        .await?;

    Ok(plaintext)
}
```

### Background Key Rotation

```rust
// Start key rotation task
fn start_key_rotation(key_manager: Arc<KeyManager>) {
    tokio::spawn(async move {
        loop {
            // Check every minute
            tokio::time::sleep(Duration::from_secs(60)).await;

            match key_manager.check_rotation().await {
                Ok(true) => {
                    info!("Keys rotated successfully");
                },
                Ok(false) => {
                    // No rotation needed yet
                },
                Err(e) => {
                    error!("Key rotation failed: {}", e);
                }
            }
        }
    });
}
```

## Nonce Generation

### Important: Never Reuse Nonces!

```rust
use std::sync::atomic::{AtomicU64, Ordering};

// Counter-based nonce (recommended for ordered packets)
struct NonceGenerator {
    counter: AtomicU64,
}

impl NonceGenerator {
    fn new() -> Self {
        Self {
            counter: AtomicU64::new(0),
        }
    }

    fn generate(&self) -> [u8; 12] {
        let count = self.counter.fetch_add(1, Ordering::SeqCst);
        let mut nonce = [0u8; 12];
        nonce[..8].copy_from_slice(&count.to_be_bytes());
        nonce
    }
}

// Random nonce (for unordered scenarios)
use rand::RngCore;

fn generate_random_nonce() -> [u8; 12] {
    let mut nonce = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut nonce);
    nonce
}

// Timestamp + counter hybrid (recommended)
fn generate_hybrid_nonce(counter: &AtomicU64) -> [u8; 12] {
    let mut nonce = [0u8; 12];

    // First 4 bytes: timestamp (seconds)
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as u32;
    nonce[..4].copy_from_slice(&timestamp.to_be_bytes());

    // Next 8 bytes: counter
    let count = counter.fetch_add(1, Ordering::SeqCst);
    nonce[4..].copy_from_slice(&count.to_be_bytes());

    nonce
}
```

## Error Handling

```rust
use lostlove_server::error::LostLoveError;

match hse.decrypt(&ciphertext, &nonce) {
    Ok(plaintext) => {
        // Success
        process_data(&plaintext);
    },
    Err(LostLoveError::Crypto(msg)) => {
        // Decryption failed (wrong key, tampered data, wrong nonce)
        error!("Decryption failed: {}", msg);
    },
    Err(e) => {
        // Other error
        error!("Unexpected error: {}", e);
    }
}
```

## Performance Tips

### 1. Reuse Encryptors

```rust
// ✅ Good: Create once, use many times
let hse = HSEEncryptor::new(&chacha_key, &aes_key);
for message in messages {
    let encrypted = hse.encrypt(message, &nonce)?;
}

// ❌ Bad: Creating new encryptor for each message
for message in messages {
    let hse = HSEEncryptor::new(&chacha_key, &aes_key);
    let encrypted = hse.encrypt(message, &nonce)?;
}
```

### 2. Batch Processing

```rust
// Process multiple packets in parallel
use futures::future::join_all;

async fn encrypt_packets(
    key_manager: &KeyManager,
    packets: Vec<Vec<u8>>,
) -> Vec<Result<Vec<u8>>> {
    let hse = key_manager.get_hse_encryptor().await;

    let futures = packets
        .into_iter()
        .enumerate()
        .map(|(i, packet)| {
            let hse = hse.clone();
            async move {
                let nonce = generate_nonce_from_index(i);
                hse.encrypt(&packet, &nonce)
            }
        });

    join_all(futures).await
}
```

### 3. Minimize Key Rotations

```rust
// Only rotate when necessary
let key_manager = KeyManager::new(
    shared_secret,
    client_random,
    server_random,
    true, // Auto-rotation every 30 minutes
)?;

// Avoid frequent manual rotations
// key_manager.rotate_keys().await?; // Only if needed
```

## Security Best Practices

### 1. Always Use Unique Nonces

```rust
// ❌ NEVER do this
let nonce = [0u8; 12];
hse.encrypt(&msg1, &nonce)?;
hse.encrypt(&msg2, &nonce)?; // DANGEROUS!

// ✅ Always use unique nonces
let nonce1 = generate_nonce();
let nonce2 = generate_nonce();
hse.encrypt(&msg1, &nonce1)?;
hse.encrypt(&msg2, &nonce2)?;
```

### 2. Store Nonces with Ciphertext

```rust
// Prepend nonce to ciphertext
fn encrypt_with_nonce(
    hse: &HSEEncryptor,
    plaintext: &[u8],
) -> Result<Vec<u8>> {
    let nonce = generate_nonce();
    let ciphertext = hse.encrypt(plaintext, &nonce)?;

    // Combine nonce + ciphertext
    let mut result = Vec::with_capacity(12 + ciphertext.len());
    result.extend_from_slice(&nonce);
    result.extend_from_slice(&ciphertext);

    Ok(result)
}

fn decrypt_with_nonce(
    hse: &HSEEncryptor,
    data: &[u8],
) -> Result<Vec<u8>> {
    if data.len() < 12 {
        return Err(LostLoveError::Crypto("Data too short".into()));
    }

    // Extract nonce and ciphertext
    let nonce: [u8; 12] = data[..12].try_into().unwrap();
    let ciphertext = &data[12..];

    hse.decrypt(ciphertext, &nonce)
}
```

### 3. Clear Sensitive Data

```rust
// Keys are automatically cleared with Zeroizing
let keys = derive_session_keys(&secret, &cr, &sr)?;
// keys.chacha_key is zeroed when dropped

// For other sensitive data, use Zeroizing wrapper
use zeroize::Zeroizing;

let password = Zeroizing::new(get_password());
// password is zeroed when dropped
```

### 4. Verify Data Before Use

```rust
// HSE automatically verifies authentication tags
match hse.decrypt(&ciphertext, &nonce) {
    Ok(plaintext) => {
        // Data is verified authentic and unmodified
        process_verified_data(&plaintext);
    },
    Err(_) => {
        // Data was tampered with or wrong key
        reject_packet();
    }
}
```

## Testing Your Implementation

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_full_encryption_flow() {
        // Setup
        let shared_secret = vec![1u8; 32];
        let client_random = [2u8; 32];
        let server_random = [3u8; 32];

        let key_manager = KeyManager::new(
            shared_secret,
            client_random,
            server_random,
            false,
        ).unwrap();

        // Test encryption/decryption
        let plaintext = b"Test message";
        let nonce = [0u8; 12];

        let hse = key_manager.get_hse_encryptor().await;
        let ciphertext = hse.encrypt(plaintext, &nonce).unwrap();

        let decrypted = key_manager
            .decrypt_with_fallback(&ciphertext, &nonce)
            .await
            .unwrap();

        assert_eq!(decrypted, plaintext);
    }

    #[tokio::test]
    async fn test_key_rotation_flow() {
        let key_manager = create_test_key_manager();

        // Encrypt with current keys
        let plaintext = b"Secret";
        let nonce = [0u8; 12];
        let hse = key_manager.get_hse_encryptor().await;
        let ciphertext = hse.encrypt(plaintext, &nonce).unwrap();

        // Rotate keys
        key_manager.rotate_keys().await.unwrap();

        // Should still decrypt with fallback
        let decrypted = key_manager
            .decrypt_with_fallback(&ciphertext, &nonce)
            .await
            .unwrap();

        assert_eq!(decrypted, plaintext);
    }
}
```

## Troubleshooting

### Decryption Fails

**Possible causes:**
1. Wrong nonce
2. Wrong keys
3. Data was tampered with
4. Key rotation without fallback

**Solution:**
```rust
// Use KeyManager's decrypt_with_fallback
let result = key_manager.decrypt_with_fallback(&data, &nonce).await;

match result {
    Ok(plaintext) => { /* Success */ },
    Err(e) => {
        error!("Decryption failed: {}", e);
        // Check: Are you using the correct nonce?
        // Check: Did keys rotate without saving previous keys?
        // Check: Was data corrupted in transit?
    }
}
```

### Performance Issues

**Symptoms:** Slow encryption/decryption

**Solutions:**
1. Reuse encryptors instead of creating new ones
2. Use release build: `cargo build --release`
3. Enable CPU optimizations in Cargo.toml:
```toml
[profile.release]
opt-level = 3
lto = true
codegen-units = 1
```

### Memory Leaks with Keys

**Solution:** Use Zeroizing wrapper (already done in Phase 2)
```rust
// ✅ Automatic zeroing
let key = ChaChaEncryptor::generate_key(); // Returns Zeroizing<[u8; 32]>
// Key is zeroed when dropped

// ❌ Manual arrays might leak
let key = [0u8; 32]; // Not zeroed on drop
```

## Next Steps

See [PHASE2_COMPLETE.md](../../PHASE2_COMPLETE.md) for:
- Complete implementation details
- Architecture diagrams
- Performance benchmarks
- Integration with Phase 3 (Quantum-Resistant Layer)
