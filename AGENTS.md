# AGENTS.md — operating manual for AI agents

> This repository is **AI-first**: AI coding agents turn requirements into
> tested code, open PRs, and merge them; the human product owner (@szbold)
> supplies direction and requirement changes. This file is the agent
> constitution — non-negotiable process rules. Requirements live in
> `docs/PRODUCT.md` and are normative for behavior. When these two conflict on
> *what to build*, PRODUCT.md wins; on *how to build and ship*, this file wins.

## Reading order (new session — do this first)

1. `README.md` — context and charter.
2. `AGENTS.md` (this file) — how you operate.
3. `docs/PRODUCT.md` — requirements (`FR-*`, `NFR-*`, milestones `M*`, open
   questions `OQ-*`). Implement against IDs.
4. `docs/ARCHITECTURE.md` — components, trait seams, threading.
5. `docs/CONVENTIONS.md` — coding/error/doc/test/commit rules (normative).
6. `justfile` — the single command surface (`just verify` is the local CI
   mirror).

If a file on this list does not exist yet, proceed with what exists and state
the gap explicitly in your PR body (early-repo mode, see Current state below).

## Golden loop (one change = one PR)

1. **Orient**: read the relevant spec sections; restate the acceptance
   criteria for the change (in your head or the PR draft).
2. **Test first**: write the failing test(s) for the behavior (unit,
   trait-level integration, or e2e). TDD is mandatory — red before green.
   Pure refactors may skip new tests but MUST keep the suite green.
3. **Implement** the minimal change; public items documented; fakes go behind
   the same traits as production adapters (never test against concrete
   adapters).
4. **Verify** locally: `just verify` (format, clippy, unit, e2e, build).
   Iterate until green.
5. **Docs in the same PR**: update the doc comments now; update
   PRODUCT/ARCHITECTURE/CONVENTIONS if this PR changes shape, behavior, or
   process (changes to requirements themselves need spec change control, see
   below). Docs precede review so the reviewer sees the final diff.
6. **Self-review before push**: launch the `reviewer` subagent over the diff
   (diff vs `main`); resolve findings or justify them in the PR.
7. **Ship**: commit (Conventional Commits format + `Co-Authored-By: OpenCode
   <noreply@opencode.ai>` trailer), push, open the PR with a What/Why/How-tested
   body, wait for CI, merge with squash when green, delete the branch.
8. **One concern per PR.** Never bundle an unrelated fix ("while I was in
   there…"); open it as its own PR.

## Definition of done (PR checklist)

- [ ] new/changed behavior covered by tests that went red → green
- [ ] `just verify` green locally (if unavailable, say so and why)
- [ ] `cargo clippy -D warnings` clean; no new `unwrap`/`expect`/`unsafe`
- [ ] docs updated (rustdoc + spec docs where behavior/shape changed)
- [ ] commit(s): type/scope, FR/NFR/M references where applicable, trailer
- [ ] PR body: what / why / how-tested (evidence, incl. perf numbers per spec
      §9 when milestones demand them)
- [ ] CI green; merge only after full green status.

## Subagents

- `reviewer` — launch for every non-trivial diff before push; it is read-only
  and reviews against the PR rubric (correctness, spec conformance, tests,
  lint-cleanliness, doc freshness). Severity-ordered findings.
- `planner` — for multi-workstream features (M2+): turns requirements into a
  task breakdown mapped to FR IDs; each task becomes its own PR.
- Built-in explore-type subagents — use for codebase orientation and spec
  lookup instead of slow manual greps. Cheaper and less error-prone.

Both custom subagents are materialized as OpenCode definitions
(`.opencode/agents/`) with read-only permission rules; command entry points
live in `.opencode/commands/` (`/feature`, `/fix`, `/ship`, `/review`,
`/verify`); project-wide allow/deny rules in `opencode.json` deny force
pushes and direct `main` pushes at the tool level.

## Requirements and change control

- Implement **only** to `docs/PRODUCT.md`. Do not invent behavior beyond the
  `OQ-*` defaults; if you need a spec decision to proceed, raise it in the PR
  and pick the documented default.
- Requirement/quality-bar changes are **never** code PRs side effects: draft an
  RFC in `docs/rfc/NNNN-title.md`, get human approval, bump `PRODUCT.md`
  version + change log, then implement.
- A MUST you cannot satisfy is a hard stop: flag it to the human (see
  Blocking conditions) instead of shipping a quiet violation.

## Git hygiene

- Conventional Commits grammar + `Co-Authored-By: OpenCode
  <noreply@opencode.ai>` trailer on every commit — mechanically enforced by
  `.githooks/` (install with `just setup`). CI re-checks everything anyway.
- No direct `main` pushes, no force pushes, no amending pushed commits. The
  whole force-push family (`--force`, `-f`, `+refspec`, `--mirror`, `--all`,
  refspec deletions) and pushes to `main` are additionally denied at the tool
  level via `opencode.json`; re-publishing a rewritten branch that was already
  pushed is a human step, never an agent workaround.
- Keep `Cargo.lock` in sync with dependency PRs (separate `chore(tools)` PRs).

## Performance and safety red lines

- `forbid(unsafe_code)`; no `unwrap`/`expect` in non-test code outside the
  bootstrap allowlist (CONVENTIONS.md).
- Hot loop constraints (NFR-2/NFR-3) are structural: no locks/allocations/
  syscalls/logging in the steady-state render path. If a design needs an
  exception, that is an RFC — not a clever workaround.
- Tests run on traits with hand-written fakes; never against concrete adapters.

## Blocking conditions — stop and surface to the human

- A MUSTed NFR/FR is impossible or conflict-driven (e.g. latency vs resource
  budget can't both hold with the chosen design).
- Spec ambiguity that changes API/architecture shape.
- Missing credentials, tooling, or CI access; infrastructure breakage.
- Security-relevant findings in dependencies or code.
- Anything that would require editing `Cargo.lock` by hand outside a
  dependency-dependency bump.

## Honesty rules

- Never claim green CI/verification you did not observe. If `just verify`
  doesn't exist yet or a suite is skipped, say it in the PR body.
- Report flakes as flakes (first occurrence), never retry-until-green to hide
  the cause.

## Current state (agents: keep up to date)

- [x] Spec package: `docs/PRODUCT.md`, `docs/ARCHITECTURE.md`,
      `docs/CONVENTIONS.md`, `README.md`, `LICENSE` (pub-domain Unlicense)
- [x] Pipeline bootstrap: Rust scaffold + pinned toolchain
      (`rust-toolchain.toml`) + lint gates (`[lints]`: `forbid(unsafe_code)`,
      unwrap/expect/panic denied) + `.githooks/` (co-author trailer, commit
      grammar, pre-commit fmt, pre-push `just verify`) + `justfile` +
      headless loop (deterministic fake source → loudness → scale mapping →
      frame counter, tested at unit/integration/e2e layers)
- [x] CI/CD: GitHub Actions — required checks `quality` (fmt + clippy -
      `-D warnings` + coverage ≥ 70 %) / `e2e` / `security` (gitleaks);
      release-please (version bump + CHANGELOG + tag → packaged Linux binary
      artifact); Renovate config (grouped cargo bumps, auto-merge patch/minor).
      Bootstrap release `v0.1.0` seeded (tag + tar.gz artifact);
      `bootstrap-sha` excludes the pre-spec root commit from release scans.
- [ ] Repo guardrails (human): partial — `main` force-push protection
      confirmed working; still pending: enable "Allow auto-merge"; require
      `quality`/`e2e`/`security` status checks + linear history in the ruleset;
      install the Renovate GitHub app
- [x] Agent wiring: `opencode.json` permission rules (project-wide autonomy
      with tool-level red lines), `reviewer`/`planner` subagents
      (`.opencode/agents/`), `/feature /fix /ship /review /verify` commands
      (`.opencode/commands/`); the last-match-wins rule array denies the full
      force-push family, multi-branch mutations, refspec deletions, and direct
      `main` pushes at the tool level
- [ ] M1 MVP: audio-reactive cube (FR-M1-*), `v1.0.0-alpha.1` — **in progress**
      (plan: FR-M1-3/FR-M1-6-core already green from bootstrap; week 0 =
      alpha-track pin ✓, deps ✓, RFC 0001 config-schema / RFC 0002
      latency-surface drafts → awaiting human approval; then route A
      (lavapipe CI → offscreen wgpu → cube scene → winit loop) and route B
      (capture contract → pulse adapter), then `--config`, perf evidence,
      release)
