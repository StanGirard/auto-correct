//! Pipeline composable - assemble les étapes
//!
//! C'est ici que la magie opère : tu peux swapper n'importe quelle
//! implémentation sans changer le reste du code.

use super::traits::{Analyzer, Checker, GrammarChecker, Tokenizer};
use super::CheckResult;
use std::sync::Arc;

/// Le pipeline principal - compose les étapes
pub struct Pipeline {
    tokenizer: Arc<dyn Tokenizer>,
    analyzer: Arc<dyn Analyzer>,
    checkers: Vec<Arc<dyn Checker>>,
}

impl Pipeline {
    pub fn new(
        tokenizer: impl Tokenizer + 'static,
        analyzer: impl Analyzer + 'static,
    ) -> Self {
        Self {
            tokenizer: Arc::new(tokenizer),
            analyzer: Arc::new(analyzer),
            checkers: Vec::new(),
        }
    }

    /// Ajoute un checker au pipeline (builder pattern)
    pub fn with_checker(mut self, checker: impl Checker + 'static) -> Self {
        self.checkers.push(Arc::new(checker));
        self
    }

    /// Ajoute plusieurs checkers
    pub fn with_checkers(mut self, checkers: Vec<Arc<dyn Checker>>) -> Self {
        self.checkers.extend(checkers);
        self
    }
}

impl GrammarChecker for Pipeline {
    fn check_text(&self, text: &str) -> CheckResult {
        // Étape 1: Tokenize
        let tokens = self.tokenizer.tokenize(text);

        // Étape 2: Analyze
        let analyzed = self.analyzer.analyze(tokens);

        // Étape 3: Check (tous les checkers en parallèle potentiellement)
        let mut result = CheckResult::new();
        for checker in &self.checkers {
            result.merge(checker.check(text, &analyzed));
        }

        // Nettoyer et trier
        result.sort_and_dedupe();
        result
    }
}

// Version parallèle avec Rayon (pour plus tard)
#[cfg(feature = "parallel")]
impl Pipeline {
    pub fn check_text_parallel(&self, text: &str) -> CheckResult {
        use rayon::prelude::*;

        let tokens = self.tokenizer.tokenize(text);
        let analyzed = self.analyzer.analyze(tokens);

        let results: Vec<CheckResult> = self
            .checkers
            .par_iter()
            .map(|checker| checker.check(text, &analyzed))
            .collect();

        let mut result = CheckResult::new();
        for r in results {
            result.merge(r);
        }
        result.sort_and_dedupe();
        result
    }
}
