//! Probability calculation results
//!
//! Wraps probability values with metadata like coverage and occurrence count.

/// Probability result from N-gram lookup
#[derive(Debug, Clone, Copy)]
pub struct Probability {
    /// The calculated probability value
    pub probability: f64,
    /// Coverage: fraction of N-gram lookups that found data (0.0-1.0)
    /// Higher coverage means more confident result
    pub coverage: f64,
    /// Raw occurrence count from the corpus
    pub occurrence: u64,
}

impl Probability {
    /// Create a new probability result
    pub fn new(probability: f64, coverage: f64, occurrence: u64) -> Self {
        Probability {
            probability,
            coverage,
            occurrence,
        }
    }

    /// Create probability for unknown word
    pub fn unknown() -> Self {
        Probability {
            probability: 1e-10,
            coverage: 0.0,
            occurrence: 0,
        }
    }

    /// Check if this is an unknown word probability
    pub fn is_unknown(&self) -> bool {
        self.occurrence == 0
    }

    /// Get log probability (useful for numerical stability)
    pub fn log_probability(&self) -> f64 {
        self.probability.ln()
    }

    /// Check if coverage meets minimum threshold
    pub fn meets_coverage(&self, min_coverage: f64) -> bool {
        self.coverage >= min_coverage
    }
}

impl Default for Probability {
    fn default() -> Self {
        Self::unknown()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_probability_creation() {
        let p = Probability::new(0.5, 1.0, 100);
        assert_eq!(p.probability, 0.5);
        assert_eq!(p.coverage, 1.0);
        assert_eq!(p.occurrence, 100);
    }

    #[test]
    fn test_unknown_probability() {
        let p = Probability::unknown();
        assert!(p.is_unknown());
        assert!(p.probability < 1e-9);
        assert_eq!(p.coverage, 0.0);
    }

    #[test]
    fn test_coverage_threshold() {
        let p = Probability::new(0.5, 0.75, 100);
        assert!(p.meets_coverage(0.5));
        assert!(p.meets_coverage(0.75));
        assert!(!p.meets_coverage(0.8));
    }
}
