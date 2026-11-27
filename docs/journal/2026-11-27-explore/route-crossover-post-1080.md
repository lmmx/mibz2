# Post-#1080 `OpenCells` vs `Segmented` route-boundary and admission dissection across 9 corpora (PASS)

*Smooth-zone `explore` investigation, 27 November 2026 (zero `src/` edits; probe code in [`examples/route_crossover_probe.rs`](../../../examples/route_crossover_probe.rs), `#![forbid(unsafe_code)]`).*

---

## 1. Motivation and frozen decision rule

Two recent changes shifted both sides of the `OpenCells` vs `Segmented` inverse-BWT boundary (`src/open.rs::OpenCellsOf::cap`, `within_cap`, and `src/runs.rs::takes_runs`):

1. **PR #1080 (`0e51f0ca`, `segmented-fallback-build`)** landed on `master`, cutting `Segmented` inverse-BWT by `-15.4%..-18.5%` via sequential `LF` column build, invariant-checked `runs::fill_avx2`, and AVX2 reverse assembly.
2. **PR #1040 (`2915d7e7`, `dense-cap-precheck`)** replaced failed dense-band first passes with `within_cap(r.records(), p, 16, cap)` in `0.15 <= r/n < 0.28`, while **#1053 (`a6b-endpoint-redesign`)** and **#1110 (`forward-anchor-lanes`)** now live on `master` behind research features.

Prior route admission (`r/n < 0.15` outright `Cap::NONE`, `0.15 <= r/n < 0.28` `within_cap(m <= min(256_000, 0.30*n))`) was fitted before `#1080` on Wikidata (`docs/journal/2026-11-21-open-dense-admission.md`) and did not measure primary `text` (`ab gen`).

### Frozen discovery / stop test (frozen BEFORE reading any timing data)

- **PASS** (open a follow-on `idea` claim): a simple `O(1)` / zero-extra-memory pre-route rule (using only features available at `dense_precheck` / `runs::front` time: `n`, `records`, `m` / retained thin rows, `thick`, `r/n`, `m/n`, `p`, or `syms.len() / max_len`) recovers **`>= 5.0%` of total inverse-BWT + MTF worker CPU** on **`>= 2` off-Wikidata corpora** (or **`>= 3.0%` on primary `text`**), with **`0` Wikidata regression (`<= +0.2%`)** and **`0` regressions across all 9 corpora**, disjoint on both baseline x86-64 and JCC-32B-aligned builds.
- **STOP** (record a scoped dead-end in `frontier/dead-ends.tsv`): the per-block oracle-route ceiling itself is `< 3.0%` on primary `text` and `< 5.0%` across off-Wikidata corpora, or no simple `O(1)` / zero-extra-memory rule clears the PASS bar without regressions.

**Verdict: PASS (both primary `text` AND 3–4 off-Wikidata corpora clear the bar by 3–8× on both baseline and JCC-32B shipped builds, and remain strongly positive under `#1053` + `#1110`).**

---

## 2. Deterministic 9-corpus panel and route census

All 9 corpora verified **100% byte-exact** across `OpenCells::invert_runs`, `ForcedOpenCells::invert_runs`, `Segmented::invert_runs`, `mtf::mtf + Segmented::invert_in_place` (on `Column` blocks), and `rle::unrle + crc::crc32 == b.crc` on every single block (**571/571 blocks**, zero mismatches).

| corpus | source (`bzip2 -9` except dump) | raw bytes | bz2 bytes | ratio | bz2 SHA-256 (12) | blks | full (`n >= 100k`) | `Column` | `OpenLow` | `OpenDense` | `RefDense` | `RefHigh` |
|---|---|---:|---:|---:|---|---:|---:|---:|---:|---:|---:|---:|
| `wikidata` | dump (`origin/bench-data`) | 209,700,196 | 11,854,933 | 17.69× | `82f65ace0731` | 431 | 233 | 0 | 228 | 5 | 0 | 0 |
| `text` | `ab gen` scale=8 (primary) | 24,270,650 | 3,706,057 | 6.55× | `a6a9ad522a2a` | 23 | 23 | 0 | 2 | 0 | **18** | 3 |
| `prose` | `/usr/share/doc` + `dpkg/status` | 10,000,000 | 1,460,017 | 6.85× | `a7ae7a7e0a61` | 11 | 11 | 1 | 4 | 0 | **5** | 1 |
| `pysrc` | Python 3.12 stdlib `.py` | 10,668,220 | 1,924,526 | 5.54× | `778519c3ffde` | 11 | 11 | 1 | 1 | 0 | 2 | 7 |
| `rustsrc` | cargo registry + repo `.rs` | 15,000,000 | 2,238,713 | 6.70× | `73c9f990197c` | 16 | 15 | 0 | 1 | 1 | **12** | 1 |
| `json` | repo `.json`/`.jsonl` | 13,199,579 | 1,626,497 | 8.12× | `82614c34d74d` | 15 | 15 | 2 | 6 | 0 | **4** | 3 |
| `mdjournal` | repo `docs/**/*.md` | 11,493,986 | 2,955,340 | 3.89× | `18e3b195e38a` | 13 | 13 | 9 | 0 | 0 | 2 | 2 |
| `logs` | `/var/log` + `/var/lib/dpkg` | 16,000,000 | 2,226,094 | 7.19× | `faca024b4d75` | 18 | 18 | 0 | 3 | 3 | **10** | 2 |
| `elf` | `/usr/bin` ELF binaries | 30,000,000 | 10,144,975 | 2.96× | `78777db4d15f` | 33 | 33 | 26 | 0 | 0 | **5** | 2 |
| **Total** | | **340,332,631** | **38,137,152** | — | — | **571** | **372** | **39** | **245** | **9** | **58** | **21** |

*(Controls for the secondary `takes_runs` check: `ab gen` scale=8 `random` 16.0 MB / 18 full `Column` blocks and `records` 19.2 MB / 22 full `Column` blocks.)*

**Host (C5 diagnostic note):** 2-vCPU Intel Xeon @ 2.30 GHz VM (Haswell model 63, L1d 32 KiB, L2 256 KiB, L3 45 MiB, AVX2 + BMI1 + BMI2 + PCLMULQDQ, no PMU; shared hypervisor with concurrent background load during wall-clock intervals). All rule decisions use **steal-free on-CPU thread time** (`/proc/thread-self/schedstat` field 0, `taskset -c 0`, CPU-calibrated `inner` passes giving `>= 240 ms` CPU per sample to keep tick quantization `<= 1.5%`, 5 rotated interleaved repeats), with wall clock reported alongside.

---

## 3. Where the post-#1080 `OpenCells` vs `Segmented` crossover actually sits

Across all **333 full run-route blocks** (`n >= 100,000`) across all 9 corpora, binning `ForcedOpenCells` vs post-`#1080` `Segmented` by **retained-row fraction `m/n`** (the actual size of `Open`'s return-map permutation) and by **record fraction `r/n`** across all 4 builds (`base_shipped`, `jcc_shipped`, `base_future`, `jcc_future`, where `future` = `--features "a6b-endpoint-redesign forward-anchor-lanes"` and `jcc` = `-C llvm-args=-x86-branches-within-32B-boundaries`):

### 3.1 Crossover by `m/n` (retained rows / block length)

| `m/n` bin | blks | `r/n` med | `base_shipped` `Open/Seg` (wins) | `jcc_shipped` `Open/Seg` (wins) | `base_future` `Open/Seg` (wins) | `jcc_future` `Open/Seg` (wins) |
|---|---:|---:|---:|---:|---:|---:|
| `[0.00, 0.10)` | 36 | 0.063 | **0.643×** (36/36) | **0.532×** (36/36) | **0.546×** (35/36) | **0.438×** (36/36) |
| `[0.10, 0.15)` | 158 | 0.066 | **0.745×** (158/158) | **0.681×** (158/158) | **0.665×** (147/158) | **0.538×** (158/158) |
| `[0.15, 0.20)` | 28 | 0.101 | **0.965×** (17/28) | **0.892×** (19/28) | **0.952×** (22/28) | **0.757×** (26/28) |
| `[0.20, 0.25)` | 23 | 0.131 | **1.283×** (1/23) | **1.089×** (2/23) | **1.311×** (4/23) | **1.030×** (10/23) |
| `[0.25, 0.30)` | 6 | 0.161 | **1.427×** (0/6) | **1.223×** (1/6) | **1.125×** (0/6) | **1.080×** (0/6) |
| `[0.30, 0.35)` | 9 | 0.213 | **1.477×** (0/9) | **1.487×** (0/9) | **1.674×** (1/9) | **1.037×** (4/9) |
| `[0.35, 0.40)` | 10 | 0.217 | **1.758×** (0/10) | **1.834×** (0/10) | **1.763×** (0/10) | **1.458×** (0/10) |
| `[0.40, 0.50)` | 27 | 0.210 | **2.459×** (0/27) | **2.356×** (0/27) | **2.158×** (0/27) | **1.909×** (0/27) |
| `[0.50, 0.65)` | 35 | 0.274 | **2.756×** (0/35) | **2.804×** (0/35) | **2.712×** (0/35) | **2.308×** (0/35) |

### 3.2 Crossover by `r/n` (run records / block length)

| `r/n` bin | blks | `m/n` range | `base_shipped` `Open/Seg` (wins) | `jcc_shipped` `Open/Seg` (wins) | `jcc_future` `Open/Seg` (wins) |
|---|---:|---|---:|---:|---:|
| `[0.050, 0.100)` | 207 | 0.072–0.210 | **0.741×** (203/207) | **0.674×** (203/207) | **0.532×** (204/207) |
| `[0.100, 0.150)` | 38 | 0.135–0.423 | **1.234×** (9/38) | **1.076×** (11/38) | **0.981×** (25/38) |
| `[0.150, 0.180)` *(DENSE)* | 16 | 0.217–0.460 | **1.663×** (0/16) | **1.663×** (1/16) | **1.532×** (2/16) |
| `[0.180, 0.210)` *(DENSE)* | 12 | 0.221–0.492 | **2.111×** (0/12) | **1.664×** (1/12) | **1.583×** (1/12) |
| `[0.210, 0.240)` *(DENSE)* | 21 | 0.309–0.533 | **2.325×** (0/21) | **2.230×** (0/21) | **1.668×** (2/21) |
| `[0.240, 0.280)` *(DENSE)* | 18 | 0.305–0.567 | **2.278×** (0/18) | **2.272×** (0/18) | **2.015×** (0/18) |
| `[0.280, 0.350)` | 19 | 0.451–0.603 | **2.862×** (0/19) | **2.768×** (0/19) | **2.430×** (0/19) |

### 3.3 Why shipped routing loses 10–25% across `text`, `prose`, `rustsrc`, and `logs`

Three separate misrouting mechanisms compound on shipped `master`:

1. **`OpenLow` (`r/n < 0.15`) has NO `m/n` check — and moderate thin runs (`width ∈ [3, 16]`) decouple `r/n` from `m/n`:**
   - Primary **`text` blks 2 & 19**: `r/n = 0.1421` and `0.1458` (`< 0.15`, admitted outright to `OpenLow`!), yet **`m/n = 0.4186` and `0.4226`** (`~380k` retained rows!). Shipped `Open` runs **`2.06–2.14×` slower** than post-`#1080` `Segmented` (`8.34 vs 4.04 ms` and `8.15 vs 3.96 ms` baseline; `7.51 vs 3.56 ms` and `7.79 vs 3.64 ms` JCC-32B), wasting **`~8.1–8.5 ms`** on a 23-block file!
   - **`prose` blks 0, 2, 3, 4**: all 4 `OpenLow` blocks have `r/n = 0.056..0.145` yet **`m/n = 0.204..0.345`**, losing **`1.33–1.96×`** baseline (`1.06–1.95×` JCC-32B).
   - **`logs` blks 2, 13, 17**: all 3 `OpenLow` blocks have `r/n = 0.124..0.150` yet **`m/n = 0.196..0.262`**, losing **`1.17–2.74×`**.
2. **The entire `DENSE` band (`0.15 <= r/n < 0.28`) is now net-negative post-`#1080`:**
   - Across all **67 dense-band blocks** in all 9 corpora, post-`#1080` `Segmented` beats shipped `Open` on **67/67 (100% baseline)** and **65/67 (97% JCC-32B)** — including **all 5 of Wikidata's dense-band blocks** (`blks 41, 56, 165, 270, 278`, `m/n = 0.221–0.257`, `Open/Seg = 1.15–1.41×` baseline, `1.17–2.06×` JCC-32B) and all 4 off-Wikidata `OpenDense` blocks (`rustsrc` blk 13 `1.30×`, `logs` blks 3/7/8 `1.12–1.86×`).
   - Even with `#1053` + `#1110` + JCC-32B enabled (`jcc_future`), `[0.20, 0.25)` is `1.030×` (tie) and `[0.25, 0.30)` is `1.080×` (0/6 Open wins).
3. **`within_cap` (`src/open.rs:1016`) pays a ~0.4–0.5 ms/block scalar scan on 58 refused dense-band blocks:**
   - Every `RefDense` block (`text` 18, `rustsrc` 12, `logs` 10, `prose` 5, `elf` 5, `json` 4, `pysrc` 2, `mdjournal` 2) runs `within_cap`'s scalar loop over ~130k–160k records before hitting `m > 256_000` — wasting `~8–9 ms` on `text` alone and `~5–6 ms` on `rustsrc` ONLY to refuse 100% of them!
4. **Zero missed `Open` opportunities in refused blocks:** across all 79 refused run-route blocks (`58 RefDense + 21 RefHigh`), `Open` beat `Segmented` on **0 blocks** on `base_shipped` (1 small noise hit on `logs`), **0 blocks** on `jcc_shipped` (`elf` 1 noise hit), and even on `jcc_future`, `m/n >= 0.35` won **0/73 blocks**.

---

## 4. Whole-corpus evaluation of simple O(1) / zero-extra-memory pre-route rules

Three simple, zero-extra-memory rules were benchmarked head-to-head against `shipped`, `oracle_runs`, and `always_seg` on **MTF/front + InverseBwt** (steal-free `/proc/thread-self/schedstat` CPU, 5 rotated interleaved repeats, CPU-calibrated `inner = 1..5` so each sample `>= 240 ms`):

- **`R1_rn015_nodense`:** delete `DENSE` (`0.15..0.28`) and `within_cap`; Open iff `1/64 <= r/n < 0.15` (0 ns overhead).
- **`R2_2tier_m22`** *(zero `src/runs.rs` edits — ~10 LOC in `src/open.rs::cap`)*:
  - `1/64 <= r/n < 0.10`: outright `Cap::NONE` (0 ns overhead on 194/233 Wikidata blocks and all fast `json` blocks);
  - `0.10 <= r/n < 0.15`: live `within_cap(m <= min(256_000, 0.22*n), strips <= 16_384)`;
  - `r/n >= 0.15`: straight to `Segmented` (deletes the dense band and all 58 wasted `within_cap` scans).
  *(Also tested at `m <= 0.20*n` and `m <= 0.25*n`.)*
- **`R3_o1_m20` / `R3_o1_m22`** *(models tracking `thin_rows` branchlessly in `src/runs.rs::front_avx2::emit!` for O(1) admission with zero `within_cap` pass)*:
  - Open iff `1/64 <= r/n < 0.15 && m <= T*n && thick <= 16_384` (also catches low-`r/n`, high-`m/n` blocks like `prose` blks 3 & 4 where `r/n = 0.056, m/n = 0.204–0.210`).

### 4.1 Baseline x86-64 (shipped `Open` + post-`#1080` `Segmented`, steal-free CPU ms)

| corpus | `shipped` | `oracle_runs` | `R1_rn015` | `R2_2tier_m20` | **`R2_2tier_m22`** | **`R3_o1_m20`** | `R3_o1_m22` |
|---|---:|---:|---:|---:|---:|---:|---:|
| **`wikidata`** | 747.75 | -0.93% | -2.87% | -4.41% | **-3.70%** | **-3.02%** | -2.00% |
| **`text`** *(primary)* | 133.99 | **-19.71% F** | **-12.89% F** | **-20.39% F** | **-20.24% F** | **-24.41% F** | **-18.41% F** |
| **`prose`** | 68.06 | **-22.89% F** | -6.02% | -15.77% | **-16.78%** | **-27.11% F** | **-21.11% F** |
| **`pysrc`** | 54.93 | -6.35% | -0.85% | -3.67% | **-3.79%** | -3.21% | +1.27% |
| **`rustsrc`** | 76.49 | **-16.37% F** | **-14.17% F** | **-12.83% F** | **-13.96% F** | -10.85% | **-16.04% F** |
| **`json`** | 67.27 | -3.21% | -12.22% | -10.36% | **-9.13%** | -5.18% | -4.45% |
| **`mdjournal`** | 104.18 | -0.51% | -0.94% | +0.14% | **-2.29%** | +0.79% | -3.82% |
| **`logs`** | 98.75 | **-21.89% F** | **-21.23% F** | **-17.34% F** | **-16.73% F** | **-18.42% F** | **-16.90% F** |
| **`elf`** | 302.82 | -2.56% | -1.75% | -3.54% | **-2.46%** | -3.23% | +0.77% |

*(Suffix **F** = disjointly **FASTER** (`max_cand < min_shipped` across all 5 repeats). `always_seg` regresses `wikidata` `+15.50% SLOWER` baseline and `+26.40% SLOWER` JCC-32B.)*

### 4.2 JCC-32B-aligned x86-64 (shipped `Open` + post-`#1080` `Segmented`, steal-free CPU ms)

| corpus | `shipped` | `oracle_runs` | `R1_rn015` | `R2_2tier_m20` | **`R2_2tier_m22`** | **`R3_o1_m20`** | `R3_o1_m22` |
|---|---:|---:|---:|---:|---:|---:|---:|
| **`wikidata`** | 651.92 | -2.49% | -4.36% | -3.51% | **-1.21%** | **-2.92%** | -2.74% |
| **`text`** *(primary)* | 123.47 | **-13.00% F** | -3.14% | **-11.61% F** | **-12.88% F** | **-11.72% F** | **-13.02% F** |
| **`prose`** | 64.28 | **-21.17% F** | -0.99% | **-13.83% F** | **-12.52% F** | **-21.27% F** | **-16.18% F** |
| **`pysrc`** | 55.81 | -2.84% | **-6.55% F** | -4.43% | **-5.55% F** | **-5.94% F** | **-8.36% F** |
| **`rustsrc`** | 73.19 | **-12.59% F** | **-12.59% F** | -11.35% | **-12.64% F** | **-12.77% F** | **-13.73% F** |
| **`json`** | 59.22 | -5.65% | **-5.97% F** | **-6.23% F** | **-6.09%** | -5.80% | -5.99% |
| **`mdjournal`** | 87.79 | -3.32% | -3.76% | -3.11% | **-2.76%** | -2.34% | -2.92% |
| **`logs`** | 90.39 | **-17.10% F** | **-14.18% F** | **-14.01% F** | **-15.77% F** | **-15.73% F** | **-16.22% F** |
| **`elf`** | 280.14 | -1.57% | +1.81% | +1.42% | **-1.09%** | +1.42% | +0.29% |

*(`always_seg` regresses `json` `+5.43%` and `wikidata` `+26.40% SLOWER`, whereas `R2_2tier_m22` is negative/faster on **18/18 cells** across baseline and JCC-32B with zero regressions!)*

### 4.3 Future-proofing check: JCC-32B + `#1053` (`a6b-endpoint-redesign`) + `#1110` (`forward-anchor-lanes`)

Even when `Open` becomes ~12–25% faster under both upcoming research features (`wikidata` `651.9 -> 573.3 ms`, `always_seg` `+42.68% SLOWER`), `R2_2tier_m22`, `R2_2tier_m25`, and `R3_o1_m22` **still win by double digits** with zero Wikidata regression:

| corpus (`jcc_future`) | `shipped` | `oracle_runs` | `R2_2tier_m22` | `R2_2tier_m25` | `R3_o1_m22` | `R3_o1_m25` |
|---|---:|---:|---:|---:|---:|---:|
| **`wikidata`** | 573.26 | +2.07% | **-1.49%** | **-0.45%** | **-3.11%** | **-4.45%** |
| **`text`** *(primary)* | 114.16 | **-7.93% F** | **-10.62% F** | **-12.69% F** | **-12.99% F** | **-10.83% F** |
| **`prose`** | 62.11 | **-18.16% F** | **-10.26% F** | **-13.28% F** | **-9.91% F** | **-10.70% F** |
| **`rustsrc`** | 69.83 | **-6.97% F** | **-8.00%** | -2.06% | **-8.16%** | **-8.04% F** |
| **`json`** | 58.65 | -4.91% | **-6.12%** | **-6.06%** | **-6.65%** | **-6.44%** |
| **`logs`** | 82.90 | **-16.13% F** | **-16.14% F** | **-15.48% F** | **-17.38% F** | **-13.84%** |

---

## 5. Bonus secondary finding: `runs::takes_runs` conservatism on `mdjournal` and `elf` post-#1080

Forcing `Column`-route blocks (`!runs::takes_runs(b)`) through `runs::front + Segmented::invert_runs` (`diag_col_via_runs`) revealed a second, completely orthogonal post-`#1080` boundary shift:

| corpus | `Column` / full | `base_shipped` CPU Δ% | `jcc_shipped` CPU Δ% |
|---|---:|---:|---:|
| **`mdjournal`** (`syms/max ≈ 0.55`) | 9 / 13 | **-26.24% FASTER** (`104.18 -> 76.84 ms`) | **-18.81% FASTER** (`87.79 -> 71.27 ms`) |
| **`elf`** (`syms/max ≈ 0.50–0.65`) | 26 / 33 | **-18.51% FASTER** (`302.82 -> 246.75 ms`) | **-13.87% FASTER** (`280.14 -> 241.29 ms`) |
| **`random`** (control, `RUN_SHARE < 1/32`) | 18 / 18 | +1.72% (overlap) | — |
| **`records`** (control, `RUN_SHARE < 1/32`) | 22 / 22 | **+8.09% SLOWER** (`329.23 -> 355.87 ms`) | — |

Because `src/runs.rs:78` hard-cuts `takes_runs` at `2 * syms.len() <= max_len` (`syms / max_len <= 0.50`, fitted in `2026-11-10` before `#1080`'s `trusted fill_avx2`), compressible markdown (`0.55`) and ELF binaries (`0.50–0.65`) with substantial zero/byte runs fall back to `mtf::mtf + build_lf`, whereas `random` and `records` are already cleanly protected by `RUN_SHARE` (`>= 1/32` run symbols). This is documented separately so `post-1080-open-admission` stays minimal and strictly scoped to `src/open.rs` (or `src/open.rs` + `src/runs.rs` thin-row tracking).

---

## 6. Follow-on claim and caveats

- **Opened claim:** `frontier/claims/post-1080-open-admission.toml` (`status = "idea"`, `kind = "build"`, `zone = "striated"`, `priority = 2`, `owned_paths = ["src/open.rs"]`), starting with the 10-line, zero-extra-memory, 100% safe `R2_2tier` rule in `src/open.rs::OpenCellsOf::cap` (and optionally comparing `R3_o1` `thin_rows` tracking if `src/runs.rs` is unheld).
- **C5 caveats:** Single-thread Haswell Xeon VM with 45 MiB L3 and no PMU. On the 10-core i9-10900K (20 MiB shared L3 at 8W), `Segmented`'s 3.6 MiB `tt` table contends more for L3 than `Open`'s smaller working set (`~1.0–1.5 MiB` at `m/n <= 0.25`); therefore the follow-on claim's frozen stop test explicitly gates Wikidata 1T/4T (CI) AND guards Wikidata's 5 dense-band blocks (and recommends `m <= 0.22*n..0.25*n` or an i9 check before removing the 5 Wikidata dense blocks if multi-thread L3 contention tilts those 5 blocks back toward `Open`). Note that on `text`, `prose`, `rustsrc`, and `logs`, the 58 refused `RefDense` blocks and `m/n = 0.32–0.42` `OpenLow` blocks already use `> 2.0–3.2 MiB` in `Open` and lose by `1.7–2.8×` on both L3-warm and L3-cold hardware.
