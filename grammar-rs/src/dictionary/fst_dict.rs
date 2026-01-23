//! FST-based dictionary for ultra-fast lookups
//!
//! Uses the `fst` crate to store words in a Finite State Transducer.
//! Benefits:
//! - O(key_length) lookup time
//! - Memory-efficient (shares common prefixes)
//! - Can be memory-mapped for instant loading

use fst::{IntoStreamer, Set, SetBuilder, Streamer};
use std::fs::File;
use std::io::{self, BufRead, BufReader, BufWriter, Write};
use std::path::Path;

/// Error type for dictionary operations
#[derive(Debug)]
pub enum DictError {
    Io(io::Error),
    Fst(fst::Error),
    NotSorted(String),
}

impl From<io::Error> for DictError {
    fn from(e: io::Error) -> Self {
        DictError::Io(e)
    }
}

impl From<fst::Error> for DictError {
    fn from(e: fst::Error) -> Self {
        DictError::Fst(e)
    }
}

impl std::fmt::Display for DictError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DictError::Io(e) => write!(f, "IO error: {}", e),
            DictError::Fst(e) => write!(f, "FST error: {}", e),
            DictError::NotSorted(w) => write!(f, "Words not sorted at: {}", w),
        }
    }
}

impl std::error::Error for DictError {}

/// FST-based dictionary for efficient word lookups
pub struct FstDictionary {
    set: Set<Vec<u8>>,
    word_count: usize,
}

impl FstDictionary {
    /// Create an empty dictionary
    pub fn empty() -> Self {
        let builder = SetBuilder::memory();
        let set = builder.into_set();
        Self { set, word_count: 0 }
    }

    /// Build dictionary from an iterator of words
    ///
    /// **Important**: Words must be provided in lexicographic (sorted) order!
    /// Use `from_unsorted` if your words are not sorted.
    pub fn from_sorted_iter<I, S>(words: I) -> Result<Self, DictError>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let mut builder = SetBuilder::memory();
        let mut count = 0;
        let mut last_word = String::new();

        for word in words {
            let word = word.as_ref().to_lowercase();
            if word.is_empty() {
                continue;
            }

            // Check sorting
            if word < last_word {
                return Err(DictError::NotSorted(word));
            }

            // Skip duplicates
            if word == last_word {
                continue;
            }

            builder.insert(&word)?;
            last_word = word;
            count += 1;
        }

        let set = builder.into_set();
        Ok(Self { set, word_count: count })
    }

    /// Build dictionary from unsorted words
    ///
    /// This sorts the words first, so it's slower than `from_sorted_iter`.
    pub fn from_iter<I, S>(words: I) -> Result<Self, DictError>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let mut sorted: Vec<String> = words
            .into_iter()
            .map(|w| w.as_ref().to_lowercase())
            .filter(|w| !w.is_empty())
            .collect();

        sorted.sort();
        sorted.dedup();

        Self::from_sorted_iter(sorted)
    }

    /// Load dictionary from a wordlist file (one word per line)
    ///
    /// The file should contain sorted, lowercase words.
    pub fn from_wordlist<P: AsRef<Path>>(path: P) -> Result<Self, DictError> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        let words = reader
            .lines()
            .filter_map(|l| l.ok())
            .map(|l| l.trim().to_string())
            .filter(|l| !l.is_empty() && !l.starts_with('#'));

        Self::from_sorted_iter(words)
    }

    /// Load dictionary from a pre-built FST file
    pub fn from_fst<P: AsRef<Path>>(path: P) -> Result<Self, DictError> {
        let data = std::fs::read(path)?;
        let set = Set::new(data)?;
        let word_count = set.len();
        Ok(Self { set, word_count })
    }

    /// Save dictionary to FST file for fast loading
    pub fn save_fst<P: AsRef<Path>>(&self, path: P) -> Result<(), DictError> {
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);
        writer.write_all(self.set.as_fst().as_bytes())?;
        Ok(())
    }

    /// Check if a word is in the dictionary
    #[inline]
    pub fn contains(&self, word: &str) -> bool {
        self.set.contains(word.to_lowercase())
    }

    /// Check if a lowercase word is in the dictionary (faster, no allocation)
    #[inline]
    pub fn contains_lowercase(&self, word: &str) -> bool {
        self.set.contains(word)
    }

    /// Get the number of words in the dictionary
    pub fn len(&self) -> usize {
        self.word_count
    }

    /// Check if dictionary is empty
    pub fn is_empty(&self) -> bool {
        self.word_count == 0
    }

    /// Find all words with a given prefix
    pub fn words_with_prefix(&self, prefix: &str) -> Vec<String> {
        let prefix_lower = prefix.to_lowercase();
        let mut results = Vec::new();

        // Create a range that matches the prefix
        let mut stream = self.set.range()
            .ge(&prefix_lower)
            .lt(&format!("{}~", prefix_lower)) // ~ is after z in ASCII
            .into_stream();

        while let Some(word) = stream.next() {
            if let Ok(s) = std::str::from_utf8(word) {
                if s.starts_with(&prefix_lower) {
                    results.push(s.to_string());
                } else {
                    break;
                }
            }
        }

        results
    }

    /// Get memory usage estimate in bytes
    pub fn memory_usage(&self) -> usize {
        self.set.as_fst().as_bytes().len()
    }
}

impl Default for FstDictionary {
    fn default() -> Self {
        Self::empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_dictionary() {
        let dict = FstDictionary::empty();
        assert!(dict.is_empty());
        assert!(!dict.contains("hello"));
    }

    #[test]
    fn test_from_sorted_iter() {
        let words = ["apple", "banana", "cherry"];
        let dict = FstDictionary::from_sorted_iter(words).unwrap();

        assert_eq!(dict.len(), 3);
        assert!(dict.contains("apple"));
        assert!(dict.contains("APPLE")); // case insensitive
        assert!(dict.contains("banana"));
        assert!(!dict.contains("grape"));
    }

    #[test]
    fn test_from_unsorted_iter() {
        let words = ["cherry", "apple", "banana", "apple"]; // unsorted + duplicate
        let dict = FstDictionary::from_iter(words).unwrap();

        assert_eq!(dict.len(), 3); // duplicate removed
        assert!(dict.contains("apple"));
        assert!(dict.contains("banana"));
        assert!(dict.contains("cherry"));
    }

    #[test]
    fn test_words_with_prefix() {
        let words = ["apple", "application", "apply", "banana", "band"];
        let dict = FstDictionary::from_sorted_iter(words).unwrap();

        let app_words = dict.words_with_prefix("app");
        assert_eq!(app_words.len(), 3);
        assert!(app_words.contains(&"apple".to_string()));
        assert!(app_words.contains(&"application".to_string()));
        assert!(app_words.contains(&"apply".to_string()));

        let ban_words = dict.words_with_prefix("ban");
        assert_eq!(ban_words.len(), 2);
    }

    #[test]
    fn test_sorted_error() {
        let words = ["banana", "apple"]; // not sorted!
        let result = FstDictionary::from_sorted_iter(words);
        assert!(matches!(result, Err(DictError::NotSorted(_))));
    }
}
