# Orunla: The Intelligent, Local-First Memory System

**Orunla** is a "Secondary Brain" for your AI agents and your personal data. Unlike standard AI memory that fades or hallucinations, Orunla stores facts as a **Knowledge Graph**—structured connections between concepts—and keeps them 100% local on your machine.

---

## 🚀 Getting Started (Zero Coding Required)

### 1. Download & Install
1.  Go to the [Releases](https://github.com/yourusername/orunla/releases) page.
2.  Download the **`Tauri Apporunla_0.1.0_x64-setup.exe`** installer.
3.  Run the installer and click "Install".
4.  Launch **Orunla Memory** from your Start Menu.

### 2. Add Your First Memory
1.  Open the Orunla Desktop App.
2.  In the "Ingest" box, type: *"The office printer code is 9988."*
3.  Click **Add**. Orunla's AI will automatically extract that Fact.

### 3. Recall Information
1.  Go to the "Recall" tab.
2.  Type *"printer"* and press enter.
3.  Orunla will show you the exact fact linking the printer to the code.

---

## 🌟 Key Features

| Feature | Description | Why it matters |
|---------|-------------|----------------|
| ** GliNER extraction** | Smart AI that understands "who did what". | Extremely accurate fact gathering. |
| **Forgetting Curve** | Memories "fade" over time if not used. | Keeps your brain focused on what's relevant now. |
| **Garbage Collection** | Automatically deletes old, useless facts. | Saves disk space and keeps the system fast. |
| **Node Merging** | Combines "Rust", "rust", and "RUST" into one. | Keeps your knowledge graph clean and organized. |

---

## 📈 Use Cases

### 🛠️ Personal Assistant
- **Input**: "My niece, Sarah, is allergic to peanuts."
- **Months Later**: "What should I know about Sarah's birthday party?"
- **Orunla Recall**: "Sarah is allergic to peanuts."

### 🏢 Customer Support
- **Input**: "The return policy for electronics is 14 days."
- **Recall**: (Searched by a support agent) "How long for electronics returns?"
- **Result**: "Electronics -> return policy -> 14 days."

---

## 🔌 No-Code Integration (Zapier / Make.com)

You can connect Orunla to tools like **Gmail**, **Slack**, or **Typeform** without writing a single line of code by using the **Webhooks** or **HTTP Request** modules.

### Example: Auto-save important emails
1.  **Trigger**: New starred email in Gmail.
2.  **Action**: HTTP Request (POST) to `http://your-local-ip:3000/ingest`.
3.  **Body**: `{"text": "{{email_body}}"}`
4.  **Result**: Every starred email is automatically turned into Facts in your Orunla brain.

### 🔒 Security for External Access

**IMPORTANT:** If you expose your server to the internet (via ngrok, tunneling services, or cloud deployment), ALWAYS use API key authentication:

```bash
# Generate a strong random API key (example)
# On Windows PowerShell:
# -Join ((48..57) + (65..90) + (97..122) | Get-Random -Count 32 | % {[char]$_})

# Start server with API key protection
orunla_cli.exe serve --port 3000 --api-key "your-secret-key-here"
```

Then add the API key to your webhook requests:
- **Header:** `X-API-Key: your-secret-key-here`
- **Or:** `Authorization: Bearer your-secret-key-here`

The server includes automatic rate limiting (60 requests/minute per IP) to prevent abuse.

---

## 🛠️ Advanced: API, CLI & MCP
For developers and power users looking to build on top of or automate Orunla:

- 📑 **[API Reference](API_REFERENCE.md)**: Full list of REST endpoints for webhooks/no-code.
- 💻 **[CLI Guide](docs/CLI.md)**: Maintenance commands (GC, Dedup, etc).
- 🤖 **[MCP Guide](docs/MCP.md)**: Connecting Orunla to AI Agents (Claude Desktop).

---

## 🛡️ Privacy & Security
Orunla is **local-first**. Your memories are stored in a SQLite database on your own machine. We do not use cloud storage, and your data is never used to train global AI models.

---

## 💾 Database Management

### 📍 Where is my data?
All your memories are stored in a single file on your computer:
- **Windows**: `%USERPROFILE%\.orunla\memory.db`
- **Linux/Mac**: `~/.orunla/memory.db`

### 🔍 How to view your data
If you want to manually browse your knowledge graph:
1.  Download a free tool like **[DB Browser for SQLite](https://sqlitebrowser.org/)**.
2.  Open the `memory.db` file listed above.
3.  Browse the `nodes` table (concepts) and `edges` table (connections).

### 🔄 How to Reset Orunla
If you want to wipe your brain and start fresh:
1.  Close the Orunla Desktop App and CLI.
2.  **Delete** the `memory.db` file.
3.  The next time you start Orunla, it will create a brand new, empty database.
