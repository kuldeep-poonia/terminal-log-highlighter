use std::io::{self, BufRead};

/// Safe chunk reader that copies data into an internal buffer.
pub struct ChunkReader<R: BufRead> {
    reader: R,
    buf: Vec<u8>,
}

impl<R: BufRead> ChunkReader<R> {
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