use crate::extractor::Triplet;
use regex::Regex;

/// Pattern-based extractor for conversational memory triplets.
/// Focuses on common conversational structures that GliNER misses.
pub struct PatternMatcher {
    patterns: Vec<ConversationalPattern>,
}

struct ConversationalPattern {
    regex: Regex,
    extractor: fn(&regex::Captures, &str) -> Option<Triplet>,
    confidence: f32,
}

impl Default for PatternMatcher {
    fn default() -> Self {
        Self::new()
    }
}

impl PatternMatcher {
    pub fn new() -> Self {
        Self {
            patterns: Self::build_patterns(),
        }
    }

    fn build_patterns() -> Vec<ConversationalPattern> {
        vec![
            // "I prefer X" / "I like X" / "I love X"
            ConversationalPattern {
                regex: Regex::new(r"(?i)\bI\s+(prefer|like|love|enjoy|use|want|need)\s+(.+?)(?:\.|$)").unwrap(),
                extractor: |caps, _| {
                    let predicate = caps.get(1)?.as_str().to_lowercase();
                    let object = caps.get(2)?.as_str().trim().to_string();
                    if object.is_empty() { return None; }
                    Some(Triplet {
                        subject: "I".to_string(),
                        predicate,
                        object,
                        confidence: 0.85,
                        source_span: (caps.get(0)?.start(), caps.get(0)?.end()),
                    })
                },
                confidence: 0.85,
            },
            // "I am a X" / "I'm a X"
            ConversationalPattern {
                regex: Regex::new(r"(?i)\bI(?:'m|\s+am)\s+(?:a\s+)?(.+?)(?:\.|$)").unwrap(),
                extractor: |caps, _| {
                    let object = caps.get(1)?.as_str().trim().to_string();
                    if object.is_empty() || object.len() < 2 { return None; }
                    Some(Triplet {
                        subject: "I".to_string(),
                        predicate: "am".to_string(),
                        object,
                        confidence: 0.85,
                        source_span: (caps.get(0)?.start(), caps.get(0)?.end()),
                    })
                },
                confidence: 0.85,
            },
            // "I work at X" / "I work for X"
            ConversationalPattern {
                regex: Regex::new(r"(?i)\bI\s+work\s+(at|for)\s+(.+?)(?:\.|$)").unwrap(),
                extractor: |caps, _| {
                    let prep = caps.get(1)?.as_str();
                    let object = caps.get(2)?.as_str().trim().to_string();
                    if object.is_empty() { return None; }
                    Some(Triplet {
                        subject: "I".to_string(),
                        predicate: format!("work {}", prep),
                        object,
                        confidence: 0.90,
                        source_span: (caps.get(0)?.start(), caps.get(0)?.end()),
                    })
                },
                confidence: 0.90,
            },
            // "I live in X"
            ConversationalPattern {
                regex: Regex::new(r"(?i)\bI\s+live\s+in\s+(.+?)(?:\.|$)").unwrap(),
                extractor: |caps, _| {
                    let object = caps.get(1)?.as_str().trim().to_string();
                    if object.is_empty() { return None; }
                    Some(Triplet {
                        subject: "I".to_string(),
                        predicate: "live in".to_string(),
                        object,
                        confidence: 0.90,
                        source_span: (caps.get(0)?.start(), caps.get(0)?.end()),
                    })
                },
                confidence: 0.90,
            },
            // "My X is Y" (e.g., "My name is Alice", "My API key is sk-123")
            ConversationalPattern {
                regex: Regex::new(r"(?i)\bMy\s+(\w+(?:\s+\w+)?)\s+is\s+(.+?)(?:\.|$)").unwrap(),
                extractor: |caps, _| {
                    let attr = caps.get(1)?.as_str().trim();
                    let value = caps.get(2)?.as_str().trim().to_string();
                    if value.is_empty() { return None; }
                    Some(Triplet {
                        subject: format!("My {}", attr),
                        predicate: "is".to_string(),
                        object: value,
                        confidence: 0.90,
                        source_span: (caps.get(0)?.start(), caps.get(0)?.end()),
                    })
                },
                confidence: 0.90,
            },
            // "My favorite X is Y"
            ConversationalPattern {
                regex: Regex::new(r"(?i)\bMy\s+favorite\s+(\w+)\s+is\s+(.+?)(?:\.|$)").unwrap(),
                extractor: |caps, _| {
                    let category = caps.get(1)?.as_str().trim();
                    let value = caps.get(2)?.as_str().trim().to_string();
                    if value.is_empty() { return None; }
                    Some(Triplet {
                        subject: format!("My favorite {}", category),
                        predicate: "is".to_string(),
                        object: value,
                        confidence: 0.90,
                        source_span: (caps.get(0)?.start(), caps.get(0)?.end()),
                    })
                },
                confidence: 0.90,
            },
            // "X is my Y" (e.g., "Alice is my manager")
            ConversationalPattern {
                regex: Regex::new(r"(?i)\b(\w+)\s+is\s+my\s+(.+?)(?:\.|$)").unwrap(),
                extractor: |caps, _| {
                    let subject = caps.get(1)?.as_str().trim().to_string();
                    let role = caps.get(2)?.as_str().trim().to_string();
                    if subject.is_empty() || role.is_empty() { return None; }
                    // Skip if subject is a common word
                    if ["the", "this", "that", "it", "there"].contains(&subject.to_lowercase().as_str()) {
                        return None;
                    }
                    Some(Triplet {
                        subject,
                        predicate: "is".to_string(),
                        object: format!("my {}", role),
                        confidence: 0.85,
                        source_span: (caps.get(0)?.start(), caps.get(0)?.end()),
                    })
                },
                confidence: 0.85,
            },
            // "The X is Y" (e.g., "The database URL is postgres://...")
            ConversationalPattern {
                regex: Regex::new(r"(?i)\bThe\s+(\w+(?:\s+\w+)?)\s+is\s+(.+?)(?:\.|$)").unwrap(),
                extractor: |caps, _| {
                    let attr = caps.get(1)?.as_str().trim();
                    let value = caps.get(2)?.as_str().trim().to_string();
                    if value.is_empty() { return None; }
                    Some(Triplet {
                        subject: attr.to_string(),
                        predicate: "is".to_string(),
                        object: value,
                        confidence: 0.85,
                        source_span: (caps.get(0)?.start(), caps.get(0)?.end()),
                    })
                },
                confidence: 0.85,
            },
            // "Use X for Y" / "Use port X"
            ConversationalPattern {
                regex: Regex::new(r"(?i)\bUse\s+(?:port\s+)?(\S+)\s+(?:for\s+)?(.+?)(?:\.|$)").unwrap(),
                extractor: |caps, text| {
                    let value = caps.get(1)?.as_str().trim().to_string();
                    let context = caps.get(2)?.as_str().trim().to_string();
                    if value.is_empty() { return None; }
                    // Check if this is a port number pattern
                    if text.to_lowercase().contains("port") {
                        Some(Triplet {
                            subject: context,
                            predicate: "uses port".to_string(),
                            object: value,
                            confidence: 0.85,
                            source_span: (caps.get(0)?.start(), caps.get(0)?.end()),
                        })
                    } else {
                        Some(Triplet {
                            subject: context,
                            predicate: "uses".to_string(),
                            object: value,
                            confidence: 0.80,
                            source_span: (caps.get(0)?.start(), caps.get(0)?.end()),
                        })
                    }
                },
                confidence: 0.85,
            },
            // "X uses Y" / "X prefers Y" (for projects/tools)
            ConversationalPattern {
                regex: Regex::new(r"(?i)\b(?:my\s+)?(\w+(?:\s+project)?)\s+(uses|prefers|requires)\s+(.+?)(?:\.|$)").unwrap(),
                extractor: |caps, _| {
                    let subject = caps.get(1)?.as_str().trim().to_string();
                    let predicate = caps.get(2)?.as_str().to_lowercase();
                    let object = caps.get(3)?.as_str().trim().to_string();
                    if subject.is_empty() || object.is_empty() { return None; }
                    Some(Triplet {
                        subject,
                        predicate,
                        object,
                        confidence: 0.85,
                        source_span: (caps.get(0)?.start(), caps.get(0)?.end()),
                    })
                },
                confidence: 0.85,
            },
        ]
    }

    /// Extract triplets using pattern matching
    pub fn extract_triplets(&self, text: &str) -> Vec<Triplet> {
        let mut triplets = Vec::new();
        let mut seen_spans: Vec<(usize, usize)> = Vec::new();

        for pattern in &self.patterns {
            for caps in pattern.regex.captures_iter(text) {
                if let Some(triplet) = (pattern.extractor)(&caps, text) {
                    // Avoid duplicate spans
                    let dominated = seen_spans.iter().any(|(s, e)| {
                        triplet.source_span.0 >= *s && triplet.source_span.1 <= *e
                    });
                    if !dominated {
                        seen_spans.push(triplet.source_span);
                        triplets.push(triplet);
                    }
                }
            }
        }

        triplets
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_i_prefer() {
        let matcher = PatternMatcher::new();
        let triplets = matcher.extract_triplets("I prefer dark mode");
        assert!(!triplets.is_empty());
        assert_eq!(triplets[0].subject, "I");
        assert_eq!(triplets[0].predicate, "prefer");
        assert_eq!(triplets[0].object, "dark mode");
    }

    #[test]
    fn test_my_name_is() {
        let matcher = PatternMatcher::new();
        let triplets = matcher.extract_triplets("My name is Alice");
        assert!(!triplets.is_empty());
        assert_eq!(triplets[0].subject, "My name");
        assert_eq!(triplets[0].predicate, "is");
        assert_eq!(triplets[0].object, "Alice");
    }

    #[test]
    fn test_i_work_at() {
        let matcher = PatternMatcher::new();
        let triplets = matcher.extract_triplets("I work at Google");
        assert!(!triplets.is_empty());
        assert_eq!(triplets[0].subject, "I");
        assert_eq!(triplets[0].predicate, "work at");
        assert_eq!(triplets[0].object, "Google");
    }

    #[test]
    fn test_x_is_my_y() {
        let matcher = PatternMatcher::new();
        let triplets = matcher.extract_triplets("Alice is my manager");
        assert!(!triplets.is_empty());
        assert_eq!(triplets[0].subject, "Alice");
        assert_eq!(triplets[0].predicate, "is");
        assert_eq!(triplets[0].object, "my manager");
    }
}
