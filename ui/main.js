const { invoke } = window.__TAURI__.core;
const { open } = window.__TAURI__.dialog;

const ingestInput = document.getElementById('ingest-input');
const ingestBtn = document.getElementById('ingest-btn');
const fileInput = document.getElementById('file-input');
const fileBtn = document.getElementById('file-btn');
const fileName = document.getElementById('file-name');
const recallInput = document.getElementById('recall-input');
const recallBtn = document.getElementById('recall-btn');
const purgeInput = document.getElementById('purge-input');
const purgeBtn = document.getElementById('purge-btn');
const nodesCount = document.getElementById('nodes-count');
const edgesCount = document.getElementById('edges-count');
const memoryContainer = document.getElementById('memory-container');
const statusMsg = document.getElementById('status-msg');

async function updateStats() {
    try {
        const stats = await invoke('get_stats');
        nodesCount.textContent = stats.node_count;
        edgesCount.textContent = stats.edge_count;
    } catch (e) {
        console.error("Failed to fetch stats:", e);
    }
}

function showStatus(msg) {
    statusMsg.textContent = msg;
    statusMsg.classList.add('show');
    setTimeout(() => {
        statusMsg.classList.remove('show');
    }, 3000);
}

ingestBtn.addEventListener('click', async () => {
    const text = ingestInput.value.trim();
    if (!text) return;

    ingestBtn.disabled = true;
    ingestBtn.textContent = 'Processing...';

    try {
        const res = await invoke('ingest', { text });
        showStatus(`Added ${res.added_triplets} triplets`);
        ingestInput.value = '';
        await updateStats();
    } catch (e) {
        console.error("Ingest failed:", e);
        showStatus("Error during ingestion");
    } finally {
        ingestBtn.disabled = false;
        ingestBtn.textContent = 'Ingest';
    }
});

fileBtn.addEventListener('click', async () => {
    try {
        const selected = await open({
            multiple: false,
            directory: false,
            filters: [{ name: 'Documents', extensions: ['txt', 'md', 'json', 'csv'] }]
        });

        if (selected) {
            fileBtn.disabled = true;
            fileBtn.textContent = 'Processing...';
            fileName.textContent = selected.split(/[/\\]/).pop();

            try {
                const res = await invoke('ingest_file', { filePath: selected });
                showStatus(`Processed ${res.chunks_processed} chunks, added ${res.total_triplets_added} triplets`);
                await updateStats();
            } catch (e) {
                console.error("File ingest failed:", e);
                showStatus("Error processing file");
            } finally {
                fileBtn.disabled = false;
                fileBtn.textContent = 'Upload File';
            }
        }
    } catch (e) {
        console.error("File picker failed:", e);
    }
});

recallBtn.addEventListener('click', async () => {
    const query = recallInput.value.trim();
    if (!query) return;

    recallBtn.disabled = true;
    try {
        const res = await invoke('recall', { query });
        renderMemories(res.memories);
    } catch (e) {
        console.error("Recall failed:", e);
    } finally {
        recallBtn.disabled = false;
    }
});

function renderMemories(memories) {
    if (memories.length === 0) {
        memoryContainer.innerHTML = '<div class="no-results">No results found</div>';
        return;
    }

    memoryContainer.innerHTML = memories.map(m => `
        <div class="memory-item">
            <div class="triplet">
                <span class="node">${escapeHtml(m.subject)}</span>
                <span class="predicate">${escapeHtml(m.predicate)}</span>
                <span class="node">${escapeHtml(m.object)}</span>
            </div>
            <div class="source-text">"${escapeHtml(m.text)}"</div>
        </div>
    `).join('');
}

function escapeHtml(text) {
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
}

if (purgeBtn) {
    purgeBtn.addEventListener('click', async () => {
        const query = purgeInput.value.trim();
        if (!query) return;

        if (!confirm(`Purge all memories related to "${query}"? This cannot be undone.`)) {
            return;
        }

        purgeBtn.disabled = true;
        purgeBtn.textContent = 'Purging...';
        try {
            const res = await invoke('purge_topic', { query });
            showStatus(res);
            purgeInput.value = '';
            await updateStats();
        } catch (e) {
            console.error("Purge failed:", e);
            showStatus("Error during purge");
        } finally {
            purgeBtn.disabled = false;
            purgeBtn.textContent = 'Purge';
        }
    });
}

// Initial stats fetch
updateStats();
setInterval(updateStats, 5000);
