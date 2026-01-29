use anyhow::Result;

pub mod edge;
pub mod node;

pub use edge::{Edge, EdgeId};
pub use node::{Node, NodeId, NodeType};

pub struct SubGraph {
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
}

pub trait GraphStore {
    // Node operations
    fn add_node(&mut self, node: Node) -> Result<NodeId>;
    fn get_node(&self, id: &NodeId) -> Result<Option<Node>>;
    fn find_node_by_label(&self, label: &str) -> Result<Option<Node>>;
    fn resolve_entity(&self, text: &str) -> Result<Option<NodeId>>;
    fn list_nodes(&self) -> Result<Vec<Node>>;

    // Edge operations
    fn add_edge(&mut self, edge: Edge) -> Result<EdgeId>;
    fn get_edges_from(&self, node_id: &NodeId) -> Result<Vec<Edge>>;
    fn get_edges_to(&self, node_id: &NodeId) -> Result<Vec<Edge>>;
    fn touch_edge(&mut self, edge_id: &EdgeId) -> Result<()>;

    // Graph traversal
    fn neighborhood(&self, node_id: &NodeId, depth: u32) -> Result<SubGraph>;
    fn shortest_path(&self, from: &NodeId, to: &NodeId) -> Result<Option<Vec<EdgeId>>>;

    // Search
    fn search_edges(&self, query: &str, limit: usize) -> Result<Vec<Edge>>;

    // Pruning
    fn delete_edge(&mut self, id: &EdgeId) -> Result<()>;
    fn delete_edges_by_query(&mut self, query: &str) -> Result<usize>;
    fn cleanup_orphaned_nodes(&mut self) -> Result<usize>;
    fn hard_gc(&mut self, threshold: f32) -> Result<usize>;

    // Maintenance
    fn merge_nodes(&mut self, winner_id: &NodeId, loser_id: &NodeId) -> Result<()>;
}
