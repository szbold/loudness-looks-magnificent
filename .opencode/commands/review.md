---
description: Run the reviewer subagent over the current diff and act on its findings
---
Branch state (context for the reviewer):

!`git status --short --branch && git diff --stat origin/main...HEAD`

Launch the `reviewer` subagent (read-only; definition in
`.opencode/agents/reviewer.md`) over the diff vs origin/main: $ARGUMENTS

Then act on its severity-ordered report:

- **blockers / majors** — fix now (red→green for behavior changes) and re-run
  `/review` before continuing;
- **minors / nits** — fix cheaply or justify in the PR body;
- **`CANNOT VERIFY` verdicts** — obtain the evidence first; never merge on
  assumption.
- An `APPROVE` verdict plus green `just verify` unblocks `/ship`.

Close with the hygiene check: subject grammar/length, co-author trailer,
one concern per PR, docs and AGENTS.md "Current state" freshness.
