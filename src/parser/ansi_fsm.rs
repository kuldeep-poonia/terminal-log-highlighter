/// Given a byte slice starting at an ESC byte (0x1B), returns the total length
/// of the escape sequence (including the ESC itself).
///
/// Returns `0` if the sequence is incomplete or the input does not start with
/// ESC.  This sentinel value tells the `Tokenizer` not to consume any bytes.
///
/// Called by `Tokenizer::next()` in `parser/tokenizer.rs`.
pub fn parse_escape_sequence(bytes: &[u8]) -> usize {
    if bytes.is_empty() || bytes[0] != 0x1B {
        return 0;
    }
    if bytes.len() == 1 {
        return 0; // incomplete
    }

    match bytes[1] {
        // ── Single-character sequences (ESC + 1 byte) ─────────────────────
        b'7' | b'8' | b'D' | b'M' | b'E' | b'H' | b'c' | b'=' | b'>' => 2,

        // ── CSI: ESC [ <params/intermediates> <final 0x40–0x7E> ───────────
        b'[' => {
            let mut i = 2;
            while i < bytes.len() {
                let ch = bytes[i];
                if (0x40..=0x7E).contains(&ch) {
                    return i + 1;
                }
                if !(0x20..=0x3F).contains(&ch) {
                    // Malformed; treat just the ESC as consumed.
                    return 1;
                }
                i += 1;
            }
            0 // incomplete
        }

        // ── OSC: ESC ] <data> BEL  |  ESC ] <data> ESC \ ─────────────────
        b']' => {
            let mut i = 2;
            while i < bytes.len() {
                match bytes[i] {
                    0x07 => return i + 1, // BEL terminator
                    0x1B => {
                        // ST = ESC \
                        if i + 1 < bytes.len() && bytes[i + 1] == b'\\' {
                            return i + 2;
                        }
                        return i + 1;
                    }
                    _ => {}
                }
                i += 1;
            }
            0 // incomplete
        }

        // ── Generic 2-byte sequence ───────────────────────────────────────
        _ => 2,
    }
}