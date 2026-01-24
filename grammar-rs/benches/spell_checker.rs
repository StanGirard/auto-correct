//! Benchmarks for the spell checker module
//! This is the most critical module to benchmark as Levenshtein is O(m*n)
//!
//! Compares HashSet vs FST dictionary backends.

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use grammar_rs::prelude::*;
use std::sync::Arc;

const WORDS: &str = include_str!("../data/words_en.txt");
const SHORT_TEXT: &str = include_str!("../data/bench_texts/short.txt");

fn load_dictionary(size: usize) -> SpellChecker {
    let words: Vec<&str> = WORDS.lines().filter(|l| !l.is_empty()).take(size).collect();
    SpellChecker::new().with_words(words)
}

fn load_fst_dictionary() -> Arc<FstDictionary> {
    // Try to load FST file, fall back to building from wordlist
    let fst_path = "data/dictionaries/en_US.fst";
    let txt_path = "data/dictionaries/en_US.txt";

    if std::path::Path::new(fst_path).exists() {
        Arc::new(FstDictionary::from_fst(fst_path).expect("Failed to load FST"))
    } else if std::path::Path::new(txt_path).exists() {
        Arc::new(FstDictionary::from_wordlist(txt_path).expect("Failed to load wordlist"))
    } else {
        // Fallback to small built-in dictionary
        let words: Vec<&str> = WORDS.lines().filter(|l| !l.is_empty()).collect();
        Arc::new(FstDictionary::from_iter(words).expect("Failed to build dictionary"))
    }
}

fn bench_spell_check_clean(c: &mut Criterion) {
    let checker = load_dictionary(200);
    let tokenizer = SimpleTokenizer::new();
    let analyzer = PassthroughAnalyzer::new();

    let tokens = tokenizer.tokenize(SHORT_TEXT);
    let analyzed = analyzer.analyze(tokens);

    c.bench_function("spell_checker/clean_text", |b| {
        b.iter(|| checker.check(black_box(SHORT_TEXT), black_box(&analyzed)))
    });
}

fn bench_spell_check_with_errors(c: &mut Criterion) {
    let checker = load_dictionary(200);
    let tokenizer = SimpleTokenizer::new();
    let analyzer = PassthroughAnalyzer::new();

    // Text with intentional misspellings
    let text_with_errors = "The quik brown fox jumps ovr the layz dog. \
        Ths sentense contans meny erors that ned to be detekted.";

    let tokens = tokenizer.tokenize(text_with_errors);
    let analyzed = analyzer.analyze(tokens);

    c.bench_function("spell_checker/with_errors", |b| {
        b.iter(|| checker.check(black_box(text_with_errors), black_box(&analyzed)))
    });
}

fn bench_suggest_single_word(c: &mut Criterion) {
    let checker = load_dictionary(200);

    let mut group = c.benchmark_group("spell_checker/suggest");

    // Short word (5 chars)
    group.bench_function("short_word", |b| {
        b.iter(|| checker.suggest(black_box("helo"), 3))
    });

    // Medium word (8 chars)
    group.bench_function("medium_word", |b| {
        b.iter(|| checker.suggest(black_box("languege"), 3))
    });

    // Long word (12 chars)
    group.bench_function("long_word", |b| {
        b.iter(|| checker.suggest(black_box("understandig"), 3))
    });

    group.finish();
}

fn bench_dictionary_scaling(c: &mut Criterion) {
    let tokenizer = SimpleTokenizer::new();
    let analyzer = PassthroughAnalyzer::new();

    let text_with_errors = "The quik brown fox";
    let tokens = tokenizer.tokenize(text_with_errors);
    let analyzed = analyzer.analyze(tokens);

    let mut group = c.benchmark_group("spell_checker/dict_size");

    for size in [50, 100, 200] {
        let checker = load_dictionary(size);

        group.bench_with_input(
            BenchmarkId::new("check", size),
            &analyzed,
            |b, analyzed| {
                b.iter(|| checker.check(black_box(text_with_errors), black_box(analyzed)))
            },
        );
    }

    group.finish();
}

fn bench_levenshtein_scaling(c: &mut Criterion) {
    // Test raw suggestion performance with different dictionary sizes
    let mut group = c.benchmark_group("spell_checker/levenshtein");

    for size in [50, 100, 200] {
        let checker = load_dictionary(size);

        group.bench_with_input(
            BenchmarkId::new("suggest", size),
            &size,
            |b, _| {
                b.iter(|| checker.suggest(black_box("tset"), 5))
            },
        );
    }

    group.finish();
}

// === FST vs HashSet Comparison ===

fn bench_fst_lookup(c: &mut Criterion) {
    let dict = load_fst_dictionary();
    let checker = SpellChecker::with_shared_fst(dict.clone());

    let tokenizer = SimpleTokenizer::new();
    let analyzer = PassthroughAnalyzer::new();

    let text = "The quick brown fox jumps over the lazy dog";
    let tokens = tokenizer.tokenize(text);
    let analyzed = analyzer.analyze(tokens);

    let mut group = c.benchmark_group("spell_checker/fst");

    // Benchmark dictionary lookup
    group.bench_function("contains", |b| {
        b.iter(|| {
            dict.contains(black_box("hello"));
            dict.contains(black_box("world"));
            dict.contains(black_box("xyzzy")); // not in dictionary
        })
    });

    // Benchmark full spell check with FST
    group.bench_function("check_clean_text", |b| {
        b.iter(|| checker.check(black_box(text), black_box(&analyzed)))
    });

    // Benchmark with misspellings
    let bad_text = "The quik brown fox jumps ovr the layz dog";
    let bad_tokens = tokenizer.tokenize(bad_text);
    let bad_analyzed = analyzer.analyze(bad_tokens);

    group.bench_function("check_with_errors", |b| {
        b.iter(|| checker.check(black_box(bad_text), black_box(&bad_analyzed)))
    });

    group.finish();
}

fn bench_fst_vs_hashset(c: &mut Criterion) {
    let fst_dict = load_fst_dictionary();
    let fst_checker = SpellChecker::with_shared_fst(fst_dict.clone());

    // Create HashSet checker with same words (subset for fair comparison)
    let words: Vec<&str> = WORDS.lines().filter(|l| !l.is_empty()).take(1000).collect();
    let hash_checker = SpellChecker::new().with_words(words);

    let tokenizer = SimpleTokenizer::new();
    let analyzer = PassthroughAnalyzer::new();

    let text = "The quick brown fox jumps over the lazy dog";
    let tokens = tokenizer.tokenize(text);
    let analyzed = analyzer.analyze(tokens);

    let mut group = c.benchmark_group("spell_checker/fst_vs_hash");

    group.bench_function("fst_370k_words", |b| {
        b.iter(|| fst_checker.check(black_box(text), black_box(&analyzed)))
    });

    group.bench_function("hash_1k_words", |b| {
        b.iter(|| hash_checker.check(black_box(text), black_box(&analyzed)))
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_spell_check_clean,
    bench_spell_check_with_errors,
    bench_suggest_single_word,
    bench_dictionary_scaling,
    bench_levenshtein_scaling,
    bench_fst_lookup,
    bench_fst_vs_hashset,
);

criterion_main!(benches);
