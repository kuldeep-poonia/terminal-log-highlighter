pub mod stream;
pub mod processor;
pub mod passthrough;
pub mod chunk_reader;
pub mod line_assembler;
pub mod events;

use std::io;

pub fn run() -> io::Result<()> {
    let stdin = io::stdin();
    let stdout = io::stdout();

    let passthrough = passthrough::Passthrough::new(stdout.lock());
    let processor = processor::Processor::with_defaults(passthrough);
    let reader = chunk_reader::ChunkReader::new(stdin.lock());

    stream::process_stream(reader, processor)
}