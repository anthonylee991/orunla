# Connecting Orunla to Your AI

Give any AI chatbot persistent memory. Open the desktop app, copy a URL, paste it into your AI tool. That's it.

**Requirements:** The Orunla desktop app must be open for these connections to work.

---

## ChatGPT (Custom GPT)

### 1. Get your relay URL

1. Open the Orunla desktop app
2. Find the **"Remote Access"** card
3. Copy the **REST API** relay URL

### 2. Create a Custom GPT

1. Go to [chat.openai.com](https://chat.openai.com)
2. Click your profile icon → **My GPTs** → **Create a GPT**
3. Click the **Configure** tab

### 3. Add Instructions

In the **Instructions** field, paste:

```
You have access to Orunla, a persistent memory system. Use it proactively to remember and recall information across conversations.

## When to SAVE memories (use the "ingest" action):
- When the user says "remember", "don't forget", "for future reference", "keep track of", or "note that"
- When the user shares important facts: names, preferences, decisions, project details, deadlines
- When you learn something that would be useful in future conversations
- After the user makes a decision or states a preference

When saving, write clear, factual sentences. Example:
  Good: "Sarah prefers dark mode in all applications."
  Bad: "The user said something about dark mode."

## When to RECALL memories (use the "recall" action):
- At the START of every conversation — search for context about what the user is working on
- When the user asks "what do you know about...", "do you remember...", or "what did I say about..."
- When you need context to give a better answer
- When the user references something from a previous conversation

## When to CLEAN UP memories:
- When the user says "forget about...", "delete memories about...", or "clear notes on..."
  → Use the "purge" action with the relevant topic

## General behavior:
- Always search memory before answering questions that might have relevant context
- Save important information immediately — don't wait to be asked
- Be specific when saving: include names, numbers, dates, and concrete details
- Tell the user when you've saved or recalled something: "I've saved that to memory" or "Based on my memory..."
```

### 4. Add Actions (OpenAPI Spec)

1. Scroll down to **Actions** → click **Create new action**
2. In the **Schema** field, paste the spec below
3. **Replace `YOUR-DEVICE-ID`** with the device ID from your relay URL

```yaml
openapi: 3.0.0
info:
  title: Orunla Memory API
  description: Persistent AI memory system
  version: 0.4.1
servers:
  - url: https://orunla-production.up.railway.app/api/YOUR-DEVICE-ID
    description: Orunla cloud relay
paths:
  /health:
    get:
      operationId: healthCheck
      summary: Check if Orunla is running
      responses:
        '200':
          description: Server is healthy
          content:
            application/json:
              schema:
                type: object
                properties:
                  status:
                    type: string
                  version:
                    type: string
  /ingest:
    post:
      operationId: ingestMemory
      summary: Save a new memory
      requestBody:
        required: true
        content:
          application/json:
            schema:
              type: object
              required:
                - text
              properties:
                text:
                  type: string
                  description: The text to extract facts from and save to memory
      responses:
        '200':
          description: Memory saved
          content:
            application/json:
              schema:
                type: object
                properties:
                  status:
                    type: string
                  added_triplets:
                    type: integer
  /recall:
    post:
      operationId: recallMemories
      summary: Search for memories
      requestBody:
        required: true
        content:
          application/json:
            schema:
              type: object
              required:
                - query
              properties:
                query:
                  type: string
                  description: Search query to find relevant memories
                limit:
                  type: integer
                  description: Maximum number of results (default 10)
      responses:
        '200':
          description: Matching memories
          content:
            application/json:
              schema:
                type: object
                properties:
                  memories:
                    type: array
                    items:
                      type: object
                      properties:
                        subject:
                          type: string
                        predicate:
                          type: string
                        object:
                          type: string
                        text:
                          type: string
                        strength:
                          type: number
  /memories/purge:
    post:
      operationId: purgeMemories
      summary: Delete all memories matching a topic
      requestBody:
        required: true
        content:
          application/json:
            schema:
              type: object
              required:
                - query
              properties:
                query:
                  type: string
                  description: Topic or keyword to purge from memory
      responses:
        '200':
          description: Memories purged
          content:
            application/json:
              schema:
                type: object
                properties:
                  status:
                    type: string
                  deleted_count:
                    type: integer
  /gc:
    post:
      operationId: garbageCollect
      summary: Clean up old decayed memories
      requestBody:
        content:
          application/json:
            schema:
              type: object
              properties:
                threshold:
                  type: number
                  description: Strength threshold (default 0.05)
      responses:
        '200':
          description: Cleanup complete
          content:
            application/json:
              schema:
                type: object
                properties:
                  status:
                    type: string
                  deleted_memories:
                    type: integer
```

### 5. Set Authentication

If you set an API key in the desktop app:

1. Under the schema, click **Authentication**
2. Choose **API Key**
3. Auth Type: **Bearer**
4. Paste your API key

### 6. Test It

Save the GPT and start a conversation:

- *"What do you remember about me?"*
- *"Remember that my favorite programming language is Rust"*
- *"Forget everything about my old project"*

---

## Claude Browser (claude.ai)

1. Open the Orunla desktop app
2. Find the **"Remote Access"** card
3. Copy the **MCP SSE** relay URL
4. Go to [claude.ai](https://claude.ai)
5. Open **Settings** → **Integrations** (or **MCP Connectors**)
6. Click **Add**
7. Paste the relay URL
8. It should show as **Connected**

**Test it** — start a new conversation and try:
- *"What memory tools do you have available?"*
- *"Search your memory for Orunla"*
- *"Remember that my favorite language is Rust"*

---

## Claude Code / Cursor / Cline / Windsurf

These tools use MCP via **stdio** — a direct connection that doesn't need the desktop app or the relay. See `MCP.md` for setup instructions.

---

## Google Gemini (Gems)

Gemini Gems can't call external APIs directly, but you can teach Gemini about your memory system:

1. Go to [gemini.google.com](https://gemini.google.com)
2. Click **Gem manager** → **New Gem**
3. Paste the system prompt from the "System Prompts" section below
4. Name it "Memory Assistant" or "Orunla"

When chatting with this Gem, it will remind you to save and recall memories. Use the desktop app to actually save/recall.

---

## n8n / Make.com / Other Automations

1. Open the Orunla desktop app
2. Find the **"Remote Access"** card
3. Copy the **REST API** relay URL
4. In your workflow, add an **HTTP Request** node
5. Use these endpoints:

| Action | Method | URL | Body |
|--------|--------|-----|------|
| Save a memory | `POST` | `{relay-url}/ingest` | `{"text": "fact to save"}` |
| Search memories | `POST` | `{relay-url}/recall` | `{"query": "search term"}` |
| Delete by topic | `POST` | `{relay-url}/memories/purge` | `{"query": "topic"}` |
| Cleanup | `POST` | `{relay-url}/gc` | `{"threshold": 0.05}` |

6. Set header `Content-Type: application/json`
7. If you set an API key in the desktop app, add header `X-API-Key: your-key`

---

## Protecting Your Memory (API Key)

By default, anyone with your relay URL can access your memory. To add protection:

1. Open the desktop app
2. Go to the **API Key** settings panel
3. Enter a key (any password you choose) and click **Save**
4. Restart the app

Now all REST API requests (including through the relay) require the key:

```
X-API-Key: your-key
```

or:

```
Authorization: Bearer your-key
```

---

## System Prompts

Copy-paste these into your AI tool's instructions to teach it how to use Orunla.

### Full Prompt

```
You have access to Orunla, a persistent memory system that stores facts as a knowledge graph.

SAVE a memory:
  POST /ingest  {"text": "The fact to remember"}

RECALL memories:
  POST /recall  {"query": "search term", "limit": 10}

DELETE memories by topic:
  POST /memories/purge  {"query": "topic to forget"}

When to SAVE (be proactive):
- User says "remember", "don't forget", "for future reference", "note that"
- User shares important facts: names, preferences, decisions, deadlines
- User makes a decision or states a preference
- You learn something useful for future conversations

When to RECALL (be proactive):
- At the START of every conversation
- When the user asks "what do you know about..." or "do you remember..."
- When past context would help you give a better answer

When to DELETE:
- User says "forget about..." or "delete memories about..."

Always tell the user when you save or recall something.
Write clear, specific facts: "Sarah prefers dark mode" not "something about preferences."
```

### Short Prompt

```
You have access to Orunla memory.

Save: POST /ingest {"text": "fact"}
Recall: POST /recall {"query": "term", "limit": 10}
Delete: POST /memories/purge {"query": "topic"}

ALWAYS recall at conversation start. Save important facts proactively. Tell the user when you save or recall.
```

### CLAUDE.md Template (for Claude Code / Cursor with MCP)

Add this to `~/.claude/CLAUDE.md` (macOS) or `%USERPROFILE%\.claude\CLAUDE.md` (Windows):

```markdown
## Memory (Orunla)

### Session Start (MANDATORY)
1. `memory_search` with project name or key terms
2. Greet user with context: "I recall that [project] uses [tech]..."

### When to Save
Use `memory_add` with subject, predicate, object:
- Project names and tech stack
- Architecture decisions
- User preferences
- Bug fixes and workarounds
- Anything the user says to "remember"

### Quality
- Be specific: "ProjectX uses PostgreSQL 16 on port 5432" not "uses a database"
- Use project name as subject for consistent retrieval

### Cleanup
- `memory_purge_topic` when user says "forget about..."
- `memory_gc` periodically (threshold 0.05)
```

---

## Tips

### Write Good Memories

| Quality | Example | Result |
|---------|---------|--------|
| **Good** | "Sarah manages the marketing budget and reports to David." | Clear entities and relationships extracted |
| **Bad** | "She does the money stuff for that team." | Too vague — nothing useful extracted |

### Memory Decay

Memories you don't access gradually fade. This keeps things clean. Important facts stay strong as long as you recall them.

### File Ingestion

File upload (`/ingest-file`) is only available through the desktop app's local interface, not through the relay. Use the desktop app UI to upload files directly.

---

*For developers who want CLI access, localhost APIs, manual server setup, or tunnel configuration, see `DEVELOPER.md`.*
