use crate::graph::{Edge, Node};
use anyhow::Result;

pub mod search;

pub struct RecallRequest {
    pub query: String,
    pub limit: usize,
    pub min_confidence: f32,
    pub min_strength: f32,
}

pub struct Memory {
    pub edge: Edge,
    pub subject_node: Node,
    pub object_node: Node,
    pub relevance_score: f32,
}

pub struct RecallResponse {
    pub memories: Vec<Memory>,
}

pub trait Retriever {
    fn recall(&self, request: RecallRequest) -> Result<RecallResponse>;
}
