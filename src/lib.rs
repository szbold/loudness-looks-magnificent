//! loudness-looks-magnificent — core library.
//!
//! Trait-first structure (docs/ARCHITECTURE.md):
//! - [`traits`]: the seams every host-specific adapter implements
//!   ([`AudioSource`](traits::AudioSource), [`Renderer`](traits::Renderer)).
//! - [`engine`]: pure, host-agnostic business logic (analysis, mapping, stats).
//! - [`errors`]: crate-wide typed error family.
//!
//! Business logic must never depend on audio/GPU/windowing/I-O crates (NFR-4).

pub mod engine;
pub mod errors;
pub mod traits;

pub use errors::LmemError;
