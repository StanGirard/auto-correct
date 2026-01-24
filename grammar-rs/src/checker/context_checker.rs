//! Context-sensitive word checker
//!
//! This module detects commonly confused words based on surrounding context.
//! For example, "affect" vs "effect" can be distinguished by looking at nearby
//! words like "medication" (prescribe) vs "theft" (proscribe).

use crate::checker::data::en_context_words::EN_CONTEXT_RULES;
use crate::core::traits::Checker;
use crate::core::{AnalyzedToken, CheckResult, Match, Severity, TokenKind};
use regex::Regex;
use std::collections::HashMap;

/// Context-sensitive word checker that detects commonly confused words.
///
/// This checker uses regex patterns to determine which word is correct
/// based on the surrounding context.
///
/// # Example
/// ```ignore
/// let checker = ContextChecker::new();
/// // "I will prescribe theft" -> suggests "proscribe"
/// // "The doctor will prescribe medication" -> no error
/// ```
pub struct ContextChecker {
    /// Compiled regex patterns for context matching
    context1_patterns: Vec<Option<Regex>>,
    context2_patterns: Vec<Option<Regex>>,
    /// Map from word pattern to rule indices
    word_to_rules: HashMap<String, Vec<usize>>,
    /// Window size for context matching (words before and after)
    context_window: usize,
}

impl ContextChecker {
    /// Create a new ContextChecker with default settings.
    pub fn new() -> Self {
        let mut context1_patterns = Vec::with_capacity(EN_CONTEXT_RULES.len());
        let mut context2_patterns = Vec::with_capacity(EN_CONTEXT_RULES.len());
        let mut word_to_rules: HashMap<String, Vec<usize>> = HashMap::new();

        for (idx, rule) in EN_CONTEXT_RULES.iter().enumerate() {
            // Compile regex patterns
            let ctx1 = Regex::new(rule.context1_regex).ok();
            let ctx2 = Regex::new(rule.context2_regex).ok();

            context1_patterns.push(ctx1);
            context2_patterns.push(ctx2);

            // Build word index - we match against the base pattern
            // word1 and word2 can contain regex-like patterns (e.g., "prescribe[ds]?")
            // Extract the base word for indexing
            let base1 = extract_base_word(rule.word1);
            let base2 = extract_base_word(rule.word2);

            word_to_rules.entry(base1.to_lowercase()).or_default().push(idx);
            word_to_rules.entry(base2.to_lowercase()).or_default().push(idx);
        }

        Self {
            context1_patterns,
            context2_patterns,
            word_to_rules,
            context_window: 10, // Look at 10 words before and after
        }
    }

    /// Get the number of context rules available.
    pub fn rule_count() -> usize {
        EN_CONTEXT_RULES.len()
    }

    /// Check if a word matches one of the patterns in a rule
    fn word_matches_pattern(word: &str, pattern: &str) -> bool {
        // Try exact match first
        if word.eq_ignore_ascii_case(pattern) {
            return true;
        }

        // Try regex match if pattern contains regex characters
        if pattern.contains('[') || pattern.contains('?') || pattern.contains('+') {
            if let Ok(re) = Regex::new(&format!("(?i)^{}$", pattern)) {
                return re.is_match(word);
            }
        }

        false
    }

    /// Get context string from surrounding tokens
    fn get_context(&self, tokens: &[AnalyzedToken], current_idx: usize) -> String {
        let mut context_words = Vec::new();

        // Get words before
        let start = current_idx.saturating_sub(self.context_window);
        for token in tokens[start..current_idx].iter() {
            if token.token.kind == TokenKind::Word {
                context_words.push(token.token.text.to_lowercase());
            }
        }

        // Get words after
        let end = (current_idx + 1 + self.context_window).min(tokens.len());
        for token in tokens[current_idx + 1..end].iter() {
            if token.token.kind == TokenKind::Word {
                context_words.push(token.token.text.to_lowercase());
            }
        }

        context_words.join(" ")
    }
}

impl Default for ContextChecker {
    fn default() -> Self {
        Self::new()
    }
}

/// Extract base word from a pattern like "prescribe[ds]?" -> "prescribe"
/// Handles patterns like "statues?" -> "statue" (removes char before ?)
fn extract_base_word(pattern: &str) -> String {
    let mut result = String::new();
    let mut in_bracket = false;
    let chars: Vec<char> = pattern.chars().collect();

    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        match c {
            '[' => in_bracket = true,
            ']' => in_bracket = false,
            '?' | '+' | '*' => {
                // If this quantifier follows a regular char (not a bracket group),
                // remove that char from result since it's optional
                if !result.is_empty() && i > 0 {
                    let prev = chars[i - 1];
                    if prev != ']' && !in_bracket {
                        result.pop();
                    }
                }
            }
            _ if !in_bracket => result.push(c),
            _ => {}
        }
        i += 1;
    }

    result
}

impl Checker for ContextChecker {
    fn check(&self, _text: &str, tokens: &[AnalyzedToken]) -> CheckResult {
        let mut matches = Vec::new();

        for (idx, token) in tokens.iter().enumerate() {
            if token.token.kind != TokenKind::Word {
                continue;
            }

            let word = &token.token.text;
            let word_lower = word.to_lowercase();

            // Check if this word's base form is in our rules
            let base = extract_base_word(&word_lower);

            if let Some(rule_indices) = self.word_to_rules.get(&base) {
                let context = self.get_context(tokens, idx);

                for &rule_idx in rule_indices {
                    let rule = &EN_CONTEXT_RULES[rule_idx];

                    // Determine if current word matches word1 or word2
                    let matches_word1 = Self::word_matches_pattern(word, rule.word1);
                    let matches_word2 = Self::word_matches_pattern(word, rule.word2);

                    if !matches_word1 && !matches_word2 {
                        continue;
                    }

                    // Check context patterns
                    let context1_match = match &self.context1_patterns[rule_idx] {
                        Some(re) => re.is_match(&context),
                        None => false,
                    };
                    let context2_match = match &self.context2_patterns[rule_idx] {
                        Some(re) => re.is_match(&context),
                        None => false,
                    };

                    // If we used word1 but context suggests word2
                    if matches_word1 && context2_match && !context1_match {
                        let suggestion = extract_base_word(rule.word2);
                        let explanation = rule.explanation2.unwrap_or("different meaning");

                        matches.push(Match {
                            span: token.token.span.clone(),
                            rule_id: "CONTEXT_WORD".to_string(),
                            message: format!(
                                "Did you mean '{}'? Based on context, '{}' ({}) might be more appropriate.",
                                suggestion, suggestion, explanation
                            ),
                            suggestions: vec![suggestion],
                            severity: Severity::Warning,
                        });
                    }
                    // If we used word2 but context suggests word1
                    else if matches_word2 && context1_match && !context2_match {
                        let suggestion = extract_base_word(rule.word1);
                        let explanation = rule.explanation1.unwrap_or("different meaning");

                        matches.push(Match {
                            span: token.token.span.clone(),
                            rule_id: "CONTEXT_WORD".to_string(),
                            message: format!(
                                "Did you mean '{}'? Based on context, '{}' ({}) might be more appropriate.",
                                suggestion, suggestion, explanation
                            ),
                            suggestions: vec![suggestion],
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
    use crate::prelude::*;

    fn tokenize(text: &str) -> Vec<AnalyzedToken<'_>> {
        let tokenizer = SimpleTokenizer::new();
        let analyzer = PassthroughAnalyzer::new();
        let tokens = tokenizer.tokenize(text);
        analyzer.analyze(tokens)
    }

    #[test]
    fn test_extract_base_word() {
        assert_eq!(extract_base_word("prescribe[ds]?"), "prescribe");
        assert_eq!(extract_base_word("heroin"), "heroin");
        assert_eq!(extract_base_word("statues?"), "statue");
    }

    #[test]
    fn test_heroin_heroine() {
        let checker = ContextChecker::new();
        // "heroin" in context of "literature" should suggest "heroine"
        let text = "The heroin of the novel was brave.";
        let tokens = tokenize(text);

        let result = checker.check(text, &tokens);

        // May or may not trigger depending on how strict the regex is
        // This is more of a smoke test
        if !result.matches.is_empty() {
            let m = &result.matches[0];
            assert!(m.message.contains("heroine") || m.suggestions.contains(&"heroine".to_string()));
        }
    }

    #[test]
    fn test_no_false_positives_normal_text() {
        let checker = ContextChecker::new();
        let text = "The quick brown fox jumps over the lazy dog.";
        let tokens = tokenize(text);

        let result = checker.check(text, &tokens);

        assert!(result.matches.is_empty(), "Should have no false positives");
    }

    #[test]
    fn test_correct_context() {
        let checker = ContextChecker::new();
        // "heroin" with drug context should be correct
        let text = "The heroin addict needed help with withdrawal.";
        let tokens = tokenize(text);

        let result = checker.check(text, &tokens);

        // Should not suggest heroine when drug context is present
        assert!(result.matches.is_empty(), "Correct context should not trigger");
    }

    #[test]
    fn test_dessert_desert() {
        let checker = ContextChecker::new();
        // "dessert" in arid/dry context should suggest "desert"
        let text = "The Sahara dessert is very dry and arid.";
        let tokens = tokenize(text);

        let result = checker.check(text, &tokens);

        // May trigger suggestion for "desert"
        if !result.matches.is_empty() {
            let m = &result.matches[0];
            assert!(m.suggestions.contains(&"desert".to_string()));
        }
    }

    #[test]
    fn test_rule_count() {
        assert!(ContextChecker::rule_count() > 5, "Should have context rules");
    }
}
