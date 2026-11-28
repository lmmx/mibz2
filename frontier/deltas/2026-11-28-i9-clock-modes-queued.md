claim: i9-clock-modes
by: antigravity
outcome: blocked
evidence: PR #1201 (merged 91f4e61fb38d8d3a5e485a303b5d6cf7a33912eb), PR #1275 (merged 6a56e754), ready/bench-clock-modes -> 91f4e61fb38d8d3a5e485a303b5d6cf7a33912eb, frontier/deltas/2026-11-28-clock-modes-flag-resolution.md

**What was asked:** integrate `#1201` (`bench-clock-modes`) and queue it on the physical i9-10900K now that flag `2026-11-27-clock-modes-settle-levels.md` is independently resolved.
**What I found:** `#1201` rebased cleanly onto `origin/master` (1-line `Justfile` merge), passed 21/21 unit tests, `ruff`, `frontier.py check`, and all applicable CI checks (`67710f7a`), and merged at `91f4e61fb38d8d3a5e485a303b5d6cf7a33912eb`. `ready/bench-clock-modes` was confirmed on `origin` at that exact commit (`run-signal.yml` run `46093745101`), and `#1275` queued `bench-clock-modes` in `bench/workstation-queue.txt` (`test_workstation_queue.py` 4/4).
**What changed:** set `i9-clock-modes` to `blocked` awaiting the archived `origin/evidence` bundle (`clock-modes/run-*.tar.gz`). Kept `stop_test` verbatim per `frontier.py check` (the pre-data 2x3x2 `{0, 0.2, 2.0}` s settle extension and unchanged verdict thresholds are documented in `docs/journal/2026-11-27-i9-clock-modes/registration.md` §3 and `2026-11-28-clock-modes-flag-resolution.md`).
**What's next:** verify SHA-256 and score the returned i9 bundle strictly against the frozen classifier (`PASS` mechanism / `PASS` real work / `STOP`), document the mode-robust estimator in `ab` / `RELEASING.md` §5, and route the 0.5.15 1T/12T restatement to an independent participant.
**Caveats:** no workstation timing has run yet; no production decoder code changed.
