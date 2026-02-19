# Orunla - Mac Installation Guide

> **Hybrid RAG Agent Memory System**
> SQLite-based knowledge graph with text + graph search powered by GliNER

---

## What's Included

```
Orunla_macOS_v0.3.4/
├── orunla-mac-aarch64/    ← Apple Silicon (M1/M2/M3/M4)
└── orunla-mac-x86_64/     ← Intel Macs

Each architecture folder contains:
├── orunla_cli            - CLI tool (ingest, recall, serve, licensing, sync)
├── orunla_mcp            - MCP server (for Claude/Cursor/Cline)
├── libonnxruntime.dylib  - ONNX Runtime (required for GliNER NER engine)
├── launch_orunla.sh      - Browser UI launcher (starts server + opens browser)
├── ui/
│   ├── index.html        - Web dashboard
│   └── main.js           - Dashboard logic
├── README.md             - This file
└── documentation/
    ├── CLI.md            - CLI command reference
    ├── MCP.md            - MCP integration guide
    ├── API_REFERENCE.md  - REST API endpoints
    ├── AI_SETUP.md       - Connect any AI chatbot (ChatGPT, Gemini, Claude browser)
    ├── OVERVIEW.md       - Architecture overview
    ├── LICENSE
    └── THIRD_PARTY_NOTICES.md
```

**Which folder do I use?**
- **orunla-mac-aarch64** — Apple Silicon (M1, M2, M3, M4)
- **orunla-mac-x86_64** — Intel Macs

Not sure? Run `uname -m` in Terminal. If it says `arm64`, use aarch64. If it says `x86_64`, use x86_64.

---

## Quick Start

### Step 1: Open your platform folder

Open Terminal and navigate to the correct folder for your Mac:

```bash
# Apple Silicon (M1/M2/M3/M4):
cd /path/to/Orunla_macOS_v0.3.4/orunla-mac-aarch64

# Intel Mac:
cd /path/to/Orunla_macOS_v0.3.4/orunla-mac-x86_64
```

Replace `/path/to/` with wherever you extracted the bundle (e.g., `~/Downloads/`).

### Step 2: Bypass macOS Gatekeeper

Orunla is unsigned. macOS will block the binaries on first run. Remove the quarantine attribute:

```bash
xattr -cr .
chmod +x orunla_cli orunla_mcp launch_orunla.sh
```

This only needs to be done once after extracting.

### Step 3: Set up the ONNX Runtime

The `libonnxruntime.dylib` must be discoverable by the binaries. Set the environment variable:

```bash
export ORT_DYLIB_PATH="$(pwd)/libonnxruntime.dylib"
```

To make this permanent, add it to your shell profile (adjust path for your setup):

```bash
# For zsh (default on macOS) — Apple Silicon example:
echo 'export ORT_DYLIB_PATH="$HOME/Orunla_macOS_v0.3.4/orunla-mac-aarch64/libonnxruntime.dylib"' >> ~/.zshrc
source ~/.zshrc

# For bash:
echo 'export ORT_DYLIB_PATH="$HOME/Orunla_macOS_v0.3.4/orunla-mac-aarch64/libonnxruntime.dylib"' >> ~/.bash_profile
source ~/.bash_profile
```

Use `orunla-mac-x86_64` instead if you're on an Intel Mac.

### Step 4: Verify it works

```bash
./orunla_cli stats
```

You should see node/edge counts and database size. On first run, the GliNER model (~40MB) will download automatically.

---

## Usage Options

### Option 1: Desktop Web UI (Browser-based)

From inside your platform folder, launch the server and web dashboard:

```bash
./launch_orunla.sh
```

This will:
- Start the Orunla REST API server on `http://localhost:3000`
- Open your browser to the dashboard
- Keep running until you press `Ctrl+C`

### Option 2: CLI (Terminal)

Run commands from inside your platform folder:

```bash
# Ingest text
./orunla_cli ingest "Project Titan is led by Mark."

# Ingest a file
./orunla_cli ingest-file ~/Documents/notes.txt

# Recall memories
./orunla_cli recall "Titan"
./orunla_cli recall "Titan" --limit 5 --min-strength 0.2

# View stats
./orunla_cli stats

# Delete a specific memory
./orunla_cli delete "uuid-of-memory"

# Purge all memories matching a topic
./orunla_cli purge "outdated project"

# Garbage collection (remove decayed memories)
./orunla_cli gc --threshold 0.05

# Deduplicate nodes
./orunla_cli dedup
```

### Option 3: REST API Server

Start the server for webhooks, n8n, Make.com, or custom integrations:

```bash
./orunla_cli serve --port 3000
```

To secure the API when exposing to a network:

```bash
./orunla_cli serve --port 3000 --api-key "your-secret-key"
```

See `documentation/API_REFERENCE.md` for all endpoints.

### Option 4: MCP Integration (Claude Desktop / Cursor / Cline)

The MCP server gives AI assistants direct access to your memory graph.

**Important:** All paths in MCP configs must be absolute. Replace the example paths below with the actual location of your Orunla folder.

#### Claude Desktop

1. Open your config file:
   ```bash
   nano ~/Library/Application\ Support/Claude/claude_desktop_config.json
   ```

2. Add the Orunla MCP server (Apple Silicon example):
   ```json
   {
     "mcpServers": {
       "orunla": {
         "command": "/Users/YourUsername/Orunla_macOS_v0.3.4/orunla-mac-aarch64/orunla_mcp",
         "env": {
           "ORT_DYLIB_PATH": "/Users/YourUsername/Orunla_macOS_v0.3.4/orunla-mac-aarch64/libonnxruntime.dylib"
         }
       }
     }
   }
   ```
   Replace `/Users/YourUsername/` with your actual home directory.
   Use `orunla-mac-x86_64` instead if you're on an Intel Mac.

3. Restart Claude Desktop.

#### Claude Code

Add to `~/.claude/settings.json` or your project's `.claude/settings.json`:

```json
{
  "mcpServers": {
    "orunla": {
      "command": "/Users/YourUsername/Orunla_macOS_v0.3.4/orunla-mac-aarch64/orunla_mcp",
      "env": {
        "ORT_DYLIB_PATH": "/Users/YourUsername/Orunla_macOS_v0.3.4/orunla-mac-aarch64/libonnxruntime.dylib"
      }
    }
  }
}
```

#### Cursor / Cline / Windsurf

Add the same configuration to your IDE's MCP settings. The command and env are identical.

---

## Licensing

Orunla uses a **Free / Pro** licensing model.

### Free Tier (Default)
- **All local features are free forever** — no time limits, no feature restrictions
- Ingest, recall, search, delete, purge, garbage collection, deduplication
- Desktop web UI, CLI, MCP server, REST API — all fully functional
- Your data stays 100% on your machine

### Trial (14 Days)
- On first launch, you automatically get a **14-day free trial of Pro features**
- This includes cross-device sync (see below)
- After the trial, you keep all Free features with no interruption

### Pro Tier
- Adds **cross-device sync**: keep your memories in sync across multiple computers
- Activate with a license key from your purchase email:

```bash
./orunla_cli activate "your-license-key-here"
```

- Check your current license status:

```bash
./orunla_cli license
```

### Cross-Device Sync (Pro)

Sync your knowledge graph across all your devices automatically:

1. **Activate the same license key** on each device:
   ```bash
   ./orunla_cli activate "your-license-key"
   ```
2. **Sync happens automatically** every 30 seconds when using the MCP server, REST API server, or desktop web UI
3. **Manual sync** (one-time push + pull) via CLI:
   ```bash
   ./orunla_cli sync
   ```

All synced data is **end-to-end encrypted** (AES-256-GCM). The relay server only sees ciphertext — it cannot read your memories.

---

## Where is my data stored?

All data is stored locally:

```
~/.orunla/memory.db
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

### "orunla_cli" cannot be opened because it is from an unidentified developer

Run the quarantine removal from Step 2:
```bash
cd /path/to/Orunla_macOS_v0.3.4/orunla-mac-aarch64
xattr -cr .
chmod +x orunla_cli orunla_mcp launch_orunla.sh
```

### "Library not loaded" or ONNX Runtime errors

The `ORT_DYLIB_PATH` environment variable is not set or points to the wrong location:
```bash
# Check if it's set
echo $ORT_DYLIB_PATH

# Check if the file exists at that path
ls -la $ORT_DYLIB_PATH
```

Make sure the path is absolute (starts with `/`), not relative.

### Port 3000 already in use

```bash
lsof -ti:3000 | xargs kill -9
```

### MCP server not connecting

- Use the **absolute path** to `orunla_mcp` in your config (no `~` or `$HOME` — spell out `/Users/YourUsername/...`)
- Make sure `ORT_DYLIB_PATH` is included in the `env` block of your MCP config
- Verify the binary is executable: `chmod +x orunla_mcp`
- Restart Claude Desktop / Cursor completely (quit and reopen)

### Model download fails

On first run, Orunla downloads the GliNER model (~40MB). If this fails:
- Check your internet connection
- Check you have write access to `~/.orunla/`

### Database errors

Reset by deleting the database:
```bash
rm ~/.orunla/memory.db
```

---

### Option 5: Any AI Chatbot (ChatGPT, Gemini, Claude Web, etc.)

Use the REST API to connect Orunla to any AI that supports custom actions, skills, or system prompts — including ChatGPT Custom GPTs, Google Gemini Gems, Claude Projects, and more.

See `documentation/AI_SETUP.md` for:
- Step-by-step ChatGPT Custom GPT setup (with OpenAPI spec for Actions)
- Google Gemini Gem setup
- Claude browser MCP connector setup (SSE mode + Cloudflare Tunnel)
- Copy-paste system prompts that teach any AI to use Orunla automatically
- A `CLAUDE.md` template for Claude Code users

---

## Documentation

- **CLI Reference:** `documentation/CLI.md`
- **MCP Guide:** `documentation/MCP.md`
- **REST API:** `documentation/API_REFERENCE.md`
- **AI Chatbot Setup:** `documentation/AI_SETUP.md`
- **Architecture:** `documentation/OVERVIEW.md`
