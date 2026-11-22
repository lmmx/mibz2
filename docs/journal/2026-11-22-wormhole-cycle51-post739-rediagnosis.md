# Wormhole Cycle 51 (Neon) — Post-`#739` (`3296195e3`) Re-Diagnosis: Six-Cost Separation, `10W` DRAM `B/B`, & Binding Wall Audit

**Date:** 2026-11-22  
**Branch:** `research/carbon-wormhole` (merged `origin/master` `3296195e3`, strictly docs-only diff)  
**Baseline:** `origin/master` **`3296195e3`** (`#739`, Wormhole's plateau stack ported as PR 1–3).  
**Real i9-10900K Anchor Points (from `#740`/`#744` & `#741`/`#742`):**
- `#739` on i9: **`0.963 [0.943, 0.983]` wall at `10T`** (`6–16T` faster, `20T` inconclusive), CPU **`0.91–0.94×`**, peak RSS **`0.60–0.68×`**.
- Mechanisms' block reader (merging on top): **`0.884×` at `8T`**, **`0.932×` at `10T`** on i9.
- Mechanisms' `parscan` entry point (not adopted): `1.11×` slower at `6T`, RSS `1.28×` at `10T` (whole input held in memory).

**Artifacts:** [`diag_c51.rs`](2026-11-22-wormhole-cycle51/diag_c51.rs), [`diag_c51_quiet.txt`](2026-11-22-wormhole-cycle51/diag_c51_quiet.txt), [`occupancy_3296195e.txt`](2026-11-22-wormhole-cycle51/occupancy_3296195e.txt), [`c51_rediagnosis.toml`](2026-11-22-wormhole-cycle51/c51_rediagnosis.toml).

---

## 1. Measured `3296195e3` (`#739`) vs `v0.5.12` (C31) on `431`-Block `210 MB` Authenticated Wikidata

### 1a. `examples/occupancy` (`--features trace`) Before (`v0.5.12`) vs After (`3296195e3`)

| `T` (`slots`) | `v0.5.12` `ready` max (mean) | **`#739` `ready` max (mean)** | `v0.5.12` `out` cap/used (max) | **`#739` `out` cap/used (max)** | `v0.5.12` worker `Open` buf | **`#739` worker `Open` buf** | **`#739` `dec%` / `i:rd%` / `scan bp%`** |
|---:|---|---|---|---|---|---|---|
| `1` (`4`) | `4` blks, `1.8 MB` | `3` blks, `0.9 MB` (`0.03`) | `3.51×` (`1.80 MB`) | **`1.06×` (`0.95 MB`)** | Cells `14.4 MB` | Cells `17.6 MB` | `80%` / `1%` / `85%` |
| `4` (`16`) | `16` blks, `6.3 MB` (`1.26`) | `14` blks, `6.3 MB` (`1.20`) | `3.38×` (`1.80 MB`) | **`1.06×` (`0.95 MB`)** | Cells `14.5 MB` | Cells `16.5 MB` | `91%` / `4%` / `48%` |
| `7` (`28`) | `27` blks, `10.8 MB` (`2.83`) | `25` blks, `11.7 MB` (`3.28`) | `3.30×` (`1.80 MB`) | **`1.06×` (`0.95 MB`)** | Cells `14.6 MB` | Cells `16.3 MB` | `72%` / `23%` / `12%` |
| `8` (`32`) | `29` blks, `12.6 MB` (`2.83`) | `26` blks, **`10.8 MB`** (`2.64`) | `3.36×` (`1.80 MB`) | **`1.06×` (`0.95 MB`)** | `1.43 MB` (`0.90 ll`) | **`0.57 MB` (`0 B ll`)** | `54%` / **`42%`** / `2.8%` |
| **`10` (`40`)** | `30` blks, `13.5 MB` (`3.90`) | **`19` blks, `9.0 MB` (`2.41`)** | `3.20×` (`1.80 MB`) | **`1.06×` (`0.95 MB`)** | `1.45 MB` (`0.90 ll`) | **`0.58 MB` (`0 B ll`)** | `36%` / **`60%`** / **`0.0%`** |

### 1b. Quiet Fresh-Process Cold-Start Scaling (`diag_c51`, `OpenCells`, `read_block()`, `3`-rd Median)

| `W` (`slots`) | `v0.5.12` `wall` | **`#739` `wall` (`Δ`)** | `v0.5.12` `sys_ms` | **`#739` `sys_ms` (`Δ`)** | `v0.5.12` `minflt` | **`#739` `minflt` (`Δ`)** | `v0.5.12` `rss_MB` | **`#739` `rss_MB` (`Δ`)** |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| `1` (`4`) | `514.90` | `518.97` | `31.34` | `20.95` (`-33.2%`) | `8,474` | `6,240` (`-26.4%`) | `40.98` | `41.27` |
| `4` (`16`) | `382.58` | **`350.77` (`-8.3%`)** | `33.62` | **`16.85` (`-49.9%`)** | `15,521` | **`10,233` (`-34.1%`)** | `58.96` | `55.28` (`-6.2%`) |
| `7` (`28`) | `381.01` | **`361.66` (`-5.1%`)** | `53.48` | **`31.57` (`-41.0%`)** | `22,359` | **`14,389` (`-35.6%`)** | `76.92` | `67.54` (`-12.2%`) |
| `8` (`32`) | `413.28` | **`369.68` (`-10.6%`)** | `57.95` | **`31.50` (`-45.6%`)** | `23,133` | **`14,261` (`-38.4%`)** | `80.11` | `66.82` (`-16.6%`) |
| **`10` (`40`)** | `422.83` | **`385.94` (`-8.7%`)** | `64.44` | **`37.48` (`-41.8%`)** | `26,729` | **`15,844` (`-40.7%`)** | `92.19` | **`73.26` (`-20.5%`)** |
| `10` (`20`) | `408.90` | `406.50` | `47.02` | `32.76` (`-30.3%`) | `22,225` | **`13,066` (`-41.2%`)** | `80.23` | **`65.29` (`-18.6%`)** |
| `16` (`64`) | `415.51` | `404.80` (`-2.6%`) | `81.73` | **`47.98` (`-41.3%`)** | `37,347` | **`19,739` (`-47.1%`)** | `138.92` | **`85.36` (`-38.6%`)** |
| `20` (`80`) | `450.98` | **`401.75` (`-10.9%`)** | `86.39` | **`47.35` (`-45.2%`)** | `37,899` | **`21,884` (`-42.3%`)** | `135.70` | **`93.99` (`-30.7%`)** |

*(Legacy copying `read_4m` on `#739` at `10W/40`: `386.87 ms`, `sys 39.99 ms`, `minflt 16,319`, `rss 76.63 MiB`.)*

---

## 2. Post-`#739` `10W` DRAM `B/B` & Six-Cost Separation (`3296195e3` vs `v0.5.12`)

### 2a. Exact `10W` L3 Working-Set & Off-Chip DRAM `B/B` Model (Comet Lake `20 MiB` L3, `42 GB/s` DDR4)

| Stage / Buffer (`431` blks, `209.8 MB`) | `v0.5.12` L2-Evicting (`B/B`) | **`#739` (`3296195e3`) L2-Evicting (`B/B`)** | `v0.5.12` `10W` L3 (`MiB`) | **`#739` `10W` L3 (`MiB`)** | Notes |
|---|---:|---:|---:|---:|---|
| Feeder + `ChannelSource` (`4/8 MiB`) | `0.26` | **`0.15`** | `6.0` | `4.2` | `spare()` swap (`free_in` cap=`1`) |
| `syms` (`Scanner` W + worker R) | `0.56` | `0.56` | `1.9` | **`0.8`** | Returned right after `runs::front` |
| `runs::front` (`runs` W + `Open` R) | `1.04` | `1.04` | `4.7` | `5.1` | `0.51 MiB/W` (`occupancy` measured) |
| `Open` scratch (`edges`+`dict`+`bands`+`strips`) | L3 spill | L3 spill | `11.3` | `11.3` | `1.13 MiB/W` (`9.5 MiB` w/ R4) |
| `Open` emit → `ll` (`0.90 MiB/W`) | `1.00` | **`0.00`** | `9.0` | **`0.0`** | **Eliminated (`decode_owned`)!** |
| **`rle::unrle_in_place` (`src/rle.rs:138`)** | `2.00` | **`1.53`** (double-shift!) | `9.0` | **`0.0`** (in `out`) | **NEW FINDING:** `195/234` blks shift `160.15 MB` right then left (`320.3 MB` `copy_within`!) |
| `crc::crc32` (`64 KiB` L2-tiled `update_with`) | `1.00` | **`0.00`** | — | — | **Eliminated (L2-hot tiled CRC)!** |
| Reader `read_block` / `BufRead` | `2.00` | **`0.00`** (`2.00` in `Read`) | `13.5` (`ready`) | **`5.5`** (`9.0` max, `2.4` mean) | `12 MiB` ready cap active |
| Caller (`AbHash` / `write_all` R) | `1.00` | `1.00` | — | — | Unavoidable floor |
| Cold `out` doubling + `open.rs:665` `memset(0)` | `0.70` | **`0.08`** | — | — | `1.06×` cap/used (`0` doublings) |
| **Total L2-Evicting / `10W` L3 Demand** | **`9.56 B/B`** | **`4.36 B/B`** (**`-54.4%`**) | **`49.4 MiB`** (`f_miss=60%`) | **`26.9 MiB`** (**`f_miss=25.6%`**) | **L3 miss rate cut `2.3×`!** |
| **Off-Chip `10W` DRAM (`B/B`) / DDR4 Floor** | **`10.82 B/B` / `54.4 ms`** | **`~4.6 B/B` / `~23.1 ms`** | — | — | **DRAM NO LONGER CAPS `10W`!** |

### 2b. Six-Cost Separation at `10W` (`v0.5.12` vs `3296195e3 #739`)

| Cost | `v0.5.12` (`6a36b0e10`) | **`3296195e3` (`#739`)** | Residual Opportunity |
|---|---|---|---|
| **W (Work)** | `1.00×` | **`0.91–0.94×` CPU (i9)** | `src/rle.rs:138` `160 MB` double-shift (`C52`), `decode_parallel` `4 MiB` input copy (`C54`), Mechanisms block reader (`0.884–0.932×`) |
| **D (Crit Path)** | `68–73 ms` (`10T` i9) | **`~66–70 ms` (`0.963×` i9, STILL `7W/28`!)** | **W0 (`7W→8..10W` grid, C56)** + **W1 (Mech block reader → PR 4 streaming `BlockSpan`, C57–C62)** |
| **B (DRAM)** | `10.82 B/B` (`54.4 ms` BW floor) | **`~4.6 B/B` (`~23.1 ms` BW floor)** | **No longer binding!** Dropping `rle.rs:138` shift cuts another `~0.6–0.9 B/B` |
| **Q (Rand Miss)** | Kernel-internal | Unchanged (`0` extra) | Orthogonal (Round 4 `WormholeNeonCells` / Teamwork `BSAC`) |
| **M (Memory)** | `49.4 MiB` L3, `92–136 MiB` RSS | **`26.9 MiB` L3, `0.60–0.68×` i9 RSS** | `free_in` depth `1→3` + `bzcat.rs:32` `resize(8M,0)` + `10W` `20` vs `40` slots (`-17%` `minflt`, `-11%` RSS) |
| **S (Sync/Sys)** | `64.4 ms` `sys`, `862` `notify_all` | **`37.5 ms` `sys` (`-42%`), `0` hot `notify_all`** | **8 mutex locks/block** (`6` `shared.lock()` + `2` `jobs.lock()`), `check_cancelled`/`ChannelSource` locking instead of `stopped` atomic (`C53`) |

---

## 3. Binding Wall Diagnosis on `3296195e3`: Which of `W0`–`W5` Still Bind?

| Wall | Status on `3296195e3` (`#739`) | Binding Magnitude & Direct Proof |
|---|---|---|
| **W0 (`trimmed_here` = `7W/28`)** | **STRONGLY BINDING (HIGHEST LEVER)** | **`src/parallel.rs:3350-3390` was untouched by `#739`!** Proof from real i9 data: Mechanisms' faster block reader gave **`0.884×` at `8T`** (unpinned = **`8` workers**!) but only **`0.932×` at `10T`** (unpinned = **hard-capped at `7` workers**!). At `10T`, `#739` + Mech block reader leaves **2–3 physical Comet Lake cores idle** while `7` workers saturate! |
| **W1 (Serial `Scanner`)** | **STRONGLY BINDING (CO-HIGHEST)** | `occupancy_3296195e.txt` at `10T`: **`scanner backpressured = 0.0%`**, **`workers idle (no job) = 40.1%`**, **`i:rd = 60%`**! Mechanisms' block reader drops serial scan from `~56 → ~43 ms`, which unlocks `8–9W` immediately, but **`10W` (`~38–40 ms` compute) STILL hits the `~43 ms` serial scan wall** until **PR 4 (`BlockSpan` bounded-window streaming)** lands! |
| **W2 (Allocator / Page Faults)** | **RESIDUAL (`~2–3 ms` cold)** | `out` doubling is **CLOSED** (`1.06×`). Remaining `15.8k` `minflt` / `37.5 ms` `sys_ms` at `10W` come from cold `Open` + `free_out` + `free_syms` + `free_in` (`cap=1`) + `examples/bzcat.rs:32` `buf.resize(8<<20, 0)`. |
| **W3 (Post-BWT / DRAM)** | **NON-BINDING ON DDR4 (`~4.6 B/B`), RESIDUAL CPU (`C52`)** | `0 B` `ll` and `64 KiB` L2-tiled CRC broke the `54.4 ms` DDR4 wall down to `23.1 ms`. **One clean bug/miss found:** `src/rle.rs:138` always runs `buf.copy_within(first..n, first+shift)` (`160.15 MB` right + `160.15 MB` left = **`320.3 MB`** across `195/234` blocks!) instead of backward-pass when `min_d >= 0` or suffix-bounded shift! |
| **W4 (Dirty `ready` L3)** | **CLOSED (`<= 9.0–12.0 MiB`)** | `READY_BYTES = 12 MiB` + `W+2` floor + immediate `syms` return work cleanly (`10W` max `9.0 MB`, mean `2.41 MB`). |
| **W5 (Hand-off & Locks)** | **MOSTLY CLOSED (`0` hot `notify_all`); AUDIT IN C53** | `Ready` ring + `Jobs` `VecDeque` + split `cv`/`scan_cv` `notify_one()` are in `#739`. Residual: **`8` mutex lock/unlocks per block** (`shared.lock()` `6×` per block, incl. `check_cancelled` + `ChannelSource` not using `shared.stopped` atomic) + `L1_AtomicFast` under PR 4 (`C53`). |

---

## 4. Cycle 51 Decision & Next Cycle Plan

- **Re-diagnosis verdict:** `#739` **permanently broke Wall W3's DDR4 roofline (`10.8 → 4.6 B/B`) and Wall W4 (`13.5 → 9.0 MB`)**. On `3296195e3` + Mechanisms' block reader, **W0 (`trimmed_here` `7W/28` cap)** and **W1 (serial scan above `~8W`)** now account for **`>80%`** of the remaining `10T` gap, with two concrete residual items inside `#739`: `src/rle.rs:138` `320 MB` double-shift (`C52`) and `8` locks/block + `L1_AtomicFast` (`C53`).
- **Cycle 52 (next):** Native safe-Rust A/B on `3296195e3` auditing `src/rle.rs:138` (`unrle_in_place` single-pass backward & minimal-shift vs `#739` unconditional full shift) + `free_in` depth `1→3` & `bzcat` `resize(8M,0)`!
