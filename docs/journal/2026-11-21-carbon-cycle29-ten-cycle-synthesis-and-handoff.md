# Carbon Cycle 29: Unified `candidate_open_c29.rs`, `moonshot check` Audit (`0` Failures), & 10-Cycle Handoff (`Cycles 20–29`)

**Date:** 2026-11-21  
**Branch:** `research/carbon-word-transport`  
**Baseline:** `origin/master` at `90e67196` (`mibz2 0.5.11` tree, including `#692` memory diet and `#703` dense-block admission)  
**Drop-In `moonshot` Deliverable:** [candidate_open_c29.rs](file:///home/antigravity/lmmx/mibz2/docs/journal/2026-11-21-carbon-cycle29/candidate_open_c29.rs) (`#![forbid(unsafe_code)]`, Rust `1.80` compatible)

---

## 1. Executive Summary

Starting from `origin/master` (`90e67196`) without redoing `#692`'s memory diet or duplicating Wormhole Cycle 10's `DietRelay8Cells`, Carbon Cycles 20–29 attacked `OpenCells`' remaining serial dependent loads past L2 (`dep_past_l2_loads`) and producer linked-list stalls under a strict **in-process native safe-Rust A/B + A/A headline discipline** at `W = 1` and `W = 4` (with `W = 8, 10` multi-worker verification and `chaingrind` strictly diagnostic).

The unified drop-in candidate [candidate_open_c29.rs](file:///home/antigravity/lmmx/mibz2/docs/journal/2026-11-21-carbon-cycle29/candidate_open_c29.rs) combines **two orthogonal native-verified breakthroughs**:
1. **`LeanRegBurst6_256` (`m >= 140_000`, Cycles 23, 24, 27):** Decoupled 6-lane scalar-register scout burst (`Stage 1`, touching *only* `edges + word_len` `[~6.5 KiB]`) + `< 256`-step L1d macro-cycle stitch (`Stage 2`) + 6-lane scalar-register `16 B` sloppy emit burst (`Stage 3`, touching *only* `edges + lens + inline16`, with a single 6-way bitwise `LONG = 0x80` sentinel check) + `<= 255 × 15 B` cut-start fixup (`Stage 4`), using exact `[...; 256]` stack arrays + `u32` `t_start` (**`+0` extra `rep_stores` over `origin/master`**) and single-caller `#[inline(always)] emit_serial` separated from `#[cold] emit_serial_cold`.
2. **`Elim::replace` Direct 4-RMW Splice + `#[inline(always)] Elim::node` (Cycle 28):** Replaces `insert(y, x, 0) + cut(y, 0)` on equal-width Rauzy steps (`aw == bw`, `x != y`) with a direct 4-RMW in-place splice (`-199k` stores / `-183k` loads on `six`), eliminating back-to-back packed-`u16` store-to-load forwarding stalls on `a[y * 4]` across all `233` Open blocks.

### Exhaustive Correctness Gate
- **`moonshot check --arm candidate --wikidata /tmp/wikidata.bz2`:** **`{"arm":"candidate","verdict":"PASS","failures":0}`** across `300` crafted random/periodic/multi-cycle/short/full columns, `Cells` witnesses `0..3`, `209,700,196`-byte `wikidata.bz2` (sequential + parallel workers), and `600` mutated fuzz inputs.
- **Worker Heap (`W = 1, 4, 10`):** `1,359.0 / 1,348.8 / 1,275.2 KiB` vs `origin/master`'s `1,351.4 / 1,342.0 / 1,268.7 KiB` (**`+6.5 KiB` = `+0.51%`** at `W = 10`).

---

## 2. Final High-Repetition Native Safe-Rust A/B + A/A Confrontation (`35` Block Reps, `9` Corpus Reps)

All timings are paired ABBA medians in the **same binary** (`rustc -C opt-level=3 -C target-cpu=native`) against `origin/master` (`90e67196`), alongside codegen-identical `MasterB / MasterA` A/A controls:

| Workload / Metric | Workers | `A/A` Control (`MasterB / MasterA`) | **`candidate_open_c29 / MasterA` (`A/B`)** | Delta |
|---|---:|---:|---:|---:|
| `six-sum` (`6` admitted blocks, `6/6 < 1.00×`) | `W = 1` | `0.9932×` (`11.59 / 11.51 ms`) | **`0.9615×`** (`11.60 / 11.16 ms`) | **`-3.85%`** |
| `dense-sum` (`5` dense blocks, `5/5 < 0.93×`) | `W = 1` | `0.9959×` (`12.56 / 12.51 ms`) | **`0.8960×`** (`12.49 / 11.19 ms`, `blk-270` **`0.8306×`**) | **`-10.40%`** |
| `wiki-open233` (`233` Open blocks, kernel) | `W = 1` | `0.9955×` (`356.30 / 354.69 ms`) | **`0.9689×`** (`354.39 / 343.37 ms`) | **`-3.11%`** |
| `wiki-all431` (`431` blocks, kernel) | `W = 1` | `0.9955×` (`356.64 / 355.02 ms`) | **`0.9690×`** (`354.70 / 343.69 ms`) | **`-3.10%`** |
| `six-pass` (barrier-synced homogeneous) | `W = 4` | `0.9888×` (`23.10 / 22.84 ms`) | **`0.9343×`** (`22.99 / 21.48 ms`) | **`-6.57%`** |
| `dense-pass` (barrier-synced homogeneous) | `W = 4` | `1.0056×` (`25.87 / 26.02 ms`) | **`0.8342×`** (`25.85 / 21.56 ms`) | **`-16.58%`** |
| `wiki-open233` (barrier-synced homogeneous) | `W = 4` | `0.9916×` (`741.38 / 735.17 ms`) | **`0.9585×`** (`740.62 / 709.90 ms`) | **`-4.15%`** |
| `wiki-all431` (barrier-synced homogeneous) | `W = 4` | `0.9905×` (`742.71 / 735.67 ms`) | **`0.9585×`** (`741.03 / 710.24 ms`) | **`-4.15%`** |
| `wiki-all431` (barrier-synced scaling check) | `W = 8` | `0.9900×` (`1513.2 / 1498.1 ms`) | **`0.9584×`** (`1496.8 / 1434.6 ms`) | **`-4.16%`** |
| `wiki-all431` (barrier-synced scaling check) | `W = 10` | `0.9942×` (`1860.9 / 1850.1 ms`) | **`0.9588×`** (`1857.4 / 1780.9 ms`) | **`-4.12%`** |
| `E2E BzReader` (`wikidata.bz2` full decode) | `W = 1` | `0.9896×` (`447.85 / 443.17 ms`) | **`0.9738×`** (`456.14 / 444.20 ms`) | **`-2.62%`** |
| `E2E BzReader` (`wikidata.bz2` full decode) | `W = 4` | `0.9919×` (`301.17 / 298.73 ms`) | **`0.9670×`** (`301.26 / 291.31 ms`) | **`-3.30%`** |
| `E2E BzReader` (`wikidata.bz2` full decode) | `W = 8` | `1.0004×` (`314.26 / 314.38 ms`) | **`0.9655×`** (`312.52 / 301.73 ms`) | **`-3.45%`** |
| `E2E BzReader` (`wikidata.bz2` full decode) | `W = 10` | `0.9965×` (`313.35 / 312.26 ms`) | **`0.9460×`** (`317.12 / 300.01 ms`) | **`-5.40%`** |

### Per-Block `W = 1` Medians (`35` Reps — `11/11` Blocks Beat `origin/master`)
- **`six`:** `block-0` (`m=89.9k`): **`0.9763×`** (`p10 0.9754×`), `block-2` (`112.1k`): **`0.9897×`** (`p10 0.9766×`), `block-85` (`156.4k`): **`0.9843×`**, `block-143` (`196.8k`): **`0.9437×`**, `block-207` (`135.9k`): **`0.9783×`**, `block-210` (`204.3k`): **`0.9292×`**.
- **`dense`:** `blk-41` (`231.6k`): **`0.9171×`**, `blk-56` (`230.0k`): **`0.8961×`**, `blk-165` (`221.1k`): **`0.9040×`**, `blk-270` (`199.2k`): **`0.8306×`** (`p10 0.8191×`), `blk-278` (`201.4k`): **`0.9263×`**.

---

## 3. Complete 10-Cycle Ledger & One-Line Kill Decisions (`Cycles 20–29`)

| Cycle | SHA | Candidate / Hypothesis | Verdict & One-Line Reason |
|---|---|---|---|
| **20** | `baf9c713` | `CodeTape8` (1-touch `u16` code tape into dead `edges[m..]` + `bands`) | **KILLED** — per-step tape packing + phase-2 reload costs `1.1956×` (`W=1`) / `1.1784×` (`W=4`) despite `-65%` `chaingrind` `dep_past_l2_loads`. |
| **21** | `90fcfb7a` | `SuperEdge` (branchless reverse-scan producer pointer-jumping into `inline16`) | **KILLED** — mutating `edges` post-wire incurs store-to-load aliasing + `+64.2%` instructions (`1.2672×` `W=1`, `1.2536×` `W=4`). |
| **22** | `7da33fd9` | Bidirectional `LIMIT ∈ {6..40}` sweep & `AdaptiveLIMIT24` (`OutQ24`) | **KILLED** — `L < 16` loses `5–26%` to `Elim`'s `~150 ns/strip` cost while `L = 24` loses `1.0145×` at `W = 4` (`dep_past_l2_loads +9.1%`). |
| **23** | `dea87ec6` | **`RegBurst6`** (`m >= 140k` 6-lane scalar-GPR scout + emit bursts, `LONG = 0x80`) | **KEPT** — eliminates stack-cursor store-buffer stalls, beating `origin/master` across `W = 1` (`0.9094×` dense) and `W = 4` (`0.9776×` wiki). |
| **24** | `cf9a290f` | `RegBurst8` (`8` lanes), zero-heap `wlen_of`, and `CUT_LOG ∈ {7, 8, 9}` | **KILLED `8L` / `wlen_of` / `128,512` cuts** — `8` lanes spill x86-64's 15 GPRs (`1.010×` `W=4`); `wlen_of` loses `+9.5%` on `dense`; `256` cuts (`CUT_LOG = 8`) is optimal. |
| **25** | `87913f4c` | `P2Serial + P2RegBurst6` (power-of-two `edges`/`inline16` bounds masking) | **KILLED** — adds `+211.5 KiB` heap and `4.3×` `rep_stores` (`0.9870×` vs `0.9776×` `W=4`) for branches hardware already predicts for free. |
| **26** | `17971266` | `RegRelay6` (scalar-GPR 1-direct + 5-scout relay, `Q_rand = 1.82m`) | **SUPERSEDED** — won `W = 1` (`0.9866×` wiki, `0.8923×` dense) but lost to `LeanRegBurst6_256` at `W = 4` due to concurrent `word_len + inline16` L1d pressure on SMT. |
| **27** | `022d8723` | `Fused DirectInline`, `&mut RelayScratch`, vs **`LeanRegBurst6_256`** | **KILLED `FusedInline` (`1.0221×`, Phase 3 evicts `inline16`) & `&mut RelayScratch` (`1.0185×`, GPR spill); KEPT `LeanRegBurst6_256`** (`+0` extra `rep_stores`, `0.8153×` `T4 dense`). |
| **28** | `c3c5760d` | `SWAR8+32 OutQ`, `TightSerial`, vs **`Elim::replace` (`aw == bw`)** | **KILLED `SWAR8+32` (`1.0029×`, branch + broken SIMD wire) & `TightSerial` (neutral); KEPT `Elim::replace` + `#[inline(always)] node`** (`0.9711×` `T1` / `0.9684×` `T4` wiki). |
| **29** | *(HEAD)* | Unified **`candidate_open_c29.rs`** (`LeanRegBurst6_256` + `Elim::replace`) | **SHIPPED TO HANDOFF** — `moonshot check` `PASS` (`0` failures), **`0.9690×` `W = 1` / `0.9585×` `W = 4..10` kernel**, **`0.9670×` (`W = 4`) / `0.9460×` (`W = 10`) E2E**. |

---

## 4. Coordinator Reproduction Commands

```bash
MOONSHOT_CANDIDATE=docs/journal/2026-11-21-carbon-cycle29/candidate_open_c29.rs \
  cargo build --release --manifest-path bench/moonshot/Cargo.toml

bench/moonshot/target/release/moonshot check --arm candidate --wikidata corpora/wikidata.bz2
```
