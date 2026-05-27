use crate::runtime::passthrough::Passthrough;
use crate::runtime::line_assembler::LineAssembler;
use crate::runtime::events::LineEvent;
use crate::matcher::{self, PatternDatabase, MatchResult};
use std::io::{self, Write};

pub struct Processor<W: Write> {
    passthrough: Passthrough<W>,
    assembler: LineAssembler,
    db: PatternDatabase,
    matcher: matcher::Matcher,
}

impl<W: Write> Processor<W> {
    /// Creates a processor with a built‑in set of danger patterns.
    pub fn with_defaults(passthrough: Passthrough<W>) -> Self {
        let db = PatternDatabase::defaults();
        let matcher = matcher::Matcher::from_db(&db);
        Self {
            passthrough,
            assembler: LineAssembler::new(),
            db,
            matcher,
        }
    }

    /// Process an incoming byte chunk.
    ///
    /// 1. Write raw bytes directly to the terminal (passthrough‑first).
    /// 2. Feed the same bytes into the line assembler and run the matcher
    ///    as a side‑channel. Matching does **not** delay output.
    pub fn process_chunk(&mut self, chunk: &[u8]) -> io::Result<()> {
        // Passthrough – never delayed by matching.
        self.passthrough.write_all(chunk)?;

        // Side‑channel: line assembly + matching.
        let db = &self.db;
        let matcher = &self.matcher;
        self.assembler.push(chunk, |event| {
            match event {
                LineEvent::Line(line) => {
                    if let Some(matched) = matcher.check(line) {
                        // `matched` is a MatchResult with pattern_id and offset.
                        // Severity and pattern string can be retrieved via `db`.
                        let _severity = db.severity(matched.pattern_id);
                        let _pattern_str = db.pattern(matched.pattern_id);
                        // Phase 4 will use these for highlighting.
                    }
                }
                LineEvent::Overflow(_overflow) => {
                    // Overflow is not a logical line – suppress matching
                    // to avoid false amplification.
                }
                LineEvent::Partial(_) => {
                    // Only produced during final flush; handled there.
                }
            }
        });
        Ok(())
    }

    pub fn flush(&mut self) -> io::Result<()> {
        // Finalise any remaining partial data.
        let db = &self.db;
        let matcher = &self.matcher;
        self.assembler.flush(|event| {
            if let LineEvent::Partial(data) = event {
                let _ = matcher.check(data);
            }
        });
        self.passthrough.flush()
    }
}