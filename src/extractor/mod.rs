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
