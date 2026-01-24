//! Auto-generated numbers word list for FR
//! Total words: 79
//!
//! French compound number words (21-99)
//! Note: "et" is used for numbers ending in 1 (except 81 and 91)

/// NUMBERS words for FR (sorted for binary search)
/// Total words: 79
pub const FR_NUMBERS: &[&str] = &[
    // 21-29 (vingt)
    "vingt-et-un\tvingt-et-un\tCD",
    "vingt-deux\tvingt-deux\tCD",
    "vingt-trois\tvingt-trois\tCD",
    "vingt-quatre\tvingt-quatre\tCD",
    "vingt-cinq\tvingt-cinq\tCD",
    "vingt-six\tvingt-six\tCD",
    "vingt-sept\tvingt-sept\tCD",
    "vingt-huit\tvingt-huit\tCD",
    "vingt-neuf\tvingt-neuf\tCD",
    // 31-39 (trente)
    "trente-et-un\ttrente-et-un\tCD",
    "trente-deux\ttrente-deux\tCD",
    "trente-trois\ttrente-trois\tCD",
    "trente-quatre\ttrente-quatre\tCD",
    "trente-cinq\ttrente-cinq\tCD",
    "trente-six\ttrente-six\tCD",
    "trente-sept\ttrente-sept\tCD",
    "trente-huit\ttrente-huit\tCD",
    "trente-neuf\ttrente-neuf\tCD",
    // 41-49 (quarante)
    "quarante-et-un\tquarante-et-un\tCD",
    "quarante-deux\tquarante-deux\tCD",
    "quarante-trois\tquarante-trois\tCD",
    "quarante-quatre\tquarante-quatre\tCD",
    "quarante-cinq\tquarante-cinq\tCD",
    "quarante-six\tquarante-six\tCD",
    "quarante-sept\tquarante-sept\tCD",
    "quarante-huit\tquarante-huit\tCD",
    "quarante-neuf\tquarante-neuf\tCD",
    // 51-59 (cinquante)
    "cinquante-et-un\tcinquante-et-un\tCD",
    "cinquante-deux\tcinquante-deux\tCD",
    "cinquante-trois\tcinquante-trois\tCD",
    "cinquante-quatre\tcinquante-quatre\tCD",
    "cinquante-cinq\tcinquante-cinq\tCD",
    "cinquante-six\tcinquante-six\tCD",
    "cinquante-sept\tcinquante-sept\tCD",
    "cinquante-huit\tcinquante-huit\tCD",
    "cinquante-neuf\tcinquante-neuf\tCD",
    // 61-69 (soixante)
    "soixante-et-un\tsoixante-et-un\tCD",
    "soixante-deux\tsoixante-deux\tCD",
    "soixante-trois\tsoixante-trois\tCD",
    "soixante-quatre\tsoixante-quatre\tCD",
    "soixante-cinq\tsoixante-cinq\tCD",
    "soixante-six\tsoixante-six\tCD",
    "soixante-sept\tsoixante-sept\tCD",
    "soixante-huit\tsoixante-huit\tCD",
    "soixante-neuf\tsoixante-neuf\tCD",
    // 70-79 (soixante-dix)
    "soixante-dix\tsoixante-dix\tCD",
    "soixante-et-onze\tsoixante-et-onze\tCD",
    "soixante-douze\tsoixante-douze\tCD",
    "soixante-treize\tsoixante-treize\tCD",
    "soixante-quatorze\tsoixante-quatorze\tCD",
    "soixante-quinze\tsoixante-quinze\tCD",
    "soixante-seize\tsoixante-seize\tCD",
    "soixante-dix-sept\tsoixante-dix-sept\tCD",
    "soixante-dix-huit\tsoixante-dix-huit\tCD",
    "soixante-dix-neuf\tsoixante-dix-neuf\tCD",
    // 80-89 (quatre-vingts)
    "quatre-vingts\tquatre-vingts\tCD",
    "quatre-vingt-un\tquatre-vingt-un\tCD",
    "quatre-vingt-deux\tquatre-vingt-deux\tCD",
    "quatre-vingt-trois\tquatre-vingt-trois\tCD",
    "quatre-vingt-quatre\tquatre-vingt-quatre\tCD",
    "quatre-vingt-cinq\tquatre-vingt-cinq\tCD",
    "quatre-vingt-six\tquatre-vingt-six\tCD",
    "quatre-vingt-sept\tquatre-vingt-sept\tCD",
    "quatre-vingt-huit\tquatre-vingt-huit\tCD",
    "quatre-vingt-neuf\tquatre-vingt-neuf\tCD",
    // 90-99 (quatre-vingt-dix)
    "quatre-vingt-dix\tquatre-vingt-dix\tCD",
    "quatre-vingt-onze\tquatre-vingt-onze\tCD",
    "quatre-vingt-douze\tquatre-vingt-douze\tCD",
    "quatre-vingt-treize\tquatre-vingt-treize\tCD",
    "quatre-vingt-quatorze\tquatre-vingt-quatorze\tCD",
    "quatre-vingt-quinze\tquatre-vingt-quinze\tCD",
    "quatre-vingt-seize\tquatre-vingt-seize\tCD",
    "quatre-vingt-dix-sept\tquatre-vingt-dix-sept\tCD",
    "quatre-vingt-dix-huit\tquatre-vingt-dix-huit\tCD",
    "quatre-vingt-dix-neuf\tquatre-vingt-dix-neuf\tCD",
];

/// Check if a word is in the numbers list
pub fn is_fr_numbers(word: &str) -> bool {
    FR_NUMBERS.iter().any(|entry| {
        entry.split('\t').next() == Some(word)
    })
}
