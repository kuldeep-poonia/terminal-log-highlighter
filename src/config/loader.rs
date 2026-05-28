use std::env;
use std::fs;
use std::path::PathBuf;

use super::defaults;
use super::schema::{Config, Filter};
use crate::matcher::classifier::Severity;

/// Config file name sentinel looks for automatically.
const CONFIG_FILE: &str = ".sentinel.toml";

/// Load configuration from the first source that is available and parseable:
///
/// 1. `$SENTINEL_CONFIG` – an explicit path to any TOML file.
/// 2. `./.sentinel.toml` – project-local config in the current directory.
/// 3. `~/.sentinel.toml` – user-wide config in the home directory.
/// 4. Built-in defaults (see `config/defaults.rs`).
///
/// The first source that exists *and* is valid is used.  Invalid TOML emits a
/// warning to stderr and the search continues.
pub fn load_config() -> Config {
    // ── 1. Explicit env-var override ─────────────────────────────────────────
    if let Ok(path) = env::var("SENTINEL_CONFIG") {
        match fs::read_to_string(&path) {
            Ok(content) => match parse_config(&content) {
                Ok(config) => return config,
                Err(e) => eprintln!("sentinel: warning: {path}: {e}; falling through"),
            },
            Err(e) => eprintln!("sentinel: warning: cannot read {path}: {e}; falling through"),
        }
    }

    // ── 2. Current working directory ──────────────────────────────────────────
    if let Ok(content) = fs::read_to_string(CONFIG_FILE) {
        if let Ok(config) = parse_config(&content) {
            return config;
        }
    }

    // ── 3. Home directory ─────────────────────────────────────────────────────
    if let Some(home) = home_dir() {
        let path = home.join(CONFIG_FILE);
        if let Ok(content) = fs::read_to_string(&path) {
            if let Ok(config) = parse_config(&content) {
                return config;
            }
        }
    }

    // ── 4. Built-in defaults ──────────────────────────────────────────────────
    defaults::default_config()
}

// ─────────────────────────────────────────────────────────────────────────────

fn home_dir() -> Option<PathBuf> {
    env::var_os("HOME").map(PathBuf::from)
}

fn parse_config(toml_str: &str) -> Result<Config, String> {
    let value: toml::Table = toml::from_str(toml_str).map_err(|e| e.to_string())?;

    let filters_table = value
        .get("filters")
        .ok_or("missing [filters] section")?
        .as_table()
        .ok_or("[filters] must be a TOML table")?;

    let mut filter_list = Vec::new();

    for (_name, filter_val) in filters_table {
        let filter = filter_val
            .as_table()
            .ok_or("each filter entry must be a TOML table")?;

        let pattern = filter
            .get("pattern")
            .and_then(|v| v.as_str())
            .ok_or("'pattern' key is required and must be a string")?;

        let severity_str = filter
            .get("severity")
            .and_then(|v| v.as_str())
            .unwrap_or("warn");

        let severity = match severity_str.to_ascii_lowercase().as_str() {
            "info"                   => Severity::Info,
            "warn" | "warning"       => Severity::Warn,
            "error"                  => Severity::Error,
            "critical" | "fatal"     => Severity::Critical,
            other => return Err(format!("unknown severity '{other}'")),
        };

        // Store patterns lowercase: the processor also lowercases each line
        // before matching, so patterns always match case-insensitively.
        filter_list.push(Filter {
            pattern: pattern.to_ascii_lowercase(),
            severity,
        });
    }

    Ok(Config { filters: filter_list })
}