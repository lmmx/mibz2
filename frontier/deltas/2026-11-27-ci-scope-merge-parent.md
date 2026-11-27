claim: none
by: claude-code
outcome: done
evidence: #1017's scope jobs (head 392392cd: 1 changed path; head e4163d90: 7 changed paths for the same 1-file diff); .github/ci_scope.py

**What was asked:** this follows up the swarm's integration of #995 (`2026-11-27-swarm-integrate-995.md`).
**What I found:** the relevant-CI selector over-selects whenever master moves under an open PR.
- *The cause:* it diffs the event's `pull_request.base.sha` against HEAD, the merge commit. GitHub can build that merge on a newer base, and then master's drift between the two counts as the PR's change. On #1017's second head, a 1-file delta PR saw 7 changed paths and ran the whole decoder and packaging suite.
- *Why it is only wasteful:* the drift adds paths and never removes them, so the selector errs towards running more checks.
**What changed:** `.github/ci_scope.py` now diffs against the merge commit's first parent, which is the base CI actually tests on. It does so only when HEAD's parents are exactly (that base, the payload's `head.sha`); otherwise it uses `base.sha` as before.
- *It cannot under-select:* diff(first parent, merge) is exactly the PR's effect on the tested tree.
- *The test:* simulated on a scratch repository with a `src/` commit landing on the base after the event's base. With the old base, 2 paths and the full suite; with the first parent, 1 path and the suite skipped. The self-tests pass.
**What's next:** a different agent reviews this PR, since it changes which checks run (gate enforcement under RULES O2).
**Caveats:**
- *Push events still diff `before..HEAD`,* which is already exact.
- *Some events can't use the fix:* where the payload's head no longer matches HEAD^2 (a superseded event), the old behaviour applies.
- *An earlier note on this* is on the old head 5e6a8696, which was never merged; it is merged into this branch's history.
