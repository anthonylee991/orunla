# RAG.PC Graph Extraction Pipeline — Full Handoff

> **Purpose**: Comprehensive technical handoff for replicating RAG.PC's graph extraction pipeline in other projects (Orunla, CGC). Covers every model, threshold, pattern, schema, and storage mechanism.

---

## Architecture Overview

RAG.PC uses a **multi-stage extraction pipeline** with four extraction methods working together:

```
Document Text/Data
    ↓
[Router] — Determines structured (CSV/JSON) vs unstructured (text)
    ├─→ UNSTRUCTURED PATH:
    │     1. Pattern Matcher (50+ regex patterns, high precision)
    │     2. Domain Router (embed doc → match to industry pack)
    │     3. Unified Extractor:
    │          - spaCy tokenizer (en_core_web_sm)
    │          - GliNER entity extraction
    │          - GliREL relation extraction
    │     → Triplets (subject-predicate-object)
    │
    └─→ STRUCTURED PATH:
          StructuredExtractor (hub-and-spoke model)
          → Triplets
    ↓
[Deduplication & Filtering]
    ↓
[Triplets → Nodes/Edges]
    ↓
[Apache AGE Graph Storage (PostgreSQL)]
```

---

## 1. GliNER — Entity Extraction

**File**: `src/ingestion/extractors/gliner_extractor.py`
**Model**: `urchade/gliner_medium-v2.1`
**Threshold**: `0.5`
**Max Labels Per Call**: `20`
**Max Entity Length**: `60 chars`

### Label Batches (4 batches, ~10 labels each)

```python
LABEL_BATCHES = {
    "core": [
        "person", "organization", "location", "date", "money",
        "product", "service", "project", "event", "document"
    ],
    "technical": [
        "technology", "software", "api", "database", "framework",
        "programming language", "platform", "tool", "library", "protocol"
    ],
    "business": [
        "company", "brand", "industry", "department", "role",
        "client", "customer", "supplier", "partner", "competitor"
    ],
    "financial": [
        "price", "budget", "revenue", "payment", "invoice",
        "transaction", "expense", "account", "contract", "deal"
    ]
}
```

### Label Normalization

Maps similar labels to canonical forms for graph consistency:

```python
LABEL_MAPPING = {
    "company": "organization",
    "brand": "organization",
    "client": "person",
    "framework": "technology",
    "library": "technology",
    "tool": "technology",
    # ... more mappings
}
```

### Key Methods

| Method | Purpose |
|--------|---------|
| `extract_entities(text, labels, threshold, batched)` | Main extraction with optional batching |
| `extract_comprehensive(text, threshold)` | Runs all 4 label batches sequentially |
| `_filter_entities(entities)` | Removes garbage entities |
| `_is_garbage_entity(text)` | Filters gerunds, mission statements, sentences with verbs, colons, long entities |

### Garbage Filtering Rules

- Pronouns (our, we, us, they, my, your, etc.)
- Section headers (overview, summary, conclusion, features, pricing, etc.)
- Gerunds (words ending in "-ing" that aren't proper nouns)
- Mission/vision statements
- Sentences containing verbs
- Entities with colons
- Entities > 60 characters

---

## 2. GliREL — Relation Extraction

**File**: `src/ingestion/extractors/glirel_extractor.py`
**Model**: `jackboyla/glirel-large-v0`
**Threshold**: `0.5`

### Relation Schema with Type Constraints

Each relation defines valid head/tail entity types to prevent nonsensical relations:

```python
RELATION_SCHEMA = {
    "founded": {
        "allowed_head": ["person"],
        "allowed_tail": ["organization", "company"]
    },
    "works at": {
        "allowed_head": ["person"],
        "allowed_tail": ["organization", "company"]
    },
    "CEO of": {
        "allowed_head": ["person"],
        "allowed_tail": ["organization", "company"]
    },
    "manages": {
        "allowed_head": ["person"],
        "allowed_tail": ["person", "team", "department", "project"]
    },
    "headquartered in": {
        "allowed_head": ["organization", "company"],
        "allowed_tail": ["location", "city", "country"]
    },
    # ... 22 total relations
}
```

### Predicate Normalization

Maps GliREL output to graph edge labels:

```python
PREDICATE_NORMALIZATION = {
    "founded": "FOUNDED",
    "leads": "LEADS",
    "ceo of": "LEADS",
    "works at": "WORKS_AT",
    "member of": "MEMBER_OF",
    "reports to": "REPORTS_TO",
    "manages": "MANAGES",
    "headquartered in": "LOCATED_IN",
    "uses": "USES",
    "acquired": "ACQUIRED",
    # ... 20 total mappings
}
```

### Key Methods

| Method | Purpose |
|--------|---------|
| `extract_relations(text, entities, relation_labels, threshold)` | Main extraction |
| `_convert_entities_to_spans(text, entities, doc)` | Converts entity dicts to GliREL format `[start, end, label, text]` |
| `_convert_relations_to_triplets(relations, ner_spans, text)` | Converts GliREL output to Triplet objects |
| `extract_with_gliner(text, gliner_extractor)` | End-to-end extraction convenience method |

---

## 3. Unified Extractor — spaCy + GliNER + GliREL Combined

**File**: `src/ingestion/extractors/unified_extractor.py`

This is the core extraction class that orchestrates all three models together.

### Pipeline Flow

```
Text
  → spaCy tokenize (en_core_web_sm) → Token list
  → GliNER predict_entities → Entities with char positions
  → Convert char positions → Token positions (critical step for GliREL)
  → GliREL predict_relations → Relations with token positions
  → Filter by semantic constraints → Valid relations only
  → Deduplicate bidirectional → Final output
```

### Entity Labels Used

```python
ENTITY_LABELS = [
    "person", "organization", "location", "date", "money",
    "product", "technology", "role", "department", "project",
    "policy", "event", "company"
]
```

### Relation Labels Used

```python
RELATION_LABELS = [
    "founded", "leads", "CEO of", "works at", "member of",
    "reports to", "manages", "headquartered in", "located in",
    "based in", "partner of", "acquired", "subsidiary of",
    "uses", "built with", "developed by", "provides",
    "governs", "applies to", "owns", "created by"
]
```

### Semantic Type Constraints

These constraints validate that extracted relations make semantic sense:

```python
SEMANTIC_CONSTRAINTS = {
    # Leadership/Founding
    "founded":          ({"person"}, {"organization", "company", "startup"}),
    "leads":            ({"person"}, {"organization", "company", "department", "team"}),
    "works at":         ({"person"}, {"organization", "company"}),

    # Location
    "headquartered in": ({"organization", "company", "startup"}, {"location", "city", "country"}),
    "located in":       (None, {"location", "city", "country", "address"}),

    # Business
    "acquired":         ({"organization", "company", "investor"}, {"organization", "company", "startup"}),
    "uses":             ({"organization", "company", "product", "project"}, {"technology", "product", "framework"}),

    # E-commerce
    "purchased":        ({"person", "customer"}, {"product", "item"}),
    "ordered":          ({"person", "customer"}, {"product", "item"}),
    "shipped_to":       ({"product", "order"}, {"location", "address"}),
}
```

### Invalid Subjects (never relation actors)

```python
invalid_subjects = {"date", "money", "price", "quantity", "percentage"}
```

### Critical Implementation Detail: Char-to-Token Conversion

GliNER outputs **character positions** but GliREL requires **token positions**. The `_char_to_token(doc, char_start, char_end)` method handles this using the spaCy Doc object:

```python
def _char_to_token(self, doc, char_start, char_end):
    """Maps character positions to token positions using spaCy Doc."""
    token_start = None
    token_end = None
    for i, token in enumerate(doc):
        if token.idx <= char_start < token.idx + len(token):
            token_start = i
        if token.idx < char_end <= token.idx + len(token):
            token_end = i + 1
    return token_start, token_end
```

---

## 4. Pattern Matcher — 50+ Regex Patterns

**File**: `src/ingestion/extractors/patterns.py`

High-precision regex extraction (confidence 0.80–0.93) as a complement to ML models. Patterns run first and provide reliable baseline extraction.

### Relationship Patterns

#### Employment & Organizational

| Pattern | Example | Relation | Confidence |
|---------|---------|----------|------------|
| `X works at/for Y` | "John works at Google" | WORKS_AT | 0.90 |
| `X is the CEO/Manager/Director of Y` | "Jane is the CEO of Acme" | LEADS | 0.92 |
| `X, VP of Engineering at Y` | "Tom, VP of Engineering at Meta" | WORKS_AT | 0.92 |
| `X, CEO of Y` (appositive) | "Jane, CEO of Acme" | LEADS | 0.93 |
| `The CEO of Y, X` (inverted) | "The CEO of Acme, Jane" | LEADS | 0.90 |
| `X serves/acts as CEO of Y` | "Jane serves as CEO of Acme" | LEADS | 0.92 |
| `X founded/co-founded Y` | "John founded Acme" | FOUNDED | 0.92 |
| `founded/established by X` | "Acme, founded by John" | FOUNDED | 0.85 |
| `X — CEO` (em-dash) | "Jane — CEO" | HAS_ROLE | 0.88 |
| `led/managed by X` | "Team led by Jane" | LEADS | 0.85 |
| `X joined/appointed CEO of Y` | "Jane joined as CEO of Acme" | LEADS | 0.90 |
| `X reports to Y` | "John reports to Jane" | REPORTS_TO | 0.90 |
| `X manages/supervises Y` | "Jane manages John" | MANAGES | 0.88 |

#### Positions & Departments

| Pattern | Relation | Confidence |
|---------|----------|------------|
| `Customer Support Manager at CommerceFlow` | POSITION_AT | 0.88 |
| `X Department handles/manages Y` | HANDLES | 0.85 |
| `Y's X Department` | PART_OF | 0.88 |

#### Locations

| Pattern | Relation | Confidence |
|---------|----------|------------|
| `X based/located/headquartered in Y` | LOCATED_IN | 0.90 |
| `headquartered in X` | LOCATED_IN | 0.80 |
| `X has offices/operations in Y` | HAS_OFFICE_IN | 0.88 |
| `X's Y office/headquarters` | HAS_OFFICE_IN | 0.85 |

#### E-Commerce & Products

| Pattern | Relation | Confidence |
|---------|----------|------------|
| `X costs/priced at $Y` | HAS_PRICE | 0.92 |
| `X sold by Y` | SOLD_BY | 0.85 |
| `X belongs to category Y` | IN_CATEGORY | 0.85 |
| `X ordered/purchased/bought Y` | ORDERED | 0.88 |
| `X supplied by/shipped from Y` | SUPPLIED_BY | 0.87 |

#### Technical

| Pattern | Relation | Confidence |
|---------|----------|------------|
| `X uses/requires/depends on Y` | USES | 0.85 |
| `will use X for Y` | USED_FOR | 0.85 |
| `X version Y` | HAS_VERSION | 0.92 |

#### Financial

| Pattern | Relation | Confidence |
|---------|----------|------------|
| `X paid $Y` | PAID | 0.90 |
| `Invoice X for Y` | INVOICED_TO | 0.90 |

#### General Business

| Pattern | Relation | Confidence |
|---------|----------|------------|
| `X owns/acquired/bought out Y` | OWNS | 0.90 |
| `X partnered with Y` | PARTNERS_WITH | 0.88 |
| `X competes with Y` | COMPETES_WITH | 0.85 |

### Entity Patterns (7 types)

| Type | Pattern Examples | Confidence |
|------|-----------------|------------|
| Company suffixes | `X Inc`, `X Corp`, `X LLC`, `X Solutions`, `X Group` | 0.90–0.92 |
| Quoted product names | `"Slack Brain"` | 0.90 |
| Product patterns | `the X app/platform/service/tool/software` | 0.88 |
| Title-case sequences | Multi-word capitalized names | 0.75 |
| Technology terms | `via/using/with/through/powered by X` | 0.85 |
| Role patterns | `VP of Y`, `Lead Developer` | 0.85 |
| Department patterns | `X Department/Team/Division` | 0.88 |

### Known Technologies Dictionary

Case-insensitive matching for common tech:

```python
KNOWN_TECHNOLOGIES = {
    'stripe', 'slack', 'notion', 'figma', 'react', 'vue', 'angular', 'nextjs',
    'vercel', 'netlify', 'aws', 'azure', 'gcp', 'docker', 'kubernetes',
    'postgresql', 'mongodb', 'redis', 'elasticsearch', 'openai', 'anthropic',
    'gemini', 'chatgpt', 'claude', 'tailwind', 'bootstrap', 'typescript',
    'javascript', 'python', 'golang', 'rust', 'java', 'node', 'fastapi',
    'django', 'flask', 'express', 'graphql', 'rest', 'api', 'oauth',
    'shopify', 'woocommerce', 'nuxt', 'svelte', 'remix', 'astro'
}
```

### Garbage Filters

```python
PRONOUNS = {"our", "we", "us", "they", "my", "your", "his", "her", "i", "me", ...}
SECTION_HEADERS = {"overview", "summary", "conclusion", "features", "pricing", "security", ...}
GARBAGE_PATTERNS = [newlines, HTML tags, sentence fragments, gerunds, mission statements]
SINGLE_NOISE_WORDS = {"left", "right", "panel", "view", "button", "link", "input", "output", "step", "phase"}
```

---

## 5. Domain Router — Embedding-Based Industry Detection

**File**: `src/ingestion/extractors/domain_router.py`
**Embedding Model**: `intfloat/e5-small-v2` (384 dimensions)
**Threshold**: `0.3`
**Max Routing Characters**: `2000`

### Routing Process

1. **At init**: Embed all industry pack descriptions + examples (cached as 384-dim vectors)
2. **For incoming document**: Embed first 2000 chars with `"query: "` prefix (e5 model requirement)
3. **Compute cosine similarity** against all pack embeddings
4. **Select best pack** if score ≥ 0.3, otherwise fallback to `GENERAL_BUSINESS`

### How the Router Selects Labels

The selected industry pack determines which entity labels and relation labels are passed to GliNER and GliREL. This means extraction is domain-aware — a healthcare document gets medical entity types, while a tech doc gets technology types.

---

## 6. Industry Packs — Domain-Specific Label Sets

**File**: `src/ingestion/extractors/industry_packs.py`

11 packs, each with tailored entity and relation labels:

### GENERAL_BUSINESS (Fallback)

```python
id = "general_business"
entity_labels = ["person", "organization", "company", "location", "date",
                 "role", "department", "product", "money", "event"]
relation_labels = ["founded", "leads", "works at", "located in", "member of",
                   "reports to", "manages", "owns", "provides", "partner of"]
```

### TECH_STARTUP

```python
id = "tech_startup"
entity_labels = ["person", "company", "product", "technology", "framework",
                 "programming_language", "investor", "funding_round", "location",
                 "startup", "platform", "api", "feature"]
relation_labels = ["founded", "leads", "built with", "integrates with",
                   "invested in", "acquired", "uses", "developed by",
                   "competes with", "partners with", "headquartered in",
                   "launched", "maintains"]
```

### ECOMMERCE_RETAIL

```python
id = "ecommerce_retail"
entity_labels = ["customer", "product", "order", "category", "brand", "seller",
                 "store", "warehouse", "sku", "price", "discount",
                 "payment_method", "shipping_method", "address", "date",
                 "quantity", "review"]
relation_labels = ["purchased", "ordered", "reviewed", "added to cart",
                   "wishlisted", "returned", "shipped to", "sold by",
                   "belongs to category", "manufactured by", "priced at",
                   "discounted by", "paid with", "delivered to", "rated",
                   "recommended"]
```

### Other Packs

| Pack ID | Focus Areas |
|---------|-------------|
| `legal_corporate` | Contracts, statutes, regulations, attorneys, courts, obligations |
| `finance_investment` | Funds, securities, stocks, bonds, exchanges, valuations |
| `hr_people` | Skills, certifications, degrees, salaries, benefits, departments |
| `healthcare_medical` | Patients, medications, diagnoses, procedures, dosages |
| `real_estate` | Properties, brokers, tenants, landlords, zoning, valuations |
| `supply_chain` | Suppliers, manufacturers, warehouses, shipments, carriers, SKUs |
| `research_academic` | Researchers, journals, papers, grants, conferences, methodologies |
| `government_public` | Agencies, legislation, regulations, policies, jurisdictions, permits |

---

## 7. Structured Extractor — Hub-and-Spoke Model

**File**: `src/ingestion/extractors/structured_extractor.py`

For CSV/JSON data. Classifies columns into types, then builds a hub-and-spoke graph.

### Column Classification (7 types)

```python
class ColumnType(Enum):
    PRIMARY_ENTITY = "primary_entity"  # Main entity per row (hub node)
    PRIMARY_ID     = "primary_id"      # Row identifier (>90% unique, id pattern)
    FOREIGN_KEY    = "foreign_key"     # Reference to another entity (<90% unique, id pattern)
    TIMESTAMP      = "timestamp"       # Date/time columns
    ENTITY         = "entity"          # Categorical data (≤500 distinct, <60% unique)
    PROPERTY       = "property"        # Numeric/text/high-cardinality columns
    TECHNICAL      = "technical"       # User agents, hashes, tokens
```

### Classification Rules (priority order)

1. ID patterns (`_id$`, `uuid`, `pk`) → PRIMARY_ID or FOREIGN_KEY
2. Timestamp patterns (`date`, `_at$`, `created`, `month`) → TIMESTAMP
3. Technical patterns (`user_agent`, `hash`, `token`) → TECHNICAL
4. Unique value patterns (tracking/invoice/serial numbers) → PROPERTY
5. Primary entity patterns (name, customer, employee, order) → PRIMARY_ENTITY
6. Force entity patterns (product, category, brand, city, status) → ENTITY
7. Numeric columns → PROPERTY
8. Long strings (avg > 100 chars) → PROPERTY
9. High uniqueness (> 90%) → PROPERTY
10. High cardinality (> 500 distinct) → PROPERTY
11. Low cardinality (≤ 500 distinct) → ENTITY
12. Default → PROPERTY

### Primary Entity Patterns

```python
PRIMARY_ENTITY_PATTERNS = re.compile(
    r'^(name|customer|client|employee|person|user|company|organization|'
    r'order|account|contact|lead|vendor|supplier|patient|student|'
    r'contractor|freelancer|consultant|worker|staff|member|associate|'
    r'sales_?rep|salesperson|representative|agent|broker|technician|'
    r'driver|engineer|manager|partner|merchant|retailer|distributor|'
    r'manufacturer|provider)s?$|'
    r'_(name|customer|client|employee|order|contractor|rep|agent|person)$|'
    r'.*_(rep|representative|person|name)$'
)
```

### Force Entity Patterns

```python
FORCE_ENTITY_PATTERNS = re.compile(
    r'^(product|item|sku|service|category|brand|model|'
    r'city|country|region|state|location|address|'
    r'department|team|role|status|type|tier|level|'
    r'channel|source|campaign|segment|'
    r'project|task|assignment|job|contract|territory|area|zone|'
    r'skill|specialty|certification|qualification)s?$|'
    r'_(product|item|sku|category|brand|city|country|region|status|type|project|task)s?$'
)
```

### Hub-and-Spoke Model

```
Primary Entity (Hub)
    ├── ENTITY column → Spoke relationship
    ├── ENTITY column → Spoke relationship
    ├── FOREIGN_KEY → Spoke relationship
    └── Properties become node metadata

Example: customer_name (hub) connects to:
    → region via LOCATED_IN
    → product via HAS_ITEM
    → status via HAS_STATUS
```

### Relationship Derivation from Column Names

```python
mappings = {
    'location': 'LOCATED_IN',    'city': 'LOCATED_IN',
    'country': 'LOCATED_IN',     'region': 'IN_REGION',
    'category': 'IN_CATEGORY',   'status': 'HAS_STATUS',
    'type': 'IS_TYPE',           'department': 'IN_DEPARTMENT',
    'team': 'ON_TEAM',           'manager': 'MANAGED_BY',
    'supervisor': 'SUPERVISED_BY', 'industry': 'IN_INDUSTRY',
    'sector': 'IN_SECTOR',       'salesperson': 'ASSIGNED_TO',
    'sales_rep': 'ASSIGNED_TO',  'engineer': 'SUPPORTED_BY',
    'owner': 'OWNED_BY',
    # Default: HAS_<COLUMN_NAME>
}
```

### Cardinality Thresholds

```python
entity_threshold = 0.6       # Max unique ratio to be classified as entity
min_rows_for_stats = 5       # Minimum rows for reliable statistics
max_entity_cardinality = 500  # Max distinct values for entity classification
```

---

## 8. Triplet Data Structure

**File**: `src/ingestion/extractors/triplet.py`

```python
@dataclass
class Triplet:
    subject: str                                    # Entity text
    predicate: str                                  # Relationship type
    object: str                                     # Entity text
    confidence: float = 1.0                         # 0.0 - 1.0
    source_span: Optional[tuple[int, int]] = None   # Character offsets
    subject_label: Optional[str] = None             # Entity type (person, org, etc.)
    object_label: Optional[str] = None              # Entity type
    metadata: dict[str, Any] = field(default_factory=dict)

    @property
    def is_high_confidence(self) -> bool:
        return self.confidence >= 0.8

    def matches_subject(query, fuzzy=True) -> bool
    def matches_object(query, fuzzy=True) -> bool
    def involves(entity, fuzzy=True) -> bool


@dataclass
class TripletCollection:
    triplets: list[Triplet]

    def add(triplet) -> None
    def add_all(triplets) -> int
    def find_by_subject(subject, fuzzy=True) -> list[Triplet]
    def find_by_object(obj, fuzzy=True) -> list[Triplet]
    def find_by_predicate(predicate) -> list[Triplet]
    def involving(entity, fuzzy=True) -> list[Triplet]
    def high_confidence(threshold=0.8) -> list[Triplet]
    def to_list() -> list[dict]
```

---

## 9. Graph Storage — Apache AGE (PostgreSQL)

### Database Setup

```sql
CREATE EXTENSION IF NOT EXISTS vector;   -- pgvector for embeddings
CREATE EXTENSION IF NOT EXISTS pg_trgm;  -- trigram for fuzzy text search
CREATE EXTENSION IF NOT EXISTS age;      -- Apache AGE for graph

-- Initialize graph
LOAD 'age';
SET search_path = ag_catalog, '$user', public;
SELECT create_graph('context_mesh');
```

### Node Schema

```python
{
    "id": "node_<hash>",       # hash(name.lower().strip()) % 10000000
    "label": "Person|Company|Location|Product|Technology|Department|Role|...",
    "name": "entity text",
    "properties": {
        "confidence": 0.85,
        "entity_type": "person",
        "doc_id": 123,
        "description": "derived from context",
        # Plus structured data properties:
        "email": "user@example.com",
        "website": "https://example.com",
        "phone": "555-1234",
    }
}
```

### Edge Schema

```python
{
    "source_label": "Person",
    "source_name": "John Smith",
    "target_label": "Company",
    "target_name": "Acme Corp",
    "relation": "WORKS_AT",
    "properties": {
        "confidence": "0.85",
        "original_predicate": "works at"
    }
}
```

### Cypher Patterns Used

```cypher
-- Node creation (MERGE = create-if-not-exists)
MERGE (n:Person {name: "John Smith"})
SET n.doc_id = 123
SET n.description = "Software Engineer"
SET n.email = "john@acme.com"
RETURN n

-- Edge creation
MATCH (a:Person {name: "John Smith"}), (b:Company {name: "Acme Corp"})
MERGE (a)-[r:WORKS_AT {confidence: "0.85", original_predicate: "works at"}]->(b)
RETURN r
```

### Label Map (entity_type → graph_label)

```python
LABEL_MAP = {
    # People
    "person": "Person", "customer": "Person", "client": "Person",
    "employee": "Person", "researcher": "Person", "patient": "Person",
    "physician": "Person", "attorney": "Person",

    # Organizations
    "organization": "Company", "company": "Company", "startup": "Company",
    "brand": "Company", "supplier": "Company", "manufacturer": "Company",
    "bank": "Company", "fund": "Company", "hospital": "Company",
    "university": "Company", "institution": "Company", "agency": "Company",

    # Locations
    "location": "Location", "city": "Location", "country": "Location",
    "address": "Location", "warehouse": "Location", "port": "Location",

    # Products/Tech
    "product": "Product", "sku": "Product", "platform": "Product",
    "software": "Technology", "technology": "Technology",
    "framework": "Technology", "programming_language": "Technology",
    "api": "Technology",

    # Organizational
    "department": "Department", "team": "Department",
    "role": "Role", "title": "Role",
    "project": "Project",

    # Events/Time
    "event": "Event", "meeting": "Event", "date": "Date",
    "funding_round": "Event",

    # Financial
    "money": "Money", "price": "Money",
    "payment_method": "PaymentMethod",

    # Domain-Specific
    "category": "Category", "task": "Task", "review": "Review",
    "order": "Order", "shipment": "Shipment",
    "condition": "Condition", "diagnosis": "Condition",
    "medication": "Medication", "policy": "Policy",
    "contract": "Contract",
}
```

### Safety Mechanisms

| Mechanism | Purpose |
|-----------|---------|
| **Advisory Locks** | `pg_advisory_xact_lock(hashtext(key))` — serializes node/edge creation to prevent race conditions |
| **Label Sanitization** | Removes non-alphanumeric chars, prefixes reserved words (`count`, `return`, `match`) with underscore |
| **Value Escaping** | Backslashes (`\` → `\\`) and quotes (`"` → `\"`) escaped for Cypher strings |
| **Transaction Handling** | `BEGIN/COMMIT/ROLLBACK` per node/edge operation |

### Relation Cleaning

```python
def _clean_relation(predicate) -> str:
    """
    1. Remove extra whitespace
    2. Convert to UPPERCASE_WITH_UNDERSCORES
    3. Remove non-alphanumeric except underscore
    4. Default to RELATED_TO if empty
    5. Limit to 50 chars
    """
    # "CEO of" → "CEO_OF"
    # "works at" → "WORKS_AT"
    # "headquartered in" → "HEADQUARTERED_IN"
```

---

## 10. Processing Pipeline — Full Ingestion Flow

**File**: `src/ingestion/processor.py`

### Entry Point: `_extract_and_save_graph(doc_id, content)`

1. **Detect Content Type**
   - Try to parse as JSON array
   - If valid list of dicts → **Structured path**
   - Otherwise → **Unstructured path**

2. **Structured Path** (`_process_structured_graph`)
   - Instantiate `StructuredExtractor`
   - Call `extract_triplets(data)`
   - Convert to nodes/edges
   - Save via `_save_graph_to_age()`

3. **Unstructured Path** (`_process_unstructured_graph`)
   - Split text into **1500-char batches** (GliREL has 512-token limit)
   - Split on sentence boundaries when possible
   - Process **5 batches concurrently** (asyncio semaphore)
   - Each batch runs full pipeline: Pattern Match → Domain Route → Unified Extract → Triplets → Graph

### Per-Batch Flow: `_process_text_graph_batch(doc_id, text_segment)`

1. Ensure `RoutedExtractor` loaded (lazy init)
2. Call `extractor.extract()` → `RoutedExtractionResult`
3. Get routing info: pack name, confidence, entity/relation results
4. Convert `result.triplets` to `Triplet` objects
5. Resolve context placeholders: `"[organization from context]"` → most common org from entities
6. Convert triplets to nodes/edges via `_triplets_to_graph()`
7. Save via `_save_graph_to_age()`

### RoutedExtractor Flow

```python
def extract(text, title=None, entity_threshold=0.5, relation_threshold=0.5, force_pack=None):
    # Step 1: Route to industry pack
    if force_pack:
        pack = router.get_pack_by_id(force_pack)
    else:
        route_result = router.route(text, title)  # DomainRouter
        pack = route_result.pack

    # Step 2: Configure extractor with pack-specific labels
    extractor.entity_labels = pack.entity_labels
    extractor.relation_labels = pack.relation_labels

    # Step 3: Extract entities + relations
    entities, relations = extractor.extract(text, entity_threshold, relation_threshold)

    # Step 4: Convert to triplets
    triplets = _relations_to_triplets(relations)

    return RoutedExtractionResult(entities, relations, triplets, route_result, pack.id, pack.name)
```

### Triplet-to-Graph Conversion: `_triplets_to_graph(triplets, doc_id)`

1. Filter out invalid entity text (fragments, stop words, unbalanced punctuation)
2. For each triplet:
   - Get graph labels via `_get_graph_label()` using LABEL_MAP
   - Create/deduplicate subject node (by `name.lower().strip()`)
   - Merge `subject_properties` (from structured data: email, phone, etc.)
   - Create/deduplicate object node
   - Create edge with cleaned relation
3. Return `(nodes_list, edges_list)`

### Deduplication Strategy

| Level | Method |
|-------|--------|
| **Entity names** | `name.lower().strip()` — same text = same node |
| **Node IDs** | `hash(name_key) % 10000000` |
| **Bidirectional relations** | Unified extractor removes duplicates in opposite directions |
| **Database level** | `MERGE` in Cypher prevents duplicate nodes/edges |

### Context Resolution

Placeholders like `"[organization from context]"` are resolved to the most common organization entity found in the current extraction batch.

---

## 11. Extraction Methods Summary

| Source | Method | Model | Precision | Recall | Notes |
|--------|--------|-------|-----------|--------|-------|
| Unstructured Text | Pattern Matcher | Regex (50+) | Very High (0.80–0.93) | Low-Medium | Runs first, catches obvious patterns |
| Unstructured Text | GliNER | `urchade/gliner_medium-v2.1` | Medium | Medium-High | Batched (4 × 10 labels) |
| Unstructured Text | GliREL | `jackboyla/glirel-large-v0` | Medium | Medium | Semantic constraints filter noise |
| Unstructured Text | Unified (spaCy + GliNER + GliREL) | All three | Medium-High | High | Char-to-token conversion critical |
| Structured Data | StructuredExtractor | Heuristics | Very High (0.85–0.95) | High | Schema-aware, hub-and-spoke |

---

## 12. Key Configuration Constants

| Parameter | Value | File |
|-----------|-------|------|
| GliNER model | `urchade/gliner_medium-v2.1` | `gliner_extractor.py` |
| GliREL model | `jackboyla/glirel-large-v0` | `glirel_extractor.py` |
| spaCy model | `en_core_web_sm` | `unified_extractor.py` |
| Embedding model (router) | `intfloat/e5-small-v2` (384d) | `domain_router.py` |
| Entity threshold | `0.4–0.5` | `processor.py` / extractors |
| Relation threshold | `0.4–0.5` | `processor.py` / extractors |
| Router threshold | `0.3` | `domain_router.py` |
| Max labels per GliNER call | `20` | `gliner_extractor.py` |
| Max entity text length | `60 chars` | `gliner_extractor.py` |
| Text batch size | `1500 chars` | `processor.py` |
| Concurrent batches | `5` | `processor.py` |
| Max entity cardinality (structured) | `500` | `structured_extractor.py` |
| Entity uniqueness threshold | `0.6` | `structured_extractor.py` |
| Graph name | `context_mesh` | `init_db.py` |
| Database | PostgreSQL + Apache AGE | `settings.py` |

---

## 13. Dependencies

```
# ML Models
gliner                    # Entity extraction
glirel                    # Relation extraction
spacy                     # Tokenization
en_core_web_sm            # spaCy English model (python -m spacy download en_core_web_sm)
sentence-transformers     # For embedding model (e5-small-v2)

# Database
psycopg[binary]           # PostgreSQL async driver
# Apache AGE extension must be installed on PostgreSQL
```

---

## 14. What to Replicate for Orunla & CGC

### Minimum Viable Extraction (replace simple pattern matching)

1. **Add Unified Extractor** — spaCy + GliNER + GliREL pipeline with char-to-token conversion
2. **Add Pattern Matcher** — 50+ regex patterns for high-precision extraction
3. **Add Semantic Type Constraints** — prevents nonsensical relations

### Full Pipeline (recommended)

1. Everything above, plus:
2. **Domain Router** — embedding-based industry detection with e5-small-v2
3. **Industry Packs** — domain-specific label sets (pick relevant packs)
4. **Structured Extractor** — if ingesting tabular data

### Key Learnings / Gotchas

- GliREL has a **512-token limit** — split long text into 1500-char batches
- GliNER has a **max 20 labels per call** — batch labels into groups
- **Char-to-token conversion** is the most fragile part — test thoroughly
- **Garbage filtering** is essential — without it, ~30% of entities are noise
- **Semantic constraints** dramatically reduce false positive relations
- **Advisory locks** in PostgreSQL prevent race conditions during concurrent graph writes
- **MERGE** in Cypher is idempotent — safe for repeated extraction of same document
