//! Unified server that combines REST API + MCP SSE on a single port.
//!
//! REST routes: /ingest, /ingest-file, /recall, /memories/*, /health, /stats
//! MCP SSE routes: /sse, /message

use anyhow::Result;
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_http::cors::CorsLayer;

use crate::extractor::hybrid::HybridExtractor;
use crate::mcp::{self, MCPServer};
use crate::server::{self, RateLimiter, ServerState};
use crate::storage::sqlite::SqliteStorage;

/// Start the unified server (REST API + MCP SSE) on a single port.
///
/// This merges both routers onto one Axum listener. MCP SSE routes (/sse, /message)
/// get CORS only. REST routes get auth + rate limiting.
pub async fn start_unified_server(
    storage: SqliteStorage,
    mcp_server: MCPServer,
    port: u16,
    api_key: Option<String>,
) -> Result<()> {
    eprintln!("[orunla] Initializing hybrid extractor (Patterns + GliNER)...");
    let extractor = Arc::new(HybridExtractor::new()?);
    let rate_limiter = Arc::new(RateLimiter::new(60));

    let rest_state = Arc::new(ServerState {
        storage: Mutex::new(storage),
        extractor,
        api_key: api_key.clone(),
        rate_limiter,
    });

    if api_key.is_some() {
        eprintln!("[orunla] API key authentication ENABLED for REST routes");
    } else {
        eprintln!("[orunla] REST routes: no API key (unauthenticated)");
    }

    // Build REST API routes (with auth + rate limiting)
    let rest_router = server::build_rest_routes(rest_state);

    // Build MCP SSE routes (with CORS only — MCP handles its own protocol)
    let mcp_router = mcp::build_mcp_sse_routes(mcp_server);

    // Merge: MCP routes first (they're more specific), then REST routes
    let app = mcp_router
        .merge(rest_router)
        .layer(CorsLayer::very_permissive());

    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], port));
    eprintln!("[orunla] Unified server listening on http://{}", addr);
    eprintln!("[orunla]   MCP SSE: http://localhost:{}/sse", port);
    eprintln!("[orunla]   REST API: http://localhost:{}/health", port);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .await?;

    Ok(())
}
