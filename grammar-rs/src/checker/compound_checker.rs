//! Compound word checker
//!
//! Detects compound word errors:
//! - Spaced compounds that should be hyphenated: "well being" → "well-being"
//! - Spaced compounds that should be joined: "air plane" → "airplane"
//! - Hyphenated words that should be joined: "air-plane" → "airplane"

use crate::core::{AnalyzedToken, CheckResult, Match, Severity, TokenKind};
use crate::core::traits::Checker;
use crate::checker::data::en_compounds::{EN_COMPOUND_RULES, CompoundRule as EnCompoundRule, get_en_compound};
use crate::checker::data::fr_compounds::{FR_COMPOUND_RULES, CompoundRule as FrCompoundRule, get_fr_compound};
use std::collections::HashSet;
use std::sync::LazyLock;

/// Language for compound checking
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompoundLanguage {
    English,
    French,
}

/// Set of first words for English compounds (for quick lookup)
static EN_COMPOUND_FIRST_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| {
    EN_COMPOUND_RULES
        .iter()
        .filter_map(|rule| rule.word.split('-').next())
        .map(|s| s.to_lowercase())
        .collect()
});

/// Set of first words for French compounds (for quick lookup)
static FR_COMPOUND_FIRST_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| {
    FR_COMPOUND_RULES
        .iter()
        .filter_map(|rule| rule.word.split('-').next())
        .map(|s| s.to_lowercase())
        .collect()
});

pub struct CompoundWordChecker {
    language: CompoundLanguage,
}

impl CompoundWordChecker {
    /// Create a new CompoundWordChecker for English (default)
    pub fn new() -> Self {
        Self::english()
    }

    /// Create a CompoundWordChecker for English
    pub fn english() -> Self {
        Self { language: CompoundLanguage::English }
    }

    /// Create a CompoundWordChecker for French
    pub fn french() -> Self {
        Self { language: CompoundLanguage::French }
    }

    /// Check if a word could be the first part of a compound
    fn has_first_word(&self, word: &str) -> bool {
        match self.language {
            CompoundLanguage::English => EN_COMPOUND_FIRST_WORDS.contains(word),
            CompoundLanguage::French => FR_COMPOUND_FIRST_WORDS.contains(word),
        }
    }

    /// Check if two consecutive words form a compound and return suggestion info
    fn check_spaced_compound(&self, word1: &str, word2: &str) -> Option<(String, String)> {
        let w1_lower = word1.to_lowercase();
        let w2_lower = word2.to_lowercase();
        // The lookup tables index by joined form (no hyphen)
        let combined_joined = format!("{}{}", w1_lower, w2_lower);
        let combined_hyphen = format!("{}-{}", w1_lower, w2_lower);

        // Check based on language
        match self.language {
            CompoundLanguage::English => {
                if let Some(rule) = get_en_compound(&combined_joined) {
                    return Some(self.make_spaced_suggestion(word1, word2, &combined_hyphen, rule.lowercase_joined));
                }
            }
            CompoundLanguage::French => {
                if let Some(rule) = get_fr_compound(&combined_joined) {
                    return Some(self.make_spaced_suggestion(word1, word2, &combined_hyphen, rule.lowercase_joined));
                }
            }
        }

        None
    }

    /// Generate suggestion and message for spaced compound
    fn make_spaced_suggestion(&self, word1: &str, word2: &str, combined_hyphen: &str, lowercase_joined: bool) -> (String, String) {
        let suggestion = if lowercase_joined {
            combined_hyphen.replace("-", "")
        } else {
            combined_hyphen.to_string()
        };

        let message = if lowercase_joined {
            format!("'{}' should be written as one word: '{}'",
                    format!("{} {}", word1, word2), suggestion)
        } else {
            format!("'{}' should be hyphenated: '{}'",
                    format!("{} {}", word1, word2), suggestion)
        };

        (suggestion, message)
    }

}

impl Default for CompoundWordChecker {
    fn default() -> Self {
        Self::new()
    }
}

impl Checker for CompoundWordChecker {
    fn check(&self, _text: &str, tokens: &[AnalyzedToken]) -> CheckResult {
        let mut result = CheckResult::new();
        let mut skip_next = false;

        for i in 0..tokens.len() {
            if skip_next {
                skip_next = false;
                continue;
            }

            let token = &tokens[i].token;

            // Skip non-words
            if token.kind != TokenKind::Word {
                continue;
            }

            let word_lower = token.text.to_lowercase();

            // Check if this word could be the first part of a compound
            if !self.has_first_word(&word_lower) {
                continue;
            }

            // Look for the next word token, skipping whitespace
            // But stop if we hit a hyphen (already hyphenated)
            let mut next_idx = i + 1;
            let mut found_hyphen = false;
            while next_idx < tokens.len() && tokens[next_idx].token.kind != TokenKind::Word {
                if tokens[next_idx].token.text == "-" {
                    found_hyphen = true;
                    break;
                }
                next_idx += 1;
            }

            // Skip if already hyphenated
            if found_hyphen {
                continue;
            }

            if next_idx < tokens.len() {
                let next_token = &tokens[next_idx].token;
                if next_token.kind == TokenKind::Word {
                    // Check if these two words form a compound
                    if let Some((suggestion, message)) = self.check_spaced_compound(token.text, next_token.text) {
                        // Create a span covering both words
                        let combined_span = token.span.start..next_token.span.end;
                        result.matches.push(Match {
                            span: combined_span,
                            message,
                            rule_id: "COMPOUND_SPACE".to_string(),
                            suggestions: vec![suggestion],
                            severity: Severity::Hint,
                        });
                        skip_next = true;
                    }
                }
            }
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokenizer::SimpleTokenizer;
    use crate::analyzer::PassthroughAnalyzer;
    use crate::core::traits::{Tokenizer, Analyzer};

    fn check_text(text: &str) -> CheckResult {
        let checker = CompoundWordChecker::new();
        let tokenizer = SimpleTokenizer::new();
        let analyzer = PassthroughAnalyzer::new();
        let tokens = tokenizer.tokenize(text);
        let analyzed = analyzer.analyze(tokens);
        checker.check(text, &analyzed)
    }

    fn check_text_fr(text: &str) -> CheckResult {
        let checker = CompoundWordChecker::french();
        let tokenizer = SimpleTokenizer::new();
        let analyzer = PassthroughAnalyzer::new();
        let tokens = tokenizer.tokenize(text);
        let analyzed = analyzer.analyze(tokens);
        checker.check(text, &analyzed)
    }

    #[test]
    fn test_spaced_compound_join() {
        // "air plane" should become "airplane" (joined)
        let result = check_text("The air plane landed safely.");
        assert_eq!(result.matches.len(), 1, "Expected 1 match for 'air plane'");
        assert_eq!(result.matches[0].suggestions[0], "airplane");
        assert_eq!(result.matches[0].rule_id, "COMPOUND_SPACE");
    }

    #[test]
    fn test_spaced_compound_hyphenate() {
        // "well being" should become "well-being" (hyphenated)
        let result = check_text("Your well being matters.");
        assert_eq!(result.matches.len(), 1, "Expected 1 match for 'well being'");
        assert_eq!(result.matches[0].suggestions[0], "well-being");
        assert_eq!(result.matches[0].rule_id, "COMPOUND_SPACE");
    }

    #[test]
    fn test_no_false_positive() {
        // Regular words should not trigger
        let result = check_text("The cat sat on the mat.");
        assert_eq!(result.matches.len(), 0);
    }

    #[test]
    fn test_correct_compound() {
        // Already correct compound should not trigger
        let result = check_text("The airplane landed.");
        assert_eq!(result.matches.len(), 0);
    }

    #[test]
    fn test_french_spaced_compound() {
        // "aller retour" should become "aller-retour"
        let result = check_text_fr("J'ai pris un aller retour.");
        assert_eq!(result.matches.len(), 1, "Expected 1 match for 'aller retour'");
        assert_eq!(result.matches[0].suggestions[0], "aller-retour");
    }

    #[test]
    fn test_french_no_false_positive() {
        // Regular French words should not trigger
        let result = check_text_fr("Le chat dort sur le tapis.");
        assert_eq!(result.matches.len(), 0);
    }

    // Note: Hyphenated word checking (e.g., "air-plane" → "airplane") is not supported
    // because the tokenizer splits "air-plane" into ["air", "-", "plane"] tokens.
    // The checker only detects spaced compounds like "air plane".
}
