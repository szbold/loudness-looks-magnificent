---
description: Fix a defect red→green and ship it as its own single-concern PR
---
Defect: $ARGUMENTS

1. **Reproduce** — write the failing test that demonstrates the defect and
   capture its red run. If the defect cannot be reproduced, that is a finding:
   report it, never "fix" blind.
2. **Root-cause** against docs/PRODUCT.md and docs/ARCHITECTURE.md expectations;
   identify the violated contract (or missing test) explicitly.
3. **Minimal fix** — nothing else. "While I was in there" fixes are forbidden;
   file them as their own PRs instead.
4. **Verify** — `just verify` green; the regression test must demonstrate
   red→green.
5. **Docs** — if a behavior contract changed, update rustdoc + the relevant
   spec doc; bump AGENTS.md "Current state" where warranted.
6. **Self-review + ship** — `reviewer` subagent over the diff, then commit as
   `fix(<scope>): subject` (≤100 chars, hook-appended trailer), push, PR with
   What/Why/How-tested, `sleep 10` then `gh pr checks --watch --interval 15`,
   merge on all-green as squash; sync local `main`; report with evidence.

Blocked conditions (AGENTS.md) stop and ask the human.
