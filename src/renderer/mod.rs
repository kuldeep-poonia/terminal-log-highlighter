pub mod ansi;

use crate::matcher::classifier::{MatchResult, PatternDatabase};
use crate::parser::Tokenizer;
use crate::parser::tokenizer::TokenKind;
use std::io::{self, Write, LineWriter};

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
    /// * If a match is found, the **entire line** is wrapped in the colour
    ///   of the matched severity, and then reset at the end.
    /// * If the line contains its own ANSI escapes, no highlighting is
    ///   applied (to avoid corruption).
    pub fn write_line(
        &mut self,
        line: &[u8],
        match_result: Option<MatchResult>,
        db: &PatternDatabase,
    ) -> io::Result<()> {
        if let Some(m) = match_result {
            // Do not highlight lines that already contain ANSI escapes.
            if !contains_ansi(line) {
                let severity = db.severity(m.pattern_id);
                self.writer.write_all(ansi::color_code(severity))?;
                self.writer.write_all(line)?;
                self.writer.write_all(ansi::RESET)?;
            } else {
                self.writer.write_all(line)?;
            }
        } else {
            self.writer.write_all(line)?;
        }

        // Append newline – triggers LineWriter flush.
        self.writer.write_all(b"\n")?;
        Ok(())
    }

    /// Write raw bytes directly (used for partial or overflow data).
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

/// Returns true if the byte slice contains any ANSI escape sequences.
fn contains_ansi(data: &[u8]) -> bool {
    memchr::memchr(b'\x1b', data).is_some()
}