//! MCP (Model Context Protocol) server implementation.
//!
//! Contains the MCPServer struct with all tool handlers, and the SSE transport layer.

use anyhow::Result;
use axum::{
    extract::{Query, State as AxumState},
    http::StatusCode,
    response::{
        sse::{Event, Sse},
        IntoResponse,
    },
    routing::{get, post},
    Json, Router,
};
use futures::stream::Stream;
use crate::graph::{Edge, GraphStore, Node, NodeType};
use crate::retriever::{search::HybridRetriever, RecallRequest, Retriever};
use crate::storage::sqlite::SqliteStorage;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::convert::Infallible;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use tokio_stream::wrappers::UnboundedReceiverStream;
use tower_http::cors::CorsLayer;

// --- Types ---

#[derive(Serialize, Deserialize)]
pub struct Memory {
    pub id: String,
    pub subject: String,
    pub predicate: String,
    pub object: String,
    pub text: String,
    pub confidence: f32,
    pub strength: f32,
    pub memory_type: String,
}

#[derive(Serialize, Deserialize)]
pub struct AddMemoryInput {
    pub subject: String,
    pub predicate: String,
    pub object: String,
    #[serde(default)]
    pub text: String,
    #[serde(default)]
    pub memory_type: String,
}

#[derive(Serialize, Deserialize)]
pub struct SearchInput {
    pub query: String,
    #[serde(default = "default_limit")]
    pub limit: usize,
    #[serde(default)]
    pub memory_type: Option<String>,
    #[serde(default = "default_min_strength")]
    pub min_strength: f32,
}

fn default_min_strength() -> f32 {
    0.0
}

fn default_limit() -> usize {
    10
}

#[derive(Serialize, Deserialize)]
pub struct MCPMessage {
    pub jsonrpc: String,
    pub id: Option<serde_json::Value>,
    pub method: String,
    pub params: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}

#[derive(Serialize, Deserialize)]
pub struct ToolCall {
    pub name: String,
    pub arguments: serde_json::Value,
}

#[derive(Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
    #[serde(default)]
    pub name: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct SyncChatInput {
    pub messages: Vec<ChatMessage>,
    #[serde(default = "default_system_prompt")]
    pub system_prompt: String,
}

#[derive(Serialize, Deserialize)]
pub struct DeleteMemoryInput {
    pub id: String,
}

#[derive(Serialize, Deserialize)]
pub struct PurgeTopicInput {
    pub query: String,
}

fn default_system_prompt() -> String {
    "You are a helpful AI assistant.".to_string()
}

// --- MCPServer ---

pub struct MCPServer {
    storage: Arc<Mutex<SqliteStorage>>,
}

impl MCPServer {
    pub fn new(storage: SqliteStorage) -> Self {
        Self {
            storage: Arc::new(Mutex::new(storage)),
        }
    }

    pub async fn handle_message(&self, msg: &MCPMessage) -> Result<Option<serde_json::Value>> {
        let response = match msg.method.as_str() {
            "initialize" => self.handle_initialize().await?,
            "notifications/tools/list" | "tools/list" => self.handle_tools_list().await?,
            "notifications/tools/call" | "tools/call" => {
                let params = msg.params.as_ref().unwrap();
                let call: ToolCall = serde_json::from_value(params.clone())?;
                self.handle_tool_call(&call.name, &call.arguments).await?
            }
            _ => return Ok(None),
        };

        if let Some(id) = &msg.id {
            let mut full_response = response;
            if let Some(obj) = full_response.as_object_mut() {
                obj.insert("id".to_string(), id.clone());
            }
            Ok(Some(full_response))
        } else {
            Ok(None)
        }
    }

    async fn handle_initialize(&self) -> Result<serde_json::Value> {
        Ok(json!({
            "jsonrpc": "2.0",
            "result": {
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "tools": {}
                },
                "serverInfo": {
                    "name": "orunla-memory",
                    "version": "0.5.0"
                }
            }
        }))
    }

    async fn handle_tools_list(&self) -> Result<serde_json::Value> {
        let tools = vec![
            ToolDefinition {
                name: "memory_add".to_string(),
                description: "Add an important fact, constant, or preference to memory. Use this when the user tells you something important they want to remember.".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "subject": {
                            "type": "string",
                            "description": "The subject/entity (e.g., 'API_KEY', 'User prefers', 'Database URL')"
                        },
                        "predicate": {
                            "type": "string",
                            "description": "The relationship (e.g., 'is', 'uses', 'prefers')"
                        },
                        "object": {
                            "type": "string",
                            "description": "The value/object (e.g., 'https://api.example.com', 'dark mode', 'sk-1234...')"
                        },
                        "text": {
                            "type": "string",
                            "description": "Original source text for context"
                        },
                        "memory_type": {
                            "type": "string",
                            "description": "Type: 'constant' (persistent), 'context' (session), 'preference' (user preference)"
                        }
                    },
                    "required": ["subject", "predicate", "object"]
                }),
            },
            ToolDefinition {
                name: "memory_search".to_string(),
                description: "Search memory for relevant facts, constants, or preferences. Use this to recall what the user has told you.".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "query": {
                            "type": "string",
                            "description": "Search query (entity name, keyword, or concept)"
                        },
                        "limit": {
                            "type": "integer",
                            "description": "Maximum results to return",
                            "default": 10
                        },
                        "memory_type": {
                            "type": "string",
                            "description": "Filter by memory type (optional)"
                        }
                    },
                    "required": ["query"]
                }),
            },
            ToolDefinition {
                name: "memory_get_all".to_string(),
                description: "Get all memories from the knowledge graph. Useful for summarizing stored constants.".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "limit": {
                            "type": "integer",
                            "description": "Maximum results",
                            "default": 50
                        }
                    }
                }),
            },
            ToolDefinition {
                name: "memory_get_context".to_string(),
                description: "Get memories formatted as a context block for injecting into prompts. Returns a formatted string.".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "query": {
                            "type": "string",
                            "description": "Query to find relevant memories"
                        }
                    },
                    "required": ["query"]
                }),
            },
            ToolDefinition {
                name: "memory_sync_chat".to_string(),
                description: "Sync chat history and extract memories automatically. Pass an array of messages (from Claude, ChatGPT, etc.) and it will extract facts, preferences, and constants into memory.".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "messages": {
                            "type": "array",
                            "description": "Array of chat messages with role and content",
                            "items": {
                                "type": "object",
                                "properties": {
                                    "role": {
                                        "type": "string",
                                        "description": "Message role: 'user', 'assistant', or 'system'"
                                    },
                                    "content": {
                                        "type": "string",
                                        "description": "Message content"
                                    }
                                },
                                "required": ["role", "content"]
                            }
                        },
                        "system_prompt": {
                            "type": "string",
                            "description": "Optional system prompt for context",
                            "default": "You are a helpful AI assistant."
                        }
                    },
                    "required": ["messages"]
                }),
            },
            ToolDefinition {
                name: "memory_delete".to_string(),
                description: "Delete a specific memory by its unique ID.".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "id": {
                            "type": "string",
                            "description": "The unique ID of the memory to delete"
                        }
                    },
                    "required": ["id"]
                }),
            },
            ToolDefinition {
                name: "memory_purge_topic".to_string(),
                description: "Forget/delete all memories related to a specific topic or keyword. This is permanent.".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "query": {
                            "type": "string",
                            "description": "The topic or keyword to purge (e.g., 'Project Alpha', 'Paris')"
                        }
                    },
                    "required": ["query"]
                }),
            },
            ToolDefinition {
                name: "memory_gc".to_string(),
                description: "Run garbage collection to permanently delete highly decayed memories. Use this to keep the memory database lean.".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "threshold": {
                            "type": "number",
                            "description": "The strength threshold for deletion (default: 0.05). Any memory below this is permanently removed.",
                            "default": 0.05
                        }
                    }
                }),
            },
        ];

        Ok(json!({
            "jsonrpc": "2.0",
            "result": {
                "tools": tools
            }
        }))
    }

    async fn handle_tool_call(
        &self,
        name: &str,
        args: &serde_json::Value,
    ) -> Result<serde_json::Value> {
        match name {
            "memory_add" => self.tool_memory_add(args).await,
            "memory_search" => self.tool_memory_search(args).await,
            "memory_get_all" => self.tool_memory_get_all(args).await,
            "memory_get_context" => self.tool_memory_get_context(args).await,
            "memory_sync_chat" => self.tool_memory_sync_chat(args).await,
            "memory_delete" => self.tool_memory_delete(args).await,
            "memory_purge_topic" => self.tool_memory_purge_topic(args).await,
            "memory_gc" => self.tool_memory_gc(args).await,
            _ => Ok(json!({ "error": "Unknown tool" })),
        }
    }

    async fn tool_memory_add(&self, args: &serde_json::Value) -> Result<serde_json::Value> {
        let input: AddMemoryInput = serde_json::from_value(args.clone())?;
        let _memory_type = if input.memory_type.is_empty() {
            "constant".to_string()
        } else {
            input.memory_type
        };

        let mut storage = self.storage.lock().await;

        let s_id = storage.resolve_entity(&input.subject)?.unwrap_or_else(|| {
            storage
                .add_node(Node::new(input.subject.clone(), NodeType::Unknown))
                .unwrap()
        });
        let o_id = storage.resolve_entity(&input.object)?.unwrap_or_else(|| {
            storage
                .add_node(Node::new(input.object.clone(), NodeType::Unknown))
                .unwrap()
        });

        let edge = Edge::new(s_id, o_id, input.predicate.clone(), input.text.clone());
        storage.add_edge(edge)?;

        Ok(json!({
            "jsonrpc": "2.0",
            "result": {
                "content": [{
                    "type": "text",
                    "text": format!("Added memory: {} {} {}", input.subject, input.predicate, input.object)
                }]
            }
        }))
    }

    async fn tool_memory_search(&self, args: &serde_json::Value) -> Result<serde_json::Value> {
        let input: SearchInput = serde_json::from_value(args.clone())?;
        let storage = self.storage.lock().await;
        let retriever = HybridRetriever::new(&*storage);

        let request = RecallRequest {
            query: input.query,
            limit: input.limit,
            min_confidence: 0.0,
            min_strength: input.min_strength,
        };
        let response = retriever.recall(request)?;

        let memories: Vec<Memory> = response
            .memories
            .into_iter()
            .map(|m| Memory {
                id: m.edge.id.clone(),
                subject: m.subject_node.label,
                predicate: m.edge.predicate,
                object: m.object_node.label,
                text: m.edge.source_text,
                confidence: m.edge.confidence,
                strength: m.relevance_score,
                memory_type: "constant".to_string(),
            })
            .collect();

        Ok(json!({
            "jsonrpc": "2.0",
            "result": {
                "content": [{
                    "type": "text",
                    "text": serde_json::to_string_pretty(&memories).unwrap()
                }]
            }
        }))
    }

    async fn tool_memory_get_all(&self, args: &serde_json::Value) -> Result<serde_json::Value> {
        let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(50) as usize;
        let storage = self.storage.lock().await;
        let retriever = HybridRetriever::new(&*storage);

        let request = RecallRequest {
            query: "".to_string(),
            limit,
            min_confidence: 0.0,
            min_strength: 0.0,
        };
        let response = retriever.recall(request)?;

        let memories: Vec<Memory> = response
            .memories
            .into_iter()
            .map(|m| Memory {
                id: m.edge.id.clone(),
                subject: m.subject_node.label,
                predicate: m.edge.predicate,
                object: m.object_node.label,
                text: m.edge.source_text,
                confidence: m.edge.confidence,
                strength: m.relevance_score,
                memory_type: "constant".to_string(),
            })
            .collect();

        Ok(json!({
            "jsonrpc": "2.0",
            "result": {
                "content": [{
                    "type": "text",
                    "text": serde_json::to_string_pretty(&memories).unwrap()
                }]
            }
        }))
    }

    async fn tool_memory_get_context(&self, args: &serde_json::Value) -> Result<serde_json::Value> {
        let query = args.get("query").and_then(|v| v.as_str()).unwrap_or("");
        let storage = self.storage.lock().await;
        let retriever = HybridRetriever::new(&*storage);

        let request = RecallRequest {
            query: query.to_string(),
            limit: 10,
            min_confidence: 0.0,
            min_strength: 0.1,
        };
        let response = retriever.recall(request)?;

        let context = response
            .memories
            .into_iter()
            .map(|m| {
                format!(
                    "{} {} {}",
                    m.subject_node.label, m.edge.predicate, m.object_node.label
                )
            })
            .collect::<Vec<_>>()
            .join("\n");

        Ok(json!({
            "jsonrpc": "2.0",
            "result": {
                "content": [{
                    "type": "text",
                    "text": context
                }]
            }
        }))
    }

    async fn tool_memory_sync_chat(&self, args: &serde_json::Value) -> Result<serde_json::Value> {
        let input: SyncChatInput = serde_json::from_value(args.clone())?;

        let mut all_text = Vec::new();
        for msg in &input.messages {
            if msg.role == "system" {
                continue;
            }
            if !msg.content.trim().is_empty() {
                all_text.push(format!("[{}]: {}", msg.role, msg.content));
            }
        }

        if all_text.is_empty() {
            return Ok(json!({
                "jsonrpc": "2.0",
                "result": {
                    "content": [{
                        "type": "text",
                        "text": "No messages to process."
                    }]
                }
            }));
        }

        let combined_text = all_text.join("\n\n");

        let extractor = crate::extractor::hybrid::HybridExtractor::new()?;
        let triplets = extractor.extract_triplets(&combined_text)?;

        let mut storage = self.storage.lock().await;
        let mut added = 0;

        for triplet in triplets {
            let (start, end) = triplet.source_span;
            let source_text = if start < combined_text.len() && end <= combined_text.len() {
                combined_text[start..end].to_string()
            } else {
                format!(
                    "{} {} {}",
                    triplet.subject, triplet.predicate, triplet.object
                )
            };

            let s_id = storage
                .resolve_entity(&triplet.subject)?
                .unwrap_or_else(|| {
                    storage
                        .add_node(Node::new(triplet.subject, NodeType::Unknown))
                        .unwrap()
                });
            let o_id = storage.resolve_entity(&triplet.object)?.unwrap_or_else(|| {
                storage
                    .add_node(Node::new(triplet.object, NodeType::Unknown))
                    .unwrap()
            });
            let edge = Edge::new(s_id, o_id, triplet.predicate, source_text);
            if storage.add_edge(edge).is_ok() {
                added += 1;
            }
        }

        Ok(json!({
            "jsonrpc": "2.0",
            "result": {
                "content": [{
                    "type": "text",
                    "text": format!("Synced {} messages, extracted and stored {} memories.", input.messages.len(), added)
                }]
            }
        }))
    }

    async fn tool_memory_delete(&self, args: &serde_json::Value) -> Result<serde_json::Value> {
        let input: DeleteMemoryInput = serde_json::from_value(args.clone())?;
        let mut storage = self.storage.lock().await;

        storage.delete_edge(&input.id)?;
        storage.cleanup_orphaned_nodes()?;

        Ok(json!({
            "jsonrpc": "2.0",
            "result": {
                "content": [{
                    "type": "text",
                    "text": format!("Memory {} deleted.", input.id)
                }]
            }
        }))
    }

    async fn tool_memory_purge_topic(&self, args: &serde_json::Value) -> Result<serde_json::Value> {
        let input: PurgeTopicInput = serde_json::from_value(args.clone())?;
        let mut storage = self.storage.lock().await;

        let count = storage.delete_edges_by_query(&input.query)?;
        let orphaned = storage.cleanup_orphaned_nodes()?;

        Ok(json!({
            "jsonrpc": "2.0",
            "result": {
                "content": [{
                    "type": "text",
                    "text": format!("Purged {} memories and cleaned up {} orphaned nodes related to '{}'.", count, orphaned, input.query)
                }]
            }
        }))
    }

    async fn tool_memory_gc(&self, args: &serde_json::Value) -> Result<serde_json::Value> {
        let threshold = args
            .get("threshold")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.05) as f32;
        let mut storage = self.storage.lock().await;

        let count = storage.hard_gc(threshold)?;
        let orphaned = storage.cleanup_orphaned_nodes()?;

        Ok(json!({
            "jsonrpc": "2.0",
            "result": {
                "content": [{
                    "type": "text",
                    "text": format!("Garbage collection complete. Permanently deleted {} decayed memories and cleaned up {} orphaned nodes.", count, orphaned)
                }]
            }
        }))
    }
}

impl Clone for MCPServer {
    fn clone(&self) -> Self {
        Self {
            storage: Arc::clone(&self.storage),
        }
    }
}

// --- SSE Transport ---

#[derive(Clone)]
pub struct SseState {
    pub server: MCPServer,
    pub sessions: Arc<Mutex<HashMap<String, mpsc::UnboundedSender<String>>>>,
}

#[derive(Deserialize)]
pub struct SessionQuery {
    pub session_id: String,
}

pub async fn sse_handler(
    AxumState(state): AxumState<SseState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let session_id = uuid::Uuid::new_v4().to_string();
    let (tx, rx) = mpsc::unbounded_channel::<String>();

    state.sessions.lock().await.insert(session_id.clone(), tx.clone());

    let endpoint_url = format!("/message?session_id={}", session_id);
    let _ = tx.send(format!("__endpoint__{}", endpoint_url));

    let stream = UnboundedReceiverStream::new(rx);
    let event_stream = futures::stream::unfold(stream, |mut stream| async move {
        use tokio_stream::StreamExt;
        match stream.next().await {
            Some(data) => {
                let event = if let Some(url) = data.strip_prefix("__endpoint__") {
                    Event::default().event("endpoint").data(url.to_string())
                } else {
                    Event::default().event("message").data(data)
                };
                Some((Ok(event), stream))
            }
            None => None,
        }
    });

    Sse::new(event_stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(std::time::Duration::from_secs(15))
            .text("ping"),
    )
}

pub async fn message_handler(
    AxumState(state): AxumState<SseState>,
    Query(query): Query<SessionQuery>,
    Json(msg): Json<MCPMessage>,
) -> impl IntoResponse {
    let sessions = state.sessions.lock().await;
    let tx = match sessions.get(&query.session_id) {
        Some(tx) => tx.clone(),
        None => return StatusCode::NOT_FOUND,
    };
    drop(sessions);

    match state.server.handle_message(&msg).await {
        Ok(Some(response)) => {
            let response_str = serde_json::to_string(&response).unwrap_or_default();
            let _ = tx.send(response_str);
        }
        Ok(None) => {}
        Err(e) => {
            eprintln!("[orunla-sse] Error handling message: {}", e);
            if let Some(id) = &msg.id {
                let error_response = json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "error": {
                        "code": -32603,
                        "message": e.to_string()
                    }
                });
                if let Ok(response_str) = serde_json::to_string(&error_response) {
                    let _ = tx.send(response_str);
                }
            }
        }
    }

    StatusCode::ACCEPTED
}

/// Build the MCP SSE routes (GET /sse, POST /message) with CORS.
pub fn build_mcp_sse_routes(server: MCPServer) -> Router {
    let state = SseState {
        server,
        sessions: Arc::new(Mutex::new(HashMap::new())),
    };

    Router::new()
        .route("/sse", get(sse_handler))
        .route("/message", post(message_handler))
        .layer(CorsLayer::very_permissive())
        .with_state(state)
}

// --- Stdio Transport ---

pub async fn run_stdio(server: MCPServer) -> Result<()> {
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter};

    let stdin = tokio::io::stdin();
    let mut reader = BufReader::new(stdin).lines();
    let mut stdout = BufWriter::new(tokio::io::stdout());

    while let Some(line) = reader.next_line().await? {
        if line.is_empty() {
            continue;
        }

        let msg: MCPMessage = match serde_json::from_str(&line) {
            Ok(m) => m,
            Err(e) => {
                eprintln!("Failed to parse message: {}", e);
                continue;
            }
        };

        match server.handle_message(&msg).await {
            Ok(Some(response)) => {
                let response_str = serde_json::to_string(&response).unwrap();
                stdout.write_all(response_str.as_bytes()).await?;
                stdout.write_all(b"\n").await?;
                stdout.flush().await?;
            }
            Ok(None) => {}
            Err(e) => {
                eprintln!("Error handling message: {}", e);

                if let Some(id) = &msg.id {
                    let error_response = json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "error": {
                            "code": -32603,
                            "message": e.to_string()
                        }
                    });
                    if let Ok(response_str) = serde_json::to_string(&error_response) {
                        let _ = stdout.write_all(response_str.as_bytes()).await;
                        let _ = stdout.write_all(b"\n").await;
                        let _ = stdout.flush().await;
                    }
                }
            }
        }
    }

    Ok(())
}

/// Start the MCP SSE server on the given port (standalone, no REST API).
pub async fn run_sse(server: MCPServer, port: u16) -> Result<()> {
    let app = build_mcp_sse_routes(server);

    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], port));
    eprintln!("[orunla] SSE transport listening on http://{}", addr);
    eprintln!("[orunla] Connect your MCP client to http://localhost:{}/sse", port);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}
