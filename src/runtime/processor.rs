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
        let _db = &self.db;                     // underscore prefix avoids warning

        self.assembler.push(chunk, |event| -> io::Result<()> {
            match event {
                LineEvent::Line(line) => {
                    let maybe_match = matcher.check(line);
                    renderer.write_line(line, maybe_match, _db)?;   // use _db here
                }
                LineEvent::Overflow(data) => {
                    renderer.write_raw(data)?;
                }
                LineEvent::Partial(_data) => {
                    // handled in flush
                }
            }
            Ok(())
        })?;
        Ok(())
    }

    pub fn flush(&mut self) -> io::Result<()> {
        let renderer = &mut self.renderer;
        let matcher = &self.matcher;
        let _db = &self.db;                     // same here

        self.assembler.flush(|event| -> io::Result<()> {
            if let LineEvent::Partial(data) = event {
                let _ = matcher.check(data);   // result unused
                renderer.write_raw(data)?;
            }
            Ok(())
        })?;

        renderer.flush()
    }
}