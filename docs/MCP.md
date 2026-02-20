# Orunla MCP Server Guide

Orunla includes a **Model Context Protocol (MCP)** server, allowing AI agents (like Claude Desktop, Claude Code, Cursor, Cline) to directly read, write, and manage your local memory graph.

---

## Finding Config File Locations

Before setting up Orunla, you'll need to find where your config files live.

### Windows

**What is `%APPDATA%`?**
This is a shortcut to your user's application data folder. To open it:
1. Press `Win + R` to open the Run dialog
2. Type `%APPDATA%` and press Enter
3. This opens `C:\Users\YourName\AppData\Roaming\`

**What is `%USERPROFILE%`?**
This is your home folder. To open it:
1. Press `Win + R` to open the Run dialog
2. Type `%USERPROFILE%` and press Enter
3. This opens `C:\Users\YourName\`

**Tip:** You can also paste these paths directly into File Explorer's address bar.

### macOS

**What is `~`?**
The tilde (`~`) means your home folder: `/Users/YourUsername/`

**How to access `~/Library/Application Support/`:**
1. Open Finder
2. Click **Go** in the menu bar
3. Hold the **Option** key — "Library" will appear in the menu
4. Click **Library**, then navigate to **Application Support**

Or in Terminal: `open ~/Library/Application\ Support/`

**How to access `~/.claude/`:**
Folders starting with `.` are hidden by default. In Terminal:
```bash
open ~/.claude
```
Or press `Cmd + Shift + .` in Finder to show hidden files.

---

## If the Config File Doesn't Exist

**Create it!** If the file doesn't exist, simply create a new text file with the exact name specified.

**Windows example:**
1. Navigate to `%APPDATA%\Claude\`
2. If the `Claude` folder doesn't exist, create it
3. Create a new file called `claude_desktop_config.json`
4. Open it with Notepad and paste the configuration

**macOS example:**
1. Navigate to `~/.claude/`
2. If the `.claude` folder doesn't exist, create it: `mkdir ~/.claude`
3. Create the file: `touch ~/.claude/mcp_settings.json`
4. Edit with any text editor

**Important:** Make sure the file has the correct extension (`.json`, not `.json.txt`). On Windows, you may need to enable "Show file extensions" in File Explorer settings.

---

## Installation

### Claude Browser (via Cloud Relay) — Easiest Setup

The simplest way to use Orunla with Claude is through the desktop app's built-in cloud relay. No tunnels, no port forwarding, no configuration files.

1. Open **Orunla.exe** (Windows) on your computer
2. Look at the **Server Status** card in the app — copy the **Relay URL**
3. In Claude browser, go to **Settings → MCP Connectors → Add**
4. Paste the relay URL and save
5. Claude can now read and write your memories

The relay URL is stable — you configure it once and it works whenever the desktop app is open. The relay is free for all users.

**How it works:** The desktop app connects outbound to a cloud relay via WebSocket. Claude browser connects to the relay via standard MCP SSE protocol. The relay forwards messages between them. Your memory data is processed locally — the relay only forwards MCP protocol messages.

---

> **Note:** The stdio MCP server (`orunla_mcp` / `orunla_mcp.exe`) works **independently of the desktop app**. You do NOT need the desktop app running to use Orunla with Claude Code, Cursor, Cline, or any MCP client that supports stdio transport. The desktop app is only required for the cloud relay (browser-based MCP clients).

> **REST API Relay (v0.4.0):** The desktop app also provides a cloud REST API relay for external services like ChatGPT Custom GPTs, n8n, and Make.com. See `AI_SETUP.md` for details.

---

### Claude Desktop

**Windows** — edit `%APPDATA%\Claude\claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "orunla": {
      "command": "C:\\Users\\YourName\\Orunla_Windows_v0.4.1\\orunla_mcp.exe",
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
      "command": "/Users/YourUsername/Orunla_macOS_v0.4.1/orunla-mac-aarch64/orunla_mcp",
      "env": {
        "ORT_DYLIB_PATH": "/Users/YourUsername/Orunla_macOS_v0.4.1/orunla-mac-aarch64/libonnxruntime.dylib"
      }
    }
  }
}
```

Use `orunla-mac-x86_64` instead if you're on an Intel Mac.

Replace the paths with the actual location of your Orunla folder. Restart Claude Desktop after saving.

---

### Claude Code (VSCode Extension)

**Windows** — edit `%USERPROFILE%\.claude\mcp_settings.json`:

```json
{
  "mcpServers": {
    "orunla": {
      "command": "C:\\Users\\YourName\\Orunla_Windows_v0.4.1\\orunla_mcp.exe",
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
      "command": "/Users/YourUsername/Orunla_macOS_v0.4.1/orunla-mac-aarch64/orunla_mcp",
      "env": {
        "ORT_DYLIB_PATH": "/Users/YourUsername/Orunla_macOS_v0.4.1/orunla-mac-aarch64/libonnxruntime.dylib"
      }
    }
  }
}
```

---

### Cursor / Windsurf / Cline

These IDEs use the same MCP config format as Claude Desktop. Add the configuration to your IDE's MCP settings file:

| IDE | Config File (Windows) | Config File (macOS) |
|-----|----------------------|---------------------|
| **Cursor** | `%USERPROFILE%\.cursor\mcp.json` | `~/.cursor/mcp.json` |
| **Windsurf** | `%USERPROFILE%\.windsurf\mcp.json` | `~/.windsurf/mcp.json` |
| **Cline** | VSCode Settings → Cline → MCP Servers | Same |

The `command`, `args`, and `env` values are identical to the Claude Desktop examples above.

---

### OpenCode

**Windows** — create or edit `opencode.json` in your project root:

```json
{
  "$schema": "https://opencode.ai/config.json",
  "mcp": {
    "orunla": {
      "type": "local",
      "command": [
        "C:\\Users\\YourName\\Orunla_Windows_v0.4.1\\orunla_mcp.exe"
      ],
      "enabled": true
    }
  }
}
```

**macOS** — create or edit `opencode.json` in your project root:

```json
{
  "$schema": "https://opencode.ai/config.json",
  "mcp": {
    "orunla": {
      "type": "local",
      "command": [
        "/Users/YourUsername/Orunla_macOS_v0.4.1/orunla-mac-aarch64/orunla_mcp"
      ],
      "env": {
        "ORT_DYLIB_PATH": "/Users/YourUsername/Orunla_macOS_v0.4.1/orunla-mac-aarch64/libonnxruntime.dylib"
      },
      "enabled": true
    }
  }
}
```

**Note:** OpenCode uses an array for `command` instead of a plain string. The `"type": "local"` and `"enabled": true` fields are required.

**Optional — Agent Prompt:** OpenCode supports custom agent prompts that instruct the AI to use Orunla proactively. Add an `agent` block to your `opencode.json`:

```json
{
  "agent": {
    "build": {
      "prompt": "You have access to Orunla memory tools. Use them proactively:\n\n- At the start of each conversation, use memory_search to recall relevant context\n- When the user shares important facts, preferences, or decisions, use memory_add to save them\n- When the user says \"remember\", \"don't forget\", or \"for future reference\", always save to memory\n- Structure memories as subject-predicate-object triplets (e.g., \"ProjectX\" - \"uses\" - \"React 18\")"
    }
  }
}
```

---

## Important Notes

- **Windows:** The `onnxruntime.dll` must be in the same folder as `orunla_mcp.exe`. Do not move the executable without the DLL.
- **macOS:** The `env` block with `ORT_DYLIB_PATH` is **required**. Without it, the MCP server cannot load the ONNX Runtime.
- **macOS:** Use absolute paths (starting with `/`). Do not use `~` or `$HOME` in JSON config files.
- **Windows:** Both forward slashes (`/`) and escaped backslashes (`\\`) work in JSON config paths. We recommend backslashes with proper escaping for consistency with Windows conventions.

---

## Troubleshooting

### MCP Tools Not Appearing

1. **Verify the path is correct** — Make sure the path to `orunla_mcp.exe` exists and is spelled correctly.
2. **Restart your IDE** — After editing the config file, restart Claude Code/VSCode completely.
3. **Check the MCP server logs** — In VSCode, open Output panel (View → Output) and select "MCP Servers" from the dropdown.

### Manual Testing

You can test the MCP server directly in PowerShell:

```powershell
cd C:\Users\YourName\Orunla_Windows_v0.4.1
.\orunla_mcp.exe
```

If it starts correctly, you'll see:
```
[orunla] License: trial | Sync: enabled
```

The server then waits for JSON-RPC input on stdin. Press Ctrl+C to exit.

### Common Windows Issues

**"Path not found" errors:**
- Ensure your Orunla folder path has no special characters
- Try using the full path with escaped backslashes (`\\`)

**DLL loading errors:**
- Make sure `onnxruntime.dll` is in the same folder as the executable
- Do not move `orunla_mcp.exe` without also moving the DLL

### Advanced Configuration (Windows)

If you encounter issues, you can try adding optional parameters:

```json
{
  "mcpServers": {
    "orunla": {
      "command": "C:\\Users\\YourName\\Orunla_Windows_v0.4.1\\orunla_mcp.exe",
      "args": [],
      "cwd": "C:\\Users\\YourName\\Orunla_Windows_v0.4.1",
      "env": {
        "USERPROFILE": "C:\\Users\\YourName",
        "HOME": "C:\\Users\\YourName"
      }
    }
  }
}
```

- `cwd` — Sets the working directory for the MCP server
- `env` — Explicitly provides environment variables

These are usually not needed, but can help in edge cases.

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

## Making Orunla Work Autonomously

By default, AI agents won't use Orunla unless you explicitly ask them to. To make your agent **proactively** save and recall memories, add instructions to your system prompt or `CLAUDE.md` file.

> **Not using MCP?** If you use ChatGPT, Gemini, or another AI that doesn't support MCP, see `AI_SETUP.md` for setup guides using the REST API, including OpenAPI specs for ChatGPT Actions and copy-paste system prompts for any platform.

### For Claude Code

Create or edit `~/.claude/CLAUDE.md` (macOS) or `%USERPROFILE%\.claude\CLAUDE.md` (Windows):

```markdown
## Memory Management (Orunla MCP)

**IMPORTANT**: Orunla is your long-term memory system. Use it PROACTIVELY.

### Session Start (MANDATORY)
1. ALWAYS search memory first: `memory_search` with project name or key terms
2. Review memories to understand prior work, decisions, and context
3. Greet user with context: "I recall that [project] uses [tech]..."

### When to Save Memories (Be Aggressive)
Save memories using `memory_add` whenever you learn:

**Constants (persistent facts)**:
- Project names and what they do
- Tech stack choices and configurations
- API keys, URLs, ports, folder structures
- Platform-specific requirements or workarounds

**Preferences (user choices)**:
- Coding style preferences
- Preferred libraries or approaches
- Naming conventions

**Context (session-relevant)**:
- Current feature being worked on
- Bugs encountered and their fixes
- Architectural decisions made

### Trigger Phrases (Save Memory When You Hear):
- "Remember that..."
- "For future reference..."
- "This project uses..."
- "We decided to..."
- "Don't forget..."
- Any architectural decision
- Any bug fix that took significant effort

### Memory Quality Guidelines
- Be specific: "ProjectX uses PostgreSQL 15" not "uses a database"
- Include context: Why was this decision made?
- Use consistent subjects: Use project name as subject when possible

### Automatic Behaviors
1. **Session start**: Search Orunla for project memories FIRST
2. **During work**: Save new learnings immediately (don't batch)
3. **After milestones**: Save architectural decisions
```

### For Claude Desktop / Other Agents

Add similar instructions to your system prompt or custom instructions. The key behaviors to instruct:

1. **On session start**: Search memory for relevant context before responding
2. **During conversation**: Save important facts as they come up
3. **Trigger phrases**: Listen for "remember", "don't forget", "for future reference"
4. **Be specific**: Store structured facts, not vague summaries

### Example System Prompt Snippet

```
You have access to Orunla memory tools. Use them proactively:

- At the start of each conversation, use memory_search to recall relevant context
- When the user shares important facts, preferences, or decisions, use memory_add to save them
- When the user says "remember", "don't forget", or "for future reference", always save to memory
- Structure memories as subject-predicate-object triplets (e.g., "ProjectX" - "uses" - "React 18")
- Be specific and include context about why decisions were made
```

---

## Licensing & Sync

The MCP server respects your current license tier:

- **Free / Expired Trial:** All 8 memory tools work normally. Sync is disabled.
- **Trial (14 days):** All tools work. Background sync runs every 30 seconds.
- **Pro:** All tools work. Background sync runs every 30 seconds.

On startup, the MCP server logs your license status to stderr:
```
[orunla] License: pro | Sync: enabled
[orunla] Background sync started (30s interval)
```

To activate Pro, use the CLI:
```bash
orunla_cli activate "your-license-key"
```

---

## Privacy Note

The MCP server communicates over standard input/output (Stdio) when used with Claude Code, Cursor, or other IDEs. When used via the cloud relay (Claude browser), messages are forwarded through the relay server but all memory operations still run locally on your machine.

- **Free tier:** No memory data is ever sent to external servers. The cloud relay only forwards MCP protocol messages — it does not store or read your data.
- **Pro tier with sync:** Only encrypted data (AES-256-GCM) is transmitted to the sync relay. The relay cannot read your memories.

**Database location:**
- macOS: `~/.orunla/memory.db`
- Windows: `%USERPROFILE%\.orunla\memory.db`
