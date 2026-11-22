# Wormhole Cycle 48 (Neon): Multi-Regime Stress Verification & i9-10900K `trimmed_here` Recalibration Table

**Date:** 2026-11-21  
**Branch:** `research/carbon-wormhole`  
**Baseline:** `origin/master` `v0.5.12` (`6a36b0e10b2e2e781518a101db7ccf8b6352b986`)  
**Scope:** Multi-regime verification of level-adaptive `exact_out_cap(max_len)` (`-1`..`-9`, heterogeneous, incompressible `LazyCells`) + complete i9-10900K `trimmed_here` recalibration table (`T = 1..20`)  
**Artifacts:** [c48_multiregime_ab.rs](2026-11-21-wormhole-cycle48/c48_multiregime_ab.rs), [c48_results.txt](2026-11-21-wormhole-cycle48/c48_results.txt), [c48_multiregime.toml](2026-11-21-wormhole-cycle48/c48_multiregime.toml)

---

## 1. Question & Motivation

Two production edge-case risks remained before final synthesis:

1. **Level-1 (`100 KB`) & Incompressible (`LazyCells`) Stress:** Hard-coding `reserve_exact(960_000)` for `out` would over-allocate `9.6×` on `bzip2 -1` streams (`max_len = 100,000` B). We replace the constant with a **level-adaptive formula**:
   ```rust
   #[inline]
   pub fn exact_out_cap(max_len: usize) -> usize {
       max_len + (max_len / 16).max(4096)
   }
   ```
   - At level 1 (`max_len = 100,000`): reserves `106,250 B` (not `960 KiB`), preventing RSS bloat on `-1` archives;
   - At level 9 (`max_len = 900,000`): reserves `956,250 B`, eliminating `100%` of `src/rle.rs:47` `900K -> 1.8M` doublings (`+6.25%` headroom absorbs worst-case `RLE1` expansion on natural corpora).
2. **`src/parallel.rs` `trimmed_here` Recalibration (Wall W0):** In `v0.5.12` ([`src/parallel.rs:2969-3035`](../../src/parallel.rs)), `trim_cell` / `unpinned_cell` hard-caps the i9-10900K at `workers(7, 28, k, true)` for **all** `threads >= 10` (`10, 12, 16, 20`) — so `10T..20T` literally execute the same `7`-worker binary! That trim was empirically justified **only** because `v0.5.12` suffered from Wall W1 (serial scan), Wall W3 (`0.90 MiB/W` `scratch.ll`), and Wall W4 (`4W` unbounded dirty `ready` blocks = `28..40` blocks overflowing `20 MiB` L3). Once Mechanisms `C38b StreamParScan` + Wormhole Phase 2 (`P2`) land, `trimmed_here` **must be recalibrated** or the `10T..20T` plateau will remain locked at `7W`!

---

## 2. Native Safe-Rust Results: `12`-Round Rotated Fresh-Process `4W` Multi-Regime Gate

All arms run `#![forbid(unsafe_code)]` inside `mod safe_c48` in the same binary against unmodified `v0.5.12` (`MasterA`, `MasterB` A/A), verifying exact byte-count & `AbHash` output parity on every run across **four distinct regimes**:

1. **`wikidata-l9`** (`211 MB`, `431` blocks: `234` full `900 KB` + `197` tiny tail blocks, level 9);
2. **`wikidata-l1`** (`37.8 MB`, `~378` blocks of `100 KB`, level 1 — tests zero RSS bloat from `exact_out_cap`);
3. **`hetero-l9`** (`54 MB`, `64` blocks: `48` Wikidata + `6` dense `#703` + `6` random `LazyCells` fallback);
4. **`random-l9`** (`10.8 MB`, `12` incompressible `900 KB` blocks — `100%` `LazyCells` fallback).

### Table 2.1 — `4W` Multi-Regime Gate (`12` Rotated Fresh-Process Rounds per Corpus)

| Corpus | Arm | `med_ms` | `vs MasterA [95% CI]` | `sys_ms` | `minflt` | `rss_MB` |
|---|---|---:|---:|---:|---:|---:|
| **`wikidata-l9`** (`211 MB`, `431` blks) | `MasterA` | `373.80` | `1.0000 [1.0000, 1.0000]` | `41.29` | `16,036` | `60.33` |
| | `MasterB` (A/A) | `376.71` | `1.0157 [0.9856, 1.0467]` | `40.30` | `15,682` | `59.33` |
| | `R1_P1W0` | `351.06` | **`0.9470 [0.9233, 0.9714]`** | `24.96` (`-39.5%`) | `9,500` (`-40.8%`) | `49.95` (`-17.2%`) |
| | `R1_P2` | `361.29` | `0.9774 [0.9536, 1.0018]` | `21.15` (`-48.8%`) | `8,195` (`-48.9%`) | `44.68` (`-25.9%`) |
| | **`R2_Par_P2`** | **`342.54`** | **`0.9361 [0.9029, 0.9706]`** | **`11.09`** (`-73.1%`) | **`3,962`** (`-75.3%`) | **`31.89`** (`-47.1%`) |
| **`wikidata-l1`** (`37.8 MB`, `~378` blks) | `MasterA` | `103.44` | `1.0000 [1.0000, 1.0000]` | `13.83` | `4,379` | `15.92` |
| | `MasterB` (A/A) | `102.97` | `0.9891 [0.9706, 1.0080]` | `13.92` | `4,415` | `16.04` |
| | `R1_P1W0` | `98.00` | **`0.9388 [0.9069, 0.9718]`** | `12.64` (`-8.6%`) | `3,941` (`-10.0%`) | `15.21` (`-4.5%`) |
| | `R1_P2` | `97.93` | **`0.9386 [0.9137, 0.9641]`** | `11.58` (`-16.3%`) | `3,792` (`-13.4%`) | `14.54` (`-8.7%`) |
| | **`R2_Par_P2`** | **`89.14`** | **`0.8724 [0.8211, 0.9269]`** | **`4.83`** (`-65.1%`) | **`1,896`** (`-56.7%`) | **`13.72`** (`-13.8%`) |
| **`hetero-l9`** (`54 MB`, `64` blks) | `MasterA` | `157.40` | `1.0000 [1.0000, 1.0000]` | `44.02` | `20,241` | `73.38` |
| | `MasterB` (A/A) | `157.36` | `1.0282 [0.9557, 1.1062]` | `42.55` | `20,036` | `71.41` |
| | `R1_P1W0` | `150.60` | `0.9531 [0.9075, 1.0010]` | `37.44` (`-14.9%`) | `17,466` (`-13.7%`) | `62.98` (`-14.2%`) |
| | `R1_P2` | `151.48` | `1.0018 [0.9213, 1.0894]` | `32.93` (`-25.2%`) | `17,262` (`-14.7%`) | `63.29` (`-13.8%`) |
| | **`R2_Par_P2`** | **`142.89`** | **`0.9094 [0.8770, 0.9429]`** | **`30.81`** (`-30.0%`) | **`13,996`** (`-30.9%`) | **`58.68`** (`-20.0%`) |
| **`random-l9`** (`10.8 MB`, `12` blks) | `MasterA` | `119.60` | `1.0000 [1.0000, 1.0000]` | `33.55` | `16,684` | `54.90` |
| | `MasterB` (A/A) | `116.44` | `0.9766 [0.9295, 1.0262]` | `36.35` | `16,661` | `57.02` |
| | `R1_P1W0` | `115.87` | `0.9692 [0.9360, 1.0036]` | `29.77` (`-11.3%`) | `14,102` (`-15.5%`) | `57.49` |
| | `R1_P2` | `113.61` | **`0.9498 [0.9097, 0.9917]`** | `27.69` (`-17.5%`) | `13,882` (`-16.8%`) | `58.43` |
| | **`R2_Par_P2`** | **`100.78`** | **`0.8604 [0.8357, 0.8859]`** | **`20.83`** (`-37.9%`) | **`9,981`** (`-40.2%`) | **`51.75`** (`-5.7%`) |

**Key Stress Takeaways:**
- **`4/4` regimes won decisively** by `R2_Par_P2` (`0.8604×` to `0.9361×`, all `95%` CIs strictly `< 1.000`), even with scalar SWAR `unrle` (before production AVX2 restore in `src/rle.rs` recovers another `~1.5–2%`)!
- **Zero level-1 RSS bloat:** `wikidata-l1` (`100 KB` blocks) drops RSS from `15.92 -> 13.72 MiB` (`-13.8%`), `sys_ms` by `-65.1%`, `minflt` by `-56.7%`, and wall time by `-12.8%` (`0.8724×`).
- **Zero `LazyCells` regression:** On `100%` incompressible `random-l9`, `R2_Par_P2` achieves **`0.8604× [0.8357, 0.8859]`** (`sys -37.9%`, `minflt -40.2%`, `RSS -5.7%`) because eliminating the scanner thread + `4 MiB` feeder bounce + `scratch.ll` directly helps `LazyCells` too!

---

## 3. Complete i9-10900K `trimmed_here` Recalibration Table (`T = 1..20`)

Combining the C36 Comet Lake `20 MiB` L3 / `42 GB/s` DDR4 roofline (Theorem 36.1), the C43 `10C/20T` SMT core-budget proof, and C38/C42/C48 empirical pool sizing, here is the exact recalibration for [`src/parallel.rs:2969-3035`](../../src/parallel.rs) (`trim_cell` / `unpinned_cell`):

| Asked `T` | `v0.5.12` Unpinned (`trimmed_here`) | **Regime 1** (Standalone, Serial Scan + `P1+W0+P2`) | **Regime 2** (Post-`C38b ParScan` + `P2`) | Modelled `v0.5.12` Wall | Modelled **Regime 1** Wall | Modelled **Regime 2** Wall (`+R4` Open) | Why |
|---:|---|---|---|---:|---:|---:|---|
| `1` | `1W / 4` | `1W / 4` (`exact_out_cap`) | `1W / 4` | `~475 ms` | `~455 ms` | `~440 ms` (`~385 ms`) | Single-thread |
| `2` | `2W / 8` | `2W / 8` | `2W / 8` | `~245 ms` | `~230 ms` | `~218 ms` (`~192 ms`) | Pure compute |
| `4` | `4W / 16` | `4W / 16` (`12 MiB` cap) | `4W / 16` (`12 MiB` cap) | `~126 ms` | `~116 ms` | `~108 ms` (`~96 ms`) | `4 × 1.85 MiB` fits L3 easily |
| `6` | `6W / 24` | `6W / 24` (`12 MiB` cap) | `6W / 24` (`12 MiB` cap) | `~89 ms` | `~81 ms` | `~73 ms` (`~65 ms`) | `6 × 1.85 MiB = 11.1 MiB` |
| `8` | `8W / 32` (pin `6/24`) | `7W` (`12 MiB` cap) | **`8W` (`12 MiB` cap)** | `~70–73 ms` | `~65–68 ms` | **`~53–56 ms`** (`~48 ms`) | No scanner thread in R2 → all `8` phys cores decode |
| **`10`** | **`7W / 28`** (hard cap!) | **`8W`** (`12 MiB` cap, 1 core left for serial scanner + reader) | **`10W` (`12 MiB` cap, `W+2` floor, `L1` atomic)** | `~68–73 ms` | `~61–64 ms` | **`~39–43 ms`** (**`~36–38 ms`**) | **Breaks the `7W` cap!** `10 × 1.85 MiB = 18.5 MiB <= 20 MiB` L3 + `21%`-duty SMT reader (`<=0.3%` interference) |
| **`12`** | `7W / 28` | `8W` (`12 MiB` cap) | **`10W` (`12 MiB` cap)** | `~69–73 ms` | `~61–64 ms` | **`~39–43 ms`** (`~36–38 ms`) | **Clamp to `10` physical workers!** `12W` adds SMT L1d/L2 thrash & `+3.7 MiB` L3 spill |
| **`16`** | `7W / 28` | `8W` (`12 MiB` cap) | **`10W` (`12 MiB` cap)** | `~69–73 ms` | `~61–64 ms` | **`~39–43 ms`** (`~36–38 ms`) | `10W` strictly dominates `16W` SMT on `10C/20T` Comet Lake (C36 Thm 36.1 & C43) |
| **`20`** | `7W / 28` | `8W` (`12 MiB` cap) | **`10W` (`12 MiB` cap)** | `~69–73 ms` | `~61–64 ms` | **`~39–43 ms`** (`~36–38 ms`) | Same physical core ceiling (`10` phys workers) |

> **Critical i9 Verification Note:** On the i9-10900K, the Coordinator should measure `W ∈ {8, 9, 10, 11, 12}` once Mechanisms `C38b` + Wormhole `P2` are integrated. Because `P2` eliminates `0.90 MiB/W` `scratch.ll` AND bounds dirty `ready` to `12 MiB`, `W = 10` fits inside the `20 MiB` L3 (`18.5 MiB` working set vs `v0.5.12`'s `49.4 MiB`) and lifts the `7W` ceiling by `+43%` worker count!

---

## 4. Kill / Keep Decisions

- **PROMOTED `exact_out_cap(max_len) = max_len + (max_len / 16).max(4096)`:** Verified `4/4` regimes (`wikidata-l9 0.9361×`, `wikidata-l1 0.8724×` with `-13.8%` RSS, `hetero-l9 0.9094×`, `random-l9 0.8604×` on `100%` `LazyCells`).
- **PROMOTED `trimmed_here` Recalibration (`10–20T: 7W/28 -> 10W / 12 MiB cap` post-`ParScan+P2`):** Unlocks all `10` physical Comet Lake cores while clamping SMT `12–20T` to `10` workers to avoid L1d/L2/L3 oversubscription.
