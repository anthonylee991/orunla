# Orunla: Complete Technical Overview

## What Is Orunla?

Orunla is a **local-first AI memory system** that helps you and your AI agents remember information accurately over time. Unlike traditional note-taking apps or AI chat histories that just store text, Orunla understands the **relationships between facts** and builds them into a knowledge graph—a web of connected information.

Think of it as a "second brain" that:
- Automatically extracts facts from anything you tell it
- Remembers connections between people, places, concepts, and events
- Recalls information when you need it
- Forgets things that stop being relevant (just like human memory)
- Keeps everything 100% private on your own computer

## How Orunla Works: The Big Picture

### 1. You Add Information (Ingestion)

You can feed Orunla information in several ways:
- Type text directly into the desktop app
- Upload files (TXT, CSV, JSON, Markdown, etc.)
- Use the REST API from other apps
- Let your AI agents (like Claude) store memories through MCP

**What happens behind the scenes:**

When you add text like: *"Sarah works at Microsoft and lives in Seattle"*

Orunla uses an AI model called **GliNER** (Generalist Model for Named Entity Recognition) to automatically extract entities and their relationships:
- **Entities**: Sarah (person), Microsoft (organization), Seattle (location)
- **Relationships**: "works at" and "lives in"

This creates **triplets** (subject-predicate-object) in the knowledge graph:
- `Sarah` → `works at` → `Microsoft`
- `Sarah` → `lives in` → `Seattle`

### 2. The Knowledge Graph Structure

Orunla stores everything in a **SQLite database** with three main components:

**Nodes** (Concepts/Entities):
- Each unique person, place, thing, or idea gets a node
- Nodes have: ID, label (name), type (person/organization/location/etc.)

**Edges** (Relationships/Facts):
- Connect two nodes with a predicate (relationship type)
- Store: source node, target node, predicate, original text, confidence score
- Track: when created, when last accessed, access count

**Full-Text Search Index** (FTS5):
- SQLite's FTS5 index on all edge source text
- Enables fast keyword searches across all stored memories

### 3. Memory Decay: The Forgetting Curve

Orunla replicates human forgetting with the **Ebbinghaus Forgetting Curve**.

**The Formula:**
```
Strength = decay x access_boost x confidence

Where:
- decay = e^(-days_since_access / 30)
- access_boost = 1.0 + ln(1 + access_count)
- confidence = GliNER's confidence score (0.0 to 1.0)
```

**What this means:**
1. **Recency matters**: Recently accessed memories are stronger
2. **Repetition helps**: Memories accessed multiple times decay slower
3. **Confidence matters**: Facts extracted with high certainty matter more
4. **30-day stability**: After 30 days without access, ~37% strength remains

### 4. Hybrid Retrieval: How Recall Works

When you search, Orunla uses a **two-pass hybrid search**:

**Pass 1: Full-Text Search (FTS5)** — Searches source text for keyword matches
**Pass 2: Graph Search** — Searches node labels and edge predicates
**Pass 3: Filtering & Ranking** — Calculates strength, filters, sorts by relevance

### 5. GliNER Entity Extraction

GliNER runs **locally** on your computer using ONNX Runtime. It recognizes entities across seven categories: Person, Organization, Location, Artifact, Concept, Software, Language.

**Why local AI matters:**
- No API costs
- Works offline
- Complete privacy
- Fast inference (milliseconds per extraction)

### 6. Maintenance

**Garbage Collection:** Removes memories below a strength threshold
**Node Deduplication:** Merges duplicate entities (e.g., "Rust", "rust", "RUST")
**Topic Purging:** Removes all memories matching a keyword

## Interfaces

### 1. Desktop App
Graphical application with cyberpunk UI. Auto-starts unified server (REST API + MCP SSE on port 8080).

### 2. MCP Server
Background service for AI agents (Claude Desktop, Cursor, Cline). Supports stdio and SSE transport.

### 3. CLI Tool
Terminal commands for ingestion, recall, maintenance, and running the server.

### 4. REST API
HTTP endpoints for integration with any programming language or no-code tools.

## Where Your Data Lives

**Database location:** `~/.orunla/memory.db` (or `%USERPROFILE%\.orunla\memory.db` on Windows)

**Tables:**
- `nodes` — All entities (people, places, concepts)
- `edges` — All relationships/facts
- `edges_fts` — Full-text search index

You can open `memory.db` with [DB Browser for SQLite](https://sqlitebrowser.org/) to browse your knowledge graph.

## Technical Architecture

```
+-------------------------------------------------------------+
|                     Input Layer                               |
|  (Desktop UI, CLI, MCP Server, REST API)                      |
+----------------------------+--------------------------------+
                             |
                             v
+-------------------------------------------------------------+
|                  GliNER Extractor                             |
|  (ONNX Runtime, Local AI, Entity Recognition)                 |
|  Input: Raw text                                              |
|  Output: Triplets (subject, predicate, object, confidence)    |
+----------------------------+--------------------------------+
                             |
                             v
+-------------------------------------------------------------+
|                 Knowledge Graph Storage                       |
|  * SQLite Database (nodes, edges tables)                      |
|  * FTS5 Full-Text Search Index                                |
|  * Tracks: creation time, access time, access count           |
+----------------------------+--------------------------------+
                             |
                             v
+-------------------------------------------------------------+
|                 Hybrid Retriever                              |
|  * Pass 1: FTS5 search on source text                         |
|  * Pass 2: Graph search on node labels & predicates           |
|  * Calculate strength (Ebbinghaus decay)                      |
|  * Filter and rank by relevance                               |
+----------------------------+--------------------------------+
                             |
                             v
+-------------------------------------------------------------+
|                 Maintenance Layer                              |
|  * Garbage Collection (delete weak memories)                  |
|  * Node Deduplication (merge duplicates)                      |
|  * Orphan Cleanup (remove disconnected nodes)                 |
|  * Topic Purging (delete by query)                            |
+-------------------------------------------------------------+
```

## System Requirements

**Minimum:**
- OS: Windows 10+, macOS 11+, Linux (Ubuntu 20.04+)
- RAM: 2GB available
- Disk: 500MB (includes AI model)
- CPU: Any modern x64 processor

**Recommended:**
- RAM: 4GB+ for large datasets
- SSD for faster database operations

## License

Orunla is open source under the Apache License 2.0.
