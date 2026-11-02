//! Crate-wide error family (CONVENTIONS.md "Error handling").
//!
//! One variant family; every message renders as a single user-oriented line
//! (FR-M1-7) — no stack internals, no jargon.

use thiserror::Error;

/// Errors the binary and library report to users and CI.
#[derive(Debug, Error)]
pub enum LmemError {
    /// A configuration value is invalid (range, sanity, or shape).
    #[error("invalid configuration: {0}")]
    Config(String),

    /// Command-line usage error (bootstrap binary layer).
    #[error("usage error: {0}")]
    Cli(String),
}
