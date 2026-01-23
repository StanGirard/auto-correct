//! Pattern-based rule checking from LanguageTool grammar.xml
//!
//! This module implements a checker that matches multi-word patterns
//! against the text. Rules are imported from LanguageTool's grammar.xml
//! via the sync-lt tool.
//!
//! Two implementations are provided:
//! - `PatternRuleChecker`: Simple O(rules × tokens) implementation
//! - `AhoPatternRuleChecker`: Optimized O(tokens) using Aho-Corasick
//!
//! Antipattern support: Rules can have antipatterns - exceptions that prevent
//! the rule from firing when matched. For example, "a one-time event" matches
//! an antipattern for the A_AN rule, so it won't incorrectly suggest "an".

use aho_corasick::AhoCorasick;
use regex::Regex;
use std::collections::HashMap;
use std::sync::LazyLock;

use crate::core::traits::Checker;
use crate::core::{AnalyzedToken, CheckResult, Match, Severity, TokenKind};

use crate::checker::data::{Antipattern, AntipatternToken};

/// A pattern rule imported from LanguageTool
#[derive(Debug, Clone)]
pub struct PatternRule {
    pub id: &'static str,
    pub pattern: &'static [&'static str],
    pub suggestion: &'static str,
    pub message: &'static str,
}

/// Checker that matches multi-word patterns from LanguageTool grammar.xml
pub struct PatternRuleChecker {
    rules: &'static [PatternRule],
}

impl PatternRuleChecker {
    /// Create a new PatternRuleChecker with the given rules
    pub fn new(rules: &'static [PatternRule]) -> Self {
        PatternRuleChecker { rules }
    }

    /// Create checker with English pattern rules
    #[cfg(feature = "en_patterns")]
    pub fn english() -> Self {
        use crate::checker::data::EN_PATTERN_RULES;
        PatternRuleChecker::new(EN_PATTERN_RULES)
    }

    /// Create checker with French pattern rules
    #[cfg(feature = "fr_patterns")]
    pub fn french() -> Self {
        use crate::checker::data::FR_PATTERN_RULES;
        PatternRuleChecker::new(FR_PATTERN_RULES)
    }

    /// Try to match a pattern starting at the given position
    fn find_pattern_match<'a>(
        &self,
        words: &[&AnalyzedToken<'a>],
        start_idx: usize,
        pattern: &[&str],
    ) -> Option<std::ops::Range<usize>> {
        if start_idx + pattern.len() > words.len() {
            return None;
        }

        for (i, pat_word) in pattern.iter().enumerate() {
            let token = words[start_idx + i];
            let token_text = token.token.text.to_lowercase();

            if token_text != *pat_word {
                return None;
            }
        }

        // Calculate the span from the first to the last matched token
        let first = words[start_idx];
        let last = words[start_idx + pattern.len() - 1];
        Some(first.token.span.start..last.token.span.end)
    }

    /// Generate a suggestion by replacing the matched pattern
    fn generate_suggestion(&self, rule: &PatternRule, matched_text: &str) -> String {
        if rule.suggestion.is_empty() {
            // No specific suggestion, just return the message
            matched_text.to_string()
        } else {
            rule.suggestion.to_string()
        }
    }
}

impl Checker for PatternRuleChecker {
    fn check(&self, text: &str, tokens: &[AnalyzedToken]) -> CheckResult {
        let mut matches = Vec::new();

        // Filter to only word tokens
        let words: Vec<_> = tokens
            .iter()
            .filter(|t| t.token.kind == TokenKind::Word)
            .collect();

        // Try each rule
        for rule in self.rules {
            if rule.pattern.is_empty() {
                continue;
            }

            // Try to match at each position
            for start in 0..words.len() {
                if let Some(span) = self.find_pattern_match(&words, start, rule.pattern) {
                    let matched_text = &text[span.clone()];
                    let suggestion = self.generate_suggestion(rule, matched_text);

                    matches.push(Match {
                        span,
                        rule_id: rule.id.to_string(),
                        message: rule.message.to_string(),
                        suggestions: if suggestion.is_empty() {
                            vec![]
                        } else {
                            vec![suggestion]
                        },
                        severity: Severity::Warning,
                    });
                }
            }
        }

        CheckResult { matches }
    }
}

/// Optimized pattern rule checker using Aho-Corasick algorithm.
///
/// This checker builds an automaton from the first words of all patterns,
/// allowing O(N) scanning of text where N is the text length, instead of
/// O(R × N) where R is the number of rules.
///
/// The algorithm:
/// 1. Build an AC automaton with the first word of each pattern
/// 2. Scan the text once to find candidate positions
/// 3. Verify full patterns only at matched positions
/// 4. Check antipatterns to filter false positives
pub struct AhoPatternRuleChecker {
    /// Aho-Corasick automaton for first words
    first_word_ac: AhoCorasick,
    /// Maps AC pattern index -> list of rule indices that start with that word
    first_word_to_rules: Vec<Vec<usize>>,
    /// All rules for full pattern verification
    rules: &'static [PatternRule],
    /// Maps first word (lowercase) -> index in first_words vec
    #[allow(dead_code)]
    word_to_idx: HashMap<String, usize>,
    /// Antipatterns indexed by rule ID for fast lookup
    antipatterns_by_rule: HashMap<String, Vec<&'static Antipattern>>,
}

impl AhoPatternRuleChecker {
    /// Create a new AhoPatternRuleChecker with the given rules.
    ///
    /// This builds the Aho-Corasick automaton from the first words of all patterns.
    /// The construction is O(R × W) where R is rules and W is average word length.
    pub fn new(rules: &'static [PatternRule]) -> Self {
        Self::with_antipatterns(rules, &[])
    }

    /// Create a new AhoPatternRuleChecker with rules and antipatterns.
    ///
    /// Antipatterns are exceptions that prevent rules from firing in specific contexts.
    pub fn with_antipatterns(
        rules: &'static [PatternRule],
        antipatterns: &'static [Antipattern],
    ) -> Self {
        let mut first_words: Vec<String> = Vec::new();
        let mut word_to_idx: HashMap<String, usize> = HashMap::new();
        let mut first_word_to_rules: Vec<Vec<usize>> = Vec::new();

        for (rule_idx, rule) in rules.iter().enumerate() {
            if rule.pattern.is_empty() {
                continue;
            }

            let first = rule.pattern[0].to_lowercase();
            let fw_idx = *word_to_idx.entry(first.clone()).or_insert_with(|| {
                let idx = first_words.len();
                first_words.push(first);
                first_word_to_rules.push(Vec::new());
                idx
            });

            first_word_to_rules[fw_idx].push(rule_idx);
        }

        let first_word_ac = AhoCorasick::builder()
            .ascii_case_insensitive(true)
            .build(&first_words)
            .unwrap();

        // Build antipattern lookup by rule ID
        let mut antipatterns_by_rule: HashMap<String, Vec<&'static Antipattern>> = HashMap::new();
        for ap in antipatterns {
            antipatterns_by_rule
                .entry(ap.rule_id.to_string())
                .or_default()
                .push(ap);
        }

        Self {
            first_word_ac,
            first_word_to_rules,
            rules,
            word_to_idx,
            antipatterns_by_rule,
        }
    }

    /// Verify that a full pattern matches starting at the given token index.
    ///
    /// Returns the byte span of the match if successful.
    fn verify_full_pattern(
        &self,
        words: &[&AnalyzedToken],
        start_idx: usize,
        rule_idx: usize,
    ) -> Option<std::ops::Range<usize>> {
        let rule = &self.rules[rule_idx];
        let pattern = rule.pattern;

        // Check if we have enough tokens
        if start_idx + pattern.len() > words.len() {
            return None;
        }

        // Verify each word in the pattern
        for (i, pat_word) in pattern.iter().enumerate() {
            let token = words[start_idx + i];
            if !token.token.text.eq_ignore_ascii_case(pat_word) {
                return None;
            }
        }

        // Calculate the span from first to last matched token
        let first = words[start_idx];
        let last = words[start_idx + pattern.len() - 1];
        Some(first.token.span.start..last.token.span.end)
    }

    /// Generate a suggestion for a matched rule
    fn generate_suggestion(&self, rule: &PatternRule, matched_text: &str) -> String {
        if rule.suggestion.is_empty() {
            matched_text.to_string()
        } else {
            rule.suggestion.to_string()
        }
    }

    /// Check if any antipattern matches at the given context.
    ///
    /// Returns true if an antipattern matches, meaning the rule should NOT fire.
    fn matches_antipattern(
        &self,
        words: &[&AnalyzedToken],
        match_start_idx: usize,
        match_len: usize,
        rule_id: &str,
    ) -> bool {
        // Get antipatterns for this rule
        let Some(antipatterns) = self.antipatterns_by_rule.get(rule_id) else {
            return false;
        };

        // Check each antipattern
        for ap in antipatterns {
            if self.check_single_antipattern(words, match_start_idx, match_len, ap) {
                return true;
            }
        }

        false
    }

    /// Check if a single antipattern matches at the context around a match.
    ///
    /// Antipatterns can overlap with the match position. We check all possible
    /// alignments where the antipattern could overlap with the matched text.
    fn check_single_antipattern(
        &self,
        words: &[&AnalyzedToken],
        match_start_idx: usize,
        match_len: usize,
        antipattern: &Antipattern,
    ) -> bool {
        let ap_len = antipattern.tokens.len();
        if ap_len == 0 {
            return false;
        }

        // Try all starting positions where antipattern could overlap with match
        // The antipattern could start before, at, or after the match start
        let earliest_start = match_start_idx.saturating_sub(ap_len - 1);
        let latest_start = match_start_idx + match_len - 1;

        for start in earliest_start..=latest_start {
            if start + ap_len > words.len() {
                continue;
            }

            if self.antipattern_matches_at(words, start, antipattern) {
                return true;
            }
        }

        false
    }

    /// Check if an antipattern matches exactly at the given position.
    fn antipattern_matches_at(
        &self,
        words: &[&AnalyzedToken],
        start: usize,
        antipattern: &Antipattern,
    ) -> bool {
        for (i, ap_token) in antipattern.tokens.iter().enumerate() {
            let word = &words[start + i];
            let word_text = word.token.text.to_lowercase();

            // Check text match
            if let Some(expected_text) = &ap_token.text {
                let expected_lower = expected_text.to_lowercase();

                // Handle regexp in text field (yes attribute)
                if ap_token.regexp.as_deref() == Some("yes") {
                    // Try to compile and match regex
                    if let Ok(re) = Regex::new(&format!("(?i)^{}$", expected_lower)) {
                        if !re.is_match(&word_text) {
                            return false;
                        }
                    } else {
                        // Fallback to exact match if regex fails
                        if word_text != expected_lower {
                            return false;
                        }
                    }
                } else {
                    // Exact match (case-insensitive)
                    if word_text != expected_lower {
                        return false;
                    }
                }
            } else {
                // No text means match any word (typically with postag constraint)
                // Since we don't have full POS support yet, accept any word
            }
        }

        true
    }
}

impl Checker for AhoPatternRuleChecker {
    fn check(&self, text: &str, tokens: &[AnalyzedToken]) -> CheckResult {
        let mut matches = Vec::new();

        // Filter to only word tokens
        let words: Vec<_> = tokens
            .iter()
            .filter(|t| t.token.kind == TokenKind::Word)
            .collect();

        if words.is_empty() {
            return CheckResult { matches };
        }

        // Build map: byte offset -> token index
        let offset_to_idx: HashMap<usize, usize> = words
            .iter()
            .enumerate()
            .map(|(i, w)| (w.token.span.start, i))
            .collect();

        // Scan text with Aho-Corasick
        for ac_match in self.first_word_ac.find_iter(text) {
            // Check if this match starts at a token boundary
            if let Some(&token_idx) = offset_to_idx.get(&ac_match.start()) {
                let token = words[token_idx];

                // Verify word boundary (AC might match substrings)
                let matched_text = &text[ac_match.start()..ac_match.end()];
                if !token.token.text.eq_ignore_ascii_case(matched_text) {
                    continue;
                }

                // Check each rule that starts with this word
                for &rule_idx in &self.first_word_to_rules[ac_match.pattern().as_usize()] {
                    if let Some(span) = self.verify_full_pattern(&words, token_idx, rule_idx) {
                        let rule = &self.rules[rule_idx];
                        let pattern_len = rule.pattern.len();

                        // Check antipatterns - skip if any antipattern matches
                        if self.matches_antipattern(&words, token_idx, pattern_len, rule.id) {
                            continue;
                        }

                        let matched_text = &text[span.clone()];
                        let suggestion = self.generate_suggestion(rule, matched_text);

                        matches.push(Match {
                            span,
                            rule_id: rule.id.to_string(),
                            message: rule.message.to_string(),
                            suggestions: if suggestion.is_empty() {
                                vec![]
                            } else {
                                vec![suggestion]
                            },
                            severity: Severity::Warning,
                        });
                    }
                }
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

    #[test]
    fn test_pattern_match() {
        static TEST_RULES: &[PatternRule] = &[PatternRule {
            id: "TEST_RULE",
            pattern: &["could", "of"],
            suggestion: "could have",
            message: "Did you mean 'could have'?",
        }];

        let checker = PatternRuleChecker::new(TEST_RULES);

        // Create tokens for "I could of done that"
        let tokens = vec![
            make_token("I", 0..1),
            make_token("could", 2..7),
            make_token("of", 8..10),
            make_token("done", 11..15),
            make_token("that", 16..20),
        ];

        let result = checker.check("I could of done that", &tokens);
        assert_eq!(result.matches.len(), 1);
        assert_eq!(result.matches[0].rule_id, "TEST_RULE");
        assert_eq!(result.matches[0].span, 2..10);
        assert_eq!(result.matches[0].suggestions, vec!["could have"]);
    }

    #[test]
    fn test_no_match() {
        static TEST_RULES: &[PatternRule] = &[PatternRule {
            id: "TEST_RULE",
            pattern: &["could", "of"],
            suggestion: "could have",
            message: "Did you mean 'could have'?",
        }];

        let checker = PatternRuleChecker::new(TEST_RULES);

        let tokens = vec![
            make_token("I", 0..1),
            make_token("could", 2..7),
            make_token("have", 8..12),
            make_token("done", 13..17),
        ];

        let result = checker.check("I could have done", &tokens);
        assert!(result.matches.is_empty());
    }

    #[test]
    fn test_case_insensitive() {
        static TEST_RULES: &[PatternRule] = &[PatternRule {
            id: "TEST_RULE",
            pattern: &["could", "of"],
            suggestion: "could have",
            message: "Did you mean 'could have'?",
        }];

        let checker = PatternRuleChecker::new(TEST_RULES);

        let tokens = vec![
            make_token("Could", 0..5),
            make_token("OF", 6..8),
        ];

        let result = checker.check("Could OF", &tokens);
        assert_eq!(result.matches.len(), 1);
    }

    // Tests for AhoPatternRuleChecker

    #[test]
    fn test_aho_pattern_match() {
        static TEST_RULES: &[PatternRule] = &[PatternRule {
            id: "TEST_RULE",
            pattern: &["could", "of"],
            suggestion: "could have",
            message: "Did you mean 'could have'?",
        }];

        let checker = AhoPatternRuleChecker::new(TEST_RULES);

        let tokens = vec![
            make_token("I", 0..1),
            make_token("could", 2..7),
            make_token("of", 8..10),
            make_token("done", 11..15),
            make_token("that", 16..20),
        ];

        let result = checker.check("I could of done that", &tokens);
        assert_eq!(result.matches.len(), 1);
        assert_eq!(result.matches[0].rule_id, "TEST_RULE");
        assert_eq!(result.matches[0].span, 2..10);
        assert_eq!(result.matches[0].suggestions, vec!["could have"]);
    }

    #[test]
    fn test_aho_no_match() {
        static TEST_RULES: &[PatternRule] = &[PatternRule {
            id: "TEST_RULE",
            pattern: &["could", "of"],
            suggestion: "could have",
            message: "Did you mean 'could have'?",
        }];

        let checker = AhoPatternRuleChecker::new(TEST_RULES);

        let tokens = vec![
            make_token("I", 0..1),
            make_token("could", 2..7),
            make_token("have", 8..12),
            make_token("done", 13..17),
        ];

        let result = checker.check("I could have done", &tokens);
        assert!(result.matches.is_empty());
    }

    #[test]
    fn test_aho_case_insensitive() {
        static TEST_RULES: &[PatternRule] = &[PatternRule {
            id: "TEST_RULE",
            pattern: &["could", "of"],
            suggestion: "could have",
            message: "Did you mean 'could have'?",
        }];

        let checker = AhoPatternRuleChecker::new(TEST_RULES);

        let tokens = vec![make_token("Could", 0..5), make_token("OF", 6..8)];

        let result = checker.check("Could OF", &tokens);
        assert_eq!(result.matches.len(), 1);
    }

    #[test]
    fn test_aho_multiple_rules_same_first_word() {
        static TEST_RULES: &[PatternRule] = &[
            PatternRule {
                id: "COULD_OF",
                pattern: &["could", "of"],
                suggestion: "could have",
                message: "Did you mean 'could have'?",
            },
            PatternRule {
                id: "COULD_CARE_LESS",
                pattern: &["could", "care", "less"],
                suggestion: "couldn't care less",
                message: "Did you mean 'couldn't care less'?",
            },
        ];

        let checker = AhoPatternRuleChecker::new(TEST_RULES);

        // Test "could of"
        let tokens1 = vec![
            make_token("I", 0..1),
            make_token("could", 2..7),
            make_token("of", 8..10),
        ];
        let result1 = checker.check("I could of", &tokens1);
        assert_eq!(result1.matches.len(), 1);
        assert_eq!(result1.matches[0].rule_id, "COULD_OF");

        // Test "could care less"
        let tokens2 = vec![
            make_token("I", 0..1),
            make_token("could", 2..7),
            make_token("care", 8..12),
            make_token("less", 13..17),
        ];
        let result2 = checker.check("I could care less", &tokens2);
        assert_eq!(result2.matches.len(), 1);
        assert_eq!(result2.matches[0].rule_id, "COULD_CARE_LESS");
    }

    #[test]
    fn test_aho_vs_naive_same_results() {
        static TEST_RULES: &[PatternRule] = &[
            PatternRule {
                id: "COULD_OF",
                pattern: &["could", "of"],
                suggestion: "could have",
                message: "Did you mean 'could have'?",
            },
            PatternRule {
                id: "TOW_THE_LINE",
                pattern: &["tow", "the", "line"],
                suggestion: "toe the line",
                message: "Did you mean 'toe the line'?",
            },
            PatternRule {
                id: "LA_PAZ",
                pattern: &["la", "paz"],
                suggestion: "La Paz",
                message: "La Paz needs to be capitalized.",
            },
        ];

        let naive = PatternRuleChecker::new(TEST_RULES);
        let aho = AhoPatternRuleChecker::new(TEST_RULES);

        let test_cases = [
            ("I could of done that", vec![
                make_token("I", 0..1),
                make_token("could", 2..7),
                make_token("of", 8..10),
                make_token("done", 11..15),
                make_token("that", 16..20),
            ]),
            ("You have to tow the line", vec![
                make_token("You", 0..3),
                make_token("have", 4..8),
                make_token("to", 9..11),
                make_token("tow", 12..15),
                make_token("the", 16..19),
                make_token("line", 20..24),
            ]),
            ("She visited la paz", vec![
                make_token("She", 0..3),
                make_token("visited", 4..11),
                make_token("la", 12..14),
                make_token("paz", 15..18),
            ]),
            ("No errors here", vec![
                make_token("No", 0..2),
                make_token("errors", 3..9),
                make_token("here", 10..14),
            ]),
        ];

        for (text, tokens) in test_cases {
            let naive_result = naive.check(text, &tokens);
            let aho_result = aho.check(text, &tokens);

            assert_eq!(
                naive_result.matches.len(),
                aho_result.matches.len(),
                "Match count differs for: {}",
                text
            );

            for (n, a) in naive_result.matches.iter().zip(aho_result.matches.iter()) {
                assert_eq!(n.rule_id, a.rule_id, "Rule ID differs for: {}", text);
                assert_eq!(n.span, a.span, "Span differs for: {}", text);
                assert_eq!(n.suggestions, a.suggestions, "Suggestions differ for: {}", text);
            }
        }
    }

    #[test]
    fn test_aho_empty_text() {
        static TEST_RULES: &[PatternRule] = &[PatternRule {
            id: "TEST_RULE",
            pattern: &["could", "of"],
            suggestion: "could have",
            message: "Did you mean 'could have'?",
        }];

        let checker = AhoPatternRuleChecker::new(TEST_RULES);
        let tokens: Vec<AnalyzedToken> = vec![];

        let result = checker.check("", &tokens);
        assert!(result.matches.is_empty());
    }

    #[test]
    fn test_aho_word_boundary() {
        // Ensure AC doesn't match "could" inside "uncould" or similar
        static TEST_RULES: &[PatternRule] = &[PatternRule {
            id: "AS_OPPOSE_TO",
            pattern: &["as", "oppose", "to"],
            suggestion: "as opposed to",
            message: "Did you mean 'as opposed to'?",
        }];

        let checker = AhoPatternRuleChecker::new(TEST_RULES);

        // "as" should not match inside "was"
        let tokens = vec![
            make_token("It", 0..2),
            make_token("was", 3..6),
            make_token("opposed", 7..14),
            make_token("to", 15..17),
        ];

        let result = checker.check("It was opposed to", &tokens);
        assert!(result.matches.is_empty());
    }
}
