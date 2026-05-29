pub mod simple;
pub mod aho;
#[cfg(feature = "regex")]
pub mod regex;
pub mod classifier;

use classifier::MatchResult;
pub use classifier::PatternDatabase;

use simple::SimpleMatcher;
use aho::AhoMatcher;

#[cfg(feature = "regex")]
use self::regex::ByteRegex;

pub enum Matcher {
    Noop,
    Simple(SimpleMatcher),
    SingleByte {
        byte: u8,
        pattern_id: u32,
    },
    Aho(AhoMatcher),
    #[cfg(feature = "regex")]
    Regex(ByteRegex, u32),
}

impl Matcher {
    pub fn from_db(db: &PatternDatabase) -> Self {
        // Use PatternDatabase::is_empty() for the early-exit check.
        // This avoids allocating the `defs` vec on an empty config and
        // is the only call site that makes the method non-dead-code.
        if db.is_empty() {
            return Matcher::Noop;
        }

        let defs: Vec<(u32, &str)> = (0..db.len() as u32)
            .map(|id| (id, db.pattern(id)))
            .collect();

        if defs.len() == 1 {
            let (id, pattern) = defs[0];
            if pattern.is_empty() {
                return Matcher::Noop;
            }

            // When the regex feature is active, compile single patterns as
            // real regular expressions so users can write patterns like
            // `"error.*connection"` in .sentinel.toml.
            // Falls through to Simple on compile error so a bad regex never
            // silently disables matching.
            #[cfg(feature = "regex")]
            match ByteRegex::new(pattern) {
                Ok(re) => return Matcher::Regex(re, id),
                Err(e) => eprintln!(
                    "sentinel: warning: pattern {:?} is not valid regex ({}); \
                     falling back to literal match",
                    pattern, e
                ),
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