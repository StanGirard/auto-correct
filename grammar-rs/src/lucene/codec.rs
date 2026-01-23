//! Lucene codec header and footer parsing
//!
//! All Lucene files start with a codec header and end with a footer.

use std::io;
use super::vint::{read_u32_be, read_i64_be, read_string};

/// Magic bytes at start of every Lucene file
pub const CODEC_MAGIC: u32 = 0x3FD76C17;

/// Codec header at the start of every Lucene file
#[derive(Debug, Clone)]
pub struct CodecHeader {
    pub codec_name: String,
    pub version: i32,
}

impl CodecHeader {
    /// Parse a codec header from bytes
    pub fn parse(bytes: &[u8], pos: &mut usize) -> io::Result<Self> {
        // Magic number (4 bytes)
        let magic = read_u32_be(bytes, pos)?;
        if magic != CODEC_MAGIC {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Invalid codec magic: expected {:08X}, got {:08X}", CODEC_MAGIC, magic),
            ));
        }

        // Codec name (VInt length + UTF-8)
        let codec_name = read_string(bytes, pos)?;

        // Version (4 bytes big-endian, signed)
        let version = read_u32_be(bytes, pos)? as i32;

        Ok(CodecHeader { codec_name, version })
    }

    /// Get the size of the header in bytes (approximate, codec name is variable)
    pub fn header_size(&self) -> usize {
        4 + // magic
        1 + self.codec_name.len() + // VInt length + name
        4 // version
    }
}

/// Codec footer at the end of every Lucene file
#[derive(Debug, Clone)]
pub struct CodecFooter {
    pub algorithm_id: i32,
    pub checksum: i64,
}

impl CodecFooter {
    /// Footer is always 16 bytes
    pub const SIZE: usize = 16;

    /// Parse a codec footer from the last 16 bytes
    /// Note: Returns default values if magic doesn't match (older Lucene versions)
    pub fn parse(bytes: &[u8]) -> io::Result<Self> {
        if bytes.len() < Self::SIZE {
            return Ok(CodecFooter { algorithm_id: 0, checksum: 0 });
        }

        let footer_start = bytes.len() - Self::SIZE;
        let mut pos = footer_start;

        // Magic (4 bytes) - may not match in older formats
        let magic = read_u32_be(bytes, &mut pos)?;
        if magic != CODEC_MAGIC {
            // Older Lucene versions may have different footer format
            // Return default values instead of failing
            return Ok(CodecFooter { algorithm_id: 0, checksum: 0 });
        }

        // Algorithm ID (4 bytes)
        let algorithm_id = read_u32_be(bytes, &mut pos)? as i32;

        // Checksum (8 bytes)
        let checksum = read_i64_be(bytes, &mut pos)?;

        Ok(CodecFooter { algorithm_id, checksum })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_codec_header() {
        // Magic + "test" + version 1
        let bytes = [
            0x3F, 0xD7, 0x6C, 0x17, // magic
            0x04, b't', b'e', b's', b't', // codec name
            0x00, 0x00, 0x00, 0x01, // version
        ];
        let mut pos = 0;
        let header = CodecHeader::parse(&bytes, &mut pos).unwrap();
        assert_eq!(header.codec_name, "test");
        assert_eq!(header.version, 1);
    }
}
