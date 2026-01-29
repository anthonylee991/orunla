use orunla::extractor::gliner::GlinerExtractor;
use orunla::graph::{Edge, GraphStore, Node, NodeType};
use orunla::retriever::{search::HybridRetriever, RecallRequest, Retriever};
use orunla::storage::{sqlite::SqliteStorage, Storage, StorageConfig};
use orunla::utils::document::{chunk_document, parse_csv, parse_json_lines};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{Manager, State};
use tokio::sync::Mutex;

pub struct AppState {
    pub storage: Mutex<SqliteStorage>,
    pub extractor: Arc<GlinerExtractor>,
}

#[derive(Serialize)]
pub struct IngestResponse {
    pub added_triplets: usize,
}

#[derive(Serialize)]
pub struct MemoryView {
    pub id: String,
    pub subject: String,
    pub predicate: String,
    pub object: String,
    pub text: String,
    pub confidence: f32,
    pub strength: f32,
}

#[derive(Serialize)]
pub struct RecallResponse {
    pub memories: Vec<MemoryView>,
}

#[derive(Serialize)]
pub struct IngestFileResponse {
    pub file: Option<String>,
    pub chunks_processed: usize,
    pub total_triplets_added: usize,
}

fn detect_file_type(path: &PathBuf) -> &str {
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    match ext {
        "txt" | "md" | "markdown" | "rst" => "text",
        "json" => "json",
        "csv" => "csv",
        _ => "text",
    }
}

#[tauri::command]
async fn ingest(state: State<'_, AppState>, text: String) -> Result<IngestResponse, String> {
    let triplets = state
        .extractor
        .extract_triplets(&text)
        .map_err(|e| e.to_string())?;
    let mut storage = state.storage.lock().await;

    let mut added = 0;
    for triplet in triplets {
        let s_id = storage
            .resolve_entity(&triplet.subject)
            .map_err(|e| e.to_string())?
            .unwrap_or_else(|| {
                storage
                    .add_node(Node::new(triplet.subject, NodeType::Unknown))
                    .unwrap()
            });
        let o_id = storage
            .resolve_entity(&triplet.object)
            .map_err(|e| e.to_string())?
            .unwrap_or_else(|| {
                storage
                    .add_node(Node::new(triplet.object, NodeType::Unknown))
                    .unwrap()
            });
        let edge = Edge::new(s_id, o_id, triplet.predicate, text.clone());
        storage.add_edge(edge).map_err(|e| e.to_string())?;
        added += 1;
    }

    Ok(IngestResponse {
        added_triplets: added,
    })
}

#[tauri::command]
async fn recall(
    state: State<'_, AppState>,
    query: String,
    limit: Option<usize>,
    min_strength: Option<f32>,
) -> Result<RecallResponse, String> {
    let storage = state.storage.lock().await;
    let retriever = HybridRetriever::new(&*storage);

    let request = RecallRequest {
        query,
        limit: limit.unwrap_or(5),
        min_confidence: 0.0,
        min_strength: min_strength.unwrap_or(0.1),
    };

    let response = retriever.recall(request).map_err(|e| e.to_string())?;

    let memories = response
        .memories
        .into_iter()
        .map(|m| MemoryView {
            id: m.edge.id,
            subject: m.subject_node.label,
            predicate: m.edge.predicate,
            object: m.object_node.label,
            text: m.edge.source_text,
            confidence: m.edge.confidence,
            strength: m.relevance_score,
        })
        .collect();

    Ok(RecallResponse { memories })
}

#[tauri::command]
async fn get_stats(state: State<'_, AppState>) -> Result<orunla::storage::StorageStats, String> {
    let storage = state.storage.lock().await;
    storage.stats().map_err(|e| e.to_string())
}

#[tauri::command]
async fn ingest_file(
    state: State<'_, AppState>,
    file_path: String,
) -> Result<IngestFileResponse, String> {
    let path = PathBuf::from(&file_path);
    if !path.exists() {
        return Err("File does not exist".to_string());
    }

    let content = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let file_name = path
        .file_name()
        .and_then(|s| s.to_str())
        .map(|s| s.to_string());

    let file_type = detect_file_type(&path);
    let chunks: Vec<(String, &str)> = match file_type {
        "json" => parse_json_lines(&content)
            .into_iter()
            .map(|c| (c, "json_line"))
            .collect(),
        "csv" => parse_csv(&content)
            .into_iter()
            .map(|c| (c, "csv_row"))
            .collect(),
        _ => chunk_document(&content, 1000)
            .into_iter()
            .map(|c| (c, "paragraph"))
            .collect(),
    };

    let mut storage = state.storage.lock().await;
    let mut total_added = 0;

    for (i, (chunk, _chunk_type)) in chunks.iter().enumerate() {
        let triplets = match state.extractor.extract_triplets(chunk) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("Extraction error on chunk {}: {}", i + 1, e);
                continue;
            }
        };

        for triplet in triplets {
            let s_id = storage
                .resolve_entity(&triplet.subject)
                .map_err(|e| e.to_string())?
                .unwrap_or_else(|| {
                    storage
                        .add_node(Node::new(triplet.subject, NodeType::Unknown))
                        .unwrap()
                });
            let o_id = storage
                .resolve_entity(&triplet.object)
                .map_err(|e| e.to_string())?
                .unwrap_or_else(|| {
                    storage
                        .add_node(Node::new(triplet.object, NodeType::Unknown))
                        .unwrap()
                });
            let edge = Edge::new(s_id, o_id, triplet.predicate, chunk.to_string());
            if storage.add_edge(edge).is_ok() {
                total_added += 1;
            }
        }
    }

    Ok(IngestFileResponse {
        file: file_name,
        chunks_processed: chunks.len(),
        total_triplets_added: total_added,
    })
}

#[tauri::command]
async fn delete_memory(state: State<'_, AppState>, id: String) -> Result<String, String> {
    let mut storage = state.storage.lock().await;
    storage.delete_edge(&id).map_err(|e| e.to_string())?;
    let orphaned = storage.cleanup_orphaned_nodes().unwrap_or(0);
    Ok(format!(
        "Memory {} deleted. Cleaned up {} orphaned nodes.",
        id, orphaned
    ))
}

#[tauri::command]
async fn purge_topic(state: State<'_, AppState>, query: String) -> Result<String, String> {
    let mut storage = state.storage.lock().await;
    let count = storage
        .delete_edges_by_query(&query)
        .map_err(|e| e.to_string())?;
    let orphaned = storage.cleanup_orphaned_nodes().unwrap_or(0);
    Ok(format!(
        "Purged {} memories and cleaned up {} orphaned nodes matching '{}'.",
        count, orphaned, query
    ))
}

#[cfg(windows)]
fn fix_dll_path() {
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let mut possible_dlls = Vec::new();
            //Adjacent to exe (dev or manual copy)
            possible_dlls.push(exe_dir.join("onnxruntime.dll"));
            // Tauri v2 resources
            possible_dlls.push(exe_dir.join("resources").join("onnxruntime.dll"));
            possible_dlls.push(exe_dir.join("resources").join("resources").join("onnxruntime.dll"));
            
            for dll_path in possible_dlls {
                if dll_path.exists() {
                    // Set DLL directory for any subsequent loads
                    if let Some(dir) = dll_path.parent() {
                        let mut wide_path: Vec<u16> = dir.to_string_lossy().encode_utf16().collect();
                        wide_path.push(0);
                        unsafe {
                            extern "system" {
                                fn SetDllDirectoryW(lpPathName: *const u16) -> i32;
                            }
                            SetDllDirectoryW(wide_path.as_ptr());
                        }
                    }

                    // Explicitly initialize ORT using environment variables for load-dynamic
                    std::env::set_var("ORT_DYLIB_PATH", &dll_path);
                    match ort::init().commit() {
                        Ok(_) => (),
                        Err(e) => eprintln!("Failed to initialize ORT: {:?}", e),
                    }
                    return;
                }
            }
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    #[cfg(windows)]
    fix_dll_path();

    let config = StorageConfig::default();
    let storage = SqliteStorage::new(config);
    storage.init().expect("Failed to initialize database");

    println!("Initializing smart extractor (GliNER) in Tauri...");
    let extractor = Arc::new(GlinerExtractor::new().expect("Failed to initialize GliNER"));

    tauri::Builder::default()
        .manage(AppState {
            storage: Mutex::new(storage),
            extractor,
        })
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            ingest,
            recall,
            get_stats,
            ingest_file,
            delete_memory,
            purge_topic
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
