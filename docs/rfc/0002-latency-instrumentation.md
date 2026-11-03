# RFC 0002 — M1 latency instrumentation surface

- **Status:** Approved — rev 2, 2026-11-03 (human directive: separate
  `--latency-report` flag; report schema unchanged from rev 1). Approval
  unblocks the M1 latency-evidence task (T11); this document is the surface
  source of truth, referenced from PRODUCT.md 1.1's FR-M1-6.
- **Author:** OpenCode (planner drafted the gap, executor drafted this spec)
- **Requirement links:** PRODUCT.md §9 (wire-to-frame latency probe for M1
  evidence), FR-M1-6 (closed flag list — extended by this RFC with
  `--latency-report`), FR-M1-7 (invalid input → exit 1, one stderr line),
  NFR-6 (deterministic CI, no devices, no wall-clock assertions in tests).

## Revision log

- **rev 2 (2026-11-03, approved):** `latency_report` is no longer folded into
  `--fps-report` implicitly. A separate `--latency-report` flag enables it;
  the report JSON gains the `latency_report` object only when that flag is
  present. Object schema, derivation, null/no-crossing shape, and the
  `--fake-audio` parse rules carry over unchanged from rev 1; rev 2 adds the
  requires-spike one-liner to the error contract.
- **rev 1 (2026-11-02, proposed):** optional `latency_report` object inside
  the existing `--fps-report` JSON; no new flag.

## Motivation

§9 requires the first latency measurement in this milestone, but no defined
*machine output* exists for it: before this RFC the FR-M1-6 flag list was
closed (`--config`, `--headless`, `--frames`, `--fake-audio`, `--fps-report`)
— rev 2 extends it with `--latency-report` — and e2e
may not assert wall-clock values (NFR-6). Deciding how latency is expressed
is a small but genuinely user-facing format decision — RFC material under
AGENTS.md change control. Rev 1 proposed folding the report into
`--fps-report`; the product owner chose an explicit flag instead (rev 2), so
that plain `--fps-report` output never changes shape and the measurement is
requested deliberately.

## Proposal

Add one boolean CLI flag, `--latency-report`. When present:

- the run's JSON report is emitted — the flag implies `--fps-report` (asking
  for a latency measurement with no output would be useless); passing both is
  fine and prints exactly one JSON object;
- the object gains a `latency_report` member:

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

- `--latency-report` requires a spike program; without one there is nothing
  to measure and the run fails with the one-liner from the error contract
  below. This is about a *missing configured program*: a configured spike
  whose run ends before crossing is the null case described below — a
  report, not an error.
- When `--latency-report` is absent, output is byte-identical to today's
  `--fps-report` behavior (or no output) — latency instrumentation never
  changes the default report.

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

All failures below exit 1 with one stderr line:

- `--latency-report` without a spike program:
  `error: --latency-report requires --fake-audio spike:<seed>:<chunk>`
- `--fake-audio` parsing moves from clap's `value_parser(u64)` to an app-level
  program grammar, so malformed values leave clap's exit-2 usage path and
  follow FR-M1-7 semantics instead. `<seed>` and `<chunk>` must be plain
  unsigned 64-bit integers — preserving the existing `value_parser(u64)`
  domain, so seeds up to 2^64 − 1 keep working and out-of-range values hit
  the same one-line exit — e.g.:

  `error: invalid --fake-audio value 'spike:7' (expected <integer> or spike:<seed>:<chunk>)`

The rollout PR re-checks any existing e2e expectation pinned to clap's
exit-2 malformed-integer behavior and updates it to the app-level exit 1.

## Alternatives considered

1. **Fold into `--fps-report` implicitly (rev 1)** — rejected by the product
   owner in approval: couples the measurement into the default report and
   makes the flag surface implicit; an explicit flag keeps plain
   `--fps-report` output stable by default.
2. **Require `--fps-report` alongside `--latency-report`** — rejected: a
   diagnostic flag that errors unless paired with another is a usability
   trap; implying the report is one line of CLI logic.
3. **Hardware-only measurement** — rejected: NFR-6 wants a CI-verifiable
   degenerate path too; hardware verification per §9 still happens on the
   reference desktop regardless (the two do not compete).

## Rollout

The spike program and report land with the scripted-fakes + measurement
planned tasks (deterministic fake first, then the report wiring in the same
genre of PR); the e2e suite grows shape checks for both the crossed and the
no-crossing variants plus the requires-spike one-liner. Owed same-PR doc
work ships with the code, not later: rustdoc on the report struct and
program parser (this RFC's formulas + null case), `--help` lines for
`--latency-report` and the `--fake-audio` grammar, and an "Instrumentation"
subsection in ARCHITECTURE.md locating `--fps-report`/`--latency-report` at
the reporting seam.
