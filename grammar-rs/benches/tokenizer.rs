//! Benchmarks for the tokenizer module

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use grammar_rs::prelude::*;

const SHORT_TEXT: &str = include_str!("../data/bench_texts/short.txt");
const MEDIUM_TEXT: &str = include_str!("../data/bench_texts/medium.txt");
const LONG_TEXT: &str = include_str!("../data/bench_texts/long.txt");

const FRENCH_TEXT: &str = "Le français est une langue très riche. \
    Elle possède des accents comme é, è, ê, ë et des caractères spéciaux. \
    La ponctuation française utilise des espaces avant les signes : ; ! ? \
    C'est une particularité intéressante de cette belle langue.";

fn bench_tokenize_sizes(c: &mut Criterion) {
    let tokenizer = SimpleTokenizer::new();

    let mut group = c.benchmark_group("tokenizer/size");

    group.bench_with_input(
        BenchmarkId::new("short", SHORT_TEXT.len()),
        SHORT_TEXT,
        |b, text| b.iter(|| tokenizer.tokenize(black_box(text))),
    );

    group.bench_with_input(
        BenchmarkId::new("medium", MEDIUM_TEXT.len()),
        MEDIUM_TEXT,
        |b, text| b.iter(|| tokenizer.tokenize(black_box(text))),
    );

    group.bench_with_input(
        BenchmarkId::new("long", LONG_TEXT.len()),
        LONG_TEXT,
        |b, text| b.iter(|| tokenizer.tokenize(black_box(text))),
    );

    group.finish();
}

fn bench_tokenize_unicode(c: &mut Criterion) {
    let tokenizer = SimpleTokenizer::new();

    c.bench_function("tokenizer/french_unicode", |b| {
        b.iter(|| tokenizer.tokenize(black_box(FRENCH_TEXT)))
    });
}

fn bench_tokenize_repeated(c: &mut Criterion) {
    let tokenizer = SimpleTokenizer::new();

    // Test with text that has many token boundaries
    let punctuation_heavy = "Hello, world! How are you? I'm fine, thanks. \
        Yes... No? Maybe! Okay: done; finished.".repeat(100);

    c.bench_function("tokenizer/punctuation_heavy", |b| {
        b.iter(|| tokenizer.tokenize(black_box(&punctuation_heavy)))
    });
}

criterion_group!(
    benches,
    bench_tokenize_sizes,
    bench_tokenize_unicode,
    bench_tokenize_repeated,
);

criterion_main!(benches);
