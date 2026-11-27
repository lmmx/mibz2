# Pre-data registration: separate the i9's two clock modes and make `ab` comparisons mode-robust (`i9-clock-modes`)

- **Claim:** `frontier/claims/i9-clock-modes.toml` (P2, `probe`, `striated`)
- **Date:** 27 November 2026
- **Premises:**
  - `docs/notebook/2026-11-27-synthesis-fifteenth-pass.md` (§2)
  - `docs/journal/2026-10-28-workstation-scaling.md` (§4)
  - `docs/journal/2026-11-09-spin-audit.md`
  - `origin/evidence` `77236472` (`jcc-aligned-release`), `bce3914d` (`release-scaling`), `9de7b6e0` (`#1034`)
- **Recipe:** `just bench-clock-modes` (`.just/clock-modes.just` -> `bench/clock_modes.py`)
- **Status:** Frozen before any i9 measurement (C2 / C6).

## 1. Frozen stop test (verbatim from `frontier/claims/i9-clock-modes.toml`)

> Frozen before data. One i9 run, wikidata x4, the default decoder at one commit, 12 rounds, a 2x2x2 design: pinned vs unpinned, settle 0 vs 0.2 s, and pair position. Each child records per-thread cycles and instructions (or APERF/MPERF) and sampled CPU placement.
> PASS (mechanism found) if one factor (or one two-way interaction) explains at least 80% of slow-mode runs, where a run is slow if its cycles-per-byte stays within 3% of the fast mode's but its wall is at least 1.3x (6T) or 1.04x (1T), so frequency is separated from executed work.
> Also PASS (mode is real work) if slow runs have at least 1.2x the cycles of fast ones: the cause is contention or placement, not clock.
> Otherwise STOP with the table recorded. In every case: document a mode-robust estimator (stratify by position, or report within-mode ratios classified by cycles), and restate the 0.5.15 1T and 12T rows in CHANGELOG.md and docs/journal/release-attribution.md with within-mode figures beside the pooled ones. That restatement is done by an agent other than the 0.5.15 release's author.

## 2. Question and motivation

Regrouping raw i9-10900K rows in the fifteenth synthesis pass (`docs/notebook/2026-11-27-synthesis-fifteenth-pass.md` §2) revealed two distinct bimodal regimes that scale every thread's CPU time by the same factor:

1. **Mode (a) (~5–10% at `1T`, pinned):** In `77236472` (`#1119`), `11/12` first-of-pair `1T` runs (immediately following the `10T` pair) ran slow (`1.448–1.461 s` vs `1.379–1.389 s`, ~`1.05x`). In `bce3914d` (`release-scaling`), pinned `1T` was bimodal (`0.350–0.367 s` fast vs `0.390–0.405 s` slow, ~`1.10x`), with `0.5.14` slow `8/10` times and `0.5.15` slow `3/10` times — inflating the pooled `1T` ratio from `0.998–1.021` within-mode to `1.057 [1.011, 1.104]`.
2. **Mode (b) (~1.45x at `6–10T`, unpinned only):** In `77236472` (`#1119`), unpinned `6T` ran slow `12/12` times in pair position 2 (`0.353–0.366 s` vs `0.241–0.250 s`, all threads scaled ~`1.45x`, matching the i9-10900K turbo/base ratio `5.3 / 3.7 = 1.43`), widening the pooled CI to `±25%`. Pinned bundles on the same day (`9de7b6e0`, `bce3914d` 4–10T) were unimodal.

This single discriminating i9 run isolates **pinning**, **settle gap**, and **pair position** (with optional `--prespin-ms` arm support) alongside in-child hardware counters (`cycles`, `instructions`, `ref_cycles`, `cycles_per_ref`) and sampled per-thread CPU placement, separating clock-frequency drops from real extra work (contention or placement).

## 3. Experimental design

- **Corpus:** `wikidata x4` (`4x` multistream concatenation of `corpora/wikidata.bz2`, verified SHA-256 `0b820d14299af637bf7f644d172f5efe91ad59e169cf2a50bd0962f5c89ce08e`, 838,800,784 B uncompressed, 47,419,732 B compressed), materialized shallow-safely via `corpora/wikidata.bz2` or `git fetch --depth=1 --no-tags origin bench-data` + `examples/windows.rs`.
- **Decoder & build:** Default `mibz2` arm at clean `HEAD`, built with `rustc 1.99.1` via `cargo build --release --locked --example ab --example windows` into a strictly isolated `CARGO_TARGET_DIR` with default baseline x86-64 codegen (`RUSTFLAGS`, `CARGO_BUILD_RUSTFLAGS`, `CARGO_ENCODED_RUSTFLAGS` unset).
- **Parity check (untimed, pre-sweep):** Independently verify decompressed byte count via Python `bz2.open` (libbzip2), verify `ab child libbzip2 1 <wikidata-x4>` matches, and verify `ab child mibz2 6 <wikidata-x4>` matches C `libbzip2`'s exact `(out, hash)` with `err == ""`.
- **Factorial grid (`2 x 2 x 2` at both `1T` and `6T`, `ROUNDS = 12`):**
  - **Threads:** `1` (targets mode (a)) and `6` (targets mode (b)).
  - **Pinning:** `pinned` (`taskset -c <core0_cpu0>` for `1T`; `taskset -c <first sibling of cores 0..5>` for `6T`) vs `unpinned` (all 20 CPUs, no `taskset`).
  - **Settle gap (`settle_s`):** `0.0 s` (immediate launch) vs `0.2 s` (`time.sleep(0.2)` before each run in the pair).
  - **Pair position (`pair_pos`):** `0` vs `1` (each `(threads, pinning, settle_s)` cell executes two back-to-back runs in order `pos 0`, `pos 1`).
  - **Ordering:** Within each of the 12 rounds, the 8 `(threads, pinning, settle_s)` pair-cells are shuffled with deterministic seed `1189` (`16` runs/round, `192` timed runs total; estimated ~3–4 minutes on the i9-10900K, well under the 15-minute workstation budget).
  - **Optional secondary arm:** `--prespin-ms` accepts comma-separated values (default `0`; e.g., `--prespin-ms 0,100`) to test prespin alongside the core factors without altering the frozen default grid.

## 4. Per-child telemetry

Every timed child (`ab child mibz2 <t> <wikidata-x4>` with `AB_MACHINE_STATE=1`) records:

1. **Timing, CPU, memory & correctness:** `wall`, `cpu` (net of sampler), `user_s`, `sys_s` (via `os.wait4`), `rss_kb`, `parent_rss_kb`, `out`, `hash`, `sha256`, `threads_busy`, `thread_cpu`, `runq_s`, `err`.
2. **Hardware counters (frequency vs work):** `ab`'s in-child inherited `perf_event_open` counters (`cycles`, `instructions`, `ref_cycles`, `task_clock_s`, `eff_ghz`, `cycles_per_ref`, `cycles_per_byte`, `perf_multiplexed`, `perf_running`). If `ab_perf` is unavailable (e.g., on restricted VMs), `bench/clock_modes.py` falls back cleanly in order to `perf stat -x,` -> `/dev/cpu/*/msr` (APERF `0xE8` / MPERF `0xE7`) -> explicit `perf_available: false` with reason.
3. **Sampled per-thread CPU placement:** Parent sampler polls `/proc/<pid>/task/*/stat` (field 39, `processor`) at 5 ms intervals throughout every `1T` and `6T` child's lifetime, recording `sample_count`, `cpus_sampled`, `n_distinct_cpus`, `n_distinct_cores`, `smt_sibling_overlap_samples` (two decoder threads simultaneously on SMT siblings of one physical core), `migrations`, and `per_tid_cpus`.
4. **Best-effort sysfs state:** `scaling_cur_freq`, `scaling_governor`, `energy_performance_preference`, and `intel_pstate/no_turbo` before and after each child.

## 5. Frozen scoring and verdict rules

For each thread count $t \in \{1, 6\}$ independently:

1. **Fast-mode anchor:** Anchor `fast_wall_median` robustly on the lower cluster (seed at the 5th-percentile sorted wall; take the median of all runs below `seed * wall_threshold`, where `wall_threshold` is `1.04` for `1T` and `1.30` for `6T`).
2. **Per-run mode classification:**
   - `fast`: `wall < fast_wall_median * wall_threshold`.
   - Among wall-slow runs (`wall >= fast_wall_median * wall_threshold`), compare `cpb_ratio = (cycles / out) / fast_cpb_median`:
     - `slow_clock`: `|cpb_ratio - 1.0| <= 0.03` (clock/frequency drop; executed work unchanged).
     - `slow_work`: `cpb_ratio >= 1.20` (real extra work: contention or placement).
     - `slow_mixed`: wall-slow with `cpb_ratio` outside `±3%` and `< 1.20`.
     - `slow_unclassified`: wall-slow with PMU cycles unavailable.
3. **Factor / two-way interaction explanation:** Evaluate every single-factor level (`pinning`, `settle_s`, `pair_pos`) and every two-way interaction cell (`pinning x settle_s`, `pinning x pair_pos`, `settle_s x pair_pos`) for `explained_share = slow_clock_in_cell / n_slow_clock`, requiring `explained_share >= 0.80` and `slow_rate > complement_slow_rate`, with a minimum noise guard `min_slow_runs = 3` (so a single stray outlier out of 96 runs cannot falsely trigger `1/1 = 100%`).
4. **Verdict:**
   - **VOID** if any prerequisite or data-integrity invariant fails (non-i9 host, restricted affinity or CPU quota, dirty tracked tree, wrong `rustc`, non-empty `RUSTFLAGS`, corpus/parity mismatch, missing/duplicate cell, decode error, non-finite wall/RSS).
   - **PASS (mechanism found)** if either `1T` or `6T` has `>= 3` `slow_clock` runs and one factor or two-way interaction explains `>= 80%` of them.
   - **PASS (mode is real work)** if either `1T` or `6T` has `>= 3` wall-slow runs where slow runs execute `>= 1.2x` the cycles of fast runs (`slow_work` >= 50% of wall-slow or median `cpb_ratio >= 1.20`).
   - **STOP** otherwise, with the full per-cell table and ANOVA `eta^2` recorded.

## 6. Mode-robust estimator for `ab` comparisons

`python3 bench/clock_modes.py analyze <runs.jsonl>` implements two complementary mode-robust estimators beside the standard pooled paired geometric mean:

1. **Position-stratified estimator:** Infers within-round appearance position `pair_pos` for each `(round, corpus, threads)` group, computes the per-position arm/ref ratio and 95% Welch log-t interval, and combines strata with sample-size weights. On `77236472` (`#1119`), this removes the alternating pair-position slow mode at both `1T` and `6T`.
2. **Within-mode estimator:** Classifies runs into `fast` vs `slow` using `cycles_per_ref` (when present) or `cycles_per_byte` + wall (or a deterministic 1D wall-gap split when PMU counters were not recorded, tagged `basis = "wall_only"` as in `bce3914d`), reporting per-arm mode occupancy, within-fast and within-slow ratios with 95% CIs, and same-mode matched-round paired ratios.

## 7. Evidence bundle

`just bench-clock-modes` writes one timestamped run directory under `target/clock-modes/run-<UTC>` containing `console.log`, `registration.md`, `provenance.json`, `lscpu.txt`, `lscpu-e.txt`, `env.json`, `binaries.json`, `inputs.json`, `parity.jsonl`, `runs.jsonl`, `summary.json`, `summary.txt`, `outcome.txt`, and `toolchain.txt`, packed into one tarball via `bash bench/bundle.sh`.
