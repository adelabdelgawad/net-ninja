//! Password and sensitive data encryption module
//!
//! Uses AES-256-GCM for authenticated encryption with the format:
//! `$ENC$v1$<base64_nonce>$<base64_ciphertext>`
//!
//! The encryption key is loaded from the `NETNINJA_ENCRYPTION_KEY` environment variable
//! which should contain a base64-encoded 32-byte key.

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Key, Nonce,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use rand::RngCore;

use crate::errors::AppError;

/// Encryption format prefix
const ENCRYPTED_PREFIX: &str = "$ENC$";
/// Current encryption version
const ENCRYPTION_VERSION: &str = "v1";
/// Nonce size in bytes (96 bits for AES-GCM)
const NONCE_SIZE: usize = 12;
/// Key size in bytes (256 bits for AES-256)
const KEY_SIZE: usize = 32;

/// Wrapper around an AES-256-GCM encryption key
#[derive(Clone)]
pub struct EncryptionKey {
    key: Key<Aes256Gcm>,
}

impl EncryptionKey {
    /// Create a new encryption key from raw bytes
    ///
    /// # Arguments
    /// * `bytes` - A 32-byte slice containing the key material
    ///
    /// # Returns
    /// * `Some(EncryptionKey)` if the bytes are exactly 32 bytes
    /// * `None` if the byte slice is not 32 bytes
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != KEY_SIZE {
            return None;
        }
        let key = Key::<Aes256Gcm>::from_slice(bytes);
        Some(Self { key: *key })
    }

    /// Get the underlying key for use with AES-GCM
    fn as_key(&self) -> &Key<Aes256Gcm> {
        &self.key
    }
}

/// Load the encryption key from the `NETNINJA_ENCRYPTION_KEY` environment variable
///
/// The environment variable should contain a base64-encoded 32-byte key.
///
/// # Returns
/// * `Some(EncryptionKey)` if the key is found and valid
/// * `None` if the environment variable is not set or the key is invalid
pub fn load_encryption_key() -> Option<EncryptionKey> {
    // First check if the env var is already set (e.g., from dotenv or system environment)
    if let Ok(key_b64) = std::env::var("NETNINJA_ENCRYPTION_KEY") {
        if let Some(key) = parse_encryption_key(&key_b64) {
            return Some(key);
        }
    }

    // Try loading .env from known paths (Tauri CWD may differ from rust-backend dir)
    let candidate_paths = [
        // Compile-time path: rust-backend directory where this crate lives
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(".env"),
        std::path::PathBuf::from("../../src/rust-backend/.env"), // from src/frontend/src-tauri
        std::path::PathBuf::from("../rust-backend/.env"),        // from src/frontend
        std::path::PathBuf::from("src/rust-backend/.env"),       // from project root
        std::path::PathBuf::from(".env"),                        // CWD
    ];

    for path in &candidate_paths {
        if path.exists() {
            tracing::debug!("Found .env at: {}", path.display());
            if dotenvy::from_path(path).is_ok() {
                if let Ok(key_b64) = std::env::var("NETNINJA_ENCRYPTION_KEY") {
                    if let Some(key) = parse_encryption_key(&key_b64) {
                        tracing::info!("Encryption key loaded from: {}", path.display());
                        return Some(key);
                    }
                }
            }
        }
    }
    tracing::warn!("No .env found in any candidate path. Tried: {:?}", candidate_paths.iter().map(|p| p.display().to_string()).collect::<Vec<_>>());

    None
}

fn parse_encryption_key(key_b64: &str) -> Option<EncryptionKey> {
    let key_bytes = BASE64.decode(key_b64.trim()).ok()?;
    EncryptionKey::from_bytes(&key_bytes)
}

/// Encrypt plaintext using AES-256-GCM
///
/// # Arguments
/// * `plaintext` - The string to encrypt
/// * `key` - The encryption key
///
/// # Returns
/// * `Ok(String)` - The encrypted string in format `$ENC$v1$<base64_nonce>$<base64_ciphertext>`
/// * `Err(AppError::Encryption)` - If encryption fails
pub fn encrypt(plaintext: &str, key: &EncryptionKey) -> Result<String, AppError> {
    let cipher = Aes256Gcm::new(key.as_key());

    // Generate random nonce
    let mut nonce_bytes = [0u8; NONCE_SIZE];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    // Encrypt (ciphertext includes auth tag automatically)
    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_bytes())
        .map_err(|e| AppError::Encryption(format!("Encryption failed: {}", e)))?;

    // Format: $ENC$v1$<base64_nonce>$<base64_ciphertext>
    let nonce_b64 = BASE64.encode(nonce_bytes);
    let ciphertext_b64 = BASE64.encode(ciphertext);

    Ok(format!(
        "{}{}${}${}",
        ENCRYPTED_PREFIX, ENCRYPTION_VERSION, nonce_b64, ciphertext_b64
    ))
}

/// Decrypt ciphertext that was encrypted with the `encrypt` function
///
/// # Arguments
/// * `ciphertext` - The encrypted string in format `$ENC$v1$<base64_nonce>$<base64_ciphertext>`
/// * `key` - The encryption key
///
/// # Returns
/// * `Ok(String)` - The decrypted plaintext
/// * `Err(AppError::DecryptionFailed)` - If decryption fails (wrong key, corrupted data, etc.)
/// * `Err(AppError::Encryption)` - If the ciphertext format is invalid
pub fn decrypt(ciphertext: &str, key: &EncryptionKey) -> Result<String, AppError> {
    // Parse format: $ENC$v1$<base64_nonce>$<base64_ciphertext>
    let stripped = ciphertext.strip_prefix(ENCRYPTED_PREFIX).ok_or_else(|| {
        AppError::Encryption("Invalid encrypted format: missing prefix".to_string())
    })?;

    let parts: Vec<&str> = stripped.splitn(3, '$').collect();
    if parts.len() != 3 {
        return Err(AppError::Encryption(
            "Invalid encrypted format: expected version$nonce$ciphertext".to_string(),
        ));
    }

    let version = parts[0];
    let nonce_b64 = parts[1];
    let ciphertext_b64 = parts[2];

    if version != ENCRYPTION_VERSION {
        return Err(AppError::Encryption(format!(
            "Unsupported encryption version: {}",
            version
        )));
    }

    // Decode base64
    let nonce_bytes = BASE64
        .decode(nonce_b64)
        .map_err(|e| AppError::Encryption(format!("Invalid nonce encoding: {}", e)))?;

    let ciphertext_bytes = BASE64
        .decode(ciphertext_b64)
        .map_err(|e| AppError::Encryption(format!("Invalid ciphertext encoding: {}", e)))?;

    if nonce_bytes.len() != NONCE_SIZE {
        return Err(AppError::Encryption(format!(
            "Invalid nonce size: expected {} bytes, got {}",
            NONCE_SIZE,
            nonce_bytes.len()
        )));
    }

    let nonce = Nonce::from_slice(&nonce_bytes);
    let cipher = Aes256Gcm::new(key.as_key());

    // Decrypt (also verifies auth tag)
    let plaintext_bytes = cipher
        .decrypt(nonce, ciphertext_bytes.as_slice())
        .map_err(|_| AppError::DecryptionFailed)?;

    String::from_utf8(plaintext_bytes)
        .map_err(|e| AppError::Encryption(format!("Decrypted data is not valid UTF-8: {}", e)))
}

/// Check if a value is encrypted (has the `$ENC$` prefix)
///
/// # Arguments
/// * `value` - The string to check
///
/// # Returns
/// * `true` if the value starts with `$ENC$`
/// * `false` otherwise
pub fn is_encrypted(value: &str) -> bool {
    value.starts_with(ENCRYPTED_PREFIX)
}

/// Encrypt plaintext if an encryption key is available, otherwise return plaintext unchanged
///
/// This is useful for optional encryption where you want to encrypt if possible
/// but still function without an encryption key configured.
///
/// # Arguments
/// * `plaintext` - The string to potentially encrypt
/// * `key` - Optional encryption key
///
/// # Returns
/// * The encrypted string if a key is provided and encryption succeeds
/// * The original plaintext if no key is provided or encryption fails
pub fn maybe_encrypt(plaintext: &str, key: Option<&EncryptionKey>) -> String {
    match key {
        Some(k) => encrypt(plaintext, k).unwrap_or_else(|_| plaintext.to_string()),
        None => plaintext.to_string(),
    }
}

/// Decrypt value if it's encrypted, otherwise return it unchanged
///
/// # Arguments
/// * `value` - The string to potentially decrypt
/// * `key` - Optional encryption key
///
/// # Returns
/// * `Ok(String)` - The decrypted plaintext, or the original value if not encrypted
/// * `Err(AppError::EncryptionKeyRequired)` - If the value is encrypted but no key is provided
/// * `Err(AppError::DecryptionFailed)` - If decryption fails
pub fn maybe_decrypt(value: &str, key: Option<&EncryptionKey>) -> Result<String, AppError> {
    if !is_encrypted(value) {
        return Ok(value.to_string());
    }

    match key {
        Some(k) => decrypt(value, k),
        None => Err(AppError::EncryptionKeyRequired),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_key() -> EncryptionKey {
        // Test key: 32 bytes of zeros (DO NOT use in production!)
        EncryptionKey::from_bytes(&[0u8; 32]).unwrap()
    }

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let key = test_key();
        let plaintext = "my_secret_password";

        let encrypted = encrypt(plaintext, &key).unwrap();
        assert!(encrypted.starts_with("$ENC$v1$"));

        let decrypted = decrypt(&encrypted, &key).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_encrypt_produces_different_ciphertext() {
        let key = test_key();
        let plaintext = "same_password";

        let encrypted1 = encrypt(plaintext, &key).unwrap();
        let encrypted2 = encrypt(plaintext, &key).unwrap();

        // Due to random nonce, same plaintext should produce different ciphertext
        assert_ne!(encrypted1, encrypted2);

        // But both should decrypt to the same value
        assert_eq!(decrypt(&encrypted1, &key).unwrap(), plaintext);
        assert_eq!(decrypt(&encrypted2, &key).unwrap(), plaintext);
    }

    #[test]
    fn test_is_encrypted() {
        assert!(is_encrypted("$ENC$v1$abc$def"));
        assert!(is_encrypted("$ENC$v2$something"));
        assert!(!is_encrypted("plaintext"));
        assert!(!is_encrypted(""));
        assert!(!is_encrypted("$NOTENC$v1$abc$def"));
    }

    #[test]
    fn test_maybe_encrypt_with_key() {
        let key = test_key();
        let plaintext = "secret";

        let result = maybe_encrypt(plaintext, Some(&key));
        assert!(is_encrypted(&result));
    }

    #[test]
    fn test_maybe_encrypt_without_key() {
        let plaintext = "secret";

        let result = maybe_encrypt(plaintext, None);
        assert_eq!(result, plaintext);
        assert!(!is_encrypted(&result));
    }

    #[test]
    fn test_maybe_decrypt_encrypted_with_key() {
        let key = test_key();
        let plaintext = "secret";

        let encrypted = encrypt(plaintext, &key).unwrap();
        let result = maybe_decrypt(&encrypted, Some(&key)).unwrap();
        assert_eq!(result, plaintext);
    }

    #[test]
    fn test_maybe_decrypt_plaintext() {
        let key = test_key();
        let plaintext = "not_encrypted";

        let result = maybe_decrypt(plaintext, Some(&key)).unwrap();
        assert_eq!(result, plaintext);
    }

    #[test]
    fn test_maybe_decrypt_encrypted_without_key() {
        let key = test_key();
        let plaintext = "secret";

        let encrypted = encrypt(plaintext, &key).unwrap();
        let result = maybe_decrypt(&encrypted, None);

        assert!(matches!(result, Err(AppError::EncryptionKeyRequired)));
    }

    #[test]
    fn test_decrypt_wrong_key() {
        let key1 = test_key();
        let key2 = EncryptionKey::from_bytes(&[1u8; 32]).unwrap();
        let plaintext = "secret";

        let encrypted = encrypt(plaintext, &key1).unwrap();
        let result = decrypt(&encrypted, &key2);

        assert!(matches!(result, Err(AppError::DecryptionFailed)));
    }

    #[test]
    fn test_decrypt_invalid_format() {
        let key = test_key();

        assert!(matches!(
            decrypt("not_encrypted", &key),
            Err(AppError::Encryption(_))
        ));

        assert!(matches!(
            decrypt("$ENC$v99$abc$def", &key),
            Err(AppError::Encryption(_))
        ));

        assert!(matches!(
            decrypt("$ENC$v1$invalid", &key),
            Err(AppError::Encryption(_))
        ));
    }

    #[test]
    fn test_encryption_key_from_bytes() {
        assert!(EncryptionKey::from_bytes(&[0u8; 32]).is_some());
        assert!(EncryptionKey::from_bytes(&[0u8; 16]).is_none());
        assert!(EncryptionKey::from_bytes(&[0u8; 64]).is_none());
        assert!(EncryptionKey::from_bytes(&[]).is_none());
    }

    #[test]
    fn test_encrypt_empty_string() {
        let key = test_key();
        let plaintext = "";

        let encrypted = encrypt(plaintext, &key).unwrap();
        let decrypted = decrypt(&encrypted, &key).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_encrypt_unicode() {
        let key = test_key();
        let plaintext = "password with unicode: \u{1F512}\u{1F511} and special chars: !@#$%^&*()";

        let encrypted = encrypt(plaintext, &key).unwrap();
        let decrypted = decrypt(&encrypted, &key).unwrap();
        assert_eq!(decrypted, plaintext);
    }
}
