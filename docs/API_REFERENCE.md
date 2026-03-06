# Orunla REST API Reference

Access your Orunla memory from any tool, automation, or AI integration.

**Base URL:** `http://localhost:8080` (desktop app) or `http://localhost:{port}` (CLI `serve`)

---

## Authentication

If you set an API key (via desktop app settings or `--api-key` flag), all protected requests require it:

```
X-API-Key: your-key
```

or:

```
Authorization: Bearer your-key
```

**Public endpoints** (no key needed): `GET /health`, `GET /stats`

**Protected endpoints** (key required if set): everything else

---

## Endpoints

### `GET /health`

Check if Orunla is running.

```bash
curl http://localhost:8080/health
```

**Response:**
```json
{ "status": "ok", "version": "0.5.1" }
```

---

### `GET /stats`

Get memory database statistics.

```bash
curl http://localhost:8080/stats
```

**Response:**
```json
{
  "node_count": 12,
  "edge_count": 15,
  "db_size_bytes": 77824
}
```

---

### `POST /ingest`

Save a memory. Orunla extracts facts from the text and adds them to your knowledge graph.

```bash
curl -X POST http://localhost:8080/ingest \
  -H "Content-Type: application/json" \
  -d '{"text": "Sarah manages the marketing budget and reports to David."}'
```

**Body:**
```json
{
  "text": "Your text here",
  "source_id": "optional_reference_id"
}
```

**Response:**
```json
{ "status": "ok", "added_triplets": 3 }
```

---

### `POST /recall`

Search for memories matching a query. Returns results ranked by relevance and recency.

```bash
curl -X POST http://localhost:8080/recall \
  -H "Content-Type: application/json" \
  -d '{"query": "Who is Sarah?", "limit": 5}'
```

**Body:**
```json
{
  "query": "search term",
  "limit": 5,
  "min_strength": 0.1
}
```

Only `query` is required. `limit` defaults to 10. `min_strength` defaults to 0.0.

**Response:**
```json
{
  "memories": [
    {
      "id": "uuid-string",
      "subject": "Sarah",
      "predicate": "manages",
      "object": "marketing budget",
      "text": "Sarah manages the marketing budget and reports to David.",
      "confidence": 0.95,
      "strength": 0.88
    }
  ]
}
```

---

### `POST /ingest-file`

Upload a file for ingestion. Supports `.txt`, `.md`, `.csv`, `.json`.

```bash
curl -X POST http://localhost:8080/ingest-file \
  -F "file=@document.txt"
```

**Response:**
```json
{
  "status": "ok",
  "file": "document.txt",
  "chunks_processed": 5,
  "total_triplets_added": 12
}
```

Max file size: 50 MB.

---

### `POST /memories/purge`

Delete all memories matching a keyword or topic.

```bash
curl -X POST http://localhost:8080/memories/purge \
  -H "Content-Type: application/json" \
  -d '{"query": "old project"}'
```

**Response:**
```json
{ "status": "ok", "purged_count": 5, "orphaned_cleaned": 2 }
```

---

### `DELETE /memories/:id`

Delete a specific memory by its ID (returned from `/recall`).

```bash
curl -X DELETE http://localhost:8080/memories/abc-123
```

---

## Limits

| Limit | Value |
|-------|-------|
| Rate limit | 60 requests/min per IP |
| Text input | 1 MB max |
| File upload | 50 MB max |
| Query length | 10 KB max |
| Results per query | 10,000 max |
