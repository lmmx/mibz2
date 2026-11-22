# Wormhole Cycle 55 (Neon) — ROUND 6 INTERIM HANDOFF #1 (`Cycles 51–55`): Post-`#739` Re-Diagnosis & PR 2 Closure

**Date:** 2026-11-22  
**Branch:** `research/carbon-wormhole` (docs-only diff vs `origin/master` `3296195e3`)  
**Baseline:** `origin/master` **`3296195e3`** (`#739`, i9 `0.963×` wall at `10T`, `0.91–0.94×` CPU, `0.60–0.68×` peak RSS).  
**Manifest:** [`c55_handoff1.toml`](2026-11-22-wormhole-cycle55/c55_handoff1.toml)

---

## 1. Charge 1 Summary — Which of `W0`–`W5` Still Bind on `3296195e3`?

| Wall | Status on `3296195e3` (`#739`) | Measured Evidence (`C51`–`C54` + Real i9 `#740`/`#742`) | Action |
|---|---|---|---|
| **W0 (`trimmed_here` `7W/28`)** | **#1 BINDING WALL** | Untouched by `#739` ([`src/parallel.rs:3350-3390`](../../src/parallel.rs)). Directly explains why Mechanisms' block reader gave **`0.884×` at `8T`** (`8` workers unpinned) vs only **`0.932×` at `10T`** (hard-capped at `7` workers!). | **Charge 2 (`C56`):** deliver full `workers × slots` i9 experimental grid (`10, 12, 16, 20T`). |
| **W1 (Serial `Scanner`)** | **#1 CO-BINDING WALL** | `occupancy` at `10T`: **`0.0%` scanner backpressure**, **`60%` worker idle waiting on scanner (`i:rd`)**. Mech block reader lowers serial scan from `~56→~43 ms`, still capping `10W` until bounded-window `BlockSpan` streaming lands. | **Charge 4 (`C57`–`C62`):** written `BlockSpan` streaming spec + prototype fixing whole-input `1.28×` RSS & `6T` `1.11×` regressions. |
| **W2 (Allocator / Page Faults)** | **MOSTLY CLOSED (`~2–3 ms` cold)** | `out` cap/used **`3.20× → 1.06×`**, `10W` `minflt -40.7%`, `sys -41.8%`. Only `1 MiB` `bzcat` chunking (`-9.9%` RSS) & `D2_AdaptCap` (`decode_parallel` `grow 2→1`) remain. | Optional hygiene; no big wall lever. |
| **W3 (Post-BWT & `10W` DRAM)** | **DDR4 FLOOR BROKEN (`10.82 → ~4.6 B/B`)** | `0 B` `ll` (`0.58 MB` worker buf) + `64 KiB` L2-tiled CRC drop `10W` L3 demand **`49.4 → 26.9 MiB`** (`f_miss 60%→25.6%`) and DDR4 BW floor **`54.4 → 23.1 ms`**. **DRAM no longer caps `10W`!** Uncovered `src/rle.rs:138` `350.8 MB` double-shift → **Theorem 52.1 (`ZeroShift52`)** cuts `copy_within` `-46.6%` (`187.4 MB`) & RLE+CRC **`0.8944×`**. | Port **`ZeroShift52`** (`~50` safe-Rust lines in `src/rle.rs`, removes `s.ll` fallback). |
| **W4 (Dirty `ready` L3)** | **CLOSED (`<= 9.0–12.0 MiB`)** | `10W` `ready` max dropped `13.5 → 9.0 MB` (mean `2.41 MB`), `0` HOL stall. | Done in `#739`. |
| **W5 (Hand-off & Locks / PR 2)** | **CLOSED (`0` hot `notify_all`)** | `nA = 1` (EOF `Jobs::close()` only). **`L1_AtomicFast`** (`0.990–1.006×`) & **`Str_TrimLock`** (`0.988–1.009×`) both **KILLED** across `4W L9`, `4W L1` (`3×` blk rate), `10W`, `16W`. | **Port nothing more from PR 2.** |

---

## 2. Phase-1 Keep / Kill Ledger (`Cycles 51–55`)

| Cycle | Commit | Candidate | Verdict | One-Line Evidence |
|---:|---|---|---|---|
| C51 | `162b7054d` | Post-`#739` Re-Diagnosis (`3296195e3`) | **PROVED** | `10W` DRAM `10.82 → ~4.6 B/B` (`23.1 ms` floor); `W0` (`7W/28`) & `W1` (`0.0%` scan bp) now bind `10T`. |
| C52 | `7796d0ef4` | **`ZeroShift52` (Theorem 52.1 In-Place RLE1)** | **PROMOTE** | `5,000/5,000` + `431/431` exact; cuts `copy_within` `-46.6%` (`350.8→187.4 MB`), RLE+CRC **`0.8944× [0.8850, 0.9039]`**, `0` `s.ll` fallback. |
| C52 | `7796d0ef4` | `bzcat_8M_noz` / `bzcat_1M_pool` | **KILL / HYGIENE** | `8M_noz` killed (`0.9985×`); `1M_pool` kept strictly as RSS hygiene (`-9.9%` RSS, `-11.2%` `minflt`, neutral wall `0.9962×`). |
| C53 | `423e7e2ab` | **`L1_AtomicFast` & `Str_TrimLock` (PR 2)** | **KILL BOTH** | `0` hot `notify_all` remain; `L1_AtomicFast` (`0.990–1.006×` stream, `0.996–1.006×` `ParScan`) & `Str_TrimLock` (`0.988–1.009×`) save `~8.6 µs`. **PR 2 closed.** |
| C54 | `6eeb79499` | `D1_SliceSrc` alone | **KILL alone** | Neutral wall (`0.9948× [0.9758, 1.0142]`) on `wikidata-l9`. |
| C54 | `6eeb79499` | **`D2_AdaptCap` & `D3_Chunks` (`decode_parallel`)** | **PROMOTE** | `D2` **`0.9786×`** (`sys -12.9%`, `grow 2→1`, `0` random bloat `-9.3%` RSS); `D3_Chunks` **`0.9540×`** (`-34 ms` user, `-8.4 MiB` RSS). |

---

## 3. Phase 2 Plan (`Cycles 56–60` → Interim Handoff #2 at Cycle 60)

- **C56 (Charge 2):** Calibrated i9-10900K **`workers × slots` recalibration grid** at `10, 12, 16, 20` threads (giving the experimental grid + predicted `10T` wall, `L3_MB`, `DRAM B/B`, and `RSS` at every cell, NOT a hardcoded table — the i9 decides!), verified on Xeon across the full grid!
- **C57 (Charge 4, Part 1):** Forensic audit of Mechanisms' `parscan` `1.11×` (`6T`) / `1.28×` RSS (`10T`) failure + `C38b` single-window barrier analysis.
- **C58 (Charge 4, Part 2):** **Written PR 4 `BlockSpan` Streaming Interface Spec** (agreed with `research/carbon-word-transport`'s `C38b` bounded-window finder, using ping-pong `2 × 4 MiB` `Arc<Vec<u8>>` windows — ZERO whole-input buffering, ZERO inter-window epoch barrier, plugging directly into `#739`'s `StreamDecoder`!).
- **C59 (Charge 4, Part 3):** Native safe-Rust `4W/6W/10W/16W` A/B confrontation of PR 4 Bounded-Window `BlockSpan` Streaming vs `3296195e3` & Whole-Input `ParScan` across all 4 regimes (`wikidata-l9`, `wikidata-l1`, `hetero-l9`, `random-l9`), proving `RSS <= 1.00×` and `6T < 1.00×`!
- **C60 — INTERIM HANDOFF #2:** Ready-to-queue i9 `workers × slots` grid + finalized Mechanisms/Coordinator PR 4 `BlockSpan` streaming interface spec!
