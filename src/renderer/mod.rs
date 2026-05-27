pub mod ansi;

use crate::matcher::classifier::{MatchResult, PatternDatabase};
use crate::parser::Tokenizer;
use crate::parser::tokenizer::TokenKind;
use std::io::{self, BufWriter, Write};

pub struct Renderer<W: Write> {
    writer: BufWriter<W>,
}

impl<W: Write> Renderer<W> {
    pub fn new(writer: W) -> Self {
        Self {
            writer: BufWriter::with_capacity(8192, writer),
        }
    }

    /// Write a complete logical line, with ANSI‑safe highlighting.
    ///
    /// If a match is found, the matched portion is highlighted **only within
    /// plain‑text segments** of the line. Existing ANSI escape sequences are
    /// preserved untouched.
    pub fn write_line(
        &mut self,
        line: &[u8],
        match_result: Option<MatchResult>,
        db: &PatternDatabase,
    ) -> io::Result<()> {
        let tokens = Tokenizer::new(line);

        if let Some(m) = match_result {
            let severity = db.severity(m.pattern_id);
            let pattern_len = db.pattern(m.pattern_id).len();
            let match_start = m.offset;
            let match_end = match_start + pattern_len;

            for token in tokens {
                match token.kind {
                    TokenKind::Text => {
                        // Only this segment may contain the match.
                        let seg = &line[token.start..token.end];
                        if match_start >= token.start && match_end <= token.end {
                            // Match lies entirely within this text segment.
                            let before = &seg[..match_start - token.start];
                            let highlighted = &seg[match_start - token.start..match_end - token.start];
                            let after = &seg[match_end - token.start..];

                            self.writer.write_all(before)?;
                            self.writer.write_all(ansi::color_code(severity))?;
                            self.writer.write_all(highlighted)?;
                            self.writer.write_all(ansi::RESET)?;
                            self.writer.write_all(after)?;
                        } else {
                            // No match in this text segment – write as is.
                            self.writer.write_all(seg)?;
                        }
                    }
                    TokenKind::Escape => {
                        // Write escape sequence untouched.
                        self.writer.write_all(&line[token.start..token.end])?;
                    }
                }
            }
        } else {
            // No match – simply write the whole line.
            self.writer.write_all(line)?;
        }

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