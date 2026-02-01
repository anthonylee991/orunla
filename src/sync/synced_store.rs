use crate::graph::{Edge, EdgeId, GraphStore, Node, NodeId, SubGraph};
use crate::sync::changelog::{ChangeEventType, ChangelogStore};
use anyhow::Result;

/// Wrapper around any GraphStore + ChangelogStore that intercepts write mutations
/// and logs them to the changelog for sync. Read operations pass through unchanged.
///
/// Only used when the tier allows sync (Trial/Pro). Free tier uses raw SqliteStorage
/// with zero overhead.
pub struct SyncedGraphStore<S> {
    inner: S,
    device_id: String,
}

impl<S> SyncedGraphStore<S>
where
    S: GraphStore + ChangelogStore,
{
    pub fn new(inner: S, device_id: String) -> Self {
        Self { inner, device_id }
    }

    /// Get a reference to the inner store (for operations that don't need sync wrapping).
    pub fn inner(&self) -> &S {
        &self.inner
    }

    /// Get a mutable reference to the inner store.
    pub fn inner_mut(&mut self) -> &mut S {
        &mut self.inner
    }
}

impl<S> GraphStore for SyncedGraphStore<S>
where
    S: GraphStore + ChangelogStore,
{
    fn add_node(&mut self, node: Node) -> Result<NodeId> {
        let id = self.inner.add_node(node.clone())?;
        self.inner.log_change(
            &self.device_id,
            ChangeEventType::NodeAdd { node },
        )?;
        Ok(id)
    }

    fn get_node(&self, id: &NodeId) -> Result<Option<Node>> {
        self.inner.get_node(id)
    }

    fn find_node_by_label(&self, label: &str) -> Result<Option<Node>> {
        self.inner.find_node_by_label(label)
    }

    fn resolve_entity(&self, text: &str) -> Result<Option<NodeId>> {
        self.inner.resolve_entity(text)
    }

    fn list_nodes(&self) -> Result<Vec<Node>> {
        self.inner.list_nodes()
    }

    fn add_edge(&mut self, edge: Edge) -> Result<EdgeId> {
        let id = self.inner.add_edge(edge.clone())?;
        self.inner.log_change(
            &self.device_id,
            ChangeEventType::EdgeAdd { edge },
        )?;
        Ok(id)
    }

    fn get_edges_from(&self, node_id: &NodeId) -> Result<Vec<Edge>> {
        self.inner.get_edges_from(node_id)
    }

    fn get_edges_to(&self, node_id: &NodeId) -> Result<Vec<Edge>> {
        self.inner.get_edges_to(node_id)
    }

    // touch_edge is NOT logged -- access_count/last_accessed are local recall stats
    fn touch_edge(&mut self, edge_id: &EdgeId) -> Result<()> {
        self.inner.touch_edge(edge_id)
    }

    fn neighborhood(&self, node_id: &NodeId, depth: u32) -> Result<SubGraph> {
        self.inner.neighborhood(node_id, depth)
    }

    fn shortest_path(&self, from: &NodeId, to: &NodeId) -> Result<Option<Vec<EdgeId>>> {
        self.inner.shortest_path(from, to)
    }

    fn search_edges(&self, query: &str, limit: usize) -> Result<Vec<Edge>> {
        self.inner.search_edges(query, limit)
    }

    fn delete_edge(&mut self, id: &EdgeId) -> Result<()> {
        self.inner.delete_edge(id)?;
        self.inner.log_change(
            &self.device_id,
            ChangeEventType::EdgeDelete {
                edge_id: id.clone(),
            },
        )?;
        Ok(())
    }

    fn delete_edges_by_query(&mut self, query: &str) -> Result<usize> {
        // Find matching edges first so we can log individual deletes
        let matching = self.inner.search_edges(query, 1000)?;
        let count = matching.len();
        for edge in matching {
            self.delete_edge(&edge.id)?;
        }
        Ok(count)
    }

    fn cleanup_orphaned_nodes(&mut self) -> Result<usize> {
        // Orphan cleanup is a local maintenance operation, not synced
        self.inner.cleanup_orphaned_nodes()
    }

    fn hard_gc(&mut self, threshold: f32) -> Result<usize> {
        // GC deletes are synced so other devices also forget
        let now = chrono::Utc::now();
        let mut edges_to_delete = Vec::new();

        // Collect edges below threshold
        {
            let all_edges = self.inner.search_edges("", 100_000)?;
            for edge in all_edges {
                let strength =
                    crate::forgetting::strength::calculate_strength(&edge, now);
                if strength < threshold {
                    edges_to_delete.push(edge.id);
                }
            }
        }

        let count = edges_to_delete.len();
        for id in edges_to_delete {
            self.delete_edge(&id)?;
        }
        Ok(count)
    }

    fn merge_nodes(&mut self, winner_id: &NodeId, loser_id: &NodeId) -> Result<()> {
        self.inner.merge_nodes(winner_id, loser_id)?;
        self.inner.log_change(
            &self.device_id,
            ChangeEventType::NodeMerge {
                winner_id: winner_id.clone(),
                loser_id: loser_id.clone(),
            },
        )?;
        Ok(())
    }
}

/// Also expose ChangelogStore methods through the wrapper
impl<S> ChangelogStore for SyncedGraphStore<S>
where
    S: GraphStore + ChangelogStore,
{
    fn init_changelog(&self) -> Result<()> {
        self.inner.init_changelog()
    }

    fn log_change(&self, device_id: &str, event_type: ChangeEventType) -> Result<i64> {
        self.inner.log_change(device_id, event_type)
    }

    fn get_unsynced_events(&self) -> Result<Vec<crate::sync::changelog::ChangeEvent>> {
        self.inner.get_unsynced_events()
    }

    fn mark_synced(&self, event_ids: &[String]) -> Result<()> {
        self.inner.mark_synced(event_ids)
    }

    fn get_latest_vector_clock(&self) -> Result<i64> {
        self.inner.get_latest_vector_clock()
    }

    fn get_last_pull_clock(&self) -> Result<i64> {
        self.inner.get_last_pull_clock()
    }

    fn set_last_pull_clock(&self, clock: i64) -> Result<()> {
        self.inner.set_last_pull_clock(clock)
    }

    fn get_device_id(&self) -> Result<String> {
        self.inner.get_device_id()
    }

    fn apply_remote_event(&mut self, event: crate::sync::changelog::ChangeEvent) -> Result<()> {
        self.inner.apply_remote_event(event)
    }
}
