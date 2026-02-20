# Orunla REST API Reference

Access your Orunla memory from any tool, automation, or AI integration through the cloud relay.

**Base URL:** Copy from the desktop app's **"Remote Access"** card. It looks like:
```
https://orunla-production.up.railway.app/api/YOUR-DEVICE-ID
```

**Requirements:** The Orunla desktop app must be open for API requests to work.

> Want step-by-step setup guides for ChatGPT, Claude, Gemini, or n8n? See `AI_SETUP.md`.

---

## Authentication

If you set an API key in the desktop app's **API Key** settings panel, all requests (except health/stats) require it:

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
curl https://orunla-production.up.railway.app/api/YOUR-DEVICE-ID/health
```

**Response:**
```json
{ "status": "ok", "version": "0.4.1" }
```

---

### `GET /stats`

Get memory database statistics.

```bash
curl https://orunla-production.up.railway.app/api/YOUR-DEVICE-ID/stats
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
curl -X POST https://orunla-production.up.railway.app/api/YOUR-DEVICE-ID/ingest \
  -H "Content-Type: application/json" \
  -H "X-API-Key: your-key" \
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
curl -X POST https://orunla-production.up.railway.app/api/YOUR-DEVICE-ID/recall \
  -H "Content-Type: application/json" \
  -H "X-API-Key: your-key" \
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

### `POST /memories/purge`

Delete all memories matching a keyword or topic.

```bash
curl -X POST https://orunla-production.up.railway.app/api/YOUR-DEVICE-ID/memories/purge \
  -H "Content-Type: application/json" \
  -H "X-API-Key: your-key" \
  -d '{"query": "old project"}'
```

**Body:**
```json
{ "query": "topic to forget" }
```

**Response:**
```json
{ "status": "ok", "deleted_count": 5 }
```

---

### `DELETE /memories/:id`

Delete a specific memory by its ID (returned from `/recall`).

```bash
curl -X DELETE https://orunla-production.up.railway.app/api/YOUR-DEVICE-ID/memories/abc-123 \
  -H "X-API-Key: your-key"
```

**Response:** `200 OK`

---

### `POST /gc`

Run garbage collection to clean up old, decayed memories.

```bash
curl -X POST https://orunla-production.up.railway.app/api/YOUR-DEVICE-ID/gc \
  -H "Content-Type: application/json" \
  -H "X-API-Key: your-key" \
  -d '{"threshold": 0.05}'
```

**Body:**
```json
{ "threshold": 0.05 }
```

Memories with strength below the threshold are deleted. Default: 0.05.

**Response:**
```json
{ "status": "ok", "deleted_memories": 2, "cleaned_nodes": 1 }
```

---

## File Upload

File upload (`/ingest-file`) is only available through the desktop app interface, not through the relay. Use the desktop app UI to upload `.txt`, `.md`, `.csv`, or `.json` files directly.

---

## Limits

| Limit | Value |
|-------|-------|
| Rate limit (relay) | 120 requests/min per IP |
| Text input | 1 MB max |
| File upload | 50 MB max (desktop app only) |
| Query length | 10 KB max |
| Results per query | 10,000 max |
| Relay timeout | 30 seconds |

---

## Background Sync (Pro)

With a Pro license (or during the 14-day trial), your memories sync automatically across devices every 30 seconds. No configuration needed — just keep the desktop app running.

---

*For local API access, CLI commands, manual server setup, and advanced configuration, see `DEVELOPER.md`.*
