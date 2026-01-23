//! Uncountable noun checker
//!
//! Detects incorrect pluralization of uncountable nouns like:
//! - "informations" → "information"
//! - "advices" → "advice"
//! - "furnitures" → "furniture"

use crate::core::{AnalyzedToken, CheckResult, Match, Severity, TokenKind};
use crate::core::traits::Checker;
use crate::checker::data::en_uncountable::is_en_uncountable;
use std::collections::HashSet;
use std::sync::LazyLock;

/// Common uncountable nouns that are frequently incorrectly pluralized
/// These are checked first for performance
static COMMON_UNCOUNTABLE: LazyLock<HashSet<&'static str>> = LazyLock::new(|| {
    [
        "information", "advice", "furniture", "luggage", "equipment",
        "knowledge", "money", "news", "progress", "research",
        "traffic", "weather", "work", "homework", "housework",
        "software", "hardware", "feedback", "evidence", "music",
        "art", "poetry", "literature", "vocabulary", "grammar",
        "machinery", "jewelry", "clothing", "underwear", "footwear",
        "garbage", "rubbish", "trash", "litter", "junk",
        "bread", "butter", "cheese", "meat", "rice",
        "water", "milk", "coffee", "tea", "wine",
        "air", "oxygen", "nitrogen", "hydrogen", "electricity",
        "sunshine", "darkness", "lightning", "thunder", "rain",
        "snow", "ice", "fog", "mist", "dust",
        "sand", "dirt", "mud", "grass", "hair",
        "wool", "cotton", "silk", "leather", "rubber",
        "gold", "silver", "copper", "iron", "steel",
        "wood", "paper", "glass", "plastic", "cement",
        "patience", "courage", "honesty", "intelligence", "wisdom",
        "happiness", "sadness", "anger", "fear", "love",
        "hate", "peace", "violence", "justice", "freedom",
        "democracy", "capitalism", "socialism", "communism", "racism",
        "education", "employment", "unemployment", "inflation", "pollution",
        "corruption", "discrimination", "immigration", "emigration", "transportation",
    ].into_iter().collect()
});

pub struct UncountableNounChecker {
    /// Whether to check all uncountable nouns or just common ones
    check_all: bool,
}

impl UncountableNounChecker {
    /// Create a new checker that only checks common uncountable nouns
    pub fn new() -> Self {
        Self { check_all: false }
    }

    /// Create a checker that checks all 5579 uncountable nouns
    pub fn with_full_dictionary() -> Self {
        Self { check_all: true }
    }

    /// Check if a word is an incorrectly pluralized uncountable noun
    fn check_word(&self, word: &str) -> Option<(String, String)> {
        let lower = word.to_lowercase();

        // Check for common plural endings
        // Try removing 's' first (most common)
        if lower.ends_with('s') && lower.len() > 2 {
            let singular = &lower[..lower.len() - 1];

            // Check if the singular form is uncountable
            if self.is_uncountable(singular) {
                return Some((singular.to_string(), format!(
                    "'{}' is an uncountable noun and should not be pluralized",
                    singular
                )));
            }

            // Check for -es ending (e.g., "advices" → "advice")
            if lower.ends_with("es") && lower.len() > 3 {
                let singular_es = &lower[..lower.len() - 2];
                if self.is_uncountable(singular_es) {
                    return Some((singular_es.to_string(), format!(
                        "'{}' is an uncountable noun and should not be pluralized",
                        singular_es
                    )));
                }
            }

            // Check for -ies ending (e.g., "monies" is actually valid for money in legal context)
            // Skip this as it's often a valid plural
        }

        None
    }

    fn is_uncountable(&self, word: &str) -> bool {
        if COMMON_UNCOUNTABLE.contains(word) {
            return true;
        }

        if self.check_all {
            return is_en_uncountable(word);
        }

        false
    }
}

impl Default for UncountableNounChecker {
    fn default() -> Self {
        Self::new()
    }
}

impl Checker for UncountableNounChecker {
    fn check(&self, _text: &str, tokens: &[AnalyzedToken]) -> CheckResult {
        let mut result = CheckResult::new();

        for analyzed in tokens {
            let token = &analyzed.token;

            // Skip non-words
            if token.kind != TokenKind::Word {
                continue;
            }

            // Skip short words
            if token.text.len() < 4 {
                continue;
            }

            // Check if this is an incorrectly pluralized uncountable noun
            if let Some((singular, message)) = self.check_word(token.text) {
                result.matches.push(Match {
                    span: token.span.clone(),
                    message,
                    rule_id: "UNCOUNTABLE_PLURAL".to_string(),
                    suggestions: vec![singular],
                    severity: Severity::Warning,
                });
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
        let checker = UncountableNounChecker::new();
        let tokenizer = SimpleTokenizer::new();
        let analyzer = PassthroughAnalyzer::new();
        let tokens = tokenizer.tokenize(text);
        let analyzed = analyzer.analyze(tokens);
        checker.check(text, &analyzed)
    }

    #[test]
    fn test_informations() {
        let result = check_text("I need more informations about this.");
        assert_eq!(result.matches.len(), 1);
        assert_eq!(result.matches[0].suggestions[0], "information");
    }

    #[test]
    fn test_advices() {
        let result = check_text("She gave me some advices.");
        assert_eq!(result.matches.len(), 1);
        assert_eq!(result.matches[0].suggestions[0], "advice");
    }

    #[test]
    fn test_furnitures() {
        let result = check_text("We bought new furnitures.");
        assert_eq!(result.matches.len(), 1);
        assert_eq!(result.matches[0].suggestions[0], "furniture");
    }

    #[test]
    fn test_correct_usage() {
        let result = check_text("I need more information about this.");
        assert_eq!(result.matches.len(), 0);
    }

    #[test]
    fn test_countable_noun() {
        // "cats" should not trigger - "cat" is countable
        let result = check_text("I have two cats.");
        assert_eq!(result.matches.len(), 0);
    }
}
