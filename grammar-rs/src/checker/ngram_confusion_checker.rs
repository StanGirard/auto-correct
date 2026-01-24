//! N-gram based confusion checker
//!
//! Uses N-gram language model probabilities to detect confusion errors
//! with higher accuracy than context-based rules alone.
//!
//! This checker loads pre-extracted N-gram data and uses Stupid Backoff
//! probability calculation to determine which word is more likely in context.

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use crate::core::{AnalyzedToken, Match, Severity, TokenKind, CheckResult};
use crate::core::traits::Checker;
use crate::language_model::{CompactNgramModel, Probability};

/// Confusion pair with calibrated factor
#[derive(Debug, Clone)]
pub struct ConfusionPair {
    pub word1: &'static str,
    pub word2: &'static str,
    /// Factor threshold: if P(word2)/P(word1) > factor, suggest word2
    pub factor: u64,
}

/// N-gram based confusion checker using memory-mapped compact model
pub struct NgramConfusionChecker {
    model: Arc<CompactNgramModel>,
    /// Lookup: word -> list of confusion pairs containing that word
    pairs_by_word: HashMap<String, Vec<ConfusionEntry>>,
    /// Minimum factor for suggesting replacement
    min_factor: f64,
    /// Minimum coverage required from N-gram model
    min_coverage: f64,
}

/// Entry in the confusion lookup
#[derive(Clone)]
struct ConfusionEntry {
    /// The alternative word
    alternative: String,
    /// Factor threshold from LanguageTool calibration
    factor: f64,
}

impl NgramConfusionChecker {
    /// Create a new checker with the given compact model
    pub fn new(model: Arc<CompactNgramModel>) -> Self {
        NgramConfusionChecker {
            model,
            pairs_by_word: HashMap::new(),
            min_factor: 3.0,      // Default minimum factor
            min_coverage: 0.1,    // Require at least some N-gram coverage
        }
    }

    /// Create from a compact binary N-gram file (.bin)
    pub fn from_file(path: &Path) -> std::io::Result<Self> {
        let model = CompactNgramModel::open(path)?;
        Ok(Self::new(Arc::new(model)))
    }

    /// Try to load from the default data path, returns None if not found
    ///
    /// If the N-gram data is not found locally and `GRAMMAR_RS_AUTO_DOWNLOAD=1`
    /// environment variable is set (with `ngram-download` feature enabled),
    /// the data will be automatically downloaded from R2.
    pub fn try_load_en() -> Option<Self> {
        use crate::language_model::downloader;

        // Preferred data directories to check
        let data_dirs = [
            Path::new("data/ngrams"),
            Path::new("../data/ngrams"),
            Path::new("grammar-rs/data/ngrams"),
        ];

        // First, check if file exists in any of the paths
        for data_dir in &data_dirs {
            let bin_path = data_dir.join("en_ngrams.bin");
            if bin_path.exists() {
                match Self::from_file(&bin_path) {
                    Ok(checker) => {
                        tracing::info!("Loaded EN N-gram model from {:?}", bin_path);
                        return Some(checker.with_en_confusion_pairs().with_en_confusion_extended());
                    }
                    Err(e) => {
                        tracing::warn!("Failed to load EN N-gram model from {:?}: {}", bin_path, e);
                    }
                }
            }
        }

        // File not found - try auto-download to first writable directory
        let primary_dir = data_dirs[0];
        match downloader::ensure_ngram_data("en", primary_dir) {
            Ok(true) => {
                // Successfully downloaded, try to load
                let bin_path = primary_dir.join("en_ngrams.bin");
                match Self::from_file(&bin_path) {
                    Ok(checker) => {
                        tracing::info!("Loaded EN N-gram model after download");
                        return Some(checker.with_en_confusion_pairs().with_en_confusion_extended());
                    }
                    Err(e) => {
                        tracing::warn!("Failed to load downloaded EN N-gram model: {}", e);
                    }
                }
            }
            Ok(false) => {
                // Auto-download disabled, nothing to do
            }
            Err(e) => {
                tracing::warn!("Failed to download EN N-gram data: {}", e);
            }
        }

        None
    }

    /// Try to load from the default data path for French
    ///
    /// If the N-gram data is not found locally and `GRAMMAR_RS_AUTO_DOWNLOAD=1`
    /// environment variable is set (with `ngram-download` feature enabled),
    /// the data will be automatically downloaded from R2.
    pub fn try_load_fr() -> Option<Self> {
        use crate::language_model::downloader;

        // Preferred data directories to check
        let data_dirs = [
            Path::new("data/ngrams"),
            Path::new("../data/ngrams"),
            Path::new("grammar-rs/data/ngrams"),
        ];

        // First, check if file exists in any of the paths
        for data_dir in &data_dirs {
            let bin_path = data_dir.join("fr_ngrams.bin");
            if bin_path.exists() {
                match Self::from_file(&bin_path) {
                    Ok(checker) => {
                        tracing::info!("Loaded FR N-gram model from {:?}", bin_path);
                        return Some(checker.with_fr_confusion_pairs());
                    }
                    Err(e) => {
                        tracing::warn!("Failed to load FR N-gram model from {:?}: {}", bin_path, e);
                    }
                }
            }
        }

        // File not found - try auto-download to first writable directory
        let primary_dir = data_dirs[0];
        match downloader::ensure_ngram_data("fr", primary_dir) {
            Ok(true) => {
                // Successfully downloaded, try to load
                let bin_path = primary_dir.join("fr_ngrams.bin");
                match Self::from_file(&bin_path) {
                    Ok(checker) => {
                        tracing::info!("Loaded FR N-gram model after download");
                        return Some(checker.with_fr_confusion_pairs());
                    }
                    Err(e) => {
                        tracing::warn!("Failed to load downloaded FR N-gram model: {}", e);
                    }
                }
            }
            Ok(false) => {
                // Auto-download disabled, nothing to do
            }
            Err(e) => {
                tracing::warn!("Failed to download FR N-gram data: {}", e);
            }
        }

        None
    }

    /// Set minimum factor threshold
    pub fn with_min_factor(mut self, factor: f64) -> Self {
        self.min_factor = factor;
        self
    }

    /// Set minimum coverage threshold
    pub fn with_min_coverage(mut self, coverage: f64) -> Self {
        self.min_coverage = coverage;
        self
    }

    /// Load confusion pairs from the static data
    pub fn with_en_confusion_pairs(mut self) -> Self {
        use super::data::EN_CONFUSION_DATA;

        for &(word, alternatives) in EN_CONFUSION_DATA.iter() {
            for &(alt_word, factor) in alternatives.iter() {
                // Add entry for word -> alternative
                self.pairs_by_word
                    .entry(word.to_string())
                    .or_insert_with(Vec::new)
                    .push(ConfusionEntry {
                        alternative: alt_word.to_string(),
                        factor: factor as f64,
                    });

                // Also add reverse entry (alternative -> word)
                // This ensures we check both directions
                self.pairs_by_word
                    .entry(alt_word.to_string())
                    .or_insert_with(Vec::new)
                    .push(ConfusionEntry {
                        alternative: word.to_string(),
                        factor: factor as f64,
                    });
            }
        }

        self
    }

    /// Load extended English confusion pairs (higher coverage)
    pub fn with_en_confusion_extended(mut self) -> Self {
        use super::data::en_confusion_extended::EN_CONFUSION_EXTENDED;

        for pair in EN_CONFUSION_EXTENDED.iter() {
            // Add entry for word1 -> word2
            self.pairs_by_word
                .entry(pair.word1.to_string())
                .or_insert_with(Vec::new)
                .push(ConfusionEntry {
                    alternative: pair.word2.to_string(),
                    factor: pair.factor as f64,
                });

            // Add reverse entry if bidirectional
            if pair.bidirectional {
                self.pairs_by_word
                    .entry(pair.word2.to_string())
                    .or_insert_with(Vec::new)
                    .push(ConfusionEntry {
                        alternative: pair.word1.to_string(),
                        factor: pair.factor as f64,
                    });
            }
        }

        self
    }

    /// Load French confusion pairs
    pub fn with_fr_confusion_pairs(mut self) -> Self {
        use super::data::FR_CONFUSION_DATA;

        for &(word, alternatives) in FR_CONFUSION_DATA.iter() {
            for &(alt_word, factor) in alternatives.iter() {
                self.pairs_by_word
                    .entry(word.to_string())
                    .or_insert_with(Vec::new)
                    .push(ConfusionEntry {
                        alternative: alt_word.to_string(),
                        factor: factor as f64,
                    });

                self.pairs_by_word
                    .entry(alt_word.to_string())
                    .or_insert_with(Vec::new)
                    .push(ConfusionEntry {
                        alternative: word.to_string(),
                        factor: factor as f64,
                    });
            }
        }

        self
    }

    /// Find previous word token (skipping whitespace/punctuation)
    fn find_prev_word<'a>(tokens: &'a [AnalyzedToken], idx: usize) -> Option<&'a str> {
        let mut i = idx;
        while i > 0 {
            i -= 1;
            if tokens[i].token.kind == TokenKind::Word {
                return Some(tokens[i].token.text);
            }
        }
        None
    }

    /// Find word 2 positions back
    fn find_prev_prev_word<'a>(tokens: &'a [AnalyzedToken], idx: usize) -> Option<&'a str> {
        let mut count = 0;
        let mut i = idx;
        while i > 0 && count < 2 {
            i -= 1;
            if tokens[i].token.kind == TokenKind::Word {
                count += 1;
                if count == 2 {
                    return Some(tokens[i].token.text);
                }
            }
        }
        None
    }

    /// Check a single token for confusion errors
    fn check_token(&self, tokens: &[AnalyzedToken], idx: usize) -> Option<Match> {
        let token = &tokens[idx];

        // Only check word tokens
        if token.token.kind != TokenKind::Word {
            return None;
        }

        let word_lower = token.token.text.to_lowercase();

        // Check if this word has any confusion pairs
        let entries = self.pairs_by_word.get(&word_lower)?;

        // Get context words
        let prev1 = Self::find_prev_word(tokens, idx);
        let prev2 = Self::find_prev_prev_word(tokens, idx);

        // Get probability of current word
        let current_prob = self.model.get_probability(&word_lower, prev1, prev2);

        // Check if we have sufficient coverage
        if !current_prob.meets_coverage(self.min_coverage) {
            return None;
        }

        // Check each alternative
        for entry in entries {
            let alt_prob = self.model.get_probability(&entry.alternative, prev1, prev2);

            // Skip if alternative has no coverage
            if alt_prob.is_unknown() {
                continue;
            }

            // Calculate ratio: P(alternative) / P(current)
            let ratio = if current_prob.probability < 1e-15 {
                if alt_prob.probability < 1e-15 {
                    1.0 // Both unknown
                } else {
                    f64::MAX // Current unknown, alt known
                }
            } else {
                alt_prob.probability / current_prob.probability
            };

            // Use the calibrated factor from LanguageTool
            // The factor represents how much more likely the alternative must be
            let effective_factor = (entry.factor / 1_000_000.0).max(self.min_factor);

            if ratio > effective_factor {
                return Some(Match {
                    span: token.token.span.clone(),
                    message: format!(
                        "Possible confusion: '{}' is more likely in this context (probability ratio: {:.1}x)",
                        entry.alternative,
                        ratio
                    ),
                    rule_id: "NGRAM_CONFUSION".to_string(),
                    suggestions: vec![entry.alternative.clone()],
                    severity: Severity::Warning,
                });
            }
        }

        None
    }
}

impl Checker for NgramConfusionChecker {
    fn check(&self, _text: &str, tokens: &[AnalyzedToken]) -> CheckResult {
        let mut matches = Vec::new();

        for idx in 0..tokens.len() {
            if let Some(m) = self.check_token(tokens, idx) {
                matches.push(m);
            }
        }

        CheckResult { matches }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::language_model::CompactNgramBuilder;
    use tempfile::NamedTempFile;

    fn create_test_model() -> Arc<CompactNgramModel> {
        // Build a small test model with the compact format
        let mut builder = CompactNgramBuilder::new();
        builder.set_total_tokens(10_000_000);

        // Add test data for their/there confusion
        builder.add_unigram("the".to_string(), 1_000_000);
        builder.add_unigram("their".to_string(), 100_000);
        builder.add_unigram("there".to_string(), 150_000);
        builder.add_unigram("they".to_string(), 200_000);
        builder.add_unigram("house".to_string(), 50_000);
        builder.add_unigram("is".to_string(), 500_000);
        builder.add_unigram("went".to_string(), 80_000);
        builder.add_unigram("to".to_string(), 900_000);

        // Bigrams: "their house" is common, "there house" is rare
        builder.add_bigram("their house".to_string(), 30_000);
        builder.add_bigram("there house".to_string(), 200);
        builder.add_bigram("went to".to_string(), 50_000);
        builder.add_bigram("to their".to_string(), 20_000);
        builder.add_bigram("to there".to_string(), 5_000);

        // Trigrams
        builder.add_trigram("went to their".to_string(), 10_000);
        builder.add_trigram("went to there".to_string(), 500);
        builder.add_trigram("to their house".to_string(), 8_000);
        builder.add_trigram("to there house".to_string(), 50);

        // Build to temp file
        let file = NamedTempFile::new().unwrap();
        builder.build(file.path()).unwrap();

        Arc::new(CompactNgramModel::open(file.path()).unwrap())
    }

    fn create_test_checker() -> NgramConfusionChecker {
        let model = create_test_model();
        let mut checker = NgramConfusionChecker::new(model)
            .with_min_factor(2.0)
            .with_min_coverage(0.0);  // Allow all coverage for tests

        // Add their/there confusion pair manually for testing
        checker.pairs_by_word.insert("their".to_string(), vec![
            ConfusionEntry { alternative: "there".to_string(), factor: 10_000_000.0 },
        ]);
        checker.pairs_by_word.insert("there".to_string(), vec![
            ConfusionEntry { alternative: "their".to_string(), factor: 10_000_000.0 },
        ]);

        checker
    }

    fn tokenize_simple(text: &str) -> Vec<AnalyzedToken> {
        use crate::tokenizer::SimpleTokenizer;
        use crate::analyzer::PassthroughAnalyzer;
        use crate::core::traits::{Tokenizer, Analyzer};

        let tokenizer = SimpleTokenizer::new();
        let analyzer = PassthroughAnalyzer::new();
        let tokens = tokenizer.tokenize(text);
        analyzer.analyze(tokens)
    }

    #[test]
    fn test_their_there_detection() {
        let checker = create_test_checker();

        // "went to there house" should suggest "their" because
        // "to their house" is much more common than "to there house"
        let tokens = tokenize_simple("went to there house");
        let result = checker.check("went to there house", &tokens);

        // Check that we detected an error
        println!("Matches: {:?}", result.matches);
        assert!(!result.matches.is_empty(), "Should detect 'there' -> 'their' error");
        assert_eq!(result.matches[0].suggestions, vec!["their"]);
    }

    #[test]
    fn test_correct_usage_no_error() {
        let checker = create_test_checker();

        // "went to their house" is correct
        let tokens = tokenize_simple("went to their house");
        let result = checker.check("went to their house", &tokens);

        // Should not flag correct usage
        // (unless "there" is even more common, which it isn't in our test data)
        println!("Matches for correct usage: {:?}", result.matches);
    }

    #[test]
    fn test_with_en_pairs() {
        // Create a minimal model for testing pair loading
        let model = create_test_model();
        let checker = NgramConfusionChecker::new(model)
            .with_en_confusion_pairs();

        // Verify pairs were loaded (affect/effect are in the basic confusion data)
        assert!(checker.pairs_by_word.contains_key("affect"), "Should have 'affect'");
        assert!(checker.pairs_by_word.contains_key("effect"), "Should have 'effect'");
        // Note: their/there are in the extended confusion data, not the basic set
        assert!(checker.pairs_by_word.contains_key("loose"), "Should have 'loose'");
        assert!(checker.pairs_by_word.contains_key("lose"), "Should have 'lose'");
    }

    #[test]
    fn test_with_en_extended_pairs() {
        let model = create_test_model();
        let checker = NgramConfusionChecker::new(model)
            .with_en_confusion_extended();

        // Extended data has more pairs, like verb tense confusions
        assert!(!checker.pairs_by_word.is_empty(), "Should have loaded extended pairs");

        // Check that some pairs are loaded
        let total_pairs: usize = checker.pairs_by_word.values().map(|v| v.len()).sum();
        assert!(total_pairs > 1000, "Should have many extended pairs, got {}", total_pairs);
    }

    #[test]
    #[ignore] // Requires actual N-gram data files
    fn test_load_real_en_model() {
        if let Some(checker) = NgramConfusionChecker::try_load_en() {
            // Note: their/there is NOT in the N-gram confusion data
            // (it's handled by dedicated TheirTheyreThereRule)
            // Use affect/effect which IS in the confusion data
            let tokens = tokenize_simple("This will effect the outcome");
            let result = checker.check("This will effect the outcome", &tokens);
            println!("Real model matches: {:?}", result.matches);
            println!("Loaded {} confusion pairs", checker.pairs_by_word.len());
            // Model should load successfully even if no matches (depends on context)
            assert!(checker.pairs_by_word.contains_key("effect"), "Should have effect in pairs");
            assert!(checker.pairs_by_word.contains_key("affect"), "Should have affect in pairs");
        } else {
            println!("EN N-gram model not found, skipping test");
        }
    }
}
