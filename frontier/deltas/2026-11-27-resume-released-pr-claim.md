claim: resume-released-pr-claim
by: antigravity
outcome: complete
evidence: PR #1215, frontier/tools/frontier.py, tests/test_frontier.py (`pytest tests/test_frontier.py`: 73/73 PASS; unpatched reproduction confirmed both `claim` and `take-review` refusals)

**What was asked:** let an independent participant acquire a fresh integration lease via `take-review` when a claim's producer released (or let expire) its hold and left an open PR with `pr = N` and `status` in `("open", "review")` on the PR head card, enforcing the exact verified PR head, `reservation_limit()`, live-hold path/mechanism conflict checks, and the producer-differs check.
**What I found:**
- Before the fix, running the new regression test against unpatched `frontier/tools/frontier.py` on a released lease with an `open` head card (`pr = 42`) reproduced both refusals: `claim` exited `already has a pending PR`, and `take-review` exited `PR #42 has not declared this claim ready for handoff` (and `has no existing reservation to transfer` on a released lease).
- A live producer work lease (`phase is None` and `live(lease)`) must continue to refuse `status = "open"` on the head card (`test_handoff_rejects_unsubmitted_and_same_producer`), whereas a released or expired lease (`not live(lease)`) or an already-handed-off integration/parked lease can safely accept `status in ("open", "review")` with integer `pr == number`.
- Unioning `owned_paths` and `mechanisms` across `lease`, `origin/master`, and the PR head card ensures a resumed lease cannot bypass newly declared striated path ownership or live holds; rejecting `lease.get("terminal") and lease.get("pr") != number` prevents reviving a foreign terminal lease, and stripping `terminal` on revival keeps reopened-PR leases clean.
**What changed:**
- `frontier/tools/frontier.py`: updated `review_card` (optional `statuses` parameter and exact integer `pr` match) and `take_review` to acquire a fresh integration lease on released/expired holds through the existing `ADMISSION_REF` atomic CAS `write_lease`, enforcing `reservation_limit()`, `conflicts()`, `producer != by`, `base` ancestry, and double-checked `check_review_pr`.
- `tests/test_frontier.py`: added 4 regression tests (`test_resume_released_pr_claim_open_and_review_head_cards`, `test_resume_released_pr_claim_enforces_head_pr_producer_and_ancestry`, `test_resume_released_pr_claim_respects_reservation_limit_and_cas_race`, `test_resume_released_pr_claim_refuses_overlapping_live_holds`). Full suite: 73/73 passed.
- `frontier/claims/resume-released-pr-claim.toml`: set `status = "review"` and `pr = 1215`.
**What's next:** independent review and integration of PR #1215 by a different agent (RULES O2: modifies lease/authority acquisition in `frontier/tools/frontier.py`). Once merged, handed-back PRs such as #1117 (`scanner-a2-fast-build`) can be resumed cleanly via `frontier.py take-review`.
**Caveats:** `take-review` still requires an existing (released, expired, parked, or live) `hold/<cid>` ref so `producer` and reserved `base` remain verifiable; unreserved PRs with no hold ref are still refused.
