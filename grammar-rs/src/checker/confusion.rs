//! Confusion pair rules - detect commonly confused words
//!
//! Uses data imported from LanguageTool's confusion_sets.txt files.
//! These rules detect homophones and other commonly confused word pairs
//! like "affect/effect", "their/there/they're", etc.

use crate::core::{AnalyzedToken, Match, Severity, TokenKind};
use super::rules::Rule;
use super::data::{get_en_confusions, get_fr_confusions};

/// English confusion pairs rule
/// Detects 700+ commonly confused word pairs
pub struct EnglishConfusionRule;

impl EnglishConfusionRule {
    /// Find the previous word token, skipping whitespace and punctuation
    fn find_prev_word<'a>(tokens: &'a [AnalyzedToken], idx: usize) -> Option<&'a str> {
        let mut i = idx;
        while i > 0 {
            i -= 1;
            if tokens[i].token.kind == TokenKind::Word {
                return Some(tokens[i].token.text);
            }
        }
        None
    }

    /// Find the next word token, skipping whitespace and punctuation
    fn find_next_word<'a>(tokens: &'a [AnalyzedToken], idx: usize) -> Option<&'a str> {
        for token in tokens.iter().skip(idx + 1) {
            if token.token.kind == TokenKind::Word {
                return Some(token.token.text);
            }
        }
        None
    }

    /// Context words that suggest a word is being used correctly
    /// These help reduce false positives
    fn check_context(&self, tokens: &[AnalyzedToken], idx: usize, word: &str, suggested: &str) -> bool {
        let prev_word = Self::find_prev_word(tokens, idx).map(|s| s.to_lowercase());
        let next_word = Self::find_next_word(tokens, idx).map(|s| s.to_lowercase());

        match (word, suggested) {
            // === A ===
            // affect/effect
            ("affect", "effect") => {
                if let Some(prev) = &prev_word {
                    if matches!(prev.as_str(), "the" | "an" | "this" | "that" | "no" | "any" | "positive" | "negative") {
                        return true;
                    }
                }
                false
            }
            ("effect", "affect") => {
                if let Some(prev) = &prev_word {
                    if matches!(prev.as_str(), "will" | "to" | "can" | "may" | "could" | "would" | "might" | "does" | "did") {
                        return true;
                    }
                }
                false
            }

            // accept/except
            ("accept", "except") => {
                if let Some(next) = &next_word {
                    if matches!(next.as_str(), "for" | "that" | "when" | "if" | "in") {
                        return true;
                    }
                }
                false
            }
            ("except", "accept") => {
                if let Some(next) = &next_word {
                    if matches!(next.as_str(), "the" | "my" | "your" | "this" | "our" | "his" | "her" | "their") {
                        return true;
                    }
                }
                false
            }

            // advice/advise
            ("advice", "advise") => {
                if let Some(prev) = &prev_word {
                    if matches!(prev.as_str(), "to" | "would" | "will" | "can" | "should" | "must" | "please") {
                        return true;
                    }
                }
                false
            }
            ("advise", "advice") => {
                if let Some(prev) = &prev_word {
                    if matches!(prev.as_str(), "the" | "my" | "your" | "his" | "her" | "some" | "good" | "bad") {
                        return true;
                    }
                }
                false
            }

            // aloud/allowed
            ("aloud", "allowed") => {
                if let Some(prev) = &prev_word {
                    if matches!(prev.as_str(), "not" | "be" | "is" | "are" | "was" | "were") {
                        return true;
                    }
                }
                false
            }
            ("allowed", "aloud") => {
                if let Some(prev) = &prev_word {
                    if matches!(prev.as_str(), "read" | "say" | "speak" | "think" | "said" | "spoke") {
                        return true;
                    }
                }
                false
            }

            // === B ===
            // bare/bear
            ("bare", "bear") => {
                if let Some(next) = &next_word {
                    if matches!(next.as_str(), "with" | "in" | "the" | "market" | "witness") {
                        return true;
                    }
                }
                false
            }
            ("bear", "bare") => {
                if let Some(next) = &next_word {
                    if matches!(next.as_str(), "feet" | "hands" | "skin" | "minimum" | "essentials" | "bones") {
                        return true;
                    }
                }
                false
            }

            // brake/break
            ("brake", "break") => {
                if let Some(next) = &next_word {
                    if matches!(next.as_str(), "the" | "a" | "time" | "down" | "up" | "in" | "out" | "free") {
                        return true;
                    }
                }
                if let Some(prev) = &prev_word {
                    if matches!(prev.as_str(), "to" | "will" | "can" | "could" | "would") {
                        return true;
                    }
                }
                false
            }
            ("break", "brake") => {
                if let Some(prev) = &prev_word {
                    if matches!(prev.as_str(), "the" | "emergency" | "hand" | "foot" | "parking") {
                        return true;
                    }
                }
                false
            }

            // breath/breathe
            ("breath", "breathe") => {
                if let Some(prev) = &prev_word {
                    if matches!(prev.as_str(), "to" | "can" | "could" | "cannot" | "can't" | "couldn't") {
                        return true;
                    }
                }
                false
            }
            ("breathe", "breath") => {
                if let Some(prev) = &prev_word {
                    if matches!(prev.as_str(), "a" | "deep" | "take" | "your" | "his" | "her" | "my" | "bad") {
                        return true;
                    }
                }
                false
            }

            // === C ===
            // complement/compliment
            ("complement", "compliment") => {
                if let Some(prev) = &prev_word {
                    if matches!(prev.as_str(), "a" | "nice" | "pay" | "give" | "receive" | "the") {
                        return true;
                    }
                }
                false
            }
            ("compliment", "complement") => {
                if let Some(next) = &next_word {
                    if matches!(next.as_str(), "each" | "the" | "to") {
                        return true;
                    }
                }
                false
            }

            // council/counsel
            ("council", "counsel") => {
                if let Some(prev) = &prev_word {
                    if matches!(prev.as_str(), "legal" | "seek" | "provide" | "give") {
                        return true;
                    }
                }
                false
            }
            ("counsel", "council") => {
                if let Some(prev) = &prev_word {
                    if matches!(prev.as_str(), "city" | "town" | "local" | "the" | "student") {
                        return true;
                    }
                }
                false
            }

            // === D ===
            // desert/dessert
            ("desert", "dessert") => {
                if let Some(prev) = &prev_word {
                    if matches!(prev.as_str(), "for" | "chocolate" | "delicious" | "sweet" | "ice") {
                        return true;
                    }
                }
                false
            }
            ("dessert", "desert") => {
                if let Some(prev) = &prev_word {
                    if matches!(prev.as_str(), "sahara" | "mojave" | "gobi" | "arabian" | "arid") {
                        return true;
                    }
                }
                if let Some(next) = &next_word {
                    if matches!(next.as_str(), "island" | "storm" | "climate") {
                        return true;
                    }
                }
                false
            }

            // === E ===
            // ensure/insure
            ("ensure", "insure") => {
                if let Some(next) = &next_word {
                    if matches!(next.as_str(), "your" | "the" | "my" | "his" | "her" | "their" | "car" | "house" | "life" | "health") {
                        return true;
                    }
                }
                false
            }
            ("insure", "ensure") => {
                if let Some(next) = &next_word {
                    if matches!(next.as_str(), "that" | "success" | "safety" | "quality") {
                        return true;
                    }
                }
                false
            }

            // === F ===
            // farther/further
            ("farther", "further") => {
                if let Some(next) = &next_word {
                    if matches!(next.as_str(), "information" | "details" | "discussion" | "research" | "investigation" | "study" | "notice") {
                        return true;
                    }
                }
                false
            }
            ("further", "farther") => {
                if let Some(prev) = &prev_word {
                    if matches!(prev.as_str(), "much" | "even" | "a" | "bit") {
                        if let Some(next) = &next_word {
                            if matches!(next.as_str(), "away" | "down" | "along" | "north" | "south" | "east" | "west") {
                                return true;
                            }
                        }
                    }
                }
                false
            }

            // === H ===
            // hear/here
            ("hear", "here") => {
                if let Some(prev) = &prev_word {
                    if matches!(prev.as_str(), "over" | "right" | "come" | "stay" | "sit" | "stand" | "around") {
                        return true;
                    }
                }
                false
            }
            ("here", "hear") => {
                if let Some(prev) = &prev_word {
                    if matches!(prev.as_str(), "can" | "did" | "to" | "want" | "wanted" | "can't" | "couldn't") {
                        return true;
                    }
                }
                false
            }

            // === L ===
            // lead/led - past tense confusion
            ("lead", "led") => {
                if let Some(prev) = &prev_word {
                    if matches!(prev.as_str(), "was" | "were" | "been" | "being" | "has" | "had" | "have") {
                        return true;
                    }
                }
                false
            }
            ("led", "lead") => {
                if let Some(prev) = &prev_word {
                    if matches!(prev.as_str(), "will" | "to" | "can" | "could" | "would" | "should" | "must" | "might") {
                        return true;
                    }
                }
                false
            }

            // loose/lose
            ("loose", "lose") => {
                if let Some(next) = &next_word {
                    if matches!(next.as_str(), "weight" | "money" | "time" | "control" | "hope" | "faith" | "track" | "sight" | "interest" | "patience" | "temper" | "sleep" | "focus" | "touch" | "ground") {
                        return true;
                    }
                }
                if let Some(prev) = &prev_word {
                    if matches!(prev.as_str(), "will" | "to" | "can" | "might" | "could" | "would" | "going" | "about" | "don't" | "didn't" | "won't") {
                        return true;
                    }
                }
                false
            }
            ("lose", "loose") => {
                if let Some(prev) = &prev_word {
                    if matches!(prev.as_str(), "a" | "the" | "too" | "very" | "quite" | "is" | "are" | "was" | "were" | "got" | "getting" | "came" | "come" | "break" | "broke" | "set" | "cut") {
                        return true;
                    }
                }
                false
            }

            // === P ===
            // passed/past
            ("passed", "past") => {
                if let Some(prev) = &prev_word {
                    if matches!(prev.as_str(), "the" | "in" | "walked" | "drove" | "ran" | "went" | "flew") {
                        return true;
                    }
                }
                false
            }
            ("past", "passed") => {
                if let Some(prev) = &prev_word {
                    if matches!(prev.as_str(), "has" | "have" | "had" | "was" | "were" | "been" | "just" | "already") {
                        return true;
                    }
                }
                false
            }

            // peace/piece
            ("peace", "piece") => {
                if let Some(prev) = &prev_word {
                    if matches!(prev.as_str(), "a" | "every" | "each" | "that" | "this") {
                        if let Some(next) = &next_word {
                            if matches!(next.as_str(), "of" | "by") {
                                return true;
                            }
                        }
                    }
                }
                false
            }
            ("piece", "peace") => {
                if let Some(prev) = &prev_word {
                    if matches!(prev.as_str(), "world" | "inner" | "at" | "in" | "make" | "rest" | "keeping") {
                        return true;
                    }
                }
                false
            }

            // principal/principle
            ("principal", "principle") => {
                if let Some(prev) = &prev_word {
                    if matches!(prev.as_str(), "a" | "the" | "basic" | "fundamental" | "key" | "main" | "core" | "guiding" | "first" | "general" | "moral" | "ethical") {
                        return true;
                    }
                }
                false
            }
            ("principle", "principal") => {
                if let Some(prev) = &prev_word {
                    if matches!(prev.as_str(), "school" | "high" | "the" | "assistant" | "vice") {
                        return true;
                    }
                }
                if let Some(next) = &next_word {
                    if matches!(next.as_str(), "amount" | "balance" | "sum" | "investment") {
                        return true;
                    }
                }
                false
            }

            // === Q ===
            // quiet/quite
            ("quiet", "quite") => {
                if let Some(next) = &next_word {
                    if matches!(next.as_str(), "a" | "good" | "bad" | "nice" | "well" | "right" | "sure" | "certain" | "different" | "similar" | "the" | "often" | "simply" | "frankly" | "honestly") {
                        return true;
                    }
                }
                false
            }
            ("quite", "quiet") => {
                if let Some(prev) = &prev_word {
                    if matches!(prev.as_str(), "be" | "stay" | "keep" | "remain" | "very" | "too" | "so" | "relatively" | "unusually") {
                        return true;
                    }
                }
                false
            }

            // === R ===
            // right/write
            ("right", "write") => {
                if let Some(prev) = &prev_word {
                    if matches!(prev.as_str(), "to" | "can" | "will" | "would" | "could" | "should" | "must" | "please" | "let" | "didn't" | "don't" | "won't" | "can't" | "couldn't") {
                        if let Some(next) = &next_word {
                            if matches!(next.as_str(), "a" | "the" | "this" | "that" | "an" | "down" | "about" | "to" | "it" | "something" | "anything" | "code" | "book" | "letter" | "email") {
                                return true;
                            }
                        }
                    }
                }
                false
            }

            // === S ===
            // stationary/stationery
            ("stationary", "stationery") => {
                if let Some(next) = &next_word {
                    if matches!(next.as_str(), "store" | "shop" | "supplies" | "paper" | "set" | "kit") {
                        return true;
                    }
                }
                false
            }
            ("stationery", "stationary") => {
                if let Some(prev) = &prev_word {
                    if matches!(prev.as_str(), "remain" | "remained" | "stay" | "stayed" | "kept" | "keep" | "is" | "was" | "were" | "are" | "completely" | "perfectly" | "absolutely") {
                        return true;
                    }
                }
                false
            }

            // === T ===
            // than/then
            ("than", "then") => {
                if let Some(prev) = &prev_word {
                    if matches!(prev.as_str(), "more" | "less" | "better" | "worse" | "rather" | "other" | "higher" | "lower" | "bigger" | "smaller" | "faster" | "slower" | "older" | "younger" | "greater" | "fewer") {
                        return false; // This is correct usage
                    }
                }
                if let Some(next) = &next_word {
                    if matches!(next.as_str(), "he" | "she" | "i" | "we" | "they" | "it" | "the" | "suddenly" | "again" | "later" | "finally") {
                        return true;
                    }
                }
                false
            }
            ("then", "than") => {
                if let Some(prev) = &prev_word {
                    if matches!(prev.as_str(), "more" | "less" | "better" | "worse" | "rather" | "other" | "higher" | "lower" | "bigger" | "smaller" | "faster" | "slower" | "older" | "younger" | "greater" | "fewer") {
                        return true;
                    }
                }
                false
            }

            // to/too
            ("to", "too") => {
                if let Some(next) = &next_word {
                    if matches!(next.as_str(), "much" | "many" | "little" | "few" | "late" | "early" | "big" | "small" | "fast" | "slow" | "hot" | "cold" | "hard" | "easy" | "long" | "short" | "good" | "bad" | "tired" | "busy" | "young" | "old") {
                        return true;
                    }
                }
                false
            }
            ("too", "to") => {
                // "too go" should be "to go"
                if let Some(next) = &next_word {
                    if matches!(next.as_str(), "go" | "be" | "do" | "have" | "get" | "make" | "see" | "take" | "know" | "come" | "think" | "say" | "use" | "find" | "give" | "tell" | "work" | "call" | "try" | "ask" | "need" | "feel" | "become" | "leave" | "put" | "mean" | "keep" | "let" | "begin" | "seem" | "help" | "show" | "hear" | "play" | "run" | "move" | "live" | "believe") {
                        return true;
                    }
                }
                false
            }

            // === W ===
            // weather/whether
            ("weather", "whether") => {
                if let Some(next) = &next_word {
                    if matches!(next.as_str(), "or" | "to" | "it" | "he" | "she" | "they" | "we" | "you" | "this" | "that" | "the") {
                        return true;
                    }
                }
                false
            }
            ("whether", "weather") => {
                if let Some(prev) = &prev_word {
                    if matches!(prev.as_str(), "the" | "bad" | "good" | "nice" | "cold" | "hot" | "warm" | "rainy" | "sunny" | "stormy") {
                        return true;
                    }
                }
                if let Some(next) = &next_word {
                    if matches!(next.as_str(), "forecast" | "report" | "conditions" | "permitting" | "changes" | "pattern") {
                        return true;
                    }
                }
                false
            }

            // whose/who's
            ("whose", "who's") => {
                if let Some(next) = &next_word {
                    if matches!(next.as_str(), "going" | "coming" | "there" | "that" | "this" | "the" | "a" | "been" | "got" | "getting" | "doing" | "calling" | "asking" | "saying" | "talking") {
                        return true;
                    }
                }
                false
            }
            ("who's", "whose") => {
                if let Some(next) = &next_word {
                    if matches!(next.as_str(), "car" | "house" | "book" | "phone" | "idea" | "fault" | "turn" | "job" | "responsibility" | "name" | "bag" | "coat" | "keys" | "money" | "problem" | "dog" | "cat") {
                        return true;
                    }
                }
                false
            }

            // For other pairs, be conservative
            _ => false,
        }
    }
}

impl Rule for EnglishConfusionRule {
    fn id(&self) -> &str {
        "EN_CONFUSION"
    }

    fn check(&self, _text: &str, tokens: &[AnalyzedToken]) -> Option<Match> {
        for (idx, token) in tokens.iter().enumerate() {
            if token.token.kind != TokenKind::Word {
                continue;
            }

            let word_lower = token.token.text.to_lowercase();

            if let Some(confusions) = get_en_confusions(&word_lower) {
                for (suggested, _factor) in confusions {
                    // Check context to decide if this is likely a confusion
                    if self.check_context(tokens, idx, &word_lower, suggested) {
                        return Some(Match {
                            span: token.token.span.clone(),
                            message: format!(
                                "Possible confusion: did you mean '{}'?",
                                suggested
                            ),
                            rule_id: self.id().to_string(),
                            suggestions: vec![suggested.to_string()],
                            severity: Severity::Warning,
                        });
                    }
                }
            }
        }
        None
    }
}

/// French confusion pairs rule
/// Detects commonly confused French words (homophones)
pub struct FrenchConfusionRule;

impl FrenchConfusionRule {
    fn check_context(&self, tokens: &[AnalyzedToken], idx: usize, word: &str, suggested: &str) -> bool {
        // Use the same helper functions as EnglishConfusionRule
        let prev_word = EnglishConfusionRule::find_prev_word(tokens, idx).map(|s| s.to_lowercase());
        let next_word = EnglishConfusionRule::find_next_word(tokens, idx).map(|s| s.to_lowercase());

        match (word, suggested) {
            // === B ===
            // bon/bond - "faire un bon" vs "faire un bond"
            ("bon", "bond") => {
                if let Some(next) = &next_word {
                    // "un bon en avant" should be "un bond en avant"
                    if matches!(next.as_str(), "en" | "de" | "vers" | "par-dessus") {
                        return true;
                    }
                }
                if let Some(prev) = &prev_word {
                    // "faire un bon" + movement word
                    if matches!(prev.as_str(), "un" | "ce" | "quel") {
                        if let Some(next) = &next_word {
                            if matches!(next.as_str(), "en" | "de" | "prodigieux" | "spectaculaire") {
                                return true;
                            }
                        }
                    }
                }
                false
            }
            ("bond", "bon") => {
                // "un bond repas" should be "un bon repas"
                if let Some(next) = &next_word {
                    if matches!(next.as_str(), "repas" | "moment" | "travail" | "choix" | "ami" | "jour" | "voyage" | "film" | "livre") {
                        return true;
                    }
                }
                false
            }

            // === D ===
            // dans/dent
            ("dans", "dent") => {
                // "mal de dans" should be "mal de dent"
                if let Some(prev) = &prev_word {
                    if matches!(prev.as_str(), "de" | "la" | "une" | "ma" | "sa" | "cette") {
                        return true;
                    }
                }
                false
            }

            // don/donc
            ("don", "donc") => {
                // At beginning of sentence or after comma
                if prev_word.is_none() {
                    return true;
                }
                if let Some(prev) = &prev_word {
                    if matches!(prev.as_str(), "," | ";" | "et" | "mais" | "or" | "car") {
                        return true;
                    }
                }
                false
            }
            ("donc", "don") => {
                // "faire un donc" should be "faire un don"
                if let Some(prev) = &prev_word {
                    if matches!(prev.as_str(), "un" | "ce" | "son" | "mon" | "ton" | "le" | "du") {
                        return true;
                    }
                }
                false
            }

            // === M ===
            // moi/mois
            ("moi", "mois") => {
                // "le moi dernier" should be "le mois dernier"
                if let Some(prev) = &prev_word {
                    if matches!(prev.as_str(), "le" | "ce" | "un" | "du" | "au" | "chaque" | "premier" | "dernier" | "prochain") {
                        return true;
                    }
                }
                if let Some(next) = &next_word {
                    if matches!(next.as_str(), "dernier" | "prochain" | "de" | "passé" | "suivant" | "précédent") {
                        return true;
                    }
                }
                false
            }
            ("mois", "moi") => {
                // "pour mois" should be "pour moi", "chez mois" should be "chez moi"
                if let Some(prev) = &prev_word {
                    if matches!(prev.as_str(), "pour" | "chez" | "avec" | "sans" | "contre" | "selon" | "après" | "devant" | "derrière" | "comme") {
                        return true;
                    }
                }
                false
            }

            // === N ===
            // notre/nôtre
            ("notre", "nôtre") => {
                // "le notre" should be "le nôtre"
                if let Some(prev) = &prev_word {
                    if matches!(prev.as_str(), "le" | "la" | "les" | "du" | "au") {
                        return true;
                    }
                }
                false
            }

            // === P ===
            // pain/pin
            ("pain", "pin") => {
                // "pomme de pain" should be "pomme de pin"
                if let Some(prev) = &prev_word {
                    if prev == "de" {
                        // Look for "pomme" before "de"
                        if idx >= 2 {
                            let prev_prev = EnglishConfusionRule::find_prev_word(tokens, idx - 1).map(|s| s.to_lowercase());
                            if let Some(pp) = prev_prev {
                                if pp == "pomme" || pp == "aiguille" {
                                    return true;
                                }
                            }
                        }
                    }
                }
                false
            }

            // peau/pot
            ("peau", "pot") => {
                // "un peau de fleurs" should be "un pot de fleurs"
                if let Some(next) = &next_word {
                    if matches!(next.as_str(), "de" | "à" | "d'échappement") {
                        if let Some(prev) = &prev_word {
                            if matches!(prev.as_str(), "un" | "le" | "ce" | "du") {
                                return true;
                            }
                        }
                    }
                }
                false
            }
            ("pot", "peau") => {
                // "une pot de bête" should be "une peau de bête"
                if let Some(prev) = &prev_word {
                    if matches!(prev.as_str(), "la" | "une" | "sa" | "ma" | "ta" | "cette") {
                        return true;
                    }
                }
                false
            }

            // père/paire
            ("père", "paire") => {
                // "une père de chaussures" should be "une paire de chaussures"
                if let Some(prev) = &prev_word {
                    if matches!(prev.as_str(), "une" | "la" | "cette") {
                        if let Some(next) = &next_word {
                            if next == "de" {
                                return true;
                            }
                        }
                    }
                }
                false
            }

            // prix/pris
            ("prix", "pris") => {
                // "j'ai prix" should be "j'ai pris"
                if let Some(prev) = &prev_word {
                    if matches!(prev.as_str(), "ai" | "as" | "a" | "avons" | "avez" | "ont" | "avoir" | "été") {
                        return true;
                    }
                }
                false
            }
            ("pris", "prix") => {
                // "le pris du pain" should be "le prix du pain"
                if let Some(prev) = &prev_word {
                    if matches!(prev.as_str(), "le" | "un" | "ce" | "du" | "au" | "quel" | "bon" | "meilleur" | "petit") {
                        return true;
                    }
                }
                false
            }

            // === T ===
            // tante/tente
            ("tante", "tente") => {
                // "planter une tante" should be "planter une tente"
                if let Some(prev) = &prev_word {
                    if matches!(prev.as_str(), "une" | "la" | "cette" | "sa" | "ma") {
                        if let Some(next) = &next_word {
                            if matches!(next.as_str(), "de" | "camping" | "familiale" | "militaire") {
                                return true;
                            }
                        }
                    }
                }
                false
            }
            ("tente", "tante") => {
                // "ma tente Jeanne" should be "ma tante Jeanne"
                if let Some(prev) = &prev_word {
                    if matches!(prev.as_str(), "ma" | "sa" | "ta" | "notre" | "leur" | "votre") {
                        // If next word starts with uppercase, likely a name
                        if let Some(next_token) = tokens.get(idx + 1) {
                            if next_token.token.kind == TokenKind::Word {
                                let first_char = next_token.token.text.chars().next();
                                if let Some(c) = first_char {
                                    if c.is_uppercase() {
                                        return true;
                                    }
                                }
                            }
                        }
                    }
                }
                false
            }

            // toi/toit
            ("toi", "toit") => {
                // "le toi de la maison" should be "le toit de la maison"
                if let Some(prev) = &prev_word {
                    if matches!(prev.as_str(), "le" | "un" | "ce" | "du" | "au" | "sur" | "sous") {
                        return true;
                    }
                }
                false
            }
            ("toit", "toi") => {
                // "pour toit" should be "pour toi"
                if let Some(prev) = &prev_word {
                    if matches!(prev.as_str(), "pour" | "chez" | "avec" | "sans" | "contre" | "sur" | "après" | "devant" | "comme") {
                        return true;
                    }
                }
                false
            }

            // très/trait
            ("très", "trait") => {
                // "un très d'union" should be "un trait d'union"
                if let Some(prev) = &prev_word {
                    if matches!(prev.as_str(), "un" | "le" | "ce" | "du") {
                        return true;
                    }
                }
                false
            }

            // === V ===
            // vain/vin/vingt
            ("vain", "vin") => {
                // "un verre de vain" should be "un verre de vin"
                if let Some(prev) = &prev_word {
                    if matches!(prev.as_str(), "de" | "du" | "au" | "le" | "un" | "ce" | "bon" | "rouge" | "blanc" | "rosé") {
                        return true;
                    }
                }
                false
            }
            ("vain", "vingt") => {
                // "vain ans" should be "vingt ans"
                if let Some(next) = &next_word {
                    if matches!(next.as_str(), "ans" | "euros" | "minutes" | "secondes" | "heures" | "jours" | "mois" | "personnes" | "et") {
                        return true;
                    }
                }
                false
            }
            ("vin", "vain") => {
                // "en vin" should be "en vain"
                if let Some(prev) = &prev_word {
                    if prev == "en" {
                        return true;
                    }
                }
                false
            }
            ("vin", "vingt") => {
                // "vin euros" should be "vingt euros"
                if let Some(next) = &next_word {
                    if matches!(next.as_str(), "euros" | "minutes" | "secondes" | "heures" | "jours" | "ans" | "mois" | "et" | "personnes" | "centimes") {
                        return true;
                    }
                }
                false
            }
            ("vingt", "vin") => {
                // "un verre de vingt" should be "un verre de vin"
                if let Some(prev) = &prev_word {
                    if prev == "de" {
                        return true;
                    }
                }
                false
            }

            // verre/vers/vert
            ("verre", "vers") => {
                // "aller verre Paris" should be "aller vers Paris"
                if let Some(prev) = &prev_word {
                    if matches!(prev.as_str(), "aller" | "va" | "allons" | "allez" | "vont" | "allait" | "tourner" | "regarder" | "se" | "tendre" | "diriger") {
                        return true;
                    }
                }
                false
            }
            ("verre", "vert") => {
                // "le feu verre" should be "le feu vert"
                if let Some(prev) = &prev_word {
                    if matches!(prev.as_str(), "feu" | "haricot" | "haricots" | "citron") {
                        return true;
                    }
                }
                false
            }
            ("vers", "verre") => {
                // "un vers d'eau" should be "un verre d'eau"
                if let Some(prev) = &prev_word {
                    if matches!(prev.as_str(), "un" | "le" | "ce" | "mon" | "son" | "du") {
                        if let Some(next) = &next_word {
                            if matches!(next.as_str(), "de" | "d'eau" | "d'") {
                                return true;
                            }
                        }
                    }
                }
                false
            }
            ("vert", "verre") => {
                // "un vert d'eau" should be "un verre d'eau"
                if let Some(prev) = &prev_word {
                    if matches!(prev.as_str(), "un" | "le" | "ce" | "mon" | "son" | "du") {
                        if let Some(next) = &next_word {
                            if matches!(next.as_str(), "de" | "d'" | "à") {
                                return true;
                            }
                        }
                    }
                }
                false
            }

            // === NEW CONFUSION PAIRS ===

            // a/à - accent confusion
            ("a", "à") => {
                // "a Paris" should be "à Paris" (preposition before place)
                if let Some(next) = &next_word {
                    // Before a capitalized word (likely a place)
                    if let Some(next_token) = tokens.get(idx + 1) {
                        if next_token.token.kind == TokenKind::Word {
                            if let Some(c) = next_token.token.text.chars().next() {
                                if c.is_uppercase() {
                                    return true;
                                }
                            }
                        }
                    }
                    // Common preposition patterns
                    if matches!(next.as_str(), "la" | "l'" | "le" | "les" | "ce" | "cette" | "mon" | "ma" | "ton" | "ta" | "son" | "sa" | "notre" | "votre" | "leur" | "cause" | "côté" | "travers" | "partir" | "nouveau" | "bientôt") {
                        return true;
                    }
                }
                false
            }

            // ou/où - accent confusion
            ("ou", "où") => {
                // "ou vas-tu" should be "où vas-tu"
                if let Some(next) = &next_word {
                    // Interrogative contexts
                    if matches!(next.as_str(), "vas" | "va" | "allez" | "allons" | "vont" | "est" | "sont" | "es" | "suis" | "êtes" | "sommes" | "habites" | "habite" | "habitez") {
                        return true;
                    }
                }
                if let Some(prev) = &prev_word {
                    // After "là" or "savoir"
                    if matches!(prev.as_str(), "là" | "savoir" | "sais" | "sait" | "savons" | "savez" | "savent" | "voilà" | "moment") {
                        return true;
                    }
                }
                false
            }

            // est/et - verb vs conjunction
            ("est", "et") => {
                // "il est moi" should be "il et moi"
                if let Some(prev) = &prev_word {
                    if matches!(prev.as_str(), "toi" | "moi" | "lui" | "elle" | "nous" | "vous" | "eux" | "elles") {
                        return true;
                    }
                }
                false
            }
            ("et", "est") => {
                // "il et parti" should be "il est parti"
                if let Some(prev) = &prev_word {
                    if matches!(prev.as_str(), "il" | "elle" | "on" | "ce" | "c'" | "qui" | "tout" | "rien" | "cela" | "ceci" | "ça") {
                        if let Some(next) = &next_word {
                            // Followed by past participle or adjective patterns
                            if matches!(next.as_str(), "parti" | "arrivé" | "venu" | "allé" | "resté" | "devenu" | "né" | "mort" | "tombé" | "entré" | "sorti" | "monté" | "descendu" | "passé" | "retourné" | "revenu" | "rentré" | "possible" | "impossible" | "vrai" | "faux" | "certain" | "évident" | "clair" | "important" | "nécessaire" | "temps" | "là" | "ici") {
                                return true;
                            }
                        }
                    }
                }
                false
            }

            // sont/son - verb vs possessive
            ("sont", "son") => {
                // "sont père" should be "son père"
                if let Some(next) = &next_word {
                    if matches!(next.as_str(), "père" | "mère" | "frère" | "soeur" | "fils" | "fille" | "ami" | "amie" | "travail" | "bureau" | "livre" | "téléphone" | "ordinateur" | "vélo" | "voiture" | "chien" | "chat" | "nom" | "prénom" | "adresse" | "numéro" | "âge" | "anniversaire") {
                        return true;
                    }
                }
                false
            }
            ("son", "sont") => {
                // "ils son partis" should be "ils sont partis"
                if let Some(prev) = &prev_word {
                    if matches!(prev.as_str(), "ils" | "elles" | "qui" | "ce") {
                        return true;
                    }
                }
                false
            }

            // ont/on - verb vs pronoun
            ("ont", "on") => {
                // "ont dit que" should be "on dit que"
                if prev_word.is_none() {
                    return true; // At start of sentence
                }
                if let Some(prev) = &prev_word {
                    if matches!(prev.as_str(), "," | ";" | "et" | "mais" | "ou" | "donc" | "car" | "quand" | "si" | "que" | "lorsque" | "comme") {
                        return true;
                    }
                }
                false
            }
            ("on", "ont") => {
                // "ils on mangé" should be "ils ont mangé"
                if let Some(prev) = &prev_word {
                    if matches!(prev.as_str(), "ils" | "elles" | "qui") {
                        return true;
                    }
                }
                false
            }

            // ce/se - demonstrative vs reflexive
            ("ce", "se") => {
                // "ce lever" should be "se lever"
                if let Some(next) = &next_word {
                    // Reflexive verbs
                    if matches!(next.as_str(), "lever" | "coucher" | "laver" | "habiller" | "réveiller" | "promener" | "souvenir" | "rappeler" | "sentir" | "trouver" | "passer" | "dépêcher" | "tromper" | "moquer" | "plaindre" | "battre") {
                        return true;
                    }
                }
                false
            }
            ("se", "ce") => {
                // "se livre" should be "ce livre"
                if let Some(next) = &next_word {
                    // Common nouns
                    if matches!(next.as_str(), "livre" | "film" | "jour" | "soir" | "matin" | "moment" | "qui" | "que" | "dont" | "n'est" | "n'" | "sera" | "serait" | "fut" | "sont" | "mois" | "week-end") {
                        return true;
                    }
                }
                false
            }

            // ces/ses - demonstrative vs possessive
            ("ces", "ses") => {
                // "ces parents" (his/her parents) might be "ses parents"
                // This is tricky - both can be correct. Skip for now.
                false
            }
            ("ses", "ces") => {
                false
            }

            // foi/fois/foie
            ("foi", "fois") => {
                // "une foi" should be "une fois"
                if let Some(prev) = &prev_word {
                    if matches!(prev.as_str(), "une" | "deux" | "trois" | "quatre" | "cinq" | "plusieurs" | "chaque" | "première" | "dernière" | "prochaine" | "cette" | "la" | "combien") {
                        return true;
                    }
                }
                false
            }
            ("fois", "foi") => {
                // "la fois" should be "la foi" (in religious context)
                if let Some(prev) = &prev_word {
                    if matches!(prev.as_str(), "ma" | "sa" | "ta" | "notre" | "votre" | "leur" | "bonne" | "mauvaise") {
                        return true;
                    }
                }
                false
            }
            ("foi", "foie") => {
                // "crise de foi" should be "crise de foie"
                if let Some(prev) = &prev_word {
                    if prev == "de" {
                        // Look for "crise" or "mal"
                        if idx >= 2 {
                            let prev_prev = EnglishConfusionRule::find_prev_word(tokens, idx - 1).map(|s| s.to_lowercase());
                            if let Some(pp) = prev_prev {
                                if matches!(pp.as_str(), "crise" | "mal") {
                                    return true;
                                }
                            }
                        }
                    }
                }
                false
            }
            ("foie", "foi") => {
                // "bonne foie" should be "bonne foi"
                if let Some(prev) = &prev_word {
                    if matches!(prev.as_str(), "bonne" | "mauvaise" | "ma" | "sa" | "ta") {
                        return true;
                    }
                }
                false
            }

            // voie/voix
            ("voie", "voix") => {
                // "a haute voie" should be "à haute voix"
                if let Some(prev) = &prev_word {
                    if matches!(prev.as_str(), "haute" | "basse" | "grosse" | "petite" | "belle" | "jolie" | "sa" | "ma" | "ta" | "une" | "la" | "cette") {
                        return true;
                    }
                }
                false
            }
            ("voix", "voie") => {
                // "la voix ferrée" should be "la voie ferrée"
                if let Some(next) = &next_word {
                    if matches!(next.as_str(), "ferrée" | "lactée" | "publique" | "rapide" | "express") {
                        return true;
                    }
                }
                if let Some(prev) = &prev_word {
                    if matches!(prev.as_str(), "bonne" | "mauvaise" | "en") {
                        return true;
                    }
                }
                false
            }

            // près/prêt
            ("près", "prêt") => {
                // "je suis près" should be "je suis prêt"
                if let Some(prev) = &prev_word {
                    if matches!(prev.as_str(), "suis" | "es" | "est" | "sommes" | "êtes" | "sont" | "étais" | "était" | "étions" | "étiez" | "étaient" | "serai" | "seras" | "sera" | "serons" | "serez" | "seront") {
                        if let Some(next) = &next_word {
                            // Not followed by "de" (which would be "près de")
                            if next != "de" {
                                return true;
                            }
                        } else {
                            return true; // End of sentence
                        }
                    }
                }
                false
            }
            ("prêt", "près") => {
                // "prêt de la maison" should be "près de la maison"
                if let Some(next) = &next_word {
                    if next == "de" {
                        return true;
                    }
                }
                false
            }

            // soi/soit
            ("soi", "soit") => {
                // "soi il vient, soi il ne vient pas" should be "soit...soit"
                if let Some(next) = &next_word {
                    if matches!(next.as_str(), "il" | "elle" | "on" | "tu" | "je" | "nous" | "vous" | "ils" | "elles" | "que") {
                        return true;
                    }
                }
                false
            }
            ("soit", "soi") => {
                // "en soit" should be "en soi"
                if let Some(prev) = &prev_word {
                    if prev == "en" {
                        return true;
                    }
                }
                false
            }

            // mer/mère
            ("mer", "mère") => {
                // "ma mer" should be "ma mère"
                if let Some(prev) = &prev_word {
                    if matches!(prev.as_str(), "ma" | "sa" | "ta" | "notre" | "votre" | "leur" | "belle" | "grand" | "la") {
                        // Check if not followed by words like "Méditerranée"
                        if let Some(next) = &next_word {
                            if !matches!(next.as_str(), "méditerranée" | "rouge" | "noire" | "baltique" | "du") {
                                return true;
                            }
                        } else {
                            return true;
                        }
                    }
                }
                false
            }
            ("mère", "mer") => {
                // "la mère Méditerranée" should be "la mer Méditerranée"
                if let Some(next) = &next_word {
                    if matches!(next.as_str(), "méditerranée" | "rouge" | "noire" | "baltique" | "du") {
                        return true;
                    }
                }
                false
            }

            // cou/coup
            ("cou", "coup") => {
                // "un cou de pied" should be "un coup de pied"
                if let Some(next) = &next_word {
                    if next == "de" {
                        return true;
                    }
                }
                if let Some(prev) = &prev_word {
                    if matches!(prev.as_str(), "un" | "le" | "ce" | "du" | "au") {
                        if let Some(next) = &next_word {
                            if matches!(next.as_str(), "de" | "d'" | "dur" | "bas" | "franc") {
                                return true;
                            }
                        }
                    }
                }
                false
            }
            ("coup", "cou") => {
                // "le coup de la girafe" should be "le cou de la girafe"
                if let Some(prev) = &prev_word {
                    if matches!(prev.as_str(), "le" | "son" | "mon" | "ton" | "un" | "du" | "au" | "long") {
                        if let Some(next) = &next_word {
                            if matches!(next.as_str(), "de" | "long" | "court" | "tendu" | "tordu") {
                                // Check for body part context
                                return true;
                            }
                        }
                    }
                }
                false
            }

            // cour/cours
            ("cour", "cours") => {
                // "la cour de français" should be "le cours de français"
                if let Some(prev) = &prev_word {
                    if matches!(prev.as_str(), "le" | "un" | "ce" | "du" | "au" | "mon" | "ton" | "son" | "notre" | "votre" | "leur") {
                        if let Some(next) = &next_word {
                            if matches!(next.as_str(), "de" | "d'" | "particulier" | "magistral") {
                                return true;
                            }
                        }
                    }
                }
                false
            }
            ("cours", "cour") => {
                // "la cours de récréation" should be "la cour de récréation"
                if let Some(prev) = &prev_word {
                    if matches!(prev.as_str(), "la" | "une" | "cette" | "sa" | "ma" | "ta") {
                        if let Some(next) = &next_word {
                            if matches!(next.as_str(), "de" | "d'" | "intérieure" | "carrée") {
                                return true;
                            }
                        }
                    }
                }
                false
            }

            // ver/verre/vers/vert
            ("ver", "verre") => {
                // "un ver d'eau" should be "un verre d'eau"
                if let Some(prev) = &prev_word {
                    if matches!(prev.as_str(), "un" | "le" | "ce" | "du" | "mon" | "ton" | "son") {
                        if let Some(next) = &next_word {
                            if matches!(next.as_str(), "de" | "d'" | "à") {
                                return true;
                            }
                        }
                    }
                }
                false
            }
            ("ver", "vers") => {
                // "aller ver Paris" should be "aller vers Paris"
                if let Some(prev) = &prev_word {
                    if matches!(prev.as_str(), "aller" | "va" | "allons" | "allez" | "vont" | "allait" | "tourner" | "regarder" | "marcher" | "courir" | "se") {
                        return true;
                    }
                }
                false
            }
            ("ver", "vert") => {
                // "feu ver" should be "feu vert"
                if let Some(prev) = &prev_word {
                    if matches!(prev.as_str(), "feu" | "haricot" | "haricots" | "citron" | "tapis") {
                        return true;
                    }
                }
                false
            }

            // ai/ait - verb conjugation
            ("ai", "ait") => {
                // "qu'il ai" should be "qu'il ait"
                if let Some(prev) = &prev_word {
                    if matches!(prev.as_str(), "il" | "elle" | "on" | "qui" | "qu'il" | "qu'elle" | "qu'on") {
                        return true;
                    }
                }
                false
            }
            ("ait", "ai") => {
                // "j'ait" should be "j'ai"
                if let Some(prev) = &prev_word {
                    if matches!(prev.as_str(), "j'" | "je") {
                        return true;
                    }
                }
                false
            }

            // For other pairs, be conservative
            _ => false,
        }
    }
}

impl Rule for FrenchConfusionRule {
    fn id(&self) -> &str {
        "FR_CONFUSION"
    }

    fn check(&self, _text: &str, tokens: &[AnalyzedToken]) -> Option<Match> {
        for (idx, token) in tokens.iter().enumerate() {
            if token.token.kind != TokenKind::Word {
                continue;
            }

            let word_lower = token.token.text.to_lowercase();

            if let Some(confusions) = get_fr_confusions(&word_lower) {
                for (suggested, _factor) in confusions {
                    if self.check_context(tokens, idx, &word_lower, suggested) {
                        return Some(Match {
                            span: token.token.span.clone(),
                            message: format!(
                                "Confusion possible : vouliez-vous dire '{}' ?",
                                suggested
                            ),
                            rule_id: self.id().to_string(),
                            suggestions: vec![suggested.to_string()],
                            severity: Severity::Warning,
                        });
                    }
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::CheckResult;
    use crate::tokenizer::SimpleTokenizer;
    use crate::analyzer::PassthroughAnalyzer;
    use crate::core::traits::{Tokenizer, Analyzer, Checker};
    use super::super::RuleChecker;

    fn check_text(text: &str, checker: &RuleChecker) -> CheckResult {
        let tokenizer = SimpleTokenizer::new();
        let analyzer = PassthroughAnalyzer::new();
        let tokens = tokenizer.tokenize(text);
        let analyzed = analyzer.analyze(tokens);
        checker.check(text, &analyzed)
    }

    #[test]
    fn test_lookup_works() {
        // Test that the lookup function actually finds entries
        let confusions = get_en_confusions("affect");
        assert!(confusions.is_some(), "Should find 'affect' in confusion data");
        let confusions = confusions.unwrap();
        assert!(!confusions.is_empty(), "'affect' should have confusion targets");
        println!("affect confusions: {:?}", confusions);
    }

    #[test]
    fn test_english_affect_effect() {
        let checker = RuleChecker::new().with_rule(EnglishConfusionRule);

        // "the affect" should suggest "effect"
        let result = check_text("the affect of the medicine", &checker);
        println!("Result for 'the affect': {:?}", result);
        assert_eq!(result.matches.len(), 1);
        assert_eq!(result.matches[0].suggestions, vec!["effect"]);

        // "to affect" is correct usage
        let result = check_text("to affect the outcome", &checker);
        assert_eq!(result.matches.len(), 0);
    }

    #[test]
    fn test_english_loose_lose() {
        let checker = RuleChecker::new().with_rule(EnglishConfusionRule);

        // "loose weight" should be "lose weight"
        let result = check_text("I want to loose weight", &checker);
        assert_eq!(result.matches.len(), 1);
        assert_eq!(result.matches[0].suggestions, vec!["lose"]);
    }

    #[test]
    fn test_english_lead_led() {
        let checker = RuleChecker::new().with_rule(EnglishConfusionRule);

        // "was lead" should be "was led"
        let result = check_text("the team was lead by him", &checker);
        assert_eq!(result.matches.len(), 1);
        assert_eq!(result.matches[0].suggestions, vec!["led"]);
    }

    #[test]
    fn test_french_notre_notr() {
        let checker = RuleChecker::new().with_rule(FrenchConfusionRule);

        // "le notre" should be "le nôtre"
        let result = check_text("c'est le notre", &checker);
        assert_eq!(result.matches.len(), 1);
        assert_eq!(result.matches[0].suggestions, vec!["nôtre"]);
    }

    #[test]
    fn test_french_moi_mois() {
        let checker = RuleChecker::new().with_rule(FrenchConfusionRule);

        // "le moi dernier" should be "le mois dernier"
        let result = check_text("le moi dernier", &checker);
        assert_eq!(result.matches.len(), 1);
        assert_eq!(result.matches[0].suggestions, vec!["mois"]);

        // "pour mois" should be "pour moi"
        let result = check_text("c'est pour mois", &checker);
        assert_eq!(result.matches.len(), 1);
        assert_eq!(result.matches[0].suggestions, vec!["moi"]);
    }

    #[test]
    fn test_french_vin_vain_vingt() {
        let checker = RuleChecker::new().with_rule(FrenchConfusionRule);

        // "en vin" should be "en vain"
        let result = check_text("c'est en vin", &checker);
        assert_eq!(result.matches.len(), 1);
        assert_eq!(result.matches[0].suggestions, vec!["vain"]);

        // "vin euros" should be "vingt euros"
        let result = check_text("ça coûte vin euros", &checker);
        assert_eq!(result.matches.len(), 1);
        assert_eq!(result.matches[0].suggestions, vec!["vingt"]);
    }

    #[test]
    fn test_french_verre_vers_vert() {
        let checker = RuleChecker::new().with_rule(FrenchConfusionRule);

        // "le feu verre" should be "le feu vert"
        let result = check_text("le feu verre", &checker);
        assert_eq!(result.matches.len(), 1);
        assert_eq!(result.matches[0].suggestions, vec!["vert"]);
    }

    #[test]
    fn test_french_prix_pris() {
        let checker = RuleChecker::new().with_rule(FrenchConfusionRule);

        // Note: Only "prix" -> "pris" exists in the data (not reverse)
        // Test that "j'ai prix" suggests "pris" (auxiliary + participle pattern)
        let result = check_text("nous avons prix", &checker);
        assert_eq!(result.matches.len(), 1, "Should detect 'avons prix' -> 'avons pris'");
        assert_eq!(result.matches[0].suggestions, vec!["pris"]);
    }

    #[test]
    fn test_french_toi_toit() {
        let checker = RuleChecker::new().with_rule(FrenchConfusionRule);

        // Note: Only "toi" -> "toit" exists in data (not reverse)
        // "le toi" should suggest "le toit"
        let result = check_text("le toi de la maison", &checker);
        assert_eq!(result.matches.len(), 1, "Should detect 'le toi' -> 'le toit'");
        assert_eq!(result.matches[0].suggestions, vec!["toit"]);
    }

    #[test]
    fn test_french_est_et() {
        let checker = RuleChecker::new().with_rule(FrenchConfusionRule);

        // "il et parti" should be "il est parti"
        let result = check_text("il et parti", &checker);
        assert_eq!(result.matches.len(), 1, "Should detect 'il et' -> 'il est'");
        assert_eq!(result.matches[0].suggestions, vec!["est"]);
    }

    #[test]
    fn test_french_on_ont() {
        let checker = RuleChecker::new().with_rule(FrenchConfusionRule);

        // "ils on mangé" should be "ils ont mangé"
        let result = check_text("ils on mangé", &checker);
        assert_eq!(result.matches.len(), 1, "Should detect 'ils on' -> 'ils ont'");
        assert_eq!(result.matches[0].suggestions, vec!["ont"]);
    }

    #[test]
    fn test_french_ce_se() {
        let checker = RuleChecker::new().with_rule(FrenchConfusionRule);

        // "ce lever" should be "se lever"
        let result = check_text("il faut ce lever tôt", &checker);
        assert_eq!(result.matches.len(), 1, "Should detect 'ce lever' -> 'se lever'");
        assert_eq!(result.matches[0].suggestions, vec!["se"]);
    }

    #[test]
    fn test_french_mer_mere() {
        let checker = RuleChecker::new().with_rule(FrenchConfusionRule);

        // "ma mer" should be "ma mère"
        let result = check_text("ma mer est gentille", &checker);
        assert_eq!(result.matches.len(), 1, "Should detect 'ma mer' -> 'ma mère'");
        assert_eq!(result.matches[0].suggestions, vec!["mère"]);
    }

    #[test]
    fn test_french_fois_foi() {
        let checker = RuleChecker::new().with_rule(FrenchConfusionRule);

        // "une foi" should be "une fois"
        let result = check_text("une foi de plus", &checker);
        assert_eq!(result.matches.len(), 1, "Should detect 'une foi' -> 'une fois'");
        assert_eq!(result.matches[0].suggestions, vec!["fois"]);
    }

    #[test]
    fn test_french_cou_coup() {
        let checker = RuleChecker::new().with_rule(FrenchConfusionRule);

        // "un cou de pied" should be "un coup de pied"
        let result = check_text("un cou de pied", &checker);
        assert_eq!(result.matches.len(), 1, "Should detect 'cou de' -> 'coup de'");
        assert_eq!(result.matches[0].suggestions, vec!["coup"]);
    }
}
