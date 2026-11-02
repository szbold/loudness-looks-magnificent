---
description: Finalize, review, and merge the work currently on this branch
---
Current branch state:

!`git status --short --branch && git diff --stat origin/main...HEAD`

1. **Inventory** — read the full `git diff origin/main...HEAD`; list what this
   PR is and is not (one concern per PR; strays get their own PR).
2. **Pre-flight** — `just verify` green (fix root causes, never retry symptoms);
   no stray/unused files; public items documented; spec docs updated where
   shape/behavior changed; AGENTS.md "Current state" fresh.
3. **Self-review before push** — launch the `reviewer` subagent over the diff;
   resolve blockers/majors (re-review if behavior changed), justify the rest
   in the PR body.
4. **Commit + PR** — logical commits (`type(scope): subject` ≤100 chars;
   grammar/trailer hook-enforced), push, `gh pr create` with What/Why/How-tested
   (perf numbers per PRODUCT.md §9 when the milestone demands them).
5. **Merge** — `sleep 10; gh pr checks --watch --interval 15` (race on check
   registration), then only on all-green `gh pr merge <N> --squash
   --delete-branch`; sync local `main`; report with links and numbers.

If verification or CI cannot go green: stop and report the root cause — no
merge-by-faith, no retry-until-green, no force pushes ever.
