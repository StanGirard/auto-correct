//! Traits définissant le pipeline - le contrat stable
//!
//! Chaque trait représente une étape du pipeline.
//! Tu peux implémenter ces traits de manière simple au début,
//! puis les remplacer par des versions plus sophistiquées.

use super::{AnalyzedToken, CheckResult, Token};

/// Étape 1: Découper le texte en tokens
pub trait Tokenizer: Send + Sync {
    fn tokenize<'a>(&self, text: &'a str) -> Vec<Token<'a>>;
}

/// Étape 2: Enrichir les tokens (lemme, POS tag)
pub trait Analyzer: Send + Sync {
    fn analyze<'a>(&self, tokens: Vec<Token<'a>>) -> Vec<AnalyzedToken<'a>>;
}

/// Étape 3: Détecter les erreurs
pub trait Checker: Send + Sync {
    fn check(&self, text: &str, tokens: &[AnalyzedToken]) -> CheckResult;
}

/// Étape 4: Générer des suggestions (optionnel, peut être intégré au Checker)
pub trait Suggester: Send + Sync {
    fn suggest(&self, word: &str, max: usize) -> Vec<String>;
}

/// Combinaison de tous les traits pour faciliter l'usage
pub trait GrammarChecker: Send + Sync {
    fn check_text(&self, text: &str) -> CheckResult;
}
