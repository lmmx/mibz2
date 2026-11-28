claim: plan-schedule-and-lazy-threshold
by: antigravity
outcome: review
evidence: branch build/plan-schedule-and-lazy-threshold rebased onto origin/master f8b0f0bb; zero new unsafe in src/; cargo fmt --check & cargo clippy --all-targets -- -D warnings PASS; 93/93 Rust release tests PASS (parallel:: 54, decode 14, in_place 7, pipelined 12, service_default 4, lazy_workers 1, cells_default 1); ruff check/format clean; pytest bench/test_plan_schedule.py 23/23 PASS; live plan-schedule --smoke + score round-trip + strict-provenance VOID check against origin/master PASS (base ab f60d3c57 vs head ab 9d18a16d, 10/10 AB_SHA256=1 libbzip2 parity rows, 24/24 timed smoke rows, wikidata RSS 0.36-0.46x at 4T-7T); frontier.py check 0 errors

**What was asked:** implement `plan-schedule-and-lazy-threshold` in `src/parallel.rs` (zero new `unsafe`) and prepare the pre-registered physical i9-10900K verification recipe (`stop_test` Clause 2) for independent C6 review prior to workstation queueing.

**What changed:**
- `src/parallel.rs` & `tests/service_default.rs` (zero new `unsafe`):
  - Lowered `LAZY_FROM_THREADS` from `8` to `4` so `OpenCells` is selected from `4` workers onward across all machines, containers, and affinity masks (`1..=3` remain eager `Cells`), eliminating the redundant `(UNPINNED_LAZY_FROM..LAZY_FROM_THREADS)` branch.
  - Added measured i9-10900K pinned odd-count trims (`TRIMMED_AT_FIVE = 4`, `TRIMMED_AT_SEVEN = 6`, `TRIMMED_AT_NINE = 7`) alongside existing `4T/6T/8T/>=10T` cells to eliminate the `v0.5.15` `5T`/`7T`/`9T` crossover sawtooth cliffs.
  - Generalized `window()` to scale slot counts whenever `n < threads` (not only when `trimmed`), preventing the 80-slot blowup on capped non-i9 machines at high thread counts.
  - Updated `parallel::` and `tests/service_default.rs` unit/integration tests for the new schedule and monotonicity/cap-scaling invariants.
- Pre-registered physical i9-10900K C6 verification recipe (`.just/plan-schedule.just`, `Justfile`, `bench/plan_schedule.py`, `bench/test_plan_schedule.py`, `docs/journal/2026-11-27-plan-schedule-i9/registration.md`):
  - `just submit bench-plan-schedule` runs the exact 22-cell frozen `stop_test` (`wikidata x4` pinned `4..=10T`, unpinned `4/8/10/12/16/20T`, plus `text`/`records`/`random` pinned `4/6/8T`, 12 order-balanced rounds, `--settle 2.0` per `#1240`, `--prespin-ms 100`, `--machine-state`, isolated `CARGO_TARGET_DIR` builds, distinct-binary VOID guard, `AB_SHA256=1` parity vs C `libbzip2`, `cpu_quota` unlimited check, strict frozen-parameter lock, and `provenance.json` validation in `cmd_score`).
  - Hardened `checkout_ref_shallow` to strip `origin/` when falling back to `remote_url` on shallow workstation checkouts.

**Verification:**
- `git diff origin/master -- src/`: zero `unsafe` additions.
- `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, and `93/93` release tests pass (`parallel::` 54, `decode` 14, `in_place` 7, `pipelined` 12, `service_default` 4, `lazy_workers` 1, `cells_default` 1).
- `ruff check`, `ruff format --check`, and `pytest -v bench/test_plan_schedule.py` (`23/23` pass).
- Live `--smoke` + `score` round-trip against `origin/master` (`base` `f60d3c57` vs `head` `9d18a16d`): `10/10` `AB_SHA256=1` libbzip2 parity rows exact, `24/24` timed smoke rows exact, `wikidata` peak RSS `0.36-0.46x` at `4T-7T`, and strict `cmd_score` correctly returns `VOID` (exit `2`) on non-registered smoke provenance. (2-vCPU VM smoke wall times are uncalibrated and non-decisive.)
- `python3 frontier/tools/frontier.py check`: 0 errors.

**What's next:** one independent C6 review by a different agent (verifying both `src/parallel.rs` and the pre-registered `bench-plan-schedule` recipe), followed by queueing `bench-plan-schedule` on Louis's i9-10900K against `origin/master` to adjudicate `stop_test` Clause 2 before integration. Do not self-review, merge, or queue from the authoring session.
