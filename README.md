# Orunla

A local-first AI memory system that stores facts as a knowledge graph on your machine. Orunla extracts entities and relationships from text using on-device AI (GliNER/ONNX), applies Ebbinghaus forgetting curves to keep memories relevant, and provides hybrid retrieval (FTS5 + graph search) ranked by recency and confidence.

[![License](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](LICENSE)

## Features

- **Knowledge graph storage** -- entities, relationships, and facts in SQLite with FTS5 indexing
- **Local AI extraction** -- GliNER runs on-device via ONNX Runtime (no API keys, no cloud, works offline)
- **Memory decay** -- Ebbinghaus forgetting curve prunes stale facts automatically
- **Hybrid search** -- full-text keyword search + graph traversal, ranked by strength
- **Multiple interfaces** -- desktop app (Tauri), CLI, REST API, MCP server
- **MCP integration** -- works with Claude Code, Claude Desktop, Cursor, Cline, Windsurf, and any MCP client
- **100% private** -- all data stays in `~/.orunla/memory.db`

## Quick Start

### Download

Grab the latest release for your platform from the [Releases](https://github.com/anthonylee991/orunla/releases) page. Each zip contains the desktop app, CLI, MCP server, and ONNX runtime.

### CLI

```bash
# Add a memory
orunla_cli ingest "Sarah works at Microsoft and lives in Seattle."

# Search memories
orunla_cli recall "Who is Sarah?"

# Start the REST API server
orunla_cli serve --port 8080
```

### MCP Server (for AI agents)

Add to your MCP config (Claude Code, Cursor, etc.):

```json
{
  "mcpServers": {
    "orunla": {
      "command": "/path/to/orunla_mcp"
    }
  }
}
```

See [MCP.md](docs/MCP.md) for full setup instructions per IDE.

### REST API

```bash
# Save a memory
curl -X POST http://localhost:8080/ingest \
  -H "Content-Type: application/json" \
  -d '{"text": "The deploy key rotates every 90 days."}'

# Search
curl -X POST http://localhost:8080/recall \
  -H "Content-Type: application/json" \
  -d '{"query": "deploy key"}'
```

See [API_REFERENCE.md](docs/API_REFERENCE.md) for all endpoints.

## How It Works

```
Text Input --> GliNER Entity Extraction --> Knowledge Graph (SQLite)
                                                  |
                                           Hybrid Retrieval
                                        (FTS5 + Graph Search)
                                                  |
                                         Ebbinghaus Decay
                                        (rank by strength)
```

1. **Ingestion** -- GliNER extracts entities (people, orgs, locations, concepts) and relationships from text, producing subject-predicate-object triplets
2. **Storage** -- triplets are stored as nodes and edges in SQLite with FTS5 indexing on source text
3. **Retrieval** -- hybrid search combines keyword matching (FTS5) with graph traversal, then ranks by memory strength
4. **Decay** -- strength = `e^(-days/30) * (1 + ln(1 + access_count)) * confidence` -- unused memories fade, frequently accessed ones persist

## Building from Source

### Prerequisites

- **Rust** toolchain (stable) -- [rustup.rs](https://rustup.rs)
- **ONNX Runtime 1.20.0** -- required for GliNER entity extraction
- **Node.js 22+** -- only needed if building the desktop app

### 1. Download ONNX Runtime

Download from [github.com/microsoft/onnxruntime/releases/v1.20.0](https://github.com/microsoft/onnxruntime/releases/tag/v1.20.0):

| Platform | Download | Library file |
|----------|----------|-------------|
| **Windows x64** | `onnxruntime-win-x64-1.20.0.zip` | `lib/onnxruntime.dll` |
| **macOS Apple Silicon** | `onnxruntime-osx-arm64-1.20.0.tgz` | `lib/libonnxruntime.1.20.0.dylib` |
| **macOS Intel** | `onnxruntime-osx-x86_64-1.20.0.tgz` | `lib/libonnxruntime.1.20.0.dylib` |
| **Linux x64** | `onnxruntime-linux-x64-1.20.0.tgz` | `lib/libonnxruntime.so.1.20.0` |

Extract the archive and place the library file where the binary can find it:
- **Windows:** put `onnxruntime.dll` in the same directory as the built `.exe`
- **macOS:** set `ORT_DYLIB_PATH=/path/to/libonnxruntime.dylib`
- **Linux:** set `LD_LIBRARY_PATH` to include the directory, or install to `/usr/local/lib`

### 2. Build

```bash
# CLI and MCP server
cargo build --release --bin orunla_cli
cargo build --release --bin orunla_mcp

# Binaries are in target/release/
```

### 3. (Optional) Build the desktop app

```bash
npm ci
npm run tauri build
```

### 4. Run

The GliNER model (~80MB) downloads automatically from HuggingFace on first run.

```bash
# Copy ONNX runtime next to the binary (Windows example)
cp onnxruntime.dll target/release/

# Test it
./target/release/orunla_cli ingest "Hello world"
```

## Documentation

- [Overview](docs/OVERVIEW.md) -- architecture and design
- [AI Setup](docs/AI_SETUP.md) -- connecting to Claude, ChatGPT, n8n, etc.
- [MCP Guide](docs/MCP.md) -- MCP server configuration per IDE
- [API Reference](docs/API_REFERENCE.md) -- REST endpoints
- [CLI Guide](docs/CLI.md) -- command-line usage
- [Developer Guide](docs/DEVELOPER.md) -- building, tunnels, advanced config

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for development setup and guidelines.

## License

Apache License 2.0. See [LICENSE](LICENSE) for details.
