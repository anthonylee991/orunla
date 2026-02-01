use crate::licensing::tier::Tier;
use aes_gcm::aead::{Aead, KeyInit, OsRng};
use aes_gcm::{AeadCore, Aes256Gcm, Nonce};
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use pbkdf2::pbkdf2_hmac;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::path::PathBuf;

const APP_SECRET: &[u8] = b"orunla-desktop-v1-2026";
const PBKDF2_ITERATIONS: u32 = 100_000;
const KEY_LEN: usize = 32;

/// License data stored encrypted in local SQLite
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct License {
    pub license_key: String,
    pub tier: Tier,
    pub trial_start: Option<DateTime<Utc>>,
    pub last_validated: DateTime<Utc>,
}

/// Manages encrypted license storage in the Orunla SQLite database
pub struct LicenseStore {
    db_path: PathBuf,
}

impl LicenseStore {
    pub fn new(db_path: PathBuf) -> Self {
        Self { db_path }
    }

    /// Initialize the license table in SQLite
    pub fn init(&self) -> Result<()> {
        let conn = self.get_connection()?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS license (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                encrypted_data TEXT NOT NULL,
                nonce TEXT NOT NULL,
                salt TEXT NOT NULL
            );",
        )
        .context("Failed to create license table")?;
        Ok(())
    }

    /// Save license data (encrypted with AES-256-GCM)
    pub fn save(&self, license: &License) -> Result<()> {
        let json = serde_json::to_vec(license).context("Failed to serialize license")?;

        // Generate random salt for key derivation
        let mut salt = [0u8; 16];
        aes_gcm::aead::rand_core::RngCore::fill_bytes(&mut OsRng, &mut salt);

        let key = derive_key(&salt);
        let cipher = Aes256Gcm::new(&key.into());

        // Generate random nonce
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        let ciphertext = cipher
            .encrypt(&nonce, json.as_ref())
            .map_err(|e| anyhow::anyhow!("Encryption failed: {}", e))?;

        let conn = self.get_connection()?;
        conn.execute(
            "INSERT OR REPLACE INTO license (id, encrypted_data, nonce, salt) VALUES (1, ?1, ?2, ?3)",
            (
                base64_encode(&ciphertext),
                base64_encode(nonce.as_slice()),
                base64_encode(&salt),
            ),
        )
        .context("Failed to save license")?;

        Ok(())
    }

    /// Load and decrypt license data
    pub fn load(&self) -> Result<Option<License>> {
        let conn = self.get_connection()?;

        let result: std::result::Result<(String, String, String), _> = conn.query_row(
            "SELECT encrypted_data, nonce, salt FROM license WHERE id = 1",
            (),
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        );

        match result {
            Ok((encrypted_b64, nonce_b64, salt_b64)) => {
                let ciphertext = base64_decode(&encrypted_b64)?;
                let nonce_bytes = base64_decode(&nonce_b64)?;
                let salt = base64_decode(&salt_b64)?;

                let key = derive_key(&salt);
                let cipher = Aes256Gcm::new(&key.into());
                let nonce = Nonce::from_slice(&nonce_bytes);

                let plaintext = cipher
                    .decrypt(nonce, ciphertext.as_ref())
                    .map_err(|e| anyhow::anyhow!("Decryption failed: {}", e))?;

                let license: License = serde_json::from_slice(&plaintext)
                    .context("Failed to deserialize license")?;
                Ok(Some(license))
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(anyhow::Error::from(e)),
        }
    }

    /// Ensure a license row exists. If not, create a trial license.
    pub fn ensure_license(&self) -> Result<License> {
        self.init()?;

        if let Some(license) = self.load()? {
            return Ok(license);
        }

        // First run: start trial
        let license = License {
            license_key: String::new(),
            tier: Tier::Trial,
            trial_start: Some(Utc::now()),
            last_validated: Utc::now(),
        };
        self.save(&license)?;
        Ok(license)
    }

    fn get_connection(&self) -> Result<Connection> {
        Connection::open(&self.db_path).context("Failed to open license database")
    }
}

/// Derive a 256-bit AES key from the app secret + salt using PBKDF2-SHA256
fn derive_key(salt: &[u8]) -> [u8; KEY_LEN] {
    let mut key = [0u8; KEY_LEN];
    pbkdf2_hmac::<Sha256>(APP_SECRET, salt, PBKDF2_ITERATIONS, &mut key);
    key
}

fn base64_encode(data: &[u8]) -> String {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.encode(data)
}

fn base64_decode(data: &str) -> Result<Vec<u8>> {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD
        .decode(data)
        .context("Invalid base64")
}
