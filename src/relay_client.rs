//! WebSocket relay client for MCP proxy.
//!
//! Connects outbound to the cloud relay server so Claude browser can reach
//! the desktop MCP server without Cloudflare Tunnel or port forwarding.

use anyhow::Result;
use futures::{SinkExt, StreamExt};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio_tungstenite::{connect_async, tungstenite::Message};

use crate::mcp::MCPServer;

const RELAY_URL: &str = "https://orunla-production.up.railway.app";

pub struct McpRelayClient {
    relay_url: String,
    device_id: String,
    mcp_server: MCPServer,
}

impl McpRelayClient {
    pub fn new(device_id: String, mcp_server: MCPServer) -> Self {
        Self {
            relay_url: RELAY_URL.to_string(),
            device_id,
            mcp_server,
        }
    }

    /// Get the public relay URL that Claude browser should connect to.
    pub fn relay_sse_url(&self) -> String {
        format!("{}/mcp/{}/sse", self.relay_url, self.device_id)
    }

    /// Connect to the relay and process messages forever.
    /// Auto-reconnects with exponential backoff on disconnect.
    pub async fn connect_loop(&self) {
        let mut backoff_secs = 1u64;
        let max_backoff = 60u64;

        loop {
            match self.connect_once().await {
                Ok(()) => {
                    eprintln!("[mcp-relay] Connection closed gracefully");
                    backoff_secs = 1; // Reset backoff on clean close
                }
                Err(e) => {
                    eprintln!("[mcp-relay] Connection error: {}", e);
                }
            }

            eprintln!(
                "[mcp-relay] Reconnecting in {}s...",
                backoff_secs
            );
            tokio::time::sleep(std::time::Duration::from_secs(backoff_secs)).await;
            backoff_secs = (backoff_secs * 2).min(max_backoff);
        }
    }

    async fn connect_once(&self) -> Result<()> {
        // Convert https:// to wss:// for WebSocket connection
        let ws_url = self
            .relay_url
            .replace("https://", "wss://")
            .replace("http://", "ws://");
        let url = format!("{}/mcp/ws?device_id={}", ws_url, self.device_id);

        eprintln!("[mcp-relay] Connecting to {}", url);

        let (ws_stream, _response) = connect_async(&url).await?;
        eprintln!("[mcp-relay] Connected to relay");

        let (write, mut read) = ws_stream.split();
        let write = Arc::new(Mutex::new(write));

        // Heartbeat task — sends WebSocket pings to keep connection alive
        let heartbeat_write = write.clone();
        let heartbeat_handle = tokio::spawn(async move {
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(30)).await;
                let mut w = heartbeat_write.lock().await;
                if w.send(Message::Ping(vec![])).await.is_err() {
                    break;
                }
            }
        });

        // Process messages from the relay (forwarded from Claude browser)
        while let Some(msg) = read.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    // Check for relay control messages
                    if let Ok(ctrl) = serde_json::from_str::<serde_json::Value>(&text) {
                        if ctrl.get("type").and_then(|v| v.as_str()) == Some("connected") {
                            eprintln!("[mcp-relay] Relay confirmed connection");
                            continue;
                        }
                    }

                    // Parse as MCP JSON-RPC message
                    let mcp_msg: crate::mcp::MCPMessage = match serde_json::from_str(&text) {
                        Ok(m) => m,
                        Err(e) => {
                            eprintln!("[mcp-relay] Failed to parse message: {}", e);
                            continue;
                        }
                    };

                    // Process with MCP server
                    let mut w = write.lock().await;
                    match self.mcp_server.handle_message(&mcp_msg).await {
                        Ok(Some(response)) => {
                            let response_str =
                                serde_json::to_string(&response).unwrap_or_default();
                            if let Err(e) = w.send(Message::Text(response_str)).await {
                                eprintln!("[mcp-relay] Failed to send response: {}", e);
                                break;
                            }
                        }
                        Ok(None) => {
                            // Notification — no response needed
                        }
                        Err(e) => {
                            eprintln!("[mcp-relay] Error handling message: {}", e);
                            // Send error response if there was a request ID
                            if let Some(id) = &mcp_msg.id {
                                let error_response = serde_json::json!({
                                    "jsonrpc": "2.0",
                                    "id": id,
                                    "error": {
                                        "code": -32603,
                                        "message": e.to_string()
                                    }
                                });
                                if let Ok(response_str) =
                                    serde_json::to_string(&error_response)
                                {
                                    let _ =
                                        w.send(Message::Text(response_str)).await;
                                }
                            }
                        }
                    }
                }
                Ok(Message::Ping(data)) => {
                    let mut w = write.lock().await;
                    let _ = w.send(Message::Pong(data)).await;
                }
                Ok(Message::Close(_)) => {
                    eprintln!("[mcp-relay] Server closed connection");
                    break;
                }
                Err(e) => {
                    eprintln!("[mcp-relay] WebSocket error: {}", e);
                    break;
                }
                _ => {} // Ignore binary, pong, etc.
            }
        }

        heartbeat_handle.abort();
        Ok(())
    }
}
