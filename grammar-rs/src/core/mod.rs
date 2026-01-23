//! Core types - le cœur stable qui ne change jamais

pub mod traits;
pub mod pipeline;

use std::ops::Range;

/// Un token avec sa position dans le texte original
#[derive(Debug, Clone, PartialEq)]
pub struct Token<'a> {
    pub text: &'a str,
    pub span: Range<usize>,
    pub kind: TokenKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    Word,
    Punctuation,
    Whitespace,
    Number,
    Unknown,
}

/// Token enrichi avec analyse linguistique
#[derive(Debug, Clone)]
pub struct AnalyzedToken<'a> {
    pub token: Token<'a>,
    pub lemma: Option<String>,
    pub pos: Option<PosTag>,
}

/// Part-of-speech tags (simplifié, extensible)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PosTag {
    Noun,
    Verb,
    Adjective,
    Adverb,
    Determiner,
    Preposition,
    Conjunction,
    Pronoun,
    Punctuation,
    Other,
}

/// Une erreur détectée
#[derive(Debug, Clone)]
pub struct Match {
    pub span: Range<usize>,
    pub message: String,
    pub rule_id: String,
    pub suggestions: Vec<String>,
    pub severity: Severity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
    Hint,
}

/// Résultat final de l'analyse
#[derive(Debug, Clone)]
pub struct CheckResult {
    pub matches: Vec<Match>,
}

impl CheckResult {
    pub fn new() -> Self {
        Self { matches: Vec::new() }
    }

    pub fn merge(&mut self, other: CheckResult) {
        self.matches.extend(other.matches);
    }

    pub fn sort_and_dedupe(&mut self) {
        self.matches.sort_by_key(|m| m.span.start);
        self.matches.dedup_by(|a, b| a.span == b.span && a.rule_id == b.rule_id);
    }
}

impl Default for CheckResult {
    fn default() -> Self {
        Self::new()
    }
}
