//! Tokenizer simple basé sur Unicode
//!
//! Version 1: Simple mais correcte
//! Version 2 (future): unicode-segmentation crate

use crate::core::{Token, TokenKind};
use crate::core::traits::Tokenizer;

pub struct SimpleTokenizer;

impl SimpleTokenizer {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SimpleTokenizer {
    fn default() -> Self {
        Self::new()
    }
}

impl Tokenizer for SimpleTokenizer {
    fn tokenize<'a>(&self, text: &'a str) -> Vec<Token<'a>> {
        let mut tokens = Vec::new();
        let mut start = 0;

        for (i, c) in text.char_indices() {
            let kind = classify_char(c);
            let prev_kind = if start < i {
                text[start..i]
                    .chars()
                    .next()
                    .map(classify_char)
                    .unwrap_or(TokenKind::Unknown)
            } else {
                kind
            };

            // Nouveau token si le type change
            if kind != prev_kind && start < i {
                tokens.push(Token {
                    text: &text[start..i],
                    span: start..i,
                    kind: prev_kind,
                });
                start = i;
            }
        }

        // Dernier token
        if start < text.len() {
            let kind = text[start..]
                .chars()
                .next()
                .map(classify_char)
                .unwrap_or(TokenKind::Unknown);
            tokens.push(Token {
                text: &text[start..],
                span: start..text.len(),
                kind,
            });
        }

        tokens
    }
}

fn is_unicode_punctuation(c: char) -> bool {
    // Unicode punctuation: « » " " ' ' … — –
    matches!(c,
        '\u{00AB}' | '\u{00BB}' |  // « »
        '\u{201C}' | '\u{201D}' |  // " "
        '\u{2018}' | '\u{2019}' |  // ' '
        '\u{2026}' |              // …
        '\u{2014}' | '\u{2013}'   // — –
    )
}

fn classify_char(c: char) -> TokenKind {
    if c.is_whitespace() {
        TokenKind::Whitespace
    } else if c.is_alphabetic() {
        TokenKind::Word
    } else if c.is_numeric() {
        TokenKind::Number
    } else if c.is_ascii_punctuation() || is_unicode_punctuation(c) {
        TokenKind::Punctuation
    } else {
        TokenKind::Word // Treat unknown as word (emoji, etc.)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_sentence() {
        let tokenizer = SimpleTokenizer::new();
        let tokens = tokenizer.tokenize("Hello, world!");

        assert_eq!(tokens.len(), 5);
        assert_eq!(tokens[0].text, "Hello");
        assert_eq!(tokens[1].text, ",");
        assert_eq!(tokens[2].text, " ");
        assert_eq!(tokens[3].text, "world");
        assert_eq!(tokens[4].text, "!");
    }

    #[test]
    fn test_french() {
        let tokenizer = SimpleTokenizer::new();
        let tokens = tokenizer.tokenize("C'est génial !");

        let words: Vec<_> = tokens
            .iter()
            .filter(|t| t.kind == TokenKind::Word)
            .map(|t| t.text)
            .collect();

        assert!(words.contains(&"est"));
        assert!(words.contains(&"génial"));
    }
}
