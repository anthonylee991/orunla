use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "orunla")]
#[command(about = "Orunla: Local-first intelligent memory system", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Start the memory server
    Serve {
        #[arg(short, long, default_value_t = 7432)]
        port: u16,
        /// API key for authentication (recommended for security)
        #[arg(long, env = "ORUNLA_API_KEY")]
        api_key: Option<String>,
    },
    /// Ingest text into memory
    Ingest {
        /// Text to ingest
        text: Option<String>,
        /// File to ingest (txt, md, json, csv)
        #[arg(short, long)]
        file: Option<PathBuf>,
    },
    /// Ingest a document file (txt, md, json, csv)
    IngestFile {
        /// File to ingest
        path: PathBuf,
    },
    /// Recall memories based on query
    Recall {
        query: String,
        #[arg(short, long, default_value_t = 5)]
        limit: usize,
        #[arg(short, long, default_value_t = 0.1)]
        min_strength: f32,
    },
    /// Show storage statistics
    Stats,
    /// Delete a specific memory by ID
    Delete {
        /// Edge ID to delete
        id: String,
    },
    /// Purge memories related to a topic
    Purge {
        /// Topic or keyword to purge
        query: String,
    },
    /// Run garbage collection to permanently delete highly decayed memories
    Gc {
        /// Strength threshold for deletion (default: 0.05)
        #[arg(short, long, default_value_t = 0.05)]
        threshold: f32,
    },
    /// Deduplicate nodes using local similarity heuristics
    Dedup,
    /// Activate a Pro license key
    Activate {
        /// License key from purchase email
        license_key: String,
    },
    /// Show current license status
    License,
    /// Manually sync memories with cloud (Pro only)
    Sync,
    /// Run extraction benchmark to evaluate extractor quality
    Benchmark {
        /// Path to test cases JSON file
        #[arg(short, long, default_value = "benchmark_cases.json")]
        cases: PathBuf,
        /// Show detailed output for each test case
        #[arg(short, long)]
        verbose: bool,
        /// Extractor mode: gliner, patterns, hybrid, or compare (runs all and compares)
        #[arg(short, long, default_value = "compare")]
        mode: String,
    },
}
