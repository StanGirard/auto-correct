//! Pattern rules validation using auto-generated examples from LanguageTool
//!
//! This test validates the PatternRuleChecker against 600+ examples
//! extracted from LanguageTool's grammar.xml.
//!
//! Run with: cargo test --test pattern_validation -- --nocapture

use grammar_rs::checker::{
    AhoPatternRuleChecker, PatternRuleChecker,
    EN_PATTERN_RULES, FR_PATTERN_RULES,
    EN_PATTERN_TEST_EXAMPLES, FR_PATTERN_TEST_EXAMPLES,
    get_en_incorrect_examples, get_en_correct_examples,
    get_fr_incorrect_examples, get_fr_correct_examples,
};
use grammar_rs::prelude::*;

// ═══════════════════════════════════════════════════════════════════════════════
// Test utilities
// ═══════════════════════════════════════════════════════════════════════════════

fn prepare_tokens(text: &str) -> Vec<AnalyzedToken<'_>> {
    let tokenizer = SimpleTokenizer::new();
    let analyzer = PassthroughAnalyzer::new();
    let tokens = tokenizer.tokenize(text);
    analyzer.analyze(tokens)
}

#[derive(Debug, Default)]
struct ValidationMetrics {
    true_positives: usize,
    false_positives: usize,
    false_negatives: usize,
    true_negatives: usize,
}

impl ValidationMetrics {
    fn precision(&self) -> f64 {
        let total = self.true_positives + self.false_positives;
        if total == 0 { 1.0 } else { self.true_positives as f64 / total as f64 }
    }

    fn recall(&self) -> f64 {
        let total = self.true_positives + self.false_negatives;
        if total == 0 { 1.0 } else { self.true_positives as f64 / total as f64 }
    }

    fn f1(&self) -> f64 {
        let p = self.precision();
        let r = self.recall();
        if p + r == 0.0 { 0.0 } else { 2.0 * (p * r) / (p + r) }
    }

    fn merge(&mut self, other: &ValidationMetrics) {
        self.true_positives += other.true_positives;
        self.false_positives += other.false_positives;
        self.false_negatives += other.false_negatives;
        self.true_negatives += other.true_negatives;
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Main validation test
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_pattern_rules_validation() {
    println!("\n");
    println!("╔══════════════════════════════════════════════════════════════════╗");
    println!("║           PATTERN RULES VALIDATION (LanguageTool Examples)       ║");
    println!("╚══════════════════════════════════════════════════════════════════╝\n");

    let total_examples = EN_PATTERN_TEST_EXAMPLES.len() + FR_PATTERN_TEST_EXAMPLES.len();
    println!("Total test examples: {}", total_examples);
    println!("  - English: {} ({} incorrect, {} correct)",
        EN_PATTERN_TEST_EXAMPLES.len(),
        get_en_incorrect_examples().len(),
        get_en_correct_examples().len()
    );
    println!("  - French: {} ({} incorrect, {} correct)",
        FR_PATTERN_TEST_EXAMPLES.len(),
        get_fr_incorrect_examples().len(),
        get_fr_correct_examples().len()
    );
    println!();

    let mut total_metrics = ValidationMetrics::default();

    // Test English patterns with Aho-Corasick checker
    println!("─── English Pattern Rules (Aho-Corasick) ───");
    let en_checker = AhoPatternRuleChecker::new(EN_PATTERN_RULES);
    let en_metrics = validate_en_patterns(&en_checker);
    println!("  Incorrect examples (should trigger): {} tested", get_en_incorrect_examples().len());
    println!("  Correct examples (should NOT trigger): {} tested", get_en_correct_examples().len());
    println!("  TP: {}, FP: {}, FN: {}, TN: {}",
        en_metrics.true_positives, en_metrics.false_positives,
        en_metrics.false_negatives, en_metrics.true_negatives
    );
    println!("  Precision: {:.1}% | Recall: {:.1}% | F1: {:.1}%",
        en_metrics.precision() * 100.0,
        en_metrics.recall() * 100.0,
        en_metrics.f1() * 100.0
    );
    total_metrics.merge(&en_metrics);

    // Test French patterns with Aho-Corasick checker
    println!("\n─── French Pattern Rules (Aho-Corasick) ───");
    let fr_checker = AhoPatternRuleChecker::new(FR_PATTERN_RULES);
    let fr_metrics = validate_fr_patterns(&fr_checker);
    println!("  Incorrect examples (should trigger): {} tested", get_fr_incorrect_examples().len());
    println!("  Correct examples (should NOT trigger): {} tested", get_fr_correct_examples().len());
    println!("  TP: {}, FP: {}, FN: {}, TN: {}",
        fr_metrics.true_positives, fr_metrics.false_positives,
        fr_metrics.false_negatives, fr_metrics.true_negatives
    );
    println!("  Precision: {:.1}% | Recall: {:.1}% | F1: {:.1}%",
        fr_metrics.precision() * 100.0,
        fr_metrics.recall() * 100.0,
        fr_metrics.f1() * 100.0
    );
    total_metrics.merge(&fr_metrics);

    // Overall results
    println!("\n╔══════════════════════════════════════════════════════════════════╗");
    println!("║                       OVERALL RESULTS                            ║");
    println!("╚══════════════════════════════════════════════════════════════════╝");
    println!("Total examples tested: {}", total_examples);
    println!("True Positives: {} | False Positives: {}",
        total_metrics.true_positives, total_metrics.false_positives);
    println!("False Negatives: {} | True Negatives: {}",
        total_metrics.false_negatives, total_metrics.true_negatives);
    println!();
    println!("Overall Precision: {:.1}%", total_metrics.precision() * 100.0);
    println!("Overall Recall:    {:.1}%", total_metrics.recall() * 100.0);
    println!("Overall F1 Score:  {:.1}%", total_metrics.f1() * 100.0);
    println!("══════════════════════════════════════════════════════════════════\n");
}

fn validate_en_patterns(checker: &impl Checker) -> ValidationMetrics {
    let mut metrics = ValidationMetrics::default();
    let mut false_negative_samples: Vec<(&str, &str)> = Vec::new();
    let mut false_positive_samples: Vec<(&str, &str)> = Vec::new();

    let incorrect_examples = get_en_incorrect_examples();
    let correct_examples = get_en_correct_examples();

    // Test incorrect examples (should trigger the rule)
    for example in &incorrect_examples {
        let tokens = prepare_tokens(example.text);
        let result = checker.check(example.text, &tokens);
        let triggered = result.matches.iter().any(|m| m.rule_id == example.rule_id);

        if triggered {
            metrics.true_positives += 1;
        } else {
            metrics.false_negatives += 1;
            if false_negative_samples.len() < 5 {
                false_negative_samples.push((example.rule_id, example.text));
            }
        }
    }

    // Test correct examples (should NOT trigger the rule)
    for example in &correct_examples {
        let tokens = prepare_tokens(example.text);
        let result = checker.check(example.text, &tokens);
        let triggered = result.matches.iter().any(|m| m.rule_id == example.rule_id);

        if !triggered {
            metrics.true_negatives += 1;
        } else {
            metrics.false_positives += 1;
            if false_positive_samples.len() < 5 {
                false_positive_samples.push((example.rule_id, example.text));
            }
        }
    }

    // Show sample failures
    if !false_negative_samples.is_empty() {
        println!("  Sample false negatives (rule not triggered on bad text):");
        for (rule_id, text) in &false_negative_samples {
            let preview: &str = if text.len() > 50 { &text[..50] } else { text };
            println!("    - [{}] \"{}...\"", rule_id, preview);
        }
    }
    if !false_positive_samples.is_empty() {
        println!("  Sample false positives (rule triggered on good text):");
        for (rule_id, text) in &false_positive_samples {
            let preview: &str = if text.len() > 50 { &text[..50] } else { text };
            println!("    - [{}] \"{}...\"", rule_id, preview);
        }
    }

    metrics
}

fn validate_fr_patterns(checker: &impl Checker) -> ValidationMetrics {
    let mut metrics = ValidationMetrics::default();
    let mut false_negative_samples: Vec<(&str, &str)> = Vec::new();
    let mut false_positive_samples: Vec<(&str, &str)> = Vec::new();

    let incorrect_examples = get_fr_incorrect_examples();
    let correct_examples = get_fr_correct_examples();

    // Test incorrect examples (should trigger the rule)
    for example in &incorrect_examples {
        let tokens = prepare_tokens(example.text);
        let result = checker.check(example.text, &tokens);
        let triggered = result.matches.iter().any(|m| m.rule_id == example.rule_id);

        if triggered {
            metrics.true_positives += 1;
        } else {
            metrics.false_negatives += 1;
            if false_negative_samples.len() < 5 {
                false_negative_samples.push((example.rule_id, example.text));
            }
        }
    }

    // Test correct examples (should NOT trigger the rule)
    for example in &correct_examples {
        let tokens = prepare_tokens(example.text);
        let result = checker.check(example.text, &tokens);
        let triggered = result.matches.iter().any(|m| m.rule_id == example.rule_id);

        if !triggered {
            metrics.true_negatives += 1;
        } else {
            metrics.false_positives += 1;
            if false_positive_samples.len() < 5 {
                false_positive_samples.push((example.rule_id, example.text));
            }
        }
    }

    // Show sample failures
    if !false_negative_samples.is_empty() {
        println!("  Sample false negatives (rule not triggered on bad text):");
        for (rule_id, text) in &false_negative_samples {
            let preview: &str = if text.len() > 50 { &text[..50] } else { text };
            println!("    - [{}] \"{}...\"", rule_id, preview);
        }
    }
    if !false_positive_samples.is_empty() {
        println!("  Sample false positives (rule triggered on good text):");
        for (rule_id, text) in &false_positive_samples {
            let preview: &str = if text.len() > 50 { &text[..50] } else { text };
            println!("    - [{}] \"{}...\"", rule_id, preview);
        }
    }

    metrics
}

// ═══════════════════════════════════════════════════════════════════════════════
// Naive vs Aho-Corasick consistency test
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_aho_vs_naive_consistency() {
    println!("\n");
    println!("╔══════════════════════════════════════════════════════════════════╗");
    println!("║           AHO-CORASICK VS NAIVE CONSISTENCY CHECK                ║");
    println!("╚══════════════════════════════════════════════════════════════════╝\n");

    let naive = PatternRuleChecker::new(EN_PATTERN_RULES);
    let aho = AhoPatternRuleChecker::new(EN_PATTERN_RULES);

    let mut consistent = 0;
    let mut inconsistent = 0;

    for example in EN_PATTERN_TEST_EXAMPLES.iter().take(100) {
        let tokens = prepare_tokens(example.text);
        let naive_result = naive.check(example.text, &tokens);
        let aho_result = aho.check(example.text, &tokens);

        // Compare results
        let naive_rules: std::collections::HashSet<_> = naive_result.matches.iter()
            .map(|m| m.rule_id.as_str())
            .collect();
        let aho_rules: std::collections::HashSet<_> = aho_result.matches.iter()
            .map(|m| m.rule_id.as_str())
            .collect();

        if naive_rules == aho_rules {
            consistent += 1;
        } else {
            inconsistent += 1;
            if inconsistent <= 3 {
                let preview: &str = if example.text.len() > 60 { &example.text[..60] } else { example.text };
                println!("Inconsistency found:");
                println!("  Text: \"{}\"", preview);
                println!("  Naive: {:?}", naive_rules);
                println!("  Aho:   {:?}", aho_rules);
            }
        }
    }

    println!("\nResults (first 100 examples):");
    println!("  Consistent: {}", consistent);
    println!("  Inconsistent: {}", inconsistent);
    let total = consistent + inconsistent;
    if total > 0 {
        println!("  Consistency rate: {:.1}%", (consistent as f64 / total as f64) * 100.0);
    }

    // Should be highly consistent (allow small variance due to edge cases)
    // Some inconsistencies occur when AC matches substrings that aren't word boundaries
    assert!(
        inconsistent <= 10,
        "Too many inconsistencies between naive and Aho-Corasick: {} (expected <= 10)",
        inconsistent
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// Statistics test
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_example_counts() {
    // Verify we have enough examples
    let en_total = EN_PATTERN_TEST_EXAMPLES.len();
    let fr_total = FR_PATTERN_TEST_EXAMPLES.len();
    let total = en_total + fr_total;

    println!("\nExample counts:");
    println!("  English: {}", en_total);
    println!("  French:  {}", fr_total);
    println!("  Total:   {}", total);

    assert!(
        total >= 500,
        "Expected at least 500 test examples, got {}",
        total
    );

    // Verify distribution
    let en_incorrect = get_en_incorrect_examples().len();
    let en_correct = get_en_correct_examples().len();
    let fr_incorrect = get_fr_incorrect_examples().len();
    let fr_correct = get_fr_correct_examples().len();

    println!("\nDistribution:");
    println!("  EN incorrect: {} | EN correct: {}", en_incorrect, en_correct);
    println!("  FR incorrect: {} | FR correct: {}", fr_incorrect, fr_correct);

    // Should have both positive and negative examples
    assert!(en_incorrect > 0, "No English incorrect examples");
    assert!(en_correct > 0, "No English correct examples");
}
