use std::io::{self, BufRead};

/// Safe chunk reader using a pre‑allocated internal buffer.
///
/// **Future improvement (Phase 8):** replace with a zero‑copy reader that
/// borrows directly from the `BufRead` internal buffer, once the ownership
/// model for PTY and stream type detection is finalised.
pub struct ChunkReader<R: BufRead> {
    reader: R,
    buf: Vec<u8>,
}

impl<R: BufRead> ChunkReader<R> {
    // 16 KB is a good balance between throughput and memory footprint
    // for typical pipe/PTY scenarios.
    const DEFAULT_CAPACITY: usize = 16384;

    pub fn new(reader: R) -> Self {
        Self {
            reader,
            buf: vec![0u8; Self::DEFAULT_CAPACITY],
        }
    }

    pub fn read_chunk(&mut self) -> io::Result<Option<&[u8]>> {
        let n = self.reader.read(&mut self.buf)?;
        if n == 0 {
            Ok(None)
        } else {
            Ok(Some(&self.buf[..n]))
        }
    }
}