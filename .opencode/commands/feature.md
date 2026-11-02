---
description: Build a requirement end-to-end through the AGENTS.md golden loop — failing test to merged PR
---
Work item: $ARGUMENTS

Execute the golden loop from AGENTS.md. No step skipped, no concern bundled:

1. **Orient** — find the governing `FR-*`/`NFR-*` IDs (and any `OQ-*` defaults)
   in docs/PRODUCT.md and the design seams in docs/ARCHITECTURE.md. Restate the
   acceptance criteria as a short list. If a decision the work needs is not in
   the spec: STOP and surface it (draft an RFC instead of inventing behavior).
2. **Branch** — `git switch -c <type>/<short-name>` off current `origin/main`.
3. **Red** — write the failing test(s) first: unit, trait-level with fakes
   behind the same traits (never concrete adapters), or e2e against the real
   headless binary. Run them and capture the failure.
4. **Green** — implement the minimal change. Keep the gates intact:
   `forbid(unsafe_code)`, no `unwrap`/`expect`/`panic` outside the allowlist,
   hot path free of locks/allocations/syscalls/logging (NFR-2/NFR-3).
5. **Verify** — `just verify` until green; fix root causes, never symptoms.
6. **Docs in the same PR** — rustdoc on public items; update
   PRODUCT/ARCHITECTURE/CONVENTIONS where shape, behavior, or process changed;
   tick AGENTS.md "Current state". Requirement changes are RFCs, never
   code-PR side effects.
7. **Self-review before push** — launch the `reviewer` subagent over the diff
   vs main; resolve blockers/majors, justify the rest in the PR body.
8. **Ship** — commit (`type(scope): subject` ≤100 chars; hooks append the
   co-author trailer and enforce grammar), push (pre-push re-runs the full
   verify), `gh pr create` with a What/Why/How-tested body, then
   `sleep 10; gh pr checks --watch --interval 15` (check registration is racy).
9. **Merge** only on all-green: squash + delete branch; sync local `main`;
   report with evidence (check runs, test counts, perf numbers per PRODUCT.md
   §9 where milestones demand them).

If anything qualifies as a blocked condition (MUSTed NFR/FR impossible,
spec ambiguity shaping API/architecture, missing credentials/tooling/CI,
security findings) — stop and ask the human; never ship a quiet violation.
