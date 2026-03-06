# Orunla MCP Server Guide

Orunla includes a **Model Context Protocol (MCP)** server, allowing AI agents (like Claude Desktop, Claude Code, Cursor, Cline) to directly read, write, and manage your local memory graph.

---

## Finding Config File Locations

### Windows

**What is `%APPDATA%`?**
Press `Win + R`, type `%APPDATA%`, press Enter. Opens `C:\Users\YourName\AppData\Roaming\`

**What is `%USERPROFILE%`?**
Press `Win + R`, type `%USERPROFILE%`, press Enter. Opens `C:\Users\YourName\`

### macOS

**`~`** means your home folder: `/Users/YourUsername/`

**Hidden folders** (starting with `.`): Press `Cmd + Shift + .` in Finder, or use Terminal: `open ~/.claude`

---

## Installation

> The stdio MCP server (`orunla_mcp`) works **independently of the desktop app**. You do NOT need the desktop app running to use Orunla with Claude Code, Cursor, Cline, or any MCP client that supports stdio transport.

### Claude Desktop

**Windows** — edit `%APPDATA%\Claude\claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "orunla": {
      "command": "C:\\path\\to\\orunla_mcp.exe",
      "args": []
    }
  }
}
```

**macOS** — edit `~/Library/Application Support/Claude/claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "orunla": {
      "command": "/path/to/orunla_mcp",
      "env": {
        "ORT_DYLIB_PATH": "/path/to/libonnxruntime.dylib"
      }
    }
  }
}
```

Replace the paths with the actual location of your Orunla binaries. Restart Claude Desktop after saving.

---

### Claude Code (VSCode Extension)

**Windows** — edit `%USERPROFILE%\.claude\mcp_settings.json`:

```json
{
  "mcpServers": {
    "orunla": {
      "command": "C:\\path\\to\\orunla_mcp.exe",
      "args": []
    }
  }
}
```

**macOS** — edit `~/.claude/mcp_settings.json`:

```json
{
  "mcpServers": {
    "orunla": {
      "command": "/path/to/orunla_mcp",
      "env": {
        "ORT_DYLIB_PATH": "/path/to/libonnxruntime.dylib"
      }
    }
  }
}
```

---

### Cursor / Windsurf / Cline

These IDEs use the same MCP config format. Add the configuration to your IDE's MCP settings file:

| IDE | Config File (Windows) | Config File (macOS) |
|-----|----------------------|---------------------|
| **Cursor** | `%USERPROFILE%\.cursor\mcp.json` | `~/.cursor/mcp.json` |
| **Windsurf** | `%USERPROFILE%\.windsurf\mcp.json` | `~/.windsurf/mcp.json` |
| **Cline** | VSCode Settings > Cline > MCP Servers | Same |

The `command`, `args`, and `env` values are identical to the Claude Desktop examples above.

---

### OpenCode

Create or edit `opencode.json` in your project root:

```json
{
  "$schema": "https://opencode.ai/config.json",
  "mcp": {
    "orunla": {
      "type": "local",
      "command": ["/path/to/orunla_mcp"],
      "enabled": true
    }
  }
}
```

On macOS, add `"env": { "ORT_DYLIB_PATH": "/path/to/libonnxruntime.dylib" }`.

---

## SSE Transport (Browser Agents & Remote Access)

The MCP server also supports **SSE (Server-Sent Events)** transport, which lets browser-based agents and remote clients connect over HTTP instead of stdio.

### Start the SSE Server

```bash
# SSE-only (MCP endpoints at /sse and /message)
orunla_mcp --transport sse --port 8080

# Unified mode: MCP SSE + REST API on the same port
orunla_mcp --transport sse --port 8080 --with-api

# With API key protection (recommended for any network exposure)
orunla_mcp --transport sse --port 8080 --with-api --api-key "your-secret-key"
```

### Connect Browser-Based MCP Clients

**Claude Browser (claude.ai):**
1. Start the SSE server (see above)
2. Expose via tunnel:
   ```bash
   # Cloudflare Tunnel (free)
   cloudflared tunnel --url http://localhost:8080

   # Or ngrok
   ngrok http 8080
   ```
3. In Claude browser: **Settings > Integrations > Add**
4. Paste your tunnel URL with `/sse` appended (e.g., `https://your-tunnel.trycloudflare.com/sse`)

**ChatGPT and other REST-based agents:** Use the unified mode (`--with-api`) and connect via REST endpoints. See `AI_SETUP.md` for OpenAPI spec and setup instructions.

### Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/sse` | GET | SSE event stream (MCP client connects here) |
| `/message` | POST | JSON-RPC messages from MCP client |
| `/health` | GET | Health check (unified mode only) |
| `/ingest` | POST | Save memory via REST (unified mode only) |
| `/recall` | POST | Search memories via REST (unified mode only) |

### Security

When exposing Orunla over a network (even via tunnel):
- **Always use `--api-key`** to protect REST endpoints
- MCP SSE endpoints use the MCP protocol's own session management
- The server binds to `127.0.0.1` by default (localhost only) — use a tunnel for remote access rather than binding to `0.0.0.0`

---

## Important Notes

- **Windows:** The `onnxruntime.dll` must be in the same folder as `orunla_mcp.exe`.
- **macOS:** The `env` block with `ORT_DYLIB_PATH` is **required**.
- **macOS:** Use absolute paths (starting with `/`). Do not use `~` or `$HOME` in JSON config files.
- **Windows:** Both forward slashes and escaped backslashes (`\\`) work in JSON config paths.

---

## Available Tools

Once connected, your AI agent will have access to these tools:

### 1. `memory_add`
Save a new fact to the knowledge graph.
- **Arguments:** `subject`, `predicate`, `object` (all String)
- **Optional:** `text` (source text), `memory_type` (constant/context/preference)

### 2. `memory_search`
Recall facts based on a query. Uses hybrid search (keyword + stability decay).
- **Arguments:** `query` (String), `limit` (Optional Integer)

### 3. `memory_get_all`
Retrieve all memories from the knowledge graph.
- **Arguments:** `limit` (Optional Integer, default 50)

### 4. `memory_get_context`
Get memories formatted as a context block for injection into prompts.
- **Arguments:** `query` (String)

### 5. `memory_delete`
Remove a specific memory by its ID.
- **Arguments:** `id` (String)

### 6. `memory_purge_topic`
Delete all memories related to a specific keyword or topic.
- **Arguments:** `query` (String)

### 7. `memory_gc`
Manually trigger garbage collection to prune old, decayed memories.
- **Arguments:** `threshold` (Optional Float, default 0.05)

### 8. `memory_sync_chat`
Sync chat history and automatically extract memories from messages.
- **Arguments:** `messages` (Array of `{role, content}` objects)

---

## Making Orunla Work Autonomously

By default, AI agents won't use Orunla unless you explicitly ask them to. To make your agent **proactively** save and recall memories, add instructions to your system prompt or `CLAUDE.md` file.

See `AI_SETUP.md` for ready-to-use system prompts and CLAUDE.md templates.

---

## Troubleshooting

### MCP Tools Not Appearing

1. **Verify the path is correct** — Make sure the path to `orunla_mcp` exists.
2. **Restart your IDE** — After editing the config file, restart completely.
3. **Check logs** — In VSCode, open Output panel and select "MCP Servers".

### Manual Testing

```bash
./orunla_mcp
```

The server waits for JSON-RPC input on stdin. Press Ctrl+C to exit.

---

## Privacy Note

The MCP server communicates over standard input/output (stdio). All memory operations run locally on your machine.

**Database location:** `~/.orunla/memory.db` (or `%USERPROFILE%\.orunla\memory.db` on Windows)
