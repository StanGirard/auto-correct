//! Benchmarks for the full pipeline

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use grammar_rs::prelude::*;

const WORDS: &str = include_str!("../data/words_en.txt");
const SHORT_TEXT: &str = include_str!("../data/bench_texts/short.txt");
const MEDIUM_TEXT: &str = include_str!("../data/bench_texts/medium.txt");
const LONG_TEXT: &str = include_str!("../data/bench_texts/long.txt");

fn create_pipeline(dict_size: usize) -> Pipeline {
    let words: Vec<&str> = WORDS.lines().filter(|l| !l.is_empty()).take(dict_size).collect();

    Pipeline::new(
        SimpleTokenizer::new(),
        PassthroughAnalyzer::new(),
    )
    .with_checker(SpellChecker::new().with_words(words))
    .with_checker(RuleChecker::new().with_english_rules())
}

fn bench_pipeline_minimal(c: &mut Criterion) {
    // Pipeline with just tokenizer + analyzer (no checkers)
    let pipeline = Pipeline::new(
        SimpleTokenizer::new(),
        PassthroughAnalyzer::new(),
    );

    c.bench_function("pipeline/minimal", |b| {
        b.iter(|| pipeline.check_text(black_box(SHORT_TEXT)))
    });
}

fn bench_pipeline_spell_only(c: &mut Criterion) {
    let words: Vec<&str> = WORDS.lines().filter(|l| !l.is_empty()).take(200).collect();

    let pipeline = Pipeline::new(
        SimpleTokenizer::new(),
        PassthroughAnalyzer::new(),
    )
    .with_checker(SpellChecker::new().with_words(words));

    c.bench_function("pipeline/spell_only", |b| {
        b.iter(|| pipeline.check_text(black_box(SHORT_TEXT)))
    });
}

fn bench_pipeline_rules_only(c: &mut Criterion) {
    let pipeline = Pipeline::new(
        SimpleTokenizer::new(),
        PassthroughAnalyzer::new(),
    )
    .with_checker(RuleChecker::new().with_english_rules());

    c.bench_function("pipeline/rules_only", |b| {
        b.iter(|| pipeline.check_text(black_box(SHORT_TEXT)))
    });
}

fn bench_pipeline_full(c: &mut Criterion) {
    let pipeline = create_pipeline(200);

    c.bench_function("pipeline/full", |b| {
        b.iter(|| pipeline.check_text(black_box(SHORT_TEXT)))
    });
}

fn bench_pipeline_text_scaling(c: &mut Criterion) {
    let pipeline = create_pipeline(200);

    let mut group = c.benchmark_group("pipeline/text_size");

    group.bench_with_input(
        BenchmarkId::new("short", SHORT_TEXT.len()),
        SHORT_TEXT,
        |b, text| b.iter(|| pipeline.check_text(black_box(text))),
    );

    group.bench_with_input(
        BenchmarkId::new("medium", MEDIUM_TEXT.len()),
        MEDIUM_TEXT,
        |b, text| b.iter(|| pipeline.check_text(black_box(text))),
    );

    group.bench_with_input(
        BenchmarkId::new("long", LONG_TEXT.len()),
        LONG_TEXT,
        |b, text| b.iter(|| pipeline.check_text(black_box(text))),
    );

    group.finish();
}

fn bench_pipeline_dict_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("pipeline/dict_size");

    for size in [50, 100, 200] {
        let pipeline = create_pipeline(size);

        group.bench_with_input(
            BenchmarkId::new("full", size),
            &size,
            |b, _| b.iter(|| pipeline.check_text(black_box(SHORT_TEXT))),
        );
    }

    group.finish();
}

fn bench_pipeline_with_errors(c: &mut Criterion) {
    let pipeline = create_pipeline(200);

    // Text with multiple types of errors
    let text_with_errors = "The the quik brown fox  jumps ovr the layz dog. \
        I saw a elephant in the the zoo. It was a amazng sight!";

    c.bench_function("pipeline/with_errors", |b| {
        b.iter(|| pipeline.check_text(black_box(text_with_errors)))
    });
}

criterion_group!(
    benches,
    bench_pipeline_minimal,
    bench_pipeline_spell_only,
    bench_pipeline_rules_only,
    bench_pipeline_full,
    bench_pipeline_text_scaling,
    bench_pipeline_dict_scaling,
    bench_pipeline_with_errors,
);

criterion_main!(benches);
