claim: none
by: antigravity
outcome: done
evidence: tests/test_frontier.py 43/43; archive-team-strata (#977 head 10a6a88, merged db83e59) stranded lease blocking migrate-rules

**What was asked:** Unblock `migrate-rules` (P2) and fix stranded direct-merge holds under #995's author-owned integration flow.
**What I found:**
- `archive-team-strata` merged via #977 (`status = "done"`, `pr = 977` on `origin/master`), yet `hold/archive-team-strata` (`owned_paths = ["docs/team/"]`) stayed unreleased because it was merged directly without `take-review` (`phase` unset).
- `finish_review` and `reconcile_integrations` only accepted leases with `phase in ("integration", "parked")`, stranding any direct-merged hold (now the default after #995) until 8h expiry and blocking `migrate-rules`.
**What changed:**
- Added `_trunk_card` and `_done_pr_on_trunk` in `frontier/tools/frontier.py`.
- Extended `finish_review` (merged only; never `cancelled=True`) to release a direct-merge lease when all safety invariants hold: verified merged PR with `Claim: <id>` and `merge_commit_sha` ancestor of `origin/master`, trunk card `status == "done"` with matching integer `pr`, and PR head descending from `lease["base"]`.
- Extended `reconcile_integrations` and `claim()`'s trigger so the next acquisition automatically reconciles verified merged done holds while failing closed if GitHub PR lookup is unavailable.
- Added 2 regression tests in `tests/test_frontier.py` (43/43 passing): direct-merge release + auto-reconcile unblocking overlapping paths, and rejections for `cancel-review`, still-open trunk cards (multi-PR claims), and stale PRs predating `lease["base"]`.
**What's next:** Independent review and merge (modifies lease-release enforcement in `frontier.py`, per RULES O2); then `archive-team-strata` auto-reconciles and `migrate-rules` is immediately claimable.
**Caveats:** Requires `status = "done"` and `pr = <N>` on `origin/master` so multi-PR or reopened claims are never prematurely released.
