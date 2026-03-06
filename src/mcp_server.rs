use anyhow::Result;
use clap::Parser;
use orunla::mcp::MCPServer;
use orunla::storage::{sqlite::SqliteStorage, Storage, StorageConfig};

#[derive(Parser)]
#[command(name = "orunla_mcp")]
struct CliArgs {
    /// Transport mode: "stdio" (default) or "sse"
    #[arg(long, default_value = "stdio")]
    transport: String,

    /// Port for SSE transport (default: 8080)
    #[arg(long, default_value_t = 8080)]
    port: u16,

    /// Also serve REST API routes alongside MCP SSE (unified server mode)
    #[arg(long)]
    with_api: bool,

    /// API key for REST API authentication (only used with --with-api)
    #[arg(long)]
    api_key: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = CliArgs::parse();

    let config = StorageConfig::default();
    let storage = SqliteStorage::new(config.clone());
    storage.init()?;

    let server = MCPServer::new(storage);

    match args.transport.as_str() {
        "sse" => {
            if args.with_api {
                let api_storage = SqliteStorage::new(config.clone());
                api_storage.init()?;
                orunla::unified_server::start_unified_server(
                    api_storage,
                    server,
                    args.port,
                    args.api_key,
                )
                .await?;
            } else {
                orunla::mcp::run_sse(server, args.port).await?;
            }
        }
        _ => orunla::mcp::run_stdio(server).await?,
    }

    Ok(())
}
