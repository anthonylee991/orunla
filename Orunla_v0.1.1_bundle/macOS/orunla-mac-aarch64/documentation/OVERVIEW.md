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

Orunla uses an AI model called **GliNER** (Generalist Model for Named Entity Recognition) to automatically extract entities and their relationships. GliNER identifies:
- **Entities**: Sarah (person), Microsoft (organization), Seattle (location)
- **Relationships**: "works at" and "lives in"

This creates **triplets** (subject-predicate-object) in the knowledge graph:
- `Sarah` → `works at` → `Microsoft`
- `Sarah` → `lives in` → `Seattle`

Each triplet becomes an **edge** (connection) in your knowledge graph, linking two **nodes** (concepts).

### 2. The Knowledge Graph Structure

Orunla stores everything in a **SQLite database** with three main components:

**Nodes** (Concepts/Entities):
- Each unique person, place, thing, or idea gets a node
- Nodes have: ID, label (name), type (person/organization/location/etc.)
- Example: `Node { id: "abc123", label: "Sarah", type: Person }`

**Edges** (Relationships/Facts):
- Connect two nodes with a predicate (relationship type)
- Store: source node, target node, predicate, original text, confidence score
- Track: when created, when last accessed, access count
- Example: `Edge { source: "Sarah", predicate: "works at", target: "Microsoft", confidence: 0.95 }`

**Full-Text Search Index** (FTS5):
- SQLite's FTS5 (Full-Text Search) index on all edge source text
- Enables fast keyword searches across all stored memories
- Works alongside graph traversal for hybrid retrieval

### 3. Memory Decay: The Forgetting Curve

This is where Orunla gets smart. Human memory doesn't work like a filing cabinet—we forget things over time, especially things we don't use. Orunla replicates this with the **Ebbinghaus Forgetting Curve**.

**The Formula:**
```
Strength = decay × access_boost × confidence

Where:
- decay = e^(-days_since_access / 30)
- access_boost = 1.0 + ln(1 + access_count)
- confidence = GliNER's confidence score (0.0 to 1.0)
```

**What this means:**

1. **Recency matters**: A memory accessed yesterday has higher strength than one from 60 days ago
2. **Repetition helps**: Memories accessed multiple times get an "access boost" that slows decay
3. **Confidence matters**: Facts extracted with high certainty matter more than uncertain ones
4. **30-day stability**: The "30" is the base stability factor—after 30 days without access, a memory retains about 37% of its original strength

**Example Timeline:**
- Day 0 (fresh memory): strength = 1.0
- Day 30 (not accessed): strength ≈ 0.37
- Day 60 (not accessed): strength ≈ 0.14
- Day 90 (not accessed): strength ≈ 0.05

But if you access a memory 5 times, the access boost (≈ 1.79) slows this decay significantly.

### 4. Hybrid Retrieval: How Recall Works

When you search for information, Orunla uses a **two-pass hybrid search** system:

**Pass 1: Full-Text Search (FTS5)**
- Tokenizes your query into keywords
- Searches the FTS5 index for matching source text
- Uses OR logic: matches any keyword
- Fast and catches semantic variations

**Pass 2: Graph Search**
- Searches node labels (entity names) for matches
- Searches edge predicates (relationship types)
- Uses SQL JSON arrays and LIKE patterns
- Catches structural relationships

**Pass 3: Filtering & Ranking**
- Calculates memory strength for each result
- Filters by minimum confidence threshold (default: 0.0)
- Filters by minimum strength threshold (default: 0.0)
- Sorts by relevance score (strength) descending
- Returns top N results

**Example:**
Query: "Sarah's job"
- FTS5 finds edges with "Sarah" or "job" in source text
- Graph search finds nodes labeled "Sarah" and predicates like "works at"
- Combines results, calculates strength for each
- Returns: `Sarah → works at → Microsoft` (strength: 0.85)

### 5. Garbage Collection: Automatic Forgetting

Over time, your memory graph can fill with old, irrelevant facts. Orunla has **garbage collection (GC)** to clean this up.

**How it works:**
```bash
orunla_cli gc --threshold 0.05
```

1. Scans every edge in the database
2. Calculates current strength for each edge
3. Deletes edges below the threshold (default: 0.05)
4. Removes orphaned nodes (nodes with no remaining edges)

**Why this matters:**
- Keeps your database lean and fast
- Removes noise from search results
- Mimics human memory—forgetting the unimportant
- You can adjust the threshold based on your needs

**Typical usage:**
- Run GC monthly with threshold 0.1 (aggressive - deletes more)
- Or quarterly with default threshold 0.05 (conservative - keeps more)
- Or never, if you want to keep everything

**Note:** The default threshold is 0.05. Higher thresholds delete more memories, lower thresholds keep more.

### 6. Node Deduplication: Keeping It Clean

Humans aren't consistent with naming. You might mention "Rust", "rust", "RUST", "Rust Programming Language"—all referring to the same thing. Orunla handles this with **deduplication**.

**How it works:**
```bash
orunla_cli dedup
```

1. Lists all nodes in the database
2. Normalizes each label (lowercase, alphanumeric only)
3. Groups nodes with identical normalized forms
4. For each group, picks a "winner" node
5. Rebinds all edges from "loser" nodes to the winner
6. Deletes the loser nodes

**Example:**
- Nodes: "Rust", "rust", "RUST"
- Normalized: "rust" (all three map to this)
- Winner: "Rust" (first in group)
- Result: All edges now point to "Rust", others deleted

**Why this matters:**
- Prevents fragmented knowledge (facts split across duplicate nodes)
- Improves recall accuracy
- Reduces database bloat

### 7. Topic-Based Purging

Sometimes you want to completely remove memories about a specific topic. The **purge** command does this.

**How it works:**
```bash
orunla_cli purge --query "Microsoft"
```

1. Runs a full hybrid search for the query
2. Deletes ALL matching edges
3. Cleans up orphaned nodes
4. Returns count of deleted memories

**Use cases:**
- Remove old project information
- Delete memories about a topic you're no longer working on
- Clear out test data
- Comply with data deletion requests

### 8. GliNER Entity Extraction: The AI Engine

GliNER is a neural network that runs **locally** on your computer using ONNX Runtime. It's trained to recognize entities across seven categories:

**Entity Types:**
1. **Person**: Sarah, John, Dr. Smith
2. **Organization**: Microsoft, Stanford University, OpenAI
3. **Location**: Seattle, California, 123 Main Street
4. **Artifact**: iPhone, Constitution, Mona Lisa
5. **Concept**: Machine Learning, Democracy, Photosynthesis
6. **Software**: Python, Visual Studio Code, Linux
7. **Language**: English, Rust, Spanish

**The Extraction Process:**
1. Receives raw text input
2. Tokenizes text and identifies entity spans
3. Classifies each entity by type
4. Extracts text between entities as predicates
5. Forms triplets with confidence scores
6. Returns structured data for storage

**Example:**
Input: *"Python is a programming language created by Guido van Rossum at CWI in Amsterdam."*

Extracted triplets:
- `Python` (software) → `is a` → `programming language` (concept) - confidence: 0.92
- `Python` (software) → `created by` → `Guido van Rossum` (person) - confidence: 0.88
- `Guido van Rossum` (person) → `at` → `CWI` (organization) - confidence: 0.85
- `CWI` (organization) → `in` → `Amsterdam` (location) - confidence: 0.91

**Why local AI matters:**
- No API costs
- Works offline
- Complete privacy
- Fast inference (milliseconds per extraction)

## How You Can Use Orunla

### 1. Desktop App (Windows)

**What it is:** A graphical application with cyberpunk UI
**How to use:**
1. Double-click `Orunla.exe`
2. Type facts in the Ingest box → click "Ingest"
3. Upload files via "Upload File" button
4. Search memories in the Recall tab
5. Purge topics you no longer need

**Best for:** Personal use, quick access, visual interface

### 2. MCP Server (Model Context Protocol)

**What it is:** A background service that lets AI agents (Claude Desktop, Cursor, Cline) read/write your memories
**How to use:**
1. Add to your `claude_desktop_config.json`:
```json
{
  "mcpServers": {
    "orunla": {
      "command": "C:/path/to/orunla_mcp.exe",
      "args": []
    }
  }
}
```
2. Restart Claude Desktop
3. Claude can now store and recall memories automatically

**Best for:** AI agent integration, hands-free memory management

### 3. CLI Tool (Command Line)

**What it is:** Terminal commands for power users
**How to use:**
```bash
# Add a memory
orunla_cli ingest --text "The API key is 12345"

# Search memories
orunla_cli recall --query "API key"

# Upload a file
orunla_cli ingest --file notes.txt

# Run maintenance
orunla_cli gc --threshold 0.1
orunla_cli dedup

# Get statistics
orunla_cli stats
```

**Best for:** Scripting, automation, scheduled maintenance

### 4. REST API (HTTP Server)

**What it is:** Web API for integration with any programming language or no-code tools
**How to use:**
```bash
# Start server
orunla_cli serve --port 3000

# Add memory (POST)
curl -X POST http://localhost:3000/ingest \
  -H "Content-Type: application/json" \
  -d '{"text": "The printer code is 9988"}'

# Search memories (POST)
curl -X POST http://localhost:3000/recall \
  -H "Content-Type: application/json" \
  -d '{"query": "printer", "limit": 10}'

# Purge topic (POST)
curl -X POST http://localhost:3000/purge \
  -H "Content-Type: application/json" \
  -d '{"query": "printer"}'
```

**Best for:** Webhooks, Zapier/Make.com, custom integrations

## Where Your Data Lives

**Database location:**
- Windows: `%USERPROFILE%\.orunla\memory.db`
- Mac/Linux: `~/.orunla/memory.db`

**What's inside:**
- `nodes` table: All entities (people, places, concepts)
- `edges` table: All relationships/facts
- `edges_fts` table: Full-text search index

**Viewing your data:**
You can open `memory.db` with [DB Browser for SQLite](https://sqlitebrowser.org/) to manually browse your knowledge graph.

**Privacy:**
Everything stays on your computer. No cloud sync, no telemetry, no data sharing. Your memories are yours.

## Use Cases: Why Orunla Is Useful

### Personal Assistant Memory
**Scenario:** You're planning your niece Sarah's birthday party.

**Months ago, you told Orunla:**
- "Sarah is allergic to peanuts"
- "Sarah loves Harry Potter"
- "Sarah's birthday is June 15th"

**Now you ask:**
"What should I know about planning Sarah's party?"

**Orunla recalls:**
- Sarah → is allergic to → peanuts (strength: 0.82)
- Sarah → loves → Harry Potter (strength: 0.78)
- Sarah → birthday → June 15th (strength: 0.91)

**Result:** You avoid a dangerous allergen, pick the perfect theme, and remember the date.

---

### Customer Support Knowledge Base
**Scenario:** You run a support team that handles hundreds of questions.

**Team members add policies to Orunla:**
- "Electronics have a 14-day return policy"
- "Software returns require original packaging"
- "Refunds take 5-7 business days to process"

**Customer asks:**
"How long do I have to return my laptop?"

**Support agent searches Orunla:** "laptop return"

**Orunla recalls:**
- Electronics → return policy → 14 days (strength: 0.95)

**Result:** Instant, accurate answer. No searching through documents or asking managers.

---

### Research Knowledge Graph
**Scenario:** You're researching machine learning papers for your thesis.

**You ingest 50 papers into Orunla. It extracts:**
- "Transformer architecture was introduced in Attention Is All You Need"
- "BERT is based on Transformer encoders"
- "GPT-3 uses Transformer decoders"
- "Vaswani et al. published Attention Is All You Need in 2017"

**You search:** "transformer origins"

**Orunla recalls the entire lineage:**
- Transformer → introduced in → Attention Is All You Need
- Attention Is All You Need → published by → Vaswani et al.
- Attention Is All You Need → published in → 2017
- BERT → based on → Transformer encoders
- GPT-3 → uses → Transformer decoders

**Result:** You instantly map the history and evolution of the concept.

---

### Developer Documentation Memory
**Scenario:** You work with dozens of microservices and constantly forget API endpoints, database schemas, and deployment procedures.

**You store facts:**
- "User service runs on port 8080"
- "Auth database connection string is in .env.production"
- "Deploy to staging with ./deploy.sh staging"
- "Redis cache expires after 3600 seconds"

**Six months later, you need to debug production:**

**You search:** "auth database"

**Orunla recalls:**
- Auth database → connection string → .env.production (strength: 0.64)

**You search:** "redis cache"

**Orunla recalls:**
- Redis cache → expires after → 3600 seconds (strength: 0.58)

**Result:** No more hunting through Slack history or outdated wikis.

---

### AI Agent Continuous Memory
**Scenario:** You use Claude Desktop daily for coding help.

**With Orunla MCP integrated:**
- Claude remembers your preferred code style
- Claude recalls past architectural decisions
- Claude knows your project structure
- Claude remembers bugs you've hit before

**Example conversation:**

**You:** "Help me implement the payment endpoint"

**Claude (using Orunla recall):**
"Based on your previous work, I see:
- You prefer Express.js for APIs
- Your auth middleware is in `src/middleware/auth.js`
- You use Stripe for payments
- You want detailed logging for financial operations

Let me create an endpoint following these patterns..."

**Result:** Claude gives contextually accurate help without you repeating yourself every session.

---

### Meeting Notes & Action Items
**Scenario:** You attend meetings all day and lose track of who's responsible for what.

**You ingest meeting notes:**
- "Tom will send the Q4 report by Friday"
- "Lisa is researching new CRM vendors"
- "Budget approval depends on CFO sign-off"

**Next week:**

**You search:** "Q4 report"

**Orunla recalls:**
- Tom → will send → Q4 report by Friday (strength: 0.71)

**You search:** "CRM"

**Orunla recalls:**
- Lisa → is researching → CRM vendors (strength: 0.69)

**Result:** You always know who owns what without re-reading pages of notes.

---

### Learning & Study Aid
**Scenario:** You're learning a new programming language (Rust).

**You ingest tutorials, examples, and notes:**
- "Rust uses ownership to manage memory"
- "The borrow checker prevents data races"
- "Use `&` for immutable references"
- "Use `&mut` for mutable references"

**While coding, you forget syntax:**

**You search:** "mutable reference"

**Orunla recalls:**
- mutable reference → syntax → `&mut` (strength: 0.88)

**Result:** Quick reference without leaving your editor.

## Why Orunla Is Different

### vs. Note-Taking Apps (Notion, Obsidian)
**Traditional apps:** Store text in hierarchies (folders, pages, tags)
**Orunla:** Stores **relationships** in a graph. Search by meaning, not location.

### vs. RAG Systems (LangChain, LlamaIndex)
**Traditional RAG:** Chunks text, embeds it, searches by similarity
**Orunla:** Extracts **structured facts** first, then indexes them. More precise, less hallucination.

### vs. Vector Databases (Pinecone, Weaviate)
**Vector DBs:** Need embeddings, API calls, cloud infrastructure
**Orunla:** Local SQLite + FTS5. No embeddings, no API costs, no latency.

### vs. Cloud Memory (Rewind, Mem.ai)
**Cloud services:** Your data on their servers, subscription fees, privacy concerns
**Orunla:** Local-first. Your data, your computer, your control.

### vs. AI Chat History
**Chat history:** Dumping ground of text, hard to search, no structure
**Orunla:** Structured knowledge graph with decay, GC, and precise recall.

## Technical Architecture Summary

```
┌─────────────────────────────────────────────────────────────┐
│                     Input Layer                             │
│  (Desktop UI, CLI, MCP Server, REST API)                    │
└───────────────────────┬─────────────────────────────────────┘
                        │
                        ▼
┌─────────────────────────────────────────────────────────────┐
│                  GliNER Extractor                           │
│  (ONNX Runtime, Local AI, Entity Recognition)               │
│  Input: Raw text                                            │
│  Output: Triplets (subject, predicate, object, confidence)  │
└───────────────────────┬─────────────────────────────────────┘
                        │
                        ▼
┌─────────────────────────────────────────────────────────────┐
│                 Knowledge Graph Storage                     │
│  • SQLite Database (nodes, edges tables)                    │
│  • FTS5 Full-Text Search Index                              │
│  • Tracks: creation time, access time, access count         │
└───────────────────────┬─────────────────────────────────────┘
                        │
                        ▼
┌─────────────────────────────────────────────────────────────┐
│                 Hybrid Retriever                            │
│  • Pass 1: FTS5 search on source text                       │
│  • Pass 2: Graph search on node labels & predicates         │
│  • Combine results                                          │
│  • Calculate strength (Ebbinghaus decay)                    │
│  • Filter by confidence & strength                          │
│  • Rank by relevance                                        │
└───────────────────────┬─────────────────────────────────────┘
                        │
                        ▼
┌─────────────────────────────────────────────────────────────┐
│                 Maintenance Layer                           │
│  • Garbage Collection (delete weak memories)                │
│  • Node Deduplication (merge duplicates)                    │
│  • Orphan Cleanup (remove disconnected nodes)               │
│  • Topic Purging (delete by query)                          │
└─────────────────────────────────────────────────────────────┘
```

## Performance Characteristics

**Ingestion Speed:**
- ~100-500ms per triplet extraction (depends on text length)
- Batch processing supported for large files
- No cloud API calls = consistent speed

**Recall Speed:**
- Typical query: 10-50ms
- FTS5 + graph search in parallel
- Sub-second response even with 100,000+ edges

**Storage:**
- ~1KB per edge (relationship)
- ~100 bytes per node (entity)
- 10,000 facts ≈ 10MB database
- SQLite handles millions of records efficiently

**Memory Usage:**
- GliNER model: ~200MB RAM when loaded
- SQLite: Minimal (kilobytes for typical queries)
- Desktop app: ~50-100MB total

## System Requirements

**Minimum:**
- OS: Windows 10+, macOS 11+, Linux (Ubuntu 20.04+)
- RAM: 2GB available
- Disk: 500MB (includes AI model)
- CPU: Any modern x64 processor

**Recommended:**
- RAM: 4GB+ for large datasets
- SSD for faster database operations

## Future Possibilities

While Orunla is feature-complete today, potential enhancements could include:

- **Temporal queries**: "What did I know about X in January?"
- **Confidence decay**: Low-confidence facts decay faster
- **Graph visualization**: See your knowledge web visually
- **Multi-database sync**: Share memories across devices (opt-in)
- **Custom entity types**: Define your own entity categories
- **Embedding search**: Optional vector similarity (hybrid mode)
- **Conflict resolution**: Handle contradictory facts intelligently

## Summary

**Orunla is:**
- A local-first AI memory system
- Built on knowledge graphs (nodes + edges)
- Powered by GliNER entity extraction
- Uses Ebbinghaus forgetting curve for decay
- Combines FTS5 text search + graph traversal
- Offers desktop UI, MCP integration, CLI, and REST API
- Completely private (everything stays on your computer)
- Zero cloud dependencies, zero API costs

**It's useful for:**
- Personal knowledge management
- AI agent memory (Claude, custom agents)
- Research and learning
- Customer support knowledge bases
- Developer documentation
- Meeting notes and task tracking
- Any scenario where you need to remember relationships between facts

**Core philosophy:**
Human memory isn't perfect—it forgets the unimportant and reinforces the important through repetition. Orunla replicates this digitally, giving you and your AI tools a memory system that's smart, private, and built to last.
