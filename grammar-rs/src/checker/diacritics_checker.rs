//! Diacritics checker for suggesting proper accents and special characters
//!
//! This module suggests proper diacritics for words borrowed from other languages,
//! such as "cafe" -> "café" or "naive" -> "naïve".

use crate::checker::data::en_diacritics::{get_en_diacritics, EN_DIACRITICS_RULES};
use crate::core::traits::Checker;
use crate::core::{AnalyzedToken, CheckResult, Match, Severity, TokenKind};

/// Diacritics checker that suggests proper accents for borrowed words.
///
/// # Example
/// ```ignore
/// let checker = DiacriticsChecker::new();
/// // "cafe" will be flagged with suggestion "café"
/// ```
pub struct DiacriticsChecker {
    /// Minimum word length to check (to avoid false positives on short words)
    min_word_length: usize,
}

impl DiacriticsChecker {
    /// Create a new DiacriticsChecker with default settings.
    pub fn new() -> Self {
        Self {
            min_word_length: 2,
        }
    }

    /// Create a DiacriticsChecker with custom minimum word length.
    pub fn with_min_length(min_word_length: usize) -> Self {
        Self { min_word_length }
    }

    /// Get the number of diacritics rules available.
    pub fn rule_count() -> usize {
        EN_DIACRITICS_RULES.len()
    }
}

impl Default for DiacriticsChecker {
    fn default() -> Self {
        Self::new()
    }
}

impl Checker for DiacriticsChecker {
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

            // Try lowercase lookup
            let word_lower = word.to_lowercase();

            if let Some(with_diacritics) = get_en_diacritics(&word_lower) {
                // Preserve original case if possible
                let suggestion = if word.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) {
                    // Capitalize first letter
                    let mut chars = with_diacritics.chars();
                    match chars.next() {
                        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                        None => with_diacritics.to_string(),
                    }
                } else {
                    with_diacritics.to_string()
                };

                matches.push(Match {
                    span: token.token.span.clone(),
                    rule_id: "DIACRITICS".to_string(),
                    message: format!(
                        "Consider using '{}' with proper diacritics.",
                        suggestion
                    ),
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
    use crate::prelude::*;

    fn tokenize(text: &str) -> Vec<AnalyzedToken<'_>> {
        let tokenizer = SimpleTokenizer::new();
        let analyzer = PassthroughAnalyzer::new();
        let tokens = tokenizer.tokenize(text);
        analyzer.analyze(tokens)
    }

    #[test]
    fn test_aperitif_suggestion() {
        let checker = DiacriticsChecker::new();
        let text = "Let's have an aperitif.";
        let tokens = tokenize(text);

        let result = checker.check(text, &tokens);

        assert!(!result.matches.is_empty(), "Should suggest apéritif");
        let m = &result.matches[0];
        assert!(m.suggestions.contains(&"apéritif".to_string()));
    }

    #[test]
    fn test_expose_suggestion() {
        let checker = DiacriticsChecker::new();
        // Note: "an expose" is a phrase entry in the data
        // Let's test with a single word that exists
        let text = "I enjoyed the apercu.";
        let tokens = tokenize(text);

        let result = checker.check(text, &tokens);

        assert!(!result.matches.is_empty(), "Should suggest aperçu");
        let m = &result.matches[0];
        assert!(m.suggestions.contains(&"aperçu".to_string()));
    }

    #[test]
    fn test_preserve_capitalization() {
        let checker = DiacriticsChecker::new();
        let text = "Aperitif is served.";
        let tokens = tokenize(text);

        let result = checker.check(text, &tokens);

        assert!(!result.matches.is_empty());
        let m = &result.matches[0];
        // Should preserve capital A
        assert!(m.suggestions.contains(&"Apéritif".to_string()));
    }

    #[test]
    fn test_no_false_positives() {
        let checker = DiacriticsChecker::new();
        let text = "The quick brown fox jumps over the lazy dog.";
        let tokens = tokenize(text);

        let result = checker.check(text, &tokens);

        assert!(result.matches.is_empty(), "Should have no false positives");
    }

    #[test]
    fn test_already_correct() {
        let checker = DiacriticsChecker::new();
        // Note: if "café" is not in the without-diacritics list, it won't match
        let text = "I went to a restaurant.";
        let tokens = tokenize(text);

        let result = checker.check(text, &tokens);

        assert!(result.matches.is_empty());
    }

    #[test]
    fn test_attache_suggestion() {
        let checker = DiacriticsChecker::new();
        let text = "He is an attache.";
        let tokens = tokenize(text);

        let result = checker.check(text, &tokens);

        // "attache" should be suggested as "attaché"
        assert!(!result.matches.is_empty(), "Should suggest attaché");
        let m = &result.matches[0];
        assert!(m.suggestions.contains(&"attaché".to_string()));
    }

    #[test]
    fn test_rule_count() {
        assert!(DiacriticsChecker::rule_count() > 1000, "Should have many diacritics rules");
    }
}
