//! Variable-length integer encoding (base-128)
//!
//! Lucene uses VInt encoding for most integer values. Each byte has 7 data bits
//! and 1 continuation bit (high bit). If high bit is set, more bytes follow.

use std::io::{self, Read};

/// Read a variable-length integer from a byte slice
pub fn read_vint(bytes: &[u8], pos: &mut usize) -> io::Result<u64> {
    let mut result = 0u64;
    let mut shift = 0;

    loop {
        if *pos >= bytes.len() {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "VInt truncated"));
        }

        let b = bytes[*pos];
        *pos += 1;

        // Add the 7 data bits
        result |= ((b & 0x7F) as u64) << shift;

        // If high bit is 0, we're done
        if b & 0x80 == 0 {
            break;
        }

        shift += 7;
        if shift > 63 {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "VInt overflow"));
        }
    }

    Ok(result)
}

/// Read a variable-length integer from a reader
pub fn read_vint_from_reader<R: Read>(reader: &mut R) -> io::Result<u64> {
    let mut result = 0u64;
    let mut shift = 0;
    let mut buf = [0u8; 1];

    loop {
        reader.read_exact(&mut buf)?;
        let b = buf[0];

        result |= ((b & 0x7F) as u64) << shift;

        if b & 0x80 == 0 {
            break;
        }

        shift += 7;
        if shift > 63 {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "VInt overflow"));
        }
    }

    Ok(result)
}

/// Read a Lucene-style string (VInt length + UTF-8 bytes)
pub fn read_string(bytes: &[u8], pos: &mut usize) -> io::Result<String> {
    let len = read_vint(bytes, pos)? as usize;

    if *pos + len > bytes.len() {
        return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "String truncated"));
    }

    let s = std::str::from_utf8(&bytes[*pos..*pos + len])
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    *pos += len;

    Ok(s.to_string())
}

/// Read a fixed-length big-endian u32
pub fn read_u32_be(bytes: &[u8], pos: &mut usize) -> io::Result<u32> {
    if *pos + 4 > bytes.len() {
        return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "u32 truncated"));
    }

    let result = u32::from_be_bytes([
        bytes[*pos],
        bytes[*pos + 1],
        bytes[*pos + 2],
        bytes[*pos + 3],
    ]);
    *pos += 4;

    Ok(result)
}

/// Read a fixed-length big-endian u64
pub fn read_u64_be(bytes: &[u8], pos: &mut usize) -> io::Result<u64> {
    if *pos + 8 > bytes.len() {
        return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "u64 truncated"));
    }

    let result = u64::from_be_bytes([
        bytes[*pos],
        bytes[*pos + 1],
        bytes[*pos + 2],
        bytes[*pos + 3],
        bytes[*pos + 4],
        bytes[*pos + 5],
        bytes[*pos + 6],
        bytes[*pos + 7],
    ]);
    *pos += 8;

    Ok(result)
}

/// Read a fixed-length big-endian i64
pub fn read_i64_be(bytes: &[u8], pos: &mut usize) -> io::Result<i64> {
    read_u64_be(bytes, pos).map(|v| v as i64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vint_single_byte() {
        let bytes = [0x00];
        let mut pos = 0;
        assert_eq!(read_vint(&bytes, &mut pos).unwrap(), 0);

        let bytes = [0x7F];
        let mut pos = 0;
        assert_eq!(read_vint(&bytes, &mut pos).unwrap(), 127);
    }

    #[test]
    fn test_vint_multi_byte() {
        // 128 = 0x80 in VInt = [0x80, 0x01]
        let bytes = [0x80, 0x01];
        let mut pos = 0;
        assert_eq!(read_vint(&bytes, &mut pos).unwrap(), 128);

        // 16383 = [0xFF, 0x7F]
        let bytes = [0xFF, 0x7F];
        let mut pos = 0;
        assert_eq!(read_vint(&bytes, &mut pos).unwrap(), 16383);
    }

    #[test]
    fn test_string() {
        // Length 5 + "hello"
        let bytes = [0x05, b'h', b'e', b'l', b'l', b'o'];
        let mut pos = 0;
        assert_eq!(read_string(&bytes, &mut pos).unwrap(), "hello");
    }
}
