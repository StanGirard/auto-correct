//! Builder for compact N-gram files
//!
//! This module provides utilities to build the compact N-gram format
//! from extracted N-gram data (HashMap or iterator).

use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead, BufReader, BufWriter, Write};
use std::path::Path;

use super::compact_model::{NgramHeader, HEADER_SIZE, ENTRY_SIZE, VERSION};

/// Builder for compact N-gram files
pub struct CompactNgramBuilder {
    unigrams: Vec<(String, u64)>,
    bigrams: Vec<(String, u64)>,
    trigrams: Vec<(String, u64)>,
    total_tokens: u64,
}

impl CompactNgramBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        CompactNgramBuilder {
            unigrams: Vec::new(),
            bigrams: Vec::new(),
            trigrams: Vec::new(),
            total_tokens: 0,
        }
    }

    /// Set total token count
    pub fn set_total_tokens(&mut self, count: u64) {
        self.total_tokens = count;
    }

    /// Add a unigram
    pub fn add_unigram(&mut self, word: String, count: u64) {
        self.unigrams.push((word, count));
    }

    /// Add a bigram
    pub fn add_bigram(&mut self, ngram: String, count: u64) {
        self.bigrams.push((ngram, count));
    }

    /// Add a trigram
    pub fn add_trigram(&mut self, ngram: String, count: u64) {
        self.trigrams.push((ngram, count));
    }

    /// Load from HashMaps (NgramData format)
    pub fn from_hashmaps(
        unigrams: &HashMap<String, u64>,
        bigrams: &HashMap<String, u64>,
        trigrams: &HashMap<String, u64>,
        total_tokens: u64,
    ) -> Self {
        let mut builder = CompactNgramBuilder::new();
        builder.total_tokens = total_tokens;

        for (word, &count) in unigrams {
            builder.unigrams.push((word.clone(), count));
        }

        for (ngram, &count) in bigrams {
            builder.bigrams.push((ngram.clone(), count));
        }

        for (ngram, &count) in trigrams {
            builder.trigrams.push((ngram.clone(), count));
        }

        builder
    }

    /// Build and write to file
    pub fn build(mut self, path: &Path) -> io::Result<BuildStats> {
        // Sort all N-grams alphabetically for binary search
        self.unigrams.sort_by(|a, b| a.0.cmp(&b.0));
        self.bigrams.sort_by(|a, b| a.0.cmp(&b.0));
        self.trigrams.sort_by(|a, b| a.0.cmp(&b.0));

        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);

        // Calculate section sizes and offsets
        let unigram_entry_size = self.unigrams.len() * ENTRY_SIZE;
        let unigram_strings_size: usize = self.unigrams.iter().map(|(s, _)| s.len()).sum();

        let bigram_entry_size = self.bigrams.len() * ENTRY_SIZE;
        let bigram_strings_size: usize = self.bigrams.iter().map(|(s, _)| s.len()).sum();

        let trigram_entry_size = self.trigrams.len() * ENTRY_SIZE;
        let trigram_strings_size: usize = self.trigrams.iter().map(|(s, _)| s.len()).sum();

        let unigram_offset = HEADER_SIZE;
        let bigram_offset = unigram_offset + unigram_entry_size + unigram_strings_size;
        let trigram_offset = bigram_offset + bigram_entry_size + bigram_strings_size;

        // Write header
        let header = NgramHeader {
            version: VERSION,
            unigram_count: self.unigrams.len() as u64,
            bigram_count: self.bigrams.len() as u64,
            trigram_count: self.trigrams.len() as u64,
            total_tokens: self.total_tokens,
            unigram_offset: unigram_offset as u64,
            bigram_offset: bigram_offset as u64,
            trigram_offset: trigram_offset as u64,
        };
        writer.write_all(&header.to_bytes())?;

        // Write sections
        Self::write_section(&mut writer, &self.unigrams)?;
        Self::write_section(&mut writer, &self.bigrams)?;
        Self::write_section(&mut writer, &self.trigrams)?;

        writer.flush()?;

        let total_size = trigram_offset + trigram_entry_size + trigram_strings_size;

        Ok(BuildStats {
            unigram_count: self.unigrams.len(),
            bigram_count: self.bigrams.len(),
            trigram_count: self.trigrams.len(),
            total_tokens: self.total_tokens,
            file_size: total_size,
        })
    }

    /// Write a section (entry table + strings)
    fn write_section<W: Write>(writer: &mut W, entries: &[(String, u64)]) -> io::Result<()> {
        // First pass: write entry table
        let mut string_offset = 0u32;
        for (ngram, count) in entries {
            let entry_bytes = Self::entry_to_bytes(string_offset, ngram.len() as u16, *count);
            writer.write_all(&entry_bytes)?;
            string_offset += ngram.len() as u32;
        }

        // Second pass: write string data
        for (ngram, _) in entries {
            writer.write_all(ngram.as_bytes())?;
        }

        Ok(())
    }

    /// Convert entry to bytes
    fn entry_to_bytes(string_offset: u32, string_len: u16, count: u64) -> [u8; ENTRY_SIZE] {
        let mut buf = [0u8; ENTRY_SIZE];
        buf[0..4].copy_from_slice(&string_offset.to_le_bytes());
        buf[4..6].copy_from_slice(&string_len.to_le_bytes());
        // bytes 6-7 are padding
        buf[8..16].copy_from_slice(&count.to_le_bytes());
        buf
    }
}

impl Default for CompactNgramBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics from building a compact N-gram file
#[derive(Debug, Clone)]
pub struct BuildStats {
    pub unigram_count: usize,
    pub bigram_count: usize,
    pub trigram_count: usize,
    pub total_tokens: u64,
    pub file_size: usize,
}

impl std::fmt::Display for BuildStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Built compact N-gram file:\n\
             - Unigrams: {}\n\
             - Bigrams: {}\n\
             - Trigrams: {}\n\
             - Total tokens: {}\n\
             - File size: {} bytes ({:.2} GB)",
            self.unigram_count,
            self.bigram_count,
            self.trigram_count,
            self.total_tokens,
            self.file_size,
            self.file_size as f64 / 1_073_741_824.0
        )
    }
}

/// Streaming builder for large N-gram files
///
/// This builder processes sorted TSV files in a streaming fashion,
/// using minimal memory regardless of file size.
pub struct StreamingNgramBuilder;

impl StreamingNgramBuilder {
    /// Build a compact N-gram file from sorted TSV files
    ///
    /// # Arguments
    /// * `unigram_tsv` - Path to sorted unigrams TSV (ngram<TAB>count)
    /// * `bigram_tsv` - Path to sorted bigrams TSV
    /// * `trigram_tsv` - Path to sorted trigrams TSV
    /// * `output_path` - Path to output .bin file
    ///
    /// # Returns
    /// Build statistics including counts and file size
    pub fn build_from_sorted_tsv(
        unigram_tsv: Option<&Path>,
        bigram_tsv: Option<&Path>,
        trigram_tsv: Option<&Path>,
        output_path: &Path,
    ) -> io::Result<BuildStats> {
        eprintln!("Phase 1: Counting entries and calculating sizes...");

        // Phase 1: Count entries and total string bytes for each section
        let (uni_count, uni_strings) = Self::count_tsv(unigram_tsv)?;
        let (bi_count, bi_strings) = Self::count_tsv(bigram_tsv)?;
        let (tri_count, tri_strings) = Self::count_tsv(trigram_tsv)?;

        eprintln!("  Unigrams: {} entries, {} bytes strings", uni_count, uni_strings);
        eprintln!("  Bigrams: {} entries, {} bytes strings", bi_count, bi_strings);
        eprintln!("  Trigrams: {} entries, {} bytes strings", tri_count, tri_strings);

        // Calculate section sizes
        let uni_entry_size = uni_count as usize * ENTRY_SIZE;
        let bi_entry_size = bi_count as usize * ENTRY_SIZE;
        let tri_entry_size = tri_count as usize * ENTRY_SIZE;

        // Calculate offsets
        let unigram_offset = HEADER_SIZE;
        let bigram_offset = unigram_offset + uni_entry_size + uni_strings as usize;
        let trigram_offset = bigram_offset + bi_entry_size + bi_strings as usize;
        let total_size = trigram_offset + tri_entry_size + tri_strings as usize;

        eprintln!("  Total file size: {:.2} GB", total_size as f64 / 1_073_741_824.0);

        // Calculate total tokens from unigram counts
        let total_tokens = Self::sum_counts(unigram_tsv)?;

        eprintln!("Phase 2: Writing compact file...");

        // Create output file
        let file = File::create(output_path)?;
        let mut writer = BufWriter::with_capacity(16 * 1024 * 1024, file); // 16MB buffer

        // Write header
        let header = NgramHeader {
            version: VERSION,
            unigram_count: uni_count,
            bigram_count: bi_count,
            trigram_count: tri_count,
            total_tokens,
            unigram_offset: unigram_offset as u64,
            bigram_offset: bigram_offset as u64,
            trigram_offset: trigram_offset as u64,
        };
        writer.write_all(&header.to_bytes())?;

        // Write each section
        eprintln!("  Writing unigrams...");
        Self::write_section_from_tsv(&mut writer, unigram_tsv)?;

        eprintln!("  Writing bigrams...");
        Self::write_section_from_tsv(&mut writer, bigram_tsv)?;

        eprintln!("  Writing trigrams...");
        Self::write_section_from_tsv(&mut writer, trigram_tsv)?;

        writer.flush()?;

        Ok(BuildStats {
            unigram_count: uni_count as usize,
            bigram_count: bi_count as usize,
            trigram_count: tri_count as usize,
            total_tokens,
            file_size: total_size,
        })
    }

    /// Count entries and total string bytes in a TSV file
    fn count_tsv(path: Option<&Path>) -> io::Result<(u64, u64)> {
        let path = match path {
            Some(p) if p.exists() => p,
            _ => return Ok((0, 0)),
        };

        let file = File::open(path)?;
        let reader = BufReader::with_capacity(8 * 1024 * 1024, file); // 8MB buffer

        let mut count = 0u64;
        let mut string_bytes = 0u64;
        let mut lines_processed = 0u64;

        for line in reader.lines() {
            let line = line?;
            if let Some(tab_pos) = line.find('\t') {
                let ngram = &line[..tab_pos];
                if !ngram.is_empty() && ngram.len() < 500 {
                    count += 1;
                    string_bytes += ngram.len() as u64;
                }
            }

            lines_processed += 1;
            if lines_processed % 10_000_000 == 0 {
                eprint!("\r    Counted {} million entries...", lines_processed / 1_000_000);
            }
        }
        if lines_processed >= 10_000_000 {
            eprintln!();
        }

        Ok((count, string_bytes))
    }

    /// Sum all counts in a TSV file (for total_tokens)
    fn sum_counts(path: Option<&Path>) -> io::Result<u64> {
        let path = match path {
            Some(p) if p.exists() => p,
            _ => return Ok(0),
        };

        let file = File::open(path)?;
        let reader = BufReader::with_capacity(8 * 1024 * 1024, file);

        let mut total = 0u64;

        for line in reader.lines() {
            let line = line?;
            if let Some(tab_pos) = line.find('\t') {
                if let Ok(count) = line[tab_pos + 1..].trim().parse::<u64>() {
                    total = total.saturating_add(count);
                }
            }
        }

        Ok(total)
    }

    /// Write a section from a sorted TSV file using two-pass streaming
    fn write_section_from_tsv<W: Write>(writer: &mut W, path: Option<&Path>) -> io::Result<()> {
        let path = match path {
            Some(p) if p.exists() => p,
            _ => return Ok(()),
        };

        // Pass 1: Write entry table (16 bytes per entry)
        {
            let file = File::open(path)?;
            let reader = BufReader::with_capacity(8 * 1024 * 1024, file);

            let mut string_offset = 0u32;
            let mut entries_written = 0u64;

            for line in reader.lines() {
                let line = line?;
                if let Some(tab_pos) = line.find('\t') {
                    let ngram = &line[..tab_pos];
                    let count_str = &line[tab_pos + 1..];

                    if ngram.is_empty() || ngram.len() >= 500 {
                        continue;
                    }

                    let count: u64 = count_str.trim().parse().unwrap_or(0);
                    if count == 0 {
                        continue;
                    }

                    // Write entry record
                    let entry_bytes = Self::entry_to_bytes(string_offset, ngram.len() as u16, count);
                    writer.write_all(&entry_bytes)?;

                    string_offset += ngram.len() as u32;
                    entries_written += 1;

                    if entries_written % 10_000_000 == 0 {
                        eprint!("\r      Entries: {} million...", entries_written / 1_000_000);
                    }
                }
            }
            if entries_written >= 10_000_000 {
                eprintln!();
            }
            eprintln!("      {} entries written", entries_written);
        }

        // Pass 2: Write strings
        {
            let file = File::open(path)?;
            let reader = BufReader::with_capacity(8 * 1024 * 1024, file);

            let mut strings_written = 0u64;

            for line in reader.lines() {
                let line = line?;
                if let Some(tab_pos) = line.find('\t') {
                    let ngram = &line[..tab_pos];
                    let count_str = &line[tab_pos + 1..];

                    if ngram.is_empty() || ngram.len() >= 500 {
                        continue;
                    }

                    let count: u64 = count_str.trim().parse().unwrap_or(0);
                    if count == 0 {
                        continue;
                    }

                    // Write string bytes
                    writer.write_all(ngram.as_bytes())?;
                    strings_written += ngram.len() as u64;

                    if strings_written % 100_000_000 == 0 {
                        eprint!("\r      Strings: {} MB...", strings_written / 1_000_000);
                    }
                }
            }
            if strings_written >= 100_000_000 {
                eprintln!();
            }
            eprintln!("      {} bytes of strings written", strings_written);
        }

        Ok(())
    }

    /// Convert entry to bytes
    fn entry_to_bytes(string_offset: u32, string_len: u16, count: u64) -> [u8; ENTRY_SIZE] {
        let mut buf = [0u8; ENTRY_SIZE];
        buf[0..4].copy_from_slice(&string_offset.to_le_bytes());
        buf[4..6].copy_from_slice(&string_len.to_le_bytes());
        // bytes 6-7 are padding
        buf[8..16].copy_from_slice(&count.to_le_bytes());
        buf
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::language_model::compact_model::CompactNgramModel;
    use tempfile::NamedTempFile;

    #[test]
    fn test_build_and_read() {
        let mut builder = CompactNgramBuilder::new();
        builder.set_total_tokens(10000);

        // Add some test data
        builder.add_unigram("the".to_string(), 1000);
        builder.add_unigram("house".to_string(), 500);
        builder.add_unigram("their".to_string(), 200);
        builder.add_unigram("there".to_string(), 300);

        builder.add_bigram("their house".to_string(), 100);
        builder.add_bigram("there house".to_string(), 10);
        builder.add_bigram("the house".to_string(), 400);

        builder.add_trigram("to their house".to_string(), 50);
        builder.add_trigram("to there house".to_string(), 5);

        // Build to file
        let file = NamedTempFile::new().unwrap();
        let stats = builder.build(file.path()).unwrap();

        assert_eq!(stats.unigram_count, 4);
        assert_eq!(stats.bigram_count, 3);
        assert_eq!(stats.trigram_count, 2);

        // Read back
        let model = CompactNgramModel::open(file.path()).unwrap();

        // Test unigrams
        assert_eq!(model.get_unigram("the"), Some(1000));
        assert_eq!(model.get_unigram("house"), Some(500));
        assert_eq!(model.get_unigram("their"), Some(200));
        assert_eq!(model.get_unigram("there"), Some(300));
        assert_eq!(model.get_unigram("unknown"), None);

        // Test bigrams
        assert_eq!(model.get_bigram("their", "house"), Some(100));
        assert_eq!(model.get_bigram("there", "house"), Some(10));
        assert_eq!(model.get_bigram("the", "house"), Some(400));
        assert_eq!(model.get_bigram("unknown", "word"), None);

        // Test trigrams
        assert_eq!(model.get_trigram("to", "their", "house"), Some(50));
        assert_eq!(model.get_trigram("to", "there", "house"), Some(5));
    }

    #[test]
    fn test_from_hashmaps() {
        let mut unigrams = HashMap::new();
        unigrams.insert("word1".to_string(), 100);
        unigrams.insert("word2".to_string(), 200);

        let bigrams = HashMap::new();
        let trigrams = HashMap::new();

        let builder = CompactNgramBuilder::from_hashmaps(&unigrams, &bigrams, &trigrams, 1000);

        let file = NamedTempFile::new().unwrap();
        let stats = builder.build(file.path()).unwrap();

        assert_eq!(stats.unigram_count, 2);
        assert_eq!(stats.bigram_count, 0);
        assert_eq!(stats.trigram_count, 0);
    }

    #[test]
    fn test_probability_calculation() {
        let mut builder = CompactNgramBuilder::new();
        builder.set_total_tokens(10000);

        builder.add_unigram("their".to_string(), 200);
        builder.add_unigram("there".to_string(), 300);
        builder.add_unigram("house".to_string(), 500);
        builder.add_unigram("to".to_string(), 1000);

        builder.add_bigram("their house".to_string(), 100);
        builder.add_bigram("there house".to_string(), 10);
        builder.add_bigram("to their".to_string(), 80);
        builder.add_bigram("to there".to_string(), 20);

        builder.add_trigram("to their house".to_string(), 50);
        builder.add_trigram("to there house".to_string(), 5);

        let file = NamedTempFile::new().unwrap();
        builder.build(file.path()).unwrap();

        let model = CompactNgramModel::open(file.path()).unwrap();

        // "their" should be more likely after "to" when followed by "house"
        let ratio = model.compare_words("their", "there", Some("to"), None);
        assert!(ratio > 1.0, "their should be more likely than there after 'to'");
    }
}
