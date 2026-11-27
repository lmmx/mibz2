claim: release-terminal-holds
by: antigravity
outcome: done
evidence: frontier/tools/frontier.py, tests/test_frontier.py (44/44 passing), frontier/deltas/2026-11-27-migrate-rules-refused-stale-hold.md

**What was asked:** Let `claim` reconcile ordinary (non-integration) live leases whose claim on `origin/master` is `done`/`dropped` and whose recorded `pr` is verified merged on `origin/master`, failing closed on any uncertainty.
**What I found:**
- `claim()` previously invoked `reconcile_integrations()` only when an unreleased `integration`/`parked` lease existed, and `reconcile_integrations()` ignored ordinary (`phase is None`) leases.
- `item()` falls back to local worktree files when absent on `origin/master`; reconciliation must read claim/blog state strictly from `origin/master` (`_trunk_item`) so local uncommitted edits cannot trigger release.
- Releasing a terminal ordinary hold must require all of: live ordinary lease (`phase is None`), `origin/master` item with matching `id`, `status in ("done", "dropped")`, positive integer `pr`, verified GitHub PR (`number`, `base.ref == "master"`, `Claim: <cid>`, `state == "closed"`, `merged is True`, 40-hex `merge_commit_sha`), `merge_commit_sha` an ancestor of `origin/master`, and `lease["base"]` an ancestor of `merge_commit_sha`.
- Releasing uses CAS on `refs/heads/hold/<cid>` (without touching `refs/heads/frontier-admission`), recording `released = true` and `terminal = <merge_commit_sha>` as `finish_review` does, preserving atomic slot admission when contenders race for a freed slot.
**What changed:**
- `frontier/tools/frontier.py`: added `_trunk_item()` (reused in `take_review`) and `_release_terminal_ordinary()`; extended `reconcile_integrations()` and `claim()` to reconcile verified terminal ordinary holds while keeping `frontier.py` standard-library-only and `check()` at 5 checks (C7).
- `tests/test_frontier.py`: added three regression tests reproducing the `archive-team-strata`/`migrate-rules` stale-lease refusal and verifying automatic reconciliation on verified merged PRs, `dropped` reconciliation at capacity under concurrent CAS race, and fail-closed refusal across 14 uncertainty cases (local-only edit, non-terminal status, missing/`0`/`bool` `pr`, API failure, open/closed-unmerged PR, wrong `Claim:` header, wrong base, non-master merge SHA, and pre-lease merge SHA).
**What's next:** Independent review and merge via `take-review` (protected `frontier/tools/frontier.py` altering reservation release enforcement). Once merged, the next `claim` will automatically reconcile stale terminal holds such as `archive-team-strata` (#977) if still live.
**Caveats:** Ordinary terminal holds whose claim lacks a positive `pr` number (or whose recorded `pr` was closed unmerged, e.g. #968) remain held until expiry by design, since merge state cannot be verified unambiguously.
