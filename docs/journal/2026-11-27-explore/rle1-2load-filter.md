# Deep Dissection of `src/rle.rs`: 2-Load `m1` Filter, 128B Unrolling & Raw-AVX2 Out-of-Place RLE1 (`explore/rle1-2load-filter`)

*Smooth-zone explore probe (`Claim: explore`, 27 November 2026). Standalone harness in `examples/rle1_dissection.rs`; zero `src/` or `Cargo.toml` edits; frozen stop test recorded before any candidate timing under RULES C1a/C2.*

**Verdict: PASS.** The clean two-function drop-in combination **`P5_m1_128b_u2stage`** (`F3_m1_128b` replacing `find_runs_avx2` + `U2_m1_64b_raw` replacing `unrle_avx2` and its `shift > n` stage-swap path, keeping shipped `copy_within`/`fill` inside `unrle_in_place`) clears every branch of the frozen stop test across BOTH build profiles (**baseline x86-64** and **JCC-32B-aligned baseline** `-C llvm-args=-x86-branches-within-32B-boundaries`), with **`101,234 / 101,234` bit-, CRC-, and error-exact checks**, `0 B` extra memory (`<= 1.00x` footprint), and **0/64 regressions** across all 8 corpora (`wikidata_x4`, `text`, `random`, `onebyte`, `pattern`, `runs`, `records`, `zeros`) in both L3-warm and L3-cold regimes:

- **Why shipped `src/rle.rs` leaves major headroom on the table (`objdump -d`):**
  1. Shipped `find_runs_avx2` issues **4 overlapping unaligned 32-byte loads** (`i, i+1, i+2, i+3`), **3 `vpcmpeqb`**, **2 `vpand`**, and **1 `vpmovmskb`** per 32 bytes (plus `panic_bounds_check` on `rle[j]`), and restarts at `j + 5` on every hit instead of draining all runs in the vector — capping sparse scan throughput at `13.2–15.9 GB/s` and dense-run throughput (`runs`/`onebyte`/`zeros`) at `0.54 GB/s`.
  2. Shipped `unrle_avx2` calls `out.extend_from_slice` and `out.resize` inside an `#[target_feature(enable = "avx2,bmi1")]` function — `objdump -d` exposes **11 `vzeroupper` instructions and 10 calls** (including `call memcpy@GLIBC`, `call memset@GLIBC`, `slice_index_fail`, `panic_bounds_check`, and `RawVecInner::reserve`)! Every single run hits `vzeroupper` + `memcpy` + `memset`.
- **The winning drop-in (`F3_m1_128b` + `U2_m1_64b_raw` = `P5_m1_128b_u2stage`):**
  - **`F3_m1_128b` (`find_runs_avx2`):** 2-load, 1-`vpcmpeqb` `m1` filter (`v0 = load(p+i)`, `v1 = load(p+i+1)`, `m = movemask(v0 == v1)`), 128-byte unrolled into two `u64` masks with exact 2-bit carry `prev` (`tri = ((m << 2) | prev) & ((m << 1) | (prev >> 1)) & m`), **1 branch per 128 bytes** (`(tri_a | tri_b) == 0`), zero `vpand`, zero `panic_bounds_check`, and **multi-hit drain** inside each `u64` mask (`mask &= !((1u64 << (tz + 5)) - 1)`). Cuts isolated `find_runs` by **`-21.6%..-23.8%` warm / `-10.9%..-11.3%` cold** on `wikidata_x4` (both DISJOINT FASTER, `17.3–19.8 GB/s` sparse), **`-36.5%..-40.6%`** on `text`, and **`-60.8%..-62.2%`** on `runs`/`onebyte`/`zeros`!
  - **`U2_m1_64b_raw` (`unrle_avx2`):** 2-load, 1-`vpcmpeqb` `m1` filter at 62-byte stride (`tri = m & (m >> 1) & (m >> 2)`), direct `vmovdqu` stores, inline AVX2 `vpbroadcastb` + `vmovdqu` run fill, multi-hit drain, and `#[cold]` `grow_out` -> **0 `memcpy`, 0 `memset`, 0 hot-loop `vzeroupper`**. Cuts out-of-place `unrle` by **`-55.1%..-55.9%` warm / `-51.8%..-53.3%` cold** on `text` (**2.25× faster!**), **`-21.5%..-23.9%` warm / `-12.7%..-13.7%` cold** on `wikidata_x4`, **`-22.9%..-30.8%`** on `runs`, and **`-18.0%..-23.4%`** on `pattern`/`records`/`random`!
  - **Whole `unrle_in_place` (`A9`) & End-to-End 1W Decode Impact (`P5_m1_128b_u2stage`):**
    - `wikidata_x4` (`838.8 MB`): **`-10.96%` base / `-9.19%` JCC-32B L3-warm (both DISJOINT FASTER)** and **`-6.99%` base / `-5.76%` JCC-32B L3-cold** (`P2` identical-code-path `-5.72%` DISJOINT FASTER on JCC-32B). Projects **`~10–12 ms` (`~0.8–0.9%` 1W / `~0.65–0.8%` 8W worker CPU)** on physical i9-10900K (where `A9` = `104–115 ms` of `1,306 ms` 1W), stacking orthogonally with `crc32-avx2-ring-barrett-build` (`-7.5%` CRC half -> **~`-17%..-18%` combined `A9`**).
    - Off-Wikidata 1W decode wins (both profiles DISJOINT FASTER): **`text`** **`-15.01% / -16.14%` A9 (`-2.29% / -2.59%` of total 1W decode!)**, **`runs`** **`-14.72% / -16.61%` A9 (`-7.01% / -9.01%` 1W)**, **`zeros`** **`-10.52% / -11.20%` A9 (`-6.04% / -6.58%` 1W)**, **`onebyte`** **`-8.79% / -9.74%` A9 (`-4.79% / -5.68%` 1W)**, **`pattern`** **`-11.47% / -13.94%` A9 (`-0.51% / -0.83%` 1W)**.
- **Scoped negative findings recorded in `frontier/dead-ends.tsv`:**
  1. Un-unrolled 32B `m1` (`F1_m1_32b`) regresses sparse corpora **`+1.0%..+7.4%`** (carry/branch per 32B offsets saved loads); 64B/128B `u64` unrolling is required.
  2. True 1-load `vperm2i128 + vpalignr::<1>` (`F5_1load_alignr_64b`) loses **`-0.2%..+13.9%`** vs shipped on L3-warm sparse corpora (and trails `F3` by `20–30%`) due to 2 Port-5 uops/32B (`12.5–13.7 GB/s` ceiling vs `17.3–19.8 GB/s`).
  3. Inline variable-length small-copy (`<= 64 B`) and short-run fill (`<= 32 B`) inside `unrle_in_place` (`P3`/`P4`) regresses **`text` by `+6.1%..+14.1%` DISJOINT SLOWER** from unpredictable branch mispredictions on variable copy/run lengths.
- **Admitted follow-on striated build claim:** `frontier/claims/rle1-2load-filter-build.toml` (`status = "idea"`, `owned_paths = ["src/rle.rs"]`, `priority = 2`).

---

## 0. Frozen Pre-Data Stop Test (RULES C1a / C2)

Recorded strictly before compiling or timing any candidate implementation:

1. **Exactness & Memory (100% mandatory):**
   - Every candidate arm must match `mibz2::rle::unrle` + `mibz2::crc::crc32` (and `Err(ErrorKind::RunUnfinished)` identically) across `miri_edge_runs`, runs of lengths `4..=40` at every offset `0..=130` (32B/64B/128B seams + short tails `<32 B`), consecutive 2-run interference (`extra1 in {0, 1, c1, c2, 255}` at offsets `0..=68`), `64 KiB` tile boundaries `±2 B`, 400 random blocks, and every real extracted block of `wikidata.bz2` + all 7 guard corpora (`text`, `random`, `onebyte`, `pattern`, `runs`, `records`, `zeros`).
   - `0 B` extra allocation (`<= 1.00x` scratch/output capacity).
2. **Performance, End-to-End Share & Disassembly Gate (across BOTH `baseline x86-64` and `base+JCC-32B` `-C llvm-args=-x86-branches-within-32B-boundaries`, 7 interleaved CPU-0 repeats, `CLOCK_THREAD_CPUTIME_ID`):**
   - **PASS** iff at least one bit-exact candidate satisfies ALL of:
     - **(a) Speedup & share:** Cuts whole `unrle_in_place` (`A9`) on `wikidata_x4` by `>= 5.0%` across both warm and cold regimes and both build profiles, AND cuts `unrle_in_place` (or `unrle`) by `>= 10.0%` disjoint on at least two off-Wikidata guard corpora across both build profiles, with end-to-end 1W decode share `>= 0.5%` on `wikidata` (i9-projected) or `>= 2.0%` on off-Wikidata corpora;
     - **(b) Zero regressions:** `<= 1.01x` (no disjoint regression) on every tested corpus in both `unrle_in_place` and `unrle` across both build profiles;
     - **(c) Disassembly hygiene (`objdump -d`):** Strictly fewer loads / `vpcmpeqb` per 32B than shipped (4-load 3-`vpcmpeqb`), zero spurious `vzeroupper` / `call memcpy` / `call memset` inside hot AVX2 loops.
   - **Otherwise STOP:** Preserve the frozen verdict verbatim (RULES C2) and record a scoped row in `frontier/dead-ends.tsv`.

---

## 1. Environment, Corpus Census & `objdump -d` Audit

- **VM hardware:** 2-vCPU Intel Xeon (Haswell) @ 2.30 GHz (`avx2`, `bmi1`, `bmi2`, `pclmulqdq`), 32 KiB L1d, 256 KiB L2, 45 MiB L3, pinned to CPU 0 (`taskset -c 0`), `CLOCK_THREAD_CPUTIME_ID`, 1 warmup + **7 interleaved repeats** (alternating arm order each repeat). L3-cold scrubs 64 MiB before each sweep.
- **Exactness:** `cargo clippy --example rle1_dissection -- -D warnings` clean (0 warnings), **`101,234 / 101,234` bit-, CRC-, and error-exact checks** across all 14 arms on both build profiles.

### 1.1 Corpus RLE1 Census

| corpus | blocks | mult | pre-RLE1 (MB/pass) | post-RLE1 (MB/pass) | runs/pass | `shift > 0` blks | `shift > n` stage-swap | `A9` share of 1W (`base` / `JCC`) |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| `wikidata_x4` | 431 | 4 | 209.71 | 209.70 | 12,029 | 59 | 0 | 4.49% / 4.87% *(i9: ~8.0–8.8%)* |
| `text` | 14 | 4 | 11.79 | 14.16 | 349,936 | 14 | 0 | **15.24% / 16.04%** |
| `random` | 9 | 4 | 8.00 | 8.00 | 3 | 0 | 0 | 0.66% / 0.66% |
| `onebyte` | 1 | 4 | 0.39 | 20.00 | 78,432 | 1 | 1 | **54.45% / 58.30%** |
| `pattern` | 22 | 4 | 19.60 | 19.60 | 0 | 0 | 0 | 4.44% / 5.96% |
| `runs` | 1 | 8 | 0.25 | 10.00 | 50,771 | 1 | 1 | **47.66% / 54.21%** |
| `records` | 11 | 4 | 9.60 | 9.60 | 2 | 0 | 0 | 0.68% / 0.69% |
| `zeros` | 1 | 4 | 0.41 | 20.97 | 82,242 | 1 | 1 | **57.42% / 58.78%** |

### 1.2 `objdump -d` Audit (`baseline x86-64`, `rustc 1.99.1`)

| function | total ins | `vmovdqu` ld/st | `vpcmpeqb` | `vpmovmskb` | `vzeroupper` | `call` count | key calls |
|---|---:|---:|---:|---:|---:|---:|---|
| `mibz2::rle::unrle_avx2` *(shipped)* | 243 | 2 / 1 | 3 | 1 | **11** | **10** | `memcpy`, `memset`, `slice_index_fail`, `panic_bounds_check`, `reserve` |
| `rle1_probe_U1_m1_32b_raw` | 177 | 1 / 11 | 1 | 1 | 3 *(exit/cold)* | 2 | `reserve`, cold `grow_out` (**0 `memcpy`/`memset`**) |
| **`rle1_probe_U2_m1_64b_raw`** | 263 | 7 / 16 | 2 | 2 | 5 *(exit/cold)* | 4 | `reserve`, cold `grow_out` (**0 `memcpy`/`memset`**) |
| `rle1_probe_F0_shipped_4load` | 105 | 1 (+3 mem-op) / 0 | **3** | 1 | 4 | 3 | `find_runs_scalar`, `panic_bounds_check`, `grow_one` |
| `rle1_probe_F1_m1_32b` | 133 | 1 (+1 mem-op) / 0 | 1 | 1 | 3 | 2 | `find_runs_scalar`, `grow_one` (0 bounds panics) |
| `rle1_probe_F2_m1_64b` | 126 | 2 (+2 mem-op) / 0 | 2 | 2 | 3 | 2 | `find_runs_scalar`, `grow_one` (0 bounds panics) |
| **`rle1_probe_F3_m1_128b`** | 293 | 5 (+5 mem-op) / 0 | 4 (+1 tail) | 4 (+1 tail) | 5 | 4 | `find_runs_scalar`, `grow_one` (0 bounds panics; 1 branch / 128B) |
| `rle1_probe_F5_1load_alignr_64b` | 170 | 6 / 2 | 2 | 2 | 3 | 2 | `vperm2i128` + `vpalignr` = 2 Port-5 uops / 32B |

---

## 2. Benchmark Results (Both Profiles: `base` and `base+JCC-32B`, 7 Interleaved Repeats, CPU 0)

### 2.1 Family A — Isolated `find_runs` (`F0` vs `F1..F5`)

| corpus (regime) | `F0_shipped` `base` / `JCC` (ms) | `F1_m1_32b` `base` / `JCC` | `F2_m1_64b` `base` / `JCC` | **`F3_m1_128b`** `base` / `JCC` | `F5_1load_alignr` `base` / `JCC` |
|---|---:|---:|---:|---:|---:|
| **`wikidata_x4` warm** (838.8 MB) | 63.64 / 61.34 | −0.31% / +1.02% | −17.48% F / −17.52% | **−23.75% F / −21.62% F** | −1.12% / −0.23% |
| **`wikidata_x4` cold** (838.8 MB) | 114.59 / 117.76 | −0.95% / −1.69% | −7.82% F / −7.66% F | **−10.93% F / −11.34% F** | −1.87% / −2.66% |
| **`text` warm** (47.2 MB) | 22.49 / 23.06 | −4.73% / −15.64% F | −31.38% / −32.59% F | **−36.50% F / −40.61% F** | −27.13% / −27.35% F |
| **`text` cold** (47.2 MB) | 24.99 / 25.18 | −4.87% / −12.77% F | −29.29% / −29.29% F | **−34.48% F / −38.54% F** | −27.03% / −24.38% F |
| `pattern` warm (78.4 MB) | 5.01 / 4.95 | +7.18% / +5.5% | −22.69% / −14.5% | **−18.39% F / −16.30% F** | +10.05% / +3.0% |
| `records` warm (38.4 MB) | 2.57 / 2.42 | +4.37% / **+7.37% S** | −19.23% / −11.50% F | **−15.45% F / −16.92% F** | +9.25% S / +5.82% |
| `random` warm (32.0 MB) | 2.06 / 2.39 | +2.19% / +4.46% | −20.70% / −20.33% | **−21.44% F / −21.65%** | +13.91% S / −5.60% |
| `runs` warm (2.0 MB) | 3.79 / 3.82 | −56.1% F / −57.5% F | −61.5% F / −61.8% F | **−60.98% F / −60.75% F** | −58.7% F / −60.6% F |
| `onebyte` warm (1.6 MB) | 2.92 / 2.95 | −56.3% F / −56.8% F | −63.6% F / −62.5% F | **−61.33% F / −61.5% F** | −59.2% F / −59.7% F |
| `zeros` warm (1.6 MB) | 3.04 / 3.01 | −57.1% F / −56.9% F | −63.4% F / −62.5% F | **−62.24% F / −61.65% F** | −60.0% F / −59.7% F |

*(F = DISJOINT FASTER `[max_cand < min_base]`, S = DISJOINT SLOWER.)*

### 2.2 Family B — Whole `unrle_in_place` (`A9`) & End-to-End 1W Decode Share (`P0` vs `P3_inline` vs **`P5_m1_128b_u2stage`**)

| corpus (MB) | `P0_shipped` warm `base` / `JCC` | `P3_inline` warm `base` / `JCC` | **`P5_m1_128b_u2stage` warm** `base` / `JCC` | **`P5` L3-cold** `base` / `JCC` | **`P5` total 1W share** (`base` / `JCC`) |
|---|---:|---:|---:|---:|---:|
| **`wikidata_x4`** (838.8) | 159.32 / 151.39 ms | −7.58% / −5.16% | **−10.96% F / −9.19% F** | **−6.99% / −5.76%** *(P2: −5.72% F)* | **−0.49% / −0.45%** *(i9: ~−0.8–0.9%)* |
| **`text`** (56.6) | 61.73 / 58.97 ms | **+7.64% S / +7.92% S** | **−15.01% F / −16.14% F** | **−13.37% F / −15.88% F** | **−2.29% / −2.59%** |
| **`runs`** (80.0) | 26.07 / 26.24 ms | −15.5% F / −15.8% F | **−14.72% F / −16.61% F** | **−12.31% F / −15.23% F** | **−7.01% / −9.01%** |
| **`zeros`** (83.9) | 25.25 / 25.36 ms | −10.25% / −11.81% F | **−10.52% F / −11.20% F** | **−8.75% F / −8.18%** | **−6.04% / −6.58%** |
| **`onebyte`** (80.0) | 23.46 / 25.11 ms | −9.2% F / −10.9% F | **−8.79% F / −9.74% F** | **−7.97% F / −9.33% F** | **−4.79% / −5.68%** |
| **`pattern`** (78.4) | 11.80 / 13.15 ms | −7.0% / −10.04% F | **−11.47% F / −13.94% F** | **−5.35% F / −6.18%** | **−0.51% / −0.83%** |
| `records` (38.4) | 5.99 / 5.79 ms | −5.6% F / −6.96% | **−5.07% / −10.44% F** | −4.50% / −3.38% | −0.03% / −0.07% |
| `random` (32.0) | 4.88 / 5.10 ms | −7.2% F / −3.21% | **−8.88% F / −10.44%** | −4.70% / **−9.60% F** | −0.06% / −0.07% |

### 2.3 Family C — Out-of-Place `unrle` (`U0_shipped` vs **`U2_m1_64b_raw`**)

| corpus (MB) | `U0` warm `base` / `JCC` | **`U2_m1_64b_raw` warm** `base` / `JCC` | `U0` cold `base` / `JCC` | **`U2_m1_64b_raw` cold** `base` / `JCC` |
|---|---:|---:|---:|---:|
| **`wikidata_x4`** (838.8) | 95.58 / 100.38 ms | **−21.51% F / −23.92% F** | 140.30 / 138.36 ms | **−13.68% F / −12.67% F** |
| **`text`** (56.6) | 47.16 / 45.25 ms | **−55.12% F / −55.88% F** *(2.25×!)* | 49.96 / 47.56 ms | **−51.82% F / −53.29% F** |
| **`runs`** (80.0) | 10.39 / 7.60 ms | **−22.94% / −30.09% F** | 11.81 / 8.62 ms | **−30.76% F / −25.88% F** |
| **`pattern`** (78.4) | 8.64 / 8.83 ms | **−21.38% F / −21.38%** | 11.57 / 11.64 ms | **−8.41% F / −7.21%** |
| **`records`** (38.4) | 4.47 / 4.29 ms | **−23.41% F / −22.92% F** | — | — |
| **`random`** (32.0) | 3.59 / 3.65 ms | **−17.95% F / −21.81% F** | — | — |
| **`zeros`** (83.9) | 10.16 / 10.16 ms | **−14.82% F / −14.82% F** | 11.25 / 10.79 ms | **−7.97% / −5.29%** |
| **`onebyte`** (80.0) | 10.39 / 10.19 ms | **−10.54% F / −2.26%** | 10.56 / 10.60 ms | **−6.72% / −9.84%** |

---

## 3. Conclusion & Follow-On

1. **Verdict: PASS** (`P5_m1_128b_u2stage` + `U2_m1_64b_raw`). The changes are confined strictly to `src/rle.rs` (`find_runs_avx2` and `unrle_avx2`), require `0 B` extra memory, eliminate 11 `vzeroupper` + `memcpy`/`memset` calls from `unrle_avx2`, achieve `0/64` regressions across all 8 corpora × 2 cache regimes × 2 build profiles, deliver **−9.2%..−11.0% A9** on `wikidata` (orthogonally stacking with `crc32-avx2-ring-barrett-build`), and cut total 1W decode by **−2.3%..−2.6%** on `text`, **−4.8%..−5.7%** on `onebyte`, **−6.0%..−6.6%** on `zeros`, and **−7.0%..−9.0%** on `runs`.
2. **Admitted striated build claim:** `frontier/claims/rle1-2load-filter-build.toml` (`status = "idea"`, `owned_paths = ["src/rle.rs"]`, `priority = 2`).
