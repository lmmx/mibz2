# Direct-Literal 4-Lane Forward Emission & Compact Dictionary Probe (`A6b-dict + A6c + A7`) — STOP

- **Date:** 2026-11-27
- **Probe binary:** `examples/direct_literal_emit_probe.rs` (`Claim: none`, zero `src/` edits, `#![forbid(unsafe_code)]`)
- **Corpus:** `wikidata.bz2` (SHA-256 `82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915`, 431 total blocks, **233 production-admitted `Open` blocks**, **26,567,581 `A7` steps**, **209,697,662 decoded bytes**).
- **Hardware:** 2-vCPU cloud VM (Intel Xeon, 45 MiB L3, AVX2 + BMI2 + PCLMULQDQ). **VM diagnostic only (C5).**
- **Verdict:** **STOP** — unanimous across all 3 builds (baseline x86-64, standard `-C target-cpu=native`, JCC-aligned `-C target-cpu=native`), both cache regimes (warm and L3-cold), and both suites (isolated `A7` and combined `A6b-dict + A6c + A7`). Recorded in `frontier/dead-ends.tsv`.

---

## 1. Hypothesis & Frozen Stop Bar (Recorded Before Timing)

In `#1062` (`composed-worker-stack-probe`), `A7 + A9` remained 46% of the fully composed 4-mechanism Open worker. Because ~82% of `A7` steps emit 1-byte literals (`code < 256`), we tested whether bypassing the 2-level dictionary lookup (`lens[code]` + `inline16[code]` + 16-byte unaligned overwrite) with direct 1-byte stores and shrinking `inline16`/`lens` by 256 entries (`-4.25 KiB/blk`) cuts `A7` or `A6b-dict + A7`.

**Frozen stop bar (set before timing):** worthwhile ONLY IF the winning direct-literal arm cuts `A7` (or `A6b-dict + A7`) by **`>= 5.0%` of `A7`** (`>= ~10–15 ms` 1W / `>= ~1.5–2.2 ms/w` 8W), with **disjoint `[min..max]` spreads on baseline x86-64 AND JCC-aligned `-C target-cpu=native`**, in **both warm AND L3-cold regimes**, and **233/233 byte-exact** vs `bwt::walk`. Otherwise STOP cleanly and record a scoped row in `frontier/dead-ends.tsv`.

### Five Arms Compared
1. `ctrl_4lane_dict`: `#1057` / `#1062` `S = 16` dynamic 4-lane forward emit (`SLAB = 32 KiB`) with shipped `inline16[code]` / `lens[code]` lookup on every step.
2. `lit_branch_per_lane`: per-lane `if code < 256` -> direct 1-byte store `arena[a] = code as u8; a += 1`, else compact `inline16_c[code - 256]`.
3. `lit_all4_fastpath`: single combined bitmask check `((ed0 | ed1 | ed2 | ed3) & non_lit_mask) == 0` testing all 4 anchors AND all 4 literal codes (`< 256`) at once -> 4 direct 1-byte stores, else `lit_branch_per_lane`.
4. `lit_streak_inner`: hoisted slab-budget tight inner `while k < lit_cap` loop for consecutive all-4-literal streaks with `#[cold]` long-word / slab-switch outline.
5. `lit_plus_compact_dict`: strongest direct-literal `A7` arm (`lit_branch_per_lane`) paired with compact `A6b-dict` (skipping the 256-entry literal table allocation `-4.25 KiB` and inlining `if node_word == 0 { a as u32 }` bypass).

---

## 2. Structural Literal-Streak & Dictionary Census (233 Admitted Open Blocks)

| Metric | Value |
|---|---:|
| Total `A7` steps (`sum m`) | 26,567,581 (26.57M) |
| Total decoded bytes (`sum n`) | 209,697,662 (199.98 MiB) |
| 1-byte literals (`code < 256`) | **21,672,913 (81.58%)**, avg sequential streak **8.81** |
| Multi-byte inline words (`2..16B`) | 3,186,535 (11.99%) |
| Long words (`len == LONG`) | 1,708,133 (6.43%) |
| Unified inline fast-path share (`1..16B`) | **93.57%** (only 6.43% `LONG` slow-path) |
| 4-lane Phase-1 iterations | 5,384,861 total |
| All-4-literal iterations (`(c0\|c1\|c2\|c3) < 256`) | **2,401,332 (44.59%)**, avg all-4 streak **2.66** |
| Bit budget | `max_bits = 18`, `max_code = 1,711` (bit-31 tag fits **233/233**) |
| Avg dict table/blk (`inline16 + lens`) | full `17.58 KiB` -> compact `13.33 KiB` (`-4.25 KiB/blk`) |

**Exactness audit (before timing):**
- All 5 arms verified **233/233 byte-exact** vs `bwt::walk` in BOTH isolated `A7` AND combined `A6b-dict + A6c + A7`, plus a **233/233 post-compact reuse check** verifying `ctrl_4lane_dict` after `lit_plus_compact_dict` on the shared `WorkerState` (`2,563 / 2,563` exact checks per build).

---

## 3. Benchmark Results (7 Interleaved Repeats, Pinned CPU 0)

Three compiler configurations tested:
- **Baseline x86-64**: `cargo build --release`
- **Standard Native**: `RUSTFLAGS="-C target-cpu=native"`
- **JCC-aligned Native**: `RUSTFLAGS="-C target-cpu=native -C llvm-args=-x86-align-branch-boundary=32 -C llvm-args=-x86-align-branch=fused+jcc+jmp"`

Steal-free `/proc/thread-self/schedstat` CPU medians are the clean primary signal (concurrent VM CPU steal inflated wall `Instant` spreads up to 1.5–4x; wall is reported alongside).

### 3.1 Isolated `A7` — Steal-Free CPU Medians (ms, 7 Interleaved Repeats)

| Build & Cache Regime | `ctrl_4lane_dict` | `lit_branch_per_lane` | `lit_all4_fastpath` | `lit_streak_inner` | `lit_plus_compact_dict` | Gate (`>=5%`) |
|---|---:|---:|---:|---:|---:|:---:|
| **Baseline WARM** | **244.39** `[240.4..256.9]` | 274.78 (**+12.44%** SLOWER) | 301.81 (**+23.50%** SLOWER) | 339.65 (**+38.98%** SLOWER) | 284.92 (**+16.58%** SLOWER) | **STOP** |
| **Baseline L3-COLD** | **323.96** `[312.3..332.9]` | 348.67 (**+7.63%** SLOWER) | 364.67 (**+12.56%** SLOWER) | 411.55 (**+27.04%** SLOWER) | 338.44 (**+4.47%** SLOWER) | **STOP** |
| **Native WARM** | **237.63** `[233.6..242.7]` | 264.88 (**+11.47%** SLOWER) | 286.80 (**+20.69%** SLOWER) | 322.19 (**+35.58%** SLOWER) | 267.82 (**+12.70%** SLOWER) | **STOP** |
| **Native L3-COLD** | **307.99** `[303.8..316.9]` | 331.00 (**+7.47%** SLOWER) | 352.66 (**+14.50%** SLOWER) | 405.31 (**+31.60%** SLOWER) | 331.58 (**+7.66%** SLOWER) | **STOP** |
| **JCC-Native WARM** | **220.81** `[216.7..230.6]` | 244.43 (**+10.70%** SLOWER) | 265.70 (**+20.33%** SLOWER) | 302.69 (**+37.09%** SLOWER) | 243.90 (**+10.46%** SLOWER) | **STOP** |
| **JCC-Native L3-COLD** | **289.85** `[287.9..297.4]` | 308.71 (**+6.50%** SLOWER) | 329.29 (**+13.61%** SLOWER) | 375.39 (**+29.51%** SLOWER) | 301.78 (**+4.12%** SLOWER) | **STOP** |

**Every single direct-literal arm was DISJOINT SLOWER than `ctrl_4lane_dict` in all 6 isolated `A7` CPU cells** (`24 / 24` arm × build × cache comparisons disjoint slower!).

### 3.2 Combined `A6b-dict + A6c + A7` — Steal-Free CPU Medians (ms, 7 Interleaved Repeats)

| Build & Cache Regime | `ctrl_4lane_dict` (`dict/wire/A7`) | `lit_branch_per_lane` | `lit_all4_fastpath` | `lit_streak_inner` | `lit_plus_compact_dict` (`dict/wire/A7`) |
|---|---:|---:|---:|---:|---:|
| **Baseline WARM** | **306.34** (`62.7 / 41.7 / 210.2`) | 338.22 (+10.41%) | 367.43 (+19.94% SLOWER) | 412.45 (+34.64% SLOWER) | 343.46 (+12.12%, `57.5 / 37.2 / 246.7`) |
| **Baseline L3-COLD** | **346.54** (`54.2 / 46.5 / 240.0`) | 376.98 (+8.78%) | 399.57 (+15.30% SLOWER) | 441.69 (+27.46% SLOWER) | 371.67 (+7.25%, `61.8 / 43.8 / 266.7`) |
| **Native WARM** | **317.59** (`54.0 / 36.7 / 225.9`) | 350.66 (+10.41% SLOWER) | 363.35 (+14.41% SLOWER) | 404.23 (+27.28% SLOWER) | 347.69 (+9.48% SLOWER, `59.9 / 34.0 / 248.5`) |
| **Native L3-COLD** | **349.56** (`55.2 / 35.2 / 253.4`) | 380.25 (+8.78%) | 418.98 (+19.86% SLOWER) | 444.43 (+27.14% SLOWER) | 386.24 (+10.49%, `73.3 / 43.5 / 264.1`) |
| **JCC-Native WARM** | **283.06** (`55.8 / 39.5 / 185.3`) | 313.05 (+10.59% SLOWER) | 328.98 (+16.22% SLOWER) | 365.02 (+28.95% SLOWER) | 310.38 (+9.65% SLOWER, `59.0 / 35.5 / 219.9`) |
| **JCC-Native L3-COLD** | **319.95** (`55.3 / 34.9 / 227.8`) | 343.04 (+7.22% SLOWER) | 361.58 (+13.01% SLOWER) | 401.72 (+25.56% SLOWER) | 349.37 (+9.19% SLOWER, `70.9 / 41.0 / 240.8`) |

*(Wall `Instant` showed identical regression ordering under host VM preemption: e.g., JCC-Native warm `A7` wall `886.1 -> 987.6 (+11.5%) / 1051.2 (+18.6%) / 1255.5 (+41.7%) / 961.7 (+8.5%) ms`; baseline warm combined wall `618.0 -> 699.9 (+13.3%) / 830.6 (+34.4%) / 872.5 (+41.2%) / 744.7 (+20.5%) ms`.)*

---

## 4. Microarchitectural Root-Cause Diagnosis

1. **Unified 16B `inline16[code]` lookup in `ctrl_4lane_dict` is an OoO/branch-prediction strength, not overhead:**
   - The full dictionary (`inline16 + lens`) averages **`17.58 KiB`/blk** — completely L1d-resident.
   - On a 4-lane pointer-chasing walk whose critical path is L2/L3/DRAM latency on `edges[j0..j3]`, the L1d loads (`lens[code]`, `inline16[code]`) and 16-byte unaligned stores execute **in parallel in the OoO shadow of `edges[j]` cache misses**.
   - Treating 1-byte literals (`81.58%`) and 2..16-byte inline words (`11.99%`) uniformly merges **93.57%** of all `A7` steps into ONE single highly predictable branch (`len != LONG && a + INLINE <= e`, `93.57%` taken).
2. **Direct-literal branching scales monotonically worse with branch entropy and kills 4-lane MLP:**
   - `lit_branch_per_lane` (`+4.1%..+12.4%` slower): splits the `93.57%` uniform path into an unpredictable data-dependent `81.58% / 18.42%` `code < 256` branch on every lane. Each branch misprediction (~17 cycles) flushes the ROB and **stalls the in-flight 4-lane `edges[j]` loads** that feed MLP.
   - `lit_all4_fastpath` (`+12.6%..+23.5%` slower): checking `(ed0 | ed1 | ed2 | ed3) & non_lit_mask == 0` is true on **`44.59%`** of Phase-1 iterations — a near-maximal-entropy coin flip (`H(0.446) = 0.992 bits`, avg streak `2.66`), multiplying mispredictions.
   - `lit_streak_inner` (`+27.0%..+39.0%` slower): turns that `2.66`-iteration coin flip into an inner-loop exit misprediction plus outer-loop re-entry every 2.66 iterations.
   - `lit_plus_compact_dict` (`+7.3%..+12.1%` slower combined): `Dictionary::code` already short-circuits `if handle == 0 { return u32::from(byte); }` before hashing, and `begin_inline` populates `0..256` only once per worker lifetime; removing the `4.25 KiB` literal prefix saves ~0 ms in `A6b-dict` and cannot offset the `A7` branch-misprediction penalty.
3. **Diagnostic bonus — JCC alignment (`-x86-align-branch-boundary=32`):**
   - Comparing standard native vs JCC-aligned native on `ctrl_4lane_dict` showed `-7.1%` warm / `-5.9%` cold on `A7`-only (`237.6 -> 220.8 ms`, `308.0 -> 289.9 ms`), confirming JCC sensitivity in 4-lane `A7` while also proving that direct-literal branching loses by `+4.1%..+37.1%` **even on JCC-clean codegen**.

**Scope of STOP:** Closes branching direct-literal specialization (`code < 256` fast-paths, all-4-literal fast-paths, inner literal-streak loops) and compact literal-stripped `inline16` tables in 4-lane forward `A7`. Does **not** close branchless `A7` levers (e.g., higher lane count `K = 6` or `8`, prefetching, or sublinear visit-reducing contractions).
