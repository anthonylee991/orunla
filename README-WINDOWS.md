# Orunla - Windows Installation Guide

> **Hybrid RAG Agent Memory System**
> SQLite-based knowledge graph with text + graph search powered by GliNER

---

## What's Included

```
Orunla_Windows_v0.4.1/
├── Orunla.exe            - Desktop Application (auto-starts MCP + REST server)
├── orunla_cli.exe        - CLI tool (ingest, recall, serve, licensing, sync)
├── orunla_mcp.exe        - Standalone MCP Server (for Claude Code/Cursor/Cline)
├── onnxruntime.dll       - ONNX Runtime (required for GliNER NER engine)
├── README.md             - This file
└── documentation/
    ├── CLI.md            - CLI command reference
    ├── MCP.md            - MCP integration guide
    ├── API_REFERENCE.md  - REST API endpoints
    ├── OVERVIEW.md       - Architecture overview
```

**Important:** Keep all files in the `Orunla_Windows_v0.4.1` folder together. The executables need `onnxruntime.dll` to be in the same directory.

---

## Quick Start

### Option 1: Desktop Application (Recommended)

1. Open the `Orunla_Windows_v0.4.1` folder
2. Double-click **`Orunla.exe`**
3. That's it — everything starts automatically:
   - **Unified server** on port 8080 (REST API + MCP SSE)
   - **Cloud relay** connection for Claude browser access
   - Desktop UI for ingestion, recall, purging, and stats
   - License activation and status

**What's running in the background:**
- `http://localhost:8080/health` — REST API health check
- `http://localhost:8080/sse` — MCP SSE endpoint (for local MCP clients)
- Cloud relay URL — shown in the app's Server Status card (for Claude browser)

Just open the app and everything works. Close the app and the servers stop.

---

### Option 2: Claude Browser (via Cloud Relay)

With the desktop app open, Claude browser can connect to your memory **without any tunnels or port forwarding**.

1. Open **Orunla.exe**
2. Copy the **Relay URL** from the Server Status card in the app
3. In Claude browser, go to **Settings → MCP Connectors → Add**
4. Paste the relay URL
5. Claude can now read and write your memories

The relay URL is stable — configure it once and it works whenever the desktop app is open.

---

### Option 3: MCP Integration (Claude Code / Cursor / Cline)

The standalone MCP server (`orunla_mcp.exe`) gives AI coding assistants direct access to your memory graph via stdio. This works **independently of the desktop app** — you don't need the app open.

**Important:** All paths in MCP configs must use forward slashes (`/`), not backslashes.

#### Claude Desktop

Edit `%APPDATA%\Claude\claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "orunla": {
      "command": "C:/Users/YourName/Orunla_Windows_v0.4.1/orunla_mcp.exe"
    }
  }
}
```

Replace `C:/Users/YourName/` with the actual path to your bundle folder.

Restart Claude Desktop.

#### Claude Code

Edit `%USERPROFILE%\.claude\settings.json`:

```json
{
  "mcpServers": {
    "orunla": {
      "command": "C:/Users/YourName/Orunla_Windows_v0.4.1/orunla_mcp.exe"
    }
  }
}
```

#### Cursor / Cline / Windsurf

Add the same configuration to your IDE's MCP settings. The command is identical.

---

### Option 4: CLI (Terminal)

Open PowerShell and navigate to the Windows folder:

```powershell
cd C:\path\to\Orunla_Windows_v0.4.1
```

Replace `C:\path\to\` with wherever you extracted the bundle (e.g., `C:\Users\YourName\Downloads\`).

Then run commands:

```powershell
# Ingest text
.\orunla_cli.exe ingest "Project Titan is led by Mark."

# Ingest a file
.\orunla_cli.exe ingest-file "C:\path\to\notes.txt"

# Recall memories
.\orunla_cli.exe recall "Titan"
.\orunla_cli.exe recall "Titan" --limit 5 --min-strength 0.2

# View stats
.\orunla_cli.exe stats

# Delete a specific memory
.\orunla_cli.exe delete "uuid-of-memory"

# Purge all memories matching a topic
.\orunla_cli.exe purge "outdated project"

# Garbage collection (remove decayed memories)
.\orunla_cli.exe gc --threshold 0.05

# Deduplicate nodes
.\orunla_cli.exe dedup
```

On first run, the GliNER model (~40MB) will download automatically.

---

### Option 5: REST API Server (Standalone)

If you want to run the REST API on a headless server or without the desktop app:

```powershell
cd C:\path\to\Orunla_Windows_v0.4.1
.\orunla_cli.exe serve --port 3000
```

To secure the API when exposing to a network:

```powershell
.\orunla_cli.exe serve --port 3000 --api-key "your-secret-key"
```

**Note:** When the desktop app is running, the REST API is already available on port 8080 — you don't need to start a separate server.

See `documentation\API_REFERENCE.md` for all endpoints.

#### API Key (v0.4.0)

The desktop app includes an **API Key** settings panel. When set, all REST API requests (both local port 8080 and cloud relay) require the key via one of:

- `X-API-Key: your-key` header
- `Authorization: Bearer your-key` header

**MCP is not affected** — the API key only applies to REST API endpoints. Stdio MCP and SSE MCP connections are not gated by the API key.

To set the API key:
1. Open the desktop app and use the API Key settings panel, **or**
2. Manually edit `%USERPROFILE%\.orunla\config.json` and add `"api_key": "your-key"`

Requires an app restart to take effect.

#### Cloud REST API Relay (v0.4.0)

The desktop app provides a **cloud REST API relay URL**, shown in the "Remote Access" card. External services (ChatGPT Custom GPTs, n8n, Make.com) can use this URL to access your Orunla REST API without setting up tunnels or port forwarding.

- The relay URL works whenever the desktop app is open
- If an API key is set, remote requests must include it
- **File upload** (`/ingest-file`) is not supported via the relay — use the local API (`http://localhost:8080`) for file ingestion

> **Stdio MCP does not require the desktop app.** The standalone `orunla_mcp.exe` works independently — you only need the desktop app for the cloud relay, the web UI, and the REST API server on port 8080.

---

## Licensing

Orunla uses a **Free / Pro** licensing model.

### Free Tier (Default)
- **All local features are free forever** — no time limits, no feature restrictions
- Ingest, recall, search, delete, purge, garbage collection, deduplication
- Desktop app, CLI, MCP server, REST API — all fully functional
- Cloud relay for Claude browser — free for all users
- Your data stays 100% on your machine

### Trial (14 Days)
- On first launch, you automatically get a **14-day free trial of Pro features**
- This includes cross-device sync (see below)
- After the trial, you keep all Free features with no interruption

### Pro Tier
- Adds **cross-device sync**: keep your memories in sync across multiple computers
- Activate with a license key from your purchase email:

```powershell
.\orunla_cli.exe activate "your-license-key-here"
```

- Check your current license status:

```powershell
.\orunla_cli.exe license
```

### Cross-Device Sync (Pro)

Sync your knowledge graph across all your devices automatically:

1. **Activate the same license key** on each device:
   ```powershell
   .\orunla_cli.exe activate "your-license-key"
   ```
2. **Sync happens automatically** every 30 seconds when using the MCP server, REST API server, or desktop app
3. **Manual sync** (one-time push + pull) via CLI:
   ```powershell
   .\orunla_cli.exe sync
   ```

All synced data is **end-to-end encrypted** (AES-256-GCM). The relay server only sees ciphertext — it cannot read your memories.

---

## Where is my data stored?

All data is stored locally:

```
%USERPROFILE%\.orunla\memory.db
```

This SQLite database contains:
- Knowledge graph (nodes + edges)
- FTS5 full-text search index
- Memory metadata, timestamps, and strength scores
- Encrypted license information
- Sync changelog (Pro only)

**Privacy:** All core functionality runs 100% locally. No data is sent to external servers unless you enable Pro sync, in which case only encrypted data is transmitted to the sync relay. The cloud MCP relay only forwards MCP protocol messages — it does not store or read your memory data.

---

## Troubleshooting

### Desktop app won't start
- **Antivirus:** Some antivirus software may block unsigned executables. Add an exception for the folder.
- **Run as Administrator:** Right-click `Orunla.exe` > "Run as administrator"
- **Missing DLL:** Make sure `onnxruntime.dll` is in the same folder as the executables. Do not move executables out of the folder without also moving the DLL.

### MCP not connecting
- Use **forward slashes** in the path (`C:/Users/...` not `C:\Users\...`)
- Use the full path including `Orunla_Windows_v0.4.1/orunla_mcp.exe`
- Restart Claude Desktop / Cursor completely (quit and reopen)
- Check logs in your AI assistant for error messages

### Claude browser relay not connecting
- Make sure the desktop app is open and running
- Check the Server Status card — the relay should show "Available"
- If it shows "No device ID", close and reopen the app
- The relay URL is stable — you only need to configure it once in Claude

### Model download fails
On first run, Orunla downloads the GliNER model (~40MB). If this fails:
- Check your internet connection
- Check you have write access to `%USERPROFILE%\.orunla\`

### Port 8080 already in use
The desktop app uses port 8080 for the unified server. If something else is using this port:
```powershell
netstat -ano | findstr :8080
taskkill /PID <pid> /F
```

### Database errors
Reset by deleting the database:
```powershell
Remove-Item "$env:USERPROFILE\.orunla\memory.db"
```

---

## Documentation

- **CLI Reference:** `documentation\CLI.md`
- **MCP Guide:** `documentation\MCP.md`
- **REST API:** `documentation\API_REFERENCE.md`
- **Architecture:** `documentation\OVERVIEW.md`
