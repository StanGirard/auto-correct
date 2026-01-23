//! Auto-generated confusion pairs for FR from LanguageTool
//! Synced: 2026-01-23T11:27:33.299176979+00:00
//! DO NOT EDIT MANUALLY - Run `cargo run --bin sync-lt` to update
//!
//! Source: LanguageTool confusion_sets.txt
//! License: LGPL 2.1+

/// Confusion pairs for FR
/// Total unique words: 53
/// Format: (source_word, &[(target_word, factor)])
pub const FR_CONFUSION_DATA: &[(&str, &[(&str, u64)])] = &[
    ("a", &[("à", 1000)]),
    ("ai", &[("ait", 100)]),
    ("ait", &[("ai", 100)]),
    ("bon", &[("bond", 100)]),
    ("bond", &[("bon", 10000)]),
    ("ce", &[("se", 100)]),
    ("ces", &[("ses", 1000)]),
    ("cou", &[("coup", 100)]),
    ("coup", &[("cou", 100)]),
    ("cour", &[("cours", 100)]),
    ("cours", &[("cour", 100)]),
    ("dans", &[("dent", 10000)]),
    ("don", &[("donc", 1000)]),
    ("donc", &[("don", 1000)]),
    ("est", &[("et", 1000)]),
    ("et", &[("est", 1000)]),
    ("foi", &[("fois", 1000), ("foie", 100)]),
    ("foie", &[("foi", 100)]),
    ("fois", &[("foi", 1000)]),
    ("mer", &[("mère", 1000)]),
    ("moi", &[("mois", 1000)]),
    ("mois", &[("moi", 10000)]),
    ("mère", &[("mer", 1000)]),
    ("notre", &[("nôtre", 10000000)]),
    ("on", &[("ont", 1000)]),
    ("ont", &[("on", 1000)]),
    ("ou", &[("où", 1000)]),
    ("pain", &[("pin", 10)]),
    ("peau", &[("pot", 10)]),
    ("pot", &[("peau", 100000)]),
    ("prix", &[("pris", 10)]),
    ("près", &[("prêt", 100)]),
    ("prêt", &[("près", 100)]),
    ("père", &[("paire", 100)]),
    ("se", &[("ce", 100)]),
    ("ses", &[("ces", 1000)]),
    ("soi", &[("soit", 100)]),
    ("soit", &[("soi", 100)]),
    ("son", &[("sont", 1000)]),
    ("sont", &[("son", 1000)]),
    ("tante", &[("tente", 1000)]),
    ("tente", &[("tante", 1000)]),
    ("toi", &[("toit", 10)]),
    ("très", &[("trait", 10000)]),
    ("vain", &[("vin", 1000), ("vingt", 100000)]),
    ("ver", &[("verre", 1000), ("vers", 1000), ("vert", 1000)]),
    ("verre", &[("vers", 1000000), ("vers", 1000000), ("vert", 100)]),
    ("vers", &[("verre", 10000)]),
    ("vert", &[("verre", 10000)]),
    ("vin", &[("vain", 100), ("vingt", 10000)]),
    ("vingt", &[("vain", 10), ("vin", 100)]),
    ("voie", &[("voix", 1000)]),
    ("voix", &[("voie", 1000)]),
];

/// Check if word might be confused with another word (binary search)
pub fn get_fr_confusions(word: &str) -> Option<&'static [(&'static str, u64)]> {
    FR_CONFUSION_DATA
        .binary_search_by_key(&word, |(w, _)| *w)
        .ok()
        .map(|idx| FR_CONFUSION_DATA[idx].1)
}

// Statistics: 53 unique words, 61 total confusion mappings
