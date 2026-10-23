//! Fuzzy subsequence matching and highlighting for terminal UIs.

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![allow(
    clippy::cast_possible_wrap,
    clippy::cast_possible_truncation,
    clippy::needless_pass_by_value
)]

pub mod match_;

pub use match_::{highlight, match_string, Match, MatchResult};

/// Convenience re-export.
pub use match_::Match as FuzzyMatch;
