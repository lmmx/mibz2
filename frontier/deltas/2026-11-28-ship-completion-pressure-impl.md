claim: ship-completion-pressure
by: antigravity
outcome: review
evidence: tests/test_frontier.py (TestShipCompletionPressure, TestResultToRelease, TestPresence, TestWorkstationPressure, TestFrontierStress)

**What was asked:** Implement persistent P1 release completion pressure (`docs/RELEASING.md` §2a) by extending `frontier/tools/frontier.py`'s existing release recognition and result-to-release projection, exposing stable performance-input fingerprints via `#1256`'s `bench/release_preflight.py`, and covering all 9 frozen `stop_test` fixture classes plus the live `v0.5.18` stall reproduction in `tests/test_frontier.py`.

**What changed:**
- Added `classify_release_paths()` (reusing `bench/release_preflight.py`'s `classify_paths` when present with fail-closed fallback), `performance_input_fingerprint()`, `candidate_invalidation_check()`, `release_completion_state()`, `format_release_completion_lines()`, and `order_review_lines()` in `frontier/tools/frontier.py`.
- Distinguishes every §2a stage explicitly (`no-releasable-work`, `release-due-unclaimed`, `preparing`, `blocked-on-named-gate`, `queued-exact-candidate`, `returned-unread`, `gate-failed`, `ready-to-publish`, `published-awaiting-reporting`, `complete`, and fail-closed `unknown` while retaining visible unresolved release pressure).
- Derives effective P1 and first eligible action (including `take-review` transfers, §5c recalibration notices, and normal `frontier.py mark <id> --status open` resumability) without promoting optional experimental/P2 research claims to release-critical.
- Exposes active candidate validation lock (`perf-fp`, scope, and invalidation) and marks unrelated performance-input merges visibly deferred in `brief` and `STATUS.md` while keeping independent branch work, required fixes, and metadata-only merges unblocked.
- Orders release-critical PRs first in `## Awaiting review or merge` with `[P1 release-critical]`.
- Added 10 tests in `TestShipCompletionPressure` (`tests/test_frontier.py`) covering all 9 frozen fixture classes plus live `v0.5.18` reproduction.

**Verification:**
- `TestShipCompletionPressure` (10/10), `TestResultToRelease` (11/11), `TestPresence` + `TestWorkstationPressure` (21/21), and `TestFrontierStress` re-run on rebased post-`#1256`/post-`#1303` master.
- `ruff check frontier/tools/frontier.py`: 0 errors; 0 ruff issues in added tests.
- `python3 frontier/tools/frontier.py check`: 0 errors.
- Live `frontier.py brief` on post-`#1256` master surfaces `release-0-5-18`, `pre-release-i9-gate`, §5c recalibration, and active candidate pin `7be50513` (`perf-fp 3583fa7da39c6726`).

**Next:** Independent review and integration of this PR (`frontier/tools/frontier.py` projection/gate change; no self-review or self-merge).
