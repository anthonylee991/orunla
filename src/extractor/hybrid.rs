use crate::extractor::{gliner::GlinerExtractor, normalize_predicate, patterns::PatternMatcher, Triplet};
use anyhow::Result;

/// Hybrid extractor that combines GliNER neural extraction with pattern-based rules.
/// Pattern matching runs first (high precision for conversational structures),
/// then GliNER fills in gaps for named entities.
pub struct HybridExtractor {
    gliner: GlinerExtractor,
    patterns: PatternMatcher,
}

impl HybridExtractor {
    pub fn new() -> Result<Self> {
        Ok(Self {
            gliner: GlinerExtractor::new()?,
            patterns: PatternMatcher::new(),
        })
    }

    /// Extract triplets using both pattern matching and GliNER.
    /// Deduplicates results based on span overlap.
    pub fn extract_triplets(&self, text: &str) -> Result<Vec<Triplet>> {
        // First, run pattern matching (higher precision for conversational text)
        let mut pattern_triplets = self.patterns.extract_triplets(text);

        // Then, run GliNER for named entity relationships
        let gliner_triplets = self.gliner.extract_triplets(text)?;

        // Merge results, avoiding duplicates
        let mut final_triplets = pattern_triplets.clone();

        for gt in gliner_triplets {
            // Check if this triplet overlaps significantly with any pattern triplet
            let dominated = pattern_triplets.iter().any(|pt| {
                spans_overlap(pt.source_span, gt.source_span)
                    || triplets_similar(pt, &gt)
            });

            if !dominated {
                final_triplets.push(gt);
            }
        }

        // Normalize predicates for graph consistency
        for triplet in &mut final_triplets {
            triplet.predicate = normalize_predicate(&triplet.predicate);
        }

        // Sort by position in text
        final_triplets.sort_by_key(|t| t.source_span.0);

        Ok(final_triplets)
    }

    /// Get the underlying GliNER extractor for direct access
    pub fn gliner(&self) -> &GlinerExtractor {
        &self.gliner
    }

    /// Get the underlying pattern matcher for direct access
    pub fn patterns(&self) -> &PatternMatcher {
        &self.patterns
    }
}

/// Check if two spans overlap significantly (more than 50%)
fn spans_overlap(a: (usize, usize), b: (usize, usize)) -> bool {
    let overlap_start = a.0.max(b.0);
    let overlap_end = a.1.min(b.1);

    if overlap_start >= overlap_end {
        return false;
    }

    let overlap_len = overlap_end - overlap_start;
    let a_len = a.1 - a.0;
    let b_len = b.1 - b.0;

    // If overlap is >50% of either span, consider them overlapping
    overlap_len * 2 > a_len || overlap_len * 2 > b_len
}

/// Check if two triplets are semantically similar
fn triplets_similar(a: &Triplet, b: &Triplet) -> bool {
    let subj_match = fuzzy_eq(&a.subject, &b.subject);
    let obj_match = fuzzy_eq(&a.object, &b.object);

    // If both subject and object match, likely the same triplet
    subj_match && obj_match
}

/// Fuzzy string equality (case-insensitive, one contains the other)
fn fuzzy_eq(a: &str, b: &str) -> bool {
    let a_lower = a.to_lowercase();
    let b_lower = b.to_lowercase();

    a_lower == b_lower || a_lower.contains(&b_lower) || b_lower.contains(&a_lower)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spans_overlap() {
        assert!(spans_overlap((0, 10), (5, 15)));
        assert!(!spans_overlap((0, 10), (20, 30)));
        assert!(spans_overlap((0, 10), (8, 12))); // 20% overlap of first, 50% of second
    }

    #[test]
    fn test_fuzzy_eq() {
        assert!(fuzzy_eq("Alice", "alice"));
        assert!(fuzzy_eq("Alice", "Alice Smith"));
        assert!(!fuzzy_eq("Alice", "Bob"));
    }
}
