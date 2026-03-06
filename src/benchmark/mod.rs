pub mod metrics;

use crate::extractor::Triplet;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Deserialize)]
pub struct ExpectedTriplet {
    pub subject: String,
    pub predicate: String,
    pub object: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TestCase {
    pub id: String,
    pub text: String,
    pub expected: Vec<ExpectedTriplet>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CaseResult {
    pub id: String,
    pub text: String,
    pub expected_count: usize,
    pub extracted_count: usize,
    pub true_positives: usize,
    pub false_positives: usize,
    pub false_negatives: usize,
    pub matched_triplets: Vec<MatchedTriplet>,
    pub unmatched_expected: Vec<ExpectedTriplet>,
    pub extra_extracted: Vec<ExtractedTriplet>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MatchedTriplet {
    pub expected: ExpectedTriplet,
    pub extracted: ExtractedTriplet,
}

#[derive(Debug, Clone, Serialize)]
pub struct ExtractedTriplet {
    pub subject: String,
    pub predicate: String,
    pub object: String,
    pub confidence: f32,
}

impl From<&Triplet> for ExtractedTriplet {
    fn from(t: &Triplet) -> Self {
        Self {
            subject: t.subject.clone(),
            predicate: t.predicate.clone(),
            object: t.object.clone(),
            confidence: t.confidence,
        }
    }
}

impl Serialize for ExpectedTriplet {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("ExpectedTriplet", 3)?;
        state.serialize_field("subject", &self.subject)?;
        state.serialize_field("predicate", &self.predicate)?;
        state.serialize_field("object", &self.object)?;
        state.end()
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct BenchmarkResult {
    pub extractor_name: String,
    pub total_cases: usize,
    pub total_expected: usize,
    pub total_extracted: usize,
    pub true_positives: usize,
    pub false_positives: usize,
    pub false_negatives: usize,
    pub precision: f32,
    pub recall: f32,
    pub f1_score: f32,
    pub case_results: Vec<CaseResult>,
}

/// Load test cases from a JSON file
pub fn load_test_cases(path: &Path) -> Result<Vec<TestCase>> {
    let content = std::fs::read_to_string(path)?;
    let cases: Vec<TestCase> = serde_json::from_str(&content)?;
    Ok(cases)
}

/// Check if two strings match (case-insensitive, allows substring containment)
fn fuzzy_match(expected: &str, actual: &str) -> bool {
    let exp = expected.to_lowercase().trim().to_string();
    let act = actual.to_lowercase().trim().to_string();

    // Exact match
    if exp == act {
        return true;
    }

    // One contains the other
    if exp.contains(&act) || act.contains(&exp) {
        return true;
    }

    false
}

/// Check if a predicate matches (more lenient - looks for key verb/preposition)
fn predicate_match(expected: &str, actual: &str) -> bool {
    let exp = expected.to_lowercase();
    let act = actual.to_lowercase();

    // Exact match
    if exp.trim() == act.trim() {
        return true;
    }

    // Check if expected is contained in actual (actual might have extra words)
    if act.contains(&exp) {
        return true;
    }

    // Check if key words match (split and compare)
    let exp_words: Vec<&str> = exp.split_whitespace().collect();
    let act_words: Vec<&str> = act.split_whitespace().collect();

    // If expected has key words that appear in actual
    let mut matched_words = 0;
    for exp_word in &exp_words {
        if act_words.contains(exp_word) {
            matched_words += 1;
        }
    }

    // If most expected words are found, consider it a match
    if !exp_words.is_empty() && matched_words as f32 / exp_words.len() as f32 >= 0.5 {
        return true;
    }

    false
}

/// Check if an extracted triplet matches an expected triplet
fn triplet_matches(expected: &ExpectedTriplet, extracted: &Triplet) -> bool {
    fuzzy_match(&expected.subject, &extracted.subject)
        && fuzzy_match(&expected.object, &extracted.object)
        && predicate_match(&expected.predicate, &extracted.predicate)
}

/// Run a single test case with a generic extraction function
fn run_case<F>(extract_fn: &F, case: &TestCase) -> Result<CaseResult>
where
    F: Fn(&str) -> Result<Vec<Triplet>>,
{
    let extracted = extract_fn(&case.text)?;

    let mut matched_expected: Vec<bool> = vec![false; case.expected.len()];
    let mut matched_extracted: Vec<bool> = vec![false; extracted.len()];
    let mut matched_triplets: Vec<MatchedTriplet> = Vec::new();

    // Find matches using greedy matching
    for (ei, exp) in case.expected.iter().enumerate() {
        for (xi, ext) in extracted.iter().enumerate() {
            if !matched_extracted[xi] && triplet_matches(exp, ext) {
                matched_expected[ei] = true;
                matched_extracted[xi] = true;
                matched_triplets.push(MatchedTriplet {
                    expected: exp.clone(),
                    extracted: ext.into(),
                });
                break;
            }
        }
    }

    let true_positives = matched_triplets.len();
    let false_negatives = matched_expected.iter().filter(|&&m| !m).count();
    let false_positives = matched_extracted.iter().filter(|&&m| !m).count();

    // Collect unmatched expected triplets
    let unmatched_expected: Vec<ExpectedTriplet> = case
        .expected
        .iter()
        .enumerate()
        .filter(|(i, _)| !matched_expected[*i])
        .map(|(_, e)| e.clone())
        .collect();

    // Collect extra extracted triplets
    let extra_extracted: Vec<ExtractedTriplet> = extracted
        .iter()
        .enumerate()
        .filter(|(i, _)| !matched_extracted[*i])
        .map(|(_, e)| e.into())
        .collect();

    Ok(CaseResult {
        id: case.id.clone(),
        text: case.text.clone(),
        expected_count: case.expected.len(),
        extracted_count: extracted.len(),
        true_positives,
        false_positives,
        false_negatives,
        matched_triplets,
        unmatched_expected,
        extra_extracted,
    })
}

/// Run the full benchmark with a generic extraction function
pub fn run_benchmark_with<F>(
    extractor_name: &str,
    extract_fn: F,
    cases: &[TestCase],
    verbose: bool,
) -> Result<BenchmarkResult>
where
    F: Fn(&str) -> Result<Vec<Triplet>>,
{
    let mut case_results: Vec<CaseResult> = Vec::new();

    println!(
        "Running benchmark [{}] with {} test cases...\n",
        extractor_name,
        cases.len()
    );

    for case in cases {
        let result = run_case(&extract_fn, case)?;

        if verbose {
            let status = if result.false_negatives == 0 && result.false_positives == 0 {
                "PASS"
            } else if result.true_positives > 0 {
                "PARTIAL"
            } else if result.expected_count == 0 && result.extracted_count == 0 {
                "PASS"
            } else {
                "FAIL"
            };

            println!(
                "[{}] {}: {}/{} expected found, {} extra",
                status,
                result.id,
                result.true_positives,
                result.expected_count,
                result.false_positives
            );

            if !result.unmatched_expected.is_empty() {
                println!("  Missing:");
                for t in &result.unmatched_expected {
                    println!("    - {} -> {} -> {}", t.subject, t.predicate, t.object);
                }
            }

            if !result.extra_extracted.is_empty() {
                println!("  Extra:");
                for t in &result.extra_extracted {
                    println!(
                        "    + {} -> {} -> {} (conf: {:.2})",
                        t.subject, t.predicate, t.object, t.confidence
                    );
                }
            }
        }

        case_results.push(result);
    }

    // Aggregate metrics
    let total_expected: usize = case_results.iter().map(|r| r.expected_count).sum();
    let total_extracted: usize = case_results.iter().map(|r| r.extracted_count).sum();
    let true_positives: usize = case_results.iter().map(|r| r.true_positives).sum();
    let false_positives: usize = case_results.iter().map(|r| r.false_positives).sum();
    let false_negatives: usize = case_results.iter().map(|r| r.false_negatives).sum();

    let (precision, recall, f1_score) =
        metrics::calculate_metrics(true_positives, false_positives, false_negatives);

    Ok(BenchmarkResult {
        extractor_name: extractor_name.to_string(),
        total_cases: cases.len(),
        total_expected,
        total_extracted,
        true_positives,
        false_positives,
        false_negatives,
        precision,
        recall,
        f1_score,
        case_results,
    })
}

/// Convenience function for GliNER-only benchmark
pub fn run_benchmark(
    extractor: &crate::extractor::gliner::GlinerExtractor,
    cases: &[TestCase],
    verbose: bool,
) -> Result<BenchmarkResult> {
    run_benchmark_with(
        "GliNER",
        |text| extractor.extract_triplets(text),
        cases,
        verbose,
    )
}

/// Print the benchmark summary
pub fn print_summary(result: &BenchmarkResult) {
    println!("\n{}", "=".repeat(50));
    println!("BENCHMARK SUMMARY: {}", result.extractor_name);
    println!("{}", "=".repeat(50));
    println!("Total test cases: {}", result.total_cases);
    println!("Total expected triplets: {}", result.total_expected);
    println!("Total extracted triplets: {}", result.total_extracted);
    println!();
    println!(
        "True Positives:  {} (correctly matched)",
        result.true_positives
    );
    println!(
        "False Positives: {} (extracted but not expected)",
        result.false_positives
    );
    println!(
        "False Negatives: {} (expected but not extracted)",
        result.false_negatives
    );
    println!();
    println!(
        "Precision: {:.2}% ({}/{} extracted were correct)",
        result.precision * 100.0,
        result.true_positives,
        result.true_positives + result.false_positives
    );
    println!(
        "Recall:    {:.2}% ({}/{} expected were found)",
        result.recall * 100.0,
        result.true_positives,
        result.total_expected
    );
    println!("F1 Score:  {:.2}%", result.f1_score * 100.0);
    println!("{}", "=".repeat(50));
}

/// Print comparison between two benchmark results
pub fn print_comparison(before: &BenchmarkResult, after: &BenchmarkResult) {
    println!("\n{}", "=".repeat(60));
    println!(
        "COMPARISON: {} vs {}",
        before.extractor_name, after.extractor_name
    );
    println!("{}", "=".repeat(60));

    let precision_delta = (after.precision - before.precision) * 100.0;
    let recall_delta = (after.recall - before.recall) * 100.0;
    let f1_delta = (after.f1_score - before.f1_score) * 100.0;

    println!(
        "Precision: {:.2}% -> {:.2}% ({:+.2}%)",
        before.precision * 100.0,
        after.precision * 100.0,
        precision_delta
    );
    println!(
        "Recall:    {:.2}% -> {:.2}% ({:+.2}%)",
        before.recall * 100.0,
        after.recall * 100.0,
        recall_delta
    );
    println!(
        "F1 Score:  {:.2}% -> {:.2}% ({:+.2}%)",
        before.f1_score * 100.0,
        after.f1_score * 100.0,
        f1_delta
    );
    println!("{}", "=".repeat(60));
}
