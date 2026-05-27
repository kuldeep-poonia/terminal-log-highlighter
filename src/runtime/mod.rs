pub mod stream;
pub mod processor;
pub mod chunk_reader;
pub mod line_assembler;
pub mod events;

use crate::config::schema::Config;
use crate::renderer::Renderer;
use crate::matcher::PatternDatabase;
use std::io::{self, BufRead};

/// Run the runtime pipeline over any buffered reader.
pub fn run<R: BufRead>(reader: R, config: Config) -> io::Result<()> {
    let stdout = io::stdout();
    let db = PatternDatabase::from_defs(&config.to_pattern_defs());
    let renderer = Renderer::new(stdout.lock());
    let processor = processor::Processor::new(renderer, db);
    let chunk_reader = chunk_reader::ChunkReader::new(reader);

    stream::process_stream(chunk_reader, processor)
}