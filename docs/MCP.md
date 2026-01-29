# Orunla MCP Server Guide

Orunla includes a **Model Context Protocol (MCP)** server, allowing AI agents (like Claude Desktop) to directly read, write, and manage your local memory graph.

---

## 🚀 Installation (Claude Desktop)

To give Claude long-term memory using Orunla, add the following to your `claude_desktop_config.json`:

**Windows Path:** `%APPDATA%\Claude\claude_desktop_config.json`

```json
{
  "mcpServers": {
    "orunla": {
      "command": "C:\\path\\to\\orunla_mcp.exe"
    }
  }
}
```
*Replace `C:\\path\\to\\` with the actual folder where you placed the `orunla_mcp.exe` file.*

---

## 🛠️ Available Tools

Once connected, your AI agent will have access to these specific "skills":

### 1. `memory_add`
**Purpose**: Save a new fact to the knowledge graph.
- **Arguments**: `text` (String)
- **Example**: *"Remember that the client's favorite color is midnight blue."*

### 2. `memory_search`
**Purpose**: Recall facts based on a query. Uses hybrid search (keyword + stability decay).
- **Arguments**: `query` (String), `limit` (Optional Integer)
- **Example**: *"What do you know about the client's preferences?"*

### 3. `memory_delete`
**Purpose**: Remove a specific memory by its ID.
- **Arguments**: `memory_id` (String)

### 4. `memory_purge_topic`
**Purpose**: Delete all memories related to a specific keyword or topic.
- **Arguments**: `query` (String)
- **Example**: *"Clear all my notes about Project X."*

### 5. `memory_gc` (Sustainability)
**Purpose**: Manually trigger garbage collection to prune old, decayed memories.
- **Arguments**: `threshold` (Optional Float, default 0.1)
- **Example**: *"Clean up old memories with a strength below 0.05."*

---

## 💡 Example Prompts for Agents

You can talk to your agent naturally, and it will use these tools in the background:

- "Keep track of the fact that I take my vitamins at 8 AM."
- "Based on what you remember, what are the travel requirements for my trip next week?"
- "Forget everything we discussed about the old marketing strategy."
- "Optimize my memory database by running a garbage collection."

---

## 🛡️ Privacy Note
The MCP server communicates over standard input/output (Stdio). **No data is ever sent to external Orunla servers.** All memory persists only in your local `%USERPROFILE%\.orunla\memory.db` file.
