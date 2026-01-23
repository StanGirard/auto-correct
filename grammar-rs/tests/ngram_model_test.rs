//! Integration tests for N-gram models

use grammar_rs::language_model::CompactNgramModel;
use std::path::Path;

#[test]
#[ignore] // Run with: cargo test --release -- --ignored ngram
fn test_en_ngram_model() {
    let path = Path::new("data/ngrams/en_ngrams.bin");
    if !path.exists() {
        eprintln!("Skipping: EN N-gram model not found at {}", path.display());
        return;
    }

    let model = CompactNgramModel::open(path).expect("Failed to open EN model");
    let stats = model.stats();

    println!("EN Model Stats:");
    println!("  Unigrams: {}", stats.unigram_count);
    println!("  Bigrams: {}", stats.bigram_count);
    println!("  Trigrams: {}", stats.trigram_count);
    println!("  Total tokens: {}", stats.total_tokens);

    // Verify counts are reasonable
    assert!(stats.unigram_count > 10_000_000, "Expected >10M unigrams");
    assert!(stats.bigram_count > 100_000_000, "Expected >100M bigrams");
    assert!(stats.trigram_count > 500_000_000, "Expected >500M trigrams");

    // Test common words
    let the_count = model.get_unigram("the").expect("'the' should exist");
    assert!(the_count > 1_000_000_000, "'the' should have >1B occurrences");

    let their_count = model.get_unigram("their").expect("'their' should exist");
    let there_count = model.get_unigram("there").expect("'there' should exist");
    println!("  'their': {}", their_count);
    println!("  'there': {}", there_count);

    // Test bigrams
    let to_their = model.get_bigram("to", "their");
    let to_there = model.get_bigram("to", "there");
    println!("  'to their': {:?}", to_their);
    println!("  'to there': {:?}", to_there);

    // Test confusion detection
    let ratio = model.compare_words("their", "there", Some("to"), Some("house"));
    println!("  P(their|to_house) / P(there|to_house) = {:.4}", ratio);

    // "to their house" should be more common than "to there house"
    assert!(ratio > 1.0, "Expected 'their' to be more likely in 'to _ house' context");
}

#[test]
#[ignore]
fn test_fr_ngram_model() {
    let path = Path::new("data/ngrams/fr_ngrams.bin");
    if !path.exists() {
        eprintln!("Skipping: FR N-gram model not found at {}", path.display());
        return;
    }

    let model = CompactNgramModel::open(path).expect("Failed to open FR model");
    let stats = model.stats();

    println!("FR Model Stats:");
    println!("  Unigrams: {}", stats.unigram_count);
    println!("  Bigrams: {}", stats.bigram_count);
    println!("  Trigrams: {}", stats.trigram_count);

    // Verify counts
    assert!(stats.unigram_count > 4_000_000, "Expected >4M unigrams");
    assert!(stats.bigram_count > 30_000_000, "Expected >30M bigrams");
    assert!(stats.trigram_count > 100_000_000, "Expected >100M trigrams");

    // Test common French words
    let de_count = model.get_unigram("de").expect("'de' should exist");
    assert!(de_count > 1_000_000_000, "'de' should be very common");

    let le_count = model.get_unigram("le").expect("'le' should exist");
    let la_count = model.get_unigram("la").expect("'la' should exist");
    println!("  'de': {}", de_count);
    println!("  'le': {}", le_count);
    println!("  'la': {}", la_count);
}
