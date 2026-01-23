//! Benchmarks for the rule checker module

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use grammar_rs::prelude::*;
use grammar_rs::checker::{
    AAnRule, AhoPatternRuleChecker, CoherencyChecker, DiacriticsChecker,
    DoubleSpaceRule, FrenchPunctuationRule,
    PatternRuleChecker, RepeatedWordRule, StyleChecker,
    EN_PATTERN_RULES, FR_PATTERN_RULES,
};
use grammar_rs::lang_detect::LanguageDetector;
use std::time::Duration;

const SHORT_TEXT: &str = include_str!("../data/bench_texts/short.txt");

fn prepare_tokens(text: &str) -> Vec<AnalyzedToken<'_>> {
    let tokenizer = SimpleTokenizer::new();
    let analyzer = PassthroughAnalyzer::new();
    let tokens = tokenizer.tokenize(text);
    analyzer.analyze(tokens)
}

/// Quick benchmarks for basic rules
fn bench_basic_rules(c: &mut Criterion) {
    let mut group = c.benchmark_group("rules");
    group.warm_up_time(Duration::from_millis(100));
    group.measurement_time(Duration::from_millis(500));
    group.sample_size(50);

    // Double space
    let checker = RuleChecker::new().with_rule(DoubleSpaceRule);
    let analyzed = prepare_tokens(SHORT_TEXT);
    group.bench_function("double_space", |b| {
        b.iter(|| checker.check(black_box(SHORT_TEXT), black_box(&analyzed)))
    });

    // Repeated word
    let checker = RuleChecker::new().with_rule(RepeatedWordRule);
    let text = "The the quick brown fox fox jumps over over the lazy dog dog.";
    let analyzed = prepare_tokens(text);
    group.bench_function("repeated_word", |b| {
        b.iter(|| checker.check(black_box(text), black_box(&analyzed)))
    });

    // A/An rule
    let checker = RuleChecker::new().with_rule(AAnRule);
    let text = "I saw a elephant and an cat. She has a umbrella and an book.";
    let analyzed = prepare_tokens(text);
    group.bench_function("a_an", |b| {
        b.iter(|| checker.check(black_box(text), black_box(&analyzed)))
    });

    // All English rules
    let checker = RuleChecker::new().with_english_rules();
    let analyzed = prepare_tokens(SHORT_TEXT);
    group.bench_function("all_english", |b| {
        b.iter(|| checker.check(black_box(SHORT_TEXT), black_box(&analyzed)))
    });

    // French punctuation
    let checker = RuleChecker::new().with_rule(FrenchPunctuationRule);
    let text = "Comment allez-vous? Je vais bien! Vraiment? Oui: certainement; merci.";
    let analyzed = prepare_tokens(text);
    group.bench_function("french_punct", |b| {
        b.iter(|| checker.check(black_box(text), black_box(&analyzed)))
    });

    group.finish();
}

/// Pattern checker benchmarks - naive vs Aho-Corasick comparison
fn bench_pattern_checkers(c: &mut Criterion) {
    let mut group = c.benchmark_group("pattern");
    group.warm_up_time(Duration::from_millis(100));
    group.measurement_time(Duration::from_millis(500));
    group.sample_size(50);

    // English patterns - short text (no matches)
    let naive_en = PatternRuleChecker::new(EN_PATTERN_RULES);
    let aho_en = AhoPatternRuleChecker::new(EN_PATTERN_RULES);
    let analyzed = prepare_tokens(SHORT_TEXT);

    group.bench_with_input(BenchmarkId::new("naive", "en_short"), &(), |b, _| {
        b.iter(|| naive_en.check(black_box(SHORT_TEXT), black_box(&analyzed)))
    });
    group.bench_with_input(BenchmarkId::new("aho", "en_short"), &(), |b, _| {
        b.iter(|| aho_en.check(black_box(SHORT_TEXT), black_box(&analyzed)))
    });

    // English patterns - with actual matches
    let text_matches = "I could of done that better. She should of been here. \
        They would of helped us. The bay area is nice. \
        You have to tow the line here. It was a breathe of fresh air.";
    let analyzed_matches = prepare_tokens(text_matches);

    group.bench_with_input(BenchmarkId::new("naive", "en_matches"), &(), |b, _| {
        b.iter(|| naive_en.check(black_box(text_matches), black_box(&analyzed_matches)))
    });
    group.bench_with_input(BenchmarkId::new("aho", "en_matches"), &(), |b, _| {
        b.iter(|| aho_en.check(black_box(text_matches), black_box(&analyzed_matches)))
    });

    // French patterns
    let naive_fr = PatternRuleChecker::new(FR_PATTERN_RULES);
    let aho_fr = AhoPatternRuleChecker::new(FR_PATTERN_RULES);
    let text_fr = "Bonjour! Comment allez-vous? Je vais bien, merci. \
        C'est une belle journée: le soleil brille; les oiseaux chantent.";
    let analyzed_fr = prepare_tokens(text_fr);

    group.bench_with_input(BenchmarkId::new("naive", "fr"), &(), |b, _| {
        b.iter(|| naive_fr.check(black_box(text_fr), black_box(&analyzed_fr)))
    });
    group.bench_with_input(BenchmarkId::new("aho", "fr"), &(), |b, _| {
        b.iter(|| aho_fr.check(black_box(text_fr), black_box(&analyzed_fr)))
    });

    group.finish();
}

/// Style checker benchmarks (wordiness & redundancy with Aho-Corasick)
fn bench_style_checker(c: &mut Criterion) {
    let mut group = c.benchmark_group("style");
    group.warm_up_time(Duration::from_millis(100));
    group.measurement_time(Duration::from_millis(500));
    group.sample_size(50);

    let checker = StyleChecker::new();

    // Clean text (no matches)
    let analyzed = prepare_tokens(SHORT_TEXT);
    group.bench_function("clean_text", |b| {
        b.iter(|| checker.check(black_box(SHORT_TEXT), black_box(&analyzed)))
    });

    // Text with style issues (wordiness + redundancy)
    let wordy_text = "In order to succeed, you need a number of things. \
        It is absolutely essential that you work hard. \
        At this point in time, the added bonus is significant. \
        Due to the fact that we worked together, we achieved success. \
        The reason is because we planned ahead in advance.";
    let analyzed_wordy = prepare_tokens(wordy_text);
    group.bench_function("wordy_text", |b| {
        b.iter(|| checker.check(black_box(wordy_text), black_box(&analyzed_wordy)))
    });

    // Longer document with mixed content
    let mixed_text = "The meeting will start at 12 noon sharp. \
        In order to prepare, please review the basic fundamentals. \
        It is absolutely essential that everyone attends. \
        We need to plan ahead for the future. \
        The added bonus is that lunch will be provided free of charge. \
        At this point in time, we have a number of items to discuss. \
        Due to the fact that resources are limited, we must prioritize. \
        The end result should be a consensus of opinion.";
    let analyzed_mixed = prepare_tokens(mixed_text);
    group.bench_function("mixed_document", |b| {
        b.iter(|| checker.check(black_box(mixed_text), black_box(&analyzed_mixed)))
    });

    group.finish();
}

/// Coherency checker benchmarks (UK/US spelling consistency)
fn bench_coherency_checker(c: &mut Criterion) {
    let mut group = c.benchmark_group("coherency");
    group.warm_up_time(Duration::from_millis(100));
    group.measurement_time(Duration::from_millis(500));
    group.sample_size(50);

    let checker = CoherencyChecker::new();

    // Clean text (no coherency words)
    let analyzed = prepare_tokens(SHORT_TEXT);
    group.bench_function("clean_text", |b| {
        b.iter(|| checker.check(black_box(SHORT_TEXT), black_box(&analyzed)))
    });

    // Text with consistent spelling (UK)
    let uk_text = "We need to organise the programme. The organisation will analyse the data. \
        We should prioritise and emphasise quality. The behaviour is normalised.";
    let analyzed_uk = prepare_tokens(uk_text);
    group.bench_function("consistent_uk", |b| {
        b.iter(|| checker.check(black_box(uk_text), black_box(&analyzed_uk)))
    });

    // Text with mixed spelling (should detect inconsistencies)
    let mixed_text = "We need to organize the programme. The organization will analyse the data. \
        We should prioritize and emphasise quality. The behavior is normalised.";
    let analyzed_mixed = prepare_tokens(mixed_text);
    group.bench_function("mixed_spelling", |b| {
        b.iter(|| checker.check(black_box(mixed_text), black_box(&analyzed_mixed)))
    });

    group.finish();
}

/// Benchmark construction time (one-time cost)
fn bench_construction(c: &mut Criterion) {
    let mut group = c.benchmark_group("construction");
    group.warm_up_time(Duration::from_millis(50));
    group.measurement_time(Duration::from_millis(300));
    group.sample_size(30);

    group.bench_function("naive_en", |b| {
        b.iter(|| PatternRuleChecker::new(black_box(EN_PATTERN_RULES)))
    });
    group.bench_function("aho_en", |b| {
        b.iter(|| AhoPatternRuleChecker::new(black_box(EN_PATTERN_RULES)))
    });
    group.bench_function("style_checker", |b| {
        b.iter(|| StyleChecker::new())
    });
    group.bench_function("coherency_checker", |b| {
        b.iter(|| CoherencyChecker::new())
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_basic_rules,
    bench_pattern_checkers,
    bench_style_checker,
    bench_coherency_checker,
    bench_construction,
);

criterion_main!(benches);
