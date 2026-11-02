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
- `#![deny(clippy::unwrap_used, clippy::expect_used)]` in non-test code. The
  single allowlisted site is the binary bootstrap path in `main`, and every
  allowed use carries a `// BOOTSTRAP:` comment explaining why `Result`
  propagation cannot replace it.
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
  `Co-Authored-By: OpenCode <noreply@opencode.ai>`. After the pipeline
  bootstrap this is appended by a `prepare-commit-msg` hook; until then agents
  add it manually.
- Commit body: imperative, explains *why*, cites requirement IDs when
  applicable (e.g. `Implements FR-M1-4`).
- Branch names: `feat|fix|docs|chore/<short-slug>`; one purpose per branch.
- PRs: describe what/why/how-tested; link FR/NFR/M ids; squash-merge on green
  CI; delete the branch after merge; never reuse a branch.
- Never: push directly to `main`, force-push, amend pushed commits, mix
  dependency bumps into behavior PRs (separate `chore(tools)` PR).
- `Cargo.lock` is committed; dependency-bump PRs include the lockfile update.

## Performance and safety red lines (summary of spec)

- No `unsafe`, no `unwrap`/`expect` outside the bootstrap allowlist.
- Hot loop: no allocations, no locks, no syscalls, no logging (NFR-2/NFR-3) —
  benchmark or instrument claims if you must violate a constraint (and then a
  spec change is required, not a clever exception).
- Anything host-specific goes behind a trait; anything pure stays pure.
