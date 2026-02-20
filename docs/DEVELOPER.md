# Orunla Developer Reference

This document is for developers who want to run Orunla without the desktop app, use the CLI tools, access the local API directly, set up tunnels, or integrate Orunla into custom applications.

**Most users don't need this.** If you just want to connect an AI tool to Orunla, see `AI_SETUP.md` and `API_REFERENCE.md`.

---

## Running the REST API Server (Standalone)

The CLI can run a standalone REST API server without the desktop app:

**Windows:**
```powershell
.\orunla_cli.exe serve --port 3000
```

**macOS:**
```bash
./orunla_cli serve --port 3000
```

With API key protection:
```bash
./orunla_cli serve --port 3000 --api-key "your-secret-key"
```

Or via environment variable:
```bash
export ORUNLA_API_KEY="your-secret-key"
./orunla_cli serve --port 3000
```

The server runs at `http://localhost:3000`. All endpoints from `API_REFERENCE.md` are available at this local URL.

**Note:** When the desktop app is running, the REST API is already available on port 8080. You don't need both.

---

## Local API Base URL

| Source | URL |
|--------|-----|
| Desktop app | `http://localhost:8080` |
| CLI `serve` | `http://localhost:{port}` (default 3000) |

All endpoints documented in `API_REFERENCE.md` work with these local URLs. Just replace the relay URL with the local one.

---

## File Upload (Local Only)

File upload is only available via the local API, not the relay.

### `POST /ingest-file`

Upload a `.txt`, `.md`, `.csv`, or `.json` file.

```bash
curl -X POST http://localhost:8080/ingest-file \
  -H "X-API-Key: your-key" \
  -F "file=@document.txt"
```

**Response:**
```json
{
  "status": "ok",
  "file": "document.txt",
  "chunks_processed": 5,
  "total_triplets_added": 12
}
```

Max file size: 50 MB.

---

## Exposing to the Internet (Tunnels)

If you're running the CLI server and need cloud services (ChatGPT, n8n, etc.) to reach it, use a tunnel.

### Cloudflare Tunnel (free, unlimited)

```bash
cloudflared tunnel --url http://localhost:3000
```

Install from [developers.cloudflare.com/cloudflare-one/connections/connect-networks/downloads](https://developers.cloudflare.com/cloudflare-one/connections/connect-networks/downloads/).

This gives you a public URL like `https://some-words-here.trycloudflare.com`. Use this URL in place of `localhost:3000` in any integration.

**Note:** The URL changes every restart. For a permanent URL, set up a [named Cloudflare Tunnel](https://developers.cloudflare.com/cloudflare-one/connections/connect-networks/).

### ngrok

```bash
ngrok http 3000
```

Free tier is limited to 1 concurrent connection.

---

## MCP Server (stdio)

The MCP server provides a direct connection for AI tools that support the Model Context Protocol (Claude Code, Cursor, Cline, Windsurf, etc.).

**Does not require the desktop app.** The standalone `orunla_mcp` binary works independently.

See `MCP.md` for full configuration details.

### Quick Setup

Add to your tool's MCP config:

**Windows:**
```json
{
  "mcpServers": {
    "orunla": {
      "command": "C:\\path\\to\\orunla_mcp.exe",
      "args": ["--transport", "stdio"]
    }
  }
}
```

**macOS:**
```json
{
  "mcpServers": {
    "orunla": {
      "command": "/path/to/orunla_mcp",
      "args": ["--transport", "stdio"],
      "env": {
        "ORT_DYLIB_PATH": "/path/to/libonnxruntime.dylib"
      }
    }
  }
}
```

---

## MCP Server (SSE Mode)

For browser-based MCP clients without the desktop app relay:

**Windows:**
```powershell
.\orunla_mcp.exe --transport sse --port 8080
```

**macOS:**
```bash
export ORT_DYLIB_PATH="$(pwd)/libonnxruntime.dylib"
./orunla_mcp --transport sse --port 8080
```

Then expose via tunnel and point your MCP client to `https://your-tunnel-url/sse`.

---

## CLI Commands

### Ingestion

```bash
# Ingest text directly
orunla_cli ingest "Sarah manages marketing and reports to David."

# Ingest from a file
orunla_cli ingest-file document.txt
```

### Recall

```bash
orunla_cli recall "Who is Sarah?"
orunla_cli recall "project status" --limit 20
```

### Deletion

```bash
# Delete by topic
orunla_cli purge "old project"

# Delete by ID
orunla_cli delete <memory-id>
```

### Maintenance

```bash
# Garbage collection (remove decayed memories)
orunla_cli gc --threshold 0.05

# Deduplicate nodes
orunla_cli dedup
```

### Licensing

```bash
# Check license status
orunla_cli license

# Activate a Pro license
orunla_cli activate <license-key>
```

---

## Security & Rate Limits

### Authentication Methods

When API key auth is enabled:

**Bearer token:**
```bash
curl -H "Authorization: Bearer your-key" \
     -X POST http://localhost:3000/ingest \
     -H "Content-Type: application/json" \
     -d '{"text": "test"}'
```

**X-API-Key header:**
```bash
curl -H "X-API-Key: your-key" \
     -X POST http://localhost:3000/ingest \
     -H "Content-Type: application/json" \
     -d '{"text": "test"}'
```

### Public vs Protected Endpoints

| Endpoint | Auth Required |
|----------|---------------|
| `GET /health` | No |
| `GET /stats` | No |
| `POST /ingest` | Yes |
| `POST /ingest-file` | Yes |
| `POST /recall` | Yes |
| `DELETE /memories/:id` | Yes |
| `POST /memories/purge` | Yes |
| `POST /gc` | Yes |

### Limits

| Limit | Value |
|-------|-------|
| Local rate limit | 60 requests/min per IP |
| Relay rate limit | 120 requests/min per IP |
| Text input | 1 MB max |
| File upload | 50 MB max |
| Query length | 10 KB max |
| Results per query | 10,000 max |
| Relay timeout | 30 seconds |

---

## App Configuration

### API Key (Desktop App)

Set in the desktop app's **API Key** settings panel, or manually edit:

**Windows:** `%USERPROFILE%\.orunla\config.json`
**macOS:** `~/.orunla/config.json`

```json
{
  "api_key": "your-secret-key"
}
```

Requires an app restart to take effect.

### Database Location

The memory database is stored at:

**Windows:** `%USERPROFILE%\.orunla\memory.db`
**macOS:** `~/.orunla/memory.db`

---

## Google AI Studio (Gemini Function Declarations)

For developers building with the Gemini API, define Orunla's endpoints as function declarations:

```json
{
  "function_declarations": [
    {
      "name": "save_memory",
      "description": "Save a fact to the Orunla knowledge graph.",
      "parameters": {
        "type": "object",
        "properties": {
          "text": {
            "type": "string",
            "description": "The fact to save"
          }
        },
        "required": ["text"]
      }
    },
    {
      "name": "recall_memories",
      "description": "Search the Orunla knowledge graph for relevant memories.",
      "parameters": {
        "type": "object",
        "properties": {
          "query": {
            "type": "string",
            "description": "Search query"
          },
          "limit": {
            "type": "integer",
            "description": "Max results (default 10)"
          }
        },
        "required": ["query"]
      }
    },
    {
      "name": "purge_memories",
      "description": "Delete all memories matching a topic.",
      "parameters": {
        "type": "object",
        "properties": {
          "query": {
            "type": "string",
            "description": "Topic to purge"
          }
        },
        "required": ["query"]
      }
    }
  ]
}
```

Map these to HTTP requests:
- `save_memory` → `POST /ingest` with `{"text": "..."}`
- `recall_memories` → `POST /recall` with `{"query": "...", "limit": N}`
- `purge_memories` → `POST /memories/purge` with `{"query": "..."}`

---

## Background Sync (Pro)

With a Pro license, memories sync automatically across devices every 30 seconds. No API endpoints needed — sync is fully automatic when the desktop app or CLI server is running.

```bash
orunla_cli license    # Check sync status
orunla_cli activate <key>  # Activate Pro
```
