use super::schema::{Config, Filter};
use crate::matcher::classifier::Severity;

/// Built-in patterns used when no `.sentinel.toml` is found.
///
/// All patterns are stored **lowercase**.  The processor lowercases each line
/// before matching, so these patterns match any capitalisation variant:
/// `ERROR`, `Error`, `error`, `eRrOr`, etc.
///
/// Patterns are matched via Aho-Corasick substring search, so they fire on
/// any line that *contains* the pattern, not just lines that start with it.
pub fn default_config() -> Config {
    Config {
        filters: vec![
            // ── Critical / Fatal (bold yellow on red + beep) ─────────────
            Filter { pattern: "panic".into(),          severity: Severity::Critical },
            Filter { pattern: "fatal".into(),          severity: Severity::Critical },
            Filter { pattern: "critical".into(),       severity: Severity::Critical },
            Filter { pattern: "segfault".into(),       severity: Severity::Critical },
            Filter { pattern: "sigsegv".into(),        severity: Severity::Critical },
            Filter { pattern: "out of memory".into(),  severity: Severity::Critical },
            Filter { pattern: "oom".into(),            severity: Severity::Critical },
            Filter { pattern: "killed".into(),         severity: Severity::Critical },
            Filter { pattern: "timeout".into(),        severity: Severity::Critical },
            Filter { pattern: "deadlock".into(),       severity: Severity::Critical },

            // ── Errors (bold white on red + beep) ────────────────────────
            Filter { pattern: "error".into(),          severity: Severity::Error },
            Filter { pattern: "err:".into(),           severity: Severity::Error },
            Filter { pattern: "exception".into(),      severity: Severity::Error },
            Filter { pattern: "traceback".into(),      severity: Severity::Error },
            Filter { pattern: "stacktrace".into(),     severity: Severity::Error },
            Filter { pattern: "stack trace".into(),    severity: Severity::Error },
            Filter { pattern: "failed".into(),         severity: Severity::Error },
            Filter { pattern: "failure".into(),        severity: Severity::Error },
            Filter { pattern: "crash".into(),          severity: Severity::Error },
            Filter { pattern: "abort".into(),          severity: Severity::Error },
            Filter { pattern: "denied".into(),         severity: Severity::Error },
            Filter { pattern: "refused".into(),        severity: Severity::Error },
            Filter { pattern: "cannot".into(),         severity: Severity::Error },
            Filter { pattern: "no such".into(),        severity: Severity::Error },
            Filter { pattern: "not found".into(),      severity: Severity::Error },
            Filter { pattern: "undefined".into(),      severity: Severity::Error },
            Filter { pattern: "unhandled".into(),      severity: Severity::Error },
            Filter { pattern: "uncaught".into(),       severity: Severity::Error },

            // ── Warnings (bold yellow, no beep) ──────────────────────────
            Filter { pattern: "warn".into(),           severity: Severity::Warn },
            Filter { pattern: "warning".into(),        severity: Severity::Warn },
            Filter { pattern: "deprecated".into(),     severity: Severity::Warn },
            Filter { pattern: "retrying".into(),       severity: Severity::Warn },
            Filter { pattern: "retry".into(),          severity: Severity::Warn },
            Filter { pattern: "slow".into(),           severity: Severity::Warn },
            Filter { pattern: "high memory".into(),    severity: Severity::Warn },

            // ── Info / Success (bold cyan, no beep) ──────────────────────
            Filter { pattern: "success".into(),        severity: Severity::Info },
            Filter { pattern: "successfully".into(),   severity: Severity::Info },
            Filter { pattern: "started".into(),        severity: Severity::Info },
            Filter { pattern: "running".into(),        severity: Severity::Info },
            Filter { pattern: "ready".into(),          severity: Severity::Info },
            Filter { pattern: "listening".into(),      severity: Severity::Info },
            Filter { pattern: "connected".into(),      severity: Severity::Info },
            Filter { pattern: "built".into(),          severity: Severity::Info },
        ],
    }
}