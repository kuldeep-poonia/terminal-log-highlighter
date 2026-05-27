use std::env;
use std::fs;
use toml;

use super::defaults;
use super::schema::{Config, Filter};
use crate::matcher::classifier::Severity;

/// Load configuration from the file specified in `SENTINEL_CONFIG`, or fall back
/// to built‑in defaults.  Silent failure preserves the invisible runtime promise.
pub fn load_config() -> Config {
    if let Ok(path) = env::var("SENTINEL_CONFIG") {
        if let Ok(content) = fs::read_to_string(&path) {
            if let Ok(config) = parse_config(&content) {
                return config;
            }
            // If parsing fails, we fall back silently – no output.
        }
    }
    defaults::default_config()
}

/// Parse a TOML string into a `Config`.
fn parse_config(toml_str: &str) -> Result<Config, String> {
    let value = toml::from_str(toml_str).map_err(|e| e.to_string())?;
    let root = value.as_table().ok_or("root must be a table")?;

    let filters_table = root.get("filters").ok_or("missing [filters] section")?;
    let filters = filters_table.as_table().ok_or("[filters] must be a table")?;

    let mut filter_list = Vec::new();
    for (_filter_name, filter_val) in filters {
        let filter = filter_val.as_table().ok_or("each filter must be a table")?;

        let pattern = filter
            .get("pattern")
            .and_then(|v| v.as_str())
            .ok_or("'pattern' is required and must be a string")?;

        let severity_str = filter
            .get("severity")
            .and_then(|v| v.as_str())
            .unwrap_or("warn");

        let severity = match severity_str.to_lowercase().as_str() {
            "info" => Severity::Info,
            "warn" | "warning" => Severity::Warn,
            "error" => Severity::Error,
            "critical" => Severity::Critical,
            other => return Err(format!("unknown severity: '{}'", other)),
        };

        filter_list.push(Filter {
            pattern: pattern.to_string(),
            severity,
        });
    }

    Ok(Config {
        filters: filter_list,
    })
}