# Orunla Developer Reference

This document is for developers who want to run Orunla without the desktop app, use the CLI tools, access the local API directly, set up tunnels, or integrate Orunla into custom applications.

**Most users don't need this.** If you just want to connect an AI tool to Orunla, see `AI_SETUP.md` and `API_REFERENCE.md`.

---

## Running the REST API Server (Standalone)

The CLI can run a standalone REST API server without the desktop app:

```bash
orunla_cli serve --port 3000
```

With API key protection:
```bash
orunla_cli serve --port 3000 --api-key "your-secret-key"
```

Or via environment variable:
```bash
export ORUNLA_API_KEY="your-secret-key"
orunla_cli serve --port 3000
```

The server runs at `http://localhost:3000`. All endpoints from `API_REFERENCE.md` are available at this local URL.

**Note:** When the desktop app is running, the REST API is already available on port 8080. You don't need both.

---

## Local API Base URL

| Source | URL |
|--------|-----|
| Desktop app | `http://localhost:8080` |
| CLI `serve` | `http://localhost:{port}` (default 7432) |

---

## Exposing to the Internet (Tunnels)

If you need cloud services (ChatGPT, n8n, etc.) to reach your local Orunla, use a tunnel.

### Cloudflare Tunnel (free, unlimited)

```bash
cloudflared tunnel --url http://localhost:8080
```

This gives you a public URL like `https://some-words-here.trycloudflare.com`. Use this in place of `localhost:8080` in any integration.

### ngrok

```bash
ngrok http 8080
```

---

## MCP Server (stdio)

The MCP server provides a direct connection for AI tools that support the Model Context Protocol (Claude Code, Cursor, Cline, Windsurf, etc.).

**Does not require the desktop app.** The standalone `orunla_mcp` binary works independently.

See `MCP.md` for full configuration details.

### Quick Setup

Add to your tool's MCP config:

```json
{
  "mcpServers": {
    "orunla": {
      "command": "/path/to/orunla_mcp",
      "args": []
    }
  }
}
```

On macOS, add `ORT_DYLIB_PATH` to the `env` block pointing to `libonnxruntime.dylib`.

---

## MCP Server (SSE Mode)

For browser-based MCP clients:

```bash
orunla_mcp --transport sse --port 8080
```

Then expose via tunnel and point your MCP client to `https://your-tunnel-url/sse`.

### Unified Mode (REST API + MCP SSE)

```bash
orunla_mcp --transport sse --port 8080 --with-api
```

This serves both the REST API and MCP SSE on a single port.

---

## CLI Commands

### Ingestion

```bash
orunla_cli ingest "Sarah manages marketing and reports to David."
orunla_cli ingest-file document.txt
```

### Recall

```bash
orunla_cli recall "Who is Sarah?"
orunla_cli recall "project status" --limit 20
```

### Deletion

```bash
orunla_cli purge "old project"
orunla_cli delete <memory-id>
```

### Maintenance

```bash
orunla_cli gc --threshold 0.05
orunla_cli dedup
```

---

## Security & Rate Limits

### Authentication Methods

When API key auth is enabled:

**Bearer token:**
```bash
curl -H "Authorization: Bearer your-key" \
     -X POST http://localhost:8080/ingest \
     -H "Content-Type: application/json" \
     -d '{"text": "test"}'
```

**X-API-Key header:**
```bash
curl -H "X-API-Key: your-key" \
     -X POST http://localhost:8080/ingest \
     -H "Content-Type: application/json" \
     -d '{"text": "test"}'
```

### Limits

| Limit | Value |
|-------|-------|
| Rate limit | 60 requests/min per IP |
| Text input | 1 MB max |
| File upload | 50 MB max |
| Query length | 10 KB max |
| Results per query | 10,000 max |

---

## App Configuration

### API Key

Set via the desktop app's **API Key** settings panel, or manually edit:

`~/.orunla/config.json` (or `%USERPROFILE%\.orunla\config.json` on Windows):

```json
{
  "api_key": "your-secret-key"
}
```

Requires an app restart to take effect.

### Database Location

`~/.orunla/memory.db` (or `%USERPROFILE%\.orunla\memory.db` on Windows)

---

## Building from Source

```bash
# Build CLI and MCP server
cargo build --release --bin orunla_cli
cargo build --release --bin orunla_mcp

# Build Tauri desktop app
npm ci
npm run tauri build
```

Requires: Rust toolchain, Node.js, ONNX Runtime (see CI workflow for details).
