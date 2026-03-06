pub struct Tokenizer;

impl Tokenizer {
    pub fn tokenize_sentences(text: &str) -> Vec<String> {
        // Very basic sentence splitter
        text.split_inclusive(['.', '!', '?'])
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    }

    pub fn tokenize_words(sentence: &str) -> Vec<String> {
        // Basic word splitter, removing punctuation
        sentence
            .split_whitespace()
            .map(|w| w.trim_matches(|c: char| !c.is_alphanumeric()).to_string())
            .filter(|w| !w.is_empty())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sentence_tokenization() {
        let text = "Hello world. This is a test! Is it working?";
        let sentences = Tokenizer::tokenize_sentences(text);
        assert_eq!(sentences.len(), 3);
        assert_eq!(sentences[0], "Hello world.");
    }

    #[test]
    fn test_word_tokenization() {
        let sentence = "Hello, world!";
        let words = Tokenizer::tokenize_words(sentence);
        assert_eq!(words, vec!["Hello", "world"]);
    }
}
