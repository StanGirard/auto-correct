//! CLI simple pour tester le grammar checker

use grammar_rs::prelude::*;
use std::io::{self, BufRead, Write};

fn main() {
    // Dictionnaire minimal pour la démo
    let words = include_str!("../../data/words_en.txt")
        .lines()
        .filter(|l| !l.is_empty());

    let pipeline = Pipeline::new(
        SimpleTokenizer::new(),
        PassthroughAnalyzer::new(),
    )
    .with_checker(SpellChecker::new().with_words(words))
    .with_checker(RuleChecker::new().with_english_rules());

    println!("Grammar Checker CLI (Ctrl+D to exit)");
    println!("=====================================");
    println!();

    let stdin = io::stdin();
    let mut stdout = io::stdout();

    loop {
        print!("> ");
        stdout.flush().unwrap();

        let mut line = String::new();
        match stdin.lock().read_line(&mut line) {
            Ok(0) => break, // EOF
            Ok(_) => {
                let text = line.trim();
                if text.is_empty() {
                    continue;
                }

                let result = pipeline.check_text(text);

                if result.matches.is_empty() {
                    println!("✓ Aucune erreur détectée\n");
                } else {
                    println!();
                    for m in &result.matches {
                        let severity = match m.severity {
                            Severity::Error => "ERROR",
                            Severity::Warning => "WARN ",
                            Severity::Hint => "HINT ",
                        };

                        let excerpt = &text[m.span.clone()];
                        println!(
                            "[{}] {} ({}..{}): '{}'",
                            severity,
                            m.rule_id,
                            m.span.start,
                            m.span.end,
                            excerpt
                        );
                        println!("       {}", m.message);

                        if !m.suggestions.is_empty() {
                            println!(
                                "       Suggestions: {}",
                                m.suggestions.join(", ")
                            );
                        }
                        println!();
                    }
                }
            }
            Err(e) => {
                eprintln!("Erreur: {}", e);
                break;
            }
        }
    }

    println!("\nBye!");
}
