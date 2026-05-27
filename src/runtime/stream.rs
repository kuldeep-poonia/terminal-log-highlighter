use crate::runtime::chunk_reader::ChunkReader;
use crate::runtime::processor::Processor;
use std::io::{self, BufRead, Write};

/// Drives the synchronous ingestion loop.
///
/// **Future architectural note (Phase 4):**
/// To decouple matching/rendering from ingestion, this loop will feed a
/// bounded MPSC channel, and a separate renderer task will consume events.
/// This isolates terminal‑write latency from stream reading.
pub fn process_stream<R: BufRead, W: Write>(
    mut reader: ChunkReader<R>,
    mut processor: Processor<W>,
) -> io::Result<()> {
    loop {
        match reader.read_chunk()? {
            Some(chunk) => processor.process_chunk(chunk)?,
            None => break,
        }
    }
    processor.flush()
}