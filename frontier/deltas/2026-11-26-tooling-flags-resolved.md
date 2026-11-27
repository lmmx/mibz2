claim: harden-frontier-tooling
by: claude-code
outcome: done
evidence: #974 head 92c1005 → master 3ced2be; tests/test_frontier.py 36/36; frontier/flags/2026-11-26-frontier-*.md

**What was asked:** Expedite the cap raise from 3 to 6 (Louis, 26 November).
**What I found:**
- **#974 review:** I reviewed #974 independently at 92c1005 and merged it.
  - It fixes both blockers from the #980 review:
    - `claim()` reconciles closed-unmerged as well as merged integration reservations.
    - `take-review` refuses to revive an expired lease at capacity.
  - Acquisition, renewal and transfer now push atomically with the shared `frontier-admission` epoch.
  - The suite passes 36/36, with no `expectedFailure` left.
  - CI is green: 29 check runs plus pre-commit.ci.
  - The `pull_request_target` mirror runs master's code only.
- **Flags:** every resolution condition of the three tooling flags (all by antigravity) is met on master, including Louis's recorded branch-protection decision (owner-decisions delta). Implementing branch protection stays with claim `branch-protection`.
**What changed:** the three flags are marked `resolved`, with `resolved_by` and how each was met.
**What's next: every running agent updates its client now.**
- **Who:** any checkout older than 3ced2be, which matters for antigravity, holder of `archive-team-strata`.
- **What to run:** `git fetch origin && git merge origin/master` (or rebase), before any further `claim`, `renew`, `release` or `verify`.
- **Why:** older clients skip the admission epoch and read flags locally only.
- **Release finished work:**
  - A holder whose PR merged releases its hold (`frontier.py release ID --session S`).
  - If the holder is gone, anyone can run `finish-review ID --pr N --head SHA`.
  - `single-pass-rle1-crc` stopped in #988.
- **Then:** the separate protected PR raises `max_live_holds` to 6.
**Caveats:** independence is declared, not authenticated. A client that hasn't updated can still acquire without the epoch, so two acquisitions at the same moment could overshoot the cap by one.
