# Architecture — loudness-looks-magnificent

Companion to `PRODUCT.md` (what) and `CONVENTIONS.md` (how we write code). This
document defines the structure of the system. When code changes shape, all three
documents change in the same PR (AGENTS.md same-PR doc rule).

## Primer: the three docs and their relationship

- **PRODUCT.md** = behavior + quality bars, normative, requirement IDs.
- **ARCHITECTURE.md** (this) = components, trait seams, threading, data flow.
- **CONVENTIONS.md** = coding, error, safety, doc, test, and commit rules.

## System sketch

```text
 ┌──────────────────────────────────────────────────────────────┐
 │                    app shell (winit event loop)               │
 │   lifecycle · window events · recovery orchestration          │
 └───────────┬────────────────────────────────┬─────────────────┘
             │ watches latest snapshot        │ owns
             ▼                                ▼
 ┌───────────────────────┐  publish    ┌───────────────────────────┐
 │   capture thread      │  AudioState │     render thread         │
 │  AudioSource impl →   │────────────►│  mapping engine (pure) →  │
 │  chunk read + analysis│  (lock-free │  geometry update →        │
 │  (pure fns)           │   snapshot) │  wgpu draw → vsync present│
 └───────────┬───────────┘             └────────────┬──────────────┘
             │ reads/presets                        │ offscreen target
             ▼                                      ▼
 ┌──────────────────────────┐            ┌───────────────────────────┐
 │ PresetStore impl (TOML)  │            │ headless CI path:         │
 └──────────────────────────┘            │ lavapipe + --frames N     │
                                         └───────────────────────────┘
```

## Trait seams (NFR-4 in structure form)

| Trait          | Responsibility                                    | MVP impl            | Test impl            |
| -------------- | ------------------------------------------------- | ------------------- | -------------------- |
| `AudioSource`  | obtain audio chunks from the OS mix; own the capture thread; publish `AudioState` | PipeWire via Pulse-compat monitor (A-2) | deterministic synthetic source (seeded) |
| Renderer       | draw one frame from (shape + channel values); own GPU resources | `wgpu` (windowed or offscreen) | frame-recording fake for golden asserts |
| `PresetStore`  | validate, load, save preset documents             | filesystem TOML     | tempdir filesystem   |

**Rule (NFR-4 restated as a build check):** everything outside an adapter module
is pure, host-agnostic, and unit-testable; adapters are thin. No
`cfg(target_os)` outside adapters. New host = new adapter + feature flag.

## Data flow and threading

- **capture thread (single writer)** — pulls chunks from the `AudioSource`,
  runs pure analysis (RMS now; FFT bands from M2), publishes an `AudioState`
  snapshot into a bounded lock-free slot (double/triple buffer with an atomic
  publish). Never locks anything the render thread needs.
- **render thread** — per frame: read latest `AudioState` (single atomic load)
  → mapping engine (pure) → write preallocated geometry buffers → `wgpu` draw
  → vsync present. Steady-state hot path: zero allocations, no syscalls
  (NFR-2/NFR-3).
- **app shell** — `winit` events only: resize (reconfigure surface), close, and
  recovery triggers. No per-frame work.

## Error model and recovery

- One error type family per module area via `thiserror`; boundaries map and
  add context; core carries no `anyhow`; only the binary bootstrap may hold
  `anyhow` (CONVENTIONS.md).
- Classification drives behavior (FR-M1-5):
  - **recoverable** (device gone, transient capture failure): recreate the
    `AudioSource` on a retry loop (backoff ≤ 2 s); the render thread keeps
    animating the idle state; user sees a stable app.
  - **fatal** (GPU init impossible, unrecoverable config): exit code 1 with a
    single stderr line.
- Headless mode treats a *deterministic* fake source as the only input; missing
  fake-audio flags in headless are a usage error (exit 1 with usage hint).

## Testing architecture (per NFR-6)

| Layer   | What                                                            | Where                  |
| ------- | --------------------------------------------------------------- | ---------------------- |
| unit    | analysis math (RMS, later FFT bands), mapping curves, bounds    | colocated with modules |
| trait-level integration | fake `AudioSource` → pipeline → renderer channel values | `tests/`        |
| e2e     | real binary headless: `--headless --fake-audio --frames N`; asserts exit code, frame count, fps-report JSON | `e2e/` |
| golden  | optional snapshot frames (insta) for renderer determinism       | `e2e/golden/`          |

Determinism rules (NFR-6): fake audio is seeded and deterministic; no test
depends on wall-clock timing, real audio devices, network, or a real GPU
(CI renders on lavapipe software Vulkan).

## Performance instrumentation (spec §9)

- `--fps-report` emits JSON frame stats (p50/p95/p99, count).
- Latency probe: fake source emits a spike at a recorded T0; instrumentation
  logs the first frame crossing the mapped threshold → wire-to-frame ms.
- These are product features (FR-M1-6), not debug tooling: agents and CI rely
  on them.

## Dependency posture

- Core is dependency-light: standard traits + `thiserror`; no audio, window,
  or GPU crates in the core modules hosting analysis/mapping.
- Adapters may use: `pipewire`/`pulse` bindings (capture), `wgpu`, `winit`,
  `toml`, `serde`, `insta` (test-only), `clap` (CLI), `tracing` + `tracing-subscriber`.
- Heavy/unsafe-candidate dependencies must be justified in a PR description.
