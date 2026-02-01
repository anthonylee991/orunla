use aes_gcm::aead::{Aead, KeyInit, OsRng};
use aes_gcm::{AeadCore, Aes256Gcm, Nonce};
use anyhow::{Context, Result};
use pbkdf2::pbkdf2_hmac;
use sha2::Sha256;

const SYNC_SALT: &[u8] = b"orunla-sync-salt-v1";
const PBKDF2_ITERATIONS: u32 = 100_000;
const KEY_LEN: usize = 32;

/// Handles client-side encryption of sync payloads.
/// Key is derived deterministically from the license key UUID,
/// so all devices with the same license can decrypt each other's data.
/// Railway relay never sees plaintext -- zero-knowledge sync.
pub struct SyncEncryption {
    cipher: Aes256Gcm,
}

impl SyncEncryption {
    /// Create a new SyncEncryption from a license key.
    /// The same license key on any device produces the same AES key.
    pub fn new(license_key: &str) -> Result<Self> {
        let mut key = [0u8; KEY_LEN];
        pbkdf2_hmac::<Sha256>(license_key.as_bytes(), SYNC_SALT, PBKDF2_ITERATIONS, &mut key);

        let cipher = Aes256Gcm::new(&key.into());
        Ok(Self { cipher })
    }

    /// Encrypt a payload. Returns nonce + ciphertext concatenated.
    pub fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>> {
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        let ciphertext = self
            .cipher
            .encrypt(&nonce, plaintext)
            .map_err(|e| anyhow::anyhow!("Sync encryption failed: {}", e))?;

        // Prepend 12-byte nonce to ciphertext
        let mut result = Vec::with_capacity(12 + ciphertext.len());
        result.extend_from_slice(nonce.as_slice());
        result.extend_from_slice(&ciphertext);
        Ok(result)
    }

    /// Decrypt a payload (expects nonce + ciphertext concatenated).
    pub fn decrypt(&self, encrypted: &[u8]) -> Result<Vec<u8>> {
        if encrypted.len() < 12 {
            anyhow::bail!("Encrypted payload too short");
        }
        let (nonce_bytes, ciphertext) = encrypted.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);

        self.cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| anyhow::anyhow!("Sync decryption failed: {}", e))
    }
}
