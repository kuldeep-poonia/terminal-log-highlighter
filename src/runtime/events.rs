/// Lightweight, zero‑allocation events emitted by the line assembler.
///
/// All variants borrow directly from the assembler’s ring buffer.
/// Later phases will add ANSI‑state‑aware fragment types.
pub enum LineEvent<'a> {
    /// A complete line **without** the trailing newline.
    Line(&'a [u8]),
    /// Partial data remaining after an explicit flush (EOF).
    Partial(&'a [u8]),
    /// Forced flush because a line exceeded `max_line_len` without a newline.
    /// This is **not** a logical line; matching and highlighting must be suppressed.
    Overflow(&'a [u8]),
}
