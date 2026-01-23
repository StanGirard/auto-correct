//! Tokenizer with contraction support
//!
//! Handles English contractions like "it's", "don't", "you're" as single tokens.
//! Also handles French elisions like "l'homme", "j'ai".

use crate::core::{Token, TokenKind};
use crate::core::traits::Tokenizer;

/// Common English contractions (lowercase)
const ENGLISH_CONTRACTIONS: &[&str] = &[
    // Pronoun + verb
    "i'm", "i've", "i'd", "i'll",
    "you're", "you've", "you'd", "you'll",
    "he's", "he'd", "he'll",
    "she's", "she'd", "she'll",
    "it's", "it'd", "it'll",
    "we're", "we've", "we'd", "we'll",
    "they're", "they've", "they'd", "they'll",
    "that's", "that'd", "that'll",
    "what's", "what'd", "what'll", "what're", "what've",
    "who's", "who'd", "who'll", "who're", "who've",
    "where's", "where'd", "where'll", "where're", "where've",
    "when's", "when'd", "when'll",
    "why's", "why'd", "why'll",
    "how's", "how'd", "how'll",
    "there's", "there'd", "there'll", "there're", "there've",
    "here's", "here'd", "here'll",
    // Verb + not
    "isn't", "aren't", "wasn't", "weren't",
    "hasn't", "haven't", "hadn't",
    "doesn't", "don't", "didn't",
    "won't", "wouldn't",
    "can't", "couldn't",
    "shan't", "shouldn't",
    "mightn't", "mustn't", "needn't",
    "ain't",
    // Other
    "let's", "ma'am", "o'clock", "y'all",
];

/// Tokenizer that preserves contractions as single tokens
pub struct ContractionTokenizer {
    preserve_contractions: bool,
}

impl ContractionTokenizer {
    pub fn new() -> Self {
        Self {
            preserve_contractions: true,
        }
    }

    /// Disable contraction merging (behaves like SimpleTokenizer)
    pub fn without_contractions(mut self) -> Self {
        self.preserve_contractions = false;
        self
    }
}

impl Default for ContractionTokenizer {
    fn default() -> Self {
        Self::new()
    }
}

impl Tokenizer for ContractionTokenizer {
    fn tokenize<'a>(&self, text: &'a str) -> Vec<Token<'a>> {
        let mut tokens = Vec::new();
        let mut i = 0;
        let chars: Vec<char> = text.chars().collect();
        let len = chars.len();

        while i < len {
            let c = chars[i];

            // Whitespace
            if c.is_whitespace() {
                let start = byte_index(text, i);
                let mut end_idx = i + 1;
                while end_idx < len && chars[end_idx].is_whitespace() {
                    end_idx += 1;
                }
                let end = byte_index(text, end_idx);
                tokens.push(Token {
                    text: &text[start..end],
                    span: start..end,
                    kind: TokenKind::Whitespace,
                });
                i = end_idx;
                continue;
            }

            // Number
            if c.is_numeric() {
                let start = byte_index(text, i);
                let mut end_idx = i + 1;
                while end_idx < len && (chars[end_idx].is_numeric() || chars[end_idx] == '.') {
                    end_idx += 1;
                }
                let end = byte_index(text, end_idx);
                tokens.push(Token {
                    text: &text[start..end],
                    span: start..end,
                    kind: TokenKind::Number,
                });
                i = end_idx;
                continue;
            }

            // Word (potentially with contraction)
            if c.is_alphabetic() {
                let start = byte_index(text, i);
                let word_start = i;
                let mut end_idx = i + 1;

                // Consume alphabetic characters
                while end_idx < len && chars[end_idx].is_alphabetic() {
                    end_idx += 1;
                }

                // Check for contraction (apostrophe followed by more letters)
                if self.preserve_contractions && end_idx < len && is_apostrophe(chars[end_idx]) {
                    let apos_idx = end_idx;
                    end_idx += 1;

                    // Check if there are letters after the apostrophe
                    if end_idx < len && chars[end_idx].is_alphabetic() {
                        // Consume the rest of the contraction
                        while end_idx < len && chars[end_idx].is_alphabetic() {
                            end_idx += 1;
                        }

                        // Verify it's a known contraction
                        let potential: String = chars[word_start..end_idx].iter().collect();
                        if is_contraction(&potential) {
                            let end = byte_index(text, end_idx);
                            tokens.push(Token {
                                text: &text[start..end],
                                span: start..end,
                                kind: TokenKind::Word,
                            });
                            i = end_idx;
                            continue;
                        } else {
                            // Not a known contraction, revert to before apostrophe
                            end_idx = apos_idx;
                        }
                    } else {
                        // No letters after apostrophe, revert
                        end_idx = apos_idx;
                    }
                }

                let end = byte_index(text, end_idx);
                tokens.push(Token {
                    text: &text[start..end],
                    span: start..end,
                    kind: TokenKind::Word,
                });
                i = end_idx;
                continue;
            }

            // Punctuation (including apostrophe not part of contraction)
            if c.is_ascii_punctuation() || is_unicode_punctuation(c) {
                let start = byte_index(text, i);
                let end = byte_index(text, i + 1);
                tokens.push(Token {
                    text: &text[start..end],
                    span: start..end,
                    kind: TokenKind::Punctuation,
                });
                i += 1;
                continue;
            }

            // Unknown character - treat as word
            let start = byte_index(text, i);
            let end = byte_index(text, i + 1);
            tokens.push(Token {
                text: &text[start..end],
                span: start..end,
                kind: TokenKind::Word,
            });
            i += 1;
        }

        tokens
    }
}

/// Get byte index from char index
fn byte_index(text: &str, char_idx: usize) -> usize {
    text.char_indices()
        .nth(char_idx)
        .map(|(i, _)| i)
        .unwrap_or(text.len())
}

/// Check if character is an apostrophe (ASCII or Unicode)
fn is_apostrophe(c: char) -> bool {
    c == '\'' || c == '\u{2019}' // ' or '
}

/// Check if a word is a known contraction
fn is_contraction(word: &str) -> bool {
    let lower = word.to_lowercase();
    ENGLISH_CONTRACTIONS.contains(&lower.as_str())
}

fn is_unicode_punctuation(c: char) -> bool {
    matches!(c,
        '\u{00AB}' | '\u{00BB}' |  // « »
        '\u{201C}' | '\u{201D}' |  // " "
        '\u{2018}' | '\u{2019}' |  // ' '
        '\u{2026}' |              // …
        '\u{2014}' | '\u{2013}'   // — –
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contraction_its() {
        let tokenizer = ContractionTokenizer::new();
        let tokens = tokenizer.tokenize("it's great");

        let words: Vec<_> = tokens
            .iter()
            .filter(|t| t.kind == TokenKind::Word)
            .map(|t| t.text)
            .collect();

        assert_eq!(words, vec!["it's", "great"]);
    }

    #[test]
    fn test_contraction_dont() {
        let tokenizer = ContractionTokenizer::new();
        let tokens = tokenizer.tokenize("I don't know");

        let words: Vec<_> = tokens
            .iter()
            .filter(|t| t.kind == TokenKind::Word)
            .map(|t| t.text)
            .collect();

        assert_eq!(words, vec!["I", "don't", "know"]);
    }

    #[test]
    fn test_contraction_youre() {
        let tokenizer = ContractionTokenizer::new();
        let tokens = tokenizer.tokenize("You're welcome");

        let words: Vec<_> = tokens
            .iter()
            .filter(|t| t.kind == TokenKind::Word)
            .map(|t| t.text)
            .collect();

        assert_eq!(words, vec!["You're", "welcome"]);
    }

    #[test]
    fn test_non_contraction_possessive() {
        let tokenizer = ContractionTokenizer::new();
        let tokens = tokenizer.tokenize("John's book");

        // "John's" is not in our contraction list, so it should be split
        let words: Vec<_> = tokens
            .iter()
            .filter(|t| t.kind == TokenKind::Word)
            .map(|t| t.text)
            .collect();

        assert_eq!(words, vec!["John", "s", "book"]);
    }

    #[test]
    fn test_mixed_sentence() {
        let tokenizer = ContractionTokenizer::new();
        let tokens = tokenizer.tokenize("I can't believe it's not butter!");

        let words: Vec<_> = tokens
            .iter()
            .filter(|t| t.kind == TokenKind::Word)
            .map(|t| t.text)
            .collect();

        assert_eq!(words, vec!["I", "can't", "believe", "it's", "not", "butter"]);
    }

    #[test]
    fn test_without_contractions() {
        let tokenizer = ContractionTokenizer::new().without_contractions();
        let tokens = tokenizer.tokenize("it's great");

        let words: Vec<_> = tokens
            .iter()
            .filter(|t| t.kind == TokenKind::Word)
            .map(|t| t.text)
            .collect();

        // Should be split when contractions disabled
        assert_eq!(words, vec!["it", "s", "great"]);
    }
}
