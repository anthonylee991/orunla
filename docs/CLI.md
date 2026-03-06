# Orunla CLI Reference Guide

The Orunla Command Line Interface (CLI) is the primary tool for administrators and advanced users to manage the memory system, perform maintenance, and run the background server.

**Binary Name:** `orunla_cli` (or `orunla_cli.exe` on Windows)

---

## Core Commands

### `ingest`
Add raw text to your memory. The AI will automatically extract relationships.

```bash
orunla_cli ingest "Project Titan is led by Mark."
```

You can also ingest from a file using the `--file` flag:

```bash
orunla_cli ingest --file ~/Documents/notes.txt
```

### `ingest-file`
Process an entire file and turn it into facts.

```bash
orunla_cli ingest-file ~/Documents/notes.txt
```

Supported formats: `.txt`, `.md`, `.csv`, `.json`

### `recall`
Search your memory for facts.

```bash
orunla_cli recall "Titan"
orunla_cli recall "Titan" --limit 5 --min-strength 0.2
```

Options:
- `--limit <N>` — Show only the top N results
- `--min-strength <FLOAT>` — Filter out faded/weak memories (0.0 to 1.0)

### `stats`
Show current database health and size.

```bash
orunla_cli stats
```

---

## Maintenance Commands

### `delete`
Delete a specific memory by its unique ID.

```bash
orunla_cli delete "uuid-of-memory"
```

### `purge`
Delete all memories matching a keyword or topic.

```bash
orunla_cli purge "outdated project"
```

### `gc` (Garbage Collection)
Permanently remove decayed memories from the database. Also cleans up orphaned nodes that no longer have connections.

```bash
orunla_cli gc --threshold 0.05
```

### `dedup` (Node Deduplication)
Merge duplicate entities (e.g., "AI", "ai", "A.I.") into a single canonical node.

```bash
orunla_cli dedup
```

---

## Server Commands

### `serve`
Start the background REST API server.

```bash
orunla_cli serve --port 3000
```

With API key authentication (recommended when exposing to network):

```bash
orunla_cli serve --port 3000 --api-key "your-secret-key"
```

**Note:** When the Orunla desktop app is running, the REST API is already available on port 8080 (along with MCP SSE). You only need `serve` if you want to run the REST API standalone without the desktop app, or on a different port.

---

## Benchmark Commands

### `benchmark`
Run extraction benchmark to evaluate extractor quality.

```bash
orunla_cli benchmark --cases benchmark_cases.json --mode compare
```

Options:
- `--cases <PATH>` — Path to test cases JSON file
- `--verbose` — Show detailed output
- `--mode <MODE>` — `gliner`, `patterns`, `hybrid`, or `compare` (default)

---

## Notes

- **Model Download:** On first run, the CLI will download the ~40MB GliNER model. Ensure you have an internet connection.
- **Database Location:** `~/.orunla/memory.db` (or `%USERPROFILE%\.orunla\memory.db` on Windows)
- **ONNX Runtime (macOS):** The `ORT_DYLIB_PATH` environment variable must be set before running any command.
- **ONNX Runtime (Windows):** The `onnxruntime.dll` must be in the same folder as the executables.
