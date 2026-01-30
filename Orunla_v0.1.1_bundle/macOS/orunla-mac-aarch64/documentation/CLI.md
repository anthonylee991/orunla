# Orunla CLI Reference Guide

The Orunla Command Line Interface (CLI) is the primary tool for administrators and advanced users to manage the memory system, perform maintenance, and run the background server.

**Binary Name:**
- Windows: `orunla_cli.exe` (in `Orunla_v0.1.1_bundle\windows\`)
- macOS: `orunla_cli` (in `Orunla_v0.1.1_bundle/macOS/orunla-mac-{arch}/`)

---

## Core Commands

### `ingest`
Add raw text to your memory. The AI will automatically extract relationships.

```bash
# macOS
./orunla_cli ingest "Project Titan is led by Mark."

# Windows
.\orunla_cli.exe ingest "Project Titan is led by Mark."
```

### `ingest-file`
Process an entire file and turn it into facts.

```bash
# macOS
./orunla_cli ingest-file ~/Documents/notes.txt

# Windows
.\orunla_cli.exe ingest-file "C:\path\to\notes.txt"
```

Supported formats: `.txt`, `.md`, `.csv`, `.json`

### `recall`
Search your memory for facts.

```bash
# macOS
./orunla_cli recall "Titan"
./orunla_cli recall "Titan" --limit 5 --min-strength 0.2

# Windows
.\orunla_cli.exe recall "Titan"
.\orunla_cli.exe recall "Titan" --limit 5 --min-strength 0.2
```

Options:
- `--limit <N>` — Show only the top N results
- `--min-strength <FLOAT>` — Filter out faded/weak memories (0.0 to 1.0)

### `stats`
Show current database health and size.

```bash
# macOS
./orunla_cli stats

# Windows
.\orunla_cli.exe stats
```

---

## Maintenance Commands

### `delete`
Delete a specific memory by its unique ID.

```bash
# macOS
./orunla_cli delete "uuid-of-memory"

# Windows
.\orunla_cli.exe delete "uuid-of-memory"
```

### `purge`
Delete all memories matching a keyword or topic.

```bash
# macOS
./orunla_cli purge "outdated project"

# Windows
.\orunla_cli.exe purge "outdated project"
```

### `gc` (Garbage Collection)
Permanently remove decayed memories from the database. Also cleans up orphaned nodes that no longer have connections.

```bash
# macOS
./orunla_cli gc --threshold 0.05

# Windows
.\orunla_cli.exe gc --threshold 0.05
```

### `dedup` (Node Deduplication)
Merge duplicate entities (e.g., "AI", "ai", "A.I.") into a single canonical node.

```bash
# macOS
./orunla_cli dedup

# Windows
.\orunla_cli.exe dedup
```

---

## Server Commands

### `serve`
Start the background REST API server. Required for the Desktop Web UI and no-code integrations (Zapier, Make, n8n).

```bash
# macOS
./orunla_cli serve --port 3000

# Windows
.\orunla_cli.exe serve --port 3000
```

With API key authentication (recommended when exposing to network):

```bash
# macOS
./orunla_cli serve --port 3000 --api-key "your-secret-key"

# Windows
.\orunla_cli.exe serve --port 3000 --api-key "your-secret-key"
```

---

## Notes

- **Model Download:** On first run, the CLI will download the ~40MB GliNER model. Ensure you have an internet connection.
- **Database Location:**
  - macOS: `~/.orunla/memory.db`
  - Windows: `%USERPROFILE%\.orunla\memory.db`
- **ONNX Runtime (macOS only):** The `ORT_DYLIB_PATH` environment variable must be set before running any command. See the Mac README for setup instructions.
- **ONNX Runtime (Windows):** The `onnxruntime.dll` must be in the same folder as the executables.
- **Logs:** If something goes wrong, check the terminal output for error messages.
