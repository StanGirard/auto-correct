//! Grammar-rs: Correcteur grammatical ultra-performant
//!
//! Architecture en pipeline composable:
//! ```text
//! Text → Tokenizer → Analyzer → Checker(s) → CheckResult
//! ```
//!
//! Chaque étape est un trait que tu peux implémenter/remplacer.
//!
//! # Exemple d'utilisation
//!
//! ```rust
//! use grammar_rs::prelude::*;
//!
//! // Construire le pipeline
//! let pipeline = Pipeline::new(
//!     SimpleTokenizer::new(),
//!     PassthroughAnalyzer::new(),
//! )
//! .with_checker(SpellChecker::new().with_words(["hello", "world"]))
//! .with_checker(RuleChecker::new().with_english_rules());
//!
//! // Vérifier du texte
//! let result = pipeline.check_text("helo  world");
//!
//! for m in result.matches {
//!     println!("{}: {} ({:?})", m.rule_id, m.message, m.suggestions);
//! }
//! ```

pub mod core;
pub mod tokenizer;
pub mod analyzer;
pub mod checker;
pub mod dictionary;
pub mod lang_detect;
pub mod filter;
pub mod lucene;
pub mod language_model;

/// Pre-warm all lazy statics to avoid first-call latency
///
/// Call this at startup (API init or test setup) to initialize
/// all LazyLock data structures upfront. This is optional but
/// improves response time for the first request.
///
/// # Example
/// ```
/// // Call once at startup
/// grammar_rs::warm_up();
/// ```
pub fn warm_up() {
    use checker::data::{
        en_compounds::EN_COMPOUND_LOOKUP,
        fr_compounds::FR_COMPOUND_LOOKUP,
        en_antipatterns::EN_ANTIPATTERNS_BY_RULE,
        fr_antipatterns::FR_ANTIPATTERNS_BY_RULE,
    };
    use checker::compound_checker::CompoundWordChecker;
    use core::traits::Checker;

    // Force initialization of compound lookup tables
    let _ = EN_COMPOUND_LOOKUP.len();
    let _ = FR_COMPOUND_LOOKUP.len();

    // Force initialization of antipattern lookup tables
    let _ = EN_ANTIPATTERNS_BY_RULE.len();
    let _ = FR_ANTIPATTERNS_BY_RULE.len();

    // Force initialization of compound first words sets
    // by calling check() which accesses them internally
    let en_checker = CompoundWordChecker::english();
    let fr_checker = CompoundWordChecker::french();
    let _ = en_checker.check("air plane", &[]);
    let _ = fr_checker.check("aller retour", &[]);
}

/// Prelude - importe tout ce dont tu as besoin
pub mod prelude {
    pub use crate::core::{
        Token, TokenKind, AnalyzedToken, PosTag,
        Match, Severity, CheckResult,
        MaskKind, MaskedRegion,
    };
    pub use crate::core::traits::{
        Tokenizer, Analyzer, Checker, Suggester, GrammarChecker,
    };
    pub use crate::core::filter::{Filter, FilterChain};
    pub use crate::core::pipeline::Pipeline;
    pub use crate::tokenizer::{SimpleTokenizer, ContractionTokenizer};
    pub use crate::analyzer::{PassthroughAnalyzer, DictAnalyzer, PosTagger};
    pub use crate::checker::{SpellChecker, RuleChecker};
    pub use crate::dictionary::FstDictionary;
    pub use crate::lang_detect::{Language, LanguageDetector, DetectionResult};
    pub use crate::filter::{
        UrlFilter, CodeBlockFilter, QuotedTextFilter, DateFilter, NumberFilter,
        default_filters, FilterBuilder,
    };
}

#[cfg(test)]
mod integration_tests {
    use super::prelude::*;

    #[test]
    fn test_full_pipeline() {
        let pipeline = Pipeline::new(
            SimpleTokenizer::new(),
            PassthroughAnalyzer::new(),
        )
        .with_checker(SpellChecker::new().with_words([
            "hello", "world", "this", "is", "a", "test",
        ]))
        .with_checker(RuleChecker::new().with_english_rules());

        let result = pipeline.check_text("helo world");

        assert!(!result.matches.is_empty());
        assert!(result.matches.iter().any(|m| m.rule_id == "SPELL"));
    }

    #[test]
    fn test_french_pipeline() {
        let pipeline = Pipeline::new(
            SimpleTokenizer::new(),
            PassthroughAnalyzer::new(),
        )
        .with_checker(RuleChecker::new().with_french_rules());

        let result = pipeline.check_text("Bonjour! Comment ça va?");

        // Should detect missing spaces before ! and ?
        assert!(result.matches.iter().any(|m| m.rule_id == "FR_PUNCT_SPACE"));
    }
}
