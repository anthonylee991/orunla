pub mod gliner;
pub mod hybrid;
pub mod patterns;
pub mod tokenizer;

pub struct ExtractionRequest {
    pub text: String,
    pub source_id: Option<String>,
    pub timestamp: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone)]
pub struct Triplet {
    pub subject: String,
    pub predicate: String,
    pub object: String,
    pub confidence: f32,
    pub source_span: (usize, usize),
}

pub struct ExtractionResult {
    pub triplets: Vec<Triplet>,
    pub raw_text: String,
}

/// Normalize a predicate to a canonical form for graph consistency.
pub fn normalize_predicate(predicate: &str) -> String {
    let lower = predicate.to_lowercase();
    let trimmed = lower.trim();

    match trimmed {
        "co-founded" | "established" => "founded".to_string(),
        "heads" | "runs" | "ceo of" | "cto of" | "coo of" | "cfo of"
        | "director of" | "president of" | "chairman of" => "leads".to_string(),
        "works for" | "employed at" | "employed by" => "works at".to_string(),
        "supervises" | "oversees" => "manages".to_string(),
        "based in" | "headquartered in" => "located in".to_string(),
        "bought" => "acquired".to_string(),
        "requires" | "depends on" | "depends" => "uses".to_string(),
        "partnered with" => "partners with".to_string(),
        other => other.to_string(),
    }
}
