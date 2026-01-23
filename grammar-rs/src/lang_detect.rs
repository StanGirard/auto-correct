//! Fast language detection for French and English
//!
//! Uses common words from LanguageTool for accurate detection.
//! Optimized for speed - runs in O(N) with early termination.

use crate::checker::{is_en_common_word, is_fr_common_word};

/// Detected language
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Language {
    English,
    French,
    Unknown,
}

impl Language {
    /// Get the ISO 639-1 code
    pub fn code(&self) -> &'static str {
        match self {
            Language::English => "en",
            Language::French => "fr",
            Language::Unknown => "unknown",
        }
    }
}

/// Language detector for French and English
///
/// Uses ~10k common words per language imported from LanguageTool.
pub struct LanguageDetector {
    /// Minimum confidence threshold (0.0 to 1.0)
    threshold: f32,
    /// Minimum words to check before making a decision
    min_words: usize,
}

impl LanguageDetector {
    /// Create a new language detector with default settings
    pub fn new() -> Self {
        Self {
            threshold: 0.55,
            min_words: 3,
        }
    }

    /// Set the confidence threshold (default: 0.55)
    pub fn with_threshold(mut self, threshold: f32) -> Self {
        self.threshold = threshold.clamp(0.0, 1.0);
        self
    }

    /// Set minimum words to analyze (default: 3)
    pub fn with_min_words(mut self, min_words: usize) -> Self {
        self.min_words = min_words;
        self
    }

    /// Detect the language of a text
    pub fn detect(&self, text: &str) -> Language {
        let result = self.detect_with_confidence(text);
        if result.confidence >= self.threshold {
            result.language
        } else {
            Language::Unknown
        }
    }

    /// Detect language with confidence score
    pub fn detect_with_confidence(&self, text: &str) -> DetectionResult {
        let mut french_score: u32 = 0;
        let mut english_score: u32 = 0;
        let mut word_count: u32 = 0;
        let mut matched_words: u32 = 0;

        // Count French-specific characters (strong indicator)
        let french_chars = text
            .chars()
            .filter(|c| FRENCH_SPECIFIC_CHARS.contains(c))
            .count() as u32;
        french_score += french_chars * 3; // Weight French characters heavily

        // Tokenize and check common words
        for word in text.split(|c: char| !c.is_alphabetic()) {
            if word.is_empty() || word.len() < 2 {
                continue;
            }

            let word_lower = word.to_lowercase();
            word_count += 1;

            // Check against LanguageTool's common words
            let is_french = is_fr_common_word(&word_lower);
            let is_english = is_en_common_word(&word_lower);

            if is_french && !is_english {
                french_score += 2;
                matched_words += 1;
            } else if is_english && !is_french {
                english_score += 2;
                matched_words += 1;
            } else if is_french && is_english {
                // Word is common in both languages - small boost to both
                french_score += 1;
                english_score += 1;
                matched_words += 1;
            }

            // Early termination if clear winner (after min_words)
            if word_count >= self.min_words as u32 && matched_words >= 3 {
                let diff = french_score.abs_diff(english_score);
                if diff > 6 {
                    break;
                }
            }
        }

        // Calculate confidence
        let total_score = french_score + english_score;
        if total_score == 0 || word_count < self.min_words as u32 {
            return DetectionResult {
                language: Language::Unknown,
                confidence: 0.0,
                french_score: french_score as f32,
                english_score: english_score as f32,
            };
        }

        let (language, confidence) = if french_score > english_score {
            (Language::French, french_score as f32 / total_score as f32)
        } else if english_score > french_score {
            (Language::English, english_score as f32 / total_score as f32)
        } else {
            // Tie - use character analysis as tiebreaker
            if french_chars > 0 {
                (Language::French, 0.5)
            } else {
                (Language::English, 0.5)
            }
        };

        DetectionResult {
            language,
            confidence,
            french_score: french_score as f32,
            english_score: english_score as f32,
        }
    }

    /// Check if text is likely French
    pub fn is_french(&self, text: &str) -> bool {
        self.detect(text) == Language::French
    }

    /// Check if text is likely English
    pub fn is_english(&self, text: &str) -> bool {
        self.detect(text) == Language::English
    }
}

impl Default for LanguageDetector {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of language detection with confidence score
#[derive(Debug, Clone)]
pub struct DetectionResult {
    pub language: Language,
    /// Confidence score between 0.0 and 1.0
    pub confidence: f32,
    /// Raw French score
    pub french_score: f32,
    /// Raw English score
    pub english_score: f32,
}

/// French-specific characters (accents, cedilla, ligatures)
const FRENCH_SPECIFIC_CHARS: &[char] = &[
    'é', 'è', 'ê', 'ë', 'à', 'â', 'ù', 'û', 'ô', 'î', 'ï', 'ç', 'œ', 'æ',
    'É', 'È', 'Ê', 'Ë', 'À', 'Â', 'Ù', 'Û', 'Ô', 'Î', 'Ï', 'Ç', 'Œ', 'Æ',
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_english() {
        let detector = LanguageDetector::new();

        let english_texts = [
            "The quick brown fox jumps over the lazy dog.",
            "I went to the store to buy some groceries.",
            "She said that he would be coming later.",
            "This is a simple English sentence.",
            "How are you doing today?",
        ];

        for text in english_texts {
            assert_eq!(
                detector.detect(text),
                Language::English,
                "Failed for: {}",
                text
            );
        }
    }

    #[test]
    fn test_detect_french() {
        let detector = LanguageDetector::new();

        let french_texts = [
            "Le petit chat est sur le tapis.",
            "Je suis allé au magasin pour acheter des légumes.",
            "Elle a dit qu'il viendrait plus tard.",
            "C'est une phrase simple en français.",
            "Bonjour, comment allez-vous aujourd'hui ?",
            "Je ne sais pas quoi faire.",
        ];

        for text in french_texts {
            assert_eq!(
                detector.detect(text),
                Language::French,
                "Failed for: {}",
                text
            );
        }
    }

    #[test]
    fn test_french_characters() {
        let detector = LanguageDetector::new();

        // Text with French accents should be detected as French
        let result = detector.detect_with_confidence("Le café est très bon à Paris.");
        assert_eq!(result.language, Language::French);
        assert!(result.confidence > 0.6);
    }

    #[test]
    fn test_mixed_text() {
        let detector = LanguageDetector::new();

        // Predominantly French
        let text = "Je voudrais un café s'il vous plaît.";
        let result = detector.detect_with_confidence(text);
        assert_eq!(result.language, Language::French);

        // Predominantly English
        let text = "I would like to have a coffee please.";
        let result = detector.detect_with_confidence(text);
        assert_eq!(result.language, Language::English);
    }

    #[test]
    fn test_confidence() {
        let detector = LanguageDetector::new();

        // Clear English should have high confidence
        let result = detector.detect_with_confidence(
            "The quick brown fox jumps over the lazy dog.",
        );
        assert!(result.confidence > 0.6, "Confidence: {}", result.confidence);

        // Clear French should have high confidence
        let result = detector.detect_with_confidence(
            "Le renard brun rapide saute par-dessus le chien paresseux.",
        );
        assert!(result.confidence > 0.6, "Confidence: {}", result.confidence);
    }

    #[test]
    fn test_short_text() {
        let detector = LanguageDetector::new();

        // Short text with clear French indicators
        let result = detector.detect_with_confidence("Je suis là maintenant.");
        assert_eq!(result.language, Language::French);

        // Short text with clear English indicators
        let result = detector.detect_with_confidence("I am here now.");
        assert_eq!(result.language, Language::English);
    }

    #[test]
    fn test_is_french_is_english() {
        let detector = LanguageDetector::new();

        assert!(detector.is_english("This is English text for sure."));
        assert!(detector.is_french("Ceci est du texte français bien sûr."));
        assert!(!detector.is_french("This is English text for sure."));
        assert!(!detector.is_english("Ceci est du texte français bien sûr."));
    }

    #[test]
    fn test_language_code() {
        assert_eq!(Language::English.code(), "en");
        assert_eq!(Language::French.code(), "fr");
        assert_eq!(Language::Unknown.code(), "unknown");
    }

    #[test]
    fn test_real_world_french() {
        let detector = LanguageDetector::new();

        let french_samples = [
            "Merci beaucoup pour votre aide.",
            "Je pense que c'est une bonne idée.",
            "Il fait très beau aujourd'hui.",
            "Nous allons au restaurant ce soir.",
            "Elle travaille dans une grande entreprise.",
        ];

        for text in french_samples {
            let result = detector.detect(text);
            assert_eq!(result, Language::French, "Failed for: {}", text);
        }
    }

    #[test]
    fn test_real_world_english() {
        let detector = LanguageDetector::new();

        let english_samples = [
            "Thank you very much for your help.",
            "I think this is a good idea.",
            "The weather is very nice today.",
            "We are going to the restaurant tonight.",
            "She works in a large company.",
        ];

        for text in english_samples {
            let result = detector.detect(text);
            assert_eq!(result, Language::English, "Failed for: {}", text);
        }
    }
}
