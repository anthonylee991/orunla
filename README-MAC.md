# ORUNLA - Mac Installation Guide

> **Hybrid RAG Agent Memory System**
> SQLite-based knowledge graph with text + graph search powered by GliNER

---

## 📦 What's Included

```
orunla-mac/
├── launch_orunla.sh    - Browser launcher script
├── ui/                 - Web interface assets
│   ├── index.html
│   └── main.js
└── README-MAC.md       - This file
```

**Note:** The CLI binary is not included. You'll need to build it from source (Rust required).

---

## 🚀 Quick Start

### Step 1: Install Rust (if not already installed)

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

### Step 2: Build Orunla

```bash
# Clone or download the Orunla source code
cd orunla-mac

# Build the CLI and MCP binaries
cargo build --release --bin orunla_cli
cargo build --release --bin orunla_mcp

# Copy binaries to current directory
cp target/release/orunla_cli .
cp target/release/orunla_mcp .
```

### Step 3: Launch Desktop Interface

```bash
# Make the launcher executable
chmod +x launch_orunla.sh

# Run the launcher
./launch_orunla.sh
```

This will:
- Start the Orunla server on `http://localhost:3000`
- Automatically open your browser to the dashboard
- Keep the server running until you press `Ctrl+C`

---

## 🔌 MCP Integration (for Claude/Cursor/Cline)

### For Claude Desktop

1. **Find your config file:**
   ```bash
   nano ~/Library/Application\ Support/Claude/claude_desktop_config.json
   ```

2. **Add Orunla MCP server:**
   ```json
   {
     "mcpServers": {
       "orunla": {
         "command": "/Users/YourUsername/orunla-mac/orunla_mcp",
         "args": []
       }
     }
   }
   ```

   Replace `/Users/YourUsername/orunla-mac/orunla_mcp` with the actual path to your `orunla_mcp` binary.

3. **Restart Claude Desktop**

### For Cursor/Cline/Windsurf

Add the same configuration to your IDE's MCP settings.

---

## 💾 Where is my data stored?

All data is stored locally in:
```
~/.orunla/memory.db
```

This SQLite database contains:
- Knowledge graph (nodes + edges)
- FTS5 full-text search index
- Memory metadata and timestamps

---

## 🔧 Advanced Usage

### Using the CLI Directly

```bash
# Ingest text
./orunla_cli ingest "Your knowledge here"

# Recall memories
./orunla_cli recall "search query"

# View statistics
./orunla_cli stats

# Delete a memory
./orunla_cli delete <edge-id>

# Purge by topic
./orunla_cli purge "topic"
```

See `CLI.md` for full documentation.

### REST API Server

For webhooks, n8n, Make.com, or custom integrations:

```bash
./orunla_cli serve --port 3000
```

API will be available at `http://localhost:3000`

See `API_REFERENCE.md` for endpoint documentation.

---

## 🛠️ Troubleshooting

### Build fails
- **Rust not found:** Run `source $HOME/.cargo/env` after installing Rust
- **Missing dependencies:** macOS may require Xcode Command Line Tools:
  ```bash
  xcode-select --install
  ```

### Launcher won't start
- **Permission denied:** Run `chmod +x launch_orunla.sh`
- **Binary not found:** Ensure you built the CLI: `cargo build --release --bin orunla_cli`

### Port already in use
- **Kill existing process:**
  ```bash
  lsof -ti:3000 | xargs kill -9
  ```

### MCP not connecting
- **Verify path:** Use absolute path to `orunla_mcp` binary
- **Check permissions:** Ensure binary is executable: `chmod +x orunla_mcp`
- **Check logs:** Look for errors in Claude Desktop logs
- **Restart assistant:** Completely quit and restart Claude/Cursor/Cline

### Database errors
- **Corrupted database:** Delete `~/.orunla/memory.db` to reset
- **Permissions:** Ensure you have write access to `~/.orunla/`

---

## 📚 Documentation

- **CLI Guide:** `CLI.md`
- **MCP Guide:** `MCP.md`
- **API Reference:** `API_REFERENCE.md`

---

## 🆘 Support

For issues, questions, or feature requests:
- GitHub Issues: [your-repo-link]
- Email: [your-email]

---

## 📄 License

[Your License Here]
