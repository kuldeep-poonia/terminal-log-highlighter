use crate::matcher::{self, PatternDatabase};
use crate::renderer::strip_ansi;
use crate::renderer::Renderer;
use crate::runtime::events::LineEvent;
use crate::runtime::line_assembler::LineAssembler;
use std::io::{self, Write};

pub struct Processor<W: Write> {
    renderer: Renderer<W>,
    assembler: LineAssembler,
    db: PatternDatabase,
    matcher: matcher::Matcher,
}

impl<W: Write> Processor<W> {
    pub fn new(renderer: Renderer<W>, db: PatternDatabase) -> Self {
        let matcher = matcher::Matcher::from_db(&db);
        Self {
            renderer,
            assembler: LineAssembler::new(),
            db,
            matcher,
        }
    }

    pub fn process_chunk(&mut self, chunk: &[u8]) -> io::Result<()> {
        let renderer = &mut self.renderer;
        let matcher = &self.matcher;
        let db = &self.db;

        self.assembler.push(chunk, |event| -> io::Result<()> {
            match event {
                LineEvent::Line(line) => {
                    // ── Build match key ───────────────────────────────────
                    //
                    // 1. Strip ANSI codes so the matcher never sees colour
                    //    sequences.  Docker Compose, for example, wraps service
                    //    names in colour codes; without stripping, a pattern
                    //    like "error" might fail to match "error" if it happens
                    //    to land inside an escape sequence boundary.
                    //
                    // 2. ASCII-lowercase the stripped text so that patterns
                    //    defined as "error" also match "ERROR", "Error", etc.
                    //    The original `line` (with ANSI intact) is still passed
                    //    to `write_line` for rendering.
                    let key = ascii_lowercase_bytes(&strip_ansi(line));
                    let maybe_match = matcher.check(&key);
                    renderer.write_line(line, maybe_match, db)?;
                }
                LineEvent::Overflow(data) => {
                    // Line exceeded max_line_len – pass through without matching.
                    renderer.write_raw(data)?;
                }
                LineEvent::Partial(_data) => {
                    // Handled in flush().
                }
            }
            Ok(())
        })?;

        Ok(())
    }

    pub fn flush(&mut self) -> io::Result<()> {
        let renderer = &mut self.renderer;
        let matcher = &self.matcher;
        let db = &self.db;

        self.assembler.flush(|event| -> io::Result<()> {
            if let LineEvent::Partial(data) = event {
                // Best-effort match on trailing partial line.
                let key = ascii_lowercase_bytes(&strip_ansi(data));
                let maybe_match = matcher.check(&key);
                // For partial lines we still write raw to avoid adding a stray \n.
                // If a match is found, wrap inline without a newline.
                if let Some(m) = maybe_match {
                    let severity = db.severity(m.pattern_id);
                    if crate::renderer::ansi::should_beep(severity) {
                        renderer.write_raw(crate::renderer::ansi::BELL)?;
                    }
                    let clean = strip_ansi(data);
                    renderer.write_raw(crate::renderer::ansi::color_code(severity))?;
                    renderer.write_raw(&clean)?;
                    renderer.write_raw(crate::renderer::ansi::RESET)?;
                } else {
                    renderer.write_raw(data)?;
                }
            }
            Ok(())
        })?;

        renderer.flush()
    }
}

//
// Helpers
//

/// ASCII-only lowercase: A–Z → a–z, all other bytes unchanged.
///
/// Operating directly on bytes avoids UTF-8 decoding and is safe because
/// ANSI codes and typical log text are all ASCII-compatible.
#[inline]
fn ascii_lowercase_bytes(bytes: &[u8]) -> Vec<u8> {
    bytes.iter().map(|b| b.to_ascii_lowercase()).collect()
}
