//! POS tag transformation for postag_replace
//!
//! Handles regex-based POS tag transformations like:
//! - Pattern: "(D|J) .*", Replace: "$1 f s" → "D f s" or "J f s"
//! - Pattern: "(J) ([mfe]) ([sp])", Replace: "$1 [ef] p" → "J e p" or "J f p"

use regex::Regex;

/// Transform a POS tag using postag pattern and postag_replace
///
/// # Arguments
/// * `source_pos` - The source POS tag to transform (e.g., "D m s")
/// * `postag_pattern` - Regex pattern to match (e.g., "(D|J) .*")
/// * `postag_replace` - Replacement pattern with backreferences (e.g., "$1 f s")
///
/// # Returns
/// Vector of possible target POS tags after transformation.
/// Multiple results occur when the replacement contains character classes like `[mf]`.
pub fn transform_pos(
    source_pos: &str,
    postag_pattern: &str,
    postag_replace: &str,
) -> Option<Vec<String>> {
    // Compile the pattern
    let re = Regex::new(postag_pattern).ok()?;

    // Match against source POS
    let caps = re.captures(source_pos)?;

    // Apply backreferences ($1, $2, etc.)
    let mut result = postag_replace.to_string();

    // Replace $N with captured groups (in reverse order to handle $10+ correctly)
    for i in (1..=caps.len().saturating_sub(1)).rev() {
        let placeholder = format!("${}", i);
        if let Some(m) = caps.get(i) {
            result = result.replace(&placeholder, m.as_str());
        }
    }

    // Expand character classes [abc] to multiple possibilities
    let expanded = expand_char_classes(&result);

    Some(expanded)
}

/// Expand character classes in POS tag to multiple variants
///
/// Examples:
/// - "D [mf] s" → ["D m s", "D f s"]
/// - "J [me] [sp]" → ["J m s", "J m p", "J e s", "J e p"]
fn expand_char_classes(pos: &str) -> Vec<String> {
    // Find all character classes
    let class_re = Regex::new(r"\[([^\]]+)\]").unwrap();

    let mut results = vec![pos.to_string()];

    // Process each character class
    for caps in class_re.captures_iter(pos) {
        let full_match = caps.get(0).unwrap().as_str();
        let chars: Vec<char> = caps.get(1).unwrap().as_str().chars().collect();

        // Expand each current result
        let mut new_results = Vec::new();
        for current in &results {
            for c in &chars {
                let expanded = current.replacen(full_match, &c.to_string(), 1);
                new_results.push(expanded);
            }
        }
        results = new_results;
    }

    // Handle optional markers like ? (e.g., "sp?" means "sp" or "s" or "p")
    // For simplicity, treat "sp?" as matching both "sp", "s", and "p"
    let mut final_results = Vec::new();
    for r in results {
        if r.contains('?') {
            // Simple handling: remove the ? and keep the result
            // More complex patterns would need full regex expansion
            final_results.push(r.replace('?', ""));
        } else {
            final_results.push(r);
        }
    }

    final_results
}

/// Check if a POS tag matches a pattern (for regex-based POS matching)
pub fn pos_matches_pattern(pos: &str, pattern: &str) -> bool {
    if pattern.is_empty() {
        return true;
    }

    // Try exact match first
    if pos == pattern {
        return true;
    }

    // Try regex match
    if let Ok(re) = Regex::new(&format!("^{}$", pattern)) {
        re.is_match(pos)
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_transform() {
        // "(D|J) .*" with "$1 f s" transforms "D m s" to "D f s"
        let result = transform_pos("D m s", "(D|J) .*", "$1 f s");
        assert!(result.is_some());
        let targets = result.unwrap();
        assert!(targets.contains(&"D f s".to_string()));
    }

    #[test]
    fn test_transform_adjective() {
        // "(J) .*" with "$1 f s" transforms "J m s" to "J f s"
        let result = transform_pos("J m s", "(J) .*", "$1 f s");
        assert!(result.is_some());
        let targets = result.unwrap();
        assert!(targets.contains(&"J f s".to_string()));
    }

    #[test]
    fn test_transform_with_char_class() {
        // "$1 [ef] p" should expand to multiple targets
        let result = transform_pos("J m s", "(J) .*", "$1 [ef] p");
        assert!(result.is_some());
        let targets = result.unwrap();
        assert!(targets.contains(&"J e p".to_string()) || targets.contains(&"J f p".to_string()));
    }

    #[test]
    fn test_multiple_capture_groups() {
        // "(J) ([mfe]) ([sp])" with "$1 $2 p" preserves gender and changes number
        let result = transform_pos("J m s", "(J) ([mfe]) ([sp])", "$1 $2 p");
        assert!(result.is_some());
        let targets = result.unwrap();
        assert!(targets.contains(&"J m p".to_string()));
    }

    #[test]
    fn test_verb_infinitive() {
        // "V .*" with "V inf" transforms any verb form to infinitive
        let result = transform_pos("V ind pres 3 s", "V .*", "V inf");
        assert!(result.is_some());
        let targets = result.unwrap();
        assert!(targets.contains(&"V inf".to_string()));
    }

    #[test]
    fn test_no_match() {
        // Pattern doesn't match source POS
        let result = transform_pos("N m s", "(J) .*", "$1 f s");
        assert!(result.is_none());
    }

    #[test]
    fn test_pos_matches() {
        assert!(pos_matches_pattern("D m s", "D.*"));
        assert!(pos_matches_pattern("J f p", "J [mf] [sp]"));
        assert!(!pos_matches_pattern("N m s", "D.*"));
    }
}
