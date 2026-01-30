use crate::extractor::Triplet;
use crate::utils::model_downloader::ModelDownloader;
use anyhow::Result;
use gliner::model::{input::text::TextInput, params::Parameters, pipeline::span::SpanMode, GLiNER};

/// Known relation verbs for extracting predicates from text between entities.
const RELATION_VERBS: &[&str] = &[
    "works", "founded", "leads", "manages", "joined", "acquired", "owns",
    "based", "located", "headquartered", "reports", "partnered", "uses",
    "built", "developed", "created", "provides", "competes", "invested",
    "launched", "serves", "appointed", "hired", "employs", "supervises",
    "sold", "purchased", "ordered", "shipped", "reviewed", "paid",
    "governs", "applies", "requires", "depends", "integrates", "runs",
    "heads", "oversees", "established", "operates", "supports", "maintains",
    "produces", "manufactures", "supplies", "distributes", "sponsors",
];

/// Prepositions that attach to verbs to form compound predicates.
const PREPOSITIONS: &[&str] = &[
    "at", "in", "for", "with", "to", "by", "of", "from", "on",
];

/// Pronouns and noise words that should never be entities.
const GARBAGE_WORDS: &[&str] = &[
    "our", "we", "us", "they", "my", "your", "his", "her", "i", "me",
    "it", "this", "that", "these", "those", "its", "their", "he", "she",
    "who", "which", "what", "where", "when", "how", "why",
];

/// Section header words that GliNER sometimes picks up as entities.
const SECTION_HEADERS: &[&str] = &[
    "overview", "summary", "conclusion", "introduction", "background",
    "features", "pricing", "security", "details", "description",
    "requirements", "objectives", "scope", "methodology", "results",
    "abstract", "appendix", "references", "contents",
];

/// Check if an entity text is garbage that should be filtered out.
fn is_garbage_entity(text: &str) -> bool {
    let trimmed = text.trim();
    // Too short or too long
    if trimmed.len() < 2 || trimmed.len() > 60 {
        return true;
    }
    let lower = trimmed.to_lowercase();
    // Pronouns and noise words
    if GARBAGE_WORDS.contains(&lower.as_str()) {
        return true;
    }
    // Section headers
    if SECTION_HEADERS.contains(&lower.as_str()) {
        return true;
    }
    false
}

/// Extract a clean verb predicate from the text between two entities.
/// Falls back to the raw text if no known verb is found.
fn extract_verb_predicate(between_text: &str) -> String {
    let words: Vec<&str> = between_text
        .split_whitespace()
        .map(|w| w.trim_matches(|c: char| !c.is_alphanumeric()))
        .filter(|w| !w.is_empty())
        .collect();

    for (i, word) in words.iter().enumerate() {
        let lower = word.to_lowercase();
        if RELATION_VERBS.iter().any(|v| lower == *v || lower.ends_with(v)) {
            // Check if next word is a preposition to attach
            if i + 1 < words.len() {
                let next_lower = words[i + 1].to_lowercase();
                if PREPOSITIONS.contains(&next_lower.as_str()) {
                    return format!("{} {}", lower, next_lower);
                }
            }
            return lower;
        }
    }

    // Fallback: return raw text between entities, trimmed
    between_text.trim().to_string()
}

pub struct GlinerExtractor {
    engine: GLiNER<SpanMode>,
}

impl GlinerExtractor {
    pub fn new() -> Result<Self> {
        let model_dir = ModelDownloader::ensure_model_files()?;

        let params = Parameters::default();
        let runtime_params = Default::default();

        let tokenizer_path = model_dir.join("tokenizer.json");
        let model_path = model_dir.join("onnx/model.onnx");

        let engine = GLiNER::<SpanMode>::new(params, runtime_params, tokenizer_path, model_path)
            .map_err(|e| anyhow::anyhow!("Gliner engine error: {:?}", e))?;

        Ok(Self { engine })
    }

    pub fn extract_triplets(&self, text: &str) -> Result<Vec<Triplet>> {
        Self::extract_with_labels(text, &self.engine, Self::default_labels())
    }

    /// Default labels for conversational memory extraction
    pub fn default_labels() -> Vec<String> {
        vec![
            // Traditional NER
            "person".to_string(),
            "organization".to_string(),
            "location".to_string(),
            // Technical
            "software".to_string(),
            "programming language".to_string(),
            "technology".to_string(),
            // Conversational memory
            "preference".to_string(),
            "setting".to_string(),
            "value".to_string(),
            "identifier".to_string(),
            "project".to_string(),
            "tool".to_string(),
        ]
    }

    /// Extract triplets with custom labels (static method)
    fn extract_with_labels(
        text: &str,
        engine: &GLiNER<SpanMode>,
        labels: Vec<String>,
    ) -> Result<Vec<Triplet>> {
        // TextInput expects vectors for batching
        let input = TextInput::new(vec![text.to_string()], labels)
            .map_err(|e| anyhow::anyhow!("Input creation error: {:?}", e))?;

        let output = engine
            .inference(input)
            .map_err(|e| anyhow::anyhow!("Gliner predict error: {:?}", e))?;

        let mut triplets = Vec::new();

        // output.spans is Vec<Vec<Span>> corresponding to input texts
        if let Some(entities) = output.spans.first() {
            // Pair-wise triplet formation
            for i in 0..entities.len() {
                for j in 0..entities.len() {
                    if i == j {
                        continue;
                    }

                    let e1 = &entities[i];
                    let e2 = &entities[j];

                    // Span uses getters
                    let (e1_start, e1_end) = e1.offsets();
                    let (e2_start, e2_end) = e2.offsets();

                    // Ensure e1 comes before e2
                    if e1_start < e2_start {
                        let subject = e1.text().to_string();
                        let object = e2.text().to_string();

                        // Filter garbage entities
                        if is_garbage_entity(&subject) || is_garbage_entity(&object) {
                            continue;
                        }

                        let start = e1_end;
                        let end = e2_start;
                        if start < end {
                            let between = &text[start..end];
                            let predicate = extract_verb_predicate(between);
                            if !predicate.is_empty() {
                                triplets.push(Triplet {
                                    subject,
                                    predicate,
                                    object,
                                    confidence: (e1.probability() + e2.probability()) / 2.0,
                                    source_span: (e1_start, e2_end),
                                });
                            }
                        }
                    }
                }
            }
        }

        Ok(triplets)
    }
}
