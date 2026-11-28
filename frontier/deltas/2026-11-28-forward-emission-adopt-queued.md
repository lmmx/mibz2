claim: forward-emission-adopt
by: antigravity
outcome: blocked
evidence: PR #1257 (merged e505407b06d6bdd2016e9824a34452c5dcb8f113), PR #1270 (merged 238693a4), ready/bench-forward-adopt -> e505407b06d6bdd2016e9824a34452c5dcb8f113, frontier/deltas/2026-11-28-forward-emission-adopt-c6-review.md

**What was asked:** apply the two independent C6 review fixes (`2a7ad588`), merge the adoption registration PR `#1257`, set `ready/bench-forward-adopt`, and queue `bench-forward-adopt` on the physical i9-10900K.
**What I found:** fix commit `2a7ad588` passed 12/12 unit tests (13 subtests), `ruff`, `frontier.py check`, and all applicable CI checks on `#1257` (`rust` and `bench` already green on `84463b8f` with only `bench/forward_adopt.py` and `bench/test_forward_adopt.py` changed). `#1257` merged at `e505407b06d6bdd2016e9824a34452c5dcb8f113`, `ready/bench-forward-adopt` was confirmed on `origin` at that exact commit via `run-signal.yml`, and `#1270` queued `bench-forward-adopt` in `bench/workstation-queue.txt` (`test_workstation_queue.py` 4/4).
**What changed:** set `forward-emission-adopt` to `blocked` awaiting the archived `origin/evidence` bundle (`forward-adopt/run-*.tar.gz`) and released the live lease so other `src/open.rs` work is not held while the workstation runs.
**What's next:** verify SHA-256 and score the returned bundle strictly against the frozen C6 `stop_test` (noting the accepted parity reading from `2026-11-28-forward-emission-adopt-c6-review.md` and `wikidata/6` null-cell note); on `ADOPT`, promote `forward-anchor-lanes` to the default `OpenCells` walk.
**Caveats:** no workstation timing has run yet; no production code changed in `#1257` or `#1270`.
