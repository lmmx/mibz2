claim: none
by: claude-code
outcome: done
evidence: origin/fix/parked-hold-moved-head at 5ed0f36c (frontier/tools/frontier.py finish_review, reconcile_integrations; tests/test_frontier.py); frontier/claims/parked-hold-moved-head.toml (question, frozen stop_test); `pytest tests/test_frontier.py` at 5ed0f36c: 74 passed

**What was asked:** the independent review that `parked-hold-moved-head`'s stop_test requires, because it changes lease enforcement. The author is a different claude-code session. I took no hold and wrote nothing to the author's branch.
**What I found: CHANGES REQUIRED, one point. The mechanism is sound.**
- *Safety:* `moved_terminal` only relaxes the lease's stale `pr_head` comparison. The cancelled path still requires the live PR to be closed and unmerged at the supplied head, based on master, with the matching `Claim:` header. Reconciliation passes the live PR head only for a closed-unmerged PR whose claim is terminal. A live claim keeps its reservation, as the new test shows before the drop.
- *Stop_test (1)–(3):* (1) the stuck lease is retained while the claim is live and released `closed-unmerged` once it is dropped; (2) the merged path with a moved head is still refused (`moved_terminal` requires `cancelled`, asserted); (3) the full suite passes, 74/74.
- *Required: the second stranding mode is not covered.* The claim's question names it: `segmented-fused-postwalk`, parked on #1183 at a moved head, with its claim terminal as **blocked** on master. It says a fix keyed only on `dropped` leaves it stranded, and that a regression fixture should cover it. The branch keys on `("dropped", "done")` in both places. Add `"blocked"` to both tuples (or share one constant). Add a fixture: a parked lease with a moved head and a claim marked `blocked` on master, retained while the PR is open and released `closed-unmerged` once the PR closes unmerged.
- *Why including blocked is safe:* the release still needs the PR itself closed unmerged. That ends the reservation's integration purpose whatever the claim's later course, and a blocked claim that resumes takes a fresh hold.
**What changed:** this delta only.
**What's next:** the author adds the blocked case and its fixture. The review then reads PASS, with no second review needed. Then come the claim's `done_when` steps (#1179 closed as superseded, reconciliation releasing the parked hold) and the rescue.
**Caveats:** I read the code and ran the tests. I did not exercise the change against live origin holds.
