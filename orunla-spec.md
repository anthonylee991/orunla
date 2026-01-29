# Orunla

## Technical Specification Document

**Version:** 0.1.0  
**Date:** December 2024  
**Status:** Draft

---

## 1. Vision

Orunla is a local-first, intelligent memory system for AI agents. Named after the Yoruba Orisha of wisdom, divination, and knowledge, Orunla provides agents with the ability to remember, recall, and selectively forget—mimicking how human memory actually works.

### 1.1 Problem Statement

Current AI agent memory solutions suffer from critical limitations:

- **Unbounded growth**: Memory files (agent.md, memory.json) grow indefinitely until they exceed context limits
- **No relevance filtering**: The entire memory is loaded into context, wasting tokens on irrelevant information
- **No forgetting mechanism**: Old, unused memories persist forever, cluttering the knowledge base
- **Developer-only tooling**: Existing solutions require technical expertise to deploy and configure

### 1.2 Solution

Orunla provides:

- **Intelligent extraction**: Automatically extracts structured knowledge (subject-predicate-object triplets) from conversations without LLM inference
- **Selective recall**: Retrieves only memories relevant to the current context
- **Graceful forgetting**: Memories decay over time based on access patterns, with unused memories consolidated or purged
- **Zero-config deployment**: Non-technical users can install and run Orunla with a single click

### 1.3 Target Users

**Primary:** Non-developers building AI agents with no-code tools (n8n, Make, Zapier, Open WebUI, etc.)

**Secondary:** Developers who want a drop-in memory layer for their agent systems

---

## 2. Development Principles

These principles are non-negotiable and must guide every implementation decision.

### 2.1 Small, Minimalist Code

- Each module should do one thing well
- Files should be small and focused (target: <300 lines per file)
- Prefer standard library functions over external dependencies
- No premature abstraction—write concrete code first, abstract only when patterns emerge
- If a function exceeds 50 lines, it probably needs to be split

### 2.2 Test Every Function

- Every function must have corresponding unit tests before the feature is considered complete
- Tests are written immediately after (or before) the function, not "later"
- When a feature is complete, run integration tests covering the full pipeline
- Test edge cases: empty inputs, malformed data, concurrent access
- Maintain >80% code coverage as a baseline

### 2.3 Usefulness Over Ease

- Every feature decision starts with: "How does this help the end user?"
- Developer convenience never trumps user experience
- If something is hard to implement but makes the user's life easier, do the hard thing
- The best code is code the user never has to think about

---

## 3. Architecture Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                        Tauri Desktop App                        │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │                    Web UI (TypeScript)                    │  │
│  │  - Dashboard: memory stats, recent activity               │  │
│  │  - Graph visualization                                    │  │
│  │  - Settings panel                                         │  │
│  │  - Manual memory input                                    │  │
│  └───────────────────────────────────────────────────────────┘  │
│                              │                                   │
│                        Tauri Bridge                              │
│                              │                                   │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │                     Rust Core Library                     │  │
│  │                                                           │  │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────────┐   │  │
│  │  │  Extractor  │  │    Graph    │  │    Retriever    │   │  │
│  │  │   Module    │  │   Module    │  │     Module      │   │  │
│  │  └─────────────┘  └─────────────┘  └─────────────────┘   │  │
│  │                                                           │  │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────────┐   │  │
│  │  │  Forgetting │  │   Storage   │  │   HTTP Server   │   │  │
│  │  │   Module    │  │   Module    │  │     Module      │   │  │
│  │  └─────────────┘  └─────────────┘  └─────────────────┘   │  │
│  │                                                           │  │
│  └───────────────────────────────────────────────────────────┘  │
│                              │                                   │
│                         SQLite File                              │
│                    (~/.orunla/memory.db)                        │
└─────────────────────────────────────────────────────────────────┘
                               │
                               │ HTTP API (localhost:7432)
                               ▼
┌─────────────────────────────────────────────────────────────────┐
│                      External Integrations                       │
│  - n8n / Make / Zapier webhooks                                 │
│  - Agent frameworks (LangChain, AutoGPT, etc.)                  │
│  - Local LLM UIs (Open WebUI, etc.)                             │
│  - CLI tools                                                     │
└─────────────────────────────────────────────────────────────────┘
```

### 3.1 Technology Choices

| Component | Technology | Rationale |
|-----------|------------|-----------|
| Core Library | Rust | Performance, single binary distribution, memory safety |
| NLP Extraction | rust-stemmers, nlprule, or custom rules | No external service dependencies, local-first |
| Graph Storage | SQLite with custom schema | Single file, portable, no server process |
| Embedding (optional) | fastembed-rs or candle | Local embeddings, no API calls |
| Desktop App | Tauri | Small binary size (~10MB vs Electron's 150MB+), native performance |
| Web UI | TypeScript + Svelte or Solid | Lightweight, fast, good Tauri integration |
| HTTP Server | axum or actix-web | Rust-native, performant |

---

## 4. Core Modules

### 4.1 Extractor Module

**Purpose:** Transform unstructured text into structured knowledge triplets.

**Approach:** Rule-based extraction using dependency parsing and pattern matching. No LLM required.

#### 4.1.1 Input

```rust
pub struct ExtractionRequest {
    pub text: String,
    pub source_id: Option<String>,  // For provenance tracking
    pub timestamp: Option<DateTime<Utc>>,
}
```

#### 4.1.2 Output

```rust
pub struct Triplet {
    pub subject: String,
    pub predicate: String,
    pub object: String,
    pub confidence: f32,        // 0.0 - 1.0
    pub source_span: (usize, usize),  // Character offsets in original text
}

pub struct ExtractionResult {
    pub triplets: Vec<Triplet>,
    pub entities: Vec<Entity>,  // Named entities found
    pub raw_text: String,
}
```

#### 4.1.3 Extraction Pipeline

```
Input Text
    │
    ▼
┌─────────────────┐
│   Tokenization  │  Split into sentences and tokens
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  POS Tagging    │  Identify nouns, verbs, adjectives, etc.
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Dependency Parse│  Build syntactic tree
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Pattern Matching│  Extract SPO triplets using rules
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Entity Linking  │  Resolve coreferences ("it" → "the project")
└────────┬────────┘
         │
         ▼
Triplets + Entities
```

#### 4.1.4 Pattern Rules (Examples)

```rust
// Rule: Subject + Active Verb + Object
// "John uses Python" → (John, uses, Python)
Pattern::new()
    .subject(POS::Noun | POS::ProperNoun)
    .predicate(POS::Verb)
    .object(POS::Noun | POS::ProperNoun)

// Rule: Subject + "is/are" + Attribute
// "The project is complete" → (project, is, complete)
Pattern::new()
    .subject(POS::Noun)
    .predicate(Token::OneOf(&["is", "are", "was", "were"]))
    .object(POS::Adjective | POS::Noun)

// Rule: Subject + Verb + Prepositional Object
// "User works at Anthropic" → (User, works_at, Anthropic)
Pattern::new()
    .subject(POS::Noun)
    .predicate(POS::Verb)
    .preposition(Token::OneOf(&["at", "in", "on", "with", "for"]))
    .object(POS::Noun | POS::ProperNoun)
```

#### 4.1.5 Files

```
src/extractor/
├── mod.rs          # Public API
├── tokenizer.rs    # Sentence/word tokenization
├── pos_tagger.rs   # Part-of-speech tagging
├── parser.rs       # Dependency parsing
├── patterns.rs     # SPO extraction rules
├── entities.rs     # Named entity recognition
└── coreference.rs  # Pronoun resolution
```

---

### 4.2 Graph Module

**Purpose:** Store and query the knowledge graph.

#### 4.2.1 Data Model

```rust
pub struct Node {
    pub id: NodeId,
    pub label: String,           // Normalized entity name
    pub aliases: Vec<String>,    // Alternative names ("FastAPI", "fastapi", "Fast API")
    pub node_type: NodeType,     // Person, Project, Technology, Concept, etc.
    pub created_at: DateTime<Utc>,
    pub metadata: HashMap<String, Value>,
}

pub struct Edge {
    pub id: EdgeId,
    pub source: NodeId,
    pub target: NodeId,
    pub predicate: String,       // The relationship type
    pub created_at: DateTime<Utc>,
    pub last_accessed: DateTime<Utc>,
    pub access_count: u32,
    pub source_text: String,     // Original text this was extracted from
    pub source_id: Option<String>,
    pub confidence: f32,
}

pub enum NodeType {
    Person,
    Project,
    Technology,
    Organization,
    Concept,
    Location,
    DateTime,
    Unknown,
}
```

#### 4.2.2 SQLite Schema

```sql
-- Nodes table
CREATE TABLE nodes (
    id TEXT PRIMARY KEY,
    label TEXT NOT NULL,
    node_type TEXT NOT NULL,
    created_at TEXT NOT NULL,
    metadata TEXT  -- JSON
);

CREATE INDEX idx_nodes_label ON nodes(label);
CREATE INDEX idx_nodes_type ON nodes(node_type);

-- Aliases for entity resolution
CREATE TABLE node_aliases (
    alias TEXT PRIMARY KEY,
    node_id TEXT NOT NULL REFERENCES nodes(id)
);

CREATE INDEX idx_aliases_node ON node_aliases(node_id);

-- Edges table
CREATE TABLE edges (
    id TEXT PRIMARY KEY,
    source_id TEXT NOT NULL REFERENCES nodes(id),
    target_id TEXT NOT NULL REFERENCES nodes(id),
    predicate TEXT NOT NULL,
    created_at TEXT NOT NULL,
    last_accessed TEXT NOT NULL,
    access_count INTEGER DEFAULT 0,
    source_text TEXT,
    source_ref TEXT,
    confidence REAL DEFAULT 1.0
);

CREATE INDEX idx_edges_source ON edges(source_id);
CREATE INDEX idx_edges_target ON edges(target_id);
CREATE INDEX idx_edges_predicate ON edges(predicate);
CREATE INDEX idx_edges_accessed ON edges(last_accessed);
CREATE INDEX idx_edges_count ON edges(access_count);

-- Full-text search on source_text
CREATE VIRTUAL TABLE edges_fts USING fts5(source_text, content=edges, content_rowid=rowid);
```

#### 4.2.3 Core Operations

```rust
pub trait GraphStore {
    // Node operations
    fn add_node(&mut self, node: Node) -> Result<NodeId>;
    fn get_node(&self, id: &NodeId) -> Result<Option<Node>>;
    fn find_node_by_label(&self, label: &str) -> Result<Option<Node>>;
    fn resolve_entity(&self, text: &str) -> Result<Option<NodeId>>;  // Check aliases
    
    // Edge operations
    fn add_edge(&mut self, edge: Edge) -> Result<EdgeId>;
    fn get_edges_from(&self, node_id: &NodeId) -> Result<Vec<Edge>>;
    fn get_edges_to(&self, node_id: &NodeId) -> Result<Vec<Edge>>;
    fn touch_edge(&mut self, edge_id: &EdgeId) -> Result<()>;  // Update access time/count
    
    // Graph traversal
    fn neighborhood(&self, node_id: &NodeId, depth: u32) -> Result<SubGraph>;
    fn shortest_path(&self, from: &NodeId, to: &NodeId) -> Result<Option<Vec<EdgeId>>>;
    
    // Search
    fn search_edges(&self, query: &str, limit: usize) -> Result<Vec<Edge>>;
}
```

#### 4.2.4 Files

```
src/graph/
├── mod.rs          # Public API, GraphStore trait
├── sqlite.rs       # SQLite implementation
├── node.rs         # Node struct and operations
├── edge.rs         # Edge struct and operations
├── query.rs        # Graph traversal algorithms
└── entity.rs       # Entity resolution logic
```

---

### 4.3 Retriever Module

**Purpose:** Given a query, retrieve the most relevant memories.

#### 4.3.1 Retrieval Strategy

```
Query Text
    │
    ├─────────────────────────────────┐
    │                                 │
    ▼                                 ▼
┌─────────────────┐          ┌─────────────────┐
│ Entity Extraction│          │  FTS Search     │
│ (from query)     │          │  (on edge text) │
└────────┬────────┘          └────────┬────────┘
         │                            │
         ▼                            │
┌─────────────────┐                   │
│ Graph Traversal │                   │
│ (neighborhood)  │                   │
└────────┬────────┘                   │
         │                            │
         └──────────┬─────────────────┘
                    │
                    ▼
           ┌─────────────────┐
           │  Merge & Rank   │
           │  (by relevance) │
           └────────┬────────┘
                    │
                    ▼
           ┌─────────────────┐
           │  Touch Edges    │
           │  (update access)│
           └────────┬────────┘
                    │
                    ▼
             Top K Memories
```

#### 4.3.2 API

```rust
pub struct RecallRequest {
    pub query: String,
    pub limit: usize,           // Default: 5
    pub min_confidence: f32,    // Default: 0.5
    pub include_context: bool,  // Include surrounding graph context
}

pub struct Memory {
    pub edge: Edge,
    pub subject_node: Node,
    pub object_node: Node,
    pub relevance_score: f32,
}

pub struct RecallResponse {
    pub memories: Vec<Memory>,
    pub query_entities: Vec<String>,  // Entities found in query
}

pub trait Retriever {
    fn recall(&self, request: RecallRequest) -> Result<RecallResponse>;
}
```

#### 4.3.3 Files

```
src/retriever/
├── mod.rs          # Public API
├── search.rs       # FTS and entity-based search
├── ranking.rs      # Relevance scoring
└── context.rs      # Graph context expansion
```

---

### 4.4 Forgetting Module

**Purpose:** Manage memory lifecycle through decay, consolidation, and pruning.

#### 4.4.1 Memory Strength Calculation

```rust
pub fn calculate_strength(edge: &Edge, now: DateTime<Utc>) -> f32 {
    let age_days = (now - edge.created_at).num_days() as f32;
    let recency_days = (now - edge.last_accessed).num_days() as f32;
    
    // Ebbinghaus forgetting curve: R = e^(-t/S)
    // Where t = time since last access, S = stability factor
    let stability = 30.0;  // Base stability in days
    let decay = (-recency_days / stability).exp();
    
    // Access count provides "spacing effect" boost
    // More accesses = slower decay
    let access_boost = (1.0 + edge.access_count as f32).ln();
    
    // Original confidence matters
    let confidence = edge.confidence;
    
    decay * access_boost * confidence
}
```

#### 4.4.2 Forgetting Operations

```rust
pub struct ForgettingConfig {
    pub min_age_days: u32,           // Don't touch memories younger than this
    pub strength_threshold: f32,      // Below this = candidate for action
    pub consolidation_threshold: f32, // Above prune threshold but below this = consolidate
    pub max_memories: Option<usize>,  // Hard cap on total memories
}

pub trait Forgetter {
    /// Find memories that are candidates for forgetting
    fn find_weak_memories(&self, config: &ForgettingConfig) -> Result<Vec<Edge>>;
    
    /// Consolidate similar weak memories into summary memories
    fn consolidate(&mut self, edges: &[EdgeId]) -> Result<Option<Edge>>;
    
    /// Permanently remove memories
    fn prune(&mut self, edges: &[EdgeId]) -> Result<usize>;
    
    /// Run full forgetting cycle
    fn forget_cycle(&mut self, config: &ForgettingConfig) -> Result<ForgettingReport>;
}

pub struct ForgettingReport {
    pub analyzed: usize,
    pub consolidated: usize,
    pub pruned: usize,
    pub retained: usize,
}
```

#### 4.4.3 Consolidation Strategy

When multiple weak memories share entities or predicates, consolidate them:

```
Before:
- (User, asked_about, Portuguese_recipes) [access: 1, age: 30d]
- (User, asked_about, Bife_Madeirense) [access: 1, age: 25d]
- (User, asked_about, seafood_rice) [access: 1, age: 20d]

After consolidation:
- (User, interested_in, Portuguese_cooking) [access: 3, age: 20d]
  source_text: "Consolidated from 3 recipe-related queries"
```

#### 4.4.4 Files

```
src/forgetting/
├── mod.rs          # Public API
├── strength.rs     # Memory strength calculation
├── consolidate.rs  # Memory merging logic
├── prune.rs        # Deletion logic
└── scheduler.rs    # Background job scheduling
```

---

### 4.5 Storage Module

**Purpose:** Unified storage abstraction and database management.

#### 4.5.1 API

```rust
pub struct StorageConfig {
    pub path: PathBuf,           // Default: ~/.orunla/memory.db
    pub backup_enabled: bool,
    pub backup_interval_hours: u32,
}

pub trait Storage {
    fn init(&mut self) -> Result<()>;
    fn graph(&self) -> &dyn GraphStore;
    fn graph_mut(&mut self) -> &mut dyn GraphStore;
    
    fn backup(&self) -> Result<PathBuf>;
    fn restore(&mut self, backup_path: &Path) -> Result<()>;
    fn export_json(&self) -> Result<String>;
    fn import_json(&mut self, json: &str) -> Result<ImportReport>;
    
    fn stats(&self) -> Result<StorageStats>;
}

pub struct StorageStats {
    pub node_count: usize,
    pub edge_count: usize,
    pub db_size_bytes: u64,
    pub oldest_memory: Option<DateTime<Utc>>,
    pub newest_memory: Option<DateTime<Utc>>,
}
```

#### 4.5.2 Files

```
src/storage/
├── mod.rs          # Public API
├── sqlite.rs       # SQLite implementation
├── backup.rs       # Backup/restore logic
└── export.rs       # JSON import/export
```

---

### 4.6 HTTP Server Module

**Purpose:** Expose Orunla's functionality via a local REST API for external integrations.

#### 4.6.1 Endpoints

```
POST /ingest
    Body: { "text": "...", "source_id": "optional" }
    Response: { "triplets": [...], "memory_ids": [...] }

POST /recall
    Body: { "query": "...", "limit": 5 }
    Response: { "memories": [...] }

GET /memories
    Query: ?limit=50&offset=0&sort=recent
    Response: { "memories": [...], "total": 1234 }

GET /memories/:id
    Response: { "memory": {...}, "context": {...} }

DELETE /memories/:id
    Response: { "deleted": true }

GET /graph/node/:id
    Response: { "node": {...}, "edges": [...] }

GET /graph/search
    Query: ?q=FastAPI&limit=10
    Response: { "nodes": [...], "edges": [...] }

POST /forget
    Body: { "strategy": "decay", "dry_run": true }
    Response: { "would_prune": 45, "would_consolidate": 12 }

GET /stats
    Response: { "node_count": ..., "edge_count": ..., ... }

GET /health
    Response: { "status": "ok", "version": "0.1.0" }
```

#### 4.6.2 Files

```
src/server/
├── mod.rs          # Server startup, configuration
├── routes.rs       # Route definitions
├── handlers.rs     # Request handlers
└── middleware.rs   # Logging, CORS, etc.
```

---

### 4.7 CLI Module

**Purpose:** Command-line interface for power users and scripting.

#### 4.7.1 Commands

```bash
# Ingest text
orunla ingest "User prefers dark mode and uses FastAPI"
orunla ingest --file conversation.txt
cat chat.log | orunla ingest --stdin

# Recall memories
orunla recall "What framework does the user prefer?"
orunla recall --limit 10 --json "tech stack"

# Memory management
orunla list --limit 20 --sort recent
orunla show <memory_id>
orunla delete <memory_id>

# Graph exploration
orunla graph node <node_id>
orunla graph search "Python"
orunla graph export --format dot > graph.dot

# Forgetting
orunla forget --dry-run
orunla forget --strategy decay

# Server
orunla serve --port 7432
orunla serve --background

# Maintenance
orunla stats
orunla backup
orunla restore backup-2024-01-15.db
orunla export > memories.json
orunla import < memories.json
```

#### 4.7.2 Files

```
src/cli/
├── mod.rs          # CLI setup, argument parsing
├── commands/
│   ├── ingest.rs
│   ├── recall.rs
│   ├── memory.rs
│   ├── graph.rs
│   ├── forget.rs
│   ├── serve.rs
│   └── maintenance.rs
└── output.rs       # Formatting (table, JSON, etc.)
```

---

## 5. Desktop Application

### 5.1 Technology

- **Framework:** Tauri 2.0
- **Frontend:** TypeScript + Svelte (or Solid)
- **Styling:** Tailwind CSS
- **State:** Svelte stores (or Solid signals)

### 5.2 Features

#### 5.2.1 Dashboard View

```
┌─────────────────────────────────────────────────────────────┐
│  Orunla                                    ● Running   [—][×]│
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────┐ │
│  │    1,247        │  │      89%        │  │   2.3 MB    │ │
│  │   Memories      │  │    Strength     │  │  Database   │ │
│  └─────────────────┘  └─────────────────┘  └─────────────┘ │
│                                                             │
│  API Endpoint: http://localhost:7432            [Copy URL]  │
│                                                             │
│  ┌─────────────────────────────────────────────────────────┐│
│  │ Recent Activity                                         ││
│  │                                                         ││
│  │ 2 min ago   Ingested: "User prefers FastAPI over..."   ││
│  │ 5 min ago   Recalled: "tech stack" → 3 memories        ││
│  │ 12 min ago  Ingested: "Project deadline is Q1..."      ││
│  │ 1 hour ago  Forgot: 12 memories consolidated           ││
│  └─────────────────────────────────────────────────────────┘│
│                                                             │
│  [View Graph]  [Browse Memories]  [Settings]                │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

#### 5.2.2 Memory Browser

```
┌─────────────────────────────────────────────────────────────┐
│  Memories                                      [Search...]  │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  Sort: [Recent ▼]  Filter: [All Types ▼]                   │
│                                                             │
│  ┌─────────────────────────────────────────────────────────┐│
│  │ User → prefers → FastAPI                                ││
│  │ "User said they prefer FastAPI for the backend"         ││
│  │ Created: 2 days ago  Accessed: 5 times  Strength: 94%   ││
│  │                                            [View] [Del] ││
│  ├─────────────────────────────────────────────────────────┤│
│  │ Project → uses → PostgreSQL                             ││
│  │ "The project database is PostgreSQL 15"                 ││
│  │ Created: 1 week ago  Accessed: 3 times  Strength: 78%   ││
│  │                                            [View] [Del] ││
│  ├─────────────────────────────────────────────────────────┤│
│  │ User → interested_in → Portuguese_cooking               ││
│  │ "Consolidated from 3 recipe queries"                    ││
│  │ Created: 2 weeks ago  Accessed: 1 time  Strength: 45%   ││
│  │                                            [View] [Del] ││
│  └─────────────────────────────────────────────────────────┘│
│                                                             │
│  Showing 1-20 of 1,247                    [< Prev] [Next >] │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

#### 5.2.3 Graph Visualization

Interactive node-link diagram showing the knowledge graph. Features:

- Pan and zoom
- Click node to see its edges
- Search to highlight matching nodes
- Filter by node type
- Color-code by memory strength

#### 5.2.4 Settings

```
┌─────────────────────────────────────────────────────────────┐
│  Settings                                                   │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  Server                                                     │
│  ────────────────────────────────────────                   │
│  Port: [7432    ]                                           │
│  [✓] Start on system boot                                   │
│  [✓] Run in background when closed                          │
│                                                             │
│  Memory Management                                          │
│  ────────────────────────────────────────                   │
│  Auto-forget: [✓] Enabled                                   │
│  Run every: [24] hours                                      │
│  Strength threshold: [0.3]                                  │
│  Minimum age before forget: [7] days                        │
│                                                             │
│  Storage                                                    │
│  ────────────────────────────────────────                   │
│  Database location: ~/.orunla/memory.db      [Change...]    │
│  [✓] Auto-backup enabled                                    │
│  Backup every: [24] hours                                   │
│  Keep backups: [7] days                                     │
│                                                             │
│                                         [Reset] [Save]      │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### 5.3 Files Structure

```
app/
├── src-tauri/
│   ├── src/
│   │   ├── main.rs       # Tauri entry point
│   │   ├── commands.rs   # Tauri commands (bridge to core)
│   │   └── tray.rs       # System tray logic
│   ├── Cargo.toml
│   └── tauri.conf.json
├── src/
│   ├── lib/
│   │   ├── api.ts        # API client
│   │   ├── stores.ts     # State management
│   │   └── types.ts      # TypeScript types
│   ├── components/
│   │   ├── Dashboard.svelte
│   │   ├── MemoryBrowser.svelte
│   │   ├── GraphView.svelte
│   │   ├── Settings.svelte
│   │   └── common/
│   │       ├── Button.svelte
│   │       ├── Card.svelte
│   │       └── ...
│   ├── App.svelte
│   └── main.ts
├── package.json
└── vite.config.ts
```

---

## 6. Testing Strategy

### 6.1 Unit Tests

Every module has a corresponding test file:

```
src/
├── extractor/
│   ├── mod.rs
│   ├── tokenizer.rs
│   └── tests/
│       ├── tokenizer_test.rs
│       ├── patterns_test.rs
│       └── ...
```

Test requirements:

- Every public function has at least one test
- Edge cases are explicitly tested (empty input, malformed data, unicode, etc.)
- Tests are deterministic (no random data without seeds)

### 6.2 Integration Tests

Located in `tests/` directory at project root:

```
tests/
├── ingest_recall_test.rs    # Full pipeline: text → triplets → storage → recall
├── forgetting_test.rs       # Memory decay and consolidation
├── api_test.rs              # HTTP API endpoints
└── fixtures/
    ├── conversations/       # Sample conversation data
    └── expected/            # Expected extraction results
```

### 6.3 Test Commands

```bash
# Run all tests
cargo test

# Run tests for specific module
cargo test extractor

# Run with coverage
cargo tarpaulin --out Html

# Run integration tests only
cargo test --test '*'
```

### 6.4 Continuous Integration

Every PR must:

1. Pass all existing tests
2. Include tests for new functionality
3. Maintain >80% code coverage
4. Pass clippy lints with no warnings

---

## 7. Development Phases

### Phase 1: Core Library (Weeks 1-4)

**Goal:** Working Rust library with CLI

**Deliverables:**

- [ ] Extractor module with basic patterns
- [ ] Graph module with SQLite storage
- [ ] Retriever module with FTS search
- [ ] Forgetting module with decay calculation
- [ ] CLI with ingest, recall, forget commands
- [ ] Unit tests for all modules
- [ ] Integration tests for full pipeline

**Milestone:** `orunla ingest "..." && orunla recall "..."` works

### Phase 2: HTTP Server (Weeks 5-6)

**Goal:** Local API for external integrations

**Deliverables:**

- [ ] HTTP server with all endpoints
- [ ] API documentation (OpenAPI spec)
- [ ] Example integration scripts (curl, Python, JavaScript)
- [ ] Server tests

**Milestone:** Can integrate with n8n/Make via HTTP

### Phase 3: Desktop App (Weeks 7-10)

**Goal:** One-click installable application

**Deliverables:**

- [ ] Tauri app shell
- [ ] Dashboard view
- [ ] Memory browser
- [ ] Settings panel
- [ ] System tray integration
- [ ] Auto-start on boot
- [ ] Windows installer (.msi)
- [ ] macOS installer (.dmg)
- [ ] Linux packages (.deb, .AppImage)

**Milestone:** Non-dev can download, install, and run

### Phase 4: Graph Visualization (Weeks 11-12)

**Goal:** Interactive knowledge graph explorer

**Deliverables:**

- [ ] Force-directed graph layout
- [ ] Node/edge interactions
- [ ] Search and filter
- [ ] Export as image

**Milestone:** Users can visually explore their memory graph

### Phase 5: Polish & Launch (Weeks 13-14)

**Goal:** Production-ready release

**Deliverables:**

- [ ] Performance optimization
- [ ] Documentation site
- [ ] Tutorial videos
- [ ] Landing page
- [ ] GitHub release automation

**Milestone:** Public v1.0.0 release

---

## 8. Future Considerations

Items explicitly out of scope for v1 but worth noting:

- **Optional embedding support:** Hybrid retrieval with local embeddings
- **Multi-user support:** Separate memory namespaces
- **Cloud sync:** Optional backup to user's cloud storage
- **Plugin system:** Custom extractors, storage backends
- **Mobile apps:** iOS/Android companions
- **LLM-assisted consolidation:** Use local LLM for smarter memory merging

---

## 9. Resources

### 9.1 Reference Projects

- [Leonata](https://www.leonata.io/) - Rule-based knowledge graph extraction
- [Mem0](https://github.com/mem0ai/mem0) - LLM memory layer (different approach, useful for API design reference)
- [Khoj](https://github.com/khoj-ai/khoj) - Local-first AI assistant with memory

### 9.2 Rust Libraries to Evaluate

- `rust-stemmers` - Word stemming
- `nlprule` - Rule-based NLP
- `rust-bert` - If transformer-based extraction needed
- `fastembed-rs` - Local embeddings
- `rusqlite` - SQLite bindings
- `axum` - HTTP server
- `clap` - CLI argument parsing
- `tauri` - Desktop app framework

### 9.3 Documentation

- [Tauri Guides](https://tauri.app/v1/guides/)
- [SQLite Full-Text Search](https://www.sqlite.org/fts5.html)
- [Svelte Tutorial](https://svelte.dev/tutorial)

---

## 10. Glossary

| Term | Definition |
|------|------------|
| **Triplet** | A subject-predicate-object fact extracted from text |
| **Node** | An entity in the knowledge graph (person, project, concept, etc.) |
| **Edge** | A relationship between two nodes, representing a memory |
| **Memory** | An edge plus its context (source text, metadata) |
| **Strength** | A 0-1 score indicating how "alive" a memory is based on recency and access |
| **Consolidation** | Merging multiple weak memories into a summary memory |
| **Pruning** | Permanently deleting weak memories |
| **Recall** | Retrieving memories relevant to a query |
| **Ingest** | Processing text to extract and store memories |

---

*Document prepared for Orunla development team. For questions, contact the project lead.*
