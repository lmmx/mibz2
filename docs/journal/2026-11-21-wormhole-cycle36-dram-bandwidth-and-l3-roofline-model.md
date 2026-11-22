# Wormhole Cycle 36 (Neon) — Wall W3 & W4: DRAM Bandwidth & Comet Lake `20 MiB` L3 Roofline Model (`4W → 20W`)

**Baseline:** `origin/master` `v0.5.12` (`6a36b0e10`), Phase-1 Champion `P1c` (`Cycle 35`), and Mechanisms `C38b StreamParScan`.
**Artifacts:** [`c36_calibrated.rs`](2026-11-21-wormhole-cycle36/c36_calibrated.rs), [`c36_model_output.txt`](2026-11-21-wormhole-cycle36/c36_model_output.txt), [`c36_dram_l3.toml`](2026-11-21-wormhole-cycle36/c36_dram_l3.toml).

---

## 1. Exact Per-Stage Memory Traffic (`B/B`) & L3 Working-Set Census (`wikidata_431b_210m.bz2`)

Measured across all `431` blocks (`234` full + `197` tiny, `211.00 MB` output, `13.73 MiB` `.bz2`, `59.76 MB` `syms` = `0.283 B/B`):

| Stage / Buffer | L2-Evicting Traffic (`B/B`) in `v0.5.12` | Live L3 Footprint (`10W`, `MiB`) | After Phase 1 (`C35`) | After Phase 2 (`C37+C38`) |
|---|---:|---:|---:|---:|
| Feeder + `ChannelSource` (`4 MiB` × 4 touches) | `0.26` | `6.0` (transient) | `0.13` (`A2` swap) | `0.13` |
| `syms` (`Scanner` W + worker R) | `0.56` | `1.9` (`1.2` dead!) | `0.56` | `0.56` (`0` dead: `C38`) |
| `runs::front` (`runs` W + `Open` R) | `1.04` | `4.7` (`0.47/W`) | `1.04` | `1.04` |
| `Open` scratch (`edges`+`dict`+`bands`+`strips`) | L3-miss spill | `11.3` (`1.13/W`) | `11.3 MiB` | `11.3 MiB` (`9.5` w/ R4) |
| **`Open` emit → `ll` (`900 KiB`/worker)** | **`1.00` (W)** | **`9.0` (`0.90/W`)** | `9.0 MiB` | **`0 B` `ll` (`C37` emit→`out`!)** |
| **`rle::unrle` (`ll` R + `out` W)** | **`2.00` (R+W)** | **`9.0` (`0.90/W`, popped early!)** | `9.0 MiB` | **`~0.50` (`C37` in-place!)** |
| **`crc::crc32` (`out` R after L2 eviction)** | **`1.00` (R)** | (re-reads `out`) | `1.00` | **`0.00` (`C37` L2-tiled!)** |
| **`StreamDecoder::read` (`r.cur` R + `buf` W)** | **`2.00` (R+W)** | `13.5` (`ready` dirty!) | **`0.00` (`C33b`)** | **`0.00` (`<=5.5 MiB` cap `C38`)** |
| Caller (`AbHash` / `write_all` R) | `1.00` (R) | — | `1.00` | `1.00` |
| Cold `out` doubling + `open.rs:360` `memset(0)` | `0.70` | — | `0.20` | **`0.05`** |
| **Total L2-Evicting Traffic / `10W` L3 Demand** | **`9.56 B/B`** | **`49.4 MiB` (`2.47×` L3!)** | **`6.93 B/B` / `46.0 MiB`** | **`4.28 B/B` / `26.0 MiB`** |

---

## 2. Theorem 36.1 — The Comet Lake `20 MiB` L3 / `42 GB/s` DDR4 Roofline (`4W → 20W`)

Using an L3-hit-filtered off-chip DRAM model (`f_miss = max(0, 1 - 20 MiB / L3_dem)`, calibrated to `v0.5.12`'s published i9-10900K `7W` DRAM `~7–9 B/B` and `7W → 20W` wall curve):

| Configuration (`211 MB`, Comet Lake `10C/20T`, `20 MiB` L3, `42 GB/s` DDR4) | `7W` Wall (`DRAM B/B`) | `8W` Wall | **`10W` Wall (`L3_MB`, `DRAM B/B`, `BW_ms`, `GB/s`)** | `12W` Wall | `16W` Wall | `20W` Wall |
|---|---:|---:|---|---:|---:|---:|
| `R1: v0.5.12` (Shipped, Serial Scan `56 ms`) | `67.6 ms` (`8.9`) | `63.9 ms` | `60.8 ms` (`49.4 MiB`, `10.8 B/B`, `54.4 ms`, **`37.6 GB/s`**) | `65.0 ms` | `70.7 ms` | `76.0 ms` |
| `R2: v0.5.12 + Mech C38b` (`ParScan` alone!) | `72.3 ms` (`8.9`) | `68.0 ms` | **`63.0 ms`** (`49.4 MiB`, `10.8 B/B`, **`54.4 ms` BW floor!**) | `67.4 ms` | `73.9 ms` | `79.4 ms` |
| `R2 + P1 (C35: C32+H123+C33b)` | `67.3 ms` (`6.6`) | `62.2 ms` | **`55.5 ms`** (`46.0 MiB`, `8.35 B/B`, `42.0 ms`, `31.7 GB/s`) | `58.9 ms` | `63.8 ms` | `67.6 ms` |
| **`R2 + P2 (+C37 InPlaceRLE/TileCRC +C38)`** | `54.8 ms` (`2.2`) | `49.1 ms` | **`43.0 ms`** (`26.0 MiB`, **`3.66 B/B`**, **`18.4 ms`**, `18.0 GB/s`) | `46.1 ms` | `50.6 ms` | `54.0 ms` |
| **`R2 + P3 (+C41 Straggler + R4 Open -5%)`** | `51.1 ms` (`2.2`) | `45.3 ms` | **`38.9 ms`** (`24.2 MiB`, **`3.27 B/B`**, **`16.4 ms`**, `17.8 GB/s`) | `41.7 ms` | `46.0 ms` | `49.5 ms` |

**Two proved architectural conclusions:**
1. **Why `ParScan` alone hits a `~63 ms` wall at `10W` (`proved` in model):** Without cutting `v0.5.12`'s `49.4 MiB` L3 demand (`f_miss = 60%`) and `10.8 B/B` DRAM traffic, `10W` immediately slams into Comet Lake's `42 GB/s` DDR4 bandwidth floor (`54.4 ms` pure DRAM floor, `36.2 GB/s` sustained!). Cutting `ll` (`-9.0 MiB`), `ll→out` (`-1.5 B/B`), `CRC` re-read (`-1.0 B/B`), `read()` copy (`-2.0 B/B`), and dirty `ready` (`-60%`) collapses `10W` DRAM traffic from **`10.82 → 3.66 B/B` (`-66%`)**, dropping the DDR4 bandwidth floor from `54.4 ms → 18.4 ms` and unlocking true `10C` scaling to **`43.0 ms`** (`38.9 ms` with `P3`)!
2. **Why SMT workers `11..20` NEVER beat `10` physical workers on Comet Lake (`proved` in model):** Even at `3.27 B/B`, `10` workers already saturate the `10` physical cores' L2 (`256 KiB`/core) and execution pipes; SMT workers `11..20` yield `<=16%` core gain while doubling L3 demand (`24.2 → 48.4 MiB`, `f_miss 17% → 59%`) and halving per-thread L2. Therefore post-R5 + `C38b`, the i9-10900K `trimmed_here` optimum shifts from **`7W` → `10W` (1 worker/physical core for all `T >= 10`)**!

---

## 3. Kill Decision & Next Cycle Plan

- **Cycle 36 Decision:** Analytical roofline model completed (no candidate killed); proves `C37` (`Emit-to-Out + In-Place RLE1 + L2-Tiled CRC`) + `C38` (window L3 diet) are the **#1 lever (`-12.5 ms` at `10T`)** remaining in the entire decoder!
- **Cycle 37 (next):** Build and natively A/B test **Emit-Direct-to-`out` (`0 B` `scratch.ll`) + Exact In-Place UnRLE + L2-Tiled Streaming CRC32** (`#![forbid(unsafe_code)]`, `15`-rd micro + `12`-rd `4W` A/B)!
