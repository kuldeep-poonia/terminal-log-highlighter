use super::ansi_fsm;

/// A token produced by the tokenizer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    Text,
    Escape,
}

#[derive(Debug, Clone, Copy)]
pub struct Token {
    pub kind: TokenKind,
    pub start: usize,
    pub end: usize, // exclusive
}

/// Splits a byte slice into alternating Text and Escape tokens.
/// Always starts with a Text token (possibly empty) and ends with a Text token.
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

        // Find the next ESC byte
        let remaining = &self.data[self.pos..];
        if let Some(esc_rel) = memchr::memchr(0x1B, remaining) {
            let esc_abs = self.pos + esc_rel;

            // Text token from current pos to ESC (exclusive)
            if esc_abs > self.pos {
                let token = Token {
                    kind: TokenKind::Text,
                    start: self.pos,
                    end: esc_abs,
                };
                self.pos = esc_abs;
                return Some(token);
            }

            // Current position is exactly at ESC; parse escape sequence
            let esc_slice = &self.data[self.pos..];
            let len = ansi_fsm::parse_escape_sequence(esc_slice);
            if len == 0 {
                // Incomplete sequence at end; consume remaining as Text
                let token = Token {
                    kind: TokenKind::Text,
                    start: self.pos,
                    end: self.data.len(),
                };
                self.pos = self.data.len();
                return Some(token);
            } else {
                let token = Token {
                    kind: TokenKind::Escape,
                    start: self.pos,
                    end: self.pos + len,
                };
                self.pos += len;
                return Some(token);
            }
        } else {
            // No more ESC; remaining all text
            let token = Token {
                kind: TokenKind::Text,
                start: self.pos,
                end: self.data.len(),
            };
            self.pos = self.data.len();
            return Some(token);
        }
    }
}
