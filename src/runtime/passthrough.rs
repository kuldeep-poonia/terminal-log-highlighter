use std::io::{self, Write};

/// The single, transparent terminal writer.
/// No internal buffering – writes go directly to the underlying handle.
/// Native terminal line‑buffering provides interactivity.
pub struct Passthrough<W: Write> {
    inner: W,
}

impl<W: Write> Passthrough<W> {
    pub fn new(inner: W) -> Self {
        Self { inner }
    }

    #[inline]
    pub fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        self.inner.write_all(buf)
    }

    #[inline]
    pub fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}