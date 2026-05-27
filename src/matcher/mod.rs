pub mod simple;
pub mod aho;
#[cfg(feature = "regex")]
pub mod regex;
pub mod classifier;

// Private import for internal use
use classifier::MatchResult;
// Public re‑export so runtime/processor can use it directly
pub use classifier::PatternDatabase;

use simple::SimpleMatcher;
use aho::AhoMatcher;

pub enum Matcher {
    Noop,
    Simple(SimpleMatcher),
    SingleByte {
        byte: u8,
        pattern_id: u32,
    },
    Aho(AhoMatcher),
    #[cfg(feature = "regex")]
    Regex(regex::bytes::Regex, u32),
}

impl Matcher {
    pub fn from_db(db: &PatternDatabase) -> Self {
        let defs: Vec<(u32, &str)> = (0..db.len() as u32)
            .map(|id| (id, db.pattern(id)))
            .collect();

        if defs.is_empty() {
            return Matcher::Noop;
        }
        if defs.len() == 1 {
            let (id, pattern) = defs[0];
            if pattern.is_empty() {
                return Matcher::Noop;
            }
            let bytes = pattern.as_bytes();
            if bytes.len() == 1 {
                return Matcher::SingleByte {
                    byte: bytes[0],
                    pattern_id: id,
                };
            }
            return Matcher::Simple(SimpleMatcher::new(pattern, id));
        }

        let patterns: Vec<String> = defs.iter().map(|(_, p)| p.to_string()).collect();
        let ids: Vec<u32> = defs.iter().map(|(id, _)| *id).collect();
        Matcher::Aho(AhoMatcher::new(&patterns, &ids))
    }

    pub fn check<'a>(&'a self, line: &'a [u8]) -> Option<MatchResult> {
        match self {
            Matcher::Noop => None,
            Matcher::Simple(s) => s.check(line),
            Matcher::SingleByte { byte, pattern_id } => {
                memchr::memchr(*byte, line).map(|offset| MatchResult {
                    pattern_id: *pattern_id,
                    offset,
                })
            }
            Matcher::Aho(a) => a.check(line),
            #[cfg(feature = "regex")]
            Matcher::Regex(re, id) => {
                re.find(line).map(|m| MatchResult {
                    pattern_id: *id,
                    offset: m.start(),
                })
            }
        }
    }
}