use crate::matcher::classifier::Severity;

/// ANSI reset – always emitted at the end of a highlighted line.
pub const RESET: &[u8] = b"\x1b[0m";

/// Terminal bell (ASCII BEL, 0x07).
///
/// Writing this byte to stdout causes the terminal emulator to play its
/// configured alert sound.  No audio file, no system call, no dependencies –
/// just one byte.  Works in every POSIX terminal (xterm, iTerm2, GNOME
/// Terminal, Alacritty, Windows Terminal, etc.).
pub const BELL: &[u8] = b"\x07";

/// ANSI colour prefix for each severity level.
///
/// Design goals
/// ────────────
/// 1. **Background colours** are used for Error/Critical so that the highlight
///    is visible even on lines that already carry their own ANSI foreground
///    colour (e.g. Docker Compose service-name colours).
/// 2. **Bold** is always set so the line stands out in the scrollback buffer.
/// 3. The palette is chosen to be highly readable on the dark backgrounds
///    that are standard for developer terminals.
pub fn color_code(severity: Severity) -> &'static [u8] {
    match severity {
        // Bold bright cyan text – clearly visible, not alarming.
        Severity::Info => b"\x1b[1;96m",

        // Bold bright yellow text – obvious but not panic-inducing.
        Severity::Warn => b"\x1b[1;93m",

        // Bold bright white text on a vivid RED background – unmissable.
        Severity::Error => b"\x1b[1;97;41m",

        // Bold bright yellow text on a vivid RED background – most alarming.
        // Yellow-on-red is a universally recognised danger signal.
        Severity::Critical => b"\x1b[1;93;41m",
    }
}

/// Returns `true` if the severity warrants an audible alert.
///
/// Only Error and Critical beep.  Info and Warn are visual-only so that
/// routine warnings during a `docker compose up --build` don't spam beeps.
#[inline]
pub fn should_beep(severity: Severity) -> bool {
    matches!(severity, Severity::Error | Severity::Critical)
}