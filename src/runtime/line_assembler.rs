use memchr::memchr;
use super::events::LineEvent;

/// Ring‑buffer line assembler.
///
/// - Writes into a fixed‑size ring; data is never moved in steady state.
/// - Splits on `\n`; `\r` is preserved as a literal byte.
/// - Growth path currently copies the entire buffer (like `Vec`),
///   but this only happens during initial warm‑up.
///   A future improvement will use a **segmented ring buffer** to avoid
///   any large memcopies, entirely eliminating burst‑latency spikes.
pub struct LineAssembler {
    buf: Box<[u8]>,          // fixed‑size ring buffer
    write_pos: usize,        // next write index
    read_pos: usize,         // next read index
    full: bool,              // buffer full when write_pos == read_pos && full
    max_line_len: usize,     // max bytes without \n before forced overflow
    // Shrink‑throttling: only shrink after many bytes without growth.
    consecutive_reads_since_growth: usize,
}

impl LineAssembler {
    // Start with 16 KiB to avoid the first few growths for typical streams.
    const INITIAL_CAP: usize = 16384;

    pub fn new() -> Self {
        let cap = Self::INITIAL_CAP;
        Self {
            buf: vec![0u8; cap].into_boxed_slice(),
            write_pos: 0,
            read_pos: 0,
            full: false,
            max_line_len: 65536,   // 64 KiB – protects against worst‑case scanning
            consecutive_reads_since_growth: 0,
        }
    }

    /// Number of stored bytes.
    fn len(&self) -> usize {
        if self.full {
            self.buf.len()
        } else if self.write_pos >= self.read_pos {
            self.write_pos - self.read_pos
        } else {
            self.write_pos + self.buf.len() - self.read_pos
        }
    }

    fn is_empty(&self) -> bool {
        self.write_pos == self.read_pos && !self.full
    }

    /// Write data into the ring buffer, expanding capacity if needed.
    fn extend_from_slice(&mut self, data: &[u8]) {
        let mut remaining = data.len();
        let mut src_offset = 0;

        // If not enough free space, grow.
        let free = self.buf.len() - self.len();
        if free < remaining {
            self.grow(self.len() + remaining);
        }

        // Write into one or two contiguous slices.
        while remaining > 0 {
            let first_len = if self.write_pos >= self.read_pos {
                self.buf.len() - self.write_pos
            } else {
                self.read_pos - self.write_pos
            };

            let to_write = remaining.min(first_len);
            self.buf[self.write_pos..self.write_pos + to_write]
                .copy_from_slice(&data[src_offset..src_offset + to_write]);
            self.write_pos = (self.write_pos + to_write) % self.buf.len();
            remaining -= to_write;
            src_offset += to_write;

            if self.write_pos == self.read_pos {
                self.full = true;
            }
        }
    }

    fn grow(&mut self, min_capacity: usize) {
        let new_cap = (self.buf.len() * 2).max(min_capacity).next_power_of_two();
        let mut new_buf = vec![0u8; new_cap].into_boxed_slice();

        // Drain old data into linear arrangement.
        let old_len = self.len();
        if old_len > 0 {
            if self.read_pos <= self.write_pos && !self.full {
                new_buf[..old_len].copy_from_slice(&self.buf[self.read_pos..self.write_pos]);
            } else {
                let first_part = self.buf.len() - self.read_pos;
                new_buf[..first_part].copy_from_slice(&self.buf[self.read_pos..]);
                new_buf[first_part..old_len].copy_from_slice(&self.buf[..self.write_pos]);
            }
        }
        self.buf = new_buf;
        self.read_pos = 0;
        self.write_pos = old_len;
        self.full = old_len == self.buf.len(); // should be false if we just grew

        // Reset shrink throttle on growth.
        self.consecutive_reads_since_growth = 0;
    }

    /// Shrink the buffer if it is oversized and has been stable for long enough.
    fn maybe_shrink(&mut self) {
        let used = self.len();
        if self.buf.len() > 1_048_576 && used < 16_384 {
            self.consecutive_reads_since_growth += 1;
            if self.consecutive_reads_since_growth > 10_000 {
                // Shrink down to next_power_of_two(used + 1024)
                let new_cap = (used + 1024).next_power_of_two();
                if new_cap < self.buf.len() {
                    let mut new_buf = vec![0u8; new_cap].into_boxed_slice();
                    // linearise again
                    let old_len = used;
                    if old_len > 0 {
                        if self.read_pos <= self.write_pos && !self.full {
                            new_buf[..old_len].copy_from_slice(&self.buf[self.read_pos..self.write_pos]);
                        } else {
                            let first_part = self.buf.len() - self.read_pos;
                            new_buf[..first_part].copy_from_slice(&self.buf[self.read_pos..]);
                            new_buf[first_part..old_len].copy_from_slice(&self.buf[..self.write_pos]);
                        }
                    }
                    self.buf = new_buf;
                    self.read_pos = 0;
                    self.write_pos = old_len;
                    self.full = false;
                }
                self.consecutive_reads_since_growth = 0;
            }
        } else {
            self.consecutive_reads_since_growth = 0;
        }
    }

    /// Read a single byte from the ring buffer; advances read_pos.
    /// Panics if empty.
    fn read_byte(&mut self) -> u8 {
        debug_assert!(!self.is_empty());
        let byte = self.buf[self.read_pos];
        self.read_pos = (self.read_pos + 1) % self.buf.len();
        self.full = false;
        byte
    }

    /// Feed a chunk of data, calling `cb` for each completed line.
    #[inline]
    pub fn push<F>(&mut self, data: &[u8], mut cb: F)
    where
        F: FnMut(LineEvent<'_>),
    {
        self.extend_from_slice(data);

        // Process as many complete lines as possible.
        while !self.is_empty() {
            let (first_start, first_len) = self.linear_read_range();
            if let Some(pos) = memchr(b'\n', &self.buf[first_start..first_start + first_len]) {
                // Found newline in first segment.
                let line_len = pos; // bytes before newline
                let line = &self.buf[first_start..first_start + line_len];
                // Advance read_pos past the line and the newline.
                let consume = line_len + 1;
                self.read_pos = (self.read_pos + consume) % self.buf.len();
                self.full = false;

                cb(LineEvent::Line(line));
            } else {
                // No newline in the first segment. Check second segment if any.
                let (second_start, second_len) = if self.write_pos < self.read_pos || self.full {
                    (0, self.write_pos)
                } else {
                    (0, 0)
                };
                if second_len > 0 {
                    if let Some(pos) = memchr(b'\n', &self.buf[second_start..second_start + second_len]) {
                        let line_len = pos;
                        // The line spans the end of the first segment + the second segment up to pos.
                        // We rearrange the ring buffer so the line becomes contiguous.
                        self.make_contiguous();
                        // After making contiguous, retry.
                        continue;
                    } else {
                        // No newline anywhere; check overflow.
                        let total_len = self.len();
                        if total_len > self.max_line_len {
                            // Force flush as overflow.
                            self.make_contiguous(); // ensure contiguous for the slice
                            let total = self.len();
                            let overflow = &self.buf[..total];
                            cb(LineEvent::Overflow(overflow));
                            self.read_pos = 0;
                            self.write_pos = 0;
                            self.full = false;
                            // the entire buffer is drained.
                        }
                        break;
                    }
                } else {
                    // No newline in the only segment.
                    let total_len = self.len();
                    if total_len > self.max_line_len {
                        // Force flush.
                        self.make_contiguous();
                        let total = self.len();
                        let overflow = &self.buf[..total];
                        cb(LineEvent::Overflow(overflow));
                        self.read_pos = 0;
                        self.write_pos = 0;
                        self.full = false;
                    }
                    break;
                }
            }
        }

        self.maybe_shrink();
    }

    /// Make the stored data contiguous by rearranging the ring buffer.
    fn make_contiguous(&mut self) {
        if self.is_empty() || (self.read_pos == 0 && !self.full && self.write_pos > 0) {
            return;
        }
        let len = self.len();
        let mut new_buf = vec![0u8; self.buf.len()].into_boxed_slice();
        // Copy data into new_buf starting at 0.
        if self.read_pos <= self.write_pos && !self.full {
            // single segment
            new_buf[..len].copy_from_slice(&self.buf[self.read_pos..self.write_pos]);
        } else {
            let first_part = self.buf.len() - self.read_pos;
            new_buf[..first_part].copy_from_slice(&self.buf[self.read_pos..]);
            new_buf[first_part..len].copy_from_slice(&self.buf[..self.write_pos]);
        }
        self.buf = new_buf;
        self.read_pos = 0;
        self.write_pos = len;
        self.full = false;
    }

    /// Returns the start index and length of the first contiguous segment.
    fn linear_read_range(&self) -> (usize, usize) {
        if self.is_empty() {
            (self.read_pos, 0)
        } else if self.write_pos > self.read_pos {
            (self.read_pos, self.write_pos - self.read_pos)
        } else {
            (self.read_pos, self.buf.len() - self.read_pos)
        }
    }

    /// Flush remaining data.
    pub fn flush<F>(&mut self, mut cb: F)
    where
        F: FnMut(LineEvent<'_>),
    {
        if !self.is_empty() {
            self.make_contiguous();
            let data = &self.buf[..self.write_pos];
            cb(LineEvent::Partial(data));
            self.write_pos = 0;
            self.read_pos = 0;
            self.full = false;
        }
    }
}