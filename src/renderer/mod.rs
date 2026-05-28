pub mod ansi;

use crate::matcher::classifier::{MatchResult, PatternDatabase};
use std::io::{self, LineWriter, Write};

pub struct Renderer<W: Write> {
    writer: LineWriter<W>,
}

impl<W: Write> Renderer<W> {
    pub fn new(writer: W) -> Self {
        Self {
            writer: LineWriter::with_capacity(8192, writer),
        }
    }

    /// Write a complete logical line.
    ///
    /// **Matched line** pipeline:
    ///  1. Emit BEL (`\x07`) for Error / Critical so the terminal plays its
    ///     alert sound – no audio files, no extra dependencies.
    ///  2. Strip any existing ANSI escape sequences from the line so that our
    ///     background colour is fully visible (not masked by the original
    ///     foreground colour, which is common in Docker Compose output).
    ///  3. Wrap the clean text in the severity colour prefix + RESET suffix.
    ///  4. Append `\n` → triggers `LineWriter` flush → output appears with
    ///     zero perceived delay.
    ///
    /// **Unmatched line** pipeline: raw bytes pass through unchanged, ANSI
    /// codes preserved, zero allocation, zero processing overhead.
    pub fn write_line(
        &mut self,
        line: &[u8],
        match_result: Option<MatchResult>,
        db: &PatternDatabase,
    ) -> io::Result<()> {
        if let Some(m) = match_result {
            let severity = db.severity(m.pattern_id);

            // ── Audible alert via terminal bell ──────────────────────────
            // Written before the colour prefix so the beep fires at the exact
            // moment the line arrives, not after the colour data is flushed.
            if ansi::should_beep(severity) {
                self.writer.write_all(ansi::BELL)?;
            }

            // ── Strip ANSI so our background colour shows end-to-end ──────
            let clean = strip_ansi(line);

            // ── Colour wrap ───────────────────────────────────────────────
            self.writer.write_all(ansi::color_code(severity))?;
            self.writer.write_all(&clean)?;
            self.writer.write_all(ansi::RESET)?;
        } else {
            // Passthrough: preserve every byte including existing ANSI codes.
            self.writer.write_all(line)?;
        }

        // Newline triggers the LineWriter's line-buffered flush → line appears
        // on screen at the same moment docker/npm/cargo would have shown it.
        self.writer.write_all(b"\n")?;
        Ok(())
    }

    /// Write raw bytes directly (used for partial / overflow data).
    #[inline]
    pub fn write_raw(&mut self, data: &[u8]) -> io::Result<()> {
        self.writer.write_all(data)
    }

    /// Flush the underlying writer.
    #[inline]
    pub fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// ANSI stripping
// ─────────────────────────────────────────────────────────────────────────────

/// Remove all ANSI / VT100 escape sequences from `data`, returning a fresh
/// `Vec<u8>` that contains only the visible text bytes.
///
/// This is called **only on matched lines** so the hot path (the vast majority
/// of unmatched lines) pays zero cost.
///
/// Sequences handled:
///   • CSI sequences: `ESC [ … <final 0x40–0x7E>`
///   • OSC sequences: `ESC ] … BEL` or `ESC ] … ESC \`
///   • All other 2-byte `ESC <x>` sequences
pub fn strip_ansi(data: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(data.len());
    let mut i = 0;

    while i < data.len() {
        if data[i] != 0x1b {
            out.push(data[i]);
            i += 1;
            continue;
        }

        // ESC found.
        if i + 1 >= data.len() {
            // Lone ESC at end of line – skip it.
            i += 1;
            continue;
        }

        match data[i + 1] {
            b'[' => {
                // CSI: ESC [ <parameter/intermediate bytes> <final byte 0x40–0x7E>
                let mut j = i + 2;
                loop {
                    if j >= data.len() {
                        i = j;
                        break;
                    }
                    if (0x40..=0x7E).contains(&data[j]) {
                        i = j + 1; // consume including final byte
                        break;
                    }
                    j += 1;
                }
            }
            b']' => {
                // OSC: ESC ] <data> BEL  |  ESC ] <data> ESC \
                let mut j = i + 2;
                loop {
                    if j >= data.len() {
                        i = j;
                        break;
                    }
                    if data[j] == 0x07 {
                        i = j + 1;
                        break;
                    }
                    if data[j] == 0x1b {
                        i = if j + 1 < data.len() && data[j + 1] == b'\\' {
                            j + 2
                        } else {
                            j + 1
                        };
                        break;
                    }
                    j += 1;
                }
            }
            _ => {
                // Generic 2-byte sequence: ESC <x>
                i += 2;
            }
        }
    }

    out
}