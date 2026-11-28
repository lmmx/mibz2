claim: forward-emission-adopt
by: antigravity
outcome: review
evidence: branch build/forward-emission-adopt rebased onto origin/master e435074c (post-#1246 + #1255); pytest bench/test_forward_adopt.py (9 passed, 9 subtests passed); live --smoke + score round-trip on origin/master e435074c (base ab 12454d6f vs cand ab 837f8996, 12/12 AB_SHA256=1 parity rows exact vs libbzip2, 24/24 timed smoke rows exact); frontier.py check 0 errors

**What was asked:** prepare, verify, and submit the C6 physical i9-10900K full-guard adoption registration and workstation recipe (`bench-forward-adopt`) for promoting the `#![forbid(unsafe_code)]` disjoint-slab four-lane forward walk (`--features forward-anchor-lanes`, `#1110` + `#1217`) to the default `OpenCells` walk on current `origin/master` (post-`#1246`).

**What changed:**
- `docs/journal/2026-11-27-forward-adopt/registration.md`: frozen C6 adoption registration matching `frontier/claims/forward-emission-adopt.toml` verbatim (`wikidata-x4` at `k = 1, 6, 8, 10, 12`, all 6 guard corpora `text`/`records`/`random`/`runs`/`pattern`/`onebyte` at `k = 1, 8`, `ROUNDS = 16`, `--settle 2`, `--prespin-ms 100`, `--machine-state`, isolated `CARGO_TARGET_DIR` per arm, distinct-binary VOID guard, `AB_SHA256=1` parity vs C `libbzip2`, and exact wall + RSS adoption thresholds).
- `bench/forward_adopt.py`: executable adoption driver + scorer (including audit fix in `_extract_cpr` to read top-level `cycles_per_ref` emitted by `examples/ab.rs --machine-state` with fallback to nested `machine.cycles_per_ref`, and unambiguous guard-failure diagnostics).
- `bench/test_forward_adopt.py`: 9 unit tests / 9 subtests covering `ADOPT`, every `DO NOT ADOPT` wall/guard/RSS threshold, all `VOID` conditions, isolated target dirs + distinct-binary enforcement, `RUSTFLAGS` rejection, `score` directory round-trip, and `Justfile` / `.just/forward-adopt.just` wiring.
- `.just/forward-adopt.just` + `Justfile` import: `just submit bench-forward-adopt` recipe pinning `RUSTUP_TOOLCHAIN=1.99.1`, writing into `target/forward-adopt/run-<UTC>`, teeing `console.log`, and bundling via `bench/bundle.sh`.

**Verification:**
- `ruff check` and `ruff format --check` clean on `bench/forward_adopt.py` and `bench/test_forward_adopt.py`.
- `pytest -v bench/test_forward_adopt.py`: 9 passed, 9 subtests passed.
- Live `--smoke` + `score` round-trip against `origin/master` `e435074c` (post-`#1246`): `base` (`default`, SHA-256 `12454d6f...`) and `cand` (`--features forward-anchor-lanes`, SHA-256 `837f8996...`) built cleanly in isolated target directories, passed `distinct_binaries: true`, passed `12/12` untimed `AB_SHA256=1` parity checks against C `libbzip2`, passed `24/24` timed `ab run` exactness checks, and re-scored identically via `bench/forward_adopt.py score`. (Local 2-vCPU VM smoke timing is uncalibrated and non-decisive.)
- `python3 frontier/tools/frontier.py check`: 0 errors.

**What's next:** one independent review by a different agent (C6 requirement before queueing on the physical i9), followed by merging this registration PR and queueing `bench-forward-adopt` on Louis's i9-10900K. Do not self-review or queue from the authoring session.
