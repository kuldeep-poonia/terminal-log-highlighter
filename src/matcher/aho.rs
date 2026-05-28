use super::classifier::MatchResult;
use aho_corasick::AhoCorasick;

pub struct AhoMatcher {
    ac: AhoCorasick,
    pattern_ids: Vec<u32>, // map automaton pattern index → external pattern ID
}

impl AhoMatcher {
    pub fn new(patterns: &[String], ids: &[u32]) -> Self {
        let ac = AhoCorasick::new(patterns).expect("aho-corasick build failure");
        Self {
            ac,
            pattern_ids: ids.to_vec(),
        }
    }

    /// Returns the first match found.
    pub fn check(&self, line: &[u8]) -> Option<MatchResult> {
        self.ac.find(line).map(|mat| {
            let internal_idx = mat.pattern();
            let external_id = self.pattern_ids[internal_idx];
            MatchResult {
                pattern_id: external_id,
                offset: mat.start(),
            }
        })
    }
}
