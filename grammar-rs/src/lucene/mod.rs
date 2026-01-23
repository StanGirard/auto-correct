//! Minimal Lucene 4.x index reader for N-gram extraction
//!
//! This module provides just enough functionality to read ngram→count
//! mappings from LanguageTool's Lucene indexes.

mod vint;
mod codec;
mod compound;
mod stored;
mod reader;

pub use codec::{CodecHeader, CodecFooter};
pub use compound::CompoundFile;
pub use stored::StoredFieldsReader;
pub use reader::{NgramIndexReader, NgramEntry};
