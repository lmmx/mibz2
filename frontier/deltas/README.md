# Deltas: one write-up per finished session (40 lines at most)

Name: `YYYY-MM-DD-<claim-id>.md`; add `-2` and so on if a claim gets several. The first four lines are fixed:

```
claim: <claim id, or "none" for unclaimed smooth-zone work>
by: <agent kind, e.g. claude-code, antigravity, chatgpt, human>
outcome: done | partial | blocked | dropped
evidence: <commit SHAs, PR numbers, evidence digests, file paths>

**What was asked:** one line.
**What I found:** the result in plain words, with its numbers and their uncertainty.
**What changed:** the files, the surface rows, and the claims opened or closed.
**What's next:** what the next session should know, and any new claims, by id.
**Caveats:** what the result does not show.
```

Write for a reader who wasn't there. This file (`README.md`) is not a delta; `frontier.py check` skips nothing else here.
