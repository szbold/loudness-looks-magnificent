//! Trait-level integration: fake `AudioSource` → analysis → mapping → renderer
//! (the FR-M1-2/3/4 pipeline; NFR-6: no devices required).
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)] // test policy (CONVENTIONS.md)

use loudness_looks_magnificent::engine::loudness::LoudnessMeter;
use loudness_looks_magnificent::engine::mapping::{
    DEFAULT_SCALE_MAX, DEFAULT_SCALE_MIN, ScaleMapping,
};
use loudness_looks_magnificent::engine::synth::{
    DEFAULT_CHUNK_SAMPLES, DEFAULT_SAMPLE_RATE, SynthAudioSource,
};
use loudness_looks_magnificent::traits::{
    AudioSource, FrameCounterRenderer, RenderState, Renderer,
};

/// Run the bootstrap pipeline (50 ms loudness window) and collect the per-frame
/// scale sequence — the exact shape the M1 wgpu loop will consume.
fn run_pipeline(seed: u64, frames: usize) -> (FrameCounterRenderer, Vec<f32>) {
    let mut source =
        SynthAudioSource::new(seed, DEFAULT_SAMPLE_RATE, DEFAULT_CHUNK_SAMPLES).unwrap();
    let meter_window = DEFAULT_SAMPLE_RATE as usize * 50 / 1000;
    let mut meter = LoudnessMeter::new(meter_window).unwrap();
    let mapping = ScaleMapping::new(DEFAULT_SCALE_MIN, DEFAULT_SCALE_MAX).unwrap();
    let mut renderer = FrameCounterRenderer::default();
    let mut chunk = Vec::with_capacity(DEFAULT_CHUNK_SAMPLES);
    let mut scales = Vec::with_capacity(frames);

    for _ in 0..frames {
        source.next_chunk(&mut chunk);
        meter.push(&chunk);
        let loudness = meter.loudness();
        let scale = mapping.apply(loudness);
        renderer.draw_frame(&RenderState { loudness, scale });
        scales.push(scale);
    }
    (renderer, scales)
}

#[test]
fn pipeline_drives_every_frame() {
    let (renderer, _) = run_pipeline(7, 60);
    assert_eq!(renderer.frames(), 60);
}

#[test]
fn scales_respect_the_mapping_contract() {
    let (_, scales) = run_pipeline(7, 300);
    for (i, &s) in scales.iter().enumerate() {
        assert!(
            ((DEFAULT_SCALE_MIN - 1e-6f32)..=(DEFAULT_SCALE_MAX + 1e-6f32)).contains(&s),
            "frame {i}: scale {s} outside [{DEFAULT_SCALE_MIN}, {DEFAULT_SCALE_MAX}]"
        );
    }
    assert!(
        scales.iter().any(|&s| s > DEFAULT_SCALE_MIN + 1e-3f32),
        "signal should lift scale above the floor at some point"
    );
}

#[test]
fn same_seed_yields_identical_scales() {
    let (_, a) = run_pipeline(42, 120);
    let (_, b) = run_pipeline(42, 120);
    assert_eq!(a, b, "same seed must reproduce the stream exactly (NFR-6)");
}
