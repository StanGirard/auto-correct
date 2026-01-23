//! Build FST dictionary from wordlist
//!
//! Usage: cargo run --bin build_dict -- <input.txt> <output.fst>

use grammar_rs::dictionary::FstDictionary;
use std::env;
use std::time::Instant;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        eprintln!("Usage: {} <input.txt> <output.fst>", args[0]);
        eprintln!("  input.txt  - Wordlist file (one word per line, sorted)");
        eprintln!("  output.fst - Output FST file");
        std::process::exit(1);
    }

    let input_path = &args[1];
    let output_path = &args[2];

    println!("Loading wordlist from: {}", input_path);
    let start = Instant::now();

    let dict = match FstDictionary::from_wordlist(input_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Error loading wordlist: {}", e);
            std::process::exit(1);
        }
    };

    let load_time = start.elapsed();
    println!("Loaded {} words in {:.2?}", dict.len(), load_time);

    println!("Saving FST to: {}", output_path);
    let start = Instant::now();

    if let Err(e) = dict.save_fst(output_path) {
        eprintln!("Error saving FST: {}", e);
        std::process::exit(1);
    }

    let save_time = start.elapsed();
    println!("Saved in {:.2?}", save_time);
    println!("Memory usage: {} KB", dict.memory_usage() / 1024);
}
