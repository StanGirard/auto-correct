//! Simple replacement rules from LanguageTool replace.txt
//!
//! This module implements a checker that looks up individual words
//! in a replacement table. Rules are imported from LanguageTool's
//! replace.txt via the sync-lt tool.

use crate::core::traits::Checker;
use crate::core::{AnalyzedToken, CheckResult, Match, Severity, TokenKind};

/// Checker that looks up words in a replacement table from LanguageTool replace.txt
pub struct ReplaceRuleChecker {
    /// Sorted array of (wrong, correct) pairs for binary search
    replacements: &'static [(&'static str, &'static str)],
    /// Rule ID prefix for generated matches
    rule_prefix: &'static str,
}

impl ReplaceRuleChecker {
    /// Create a new ReplaceRuleChecker with the given replacement table
    pub fn new(replacements: &'static [(&'static str, &'static str)], rule_prefix: &'static str) -> Self {
        ReplaceRuleChecker {
            replacements,
            rule_prefix,
        }
    }

    /// Create checker with English replacement rules
    #[cfg(feature = "en_replace")]
    pub fn english() -> Self {
        use crate::checker::data::EN_REPLACE_RULES;
        ReplaceRuleChecker::new(EN_REPLACE_RULES, "EN_REPLACE")
    }

    /// Create checker with French replacement rules
    #[cfg(feature = "fr_replace")]
    pub fn french() -> Self {
        use crate::checker::data::FR_REPLACE_RULES;
        ReplaceRuleChecker::new(FR_REPLACE_RULES, "FR_REPLACE")
    }

    /// Look up a word in the replacement table using binary search
    fn get_replacement(&self, word: &str) -> Option<&'static str> {
        self.replacements
            .binary_search_by_key(&word, |(w, _)| *w)
            .ok()
            .map(|idx| self.replacements[idx].1)
    }
}

impl Checker for ReplaceRuleChecker {
    fn check(&self, _text: &str, tokens: &[AnalyzedToken]) -> CheckResult {
        let mut matches = Vec::new();

        for token in tokens {
            if token.token.kind != TokenKind::Word {
                continue;
            }

            let word_lower = token.token.text.to_lowercase();

            if let Some(replacement) = self.get_replacement(&word_lower) {
                // Preserve original casing if the word was capitalized
                let suggestion = if token.token.text.chars().next().map(|c| c.is_uppercase()).unwrap_or(false)
                    && replacement.chars().next().map(|c| c.is_lowercase()).unwrap_or(false)
                {
                    // Capitalize the replacement
                    let mut chars = replacement.chars();
                    match chars.next() {
                        None => replacement.to_string(),
                        Some(first) => first.to_uppercase().chain(chars).collect(),
                    }
                } else {
                    replacement.to_string()
                };

                matches.push(Match {
                    span: token.token.span.clone(),
                    rule_id: format!("{}_{}", self.rule_prefix, word_lower.to_uppercase()),
                    message: format!("'{}' might be incorrect. Did you mean '{}'?", token.token.text, suggestion),
                    suggestions: vec![suggestion],
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
    use crate::core::Token;
    use std::ops::Range;

    fn make_token<'a>(text: &'a str, span: Range<usize>) -> AnalyzedToken<'a> {
        AnalyzedToken {
            token: Token {
                text,
                span,
                kind: TokenKind::Word,
            },
            lemma: None,
            pos: None,
        }
    }

    static TEST_REPLACEMENTS: &[(&str, &str)] = &[
        ("alot", "a lot"),
        ("becuase", "because"),
        ("definately", "definitely"),
        ("recieve", "receive"),
        ("seperate", "separate"),
        ("teh", "the"),
    ];

    #[test]
    fn test_replacement_found() {
        let checker = ReplaceRuleChecker::new(TEST_REPLACEMENTS, "TEST");

        let tokens = vec![
            make_token("I", 0..1),
            make_token("recieve", 2..9),
            make_token("alot", 10..14),
        ];

        let result = checker.check("I recieve alot", &tokens);
        assert_eq!(result.matches.len(), 2);

        // Check recieve -> receive
        assert_eq!(result.matches[0].span, 2..9);
        assert_eq!(result.matches[0].suggestions, vec!["receive"]);

        // Check alot -> a lot
        assert_eq!(result.matches[1].span, 10..14);
        assert_eq!(result.matches[1].suggestions, vec!["a lot"]);
    }

    #[test]
    fn test_case_preservation() {
        let checker = ReplaceRuleChecker::new(TEST_REPLACEMENTS, "TEST");

        let tokens = vec![
            make_token("Teh", 0..3),
            make_token("cat", 4..7),
        ];

        let result = checker.check("Teh cat", &tokens);
        assert_eq!(result.matches.len(), 1);
        assert_eq!(result.matches[0].suggestions, vec!["The"]);
    }

    #[test]
    fn test_no_replacement() {
        let checker = ReplaceRuleChecker::new(TEST_REPLACEMENTS, "TEST");

        let tokens = vec![
            make_token("The", 0..3),
            make_token("correct", 4..11),
            make_token("spelling", 12..20),
        ];

        let result = checker.check("The correct spelling", &tokens);
        assert!(result.matches.is_empty());
    }

    #[test]
    fn test_binary_search() {
        let checker = ReplaceRuleChecker::new(TEST_REPLACEMENTS, "TEST");

        // Test that binary search works correctly
        assert_eq!(checker.get_replacement("alot"), Some("a lot"));
        assert_eq!(checker.get_replacement("definately"), Some("definitely"));
        assert_eq!(checker.get_replacement("teh"), Some("the"));
        assert_eq!(checker.get_replacement("nonexistent"), None);
    }
}
