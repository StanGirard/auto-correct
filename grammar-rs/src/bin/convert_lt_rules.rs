//! Convert LanguageTool confusion_sets.txt to Rust code
//!
//! Usage: cargo run --bin convert-lt-rules -- <path-to-confusion_sets.txt> <output.rs>

use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

#[derive(Debug)]
struct ConfusionPair {
    word1: String,
    word2: String,
    factor: u64,
    bidirectional: bool,
}

fn parse_confusion_sets(path: &Path) -> Vec<ConfusionPair> {
    let file = fs::File::open(path).expect("Failed to open file");
    let reader = BufReader::new(file);
    let mut pairs = Vec::new();

    for line in reader.lines() {
        let line = line.unwrap();
        let line = line.trim();

        // Skip comments and empty lines
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // Remove inline comments
        let line = if let Some(idx) = line.find('#') {
            line[..idx].trim()
        } else {
            line
        };

        // Parse format: word1 -> word2; factor   OR   word1; word2; factor
        let bidirectional = !line.contains("->");

        let parts: Vec<&str> = if bidirectional {
            line.split(';').map(|s| s.trim()).collect()
        } else {
            // word1 -> word2; factor
            let arrow_split: Vec<&str> = line.split("->").collect();
            if arrow_split.len() != 2 {
                continue;
            }
            let word1 = arrow_split[0].trim();
            let rest: Vec<&str> = arrow_split[1].split(';').map(|s| s.trim()).collect();
            if rest.len() < 2 {
                continue;
            }
            vec![word1, rest[0], rest[1]]
        };

        if parts.len() < 3 {
            continue;
        }

        let word1 = parts[0].to_lowercase();
        let word2 = parts[1].to_lowercase();
        let factor: u64 = parts[2].parse().unwrap_or(1000);

        // Skip pairs with empty words
        if word1.is_empty() || word2.is_empty() {
            continue;
        }

        pairs.push(ConfusionPair {
            word1,
            word2,
            factor,
            bidirectional,
        });
    }

    pairs
}

fn generate_rust_code(pairs: &[ConfusionPair], lang: &str) -> String {
    let mut output = String::new();

    output.push_str(&format!(
        "//! Auto-generated confusion pairs for {} from LanguageTool\n",
        lang.to_uppercase()
    ));
    output.push_str("//! DO NOT EDIT - regenerate with convert-lt-rules\n");
    output.push_str("//!\n");
    output.push_str("//! Source: LanguageTool confusion_sets.txt\n");
    output.push_str("//! License: LGPL 2.1+\n\n");

    // Group pairs by word1
    let mut grouped: HashMap<String, Vec<(String, u64)>> = HashMap::new();

    for pair in pairs {
        grouped
            .entry(pair.word1.clone())
            .or_default()
            .push((pair.word2.clone(), pair.factor));

        // For bidirectional pairs, add reverse
        if pair.bidirectional {
            grouped
                .entry(pair.word2.clone())
                .or_default()
                .push((pair.word1.clone(), pair.factor));
        }
    }

    // Sort for deterministic output
    let mut keys: Vec<_> = grouped.keys().cloned().collect();
    keys.sort();

    // Generate as a static slice of tuples for zero-cost lookup
    output.push_str(&format!(
        "/// Confusion pairs for {}\n",
        lang.to_uppercase()
    ));
    output.push_str(&format!(
        "/// Total unique words: {}\n",
        keys.len()
    ));
    output.push_str(&format!(
        "/// Format: (source_word, &[(target_word, factor)])\n"
    ));
    output.push_str(&format!(
        "pub const {}_CONFUSION_DATA: &[(&str, &[(&str, u64)])] = &[\n",
        lang.to_uppercase()
    ));

    for key in &keys {
        let targets = grouped.get(key).unwrap();
        output.push_str(&format!("    (\"{}\", &[", key));
        for (i, (target, factor)) in targets.iter().enumerate() {
            if i > 0 {
                output.push_str(", ");
            }
            output.push_str(&format!("(\"{}\", {})", target, factor));
        }
        output.push_str("]),\n");
    }

    output.push_str("];\n\n");

    // Generate a binary search lookup function
    output.push_str(&format!(
        "/// Check if word might be confused with another word (binary search)\n"
    ));
    output.push_str(&format!(
        "pub fn get_{}_confusions(word: &str) -> Option<&'static [(&'static str, u64)]> {{\n",
        lang.to_lowercase()
    ));
    output.push_str(&format!(
        "    {}_CONFUSION_DATA\n",
        lang.to_uppercase()
    ));
    output.push_str("        .binary_search_by_key(&word, |(w, _)| *w)\n");
    output.push_str("        .ok()\n");
    output.push_str(&format!(
        "        .map(|idx| {}_CONFUSION_DATA[idx].1)\n",
        lang.to_uppercase()
    ));
    output.push_str("}\n\n");

    // Generate stats as a regular comment (not doc comment)
    let total_pairs: usize = grouped.values().map(|v| v.len()).sum();
    output.push_str(&format!(
        "// Statistics: {} unique words, {} total confusion mappings\n",
        keys.len(),
        total_pairs
    ));

    output
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        eprintln!("Usage: {} <confusion_sets.txt> <output.rs> [lang]", args[0]);
        eprintln!("  lang: en (default) or fr");
        std::process::exit(1);
    }

    let input_path = Path::new(&args[1]);
    let output_path = Path::new(&args[2]);
    let lang = args.get(3).map(|s| s.as_str()).unwrap_or("en");

    println!("Parsing {}...", input_path.display());
    let pairs = parse_confusion_sets(input_path);
    println!("Found {} confusion pairs", pairs.len());

    println!("Generating Rust code...");
    let code = generate_rust_code(&pairs, lang);

    let mut file = fs::File::create(output_path).expect("Failed to create output file");
    file.write_all(code.as_bytes()).expect("Failed to write output");

    println!("Written to {}", output_path.display());
}
