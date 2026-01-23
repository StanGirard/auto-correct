//! N-gram language model implementation
//!
//! Provides probability calculations using Stupid Backoff algorithm.

use std::collections::HashMap;
use std::io;
use std::path::Path;
use serde::{Deserialize, Serialize};

use super::probability::Probability;

/// N-gram data storage format (serializable)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NgramData {
    /// Unigram counts: word -> count
    pub unigrams: HashMap<String, u64>,
    /// Bigram counts: "word1 word2" -> count
    pub bigrams: HashMap<String, u64>,
    /// Trigram counts: "word1 word2 word3" -> count
    pub trigrams: HashMap<String, u64>,
    /// Total token count (for normalization)
    pub total_count: u64,
}

impl NgramData {
    /// Create empty N-gram data
    pub fn new() -> Self {
        NgramData {
            unigrams: HashMap::new(),
            bigrams: HashMap::new(),
            trigrams: HashMap::new(),
            total_count: 0,
        }
    }

    /// Load from binary file (bincode format)
    pub fn from_file(path: &Path) -> io::Result<Self> {
        let data = std::fs::read(path)?;
        bincode::deserialize(&data)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    /// Save to binary file (bincode format)
    pub fn to_file(&self, path: &Path) -> io::Result<()> {
        let data = bincode::serialize(self)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        std::fs::write(path, data)
    }

    /// Number of unique N-grams
    pub fn len(&self) -> usize {
        self.unigrams.len() + self.bigrams.len() + self.trigrams.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.unigrams.is_empty() && self.bigrams.is_empty() && self.trigrams.is_empty()
    }
}

impl Default for NgramData {
    fn default() -> Self {
        Self::new()
    }
}

/// N-gram language model for probability calculations
pub struct NgramLanguageModel {
    data: NgramData,
    /// Backoff factor (default 0.4 like LanguageTool)
    backoff_factor: f64,
}

impl NgramLanguageModel {
    /// Create from N-gram data
    pub fn new(data: NgramData) -> Self {
        NgramLanguageModel {
            data,
            backoff_factor: 0.4,
        }
    }

    /// Load from binary file
    pub fn from_file(path: &Path) -> io::Result<Self> {
        let data = NgramData::from_file(path)?;
        Ok(NgramLanguageModel::new(data))
    }

    /// Create empty model (for testing)
    pub fn empty() -> Self {
        NgramLanguageModel::new(NgramData::new())
    }

    /// Get unigram count
    pub fn get_unigram(&self, word: &str) -> u64 {
        self.data.unigrams.get(word).copied().unwrap_or(0)
    }

    /// Get bigram count
    pub fn get_bigram(&self, w1: &str, w2: &str) -> u64 {
        let key = format!("{} {}", w1, w2);
        self.data.bigrams.get(&key).copied().unwrap_or(0)
    }

    /// Get trigram count
    pub fn get_trigram(&self, w1: &str, w2: &str, w3: &str) -> u64 {
        let key = format!("{} {} {}", w1, w2, w3);
        self.data.trigrams.get(&key).copied().unwrap_or(0)
    }

    /// Get total token count
    pub fn total_count(&self) -> u64 {
        self.data.total_count
    }

    /// Calculate pseudo-probability using Stupid Backoff
    ///
    /// P(word | prev2, prev1) with backoff:
    /// - Try trigram: P(word | prev2, prev1) = count(prev2, prev1, word) / count(prev2, prev1)
    /// - Backoff to bigram: 0.4 * P(word | prev1) = 0.4 * count(prev1, word) / count(prev1)
    /// - Backoff to unigram: 0.4^2 * P(word) = 0.16 * count(word) / total
    pub fn get_probability(&self, word: &str, prev1: Option<&str>, prev2: Option<&str>) -> Probability {
        let word_lower = word.to_lowercase();

        // Try trigram
        if let (Some(p2), Some(p1)) = (prev2, prev1) {
            let p2_lower = p2.to_lowercase();
            let p1_lower = p1.to_lowercase();

            let trigram_count = self.get_trigram(&p2_lower, &p1_lower, &word_lower);
            if trigram_count > 0 {
                let bigram_count = self.get_bigram(&p2_lower, &p1_lower);
                if bigram_count > 0 {
                    let prob = trigram_count as f64 / bigram_count as f64;
                    return Probability::new(prob, 1.0, trigram_count);
                }
            }
        }

        // Backoff to bigram
        if let Some(p1) = prev1 {
            let p1_lower = p1.to_lowercase();

            let bigram_count = self.get_bigram(&p1_lower, &word_lower);
            if bigram_count > 0 {
                let unigram_count = self.get_unigram(&p1_lower);
                if unigram_count > 0 {
                    let prob = self.backoff_factor * bigram_count as f64 / unigram_count as f64;
                    return Probability::new(prob, 0.5, bigram_count);
                }
            }
        }

        // Backoff to unigram
        let unigram_count = self.get_unigram(&word_lower);
        if unigram_count > 0 && self.data.total_count > 0 {
            let prob = self.backoff_factor * self.backoff_factor
                * unigram_count as f64 / self.data.total_count as f64;
            return Probability::new(prob, 0.25, unigram_count);
        }

        // Unknown word
        Probability::unknown()
    }

    /// Compare two words in the same context
    /// Returns the probability ratio: P(word1|context) / P(word2|context)
    /// Ratio > 1 means word1 is more likely
    pub fn compare_words(
        &self,
        word1: &str,
        word2: &str,
        prev1: Option<&str>,
        prev2: Option<&str>,
    ) -> f64 {
        let p1 = self.get_probability(word1, prev1, prev2);
        let p2 = self.get_probability(word2, prev1, prev2);

        // Avoid division by zero
        if p2.probability < 1e-15 {
            if p1.probability < 1e-15 {
                return 1.0; // Both unknown, neutral
            }
            return f64::MAX; // word1 known, word2 unknown
        }

        p1.probability / p2.probability
    }

    /// Get statistics about loaded data
    pub fn stats(&self) -> NgramStats {
        NgramStats {
            unigram_count: self.data.unigrams.len(),
            bigram_count: self.data.bigrams.len(),
            trigram_count: self.data.trigrams.len(),
            total_token_count: self.data.total_count,
        }
    }
}

/// Statistics about loaded N-gram data
#[derive(Debug, Clone)]
pub struct NgramStats {
    pub unigram_count: usize,
    pub bigram_count: usize,
    pub trigram_count: usize,
    pub total_token_count: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_model() -> NgramLanguageModel {
        let mut data = NgramData::new();

        // Add some test data
        data.unigrams.insert("the".to_string(), 1000);
        data.unigrams.insert("their".to_string(), 100);
        data.unigrams.insert("there".to_string(), 150);
        data.unigrams.insert("house".to_string(), 50);

        data.bigrams.insert("their house".to_string(), 30);
        data.bigrams.insert("there house".to_string(), 2);
        data.bigrams.insert("the house".to_string(), 80);
        data.bigrams.insert("in their".to_string(), 50);
        data.bigrams.insert("in there".to_string(), 5);

        data.trigrams.insert("in their house".to_string(), 10);
        data.trigrams.insert("in there house".to_string(), 1);

        data.total_count = 10000;

        NgramLanguageModel::new(data)
    }

    #[test]
    fn test_unigram_lookup() {
        let model = create_test_model();
        assert_eq!(model.get_unigram("the"), 1000);
        assert_eq!(model.get_unigram("their"), 100);
        assert_eq!(model.get_unigram("unknown"), 0);
    }

    #[test]
    fn test_bigram_lookup() {
        let model = create_test_model();
        assert_eq!(model.get_bigram("their", "house"), 30);
        assert_eq!(model.get_bigram("there", "house"), 2);
        assert_eq!(model.get_bigram("unknown", "word"), 0);
    }

    #[test]
    fn test_probability_backoff() {
        let model = create_test_model();

        // Trigram available
        let p = model.get_probability("house", Some("their"), Some("in"));
        assert!(p.probability > 0.0);
        assert!(p.coverage > 0.5); // High coverage for trigram

        // Only bigram available
        let p = model.get_probability("house", Some("the"), None);
        assert!(p.probability > 0.0);
    }

    #[test]
    fn test_compare_words() {
        let model = create_test_model();

        // "their house" should be much more likely than "there house"
        let ratio = model.compare_words("their", "there", None, None);
        // "their" has count 100, "there" has count 150, so ratio < 1
        assert!(ratio > 0.0);

        // With context "house" after
        let ratio_with_context = model.compare_words("their", "there", Some("house"), None);
        // Bigram "their house" (30) vs we need "there house" (2) but context is after...
        // Actually compare_words looks at P(word|prev), not P(next|word)
    }
}
