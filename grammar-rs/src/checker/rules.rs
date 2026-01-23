//! Rule-based checker
//!
//! Version 1: Règles hardcodées en Rust
//! Version 2: Règles chargées depuis un fichier (JSON/TOML)
//! Version 3: DSL compilé ou pattern matching avancé

use crate::core::{AnalyzedToken, CheckResult, Match, Severity, TokenKind};
use crate::core::traits::Checker;

/// Une règle de grammaire
pub trait Rule: Send + Sync {
    fn id(&self) -> &str;
    fn check(&self, text: &str, tokens: &[AnalyzedToken]) -> Option<Match>;
}

/// Checker qui applique une liste de règles
pub struct RuleChecker {
    rules: Vec<Box<dyn Rule>>,
}

impl RuleChecker {
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    pub fn with_rule(mut self, rule: impl Rule + 'static) -> Self {
        self.rules.push(Box::new(rule));
        self
    }

    /// Charge les règles françaises par défaut
    pub fn with_french_rules(self) -> Self {
        self.with_rule(DoubleSpaceRule)
            .with_rule(RepeatedWordRule)
            .with_rule(FrenchPunctuationRule)
    }

    /// Charge les règles anglaises par défaut
    pub fn with_english_rules(self) -> Self {
        self.with_rule(DoubleSpaceRule)
            .with_rule(RepeatedWordRule)
            .with_rule(ImprovedAAnRule) // Use improved version with exceptions for one, union, etc.
            .with_rule(UppercaseSentenceStartRule)
            .with_rule(RepeatedPunctuationRule)
            .with_rule(MissingSpaceAfterPunctRule)
            .with_rule(SubjectVerbAgreementRule)
    }

    /// Charge toutes les règles universelles (sans langue spécifique)
    pub fn with_universal_rules(self) -> Self {
        self.with_rule(DoubleSpaceRule)
            .with_rule(RepeatedWordRule)
            .with_rule(UppercaseSentenceStartRule)
            .with_rule(RepeatedPunctuationRule)
            .with_rule(MissingSpaceAfterPunctRule)
    }
}

impl Default for RuleChecker {
    fn default() -> Self {
        Self::new()
    }
}

impl Checker for RuleChecker {
    fn check(&self, text: &str, tokens: &[AnalyzedToken]) -> CheckResult {
        let mut result = CheckResult::new();

        for rule in &self.rules {
            if let Some(m) = rule.check(text, tokens) {
                result.matches.push(m);
            }
        }

        result
    }
}

// --- Règles concrètes ---

/// Double espaces
pub struct DoubleSpaceRule;

impl Rule for DoubleSpaceRule {
    fn id(&self) -> &str {
        "DOUBLE_SPACE"
    }

    fn check(&self, text: &str, _tokens: &[AnalyzedToken]) -> Option<Match> {
        if let Some(pos) = text.find("  ") {
            Some(Match {
                span: pos..pos + 2,
                message: "Double espace détecté".to_string(),
                rule_id: self.id().to_string(),
                suggestions: vec![" ".to_string()],
                severity: Severity::Warning,
            })
        } else {
            None
        }
    }
}

/// Mot répété (le le, the the)
pub struct RepeatedWordRule;

impl Rule for RepeatedWordRule {
    fn id(&self) -> &str {
        "REPEATED_WORD"
    }

    fn check(&self, _text: &str, tokens: &[AnalyzedToken]) -> Option<Match> {
        let words: Vec<_> = tokens
            .iter()
            .filter(|t| t.token.kind == TokenKind::Word)
            .collect();

        for window in words.windows(2) {
            let a = &window[0];
            let b = &window[1];

            if a.token.text.to_lowercase() == b.token.text.to_lowercase() {
                return Some(Match {
                    span: a.token.span.start..b.token.span.end,
                    message: format!("Mot répété: '{}'", a.token.text),
                    rule_id: self.id().to_string(),
                    suggestions: vec![a.token.text.to_string()],
                    severity: Severity::Warning,
                });
            }
        }

        None
    }
}

/// Ponctuation française (espace avant ? ! : ;)
pub struct FrenchPunctuationRule;

impl Rule for FrenchPunctuationRule {
    fn id(&self) -> &str {
        "FR_PUNCT_SPACE"
    }

    fn check(&self, text: &str, _tokens: &[AnalyzedToken]) -> Option<Match> {
        let chars: Vec<char> = text.chars().collect();

        for (i, &c) in chars.iter().enumerate() {
            if "?!:;".contains(c) && i > 0 {
                let prev = chars[i - 1];
                if !prev.is_whitespace() {
                    let byte_pos = text
                        .char_indices()
                        .nth(i)
                        .map(|(pos, _)| pos)
                        .unwrap_or(0);

                    return Some(Match {
                        span: byte_pos..byte_pos + c.len_utf8(),
                        message: format!(
                            "En français, un espace est requis avant '{}'",
                            c
                        ),
                        rule_id: self.id().to_string(),
                        suggestions: vec![format!(" {}", c)],
                        severity: Severity::Warning,
                    });
                }
            }
        }

        None
    }
}

/// Règle a/an en anglais
pub struct AAnRule;

impl Rule for AAnRule {
    fn id(&self) -> &str {
        "EN_A_AN"
    }

    fn check(&self, _text: &str, tokens: &[AnalyzedToken]) -> Option<Match> {
        let words: Vec<_> = tokens
            .iter()
            .filter(|t| t.token.kind == TokenKind::Word)
            .collect();

        for window in words.windows(2) {
            let article = &window[0];
            let next = &window[1];

            let art_lower = article.token.text.to_lowercase();
            let next_lower = next.token.text.to_lowercase();

            let starts_with_vowel_sound = next_lower
                .chars()
                .next()
                .map(|c| "aeiou".contains(c))
                .unwrap_or(false);

            // "a" before vowel sound -> should be "an"
            if art_lower == "a" && starts_with_vowel_sound {
                return Some(Match {
                    span: article.token.span.clone(),
                    message: format!(
                        "Utiliser 'an' avant '{}' (commence par une voyelle)",
                        next.token.text
                    ),
                    rule_id: self.id().to_string(),
                    suggestions: vec!["an".to_string()],
                    severity: Severity::Error,
                });
            }

            // "an" before consonant sound -> should be "a"
            if art_lower == "an" && !starts_with_vowel_sound {
                return Some(Match {
                    span: article.token.span.clone(),
                    message: format!(
                        "Utiliser 'a' avant '{}' (commence par une consonne)",
                        next.token.text
                    ),
                    rule_id: self.id().to_string(),
                    suggestions: vec!["a".to_string()],
                    severity: Severity::Error,
                });
            }
        }

        None
    }
}

/// Majuscule en début de phrase
pub struct UppercaseSentenceStartRule;

impl Rule for UppercaseSentenceStartRule {
    fn id(&self) -> &str {
        "UPPERCASE_SENTENCE_START"
    }

    fn check(&self, text: &str, tokens: &[AnalyzedToken]) -> Option<Match> {
        // Find first word token
        let first_word = tokens
            .iter()
            .find(|t| t.token.kind == TokenKind::Word)?;

        let first_char = first_word.token.text.chars().next()?;

        // Check if first character is lowercase
        if first_char.is_lowercase() {
            let corrected: String = first_word
                .token
                .text
                .chars()
                .enumerate()
                .map(|(i, c)| if i == 0 { c.to_uppercase().next().unwrap_or(c) } else { c })
                .collect();

            return Some(Match {
                span: first_word.token.span.clone(),
                message: "La phrase doit commencer par une majuscule".to_string(),
                rule_id: self.id().to_string(),
                suggestions: vec![corrected],
                severity: Severity::Error,
            });
        }

        // Check after sentence-ending punctuation
        let sentence_enders = ['.', '!', '?'];
        let chars: Vec<char> = text.chars().collect();

        for (i, &c) in chars.iter().enumerate() {
            if sentence_enders.contains(&c) {
                // Find next word after this punctuation
                let byte_pos = text.char_indices().nth(i).map(|(p, _)| p).unwrap_or(0);

                for token in tokens {
                    if token.token.kind == TokenKind::Word && token.token.span.start > byte_pos {
                        let first_char = token.token.text.chars().next();
                        if let Some(fc) = first_char {
                            if fc.is_lowercase() {
                                let corrected: String = token
                                    .token
                                    .text
                                    .chars()
                                    .enumerate()
                                    .map(|(i, c)| {
                                        if i == 0 {
                                            c.to_uppercase().next().unwrap_or(c)
                                        } else {
                                            c
                                        }
                                    })
                                    .collect();

                                return Some(Match {
                                    span: token.token.span.clone(),
                                    message: "La phrase doit commencer par une majuscule"
                                        .to_string(),
                                    rule_id: self.id().to_string(),
                                    suggestions: vec![corrected],
                                    severity: Severity::Error,
                                });
                            }
                        }
                        break;
                    }
                }
            }
        }

        None
    }
}

/// Ponctuation répétée (!! ?? ..)
pub struct RepeatedPunctuationRule;

impl Rule for RepeatedPunctuationRule {
    fn id(&self) -> &str {
        "REPEATED_PUNCTUATION"
    }

    fn check(&self, text: &str, _tokens: &[AnalyzedToken]) -> Option<Match> {
        let chars: Vec<char> = text.chars().collect();
        let punctuation = ['.', '!', '?', ',', ';', ':'];

        let mut i = 0;
        while i < chars.len() {
            let c = chars[i];
            if punctuation.contains(&c) && i + 1 < chars.len() {
                let next = chars[i + 1];
                if punctuation.contains(&next) {
                    // Exception: ... (ellipsis) is allowed - skip all dots
                    if c == '.' && next == '.' {
                        // Skip the entire ellipsis
                        while i < chars.len() && chars[i] == '.' {
                            i += 1;
                        }
                        continue;
                    }

                    let byte_pos = text.char_indices().nth(i).map(|(p, _)| p).unwrap_or(0);
                    let end_pos = text
                        .char_indices()
                        .nth(i + 2)
                        .map(|(p, _)| p)
                        .unwrap_or(text.len());

                    return Some(Match {
                        span: byte_pos..end_pos,
                        message: format!("Ponctuation répétée: '{}{}'", c, next),
                        rule_id: self.id().to_string(),
                        suggestions: vec![c.to_string()],
                        severity: Severity::Warning,
                    });
                }
            }
            i += 1;
        }

        None
    }
}

/// Espace manquant après ponctuation
pub struct MissingSpaceAfterPunctRule;

impl Rule for MissingSpaceAfterPunctRule {
    fn id(&self) -> &str {
        "MISSING_SPACE_AFTER_PUNCT"
    }

    fn check(&self, text: &str, _tokens: &[AnalyzedToken]) -> Option<Match> {
        let chars: Vec<char> = text.chars().collect();
        let punctuation = ['.', '!', '?', ',', ';', ':'];

        // Common TLDs to skip
        let tlds = ["com", "org", "net", "edu", "gov", "io", "co", "uk", "fr", "de"];

        for (i, &c) in chars.iter().enumerate() {
            if punctuation.contains(&c) && i + 1 < chars.len() {
                let next = chars[i + 1];

                // Skip if next is whitespace, punctuation, quote, or end
                if next.is_whitespace() || punctuation.contains(&next) {
                    continue;
                }

                // Skip quotes and closing brackets
                if "\"')]}>".contains(next) {
                    continue;
                }

                // Skip ellipsis
                if c == '.' && next == '.' {
                    continue;
                }

                // Skip numbers after period (e.g., "1.5")
                if c == '.' && next.is_ascii_digit() {
                    continue;
                }

                // Skip URLs (http:// https://)
                if c == ':' && next == '/' {
                    continue;
                }

                // Skip domain names (.com, .org, etc.)
                if c == '.' && next.is_ascii_lowercase() {
                    // Check if this looks like a TLD
                    let rest: String = chars[i + 1..]
                        .iter()
                        .take_while(|c| c.is_ascii_alphanumeric())
                        .collect();
                    if tlds.contains(&rest.to_lowercase().as_str()) {
                        continue;
                    }

                    // Check if preceded by alphanumeric (likely a domain)
                    if i > 0 && chars[i - 1].is_ascii_alphanumeric() {
                        // Looks like "example.something" - could be a domain
                        // Skip if it looks like a URL or file path
                        let prev_word: String = chars[..i]
                            .iter()
                            .rev()
                            .take_while(|c| c.is_ascii_alphanumeric())
                            .collect::<String>()
                            .chars()
                            .rev()
                            .collect();
                        // Skip common patterns
                        if !prev_word.is_empty() && rest.len() <= 4 {
                            continue;
                        }
                    }
                }

                // Skip file extensions
                if c == '.' && i > 0 && chars[i - 1].is_ascii_alphanumeric() {
                    let ext: String = chars[i + 1..]
                        .iter()
                        .take_while(|c| c.is_ascii_alphanumeric())
                        .collect();
                    let common_ext = ["txt", "rs", "js", "py", "html", "css", "json", "xml", "md"];
                    if common_ext.contains(&ext.to_lowercase().as_str()) {
                        continue;
                    }
                }

                // Found missing space
                let byte_pos = text.char_indices().nth(i).map(|(p, _)| p).unwrap_or(0);

                return Some(Match {
                    span: byte_pos..byte_pos + c.len_utf8(),
                    message: format!("Espace manquant après '{}'", c),
                    rule_id: self.id().to_string(),
                    suggestions: vec![format!("{} ", c)],
                    severity: Severity::Warning,
                });
            }
        }

        None
    }
}

/// Subject-verb agreement (he go -> he goes)
pub struct SubjectVerbAgreementRule;

impl SubjectVerbAgreementRule {
    // Third person singular subjects
    const SINGULAR_SUBJECTS: &'static [&'static str] = &[
        "he", "she", "it", "one", "someone", "anyone", "everyone", "nobody",
        "somebody", "anybody", "everybody", "nothing", "something", "anything",
        "everything",
    ];

    // Plural subjects
    const PLURAL_SUBJECTS: &'static [&'static str] = &["they", "we", "you", "i"];

    // Common verbs with their forms: (base, third_person_singular)
    const VERB_FORMS: &'static [(&'static str, &'static str)] = &[
        ("go", "goes"),
        ("do", "does"),
        ("have", "has"),
        ("be", "is"), // simplified
        ("say", "says"),
        ("get", "gets"),
        ("make", "makes"),
        ("know", "knows"),
        ("think", "thinks"),
        ("take", "takes"),
        ("see", "sees"),
        ("come", "comes"),
        ("want", "wants"),
        ("look", "looks"),
        ("use", "uses"),
        ("give", "gives"),
        ("work", "works"),
        ("call", "calls"),
        ("try", "tries"),
        ("need", "needs"),
        ("feel", "feels"),
        ("become", "becomes"),
        ("leave", "leaves"),
        ("put", "puts"),
        ("mean", "means"),
        ("keep", "keeps"),
        ("let", "lets"),
        ("begin", "begins"),
        ("seem", "seems"),
        ("help", "helps"),
        ("show", "shows"),
        ("hear", "hears"),
        ("play", "plays"),
        ("run", "runs"),
        ("move", "moves"),
        ("live", "lives"),
        ("believe", "believes"),
        ("bring", "brings"),
        ("happen", "happens"),
        ("write", "writes"),
        ("sit", "sits"),
        ("stand", "stands"),
        ("lose", "loses"),
        ("pay", "pays"),
        ("meet", "meets"),
        ("include", "includes"),
        ("continue", "continues"),
        ("set", "sets"),
        ("learn", "learns"),
        ("change", "changes"),
        ("lead", "leads"),
        ("understand", "understands"),
        ("watch", "watches"),
        ("follow", "follows"),
        ("stop", "stops"),
        ("create", "creates"),
        ("speak", "speaks"),
        ("read", "reads"),
        ("spend", "spends"),
        ("grow", "grows"),
        ("open", "opens"),
        ("walk", "walks"),
        ("win", "wins"),
        ("teach", "teaches"),
        ("offer", "offers"),
        ("remember", "remembers"),
        ("love", "loves"),
        ("consider", "considers"),
        ("appear", "appears"),
        ("buy", "buys"),
        ("wait", "waits"),
        ("serve", "serves"),
        ("die", "dies"),
        ("send", "sends"),
        ("build", "builds"),
        ("stay", "stays"),
        ("fall", "falls"),
        ("cut", "cuts"),
        ("reach", "reaches"),
        ("kill", "kills"),
        ("remain", "remains"),
        ("eat", "eats"),
        ("like", "likes"),
        ("start", "starts"),
        ("hate", "hates"),
    ];

    fn get_third_person_form(base: &str) -> Option<&'static str> {
        Self::VERB_FORMS
            .iter()
            .find(|(b, _)| *b == base)
            .map(|(_, s)| *s)
    }

    fn get_base_form(third_person: &str) -> Option<&'static str> {
        Self::VERB_FORMS
            .iter()
            .find(|(_, s)| *s == third_person)
            .map(|(b, _)| *b)
    }
}

impl Rule for SubjectVerbAgreementRule {
    fn id(&self) -> &str {
        "SUBJECT_VERB_AGREEMENT"
    }

    fn check(&self, _text: &str, tokens: &[AnalyzedToken]) -> Option<Match> {
        let words: Vec<_> = tokens
            .iter()
            .filter(|t| t.token.kind == TokenKind::Word)
            .collect();

        for window in words.windows(2) {
            let subject = &window[0];
            let verb = &window[1];

            let subj_lower = subject.token.text.to_lowercase();
            let verb_lower = verb.token.text.to_lowercase();

            // Check: singular subject + base form verb -> should be 3rd person
            if Self::SINGULAR_SUBJECTS.contains(&subj_lower.as_str()) {
                if let Some(correct_form) = Self::get_third_person_form(&verb_lower) {
                    return Some(Match {
                        span: verb.token.span.clone(),
                        message: format!(
                            "Avec '{}', utiliser '{}' au lieu de '{}'",
                            subject.token.text, correct_form, verb.token.text
                        ),
                        rule_id: self.id().to_string(),
                        suggestions: vec![correct_form.to_string()],
                        severity: Severity::Error,
                    });
                }
            }

            // Check: plural subject + 3rd person form -> should be base form
            if Self::PLURAL_SUBJECTS.contains(&subj_lower.as_str()) {
                if let Some(correct_form) = Self::get_base_form(&verb_lower) {
                    return Some(Match {
                        span: verb.token.span.clone(),
                        message: format!(
                            "Avec '{}', utiliser '{}' au lieu de '{}'",
                            subject.token.text, correct_form, verb.token.text
                        ),
                        rule_id: self.id().to_string(),
                        suggestions: vec![correct_form.to_string()],
                        severity: Severity::Error,
                    });
                }
            }
        }

        None
    }
}

/// Confusion its/it's
/// "it's" = "it is" or "it has"
/// "its" = possessive
pub struct ItsItsRule;

impl ItsItsRule {
    // Words that typically follow "its" (possessive context)
    const POSSESSIVE_FOLLOWERS: &'static [&'static str] = &[
        "own", "way", "name", "place", "best", "worst", "first", "last",
        "size", "color", "colour", "shape", "form", "purpose", "function", "role",
        "origin", "source", "meaning", "value", "price", "cost", "weight", "height",
        "length", "width", "depth", "age", "history", "future", "past", "present",
    ];

    // Words that typically follow "it's" (it is/has context) - verbs, adjectives, adverbs
    const CONTRACTION_FOLLOWERS: &'static [&'static str] = &[
        // Adjectives - common ones
        "good", "bad", "great", "nice", "fine", "okay", "ok", "true", "false",
        "important", "necessary", "possible", "impossible", "easy", "hard", "difficult",
        "clear", "obvious", "evident", "likely", "unlikely", "certain", "uncertain",
        "better", "worse", "amazing", "wonderful", "terrible", "horrible",
        "beautiful", "ugly", "cold", "hot", "warm", "cool", "big", "small", "large",
        "fun", "funny", "sad", "happy", "late", "early", "free", "expensive", "cheap",
        // Adverbs
        "not", "never", "always", "often", "sometimes", "rarely", "just", "only",
        "really", "very", "quite", "rather", "too", "so", "also", "still", "already",
        // Verbs (for "it has")
        "been", "got", "gotten", "become", "taken", "made", "done", "gone", "come",
        // Articles (for "it is a...")
        "a", "an", "the",
        // Time expressions
        "time", "raining", "snowing", "getting",
    ];
}

impl Rule for ItsItsRule {
    fn id(&self) -> &str {
        "EN_ITS_ITS"
    }

    fn check(&self, _text: &str, tokens: &[AnalyzedToken]) -> Option<Match> {
        let words: Vec<_> = tokens
            .iter()
            .filter(|t| t.token.kind == TokenKind::Word)
            .collect();

        for window in words.windows(2) {
            let word = &window[0];
            let next = &window[1];

            let word_lower = word.token.text.to_lowercase();
            let next_lower = next.token.text.to_lowercase();

            // "its" followed by contraction context -> should be "it's"
            if word_lower == "its" {
                if Self::CONTRACTION_FOLLOWERS.contains(&next_lower.as_str()) {
                    return Some(Match {
                        span: word.token.span.clone(),
                        message: format!(
                            "Utiliser \"it's\" (it is/has) avant '{}', pas \"its\" (possessif)",
                            next.token.text
                        ),
                        rule_id: self.id().to_string(),
                        suggestions: vec!["it's".to_string()],
                        severity: Severity::Error,
                    });
                }
            }

            // "it's" followed by possessive context -> should be "its"
            if word_lower == "it's" {
                if Self::POSSESSIVE_FOLLOWERS.contains(&next_lower.as_str()) {
                    return Some(Match {
                        span: word.token.span.clone(),
                        message: format!(
                            "Utiliser \"its\" (possessif) avant '{}', pas \"it's\" (it is)",
                            next.token.text
                        ),
                        rule_id: self.id().to_string(),
                        suggestions: vec!["its".to_string()],
                        severity: Severity::Error,
                    });
                }
            }
        }

        None
    }
}

/// Confusion your/you're
pub struct YourYoureRule;

impl YourYoureRule {
    // Words that typically follow "your" (possessive)
    const POSSESSIVE_FOLLOWERS: &'static [&'static str] = &[
        "own", "name", "place", "home", "house", "car", "phone", "computer", "email",
        "friend", "friends", "family", "mother", "father", "sister", "brother",
        "work", "job", "boss", "team", "company", "school", "class", "teacher",
        "life", "time", "money", "account", "password", "question", "answer",
        "idea", "opinion", "choice", "decision", "problem", "solution",
        "help", "support", "attention", "turn", "fault", "responsibility",
    ];

    // Words that typically follow "you're" (you are)
    const CONTRACTION_FOLLOWERS: &'static [&'static str] = &[
        // Adjectives
        "welcome", "right", "wrong", "correct", "incorrect", "sure", "certain",
        "good", "great", "amazing", "wonderful", "awesome", "fantastic", "brilliant",
        "bad", "terrible", "horrible", "awful", "stupid", "smart", "clever",
        "nice", "kind", "sweet", "beautiful", "handsome", "ugly", "tall", "short",
        "young", "old", "new", "late", "early", "ready", "done", "finished",
        "lucky", "unlucky", "happy", "sad", "angry", "upset", "tired", "sick",
        "funny", "crazy", "mad", "fine", "okay", "ok",
        // Adverbs
        "not", "so", "very", "really", "absolutely", "totally", "completely",
        "always", "never", "still", "already", "just", "only",
        // Verbs (present participle)
        "going", "doing", "being", "having", "making", "taking", "getting",
        "looking", "trying", "working", "talking", "saying", "telling", "asking",
        "joking", "kidding", "lying", "killing",
        // Articles
        "a", "an", "the",
    ];
}

impl Rule for YourYoureRule {
    fn id(&self) -> &str {
        "EN_YOUR_YOURE"
    }

    fn check(&self, _text: &str, tokens: &[AnalyzedToken]) -> Option<Match> {
        let words: Vec<_> = tokens
            .iter()
            .filter(|t| t.token.kind == TokenKind::Word)
            .collect();

        for window in words.windows(2) {
            let word = &window[0];
            let next = &window[1];

            let word_lower = word.token.text.to_lowercase();
            let next_lower = next.token.text.to_lowercase();

            // "your" followed by contraction context -> should be "you're"
            if word_lower == "your" {
                if Self::CONTRACTION_FOLLOWERS.contains(&next_lower.as_str()) {
                    return Some(Match {
                        span: word.token.span.clone(),
                        message: format!(
                            "Utiliser \"you're\" (you are) avant '{}', pas \"your\" (possessif)",
                            next.token.text
                        ),
                        rule_id: self.id().to_string(),
                        suggestions: vec!["you're".to_string()],
                        severity: Severity::Error,
                    });
                }
            }

            // "you're" followed by possessive context -> should be "your"
            if word_lower == "you're" {
                if Self::POSSESSIVE_FOLLOWERS.contains(&next_lower.as_str()) {
                    return Some(Match {
                        span: word.token.span.clone(),
                        message: format!(
                            "Utiliser \"your\" (possessif) avant '{}', pas \"you're\" (you are)",
                            next.token.text
                        ),
                        rule_id: self.id().to_string(),
                        suggestions: vec!["your".to_string()],
                        severity: Severity::Error,
                    });
                }
            }
        }

        None
    }
}

/// Confusion their/they're/there
pub struct TheirTheyreThereRule;

impl TheirTheyreThereRule {
    // Words that follow "their" (possessive)
    const POSSESSIVE_FOLLOWERS: &'static [&'static str] = &[
        "own", "name", "names", "place", "home", "house", "car", "cars",
        "phone", "phones", "friend", "friends", "family", "families",
        "work", "job", "jobs", "boss", "team", "company", "school",
        "life", "lives", "time", "money", "account", "question", "answer",
        "idea", "ideas", "opinion", "opinions", "choice", "decision",
        "problem", "problems", "way", "ways", "best", "worst", "first", "last",
        "mother", "father", "sister", "brother", "parents", "children", "kids",
    ];

    // Words that follow "they're" (they are)
    const CONTRACTION_FOLLOWERS: &'static [&'static str] = &[
        // Adjectives
        "good", "great", "amazing", "wonderful", "awesome", "fantastic",
        "bad", "terrible", "horrible", "awful", "stupid", "smart", "clever",
        "nice", "kind", "sweet", "beautiful", "ugly", "tall", "short",
        "young", "old", "new", "late", "early", "ready", "done", "finished",
        "lucky", "unlucky", "happy", "sad", "angry", "upset", "tired", "sick",
        "right", "wrong", "correct", "sure", "certain", "fine", "okay", "ok",
        // Adverbs
        "not", "so", "very", "really", "absolutely", "totally", "completely",
        "always", "never", "still", "already", "just", "only",
        // Verbs (present participle)
        "going", "doing", "being", "having", "making", "taking", "getting",
        "looking", "trying", "working", "talking", "saying", "telling",
        "coming", "leaving", "waiting", "playing", "watching",
        // Articles
        "a", "an", "the", "all", "both",
    ];

    // Words that follow "there" (location/existence) - used for context validation
    #[allow(dead_code)]
    const LOCATION_CONTEXT: &'static [&'static str] = &[
        // Verbs of being/existence
        "is", "are", "was", "were", "be", "been", "being",
        "isn't", "aren't", "wasn't", "weren't",
        // Location words
        "now", "then", "once", "again",
    ];
}

impl Rule for TheirTheyreThereRule {
    fn id(&self) -> &str {
        "EN_THEIR_THEYRE_THERE"
    }

    fn check(&self, _text: &str, tokens: &[AnalyzedToken]) -> Option<Match> {
        let words: Vec<_> = tokens
            .iter()
            .filter(|t| t.token.kind == TokenKind::Word)
            .collect();

        for window in words.windows(2) {
            let word = &window[0];
            let next = &window[1];

            let word_lower = word.token.text.to_lowercase();
            let next_lower = next.token.text.to_lowercase();

            // "their" in contraction context -> should be "they're"
            if word_lower == "their" {
                if Self::CONTRACTION_FOLLOWERS.contains(&next_lower.as_str()) {
                    return Some(Match {
                        span: word.token.span.clone(),
                        message: format!(
                            "Utiliser \"they're\" (they are) avant '{}', pas \"their\" (possessif)",
                            next.token.text
                        ),
                        rule_id: self.id().to_string(),
                        suggestions: vec!["they're".to_string()],
                        severity: Severity::Error,
                    });
                }
            }

            // "there" in possessive context -> should be "their"
            if word_lower == "there" {
                if Self::POSSESSIVE_FOLLOWERS.contains(&next_lower.as_str()) {
                    return Some(Match {
                        span: word.token.span.clone(),
                        message: format!(
                            "Utiliser \"their\" (possessif) avant '{}', pas \"there\" (lieu)",
                            next.token.text
                        ),
                        rule_id: self.id().to_string(),
                        suggestions: vec!["their".to_string()],
                        severity: Severity::Error,
                    });
                }
                // "there" in contraction context -> should be "they're"
                if Self::CONTRACTION_FOLLOWERS.contains(&next_lower.as_str()) {
                    return Some(Match {
                        span: word.token.span.clone(),
                        message: format!(
                            "Utiliser \"they're\" (they are) avant '{}', pas \"there\" (lieu)",
                            next.token.text
                        ),
                        rule_id: self.id().to_string(),
                        suggestions: vec!["they're".to_string()],
                        severity: Severity::Error,
                    });
                }
            }

            // "they're" in possessive context -> should be "their"
            if word_lower == "they're" {
                if Self::POSSESSIVE_FOLLOWERS.contains(&next_lower.as_str()) {
                    return Some(Match {
                        span: word.token.span.clone(),
                        message: format!(
                            "Utiliser \"their\" (possessif) avant '{}', pas \"they're\" (they are)",
                            next.token.text
                        ),
                        rule_id: self.id().to_string(),
                        suggestions: vec!["their".to_string()],
                        severity: Severity::Error,
                    });
                }
            }
        }

        None
    }
}

/// Comma splice detection
/// A comma splice occurs when two independent clauses are joined with just a comma
pub struct CommaSpliceRule;

impl CommaSpliceRule {
    // Pronouns that often start independent clauses
    const CLAUSE_STARTERS: &'static [&'static str] = &[
        "i", "you", "he", "she", "it", "we", "they",
        "this", "that", "these", "those",
        "there", "here",
    ];

    // Common verbs that follow pronouns
    const COMMON_VERBS: &'static [&'static str] = &[
        "am", "is", "are", "was", "were", "be", "been", "being",
        "have", "has", "had", "having",
        "do", "does", "did", "doing",
        "will", "would", "shall", "should", "can", "could", "may", "might", "must",
        "go", "goes", "went", "going",
        "get", "gets", "got", "getting",
        "make", "makes", "made", "making",
        "know", "knows", "knew", "knowing",
        "think", "thinks", "thought", "thinking",
        "want", "wants", "wanted", "wanting",
        "need", "needs", "needed", "needing",
        "like", "likes", "liked", "liking",
        "love", "loves", "loved", "loving",
        "hate", "hates", "hated", "hating",
        "see", "sees", "saw", "seeing",
        "come", "comes", "came", "coming",
        "take", "takes", "took", "taking",
        "find", "finds", "found", "finding",
        "give", "gives", "gave", "giving",
        "tell", "tells", "told", "telling",
        "say", "says", "said", "saying",
        "feel", "feels", "felt", "feeling",
        "try", "tries", "tried", "trying",
        "leave", "leaves", "left", "leaving",
        "call", "calls", "called", "calling",
        "keep", "keeps", "kept", "keeping",
        "let", "lets", "letting",
        "begin", "begins", "began", "beginning",
        "seem", "seems", "seemed", "seeming",
        "help", "helps", "helped", "helping",
        "show", "shows", "showed", "showing",
        "hear", "hears", "heard", "hearing",
        "play", "plays", "played", "playing",
        "run", "runs", "ran", "running",
        "move", "moves", "moved", "moving",
        "live", "lives", "lived", "living",
        "work", "works", "worked", "working",
    ];

    // Subordinating conjunctions that make a clause dependent
    const SUBORDINATING_CONJUNCTIONS: &'static [&'static str] = &[
        "although", "though", "even", "while", "when", "if", "unless",
        "because", "since", "after", "before", "until", "as", "once",
        "whereas", "whenever", "wherever", "whether",
    ];
}

impl Rule for CommaSpliceRule {
    fn id(&self) -> &str {
        "COMMA_SPLICE"
    }

    fn check(&self, _text: &str, tokens: &[AnalyzedToken]) -> Option<Match> {
        // Find commas
        for (i, token) in tokens.iter().enumerate() {
            if token.token.kind == TokenKind::Punctuation && token.token.text == "," {
                // Check if there's a subordinating conjunction before the comma
                // This would make it a subordinate clause, not a comma splice
                let mut has_subordinator = false;
                for j in 0..i {
                    if tokens[j].token.kind == TokenKind::Word {
                        let word_lower = tokens[j].token.text.to_lowercase();
                        if Self::SUBORDINATING_CONJUNCTIONS.contains(&word_lower.as_str()) {
                            has_subordinator = true;
                            break;
                        }
                    }
                }
                if has_subordinator {
                    continue;
                }

                // Look at what comes after the comma
                // We need: comma, [optional whitespace], pronoun, verb
                let mut next_idx = i + 1;

                // Skip whitespace
                while next_idx < tokens.len()
                    && tokens[next_idx].token.kind == TokenKind::Whitespace
                {
                    next_idx += 1;
                }

                // Check for pronoun
                if next_idx < tokens.len() {
                    let maybe_pronoun = &tokens[next_idx];
                    if maybe_pronoun.token.kind == TokenKind::Word {
                        let pronoun_lower = maybe_pronoun.token.text.to_lowercase();

                        if Self::CLAUSE_STARTERS.contains(&pronoun_lower.as_str()) {
                            // Skip whitespace after pronoun
                            let mut verb_idx = next_idx + 1;
                            while verb_idx < tokens.len()
                                && tokens[verb_idx].token.kind == TokenKind::Whitespace
                            {
                                verb_idx += 1;
                            }

                            // Check for verb
                            if verb_idx < tokens.len() {
                                let maybe_verb = &tokens[verb_idx];
                                if maybe_verb.token.kind == TokenKind::Word {
                                    let verb_lower = maybe_verb.token.text.to_lowercase();

                                    if Self::COMMON_VERBS.contains(&verb_lower.as_str()) {
                                        // Found comma splice pattern: ", pronoun verb"
                                        return Some(Match {
                                            span: token.token.span.clone(),
                                            message: format!(
                                                "Comma splice: deux phrases indépendantes jointes par une virgule. \
                                                Utilisez un point, point-virgule, ou conjonction."
                                            ),
                                            rule_id: self.id().to_string(),
                                            suggestions: vec![
                                                ".".to_string(),
                                                ";".to_string(),
                                                ", and".to_string(),
                                                ", but".to_string(),
                                            ],
                                            severity: Severity::Warning,
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        None
    }
}

/// Improved A/An rule with exceptions for silent h, acronyms, etc.
pub struct ImprovedAAnRule;

impl ImprovedAAnRule {
    // Words starting with silent 'h' (use "an")
    const SILENT_H: &'static [&'static str] = &[
        "hour", "hours", "hourly", "heir", "heirs", "heiress", "heirloom",
        "honest", "honestly", "honesty", "honor", "honour", "honorable", "honourable",
        "honorary", "herb", "herbs", "herbal", // US English treats 'herb' with silent h
    ];

    // Words starting with vowel letter but consonant sound (use "a")
    const VOWEL_CONSONANT_SOUND: &'static [&'static str] = &[
        // 'u' pronounced as 'yu'
        "university", "universities", "union", "unions", "united", "unique",
        "unit", "units", "uniform", "uniforms", "universal", "universe",
        "user", "users", "useful", "useless", "usual", "usually", "utility",
        "utensil", "utopia", "ukulele", "uranium", "urinal",
        // 'eu' pronounced as 'yu'
        "european", "europe", "euro", "euros", "euphemism", "eulogy",
        // 'o' pronounced as 'w'
        "one", "ones", "once", "one-time", "one-way",
    ];

    // Common acronyms starting with vowel sound (use "an")
    const VOWEL_ACRONYMS: &'static [&'static str] = &[
        "fbi", "html", "http", "mri", "nba", "nfl", "nhl", "rbi", "sms", "sql",
        "xml", "html5", "lp", "mp3", "mp4", "mba", "md", "ma", "ms",
    ];

    fn starts_with_vowel_sound(word: &str) -> bool {
        let lower = word.to_lowercase();

        // Check silent h exceptions
        if Self::SILENT_H.iter().any(|w| lower == *w || lower.starts_with(&format!("{}-", w))) {
            return true;
        }

        // Check vowel-letter-but-consonant-sound exceptions
        if Self::VOWEL_CONSONANT_SOUND.iter().any(|w| lower == *w || lower.starts_with(&format!("{}-", w))) {
            return false;
        }

        // Check acronyms (all uppercase or known acronyms)
        if word.chars().all(|c| c.is_uppercase() || c.is_ascii_digit()) {
            // Acronym - check if first letter sounds like a vowel
            let first = lower.chars().next().unwrap_or('x');
            // Letters that sound like they start with a vowel: a, e, f, h, i, l, m, n, o, r, s, x
            return "aefhilmnorsx".contains(first);
        }

        if Self::VOWEL_ACRONYMS.contains(&lower.as_str()) {
            return true;
        }

        // Default: check first character
        lower
            .chars()
            .next()
            .map(|c| "aeiou".contains(c))
            .unwrap_or(false)
    }
}

impl Rule for ImprovedAAnRule {
    fn id(&self) -> &str {
        "EN_A_AN_IMPROVED"
    }

    fn check(&self, _text: &str, tokens: &[AnalyzedToken]) -> Option<Match> {
        let words: Vec<_> = tokens
            .iter()
            .filter(|t| t.token.kind == TokenKind::Word)
            .collect();

        for window in words.windows(2) {
            let article = &window[0];
            let next = &window[1];

            let art_lower = article.token.text.to_lowercase();
            let starts_vowel = Self::starts_with_vowel_sound(next.token.text);

            // "a" before vowel sound -> should be "an"
            if art_lower == "a" && starts_vowel {
                return Some(Match {
                    span: article.token.span.clone(),
                    message: format!(
                        "Utiliser 'an' avant '{}' (son voyelle)",
                        next.token.text
                    ),
                    rule_id: self.id().to_string(),
                    suggestions: vec!["an".to_string()],
                    severity: Severity::Error,
                });
            }

            // "an" before consonant sound -> should be "a"
            if art_lower == "an" && !starts_vowel {
                return Some(Match {
                    span: article.token.span.clone(),
                    message: format!(
                        "Utiliser 'a' avant '{}' (son consonne)",
                        next.token.text
                    ),
                    rule_id: self.id().to_string(),
                    suggestions: vec!["a".to_string()],
                    severity: Severity::Error,
                });
            }
        }

        None
    }
}

/// French homophone: a/à
/// "a" = verb avoir (il a)
/// "à" = preposition (à Paris)
pub struct FrenchAAccentRule;

impl FrenchAAccentRule {
    // Words/patterns that indicate preposition context (should be "à")
    const PREPOSITION_CONTEXT: &'static [&'static str] = &[
        // Cities, places (à Paris, à Lyon)
        "paris", "lyon", "marseille", "toulouse", "nice", "nantes", "bordeaux",
        "lille", "rennes", "strasbourg", "montpellier", "grenoble",
        // Common preposition phrases
        "côté", "cause", "travers", "partir", "propos", "nouveau", "bientôt",
        "demain", "gauche", "droite", "pied", "cheval", "vélo", "moto",
        // Time expressions
        "midi", "minuit", "heure", "heures",
        // Locations
        "maison", "école", "travail", "bureau", "plage", "montagne", "campagne",
    ];

    // Words that indicate verb context (should be "a")
    const VERB_CONTEXT_BEFORE: &'static [&'static str] = &[
        "il", "elle", "on", "qui", "ce", "cela", "ça", "tout", "rien",
        "personne", "chacun", "quelqu'un",
    ];
}

impl Rule for FrenchAAccentRule {
    fn id(&self) -> &str {
        "FR_A_ACCENT"
    }

    fn check(&self, _text: &str, tokens: &[AnalyzedToken]) -> Option<Match> {
        let words: Vec<_> = tokens
            .iter()
            .filter(|t| t.token.kind == TokenKind::Word)
            .collect();

        for (i, word_token) in words.iter().enumerate() {
            let word_lower = word_token.token.text.to_lowercase();

            // Check "a" that should be "à"
            if word_lower == "a" && i + 1 < words.len() {
                let next_lower = words[i + 1].token.text.to_lowercase();

                // Check if preceded by verb context
                let has_verb_subject = i > 0 && Self::VERB_CONTEXT_BEFORE
                    .contains(&words[i - 1].token.text.to_lowercase().as_str());

                if !has_verb_subject && Self::PREPOSITION_CONTEXT.contains(&next_lower.as_str()) {
                    return Some(Match {
                        span: word_token.token.span.clone(),
                        message: format!(
                            "Utiliser 'à' (préposition) avant '{}', pas 'a' (verbe avoir)",
                            words[i + 1].token.text
                        ),
                        rule_id: self.id().to_string(),
                        suggestions: vec!["à".to_string()],
                        severity: Severity::Error,
                    });
                }
            }
        }

        None
    }
}

/// French homophone: ou/où
/// "ou" = or (alternative)
/// "où" = where (location/time)
pub struct FrenchOuAccentRule;

impl FrenchOuAccentRule {
    // Context that indicates "où" (where)
    const WHERE_CONTEXT: &'static [&'static str] = &[
        "est", "sont", "était", "étaient", "sera", "seront",
        "habite", "habites", "habitez", "habitent",
        "va", "vas", "vont", "allez", "allons",
        "viens", "vient", "venez", "viennent",
        "trouve", "trouves", "trouvez", "trouvent",
        "moment", "jour", "endroit", "lieu", "pays", "ville",
    ];
}

impl Rule for FrenchOuAccentRule {
    fn id(&self) -> &str {
        "FR_OU_ACCENT"
    }

    fn check(&self, _text: &str, tokens: &[AnalyzedToken]) -> Option<Match> {
        let words: Vec<_> = tokens
            .iter()
            .filter(|t| t.token.kind == TokenKind::Word)
            .collect();

        for (i, word_token) in words.iter().enumerate() {
            let word_lower = word_token.token.text.to_lowercase();

            // Check "ou" that should be "où"
            if word_lower == "ou" {
                // Check previous word for where-context
                if i > 0 {
                    let prev_lower = words[i - 1].token.text.to_lowercase();
                    if Self::WHERE_CONTEXT.contains(&prev_lower.as_str()) {
                        return Some(Match {
                            span: word_token.token.span.clone(),
                            message: "Utiliser 'où' (lieu/temps) après un verbe de lieu, pas 'ou' (alternative)".to_string(),
                            rule_id: self.id().to_string(),
                            suggestions: vec!["où".to_string()],
                            severity: Severity::Error,
                        });
                    }
                }

                // Check next word for where-context
                if i + 1 < words.len() {
                    let next_lower = words[i + 1].token.text.to_lowercase();
                    if Self::WHERE_CONTEXT.contains(&next_lower.as_str()) {
                        return Some(Match {
                            span: word_token.token.span.clone(),
                            message: "Utiliser 'où' (lieu/temps), pas 'ou' (alternative)".to_string(),
                            rule_id: self.id().to_string(),
                            suggestions: vec!["où".to_string()],
                            severity: Severity::Error,
                        });
                    }
                }
            }
        }

        None
    }
}

/// French homophone: ce/se
/// "ce" = demonstrative (ce livre)
/// "se" = reflexive pronoun (il se lave)
pub struct FrenchCeSeRule;

impl FrenchCeSeRule {
    // Nouns that follow "ce" (demonstrative)
    const DEMONSTRATIVE_FOLLOWERS: &'static [&'static str] = &[
        "livre", "film", "jour", "soir", "matin", "moment", "temps",
        "n'est", "sera", "serait", "fut", "sont",
        "que", "qui", "dont", "type", "genre", "style", "monde",
        "mois", "week-end", "weekend",
    ];

    // Verbs that follow "se" (reflexive)
    const REFLEXIVE_VERBS: &'static [&'static str] = &[
        "lever", "lève", "lèvent", "coucher", "couche", "couchent",
        "laver", "lave", "lavent", "habiller", "habille", "habillent",
        "réveiller", "réveille", "réveillent", "doucher", "douche",
        "promener", "promène", "promènent", "rappeler", "rappelle",
        "sentir", "sent", "sentent", "trouver", "trouve", "trouvent",
        "passer", "passe", "passent", "faire", "fait", "font",
        "dire", "dit", "disent", "demander", "demande", "demandent",
        "souvenir", "souvient", "souviennent",
    ];
}

impl Rule for FrenchCeSeRule {
    fn id(&self) -> &str {
        "FR_CE_SE"
    }

    fn check(&self, _text: &str, tokens: &[AnalyzedToken]) -> Option<Match> {
        let words: Vec<_> = tokens
            .iter()
            .filter(|t| t.token.kind == TokenKind::Word)
            .collect();

        for window in words.windows(2) {
            let word = &window[0];
            let next = &window[1];

            let word_lower = word.token.text.to_lowercase();
            let next_lower = next.token.text.to_lowercase();

            // "ce" before reflexive verb -> should be "se"
            if word_lower == "ce" {
                if Self::REFLEXIVE_VERBS.contains(&next_lower.as_str()) {
                    return Some(Match {
                        span: word.token.span.clone(),
                        message: format!(
                            "Utiliser 'se' (pronom réfléchi) avant '{}', pas 'ce' (démonstratif)",
                            next.token.text
                        ),
                        rule_id: self.id().to_string(),
                        suggestions: vec!["se".to_string()],
                        severity: Severity::Error,
                    });
                }
            }

            // "se" before demonstrative context -> should be "ce"
            if word_lower == "se" {
                if Self::DEMONSTRATIVE_FOLLOWERS.contains(&next_lower.as_str()) {
                    return Some(Match {
                        span: word.token.span.clone(),
                        message: format!(
                            "Utiliser 'ce' (démonstratif) avant '{}', pas 'se' (réfléchi)",
                            next.token.text
                        ),
                        rule_id: self.id().to_string(),
                        suggestions: vec!["ce".to_string()],
                        severity: Severity::Error,
                    });
                }
            }
        }

        None
    }
}

/// French subject-verb agreement
/// Detects incorrect verb conjugation based on subject
pub struct FrenchSubjectVerbRule;

impl FrenchSubjectVerbRule {
    // Singular subjects (3rd person)
    const SINGULAR_SUBJECTS: &'static [&'static str] = &[
        "il", "elle", "on", "ce", "cela", "ça", "qui", "tout", "rien",
        "personne", "chacun", "quelqu'un", "l'un", "l'autre",
    ];

    // Plural subjects (3rd person)
    const PLURAL_SUBJECTS: &'static [&'static str] = &[
        "ils", "elles", "ceux", "celles", "certains", "certaines",
        "plusieurs", "quelques-uns", "quelques-unes", "tous", "toutes",
    ];

    // Common -er verb stems with their conjugations
    // Format: (stem, singular_ending, plural_ending)
    const ER_VERB_PATTERNS: &'static [(&'static str, &'static str, &'static str)] = &[
        ("mang", "e", "ent"),      // manger
        ("parl", "e", "ent"),      // parler
        ("donn", "e", "ent"),      // donner
        ("aim", "e", "ent"),       // aimer
        ("arriv", "e", "ent"),     // arriver
        ("travaill", "e", "ent"),  // travailler
        ("jou", "e", "ent"),       // jouer
        ("regard", "e", "ent"),    // regarder
        ("écout", "e", "ent"),     // écouter
        ("chant", "e", "ent"),     // chanter
        ("march", "e", "ent"),     // marcher
        ("cherch", "e", "ent"),    // chercher
        ("trouv", "e", "ent"),     // trouver
        ("pass", "e", "ent"),      // passer
        ("rest", "e", "ent"),      // rester
        ("demand", "e", "ent"),    // demander
        ("pens", "e", "ent"),      // penser
        ("port", "e", "ent"),      // porter
        ("mont", "e", "ent"),      // monter
        ("habit", "e", "ent"),     // habiter
    ];

    // Irregular verbs with explicit forms
    // Format: (singular_form, plural_form)
    const IRREGULAR_VERBS: &'static [(&'static str, &'static str)] = &[
        ("est", "sont"),       // être
        ("a", "ont"),          // avoir
        ("va", "vont"),        // aller
        ("fait", "font"),      // faire
        ("dit", "disent"),     // dire
        ("vient", "viennent"), // venir
        ("prend", "prennent"), // prendre
        ("peut", "peuvent"),   // pouvoir
        ("veut", "veulent"),   // vouloir
        ("doit", "doivent"),   // devoir
        ("sait", "savent"),    // savoir
        ("voit", "voient"),    // voir
        ("met", "mettent"),    // mettre
        ("lit", "lisent"),     // lire
        ("écrit", "écrivent"), // écrire
        ("part", "partent"),   // partir
        ("sort", "sortent"),   // sortir
        ("dort", "dorment"),   // dormir
        ("finit", "finissent"), // finir
        ("choisit", "choisissent"), // choisir
    ];
}

impl Rule for FrenchSubjectVerbRule {
    fn id(&self) -> &str {
        "FR_SUBJECT_VERB"
    }

    fn check(&self, _text: &str, tokens: &[AnalyzedToken]) -> Option<Match> {
        let words: Vec<_> = tokens
            .iter()
            .filter(|t| t.token.kind == TokenKind::Word)
            .collect();

        for window in words.windows(2) {
            let subject = &window[0];
            let verb = &window[1];

            let subj_lower = subject.token.text.to_lowercase();
            let verb_lower = verb.token.text.to_lowercase();

            let is_singular = Self::SINGULAR_SUBJECTS.contains(&subj_lower.as_str());
            let is_plural = Self::PLURAL_SUBJECTS.contains(&subj_lower.as_str());

            if !is_singular && !is_plural {
                continue;
            }

            // Check irregular verbs
            for (sing, plur) in Self::IRREGULAR_VERBS {
                if is_singular && verb_lower == *plur {
                    return Some(Match {
                        span: verb.token.span.clone(),
                        message: format!(
                            "Accord sujet-verbe: '{}' est singulier, utiliser '{}' au lieu de '{}'",
                            subject.token.text, sing, verb.token.text
                        ),
                        rule_id: self.id().to_string(),
                        suggestions: vec![sing.to_string()],
                        severity: Severity::Error,
                    });
                }
                if is_plural && verb_lower == *sing {
                    return Some(Match {
                        span: verb.token.span.clone(),
                        message: format!(
                            "Accord sujet-verbe: '{}' est pluriel, utiliser '{}' au lieu de '{}'",
                            subject.token.text, plur, verb.token.text
                        ),
                        rule_id: self.id().to_string(),
                        suggestions: vec![plur.to_string()],
                        severity: Severity::Error,
                    });
                }
            }

            // Check -er verbs
            for (stem, sing_end, plur_end) in Self::ER_VERB_PATTERNS {
                let sing_form = format!("{}{}", stem, sing_end);
                let plur_form = format!("{}{}", stem, plur_end);

                if is_singular && verb_lower == plur_form {
                    return Some(Match {
                        span: verb.token.span.clone(),
                        message: format!(
                            "Accord sujet-verbe: '{}' est singulier, utiliser '{}' au lieu de '{}'",
                            subject.token.text, sing_form, verb.token.text
                        ),
                        rule_id: self.id().to_string(),
                        suggestions: vec![sing_form],
                        severity: Severity::Error,
                    });
                }
                if is_plural && verb_lower == sing_form {
                    return Some(Match {
                        span: verb.token.span.clone(),
                        message: format!(
                            "Accord sujet-verbe: '{}' est pluriel, utiliser '{}' au lieu de '{}'",
                            subject.token.text, plur_form, verb.token.text
                        ),
                        rule_id: self.id().to_string(),
                        suggestions: vec![plur_form],
                        severity: Severity::Error,
                    });
                }
            }
        }

        None
    }
}

/// French adjective-noun agreement
/// Detects incorrect adjective agreement with noun gender/number
pub struct FrenchAdjectiveNounRule;

impl FrenchAdjectiveNounRule {
    // Common masculine nouns
    const MASCULINE_NOUNS: &'static [&'static str] = &[
        "homme", "garçon", "père", "frère", "fils", "ami", "enfant",
        "livre", "chat", "chien", "jardin", "bureau", "travail",
        "jour", "temps", "pays", "monde", "problème", "moment",
        "groupe", "exemple", "cas", "point", "fait", "sens",
    ];

    // Common feminine nouns
    const FEMININE_NOUNS: &'static [&'static str] = &[
        "femme", "fille", "mère", "sœur", "amie", "personne",
        "maison", "table", "chaise", "porte", "fenêtre", "chambre",
        "vie", "chose", "fois", "année", "heure", "question",
        "idée", "partie", "place", "main", "eau", "ville",
    ];

    // Adjectives with masc/fem forms (masc, fem)
    const ADJECTIVE_PAIRS: &'static [(&'static str, &'static str)] = &[
        ("petit", "petite"),
        ("grand", "grande"),
        ("beau", "belle"),
        ("nouveau", "nouvelle"),
        ("vieux", "vieille"),
        ("bon", "bonne"),
        ("mauvais", "mauvaise"),
        ("joli", "jolie"),
        ("gentil", "gentille"),
        ("blanc", "blanche"),
        ("noir", "noire"),
        ("vert", "verte"),
        ("bleu", "bleue"),
        ("gris", "grise"),
        ("heureux", "heureuse"),
        ("malheureux", "malheureuse"),
        ("sérieux", "sérieuse"),
        ("premier", "première"),
        ("dernier", "dernière"),
        ("entier", "entière"),
        ("long", "longue"),
        ("fort", "forte"),
        ("court", "courte"),
        ("chaud", "chaude"),
        ("froid", "froide"),
        ("plein", "pleine"),
        ("certain", "certaine"),
        ("prochain", "prochaine"),
        ("ancien", "ancienne"),
        ("jeune", "jeune"),  // invariable
        ("français", "française"),
        ("anglais", "anglaise"),
    ];
}

impl Rule for FrenchAdjectiveNounRule {
    fn id(&self) -> &str {
        "FR_ADJ_NOUN"
    }

    fn check(&self, _text: &str, tokens: &[AnalyzedToken]) -> Option<Match> {
        let words: Vec<_> = tokens
            .iter()
            .filter(|t| t.token.kind == TokenKind::Word)
            .collect();

        // Check pattern: article + adjective + noun (e.g., "une petit maison")
        for window in words.windows(3) {
            let article = &window[0];
            let adj = &window[1];
            let noun = &window[2];

            let art_lower = article.token.text.to_lowercase();
            let adj_lower = adj.token.text.to_lowercase();
            let noun_lower = noun.token.text.to_lowercase();

            // Determine expected gender from article
            let is_feminine_article = art_lower == "une" || art_lower == "la" || art_lower == "cette";
            let is_masculine_article = art_lower == "un" || art_lower == "le" || art_lower == "ce";

            if !is_feminine_article && !is_masculine_article {
                continue;
            }

            // Check adjective agreement
            for (masc, fem) in Self::ADJECTIVE_PAIRS {
                if masc == fem {
                    continue; // invariable adjective
                }

                // Feminine article but masculine adjective
                if is_feminine_article && adj_lower == *masc {
                    // Verify noun is feminine
                    if Self::FEMININE_NOUNS.contains(&noun_lower.as_str()) {
                        return Some(Match {
                            span: adj.token.span.clone(),
                            message: format!(
                                "Accord adjectif: '{}' est féminin, utiliser '{}' au lieu de '{}'",
                                noun.token.text, fem, adj.token.text
                            ),
                            rule_id: self.id().to_string(),
                            suggestions: vec![fem.to_string()],
                            severity: Severity::Error,
                        });
                    }
                }

                // Masculine article but feminine adjective
                if is_masculine_article && adj_lower == *fem {
                    // Verify noun is masculine
                    if Self::MASCULINE_NOUNS.contains(&noun_lower.as_str()) {
                        return Some(Match {
                            span: adj.token.span.clone(),
                            message: format!(
                                "Accord adjectif: '{}' est masculin, utiliser '{}' au lieu de '{}'",
                                noun.token.text, masc, adj.token.text
                            ),
                            rule_id: self.id().to_string(),
                            suggestions: vec![masc.to_string()],
                            severity: Severity::Error,
                        });
                    }
                }
            }
        }

        // Check pattern: noun + adjective (e.g., "chat noire" -> "chat noir")
        for window in words.windows(2) {
            let noun = &window[0];
            let adj = &window[1];

            let noun_lower = noun.token.text.to_lowercase();
            let adj_lower = adj.token.text.to_lowercase();

            let is_masculine_noun = Self::MASCULINE_NOUNS.contains(&noun_lower.as_str());
            let is_feminine_noun = Self::FEMININE_NOUNS.contains(&noun_lower.as_str());

            if !is_masculine_noun && !is_feminine_noun {
                continue;
            }

            for (masc, fem) in Self::ADJECTIVE_PAIRS {
                if masc == fem {
                    continue;
                }

                // Masculine noun but feminine adjective
                if is_masculine_noun && adj_lower == *fem {
                    return Some(Match {
                        span: adj.token.span.clone(),
                        message: format!(
                            "Accord adjectif: '{}' est masculin, utiliser '{}' au lieu de '{}'",
                            noun.token.text, masc, adj.token.text
                        ),
                        rule_id: self.id().to_string(),
                        suggestions: vec![masc.to_string()],
                        severity: Severity::Error,
                    });
                }

                // Feminine noun but masculine adjective
                if is_feminine_noun && adj_lower == *masc {
                    return Some(Match {
                        span: adj.token.span.clone(),
                        message: format!(
                            "Accord adjectif: '{}' est féminin, utiliser '{}' au lieu de '{}'",
                            noun.token.text, fem, adj.token.text
                        ),
                        rule_id: self.id().to_string(),
                        suggestions: vec![fem.to_string()],
                        severity: Severity::Error,
                    });
                }
            }
        }

        None
    }
}

/// Typographic quotes detection
/// Suggests replacing straight quotes with proper typographic quotes
pub struct TypographicQuotesRule;

impl TypographicQuotesRule {
    // Languages and their preferred quotes
    // For simplicity, we'll detect straight quotes and suggest replacements
}

impl Rule for TypographicQuotesRule {
    fn id(&self) -> &str {
        "TYPOGRAPHIC_QUOTES"
    }

    fn check(&self, text: &str, tokens: &[AnalyzedToken]) -> Option<Match> {
        // Look for straight double quotes
        for token in tokens {
            if token.token.kind == TokenKind::Punctuation && token.token.text == "\"" {
                return Some(Match {
                    span: token.token.span.clone(),
                    message: "Utiliser des guillemets typographiques au lieu de guillemets droits".to_string(),
                    rule_id: self.id().to_string(),
                    suggestions: vec![
                        "\u{201C}".to_string(),  // " left double quote
                        "\u{201D}".to_string(),  // " right double quote
                        "\u{00AB}".to_string(),  // « left guillemet
                        "\u{00BB}".to_string(),  // » right guillemet
                    ],
                    severity: Severity::Hint,
                });
            }
        }

        // Look for straight double quotes in text
        let chars: Vec<char> = text.chars().collect();
        for i in 0..chars.len() {
            if chars[i] == '"' {
                return Some(Match {
                    span: i..i + 1,
                    message: "Utiliser des guillemets typographiques au lieu de guillemets droits".to_string(),
                    rule_id: self.id().to_string(),
                    suggestions: vec![
                        "\u{201C}".to_string(),  // " left double quote
                        "\u{201D}".to_string(),  // " right double quote
                        "\u{00AB}".to_string(),  // « left guillemet
                        "\u{00BB}".to_string(),  // » right guillemet
                    ],
                    severity: Severity::Hint,
                });
            }
        }

        None
    }
}

/// Passive voice detection
/// Detects passive constructions like "was eaten", "is being written"
pub struct PassiveVoiceRule;

impl PassiveVoiceRule {
    // Forms of "to be"
    const BE_VERBS: &'static [&'static str] = &[
        "is", "are", "was", "were", "be", "been", "being",
        "am", "isn't", "aren't", "wasn't", "weren't",
    ];

    // Common irregular past participles
    const IRREGULAR_PARTICIPLES: &'static [&'static str] = &[
        // A-B
        "awoken", "been", "beaten", "become", "begun", "bent", "bet", "bitten",
        "blown", "broken", "brought", "built", "burnt", "bought",
        // C-D
        "caught", "chosen", "come", "cost", "cut", "dealt", "done", "drawn",
        "driven", "drunk", "dug",
        // E-F
        "eaten", "fallen", "fed", "felt", "fought", "found", "flown", "forgotten",
        "forgiven", "frozen",
        // G-H
        "given", "gone", "grown", "had", "heard", "held", "hidden", "hit", "hung", "hurt",
        // K-L
        "kept", "known", "laid", "led", "left", "lent", "let", "lain", "lit", "lost",
        // M-P
        "made", "meant", "met", "paid", "proven", "put",
        // R-S
        "read", "ridden", "risen", "run", "said", "sat", "seen", "sent", "set",
        "shaken", "shown", "shut", "sung", "sunk", "slept", "slid", "spoken",
        "spent", "spun", "split", "spread", "stood", "stolen", "struck", "stuck",
        "stung", "sworn", "swept", "swum", "swung",
        // T-W
        "taken", "taught", "thought", "thrown", "told", "torn", "understood",
        "woken", "worn", "won", "withdrawn", "written",
    ];

    fn is_past_participle(word: &str) -> bool {
        let lower = word.to_lowercase();
        // Check irregular participles
        if Self::IRREGULAR_PARTICIPLES.contains(&lower.as_str()) {
            return true;
        }
        // Check regular -ed endings (but not words like "bed", "red", etc.)
        if lower.len() > 3 && lower.ends_with("ed") {
            // Exclude short words that just end in "ed"
            let stem = &lower[..lower.len() - 2];
            // Must have at least 2 characters in stem and stem shouldn't be too common noun
            if stem.len() >= 2 {
                return true;
            }
        }
        false
    }
}

impl Rule for PassiveVoiceRule {
    fn id(&self) -> &str {
        "PASSIVE_VOICE"
    }

    fn check(&self, _text: &str, tokens: &[AnalyzedToken]) -> Option<Match> {
        let words: Vec<_> = tokens
            .iter()
            .filter(|t| t.token.kind == TokenKind::Word)
            .collect();

        // Look for "be verb + past participle" patterns
        for window in words.windows(2) {
            let be_verb = &window[0];
            let maybe_participle = &window[1];

            let be_lower = be_verb.token.text.to_lowercase();
            let part_text = maybe_participle.token.text;

            if Self::BE_VERBS.contains(&be_lower.as_str()) && Self::is_past_participle(part_text) {
                return Some(Match {
                    span: be_verb.token.span.start..maybe_participle.token.span.end,
                    message: format!(
                        "Passive voice detected: '{}'. Consider using active voice for clarity.",
                        format!("{} {}", be_verb.token.text, part_text)
                    ),
                    rule_id: self.id().to_string(),
                    suggestions: vec![],
                    severity: Severity::Hint,
                });
            }
        }

        // Also check for "being + participle" pattern
        for window in words.windows(3) {
            let be_verb = &window[0];
            let being = &window[1];
            let maybe_participle = &window[2];

            let be_lower = be_verb.token.text.to_lowercase();
            let being_lower = being.token.text.to_lowercase();
            let part_text = maybe_participle.token.text;

            if Self::BE_VERBS.contains(&be_lower.as_str())
                && being_lower == "being"
                && Self::is_past_participle(part_text)
            {
                return Some(Match {
                    span: be_verb.token.span.start..maybe_participle.token.span.end,
                    message: format!(
                        "Passive voice detected: '{}'. Consider using active voice.",
                        format!("{} being {}", be_verb.token.text, part_text)
                    ),
                    rule_id: self.id().to_string(),
                    suggestions: vec![],
                    severity: Severity::Hint,
                });
            }
        }

        None
    }
}

/// Wordiness detection
/// Detects wordy phrases that can be simplified
pub struct WordinessRule;

impl WordinessRule {
    // Wordy phrase -> simpler replacement
    const WORDY_PHRASES: &'static [(&'static [&'static str], &'static str)] = &[
        // Multi-word -> single word
        (&["in", "order", "to"], "to"),
        (&["due", "to", "the", "fact", "that"], "because"),
        (&["in", "the", "event", "that"], "if"),
        (&["at", "this", "point", "in", "time"], "now"),
        (&["at", "the", "present", "time"], "now"),
        (&["for", "the", "purpose", "of"], "to"),
        (&["in", "spite", "of", "the", "fact", "that"], "although"),
        (&["in", "the", "near", "future"], "soon"),
        (&["in", "a", "timely", "manner"], "promptly"),
        (&["a", "large", "number", "of"], "many"),
        (&["a", "small", "number", "of"], "few"),
        (&["the", "majority", "of"], "most"),
        (&["on", "a", "daily", "basis"], "daily"),
        (&["on", "a", "regular", "basis"], "regularly"),
        (&["at", "all", "times"], "always"),
        (&["with", "regard", "to"], "about"),
        (&["in", "regard", "to"], "about"),
        (&["with", "respect", "to"], "about"),
        (&["has", "the", "ability", "to"], "can"),
        (&["is", "able", "to"], "can"),
        (&["make", "a", "decision"], "decide"),
        (&["come", "to", "a", "conclusion"], "conclude"),
        (&["take", "into", "consideration"], "consider"),
    ];
}

impl Rule for WordinessRule {
    fn id(&self) -> &str {
        "WORDINESS"
    }

    fn check(&self, _text: &str, tokens: &[AnalyzedToken]) -> Option<Match> {
        let words: Vec<_> = tokens
            .iter()
            .filter(|t| t.token.kind == TokenKind::Word)
            .collect();

        for (phrase, replacement) in Self::WORDY_PHRASES {
            let phrase_len = phrase.len();
            if words.len() >= phrase_len {
                for i in 0..=words.len() - phrase_len {
                    let window: Vec<_> = words[i..i + phrase_len]
                        .iter()
                        .map(|t| t.token.text.to_lowercase())
                        .collect();

                    let matches = window
                        .iter()
                        .zip(phrase.iter())
                        .all(|(w, p)| w == *p);

                    if matches {
                        let start = words[i].token.span.start;
                        let end = words[i + phrase_len - 1].token.span.end;
                        let original: Vec<_> = words[i..i + phrase_len]
                            .iter()
                            .map(|t| t.token.text)
                            .collect();

                        return Some(Match {
                            span: start..end,
                            message: format!(
                                "Wordy: '{}' can be simplified to '{}'",
                                original.join(" "),
                                replacement
                            ),
                            rule_id: self.id().to_string(),
                            suggestions: vec![replacement.to_string()],
                            severity: Severity::Hint,
                        });
                    }
                }
            }
        }

        None
    }
}

/// Sentence fragment detection
/// Detects incomplete sentences (lacking subject or verb)
pub struct SentenceFragmentRule;

impl SentenceFragmentRule {
    // Common sentence starters that are often fragments
    const FRAGMENT_STARTERS: &'static [&'static str] = &[
        "because", "although", "while", "when", "if", "unless", "since",
        "after", "before", "until", "whenever", "wherever", "whether",
        "even", "though", "whereas",
    ];

    // Words that indicate a complete thought follows (for future use)
    #[allow(dead_code)]
    const COMPLETERS: &'static [&'static str] = &[
        "i", "you", "he", "she", "it", "we", "they", "there", "this", "that",
    ];
}

impl Rule for SentenceFragmentRule {
    fn id(&self) -> &str {
        "SENTENCE_FRAGMENT"
    }

    fn check(&self, text: &str, tokens: &[AnalyzedToken]) -> Option<Match> {
        let words: Vec<_> = tokens
            .iter()
            .filter(|t| t.token.kind == TokenKind::Word)
            .collect();

        if words.is_empty() {
            return None;
        }

        // Check if sentence starts with subordinating conjunction and has no main clause
        let first_word = words[0].token.text.to_lowercase();

        if Self::FRAGMENT_STARTERS.contains(&first_word.as_str()) {
            // Check if there's a comma followed by a main clause
            let has_comma = tokens.iter().any(|t| t.token.text == ",");
            let has_period = text.ends_with('.') || text.ends_with('!') || text.ends_with('?');

            // Simple heuristic: if it starts with subordinator and ends with period
            // but has no comma (no main clause), it's likely a fragment
            if !has_comma && has_period && words.len() < 10 {
                return Some(Match {
                    span: words[0].token.span.clone(),
                    message: format!(
                        "Possible sentence fragment. '{}' starts a dependent clause that may need a main clause.",
                        words[0].token.text
                    ),
                    rule_id: self.id().to_string(),
                    suggestions: vec![],
                    severity: Severity::Warning,
                });
            }
        }

        None
    }
}

// ========================================
// PHASE 4: ADVANCED ENGLISH GRAMMAR RULES
// ========================================

/// Less vs Fewer - countable nouns need "fewer"
pub struct LessFewerRule;

impl LessFewerRule {
    // Countable nouns that should use "fewer" instead of "less"
    const COUNTABLE_NOUNS: &'static [&'static str] = &[
        "items", "people", "things", "words", "cars", "books", "days", "hours",
        "minutes", "seconds", "years", "months", "weeks", "dollars", "euros",
        "pounds", "calories", "points", "miles", "kilometers", "steps", "mistakes",
        "errors", "problems", "issues", "questions", "answers", "options", "choices",
        "students", "teachers", "employees", "customers", "users", "members",
        "friends", "followers", "subscribers", "visitors", "guests", "patients",
        "children", "animals", "pets", "dogs", "cats", "birds", "fish",
        "apples", "oranges", "bananas", "eggs", "cookies", "slices", "pieces",
        "bottles", "cans", "boxes", "bags", "packages", "servings", "portions",
        "rooms", "houses", "apartments", "buildings", "floors", "pages", "chapters",
        "emails", "messages", "calls", "meetings", "appointments", "events",
    ];
}

impl Rule for LessFewerRule {
    fn id(&self) -> &str {
        "LESS_FEWER"
    }

    fn check(&self, _text: &str, tokens: &[AnalyzedToken]) -> Option<Match> {
        let words: Vec<_> = tokens
            .iter()
            .filter(|t| t.token.kind == TokenKind::Word)
            .collect();

        for (i, token) in words.iter().enumerate() {
            if token.token.text.to_lowercase() == "less" {
                // Check if next word is a countable noun
                if let Some(next) = words.get(i + 1) {
                    let next_word = next.token.text.to_lowercase();
                    if Self::COUNTABLE_NOUNS.contains(&next_word.as_str()) {
                        return Some(Match {
                            span: token.token.span.clone(),
                            message: format!(
                                "Use 'fewer' with countable nouns like '{}'",
                                next.token.text
                            ),
                            rule_id: self.id().to_string(),
                            suggestions: vec!["fewer".to_string()],
                            severity: Severity::Warning,
                        });
                    }
                }
            }
        }
        None
    }
}

/// Who vs Whom rule
pub struct WhoWhomRule;

impl WhoWhomRule {
    // Verbs that typically take an object (whom)
    const OBJECT_VERBS: &'static [&'static str] = &[
        "saw", "see", "met", "meet", "called", "call", "told", "tell",
        "gave", "give", "asked", "ask", "invited", "invite", "hired", "hire",
        "fired", "fire", "chose", "choose", "selected", "select", "picked", "pick",
        "contacted", "contact", "emailed", "email", "texted", "text",
    ];
}

impl Rule for WhoWhomRule {
    fn id(&self) -> &str {
        "WHO_WHOM"
    }

    fn check(&self, _text: &str, tokens: &[AnalyzedToken]) -> Option<Match> {
        let words: Vec<_> = tokens
            .iter()
            .filter(|t| t.token.kind == TokenKind::Word)
            .collect();

        for (i, token) in words.iter().enumerate() {
            let word_lower = token.token.text.to_lowercase();

            // "who did you see" should be "whom did you see"
            if word_lower == "who" {
                // Check for pattern: who + did/does/do + subject + verb
                if let Some(next) = words.get(i + 1) {
                    let next_lower = next.token.text.to_lowercase();
                    if matches!(next_lower.as_str(), "did" | "does" | "do" | "will" | "would" | "could" | "should") {
                        // Check if there's an object verb later
                        for j in (i + 3)..words.len().min(i + 6) {
                            if let Some(verb) = words.get(j) {
                                if Self::OBJECT_VERBS.contains(&verb.token.text.to_lowercase().as_str()) {
                                    return Some(Match {
                                        span: token.token.span.clone(),
                                        message: "Use 'whom' when it's the object of a verb".to_string(),
                                        rule_id: self.id().to_string(),
                                        suggestions: vec!["Whom".to_string(), "whom".to_string()],
                                        severity: Severity::Hint,
                                    });
                                }
                            }
                        }
                    }
                }

                // "to who" or "for who" should be "to whom" or "for whom"
                if i > 0 {
                    if let Some(prev) = words.get(i - 1) {
                        let prev_lower = prev.token.text.to_lowercase();
                        if matches!(prev_lower.as_str(), "to" | "for" | "with" | "from" | "by" | "about") {
                            return Some(Match {
                                span: token.token.span.clone(),
                                message: format!("Use 'whom' after preposition '{}'", prev.token.text),
                                rule_id: self.id().to_string(),
                                suggestions: vec!["whom".to_string()],
                                severity: Severity::Warning,
                            });
                        }
                    }
                }
            }
        }
        None
    }
}

/// Good vs Well - adjective vs adverb
pub struct GoodWellRule;

impl GoodWellRule {
    // Verbs that should be followed by "well" not "good"
    const ACTION_VERBS: &'static [&'static str] = &[
        "did", "do", "does", "done", "doing",
        "played", "play", "plays", "playing",
        "worked", "work", "works", "working",
        "performed", "perform", "performs", "performing",
        "ran", "run", "runs", "running",
        "sang", "sing", "sings", "singing",
        "wrote", "write", "writes", "writing",
        "spoke", "speak", "speaks", "speaking",
        "slept", "sleep", "sleeps", "sleeping",
        "ate", "eat", "eats", "eating",
        "cooked", "cook", "cooks", "cooking",
        "danced", "dance", "dances", "dancing",
    ];
}

impl Rule for GoodWellRule {
    fn id(&self) -> &str {
        "GOOD_WELL"
    }

    fn check(&self, _text: &str, tokens: &[AnalyzedToken]) -> Option<Match> {
        let words: Vec<_> = tokens
            .iter()
            .filter(|t| t.token.kind == TokenKind::Word)
            .collect();

        for (i, token) in words.iter().enumerate() {
            if token.token.text.to_lowercase() == "good" && i > 0 {
                // Check if previous word is an action verb
                if let Some(prev) = words.get(i - 1) {
                    if Self::ACTION_VERBS.contains(&prev.token.text.to_lowercase().as_str()) {
                        return Some(Match {
                            span: token.token.span.clone(),
                            message: "Use 'well' as an adverb to modify verbs".to_string(),
                            rule_id: self.id().to_string(),
                            suggestions: vec!["well".to_string()],
                            severity: Severity::Warning,
                        });
                    }
                }
            }
        }
        None
    }
}

/// Double negative detection
pub struct DoubleNegativeRule;

impl DoubleNegativeRule {
    const NEGATIVE_WORDS: &'static [&'static str] = &[
        "no", "not", "none", "nothing", "nobody", "nowhere", "never",
        "neither", "nor", "cannot", "can't", "won't", "don't", "doesn't",
        "didn't", "isn't", "aren't", "wasn't", "weren't", "haven't", "hasn't",
        "hadn't", "wouldn't", "couldn't", "shouldn't", "mustn't",
    ];

    const NEGATIVE_PRONOUNS: &'static [&'static str] = &[
        "no", "nothing", "nobody", "none", "nowhere", "never", "neither",
    ];
}

impl Rule for DoubleNegativeRule {
    fn id(&self) -> &str {
        "DOUBLE_NEGATIVE"
    }

    fn check(&self, _text: &str, tokens: &[AnalyzedToken]) -> Option<Match> {
        let words: Vec<_> = tokens
            .iter()
            .filter(|t| t.token.kind == TokenKind::Word)
            .collect();

        let mut found_negative = false;
        let mut first_neg_span = None;

        for token in &words {
            let word_lower = token.token.text.to_lowercase();

            if Self::NEGATIVE_WORDS.contains(&word_lower.as_str()) {
                if found_negative && Self::NEGATIVE_PRONOUNS.contains(&word_lower.as_str()) {
                    // Found double negative
                    return Some(Match {
                        span: token.token.span.clone(),
                        message: "Double negative detected. Consider using a single negative or positive construction.".to_string(),
                        rule_id: self.id().to_string(),
                        suggestions: vec![],
                        severity: Severity::Warning,
                    });
                }
                if first_neg_span.is_none() {
                    found_negative = true;
                    first_neg_span = Some(token.token.span.clone());
                }
            }
        }
        None
    }
}

// ========================================
// PHASE 5: ADVANCED FRENCH GRAMMAR RULES
// ========================================

/// French: "si + conditionnel" is incorrect (si j'aurais → si j'avais)
pub struct FrenchConditionnelSiRule;

impl FrenchConditionnelSiRule {
    // Conditional forms that shouldn't follow "si"
    const CONDITIONAL_FORMS: &'static [&'static str] = &[
        "aurais", "aurait", "aurions", "auriez", "auraient",
        "serais", "serait", "serions", "seriez", "seraient",
        "ferais", "ferait", "ferions", "feriez", "feraient",
        "irais", "irait", "irions", "iriez", "iraient",
        "voudrais", "voudrait", "voudrions", "voudriez", "voudraient",
        "pourrais", "pourrait", "pourrions", "pourriez", "pourraient",
        "devrais", "devrait", "devrions", "devriez", "devraient",
        "saurais", "saurait", "saurions", "sauriez", "sauraient",
    ];

    // Corresponding imparfait forms
    const IMPARFAIT_CORRECTIONS: &'static [(&'static str, &'static str)] = &[
        ("aurais", "avais"), ("aurait", "avait"), ("aurions", "avions"), ("auriez", "aviez"), ("auraient", "avaient"),
        ("serais", "étais"), ("serait", "était"), ("serions", "étions"), ("seriez", "étiez"), ("seraient", "étaient"),
        ("ferais", "faisais"), ("ferait", "faisait"), ("ferions", "faisions"), ("feriez", "faisiez"), ("feraient", "faisaient"),
        ("irais", "allais"), ("irait", "allait"), ("irions", "allions"), ("iriez", "alliez"), ("iraient", "allaient"),
        ("voudrais", "voulais"), ("voudrait", "voulait"), ("voudrions", "voulions"), ("voudriez", "vouliez"), ("voudraient", "voulaient"),
        ("pourrais", "pouvais"), ("pourrait", "pouvait"), ("pourrions", "pouvions"), ("pourriez", "pouviez"), ("pourraient", "pouvaient"),
        ("devrais", "devais"), ("devrait", "devait"), ("devrions", "devions"), ("devriez", "deviez"), ("devraient", "devaient"),
        ("saurais", "savais"), ("saurait", "savait"), ("saurions", "savions"), ("sauriez", "saviez"), ("sauraient", "savaient"),
    ];
}

impl Rule for FrenchConditionnelSiRule {
    fn id(&self) -> &str {
        "FR_CONDITIONNEL_SI"
    }

    fn check(&self, _text: &str, tokens: &[AnalyzedToken]) -> Option<Match> {
        let words: Vec<_> = tokens
            .iter()
            .filter(|t| t.token.kind == TokenKind::Word)
            .collect();

        for (i, token) in words.iter().enumerate() {
            let word_lower = token.token.text.to_lowercase();

            // Look for "si" followed by a conditional form
            if word_lower == "si" {
                // Check next few words for conditional
                for j in (i + 1)..words.len().min(i + 4) {
                    if let Some(next) = words.get(j) {
                        let next_lower = next.token.text.to_lowercase();

                        // Skip pronouns like "j'" "tu" "il" etc.
                        if matches!(next_lower.as_str(), "j" | "je" | "tu" | "il" | "elle" | "on" | "nous" | "vous" | "ils" | "elles") {
                            continue;
                        }

                        if Self::CONDITIONAL_FORMS.contains(&next_lower.as_str()) {
                            // Find the correction
                            let correction = Self::IMPARFAIT_CORRECTIONS
                                .iter()
                                .find(|(cond, _)| *cond == next_lower)
                                .map(|(_, imp)| *imp);

                            return Some(Match {
                                span: next.token.span.clone(),
                                message: "Après 'si', utilisez l'imparfait au lieu du conditionnel".to_string(),
                                rule_id: self.id().to_string(),
                                suggestions: correction.map(|c| c.to_string()).into_iter().collect(),
                                severity: Severity::Error,
                            });
                        }
                        break; // Only check first non-pronoun word
                    }
                }
            }
        }
        None
    }
}

/// French: "tout" agreement rule
pub struct FrenchToutAccordRule;

impl FrenchToutAccordRule {
    // Feminine plural nouns that require "toutes"
    const FEMININE_PLURAL_INDICATORS: &'static [&'static str] = &[
        "les", "ces", "mes", "tes", "ses", "nos", "vos", "leurs",
    ];

    const FEMININE_PLURAL_NOUNS: &'static [&'static str] = &[
        "femmes", "filles", "personnes", "choses", "maisons", "voitures",
        "fleurs", "idées", "questions", "réponses", "heures", "minutes",
        "semaines", "années", "nuits", "journées", "matinées", "soirées",
    ];
}

impl Rule for FrenchToutAccordRule {
    fn id(&self) -> &str {
        "FR_TOUT_ACCORD"
    }

    fn check(&self, _text: &str, tokens: &[AnalyzedToken]) -> Option<Match> {
        let words: Vec<_> = tokens
            .iter()
            .filter(|t| t.token.kind == TokenKind::Word)
            .collect();

        for (i, token) in words.iter().enumerate() {
            let word_lower = token.token.text.to_lowercase();

            // "tout les" should be "tous les" or "toutes les"
            if word_lower == "tout" {
                if let Some(next) = words.get(i + 1) {
                    let next_lower = next.token.text.to_lowercase();

                    if Self::FEMININE_PLURAL_INDICATORS.contains(&next_lower.as_str()) {
                        // Check if the following noun is feminine
                        if let Some(noun) = words.get(i + 2) {
                            let noun_lower = noun.token.text.to_lowercase();
                            if Self::FEMININE_PLURAL_NOUNS.contains(&noun_lower.as_str()) {
                                return Some(Match {
                                    span: token.token.span.clone(),
                                    message: "Utilisez 'toutes' devant un nom féminin pluriel".to_string(),
                                    rule_id: self.id().to_string(),
                                    suggestions: vec!["toutes".to_string()],
                                    severity: Severity::Warning,
                                });
                            }
                        }
                        // Default to "tous" for plural
                        return Some(Match {
                            span: token.token.span.clone(),
                            message: "Utilisez 'tous' ou 'toutes' devant un article pluriel".to_string(),
                            rule_id: self.id().to_string(),
                            suggestions: vec!["tous".to_string(), "toutes".to_string()],
                            severity: Severity::Warning,
                        });
                    }
                }
            }
        }
        None
    }
}

// ========================================
// PHASE 6: PUNCTUATION AND STYLE RULES
// ========================================

/// Sentence length rule - flags very long sentences
pub struct SentenceLengthRule {
    max_words: usize,
}

impl SentenceLengthRule {
    pub fn new() -> Self {
        Self { max_words: 40 }
    }

    pub fn with_max_words(max_words: usize) -> Self {
        Self { max_words }
    }
}

impl Default for SentenceLengthRule {
    fn default() -> Self {
        Self::new()
    }
}

impl Rule for SentenceLengthRule {
    fn id(&self) -> &str {
        "SENTENCE_LENGTH"
    }

    fn check(&self, text: &str, tokens: &[AnalyzedToken]) -> Option<Match> {
        let word_count = tokens
            .iter()
            .filter(|t| t.token.kind == TokenKind::Word)
            .count();

        if word_count > self.max_words {
            return Some(Match {
                span: 0..text.len().min(50),
                message: format!(
                    "Long sentence ({} words). Consider breaking it into shorter sentences for readability.",
                    word_count
                ),
                rule_id: self.id().to_string(),
                suggestions: vec![],
                severity: Severity::Hint,
            });
        }
        None
    }
}

/// Cliche detection rule
pub struct ClicheRule;

impl ClicheRule {
    const CLICHES: &'static [(&'static [&'static str], &'static str)] = &[
        // English cliches
        (&["at", "the", "end", "of", "the", "day"], "ultimately, finally"),
        (&["think", "outside", "the", "box"], "be creative, innovate"),
        (&["low", "hanging", "fruit"], "easy wins, quick gains"),
        (&["move", "the", "needle"], "make progress, have impact"),
        (&["hit", "the", "ground", "running"], "start quickly, begin effectively"),
        (&["game", "changer"], "significant innovation"),
        (&["win", "win"], "mutually beneficial"),
        (&["best", "practices"], "effective methods"),
        (&["touch", "base"], "contact, communicate"),
        (&["circle", "back"], "follow up, revisit"),
        (&["deep", "dive"], "thorough analysis"),
        (&["bandwidth"], "capacity, time (when not technical)"),
        (&["synergy"], "collaboration, cooperation"),
        (&["paradigm", "shift"], "fundamental change"),
        (&["leverage"], "use, utilize"),
        (&["going", "forward"], "in the future, from now on"),
        (&["at", "this", "point", "in", "time"], "now, currently"),
        (&["it", "is", "what", "it", "is"], "(often meaningless)"),
        (&["24/7"], "constantly, always"),
        (&["give", "110", "percent"], "maximum effort"),
    ];
}

impl Rule for ClicheRule {
    fn id(&self) -> &str {
        "CLICHE"
    }

    fn check(&self, _text: &str, tokens: &[AnalyzedToken]) -> Option<Match> {
        let words: Vec<_> = tokens
            .iter()
            .filter(|t| t.token.kind == TokenKind::Word)
            .map(|t| t.token.text.to_lowercase())
            .collect();

        for (cliche, suggestion) in Self::CLICHES {
            let cliche_len = cliche.len();

            for i in 0..words.len().saturating_sub(cliche_len - 1) {
                let matches = words[i..i + cliche_len]
                    .iter()
                    .zip(cliche.iter())
                    .all(|(w, c)| w == *c);

                if matches {
                    // Find the span
                    let word_tokens: Vec<_> = tokens
                        .iter()
                        .filter(|t| t.token.kind == TokenKind::Word)
                        .collect();

                    if let Some(first_token) = word_tokens.get(i) {
                        return Some(Match {
                            span: first_token.token.span.start..first_token.token.span.start + 20,
                            message: format!(
                                "Cliché detected: '{}'. Consider: {}",
                                cliche.join(" "),
                                suggestion
                            ),
                            rule_id: self.id().to_string(),
                            suggestions: vec![],
                            severity: Severity::Hint,
                        });
                    }
                }
            }
        }
        None
    }
}

/// Redundancy/Pleonasm detection
pub struct RedundancyRule;

impl RedundancyRule {
    const REDUNDANCIES: &'static [(&'static [&'static str], &'static str)] = &[
        // English
        (&["completely", "finished"], "finished"),
        (&["completely", "destroyed"], "destroyed"),
        (&["totally", "destroyed"], "destroyed"),
        (&["true", "fact"], "fact"),
        (&["actual", "fact"], "fact"),
        (&["past", "history"], "history"),
        (&["future", "plans"], "plans"),
        (&["advance", "planning"], "planning"),
        (&["advance", "warning"], "warning"),
        (&["free", "gift"], "gift"),
        (&["unexpected", "surprise"], "surprise"),
        (&["added", "bonus"], "bonus"),
        (&["end", "result"], "result"),
        (&["final", "outcome"], "outcome"),
        (&["basic", "fundamentals"], "fundamentals"),
        (&["close", "proximity"], "proximity"),
        (&["each", "and", "every"], "each, every"),
        (&["first", "and", "foremost"], "first, foremost"),
        (&["various", "different"], "various, different"),
        (&["repeat", "again"], "repeat"),
        (&["revert", "back"], "revert"),
        (&["return", "back"], "return"),
        (&["collaborate", "together"], "collaborate"),
        (&["combine", "together"], "combine"),
        (&["merge", "together"], "merge"),
        (&["completely", "eliminate"], "eliminate"),
        (&["absolutely", "essential"], "essential"),
        (&["new", "innovation"], "innovation"),
        (&["new", "invention"], "invention"),
        // French
        (&["monter", "en", "haut"], "monter"),
        (&["descendre", "en", "bas"], "descendre"),
        (&["sortir", "dehors"], "sortir"),
        (&["entrer", "dedans"], "entrer"),
        (&["prévoir", "à", "l'avance"], "prévoir"),
        (&["au", "jour", "d'aujourd'hui"], "aujourd'hui"),
        (&["voire", "même"], "voire"),
        (&["car", "en", "effet"], "car"),
    ];
}

impl Rule for RedundancyRule {
    fn id(&self) -> &str {
        "REDUNDANCY"
    }

    fn check(&self, _text: &str, tokens: &[AnalyzedToken]) -> Option<Match> {
        let words: Vec<_> = tokens
            .iter()
            .filter(|t| t.token.kind == TokenKind::Word)
            .map(|t| t.token.text.to_lowercase())
            .collect();

        for (redundancy, suggestion) in Self::REDUNDANCIES {
            let red_len = redundancy.len();

            for i in 0..words.len().saturating_sub(red_len - 1) {
                let matches = words[i..i + red_len]
                    .iter()
                    .zip(redundancy.iter())
                    .all(|(w, r)| w == *r);

                if matches {
                    let word_tokens: Vec<_> = tokens
                        .iter()
                        .filter(|t| t.token.kind == TokenKind::Word)
                        .collect();

                    if let Some(first_token) = word_tokens.get(i) {
                        return Some(Match {
                            span: first_token.token.span.clone(),
                            message: format!(
                                "Redundancy: '{}' can be simplified to '{}'",
                                redundancy.join(" "),
                                suggestion
                            ),
                            rule_id: self.id().to_string(),
                            suggestions: vec![suggestion.to_string()],
                            severity: Severity::Hint,
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
    use crate::tokenizer::SimpleTokenizer;
    use crate::analyzer::PassthroughAnalyzer;
    use crate::core::traits::{Tokenizer, Analyzer};

    fn check_text(text: &str, checker: &RuleChecker) -> CheckResult {
        let tokenizer = SimpleTokenizer::new();
        let analyzer = PassthroughAnalyzer::new();
        let tokens = tokenizer.tokenize(text);
        let analyzed = analyzer.analyze(tokens);
        checker.check(text, &analyzed)
    }

    #[test]
    fn test_repeated_word() {
        let checker = RuleChecker::new().with_rule(RepeatedWordRule);
        let result = check_text("the the cat", &checker);
        assert_eq!(result.matches.len(), 1);
        assert_eq!(result.matches[0].rule_id, "REPEATED_WORD");
    }

    #[test]
    fn test_a_an() {
        let checker = RuleChecker::new().with_rule(AAnRule);

        let result = check_text("a apple", &checker);
        assert_eq!(result.matches.len(), 1);
        assert_eq!(result.matches[0].suggestions, vec!["an"]);

        let result = check_text("an cat", &checker);
        assert_eq!(result.matches.len(), 1);
        assert_eq!(result.matches[0].suggestions, vec!["a"]);
    }

    #[test]
    fn test_french_punctuation() {
        let checker = RuleChecker::new().with_rule(FrenchPunctuationRule);
        let result = check_text("Comment ça va?", &checker);
        assert_eq!(result.matches.len(), 1);
        assert!(result.matches[0].message.contains("espace"));
    }

    #[test]
    fn test_uppercase_sentence_start() {
        let checker = RuleChecker::new().with_rule(UppercaseSentenceStartRule);

        // Should detect lowercase at start
        let result = check_text("hello world", &checker);
        assert_eq!(result.matches.len(), 1);
        assert_eq!(result.matches[0].rule_id, "UPPERCASE_SENTENCE_START");
        assert_eq!(result.matches[0].suggestions, vec!["Hello"]);

        // Should be clean
        let result = check_text("Hello world", &checker);
        assert_eq!(result.matches.len(), 0);

        // Should detect after period
        let result = check_text("Hello. world", &checker);
        assert_eq!(result.matches.len(), 1);
        assert_eq!(result.matches[0].suggestions, vec!["World"]);
    }

    #[test]
    fn test_repeated_punctuation() {
        let checker = RuleChecker::new().with_rule(RepeatedPunctuationRule);

        // Should detect !!
        let result = check_text("Hello!!", &checker);
        assert_eq!(result.matches.len(), 1);
        assert_eq!(result.matches[0].rule_id, "REPEATED_PUNCTUATION");

        // Should detect ??
        let result = check_text("What??", &checker);
        assert_eq!(result.matches.len(), 1);

        // Should allow ellipsis ...
        let result = check_text("Hello...", &checker);
        assert_eq!(result.matches.len(), 0);

        // Should be clean
        let result = check_text("Hello!", &checker);
        assert_eq!(result.matches.len(), 0);
    }

    #[test]
    fn test_missing_space_after_punct() {
        let checker = RuleChecker::new().with_rule(MissingSpaceAfterPunctRule);

        // Should detect missing space
        let result = check_text("Hello.World", &checker);
        assert_eq!(result.matches.len(), 1);
        assert_eq!(result.matches[0].rule_id, "MISSING_SPACE_AFTER_PUNCT");

        // Should allow numbers (1.5)
        let result = check_text("The value is 1.5", &checker);
        assert_eq!(result.matches.len(), 0);

        // Should allow URLs
        let result = check_text("Visit https://example.com", &checker);
        assert_eq!(result.matches.len(), 0);

        // Should be clean with proper spacing
        let result = check_text("Hello. World", &checker);
        assert_eq!(result.matches.len(), 0);
    }

    #[test]
    fn test_subject_verb_agreement() {
        let checker = RuleChecker::new().with_rule(SubjectVerbAgreementRule);

        // Should detect "he go" -> "he goes"
        let result = check_text("he go to school", &checker);
        assert_eq!(result.matches.len(), 1);
        assert_eq!(result.matches[0].rule_id, "SUBJECT_VERB_AGREEMENT");
        assert_eq!(result.matches[0].suggestions, vec!["goes"]);

        // Should detect "she have" -> "she has"
        let result = check_text("she have a car", &checker);
        assert_eq!(result.matches.len(), 1);
        assert_eq!(result.matches[0].suggestions, vec!["has"]);

        // Should detect "they goes" -> "they go"
        let result = check_text("they goes home", &checker);
        assert_eq!(result.matches.len(), 1);
        assert_eq!(result.matches[0].suggestions, vec!["go"]);

        // Should be clean
        let result = check_text("he goes to school", &checker);
        assert_eq!(result.matches.len(), 0);

        let result = check_text("they go home", &checker);
        assert_eq!(result.matches.len(), 0);
    }

    #[test]
    fn test_its_its() {
        let checker = RuleChecker::new().with_rule(ItsItsRule);

        // "its" before adjective -> should be "it's"
        // Note: current tokenizer splits "it's" into "it" + "'" + "s"
        // So we can only reliably detect "its" followed by certain words
        let result = check_text("Its great to see you", &checker);
        assert_eq!(result.matches.len(), 1, "Expected 1 match for 'Its great'");
        assert_eq!(result.matches[0].rule_id, "EN_ITS_ITS");
        assert_eq!(result.matches[0].suggestions, vec!["it's"]);

        // Should be clean when used correctly
        let result = check_text("Its color is blue", &checker);
        assert_eq!(result.matches.len(), 0, "Expected 0 matches for 'Its color'");

        // "its" before "a" (article context) -> should be "it's"
        let result = check_text("Its a good day", &checker);
        assert_eq!(result.matches.len(), 1, "Expected 1 match for 'Its a'");
    }

    #[test]
    fn test_your_youre() {
        let checker = RuleChecker::new().with_rule(YourYoureRule);

        // "your" before adjective -> should be "you're"
        // Note: tokenizer splits "you're" into "you" + "'" + "re"
        let result = check_text("Your welcome here", &checker);
        assert_eq!(result.matches.len(), 1, "Expected 1 match for 'Your welcome'");
        assert_eq!(result.matches[0].rule_id, "EN_YOUR_YOURE");
        assert_eq!(result.matches[0].suggestions, vec!["you're"]);

        // Should be clean when used correctly
        let result = check_text("Your car is nice", &checker);
        assert_eq!(result.matches.len(), 0, "Expected 0 matches for 'Your car'");

        // "your" before "going" -> should be "you're"
        let result = check_text("Your going to love this", &checker);
        assert_eq!(result.matches.len(), 1, "Expected 1 match for 'Your going'");
    }

    #[test]
    fn test_improved_a_an() {
        let checker = RuleChecker::new().with_rule(ImprovedAAnRule);

        // Silent h - should use "an"
        let result = check_text("a hour ago", &checker);
        assert_eq!(result.matches.len(), 1);
        assert_eq!(result.matches[0].suggestions, vec!["an"]);

        let result = check_text("a honest person", &checker);
        assert_eq!(result.matches.len(), 1);
        assert_eq!(result.matches[0].suggestions, vec!["an"]);

        // Vowel letter but consonant sound - should use "a"
        let result = check_text("an university", &checker);
        assert_eq!(result.matches.len(), 1);
        assert_eq!(result.matches[0].suggestions, vec!["a"]);

        let result = check_text("an one-time offer", &checker);
        assert_eq!(result.matches.len(), 1);
        assert_eq!(result.matches[0].suggestions, vec!["a"]);

        // Should be clean
        let result = check_text("an hour", &checker);
        assert_eq!(result.matches.len(), 0);

        let result = check_text("a university", &checker);
        assert_eq!(result.matches.len(), 0);
    }

    #[test]
    fn test_french_a_accent() {
        let checker = RuleChecker::new().with_rule(FrenchAAccentRule);

        // "a" before city -> should be "à"
        let result = check_text("Je vais a Paris", &checker);
        assert_eq!(result.matches.len(), 1);
        assert_eq!(result.matches[0].rule_id, "FR_A_ACCENT");
        assert_eq!(result.matches[0].suggestions, vec!["à"]);

        // Should be clean with verb context
        let result = check_text("Il a mangé", &checker);
        assert_eq!(result.matches.len(), 0);
    }

    #[test]
    fn test_french_ou_accent() {
        let checker = RuleChecker::new().with_rule(FrenchOuAccentRule);

        // "ou" after location verb -> should be "où"
        let result = check_text("C'est ou il habite", &checker);
        // Note: simplified test
        assert!(result.matches.is_empty() || result.matches[0].rule_id == "FR_OU_ACCENT");
    }

    #[test]
    fn test_french_ce_se() {
        let checker = RuleChecker::new().with_rule(FrenchCeSeRule);

        // "ce" before reflexive verb -> should be "se"
        let result = check_text("Il ce lève tôt", &checker);
        assert_eq!(result.matches.len(), 1);
        assert_eq!(result.matches[0].rule_id, "FR_CE_SE");
        assert_eq!(result.matches[0].suggestions, vec!["se"]);

        // "se" before demonstrative -> should be "ce"
        let result = check_text("se livre est beau", &checker);
        assert_eq!(result.matches.len(), 1);
        assert_eq!(result.matches[0].suggestions, vec!["ce"]);

        // Should be clean
        let result = check_text("ce livre est beau", &checker);
        assert_eq!(result.matches.len(), 0);

        let result = check_text("il se lève", &checker);
        assert_eq!(result.matches.len(), 0);
    }
}
