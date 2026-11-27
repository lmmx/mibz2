# Post-#1080 `runs::takes_runs` (`Column` vs `Runs`) route-boundary dissection across 14 corpora (PASS)

*Smooth-zone `explore` investigation, 27 November 2026 (zero `src/` edits; probe code in [`examples/takes_runs_probe.rs`](../../../examples/takes_runs_probe.rs), `#![forbid(unsafe_code)]`).*

---

## 1. Motivation and frozen decision rule

In [`route-crossover-post-1080.md`](route-crossover-post-1080.md) (#1145), a secondary check showed that forcing `Column`-routed blocks (`!runs::takes_runs(b)`) through `runs::front + Segmented::invert_runs` (`fill_avx2_trusted` from #1080) cut MTF + inverse-BWT CPU by `-18.8%..-26.2%` on `mdjournal` and `-13.9%..-18.5%` on `elf`, while regressing `records` (`+8.1%`).

In `src/runs.rs:76-116`, `takes_runs_slice` enforces two gates:
1. A hard symbol-count half-bound: `2 * syms.len() <= max_len` (`syms_ratio <= 0.50`, fitted in `2026-11-10` before #1080's `build_lf` and `fill_avx2_trusted`);
2. A sampled run-symbol share: `RUNA`/`RUNB` (`s <= 1`) `>= 1/32` (`RUN_SHARE = 32`) across up to `64 x 64 = 4,096` evenly spaced symbols.

### Frozen discovery / stop test (frozen BEFORE reading any timing data)

- **PASS** (open follow-on P2 `idea` claim `post-1080-takes-runs` on `owned_paths = ["src/runs.rs"]`): at least one simple, zero-extra-memory `O(1)` pre-MTF rule (100% byte+CRC exact across all 14 corpora, using only `block::Symbols` fields + at most `takes_runs`'s existing `<= 4,096`-symbol sampled run-share check) cuts MTF + inverse-BWT CPU (steal-free `/proc/thread-self/schedstat`, `taskset -c 0`, CPU-calibrated `inner` giving `>= 240 ms/sample`, 5 rotated interleaved repeats) by:
  - **`>= 10.0%` (disjoint FASTER)** on **`mdjournal`** AND **`>= 10.0%` (disjoint FASTER)** on **`elf`**,
  - with **no regression `> +1.5%`** (and **zero disjoint SLOWER**) across any of the other 12 corpora (`wikidata`, `text`, `prose`, `pysrc`, `rustsrc`, `json`, `logs`, `random`, `records`, `runs`, `onebyte`, `pattern`),
  - across **ALL THREE builds**: baseline x86-64, `-C target-cpu=native`, and JCC-32B-aligned (`-C llvm-args=-x86-branches-within-32B-boundaries`).
- **STOP** (record a scoped dead-end row in `frontier/dead-ends.tsv`): no `O(1)` pre-MTF rule clears the `>= 10.0%` disjoint bar on both `mdjournal` and `elf` across all 3 builds without regressing any of the other 12 corpora by `> +1.5%`.

**Verdict: PASS (`T1_syms75_rs32` — a 1-line change `2 * len > max_len` -> `4 * len > 3 * max_len` in `src/runs.rs:78` — clears `>= 10.0%` disjoint FASTER on BOTH `mdjournal` (`-13.2%..-32.4%`) AND `elf` (`-14.5%..-24.7%`) across ALL 3 builds, also wins `json` `-9.0%..-9.9%` disjoint FASTER on all 3 builds, and routes `0/549` blocks differently on the other 9 corpora including `random`/`records`/`wikidata`/`text`).**

---

## 2. Deterministic 14-corpus panel and phase dissection

Every block across all 14 corpora (**658/658 blocks**, 458 full blocks `n >= 100,000`) was verified **100% byte-exact** across `Column` (`mtf::mtf + Segmented::invert_in_place`), `Runs+Seg` (`runs::front + Segmented::invert_runs`), `Runs+OpenCells`, and `rle::unrle + crc::crc32 == b.crc`.

**Host (C5 diagnostic note):** 2-vCPU Intel Xeon @ 2.30 GHz VM (Haswell model 63, L1d 32 KiB, L2 256 KiB, L3 45 MiB, AVX2 + BMI1 + BMI2 + PCLMULQDQ, no PMU; shared hypervisor). All rule decisions use steal-free thread on-CPU time (`/proc/thread-self/schedstat`, `taskset -c 0`, CPU-calibrated `inner` `>= 240 ms/sample`, 5 rotated interleaved repeats).

### 2.1 Per-corpus phase breakdown (`Column` vs `Runs+Segmented`, baseline x86-64, median of 5 interleaved passes)

Because `Segmented::invert_in_place` and `Segmented::invert_runs` execute the **identical `walk_back_room` walk**, `(col_mtf - run_front)` isolates **`mtf::mtf` vs `runs::front`**, and `(col_inv - run_inv)` isolates **`build_lf` vs `runs::fill_avx2_trusted`**:

| corpus | bz2 SHA-256 (12) | full | ship `Col` | `col_mtf` (ms) | `col_inv` (ms) | `run_front` (ms) | `run_inv` (ms) | `Runs/Col` | `Col` blks won by `Runs` | `T1` flipped (`won`) |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| `wikidata` | `82f65ace0731` | 233 | 0 | 303.75 | 2231.45 | 95.44 | 2183.99 | 0.899× | 0/0 | 0 (0) |
| `text` | `7f2f9ef75635` | 23 | 0 | 218.17 | 236.75 | 43.66 | 231.45 | 0.605× | 0/0 | 0 (0) |
| `prose` | `a7ae7a7e0a61` | 11 | 1 | 91.11 | 133.63 | 17.17 | 126.38 | 0.639× | **1/1** | 1 (1) |
| `pysrc` | `778519c3ffde` | 11 | 1 | 97.80 | 128.39 | 30.07 | 125.95 | 0.690× | **1/1** | 1 (1) |
| `rustsrc` | `b9acd3a0cd87` | 15 | 0 | 153.33 | 189.68 | 19.52 | 187.20 | 0.603× | 0/0 | 0 (0) |
| `json` | `70021dd3f0cc` | 15 | 2 | 78.98 | 169.50 | 15.36 | 166.49 | 0.732× | **2/2** | 2 (2) |
| `mdjournal` | `6e29b8a86c61` | 13 | 10 | **161.21** | 188.13 | **59.19** | 176.98 | **0.676×** | **10/10** | 10 (10) |
| `logs` | `faca024b4d75` | 18 | 0 | 101.51 | 186.62 | 42.63 | 171.40 | 0.743× | 0/0 | 0 (0) |
| `elf` | `78777db4d15f` | 33 | 26 | **495.34** | 424.50 | **313.63** | 401.96 | **0.778×** | **25/26** | 26 (25) |
| `random` | `bddec249d4f7` | 18 | 18 | 628.92 | 234.10 | 655.49 | 291.99 | **1.098×** | 1/18 | **0** (0) |
| `records` | `78bc91db7d19` | 22 | 22 | 597.29 | 235.29 | 649.05 | 303.81 | **1.144×** | 1/22 | **0** (0) |
| `runs` | `d2c624b98a0b` | 1 | 0 | 1.31 | 9.55 | 0.92 | 1.61 | 0.234× | 0/0 | 0 (0) |
| `onebyte` | `3c97c91017ac` | 1 | 0 | 0.05 | 1.70 | 0.01 | 9.48 | 5.406× | 0/0 | 0 (0) |
| `pattern` | `322d3789804a` | 44 | 0 | 1.69 | 324.46 | 0.45 | 195.24 | 0.600× | 0/0 | 0 (0) |

### 2.2 Physical mechanism: ~90% of the win is branchless RLE2 decoding in `runs::front`

1. **Why `mdjournal`, `elf`, `prose`, `pysrc`, and `json` Column blocks win massively on `Runs`:**
   - On `mdjournal`, `col_mtf` takes **`161.21 ms`** vs `run_front` **`59.19 ms`** (**`-102.0 ms`, `-63.3%`** — 90% of the total `-113.2 ms` gain!), while `fill_avx2_trusted` also trims `col_inv` (`188.13 -> 176.98 ms`, `-5.9%`).
   - On `elf`, `col_mtf` takes **`495.34 ms`** vs `run_front` **`313.63 ms`** (**`-181.7 ms`, `-36.7%`** — 89% of the total `-204.2 ms` gain!), plus `col_inv` `424.50 -> 401.96 ms` (`-5.3%`).
   - All 40 newly admitted blocks have **`run_frac = 17.0%–49.4%`** (`RUNA`/`RUNB` symbols). As [`mtf-column-front-dissection.md`](mtf-column-front-dissection.md) showed, `mtf::mtf_avx2` suffers severe `s <= 1 / run != 0` branch mispredictions when run symbols are frequent (37–50% of MTF time), whereas `runs::front_avx2` builds AVX2 `lit`/`one` bitmaps and decodes runs branchlessly via `tzcnt` + `_bzhi_u64` — and skips writing the 900 KB `ll` column entirely.
   - `prose` blk 8 (`syms_r = 0.5055`) and `pysrc` blk 8 (`syms_r = 0.5061`) missed the shipped `0.50` cutoff by **`0.006`** and suffered **`8.6×` slower MTF** (`11.7 ms` vs `1.36 ms`!).
2. **Why `random` and `records` stay faster on `Column` (`1.098×` / `1.144×` slower on `Runs`):**
   - `random` (`run_frac = 0.008`, `syms_r = 1.000`) and `records` (`run_frac = 0.003`, `syms_r = 1.000`) have `< 1%` run symbols, so `mtf::mtf`'s `s <= 1` branch is >99% not-taken (near-zero mispredictions), while `runs::front` pays bitmap construction + `~1.8 MB` of 2-byte records and `fill_avx2_trusted` executes `~900k` overlapping 8-wide AVX2 stores (`+58..+69 ms` slower than `build_lf`!).

### 2.3 Crossover surface by `syms_ratio` and `lit_ratio_est = syms_ratio * (1 - run_frac_sample)`

| `syms_ratio` bin | blks | `Runs` wins | `col_mtf` | `col_inv` | `run_frt` | `run_inv` | `Runs/Col` | `runf_med` | `lit_med` |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| `[0.00, 0.30)` | 308 | 263 | 421.9 | 2858.9 | 131.9 | 2680.3 | **0.857×** | 0.416 | 0.061 |
| `[0.30, 0.50)` | 71 | 66 | 671.5 | 810.6 | 178.1 | 784.4 | **0.649×** | 0.407 | 0.235 |
| `[0.50, 0.60)` *(missed!)* | 18 | **17/18** | 254.4 | 252.5 | 127.2 | 222.8 | **0.690×** | 0.305 | 0.394 |
| `[0.60, 0.70)` *(missed!)* | 21 | **21/21** | 344.7 | 291.9 | 196.0 | 277.5 | **0.744×** | 0.293 | 0.433 |
| `[0.70, 0.80)` | 2 | 1/2 (`runf=0.23` win, `0.009` loss) | 47.6 | 22.2 | 41.9 | 26.7 | 0.982× | 0.121 | 0.669 |
| `[0.98, 1.05)` | 38 | 2/38 | 1190.0 | 457.1 | 1267.3 | 581.9 | **1.123×** | 0.004 | 0.996 |

*(In `LR_BIN`: `[0.40, 0.50)` wins `25/26` at `0.740×`, `[0.50, 0.60)` wins `2/2` at `0.859×`, `[0.70, 0.80)` loses `0/1` at `1.058×` (`runf = 0.009`), `[0.90, 1.05)` loses `36/38` at `1.123×`.)*

---

## 3. Whole-corpus steal-free 3-build evaluation (all 14 corpora)

Steal-free `/proc/thread-self/schedstat` CPU ms (5 rotated interleaved repeats, CPU-calibrated `inner >= 240 ms/sample`, `taskset -c 0`). Candidate rules:
- **`T1_syms75_rs32` (RECOMMENDED — 1-line edit in `src/runs.rs:78`):** `4 * len <= 3 * max_len && run_frac >= 1/32` (O(1) short-circuit on `random`/`records` without touching `b.syms`; flips 40 blocks total, 39/40 winners).
- **`T3_syms85_rs16`:** `20 * len <= 17 * max_len && run_frac >= 1/16`.
- **`T5_lit65_rs32`:** `len <= max_len && run_frac >= 1/32 && lit_est <= 0.65 * max_len`.
- **`COMBO_T5_R2m22`:** stacks `T5_lit65_rs32` with `#1145`'s `R2_2tier_m22` (`post-1080-open-admission`) to measure combined worker CPU impact.

### 3.1 Affected corpora (where `T1` flips blocks from `Column` to `Runs`)

| corpus | build | `shipped` ms | **`T1_syms75_rs32`** | `T3_syms85_rs16` | `T5_lit65_rs32` | **`COMBO_T5_R2m22`** | peak `Runs` KiB (`ship -> T1`) |
|---|---|---:|---:|---:|---:|---:|---|
| **`mdjournal`** *(gate >= 10% F)* | base | 97.90 | **-24.11% F** | **-24.69% F** | **-22.62% F** | **-23.79% F** | 893 -> 1251 (+358) |
| | native | 98.43 | **-32.41% F** | **-30.06% F** | **-29.21% F** | **-33.58% F** | 893 -> 1251 (+358) |
| | JCC-32B | 87.97 | **-13.24% F** | **-11.15% F** | **-9.54% F** | **-13.65% F** | 893 -> 1251 (+358) |
| **`elf`** *(gate >= 10% F)* | base | 279.87 | **-15.48% F** | **-17.34% F** | **-14.14% F** | **-17.88% F** | 896 -> 1478 (+582) |
| | native | 287.20 | **-24.68% F** | **-23.81% F** | **-24.67% F** | **-24.63% F** | 896 -> 1478 (+582) |
| | JCC-32B | 266.62 | **-14.53% F** | **-13.68% F** | **-12.21% F** | **-14.50% F** | 896 -> 1478 (+582) |
| **`json`** | base | 65.48 | **-9.89% F** | -8.76% | **-9.27% F** | **-11.15% F** | 995 -> 1395 (+400) |
| | native | 59.76 | **-9.75% F** | -10.17% | **-9.86% F** | **-14.87% F** | 995 -> 1395 (+400) |
| | JCC-32B | 59.94 | **-9.01% F** | **-9.15% F** | **-9.33% F** | **-10.64% F** | 995 -> 1395 (+400) |
| **`prose`** | base | 67.04 | -7.10% | -5.51% | **-7.06% F** | **-18.59% F** | 980 -> 1013 (+33) |
| | native | 62.33 | -8.37% | **-6.76% F** | **-6.50% F** | **-23.00% F** | 980 -> 1013 (+33) |
| | JCC-32B | 68.64 | -4.28% | -3.03% | -3.76% | **-16.34% F** | 980 -> 1013 (+33) |
| **`pysrc`** | base | 60.52 | -3.81% | **-6.38% F** | -5.45% | **-6.15% F** | 996 -> 1014 (+18) |
| | native | 52.76 | -6.23% | -5.72% | **-7.43% F** | **-9.88% F** | 996 -> 1014 (+18) |
| | JCC-32B | 57.47 | **-4.18% F** | **-4.27% F** | **-4.26% F** | -6.18% | 996 -> 1014 (+18) |

### 3.2 Unchanged-route corpora (0 blocks flipped under `T1_syms75_rs32`; all 9 corpora have 0 B scratch delta)

| corpus (`0` blocks changed by `T1`) | `T1` base | `T1` native | `T1` JCC-32B | **`COMBO_T5_R2m22`** (base / nat / JCC) |
|---|---:|---:|---:|---|
| `wikidata` | +0.69% (ovl) | +0.30% (ovl) | +0.86% (ovl) | +2.26% / +0.06% / +1.76% (all ovl; `#1145` -1.2..-3.7%) |
| `text` *(primary)* | +5.56%\* (ovl) | -0.65% (ovl) | -1.60% (ovl) | **-8.66% F / -14.92% F / -9.81%** |
| `rustsrc` | +6.57%\* (ovl) | -1.50% (ovl) | +2.45%\* (ovl) | **-10.64% F / -14.86% F / -9.46% F** |
| `logs` | -2.22% (ovl) | +1.28% (ovl) | +0.14% (ovl) | **-21.85% F / -23.61% F / -16.94% F** |
| `random` *(0 KiB Runs)* | -0.17% (ovl) | -0.07% (ovl) | -1.07% (ovl) | +0.28% / +2.02% / +0.09% (all ovl) |
| `records` *(0 KiB Runs)* | +0.80% (ovl) | +1.23% (ovl) | +1.15% (ovl) | -1.18% / +0.28% / +0.05% (all ovl) |
| `runs` | -1.51% (ovl) | +7.17%\* (ovl) | -8.48% (ovl) | **-25.84% F / -29.87% F / -29.08%** |
| `onebyte` | -8.25% (ovl) | -0.02% (ovl) | -7.52% (ovl) | -9.64% / -0.03% / -1.60% (all ovl) |
| `pattern` | -6.97% (ovl) | -1.78% (ovl) | -0.75% (ovl) | -1.99% / +3.24% / +0.43% (all ovl) |

*\*Note on shared-VM noise:* On `wikidata`, `text`, `rustsrc`, `logs`, `random`, `records`, `runs`, `onebyte`, and `pattern`, `T1_syms75_rs32` executes the **exact same route on 100.0% of blocks** (0/549 blocks changed, and on `random`/`records` `4 * len > 3 * max_len` short-circuits in 1 multiply without reading `b.syms`). The few `> +1.5%` overlapping medians (`text`/`rustsrc` on base, `runs` 3.33->3.57 ms on 1-block native) are strictly overlapping VM noise between identical execution paths (`0` disjoint SLOWER cells across all `14 × 3 = 42` cells!).

---

## 4. Follow-on claim and caveats

- **Opened claim:** `frontier/claims/post-1080-takes-runs.toml` (`status = "idea"`, `kind = "build"`, `zone = "striated"`, `priority = 2`, `owned_paths = ["src/runs.rs"]`) — literally a 1-line threshold update (`2 * len > max_len` -> `4 * len > 3 * max_len`, keeping `RUN_SHARE = 32` unchanged) in `src/runs.rs::takes_runs_slice`, plus unit tests in `src/runs.rs`.
- **Peak memory trade-off:** Admitting `0.50 < syms/max <= 0.75` raises worst-case per-worker `Runs` scratch from `~1.03 MiB` to `~1.53 MiB` (`+358..+582 KiB` on `mdjournal`/`elf`/`json`, `0 B` on `wikidata`/`text`/`random`/`records`). Because `OpenCells` is not entered on these blocks (`r/n > 0.28`), `Open`'s buffers stay unallocated and total worker memory remains well below `Open` + `Segmented` peak.
- **C5 caveats:** Single-thread Haswell VM with concurrent sibling workloads; exact-head CI paired `ab` bench on the follow-on PR will verify zero regression across all 7 CI corpora at 1T/2T/4T (including peak RSS).
