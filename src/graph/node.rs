use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use uuid::Uuid;

pub type NodeId = String;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: NodeId,
    pub label: String,
    pub node_type: NodeType,
    pub created_at: DateTime<Utc>,
    pub metadata: HashMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum NodeType {
    Person,
    Project,
    Technology,
    Organization,
    Concept,
    Location,
    DateTime,
    Unknown,
}

impl Node {
    pub fn new(label: String, node_type: NodeType) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            label,
            node_type,
            created_at: Utc::now(),
            metadata: HashMap::new(),
        }
    }
}
