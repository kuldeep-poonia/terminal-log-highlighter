/// Severity levels for danger signals.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    Info,
    Warn,
    Error,
    Critical,
}

/// A compact result of a match.
///
/// `pattern_id` is used immediately to look up severity for highlighting.
/// `offset` records where in the line the match occurred; it is retained for
/// future use (e.g. per-word inline highlighting) rather than full-line
/// background colour.
#[derive(Debug, Clone, Copy)]
pub struct MatchResult {
    pub pattern_id: u32,
    /// Byte offset of the first matching byte within the line.
    /// Not currently read by the renderer; kept for forward compatibility.
    #[allow(dead_code)]
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

    /// Returns `true` if no patterns are loaded.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
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