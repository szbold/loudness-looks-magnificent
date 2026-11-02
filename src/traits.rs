//! Trait seams (NFR-4): host-specific behavior lives *only* behind these traits.
//!
//! Adapters (PipeWire capture, wgpu renderer, filesystem presets, …) implement
//! these traits inside adapter modules. Core logic and tests depend on the
//! traits — with hand-written fakes such as
//! [`synth`](crate::engine::synth::SynthAudioSource) — never on concrete
//! adapters (docs/CONVENTIONS.md "Testing").

/// A source of mono audio chunks pulled from the host's audio output mix.
///
/// Contract (FR-M1-2, NFR-2, NFR-3):
/// - fills `out` with interleaved-mono samples in `[-1, 1]`, clearing it first;
/// - never blocks the render thread: capture adapters fetch on their own
///   thread and publish atomically (docs/ARCHITECTURE.md threading) — the
///   bootstrap loop simulates that handoff with a synchronous pull;
/// - reuses `out` across calls in the steady state; implementations MUST NOT
///   allocate per call in the hot loop.
pub trait AudioSource {
    /// Fill `out` with the next captured chunk (FR-M1-2).
    fn next_chunk(&mut self, out: &mut Vec<f32>);
}

/// Per-frame render input (FR-M1-4).
#[derive(Clone, Copy, Debug)]
pub struct RenderState {
    /// Normalized loudness in `[0, 1]` (FR-M1-3).
    pub loudness: f32,
    /// Cube scale factor in `[scale_min, scale_max]` (FR-M1-4).
    pub scale: f32,
}

/// Draws exactly one frame from `state` (FR-M1-1 / FR-M1-4).
///
/// The wgpu windowed/offscreen adapter arrives with FR-M1-1 (M1);
/// [`FrameCounterRenderer`] stands in for the harness until then.
pub trait Renderer {
    /// Render one frame; steady-state calls perform no allocation (NFR-3).
    fn draw_frame(&mut self, state: &RenderState);
}

/// Headless frame counter — the bootstrap [`Renderer`] stand-in. It records
/// frame counts for e2e assertions (FR-M1-6) without any GPU dependency; the
/// M1 wgpu adapter replaces it behind the same trait.
#[derive(Clone, Copy, Debug, Default)]
pub struct FrameCounterRenderer {
    frames: u64,
}

impl Renderer for FrameCounterRenderer {
    fn draw_frame(&mut self, _state: &RenderState) {
        self.frames += 1;
    }
}

impl FrameCounterRenderer {
    /// Rendered frame count so far (e2e assertion surface).
    pub fn frames(&self) -> u64 {
        self.frames
    }
}
