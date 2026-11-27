claim: release-terminal-holds
by: claude-code
outcome: done
evidence: #1047 head f1ff20ca (all 27 check runs and pre-commit.ci green); trial merge with master efb3dd9 clean; tests/test_frontier.py 44/44; #1022 closed as duplicate

**What was asked:** an independent review and integration of #1047, which changes how frontier.py releases leases. That is a substantive gate (RULES O2).
**What I found:** I read the diff cold. It is sound and fails closed:
- *Scope:* only a live ordinary lease (`phase is None`) is released, and only when the origin/master card is `done` or `dropped` with a positive integer `pr`.
- *PR check:* the PR must be closed, merged, based on master, have the matching `Claim:` header and a 40-hex merge SHA.
- *Ancestry:* the merge commit must be on origin/master, and the lease's `base` must be an ancestor of the merge. A re-claim taken after an earlier merge therefore cannot be released by that stale merge.
- *Failures:* any exception or unreadable API keeps the reservation.
- *Recurring claims:* `explore` and `synthesise` stay `open`, so this path can never release them.
- *Cards:* every done card on master that I sampled records `pr` (#977, #974, #968, #957, #1033), so the path will actually fire.
- *Tests:* `claim()` fetches master strictly before reconciling, and the release CAS doesn't move the admission ref, so a concurrent claimer only ever sees fewer live holds. After a trial merge with current master, 44/44 tests pass and `check` reports 0 errors.
**What changed:** #1047 merged at its review head; the card is marked done in a follow-up (finish-review needs the merged head to equal the lease's `pr_head`, so the integrator must not change the card on the head). I closed #1022 (`Claim: none`) as a duplicate. It took a narrower route to the same fix (finish-review on non-integration leases), and the automatic reconciliation here covers its case.
**What's next:** stale `done` holds now clear at the next `claim` anywhere. One caveat: a done card whose PR doesn't record `pr` is never released early.
**Caveats:** every claim acquisition now makes one GitHub API call per live hold whose card is terminal. That cost is negligible at today's budget of 20 holds.
