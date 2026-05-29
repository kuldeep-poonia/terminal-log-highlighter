use super::ansi_fsm;

/// The kind of a token produced by `Tokenizer`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    /// Visible text bytes (no ANSI codes).
    Text,
    /// A complete ANSI / VT100 escape sequence.
    Escape,
}

/// A byte-range token referencing the original slice.
///
/// `start` and `end` are byte indices into the slice passed to `Tokenizer::new`.
#[derive(Debug, Clone, Copy)]
pub struct Token {
    pub kind: TokenKind,
    pub start: usize,
    pub end: usize, // exclusive
}

/// Splits a byte slice into alternating `Text` and `Escape` tokens.
///
/// Used by `renderer::strip_ansi` to remove escape sequences before applying
/// highlight colours, so that background colours are not broken by existing
/// ANSI codes (e.g. Docker Compose service-name colours).
pub struct Tokenizer<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> Tokenizer<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }
}

impl<'a> Iterator for Tokenizer<'a> {
    type Item = Token;

    fn next(&mut self) -> Option<Token> {
        if self.pos >= self.data.len() {
            return None;
        }

        let remaining = &self.data[self.pos..];

        if let Some(esc_rel) = memchr::memchr(0x1B, remaining) {
            let esc_abs = self.pos + esc_rel;

            // Emit a Text token for bytes before the ESC.
            if esc_abs > self.pos {
                let token = Token { kind: TokenKind::Text, start: self.pos, end: esc_abs };
                self.pos = esc_abs;
                return Some(token);
            }

            // We are exactly at an ESC; parse the escape sequence.
            let len = ansi_fsm::parse_escape_sequence(&self.data[self.pos..]);
            if len == 0 {
                // Incomplete sequence at end of data – emit as Text.
                let token = Token {
                    kind: TokenKind::Text,
                    start: self.pos,
                    end: self.data.len(),
                };
                self.pos = self.data.len();
                return Some(token);
            }

            let token = Token {
                kind: TokenKind::Escape,
                start: self.pos,
                end: self.pos + len,
            };
            self.pos += len;
            Some(token)
        } else {
            // No more ESC bytes – emit remaining bytes as Text.
            let token = Token {
                kind: TokenKind::Text,
                start: self.pos,
                end: self.data.len(),
            };
            self.pos = self.data.len();
            Some(token)
        }
    }
}