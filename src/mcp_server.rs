use anyhow::Result;
use clap::Parser;
use orunla::licensing::{LicenseStore, LicenseValidator};
use orunla::mcp::MCPServer;
use orunla::storage::{sqlite::SqliteStorage, Storage, StorageConfig};
use orunla::sync::changelog::ChangelogStore;
use orunla::sync::client::{SyncClient, SyncConfig};

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

    // Initialize licensing and log status to stderr
    let license_store = LicenseStore::new(config.path.clone());
    let tier = LicenseValidator::get_tier_local(&license_store)
        .unwrap_or(orunla::licensing::Tier::Free);
    let sync_status = if tier.allows_sync() {
        "enabled"
    } else {
        "disabled"
    };
    eprintln!("[orunla] License: {} | Sync: {}", tier, sync_status);

    // Start background sync if tier allows it
    if tier.allows_sync() {
        let license = license_store.ensure_license().ok();
        if let Some(license) = license {
            if !license.license_key.is_empty() {
                let sync_storage = SqliteStorage::new(config.clone());
                sync_storage.init().ok();
                let device_id = sync_storage.get_device_id().unwrap_or_default();

                let sync_config = SyncConfig {
                    device_id: device_id.clone(),
                    license_key: license.license_key.clone(),
                    ..SyncConfig::default()
                };

                if let Ok(client) = SyncClient::new(sync_config) {
                    tokio::spawn(async move {
                        let _ = client.register_device().await;

                        let mut storage = sync_storage;
                        let mut interval =
                            tokio::time::interval(tokio::time::Duration::from_secs(30));
                        loop {
                            interval.tick().await;
                            if let Err(e) = client.sync_once(&mut storage).await {
                                eprintln!("[orunla-sync] Error: {}", e);
                            }
                        }
                    });
                    eprintln!("[orunla] Background sync started (30s interval)");
                }
            }
        }
    }

    let server = MCPServer::new(storage);

    match args.transport.as_str() {
        "sse" => {
            if args.with_api {
                // Unified mode: REST API + MCP SSE on one port
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
                // SSE-only mode (backward compatible)
                orunla::mcp::run_sse(server, args.port).await?;
            }
        }
        _ => orunla::mcp::run_stdio(server).await?,
    }

    Ok(())
}
