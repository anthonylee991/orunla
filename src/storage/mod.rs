use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub mod sqlite;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub path: PathBuf,
    pub backup_enabled: bool,
    pub backup_interval_hours: u32,
}

impl Default for StorageConfig {
    fn default() -> Self {
        let mut path = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push(".orunla");
        path.push("memory.db");
        Self {
            path,
            backup_enabled: true,
            backup_interval_hours: 24,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageStats {
    pub node_count: usize,
    pub edge_count: usize,
    pub db_size_bytes: u64,
    pub oldest_memory: Option<DateTime<Utc>>,
    pub newest_memory: Option<DateTime<Utc>>,
}

pub trait Storage {
    fn init(&self) -> Result<()>;
    fn stats(&self) -> Result<StorageStats>;
}

/// Application configuration stored in ~/.orunla/config.json
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfig {
    #[serde(default)]
    pub api_key: Option<String>,
}

impl AppConfig {
    /// Load config from ~/.orunla/config.json, returning defaults if missing.
    pub fn load() -> Self {
        let path = Self::config_path();
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(config) = serde_json::from_str(&content) {
                    return config;
                }
            }
        }
        Self::default()
    }

    /// Save config to ~/.orunla/config.json.
    pub fn save(&self) -> Result<()> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    fn config_path() -> PathBuf {
        let mut path = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push(".orunla");
        path.push("config.json");
        path
    }
}
