use crate::matcher::classifier::{PatternDef, Severity};

/// A single filter definition.
#[derive(Debug, Clone)]
pub struct Filter {
    pub pattern: String,
    pub severity: Severity,
}

/// Top‑level configuration.
#[derive(Debug, Clone, Default)]
pub struct Config {
    pub filters: Vec<Filter>,
}

impl Config {
    /// Convert the filter list into a vector of `PatternDef` for the matcher engine.
    pub fn to_pattern_defs(&self) -> Vec<PatternDef> {
        self.filters
            .iter()
            .map(|f| PatternDef {
                pattern: f.pattern.clone(),
                severity: f.severity,
            })
            .collect()
    }
}