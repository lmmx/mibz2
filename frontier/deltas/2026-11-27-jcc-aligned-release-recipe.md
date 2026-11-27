claim: jcc-aligned-release-build
by: antigravity
outcome: partial
evidence: docs/journal/2026-11-27-jcc-aligned-release/registration.md; #1077 (c1239ce0); #1107 (e4bef3e6); #1109 (578e5951)

## Summary
Implemented the frozen C6 adoption registration and executable i9-10900K recipe (`just bench-jcc-aligned-release`) for `jcc-aligned-release-build`, comparing baseline x86-64 + `-C llvm-args=-x86-branches-within-32B-boundaries` against default baseline x86-64 (zero `-C target-cpu` flags, per `#1107` / `#1109`) at pinned `master` commit `59cac2c52d7168e09a4b4d6edeb658c16ef9baf7`.

## Frozen registration & recipe (`bench/jcc_aligned.py`, `.just/bench.just`)
- **Identical shallow-safe builds + static codegen proof:** Verifies `HEAD:src`, `HEAD:Cargo.toml`, and `HEAD:Cargo.lock` match `59cac2c5` and builds directly from checked-out `HEAD` into separate `CARGO_TARGET_DIR`s under pinned `rustc 1.99.1` (`--release --locked`, zero `git clone --local` or history fetches so `--depth 1` i9 checkouts work cleanly; isolated `git init` + `fetch --depth 1` for `bench-data`), auditing `.text` growth + `objdump` unfused 32-byte JCC/JMP crossings in `mibz2::*` (`default > 0`, `aligned == 0`, distinct SHA-256) into `codegen_audit.json`.
- **Exact byte parity:** Untimed `AB_SHA256=1` `ab child` check against independent Python `bz2` (`libbzip2`) on all 14 `(corpus, k)` cells × 2 arms before timing, plus per-row `termination == "success"`, `err == ""`, `out`, and `hash` checks on every timed `ab run` record.
- **Frozen C2 rule (zero relaxation):** 12 balanced `ab run` rounds (`--prespin-ms 100 --settle 0.2`) across `wikidata x4` (`k = 1, 4, 6, 8, 10`) and guard corpora `text`, `records`, `random` (`k = 1, 8, 10`). PASS iff `wikidata/1` 95% CI upper `<= 0.970`, `wikidata/8` 95% CI upper `<= 0.985`, `wikidata/10` and every guard cell point ratio `<= 1.010`, and peak RSS `<= 1.020×` everywhere. Records diagnostic `k = 4, 6` and 5 paired rounds of 1T `examples/station_balance.rs` (`S`, `W`, `k*`, `r_aligned`) to settle i9 `k = 8` attribution.
- **Part B CI AMD gate preserved:** `summary.json` marks `amd_ci_gate: "pending_release_pr_ci_bench"`; no release flags (`pyproject.toml` / `.github/workflows/CI.yml`) or `bench/workstation-queue.txt` are touched in this PR.

## Local validation
- `python3 -W error -m unittest discover -s bench -p 'test_jcc_aligned.py' -v`: 12/12 tests pass (including genuine `git clone --depth 1` shallow-checkout test).
- `ruff check bench/jcc_aligned.py bench/test_jcc_aligned.py`: clean.
- Live Rust 1.99.1 smoke run: `ab` `.text` `1,654,406 -> 1,703,030 B` (`+2.94%`), `mibz2::*` unfused 32B crossings `2,254 -> 0`; `station_balance` `.text` `412,040 -> 426,664 B` (`+3.55%`), crossings `354 -> 0`; `AB_SHA256=1` libbz2 parity (`5,997,004 B`, SHA-256 `15959207603b...`) and `station_balance` (`k* 5.38 -> 5.17`) verified on `text.bz2`. Ready for independent C6 review (`status = "review"`).
