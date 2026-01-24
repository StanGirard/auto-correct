//! Coherency checker for spelling consistency within a document
//!
//! This module detects when different spelling variants of the same word
//! are used within the same document (e.g., "colour" and "color", or
//! "analyse" and "analyze").

use std::collections::HashMap;

use crate::checker::data::en_coherency::{get_en_coherency_pair, get_en_coherency_variants};
use crate::core::traits::Checker;
use crate::core::{AnalyzedToken, CheckResult, Match, Severity, TokenKind};

/// Coherency checker that detects inconsistent spelling variants.
///
/// This checker tracks which spelling variants are used in a document
/// and reports when different variants of the same word are used.
///
/// # Example
/// ```ignore
/// let mut checker = CoherencyChecker::new();
/// // In a document with both "colour" and "color", the second usage
/// // will be flagged as inconsistent with the first.
/// ```
pub struct CoherencyChecker {
    /// Maps pair_id -> (first_variant_used, first_position)
    used_variants: HashMap<usize, (String, usize)>,
}

impl CoherencyChecker {
    /// Create a new CoherencyChecker.
    pub fn new() -> Self {
        Self {
            used_variants: HashMap::new(),
        }
    }

    /// Reset the checker for a new document.
    pub fn reset(&mut self) {
        self.used_variants.clear();
    }

    /// Check a single word for coherency issues.
    /// Returns Some(Match) if this word is inconsistent with earlier usage.
    fn check_word(&mut self, word: &str, position: usize) -> Option<Match> {
        let word_lower = word.to_lowercase();

        // Check if this word is part of a coherency pair
        let pair_id = get_en_coherency_pair(&word_lower)?;

        // Check if we've already seen a variant from this pair
        if let Some((first_variant, _first_pos)) = self.used_variants.get(&pair_id) {
            // If it's the same variant, no problem
            if first_variant == &word_lower {
                return None;
            }

            // Different variant - this is an inconsistency
            let suggestion = first_variant.clone();

            return Some(Match {
                span: position..position + word.len(),
                rule_id: format!("COHERENCY_{}", pair_id),
                message: format!(
                    "Inconsistent spelling: '{}' was used earlier. Consider using '{}' for consistency.",
                    first_variant, first_variant
                ),
                suggestions: vec![suggestion],
                severity: Severity::Hint,
            });
        }

        // First time seeing this pair - record the variant used
        self.used_variants
            .insert(pair_id, (word_lower, position));

        None
    }
}

impl Default for CoherencyChecker {
    fn default() -> Self {
        Self::new()
    }
}

impl Checker for CoherencyChecker {
    fn check(&self, _text: &str, tokens: &[AnalyzedToken]) -> CheckResult {
        // Note: This implementation creates a temporary mutable state
        // For true stateful checking across multiple check() calls,
        // use check_document() instead
        let mut state = HashMap::new();
        let mut matches = Vec::new();

        for token in tokens {
            if token.token.kind != TokenKind::Word {
                continue;
            }

            let word = &token.token.text;
            let word_lower = word.to_lowercase();

            // Check if this word is part of a coherency pair
            if let Some(pair_id) = get_en_coherency_pair(&word_lower) {
                // Check if we've already seen a variant from this pair
                if let Some((first_variant, _first_pos)) = state.get(&pair_id) {
                    let first_variant: &String = first_variant;
                    // If it's the same variant, no problem
                    if first_variant == &word_lower {
                        continue;
                    }

                    // Different variant - this is an inconsistency
                    matches.push(Match {
                        span: token.token.span.clone(),
                        rule_id: format!("COHERENCY_{}", pair_id),
                        message: format!(
                            "Inconsistent spelling: '{}' was used earlier. Consider using '{}' for consistency.",
                            first_variant, first_variant
                        ),
                        suggestions: vec![first_variant.clone()],
                        severity: Severity::Hint,
                    });
                } else {
                    // First time seeing this pair - record the variant used
                    state.insert(pair_id, (word_lower, token.token.span.start));
                }
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
    fn test_coherency_detection() {
        let checker = CoherencyChecker::new();

        // Use both UK and US spelling in the same text
        let text = "I need to analyse this data. Let me analyze the results.";
        let tokens = tokenize(text);

        let result = checker.check(text, &tokens);

        // Should detect the inconsistency
        assert!(
            !result.matches.is_empty(),
            "Should detect analyse/analyze inconsistency"
        );

        let m = &result.matches[0];
        assert!(m.rule_id.starts_with("COHERENCY_"));
        assert!(m.message.contains("analyse"));
    }

    #[test]
    fn test_consistent_usage() {
        let checker = CoherencyChecker::new();

        // Consistently use UK spelling
        let text = "I need to analyse this data. I will analyse the results.";
        let tokens = tokenize(text);

        let result = checker.check(text, &tokens);

        // No inconsistency
        assert!(
            result.matches.is_empty(),
            "Should not flag consistent usage"
        );
    }

    #[test]
    fn test_ize_ise_variants() {
        let checker = CoherencyChecker::new();

        // Mix -ize and -ise
        let text = "We need to organize the event. Let's also organise the food.";
        let tokens = tokenize(text);

        let result = checker.check(text, &tokens);

        // Should detect the inconsistency
        assert!(
            !result.matches.is_empty(),
            "Should detect organize/organise inconsistency"
        );
    }

    #[test]
    fn test_multiple_pairs() {
        let checker = CoherencyChecker::new();

        // Multiple coherency pairs, all consistent
        let text = "We should organise the meeting. The organisation will handle it.";
        let tokens = tokenize(text);

        let result = checker.check(text, &tokens);

        // No inconsistencies (each pair is consistent within itself)
        // Note: organise and organisation are different pairs
        assert!(
            result.matches.is_empty(),
            "Each pair should be internally consistent"
        );
    }

    #[test]
    fn test_suggestion_matches_first_usage() {
        let checker = CoherencyChecker::new();

        // First use US, then UK
        let text = "I organize the files. Then I organise the folders.";
        let tokens = tokenize(text);

        let result = checker.check(text, &tokens);

        assert!(!result.matches.is_empty(), "Should detect inconsistency");
        let m = &result.matches[0];

        // Suggestion should be the first variant used
        assert!(
            m.suggestions.contains(&"organize".to_string()),
            "Suggestion should match first usage: {:?}",
            m.suggestions
        );
    }

    #[test]
    fn test_no_false_positives() {
        let checker = CoherencyChecker::new();

        // Text without any coherency pairs
        let text = "The quick brown fox jumps over the lazy dog.";
        let tokens = tokenize(text);

        let result = checker.check(text, &tokens);

        assert!(result.matches.is_empty(), "Should have no false positives");
    }

    #[test]
    fn test_case_insensitive() {
        let checker = CoherencyChecker::new();

        // Same word, different case
        let text = "I need to Analyse the data. Then I will analyze the results.";
        let tokens = tokenize(text);

        let result = checker.check(text, &tokens);

        // Should detect the inconsistency regardless of case
        assert!(
            !result.matches.is_empty(),
            "Should detect inconsistency case-insensitively"
        );
    }
}
