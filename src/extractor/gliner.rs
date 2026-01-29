use crate::extractor::Triplet;
use crate::utils::model_downloader::ModelDownloader;
use anyhow::Result;
use gliner::model::{input::text::TextInput, params::Parameters, pipeline::span::SpanMode, GLiNER};

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

                        let start = e1_end;
                        let end = e2_start;
                        if start < end {
                            let predicate = text[start..end].trim().to_string();
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
