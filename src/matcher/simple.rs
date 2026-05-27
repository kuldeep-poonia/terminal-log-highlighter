use super::classifier::MatchResult;
use memchr::memmem;

pub struct SimpleMatcher {
    pattern: Vec<u8>,    // byte pattern
    pattern_id: u32,
}

impl SimpleMatcher {
    pub fn new(pattern_str: &str, id: u32) -> Self {
        Self {
            pattern: pattern_str.as_bytes().to_vec(),
            pattern_id: id,
        }
    }

    pub fn check(&self, line: &[u8]) -> Option<MatchResult> {
        memmem::find(line, &self.pattern).map(|offset| MatchResult {
            pattern_id: self.pattern_id,
            offset,
        })
    }
}