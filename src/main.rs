//! Bootstrap binary (FR-M1-6): the agent/CI surface of the application.
//!
//! Window rendering is FR-M1-1's work (M1) — until that lands, non-headless
//! runs exit with guidance instead of pretending to work. The fake-audio
//! headless loop is a *first-class product feature*, not a debug hack.
//!
//! Bootstrap layer rules (CONVENTIONS.md): typed errors exit cleanly with a
//! single stderr line; no panics; the `unwrap`/`expect` allowance below is the
//! entire allowlist in the repo.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)] // BOOTSTRAP: clap's `parse()` exits the process itself; obeying typed-Result flow here would only re-wrap clap-validated values.

use clap::{Arg, ArgAction, Command};
use loudness_looks_magnificent::engine::loudness::LoudnessMeter;
use loudness_looks_magnificent::engine::mapping::{
    DEFAULT_SCALE_MAX, DEFAULT_SCALE_MIN, ScaleMapping,
};
use loudness_looks_magnificent::engine::stats::summarize;
use loudness_looks_magnificent::engine::synth::{
    DEFAULT_CHUNK_SAMPLES, DEFAULT_SAMPLE_RATE, SynthAudioSource,
};
use loudness_looks_magnificent::errors::LmemError;
use loudness_looks_magnificent::traits::{
    AudioSource, FrameCounterRenderer, RenderState, Renderer,
};

/// Loudness window in milliseconds (FR-M1-3 default: ≤ 50 ms smoothing).
const LOUDNESS_WINDOW_MS: u32 = 50;
/// Seed used when `--headless` runs without an explicit `--fake-audio`.
const DEFAULT_SEED: u64 = 7;

fn cli() -> Command {
    Command::new("loudness-looks-magnificent")
        .version(env!("CARGO_PKG_VERSION"))
        .about("Native OS-agnostic audio-reactive visualizer (spec: docs/PRODUCT.md)")
        .arg(
            Arg::new("headless")
                .long("headless")
                .action(ArgAction::SetTrue)
                .help("Headless offscreen loop: the agent/CI surface (FR-M1-6)"),
        )
        .arg(
            Arg::new("frames")
                .long("frames")
                .value_parser(clap::value_parser!(usize))
                .default_value("180")
                .help("Frames to render in headless mode"),
        )
        .arg(
            Arg::new("fake-audio")
                .long("fake-audio")
                .num_args(0..=1)
                .default_missing_value("7")
                .value_parser(clap::value_parser!(u64))
                .help("Deterministic synthetic source seed (headless); bare flag = 7"),
        )
        .arg(
            Arg::new("fps-report")
                .long("fps-report")
                .action(ArgAction::SetTrue)
                .help("Print frame-time percentile JSON (FR-M1-6, spec §9)"),
        )
        .arg(
            Arg::new("config")
                .long("config")
                .value_parser(clap::value_parser!(String))
                .help("Preset/config file — reserved for the M1 config work"),
        )
}

fn run() -> Result<(), LmemError> {
    let m = cli().get_matches();

    if let Some(path) = m.get_one::<String>("config") {
        return Err(LmemError::Cli(format!(
            "--config '{path}' is reserved for the M1 preset/config work (FR-M1-6)"
        )));
    }
    let headless = m.get_flag("headless");
    if !headless {
        return Err(LmemError::Cli(
            "window rendering (FR-M1-1, milestone M1) is not implemented yet — run with --headless for the CI path".into(),
        ));
    }
    let frames = *m
        .get_one::<usize>("frames")
        .ok_or_else(|| LmemError::Cli("internal: --frames default binding is missing".into()))?;
    if frames == 0 {
        return Err(LmemError::Cli("--frames must be positive".into()));
    }
    let seed = m
        .get_one::<u64>("fake-audio")
        .copied()
        .unwrap_or(DEFAULT_SEED);
    let fps_report = m.get_flag("fps-report");

    // FR-M1-6: the fake source is deterministic per seed (NFR-6), emitting
    // 10 ms stereo mono chunks at 48 kHz.
    let mut source = SynthAudioSource::new(seed, DEFAULT_SAMPLE_RATE, DEFAULT_CHUNK_SAMPLES)?;
    let meter_window = DEFAULT_SAMPLE_RATE as usize * LOUDNESS_WINDOW_MS as usize / 1000;
    let mut meter = LoudnessMeter::new(meter_window)?;
    let mapping = ScaleMapping::new(DEFAULT_SCALE_MIN, DEFAULT_SCALE_MAX)?;
    let mut renderer = FrameCounterRenderer::default();

    // Steady-state loop: preallocated buffers only (NFR-3). This is the
    // structural template for the M1 wgpu render loop.
    let mut chunk: Vec<f32> = Vec::with_capacity(DEFAULT_CHUNK_SAMPLES);
    let mut samples_ms: Vec<f64> = Vec::with_capacity(frames);
    for _ in 0..frames {
        let t0 = std::time::Instant::now();
        source.next_chunk(&mut chunk);
        meter.push(&chunk);
        let loudness = meter.loudness();
        renderer.draw_frame(&RenderState {
            loudness,
            scale: mapping.apply(loudness),
        });
        samples_ms.push(t0.elapsed().as_secs_f64() * 1000.0);
    }

    if fps_report {
        let summary = summarize(samples_ms)
            .ok_or_else(|| LmemError::Cli("internal: empty frame sample set".into()))?;
        let p50 = summary.p50;
        let p95 = summary.p95;
        let p99 = summary.p99;
        println!(
            "{{\"frames\":{frames},\"p50_ms\":{p50:.3},\"p95_ms\":{p95:.3},\"p99_ms\":{p99:.3}}}"
        );
    }
    Ok(())
}

fn main() -> std::process::ExitCode {
    match run() {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            std::process::ExitCode::FAILURE
        }
    }
}
