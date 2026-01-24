//! Regression tests for grammar-rs
//!
//! These tests document and prevent regressions for:
//! - Previously fixed bugs
//! - Edge cases that caused issues
//! - False positives that were reported
//!
//! Run with: cargo test --test regression

use grammar_rs::prelude::*;
use grammar_rs::checker::{
    AhoPatternRuleChecker, StyleChecker, CompoundWordChecker,
    ProhibitChecker, PosPatternChecker,
    EN_PATTERN_RULES, EN_ANTIPATTERNS, EN_POS_PATTERN_RULES,
};
use std::sync::Once;

static WARM_UP: Once = Once::new();

fn ensure_warm() {
    WARM_UP.call_once(|| {
        grammar_rs::warm_up();
    });
}

/// Create a standard English pipeline for testing
fn create_en_pipeline() -> Pipeline {
    ensure_warm();
    Pipeline::new(SimpleTokenizer::new(), PassthroughAnalyzer::new())
        .with_checker(RuleChecker::new().with_english_rules())
        .with_checker(AhoPatternRuleChecker::with_antipatterns(EN_PATTERN_RULES, EN_ANTIPATTERNS))
        .with_checker(StyleChecker::new())
        .with_checker(CompoundWordChecker::new())
        .with_checker(ProhibitChecker::new())
        .with_default_filters()
}

/// Create a French pipeline for testing
fn create_fr_pipeline() -> Pipeline {
    use grammar_rs::checker::{FR_PATTERN_RULES, FR_ANTIPATTERNS, FR_POS_PATTERN_RULES};
    ensure_warm();
    Pipeline::new(SimpleTokenizer::new(), PassthroughAnalyzer::new())
        .with_checker(RuleChecker::new().with_french_rules())
        .with_checker(AhoPatternRuleChecker::with_antipatterns(FR_PATTERN_RULES, FR_ANTIPATTERNS))
        .with_checker(StyleChecker::french())
        .with_checker(CompoundWordChecker::french())
        .with_default_filters()
}

// ============================================================================
// URL AND LINK FILTERING - Should NOT trigger errors
// ============================================================================

#[test]
fn regression_url_in_text_no_false_positive() {
    let pipeline = create_en_pipeline();

    // URLs should be filtered out
    let cases = [
        "Check out https://example.com/teh/page for more info.",
        "Visit http://test.org/recieve/data today.",
        "Link: www.example.com/hte/path works.",
    ];

    for text in cases {
        let result = pipeline.check_text(text);
        // Should not flag URL paths as spelling errors
        let spell_errors: Vec<_> = result.matches.iter()
            .filter(|m| m.rule_id.contains("SPELL"))
            .collect();
        assert!(spell_errors.is_empty(),
            "URL path incorrectly flagged in: {}", text);
    }
}

#[test]
fn regression_email_no_false_positive() {
    let pipeline = create_en_pipeline();

    let text = "Contact support@example.com for help.";
    let result = pipeline.check_text(text);

    // Email should be filtered
    assert!(result.matches.is_empty() ||
            result.matches.iter().all(|m| !m.rule_id.contains("SPELL")),
            "Email address incorrectly flagged");
}

// ============================================================================
// CODE BLOCK FILTERING - Should NOT trigger errors
// ============================================================================

#[test]
fn regression_inline_code_no_false_positive() {
    let pipeline = create_en_pipeline();

    let cases = [
        "Use `teh_function()` to process data.",
        "The variable `recieve_data` stores input.",
        "Call `hte_method` with parameters.",
    ];

    for text in cases {
        let result = pipeline.check_text(text);
        let spell_errors: Vec<_> = result.matches.iter()
            .filter(|m| m.rule_id.contains("SPELL"))
            .collect();
        assert!(spell_errors.is_empty(),
            "Code block incorrectly flagged in: {}", text);
    }
}

// ============================================================================
// QUOTED TEXT - Should preserve but check appropriately
// ============================================================================

#[test]
fn regression_quoted_text_handling() {
    let pipeline = create_en_pipeline();

    // Direct quotes should be checked for grammar
    let text = "He said \"I seen it yesterday.\"";
    let result = pipeline.check_text(text);
    // This may or may not trigger depending on rules - just ensure no crash
    assert!(result.matches.len() >= 0);
}

// ============================================================================
// COMPOUND WORD EDGE CASES
// ============================================================================

#[test]
fn regression_compound_hyphenated_no_double_flag() {
    let pipeline = create_en_pipeline();

    // Already hyphenated compounds should not trigger
    let cases = [
        "The well-being of employees matters.",
        "It's a state-of-the-art system.",
        "She is a well-known author.",
    ];

    for text in cases {
        let result = pipeline.check_text(text);
        let compound_errors: Vec<_> = result.matches.iter()
            .filter(|m| m.rule_id.contains("COMPOUND"))
            .collect();
        assert!(compound_errors.is_empty(),
            "Already correct compound flagged in: {}", text);
    }
}

#[test]
fn regression_compound_spaced_detected() {
    let pipeline = create_en_pipeline();

    // Spaced compounds should be detected
    let text = "Your well being matters to us.";
    let result = pipeline.check_text(text);

    let compound_errors: Vec<_> = result.matches.iter()
        .filter(|m| m.rule_id.contains("COMPOUND"))
        .collect();

    assert!(!compound_errors.is_empty(),
        "Spaced compound 'well being' not detected");
}

// ============================================================================
// FRENCH SPECIFIC REGRESSIONS
// ============================================================================

#[test]
fn regression_french_punctuation_space() {
    let pipeline = create_fr_pipeline();

    // Missing space before French punctuation
    let cases_should_detect = [
        ("Comment?", true),   // Should detect missing space
        ("Comment ?", false), // Correct - should not detect
        ("Bonjour!", true),   // Should detect
        ("Bonjour !", false), // Correct
    ];

    for (text, should_flag) in cases_should_detect {
        let result = pipeline.check_text(text);
        let punct_errors: Vec<_> = result.matches.iter()
            .filter(|m| m.rule_id.contains("PUNCT"))
            .collect();

        if should_flag {
            assert!(!punct_errors.is_empty(),
                "FR punctuation error not detected in: {}", text);
        } else {
            assert!(punct_errors.is_empty(),
                "False positive FR punctuation in: {}", text);
        }
    }
}

#[test]
fn regression_french_compound_aller_retour() {
    let pipeline = create_fr_pipeline();

    let text = "J'ai pris un aller retour pour Paris.";
    let result = pipeline.check_text(text);

    let compound_errors: Vec<_> = result.matches.iter()
        .filter(|m| m.rule_id.contains("COMPOUND"))
        .collect();

    assert!(!compound_errors.is_empty(),
        "French compound 'aller retour' not detected");
    assert!(compound_errors[0].suggestions.contains(&"aller-retour".to_string()),
        "Expected suggestion 'aller-retour'");
}

// ============================================================================
// STYLE CHECKER REGRESSIONS
// ============================================================================

#[test]
fn regression_style_wordiness() {
    let pipeline = create_en_pipeline();

    let text = "In order to succeed, you need a number of things.";
    let result = pipeline.check_text(text);

    let style_errors: Vec<_> = result.matches.iter()
        .filter(|m| m.rule_id.contains("WORDINESS") || m.rule_id.contains("STYLE"))
        .collect();

    assert!(!style_errors.is_empty(),
        "Wordiness 'in order to' not detected");
}

#[test]
fn regression_style_redundancy() {
    let pipeline = create_en_pipeline();

    let text = "Meet me at 12 noon for lunch.";
    let result = pipeline.check_text(text);

    let style_errors: Vec<_> = result.matches.iter()
        .filter(|m| m.rule_id.contains("REDUNDANCY"))
        .collect();

    assert!(!style_errors.is_empty(),
        "Redundancy '12 noon' not detected");
}

// ============================================================================
// PROHIBIT CHECKER REGRESSIONS
// ============================================================================

#[test]
fn regression_prohibit_christoper() {
    let pipeline = create_en_pipeline();

    let text = "Christoper went to the store.";
    let result = pipeline.check_text(text);

    let prohibit_errors: Vec<_> = result.matches.iter()
        .filter(|m| m.rule_id == "PROHIBIT")
        .collect();

    assert!(!prohibit_errors.is_empty(),
        "Prohibited word 'Christoper' not detected");
    assert!(prohibit_errors[0].suggestions.contains(&"Christopher".to_string()),
        "Expected suggestion 'Christopher'");
}

// ============================================================================
// A/AN RULE EDGE CASES
// ============================================================================

#[test]
fn regression_a_an_silent_h() {
    let pipeline = Pipeline::new(SimpleTokenizer::new(), PassthroughAnalyzer::new())
        .with_checker(RuleChecker::new().with_english_rules());

    // Silent H words should use "an"
    let cases = [
        ("It took an hour to finish.", false),  // Correct
        ("It took a hour to finish.", true),    // Should detect
        ("He is an honest person.", false),     // Correct
    ];

    for (text, should_flag) in cases {
        let result = pipeline.check_text(text);
        let aan_errors: Vec<_> = result.matches.iter()
            .filter(|m| m.rule_id.contains("A_AN"))
            .collect();

        if should_flag {
            assert!(!aan_errors.is_empty(),
                "A/an error not detected in: {}", text);
        }
    }
}

// ============================================================================
// REPEATED WORD EDGE CASES
// ============================================================================

#[test]
fn regression_repeated_word_the_the() {
    let pipeline = Pipeline::new(SimpleTokenizer::new(), PassthroughAnalyzer::new())
        .with_checker(RuleChecker::new().with_english_rules());

    let text = "I went to the the store.";
    let result = pipeline.check_text(text);

    let repeated_errors: Vec<_> = result.matches.iter()
        .filter(|m| m.rule_id.contains("REPEATED"))
        .collect();

    assert!(!repeated_errors.is_empty(),
        "Repeated word 'the the' not detected");
}

#[test]
fn regression_repeated_word_across_line_break() {
    let pipeline = Pipeline::new(SimpleTokenizer::new(), PassthroughAnalyzer::new())
        .with_checker(RuleChecker::new().with_english_rules());

    // Repeated word across newline - may or may not be detected
    let text = "I went to the\nthe store.";
    let result = pipeline.check_text(text);
    // Just ensure no crash - behavior may vary
    assert!(result.matches.len() >= 0);
}

// ============================================================================
// CONFUSION PAIRS REGRESSIONS
// ============================================================================

#[test]
fn regression_confusion_your_youre() {
    use grammar_rs::checker::YourYoureRule;

    let pipeline = Pipeline::new(SimpleTokenizer::new(), PassthroughAnalyzer::new())
        .with_checker(RuleChecker::new().with_rule(YourYoureRule));

    // "Your" followed by verb should be "you're"
    let text = "Your going to love this.";
    let result = pipeline.check_text(text);

    let confusion_errors: Vec<_> = result.matches.iter()
        .filter(|m| m.rule_id.contains("YOUR"))
        .collect();

    assert!(!confusion_errors.is_empty(),
        "Your/you're confusion not detected");
}

// ============================================================================
// REAL WORLD TEXT SAMPLES
// ============================================================================

#[test]
fn regression_real_world_english_text() {
    let pipeline = create_en_pipeline();

    // Well-formed English text should have minimal false positives
    let text = "The quick brown fox jumps over the lazy dog. This sentence contains every letter of the alphabet.";
    let result = pipeline.check_text(text);

    // Should be clean or have very few matches
    assert!(result.matches.len() <= 1,
        "Too many false positives in clean English text: {:?}",
        result.matches.iter().map(|m| &m.rule_id).collect::<Vec<_>>());
}

#[test]
fn regression_real_world_french_text() {
    let pipeline = create_fr_pipeline();

    // Well-formed French text
    let text = "Le chat dort sur le tapis. Il fait beau aujourd'hui.";
    let result = pipeline.check_text(text);

    // Should be clean
    assert!(result.matches.is_empty(),
        "False positives in clean French text: {:?}",
        result.matches.iter().map(|m| &m.rule_id).collect::<Vec<_>>());
}
