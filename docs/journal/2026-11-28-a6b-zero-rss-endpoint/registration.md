# Zero-extra-RSS `A6b` (`overlay_aos`) build: i9-10900K adoption run (registration)

*Frozen 28 November 2026, before any physical workstation data, under the claim [`a6b-zero-rss-endpoint-build`](../../../frontier/claims/a6b-zero-rss-endpoint-build.toml). This is an adoption registration (RULES C6). It decides whether `overlay_aos` merges into the default decoder, so one independent review by an agent other than its author is required before its ready tag is set. Frozen verdicts are never relaxed after data (C2).*

## 1. What is compared

- **Candidate:** the ready-tagged commit of this registration's PR. It is `BASE` plus `src/open/endpoint.rs` (`#![forbid(unsafe_code)]`, `mod overlay_aos`) and `src/open.rs` (routing `Narrow` Open blocks with `ns >= 64` and `n_nodes <= 32,768` through `induced_overlay_aos`).
  - Unified 32-byte AoS nodes `[prev0, next0, prev1, next1, word_lo, word_hi, width, pad]` (`word` 8-byte-aligned at `+0x10`, 1 half cache line per node) + inline-7 `u64` words + rules + `16 KiB` `contents[4096]` cache all overlaid inside `Kernel.edges` with `reserve_exact`-only growth (`0 B` extra `Endpoint` heap, `Endpoint::scratch() == 0`).
  - Exact `n_nodes` tight power-of-two `CAP` (`2K..32K`, fallback to `Narrow` above `32,768`), register-resident `(x, y)` tails, `q == 1` adjacent link-write no-op, cold-only `rules.room()` check, and `[u32; 64]` stack DFS in `append_fast`.
- **Control:** `BASE = 6679ddeed640f9e9cdd6aa3f71e80a7fb43326b7`, `master` when frozen.
- **Source identity (VOID on violation):** the recipe refuses to run unless the working tree has no tracked changes, `BASE` is an ancestor of `HEAD` (or on a `--depth 1` shallow checkout, `BASE` is fetched at depth 1 and ancestry skipping is recorded in `provenance.json`), and the ONLY files differing between `BASE` and `HEAD` under `src/`, `examples/`, `benches/`, `Cargo.toml`, `Cargo.lock`, and `build.rs` are `src/open.rs` and `src/open/endpoint.rs`. The base binary is built in an isolated `git worktree` at `BASE` kept alive until `run()`'s `finally:` block (preventing `CARGO_MANIFEST_DIR` loss during `base-ab gen`).

## 2. Correctness, environment, and codegen identity (VOID if any fails; never STOP)

- **Exact output:** untimed `AB_SHA256=1` `ab child` on every `(arm, corpus, k)` cell in the grid, both `base` and `cand`, must match C `libbzip2`'s exact byte count and SHA-256 digest before any timed sweep starts. Every timed row validates `out`, `hash`, `err == ""`, and finite positive `wall`, `cpu`, and `rss_kb`.
- **Codegen & binary identity:**
  - `base_ab` and `cand_ab` are built in isolated `CARGO_TARGET_DIR`s and must have distinct SHA-256 digests.
  - `cand_ab` (`nm -C` + `objdump -dC`) contains `mibz2::open::endpoint::overlay_aos::elim_aos::<131072, false>` with `0` `panic_bounds_check` calls; `base_ab` contains `0` `overlay_aos::elim_aos` symbols. Both disassemblies and `identity.json` are archived in the run directory.
- **Prerequisites (VOID on violation):**
  - Unrestricted 10-core / 20-CPU `i9-10900K` (`lscpu` + `lscpu -e` + `sched_getaffinity`) on registered runs.
  - No CPU quota (`check_no_quota` accepts `None` or `{"state": "unlimited", "cpus": None, ...}`, fails closed on missing/limited/unknown/numeric).
  - Rust `1.99.1` (`rustc -Vv`).
  - Baseline environment: no `RUSTFLAGS`, `CARGO_BUILD_RUSTFLAGS`, `CARGO_TARGET_*_RUSTFLAGS`, or `CARGO_PROFILE_RELEASE_*` overrides set.
  - No CLI parameter overrides on registered runs (`score_dir` returns `VOID` for `--smoke` or any non-registered `rounds`, `base`, `settle_s`, or `prespin_ms`).

## 3. Grid

- **Pairing:** `ab run --arms <arm>,<arm>@<cand>` with fresh child processes, `2 s` settle, `100 ms` prespin, `--max-busy 1.5`, `--pin` at core 0's first CPU, `--machine-state` counters, and `12 paired rounds` (alternating arm order each round).
- **Cells (20 total; 480 timed processes, ~18–22 min on i9-10900K):**
  - `mibz2-workers-lazy` on `wikidata` (`0b820d14…ce08e`) at `k = 1, 6, 8` (3 cells — exercises `OpenCells` & `A6b` at `1W`, `6W`, and `8W`; note `mibz2` at `k = 1` runs eager `bwt::Cells` which does not enter `OpenCells`).
  - `mibz2` (production default) on `wikidata` at `k = 1, 4, 6, 8, 10` (5 cells).
  - `mibz2` (production default) on all 6 `ab gen` scale-4 guard corpora (`text`, `records`, `random`, `runs`, `pattern`, `onebyte`) at `k = 1, 8` (12 cells).

## 4. Decision (frozen, matching `a6b-zero-rss-endpoint-build.toml` `stop_test` Clause 2)

Per-cell wall and CPU statistics are paired geometric-mean ratios (`cand / base`) with 95% t-intervals (`mtf_fold.paired_ratio_ci`). The memory statistic is the median of paired peak-RSS (`rss_kb`, `ru_maxrss` / `VmHWM`) ratios. The verdict is **PASS** iff all four rules hold, otherwise **STOP**:

1. **Single-worker `OpenCells` gain (`wikidata/1`):** `mibz2-workers-lazy/wikidata/1` paired geometric-mean wall ratio `≤ 0.985` (`>= 1.5%` faster) AND its 95% CI upper bound `< 1.000`.
2. **Multi-worker `wikidata/6` & `wikidata/8` (`mibz2` and `mibz2-workers-lazy`, no `#1053`-style 8W L3 cliff):** across all four cells (`mibz2/wikidata/6`, `mibz2/wikidata/8`, `mibz2-workers-lazy/wikidata/6`, `mibz2-workers-lazy/wikidata/8`), paired geometric-mean process `cpu` ratio (`getrusage`, high-precision scanner+worker busy CPU) `≤ 0.990` AND paired geometric-mean `wall` ratio `≤ 1.000`. (Parsed per-thread `worker_thread_cpu` from `thread_cpu` is also recorded diagnostically.)
3. **Zero-extra-RSS bound:** median paired `rss_kb` ratio `≤ 1.005` across every cell in the 20-cell grid (`1W, 4W, 6W, 8W, 10W` on `wikidata` and `1W, 8W` on all 6 guard corpora).
4. **Zero guard or non-Open regressions:** paired geometric-mean `wall` ratio `≤ 1.010` across all 12 guard cells (`text`, `records`, `random`, `runs`, `pattern`, `onebyte` at `k = 1, 8`) and non-Rule-1/2 Wikidata cells (`mibz2/wikidata/1`, `mibz2/wikidata/4`, `mibz2/wikidata/10`).

Position-stratified ratios, bootstrap median CIs, and `worker_thread_cpu` ratios are archived diagnostically per cell.

## 5. Outcomes

- **PASS:** PR merges into `master` once CI (including Miri, fuzz-smoke, wheels, and clippy) is green and independent review has no open objection; ledger and claim updated.
- **STOP:** PR is not merged; a scoped row goes in `frontier/dead-ends.tsv` citing the `origin/evidence` bundle SHA. Never rerun unchanged.
- **VOID:** fix the failed prerequisite and rerun under a new ready tag, logging why in a delta.
