# Orunla MCP Server Guide

Orunla includes a **Model Context Protocol (MCP)** server, allowing AI agents (like Claude Desktop, Claude Code, Cursor, Cline) to directly read, write, and manage your local memory graph.

---

## Installation

### Claude Desktop

**Windows** — edit `%APPDATA%\Claude\claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "orunla": {
      "command": "C:/Users/YourName/Orunla_v0.1.1_bundle/windows/orunla_mcp.exe"
    }
  }
}
```

**macOS** — edit `~/Library/Application Support/Claude/claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "orunla": {
      "command": "/Users/YourUsername/Orunla_v0.1.1_bundle/macOS/orunla-mac-aarch64/orunla_mcp",
      "env": {
        "ORT_DYLIB_PATH": "/Users/YourUsername/Orunla_v0.1.1_bundle/macOS/orunla-mac-aarch64/libonnxruntime.dylib"
      }
    }
  }
}
```

Use `orunla-mac-x86_64` instead if you're on an Intel Mac.

Replace the paths with the actual location of your Orunla folder. Restart Claude Desktop after saving.

---

### Claude Code

**Windows** — edit `%USERPROFILE%\.claude\settings.json`:

```json
{
  "mcpServers": {
    "orunla": {
      "command": "C:/Users/YourName/Orunla_v0.1.1_bundle/windows/orunla_mcp.exe"
    }
  }
}
```

**macOS** — edit `~/.claude/settings.json`:

```json
{
  "mcpServers": {
    "orunla": {
      "command": "/Users/YourUsername/Orunla_v0.1.1_bundle/macOS/orunla-mac-aarch64/orunla_mcp",
      "env": {
        "ORT_DYLIB_PATH": "/Users/YourUsername/Orunla_v0.1.1_bundle/macOS/orunla-mac-aarch64/libonnxruntime.dylib"
      }
    }
  }
}
```

---

### Cursor / Cline / Windsurf

Add the same configuration to your IDE's MCP settings. The command and env values are identical to the examples above.

---

## Important Notes

- **Windows:** The `onnxruntime.dll` must be in the same folder as `orunla_mcp.exe`. Do not move the executable without the DLL.
- **macOS:** The `env` block with `ORT_DYLIB_PATH` is **required**. Without it, the MCP server cannot load the ONNX Runtime.
- **macOS:** Use absolute paths (starting with `/`). Do not use `~` or `$HOME` in JSON config files.
- **Windows:** Use forward slashes (`/`) in JSON config paths, not backslashes.

---

## Available Tools

Once connected, your AI agent will have access to these tools:

### 1. `memory_add`
**Purpose:** Save a new fact to the knowledge graph.
- **Arguments:** `subject` (String), `predicate` (String), `object` (String)
- **Optional:** `text` (String) — source text for context
- **Example prompt:** *"Remember that the client's favorite color is midnight blue."*

### 2. `memory_search`
**Purpose:** Recall facts based on a query. Uses hybrid search (keyword + stability decay).
- **Arguments:** `query` (String), `limit` (Optional Integer)
- **Example prompt:** *"What do you know about the client's preferences?"*

### 3. `memory_get_all`
**Purpose:** Retrieve all memories from the knowledge graph.
- **Arguments:** `limit` (Optional Integer, default 50)

### 4. `memory_get_context`
**Purpose:** Get memories formatted as a context block for injection into prompts.
- **Arguments:** `query` (String)

### 5. `memory_delete`
**Purpose:** Remove a specific memory by its ID.
- **Arguments:** `id` (String)

### 6. `memory_purge_topic`
**Purpose:** Delete all memories related to a specific keyword or topic.
- **Arguments:** `query` (String)
- **Example prompt:** *"Clear all my notes about Project X."*

### 7. `memory_gc` (Sustainability)
**Purpose:** Manually trigger garbage collection to prune old, decayed memories.
- **Arguments:** `threshold` (Optional Float, default 0.05)
- **Example prompt:** *"Clean up old memories with a strength below 0.05."*

### 8. `memory_sync_chat`
**Purpose:** Sync chat history and automatically extract memories from messages.
- **Arguments:** `messages` (Array of `{role, content}` objects)

---

## Example Prompts for Agents

You can talk to your agent naturally, and it will use these tools in the background:

- "Keep track of the fact that I take my vitamins at 8 AM."
- "Based on what you remember, what are the travel requirements for my trip next week?"
- "Forget everything we discussed about the old marketing strategy."
- "Optimize my memory database by running a garbage collection."

---

## Privacy Note

The MCP server communicates over standard input/output (Stdio). **No data is ever sent to external servers.** All memory persists only in your local database:
- macOS: `~/.orunla/memory.db`
- Windows: `%USERPROFILE%\.orunla\memory.db`
