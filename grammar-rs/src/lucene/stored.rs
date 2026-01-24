//! Lucene stored fields reader (Lucene41StoredFieldsFormat)
//!
//! Stored fields contain the actual document data (ngram text and count).
//! Format: header, then chunks of compressed documents.

use std::io;
use flate2::read::DeflateDecoder;
use std::io::Read;

use super::codec::CodecHeader;
use super::vint::{read_vint, read_u32_be};

/// A stored document with its fields
#[derive(Debug, Clone)]
pub struct StoredDocument {
    pub fields: Vec<StoredField>,
}

/// A single stored field
#[derive(Debug, Clone)]
pub struct StoredField {
    pub field_num: u32,
    pub value: FieldValue,
}

/// Field value types
#[derive(Debug, Clone)]
pub enum FieldValue {
    String(String),
    Binary(Vec<u8>),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
}

impl FieldValue {
    /// Get as string if possible
    pub fn as_string(&self) -> Option<&str> {
        match self {
            FieldValue::String(s) => Some(s),
            _ => None,
        }
    }

    /// Get as i64 if numeric
    pub fn as_i64(&self) -> Option<i64> {
        match self {
            FieldValue::Int(i) => Some(*i as i64),
            FieldValue::Long(l) => Some(*l),
            FieldValue::String(s) => s.parse().ok(),
            _ => None,
        }
    }
}

/// Stored fields reader
pub struct StoredFieldsReader<'a> {
    data: &'a [u8],
    index: &'a [u8],
    num_docs: u32,
}

impl<'a> StoredFieldsReader<'a> {
    /// Create a new reader from .fdt (data) and .fdx (index) bytes
    pub fn new(fdt_data: &'a [u8], fdx_data: &'a [u8]) -> io::Result<Self> {
        // Parse .fdx header to get document count
        let mut pos = 0;
        let _header = CodecHeader::parse(fdx_data, &mut pos)?;

        // The .fdx contains chunk start positions
        // For now, we'll iterate through .fdt directly

        // Parse .fdt header
        let mut fdt_pos = 0;
        let header = CodecHeader::parse(fdt_data, &mut fdt_pos)?;

        if !header.codec_name.contains("StoredFields") {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Expected StoredFields codec, got: {}", header.codec_name),
            ));
        }

        // Version determines format
        // Lucene 4.1+ uses compression

        // Get document count from .fdx
        // fdx format: header, then packed longs for chunk positions
        let num_docs = Self::count_docs_from_fdt(fdt_data)?;

        Ok(StoredFieldsReader {
            data: fdt_data,
            index: fdx_data,
            num_docs,
        })
    }

    /// Count documents by scanning the .fdt file
    fn count_docs_from_fdt(fdt_data: &[u8]) -> io::Result<u32> {
        // This is a simplified count - in practice we'd parse properly
        // For small test files, we can estimate from file structure
        // Each doc chunk has a header with doc count

        let mut pos = 0;
        let _header = CodecHeader::parse(fdt_data, &mut pos)?;

        // Read first chunk header to get numDocs
        if pos + 4 <= fdt_data.len() {
            // First VInt is typically doc base, second is numDocs
            let _doc_base = read_vint(fdt_data, &mut pos).unwrap_or(0);
            let num_docs = read_vint(fdt_data, &mut pos).unwrap_or(0);
            return Ok(num_docs as u32);
        }

        Ok(0)
    }

    /// Get number of documents
    pub fn num_docs(&self) -> u32 {
        self.num_docs
    }

    /// Read all stored documents (simplified - reads first chunk only)
    pub fn read_all_documents(&self) -> io::Result<Vec<StoredDocument>> {
        let mut pos = 0;

        // Skip header
        let _header = CodecHeader::parse(self.data, &mut pos)?;

        // Read chunk header
        let _doc_base = read_vint(self.data, &mut pos)?;
        let num_docs = read_vint(self.data, &mut pos)? as usize;

        if num_docs == 0 {
            return Ok(Vec::new());
        }

        // Read num_buffered_docs (same as num_docs for small chunks)
        let num_buffered = read_vint(self.data, &mut pos)? as usize;

        // Read doc lengths (VInt for each doc, representing stored field data length)
        let mut doc_lengths = Vec::with_capacity(num_buffered);
        for _ in 0..num_buffered {
            doc_lengths.push(read_vint(self.data, &mut pos)? as usize);
        }

        // Check if data is compressed
        // Lucene 4.1 uses DEFLATE compression with a flag
        let is_compressed = if pos < self.data.len() {
            // Check for compression marker or raw data
            // Compressed data starts with DEFLATE header
            self.data.get(pos).map(|&b| b == 0x78).unwrap_or(false) // zlib header
        } else {
            false
        };

        // Read document data
        let mut documents = Vec::with_capacity(num_buffered);

        if is_compressed {
            // Decompress all document data
            let remaining = &self.data[pos..];

            // Try to decompress
            let mut decoder = DeflateDecoder::new(remaining);
            let mut decompressed = Vec::new();
            if decoder.read_to_end(&mut decompressed).is_ok() {
                // Parse decompressed documents
                let mut doc_pos = 0;
                for &len in &doc_lengths {
                    if doc_pos + len <= decompressed.len() {
                        let doc_data = &decompressed[doc_pos..doc_pos + len];
                        let doc = Self::parse_document(doc_data)?;
                        documents.push(doc);
                        doc_pos += len;
                    }
                }
            }
        } else {
            // Uncompressed - read directly
            for &len in &doc_lengths {
                if pos + len <= self.data.len() {
                    let doc_data = &self.data[pos..pos + len];
                    let doc = Self::parse_document(doc_data)?;
                    documents.push(doc);
                    pos += len;
                }
            }
        }

        Ok(documents)
    }

    /// Parse a single document's stored fields
    fn parse_document(data: &[u8]) -> io::Result<StoredDocument> {
        let mut pos = 0;
        let mut fields = Vec::new();

        while pos < data.len() {
            // Field info: VInt encoding field number and type
            let field_info = match read_vint(data, &mut pos) {
                Ok(v) => v,
                Err(_) => break,
            };

            // Lower 3 bits = type, rest = field number
            let field_type = (field_info & 0x07) as u8;
            let field_num = (field_info >> 3) as u32;

            let value = match field_type {
                0 => {
                    // String
                    let len = read_vint(data, &mut pos)? as usize;
                    if pos + len > data.len() {
                        break;
                    }
                    let s = std::str::from_utf8(&data[pos..pos + len])
                        .unwrap_or("")
                        .to_string();
                    pos += len;
                    FieldValue::String(s)
                }
                1 => {
                    // Binary
                    let len = read_vint(data, &mut pos)? as usize;
                    if pos + len > data.len() {
                        break;
                    }
                    let b = data[pos..pos + len].to_vec();
                    pos += len;
                    FieldValue::Binary(b)
                }
                2 => {
                    // Int (VInt encoded)
                    let v = read_vint(data, &mut pos)? as i32;
                    FieldValue::Int(v)
                }
                3 => {
                    // Long (VLong encoded)
                    let v = read_vint(data, &mut pos)? as i64;
                    FieldValue::Long(v)
                }
                4 => {
                    // Float (4 bytes)
                    if pos + 4 > data.len() {
                        break;
                    }
                    let bits = u32::from_be_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]]);
                    pos += 4;
                    FieldValue::Float(f32::from_bits(bits))
                }
                5 => {
                    // Double (8 bytes)
                    if pos + 8 > data.len() {
                        break;
                    }
                    let bits = u64::from_be_bytes([
                        data[pos], data[pos + 1], data[pos + 2], data[pos + 3],
                        data[pos + 4], data[pos + 5], data[pos + 6], data[pos + 7],
                    ]);
                    pos += 8;
                    FieldValue::Double(f64::from_bits(bits))
                }
                _ => {
                    // Unknown type, try to skip
                    break;
                }
            };

            fields.push(StoredField { field_num, value });
        }

        Ok(StoredDocument { fields })
    }
}

impl StoredDocument {
    /// Get field by number
    pub fn get_field(&self, field_num: u32) -> Option<&StoredField> {
        self.fields.iter().find(|f| f.field_num == field_num)
    }

    /// Get string field value
    pub fn get_string(&self, field_num: u32) -> Option<&str> {
        self.get_field(field_num)?.value.as_string()
    }

    /// Get numeric field value
    pub fn get_i64(&self, field_num: u32) -> Option<i64> {
        self.get_field(field_num)?.value.as_i64()
    }
}
