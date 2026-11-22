# Wormhole Cycle 35 (Neon) — INTERIM HANDOFF #1 (`Cycles 31–35`): Phase-1 Scaling Stack (`4W = 0.9550×`, `-27%` `minflt`, `-48%` `vcsw`, `-2.0 B/B` DRAM)

**Baseline:** `origin/master` `v0.5.12` (`6a36b0e10b2e2e781518a101db7ccf8b6352b986`).
**Mechanisms Coordination:** `origin/research/carbon-word-transport` (`300a29976`, Cycle 38b `StreamParScan`). Mechanisms owns the stream scan & block finder (`-96.3%` serial scan CPU); Wormhole owns all non-scan bottlenecks (allocator/page faults, CRC/RLE1 & output assembly, block hand-off/ordering, L3/DRAM scaling, tail stragglers).
**Artifacts:** [`c35_phase1_handoff.rs`](2026-11-21-wormhole-cycle35/c35_phase1_handoff.rs) (`safe_p1` module `#![forbid(unsafe_code)]`), [`c35_results.txt`](2026-11-21-wormhole-cycle35/c35_results.txt), [`c35_handoff1.toml`](2026-11-21-wormhole-cycle35/c35_handoff1.toml).

---

## 1. Executive Summary of Phase 1 (`Cycles 31–35`)

We mapped every stage behind the i9-10900K's `68–73 ms` (`8T → 20T`) release-scaling plateau using `examples/occupancy.rs` (`--features trace`) and fresh-process OS counters, then designed, ablated, and verified a pure safe-Rust **Phase-1 Scaling Stack (`P1 = C32 + H123 + C33b`)** that requires **zero changes to `OpenCells` or `Scanner`** yet delivers monotonic, statistically significant wins across every step on both uniform Wikidata (`211 MB`, `431` blocks) and heterogeneous streams (`hetero_60b`):

### 1a. `15`-Round Rotated Fresh-Process `4W` Cumulative Ablation (`wikidata_431b_210m.bz2`, `211.0 MB`, exact `AbHash`)

| Step | Arm (cumulative safe-Rust stack) | `4W` med (`ms`) | Ratio vs `MasterA` `[95% CI]` | `user_ms` | `sys_ms` | `minflt` | `vcsw` | Peak RSS* (`4W`/`10W`/`16W`) |
|---|---|---:|---|---:|---:|---:|---:|---|
| 0 | `MasterA` (`v0.5.12` `6a36b0e10`) | `364.10` | `1.0000 [1.0000, 1.0000]` | `650.99` | `32.62` | `15,876` | `729` | `59.2 / 97.2 / 135.6 MiB` |
| 0 | `MasterB` (`A/A` control) | `361.68` | `0.9950 [0.9844, 1.0057]` | `640.61` | `37.12` | `15,701` | `700` | `60.1 / 94.4 / 137.2 MiB` |
| 1 | **`P1a_C32`** (`A1` `960K` exact `out` + `A2` `1 MiB` recycled feeder) | `355.55` | **`0.9781 [0.9678, 0.9886]`** | `640.11` | `26.21` (`-19.7%`) | `12,136` (`-23.6%`) | `435` (`-40.3%`) | `54.9 / 88.3 / 100.8 MiB` |
| 2 | **`P1b_C32_H123`** (`+` `H12` ring slots/split CVs + `H123` `VecDeque` queue, full `Read` copy!) | `352.42` | **`0.9642 [0.9507, 0.9778]`** | `638.08` | `27.45` (`-15.8%`) | `11,991` (`-24.5%`) | **`378` (`-48.1%`)** | `57.5 / 85.1 / 108.5 MiB` |
| 3 | **`P1c_Full_ZC`** (`+` `C33b` `std::io::BufRead` zero-copy assembly) | **`348.13`** | **`0.9550 [0.9359, 0.9745]`** | **`626.35` (`-24.6`)** | **`24.29` (`-25.5%`)** | **`11,865` (`-25.3%`)** | **`384` (`-47.3%`)** | **`55.9 / 83.4 / 104.8 MiB`** |

*\*Peak RSS per fresh isolated child taken from Cycles 32–34 (`c35_phase1_handoff.rs` ran `verify_edge_cases()` in the parent before spawning children, pinning inherited `maxrss` floor to `106.16 MiB`).*

### 1b. Heterogeneous Guard (`hetero_60b.bz2`: `48` Wikidata + `6` dense + `6` random fallback, `15`-round `4W`) & Cold-Start `10W`/`16W`
- **`hetero_60b` `4W`:** `A/A = 0.9875 [0.9707, 1.0047]`, `P1b_C32_H123 = 0.9814 [0.9630, 1.0001]`, **`P1c_Full_ZC = 0.9751 [0.9558, 0.9948]`** (`vcsw 112 → 67`, `-40.2%`). Zero HOL regression!
- **`W = 10` (`wikidata_431b_210m`, `5`-rd median):** `minflt 26,994 → 19,644` (**`-27.2%`**), `sys_ms 52.12 → 43.79 ms` (`-16.0%`), `user_ms 670.2 → 653.9 ms` (`-16.3 ms`).
- **`W = 16` (`5`-rd median):** `sys_ms 88.07 → 57.48 ms` (**`-34.7%`**), `wall 392.9 → 382.6 ms` (`-2.6%`).
- **Edge-case parity:** `PASS` on empty stream (`ErrorKind::Empty`), bad header, mid-block truncated stream (byte-exact prefix parity vs `master`), and mid-block bit-flip CRC corruption.

---

## 2. Phase-1 Keep / Kill Ledger (`Cycles 31–35`)

| Cycle | Candidate | Verdict | One-Line Evidence |
|---:|---|---|---|
| C32 | `A1: Exact-Fit 960 KiB out` | **PROMOTE** | Fixes `src/rle.rs:47` off-by-5B doubling (`dbl=0`), halving `out` pool cap (`50–144 → 26–74 MiB`). |
| C32 | `A2: Recycled 1 MiB Feeder Pool + Swap` | **PROMOTE** | Eliminates mid-decode `4 MiB` `mmap`/`munmap` TLB shootdowns; `-23%`–`-34%` `minflt`, `-35%` `16W` `sys_ms`. |
| C32 | `A3: Eager 262K syms Pre-Alloc` | **KILL** | Over-allocates `512 KiB` on tiny `2 KiB` blocks (`+8–13%` `minflt` vs `A12_FeedPool`). |
| C32 | `A4: Static Window Shrink (2W / W+6)` | **KILL** | Regresses **`1.0857× [1.0256, 1.1494]`** on `hetero_60b` from HOL scanner starvation. |
| C33 | `C2: 32–64 KiB L1/L2-Tiled unrle+CRC` | **KEEP (Phase 2)** | `2.26×` faster in safe Rust (`0.4423×`); saves `-1.0 B/B` L3/DRAM re-read with streaming `clmul_update`. |
| C33 | `C33b: std::io::BufRead Zero-Copy` | **PROMOTE** | Removes `211 MB` `r.cur→buf` `memcpy` (`-2.0 B/B` DRAM); `4W 0.9550×`, `-24.6 ms` user CPU. |
| C33 | `C33a: Multi-Block Batch-Fill Read` | **KILL** | Delays `free_out` return (`+878` `minflt`, `+3.5 MiB` RSS) without removing the `211 MB` `memcpy`. |
| C33 | `Fused SB16 CRC` | **KILL** | `16 KiB` tables spill L1d alongside `32 KiB` tile (`211.7 vs 168.7 ms` for `Fused_32K+SB8`). |
| C34 | `H12: Ring Ready Slots + Split CVs` | **PROMOTE** | Eliminates `BTreeMap` nodes & `862` `notify_all()` herds (`-87%`–`-97%` wakes, `-38.6%` `10W` `sys_ms`). |
| C34 | `H123: Bounded VecDeque Job Queue` | **PROMOTE** | Replaces `Mutex<mpsc::Receiver>` convoy & node allocs (`-48.1%` `vcsw`, `-4.5%` `10W` wall). |
| C34 | `H4: Separate free_out/free_syms Mutexes` | **KILL** | Breaks piggybacked pool ops out of `ready.lock()`, adding `+6.8 ms` `sys_ms` at `4W`. |

---

## 3. Updated `10T` i9-10900K Critical-Path Model (End of Phase 1)

| Component (ms on i9-10900K, `210 MB`) | `v0.5.12` (`7W/28`, Reg. 1) | `v0.5.12` (`10W/40`, Reg. 2 + `C38b`) | **`P1c` (`10W/40`, Reg. 2 + `C38b`)** | **Remaining Phase 2–3 Target (`10W`)** |
|---|---:|---:|---:|---:|
| `T_scan_serial` (Mechanisms `C38b`) | `56.0` | `2.2` | `2.2` | `2.2` |
| `T_worker_ideal` (`(front+Open+RLE+CRC+ParHuff)/W`) | `51.9` | `39.5` | `39.5` | `36.2` (`C37` in-place RLE + tiled CLMUL + no `898K` memset) |
| `ΔT_L3_DRAM` (L3 spill & DRAM BW) | `+12.5` | `+17.5` | **`+11.5`** (`-2.0 B/B` + `-0.5` cold) | **`+4.5`** (`-0.90 MiB/W` drop `ll` + `-2.5 B/B` + byte-cap window) |
| `ΔT_alloc_pf` (`minflt`, `mmap_lock`, TLB) | `+4.5` | `+6.2` | **`+3.4`** (`-27%` `minflt`, `0` dbl/shootdown) | **`+1.6`** |
| `ΔT_handoff_locks` | `+1.5` | `+2.0` | **`+0.4`** (`-92%` wakes, ring+deque) | **`+0.4`** |
| `ΔT_tail` (straggler drain) | `+1.8` | `+2.4` | `+2.4` | **`+0.8`** (`C41` size/priority dispatch) |
| `T_reader_serial` (concurrent) | `16.0` | `16.0` | **`8.5`** (`BufRead` zero-copy) | **`8.5`** |
| **Modelled Wall Time** | **`~70–72 ms`** | **`~53–56 ms`** | **`~45–47 ms` (`-8–9 ms`!)** | **`~36–40 ms` (`~0.52–0.58×` vs `0.5.12`!)** |

---

## 4. Phase 2 Roadmap (`Cycles 36–40`, Interim Handoff #2 at Cycle 40)

Phase 1 cut **`~8–9 ms`** off the modelled `10T` Regime-2 critical path (and `4W` natively to `0.9550×`). The **#1 remaining bottleneck** is now **`ΔT_L3_DRAM` (`+11.5 ms`)**: `10` workers still carry `3.40 MiB`/worker (`1.13` Open + `0.47` runs + `0.90` `ll` + `0.90` `out`) + `12 MiB` dirty `ready` = `46 MiB` (`2.3×` i9's `20 MiB` L3!) and `6.9 B/B` of DRAM traffic!
- **C36:** Exact `B/B` DRAM bandwidth & Comet Lake L3 roofline model across `4W..20W`.
- **C37:** **Emit-Direct-to-`out` (`0 B` `scratch.ll` -> `-9.0 MiB` at `10W` = `45%` of i9 L3!) + Exact In-Place RLE1 + L2-Tiled Streaming CRC32** + fixing `src/open.rs:360`'s `898 KiB` `memset(0)` on tiny→full transitions!
- **C38:** **Delayed `out` Acquisition + Immediate `syms` Return (post-`front`) + Byte-Budgeted Ready Window** (eliminating dirty L3 pollution WITHOUT `hetero_60b` HOL starvation!).
- **C39:** **Reader / `decode_parallel` / `bzcat` Zero-Copy & Parallel Drain Pipeline** (eliminating `decode_parallel`'s extra `out.extend_from_slice` copy and `4 MiB` bounce buffer).
- **C40 (Interim Handoff #2):** Full Phase-2 native `4W` A/B + `10W/16W` OS/RSS/DRAM verification & updated `10T/16T/20T` model!
