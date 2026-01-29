use crate::graph::Edge;
use chrono::{DateTime, Utc};

pub fn calculate_strength(edge: &Edge, now: DateTime<Utc>) -> f32 {
    let recency_days = (now - edge.last_accessed).num_days() as f32;

    // Ebbinghaus forgetting curve: R = e^(-t/S)
    // t = time since last access, S = stability factor
    let stability = 30.0; // Base stability in days
    let decay = (-recency_days / stability).exp();

    // Access count boost (spacing effect)
    // Use 1.0 + ln(1 + count) to ensure boost >= 1.0
    let access_boost = 1.0 + (edge.access_count as f32).ln_1p();

    // Confidence factor (0.0 - 1.0)
    let confidence = edge.confidence;

    decay * access_boost * confidence
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::Edge;
    use chrono::{Duration, Utc};

    #[test]
    fn test_strength_decay() {
        let now = Utc::now();
        let mut edge = Edge::new("s".into(), "t".into(), "p".into(), "text".into());

        // Fresh memory
        let s1 = calculate_strength(&edge, now);

        // Memory 30 days later
        edge.last_accessed = now - Duration::days(30);
        let s2 = calculate_strength(&edge, now);

        assert!(s2 < s1);
        println!("S1: {}, S2: {}", s1, s2);
    }
}
