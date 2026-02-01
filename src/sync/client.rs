use crate::sync::changelog::ChangelogStore;
use crate::sync::encryption::SyncEncryption;
use anyhow::Result;

/// Relay URL -- same as licensing, separate service for sync endpoints.
const RELAY_URL: &str = "https://orunla-production.up.railway.app";

/// Configuration for the sync client.
#[derive(Debug, Clone)]
pub struct SyncConfig {
    pub relay_url: String,
    pub device_id: String,
    pub license_key: String,
    pub sync_interval_secs: u64,
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            relay_url: RELAY_URL.to_string(),
            device_id: String::new(),
            license_key: String::new(),
            sync_interval_secs: 30,
        }
    }
}

/// Sync client that pushes/pulls encrypted changelog events via the Railway relay.
pub struct SyncClient {
    config: SyncConfig,
    http: reqwest::Client,
    #[allow(dead_code)]
    encryption: SyncEncryption,
}

impl SyncClient {
    pub fn new(config: SyncConfig) -> Result<Self> {
        let encryption = SyncEncryption::new(&config.license_key)?;
        Ok(Self {
            config,
            http: reqwest::Client::new(),
            encryption,
        })
    }

    /// Register this device with the relay. Called once on first sync.
    pub async fn register_device(&self) -> Result<()> {
        let hostname = hostname::get()
            .map(|h| h.to_string_lossy().to_string())
            .unwrap_or_else(|_| "unknown".to_string());

        self.http
            .post(format!("{}/orunla/register", self.config.relay_url))
            .header("X-License-Key", &self.config.license_key)
            .json(&serde_json::json!({
                "device_id": self.config.device_id,
                "device_name": hostname,
            }))
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }

    /// Push unsynced local changes to the relay (encrypted).
    pub async fn push_changes<S: ChangelogStore>(&self, store: &S) -> Result<usize> {
        let events = store.get_unsynced_events()?;
        if events.is_empty() {
            return Ok(0);
        }

        let encrypted_events: Vec<serde_json::Value> = events
            .iter()
            .map(|e| {
                let json = serde_json::to_vec(e).unwrap();
                let encrypted = self.encryption.encrypt(&json).unwrap();
                serde_json::json!({
                    "id": e.id,
                    "payload": base64_encode(&encrypted),
                    "vector_clock": e.vector_clock,
                    "created_at": e.created_at.to_rfc3339(),
                })
            })
            .collect();

        self.http
            .post(format!("{}/orunla/push", self.config.relay_url))
            .header("X-Device-ID", &self.config.device_id)
            .header("X-License-Key", &self.config.license_key)
            .json(&serde_json::json!({
                "events": encrypted_events,
            }))
            .send()
            .await?
            .error_for_status()?;

        let ids: Vec<String> = events.iter().map(|e| e.id.clone()).collect();
        store.mark_synced(&ids)?;

        Ok(events.len())
    }

    /// Pull remote changes from other devices (encrypted).
    pub async fn pull_changes<S: ChangelogStore>(&self, store: &mut S) -> Result<usize> {
        let since = store.get_last_pull_clock()?;

        let response = self
            .http
            .get(format!("{}/orunla/pull", self.config.relay_url))
            .query(&[("since", since.to_string())])
            .header("X-Device-ID", &self.config.device_id)
            .header("X-License-Key", &self.config.license_key)
            .send()
            .await?
            .error_for_status()?;

        let data: serde_json::Value = response.json().await?;
        let events = data["events"].as_array();
        let Some(events) = events else {
            return Ok(0);
        };

        let mut count = 0;
        let mut max_clock: i64 = since;

        for event_json in events {
            let encrypted_b64 = event_json["payload"]
                .as_str()
                .ok_or_else(|| anyhow::anyhow!("Missing payload"))?;
            let encrypted = base64_decode(encrypted_b64)?;
            let decrypted = self.encryption.decrypt(&encrypted)?;
            let event: crate::sync::changelog::ChangeEvent = serde_json::from_slice(&decrypted)?;

            if event.vector_clock > max_clock {
                max_clock = event.vector_clock;
            }

            store.apply_remote_event(event)?;
            count += 1;
        }

        if count > 0 {
            store.set_last_pull_clock(max_clock)?;
        }

        Ok(count)
    }

    /// Run a single push + pull cycle. Used by `orunla sync` CLI command.
    pub async fn sync_once<S: ChangelogStore>(&self, store: &mut S) -> Result<(usize, usize)> {
        let pushed = self.push_changes(store).await?;
        let pulled = self.pull_changes(store).await?;
        Ok((pushed, pulled))
    }
}

fn base64_encode(data: &[u8]) -> String {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.encode(data)
}

fn base64_decode(data: &str) -> Result<Vec<u8>> {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD
        .decode(data)
        .map_err(|e| anyhow::anyhow!("Invalid base64: {}", e))
}
