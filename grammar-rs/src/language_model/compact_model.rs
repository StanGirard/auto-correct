//! Compact N-gram model with memory-mapped storage
//!
//! This module provides a memory-efficient N-gram storage format that uses
//! memory-mapped files for instant loading and zero RAM overhead.
//!
//! File format:
//! ```text
//! Header (64 bytes):
//!   - magic: "NGRM" (4 bytes)
//!   - version: u32 (4 bytes)
//!   - unigram_count: u64 (8 bytes)
//!   - bigram_count: u64 (8 bytes)
//!   - trigram_count: u64 (8 bytes)
//!   - total_tokens: u64 (8 bytes)
//!   - unigram_offset: u64 (8 bytes)
//!   - bigram_offset: u64 (8 bytes)
//!   - trigram_offset: u64 (8 bytes)
//!   - padding (0 bytes to align)
//!
//! Section (for each n-gram type):
//!   - Entry table: [(string_offset: u32, string_len: u16, count: u64)] * count
//!   - String data: packed UTF-8 strings (sorted)
//! ```

use std::fs::File;
use std::io::{self, Read, Write};
use std::path::Path;
use memmap2::Mmap;

use super::probability::Probability;

/// Magic bytes for the N-gram file format
pub const MAGIC: &[u8; 4] = b"NGRM";
/// Current format version
pub const VERSION: u32 = 1;
/// Header size in bytes
pub const HEADER_SIZE: usize = 64;
/// Entry size: string_offset (4) + string_len (2) + padding (2) + count (8) = 16 bytes
pub const ENTRY_SIZE: usize = 16;

/// Header structure for the compact N-gram file
#[derive(Debug, Clone, Copy)]
pub struct NgramHeader {
    pub version: u32,
    pub unigram_count: u64,
    pub bigram_count: u64,
    pub trigram_count: u64,
    pub total_tokens: u64,
    pub unigram_offset: u64,
    pub bigram_offset: u64,
    pub trigram_offset: u64,
}

impl NgramHeader {
    /// Read header from bytes
    pub fn from_bytes(data: &[u8]) -> io::Result<Self> {
        if data.len() < HEADER_SIZE {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Header too small"));
        }

        // Check magic
        if &data[0..4] != MAGIC {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid magic bytes"));
        }

        let version = u32::from_le_bytes(data[4..8].try_into().unwrap());
        if version != VERSION {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Unsupported version: {}", version),
            ));
        }

        Ok(NgramHeader {
            version,
            unigram_count: u64::from_le_bytes(data[8..16].try_into().unwrap()),
            bigram_count: u64::from_le_bytes(data[16..24].try_into().unwrap()),
            trigram_count: u64::from_le_bytes(data[24..32].try_into().unwrap()),
            total_tokens: u64::from_le_bytes(data[32..40].try_into().unwrap()),
            unigram_offset: u64::from_le_bytes(data[40..48].try_into().unwrap()),
            bigram_offset: u64::from_le_bytes(data[48..56].try_into().unwrap()),
            trigram_offset: u64::from_le_bytes(data[56..64].try_into().unwrap()),
        })
    }

    /// Write header to bytes
    pub fn to_bytes(&self) -> [u8; HEADER_SIZE] {
        let mut buf = [0u8; HEADER_SIZE];
        buf[0..4].copy_from_slice(MAGIC);
        buf[4..8].copy_from_slice(&self.version.to_le_bytes());
        buf[8..16].copy_from_slice(&self.unigram_count.to_le_bytes());
        buf[16..24].copy_from_slice(&self.bigram_count.to_le_bytes());
        buf[24..32].copy_from_slice(&self.trigram_count.to_le_bytes());
        buf[32..40].copy_from_slice(&self.total_tokens.to_le_bytes());
        buf[40..48].copy_from_slice(&self.unigram_offset.to_le_bytes());
        buf[48..56].copy_from_slice(&self.bigram_offset.to_le_bytes());
        buf[56..64].copy_from_slice(&self.trigram_offset.to_le_bytes());
        buf
    }
}

/// An entry in the N-gram index
#[derive(Debug, Clone, Copy)]
struct NgramEntry {
    string_offset: u32,
    string_len: u16,
    count: u64,
}

impl NgramEntry {
    fn from_bytes(data: &[u8]) -> Self {
        NgramEntry {
            string_offset: u32::from_le_bytes(data[0..4].try_into().unwrap()),
            string_len: u16::from_le_bytes(data[4..6].try_into().unwrap()),
            // bytes 6-7 are padding
            count: u64::from_le_bytes(data[8..16].try_into().unwrap()),
        }
    }

    fn to_bytes(&self) -> [u8; ENTRY_SIZE] {
        let mut buf = [0u8; ENTRY_SIZE];
        buf[0..4].copy_from_slice(&self.string_offset.to_le_bytes());
        buf[4..6].copy_from_slice(&self.string_len.to_le_bytes());
        // bytes 6-7 are padding (already zeroed)
        buf[8..16].copy_from_slice(&self.count.to_le_bytes());
        buf
    }
}

/// Memory-mapped compact N-gram model
///
/// This struct provides O(log n) lookup performance with zero RAM overhead
/// by using memory-mapped files and binary search on sorted arrays.
pub struct CompactNgramModel {
    mmap: Mmap,
    header: NgramHeader,
}

impl CompactNgramModel {
    /// Open a compact N-gram file
    pub fn open(path: &Path) -> io::Result<Self> {
        let file = File::open(path)?;
        let mmap = unsafe { Mmap::map(&file)? };

        if mmap.len() < HEADER_SIZE {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "File too small"));
        }

        let header = NgramHeader::from_bytes(&mmap[..HEADER_SIZE])?;

        Ok(CompactNgramModel { mmap, header })
    }

    /// Get the header information
    pub fn header(&self) -> &NgramHeader {
        &self.header
    }

    /// Get total token count (for probability normalization)
    pub fn total_count(&self) -> u64 {
        self.header.total_tokens
    }

    /// Get unigram count for a word
    pub fn get_unigram(&self, word: &str) -> Option<u64> {
        self.binary_search_section(
            self.header.unigram_offset as usize,
            self.header.unigram_count as usize,
            word,
        )
    }

    /// Get bigram count for two words
    pub fn get_bigram(&self, w1: &str, w2: &str) -> Option<u64> {
        let key = format!("{} {}", w1, w2);
        self.binary_search_section(
            self.header.bigram_offset as usize,
            self.header.bigram_count as usize,
            &key,
        )
    }

    /// Get trigram count for three words
    pub fn get_trigram(&self, w1: &str, w2: &str, w3: &str) -> Option<u64> {
        let key = format!("{} {} {}", w1, w2, w3);
        self.binary_search_section(
            self.header.trigram_offset as usize,
            self.header.trigram_count as usize,
            &key,
        )
    }

    /// Binary search in a section for a key
    fn binary_search_section(&self, section_offset: usize, count: usize, key: &str) -> Option<u64> {
        if count == 0 {
            return None;
        }

        let entry_table_size = count * ENTRY_SIZE;
        let entry_table = &self.mmap[section_offset..section_offset + entry_table_size];
        let strings_offset = section_offset + entry_table_size;

        let mut low = 0usize;
        let mut high = count;

        while low < high {
            let mid = low + (high - low) / 2;
            let entry = NgramEntry::from_bytes(&entry_table[mid * ENTRY_SIZE..]);

            let str_start = strings_offset + entry.string_offset as usize;
            let str_end = str_start + entry.string_len as usize;

            if str_end > self.mmap.len() {
                return None; // Corrupted file
            }

            let ngram = std::str::from_utf8(&self.mmap[str_start..str_end]).ok()?;

            match ngram.cmp(key) {
                std::cmp::Ordering::Equal => return Some(entry.count),
                std::cmp::Ordering::Less => low = mid + 1,
                std::cmp::Ordering::Greater => high = mid,
            }
        }

        None
    }

    /// Calculate probability using Stupid Backoff algorithm
    pub fn get_probability(&self, word: &str, prev1: Option<&str>, prev2: Option<&str>) -> Probability {
        let word_lower = word.to_lowercase();

        // Try trigram
        if let (Some(p2), Some(p1)) = (prev2, prev1) {
            let p2_lower = p2.to_lowercase();
            let p1_lower = p1.to_lowercase();

            if let Some(trigram_count) = self.get_trigram(&p2_lower, &p1_lower, &word_lower) {
                if let Some(bigram_count) = self.get_bigram(&p2_lower, &p1_lower) {
                    if bigram_count > 0 {
                        let prob = trigram_count as f64 / bigram_count as f64;
                        return Probability::new(prob, 1.0, trigram_count);
                    }
                }
            }
        }

        // Backoff to bigram (factor 0.4)
        if let Some(p1) = prev1 {
            let p1_lower = p1.to_lowercase();

            if let Some(bigram_count) = self.get_bigram(&p1_lower, &word_lower) {
                if let Some(unigram_count) = self.get_unigram(&p1_lower) {
                    if unigram_count > 0 {
                        let prob = 0.4 * bigram_count as f64 / unigram_count as f64;
                        return Probability::new(prob, 0.5, bigram_count);
                    }
                }
            }
        }

        // Backoff to unigram (factor 0.4^2 = 0.16)
        if let Some(unigram_count) = self.get_unigram(&word_lower) {
            if self.header.total_tokens > 0 {
                let prob = 0.16 * unigram_count as f64 / self.header.total_tokens as f64;
                return Probability::new(prob, 0.25, unigram_count);
            }
        }

        // Unknown word
        Probability::unknown()
    }

    /// Compare two words in the same context
    pub fn compare_words(
        &self,
        word1: &str,
        word2: &str,
        prev1: Option<&str>,
        prev2: Option<&str>,
    ) -> f64 {
        let p1 = self.get_probability(word1, prev1, prev2);
        let p2 = self.get_probability(word2, prev1, prev2);

        if p2.probability < 1e-15 {
            if p1.probability < 1e-15 {
                return 1.0;
            }
            return f64::MAX;
        }

        p1.probability / p2.probability
    }

    /// Get statistics about the loaded model
    pub fn stats(&self) -> CompactModelStats {
        CompactModelStats {
            unigram_count: self.header.unigram_count as usize,
            bigram_count: self.header.bigram_count as usize,
            trigram_count: self.header.trigram_count as usize,
            total_tokens: self.header.total_tokens,
            file_size: self.mmap.len(),
        }
    }
}

/// Statistics about a compact N-gram model
#[derive(Debug, Clone)]
pub struct CompactModelStats {
    pub unigram_count: usize,
    pub bigram_count: usize,
    pub trigram_count: usize,
    pub total_tokens: u64,
    pub file_size: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_file() -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();

        // Create a simple test file with some unigrams
        let header = NgramHeader {
            version: VERSION,
            unigram_count: 3,
            bigram_count: 0,
            trigram_count: 0,
            total_tokens: 1000,
            unigram_offset: HEADER_SIZE as u64,
            bigram_offset: 0,
            trigram_offset: 0,
        };

        // Write header
        file.write_all(&header.to_bytes()).unwrap();

        // Strings (sorted): "apple", "banana", "cherry"
        let strings = ["apple", "banana", "cherry"];
        let counts = [100u64, 200u64, 50u64];

        // Calculate string offsets
        let entry_table_size = 3 * ENTRY_SIZE;
        let mut string_offset = 0u32;
        let mut entries = Vec::new();
        let mut string_data = Vec::new();

        for (s, &count) in strings.iter().zip(counts.iter()) {
            entries.push(NgramEntry {
                string_offset,
                string_len: s.len() as u16,
                count,
            });
            string_data.extend_from_slice(s.as_bytes());
            string_offset += s.len() as u32;
        }

        // Write entry table
        for entry in &entries {
            file.write_all(&entry.to_bytes()).unwrap();
        }

        // Write string data
        file.write_all(&string_data).unwrap();

        file.flush().unwrap();
        file
    }

    #[test]
    fn test_header_roundtrip() {
        let header = NgramHeader {
            version: VERSION,
            unigram_count: 100,
            bigram_count: 200,
            trigram_count: 300,
            total_tokens: 1_000_000,
            unigram_offset: 64,
            bigram_offset: 1000,
            trigram_offset: 5000,
        };

        let bytes = header.to_bytes();
        let parsed = NgramHeader::from_bytes(&bytes).unwrap();

        assert_eq!(parsed.version, header.version);
        assert_eq!(parsed.unigram_count, header.unigram_count);
        assert_eq!(parsed.bigram_count, header.bigram_count);
        assert_eq!(parsed.trigram_count, header.trigram_count);
        assert_eq!(parsed.total_tokens, header.total_tokens);
    }

    #[test]
    fn test_unigram_lookup() {
        let file = create_test_file();
        let model = CompactNgramModel::open(file.path()).unwrap();

        assert_eq!(model.get_unigram("apple"), Some(100));
        assert_eq!(model.get_unigram("banana"), Some(200));
        assert_eq!(model.get_unigram("cherry"), Some(50));
        assert_eq!(model.get_unigram("unknown"), None);
    }

    #[test]
    fn test_stats() {
        let file = create_test_file();
        let model = CompactNgramModel::open(file.path()).unwrap();

        let stats = model.stats();
        assert_eq!(stats.unigram_count, 3);
        assert_eq!(stats.total_tokens, 1000);
    }
}
