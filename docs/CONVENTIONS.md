# Engineering Conventions — loudness-looks-magnificent

Normative for every agent (and human) in this repo. `AGENTS.md` governs process;
this file governs code. Anything below marked (lint) is mechanically enforced
once the pipeline bootstrap lands; until then it is enforced by review.

## Toolchain and configuration

- `rust-toolchain.toml` pins one stable toolchain; bumping it is its own
  `chore(tools)` PR named in the commit message.
- Formatting: `rustfmt` defaults. No forks, no overrides.
- Linting: `clippy` with `all`; warnings deny everywhere, tests included.
- Additive lint tightening is welcome; *loosening* a gate requires a PR that
  says why in one sentence per loosened lint.

## Lints policy (lint)

- Crate-level `#![forbid(unsafe_code)]` in every crate. There MUST be zero
  local `unsafe` (spec NFR-5). If ever truly required: RFC + local `allow` +
  `// SAFETY:` justification + reviewer subagent sign-off in the PR.
- `clippy::unwrap_used` and `clippy::expect_used` are denied in non-test code
  (wired through the `[lints]` crate tables in `Cargo.toml`). Allowlisted:
  1. the binary bootstrap file `src/main.rs` — every crate-level allowance
     there carries a `// BOOTSTRAP:` comment explaining why typed `Result`
     flow cannot replace it; and
  2. test modules or test files, which may open with an explicit
     `#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]` —
     tests may unwrap and panic freely; non-test code may not.
- `#![warn(missing_docs)]` on public items: every public item gets rustdoc at
  birth.
- `cargo clippy -D warnings` is a no-exception gate (CI + pre-push).

## Error handling

- Library modules return typed errors (`thiserror` enums; one family per
  module area). No `anyhow` in core; the binary bootstrap path may use `anyhow`
  for context-rich unwrapping-free exit reporting.
- No swallowed errors: map or propagate with context (`?`, `.map_err`).
- `Debug`/`Display` for error variants must produce user-oriented messages
  (FR-M1-7): one line, no stack internals, no jargon.
- Never panic for an expected failure mode; classify per ARCHITECTURE.md
  (recoverable → degrade + retry; fatal → clean exit 1).
- Retry policy: bounded backoff ≤ 2 s for device/capture recovery (NFR-5);
  no hot-spinning.

## Documentation

- Every public item: rustdoc from the moment it lands ("docs ship with code").
- Behavior changes update doc comments in the same PR; doc drift is a review
  blocker.
- When a PR changes architecture shape, update `ARCHITECTURE.md`; when it
  changes behavior/quality bars, the change goes through spec change control
  (`PRODUCT.md` + RFC) — never as a side effect of a code PR.

## Testing

- Tests never depend on real audio devices, network, wall-clock timing, or
  ordering; synthetic sources are seeded and deterministic (NFR-6).
- Unit tests colocated with the module; trait-level integration in `tests/`;
  e2e drives the real binary headless from `e2e/`; golden-frame snapshots
  (insta) optional under `e2e/golden/`.
- Prefer hand-written fakes over mock frameworks: explicit, inspectable,
  dependency-free. A fake implements the same trait the production adapter does
  (tests depend on traits, not structs — spec NFR-4).
- Coverage: `cargo-llvm-cov`; gate ≥ 70 % and ratcheting (CI-configured).

## Git, commits, and PRs

- **Conventional Commits** (this grammar feeds release automation):
  `type(scope): subject` with `type` ∈ `feat|fix|docs|refactor|test|chore|
  perf|build|ci` and `scope` ∈ `capture|render|core|preset|docs|tools|ci|
  agents`.
- Every commit carries the trailer
  `Co-Authored-By: OpenCode <noreply@opencode.ai>`, appended mechanically by
  the `prepare-commit-msg` git hook (install with `just setup`).
- Hooks are enforcement, not decoration: `commit-msg` rejects grammar drift,
  `pre-commit` checks formatting, `pre-push` runs the full `just verify`.
  When a hook cannot run (e.g. bare plumbing ops), the same grammar is CI's
  problem instead.
- Tool-level push red lines live in `opencode.json` (OpenCode): agents are
  denied force pushes (`--force`/`-f`/`+refspec`/`--mirror`), refspec
  deletions, `--all` mutations, and direct pushes to `main` — hooks and CI
  remain the primary gates; these lines are the last-resort backstop.
- Commit body: imperative, explains *why*, cites requirement IDs when
  applicable (e.g. `Implements FR-M1-4`).
- Branch names: `feat|fix|docs|chore/<short-slug>`; one purpose per branch.
- PRs: describe what/why/how-tested; link FR/NFR/M ids; squash-merge on green
  CI; delete the branch after merge; never reuse a branch.
- Never: push directly to `main`, force-push, amend pushed commits, mix
  dependency bumps into behavior PRs (separate `chore(tools)` PR).
- `Cargo.lock` is committed; dependency-bump PRs include the lockfile update.
- Secret hygiene is enforced by the CI `security` job (gitleaks); the
  `pre-commit` hook runs the same scan when gitleaks is installed locally.

## Performance and safety red lines (summary of spec)

- No `unsafe`, no `unwrap`/`expect` outside the bootstrap allowlist.
- Hot loop: no allocations, no locks, no syscalls, no logging (NFR-2/NFR-3) —
  benchmark or instrument claims if you must violate a constraint (and then a
  spec change is required, not a clever exception).
- Anything host-specific goes behind a trait; anything pure stays pure.
