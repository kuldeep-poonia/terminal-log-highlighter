use crate::runtime::line_assembler::LineAssembler;
use crate::runtime::events::LineEvent;
use crate::matcher::{self, PatternDatabase};
use crate::renderer::Renderer;
use std::io::{self, Write};

pub struct Processor<W: Write> {
    renderer: Renderer<W>,
    assembler: LineAssembler,
    db: PatternDatabase,
    matcher: matcher::Matcher,
}

impl<W: Write> Processor<W> {
    pub fn with_defaults(renderer: Renderer<W>) -> Self {
        let db = PatternDatabase::defaults();
        let matcher = matcher::Matcher::from_db(&db);
        Self {
            renderer,
            assembler: LineAssembler::new(),
            db,
            matcher,
        }
    }

    /// Process an incoming byte chunk.
    ///
    /// Raw bytes are fed to the line assembler.  Completed lines are then
    /// handed to the renderer, which may add ANSI highlighting based on
    /// match results.  Overflow and partial data are written raw.
    pub fn process_chunk(&mut self, chunk: &[u8]) -> io::Result<()> {
        let renderer = &mut self.renderer;
        let matcher = &self.matcher;
        let db = &self.db;

        self.assembler.push(chunk, |event| -> io::Result<()> {
            match event {
                LineEvent::Line(line) => {
                    let maybe_match = matcher.check(line);
                    renderer.write_line(line, maybe_match, db)?;
                }
                LineEvent::Overflow(data) => {
                    // Write raw – no newline, no highlighting.
                    renderer.write_raw(data)?;
                }
                LineEvent::Partial(_data) => {
                    // Only produced during explicit flush; handled there.
                }
            }
            Ok(())
        })?; // propagates any I/O error from callback

        // No per‑chunk flush – renderer writes immediately.
        Ok(())
    }

    pub fn flush(&mut self) -> io::Result<()> {
        let renderer = &mut self.renderer;
        let matcher = &self.matcher;
        let db = &self.db;

        // Flush any remaining partial data from the assembler.
        self.assembler.flush(|event| -> io::Result<()> {
            if let LineEvent::Partial(data) = event {
                // Optional: match on the final partial line.
                let _ = matcher.check(data);
                renderer.write_raw(data)?;
            }
            Ok(())
        })?;

        // Flush the underlying writer.
        renderer.flush()
    }
}