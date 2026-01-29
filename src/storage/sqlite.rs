use crate::graph::{Edge, EdgeId, GraphStore, Node, NodeId, NodeType, SubGraph};
use crate::storage::{Storage, StorageConfig, StorageStats};
use anyhow::{Context, Result};
use rusqlite::Connection;
use serde_json::Value;
use std::collections::HashMap;
use std::fs;

pub struct SqliteStorage {
    config: StorageConfig,
}

impl SqliteStorage {
    pub fn new(config: StorageConfig) -> Self {
        Self { config }
    }

    fn get_connection(&self) -> Result<Connection> {
        let path = &self.config.path;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).context("Failed to create storage directory")?;
        }
        Connection::open(path).context("Failed to open SQLite connection")
    }
}

fn parse_date(s: &str) -> chrono::DateTime<chrono::Utc> {
    chrono::DateTime::parse_from_rfc3339(s)
        .map(|dt| dt.with_timezone(&chrono::Utc))
        .or_else(|_| {
            // Try ISO 8601 without 'Z'
            chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S%.f")
                .map(|dt| chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(dt, chrono::Utc))
        })
        .unwrap_or_else(|_| {
            // Fallback to Unix Epoch if all else fails, to avoid "now" which bypasses decay checks
            chrono::DateTime::<chrono::Utc>::UNIX_EPOCH
        })
}

impl Storage for SqliteStorage {
    fn init(&self) -> Result<()> {
        let conn = self.get_connection()?;

        conn.execute_batch(
            "BEGIN;
            CREATE TABLE IF NOT EXISTS nodes (
                id TEXT PRIMARY KEY,
                label TEXT NOT NULL,
                node_type TEXT NOT NULL,
                created_at TEXT NOT NULL,
                metadata TEXT
            );
            CREATE INDEX IF NOT EXISTS idx_nodes_label ON nodes(label);
            CREATE INDEX IF NOT EXISTS idx_nodes_type ON nodes(node_type);

            CREATE TABLE IF NOT EXISTS node_aliases (
                alias TEXT PRIMARY KEY,
                node_id TEXT NOT NULL REFERENCES nodes(id)
            );
            CREATE INDEX IF NOT EXISTS idx_aliases_node ON node_aliases(node_id);

            CREATE TABLE IF NOT EXISTS edges (
                id TEXT PRIMARY KEY,
                source_id TEXT NOT NULL REFERENCES nodes(id),
                target_id TEXT NOT NULL REFERENCES nodes(id),
                predicate TEXT NOT NULL,
                created_at TEXT NOT NULL,
                last_accessed TEXT NOT NULL,
                access_count INTEGER DEFAULT 0,
                source_text TEXT,
                ext_source_id TEXT,
                confidence REAL DEFAULT 1.0
            );
            CREATE INDEX IF NOT EXISTS idx_edges_source ON edges(source_id);
            CREATE INDEX IF NOT EXISTS idx_edges_target ON edges(target_id);
            CREATE INDEX IF NOT EXISTS idx_edges_predicate ON edges(predicate);
            CREATE INDEX IF NOT EXISTS idx_edges_accessed ON edges(last_accessed);
            CREATE INDEX IF NOT EXISTS idx_edges_count ON edges(access_count);

            CREATE VIRTUAL TABLE IF NOT EXISTS edges_fts USING fts5(
                source_text,
                content='edges',
                content_rowid='rowid'
            );
            COMMIT;",
        )
        .context("Failed to initialize database schema")?;

        Ok(())
    }

    fn stats(&self) -> Result<StorageStats> {
        let conn = self.get_connection()?;
        let node_count: usize = conn
            .query_row("SELECT COUNT(*) FROM nodes", (), |r| r.get(0))
            .context("Failed to count nodes")?;
        let edge_count: usize = conn
            .query_row("SELECT COUNT(*) FROM edges", (), |r| r.get(0))
            .context("Failed to count edges")?;

        let path = &self.config.path;
        let db_size_bytes = if path.exists() {
            fs::metadata(path)?.len()
        } else {
            0
        };

        Ok(StorageStats {
            node_count,
            edge_count,
            db_size_bytes,
            oldest_memory: None,
            newest_memory: None,
        })
    }
}

impl GraphStore for SqliteStorage {
    fn add_node(&mut self, node: Node) -> Result<NodeId> {
        let conn = self.get_connection()?;
        let metadata = serde_json::to_string(&node.metadata)?;
        let node_type_val = serde_json::to_value(&node.node_type)?;
        let node_type = node_type_val.as_str().unwrap_or("Unknown").to_string();

        conn.execute(
            "INSERT INTO nodes (id, label, node_type, created_at, metadata) VALUES (?1, ?2, ?3, ?4, ?5)",
            (&node.id, &node.label, &node_type, &node.created_at.to_rfc3339(), &metadata),
        ).context("Failed to insert node")?;

        Ok(node.id)
    }

    fn get_node(&self, id: &NodeId) -> Result<Option<Node>> {
        let conn = self.get_connection()?;
        let mut stmt = conn.prepare(
            "SELECT id, label, node_type, created_at, metadata FROM nodes WHERE id = ?1",
        )?;
        let node = stmt.query_row([id], |row| {
            let node_type_str: String = row.get(2)?;
            let node_type: NodeType = serde_json::from_str(&format!("\"{}\"", node_type_str))
                .unwrap_or(NodeType::Unknown);
            let created_at_str: String = row.get(3)?;
            let metadata_str: String = row.get(4)?;
            let metadata: HashMap<String, Value> =
                serde_json::from_str(&metadata_str).unwrap_or_default();

            Ok(Node {
                id: row.get(0)?,
                label: row.get(1)?,
                node_type,
                created_at: parse_date(&created_at_str),
                metadata,
            })
        });

        match node {
            Ok(n) => Ok(Some(n)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(anyhow::Error::from(e)),
        }
    }

    fn find_node_by_label(&self, label: &str) -> Result<Option<Node>> {
        let conn = self.get_connection()?;
        let mut stmt = conn.prepare("SELECT id FROM nodes WHERE label = ?1")?;
        let id: Result<String, _> = stmt.query_row([label], |row| row.get(0));

        match id {
            Ok(id) => self.get_node(&id),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(anyhow::Error::from(e)),
        }
    }

    fn resolve_entity(&self, text: &str) -> Result<Option<NodeId>> {
        let conn = self.get_connection()?;
        // Check exact match label
        let mut stmt = conn.prepare("SELECT id FROM nodes WHERE label = ?1")?;
        let id: Result<String, _> = stmt.query_row([text], |row| row.get(0));
        if let Ok(id) = id {
            return Ok(Some(id));
        }

        // Check aliases
        let mut stmt = conn.prepare("SELECT node_id FROM node_aliases WHERE alias = ?1")?;
        let id: Result<String, _> = stmt.query_row([text], |row| row.get(0));
        match id {
            Ok(id) => Ok(Some(id)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(anyhow::Error::from(e)),
        }
    }

    fn list_nodes(&self) -> Result<Vec<Node>> {
        let conn = self.get_connection()?;
        let mut stmt = conn.prepare("SELECT id, label, node_type, created_at, metadata FROM nodes")?;
        let node_iter = stmt.query_map([], |row| {
            let node_type_str: String = row.get(2)?;
            let node_type: NodeType = serde_json::from_str(&format!("\"{}\"", node_type_str))
                .unwrap_or(NodeType::Unknown);
            let created_at_str: String = row.get(3)?;
            let metadata_str: String = row.get(4)?;
            let metadata: HashMap<String, Value> =
                serde_json::from_str(&metadata_str).unwrap_or_default();

            Ok(Node {
                id: row.get(0)?,
                label: row.get(1)?,
                node_type,
                created_at: chrono::DateTime::parse_from_rfc3339(&created_at_str)
                    .map(|dt| dt.with_timezone(&chrono::Utc))
                    .unwrap_or_else(|_| chrono::Utc::now()),
                metadata,
            })
        })?;

        let mut nodes = Vec::new();
        for node in node_iter {
            nodes.push(node?);
        }
        Ok(nodes)
    }

    fn add_edge(&mut self, edge: Edge) -> Result<EdgeId> {
        let conn = self.get_connection()?;
        conn.execute(
            "INSERT INTO edges (id, source_id, target_id, predicate, created_at, last_accessed, access_count, source_text, ext_source_id, confidence) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            (
                &edge.id, &edge.source, &edge.target, &edge.predicate, 
                &edge.created_at.to_rfc3339(), &edge.last_accessed.to_rfc3339(), 
                &edge.access_count, &edge.source_text, &edge.source_id, &edge.confidence
            ),
        ).context("Failed to insert edge")?;

        let row_id = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO edges_fts (rowid, source_text) VALUES (?1, ?2)",
            (row_id, &edge.source_text),
        )
        .context("Failed to update FTS index")?;

        Ok(edge.id)
    }

    fn get_edges_from(&self, node_id: &NodeId) -> Result<Vec<Edge>> {
        let conn = self.get_connection()?;
        let mut stmt = conn.prepare("SELECT id, source_id, target_id, predicate, created_at, last_accessed, access_count, source_text, source_id, confidence FROM edges WHERE source_id = ?1")?;
        let edge_iter = stmt.query_map([node_id], |row| {
            let created_at_str: String = row.get(4)?;
            let last_accessed_str: String = row.get(5)?;
            Ok(Edge {
                id: row.get(0)?,
                source: row.get(1)?,
                target: row.get(2)?,
                predicate: row.get(3)?,
                created_at: chrono::DateTime::parse_from_rfc3339(&created_at_str)
                    .map(|dt| dt.with_timezone(&chrono::Utc))
                    .unwrap_or_else(|_| chrono::Utc::now()),
                last_accessed: chrono::DateTime::parse_from_rfc3339(&last_accessed_str)
                    .map(|dt| dt.with_timezone(&chrono::Utc))
                    .unwrap_or_else(|_| chrono::Utc::now()),
                access_count: row.get(6)?,
                source_text: row.get(7)?,
                source_id: row.get(8)?,
                confidence: row.get(9)?,
            })
        })?;

        let mut edges = Vec::new();
        for edge in edge_iter {
            edges.push(edge?);
        }
        Ok(edges)
    }

    fn get_edges_to(&self, node_id: &NodeId) -> Result<Vec<Edge>> {
        let conn = self.get_connection()?;
        let mut stmt = conn.prepare("SELECT id, source_id, target_id, predicate, created_at, last_accessed, access_count, source_text, source_id, confidence FROM edges WHERE target_id = ?1")?;
        let edge_iter = stmt.query_map([node_id], |row| {
            let created_at_str: String = row.get(4)?;
            let last_accessed_str: String = row.get(5)?;
            Ok(Edge {
                id: row.get(0)?,
                source: row.get(1)?,
                target: row.get(2)?,
                predicate: row.get(3)?,
                created_at: chrono::DateTime::parse_from_rfc3339(&created_at_str)
                    .map(|dt| dt.with_timezone(&chrono::Utc))
                    .unwrap_or_else(|_| chrono::Utc::now()),
                last_accessed: chrono::DateTime::parse_from_rfc3339(&last_accessed_str)
                    .map(|dt| dt.with_timezone(&chrono::Utc))
                    .unwrap_or_else(|_| chrono::Utc::now()),
                access_count: row.get(6)?,
                source_text: row.get(7)?,
                source_id: row.get(8)?,
                confidence: row.get(9)?,
            })
        })?;

        let mut edges = Vec::new();
        for edge in edge_iter {
            edges.push(edge?);
        }
        Ok(edges)
    }

    fn touch_edge(&mut self, edge_id: &EdgeId) -> Result<()> {
        let conn = self.get_connection()?;
        conn.execute(
            "UPDATE edges SET last_accessed = ?1, access_count = access_count + 1 WHERE id = ?2",
            (chrono::Utc::now().to_rfc3339(), edge_id),
        )?;
        Ok(())
    }

    fn neighborhood(&self, node_id: &NodeId, _depth: u32) -> Result<SubGraph> {
        let edges_from = self.get_edges_from(node_id)?;
        let edges_to = self.get_edges_to(node_id)?;

        let mut nodes = Vec::new();
        if let Some(node) = self.get_node(node_id)? {
            nodes.push(node);
        }

        let mut edges = edges_from;
        edges.extend(edges_to);

        for edge in &edges {
            if edge.source != *node_id {
                if let Some(n) = self.get_node(&edge.source)? {
                    if !nodes.iter().any(|existing| existing.id == n.id) {
                        nodes.push(n);
                    }
                }
            }
            if edge.target != *node_id {
                if let Some(n) = self.get_node(&edge.target)? {
                    if !nodes.iter().any(|existing| existing.id == n.id) {
                        nodes.push(n);
                    }
                }
            }
        }

        Ok(SubGraph { nodes, edges })
    }

    fn shortest_path(&self, _from: &NodeId, _to: &NodeId) -> Result<Option<Vec<EdgeId>>> {
        Ok(None)
    }
    fn search_edges(&self, query: &str, limit: usize) -> Result<Vec<Edge>> {
        use crate::utils::query::{expand_query, build_fts_query};

        let conn = self.get_connection()?;

        if query.trim().is_empty() {
            let mut stmt = conn.prepare(
                "SELECT id, source_id, target_id, predicate, created_at, last_accessed, access_count, source_text, ext_source_id, confidence
                 FROM edges
                 ORDER BY created_at DESC
                 LIMIT ?1"
            )?;
            let edge_iter = stmt.query_map([limit.to_string()], |row| {
                let created_at_str: String = row.get(4)?;
                let last_accessed_str: String = row.get(5)?;
                let created_at = parse_date(&created_at_str);
                let last_accessed = parse_date(&last_accessed_str);
                Ok(Edge {
                    id: row.get(0)?,
                    source: row.get(1)?,
                    target: row.get(2)?,
                    predicate: row.get(3)?,
                    created_at,
                    last_accessed,
                    access_count: row.get(6)?,
                    source_text: row.get(7)?,
                    source_id: row.get(8)?,
                    confidence: row.get(9)?,
                })
            })?;
            let mut results = Vec::new();
            for res in edge_iter {
                results.push(res?);
            }
            Ok(results)
        } else {
            let mut results = HashMap::new();

            // Expand query with stemming and synonyms
            let expanded_terms = expand_query(query);

            if !expanded_terms.is_empty() {
                // 1. FTS Search Pass with expanded terms and prefix matching
                let fts_query = build_fts_query(&expanded_terms);
                let mut stmt = conn.prepare(
                    "SELECT e.id, e.source_id, e.target_id, e.predicate, e.created_at, e.last_accessed, e.access_count, e.source_text, e.ext_source_id, e.confidence
                     FROM edges e
                     JOIN edges_fts f ON e.rowid = f.rowid
                     WHERE f.source_text MATCH ?1
                     LIMIT ?2"
                )?;

                let fts_iter = stmt.query_map([&fts_query, &limit.to_string()], |row| {
                    let created_at_str: String = row.get(4)?;
                    let last_accessed_str: String = row.get(5)?;
                    Ok(Edge {
                        id: row.get(0)?,
                        source: row.get(1)?,
                        target: row.get(2)?,
                        predicate: row.get(3)?,
                        created_at: parse_date(&created_at_str),
                        last_accessed: parse_date(&last_accessed_str),
                        access_count: row.get(6)?,
                        source_text: row.get(7)?,
                        source_id: row.get(8)?,
                        confidence: row.get(9)?,
                    })
                });

                if let Ok(iter) = fts_iter {
                    for res in iter {
                        if let Ok(edge) = res {
                            results.insert(edge.id.clone(), edge);
                        }
                    }
                }

                // 2. LIKE Search Pass for predicates (check all expanded terms)
                if results.len() < limit {
                    for term in &expanded_terms {
                        if results.len() >= limit {
                            break;
                        }
                        let like_pattern = format!("%{}%", term);
                        let mut stmt = conn.prepare(
                            "SELECT e.id, e.source_id, e.target_id, e.predicate, e.created_at, e.last_accessed, e.access_count, e.source_text, e.ext_source_id, e.confidence
                             FROM edges e
                             WHERE e.predicate LIKE ?1
                             LIMIT ?2"
                        )?;

                        let iter = stmt.query_map([&like_pattern, &limit.to_string()], |row| {
                            let created_at_str: String = row.get(4)?;
                            let last_accessed_str: String = row.get(5)?;
                            Ok(Edge {
                                id: row.get(0)?,
                                source: row.get(1)?,
                                target: row.get(2)?,
                                predicate: row.get(3)?,
                                created_at: parse_date(&created_at_str),
                                last_accessed: parse_date(&last_accessed_str),
                                access_count: row.get(6)?,
                                source_text: row.get(7)?,
                                source_id: row.get(8)?,
                                confidence: row.get(9)?,
                            })
                        })?;
                        for res in iter {
                            if let Ok(edge) = res {
                                if !results.contains_key(&edge.id) {
                                    results.insert(edge.id.clone(), edge);
                                }
                            }
                        }
                    }
                }

                // 3. LIKE Search Pass for node labels (check all expanded terms)
                if results.len() < limit {
                    for term in &expanded_terms {
                        if results.len() >= limit {
                            break;
                        }
                        let like_pattern = format!("%{}%", term);
                        let mut stmt = conn.prepare(
                            "SELECT e.id, e.source_id, e.target_id, e.predicate, e.created_at, e.last_accessed, e.access_count, e.source_text, e.ext_source_id, e.confidence
                             FROM edges e
                             LEFT JOIN nodes ns ON e.source_id = ns.id
                             LEFT JOIN nodes nt ON e.target_id = nt.id
                             WHERE ns.label LIKE ?1 OR nt.label LIKE ?1
                             LIMIT ?2"
                        )?;

                        let iter = stmt.query_map([&like_pattern, &limit.to_string()], |row| {
                            let created_at_str: String = row.get(4)?;
                            let last_accessed_str: String = row.get(5)?;
                            Ok(Edge {
                                id: row.get(0)?,
                                source: row.get(1)?,
                                target: row.get(2)?,
                                predicate: row.get(3)?,
                                created_at: parse_date(&created_at_str),
                                last_accessed: parse_date(&last_accessed_str),
                                access_count: row.get(6)?,
                                source_text: row.get(7)?,
                                source_id: row.get(8)?,
                                confidence: row.get(9)?,
                            })
                        })?;
                        for res in iter {
                            if let Ok(edge) = res {
                                if !results.contains_key(&edge.id) {
                                    results.insert(edge.id.clone(), edge);
                                }
                            }
                        }
                    }
                }
            } else {
                // No keywords found, fall back to recent
                return self.search_edges("", limit);
            }
            Ok(results.into_values().collect())
        }
    }

    fn delete_edge(&mut self, id: &EdgeId) -> Result<()> {
        let conn = self.get_connection()?;
        // Sync FTS delete for external content table
        conn.execute(
            "INSERT INTO edges_fts(edges_fts, rowid, source_text) 
             SELECT 'delete', rowid, source_text FROM edges WHERE id = ?1",
            [id],
        )?;
        // Delete actual edge
        conn.execute("DELETE FROM edges WHERE id = ?1", [id])?;
        Ok(())
    }

    fn delete_edges_by_query(&mut self, query: &str) -> Result<usize> {
        // Find matching edges first
        let matching = self.search_edges(query, 1000)?;
        let count = matching.len();
        
        for edge in matching {
            self.delete_edge(&edge.id)?;
        }
        
        Ok(count)
    }

    fn cleanup_orphaned_nodes(&mut self) -> Result<usize> {
        let conn = self.get_connection()?;
        let count = conn.execute(
            "DELETE FROM nodes 
             WHERE id NOT IN (SELECT source_id FROM edges) 
             AND id NOT IN (SELECT target_id FROM edges)",
            [],
        )?;
        Ok(count as usize)
    }

    fn hard_gc(&mut self, threshold: f32) -> Result<usize> {
        let now = chrono::Utc::now();
        let mut edges_to_delete = Vec::new();
        
        {
            let conn = self.get_connection()?;
            let mut stmt = conn.prepare("SELECT id, source_id, target_id, predicate, created_at, last_accessed, access_count, source_text, ext_source_id, confidence FROM edges")?;
            let edge_iter = stmt.query_map([], |row| {
                let created_at_str: String = row.get(4)?;
                let last_accessed_str: String = row.get(5)?;
                
                let created_at = parse_date(&created_at_str);
                let last_accessed = parse_date(&last_accessed_str);

                Ok(Edge {
                    id: row.get(0)?,
                    source: row.get(1)?,
                    target: row.get(2)?,
                    predicate: row.get(3)?,
                    created_at,
                    last_accessed,
                    access_count: row.get(6)?,
                    source_text: row.get(7)?,
                    source_id: row.get(8)?,
                    confidence: row.get(9)?,
                })
            })?;

            for edge in edge_iter {
                let edge = edge?;
                let strength = crate::forgetting::strength::calculate_strength(&edge, now);
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
        let conn = self.get_connection()?;
        
        // Rebind edges
        conn.execute("UPDATE edges SET source_id = ?1 WHERE source_id = ?2", [winner_id, loser_id])?;
        conn.execute("UPDATE edges SET target_id = ?1 WHERE target_id = ?2", [winner_id, loser_id])?;
        
        // Delete loser node
        conn.execute("DELETE FROM nodes WHERE id = ?1", [loser_id])?;
        
        Ok(())
    }
}
