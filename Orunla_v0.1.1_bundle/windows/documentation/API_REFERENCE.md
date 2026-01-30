# Orunla REST API Reference

The Orunla background server provides a JSON REST API for integrating with AI agents, webhooks, and no-code tools.

**Default Base URL:** `http://localhost:3000`

---

## 🔐 Security & Authentication

### Starting the Server with API Key Protection

**IMPORTANT:** If you expose the API to the network (via ngrok, tunneling, or cloud deployment), you MUST use API key authentication to protect your data.

```bash
# Start server with API key authentication
orunla_cli serve --port 3000 --api-key "your-secret-key-here"

# Or use environment variable
export ORUNLA_API_KEY="your-secret-key-here"
orunla_cli serve --port 3000
```

### Authentication Methods

When API key authentication is enabled, all protected endpoints require one of:

**1. Authorization Header (Bearer Token)**
```bash
curl -H "Authorization: Bearer your-secret-key-here" \
     -X POST http://localhost:3000/ingest \
     -H "Content-Type: application/json" \
     -d '{"text": "Test memory"}'
```

**2. X-API-Key Header**
```bash
curl -H "X-API-Key: your-secret-key-here" \
     -X POST http://localhost:3000/ingest \
     -H "Content-Type: application/json" \
     -d '{"text": "Test memory"}'
```

### Rate Limiting

**All endpoints are rate-limited to 60 requests per minute per IP address.**

If you exceed the rate limit, you'll receive a `429 Too Many Requests` response.

### Input Size Limits

- **Text input:** Maximum 1MB (1,000,000 characters)
- **File upload:** Maximum 50MB
- **Query length:** Maximum 10KB (10,000 characters)
- **Result limit:** Maximum 10,000 results per query

### Protected vs Public Endpoints

**Public Endpoints** (no authentication required):
- `GET /health`
- `GET /stats`

**Protected Endpoints** (require API key if configured):
- `POST /ingest`
- `POST /ingest-file`
- `POST /recall`
- `DELETE /memories/:id`
- `POST /memories/purge`

---

## 🏥 Health & Stats

### `GET /health`
Verify if the server is running.
- **Response:**
  ```json
  { "status": "ok", "version": "0.1.0" }
  ```

### `GET /stats`
Get high-level database statistics.
- **Response:**
  ```json
  {
    "node_count": 12,
    "edge_count": 15,
    "db_size_bytes": 77824
  }
  ```

---

## 📥 Ingestion (Adding Data)

### `POST /ingest`
Extract facts from raw text and add them to the knowledge graph.
- **Body:**
  ```json
  {
    "text": "Your raw text here",
    "source_id": "optional_reference_id"
  }
  ```
- **Response:**
  ```json
  { "status": "ok", "added_triplets": 3 }
  ```

### `POST /ingest-file`
Upload a file (`.txt`, `.md`, `.csv`, or `.json`) to be processed.
- **Method:** `POST` (Multipart Content)
- **Field:** `file` (the file attachment)
- **Response:**
  ```json
  {
    "status": "ok",
    "file": "example.txt",
    "chunks_processed": 5,
    "total_triplets_added": 12
  }
  ```

---

## 🔍 Retrieval (Getting Data)

### `POST /recall`
Search for facts based on a query. Uses hybrid keyword and strength-decay ranking.
- **Body:**
  ```json
  {
    "query": "Who is Jane?",
    "limit": 5,
    "min_strength": 0.1
  }
  ```
- **Response:**
  ```json
  {
    "memories": [
      {
        "id": "uuid-string",
        "subject": "Jane",
        "predicate": "manages",
        "object": "Marketing",
        "text": "Jane manages the Marketing budget.",
        "confidence": 0.95,
        "strength": 0.88
      }
    ]
  }
  ```

---

## 🧹 Maintenance & Deletion

### `DELETE /memories/:id`
Permanently delete a specific memory by its ID.
- **Response:** `200 OK`

### `POST /memories/purge`
Delete all memories matching a keyword query.
- **Body:**
  ```json
  { "query": "project x" }
  ```
- **Response:**
  ```json
  { "status": "ok", "deleted_count": 5 }
  ```

---

## 🧠 Data Sustainability

### `POST /gc`
Manually trigger Garbage Collection to prune decayed memories.
- **Body:**
  ```json
  { "threshold": 0.05 }
  ```
- **Response:**
  ```json
  { "status": "ok", "deleted_memories": 2, "cleaned_nodes": 1 }
  ```
