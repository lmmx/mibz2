claim: merged-hold-moved-head
by: claude-code
outcome: done
evidence: PR #1303 (5fefd6ef by another session; tightening and tests by this session); tests/test_frontier.py 90 passed; cold review below

**What was asked:** release an integration reservation whose PR merged at a head that moved after handoff (#1117 / hold/scanner-a2-fast-build, recorded eb1ec40f, merged at ef5b4233), under the stop test frozen in #1337.
**What I found:** #1303, opened earlier by another session, already accepted a merged PR whose head descends from the recorded head. Against the frozen stop test it lacked two things: the terminal-claim requirement (item 3) and a `moved_head` record (item 1).
**What changed:** frontier/tools/frontier.py: `finish_review(merged)` at a moved head now also requires the claim to be terminal on origin/master, and the release records `moved_head = [recorded, merged]`. tests/test_frontier.py: the fixtures land the squash with c1 marked done and assert `moved_head`. A new fixture keeps the reservation while the claim is live. The reconcile fixture uses a `--no-local` clone, so `refs/pull/42/head` must really be fetched.
**Review (independent, a fresh agent with no part in the change, given the code and the stop test, not my conclusion): PASS.** Each condition was mutation-tested: removing the descent gate, forcing the merge-ancestor check true, or removing the terminal check each fails a test. It found one gap: the fetch branch was never exercised, because local clones copy every object. With the fetch disabled, 20 related tests still passed. I fixed that with the `--no-local` clone; disabling the fetch now fails the reconcile fixture. At its suggestion I also added a comment on the merge-ancestor check. The exact-head and closed-unmerged paths are unchanged.
**What's next:** after merge, `finish-review scanner-a2-fast-build --pr 1117 --head ef5b4233...` (or any claim's reconciliation) releases that hold, unblocking scanner-a3-k3-pipe-u8.
**Caveats:** re-calling `finish_review` with the old recorded head after a moved release exits rather than printing "already released"; harmless, since released leases are not live.
