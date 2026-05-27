use super::schema::{Config, Filter};
use crate::matcher::classifier::Severity;

/// Default danger patterns (used when no config file is provided).
pub fn default_config() -> Config {
    Config {
        filters: vec![
            Filter {
                pattern: "error".into(),
                severity: Severity::Error,
            },
            Filter {
                pattern: "warn".into(),
                severity: Severity::Warn,
            },
            Filter {
                pattern: "timeout".into(),
                severity: Severity::Critical,
            },
        ],
    }
}