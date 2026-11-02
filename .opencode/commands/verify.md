---
description: Run the full local verification suite and report it honestly, stage by stage
---
Run the local CI mirror: `just verify` (fmt-check → clippy `-D warnings` →
unit/integration tests → e2e headless suite). $ARGUMENTS

Report per stage: `PASS`/`FAIL` plus the numbers (test counts; on FAIL the
failing command and its actual output). If a stage fails: diagnose to root
cause and propose the minimal fix — do not implement it in this command
unless asked.

Rules you must obey while reporting:

- No retry-until-green to hide a flake: report it as a flake with first-
  occurrence evidence.
- Never claim a check passed that you did not observe in this run. Tooling
  gaps (e.g. a coverage tool not installed locally) are reported as gaps, not
  excuses.
- `--fps-report` runs go through the real binary: prefer
  `cargo run -- --headless --fake-audio 7 --frames 180 --fps-report` when the
  request needs runtime evidence.
