# RFC 0001 — M1 `--config` TOML schema

- **Status:** Approved — 2026-11-03 (human approval recorded in the agent
  session; PRODUCT.md 1.1 bump and changelog in the same PR). Unblocks the
  M1 plan's `--config` implementation task (T10); this document is the
  schema source of truth, referenced from PRODUCT.md FR-M1-6.
- **Author:** OpenCode (planner drafted the gap, executor drafted this spec)
- **Requirement links:** FR-M1-6 (`--config <file>` must exist), FR-M1-3
  (smoothing window default ≤ 50 ms), FR-M1-4 (configurable maximum, rotation
  switchable off), FR-M1-7 (invalid input → exit 1 with one stderr line).

## Motivation

PRODUCT.md names the tunables M1 must honor but no *file schema* is defined
anywhere — and a user-facing config surface is exactly the kind of shape
decision AGENTS.md change control reserves for an RFC rather than agent
invention. This document fills that gap for M1; the M2 preset store
generalizes beyond it later.

## Proposal

`--config <path>` loads a TOML document. Values apply at process startup
only — no hot reload in M1. Any path the loader parses as TOML is accepted;
the conventional name is `loudness.toml`. The flag may be combined with both
`--headless` and windowed mode, and is accepted at most once.

Load-time precedence is simple: built-in defaults → file values → CLI
validations. No environment variables participate in M1.

### Schema

| key                          | type    | default | validation (violation → exit 1, one stderr line)                        |
| ---------------------------- | ------- | ------- | ----------------------------------------------------------------------- |
| `scale_min`                  | float   | `0.6`   | finite, `> 0`, `<= scale_max` (mirrors `ScaleMapping::new`)              |
| `scale_max`                  | float   | `3.0`   | finite, `>= scale_min`                                                  |
| `loudness_window_ms`         | integer | `50`    | `1..=50` — RFC-local M1 cap (this RFC's decision; FR-M1-3 pins only the default ≤ 50 ms), larger windows deferred to M2 |
| `rotation_enabled`           | boolean | `true`  | —                                                                       |
| `rotation_speed_deg_per_sec` | float   | `15.0`  | finite, `> 0`; ignored when `rotation_enabled = false`; rate expressed on the 60 fps reference grid (see Determinism rule) |

Missing keys fall back to defaults. Unknown keys, wrong types, malformed
TOML, and a missing file are all fatal per FR-M1-7 with a single line naming
the exact key or parse position, e.g.:

- `error: config file 'loudness.toml' not found`
- `error: unknown config key 'scale_max_deg' (allowed: scale_min, scale_max, loudness_window_ms, rotation_enabled, rotation_speed_deg_per_sec)`
- `error: config key 'scale_max' must be a float`
- `error: invalid TOML in 'loudness.toml': <crate message, same line>`

### Determinism rule for rotation

Rotation phase is a pure function of frame index —
`phase_deg(i) = speed × i / 60.0` (60 fps reference grid) — never wall clock.
This keeps `--headless` runs bit-reproducible at the same
`--fake-audio`/`--frames` inputs (NFR-6) while looking equally smooth at
unlocked present rates; frame-pacing smoothness is present-mode territory,
not scene determinism.

Accepted tradeoff, explicit not accidental: with phase advancing per frame
index, the displayed rotation rate is `speed × present_fps / 60` off the grid
(e.g. ≈ 22.5°/s at a 90 Hz present rate). M1 chooses NFR-6 determinism over
real-time rate fidelity; a wall-clock phase source is the M2 alternative if
the human wants true deg/s at unlocked present rates (it forfeits headless
bit-reproducibility).

## Alternatives considered

1. **CLI-only knobs** — rejected: FR-M1-6 binds the file surface, and an
   ever-growing flag list becomes its own usability bug in M2.
2. **JSON** — rejected: no comment support in practice; the `toml` crate
   (which rides the implementation PR) makes round-tripping trivial.
3. **Hot reload** — rejected for M1: text-presets arrive with the M2 preset
   store; startup-only semantics keep the M1 error model honest.

## Rollout

One `feat(core): --config toml tuning surface` PR implementing exactly the
table above (loader + validation unit tests, trait-level pipeline with a
custom `[0.6, 2.0]` range, e2e good/bad-file contract tests); the `toml`
dependency rides that PR. Owed same-PR doc work ships with the code: rustdoc
on the loaded-config type and the `--help` entry for `--config`.
PRODUCT.md is bumped to 1.1 alongside the approval (version + changelog +
an FR-M1-6 pointer); this document is the schema source of truth, not a
rephrased requirement.
