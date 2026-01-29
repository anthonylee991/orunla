use crate::extractor::hybrid::HybridExtractor;
use crate::graph::{Edge, GraphStore, Node, NodeType};
use crate::retriever::{search::HybridRetriever, RecallRequest, Retriever};
use crate::storage::sqlite::SqliteStorage;
use crate::storage::{Storage, StorageStats};
use crate::utils::document::{chunk_document, parse_csv, parse_json_lines};
use axum::{
    extract::{ConnectInfo, Multipart, Path, Request, State},
    http::{HeaderMap, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{delete, get, post},
    Json, Router,
};
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::net::{SocketAddr, IpAddr};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

// Simple token bucket rate limiter
struct RateLimiterEntry {
    tokens: f64,
    last_update: Instant,
}

pub struct RateLimiter {
    // IP -> (tokens, last_update)
    buckets: Mutex<HashMap<IpAddr, RateLimiterEntry>>,
    max_tokens: f64,
    refill_rate: f64, // tokens per second
}

impl RateLimiter {
    fn new(max_requests_per_minute: u32) -> Self {
        let max_tokens = max_requests_per_minute as f64;
        let refill_rate = max_tokens / 60.0; // Convert to per-second

        Self {
            buckets: Mutex::new(HashMap::new()),
            max_tokens,
            refill_rate,
        }
    }

    async fn check_rate_limit(&self, ip: IpAddr) -> bool {
        let mut buckets = self.buckets.lock().await;
        let now = Instant::now();

        let entry = buckets.entry(ip).or_insert(RateLimiterEntry {
            tokens: self.max_tokens,
            last_update: now,
        });

        // Refill tokens based on time elapsed
        let elapsed = now.duration_since(entry.last_update).as_secs_f64();
        entry.tokens = (entry.tokens + elapsed * self.refill_rate).min(self.max_tokens);
        entry.last_update = now;

        // Check if we have at least 1 token
        if entry.tokens >= 1.0 {
            entry.tokens -= 1.0;
            true
        } else {
            false
        }
    }
}

pub struct ServerState {
    pub storage: Mutex<SqliteStorage>,
    pub extractor: Arc<HybridExtractor>,
    pub api_key: Option<String>,
    pub rate_limiter: Arc<RateLimiter>,
}

// Rate limiting middleware
async fn rate_limit_middleware(
    State(state): State<Arc<ServerState>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let ip = addr.ip();

    if !state.rate_limiter.check_rate_limit(ip).await {
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }

    Ok(next.run(request).await)
}

// Auth middleware
async fn auth_middleware(
    State(state): State<Arc<ServerState>>,
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Skip auth if no API key is configured
    if state.api_key.is_none() {
        return Ok(next.run(request).await);
    }

    let api_key = state.api_key.as_ref().unwrap();

    // Check Authorization header (Bearer token)
    if let Some(auth_header) = headers.get("authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if auth_str.starts_with("Bearer ") {
                let token = &auth_str[7..];
                if token == api_key {
                    return Ok(next.run(request).await);
                }
            }
        }
    }

    // Check X-API-Key header
    if let Some(key_header) = headers.get("x-api-key") {
        if let Ok(key_str) = key_header.to_str() {
            if key_str == api_key {
                return Ok(next.run(request).await);
            }
        }
    }

    Err(StatusCode::UNAUTHORIZED)
}

pub async fn start_server(storage: SqliteStorage, port: u16, api_key: Option<String>) -> anyhow::Result<()> {
    println!("Initializing hybrid extractor (Patterns + GliNER)...");
    let extractor = Arc::new(HybridExtractor::new()?);
    let rate_limiter = Arc::new(RateLimiter::new(60)); // 60 requests per minute per IP

    let state = Arc::new(ServerState {
        storage: Mutex::new(storage),
        extractor,
        api_key: api_key.clone(),
        rate_limiter,
    });

    if api_key.is_some() {
        println!("✓ API key authentication ENABLED");
    } else {
        println!("⚠️  WARNING: API key authentication DISABLED - server is unprotected!");
        println!("   For production use, restart with --api-key <your-secret-key>");
    }
    println!("✓ Rate limiting ENABLED (60 requests/minute per IP)");

    // Protected routes (require auth if API key is set + rate limiting)
    let protected_routes = Router::new()
        .route("/ingest", post(ingest_handler))
        .route("/ingest-file", post(ingest_file_handler))
        .route("/recall", post(recall_handler))
        .route("/memories/:id", delete(delete_memory_handler))
        .route("/memories/purge", post(purge_topic_handler))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            rate_limit_middleware,
        ));

    // Public routes (no auth required, but still rate limited)
    let public_routes = Router::new()
        .route("/health", get(health_check))
        .route("/stats", get(get_stats))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            rate_limit_middleware,
        ));

    let app = Router::new()
        .merge(protected_routes)
        .merge(public_routes)
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    println!("Orunla server listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;

    Ok(())
}

async fn health_check() -> Json<Value> {
    Json(json!({ "status": "ok", "version": "0.1.0" }))
}

async fn get_stats(State(state): State<Arc<ServerState>>) -> Json<Value> {
    let storage = state.storage.lock().await;
    let stats = storage.stats().unwrap_or(StorageStats {
        node_count: 0,
        edge_count: 0,
        db_size_bytes: 0,
        oldest_memory: None,
        newest_memory: None,
    });
    Json(json!({
        "node_count": stats.node_count,
        "edge_count": stats.edge_count,
        "db_size_bytes": stats.db_size_bytes,
    }))
}

const MAX_TEXT_LENGTH: usize = 1_000_000; // 1MB
const MAX_QUERY_LENGTH: usize = 10_000; // 10KB
const MAX_FILE_SIZE: usize = 50_000_000; // 50MB

#[derive(Deserialize)]
struct IngestPayload {
    text: String,
    source_id: Option<String>,
}

async fn ingest_handler(
    State(state): State<Arc<ServerState>>,
    Json(payload): Json<IngestPayload>,
) -> Json<Value> {
    // Input validation
    if payload.text.len() > MAX_TEXT_LENGTH {
        return Json(json!({
            "status": "error",
            "message": format!("Text too long. Maximum {} characters allowed.", MAX_TEXT_LENGTH)
        }));
    }

    if payload.text.trim().is_empty() {
        return Json(json!({
            "status": "error",
            "message": "Text cannot be empty"
        }));
    }

    let triplets = match state.extractor.extract_triplets(&payload.text) {
        Ok(t) => t,
        Err(_e) => {
            eprintln!("Extraction error: {}", _e); // Log server-side only
            return Json(json!({
                "status": "error",
                "message": "Failed to extract information from text"
            }));
        }
    };

    let mut storage = state.storage.lock().await;

    let mut added_count = 0;
    for triplet in triplets {
        // Resolve or create nodes
        let s_id = storage
            .resolve_entity(&triplet.subject)
            .unwrap()
            .unwrap_or_else(|| {
                storage
                    .add_node(Node::new(triplet.subject, NodeType::Unknown))
                    .unwrap()
            });

        let o_id = storage
            .resolve_entity(&triplet.object)
            .unwrap()
            .unwrap_or_else(|| {
                storage
                    .add_node(Node::new(triplet.object, NodeType::Unknown))
                    .unwrap()
            });

        let edge = Edge::new(s_id, o_id, triplet.predicate, payload.text.clone());
        if storage.add_edge(edge).is_ok() {
            added_count += 1;
        }
    }

    Json(json!({ "status": "ok", "added_triplets": added_count }))
}

async fn ingest_file_handler(
    State(state): State<Arc<ServerState>>,
    mut multipart: Multipart,
) -> Json<Value> {
    let extractor = &state.extractor;
    let mut storage = state.storage.lock().await;

    let mut file_name: Option<String> = None;
    let mut file_content: Option<String> = None;
    let mut total_size = 0usize;

    while let Ok(Some(field)) = multipart.next_field().await {
        let name = field.name().unwrap_or("").to_string();

        if name == "file" {
            file_name = field.file_name().map(|s| s.to_string());

            // Read with size limit
            match field.text().await {
                Ok(text) => {
                    total_size += text.len();
                    if total_size > MAX_FILE_SIZE {
                        return Json(json!({
                            "status": "error",
                            "message": format!("File too large. Maximum {} bytes allowed.", MAX_FILE_SIZE)
                        }));
                    }
                    file_content = Some(text);
                }
                Err(_e) => {
                    eprintln!("File read error: {}", _e); // Log server-side only
                    return Json(json!({
                        "status": "error",
                        "message": "Failed to read file content"
                    }));
                }
            }
        }
    }

    let content = match file_content {
        Some(c) if !c.is_empty() => c,
        _ => {
            return Json(json!({
                "status": "error",
                "message": "No file provided. Send a multipart form with 'file' field."
            }));
        }
    };

    let file_type = file_name
        .as_ref()
        .and_then(|n| {
            std::path::Path::new(n)
                .extension()
                .and_then(|e| e.to_str())
                .map(|s| s.to_lowercase())
        })
        .unwrap_or_else(|| "txt".to_string());

    let chunks: Vec<(String, &str)> = match file_type.as_str() {
        "json" => parse_json_lines(&content)
            .into_iter()
            .map(|c| (c, "json_line"))
            .collect(),
        "csv" => parse_csv(&content)
            .into_iter()
            .map(|c| (c, "csv_row"))
            .collect(),
        _ => chunk_document(&content, 1000)
            .into_iter()
            .map(|c| (c, "paragraph"))
            .collect(),
    };

    let mut total_added = 0;
    let source_name = file_name
        .clone()
        .unwrap_or_else(|| "uploaded_file".to_string());

    for (i, (chunk, chunk_type)) in chunks.iter().enumerate() {
        let triplets = match extractor.extract_triplets(chunk) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("Extraction error on chunk {}: {}", i + 1, e);
                continue;
            }
        };

        for triplet in triplets {
            let s_id = storage
                .resolve_entity(&triplet.subject)
                .unwrap()
                .unwrap_or_else(|| {
                    storage
                        .add_node(Node::new(triplet.subject, NodeType::Unknown))
                        .unwrap()
                });

            let o_id = storage
                .resolve_entity(&triplet.object)
                .unwrap()
                .unwrap_or_else(|| {
                    storage
                        .add_node(Node::new(triplet.object, NodeType::Unknown))
                        .unwrap()
                });

            let edge = Edge::new(s_id, o_id, triplet.predicate, chunk.to_string());
            if storage.add_edge(edge).is_ok() {
                total_added += 1;
            }
        }
    }

    Json(json!({
        "status": "ok",
        "file": file_name,
        "chunks_processed": chunks.len(),
        "total_triplets_added": total_added
    }))
}

#[derive(Deserialize)]
struct RecallPayload {
    query: String,
    limit: Option<usize>,
    min_strength: Option<f32>,
}

async fn recall_handler(
    State(state): State<Arc<ServerState>>,
    Json(payload): Json<RecallPayload>,
) -> Json<Value> {
    // Input validation
    if payload.query.len() > MAX_QUERY_LENGTH {
        return Json(json!({
            "status": "error",
            "message": format!("Query too long. Maximum {} characters allowed.", MAX_QUERY_LENGTH)
        }));
    }

    let limit = payload.limit.unwrap_or(5).min(10000); // Cap at 10k results

    let storage = state.storage.lock().await;
    let retriever = HybridRetriever::new(&*storage);

    let request = RecallRequest {
        query: payload.query,
        limit,
        min_confidence: 0.0,
        min_strength: payload.min_strength.unwrap_or(0.1),
    };

    let response = match retriever.recall(request) {
        Ok(r) => r,
        Err(_e) => {
            eprintln!("Recall error: {}", _e); // Log server-side only
            return Json(json!({
                "status": "error",
                "message": "Failed to recall memories"
            }));
        }
    };

    let memories: Vec<Value> = response
        .memories
        .into_iter()
        .map(|m| {
            json!({
                "id": m.edge.id,
                "subject": m.subject_node.label,
                "predicate": m.edge.predicate,
                "object": m.object_node.label,
                "text": m.edge.source_text,
                "confidence": m.edge.confidence,
                "strength": m.relevance_score,
            })
        })
        .collect();

    Json(json!({ "memories": memories }))
}

async fn delete_memory_handler(
    State(state): State<Arc<ServerState>>,
    Path(id): Path<String>,
) -> Json<Value> {
    let mut storage = state.storage.lock().await;
    match storage.delete_edge(&id) {
        Ok(_) => {
            let orphaned = storage.cleanup_orphaned_nodes().unwrap_or(0);
            Json(json!({ "status": "ok", "message": format!("Memory deleted. Cleaned up {} orphaned nodes.", orphaned) }))
        }
        Err(_e) => {
            eprintln!("Delete error: {}", _e); // Log server-side only
            Json(json!({ "status": "error", "message": "Failed to delete memory" }))
        }
    }
}

#[derive(Deserialize)]
struct PurgePayload {
    query: String,
}

async fn purge_topic_handler(
    State(state): State<Arc<ServerState>>,
    Json(payload): Json<PurgePayload>,
) -> Json<Value> {
    // Input validation
    if payload.query.len() > MAX_QUERY_LENGTH {
        return Json(json!({
            "status": "error",
            "message": format!("Query too long. Maximum {} characters allowed.", MAX_QUERY_LENGTH)
        }));
    }

    let mut storage = state.storage.lock().await;
    match storage.delete_edges_by_query(&payload.query) {
        Ok(count) => {
            let orphaned = storage.cleanup_orphaned_nodes().unwrap_or(0);
            Json(json!({
                "status": "ok",
                "purged_count": count,
                "orphaned_cleaned": orphaned
            }))
        }
        Err(_e) => {
            eprintln!("Purge error: {}", _e); // Log server-side only
            Json(json!({ "status": "error", "message": "Failed to purge memories" }))
        }
    }
}
