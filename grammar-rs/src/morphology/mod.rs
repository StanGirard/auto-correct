//! French morphology module using Lefff lexicon
//!
//! Provides morphological analysis and synthesis for French words.
//! Used for `postag_replace` suggestions in DynamicPatternChecker.

mod french;
mod pos_transform;

pub use french::FrenchMorphology;
pub use pos_transform::transform_pos;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_synthesis() {
        let morph = FrenchMorphology::load();

        // le (D m s) -> la (D f s)
        let forms = morph.synthesize("le", "D f s");
        assert!(forms.iter().any(|f| *f == "la"), "Expected 'la' for lemma 'le' with POS 'D f s'");
    }

    #[test]
    fn test_basic_analysis() {
        let morph = FrenchMorphology::load();

        // "mange" should analyze to verb "manger"
        let readings = morph.analyze("mange");
        assert!(!readings.is_empty(), "Expected readings for 'mange'");
        assert!(readings.iter().any(|e| e.lemma == "manger"),
                "Expected lemma 'manger' for form 'mange'");
    }
}
