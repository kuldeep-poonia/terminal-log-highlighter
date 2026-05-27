/// Severity levels for danger signals.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    Info,
    Warn,
    Error,
    Critical,
}

/// A compact result of a match.
/// Carries only a numeric pattern ID and the byte offset;
/// the consumer can look up the severity and display string from a database.
#[derive(Debug, Clone, Copy)]
pub struct MatchResult {
    pub pattern_id: u32,
    pub offset: usize,
}

/// A single pattern definition.
#[derive(Debug, Clone)]
pub struct PatternDef {
    pub pattern: String,
    pub severity: Severity,
}

/// Owns all pattern metadata and assigns stable IDs.
#[derive(Debug, Clone)]
pub struct PatternDatabase {
    entries: Vec<PatternEntry>,
}

#[derive(Debug, Clone)]
struct PatternEntry {
    pattern: String,
    severity: Severity,
}

impl PatternDatabase {
    /// Build a database from a slice of definitions.
    /// Pattern IDs are assigned consecutively, starting at 0.
    pub fn from_defs(defs: &[PatternDef]) -> Self {
        let entries = defs
            .iter()
            .map(|d| PatternEntry {
                pattern: d.pattern.clone(),
                severity: d.severity,
            })
            .collect();
        Self { entries }
    }

    /// Provide a built‑in set of danger patterns.
    pub fn defaults() -> Self {
        let defs = vec![
            PatternDef { pattern: "error".into(), severity: Severity::Error },
            PatternDef { pattern: "warn".into(), severity: Severity::Warn },
            PatternDef { pattern: "timeout".into(), severity: Severity::Critical },
        ];
        Self::from_defs(&defs)
    }

    /// Number of patterns stored.
    #[inline]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Retrieve the severity for a pattern ID.
    #[inline]
    pub fn severity(&self, id: u32) -> Severity {
        self.entries[id as usize].severity
    }

    /// Retrieve the pattern string for a pattern ID.
    #[inline]
    pub fn pattern(&self, id: u32) -> &str {
        &self.entries[id as usize].pattern
    }
}