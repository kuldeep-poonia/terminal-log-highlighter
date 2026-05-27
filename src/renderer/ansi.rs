use crate::matcher::classifier::Severity;

/// Standard ANSI Reset sequence.
pub const RESET: &[u8] = b"\x1b[0m";

/// Return the ANSI colour escape sequence for a given severity.
pub fn color_code(severity: Severity) -> &'static [u8] {
    match severity {
        Severity::Info    => b"\x1b[34m",   // blue
        Severity::Warn    => b"\x1b[33m",   // yellow
        Severity::Error   => b"\x1b[31m",   // red
        Severity::Critical=> b"\x1b[1;31m", // bold red
    }
}