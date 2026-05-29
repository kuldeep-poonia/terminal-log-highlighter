/// Re-export the regex bytes engine under a short local alias.
///
/// Using `ByteRegex` instead of `regex::bytes::Regex` throughout `matcher/mod.rs`
/// keeps the type signature readable and makes the `#[cfg(feature = "regex")]`
/// gating cleaner.
pub use regex::bytes::Regex as ByteRegex;