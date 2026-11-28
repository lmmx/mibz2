claim: parked-hold-moved-head
by: claude-code
outcome: done
evidence: this PR; review frontier/deltas/2026-11-28-parked-hold-moved-head-review.md (other session, branch ccr-dd054673-wldwkt: CHANGES REQUIRED on 5ed0f36c, PASS once blocked is covered); tests/test_frontier.py 75/75

**What was asked:** release a parked reservation whose claim is terminal on master and whose PR closed unmerged at a head other than the one the reservation recorded (stranding modes: crc32-avx2-ring-barrett-build on #1179, segmented-fused-postwalk on #1183).
**What I found:** `live()` keeps parked leases live forever and `finish_review` / `reconcile_integrations` released them only on an exact head match, so a rebase after handoff stranded the paths for good.
**What changed:** `frontier/tools/frontier.py`: new `TERMINAL_CLAIM_STATES = ("dropped", "done", "blocked")`; in `finish_review(cancelled=True)` and in reconciliation, when the claim is in one of those states on origin/master, the reservation is identified by claim + PR number and the closed PR's actual head. The live-PR check (closed, unmerged, base master, `Claim:` header, exact supplied head) still runs; the merged path and non-terminal claims are unchanged. Tests: one fixture per mode (dropped, blocked), each showing the lease retained while the claim is live, a merged-path finish refused at the moved head, and release with terminal `closed-unmerged` once the claim is terminal.
**What's next:** close #1179 as superseded so the next acquisition releases `hold/crc32-avx2-ring-barrett-build`, then crc32-avx2-ring-rescue can be claimed. #1183's hold releases once segmented-fused-postwalk is marked blocked or dropped on master and #1183 is closed.
**Caveats:** releasing on `blocked` assumes a closed-unmerged PR's reservation is dead; a resumed blocked claim takes a fresh hold.
