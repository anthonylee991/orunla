use crate::forgetting::strength::calculate_strength;
use crate::graph::GraphStore;
use crate::retriever::{Memory, RecallRequest, RecallResponse, Retriever};
use anyhow::Result;
use chrono::Utc;

pub struct HybridRetriever<'a> {
    store: &'a dyn GraphStore,
}

impl<'a> HybridRetriever<'a> {
    pub fn new(store: &'a dyn GraphStore) -> Self {
        Self { store }
    }
}

impl<'a> Retriever for HybridRetriever<'a> {
    fn recall(&self, request: RecallRequest) -> Result<RecallResponse> {
        let edges = self.store.search_edges(&request.query, request.limit)?;
        let now = Utc::now();

        let mut memories = Vec::new();
        for edge in edges {
            let strength = calculate_strength(&edge, now);
            
            if edge.confidence >= request.min_confidence && strength >= request.min_strength {
                let subject_node = self.store.get_node(&edge.source)?.unwrap_or_else(|| {
                    crate::graph::Node::new("Unknown".to_string(), crate::graph::NodeType::Unknown)
                });
                let object_node = self.store.get_node(&edge.target)?.unwrap_or_else(|| {
                    crate::graph::Node::new("Unknown".to_string(), crate::graph::NodeType::Unknown)
                });

                memories.push(Memory {
                    edge,
                    subject_node,
                    object_node,
                    relevance_score: strength,
                });
            }
        }

        // Sort by strength score descending
        memories.sort_by(|a, b| b.relevance_score.partial_cmp(&a.relevance_score).unwrap());

        Ok(RecallResponse { memories })
    }
}
