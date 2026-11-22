# Wormhole Cycle 50 (Neon) — FINAL ROUND 5 HANDOFF (`Cycles 31–50`): Breaking the i9-10900K `8→20T` Wall-Time Plateau

**Date:** 2026-11-21  
**Branch:** `research/carbon-wormhole` (strictly docs-only diff vs `origin/master`)  
**Baseline:** `origin/master` `v0.5.12` (`6a36b0e10b2e2e781518a101db7ccf8b6352b986`)  
**Mechanisms Coordination:** `origin/research/carbon-word-transport` (`300a29976`, `C38b StreamParScan`). Mechanisms owns the stream scan & block finder (`-96.3%` serial scan CPU); Wormhole did **zero** scan/finder duplication and instead solved every non-scan serial, memory, allocator, hand-off, and L3/DRAM bottleneck, modelling both **Regime 1** (standalone `v0.5.12` serial scan) and **Regime 2** (coordinated with Mechanisms `C38b`).  
**Manifest:** [`c50_final_handoff.toml`](2026-11-21-wormhole-cycle50/c50_final_handoff.toml)

---

## 1. Executive Summary: Why `v0.5.12` Plateaus at `68–73 ms` (`8→20T`) & How We Break It

Using `examples/occupancy.rs` (`--features trace`), fresh-process cold-start OS counters (`1..20W`), and a calibrated Comet Lake `20 MiB` L3 / `42 GB/s` DDR4 roofline (**Theorem 36.1**), we decomposed `v0.5.12`'s flat `68–73 ms` (`8T → 20T`) plateau on `210 MB` Wikidata into **six compounding walls**:

| Wall | Root Cause in `v0.5.12` (`6a36b0e10`) | Owner & Resolution | Measured / Modelled Impact |
|---|---|---|---|
| **W0** | `trimmed_here` (`src/parallel.rs:2969-3035`) hard-caps **all** `T ∈ {10, 12, 16, 20}` unpinned to the exact same `7W/28` decoder (`workers(7, 28, k, true)`). | **Recalibrate in PR 6 (C48)** after W1–W5 land: `10–20T` → **`10W / 12 MiB cap`** (1/phys core). | Unlocks `+43%` physical workers (`7W → 10W`) while clamping SMT `12–20T` to `10W`. |
| **W1** | Serial `Scanner` (`~56 ms` on i9) caps `10W` at `100%` scanner saturation. | **Mechanisms `C38b StreamParScan`** (`300a29976`) + **Wormhole C42** `ParScan`-coupled interface. | Serial scan `56 ms → 2.2 ms` (`-96.3%`); `0` scanner thread, `0` `JobQueue`, TLS `syms`. |
| **W2** | `src/rle.rs:47` `reserve(n+64)` off-by-5B doubles `100%` of `out` `900K→1.8M` (`3.2–3.5×` cap/used); `4/8 MiB` feeder `mmap`/`munmap` TLB shootdowns; tiny blocks poison `free_out` triggering `src/open.rs:665` `898 KiB` `memset(0)`. | **Wormhole C32 + C38 (`W0`) + C46 (`Slab_THP`) + C48 (`exact_out_cap`)**. | `dbl = 0`, `minflt` **`-68%`–`-79%`**, `sys_ms` **`-67%`–`-78%`**, opt-in Linux `Slab_THP` cuts remaining slab faults **`-95%`–`-98%`**. |
| **W3** | `9.56 B/B` L2-evicting traffic (`7.0 B/B` post-BWT: `Open→ll` `1.0` + `ll→out` `2.0` + `CRC` `1.0` + `read()` `2.0` + caller `1.0`); `W × 0.90 MiB` `scratch.ll` burns **`9.0 MiB`** (`45%` of i9 L3 at `10W`)! | **Wormhole C33b (`BufRead` ZC) + C37a (Emit-to-`out`, `0 B` `ll`, exact In-Place `RLE1`) + C37b (`64 KiB` L2-Tiled `CLMUL`) + C39 (`bzcat`/`decode_parallel`)**. | Eliminates `9.0 MiB` `ll` at `10W`, cuts DRAM **`10.82 → 3.66 B/B` (`-66%`)**, DDR4 BW floor **`54.4 → 18.4 ms`**. |
| **W4** | `4W` unbounded dirty `ready` blocks (`30` blocks = `13.5 MiB` at `10W`, `49.4 MiB` total L3 demand = `2.47×` L3) plus dead `syms` held until block completion. | **Wormhole C38 (`W1` immediate `syms` return + `W13b` `12 MiB` byte-budgeted ready cap w/ `W+2` floor)**. | Caps `10W` peak ready at **`12.04 MiB`**, cuts `10W` RSS **`-49%`–`-57%`** (`118→51 MiB`) & `16W` RSS **`-50%`–`-54%`** with **zero** `hetero_60b` HOL stall. |
| **W5** | `2,155` global locks, `862` `notify_all()` thundering herds, `BTreeMap` nodes, `Mutex<mpsc::Receiver>` convoy, `431` `read()` calls, and `2.36×` tail stragglers. | **Wormhole C34 (`H123` ring/split-CV/deque) + C41 (Regime-2 LPT) + C44 (`L1_AtomicFast` + directed `notify_one()`)**. | Wakes `-87%`–`-97%`, `vcsw` **`-70%`–`-80%`**, lock hold **`~13 µs`** (`0.03%` of `40 ms` — **Wall W5 closed**, Theorem 44.1). |

**Crucial Systems Proof (Theorem 36.1):** Even **with** Mechanisms `C38b StreamParScan`, `v0.5.12` at `10W` demands `49.4 MiB` of L3 (`60%` miss rate) and **`10.82 B/B`** of off-chip DRAM — hitting a **`54.4 ms` pure DDR4 bandwidth floor** (`36.2 GB/s` sustained, `~63.0 ms` wall). Wormhole Round 5 is the exact second half required to break the plateau: cutting `10W` DRAM to **`3.66 B/B`** (`18.4 ms` BW floor) unlocks true `10`-core Comet Lake scaling down to **`~39–43 ms`** (**`~36–38 ms`** with Round 4's `WormholeNeonCells`).

---

## 2. Final Native Safe-Rust Headlines (`#![forbid(unsafe_code)]`, Same-Binary A/B vs `v0.5.12`)

### 2a. End-to-End Streaming & `ParScan`-Coupled (`wikidata` `211 MB`, Fresh-Process Rotated A/B)

| Configuration | Cycle | `4W` Ratio vs `v0.5.12` `[95% CI]` (`A/A`) | `10W` Wall (`Δ`) | `10W` `sys_ms` (`Δ`) | `10W` `minflt` (`Δ`) | `10W` `vcsw` (`Δ`) | `10W` Peak RSS (`Δ`) |
|---|---:|---|---:|---:|---:|---:|---:|
| **`R1_Stream_P1W0`** (Drop-in streaming, AVX2 `unrle`) | C40 / C45 / C49 | **`0.9221–0.9413×`** `[0.901, 0.984]` (`0.988–1.003`) | `-5.7%..-6.6%` | `-38%..-50%` | `-39%..-41%` | `-43%` (`4W`) | `-23%..-31%` |
| **`R1_Stream_P2`** (Streaming + `0 B ll` + `12 MiB` cap) | C45 / C49 | **`0.9497×`** `[0.913, 0.988]` (scalar SWAR*) | `-5.3%..-5.8%` | **`-53%..-65%`** | **`-51%..-55%`** | `-45%` (`4W`) | **`-39%..-41%`** |
| **`R2_Par_P1W0`** (`ParScan`-coupled, AVX2 `unrle`) | C45 / C49 | **`0.8935–0.8989×`** `[0.868, 0.931]` (`0.988–1.003`) | **`-13.1%..-15.0%`** | `-57%..-63%` | `-59%..-63%` | **`-64%..-74%`** | `-38%..-50%` |
| **`R2_Par_P2`** (Full **`WormholeScaleDecoder`**) | C45 / C49 | **`0.9082–0.9145×`** `[0.890, 0.930]` (scalar SWAR*) | **`-11.5%..-14.8%`** | **`-67%..-77%`** (`17.9–21.2 ms`) | **`-68%..-71%`** (`8.5–9.0k`) | **`-70%..-72%`** (`97–135`) | **`-49%..-57%`** (`50.8–51.5 MiB`) |

*\*In production `src/rle.rs`, replacing our `#![forbid(unsafe_code)]` scalar `u64` SWAR run-scan (`8 B/iter`) with `src/rle.rs`'s existing `32 B/iter` AVX2 `_mm256_cmpeq_epi8` scan recovers the exact `14.82 ms` (`~1.5–2.0%`) gap measured in C43, combining `R2_Par_P1W0`'s `0.8935×` CPU speed with `R2_Par_P2`'s `-57%` RSS / `-71%` `minflt` / `-77%` `sys_ms` / `-9.0 MiB` L3 win!*

### 2b. Consumer Output Assembly & Multi-Regime Stress Gates (`12`-Round `4W` A/B)

| Surface / Regime | Cycle | Arm | `4W` Ratio vs `v0.5.12` `[95% CI]` | Key Resource Delta |
|---|---:|---|---|---|
| `examples/bzcat` zero-copy sink (`211 MB`) | C39 | **`R3_ZeroCopySink`** | **`0.9043 [0.8866, 0.9225]`** | `sys -47.1%`, `minflt -55.6%`, `RSS 66.0→40.5 MiB (-38.7%)` |
| `decode_parallel -> Result<Vec<u8>>` (`211 MB`) | C47 | **`Par_SerialExt`** | **`0.9285 [0.9098, 0.9477]`** | `sys -25.3%`, `minflt -16.7%`, `RSS 272.4→234.8 MiB (-13.8%)` |
| `decode_parallel_chunks -> Vec<Vec<u8>>` (`211 MB`) | C47 | **`R2_Chunks`** | **`0.8586 [0.8486, 0.8687]`** | `user -41.6 ms`, `minflt -20.0%`, `RSS 272.4→225.9 MiB (-17.1%)` |
| Level-1 small-block stress (`wikidata-l1`, `100 KB` blks) | C48 | **`R2_Par_P2`** | **`0.8724 [0.8211, 0.9269]`** | `sys -65.1%`, `minflt -56.7%`, `RSS 15.9→13.7 MiB (-13.8%)` (0 bloat!) |
| Heterogeneous straggler stress (`hetero-l9`, `64` blks) | C48 | **`R2_Par_P2`** | **`0.9094 [0.8770, 0.9429]`** | `sys -30.0%`, `minflt -30.9%`, `RSS -20.0%` (0 HOL stall!) |
| `100%` incompressible `LazyCells` (`random-l9`, `12` blks) | C48 | **`R2_Par_P2`** | **`0.8604 [0.8357, 0.8859]`** | `sys -37.9%`, `minflt -40.2%`, `RSS -5.7%` |
| Linux opt-in `2 MiB` THP slab (`16 MiB` arena, `10T`) | C46 | **`Slab_THP`** | **`0.4364–0.4887×`** (cold slab) | `minflt -94.7%..-97.8%` (`8,836→198`), `sys -72.8%`, `8` TLB entries |

---

## 3. Six-Cost Separation (`10W` on `211 MB` Wikidata)

| Cost Dimension | `v0.5.12` (`10W/40`) | **Regime 1** (`S5` Streaming + `P2`) | **Regime 2** (`S7 WormholeScaleDecoder` + `C38b`) |
|---|---|---|---|
| **W (Work / CPU)** | `724.7 ms` user (`10W` Xeon) | `707–725 ms` user (`-2.4%` w/ AVX2) | **`663.9 ms`** user (**`-8.4%`**, `0` scanner bounce + ZC) |
| **D (Critical Path)** | `68–73 ms` i9 (`7W` trim / `56 ms` scan) | **`~61–64 ms`** i9 (`8W`, serial-scan capped) | **`~39–43 ms`** i9 (**`~36–38 ms`** w/ R4 `WormholeNeonCells`) |
| **B (DRAM Traffic)** | `10.82 B/B` (`54.4 ms` DDR4 BW floor) | `~4.5 B/B` (`22.6 ms` BW floor) | **`3.66 B/B`** (`18.4 ms` BW floor; **`3.27 B/B`** w/ R4) |
| **Q (Random Misses)** | Kernel-internal (R4 owns `Open`) | Unchanged (`0` extra random misses) | Unchanged (`0` extra random misses; `8` THP TLB entries opt-in) |
| **M (Memory / L3 / RSS)** | `49.4 MiB` L3, `118.3 MiB` RSS, `29.0k` `minflt` | `~28 MiB` L3, `70.0 MiB` RSS (`-41%`), `14.2k` `minflt` | **`24–26 MiB` L3**, **`50.8 MiB` RSS (`-57%`)**, **`8.5k` `minflt` (`-71%`)** |
| **S (Sync / Kernel)** | `64.8 ms` `sys_ms`, `862` `notify_all`, `473` `vcsw` | `30.7 ms` `sys_ms` (`-53%`), directed `notify_one` | **`17.9–21.2 ms` `sys_ms` (`-67%..-77%`)**, **`97–135` `vcsw` (`-71%`)**, `~13 µs` lock hold |

---

## 4. Modelled `10T / 16T / 20T` i9-10900K Critical Path (Regime 1 vs Regime 2)

| Configuration (`211 MB`, Comet Lake `10C/20T`, `20 MiB` L3, `42 GB/s` DDR4) | `7W` | `8W` | **`10T` Wall (`Policy`)** | **`16T` Wall (`Policy`)** | **`20T` Wall (`Policy`)** |
|---|---:|---:|---|---|---|
| `v0.5.12` Shipped (`6a36b0e10`, `trimmed_here` = `7W/28`) | `68–72 ms` | `68–73 ms` | **`68–73 ms`** (`7W/28` trim) | `68–73 ms` (`7W/28` trim) | `68–73 ms` (`7W/28` trim) |
| `v0.5.12` + Mechanisms `C38b` alone (Theorem 36.1) | `72.3 ms` | `68.0 ms` | **`63.0 ms`** (`10W`: `54.4 ms` DDR4 BW floor!) | `73.9 ms` (unclamped SMT) | `79.4 ms` (unclamped SMT) |
| **Regime 1:** Wormhole `P1+W0+P2` standalone (no `ParScan`) | `~63 ms` | `~61–64 ms` | **`~61–64 ms`** (`8W / 12M`, `56 ms` serial scan floor) | `~61–64 ms` (`8W` clamp) | `~61–64 ms` (`8W` clamp) |
| **Regime 2:** **Wormhole `WormholeScaleDecoder` + Mechanisms `C38b`** | `54.8 ms` | `49.1 ms` | **`~39–43 ms`** (**`10W / 12 MiB cap`**, `3.66 B/B`) | **`~39–43 ms`** (`10W` clamp) | **`~39–43 ms`** (`10W` clamp) |
| **Regime 2 + Round 4 `WormholeNeonCells` (`-12%` Open + `-19%` scratch)** | `~49 ms` | `~44 ms` | **`~36–38 ms`** (**`10W / 12 MiB cap`**, `3.27 B/B`, **`~1.85–1.95×` vs `v0.5.12`!**) | **`~36–38 ms`** (`10W` clamp) | **`~36–38 ms`** (`10W` clamp) |

---

## 5. Complete Round 5 Keep / Kill Ledger (`Cycles 31–50`)

### Promoted & Kept (`14` Items)

| Cycle | Commit | Item | One-Line Verified Evidence |
|---:|---|---|---|
| C31 | `cee7c3290` | **6-Wall Plateau Model (`W0`–`W5`)** | Mapped `v0.5.12` `8→20T` `68–73 ms` plateau via `occupancy.rs` (`--features trace`) + `1..20W` OS counters. |
| C32 | `790ad8fab` | **`A1 + A2` (`exact_out_cap` + `1 MiB` Feeder Pool)** | Fixes `src/rle.rs:47` `900K→1.8M` doubling (`dbl=0`) + `4 MiB` `munmap` shootdowns; `minflt -23%..-34%`, `16W sys -35%`. |
| C33 | `8130c35be` | **`C33b` (`BufRead` Zero-Copy) + `C2` (`32–64K` Tiled `unrle+CRC`)** | `BufRead` removes `-2.0 B/B` reader copy (`4W 0.9625×`, `-27 ms` `10W` user); tiled `unrle+CRC` `2.26×` faster safe Rust + `-1.0 B/B`. |
| C34 | `763ae89d2` | **`H123` (Ring Ready + Split CVs + Bounded `VecDeque`)** | Cuts condvar wakes `-87%..-97%`, `vcsw -37%..-45%`, `10W sys -38.6%`, `4W 0.9836×` even with full copy. |
| C36 | `eca81d1bc` | **Theorem 36.1 (Comet Lake `20 MiB` L3 / `42 GB/s` Roofline)** | Proves `ParScan` alone hits `54.4 ms` DDR4 BW floor (`63 ms` wall) and `10` phys workers strictly beat `12–20W` SMT. |
| C37 | `5925f5b64` | **`C37a` (Emit-to-`out` `0 B` `ll` + In-Place `RLE1`) + `C37b` (`64K` `CLMUL`)** | Eliminates `W×0.90 MiB` (`-9 MiB` @`10W` = `45%` i9 L3) + `open.rs:665` `211 MB` `memset(0)`; `2,000/2,000` exact. |
| C38 | `82dd58682` | **`W0` (Full-Pool Sep) + `W1` (Immed `syms`) + `W13b` (`12 MiB` Cap)** | `W0` hits `4W 0.9417×` (`16W RSS -45%`); `W13b` cuts `10W minflt -57%`, `pkRdy 10.3–12.0 MiB` while winning `hetero_60b` (`0.9627×`). |
| C39 | `5988d0419` | **`R1` (`decode_parallel`) + `R2` (`chunks`) + `R3` (`bzcat` Sink)** | `R1 0.9559×`, `R2 0.8754×` (C47: `0.8586×`), `R3 bzcat 0.9043×` (`sys -47%`, `minflt -56%`, `RSS -39%`). |
| C41 | `c56119251` | **Lemma 41.1 (`syms.len()` Proxy `r=0.98–0.99`) + Regime-2 LPT** | Exact `10W` `ParScan` sim: LPT cuts tail `-2.17%` (`-1.0 ms`) Wikidata & **`-27.6%`** (`-6.8 ms`) `hetero_60b`. |
| C42 | `184361b3d` | **`C42` `ParScan`-Coupled Interface (`TLS syms + Atomic Steal + P2`)** | `4W 0.8932×`, `10W -14.2%` wall, `sys -64%..-77%`, `minflt -68%..-75%`, `RSS -48%..-57%` (`0` scan duplication). |
| C44 | `b1c3c32ab` | **`L1_AtomicFast` + Directed `cv_worker.notify_one()` (Theorem 44.1)** | Cuts `16W vcsw 1,078→217 (-80%)`, `10W vcsw = 82..97`, `~13 µs` total lock hold (**Wall W5 closed**). |
| C46 | `fcb3ee56f` | **`Slab_THP` (Opt-In Linux `MADV_HUGEPAGE` `16 MiB` Slab)** | Cuts slab `minflt -94.7%..-97.8%` (`8,836→198`), `sys -72.8%`, cold slab `0.436–0.489×`, `8` hugepages vs `3,840` 4K TLB. |
| C47 | `3747518ce` | **`Par_SerialExt` (API-Identical `Vec<u8>`) + `R2_Chunks`** | `Par_SerialExt 0.9285×` (`sys -25%`, `RSS 235 MiB`); `R2_Chunks 0.8586×` (`RSS 226 MiB`) — best contiguous vs chunked APIs. |
| C48 | `2d04ce1e7` | **Level-Adaptive `exact_out_cap(max_len)` + `trimmed_here` Table** | Wins `4/4` regimes (`wikidata-l1 0.8724×` w/ `-13.8%` RSS, `random-l9 0.8604×`); moves `10–20T` trim `7W/28 → 10W / 12 MiB`. |

### Honestly Killed (`14` Items)

| Cycle | Candidate | One-Line Kill Verdict |
|---:|---|---|
| C32 | `A3` Eager `262K` `syms` Pre-Alloc | Over-allocates `512 KiB` on tiny `2 KiB` blocks (`+8–13%` `minflt`). |
| C32 | `A4` Static Window Shrink (`2W / W+6`) | Regresses **`1.0857× [1.0256, 1.1494]`** on `hetero_60b` from HOL scanner starvation. |
| C33 | `C33a` Multi-Block Batch-Fill `Read::read` | Delays `free_out` return (`+878` `minflt`, `+3.5 MiB` RSS) without removing the `211 MB` `memcpy`. |
| C33 | `Fused SB16` CRC | `16 KiB` tables spill L1d alongside `32 KiB` tile (`211.7` vs `168.7 ms` for `SB8`). |
| C34 | `H4` Separate `free_out`/`free_syms` Mutexes | Breaks piggybacked pool ops out of `ready.lock()`, adding `+6.8 ms` `sys_ms` at `4W`. |
| C37 | `C37c` Two-Buffer Tiled (keeping `scratch.ll`) | Retains `900 KiB/W` `ll`, regresses `4W` (`0.9775×`) and `10W/16W` `sys_ms` (`58/90 ms`). |
| C38 | `W13a` `8 MiB` Ready Cap | Tighter on `hetero_60b` (`0.9775×`) and higher `10W` `minflt` than `12 MiB` cap. |
| C41 | `S1/S2` Streaming Serial-Scan `JobQueue` LPT | Regresses `1.0145×` on `hetero_60b` due to mid-stream HOL delay (keep LPT strictly for Regime 2 `ParScan`). |
| C42 | `cv_worker.notify_all()` on Single-Block Drain | Creates `1,078` `vcsw` herd at `16W` under `12 MiB` cap (`notify_one()` drops `vcsw` `-80%`). |
| C43 | `M1_PreSyms` (`384 KiB` TLS Pre-Alloc) | Neutral vs A/A (`1.0009×` `10W`) while adding `+219` `minflt`. |
| C43 | `M2_SpanLPT` (16-blk Mid-Stream Reorder) | Neutral vs A/A (`0.9979×` `10W`) while adding `+436` `minflt`. |
| C44 | `L2_Sharded` Per-Slot Mutexes | Double-checked `wake_mu` multiplies `vcsw` `3.3×` (`262 → 870` at `16W`) with neutral wall. |
| C46 | `Slab_Safe` (`4 KiB` Contiguous VMA w/o THP) | Only `0.982×` at `10T` — proves `4 KiB` page faults (not VMA count) dominate cold slab latency. |
| C47 | `F1_CallocScat` ($W$-Way Parallel Scatter) | Regresses **`1.0751× [1.0418, 1.1096]`**, `+134 ms` `sys_ms`, `105.7k` `minflt`, **`420.2 MiB` RSS** (Theorem 47.1). |

---

## 6. Ordered Coordinator Port Spec (`6` Incremental PRs — "The i9 Decides")

Each PR is independently testable, strictly safe Rust (`#![forbid(unsafe_code)]`, except PR 3's existing `src/rle.rs` AVX2 & optional PR 6b Linux `madvise`), and ordered by expected i9-10900K wall-time ROI & zero risk:

1. **PR 1 — Allocator & Zero-Copy Assembly (`C32 + C33b + C38 W0 + C39`, Standalone Win Immediately):**
   - `src/rle.rs:47` & worker `free_out.pop()`: replace `reserve(n+64)` with `exact_out_cap(max_len) = max_len + (max_len / 16).max(4096)` (`dbl = 0`, zero level-1 bloat).
   - `src/parallel.rs:564`: filter `free_out` returns by `spent.capacity() >= (max_len / 4).max(16_384)` (`W0`) so tiny blocks never poison full-block buffers or trigger `src/open.rs:665`'s `out.resize(n + INLINE, 0)` `memset(0)`.
   - `src/parallel.rs:611-657` (`ChannelSource`) + `src/read.rs:307-329` + `examples/bzcat.rs:27-45`: recycle `1 MiB` feeder buffers via zero-copy `std::mem::replace` swap; expose `BufRead` (`fill_buf`/`consume`) or `for_each_block` on `StreamDecoder`/`BzReader`, removing `bzcat.rs:40`'s `16 MiB` bounce (`-9.6%` wall, `-56%` `minflt`, `-39%` RSS) and `src/lib.rs:549-560`'s `4 MiB` bounce (`Par_SerialExt`, plus optional additive `decode_parallel_chunks -> Result<Vec<Vec<u8>>>`).
2. **PR 2 — Lock-Free-Fast Hand-Off & Sync Closure (`C34 H123 + C44 L1`):**
   - `src/parallel.rs:62-78, 541-592`: replace `ready: BTreeMap<u64, _>` with an O(1) power-of-2 ring `Vec<Option<Result<Vec<u8>, Error>>>`; replace single `cv.notify_all()` (`parallel.rs:535, 577, 681, 791`) with split directed `cv_reader.notify_one()` + `cv_bp.notify_one()`; replace `Mutex<mpsc::Receiver<Job>>` with bounded `VecDeque<Job>`; add `L1_AtomicFast` (`taken: AtomicUsize`, `ready_bytes: AtomicUsize`) backpressure fast path.
3. **PR 3 — L3/DRAM Roofline Breaker (`C37a + C37b + C38 W1/W13b`):**
   - `src/open.rs:665` + `src/rle.rs` + `src/crc.rs`: emit `Open::invert_runs` directly into recycled `out` (`0 B` `scratch.ll`, saving **`-9.0 MiB` at `10W`** = `45%` of i9 L3!), run exact in-place `unrle_in_place` using `src/rle.rs`'s existing `32 B/iter` AVX2 `_mm256_cmpeq_epi8` scan + `64 KiB` L2-tiled streaming `clmul_update` CRC32 (Lemma 37.2, `-1.0 B/B` L3 re-read).
   - Return `b.syms` immediately after `runs::front` (`W1`), and enforce the `12 MiB` byte-budgeted ready cap with `W + 2` slot floor (`W13b`, `0` HOL stall on `hetero_60b`).
4. **PR 4 — Mechanisms `C38b StreamParScan` Integration (`C42 + C41`):**
   - **Boundary with Mechanisms (`origin/research/carbon-word-transport`, `300a29976`):** Mechanisms provides the parallel block spans (`BlockSpan` stream from `4 MiB` chunks). Wormhole `C42` consumes those spans directly via `AtomicUsize::fetch_add(1)` work-stealing, `0` scanner thread, `0` `JobQueue`, and `1` thread-local `Vec<u16>` `syms` per worker (optional C41 LPT ordering by compressed span / `syms.len()` on large multi-block chunks).
5. **PR 5 — i9-10900K `trimmed_here` Recalibration (`C48`, `src/parallel.rs:2969-3035`):**
   - After PRs 1–4 land, benchmark `W ∈ {7, 8, 9, 10, 11, 12, 16, 20}` on the i9-10900K (`210 MB` Wikidata + `hetero` + `level-1`). Expected optimum: shift `unpinned_cell` for `T >= 10` from `7W/28` → **`10W / 12 MiB cap`** (1 worker per physical core, clamping SMT `12–20T` to `10W`).
6. **PR 6 (Optional Linux Cold-Start Polish, `C46`):**
   - `#[cfg(target_os = "linux")]` opt-in `16 MiB` `2 MiB`-aligned `madvise(MADV_HUGEPAGE)` output slab (`8` hugepages, graceful `4 KiB` fallback) if cold-start CLI (`bzcat`) latency on the i9 shows measurable `minflt` savings (`-1.5 to -3.2 ms` modelled).

### i9-10900K Hardware Verification Checklist ("The i9 Decides")
- [ ] Verify PR 1–3 standalone (Regime 1) `4T/7T/8T/10T` wall time, `perf stat` `LLC-load-misses`, `DRAM B/B`, `minor-faults`, and `maxrss`.
- [ ] Verify PR 3 `unrle_in_place` with native `src/rle.rs` AVX2 scan recovers the `+14.8 ms` scalar-SWAR delta while retaining `-9.0 MiB` L3 / `-50%+` RSS.
- [ ] Verify PR 4 (`Mechanisms C38b` + `WormholeScaleDecoder`) across `W ∈ {7, 8, 9, 10, 11, 12, 16, 20}` against `v0.5.12`'s `68–73 ms` plateau (target: **`~39–43 ms`** at `10T`, **`~36–38 ms`** once Round 4's `WormholeNeonCells` is also landed).
- [ ] Verify byte-identical SHA-256 across all `431` Wikidata blocks, `bzip2 -1..-9`, `hetero`, incompressible `LazyCells`, empty/truncated/corrupt streams, and `1..20` threads.

---

**Round 5 (`Cycles 31–50`) complete. Stopping cleanly as instructed.**
