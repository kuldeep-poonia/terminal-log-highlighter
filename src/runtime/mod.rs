pub mod stream;
pub mod processor;
pub mod chunk_reader;
pub mod line_assembler;
pub mod events;

use crate::config::schema::Config;
use crate::renderer::Renderer;
use crate::matcher::PatternDatabase;
use std::io;

pub fn run(config: Config) -> io::Result<()> {
    let stdin = io::stdin();
    let stdout = io::stdout();

    let db = PatternDatabase::from_defs(&config.to_pattern_defs());
    let renderer = Renderer::new(stdout.lock());
    let processor = processor::Processor::new(renderer, db);
    let reader = chunk_reader::ChunkReader::new(stdin.lock());

    stream::process_stream(reader, processor)
}