//! Pure business logic: analysis, mapping, statistics, deterministic synthesis.
//!
//! No audio, GPU, window, or I/O dependencies in here — that is what keeps the
//! logic host-agnostic and CI-testable (NFR-4, NFR-6).

pub mod loudness;
pub mod mapping;
pub mod stats;
pub mod synth;
