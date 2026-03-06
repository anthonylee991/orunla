use clap::Parser;
use orunla::benchmark;
use orunla::cli;
use orunla::extractor::gliner::GlinerExtractor;
use orunla::extractor::hybrid::HybridExtractor;
use orunla::graph::{Edge, GraphStore, Node, NodeType};
use orunla::retriever::{search::HybridRetriever, RecallRequest, Retriever};
use orunla::server;
use orunla::storage::{sqlite::SqliteStorage, Storage, StorageConfig};
use orunla::utils::document::{
    chunk_document, detect_file_type, parse_csv, parse_json_lines, read_file_content,
};

fn ingest_text(
    storage: &mut SqliteStorage,
    extractor: &HybridExtractor,
    text: &str,
    source_name: &str,
) -> usize {
    let triplets = match extractor.extract_triplets(text) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Extraction error: {}", e);
            return 0;
        }
    };

    let mut added = 0;
    for triplet in triplets {
        let s_id = storage
            .resolve_entity(&triplet.subject)
            .unwrap()
            .unwrap_or_else(|| {
                storage
                    .add_node(Node::new(triplet.subject, NodeType::Unknown))
                    .unwrap()
            });
        let o_id = storage
            .resolve_entity(&triplet.object)
            .unwrap()
            .unwrap_or_else(|| {
                storage
                    .add_node(Node::new(triplet.object, NodeType::Unknown))
                    .unwrap()
            });
        let edge = Edge::new(s_id, o_id, triplet.predicate, text.to_string());
        if storage.add_edge(edge).is_ok() {
            added += 1;
        }
    }
    println!("[{}] Extracted {} triplets", source_name, added);
    added
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = cli::Cli::parse();

    let config = StorageConfig::default();
    let mut storage = SqliteStorage::new(config.clone());

    storage.init()?;

    match args.command {
        cli::Commands::Serve { port, api_key } => {
            println!("Starting Orunla memory server...");
            server::start_server(storage, port, api_key).await?;
        }
        cli::Commands::Ingest { text, file } => {
            println!("Initializing hybrid extractor...");
            let extractor = HybridExtractor::new()?;

            match (text, file) {
                (Some(t), None) => {
                    ingest_text(&mut storage, &extractor, &t, "cli");
                }
                (None, Some(f)) => {
                    let content = read_file_content(&f)?;
                    println!("Read file: {} ({} bytes)", f.display(), content.len());

                    let file_type = detect_file_type(&f);
                    let chunks: Vec<(String, &str)> = match file_type {
                        "json" => parse_json_lines(&content)
                            .into_iter()
                            .map(|c| (c, "json_line"))
                            .collect(),
                        "csv" => parse_csv(&content)
                            .into_iter()
                            .map(|c| (c, "csv_row"))
                            .collect(),
                        _ => {
                            let docs = chunk_document(&content, 1000);
                            docs.into_iter().map(|c| (c, "paragraph")).collect()
                        }
                    };

                    let mut total = 0;
                    for (i, (chunk, chunk_type)) in chunks.iter().enumerate() {
                        println!(
                            "Processing chunk {}/{} ({})...",
                            i + 1,
                            chunks.len(),
                            chunk_type
                        );
                        total +=
                            ingest_text(&mut storage, &extractor, chunk, &f.display().to_string());
                    }
                    println!("Total: {} triplets from file", total);
                }
                (None, None) => {
                    eprintln!("Error: Must provide --text or --file");
                    std::process::exit(1);
                }
                (Some(_), Some(_)) => {
                    eprintln!("Error: Cannot use both --text and --file");
                    std::process::exit(1);
                }
            }
        }
        cli::Commands::IngestFile { path } => {
            println!("Initializing hybrid extractor...");
            let extractor = HybridExtractor::new()?;

            let content = read_file_content(&path)?;
            println!("Read file: {} ({} bytes)", path.display(), content.len());

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
                _ => {
                    let docs = chunk_document(&content, 2000);
                    docs.into_iter().map(|c| (c, "paragraph")).collect()
                }
            };

            let mut total = 0;
            for (i, (chunk, chunk_type)) in chunks.iter().enumerate() {
                println!(
                    "Processing chunk {}/{} ({})...",
                    i + 1,
                    chunks.len(),
                    chunk_type
                );
                total += ingest_text(&mut storage, &extractor, chunk, &path.display().to_string());
            }
            println!("Total: {} triplets from file", total);
        }
        cli::Commands::Recall { query, limit, min_strength } => {
            println!("Recalling memories for: {} (min_strength: {})", query, min_strength);
            let retriever = HybridRetriever::new(&storage);
            let request = RecallRequest {
                query,
                limit,
                min_confidence: 0.0,
                min_strength,
            };
            let response = retriever.recall(request)?;
            for m in response.memories {
                println!(
                    "- {} -> {} -> {} (strength: {:.2}, conf: {:.2})",
                    m.subject_node.label, m.edge.predicate, m.object_node.label, m.relevance_score, m.edge.confidence
                );
            }
        }
        cli::Commands::Stats => {
            let stats = storage.stats()?;
            println!(
                "Nodes: {}, Edges: {}, DB Size: {} bytes",
                stats.node_count, stats.edge_count, stats.db_size_bytes
            );
        }
        cli::Commands::Delete { id } => {
            println!("Deleting memory: {}", id);
            storage.delete_edge(&id)?;
            let orphaned = storage.cleanup_orphaned_nodes()?;
            println!("Memory deleted. Cleaned up {} orphaned nodes.", orphaned);
        }
        cli::Commands::Purge { query } => {
            println!("Purging memories matching: {}", query);
            let count = storage.delete_edges_by_query(&query)?;
            let orphaned = storage.cleanup_orphaned_nodes()?;
            println!(
                "Purged {} memories and cleaned up {} orphaned nodes.",
                count, orphaned
            );
        }
        cli::Commands::Gc { threshold } => {
            println!("Running Garbage Collection (threshold: {})...", threshold);
            let count = storage.hard_gc(threshold)?;
            let orphaned = storage.cleanup_orphaned_nodes()?;
            println!(
                "GC Complete. Permanently deleted {} decayed memories and cleaned up {} orphaned nodes.",
                count, orphaned
            );
        }
        cli::Commands::Dedup => {
            println!("Running Node Deduplication...");
            let nodes = storage.list_nodes()?;
            let mut normalized_map: std::collections::HashMap<String, Vec<Node>> = std::collections::HashMap::new();

            for node in nodes {
                let norm = node.label.to_lowercase().chars().filter(|c| c.is_alphanumeric()).collect::<String>();
                normalized_map.entry(norm).or_default().push(node);
            }

            let mut merged_count = 0;
            for (norm, group) in normalized_map {
                if group.len() > 1 && !norm.is_empty() {
                    let winner = &group[0];
                    for loser in &group[1..] {
                        println!("Merging '{}' (id: {}) -> '{}' (id: {})", loser.label, loser.id, winner.label, winner.id);
                        storage.merge_nodes(&winner.id, &loser.id)?;
                        merged_count += 1;
                    }
                }
            }
            println!("Deduplication complete. Merged {} nodes.", merged_count);
        }
        cli::Commands::Benchmark { cases, verbose, mode } => {
            println!("Loading test cases from: {}", cases.display());
            let test_cases = benchmark::load_test_cases(&cases)?;

            match mode.as_str() {
                "gliner" => {
                    println!("Initializing GliNER extractor...");
                    let extractor = GlinerExtractor::new()?;
                    let result = benchmark::run_benchmark(&extractor, &test_cases, verbose)?;
                    benchmark::print_summary(&result);
                }
                "patterns" => {
                    println!("Using pattern-based extractor...");
                    let matcher = orunla::extractor::patterns::PatternMatcher::new();
                    let result = benchmark::run_benchmark_with(
                        "Patterns",
                        |text| Ok(matcher.extract_triplets(text)),
                        &test_cases,
                        verbose,
                    )?;
                    benchmark::print_summary(&result);
                }
                "hybrid" => {
                    println!("Initializing hybrid extractor...");
                    let extractor = orunla::extractor::hybrid::HybridExtractor::new()?;
                    let result = benchmark::run_benchmark_with(
                        "Hybrid",
                        |text| extractor.extract_triplets(text),
                        &test_cases,
                        verbose,
                    )?;
                    benchmark::print_summary(&result);
                }
                "compare" | _ => {
                    println!("Running comparison benchmark...\n");

                    println!("=== GliNER Extractor ===");
                    let gliner = GlinerExtractor::new()?;
                    let gliner_result = benchmark::run_benchmark(&gliner, &test_cases, verbose)?;
                    benchmark::print_summary(&gliner_result);

                    println!("\n=== Pattern Extractor ===");
                    let matcher = orunla::extractor::patterns::PatternMatcher::new();
                    let pattern_result = benchmark::run_benchmark_with(
                        "Patterns",
                        |text| Ok(matcher.extract_triplets(text)),
                        &test_cases,
                        verbose,
                    )?;
                    benchmark::print_summary(&pattern_result);

                    println!("\n=== Hybrid Extractor ===");
                    let hybrid = orunla::extractor::hybrid::HybridExtractor::new()?;
                    let hybrid_result = benchmark::run_benchmark_with(
                        "Hybrid",
                        |text| hybrid.extract_triplets(text),
                        &test_cases,
                        verbose,
                    )?;
                    benchmark::print_summary(&hybrid_result);

                    benchmark::print_comparison(&gliner_result, &hybrid_result);
                }
            }
        }
    }

    Ok(())
}
