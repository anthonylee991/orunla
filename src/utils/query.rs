/// Query expansion and stemming utilities for improved recall.

use std::collections::HashSet;

/// Simple English stemmer that removes common suffixes.
/// Not as sophisticated as Porter/Snowball but handles common cases.
pub fn stem_word(word: &str) -> String {
    let w = word.to_lowercase();

    // Skip short words
    if w.len() <= 3 {
        return w;
    }

    // Handle important special cases explicitly
    match w.as_str() {
        "preference" | "preferences" | "preferred" | "preferring" => return "prefer".to_string(),
        "workplace" | "workplaces" => return "work".to_string(),
        "working" | "worked" | "worker" | "workers" => return "work".to_string(),
        "studies" | "studied" | "studying" => return "study".to_string(),
        "running" | "runner" | "runners" => return "run".to_string(),
        _ => {}
    }

    // Order matters - check longer suffixes first
    let suffixes = [
        // Doubled consonant + ing
        ("tting", "t"),     // setting -> set
        ("nning", "n"),     // running -> run
        ("rring", "r"),     // occurring -> occur
        ("pping", "p"),     // mapping -> map
        ("gging", "g"),     // logging -> log
        ("dding", "d"),     // adding -> add
        ("bbing", "b"),     // grabbing -> grab
        ("mming", "m"),     // programming -> program
        // Long suffixes
        ("ization", ""),    // organization -> organ
        ("isation", ""),    // organisation -> organ
        ("ational", ""),    // organizational -> organiz
        ("ation", ""),      // organization -> organiz
        ("ition", ""),      // definition -> defin
        ("ement", ""),      // management -> manag
        ("erence", "er"),   // preference -> prefer (special case)
        ("ence", ""),       // violence -> violen
        ("ance", ""),       // performance -> perform
        ("ment", ""),       // development -> develop
        ("ness", ""),       // darkness -> dark
        ("ying", "y"),      // studying -> study
        ("ting", "t"),      // setting -> set
        ("ning", "n"),      // running -> run
        ("ring", "r"),      // preferring -> prefer
        ("zing", ""),       // organizing -> organiz
        ("ling", "l"),      // controlling -> control
        ("ding", "d"),      // adding -> add
        ("ging", "g"),      // logging -> log
        ("ping", "p"),      // mapping -> map
        ("bing", "b"),      // grabbing -> grab
        ("ming", "m"),      // programming -> program
        ("ible", ""),       // possible -> poss
        ("able", ""),       // comfortable -> comfort
        ("tion", ""),       // action -> act
        ("sion", ""),       // discussion -> discus
        ("ious", ""),       // various -> var
        ("eous", ""),       // gorgeous -> gorg
        ("place", ""),      // workplace -> work
        ("ful", ""),        // helpful -> help
        ("ive", ""),        // active -> act
        ("ing", ""),        // working -> work
        ("ies", "y"),       // studies -> study
        ("ied", "y"),       // studied -> study
        ("ers", ""),        // workers -> work
        ("est", ""),        // fastest -> fast
        ("ess", ""),        // actress -> actr
        ("dom", ""),        // freedom -> free
        ("ity", ""),        // activity -> activ
        ("ure", ""),        // pressure -> press
        ("ous", ""),        // famous -> fam
        ("ish", ""),        // stylish -> styl
        ("ed", ""),         // worked -> work
        ("er", ""),         // worker -> work
        ("es", ""),         // boxes -> box
        ("ly", ""),         // quickly -> quick
        ("'s", ""),         // user's -> user
        ("s", ""),          // works -> work
    ];

    for (suffix, replacement) in suffixes {
        if w.ends_with(suffix) && w.len() > suffix.len() + 2 {
            let stem = &w[..w.len() - suffix.len()];
            return format!("{}{}", stem, replacement);
        }
    }

    w
}

/// Get synonyms and related terms for common query words.
/// Returns the original word plus any expansions.
pub fn expand_synonyms(word: &str) -> Vec<String> {
    let w = word.to_lowercase();
    let stem = stem_word(&w);

    let mut expansions: Vec<String> = vec![w.clone(), stem.clone()];

    // Add specific synonyms for common query patterns
    let synonyms: &[(&str, &[&str])] = &[
        // Workplace/work
        ("workplace", &["work", "job", "employ", "company", "office"]),
        ("work", &["job", "employ", "occupation"]),
        ("job", &["work", "employ", "occupation"]),
        ("employ", &["work", "job"]),

        // Preferences
        ("preference", &["prefer", "like", "want", "favorite", "favourite"]),
        ("prefer", &["like", "want", "favorite", "favourite", "preference"]),
        ("favorite", &["prefer", "like", "favourite", "best"]),
        ("favourite", &["prefer", "like", "favorite", "best"]),
        ("like", &["prefer", "enjoy", "love"]),

        // Location
        ("location", &["place", "live", "city", "address", "where"]),
        ("live", &["location", "reside", "home", "city"]),
        ("home", &["live", "house", "residence"]),
        ("where", &["location", "place"]),

        // Identity
        ("name", &["called", "named", "identity"]),
        ("who", &["person", "name", "identity"]),

        // Technical
        ("api", &["key", "token", "secret"]),
        ("database", &["db", "sql", "storage"]),
        ("url", &["link", "address", "endpoint"]),
        ("port", &["server", "host"]),

        // Relationships
        ("manager", &["boss", "supervisor", "lead"]),
        ("boss", &["manager", "supervisor"]),
        ("team", &["group", "coworker", "colleague"]),

        // Common verbs (stem forms)
        ("use", &["using", "used", "utilize"]),
        ("create", &["make", "build", "generate"]),
        ("config", &["setting", "configuration", "setup"]),
    ];

    // Check both original and stemmed form for synonym matches
    for (key, values) in synonyms {
        if w == *key || stem == *key || w.starts_with(key) {
            for v in *values {
                if !expansions.contains(&v.to_string()) {
                    expansions.push(v.to_string());
                }
            }
        }
    }

    expansions
}

/// Expand a full query into stemmed and synonym-expanded terms.
/// Returns a deduplicated list of search terms.
pub fn expand_query(query: &str) -> Vec<String> {
    let mut all_terms: HashSet<String> = HashSet::new();

    // Tokenize
    let words: Vec<&str> = query
        .split_whitespace()
        .map(|s| s.trim_matches(|c: char| !c.is_alphanumeric()))
        .filter(|s| !s.is_empty() && s.len() > 1)
        .collect();

    // Skip common stop words
    let stop_words: HashSet<&str> = [
        "the", "a", "an", "is", "are", "was", "were", "be", "been", "being",
        "have", "has", "had", "do", "does", "did", "will", "would", "could",
        "should", "may", "might", "must", "shall", "can", "to", "of", "in",
        "for", "on", "with", "at", "by", "from", "as", "into", "through",
        "that", "which", "who", "whom", "this", "these", "those", "it",
        "its", "i", "me", "my", "you", "your", "he", "him", "his", "she",
        "her", "we", "us", "our", "they", "them", "their", "what", "all",
        "any", "both", "each", "more", "most", "other", "some", "such",
        "and", "but", "or", "so", "if", "then", "because", "about", "up",
        "look", "find", "get", "tell", "show", "give", "know", "mention",
        "memory", "memories", "remember", "recall",
    ].iter().cloned().collect();

    for word in words {
        let lower = word.to_lowercase();

        // Skip stop words but keep important ones
        if stop_words.contains(lower.as_str()) && lower != "i" && lower != "my" {
            continue;
        }

        // Add original
        all_terms.insert(lower.clone());

        // Add stem
        let stem = stem_word(&lower);
        all_terms.insert(stem.clone());

        // Add synonyms
        for syn in expand_synonyms(&lower) {
            all_terms.insert(syn);
        }
    }

    // Convert to vec and sort by length (longer = more specific = search first)
    let mut terms: Vec<String> = all_terms.into_iter().collect();
    terms.sort_by(|a, b| b.len().cmp(&a.len()));
    terms
}

/// Build an FTS5 query with OR logic and prefix matching.
pub fn build_fts_query(terms: &[String]) -> String {
    terms
        .iter()
        .map(|t| format!("{}*", t)) // Prefix match
        .collect::<Vec<_>>()
        .join(" OR ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stem_word() {
        assert_eq!(stem_word("working"), "work");
        assert_eq!(stem_word("preference"), "prefer");
        assert_eq!(stem_word("preferences"), "prefer");
        assert_eq!(stem_word("preferred"), "prefer");
        assert_eq!(stem_word("workplace"), "work");
        assert_eq!(stem_word("running"), "run");
        assert_eq!(stem_word("studies"), "study");
    }

    #[test]
    fn test_expand_synonyms() {
        let expanded = expand_synonyms("workplace");
        assert!(expanded.contains(&"work".to_string()));
        assert!(expanded.contains(&"job".to_string()));

        let expanded = expand_synonyms("preference");
        assert!(expanded.contains(&"prefer".to_string()));
        assert!(expanded.contains(&"like".to_string()));
    }

    #[test]
    fn test_expand_query() {
        let terms = expand_query("where do i work");
        assert!(terms.contains(&"work".to_string()));

        let terms = expand_query("my preference");
        assert!(terms.contains(&"prefer".to_string()));
    }
}
