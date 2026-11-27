claim: none
by: antigravity
outcome: done
evidence: frontier/claims/release-terminal-holds.toml, frontier/deltas/2026-11-27-migrate-rules-refused-stale-hold.md

**What was asked:** Admit `release-terminal-holds` to `open` (12 of 12 open claims) to unblock stranded ordinary work leases whose claim is `done`/`dropped` on `origin/master` with a verified merged PR (currently `archive-team-strata` / #977 blocking `migrate-rules`).
**What changed:** Moved `release-terminal-holds` from `idea` to `open` (12 open claims, <= `max_open_claims`). No `stop_test`, `surface.tsv`, or protected files touched.
**What's next:** Independent participants can claim `release-terminal-holds` from `frontier.py brief`, implement and test safe reconciliation, and open a PR for independent review.
**Caveats:** Admission only; implementation touches protected `frontier/tools/frontier.py` and requires independent review and merge.
