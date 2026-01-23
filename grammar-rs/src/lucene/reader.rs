//! High-level N-gram index reader
//!
//! Combines compound file and stored fields readers to extract ngram→count mappings.
//! Supports both compound (.cfs/.cfe) and non-compound Lucene indexes.

use std::collections::HashMap;
use std::fs;
use std::io::{self, BufRead, BufReader};
use std::path::Path;

use super::codec::CodecHeader;
use super::stored::StoredFieldsReader;
use super::vint::read_vint;
use flate2::read::DeflateDecoder;
use lz4_flex::decompress;

/// A single N-gram entry
#[derive(Debug, Clone)]
pub struct NgramEntry {
    pub ngram: String,
    pub count: u64,
}

/// N-gram index reader for a single n-gram level (1grams, 2grams, etc.)
pub struct NgramIndexReader {
    /// All ngram entries loaded from the index
    entries: HashMap<String, u64>,
    /// Total token count (from 1grams totalTokenCount field)
    total_count: u64,
}

impl NgramIndexReader {
    /// Open an N-gram index from a directory (e.g., "ngrams-en/1grams")
    pub fn open(index_dir: &Path) -> io::Result<Self> {
        // Find the segment name
        let segment_name = Self::find_segment(index_dir)?;

        // Try compound file first
        let cfs_path = index_dir.join(format!("{}.cfs", segment_name));
        if cfs_path.exists() {
            return Self::open_compound(index_dir, &segment_name);
        }

        // Otherwise, try non-compound format
        Self::open_non_compound(index_dir, &segment_name)
    }

    /// Open from compound file format
    fn open_compound(index_dir: &Path, segment_name: &str) -> io::Result<Self> {
        use super::compound::CompoundFile;

        let compound = CompoundFile::open_from_dir(index_dir, segment_name)?;

        // Get stored fields data
        let fdt_data = compound.get_stored_fields_data()
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "No .fdt file found"))?;
        let fdx_data = compound.get_stored_fields_index()
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "No .fdx file found"))?;

        // Parse stored fields
        Self::parse_stored_fields(fdt_data, fdx_data)
    }

    /// Open from non-compound format (individual files)
    fn open_non_compound(index_dir: &Path, segment_name: &str) -> io::Result<Self> {
        // Find .fdt file (may have codec suffix like _Lucene41StoredFields)
        let fdt_path = Self::find_file_with_suffix(index_dir, segment_name, ".fdt")?;
        let fdx_path = Self::find_file_with_suffix(index_dir, segment_name, ".fdx")?;

        let fdt_data = fs::read(&fdt_path)?;
        let fdx_data = fs::read(&fdx_path)?;

        Self::parse_stored_fields(&fdt_data, &fdx_data)
    }

    /// Find a file with a given suffix, handling codec naming conventions
    fn find_file_with_suffix(index_dir: &Path, segment_name: &str, suffix: &str) -> io::Result<std::path::PathBuf> {
        // Try direct path first
        let direct_path = index_dir.join(format!("{}{}", segment_name, suffix));
        if direct_path.exists() {
            return Ok(direct_path);
        }

        // Search for files matching pattern
        for entry in fs::read_dir(index_dir)? {
            let entry = entry?;
            let name = entry.file_name();
            let name_str = name.to_string_lossy();

            if name_str.starts_with(segment_name) && name_str.ends_with(suffix) {
                return Ok(entry.path());
            }
        }

        Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("No {} file found for segment {}", suffix, segment_name),
        ))
    }

    /// Parse stored fields to extract ngram → count mappings
    fn parse_stored_fields(fdt_data: &[u8], fdx_data: &[u8]) -> io::Result<Self> {
        let mut entries = HashMap::new();
        let mut total_count = 0u64;

        // Lucene 4.1 stored fields format:
        // Header + Compressed blocks of documents

        let mut pos = 0;

        if fdt_data.len() < 20 {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "FDT file too short"));
        }

        // Parse header
        let header = CodecHeader::parse(fdt_data, &mut pos)?;

        // Check codec version
        let is_lucene41 = header.codec_name.contains("Lucene41") ||
                          header.codec_name.contains("StoredFields");

        if !is_lucene41 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Unsupported stored fields format: {}", header.codec_name),
            ));
        }

        // Read packed integers header (chunk size)
        if pos + 8 > fdt_data.len() {
            return Ok(NgramIndexReader { entries, total_count });
        }

        // PackedInts version (vint)
        let _packed_ints_version = read_vint(fdt_data, &mut pos)?;

        // Chunk size (vint, usually 16384)
        let chunk_size = read_vint(fdt_data, &mut pos)? as usize;

        // Read chunk offsets from FDX file to know where chunks start
        let chunk_offsets = Self::parse_fdx_offsets(fdx_data)?;

        // Process each chunk
        for (chunk_idx, &offset) in chunk_offsets.iter().enumerate() {
            if offset as usize >= fdt_data.len() {
                break;
            }

            // Position at chunk start
            pos = offset as usize;

            // Read chunk header
            let doc_base = match Self::safe_read_vint(fdt_data, &mut pos) {
                Some(v) => v as usize,
                None => continue,
            };

            // Bitmask or numBufferedDocs
            let buffered_docs_info = match Self::safe_read_vint(fdt_data, &mut pos) {
                Some(v) => v,
                None => continue,
            };

            // If high bit is set, it's a slice with single doc
            let (num_docs, is_sliced) = if buffered_docs_info == 0 {
                continue;
            } else if buffered_docs_info & 0x8000_0000_0000_0000 != 0 {
                // Sliced format
                (1, true)
            } else {
                // Number of buffered docs (max chunk_size)
                let nd = buffered_docs_info as usize;
                if nd > chunk_size + 1 {
                    continue;
                }
                (nd, false)
            };

            if num_docs == 0 || num_docs > 20000 {
                continue;
            }

            // Read numStoredFields for each doc
            let mut num_fields = Vec::with_capacity(num_docs);
            for _ in 0..num_docs {
                match Self::safe_read_vint(fdt_data, &mut pos) {
                    Some(n) if n <= 100 => num_fields.push(n as usize),
                    _ => break,
                }
            }

            if num_fields.len() != num_docs {
                continue;
            }

            // Read lengths for each doc
            let mut lengths = Vec::with_capacity(num_docs);
            for _ in 0..num_docs {
                match Self::safe_read_vint(fdt_data, &mut pos) {
                    Some(n) if n <= 10_000_000 => lengths.push(n as usize),
                    _ => break,
                }
            }

            if lengths.len() != num_docs {
                continue;
            }

            // Read compression mode (1 byte)
            if pos >= fdt_data.len() {
                continue;
            }
            let compression_mode = fdt_data[pos];
            pos += 1;

            // Calculate total decompressed size
            let decompressed_size: usize = lengths.iter().sum();

            // Read compressed data length (remaining until next chunk or end)
            let next_offset = if chunk_idx + 1 < chunk_offsets.len() {
                chunk_offsets[chunk_idx + 1] as usize
            } else {
                fdt_data.len() - 16 // Approximate, leave room for footer
            };

            let compressed_len = if next_offset > pos { next_offset - pos } else { 0 };
            if compressed_len == 0 || pos + compressed_len > fdt_data.len() {
                continue;
            }

            let compressed_data = &fdt_data[pos..pos + compressed_len];

            // Decompress based on mode
            let decompressed = match compression_mode {
                0 => {
                    // No compression
                    compressed_data[..decompressed_size.min(compressed_data.len())].to_vec()
                }
                1 | 2 => {
                    // LZ4 (mode 1 = LZ4_FAST, mode 2 = LZ4_HIGH_COMPRESSION)
                    match decompress(compressed_data, decompressed_size) {
                        Ok(data) => data,
                        Err(_) => {
                            // Try DEFLATE as fallback
                            Self::try_deflate_decompress(compressed_data, decompressed_size)
                                .unwrap_or_default()
                        }
                    }
                }
                _ => {
                    // Unknown compression, try LZ4 then DEFLATE
                    decompress(compressed_data, decompressed_size)
                        .or_else(|_| Self::try_deflate_decompress(compressed_data, decompressed_size))
                        .unwrap_or_default()
                }
            };

            if decompressed.is_empty() {
                continue;
            }

            // Parse decompressed document data
            match Self::parse_raw_docs(&decompressed, &num_fields) {
                Ok(doc_entries) => {
                    for (ngram, count) in doc_entries {
                        total_count += count;
                        entries.insert(ngram, count);
                    }
                }
                Err(_) => continue,
            }

            // Print progress every 100 chunks
            if chunk_idx > 0 && chunk_idx % 100 == 0 {
                eprintln!("  Processed {} chunks, {} entries so far...", chunk_idx, entries.len());
            }
        }

        Ok(NgramIndexReader { entries, total_count })
    }

    /// Parse FDX file to get chunk offsets
    fn parse_fdx_offsets(fdx_data: &[u8]) -> io::Result<Vec<u64>> {
        let mut offsets = Vec::new();
        let mut pos = 0;

        // Parse header
        let _header = CodecHeader::parse(fdx_data, &mut pos)?;

        // Read packed block metadata
        // FDX contains packed longs for start pointers

        // Skip to actual offsets (simplified parsing)
        // The format varies, so we'll try to extract reasonable offsets

        // Look for offset patterns (ascending u64 values)
        while pos + 8 <= fdx_data.len() - 16 {
            // Try reading as big-endian u64
            let offset = u64::from_be_bytes([
                fdx_data[pos],
                fdx_data[pos + 1],
                fdx_data[pos + 2],
                fdx_data[pos + 3],
                fdx_data[pos + 4],
                fdx_data[pos + 5],
                fdx_data[pos + 6],
                fdx_data[pos + 7],
            ]);

            if offset > 0 && offset < 100_000_000_000 {
                if offsets.is_empty() || offset > *offsets.last().unwrap() {
                    offsets.push(offset);
                }
            }
            pos += 8;
        }

        // If no offsets found, create artificial ones
        if offsets.is_empty() {
            // Start after header (approximately 40 bytes)
            offsets.push(40);
        }

        Ok(offsets)
    }

    /// Try DEFLATE decompression
    fn try_deflate_decompress(data: &[u8], expected_size: usize) -> io::Result<Vec<u8>> {
        use std::io::Read;

        let mut decoder = DeflateDecoder::new(data);
        let mut decompressed = Vec::with_capacity(expected_size);
        decoder.read_to_end(&mut decompressed)?;
        Ok(decompressed)
    }

    /// Safe vint reading that returns None on error
    fn safe_read_vint(data: &[u8], pos: &mut usize) -> Option<u64> {
        if *pos >= data.len() {
            return None;
        }

        let mut result = 0u64;
        let mut shift = 0;

        while *pos < data.len() {
            let b = data[*pos];
            *pos += 1;

            result |= ((b & 0x7F) as u64) << shift;

            if b & 0x80 == 0 {
                return Some(result);
            }

            shift += 7;
            if shift > 63 {
                return None;
            }
        }

        None
    }


    /// Parse raw document data (after decompression)
    fn parse_raw_docs(data: &[u8], num_fields: &[usize]) -> io::Result<Vec<(String, u64)>> {
        let mut results = Vec::new();
        let mut pos = 0;

        for &nf in num_fields {
            let mut ngram: Option<String> = None;
            let mut count: Option<u64> = None;

            for _ in 0..nf {
                if pos >= data.len() {
                    break;
                }

                // Read field info (vint: fieldNum + type)
                let info = Self::safe_read_vint(data, &mut pos);
                if info.is_none() {
                    break;
                }
                let info = info.unwrap();

                let _field_num = info >> 3;
                let field_type = info & 0x07;

                match field_type {
                    0 => {
                        // String field
                        let len = Self::safe_read_vint(data, &mut pos);
                        if let Some(len) = len {
                            let len = len as usize;
                            if pos + len <= data.len() {
                                if let Ok(s) = std::str::from_utf8(&data[pos..pos + len]) {
                                    if ngram.is_none() {
                                        ngram = Some(s.to_string());
                                    }
                                }
                                pos += len;
                            }
                        }
                    }
                    1 | 2 => {
                        // Numeric field (vint or fixed)
                        let val = Self::safe_read_vint(data, &mut pos);
                        if let Some(v) = val {
                            if count.is_none() {
                                count = Some(v);
                            }
                        }
                    }
                    _ => {
                        // Unknown type, skip
                        let len = Self::safe_read_vint(data, &mut pos).unwrap_or(0) as usize;
                        if pos + len <= data.len() {
                            pos += len;
                        }
                    }
                }
            }

            if let (Some(n), Some(c)) = (ngram, count) {
                if !n.is_empty() && Self::is_valid_ngram(&n) {
                    results.push((n, c));
                }
            }
        }

        Ok(results)
    }

    /// Check if a string looks like a valid ngram (words separated by spaces)
    fn is_valid_ngram(s: &str) -> bool {
        // Must have at least one letter
        if !s.chars().any(|c| c.is_alphabetic()) {
            return false;
        }

        // Each "word" should be reasonable
        for word in s.split_whitespace() {
            if word.is_empty() || word.len() > 50 {
                return false;
            }
        }

        true
    }

    /// Find the segment name in the index directory
    fn find_segment(index_dir: &Path) -> io::Result<String> {
        // Look for .si file which indicates segment name
        for entry in fs::read_dir(index_dir)? {
            let entry = entry?;
            let name = entry.file_name();
            let name_str = name.to_string_lossy();

            if name_str.ends_with(".si") {
                // Extract segment name (e.g., "_1p.si" -> "_1p")
                return Ok(name_str.trim_end_matches(".si").to_string());
            }
        }

        // Fallback: look for .fdt file
        for entry in fs::read_dir(index_dir)? {
            let entry = entry?;
            let name = entry.file_name();
            let name_str = name.to_string_lossy();

            if name_str.ends_with(".fdt") {
                // Extract segment name (e.g., "_1p.fdt" -> "_1p")
                let segment = name_str.trim_end_matches(".fdt");
                // Handle Lucene suffix (e.g., "_1p_Lucene41StoredFields.fdt")
                if let Some(idx) = segment.find("_Lucene") {
                    return Ok(segment[..idx].to_string());
                }
                return Ok(segment.to_string());
            }
        }

        // Last resort: assume "_0"
        Ok("_0".to_string())
    }

    /// Get the count for an ngram
    pub fn get(&self, ngram: &str) -> Option<u64> {
        self.entries.get(ngram).copied()
    }

    /// Get all entries
    pub fn entries(&self) -> &HashMap<String, u64> {
        &self.entries
    }

    /// Get total token count (valid for 1grams only)
    pub fn total_count(&self) -> u64 {
        self.total_count
    }

    /// Number of unique ngrams
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Iterate over all entries
    pub fn iter(&self) -> impl Iterator<Item = (&String, &u64)> {
        self.entries.iter()
    }

    /// Filter entries to only keep ngrams containing specified words
    pub fn filter_by_words(&self, words: &std::collections::HashSet<&str>) -> HashMap<String, u64> {
        self.entries
            .iter()
            .filter(|(ngram, _)| {
                ngram.split_whitespace().any(|w| words.contains(w))
            })
            .map(|(k, v)| (k.clone(), *v))
            .collect()
    }
}

/// Combined N-gram reader for 1grams, 2grams, 3grams
pub struct FullNgramReader {
    pub unigrams: NgramIndexReader,
    pub bigrams: NgramIndexReader,
    pub trigrams: NgramIndexReader,
}

impl FullNgramReader {
    /// Open a full ngram index (directory containing 1grams, 2grams, 3grams)
    pub fn open(ngram_dir: &Path) -> io::Result<Self> {
        let unigrams = NgramIndexReader::open(&ngram_dir.join("1grams"))?;
        let bigrams = NgramIndexReader::open(&ngram_dir.join("2grams"))?;
        let trigrams = NgramIndexReader::open(&ngram_dir.join("3grams"))?;

        Ok(FullNgramReader { unigrams, bigrams, trigrams })
    }

    pub fn get_unigram(&self, word: &str) -> Option<u64> {
        self.unigrams.get(word)
    }

    pub fn get_bigram(&self, w1: &str, w2: &str) -> Option<u64> {
        let key = format!("{} {}", w1, w2);
        self.bigrams.get(&key)
    }

    pub fn get_trigram(&self, w1: &str, w2: &str, w3: &str) -> Option<u64> {
        let key = format!("{} {} {}", w1, w2, w3);
        self.trigrams.get(&key)
    }

    pub fn total_count(&self) -> u64 {
        self.unigrams.total_count()
    }

    pub fn stats(&self) -> NgramStats {
        NgramStats {
            unigram_count: self.unigrams.len(),
            bigram_count: self.bigrams.len(),
            trigram_count: self.trigrams.len(),
            total_tokens: self.unigrams.total_count(),
        }
    }
}

/// Statistics about N-gram data
#[derive(Debug, Clone)]
pub struct NgramStats {
    pub unigram_count: usize,
    pub bigram_count: usize,
    pub trigram_count: usize,
    pub total_tokens: u64,
}
