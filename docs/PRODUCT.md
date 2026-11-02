# Product Specification — loudness-looks-magnificent

| Field             | Value                                                        |
| ----------------- | ------------------------------------------------------------ |
| Spec version      | 1.0                                                          |
| Status            | In force                                                     |
| First published   | 2026-11-02                                                   |
| Product owner     | @szbold                                                      |
| Interpretation    | MUST/SHOULD/MAY in the RFC 2119 sense; MUST items are hard gates |
| Change control    | Requirements change only via an RFC in `docs/rfc/` plus a version bump here |

> This document is the single source of truth for what the product does and how
> well it must do it. Agents implement against requirement IDs (`FR-*`, `NFR-*`,
> `M*`, `A-*`, `OQ-*`) and MUST NOT silently change a requirement — behavior and
> quality bars evolve only through the change-control process above.

## 1 Purpose and vision

A native desktop application that renders real-time visuals driven by the
**operating system's audio output buffer** — the system output mix the user is
actually hearing (not the microphone). The application must be:

1. **Real-time and responsive** — visuals react to audio with effectively no
   perceptible delay (see NFR-1, NFR-2).
2. **System-agnostic by construction** — host-specific behavior (audio capture,
   rendering, persistence) lives behind trait implementations, so supporting a
   new OS means adding an implementation, not rewriting the app (NFR-4).
3. **Light** — comfortable on low-end/consumer hardware (NFR-3).
4. **Exclusively LLM-built** — all code is produced by AI coding agents under
   human product supervision; the agent operating manual is `AGENTS.md`.

## 2 Goals

- **G-1** Real-time audio-reactive GPU visuals shipped as a single native binary.
- **G-2** Runs at the NFR performance floor on consumer Linux hardware.
- **G-3** Trait-seamed host integration: audio capture, rendering, and preset
  persistence are pluggable interfaces; new host support = new impl + flag.
- **G-4** Open, no-copyright distribution: public domain (Unlicense), no
  attribution required.
- **G-5** Legible, well-documented code: public items are documented and docs
  evolve with the code in the same change.

## 3 Non-goals (v1.x)

- **NG-1** Not an audio player, mixer, or equalizer: the app never *writes*
  audio to the system; it only reads the output mix via loopback capture.
- **NG-2** No microphone capture in v1.x.
- **NG-3** No network features, telemetry, accounts, or analytics.
- **NG-4** No 3D asset pipeline or model editor: users tweak parameters and
  mappings in-app; shapes come from a built-in parametric registry.

## 4 Assumptions

- **A-1** Visuals were not specified up front. Working assumption: a small
  registry of parametric shapes (MVP: cube). The user selects a shape, tweaks
  its parameters, and each vertex/edge/parameter can be bound to an audio
  parameter. This is explicitly iterable; changing it is an RFC (OQ-1).
- **A-2** Primary Linux audio path: PipeWire (default on current distros) via
  the PulseAudio compatibility layer's monitor/loopback capture of the default
  output sink. Native-PipeWire or ALSA variants may supersede this (OQ-2).
- **A-3** MVP simplicity wins over depth: one shape, one mapping channel
  (loudness → scale). Generality arrives with M2/M3.
- **A-4** User profile: desktop end user on a commodity laptop/PC. No terminal
  literacy required; CLI flags exist for agent/CI use only.
- **A-5** GPU/windowing stack: `wgpu` + `winit` for rendering and window
  management (the leading current Rust stack; revisit only via RFC).

## 5 Functional requirements

### 5.1 M1 — MVP: audio-reactive cube (`v1.0.0-alpha.1`)

- **FR-M1-1 Native window and frame loop** — The app opens a native window
  (Linux x11/Wayland via `winit`), renders continuously to it, and exits with
  status 0 when the user or window manager closes it. Present is vsync-capped.

- **FR-M1-2 System audio capture (Linux)** — The app MUST capture the OS audio
  output mix (monitor/loopback of the default sink) through an `AudioSource`
  trait implementation. Capture runs on a dedicated thread; device loss or
  device change is detected and reconnected automatically (NFR-5). No
  microphone-derived data is ever read (NG-2).

- **FR-M1-3 Loudness analysis** — The app computes an amplitude loudness
  measure (RMS over a short window, default ≤ 50 ms) from captured frames,
  normalized to [0, 1] against a documented dB reference. Analysis is pure,
  host-agnostic, and unit-testable.

- **FR-M1-4 Reactive cube** — The app renders a single 3D cube whose overall
  scale is a monotonic, non-decreasing function of loudness: silence → base
  size, full scale → configurable maximum. Defaults: scale ∈ [0.6, 3.0]. The
  cube rotates slowly by default (plain aesthetic; switchable off). No visual
  flicker beyond the loudness smoothing window.

- **FR-M1-5 Resilience and degradation** — The app MUST NOT exit or panic when
  (a) no audio device is available, (b) the device disappears or changes, or
  (c) a recoverable error occurs anywhere in the capture path. In those cases
  it keeps rendering an idle cube and retries capture with backoff ≤ 2 s.

- **FR-M1-6 Config and headless mode (automation surface)** — CLI flags:
  `--config <file>`, `--headless`, `--frames <N>`, `--fake-audio [seed]`,
  `--fps-report`. The fake audio source MUST be deterministic for a given seed
  so CI runs are reproducible. Headless mode renders offscreen (no window) —
  this is the agent/CI path and a first-class feature, not a debug hack.

- **FR-M1-7 Failure reporting** — Exit codes: `0` clean exit; `1` fatal error
  (single user-readable line on stderr). Recoverable errors are logged via
  `tracing` and never crash the app.

### 5.2 M2 — Shapes, mappings, presets, UI (`v1.1.0`)

- **FR-M2-1 Shape selection** — Built-in registry of parametric shapes
  (initial set: cube, ring/waveform, spectrum bars — registry is code
  extensible). User selects the active shape in-app; the choice persists.

- **FR-M2-2 Audio parameter mappings** — The user can bind any vertex/edge/
  shape parameter to one of the canonical audio parameters:
  - `loudness` — overall loudness (FR-M1-3),
  - `band_loudness(range)` — energy within a frequency band `[f_min, f_max]`
    computed from an FFT of the captured stream (default window 2048, hop =
    capture chunk; band energy normalized),
  - `peak` — short-term waveform amplitude.
  
  Each mapping has: target (vertex/edge/parameter id), source (above), gain,
  and response curve (linear default; eased variants: `ease_in`, `ease_out`,
  `ease_in_out`). Mapping evaluation is a pure function: `(AudioState,
  mappings) → per-target values`, unit-tested.

- **FR-M2-3 Palette** — User-defined color palette (ordered color stops)
  applicable to shape parameters (e.g., edge color follows a band mapping).
  Palettes are editable in-app.

- **FR-M2-4 Presets** — Named presets capture palette + shape choice +
  mappings + tuning values. Save/load/import/export as versioned TOML on the
  filesystem (`PresetStore` trait; filesystem impl; XDG config path). Invalid
  documents fail with a precise validation error, never a partial load.
  Example document shape:

  ```toml
  preset_version = 1
  name = "bass-hum"

  [palette]
  colors = ["#0b0b12", "#00ddff", "#ffffff"]

  [shape]
  kind = "cube"
  rotation_speed = 0.2

  [[mappings]]
  target = "cube.scale"
  source = { kind = "loudness", window_ms = 50 }
  gain = 1.5
  curve = "linear"

  [[mappings]]
  target = "cube.edge[3].color"
  source = { kind = "band_loudness", f_min_hz = 20, f_max_hz = 120 }
  curve = "ease_out"
  ```

- **FR-M2-5 Editing UI** — Lightweight in-app panel (toolkit per OQ-3): shape
  picker, mapping add/edit/remove, palette editing, preset save/load. Live
  preview applies changes immediately; the last-used preset restores on start.

### 5.3 M3 — OS portability (`v1.2.0`)

- **FR-M3-1 Windows audio source** — `AudioSource` implementation for Windows
  (WASAPI loopback), behind a feature flag, honoring the same trait contract,
  latency budget, and resilience behavior as the Linux implementation.
- **FR-M3-2 Windowing/rendering portability** — The same `winit`/`wgpu` code
  path must run on Windows (Direct3D backend) without platform-specific edits
  outside adapter modules (NFR-4).

## 6 Non-functional requirements

- **NFR-1 Frame rate floor (MUST)** — Sustain **≥ 60 fps** at 1080p on the
  reference hardware class (mid-range integrated GPU, e.g. Intel UHD/Iris)
  with default settings. If the floor is at risk, the app degrades *quality*
  (ladder per OQ-5) rather than dropping frames. Measured per §9.

- **NFR-2 Audio→visual latency (MUST)** — End-to-end latency from an audio
  event reaching the capture source to the matching visual change on screen:
  **target ≤ 50 ms, hard ceiling ≤ 100 ms**, *including* smoothing. To make
  this enforceable, the architecture MUST: hand off capture→render lock-free
  (no mutex in the hot path), never block the render loop on audio or I/O, and
  analyze incrementally (O(1) per frame; no whole-history scans).

- **NFR-3 Resource footprint** — Reference budget at 1080p60 defaults:
  **≤ 25 % of one CPU core** and **≤ 200 MB RSS**. The render hot loop performs
  **zero steady-state allocations** (preallocated buffers; bounded scratch
  only); every queue is bounded with oldest-dropped overflow policy; no log
  calls in the hot loop except warnings/errors.

- **NFR-4 System-agnostic architecture (MUST)** — All host-specific behavior
  lives behind traits (`AudioSource`, rendering backend, `PresetStore`).
  Business logic (analysis, mapping, animation) is host-agnostic pure code.
  Supporting a new host MUST be expressible as: new trait implementation +
  feature flag (+ CI note), with **no edits to core logic or its tests**.
  Avoid `cfg(target_os)` outside adapter modules.

- **NFR-5 Safety and robustness (MUST)** —
  - `#![forbid(unsafe_code)]` crate-wide; no local `unsafe` without an RFC,
    local allow, and a `// SAFETY:` justification (target: zero local unsafe).
  - `unwrap`/`expect`/`panic!` are forbidden in non-test code; the only
    allowlisted site is the binary bootstrap path in `main` (see
    CONVENTIONS.md), each occurrence carrying a `// BOOTSTRAP:` reason.
  - Recoverable errors degrade gracefully and retry (backoff ≤ 2 s); the app
    never takes down the window thread for a recoverable failure.

- **NFR-6 Testability (MUST)** — Unit, integration, and E2E test suites run on
  headless Linux CI with **no audio device and no real GPU** (fake audio
  sources + software Vulkan: lavapipe). All new behavior is written test-first
  (AGENTS.md golden loop).

- **NFR-7 Quality gates** — `cargo clippy -D warnings` clean (tests included),
  coverage ≥ 70 % (ratcheted upward), and a headless E2E smoke run
  (`--headless --frames N --fake-audio`) that exits 0 and reports frames.
  Commit/PR grammar per CONVENTIONS.md.

## 7 Milestones and release strategy

| Milestone | Contents                                   | Release            |
| --------- | ------------------------------------------ | ------------------ |
| M0        | Spec + repo contracts (this document)      | spec v1.0          |
| M1        | Audio-reactive cube (FR-M1-*)              | `v1.0.0-alpha.1`   |
| M2        | Shapes, mappings, presets, UI (FR-M2-*)    | `v1.1.0`           |
| M3        | Secondary platforms (FR-M3-*)              | `v1.2.0`           |

Post-M1, releases are automated from Conventional Commits on `main`
(release-please fixes patch/feature versioning inside the ranges above).

## 8 M1 acceptance checklist (also the PR review rubric)

- [ ] FR-M1-1…FR-M1-7 implemented, each with tests referencing the FR id
- [ ] Unit: loudness monotonicity/bounds; scale mapping monotonicity
- [ ] Integration: fake `AudioSource` → analysis → mapping → renderer channel
- [ ] E2E: `--headless --fake-audio 7 --frames 180` exits 0, prints ≥ 180 frames
- [ ] `cargo clippy -D warnings` clean; zero local unwraps/unsafe
- [ ] Coverage gate green (NFR-7)
- [ ] FPS floor (NFR-1) measured on reference hardware per §9, recorded in PR
- [ ] Latency (NFR-2) measured per §9, recorded in PR
- [ ] Docs updated in the same PR (ARCHITECTURE threading, rustdoc on APIs)

## 9 Measurement procedures

- **Frame rate**: in-app `--fps-report` prints JSON (mean/p95/p99 frame ms,
  frame count). E2E asserts counts/exit; hardware fps gate is human-verified
  per milestone and recorded in the PR body. CI runs an *indicative,
  non-blocking* budget on lavapipe only once stable.
- **Latency**: the fake audio source emits a loudness spike at a known time
  (T0); instrumentation reports the first rendered frame whose cube scale
  crosses a threshold after T0 → "wire-to-frame" ms. This spans capture →
  analysis → mapping → frame; camera-to-photon is measured once on real
  hardware during M1 and recorded in the PR body.
- **CPU/RSS**: `/usr/bin/time -v` or in-app counters; recorded in PR body.

CI enforces correctness/determinism gates (NFR-6, NFR-7); hardware-dependent
numbers (NFR-1, NFR-2, NFR-3) are enforced by milestone review against §9
procedures with the recorded evidence.

## 10 Open questions

| ID   | Question                                          | Owner   | Due        | Default if unresolved            |
| ---- | ------------------------------------------------- | ------- | ---------- | -------------------------------- |
| OQ-1 | Visual direction beyond the cube (which shapes)   | szbold  | before M2  | parametric registry per A-1      |
| OQ-2 | Audio API: PipeWire native vs Pulse compat layer  | agent   | M1 impl    | Pulse monitor capture per A-2    |
| OQ-3 | UI toolkit for M2 (egui vs alternatives)          | agent   | before M2  | egui (wgpu-native)               |
| OQ-4 | Preset schema finalization                        | agent   | before M2  | example in FR-M2-4               |
| OQ-5 | Quality-degradation ladder order                  | agent   | M2 data    | resolution scale first           |

## 11 Glossary

- **Audio buffer / loopback** — the feed of the system output signal intended
  for the speakers; visualizer input (NG-1: read-only, never written).
- **Monitor source** — capture stream mirroring a playback stream (PulseAudio
  / PipeWire terminology).
- **Hot loop** — the per-frame path: latest `AudioState` → mapping → geometry
  → draw. Subject to NFR-2/NFR-3 constraints.
- **Mapping** — a rule binding an audio parameter to a visual target.
- **Preset** — palette + shape + mappings + tuning, versioned TOML (FR-M2-4).
- **lavapipe** — Mesa software Vulkan rasterizer used for headless CI renders.

## 12 Change log

| Spec version | Date       | Change                                              |
| ------------ | ---------- | --------------------------------------------------- |
| 1.0          | 2026-11-02 | Initial draft from requirements supplied by @szbold |
