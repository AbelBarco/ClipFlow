//! Optional encryption at rest for `content` / `ocr_text` columns.
//!
//! - Cipher: XChaCha20-Poly1305 (256-bit, pure Rust).
//! - Key: 32 random bytes kept in the OS keychain via the `keyring` crate
//!   (DPAPI on Windows, Keychain on macOS, Secret Service on Linux) — never
//!   in the database or config files.
//! - Envelope: `"cf1:" + base64(nonce24 || ciphertext)`.
//! - Reads are backward compatible: values without the prefix are treated
//!   as legacy plaintext (the `security_set_encryption` command migrates
//!   every row when the user opts in).

use base64::Engine as _;
use chacha20poly1305::aead::{Aead, KeyInit};
use chacha20poly1305::{XChaCha20Poly1305, XNonce};

const PREFIX: &str = "cf1:";
const KEYRING_SERVICE: &str = "clipflow";
const KEYRING_USER: &str = "clipflow-db-key";

/// Encrypt a plaintext value for storage. Fails when the OS keychain is
/// unavailable — callers must surface the error instead of silently
/// storing plaintext while the UI claims encryption is on.
pub fn encrypt_to_storage(plaintext: &str) -> Result<String, String> {
    let key = load_or_create_key()?;
    Ok(encrypt_with_key(plaintext, &key))
}

/// Decrypt a stored value. Legacy plaintext (no prefix) passes through so
/// enabling encryption on an existing database never bricks old rows.
pub fn decrypt_from_storage(stored: &str) -> Result<String, String> {
    if !is_encrypted(stored) {
        return Ok(stored.to_string());
    }
    let key = load_or_create_key()?;
    decrypt_with_key(stored, &key)
}

pub fn is_encrypted(stored: &str) -> bool {
    stored.starts_with(PREFIX)
}

fn load_or_create_key() -> Result<[u8; 32], String> {
    let entry = keyring::Entry::new(KEYRING_SERVICE, KEYRING_USER).map_err(|e| e.to_string())?;
    match entry.get_password() {
        Ok(encoded) => {
            let raw = base64::engine::general_purpose::STANDARD
                .decode(encoded.trim())
                .map_err(|e| format!("Stored encryption key is corrupt: {e}"))?;
            raw.try_into()
                .map_err(|_| "Stored encryption key has wrong length".to_string())
        }
        Err(_) => {
            // No key yet (or unreadable) — generate and persist one.
            // Randomness: two v4 UUIDs (CSPRNG-backed) concatenated.
            let a = uuid::Uuid::new_v4().into_bytes();
            let b = uuid::Uuid::new_v4().into_bytes();
            let mut key = [0u8; 32];
            key[..16].copy_from_slice(&a);
            key[16..].copy_from_slice(&b);
            let encoded = base64::engine::general_purpose::STANDARD.encode(key);
            entry
                .set_password(&encoded)
                .map_err(|e| format!("OS keychain unavailable: {e}"))?;
            Ok(key)
        }
    }
}

/// Pure encryptor (key injected) — used by the storage layer and unit tests.
fn encrypt_with_key(plaintext: &str, key: &[u8; 32]) -> String {
    let cipher = XChaCha20Poly1305::new_from_slice(key).expect("32-byte key");
    // 24-byte nonce from two CSPRNG-backed UUIDs; uniqueness per message
    // is what matters, and collisions are cryptographically negligible.
    let a = uuid::Uuid::new_v4().into_bytes();
    let b = uuid::Uuid::new_v4().into_bytes();
    let mut nonce_bytes = [0u8; 24];
    nonce_bytes[..16].copy_from_slice(&a);
    nonce_bytes[16..].copy_from_slice(&b[..8]);

    let ciphertext = cipher
        .encrypt(XNonce::from_slice(&nonce_bytes), plaintext.as_bytes())
        .expect("encryption cannot fail with valid inputs");
    let mut blob = Vec::with_capacity(24 + ciphertext.len());
    blob.extend_from_slice(&nonce_bytes);
    blob.extend_from_slice(&ciphertext);
    format!(
        "{PREFIX}{}",
        base64::engine::general_purpose::STANDARD.encode(blob)
    )
}

fn decrypt_with_key(stored: &str, key: &[u8; 32]) -> Result<String, String> {
    let blob = base64::engine::general_purpose::STANDARD
        .decode(stored.trim_start_matches(PREFIX))
        .map_err(|e| format!("Encrypted value is not valid base64: {e}"))?;
    if blob.len() < 24 + 16 {
        return Err("Encrypted value is truncated".to_string());
    }
    let cipher = XChaCha20Poly1305::new_from_slice(key).expect("32-byte key");
    let plaintext = cipher
        .decrypt(XNonce::from_slice(&blob[..24]), &blob[24..])
        .map_err(|_| "Decryption failed (wrong key or tampered data)".to_string())?;
    String::from_utf8(plaintext).map_err(|e| format!("Decrypted bytes are not UTF-8: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_key() -> [u8; 32] {
        *b"0123456789abcdef0123456789abcdef"
    }

    #[test]
    fn roundtrip() {
        let key = test_key();
        let enc = encrypt_with_key("hello secret world", &key);
        assert!(is_encrypted(&enc));
        assert_ne!(enc, "hello secret world");
        assert_eq!(decrypt_with_key(&enc, &key).unwrap(), "hello secret world");
    }

    #[test]
    fn nonces_differ_per_message() {
        let key = test_key();
        let a = encrypt_with_key("same", &key);
        let b = encrypt_with_key("same", &key);
        assert_ne!(a, b);
        assert_eq!(decrypt_with_key(&a, &key).unwrap(), "same");
    }

    #[test]
    fn wrong_key_fails() {
        let enc = encrypt_with_key("data", &test_key());
        assert!(decrypt_with_key(&enc, &[9u8; 32]).is_err());
    }

    #[test]
    fn tampered_ciphertext_fails() {
        let key = test_key();
        let mut enc = encrypt_with_key("data", &key);
        enc.pop();
        enc.push('A');
        assert!(decrypt_with_key(&enc, &key).is_err());
    }

    #[test]
    fn unicode_roundtrip() {
        let key = test_key();
        let text = "contraseña ñ 中文 🎨";
        assert_eq!(
            decrypt_with_key(&encrypt_with_key(text, &key), &key).unwrap(),
            text
        );
    }
}
