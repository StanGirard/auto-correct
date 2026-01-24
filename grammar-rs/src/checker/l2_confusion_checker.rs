//! L2 Confusion Checker for French native speakers learning English
//!
//! Detects false friends - words that look similar to French words
//! but have different meanings in English.
//!
//! # Example
//! ```ignore
//! let checker = L2ConfusionChecker::new();
//! // "lecture" will be flagged with suggestion "reading"
//! // (FR "lecture" means reading, not a lecture)
//! ```

use crate::checker::data::en_confusion_l2_fr::get_en_l2_fr_confusion;
use crate::core::{AnalyzedToken, CheckResult, Match, Severity, TokenKind};
use crate::core::traits::Checker;

/// L2 Confusion Checker for French speakers learning English
///
/// Detects false friends - words that French speakers commonly misuse
/// because they look similar to French words with different meanings.
///
/// # Examples
/// - "lecture" → "reading" (FR "lecture" = reading)
/// - "fabric" → "factory" (FR "fabrique" = factory)
/// - "pretend" → "claim" (FR "prétendre" = to claim)
pub struct L2ConfusionChecker {
    /// Minimum confidence factor to trigger a match (higher = fewer matches)
    min_factor: u64,
    /// Minimum word length to check (skip short words)
    min_word_length: usize,
}

impl L2ConfusionChecker {
    /// Create a new L2ConfusionChecker with default settings
    pub fn new() -> Self {
        Self {
            min_factor: 1,       // Flag all matches by default
            min_word_length: 3,  // Skip very short words
        }
    }

    /// Set minimum confidence factor threshold
    ///
    /// Higher values = fewer, higher-confidence matches
    /// - 1: All matches (default)
    /// - 100: Moderate confidence
    /// - 10000: High confidence only
    pub fn with_min_factor(mut self, min_factor: u64) -> Self {
        self.min_factor = min_factor;
        self
    }

    /// Set minimum word length to check
    pub fn with_min_word_length(mut self, length: usize) -> Self {
        self.min_word_length = length;
        self
    }

    /// Generate message for the false friend
    fn generate_message(&self, word: &str, suggestion: &str) -> String {
        format!(
            "Possible faux ami: '{}' ne signifie pas ce que vous pensez. \
             Les francophones confondent souvent ce mot. Vouliez-vous dire '{}' ?",
            word, suggestion
        )
    }
}

impl Default for L2ConfusionChecker {
    fn default() -> Self {
        Self::new()
    }
}

impl Checker for L2ConfusionChecker {
    fn check(&self, _text: &str, tokens: &[AnalyzedToken]) -> CheckResult {
        let mut matches = Vec::new();

        for token in tokens {
            // Skip non-words
            if token.token.kind != TokenKind::Word {
                continue;
            }

            let word = token.token.text;

            // Skip short words
            if word.len() < self.min_word_length {
                continue;
            }

            // Look up the word in L2 FR confusion data
            if let Some(pair) = get_en_l2_fr_confusion(word) {
                // Check if factor meets threshold
                if pair.factor < self.min_factor {
                    continue;
                }

                // Preserve original case in suggestion
                let suggestion = if word.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) {
                    let mut chars = pair.word2.chars();
                    match chars.next() {
                        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                        None => pair.word2.to_string(),
                    }
                } else {
                    pair.word2.to_string()
                };

                matches.push(Match {
                    span: token.token.span.clone(),
                    rule_id: "EN_L2_FR_CONFUSION".to_string(),
                    message: self.generate_message(word, &suggestion),
                    suggestions: vec![suggestion],
                    severity: Severity::Hint,
                });
            }
        }

        CheckResult { matches }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokenizer::SimpleTokenizer;
    use crate::analyzer::PassthroughAnalyzer;
    use crate::core::traits::{Tokenizer, Analyzer};

    fn check_text(text: &str) -> CheckResult {
        let checker = L2ConfusionChecker::new();
        let tokenizer = SimpleTokenizer::new();
        let analyzer = PassthroughAnalyzer::new();
        let tokens = tokenizer.tokenize(text);
        let analyzed = analyzer.analyze(tokens);
        checker.check(text, &analyzed)
    }

    fn check_text_with_factor(text: &str, min_factor: u64) -> CheckResult {
        let checker = L2ConfusionChecker::new().with_min_factor(min_factor);
        let tokenizer = SimpleTokenizer::new();
        let analyzer = PassthroughAnalyzer::new();
        let tokens = tokenizer.tokenize(text);
        let analyzed = analyzer.analyze(tokens);
        checker.check(text, &analyzed)
    }

    #[test]
    fn test_lecture_reading() {
        // FR "lecture" = reading, not a lecture
        let result = check_text("I need to finish my lecture.");
        assert!(!result.matches.is_empty(), "Should flag 'lecture'");
        assert!(result.matches[0].suggestions.contains(&"reading".to_string()));
        assert_eq!(result.matches[0].rule_id, "EN_L2_FR_CONFUSION");
    }

    #[test]
    fn test_fabric_factory() {
        // FR "fabrique" = factory, not fabric
        let result = check_text("The fabric produces cars.");
        assert!(!result.matches.is_empty(), "Should flag 'fabric'");
        assert!(result.matches[0].suggestions.contains(&"factory".to_string()));
    }

    #[test]
    fn test_abilities_skills() {
        // FR "habilités" ≠ abilities
        let result = check_text("He has great abilities.");
        assert!(!result.matches.is_empty(), "Should flag 'abilities'");
        assert!(result.matches[0].suggestions.contains(&"skills".to_string()));
    }

    #[test]
    fn test_pretend_claim() {
        // FR "prétendre" = to claim, not pretend
        let result = check_text("I pretend that this is true.");
        assert!(!result.matches.is_empty(), "Should flag 'pretend'");
        assert!(result.matches[0].suggestions.contains(&"claim".to_string()));
    }

    #[test]
    fn test_no_false_positives() {
        // Normal English text should have no matches
        let result = check_text("The quick brown fox jumps over the lazy dog.");
        assert!(result.matches.is_empty(), "Should have no false positives");
    }

    #[test]
    fn test_case_preservation() {
        // Capitalized word should have capitalized suggestion
        let result = check_text("Lecture is important.");
        assert!(!result.matches.is_empty());
        assert!(result.matches[0].suggestions.contains(&"Reading".to_string()));
    }

    #[test]
    fn test_min_factor_threshold() {
        // "lecture" has factor 1000000, should be flagged with threshold 100
        let result = check_text_with_factor("I need a lecture.", 100);
        assert!(!result.matches.is_empty());

        // With very high threshold, low-factor words should be skipped
        let result_high = check_text_with_factor("I achieve my goals.", 1000000);
        // "achieve" has factor 10, should be skipped
        let achieve_matches: Vec<_> = result_high.matches.iter()
            .filter(|m| m.message.contains("achieve"))
            .collect();
        assert!(achieve_matches.is_empty(), "Low-factor word should be skipped");
    }

    #[test]
    fn test_multiple_matches() {
        let result = check_text("My lecture about fabric design.");
        assert!(result.matches.len() >= 2, "Should find multiple false friends");
    }

    #[test]
    fn test_short_words_skipped() {
        // Very short words should be skipped
        let checker = L2ConfusionChecker::new().with_min_word_length(10);
        let tokenizer = SimpleTokenizer::new();
        let analyzer = PassthroughAnalyzer::new();
        let tokens = tokenizer.tokenize("fabric");
        let analyzed = analyzer.analyze(tokens);
        let result = checker.check("fabric", &analyzed);
        assert!(result.matches.is_empty(), "Short word should be skipped with high min_length");
    }
}
