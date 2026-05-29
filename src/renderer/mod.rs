pub mod ansi;
pub mod flush;
pub mod highlight;
pub mod passthrough;

use crate::matcher::classifier::{MatchResult, PatternDatabase};
use crate::parser::tokenizer::{TokenKind, Tokenizer};
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
    ///  1. Emit BEL (`\x07`) for Error / Critical – terminal plays its alert.
    ///     No audio files, no syscalls beyond the write itself.
    ///  2. Strip existing ANSI codes via `strip_ansi()` so our background
    ///     colour is not interrupted by Docker / cargo colour sequences.
    ///  3. Wrap cleaned text in severity colour prefix + RESET.
    ///  4. Append `\n` → triggers `LineWriter` flush → immediate display.
    ///
    /// **Unmatched line** pipeline: raw bytes pass through unchanged.
    pub fn write_line(
        &mut self,
        line: &[u8],
        match_result: Option<MatchResult>,
        db: &PatternDatabase,
    ) -> io::Result<()> {
        if let Some(m) = match_result {
            let severity = db.severity(m.pattern_id);

            // Audible alert: one byte, zero extra dependencies.
            if ansi::should_beep(severity) {
                self.writer.write_all(ansi::BELL)?;
            }

            let clean = strip_ansi(line);

            self.writer.write_all(ansi::color_code(severity))?;
            self.writer.write_all(&clean)?;
            self.writer.write_all(ansi::RESET)?;
        } else {
            // Passthrough: preserve every byte including existing ANSI codes.
            self.writer.write_all(line)?;
        }

        // Newline triggers LineWriter's line-buffered flush.
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
// ANSI stripping via the parser tokenizer
// ─────────────────────────────────────────────────────────────────────────────

/// Remove all ANSI / VT100 escape sequences from `data`.
///
/// Uses `parser::tokenizer::Tokenizer` which drives `parser::ansi_fsm` for
/// correct sequence boundary detection.  Only `TokenKind::Text` segments are
/// kept; `TokenKind::Escape` segments are dropped.
///
/// Called **only on matched lines**, so the common passthrough path pays zero
/// cost.
pub fn strip_ansi(data: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(data.len());
    for token in Tokenizer::new(data) {
        if token.kind == TokenKind::Text {
            out.extend_from_slice(&data[token.start..token.end]);
        }
    }
    out
}
