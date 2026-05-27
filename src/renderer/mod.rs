pub mod ansi;

use crate::matcher::classifier::{MatchResult, PatternDatabase};
use std::io::{self, BufWriter, Write};

/// The single, exclusive terminal writer.
/// Every byte that reaches the terminal goes through this struct.
///
/// Internally uses a `BufWriter` to minimise syscall pressure.
pub struct Renderer<W: Write> {
    writer: BufWriter<W>,
}

impl<W: Write> Renderer<W> {
    /// Create a new renderer that wraps the given writer with a
    /// default buffer capacity (8 KiB).
    pub fn new(writer: W) -> Self {
        Self {
            writer: BufWriter::with_capacity(8192, writer),
        }
    }

    /// Write a complete logical line, optionally with ANSI highlighting.
    ///
    /// * If `match_result` is provided **and** the line contains no existing
    ///   ANSI escape sequences, the matched substring is wrapped in the
    ///   appropriate colour codes (reset at the end).
    /// * If the line already contains ANSI escapes, highlighting is
    ///   suppressed to avoid terminal corruption.
    /// * A trailing newline is always written – this matches the original
    ///   stream because the assembler removed the newline to enable
    ///   matching.
    pub fn write_line(
        &mut self,
        line: &[u8],
        match_result: Option<MatchResult>,
        db: &PatternDatabase,
    ) -> io::Result<()> {
        let should_highlight = match_result.is_some() && !contains_ansi(line);

        if let Some(m) = match_result {
            if !should_highlight {
                // Write the line without any highlighting.
                self.writer.write_all(line)?;
                self.writer.write_all(b"\n")?;
                return Ok(());
            }

            let severity = db.severity(m.pattern_id);
            let pattern_str = db.pattern(m.pattern_id);
            let pattern_len = pattern_str.len();
            let start = m.offset;
            let end = start + pattern_len;

            // Guard against inconsistent matcher results.
            if start > line.len() || end > line.len() {
                // Invalid offsets – write uncolored.
                self.writer.write_all(line)?;
                self.writer.write_all(b"\n")?;
                return Ok(());
            }

            // Write the line in parts.
            self.writer.write_all(&line[..start])?;
            self.writer.write_all(ansi::color_code(severity))?;
            self.writer.write_all(&line[start..end])?;
            self.writer.write_all(ansi::RESET)?;
            self.writer.write_all(&line[end..])?;
        } else {
            self.writer.write_all(line)?;
        }

        // Restore the newline that was stripped by the line assembler.
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
/// This is a minimal check: any occurrence of `\x1b` is treated as
/// a potential escape sequence.  A full ANSI‑safe parser will replace
/// this function in Phase 7.
fn contains_ansi(data: &[u8]) -> bool {
    memchr::memchr(b'\x1b', data).is_some()
}