# Explore: Residual B4 `A5+A6a` (`src/runs/fold.rs` + `FoldSink` + `OutQ16`) Dissection

**Scope & honesty notice:** All timings and disassembly counts in this note were measured on a shared Linux KVM VM (`rustc 1.99.1`, pinned CPU 0, steal-free `CLOCK_THREAD_CPUTIME_ID`, `[profile.release]` `opt-level = 3`, `lto = "fat"`, `codegen-units = 1`) across three compiler profiles: default baseline x86-64 (`base`), JCC-32B-aligned (`jcc`, `-C llvm-args=-x86-branches-within-32B-boundaries`), and native Haswell (`native`, `-C target-cpu=native`). **These are VM exploration measurements, NOT physical i9-10900K adoption evidence.**

## 1. Motivation

In `#1305` (`docs/notebook/2026-11-28-explore-composed-worker-stack.md`), residual per-stage profiling of the 4-way composed worker stack (`B4`: `#1217` + `#1294` + `L6a` + `mtf-fold-guard-rescue`) across all 233 Open blocks of `wikidata.bz2` showed `A5+A6a` (`src/runs/fold.rs` + `FoldSink` + `OutQ16<1008,1024>`) is the **#2 remaining worker stage at `30.0%` (`177.81 ms` `jcc`) / `31.1%` (`203.03 ms` `base`)** of `B4` worker CPU. Meanwhile, `#1274` (`2f02a288`, physical i9 `PASS` on `origin/evidence 95edea2a`) just landed the 64-symbol `u64 vptest` Pass 1 in `src/runs.rs`, leaving `src/runs/fold.rs` on the older 32-symbol Pass 1 + unhoisted `FoldSink`/`OutQ16` hot path.

## 2. Internal Sub-Stage Split of `F0` (`74967ee2`, 233 Wikidata Open Blocks)

Census across all 233 Open blocks of `corpora/wikidata.bz2`:
- `dense_blocks = 31 / 233` (`lits >= bound / 10`, entering `bitmap_within_dense_cap`)
- `total_runs = 15,141,602` (`14,386,581` thin `<= 16`, `755,021` thick `> 16`, `40,597` `push_slow` calls)

Uninstrumented warm / L3-cold (`24 MiB` LLC flush between blocks) totals and isolated sub-stage medians (`n = 7`, zero per-batch `clock_gettime` perturbation for Pass 1 and dense prescan; Round-1 per-batch split for Pass 2 vs Sink ratio):

| Sub-stage of `F0` (`74967ee2`) | `base` (ms) | `jcc` (ms) | `native` (ms) | Notes |
|---|---:|---:|---:|---|
| **Total `A5+A6a` (warm, uninstrumented)** | `158.46` | `166.43` | `168.16` | `n = 9` interleaved |
| **Total `A5+A6a` (L3-cold, uninstrumented)** | `179.78` | `173.54` | `182.20` | `n = 5` interleaved (`24 MiB` LLC flush/block) |
| Isolated Pass 1 (`build_bitmaps_avx2`, 32-sym) | `9.01` (`5.7%`) | `8.82` (`5.3%`) | `8.94` (`5.3%`) | `363–385` insns, `4` `vzeroupper`, `vmax` + full tail zeroing |
| Isolated dense prescan (`bitmap_within_dense_cap`, 31/233 blocks) | `10.41` (`6.6%`) | `10.28` (`6.2%`) | `8.40` (`5.0%`) | Pure scalar loop outside `#[target_feature]` (`232–258` insns) |
| `buf.resize(bound + INLINE, 0)` (preserved `len`) | `0.25` (`0.2%`) | `0.28` (`0.2%`) | — | **If `buf.len() == 0` (cleared): `21.52 ms` `base` / `22.32 ms` `jcc` (`13.4%`!)** |
| Remainder (`Pass 2` + `Sink/OutQ16`, `~88%`) | `~139` | `~147` | `~151` | Round-1 split: `~41–47%` `fill_batch_avx2` + `~53–59%` `FoldSink`/`OutQ16` |

### Critical Composition Catch: `buf.len()` Preservation (`21.5–22.3 ms` `memset(0)` Tax)
When `buf` preserves `len >= bound + INLINE` across blocks, `buf.resize(bound + INLINE, 0)` costs `0.25–0.28 ms`. When `buf` is cleared (`len == 0`) before the next block — as happens when `L6a`'s `walk_fused_same` calls `out.clear()` and `decode_folded` swaps `out` with `s.ll` — `buf.resize(bound + INLINE, 0)` zeroes `~210 MiB` (`233 × 900 KiB`) at **`21.52 ms` (`base`) / `22.32 ms` (`jcc`)** (`~13.4%` of `A5+A6a`). Any future `mtf-fold` + `L6a` integration must preserve `s.ll.len()` rather than clearing it to `0`.

## 3. Exactness & Two-Round Optimization Sweep

All candidate arms were verified bit-for-bit identical against `F0` on **every** output field (`n`, `records`, `f.m`, `f.mark`, `f.mark_byte`, `f.runs`, `ns`, `thin[0..256]`, `thick[0..256]`, `thick_rows[0..256]`, all `3 * ns` strip words, every byte's complete `OutQ16` rank stream, and `spill.cuts`) across all **233/233 Wikidata Open blocks**: **`1,631 / 1,631` checks PASS** in Round 1 and **`1,631 / 1,631` checks PASS** in Round 2 across all 3 compiler builds (**`4,893 / 4,893` exact checks total**).

### Round 1 — Isolating Real Gains vs JCC Artifacts (`base` & `jcc`, `n = 9` warm / `n = 5` cold)

| Arm | Mechanism | `base` warm (`vs F0`) | `jcc` warm (`vs F0`) | `jcc` cold (`vs F0`) | Verdict |
|---|---|---:|---:|---:|---|
| **`F0`** | Exact `74967ee2` baseline (`191.24 ms` `base` / `156.52 ms` `jcc` warm) | `+0.00%` | `+0.00%` | `+0.00%` | Baseline |
| **`F1`** | `+ 791ee415` 64-sym `u64 vptest` Pass 1 (zero `vmax`, 2 trailing zero words, O(1) `s2u_injective`) | `-0.74%` | `-0.18%` | `+1.38%` | Real Pass 1 win (`-11.3%..-13.6%` isolated, `363 -> 241` insns, `vzu 4 -> 1`) |
| **`F2`** | `F1` + duplicated inner `while w != 0` in `fill_batch_avx2` | `-13.85%` | **`+4.31%`** (`+7.59%` P2) | **`+3.74%`** | **REJECTED (negative ablation):** `base` win was JCC-32B luck; duplicating `mtf_step!` spilled `spw` & `out0` (`rld 16 -> 26`) |
| **`F3`** | `F1` + `#![forbid(unsafe_code)]` packed `Slot8 { cur, end }` `OutQ16` + `NO_MARK` batch hoisting | **`-7.76%`** | **`-7.76%`** `[0.916, 0.929]` | **`-6.46%`** `[0.921, 0.951]` | **REAL CROSS-PROFILE WIN** (identical `-7.76%` on both `base` and `jcc`!) |
| **`F5`** | `F2` + 16-byte `Slot16 { cur, end, thin, _pad }` (`4 KiB`) | `-10.60%` | **`+3.45%`** (`+7.8%` vs `F4`) | **`+5.13%`** | **REJECTED (negative ablation):** doubling slot table `2 -> 4 KiB` hurts L1d vs `Slot8` |

### Round 2 — Surgical Stack (`F7..F11`, `n = 9` warm / `n = 5` L3-cold across `base`, `jcc`, `native`)

Guided by Round-1 `objdump -dC`, Round 2 replaced `F2` with a clean single-loop `fill_batch_rows` (`+1..+3` insns to return `(cnt, brows)` for 1-check `NO_MARK`, zero `spw` spill), split-borrowed `out` & `slots` from cold `OutQMeta` so LLVM hoists `out.ptr` & `out.len()` out of the 14.39M-run thin loop in 100% safe Rust, outlined `#[cold] push_strip_cold`, and accelerated `bitmap_within_dense_cap` with `#[target_feature(enable = "bmi1,bmi2,lzcnt")]` + inner `while w != 0` + `_bzhi_u64`:

| Arm | `base` warm (`vs F0`) | `base` cold (`vs F0`) | `jcc` warm (`vs F0`) | `jcc` cold (`vs F0`) | `native` warm (`vs F0`) | `native` cold (`vs F0`) |
|---|---:|---:|---:|---:|---:|---:|
| **`F0_base_74967ee2`** | `158.46 ms` | `179.78 ms` | `166.43 ms` | `173.54 ms` | `168.16 ms` | `182.20 ms` |
| **`F1_p1_vptest64`** | `-0.18%` | `-1.18%` | `-0.20%` | `-1.91%` | `-0.84%` `[0.987, 0.997]` | `-1.61%` |
| **`F7_hoist_clean_512`** | **`-7.60%`** `[0.895, 0.954]` | **`-5.78%`** `[0.898, 0.988]` | **`-9.26%`** `[0.883, 0.933]` | **`-7.51%`** `[0.892, 0.959]` | **`-19.92%`** `[0.791, 0.811]` | **`-14.81%`** `[0.819, 0.886]` |
| **`F8_f7+bmi2_prescan`** (`BATCH=512`) | **`-12.28%`** `[0.867, 0.887]` | **`-8.93%`** `[0.883, 0.940]` | **`-12.62%`** `[0.868, 0.880]` | **`-9.16%`** `[0.869, 0.950]` | **`-21.94%`** `[0.773, 0.789]` | **`-14.62%`** `[0.797, 0.915]` |
| **`F9_f8_batch256`** (`BATCH=256`) | `-11.37%` | `-8.43%` | `-12.00%` | `-9.14%` | `-20.36%` | `-14.18%` |
| **`F10_f8_batch1024`** (`BATCH=1024`) | **`-12.94%`** `[0.859, 0.883]` | **`-9.16%`** `[0.875, 0.943]` | **`-13.33%`** `[0.855, 0.878]` | **`-8.76%`** `[0.868, 0.959]` | **`-21.43%`** `[0.780, 0.791]` | **`-14.95%`** `[0.816, 0.887]` |
| **`F11_f8_no_prescan`** (floor) | `-16.83%` | `-11.30%` | `-17.02%` | `-13.34%` | `-25.87%` | `-17.52%` |

- **Isolated Pass 1 (`F0 -> F1`):** `9.01 -> 7.97 ms` (`-11.48%` `base`), `8.82 -> 7.62 ms` (`-13.60%` `jcc`), `8.94 -> 7.92 ms` (`-11.33%` `native`).
- **Isolated Dense Prescan (`31/233` blocks, scalar -> `bitmap_prescan_bmi2`):** `10.41 -> 6.11 ms` (**`-41.31%`** `base`), `10.28 -> 6.34 ms` (**`-38.36%`** `jcc`), `8.40 -> 6.40 ms` (**`-23.76%`** `native`).
- **Whole-`B4` worker leverage of `F8` (`A5+A6a` = `30.0%` `jcc` / `31.1%` `base` of `B4`):** **`-3.7%..-4.0%`** (`base`/`jcc`, `~-20..22 ms` 1W VM) and **`-6.6%`** (`native`, `~-36.7 ms`) of total `B4` worker CPU at **0 B extra heap/RSS**.

## 4. `objdump -dC` Hygiene Audit (`base`, `jcc`, `native`)

| Function | `base` insns (`vzu/ymm`, `rld/rst`) | `jcc` insns (`vzu/ymm`, `rld/rst`) | `native` insns (`vzu/ymm`, `rld/rst`) |
|---|---|---|---|
| `pass1_base_32` -> `pass1_vptest_64` | `363` (`4/40`) -> **`241`** (`1/47`, `11/5`) | `385` (`4/40`) -> **`244`** (`1/47`, `11/5`) | `364` (`4/40`) -> **`243`** (`1/47`, `11/5`) |
| `fill_batch_base::<512>` -> `fill_batch_rows::<512>` | `241` -> **`244`** (`0/0`, `19/18`) | `258` -> **`259`** (`0/0`, `19/18`) | `241` -> **`246`** (`0/0`, `19/18`) |
| `flush_base_inner::<true>` -> `flush_slot8_hoisted::<true,true>` | `439` (`80/36`) -> **`124`** (`0/0`, **`18/13`**) | `455` (`80/36`) -> **`153`** (`0/0`, **`18/13`**) | `446` (`11/12`, `80/36`) -> **`129`** (`4*/6`, **`18/13`**) |
| `OutQMeta::push_slow` | `318` (**`0/0`**, `34/14`) | `334` (**`0/0`**, `34/14`) | `318` (**`0/0`**, `34/14`) |
| `push_strip_cold` | `100` (**`0/0`**, `9/5`) | `110` (**`0/0`**, `9/5`) | `100` (**`0/0`**, `9/5`) |
| `bitmap_prescan_scalar` -> `bitmap_prescan_bmi2` | `232` (`24/7`) -> **`188`** (`0/0`, **`9/0`**) | `258` (`24/7`) -> **`180`** (`0/0`, **`9/0`**) | `215` (`25/7`) -> **`190`** (`0/0`, **`9/0`**) |

*\*On `native`, `flush_slot8_hoisted::<true,true>` emits a 4-instruction AVX2 32-byte iota store (`vmovd + vpbroadcastw + vpaddw + vmovdqu %ymm0,(%r8,%r10,1)`) with `0` `vzeroupper` in the thin-run loop and only `4` `vzeroupper`s strictly before function exit / cold calls (`push_strip_cold`, `push_slow`, panic, `ret`), down from `11` in `flush_base_inner`.*

## 5. Scratch / RSS Accounting & Admitted Claim

- **Heap / RSS impact:** **0 B extra heap.** `Slot8` (`[Slot8; 256]` = `2 KiB` on stack) replaces `[u32; 256]` `fill` + `[u32; 256]` `tails` (`2 KiB` on stack) at identical footprint; `BATCH = 512` (`2 KiB` stack) is unchanged; `pass1_vptest_64` writes only 2 trailing zero `u64` words.
- **Admitted claim:** Because `F8` clears the `>= 10%` `A5+A6a` (`>= 3%` `B4` worker CPU) bar across all three builds (`-12.28%` `base`, `-12.62%` `jcc`, `-21.94%` `native` warm; `-8.93%..-14.62%` L3-cold; `0` `vzeroupper`/`0` `ymm` in `fill_batch_rows` & `push_slow`; `0 B` extra RSS), filed `frontier/claims/fold-a5a6a-fast-pass1-pass2.toml` (`status = "idea"`, sequenced behind `mtf-fold-streaming-admission`).
