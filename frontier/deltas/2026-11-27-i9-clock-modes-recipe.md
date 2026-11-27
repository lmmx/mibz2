claim: i9-clock-modes
by: antigravity
outcome: partial
evidence: docs/journal/2026-11-27-i9-clock-modes/registration.md

## Frozen registration, shallow-safe i9 recipe (`just bench-clock-modes`), and mode-robust `ab` estimator

- **Claim:** `i9-clock-modes` (admitted in `#1198` at `14d656df`; `stop_test` preserved verbatim under C2).
- **Files:** `docs/journal/2026-11-27-i9-clock-modes/registration.md`, `Justfile` (`import ".just/clock-modes.just"`), `.just/clock-modes.just`, `bench/clock_modes.py`, `bench/test_clock_modes.py`.
- **Factorial probe (`2×2×2` at both `1T` and `6T`, `12` rounds, `192` timed runs):**
  - Target corpus: `wikidata x4` (`0b820d14...`, 838.8 MB raw), materialized shallow-safely via local `corpora/wikidata.bz2` or `git fetch --depth=1 --no-tags origin bench-data` + `examples/windows.rs` into a private temporary repo.
  - Factors: `pinning` (`pinned` core 0 / cores 0..5 first siblings vs `unpinned` all 20 CPUs) × `settle_s` (`0.0` vs `0.2` s) × `pair_pos` (`0` vs `1` back-to-back), shuffled per round with deterministic seed `1189`, plus optional `--prespin-ms` arm.
  - Strictly isolated `CARGO_TARGET_DIR` (`rustc 1.99.1`, baseline x86-64 codegen with `RUSTFLAGS`/`CARGO_BUILD_RUSTFLAGS`/`CARGO_ENCODED_RUSTFLAGS` rejected), i9-10900K 10C/20T affinity + unlimited cgroup CPU quota + untimed `libbzip2` parity checks.
  - Per-child telemetry: `ab child mibz2 <t>` under `AB_MACHINE_STATE=1` (`cycles`, `instructions`, `ref_cycles`, `cycles_per_ref`, `wall`, `cpu`, `rss_kb`, `out`, `hash`, `thread_cpu`, `runq_s`), `os.wait4` `user_s`/`sys_s`, graceful PMU fallback (`ab_perf` -> `perf stat` -> `/dev/cpu/*/msr` APERF/MPERF -> explicit `perf_available: false`), best-effort sysfs snapshots, and 5 ms parent `/proc/<pid>/task/*/stat` per-thread CPU placement sampler (`cpus_sampled`, `n_distinct_cpus`, `n_distinct_cores`, `smt_sibling_overlap_samples`, `migrations`, skipping `taskset`/`perf` wrapper comms).
- **Frozen `stop_test` scoring (`score`):** classifies slow runs (`>= 1.30x` at `6T`, `>= 1.04x` at `1T` vs fast-cluster median) into `slow_clock` (`|cpb_ratio - 1.0| <= 3%`), `slow_work` (`cpb_ratio >= 1.20x`), `slow_mixed`, or `slow_unclassified`; returns `PASS (mechanism found)` if one factor or two-way interaction explains `>= 80%` of `slow_clock` runs (`min_slow_runs = 3`), `PASS (mode is real work)` if slow runs execute `>= 1.2x` cycles, `STOP` with full table & `eta^2` otherwise, and `VOID` on any prerequisite/integrity violation.
- **Mode-robust `ab` estimator (`analyze`):** computes pooled paired ratio + 95% CI, position-stratified ratio + 95% CI, and within-mode ratio + 95% CI (`cycles_per_ref` / `cycles+wall` / `wall_only` fallback) on any `ab` `runs.jsonl`. Verified directly on archived i9 bundles:
  - `origin/evidence bce3914d` (`0.5.14 / 0.5.15`): `1T` pooled `1.0567 [1.0115, 1.1039]` -> slow occupancy `8/10` vs `3/10`, within-fast `1.0175`, within-slow `1.0019`, same-mode paired `1.0159`; `12T` pooled `1.095 [1.0303, 1.1638]` -> slow occupancy `3/10` vs `0/10`, within-fast `1.043`.
  - `origin/evidence 77236472` (`#1119` `aligned / default`): `1T` pooled `1.0065 [0.9753, 1.0387]` -> `pos=1` `1.0051 [1.0039, 1.0063]` (median `1.0054`); `6T` pooled `1.0064 [0.7778, 1.3022]` -> `pos=0` `1.013 [1.0004, 1.0257]` (median `1.0076`), `pos=1` `0.9999 [0.9833, 1.0168]` (median `1.001`).
- **Verification:** `pytest bench/test_clock_modes.py -v` **20/20 PASS**; `ruff check`, `ruff check --select E,F,W,I,B,UP`, and `ruff format --check` clean; end-to-end local `--smoke` run with freshly built `ab` in isolated `CARGO_TARGET_DIR` exited 0 producing all 12 verified bundle artifacts; `frontier.py check` 0 errors, 0 warnings.
- **Next:** independent C6 review, merge, and workstation queueing under the standing i9 authority, followed by reading the i9 bundle against the frozen stop test and restating the `0.5.15` `1T`/`12T` rows.
