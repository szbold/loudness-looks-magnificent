# RFC 0002 — M1 latency instrumentation surface

- **Status:** Proposed — awaiting human approval. Approval gates the
  latency-evidence task in the M1 plan.
- **Author:** OpenCode (planner drafted the gap, executor drafted this spec)
- **Requirement links:** PRODUCT.md §9 (wire-to-frame latency probe for M1
  evidence), FR-M1-6 (closed CLI flag list), NFR-6 (deterministic CI, no
  devices, no wall-clock assertions in tests).

## Motivation

§9 requires the first latency measurement in this milestone, but no defined
*machine output* exists for it: the FR-M1-6 flag list is closed
(`--config`, `--headless`, `--frames`, `--fake-audio`, `--fps-report`) and e2e
may not assert wall-clock values (NFR-6). Deciding how latency is expressed
without touching a closed flag list is a small but genuinely user-facing
format decision — RFC material under AGENTS.md change control.

## Proposal

Extend the **existing** `--fps-report` JSON with an optional
`latency_report` object — additive and backward compatible, no new flag:

```json
{
  "frames": 180, "p50_ms": 0.402, "p95_ms": 0.411, "p99_ms": 0.415,
  "latency_report": {
    "program": "spike:7:48",
    "spike_chunk_index": 48,
    "first_crossing_frame": 29,
    "propagation_frames": 1
  }
}
```

The worked example is derived on the 60 fps reference grid (frame length
`sample_rate / 60` = 800 samples; a frame's loudness is the shipped engine's
trailing 50 ms window query at that frame point, src/engine/loudness.rs):

- `--fake-audio` gains one program form: `spike:<seed>:<chunk>` — silence
  until global chunk index `<chunk>`, then a full-scale two-chunk (20 ms,
  seed-mixed) burst, then silence. Chunk = 480 samples = 10 ms at the
  engine's 48 kHz contract (src/engine/synth.rs). Plain integers keep the
  existing plain-seed meaning, so all current e2e invocations remain valid.
- `first_crossing_frame`: first frame index whose reported loudness is
  ≥ 0.5 — under the engine's `(20·log10(rms) + 60) / 60` mapping
  (src/engine/loudness.rs) that gate is rms ≥ 10^(−30/20) ≈ 0.032.
  Derivation for the canonical spike: onset at sample 48 × 480 = 23040; a
  frame k's loudness window is the trailing 2400 samples before k × 800.
  Frame 28's window [20000, 22400) holds no burst samples; frame 29's
  window [20800, 23200) holds 160 full-scale ones → rms ≈ 0.26 → loudness
  ≈ 0.8 ≥ 0.5 → first crossing at 29. The expected index is pinned by a
  documented unit test at rollout — formula-driven, not folklore. Quieter
  or longer bursts slide it later.
- `propagation_frames` = `first_crossing_frame − spike_first_frame`, where
  `spike_first_frame = floor(spike_chunk_index × chunk_samples ÷ 800)`
  (480-sample chunks ÷ 800-sample frames = 0.6 frames per chunk; chunk 48 →
  28, so propagation 1). Dimensionally clean: burst-onset position
  [samples] ÷ frame length [samples] = frame index. Quantization
  granularity is one frame ≈ 16.7 ms; computed by the same meter path the
  renderer consumed on the 60 fps reference grid — no wall clock.
- `first_crossing_frame` and `propagation_frames` are null when the run ends
  before any frame window reaches 0.5 — e.g. spike chunk 48 with
  `--frames 27` (run ends at 27 × 16.7 = 450 ms, before the 480 ms onset).
  `latency_report` is still emitted with the program fields, so CI can
  shape-assert both the crossed and the no-crossing variants; null is
  distinguishable from "program missing".
- When no spike program is set, `latency_report` is simply absent — old
  consumers keep parsing.
- CI asserts only shape and integer invariants, named here so the check is
  literally writable: crossed shape — `first_crossing_frame ≥
  spike_first_frame` with `spike_first_frame = floor(spike_chunk_index × 480
  ÷ 800)` derivable from the echoed `spike_chunk_index`, and
  `propagation_frames` non-null and equal to that difference; no-crossing
  shape — `program` and `spike_chunk_index` present, both latency fields
  null. Wall-clock wire-to-frame remains human/hardware evidence per
  §9 and is recorded in PR bodies, never enforced in CI (spec §9 last
  paragraph).

### Error contract (FR-M1-7)

`--fake-audio` parsing moves from clap's `value_parser(u64)` to an app-level
program grammar, so malformed values leave clap's exit-2 usage path and
follow FR-M1-7 semantics instead: exit 1, one stderr line. `<seed>` and
`<chunk>` must be plain unsigned 64-bit integers — preserving the existing
`value_parser(u64)` domain, so seeds up to 2^64 − 1 keep working and
out-of-range values hit the same one-line exit — e.g.:

- `error: invalid --fake-audio value 'spike:7' (expected <integer> or spike:<seed>:<chunk>)`

The rollout PR re-checks any existing e2e expectation pinned to clap's
exit-2 malformed-integer behavior and updates it to the app-level exit 1.

## Alternatives considered

1. **`--latency-report` flag** — rejected: adds an item to a closed flag
   list for a capability the existing reporting flag can carry additively.
2. **Hardware-only measurement** — rejected: NFR-6 wants a CI-verifiable
   degenerate path too; hardware verification per §9 still happens on the
   reference desktop regardless (the two do not compete).

## Rollout (if approved)

The spike program and report land with the scripted-fakes + measurement
planned tasks (deterministic fake first, then the report wiring in the same
genre of PR); the e2e assertion grows a `latency_report` shape check for both
the crossed and the no-crossing variants. Owed same-PR doc work ships with
the code, not later: rustdoc on the report struct and program parser (this
RFC's formulas + null case), an `--help` line for `--fake-audio` reflecting
the program grammar, and an "Instrumentation" subsection in ARCHITECTURE.md
locating `--fps-report`/`latency_report` at the reporting seam.
No PRODUCT.md edits — the RFC is the surface source of truth once approved.
