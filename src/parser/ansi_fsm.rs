/// Minimal ANSI escape sequence parser.
/// Given a byte slice starting at an ESC character (0x1B),
/// returns the length of the entire escape sequence (including the ESC).
/// Returns 0 if the sequence is incomplete or invalid.
pub fn parse_escape_sequence(bytes: &[u8]) -> usize {
    if bytes.is_empty() || bytes[0] != 0x1B {
        return 0;
    }
    if bytes.len() == 1 {
        return 0; // incomplete
    }

    let second = bytes[1];
    match second {
        // Single-character sequences (ESC + one char)
        b'7' | b'8' | b'D' | b'M' | b'E' | b'H' | b'c' | b'=' | b'>' => return 2,

        // CSI sequences: ESC [ ... final byte
        b'[' => {
            // Scan until final byte (0x40..0x7E)
            let mut i = 2; // start after ESC [
            while i < bytes.len() {
                let ch = bytes[i];
                if (0x40..=0x7E).contains(&ch) {
                    return i + 1; // include final byte
                }
                // Parameter and intermediate bytes are 0x30..0x3F and 0x20..0x2F
                // If we encounter something else, it's malformed, treat as plain ESC
                if !(0x20..=0x3F).contains(&ch) {
                    // Not part of a valid CSI, so just the ESC itself.
                    return 1;
                }
                i += 1;
            }
            // reached end without final byte -> incomplete, don't consume
            return 0;
        }

        // OSC sequences: ESC ] ... (terminated by BEL or ST)
        b']' => {
            let mut i = 2;
            while i < bytes.len() {
                if bytes[i] == 0x07 || bytes[i] == 0x1B {
                    // BEL or ESC (ST usually ESC \)
                    if bytes[i] == 0x1B && i + 1 < bytes.len() && bytes[i + 1] == b'\\' {
                        return i + 2;
                    }
                    return i + 1;
                }
                i += 1;
            }
            // incomplete
            return 0;
        }

        // Other two‑character sequences (ESC + one byte in range 0x20..0x2F, etc.)
        _ => {
            // In general, an escape sequence is ESC followed by one byte; we'll just consume ESC + that byte.
            return 2;
        }
    }
}