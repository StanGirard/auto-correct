//! Dictionary module for spell checking
//!
//! Provides efficient dictionary storage using FST (Finite State Transducer).
//! FST provides O(key_length) lookup with minimal memory usage.

mod fst_dict;

pub use fst_dict::FstDictionary;
