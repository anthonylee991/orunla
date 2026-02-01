# Orunla - Windows Installation Guide

> **Hybrid RAG Agent Memory System**
> SQLite-based knowledge graph with text + graph search powered by GliNER

---

## What's Included

```
Orunla_v0.3.0_bundle/
└── windows/
    ├── Orunla.exe            - Desktop Application
    ├── orunla_cli.exe        - CLI tool (ingest, recall, serve, licensing, sync)
    ├── orunla_mcp.exe        - MCP Server (for Claude/Cursor/Cline)
    ├── onnxruntime.dll       - ONNX Runtime (required for GliNER NER engine)
    ├── README.md             - This file
    └── documentation/
        ├── CLI.md            - CLI command reference
        ├── MCP.md            - MCP integration guide
        ├── API_REFERENCE.md  - REST API endpoints
        ├── OVERVIEW.md       - Architecture overview
        ├── LICENSE
        └── THIRD_PARTY_NOTICES.md
```

**Important:** Keep all files in the `windows` folder together. The executables need `onnxruntime.dll` to be in the same directory.

---

## Quick Start

### Option 1: Desktop Application (Recommended)

1. Open the `windows` folder inside `Orunla_v0.3.0_bundle`
2. Double-click **`Orunla.exe`** to launch the desktop app
3. The app includes:
   - Memory ingestion (text + file upload)
   - Memory recall/search
   - Context purging
   - Real-time stats
   - License activation and status

---

### Option 2: CLI (Terminal)

Open PowerShell and navigate to the Windows folder:

```powershell
cd C:\path\to\Orunla_v0.3.0_bundle\windows
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

### Option 3: REST API Server

Start the server for webhooks, n8n, Make.com, or custom integrations:

```powershell
cd C:\path\to\Orunla_v0.3.0_bundle\windows
.\orunla_cli.exe serve --port 3000
```

To secure the API when exposing to a network:

```powershell
.\orunla_cli.exe serve --port 3000 --api-key "your-secret-key"
```

See `documentation\API_REFERENCE.md` for all endpoints.

---

### Option 4: MCP Integration (Claude Desktop / Cursor / Cline)

The MCP server gives AI assistants direct access to your memory graph.

**Important:** All paths in MCP configs must use forward slashes (`/`), not backslashes.

#### Claude Desktop

Edit `%APPDATA%\Claude\claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "orunla": {
      "command": "C:/Users/YourName/Orunla_v0.3.0_bundle/windows/orunla_mcp.exe"
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
      "command": "C:/Users/YourName/Orunla_v0.3.0_bundle/windows/orunla_mcp.exe"
    }
  }
}
```

#### Cursor / Cline / Windsurf

Add the same configuration to your IDE's MCP settings. The command is identical.

---

## Licensing

Orunla uses a **Free / Pro** licensing model.

### Free Tier (Default)
- **All local features are free forever** — no time limits, no feature restrictions
- Ingest, recall, search, delete, purge, garbage collection, deduplication
- Desktop app, CLI, MCP server, REST API — all fully functional
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

**Privacy:** All core functionality runs 100% locally. No data is sent to external servers unless you enable Pro sync, in which case only encrypted data is transmitted to the sync relay.

---

## Troubleshooting

### Desktop app won't start
- **Antivirus:** Some antivirus software may block unsigned executables. Add an exception for the `windows` folder.
- **Run as Administrator:** Right-click `Orunla.exe` > "Run as administrator"
- **Missing DLL:** Make sure `onnxruntime.dll` is in the same folder as the executables. Do not move executables out of the folder without also moving the DLL.

### MCP not connecting
- Use **forward slashes** in the path (`C:/Users/...` not `C:\Users\...`)
- Use the full path including `Orunla_v0.3.0_bundle/windows/orunla_mcp.exe`
- Restart Claude Desktop / Cursor completely (quit and reopen)
- Check logs in your AI assistant for error messages

### Model download fails
On first run, Orunla downloads the GliNER model (~40MB). If this fails:
- Check your internet connection
- Check you have write access to `%USERPROFILE%\.orunla\`

### Port already in use
```powershell
netstat -ano | findstr :3000
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
