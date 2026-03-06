# Connecting Orunla to Your AI

Give any AI chatbot persistent memory. Set up an MCP connection or use the REST API.

---

## Claude Code / Cursor / Cline / Windsurf

These tools use MCP via **stdio** — a direct local connection. See `MCP.md` for setup instructions.

---

## Claude Browser (claude.ai)

Use the MCP SSE transport to connect Claude browser to Orunla:

1. Start the MCP server in SSE mode:
   ```bash
   orunla_mcp --transport sse --port 8080
   ```
2. Expose via tunnel (e.g., Cloudflare Tunnel):
   ```bash
   cloudflared tunnel --url http://localhost:8080
   ```
3. In Claude browser, go to **Settings > Integrations > Add**
4. Paste the tunnel URL with `/sse` appended

---

## ChatGPT (Custom GPT)

### 1. Start the REST API

```bash
orunla_cli serve --port 8080
```

Or use the desktop app (serves on port 8080 automatically).

### 2. Create a Custom GPT

1. Go to [chat.openai.com](https://chat.openai.com)
2. Click your profile icon > **My GPTs** > **Create a GPT**
3. Click the **Configure** tab

### 3. Add Instructions

In the **Instructions** field, paste:

```
You have access to Orunla, a persistent memory system. Use it proactively to remember and recall information across conversations.

## When to SAVE memories (use the "ingest" action):
- When the user says "remember", "don't forget", "for future reference"
- When the user shares important facts: names, preferences, decisions, project details
- When you learn something useful for future conversations

## When to RECALL memories (use the "recall" action):
- At the START of every conversation
- When the user asks "what do you know about..." or "do you remember..."
- When past context would help you give a better answer

## When to CLEAN UP memories:
- When the user says "forget about..." or "delete memories about..."
  -> Use the "purge" action
```

### 4. Add Actions (OpenAPI Spec)

1. Scroll down to **Actions** > click **Create new action**
2. In the **Schema** field, paste the spec below
3. **Replace the server URL** with your Orunla URL (localhost or tunnel)

```yaml
openapi: 3.0.0
info:
  title: Orunla Memory API
  description: Persistent AI memory system
  version: 0.5.0
servers:
  - url: http://localhost:8080
    description: Orunla local server
paths:
  /health:
    get:
      operationId: healthCheck
      summary: Check if Orunla is running
      responses:
        '200':
          description: Server is healthy
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
              required: [text]
              properties:
                text:
                  type: string
                  description: The text to extract facts from and save to memory
      responses:
        '200':
          description: Memory saved
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
              required: [query]
              properties:
                query:
                  type: string
                limit:
                  type: integer
      responses:
        '200':
          description: Matching memories
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
              required: [query]
              properties:
                query:
                  type: string
      responses:
        '200':
          description: Memories purged
```

### 5. Set Authentication

If you set an API key:
1. Under the schema, click **Authentication**
2. Choose **API Key**
3. Auth Type: **Bearer**
4. Paste your API key

---

## n8n / Make.com / Other Automations

1. Start Orunla: `orunla_cli serve --port 8080` or open the desktop app
2. In your workflow, add an **HTTP Request** node
3. Use these endpoints:

| Action | Method | URL | Body |
|--------|--------|-----|------|
| Save a memory | `POST` | `http://localhost:8080/ingest` | `{"text": "fact to save"}` |
| Search memories | `POST` | `http://localhost:8080/recall` | `{"query": "search term"}` |
| Delete by topic | `POST` | `http://localhost:8080/memories/purge` | `{"query": "topic"}` |

4. Set header `Content-Type: application/json`
5. If using API key, add header `X-API-Key: your-key`

For external access, expose via tunnel (see `DEVELOPER.md`).

---

## System Prompts

### CLAUDE.md Template (for Claude Code / Cursor with MCP)

Add this to `~/.claude/CLAUDE.md`:

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

### Quality
- Be specific: "ProjectX uses PostgreSQL 16 on port 5432" not "uses a database"
- Use project name as subject for consistent retrieval

### Cleanup
- `memory_purge_topic` when user says "forget about..."
- `memory_gc` periodically (threshold 0.05)
```

### Short Prompt (for any AI)

```
You have access to Orunla memory.

Save: POST /ingest {"text": "fact"}
Recall: POST /recall {"query": "term", "limit": 10}
Delete: POST /memories/purge {"query": "topic"}

ALWAYS recall at conversation start. Save important facts proactively. Tell the user when you save or recall.
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
