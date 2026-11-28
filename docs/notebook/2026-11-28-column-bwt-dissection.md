# Dissecting `src/bwt.rs` `Column` (`random` / `records` Fallback Kernel): MLP, `build_lf`, and Fused `MTF -> Relative-LF`

*28 November 2026. Smooth-zone exploration off `origin/master` (`5c577161`, verified against `701366e1`). Local probe harness committed strictly locally on unpushed branch `explore/column-dissection` (`79e5a443`, `examples/column_dissection.rs` & `examples/column_round2.rs`); zero `src/` edits pushed while PR `#1311` holds `src/bwt.rs`.*

**Scope & honesty notice:** All new timings were measured on a shared 2-vCPU Linux KVM VM (`rustc 1.99.1`, `opt-level = 3`, `lto = "fat"`, `codegen-units = 1`) using steal-free `CLOCK_THREAD_CPUTIME_ID` (`n = 11` interleaved alternating sweeps, pinned CPU 0, warm and 64 MiB L3-cold scrub) under both default x86-64 (`base`) and `JCC-32B` (`-C llvm-args=-x86-align-branch-boundary=32 -C llvm-args=-x86-align-branch=fused+jcc+jmp`). **These are local VM dissection measurements, NOT physical i9-10900K adoption evidence.**

**Verdict: STOP (honest scoped dead-end recorded in `frontier/dead-ends.tsv`; no P2 `idea` claim filed).**
1. **`Column` is NOT a 1-miss serial chase:** Since `#1080`, `Column::decode` (`src/bwt.rs`) already dispatches to `Segmented::invert_in_place`, which runs sequential `build_lf` (`7.7–8.0%` of worker time) followed by 32-lane MLP `room_segments::<32, PF_L1, true>` (`22–26%`). The true bottleneck is serial **`mtf::mtf` (`62.1–66.7%` of `Column` worker time)**, not walk MLP or `build_lf`.
2. **Zero primary-corpus reach:** Under `origin/master`'s widened `takes_runs` gate (`WIDENED_RUN_SHARE`, `0.50..0.75`), **`0` full native-`Column` blocks** exist on `wikidata`, `text`, `prose` (`9 MiB` of `docs/*.md`), `elf` (`9 MiB` of `/usr/bin/*`), `runs`, `pattern`, or `onebyte`. Only high-entropy secondary guards `random` and `records` take `Column`, and on physical i9-10900K (`#1119`, `origin/evidence` `77236472`) both are **scanner-bound** from `4W` upward (`k* = 3.90` on `random`, `3.68` on `records`).
3. **Best novel zero-`ll` stack misses the `>= 5%` `JCC-32B` bar:** Eliminating the `0.9 MiB` `ll` intermediate via fused `Q_signed_flat7` `MTF -> Relative-LF` + pure-leaf `rsp = 0` `walk_chunk_leaf` + direct reverse-to-`out` RLE1/CRC (`R2_outl_rel_signed7`) cuts worker scratch by **`-0.86 MiB/w` (`0.908–0.911x`, `-6.9 MiB` at 8W)** and eliminates relative-walk overhead on `random`, yet caps at **`-4.21%` warm / `-3.25%` cold on `random` (`JCC-32B`)** and **`-0.66%` warm / `-0.95%` cold on `records` (`JCC-32B`)**. Under `#1368`'s calibrated transfer ledger (`0.45x` compute prior), that prices to `~-1.5%` on i9 `random/1W`, `~-0.4%` on `records/1W`, and `~0%` at `>= 4W`.

---

## 1. Route Census, PR `#1311` Context, and Whole-Decoder Baseline

### 1.1 Native `Column` Route Census (`origin/master`)

In `src/bwt.rs`, `Column::decode` only executes when a block fails `runs::takes_runs` (used by `OpenCells` and `RunsFallback` / `Segmented::invert_runs`). Auditing all production and real-world test corpora at `origin/master` (`5c577161`):

| Corpus | Source / Size | Total Blocks | Native-`Column` Full Blocks | Actual Production Route |
|---|---|---:|---:|---|
| `wikidata` | 209.7 MB (`431` blocks) | 431 | **0** *(198 tiny runts only)* | 233 `OpenCells` (`r/n < 0.21`), 198 tiny runts (`avg 45 B`) |
| `text` | 28.7 MB (`/tmp/l6a_corpora/text.bz2`) | 27 | **0** | `RunsFallback` (`Segmented::invert_runs`) / `OpenCells` |
| `prose` | 9.0 MB (`docs/**/*.md`, `bz2 -9`) | 10 | **0** | `RunsFallback` / `OpenCells` |
| `elf` | 9.0 MB (`/usr/bin/*`, `bz2 -9`) | 10 | **0** | `RunsFallback` / `OpenCells` |
| `runs`, `pattern`, `onebyte` | standard CI guards | all | **0** | `RunsFallback` / `OpenCells` |
| **`random`** | 16.0 MB (`/tmp/l6a_corpora/random.bz2`) | **18** | **18 (100%)** | **`Column::decode` -> `Segmented::invert_in_place`** |
| **`records`** | 19.2 MB (`/tmp/l6a_corpora/records.bz2`) | **22** | **22 (100%)** | **`Column::decode` -> `Segmented::invert_in_place`** |

PR `#1311` (`build/segmented-fused-postwalk-rescue`) restricts `FusedPostwalk` exclusively to `Segmented::invert_runs` (`RunsFallback`: `text`, `runs`, `onebyte`, `pattern`) and restores `Column`, `Segmented::invert_in_place`, `build_lf`, and `room_segments` to exact `origin/master` (dropping `#1183`'s `u64`-store `build_lf`, which had contributed to a `+1.5..+2.2%` `random/1T` regression on CI). Prior `frontier/dead-ends.tsv` entries (Rows 28, 30, 43, 45) already closed `room_segments` micro-tuning (`SEG = 256`, `vpgatherdd`, `cmov`, transposed `tile`), KKP `LF^2` (`+105..+121%`), and standalone `mtf_avx2` inlining.

### 1.2 Whole-Decoder `1W` / `8W` Baseline, `phases`, and `12/12` SHA-256 Parity

Untimed `AB_SHA256=1` `ab child` verification against C `libbzip2` (`bz2.open`) passed **12/12** across `{base, JCC-32B} x {random, records, text} x {1W mibz2-workers-lazy, 8W mibz2}` (`random`: `16,000,000 B`; `records`: `19,200,000 B`; `text`: `28,672,000 B`, 100% SHA-256 exact).

| Profile | Corpus | `1W` (`mibz2-workers-lazy`, `n=5`) Wall / CPU / RSS | `8W` (`mibz2`, `n=5`) Wall / CPU / RSS | `examples/phases` Scanner `S` Share | Physical i9 `k*` (`#1119`, `77236472`) |
|---|---|---:|---:|---:|---:|
| `base` | `random` | `2054.7 ms` / `452.6 ms` / `36.43 MiB` | `890.8 ms` / `517.2 ms` / `83.23 MiB` | `15.7%` (`117.3 / 746.0 c/B`) | **`3.90`** (scanner-bound `>= 4W`) |
| `base` | `records` | `2358.2 ms` / `525.1 ms` / `41.12 MiB` | `1069.1 ms` / `596.2 ms` / `85.26 MiB` | `18.4%` (`51.1 / 277.5 c/B`) | **`3.68`** (scanner-bound `>= 4W`) |
| `base` | `text` | `1457.3 ms` / `304.0 ms` / `23.90 MiB` | `610.1 ms` / `404.3 ms` / `100.92 MiB` | `9.9%` (`14.3 / 143.8 c/B`) | `5.09` (`0` `Column` blocks) |
| `JCC-32B` | `random` | `1615.0 ms` / `448.7 ms` / `36.36 MiB` | `811.7 ms` / `499.5 ms` / `82.55 MiB` | `15.3%` (`62.4 / 407.0 c/B`) | **`3.90`** |
| `JCC-32B` | `records` | `1904.3 ms` / `517.1 ms` / `40.99 MiB` | `991.3 ms` / `586.4 ms` / `84.02 MiB` | `15.1%` (`60.3 / 399.3 c/B`) | **`3.68`** |
| `JCC-32B` | `text` | `1024.5 ms` / `303.4 ms` / `26.37 MiB` | `543.8 ms` / `392.1 ms` / `99.67 MiB` | `9.3%` (`11.0 / 118.1 c/B`) | `5.09` (`0` `Column` blocks) |

*(Note: `examples/phases` times its legacy standalone `T-vector build` harness rather than `Segmented::build_lf` and uses wall clock under shared VM load; the exact `CLOCK_THREAD_CPUTIME_ID` breakdown of shipped `Column::decode` is measured directly in §2.)*

---

## 2. Stage 0: Exact 6-Phase Breakdown of Shipped `Column::decode`

Measuring every stage of shipped `Column::decode` (`mtf::mtf` -> `build_lf` -> `room_segments::<32, PF_L1, true>` -> `assemble_reversed` -> `rle::unrle_in_place` -> `crc::bzip2_crc`) on pinned CPU 0 with steal-free `CLOCK_THREAD_CPUTIME_ID`:

| Build | Corpus (Blocks / Rows) | `mtf::mtf` | `build_lf` | `room_segments` Walk (`L=32`) | `assemble_rev` | `unrle` | `crc32` | Total `Column` |
|---|---|---:|---:|---:|---:|---:|---:|---:|
| `base` | `random` (`18` / `16.00M`) | **`227.44 ms` (`64.0%`)** | `28.08 ms` (`7.9%`) | `85.96 ms` (`24.2%`) | `8.62 ms` (`2.4%`) | `3.74 ms` (`1.1%`) | `1.62 ms` (`0.5%`) | `355.46 ms` |
| `base` | `records` (`22` / `19.20M`) | **`231.85 ms` (`64.8%`)** | `27.91 ms` (`7.8%`) | `83.97 ms` (`23.5%`) | `8.72 ms` (`2.4%`) | `3.65 ms` (`1.0%`) | `1.70 ms` (`0.5%`) | `357.79 ms` |
| `JCC-32B` | `random` (`18` / `16.00M`) | **`200.20 ms` (`64.4%`)** | `24.48 ms` (`7.9%`) | `74.44 ms` (`23.9%`) | `7.14 ms` (`2.3%`) | `3.17 ms` (`1.0%`) | `1.61 ms` (`0.5%`) | `311.04 ms` |
| `JCC-32B` | `records` (`22` / `19.20M`) | **`227.28 ms` (`62.1%`)** | `29.29 ms` (`8.0%`) | `93.41 ms` (`25.5%`) | `9.19 ms` (`2.5%`) | `4.71 ms` (`1.3%`) | `1.94 ms` (`0.5%`) | `365.81 ms` |

**Architectural ceiling:**
- `build_lf` is only **`7.8–8.0%`** (`24.5–29.3 ms`). Even completely deleting `build_lf` with zero walk penalty caps at `-8.0%`.
- Post-walk (`assemble_rev` + `unrle` + `crc32`) is only **`3.8–4.3%`** (`11.9–15.8 ms`).
- `room_segments` already runs **`L = 32` concurrent backward lanes** (`SEG = 512`, `PF_L1`) — there is no 1-miss pointer-chase bottleneck left to unlock.
- **`mtf::mtf` is `62.1–64.8%`** of `Column` because high-entropy `random`/`records` hit deep MTF ranks (`rank >= 32` on `> 85%` of symbols), forcing serial 256-byte AVX2 table shifts on every symbol.

---

## 3. Round 1 Exploration: Fused `MTF -> Relative-LF`, `u64` `build_lf`, Direct Post-Walk, and `L` / Prefetch Sweeps

### 3.1 Candidate Arms Tested (100% Exact vs `bwt::Plain`: `144/144` `random`, `176/176` `records`)

- **`A0_shipped`**: Exact `origin/master` `Column` (`mtf_avx2` -> `ll: [u8; n]` -> `build_lf` -> `room_segments::<32, PF_L1, false>` -> `assemble_reversed` -> `unrle_in_place` -> `bzip2_crc`). Scratch: `9.39 MiB` (`random`), `9.61 MiB` (`records`).
- **`A1_u64_build_lf`**: Replaces scalar `build_lf` with 8-byte `u64` load + 4x-unrolled scatter into `tt`.
- **`A2_mtf_lf_rel`**: Fuses `MTF` directly into **Relative-LF** (`tt[i] = (occ[c] << 8) | c`, skipping `ll` write + `build_lf` entirely!); `room_segments` resolves `p = (w >> 8) + cum[c]`.
- **`A3_flat7_lf_rel`**: Same Relative-LF fusion using `R_t32_flat_unal7` (tier-32 fast path + flat unaligned 7-row `vmovdqu` shift).
- **`A4_mtf_lf_rel_direct`** / **`A5_flat7_lf_rel_direct`**: Removes the `0.86 MiB` `ll` buffer completely (`9.39 -> 8.53 MiB` on `random`, `9.61 -> 8.75 MiB` on `records`) and replaces `assemble_reversed + unrle + crc` with direct reverse-copy from segment buffers into `out` with an in-register 4-byte run check + L2-hot 64 KiB CRC-32.
- **`A6_flat7_lf_rel_L48`**: `A5` with `L = 48` concurrent walk lanes (also ablated `A7_L24` and `A8_PF_L2` on `base`).
- **`A7_flat7_u64_abs_dir`**: Unfused control (`flat7 -> ll` + `u64` `build_lf` + absolute `L = 32` walk + direct post-walk).

### 3.2 Round 1 Results (`n = 11` Interleaved Sweeps, Pinned CPU 0, Steal-Free Thread CPU)

#### `JCC-32B` (`random` & `records`, Warm and 64 MiB L3-Cold)

| Arm | `random` Warm (`delta%` [95% CI] / `mtf+bl` / `walk` / `post`) | `random` L3-Cold (`delta%` [95% CI] / `mtf+bl` / `walk` / `post`) | `records` Warm (`delta%` [95% CI] / `mtf+bl` / `walk` / `post`) | `records` L3-Cold (`delta%` [95% CI] / `mtf+bl` / `walk` / `post`) | Scratch (`rand / rec`) |
|---|---|---|---|---|---:|
| `A0_shipped` | `310.84 ms` (`+0.00%` / `226.1` / `69.1` / `11.8`) | `325.38 ms` (`+0.00%` / `228.6` / `81.3` / `12.9`) | `348.58 ms` (`+0.00%` / `255.2` / `80.0` / `14.7`) | `358.35 ms` (`+0.00%` / `257.1` / `86.3` / `14.6`) | `9.39 / 9.61 MiB` |
| `A1_u64_build_lf` | `+0.39%` `[0.975, 1.034]` (`224.7 / 74.5 / 12.8`) | `-2.39%` `[0.950, 1.002]` (`226.0 / 76.6 / 12.7`) | `-2.69%` `[0.936, 1.011]` (`252.0 / 87.9 / 14.3`) | `+0.52%` `[0.951, 1.063]` (`254.1 / 83.9 / 14.8`) | `9.39 / 9.61 MiB` |
| `A2_mtf_lf_rel` | `+0.52%` `[0.981, 1.030]` (`220.9 / 73.8 / 12.0`) | `-2.21%` `[0.956, 1.000]` (`220.8 / 79.9 / 12.2`) | `+0.10%` `[0.959, 1.045]` (`258.2 / 92.2 / 14.0`) | `+3.40%` `[0.999, 1.070]` (`262.7 / 93.3 / 15.1`) | `9.39 / 9.61 MiB` |
| `A3_flat7_lf_rel` | `-3.16%` `[0.930, 1.008]` (`211.7 / 77.1 / 12.0`) | `-4.04%` `[0.933, 0.988]` (`214.0 / 82.6 / 12.8`) | `-3.19%` `[0.933, 1.004]` (`246.4 / 93.4 / 14.6`) | `-0.25%` `[0.955, 1.042]` (`248.0 / 90.2 / 14.8`) | `9.39 / 9.61 MiB` |
| `A4_mtf_lf_rel_direct` | `+2.45%` `[0.965, 1.087]` (`222.5 / 80.8 / 11.7`) | `-2.03%` `[0.956, 1.005]` (`221.4 / 84.1 / 11.8`) | `+1.77%` `[0.992, 1.045]` (`257.2 / 87.1 / 13.0`) | `-0.35%` `[0.960, 1.035]` (`256.4 / 87.2 / 13.2`) | **`8.53 / 8.75 MiB`** |
| `A5_flat7_lf_rel_direct` | `-2.42%` `[0.941, 1.012]` (`213.4 / 80.5 / 11.7`) | **`-5.12%`** `[0.933, 0.965]` (`213.7 / 81.7 / 11.7`) | `-1.22%` `[0.937, 1.042]` (`246.8 / 93.7 / 13.2`) | `-3.33%` `[0.927, 1.009]` (`246.3 / 89.4 / 13.0`) | **`8.53 / 8.75 MiB`** |
| `A6_flat7_lf_rel_L48` | `-4.60%` `[0.909, 1.001]` (`209.8 / 74.6 / 11.2`) | **`-5.95%`** `[0.916, 0.966]` (`213.2 / 78.1 / 11.6`) | `-2.32%` `[0.925, 1.031]` (`245.5 / 93.2 / 12.9`) | `-3.13%` `[0.937, 1.001]` (`247.8 / 90.0 / 12.7`) | **`8.53 / 8.75 MiB`** |
| `A7_flat7_u64_abs_dir` | `-0.59%` `[0.960, 1.030]` (`226.2 / 70.4 / 11.5`) | `-1.84%` `[0.961, 1.003]` (`229.1 / 77.9 / 11.5`) | `-0.15%` `[0.967, 1.031]` (`260.6 / 86.1 / 12.9`) | `+0.65%` `[0.969, 1.046]` (`264.2 / 83.9 / 12.7`) | `9.39 / 9.61 MiB` |

#### `base` (`random` Warm & L3-Cold, `records` Warm; plus `L = 24` & `PF_L2` Negative Ablations)

| Arm | `random` Warm (`delta%` [95% CI] / `mtf+bl` / `walk` / `post`) | `random` L3-Cold (`delta%` [95% CI] / `mtf+bl` / `walk` / `post`) | `records` Warm (`delta%` [95% CI] / `mtf+bl` / `walk` / `post`) |
|---|---|---|---|
| `A0_shipped` | `326.65 ms` (`+0.00%` / `235.3` / `78.0` / `13.0`) | `317.78 ms` (`+0.00%` / `231.4` / `71.9` / `12.6`) | `359.98 ms` (`+0.00%` / `259.6` / `84.2` / `14.4`) |
| `A1_u64_build_lf` | `-2.37%` `[0.947, 1.006]` (`230.0 / 73.5 / 12.3`) | `+0.46%` `[0.992, 1.018]` (`229.6 / 75.9 / 13.0`) | `-2.95%` `[0.952, 0.989]` (`253.8 / 82.7 / 13.2`) |
| `A2_mtf_lf_rel` | `-1.34%` `[0.970, 1.004]` (`231.0 / 79.0 / 12.4`) | `+1.49%` `[0.997, 1.033]` (`229.4 / 78.1 / 12.4`) | `+2.93%` `[1.007, 1.052]` (`263.8 / 92.6 / 13.3`) |
| `A3_flat7_lf_rel` | `-4.05%` `[0.929, 0.992]` (`221.9 / 78.4 / 12.4`) | `-1.65%` `[0.959, 1.008]` (`220.6 / 79.6 / 11.8`) | `-0.54%` `[0.970, 1.019]` (`249.7 / 90.8 / 13.9`) |
| `A5_flat7_lf_rel_direct` | `-4.40%` `[0.922, 0.992]` (`222.9 / 80.4 / 10.0`) | `-1.33%` `[0.958, 1.016]` (`222.1 / 77.9 / 10.4`) | `-0.71%` `[0.964, 1.023]` (`248.5 / 93.8 / 13.7`) |
| `A6_flat7_lf_rel_L48` | `-3.83%` `[0.928, 0.996]` (`222.9 / 78.0 / 10.3`) | `-2.66%` `[0.944, 1.004]` (`220.5 / 74.3 / 10.4`) | `-0.72%` `[0.977, 1.009]` (`251.0 / 91.3 / 13.8`) |
| `A7_flat7_u64_abs_dir` | `+2.95%` `[0.988, 1.073]` (`242.6 / 78.2 / 10.5`) | `+1.67%` `[0.983, 1.052]` (`239.7 / 70.3 / 10.5`) | `+2.56%` `[0.998, 1.054]` (`266.0 / 86.9 / 13.8`) |

In the initial `base` sweep (`162/162` `random`, `198/198` `records` exact), reducing lanes to `L = 24` (`A7_L24`) starved MLP (`walk` `+7.9 ms` on `random` L3-cold and `+6.5 ms` on `records` warm vs `A5`), and `PF_L2` (`A8_PFL2`) regressed `walk` by `+8.5 ms` on `random` L3-cold and `+10.5 ms` on `records` warm, confirming `frontier/dead-ends.tsv` Row 28.

---

## 4. `objdump -dC` Audit & Round 2 Microarchitectural Rescue

Inspecting `objdump -dC` of Round 1 exposed two concrete compiler/microarchitectural defects eating the fused `MTF -> Relative-LF` gain:

1. **Stack-spilled `cum.as_ptr()` in `room_segments_local`:** Because `room_segments_local::<32, 1, true, false>` held the entire outer frame (`1629` insns, `327` `(%rsp)` refs), LLVM spilled `cum.as_ptr()` to the stack and reloaded it on every lane step, explaining the `+7..+14 ms` walk penalty in `A2..A5`. Outlining the 32-lane inner step loop into a pure `#[inline(never)]` leaf **`walk_chunk_leaf`** dropped stack traffic to **`rsp = 0`**, pinned `cp` permanently in `%rsi`, and kept `0 vzeroupper / 0 ymm / 0 xmm / 0 calls`.
2. **Store-to-load forwarding stalls in `R_t32_flat_unal7`:** Unaligned `vmovdqu` loads at `offset - 1` (`0x2df(%rsp)`) across aligned 32-byte stores caused 7x store-forwarding stalls per deep symbol. Replacing it with **`Q_signed_flat7`** (`vperm2i128 + vpalignr` aligned lane shifts + signed `vpcmpgtb`) removed all unaligned cross-line loads. We also tested **`mtf_to_lf_rel_p2_branchless`** (branchless 8-register AVX2 shift for `rank >= 32`, `R3`/`R5`) and an unfused `no-memset` `flat7 + u64` control (`R4`).

### 4.1 Disassembly Audit (`objdump -dC`, Round 1 vs Round 2)

| Symbol | Build | Insns | `vzu` | `ymm` | `xmm` | `(%rsp)` | `calls` | Role / Finding |
|---|---|---:|---:|---:|---:|---:|---:|---|
| `room_segments_local::<32, 1, true, false>` (R1) | `JCC-32B` | `1629` | `0` | `0` | `62` | **`327`** | `36` | Spilled `cum.as_ptr()` on every lane step |
| `walk_chunk_leaf::<32, 1, false>` (R2 abs) | `base` / `JCC-32B` | `154 / 169` | `0` | `0` | `0` | **`0`** | `0` | Pure-leaf absolute walk |
| `walk_chunk_leaf::<32, 1, true>` (R2 rel) | `base` / `JCC-32B` | `175 / 201` | `0` | `0` | `0` | **`0`** | `0` | Pure-leaf relative walk (`cp` pinned in `%rsi`) |
| `walk_chunk_leaf::<48, 1, true>` (R2 rel `L=48`) | `base` / `JCC-32B` | `175 / 201` | `0` | `0` | `0` | **`0`** | `0` | Pure-leaf relative walk (`L = 48`) |
| `mtf_to_lf_rel_tier32::<false>` (`R1` `flat7`) | `base` / `JCC-32B` | `519 / 538` | `1` | `99` | `25` | `74` | `2` | Unaligned 7-row `vmovdqu` store-forwarding stalls |
| `mtf_to_lf_rel_tier32::<true>` (`R2` `signed7`) | `base` / `JCC-32B` | `535 / 543` | `1` | `119` | `26` | `67` | `2` | Aligned `vperm2i128+vpalignr` + `vpcmpgtb` (`-8.8 ms` JCC) |
| `mtf_to_lf_rel_p2_branchless` (`R3`/`R5`) | `base` / `JCC-32B` | `520 / 542` | `1` | `123` | `10` | `78` | `2` | Branchless 8-reg AVX2 shift (`+8.7..+16.3%` slower) |
| `build_lf_shipped` / `build_lf_u64store` | `JCC-32B` | `236 / 256` | `0` | `0` | `30` | `2 / 4` | `4` | Already tight; `u64` scatter is noise on `JCC-32B` |

### 4.2 Round 2 Results (`n = 11` Interleaved Sweeps, 100% Exact: `108/108` `random`, `132/132` `records`)

#### `JCC-32B` (`random` & `records`, Warm and 64 MiB L3-Cold)

| Arm | `random` Warm (`delta%` [95% CI] / `mtf+bl` / `walk` / `post`) | `random` L3-Cold (`delta%` [95% CI] / `mtf+bl` / `walk` / `post`) | `records` Warm (`delta%` [95% CI] / `mtf+bl` / `walk` / `post`) | `records` L3-Cold (`delta%` [95% CI] / `mtf+bl` / `walk` / `post`) | Scratch (`rand / rec`) |
|---|---|---|---|---|---:|
| `R0_shipped` | `318.50 ms` (`+0.00%` / `226.0` / `79.0` / `12.7`) | `319.48 ms` (`+0.00%` / `226.8` / `80.6` / `12.8`) | `362.14 ms` (`+0.00%` / `255.5` / `89.5` / `14.5`) | `358.11 ms` (`+0.00%` / `257.5` / `88.1` / `14.8`) | `9.39 / 9.61 MiB` |
| `R1_outl_rel_flat7` | `-0.82%` `[0.972, 1.013]` (`222.5 / 80.0 / 11.8`) | `+1.33%` `[0.989, 1.039]` (`225.5 / 83.3 / 11.8`) | `+1.51%` `[0.995, 1.036]` (`256.9 / 95.9 / 13.9`) | `-0.63%` `[0.966, 1.023]` (`253.9 / 92.5 / 13.4`) | `8.53 / 8.75 MiB` |
| **`R2_outl_rel_signed7`** | **`-4.21%`** `[0.936, 0.980]` (`213.8 / 77.8 / 11.5`) | **`-3.25%`** `[0.944, 0.991]` (`214.7 / 82.4 / 11.9`) | **`-0.66%`** `[0.982, 1.005]` (`250.9 / 93.9 / 13.2`) | **`-0.95%`** `[0.967, 1.015]` (`251.6 / 91.9 / 13.6`) | **`8.53 / 8.75 MiB`** |
| `R3_outl_rel_adapt_p2` | `+8.68%` `[1.065, 1.109]` (`251.8 / 81.8 / 12.2`) | `+9.50%` `[1.063, 1.128]` (`254.2 / 81.6 / 12.0`) | `+14.43%` `[1.123, 1.167]` (`300.8 / 96.1 / 14.2`) | `+16.27%` `[1.144, 1.182]` (`306.2 / 96.8 / 13.4`) | `8.53 / 8.75 MiB` |
| `R4_outl_abs_flat7_u64` | `-1.12%` `[0.966, 1.013]` (`224.6 / 77.9 / 11.9`) | `-2.94%` `[0.954, 0.987]` (`224.2 / 74.1 / 11.4`) | `+0.78%` `[0.993, 1.023]` (`260.2 / 91.0 / 13.8`) | `+2.36%` `[1.002, 1.046]` (`265.2 / 88.7 / 13.9`) | `9.39 / 9.61 MiB` |
| `R5_outl_rel_adapt_L48` | `+6.65%` `[1.044, 1.089]` (`250.5 / 77.0 / 11.4`) | `+8.51%` `[1.048, 1.124]` (`250.9 / 77.8 / 11.1`) | `+14.18%` `[1.126, 1.158]` (`304.5 / 94.6 / 13.7`) | `+11.43%` `[1.090, 1.140]` (`299.9 / 90.3 / 13.5`) | `8.53 / 8.75 MiB` |

#### `base` (`random` Warm & L3-Cold)

| Arm | `random` Warm (`delta%` [95% CI] / `mtf+bl` / `walk` / `post`) | `random` L3-Cold (`delta%` [95% CI] / `mtf+bl` / `walk` / `post`) | Scratch |
|---|---|---|---:|
| `R0_shipped` | `335.34 ms` (`+0.00%` / `247.8` / `73.7` / `12.6`) | `356.35 ms` (`+0.00%` / `253.7` / `88.9` / `13.0`) | `9.39 MiB` |
| `R1_outl_rel_flat7` | `-3.66%` `[0.930, 0.998]` (`231.7 / 73.6 / 11.7`) | `-2.24%` `[0.944, 1.012]` (`239.4 / 88.6 / 12.4`) | `8.53 MiB` |
| **`R2_outl_rel_signed7`** | **`-3.67%`** `[0.944, 0.983]` (`231.9 / 77.4 / 11.8`) | **`-3.67%`** `[0.946, 0.981]` (`238.4 / 89.8 / 12.4`) | **`8.53 MiB`** |
| `R3_outl_rel_adapt_p2` | `+11.97%` `[1.091, 1.149]` (`279.3 / 80.2 / 12.1`) | `+10.97%` `[1.080, 1.141]` (`286.1 / 88.2 / 12.2`) | `8.53 MiB` |
| `R4_outl_abs_flat7_u64` | `-4.04%` `[0.934, 0.986]` (`229.4 / 75.7 / 12.4`) | `-6.13%` `[0.926, 0.952]` (`234.0 / 86.3 / 12.4`) | `9.39 MiB` |
| `R5_outl_rel_adapt_L48` | `+11.02%` `[1.096, 1.125]` (`280.5 / 78.8 / 11.6`) | `+9.83%` `[1.081, 1.116]` (`286.0 / 83.7 / 12.2`) | `8.53 MiB` |

---

## 5. Mechanism Diagnosis & Calibrated i9 Transfer Pricing

1. **Why fusing `MTF -> Relative-LF` only nets `~12 ms` (`-3.3..-4.2%` `random`, `-0.7..-1.0%` `records` `JCC-32B`) out of `build_lf`'s `24.5–29.3 ms`:**
   - Writing `3.6 MiB` of `u32` `tt` directly inside the serial MTF dependency chain (plus random RMW increments of `occ[256]`) adds `~12–17 ms` of L3 write-allocate / RFO pressure compared to streaming `0.9 MiB` of `u8` `ll`.
   - Even with `walk_chunk_leaf` eliminating every stack spill (`rsp = 0`, matching absolute walk on `random`: `77.8 vs 79.0 ms` JCC warm), the extra `add (%rsi,%rdx,4), %eax` (`cum[c]`) still costs `+3.8..+4.4 ms` on `records` (`93.9 vs 89.5 ms` warm, `91.9 vs 88.1 ms` cold) because `records` has higher byte skew and tighter walk dependency chains.
   - `P2` branchless 8-register AVX2 (`R3`/`R5`) regresses `+8.7..+16.3%` because it executes 8 full vector shifts + blends unconditionally on every symbol whereas `Q_signed_flat7` short-circuits `rank < 32` (`tier-32`) and only touches the prefix up to `rank / 32`.
2. **Why `R4_outl_abs_flat7_u64` wins `base` (`-4.04% / -6.13%` `random`) yet collapses on `JCC-32B` (`-1.12% / -2.94%` `random`, `+0.78% / +2.36%` `records`):**
   - Classic Haswell unaligned JCC artifact (`#1119`), confirming why `JCC-32B` is mandatory on this VM before promoting any candidate.
3. **Transfer-ledger pricing (`#1368`, `docs/notebook/2026-11-28-vm-i9-transfer-ledger.md`):**
   - `R2_outl_rel_signed7` is a Station `W` compute / pass-fusion lever (`0.45x` prior `[0.20x–0.47x]` anchored on `JCC-32B`). Applying `0.45x` to `-3.25..-4.21%` (`random`) and `-0.66..-0.95%` (`records`) predicts **`~-1.5%` on i9 `random/1W`** and **`~-0.4%` on `records/1W`** (within whole-decoder noise).
   - At `4W–10W`, both `random` (`k* = 3.90`) and `records` (`k* = 3.68`) are **scanner-bound**, so any `Column` worker speedup is completely masked by Station `S` (`0%` wall gain).
   - Primary corpora (`wikidata`, `text`, `prose`, `elf`) execute **`0` full `Column` blocks**.

**Honest C2 Decision:** **STOP.** Record a scoped dead-end in `frontier/dead-ends.tsv` closing fused `MTF -> Relative-LF`, `u64`-store `build_lf`, direct reverse-to-`out` post-walk, and branchless `P2` MTF on `Column` (`random`/`records`) for baseline AVX2 x86-64. This does **not** close AVX-512 `vpermb` MTF (which attacks the `64%` MTF ceiling directly on AVX-512 hardware) or Station `S` scanner optimizations (`u8-syms-handover`, `scanner-a3-k3-pipe-u8`) which transfer ~`1:1` to `random`/`records` wall at all thread counts.
