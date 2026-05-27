pub mod stream;
pub mod processor;
pub mod chunk_reader;
pub mod line_assembler;
pub mod events;

use crate::renderer::Renderer;
use std::io;

pub fn run() -> io::Result<()> {
    let stdin = io::stdin();
    let stdout = io::stdout();

    let renderer = Renderer::new(stdout.lock());
    let processor = processor::Processor::with_defaults(renderer);
    let reader = chunk_reader::ChunkReader::new(stdin.lock());

    stream::process_stream(reader, processor)
}