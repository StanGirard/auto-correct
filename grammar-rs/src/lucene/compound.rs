//! Lucene compound file (.cfs/.cfe) parser
//!
//! Compound files bundle multiple index files into a single file for efficiency.
//! The .cfe file contains the directory, and .cfs contains the actual data.

use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::Path;

use super::codec::{CodecHeader, CodecFooter};
use super::vint::{read_vint, read_string, read_u64_be};

/// Entry in the compound file directory
#[derive(Debug, Clone)]
pub struct CompoundEntry {
    pub name: String,
    pub offset: u64,
    pub length: u64,
}

/// Compound file reader
#[derive(Debug)]
pub struct CompoundFile {
    /// Directory mapping filename to (offset, length)
    entries: HashMap<String, CompoundEntry>,
    /// Raw data from .cfs file
    data: Vec<u8>,
}

impl CompoundFile {
    /// Open a compound file from .cfs and .cfe paths
    pub fn open(cfs_path: &Path, cfe_path: &Path) -> io::Result<Self> {
        // Read the entry table from .cfe
        let cfe_data = fs::read(cfe_path)?;
        let entries = Self::parse_entries(&cfe_data)?;

        // Read the data from .cfs
        let data = fs::read(cfs_path)?;

        Ok(CompoundFile { entries, data })
    }

    /// Open a compound file from a directory (auto-detect .cfs/.cfe files)
    pub fn open_from_dir(dir: &Path, segment_name: &str) -> io::Result<Self> {
        let cfs_path = dir.join(format!("{}.cfs", segment_name));
        let cfe_path = dir.join(format!("{}.cfe", segment_name));
        Self::open(&cfs_path, &cfe_path)
    }

    /// Parse the entry table from .cfe file
    fn parse_entries(cfe_data: &[u8]) -> io::Result<HashMap<String, CompoundEntry>> {
        let mut pos = 0;

        // Parse header
        let header = CodecHeader::parse(cfe_data, &mut pos)?;
        if !header.codec_name.contains("CompoundFile") {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Expected CompoundFile codec, got: {}", header.codec_name),
            ));
        }

        // Parse footer to validate
        let _footer = CodecFooter::parse(cfe_data)?;

        // Number of entries (VInt)
        let num_entries = read_vint(cfe_data, &mut pos)? as usize;

        let mut entries = HashMap::with_capacity(num_entries);

        for _ in 0..num_entries {
            // Filename
            let name = read_string(cfe_data, &mut pos)?;

            // Offset (8 bytes big-endian)
            let offset = read_u64_be(cfe_data, &mut pos)?;

            // Length (8 bytes big-endian)
            let length = read_u64_be(cfe_data, &mut pos)?;

            entries.insert(name.clone(), CompoundEntry { name, offset, length });
        }

        Ok(entries)
    }

    /// List all embedded files
    pub fn list_files(&self) -> Vec<&str> {
        self.entries.keys().map(|s| s.as_str()).collect()
    }

    /// Get an embedded file's data
    pub fn get_file(&self, name: &str) -> Option<&[u8]> {
        let entry = self.entries.get(name)?;
        let start = entry.offset as usize;
        let end = start + entry.length as usize;

        if end <= self.data.len() {
            Some(&self.data[start..end])
        } else {
            None
        }
    }

    /// Get file data by suffix (e.g., ".fdt", ".tim")
    pub fn get_file_by_suffix(&self, suffix: &str) -> Option<(&str, &[u8])> {
        for (name, _) in &self.entries {
            if name.ends_with(suffix) {
                return self.get_file(name).map(|data| (name.as_str(), data));
            }
        }
        None
    }

    /// Get the stored fields data file (.fdt)
    pub fn get_stored_fields_data(&self) -> Option<&[u8]> {
        self.get_file_by_suffix(".fdt").map(|(_, data)| data)
    }

    /// Get the stored fields index file (.fdx)
    pub fn get_stored_fields_index(&self) -> Option<&[u8]> {
        self.get_file_by_suffix(".fdx").map(|(_, data)| data)
    }

    /// Get the term dictionary file (.tim)
    pub fn get_term_dictionary(&self) -> Option<&[u8]> {
        self.get_file_by_suffix(".tim").map(|(_, data)| data)
    }

    /// Get entry info
    pub fn get_entry(&self, name: &str) -> Option<&CompoundEntry> {
        self.entries.get(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_open_compound_file() {
        let test_dir = Path::new("/home/ubuntu/auto-correct/languagetool/languagetool-core/src/test/resources/org/languagetool/resource/yy/ngram-index/1grams");

        if test_dir.exists() {
            let cf = CompoundFile::open_from_dir(test_dir, "_0").unwrap();
            let files = cf.list_files();

            println!("Found {} files in compound:", files.len());
            for f in &files {
                let entry = cf.get_entry(f).unwrap();
                println!("  {} -> offset={}, len={}", f, entry.offset, entry.length);
            }

            // Should have some files
            assert!(!files.is_empty());

            // Should be able to get stored fields
            let fdt = cf.get_stored_fields_data();
            println!("FDT data: {:?}", fdt.map(|d| d.len()));
            assert!(fdt.is_some());

            // Print first 50 bytes of FDT
            if let Some(data) = fdt {
                println!("FDT first 50 bytes: {:02x?}", &data[..data.len().min(50)]);
            }
        }
    }
}
