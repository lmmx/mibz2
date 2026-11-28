# Dissecting Residual `A5+A6a` (`runs/fold.rs` + `FoldSink`/`OutQ16`) with `mtf-fold + F8` and `u8 SymBuf`

**Date:** 2026-11-28  
**Scope:** Local VM exploration (zero `src/` changes shipped; all prototypes reverted prior to this PR) on `origin/master` (`3c7212bb` / rebased `f0a340b0`), composing `wt-mtf-fold-full` (`16f37aa2`, `#1373` `M1 + F8`) and `wt-u8-syms-build` (`ffac2b97`, `#1339` `u8 SymBuf`), followed by micro-stage dissection and targeted ablations of the remaining `A5+A6a` budget.  
**Testbed:** KVM guest on Intel Xeon Haswell-EP `@ 2.30 GHz` (Family 6 Model 63, `2` vCPU, `32 KiB L1d`, `256 KiB L2`, `45 MiB L3`), compiled with production `[profile.release]` (`opt-level = 3`, `lto = "fat"`, `codegen-units = 1`) under both **`base`** (default `RUSTFLAGS`) and **`JCC-32B`** (`RUSTFLAGS="-C llvm-args=-x86-align-branch-boundary=32 -C llvm-args=-x86-align-branch=fused+jcc+jmp"`).  
**Evidence status:** VM exploration (not physical i9 evidence). Physical i9 projections below are explicitly marked **DERIVED** using `#1368`'s calibrated transfer priors (`0.45×` worker compute, `1.05×` base / `1.01×` JCC scanner).

---

## Executive Verdict

1. **Where the remaining `A5+A6a` time actually lives (`A0 = 16f37aa2` `M1 + F8`, `u16` syms):**
   - Across all `233` folded Open blocks of `wikidata.bz2` (`15,141,602` runs, `29,696` batches), **`87–89%` of residual `A5+A6a` sits in just two loops**:
     - `P2_mtf` (`fill_batch_avx2`): `62.0–62.6 ms` JCC / `79.2 ms` base (**`40.0–42.5%`**)
     - `P2_sink` (`FoldSink::batch` = `flush_batch_fast` + `OutQ16` + `push_strip` + `push_slow`): `69.6–73.0 ms` JCC / `86.0 ms` base (**`46.0–46.9%`**)
   - All four remaining sub-stages (`P1_bm` `9.4–9.8 ms`, `P1b_prescan` `6.7–6.8 ms`, `P1c_make_sink` `2.3–3.3 ms`, `P3_fin` `0.65–0.70 ms`) sum to only **`19–20 ms` (`~13%`)**.
   - `batches_thin_only` is only `15.4%` (`4,573 / 29,696`), and `resize_zero_bytes` is only `1.81 MB` across all `233` blocks (pooled `free_out` preserves buffer `len`), ruling out both thin-only batch specialization and zero-fill avoidance as high-leverage targets.
2. **Composing `u8 SymBuf` (`ffac2b97`) into `mtf-fold + F8` (`A1`) is a clean, zero-downside win:**
   - `P1_bm` (`build_bitmaps_avx2_u8`, `0 vpermq / 0 vpack / 0 vptest`) drops **`-39.4%`** (`9.39 → 5.68 ms` JCC, `-3.71 ms`).
   - `P2_mtf` stays flat (`62.55 → 61.41 ms`, Port-5 / branch-bound rather than symbol load-bandwidth-bound).
   - Peak `VmHWM` RSS drops **`-1.7%` to `-2.0%`** at `1W lazy` (`34.4 → 33.8 MiB`) and **`-5.5%` to `-6.0%`** at `8W` (`60.6 → 57.0 MiB` uncached; `73.5 → 69.4 MiB` warmed).
   - Heavy non-Open guard corpora (`text`, `records`, `random`) also improve **`-1.5%` to `-5.2%`** CPU and **`-5.5%` to `-7.0%`** RSS under `JCC-32B`.
   - **Recommendation:** Compose `u8 SymBuf` (`#1339`) directly into `runs/fold.rs` (`#1373`) when integrating `M5` / `v0.5.19+` (`-3.7 ms` `P1_bm` + `#1339`'s `-10%` scanner `S` win + `-5.5%` `8W` RSS).
3. **Every branching micro-optimization inside `P2_mtf` and `P2_sink` hits a branch-mispredict wall (STOP / dead ends):**
   - **`A3` (two-tier `s < 17` 1-`vpshufb` `LUT16` in `fill_batch_avx2`):** Despite textbook 1-instruction `vpshufb (%rax,%rdx,1),%xmm2,%xmm2` (`0 vzeroupper`, `0 ymm`), `P2_mtf` regresses **`+10.62 ms` (`+17.3%` slower)** under `JCC-32B` (`61.41 → 72.03 ms`). Exact Wikidata histograms show `13.65%` (`2.07M`) of literals fall in `17..=32` mixed unpredictably with `84.83%` `< 17`, causing ~`2M` extra mispredicts that swamp the Port-5 `vpshufb` reduction. Branchless `s < 33` (`98.48%` taken) `STEPS` is already optimal.
   - **`A2` (`width <= 4` 8-byte `u64` SWAR) & `A2b` (`width <= 8` 16-byte SSE2) in `FoldSink::flush_batch_fast`:** Clean monotonic branch-mispredict cliff: `w <= 16` (`4.99%` thick fail) → `w <= 8` (`8.49%` fail, `+3.50 pp`) is flat (`-0.31 ms`) → `w <= 4` SWAR (`11.98%` fail, `+6.99 pp`) regresses **`+6.48 ms` (`+7.8%` slower)** under `JCC-32B`.
   - **Shrinking `OutQ16` fold chunks below `1008/1024` (`496/512` and `240/256`):** `1008/1024` (`40,597` `push_slow`) strictly beats both `496/512` (`67,001` `push_slow`, `+1.47..+1.71 ms` `P2_sink`, `+4.5 ms` `A6bc`) and `240/256` (`123,408` `push_slow`, `+8.47 ms` `P2_sink`, `+12.8 ms` `A6bc`). `F8`'s `1008/1024` chunk stride is already optimal.
4. **Portable runtime-dispatched AVX2 `flush_batch_fast_avx2` (`A4`) is a real but sub-threshold compute micro-win (`~0.5%` i9 worker DERIVED):**
   - Wrapping `flush_batch_inner`'s unchanged safe body in `#[target_feature(enable = "avx2")]` auto-vectorizes the 8-instruction legacy SSE2 iota + 2× 16 B stores into 4 AVX2 instructions (`vmovd` + `vpbroadcastw` + `vpaddw` with hoisted `ymm1` + single 32 B `vmovdqu %ymm0`), cutting `P2_sink` by **`-4.75 ms` (`-6.8%`)** at `1008/1024` (`69.64 → 64.89 ms`) and **`-4.99 ms` (`-7.0%`)** at `496/512` (`71.35 → 66.36 ms`), bringing total `A5+A6a` to **`143.17 ms`** (`-8.17 ms` / `-5.4%` vs `F8`).
   - However, `-4.8 ms` is only **`~0.85%` of total worker `W` (`~550 ms`)**, projecting (**DERIVED**, `#1368` `0.45×` compute prior) to **`~2.1 ms` (`~0.5%` of `W`) on physical i9-10900K**, well below the `>= 3%` standalone-claim bar, and would require introducing `unsafe` `#[target_feature]` dispatch into `src/open.rs` (currently `0` `unsafe`; `-C target-cpu=native` already emits this automatically).
5. **Bottom line:** Beyond composing `u8 SymBuf` into `runs/fold.rs`, residual `A5+A6a` (`~143–151 ms` JCC) sits near the AVX2 hardware floor (`4`-`vpshufb` Port-5 floor on `P2_mtf` + 256-stream scatter floor on `P2_sink`; direct MTF+OutQ fusion already hit `#1010`'s register-spill wall). Do **not** file a standalone `A5+A6a` micro-claim; focus worker effort on `A7` (`Walk::run<4>` / forward-lanes) and `A6b` (`endpoint.rs`).

---

## 1. Methodology & Zero-Syscall Micro-Breakdown of `A0` (`16f37aa2` `M1 + F8`)

### Measurement Pitfall Avoided
On this KVM guest, `clock_gettime(CLOCK_THREAD_CPUTIME_ID)` is a full ~`3 µs` syscall. Calling it per batch (`29,696` batches × 2 timestamps ≈ `60k` calls) inflated profiled `A5+A6a` by ~`180–237 ms`. Switching to **block-level timing** (`233` blocks = `< 1 ms` overhead) plus an **isolated untimed-boundary `fill_batch_avx2` replay** cleanly separates `P2_mtf` and `P2_sink = P2_tot - P2_mtf` while keeping `true_cpu` within `1–2%` of uninstrumented binaries.

### Exact Sub-Stage Breakdown of `A0` (`16f37aa2` `M1 + F8`, `wikidata.bz2`, `lazy 1W`, `n = 5` medians)

| Sub-stage | Mechanism | `base` (ms) | `% A5+A6a` | `JCC-32B` (ms) | `% A5+A6a` | Per-item (JCC) |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| `P1_bm` | `build_bitmaps_avx2` (`u16`) | `9.78` | `5.2%` | `9.75` | `6.3%` | `41.8 µs/blk` |
| `P1b_prescan` | `bitmap_prescan_bmi2` + `s2u_injective` | `7.54` | `4.0%` | `6.78` | `4.4%` | `29.1 µs/blk` |
| `P1c_make_sink` | pool `acquire` + `resize` + `Slot8` init | `3.32` | `1.8%` | `3.27` | `2.1%` | `14.0 µs/blk` |
| **`P2_mtf`** | **`fill_batch_avx2` (4× `vpshufb`)** | **`79.19`** | **`42.5%`** | **`62.27`** | **`40.0%`** | **`4.11 ns/lit`** |
| **`P2_sink`** | **`FoldSink::batch` (`OutQ16` + `push_strip`)** | **`85.99`** | **`46.1%`** | **`73.02`** | **`46.9%`** | **`4.82 ns/run`** |
| `P3_fin` | `flush` + `cap` + `write_thin` + `admitted` | `0.71` | `0.4%` | `0.65` | `0.4%` | `2.8 µs/blk` |
| **Total `A5+A6a`** | `P1..P3` sum | **`186.54`** | **`100.0%`** | **`155.74`** | **`100.0%`** | **`10.29 ns/run`** |
| `A6bc` | endpoint + dict + thin readback | `168.23` | — | `153.45` | — | — |
| `A7` | forward 4-lane emission walk | `259.87` | — | `264.99` | — | — |

*(Note: Under calmer host load in Section 3's 5-arm interleaved run, `A0` `JCC-32B` read `P1_bm 9.39`, `P2_mtf 62.55`, `P2_sink 69.64`, `A5+A6a 151.34 ms`, `A6bc 151.24 ms`, `A7 234.23 ms` — identical relative split.)*

### Exact Deterministic Wikidata Counters (`233` Folded Open Blocks, `15,141,602` Runs)

| Counter | Exact count | Share |
| --- | ---: | ---: |
| `blocks_folded` | `233` | `100%` of Open blocks (`0` folded on all 6 guard corpora) |
| `batches` (`BATCH = 512`) | `29,696` | — |
| `batches_thin_only` (zero thick runs) | `4,573` | **`15.40%`** of batches |
| `w == 1` runs | `10,851,740` | **`71.67%`** of all runs (`75.43%` of thin) |
| `w <= 4` runs | `13,327,312` | **`88.02%`** of all runs (`92.64%` of thin) |
| `w <= 8` runs | `13,855,671` | **`91.51%`** of all runs (`96.31%` of thin) |
| `thin_runs` (`w <= 16`) | `14,386,587` | **`95.01%`** of all runs |
| `thick_runs` (`w > 16`) | `755,015` | **`4.99%`** of all runs |
| MTF `s < 17` (`nn ∈ 1..=15`) | `12,844,873` | **`84.83%`** of literals |
| MTF `17 <= s < 33` (`nn ∈ 16..=31`) | `2,066,928` | **`13.65%`** of literals |
| MTF `s >= 33` (`second_cold`) | `229,801` | **`1.52%`** of literals |
| `push_slow_calls` (`1008/1024`) | `40,597` | `0.28%` of thin runs (`0` overflow chunks) |
| `resize_zero_bytes` | `1,812,357 B` | `7.8 KiB/blk` avg (pooled `free_out` preserves `len`) |

Guard corpus fold behavior: `0` folded blocks across all 6 guards (`text` bails in `P1_bm 1.72 ms + P1b_prescan 2.84 ms`; `runs` bails in `0.21 ms`; `onebyte`, `pattern`, `records`, `random` `0.00 ms`).

---

## 2. Isolated Micro-Stage Ablations Under `JCC-32B`

All micro-stage arms were compared interleaved (`n = 5` alternating forward/reverse order, `lazy 1W`, `wikidata.bz2`, `--warmup`) using the identical zero-syscall block-level profiler.

### 2.1 Run-Width Store Narrowing (`A2`, `A2b`) and Two-Tier MTF (`A3`)

| Arm | Description | `P1_bm` | `P1b` | `P1c` | `P2_mtf` | `P2_sink` | `P3` | **`A5+A6a`** | `A6bc` | `A7` |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `A0_f8` | `u16`, 32 B SSE2 (`w<=16`, `4.99%` fail), 4-`vpshufb` | `9.44` | `6.75` | `2.28` | `61.82` | `83.49` | `0.70` | `164.48` | `157.75` | `282.03` |
| `A1_u8`* | `u8 SymBuf`, 32 B SSE2, 4-`vpshufb` | **`5.94`** | `6.78` | `2.74` | `61.41` | `88.23`* | `2.97`* | `168.08`* | `161.49` | `269.99` |
| `A2b_w8` | `A1` + 16 B SSE2 (`w<=8`, `8.49%` fail) | **`5.72`** | `6.56` | `3.28` | `61.17` | `83.18` | `0.66` | `160.57` | `156.66` | `292.88` |
| `A2w4+A3` | `A1` + 8 B SWAR (`w<=4`, `11.98%` fail) + `LUT16` (`s<17`) | **`5.72`** | `6.66` | `2.59` | **`72.03`** | **`89.97`** | `0.75` | **`177.71`** | `157.06` | `290.59` |

*\*`A1_u8` in this run used the temporary histogram-counting binary (one extra untimed replay pass immediately before `P3`, which accounts for `P3 = 2.97 ms` and cache eviction); all other rows used the clean profiler (`P3 = 0.66–0.75 ms`).*

**Microarchitectural mechanism proven:**
- **`A3` (`LUT16` `s < 17`):** `P2_mtf` regressed **`+10.62 ms` (`+17.3%`)** (`61.41 → 72.03 ms`) despite ideal codegen (`movzbl`, `cmp $0x11`, `shl $4`, `lea LUT16(%rip)`, `vpshufb (%rax,%rdx,1),%xmm2,%xmm2`, `0 vzeroupper / 0 ymm`). With `13.65%` (`2.07M`) literals in `17..=32`, branch mispredicts completely swamp the Port-5 `vpshufb` savings.
- **`A2` (`SWAR u64` `w <= 4`) & `A2b` (`SSE2` `w <= 8`):** Every `+3.5 pp` of extra unpredictable run-width branch failures (`~530k` runs) adds ~`+3–4 ms` in mispredicts:
  - `w <= 8` (`+3.50 pp` fail) saves ~`3–4 ms` on store traffic, netting **`-0.31 ms`** (flat).
  - `w <= 4` SWAR (`+6.99 pp` fail, plus two 10-byte `movabs` immediates and 3-cycle `imul` inside the loop) nets **`+6.48 ms` (`+7.8%` slower)**.

### 2.2 Portable AVX2 `flush_batch_fast_avx2` (`A4`) and `OutQ16` Chunk-Stride Sweep (`1008` vs `496` vs `240`)

| Arm | `OutQ16` chunk (`bytes`) | `flush` ISA | `P1_bm` | `P2_mtf` | **`P2_sink`** | **`A5+A6a`** | `A6bc` | `A7` | `push_slow` | HWM (MiB) |
| --- | --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `A0_f8_1008` | `1008/1024` (`2048 B`) | SSE2 (safe) | `9.39` | `62.55` | `69.64` | `151.34` | **`151.24`** | `234.23` | `40,597` | `39.4` |
| **`C1008_avx2` (`A4`)** | **`1008/1024` (`2048 B`)** | **AVX2 (32 B)** | **`5.68`** | `62.89` | **`64.89`** | **`143.17`** | `154.99` | `261.05` | `40,597` | **`37.9`** |
| `C496_safe` | `496/512` (`1024 B`) | SSE2 (safe) | `5.68` | `61.22` | `71.35` | `148.26` | `155.75` | `246.79` | `67,001` | `38.8` |
| `C496_avx2` | `496/512` (`1024 B`) | AVX2 (32 B) | `5.67` | `62.40` | `66.36` | `144.11` | `155.29` | `258.73` | `67,001` | `38.3` |
| `C240_safe` | `240/256` (`512 B`) | SSE2 (safe) | `5.82` | `63.00` | `78.11` | `156.77` | `164.05` | `272.51` | `123,408` | `38.6` |

**Two crisp conclusions from the 5-arm chunk/SIMD sweep:**
1. **`1008/1024` is strictly optimal over smaller chunks:** Shrinking to `496/512` (`1.65×` `push_slow`) adds `+1.47..+1.71 ms` to `P2_sink` and `+4.5 ms` to `A6bc`; shrinking to `240/256` (`3.04×` `push_slow`) adds `+8.47 ms` to `P2_sink` and `+12.8 ms` to `A6bc` (more chunk-link hops during `A6c` readback).
2. **`A4` (`flush_batch_fast_avx2`) cuts `P2_sink` cleanly at both chunk sizes (`-4.75 ms` / `-6.8%` at `1008/1024`, `-4.99 ms` / `-7.0%` at `496/512`):** Replacing 8 legacy SSE2 instructions (`movd` + `pshuflw` + `pshufd` + `movdqa` + `2× paddw` + `2× movdqu`) with 4 AVX2 instructions (`vmovd` + `vpbroadcastw` + `vpaddw %ymm1` + single 32 B `vmovdqu %ymm0`) cuts `A5+A6a` to **`143.17 ms`** (`-8.17 ms` / `-5.4%` vs `F8`). Note that on this Haswell-EP VM, `A7` showed ~`+25 ms` variance in the profiled AVX2 arms (consistent with Haswell-EP's 256-bit AVX2 turbo-license downclocking when YMM executes `127×/block` instead of once in `P1_bm`; uninstrumented multi-threaded `A4` runs in Section 3 below did not suffer that regression).

---

## 3. Uninstrumented Whole-Decoder Matrix (`base` & `JCC-32B`), Parity, Guards, and `objdump` Hygiene

### 3.1 70/70 Exact Parity vs C `bzip2`
Every arm (`A0_f8`, `A1_u8`, `A2_swar`, `A3_lut16`, `A4_avx2`) × both builds (`base` at `1W lazy`, `JCC-32B` at `8W mibz2`) × all 7 corpora (`wikidata`, `text`, `runs`, `onebyte`, `pattern`, `records`, `random`) matched `bzip2 -dc` byte length and SHA-256 bit-for-bit (**70/70 exact**; plus `7/7` exact on `A2b_w8`, `C496_safe`, `C496_avx2`, and `C240_safe`).

### 3.2 `objdump -dC` Hygiene Audit (`JCC-32B` Release Binaries)

| Binary / Function | Insns | `vzeroupper` | `%ymm` | Key hot-loop audit |
| --- | ---: | ---: | ---: | --- |
| `a1_jcc` `build_bitmaps_avx2_u8` | `267` | `2` | `39` | **`0 vpermq`, `0 vpack`, `0 vptest`**, `0 movzwl` (`u8` native 32 B AVX2) |
| `a1_jcc` `fill_batch_avx2::<u8>` | `313` | **`0`** | **`0`** | `4 vpshufb`, `movzbl` (`0 movzwl`), pure 128-bit XMM |
| `a1_jcc` `FoldSink::flush_batch_fast` | `199` | **`0`** | **`0`** | Legacy SSE2 (`movd+pshuflw+pshufd+movdqa+2×paddw+2×movdqu`) |
| `a2_jcc` `FoldSink::flush_batch_fast` | `251` | **`0`** | **`0`** | `cmp $5`, `movzwl`, `2× movabs`, `imul`, `add`, 8 B `mov` |
| `a3_jcc` `fill_batch_avx2::<u8>` | `325` | **`0`** | **`0`** | `cmp $0x11`, `shl $4`, `lea LUT16`, single `vpshufb (%rax,%rdx,1)` |
| `a4_jcc` `FoldSink::flush_batch_fast_avx2` | `189` | `5`* | `6` | `vmovd + vpbroadcastw + vpaddw %ymm1 + 1× 32 B vmovdqu %ymm0` |

*\*In `a4_jcc`, all 5 `vzeroupper` instructions are exclusively at function exit / cold `push_slow` / panic ABI boundaries (`0` inside the hot loop).*

### 3.3 Symmetric Interleaved `wikidata.bz2` Matrix (`cpu_ms` Medians, Uninstrumented, `n = 7` `1W lazy`, `n = 5` `8W mibz2`)

*(Measured under active sibling KVM load; process `cpu_ms` medians are shown alongside paired mean ± 95% Student-t CI vs `A0_f8` and peak `VmHWM` MiB. Because KVM steal spikes widen whole-file CIs to `±29–79 ms`, isolated zero-syscall stage profiling in Section 2 is the ground-truth arbiter for sub-15 ms stage deltas.)*

| Build & Mode | Arm | Median `cpu_ms` | Median Δ vs `A0` | Paired Mean ± 95% CI vs `A0` | Peak `VmHWM` (MiB) |
| --- | --- | ---: | ---: | ---: | ---: |
| `base` `1W lazy` (`n=7`) | `A0_f8` | `846.16` | `+0.00%` | `+0.00 ± 0.00 ms` | `34.4` |
| | `A1_u8` | **`829.14`** | **`-2.01%`** | `-19.41 ± 30.48 ms` | **`33.8`** (`-1.7%`) |
| | `A2_swar` | `847.12` | `+0.11%` | `+5.13 ± 35.73 ms` | `33.8` |
| | `A3_lut16` | `837.14` | `-1.07%` | `-7.39 ± 28.91 ms` | `33.8` |
| | `A4_avx2` | `833.87` | `-1.45%` | `-20.67 ± 28.57 ms` | `33.8` |
| `base` `8W mibz2` (`n=5`) | `A0_f8` | `874.06` | `+0.00%` | `+0.00 ± 0.00 ms` | `60.6` |
| | `A1_u8` | `888.73` | `+1.68%` | `-6.46 ± 42.24 ms` | **`57.0`** (`-5.9%`) |
| | `A2_swar` | `895.62` | `+2.47%` | `+8.50 ± 72.64 ms` | `57.1` |
| | `A3_lut16` | `904.97` | `+3.54%` | `+20.54 ± 63.89 ms` | `57.6` |
| | `A4_avx2` | **`834.87`** | **`-4.48%`** | `-41.85 ± 44.44 ms` | `57.9` (`-4.5%`) |
| `JCC-32B` `1W lazy` (`n=7`) | `A0_f8` | `804.07` | `+0.00%` | `+0.00 ± 0.00 ms` | `34.6` |
| | `A1_u8` | **`790.40`** | **`-1.70%`** | `-52.42 ± 78.80 ms` | **`33.8`** (`-2.3%`) |
| | `A2_swar` | `798.03` | `-0.75%` | `-104.27 ± 210.38 ms` | `33.8` |
| | `A3_lut16` | `784.32` | `-2.46%` | `-142.23 ± 235.40 ms` | `33.8` |
| | `A4_avx2` | **`760.22`** | **`-5.45%`** | `-152.14 ± 245.59 ms` | `33.9` (`-2.0%`) |
| `JCC-32B` `8W mibz2` (`n=5`) | `A0_f8` | `814.89` | `+0.00%` | `+0.00 ± 0.00 ms` | `60.7` |
| | `A1_u8` | `827.62` | `+1.56%` | `+4.16 ± 74.32 ms` | `57.4` (`-5.4%`) |
| | `A2_swar` | `790.18` | `-3.03%` | `-12.86 ± 32.47 ms` | `57.4` |
| | `A3_lut16` | `781.83` | `-4.06%` | `-26.90 ± 43.79 ms` | `57.9` |
| | `A4_avx2` | **`789.86`** | **`-3.07%`** | `-26.08 ± 54.21 ms` | **`56.7`** (`-6.6%`) |

*(An earlier `--warmup` paired `A1 vs A0` run at `4W` and `8W` showed the exact same RSS reduction: `4W` `124.9 → 119.4 MiB` `-4.4%`, `8W` `73.5 → 69.4 MiB` `-5.5%`.)*

### 3.4 Six Guard Corpora (`lazy 1W`, `n = 5` Interleaved Medians, `cpu_ms` & Peak `VmHWM` MiB)

| Corpus | `base` `A0` | `base` `A1` (Δ) | `base` `A4` (Δ) | `JCC` `A0` | `JCC` `A1` (Δ) | `JCC` `A4` (Δ) | `JCC` HWM `A0 → A1` |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `text` | `300.88` | `296.93` (`-1.31%`) | `294.56` (`-2.10%`) | `298.44` | **`293.91`** (`-1.52%`) | `313.42` (`+5.02%`) | `29.3 → 27.7` (`-5.5%`) |
| `runs` | `48.12` | `45.60` (`-5.24%`) | `47.99` (`-0.27%`) | `42.36` | `43.11` (`+1.76%`) | `43.88` (`+3.59%`) | `30.8 → 30.6` (`-0.6%`) |
| `onebyte` | `70.76` | `74.26` (`+4.96%`) | `75.51` (`+6.71%`) | `75.37` | `72.81` (`-3.40%`) | `70.92` (`-5.91%`) | `50.8 → 50.1` (`-1.4%`) |
| `pattern` | `224.46` | `220.31` (`-1.85%`) | `210.68` (`-6.14%`) | `222.10` | `227.77` (`+2.55%`) | `223.60` (`+0.67%`) | `11.2 → 11.5` (`+2.7%`) |
| `records` | `535.83` | `526.31` (`-1.78%`) | `516.07` (`-3.69%`) | `533.12` | **`508.38`** (`-4.64%`) | `505.95` (`-5.09%`) | `58.3 → 55.0` (`-5.7%`) |
| `random` | `470.95` | `454.54` (`-3.48%`) | `445.29` (`-5.45%`) | `453.49` | **`430.07`** (`-5.16%`) | `426.85` (`-5.87%`) | `48.9 → 45.5` (`-7.0%`) |

*(All 6 guard corpora take `0` folded blocks; `A1`'s consistent `-1.5%` to `-5.2%` JCC speedups and `-5.5%` to `-7.0%` RSS drops on `text`, `records`, and `random` come purely from `u8 SymBuf`.)*

---

## 4. Derived Physical i9-10900K Transfer Projection & Strategic Guidance

Applying `#1368`'s calibrated VM→i9 transfer priors (`0.45×` worker compute levers, `1.01×` JCC / `1.05×` base scanner levers):

| Lever | Isolated VM JCC-32B delta | **DERIVED** i9-10900K projection | Decision |
| --- | --- | --- | --- |
| **`A1`: `u8 SymBuf` in `runs/fold.rs`** | `P1_bm -3.71 ms` (`-39.5%`), `8W` RSS `-5.5%`, plus `#1339` `S ~ -10%` | **DERIVED:** `~-1.7 ms` worker `W` + `~-10%` scanner `S` (`1.01×` transfer) + `-5.5%` `8W` RSS | **KEEP / COMPOSE** when merging `#1339` + `#1373` into `M5` |
| `A2` (`w <= 4` SWAR) & `A2b` (`w <= 8` SSE2) | `P2_sink +6.48 ms` (`A2`), `-0.31 ms` (`A2b`) | **DERIVED:** negative / flat | **STOP** (branch-mispredict wall) |
| `A3` (`s < 17` 1-`vpshufb` `LUT16`) | `P2_mtf +10.62 ms` (`+17.3%`) | **DERIVED:** negative | **STOP** (`13.65%` `17..32` mispredict cliff) |
| `OutQ16` chunk shrink (`496/512`, `240/256`) | `P2_sink +1.5..+8.5 ms`, `A6bc +4.5..+12.8 ms` | **DERIVED:** negative | **STOP** (`1008/1024` already optimal) |
| `A4` (`#[target_feature(enable="avx2")]` `flush_batch_fast_avx2`) | `P2_sink -4.75..-4.99 ms` (`~0.85%` of `W`) | **DERIVED:** `~-2.1 ms` (`~0.5%` of `W`; `-C target-cpu=native` already emits it automatically) | **DO NOT FILE STANDALONE CLAIM** (`< 3%` bar; adds `unsafe` dispatch to `src/open.rs`) |

**Strategic conclusion:** With `mtf-fold + F8` (`16f37aa2`) + `u8 SymBuf` (`ffac2b97`), `A5+A6a` is genuinely near its x86-64 AVX2 hardware floor. No standalone `A5+A6a` micro-claim is warranted; compose `u8 SymBuf` into `runs/fold.rs` during `M5` integration and direct worker optimization effort toward `A7` (`Walk::run<4>` / forward-lanes) and `A6b` (`endpoint.rs`).
