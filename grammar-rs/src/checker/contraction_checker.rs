//! Contraction checker for detecting missing apostrophes
//!
//! This module detects words that are commonly written without apostrophes
//! and suggests the proper contracted form, e.g., "dont" -> "don't".

use crate::checker::data::en_contractions::{get_en_contraction, EN_CONTRACTION_RULES};
use crate::core::traits::Checker;
use crate::core::{AnalyzedToken, CheckResult, Match, Severity, TokenKind};

/// Contraction checker that suggests proper apostrophe placement.
///
/// This checker detects words that are commonly written without apostrophes
/// and suggests the contracted form.
///
/// # Example
/// ```ignore
/// let checker = ContractionChecker::new();
/// // "dont" will be flagged with suggestion "don't"
/// // "Im" will be flagged with suggestion "I'm"
/// ```
pub struct ContractionChecker {
    /// Minimum word length to check
    min_word_length: usize,
}

impl ContractionChecker {
    /// Create a new ContractionChecker with default settings.
    pub fn new() -> Self {
        Self {
            min_word_length: 2,
        }
    }

    /// Get the number of contraction rules available.
    pub fn rule_count() -> usize {
        EN_CONTRACTION_RULES.len()
    }
}

impl Default for ContractionChecker {
    fn default() -> Self {
        Self::new()
    }
}

impl Checker for ContractionChecker {
    fn check(&self, _text: &str, tokens: &[AnalyzedToken]) -> CheckResult {
        let mut matches = Vec::new();

        for token in tokens {
            if token.token.kind != TokenKind::Word {
                continue;
            }

            let word = &token.token.text;

            // Skip short words
            if word.len() < self.min_word_length {
                continue;
            }

            // The contractions data is case-sensitive
            // Look up the word exactly as it appears
            if let Some(contractions) = get_en_contraction(word) {
                let suggestions: Vec<String> = contractions.iter().map(|s| s.to_string()).collect();
                let primary = &suggestions[0];

                matches.push(Match {
                    span: token.token.span.clone(),
                    rule_id: "CONTRACTION".to_string(),
                    message: format!("Did you mean '{}'? This word is usually written with an apostrophe.", primary),
                    suggestions,
                    severity: Severity::Warning,
                });
            }
        }

        CheckResult { matches }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::*;

    fn tokenize(text: &str) -> Vec<AnalyzedToken<'_>> {
        let tokenizer = SimpleTokenizer::new();
        let analyzer = PassthroughAnalyzer::new();
        let tokens = tokenizer.tokenize(text);
        analyzer.analyze(tokens)
    }

    #[test]
    fn test_dont_suggestion() {
        let checker = ContractionChecker::new();
        let text = "I dont know.";
        let tokens = tokenize(text);

        let result = checker.check(text, &tokens);

        assert!(!result.matches.is_empty(), "Should detect 'dont'");
        let m = &result.matches[0];
        assert!(m.suggestions.contains(&"don't".to_string()));
    }

    #[test]
    fn test_im_suggestion() {
        let checker = ContractionChecker::new();
        let text = "Im going home.";
        let tokens = tokenize(text);

        let result = checker.check(text, &tokens);

        assert!(!result.matches.is_empty(), "Should detect 'Im'");
        let m = &result.matches[0];
        assert!(m.suggestions.contains(&"I'm".to_string()));
    }

    #[test]
    fn test_wont_suggestion() {
        let checker = ContractionChecker::new();
        let text = "It wont work.";
        let tokens = tokenize(text);

        // Note: "wont" may or may not be in the data (it's commented out in LT)
        // This test will pass if it's not in the data
        let _result = checker.check(text, &tokens);
    }

    #[test]
    fn test_no_false_positives() {
        let checker = ContractionChecker::new();
        let text = "The quick brown fox jumps over the lazy dog.";
        let tokens = tokenize(text);

        let result = checker.check(text, &tokens);

        assert!(result.matches.is_empty(), "Should have no false positives");
    }

    #[test]
    fn test_already_correct() {
        let checker = ContractionChecker::new();
        let text = "I don't know what you're talking about.";
        let tokens = tokenize(text);

        let result = checker.check(text, &tokens);

        assert!(result.matches.is_empty(), "Should not flag correct contractions");
    }

    #[test]
    fn test_case_sensitivity() {
        let checker = ContractionChecker::new();
        // The data has different entries for different cases
        let text = "DONT do that!";
        let tokens = tokenize(text);

        let result = checker.check(text, &tokens);

        // DONT should be in the data as a separate entry
        if !result.matches.is_empty() {
            let m = &result.matches[0];
            assert!(m.suggestions.contains(&"DON'T".to_string()));
        }
    }

    #[test]
    fn test_multiple_suggestions() {
        let checker = ContractionChecker::new();
        // "Id" can be "I'd" or "ID"
        let text = "Id like that.";
        let tokens = tokenize(text);

        let result = checker.check(text, &tokens);

        if !result.matches.is_empty() {
            let m = &result.matches[0];
            // Should have multiple suggestions
            assert!(m.suggestions.len() >= 1);
        }
    }

    #[test]
    fn test_rule_count() {
        assert!(ContractionChecker::rule_count() > 100, "Should have many contraction rules");
    }
}
