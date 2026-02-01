use crate::graph::{Edge, EdgeId, Node, NodeId};
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Types of mutations that get recorded in the changelog for sync.
/// Note: touch_edge (access_count/last_accessed) is intentionally NOT synced --
/// those are local recall stats, not shared state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChangeEventType {
    NodeAdd { node: Node },
    EdgeAdd { edge: Edge },
    EdgeDelete { edge_id: EdgeId },
    NodeMerge { winner_id: NodeId, loser_id: NodeId },
}

/// A single changelog entry representing a mutation to the knowledge graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeEvent {
    pub id: String,
    pub event_type: ChangeEventType,
    pub entity_id: String,
    pub vector_clock: i64,
    pub device_id: String,
    pub created_at: DateTime<Utc>,
    pub synced: bool,
}

/// Trait for storing and retrieving changelog events.
/// Implemented by SqliteStorage to keep changelog in the same memory.db.
pub trait ChangelogStore {
    /// Initialize changelog tables (called during storage init).
    fn init_changelog(&self) -> Result<()>;

    /// Log a new change event. Returns the assigned vector_clock value.
    fn log_change(&self, device_id: &str, event_type: ChangeEventType) -> Result<i64>;

    /// Get all unsynced events (synced = 0), ordered by vector_clock.
    fn get_unsynced_events(&self) -> Result<Vec<ChangeEvent>>;

    /// Mark events as synced by their IDs.
    fn mark_synced(&self, event_ids: &[String]) -> Result<()>;

    /// Get the latest vector_clock value for this device.
    fn get_latest_vector_clock(&self) -> Result<i64>;

    /// Get the last pull clock (cursor for pulling remote events).
    fn get_last_pull_clock(&self) -> Result<i64>;

    /// Update the last pull clock after pulling remote events.
    fn set_last_pull_clock(&self, clock: i64) -> Result<()>;

    /// Get or generate the device_id for this installation.
    fn get_device_id(&self) -> Result<String>;

    /// Apply a remote change event to the local graph.
    /// Handles dedup (skip if entity already exists) and tombstones (delete wins).
    fn apply_remote_event(&mut self, event: ChangeEvent) -> Result<()>;
}
