//! French morphology using Lefff lexicon
//!
//! Provides analysis (form → lemma+POS) and synthesis (lemma+POS → form)
//! for French words.

use std::collections::HashMap;
use std::sync::OnceLock;
use regex::Regex;

/// Entry in the morphology dictionary
#[derive(Debug, Clone)]
pub struct MorphEntry {
    pub lemma: String,
    pub pos: String,
}

/// French morphology dictionary for analysis and synthesis
pub struct FrenchMorphology {
    /// form (lowercase) → Vec<(lemma, POS)>
    analysis: HashMap<String, Vec<MorphEntry>>,
    /// (lemma, POS) → Vec<form>
    synthesis: HashMap<(String, String), Vec<String>>,
    /// All unique POS tags for regex matching
    all_pos_tags: Vec<String>,
}

/// Static instance loaded once
static FRENCH_MORPHOLOGY: OnceLock<FrenchMorphology> = OnceLock::new();

impl FrenchMorphology {
    /// Load the French morphology dictionary (singleton)
    pub fn load() -> &'static FrenchMorphology {
        FRENCH_MORPHOLOGY.get_or_init(|| {
            Self::load_from_embedded()
        })
    }

    /// Load from embedded data
    fn load_from_embedded() -> FrenchMorphology {
        // Try to load from binary file, fallback to built-in minimal dictionary
        if let Some(morph) = Self::try_load_from_file() {
            return morph;
        }

        // Fallback: minimal built-in dictionary for common cases
        Self::load_minimal()
    }

    /// Try to load from data/morphology/fr_morphology.bin
    fn try_load_from_file() -> Option<FrenchMorphology> {
        use std::path::Path;
        use std::fs::File;
        use std::io::{BufRead, BufReader};

        let path = Path::new("data/morphology/fr_lefff.tsv");
        if !path.exists() {
            tracing::debug!("French morphology file not found at {:?}", path);
            return None;
        }

        tracing::info!("Loading French morphology from {:?}", path);

        let file = File::open(path).ok()?;
        let reader = BufReader::new(file);

        let mut analysis: HashMap<String, Vec<MorphEntry>> = HashMap::new();
        let mut synthesis: HashMap<(String, String), Vec<String>> = HashMap::new();
        let mut all_pos_tags: Vec<String> = Vec::new();
        let mut pos_set: std::collections::HashSet<String> = std::collections::HashSet::new();

        for line in reader.lines() {
            let line = line.ok()?;
            // Format: form\tlemma\tpos
            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() >= 3 {
                let form = parts[0].to_lowercase();
                let lemma = parts[1].to_string();
                let pos = parts[2].to_string();

                // Analysis: form → (lemma, pos)
                analysis.entry(form.clone())
                    .or_default()
                    .push(MorphEntry { lemma: lemma.clone(), pos: pos.clone() });

                // Synthesis: (lemma, pos) → form
                synthesis.entry((lemma, pos.clone()))
                    .or_default()
                    .push(form);

                // Track all POS tags for regex matching
                if !pos_set.contains(&pos) {
                    pos_set.insert(pos.clone());
                    all_pos_tags.push(pos);
                }
            }
        }

        let entry_count = analysis.len();
        let synth_count = synthesis.len();
        tracing::info!("Loaded French morphology: {} forms, {} synthesis entries, {} POS tags",
                      entry_count, synth_count, all_pos_tags.len());

        Some(FrenchMorphology { analysis, synthesis, all_pos_tags })
    }

    /// Minimal built-in dictionary for common cases
    fn load_minimal() -> FrenchMorphology {
        let mut analysis: HashMap<String, Vec<MorphEntry>> = HashMap::new();
        let mut synthesis: HashMap<(String, String), Vec<String>> = HashMap::new();
        let mut all_pos_tags = Vec::new();

        // Common determiners
        let determiners = [
            // Definite articles
            ("le", "le", "D m s"),
            ("la", "le", "D f s"),
            ("l'", "le", "D e s"),
            ("les", "le", "D e p"),
            // Indefinite articles
            ("un", "un", "D m s"),
            ("une", "un", "D f s"),
            ("des", "un", "D e p"),
            // Possessives
            ("mon", "mon", "D m s"),
            ("ma", "mon", "D f s"),
            ("mes", "mon", "D e p"),
            ("ton", "ton", "D m s"),
            ("ta", "ton", "D f s"),
            ("tes", "ton", "D e p"),
            ("son", "son", "D m s"),
            ("sa", "son", "D f s"),
            ("ses", "son", "D e p"),
            ("notre", "notre", "D e s"),
            ("nos", "notre", "D e p"),
            ("votre", "votre", "D e s"),
            ("vos", "votre", "D e p"),
            ("leur", "leur", "D e s"),
            ("leurs", "leur", "D e p"),
            // Demonstratives
            ("ce", "ce", "D m s"),
            ("cet", "ce", "D m s"),
            ("cette", "ce", "D f s"),
            ("ces", "ce", "D e p"),
        ];

        // Common adjectives (with gender/number inflection)
        let adjectives = [
            ("grand", "grand", "J m s"),
            ("grande", "grand", "J f s"),
            ("grands", "grand", "J m p"),
            ("grandes", "grand", "J f p"),
            ("petit", "petit", "J m s"),
            ("petite", "petit", "J f s"),
            ("petits", "petit", "J m p"),
            ("petites", "petit", "J f p"),
            ("beau", "beau", "J m s"),
            ("belle", "beau", "J f s"),
            ("beaux", "beau", "J m p"),
            ("belles", "beau", "J f p"),
            ("nouveau", "nouveau", "J m s"),
            ("nouvelle", "nouveau", "J f s"),
            ("nouveaux", "nouveau", "J m p"),
            ("nouvelles", "nouveau", "J f p"),
            ("bon", "bon", "J m s"),
            ("bonne", "bon", "J f s"),
            ("bons", "bon", "J m p"),
            ("bonnes", "bon", "J f p"),
            ("mauvais", "mauvais", "J m s"),
            ("mauvaise", "mauvais", "J f s"),
            ("mauvaises", "mauvais", "J f p"),
            ("premier", "premier", "J m s"),
            ("première", "premier", "J f s"),
            ("premiers", "premier", "J m p"),
            ("premières", "premier", "J f p"),
            ("dernier", "dernier", "J m s"),
            ("dernière", "dernier", "J f s"),
            ("derniers", "dernier", "J m p"),
            ("dernières", "dernier", "J f p"),
            ("vieux", "vieux", "J m s"),
            ("vieille", "vieux", "J f s"),
            ("vieilles", "vieux", "J f p"),
            ("jeune", "jeune", "J e s"),
            ("jeunes", "jeune", "J e p"),
            ("rouge", "rouge", "J e s"),
            ("rouges", "rouge", "J e p"),
            ("blanc", "blanc", "J m s"),
            ("blanche", "blanc", "J f s"),
            ("blancs", "blanc", "J m p"),
            ("blanches", "blanc", "J f p"),
            ("noir", "noir", "J m s"),
            ("noire", "noir", "J f s"),
            ("noirs", "noir", "J m p"),
            ("noires", "noir", "J f p"),
            // Invariable color adjectives
            ("marron", "marron", "J e sp"),
            ("orange", "orange", "J e sp"),
        ];

        // Add entries
        for (form, lemma, pos) in determiners.iter().chain(adjectives.iter()) {
            let form_lower = form.to_lowercase();
            let lemma_str = lemma.to_string();
            let pos_str = pos.to_string();

            analysis.entry(form_lower.clone())
                .or_default()
                .push(MorphEntry { lemma: lemma_str.clone(), pos: pos_str.clone() });

            synthesis.entry((lemma_str, pos_str.clone()))
                .or_default()
                .push(form_lower);

            if !all_pos_tags.contains(&pos_str) {
                all_pos_tags.push(pos_str);
            }
        }

        tracing::debug!("Loaded minimal French morphology: {} forms", analysis.len());

        FrenchMorphology { analysis, synthesis, all_pos_tags }
    }

    /// Analyze a word form: returns all possible (lemma, POS) readings
    pub fn analyze(&self, form: &str) -> &[MorphEntry] {
        let lower = form.to_lowercase();
        self.analysis.get(&lower)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    /// Synthesize: given lemma + exact POS tag, return matching forms
    pub fn synthesize(&self, lemma: &str, pos: &str) -> Vec<&str> {
        let key = (lemma.to_string(), pos.to_string());
        self.synthesis.get(&key)
            .map(|forms| forms.iter().map(|s| s.as_str()).collect())
            .unwrap_or_default()
    }

    /// Synthesize with regex: find forms where POS matches the pattern
    pub fn synthesize_regex(&self, lemma: &str, pos_pattern: &str) -> Vec<&str> {
        // First try exact match (faster)
        let exact = self.synthesize(lemma, pos_pattern);
        if !exact.is_empty() {
            return exact;
        }

        // Try regex match against all POS tags for this lemma
        let re = match Regex::new(&format!("^{}$", pos_pattern)) {
            Ok(r) => r,
            Err(_) => return Vec::new(),
        };

        let mut results = Vec::new();
        for pos in &self.all_pos_tags {
            if re.is_match(pos) {
                let key = (lemma.to_string(), pos.clone());
                if let Some(forms) = self.synthesis.get(&key) {
                    results.extend(forms.iter().map(|s| s.as_str()));
                }
            }
        }
        results
    }

    /// Get the first lemma for a form (for simple cases)
    pub fn get_lemma(&self, form: &str) -> Option<&str> {
        self.analyze(form).first().map(|e| e.lemma.as_str())
    }

    /// Get the first POS for a form (for simple cases)
    pub fn get_pos(&self, form: &str) -> Option<&str> {
        self.analyze(form).first().map(|e| e.pos.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_minimal_determiners() {
        let morph = FrenchMorphology::load_minimal();

        // le -> la (D m s -> D f s)
        let forms = morph.synthesize("le", "D f s");
        assert!(forms.contains(&"la"), "Expected 'la' for D f s");

        // un -> une
        let forms = morph.synthesize("un", "D f s");
        assert!(forms.contains(&"une"), "Expected 'une' for D f s");
    }

    #[test]
    fn test_minimal_adjectives() {
        let morph = FrenchMorphology::load_minimal();

        // grand -> grande
        let forms = morph.synthesize("grand", "J f s");
        assert!(forms.contains(&"grande"), "Expected 'grande' for J f s");

        // beau -> belle
        let forms = morph.synthesize("beau", "J f s");
        assert!(forms.contains(&"belle"), "Expected 'belle' for J f s");
    }

    #[test]
    fn test_analysis() {
        let morph = FrenchMorphology::load_minimal();

        let readings = morph.analyze("grande");
        assert!(!readings.is_empty());
        assert_eq!(readings[0].lemma, "grand");
        assert_eq!(readings[0].pos, "J f s");
    }

    #[test]
    fn test_synthesize_regex() {
        let morph = FrenchMorphology::load_minimal();

        // Match any feminine adjective form of "grand"
        let forms = morph.synthesize_regex("grand", "J f .*");
        assert!(forms.contains(&"grande") || forms.contains(&"grandes"),
                "Expected feminine forms of 'grand'");
    }
}
