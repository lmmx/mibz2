# Frozen Diagnostic PMU Registration: 8-worker scanner contention residual (`scanner-contention-residual`)

- **Claim:** `frontier/claims/scanner-contention-residual.toml` (P3, `probe`, `smooth`)
- **Recipe:** `just submit bench-scanner-contention` (`.just/scanner-contention.just` -> `bench/scanner_contention.py`)
- **Prior evidence (motivating probe):**
  - `docs/notebook/2026-11-28-explore-scanner-inflation-is-clock.md`: Comet Lake turbo bin drops from `5.28 GHz` (1 core active) to `4.91 GHz` (`>= 4` cores active), explaining ~75% of base's and ~50% of `forward-anchor-lanes`'s apparent 1W->8W scanner busy-time rise in `origin/evidence e98948b6`. The clock-adjusted residual is ~`+4.5 ms` (`+2.7%`) on base and ~`+11.1 ms` (`+6.6%`) on `forward-anchor-lanes`, measured against an `n = 1` 1W baseline without per-thread PMU counters.

---

## 1. Question and mechanism under test

Measure per-thread hardware PMU counters on the pinned `"mibz2-scan"` thread alone at `k ∈ {1, 4, 8}` workers for `base` vs `forward-anchor-lanes` built from the exact same commit on Louis's physical i9-10900K:

- Does the scanner thread consume genuinely more clock-normalized cycles at `k = 8` than at `k = 4` (where both runs sit on the same all-core `4.91 GHz` turbo bin)?
- If so, does at least `50%` of the `8W - 4W` cycle excess appear in `cycle_activity.stalls_l3_miss` (confirming shared L3 / slab-pool / DRAM miss contention), or in another microarchitectural component?

---

## 2. Experimental design

- **Machine & toolchain:** Louis's physical Intel Core i9-10900K (unrestricted 10 physical cores / 20 CPUs, unlimited cgroup v2 quota checked via dict-safe `ab env --facts`, `rustc 1.99.1`), clean git checkout, baseline x86-64 codegen (`RUSTFLAGS`, `CARGO_BUILD_RUSTFLAGS`, `CARGO_ENCODED_RUSTFLAGS`, `CARGO_PROFILE_RELEASE_*` all unset).
- **Arms (same commit, isolated `CARGO_TARGET_DIR` per arm):**
  - **`base`:** `--features scan-oracle` (`examples/scanner_coalesced_run`, plus `ab` and `windows` for corpus materialization and env facts).
  - **`lanes`:** `--features "scan-oracle forward-anchor-lanes"` (`examples/scanner_coalesced_run`).
  - **Distinct-binary VOID guard:** `base_run.sha256 != lanes_run.sha256` required.
- **Corpus:** `wikidata-x4` (verified SHA-256 `0b820d14299af637bf7f644d172f5efe91ad59e169cf2a50bd0962f5c89ce08e`, `838,800,784 B`), shallow-safely materialized from local cache or `--depth=1 --no-tags` `origin bench-data`.
- **Placement:** scanner thread (`mibz2-scan`) pinned to physical core 0 (`cores[0][0]`) via `scan_oracle::set_scanner_cpu`; workers constrained via `taskset -c <mask>` excluding core 0 and its SMT sibling (`cores[0][1]`). Every sample checks `scanner_cpu == pin`, `bmi2 == true`, `out == ref["out"]`, and `scanned == ref["scanned"]`.
- **Per-thread PMU events (user-space `:u`, no `sudo` required, `<= 4` programmable counters so zero multiplexing):**
  - `cycles:u`
  - `ref-cycles:u`
  - `instructions:u`
  - `cycle_activity.stalls_l3_miss:u`
  - `LLC-load-misses:u`
  - Captured via `perf stat --per-thread -x, -o <csv> -e <events> -- taskset -c <mask> <exe> <wikidata> <k> <scanner_cpu> real time` and filtered strictly to the unique `mibz2-scan-<tid>` thread.
- **Grid & settle:** `ARMS = (base, lanes)`, `COUNTS = (1, 4, 8)` (`6` cells), **`ROUNDS = 12`** (`>= 9`) interleaved rounds (all `6` cells shuffled within each round, seed `1301`, `72` timed processes), **`2.0 s` settle** (`SETTLE_S = "2"`, matching `ab`'s default settle) before every sample.
- **Exactness gate:** pre-sweep untimed `check` runs piped to `sha256sum` at `k = 1` and `k = 8` for both `base` and `lanes` verified byte- and SHA-256-exact against C `libbzip2`.

---

## 3. Frozen Decision Rule (`CONFIRMED` / `REFUTED` / `INCONCLUSIVE` / `VOID`)

**`VOID`** (exit 1) if any prerequisite, quota, host, distinct-binary, parity, `mibz2-scan` thread matching, PMU counter validity, or grid completeness check fails (or in `score`, if `provenance.json` does not match registered parameters).

For each arm (`base`, `lanes`), pair round `r`'s `k = 8` and `k = 4` `mibz2-scan` cycle counts (`r = 0..11`) and compute the paired geometric-mean 95% Student-t CI `[lo, hi]` for `cycles_8w / cycles_4w`, plus the paired mean excess cycles `Δcycles = mean(cycles_8w - cycles_4w)` and paired mean L3-miss stall excess `Δstalls_l3 = mean(stalls_l3_8w - stalls_l3_4w)`:

1. **`CONFIRMED`** if `lo_lanes > 1.0` (the 8W-over-4W scanner-cycle ratio's 95% CI excludes `1` above `1` for `lanes`) **AND** `Δcycles_lanes > 0` with `Δstalls_l3_lanes / Δcycles_lanes >= 0.50` (at least half of the excess shows up in `cycle_activity.stalls_l3_miss`).
2. **`REFUTED`** if BOTH `base` and `lanes` have 95% CIs including `1.0` (`lo_base <= 1.0 <= hi_base` and `lo_lanes <= 1.0 <= hi_lanes`).
3. **`INCONCLUSIVE`** otherwise (e.g. `lanes` 8W-over-4W CI excludes `1` but `< 50%` of the excess is in L3-miss stall cycles, implicating a non-L3-miss resource; or `base` excludes `1` while `lanes` includes `1`).

Either way, record the full per-`k` scanner cycle, clock (`cycles / ref-cycles`), IPC, `stalls_l3_miss`, and `LLC-load-misses` split in `docs/notebook/` and `frontier/DECODER.md`.
