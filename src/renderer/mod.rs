pub mod ansi;
pub mod highlight;   // currently unused – ready for future expansion
pub mod passthrough; // placeholder
pub mod flush;       // placeholder

use crate::matcher::classifier::{MatchResult, PatternDatabase};
use std::io::{self, Write};

/// The single, exclusive terminal writer.
/// Every byte that reaches the terminal goes through this struct.
pub struct Renderer<W: Write> {
    writer: W,
}

impl<W: Write> Renderer<W> {
    pub fn new(writer: W) -> Self {
        Self { writer }
    }

    /// Write a complete logical line, optionally with ANSI highlighting.
    ///
    /// If `match_result` is provided, the matched substring is wrapped in
    /// ANSI colour codes (reset at the end).  A trailing newline is appended.
    pub fn write_line(
        &mut self,
        line: &[u8],
        match_result: Option<MatchResult>,
        db: &PatternDatabase,
    ) -> io::Result<()> {
        if let Some(m) = match_result {
            let severity = db.severity(m.pattern_id);
            let pattern_len = db.pattern(m.pattern_id).len();
            let start = m.offset;
            let end = start + pattern_len;

            // Write the part before the match.
            self.writer.write_all(&line[..start])?;
            // Write the ANSI colour code.
            self.writer.write_all(ansi::color_code(severity))?;
            // Write the matched portion.
            self.writer.write_all(&line[start..end])?;
            // Reset attributes.
            self.writer.write_all(ansi::RESET)?;
            // Write the rest of the line.
            self.writer.write_all(&line[end..])?;
        } else {
            self.writer.write_all(line)?;
        }
        // Append newline (always present in original stream).
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