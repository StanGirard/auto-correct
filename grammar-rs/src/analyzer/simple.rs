//! Analyzer simple - pas d'analyse pour commencer
//!
//! Version 1: Pass-through (juste convertit Token -> AnalyzedToken)
//! Version 2 (future): Dictionnaire FST pour lemmes + POS

use crate::core::{AnalyzedToken, Token, TokenKind, PosTag};
use crate::core::traits::Analyzer;

/// Analyzer minimal qui ne fait que passer les tokens
pub struct PassthroughAnalyzer;

impl PassthroughAnalyzer {
    pub fn new() -> Self {
        Self
    }
}

impl Default for PassthroughAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl Analyzer for PassthroughAnalyzer {
    fn analyze<'a>(&self, tokens: Vec<Token<'a>>) -> Vec<AnalyzedToken<'a>> {
        tokens
            .into_iter()
            .map(|token| {
                let pos = match token.kind {
                    TokenKind::Punctuation => Some(PosTag::Punctuation),
                    _ => None,
                };
                AnalyzedToken {
                    token,
                    lemma: None,
                    pos,
                }
            })
            .collect()
    }
}

/// Analyzer avec dictionnaire simple (HashMap)
/// Pour itérer avant de passer à FST
pub struct DictAnalyzer {
    // word -> (lemma, pos)
    dict: std::collections::HashMap<String, (String, PosTag)>,
}

impl DictAnalyzer {
    pub fn new() -> Self {
        Self {
            dict: std::collections::HashMap::new(),
        }
    }

    /// Charge un dictionnaire depuis des lignes "word\tlemma\tPOS"
    pub fn load_from_lines<I, S>(&mut self, lines: I)
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        for line in lines {
            let parts: Vec<&str> = line.as_ref().split('\t').collect();
            if parts.len() >= 3 {
                let word = parts[0].to_lowercase();
                let lemma = parts[1].to_string();
                let pos = parse_pos(parts[2]);
                self.dict.insert(word, (lemma, pos));
            }
        }
    }
}

impl Default for DictAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl Analyzer for DictAnalyzer {
    fn analyze<'a>(&self, tokens: Vec<Token<'a>>) -> Vec<AnalyzedToken<'a>> {
        tokens
            .into_iter()
            .map(|token| {
                let lookup = self.dict.get(&token.text.to_lowercase());
                AnalyzedToken {
                    lemma: lookup.map(|(l, _)| l.clone()),
                    pos: lookup.map(|(_, p)| *p).or_else(|| {
                        match token.kind {
                            TokenKind::Punctuation => Some(PosTag::Punctuation),
                            _ => None,
                        }
                    }),
                    token,
                }
            })
            .collect()
    }
}

fn parse_pos(s: &str) -> PosTag {
    match s.to_uppercase().as_str() {
        "NOUN" | "NN" | "NNS" | "NNP" => PosTag::Noun,
        "VERB" | "VB" | "VBD" | "VBG" | "VBN" | "VBP" | "VBZ" => PosTag::Verb,
        "ADJ" | "JJ" | "JJR" | "JJS" => PosTag::Adjective,
        "ADV" | "RB" | "RBR" | "RBS" => PosTag::Adverb,
        "DET" | "DT" => PosTag::Determiner,
        "PREP" | "IN" => PosTag::Preposition,
        "CONJ" | "CC" => PosTag::Conjunction,
        "PRON" | "PRP" => PosTag::Pronoun,
        _ => PosTag::Other,
    }
}
