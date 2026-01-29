use crate::graph::NodeId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub type EdgeId = String;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    pub id: EdgeId,
    pub source: NodeId,
    pub target: NodeId,
    pub predicate: String,
    pub created_at: DateTime<Utc>,
    pub last_accessed: DateTime<Utc>,
    pub access_count: u32,
    pub source_text: String,
    pub source_id: Option<String>,
    pub confidence: f32,
}

impl Edge {
    pub fn new(source: NodeId, target: NodeId, predicate: String, source_text: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            source,
            target,
            predicate,
            created_at: now,
            last_accessed: now,
            access_count: 0,
            source_text,
            source_id: None,
            confidence: 1.0,
        }
    }
}
