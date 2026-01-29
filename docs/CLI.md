# Orunla CLI Reference Guide

The Orunla Command Line Interface (CLI) is the primary tool for administrators and advanced users to manage the memory system, perform maintenance, and run the background server.

**Binary Name:** `orunla_cli.exe`

---

## 🛠️ Core Commands

### `ingest`
Add raw text to your memory. The AI will automatically extract relationships.
```powershell
.\orunla_cli.exe ingest "Project Titan is led by Mark."
```

### `ingest-file`
Process an entire file and turn it into facts.
```powershell
.\orunla_cli.exe ingest-file "C:\path\to\notes.txt"
```
*Supported formats: .txt, .md, .csv, .json*

### `recall`
Search your memory for facts.
```powershell
.\orunla_cli.exe recall "Titan"
```
- `--limit <N>`: Show only the top N results.
- `--min-strength <FLOAT>`: Filter out faded/weak memories (0.0 to 1.0).

### `stats`
Show current database health and size.
```powershell
.\orunla_cli.exe stats
```

---

## 🧹 Maintenance Commands

### `delete`
Delete a specific memory by its unique ID.
```powershell
.\orunla_cli.exe delete "uuid-of-memory"
```

### `purge`
Delete all memories matching a keyword or topic.
```powershell
.\orunla_cli.exe purge "outdated project"
```

### `gc` (Garbage Collection)
Permanently remove decayed memories from the database.
```powershell
.\orunla_cli.exe gc --threshold 0.05
```
*This command also cleans up "orphaned" nodes that no longer have connections.*

### `dedup` (Node Deduplication)
Merge duplicate entities (e.g., "AI", "ai", "A.I.") into a single canonical node.
```powershell
.\orunla_cli.exe dedup
```

---

## 🌐 Server Commands

### `serve`
Start the background REST API server.
```powershell
.\orunla_cli.exe serve --port 3000
```
*Required for Desktop UI and No-Code integrations (Zapier/Make).*

---

## 💡 Troubleshooting
- **Model Download**: On first run, the CLI will download the ~40MB GliNER model. Ensure you have an internet connection.
- **Database Location**: By default, memories are stored in `%USERPROFILE%\.orunla\memory.db`.
- **Logs**: If something goes wrong, check the terminal output for error messages.
