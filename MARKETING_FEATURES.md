# Orunla — Feature & Benefit List (for Website)

## Tagline Options
- "AI Memory That Thinks Like You Do"
- "Local-First Knowledge Graph for AI Agents and Humans"
- "Your Second Brain — Private, Intelligent, Always Ready"

---

## Hero Section / Value Proposition

**Orunla is a local-first AI memory system** that extracts facts from anything you tell it, builds a knowledge graph of relationships, and recalls information when you need it — all running 100% on your own computer.

Give your AI agents (Claude, Cursor, Cline) persistent memory that survives between sessions. No cloud required. No API costs. No privacy compromises.

---

## Core Features

### Intelligent Fact Extraction
- Powered by **GliNER**, a local AI model that runs on your machine
- Automatically extracts people, organizations, locations, concepts, and their relationships
- Understands sentence structure — turns "Sarah works at Microsoft in Seattle" into structured knowledge
- No API calls, no cloud processing, no per-token costs

### Knowledge Graph Storage
- Facts stored as a web of connected entities, not flat text
- SQLite-based — lightweight, portable, zero configuration
- Recognizes 7 entity types: Person, Organization, Location, Artifact, Concept, Software, Language
- Every fact includes a confidence score from the AI extraction

### Human-Like Memory Decay
- Implements the **Ebbinghaus Forgetting Curve** — memories fade over time, just like human memory
- Frequently accessed facts stay strong; unused facts gradually weaken
- Automatic garbage collection removes forgotten memories
- Keeps your knowledge base lean and relevant without manual curation

### Hybrid Search
- Combines **full-text search** (FTS5) with **graph traversal** for precise recall
- Searches by keywords, entity names, and relationship types simultaneously
- Results ranked by relevance and memory strength
- Sub-second responses even with 100,000+ stored facts

### Complete Privacy
- **Everything runs locally** — your data never leaves your computer
- SQLite database stored in your home directory
- No cloud accounts, no telemetry, no data collection
- You own your data, period

---

## Access Methods (4 Ways to Use Orunla)

### 1. Desktop Application (Windows)
- Clean, modern interface for ingesting text and files
- Visual memory search and recall
- One-click topic purging
- Real-time database statistics
- License activation and sync status

### 2. MCP Server (AI Agent Integration)
- Give Claude Desktop, Claude Code, Cursor, or Cline persistent memory
- 8 built-in tools: add, search, get all, get context, delete, purge, garbage collect, sync chat
- Your AI remembers your preferences, past decisions, and project context across sessions
- Drop-in JSON config — no coding required

### 3. CLI Tool (Power Users)
- Full command-line interface for scripting and automation
- Ingest text, files, recall, delete, purge, GC, dedup
- License management and manual sync
- Perfect for cron jobs and CI/CD pipelines

### 4. REST API (Integrations)
- HTTP server for webhooks, Zapier, Make.com, n8n, and custom apps
- JSON endpoints for ingest, recall, purge, stats, and maintenance
- API key authentication for secure network exposure
- Rate limiting built in

---

## File Ingestion
- **Supported formats:** TXT, Markdown, CSV, JSON
- Drag-and-drop in the desktop app or CLI `ingest-file` command
- Automatic chunking for large files
- Batch extraction of all facts in a document

---

## Maintenance & Sustainability

### Garbage Collection
- Automatically prune weak/forgotten memories
- Configurable threshold — aggressive or conservative
- Removes orphaned entities with no remaining connections

### Node Deduplication
- Merges duplicate entities ("Rust", "rust", "RUST" → single node)
- Preserves all relationships during merge
- Prevents knowledge fragmentation

### Topic Purging
- Delete all memories about a specific subject
- Clean removal including orphaned entities
- Useful for clearing test data or outdated projects

---

## Cross-Device Sync (Pro)

### How It Works
- Activate the same license key on each device
- Memories sync automatically every 30 seconds in the background
- Works across Windows and macOS

### End-to-End Encryption
- **AES-256-GCM** encryption before any data leaves your machine
- Encryption key derived from your license key — only your devices can decrypt
- Relay server stores and forwards only ciphertext
- Zero-knowledge architecture — we cannot read your memories

### Conflict Resolution
- Automatic deduplication of concurrent edits
- Deletions propagate across all devices
- Node merges sync cleanly
- Last-write-wins for edge cases

---

## Licensing — Generous Free Tier

### Free (Forever)
- All local features — no time limits, no artificial restrictions
- Unlimited ingestion, recall, search, delete, purge, GC, dedup
- Desktop app, CLI, MCP server, REST API — all fully functional
- Your data is yours, no strings attached

### 14-Day Pro Trial
- Automatically starts on first launch
- Try cross-device sync with no commitment
- Seamlessly transitions to Free if you don't upgrade

### Pro
- Adds cross-device sync
- One-time purchase — no subscriptions
- Activate with a single CLI command
- Supports unlimited devices per license

---

## Who Is Orunla For?

### Developers
- Give your AI coding assistant (Claude, Cursor) memory that persists between sessions
- Store API endpoints, database schemas, deployment procedures
- Never re-explain your project architecture to Claude again

### Knowledge Workers
- Ingest meeting notes, policies, and procedures
- Search across months of accumulated knowledge
- Remember who said what and when

### Researchers
- Build a knowledge graph from papers and sources
- Map relationships between concepts, authors, and findings
- Query your research library with natural language

### Teams (with Pro)
- Shared knowledge base across multiple workstations
- Sync facts between home and office
- End-to-end encrypted — safe for sensitive information

### AI Enthusiasts
- Build agents with real persistent memory
- Integrate via MCP, REST API, or direct SQLite access
- Open architecture — extend and customize

---

## Technical Highlights

| Feature | Detail |
|---------|--------|
| AI Engine | GliNER (local ONNX, ~40MB model) |
| Storage | SQLite with FTS5 |
| Search | Hybrid: full-text + graph traversal |
| Memory Model | Ebbinghaus forgetting curve |
| Entity Types | 7 (Person, Org, Location, Artifact, Concept, Software, Language) |
| Sync Encryption | AES-256-GCM with PBKDF2 key derivation |
| Platforms | Windows 10+, macOS 11+ (Intel & Apple Silicon) |
| RAM Usage | ~200MB (AI model loaded) |
| Recall Speed | 10-50ms typical |
| Storage Efficiency | ~1KB per fact |

---

## Comparison

| Feature | Orunla | Notion/Obsidian | LangChain RAG | Pinecone/Weaviate | Mem.ai |
|---------|--------|-----------------|---------------|-------------------|--------|
| Local-first | Yes | No (cloud) | Varies | No (cloud) | No |
| Structured facts | Yes (knowledge graph) | No (text) | No (chunks) | No (vectors) | No |
| AI extraction | Built-in (local) | None | Requires API | Requires API | Cloud |
| Memory decay | Yes (forgetting curve) | No | No | No | No |
| Privacy | 100% local | Cloud-dependent | API-dependent | Cloud only | Cloud only |
| Cross-device sync | Pro (E2E encrypted) | Built-in (cloud) | Manual | Built-in (cloud) | Built-in (cloud) |
| Cost | Free core + optional Pro | Subscription | API costs | Subscription | Subscription |
| MCP integration | Native | No | No | No | No |

---

## What Makes Orunla Different

1. **Facts, not text** — Orunla doesn't just store documents. It understands them and builds a web of relationships.
2. **Memory that fades** — Like human memory, unused facts naturally decay. Your knowledge base stays relevant without manual cleanup.
3. **AI-native** — Purpose-built for AI agents. Your Claude/Cursor gets persistent memory with zero code.
4. **Truly private** — Local SQLite, local AI, no cloud accounts. Optional sync is end-to-end encrypted.
5. **Free core** — Not a trial, not freemium with crippled features. The full local product is free forever.
