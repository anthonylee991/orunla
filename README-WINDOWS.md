# ORUNLA - Windows Installation Guide

> **Hybrid RAG Agent Memory System**
> SQLite-based knowledge graph with text + graph search powered by GliNER

---

## 📦 What's Included

```
orunla-windows/
├── Orunla.exe          - Desktop Application
├── orunla_mcp.exe      - MCP Server (for AI assistants)
└── README-WINDOWS.md   - This file
```

---

## 🚀 Quick Start

### Option 1: Desktop Application (Recommended)

1. **Double-click `Orunla.exe`** to launch the desktop app
2. The app includes:
   - Memory ingestion (text + file upload)
   - Memory recall/search
   - Context purging
   - Real-time stats

### Option 2: MCP Integration (for Claude/Cursor/Cline)

1. **Locate `orunla_mcp.exe`** in the installation folder
2. **Copy the full path** (e.g., `C:\Users\YourName\orunla\orunla_mcp.exe`)
3. **Add to your MCP config:**

For **Claude Desktop**, edit `%APPDATA%\Claude\claude_desktop_config.json`:
```json
{
  "mcpServers": {
    "orunla": {
      "command": "C:/Users/YourName/orunla/orunla_mcp.exe",
      "args": []
    }
  }
}
```

For **Cursor/Cline/Windsurf**, add to your IDE's MCP settings with the same format.

4. **Restart your AI assistant** to activate the memory server

---

## 💾 Where is my data stored?

All data is stored locally in:
```
%USERPROFILE%\.orunla\memory.db
```

This SQLite database contains:
- Knowledge graph (nodes + edges)
- FTS5 full-text search index
- Memory metadata and timestamps

---

## 🔧 Advanced Usage

### REST API Server (Optional)

For webhooks, n8n, Make.com, or custom integrations:

1. Open **Command Prompt** or **PowerShell**
2. Navigate to the installation folder
3. Run:
   ```bash
   orunla_cli.exe serve --port 3000
   ```
4. API will be available at `http://localhost:3000`

See `API_REFERENCE.md` for endpoint documentation.

---

## 🛠️ Troubleshooting

### Desktop app won't start
- **Check antivirus:** Some antivirus software may block unsigned executables
- **Run as Administrator:** Right-click `Orunla.exe` → "Run as administrator"
- **Missing dependencies:** Ensure you have the latest Visual C++ Redistributable

### MCP not connecting
- **Verify path:** Make sure the path in your config uses forward slashes (`/`)
- **Check logs:** Look for errors in your AI assistant's logs
- **Restart assistant:** Completely quit and restart Claude/Cursor/Cline

### Database errors
- **Corrupted database:** Delete `%USERPROFILE%\.orunla\memory.db` to reset
- **Permissions:** Ensure you have write access to your user folder

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
