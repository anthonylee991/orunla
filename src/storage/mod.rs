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
