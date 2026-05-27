use crate::matcher::classifier::Severity;

pub const RESET: &[u8] = b"\x1b[0m";

/// Returns the ANSI colour escape sequence for a given severity.
/// All colours are **bright** (intense) variants for maximum visibility.
pub fn color_code(severity: Severity) -> &'static [u8] {
    match severity {
        Severity::Info     => b"\x1b[94m",   // bright blue
        Severity::Warn     => b"\x1b[93m",   // bright yellow
        Severity::Error    => b"\x1b[91m",   // bright red
        Severity::Critical => b"\x1b[1;91m", // bold bright red
    }
}