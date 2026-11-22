# Carbon Kitchen Sink (Neon) — Cycle 37: `S1` Attack — Canonical `Cap127 + Pt64` Register-Spill Synergy & Write-Combining (`WC64/WC128`) Kill

**Date:** 2026-11-21  
**Branch:** `research/carbon-kitchen-sink`  
**Persona:** Gemini Neon  
**Baseline:** `origin/master` (`53f7ecf93`, `#719` construction stack, `945.8 KiB` scratch)  
**Scope:** Research-only; `src/`, `Cargo.toml`, `benches/`, and `examples/` untouched.

---

## 1. Mathematical Framing — Attacking Rank #1 Construction Stage (`S1: 2,044 µs`, `38.5%` of `Nobi Pre`)

Cycle 36 established `S1 (Pass 1 + OutQ16)` as the #1 remaining construction bottleneck (`2,044 µs` = `33.7%` of `Master Pre` / `38.5%` of `Nobi Pre`), driven by two distinct physical costs identified in Wormhole Cycles 22 & 27:
1. **Unnecessary Continuation Lookahead on `95.3%` of Runs:** `runs.rs` (`CAP = 127`) emits canonical maximal runs cut *only* at `127`. Since `LIMIT = 16 < 127`, any record with `width <= limit` (`489,099` thin runs = `95.3%` of all runs!) **cannot** have a same-byte continuation record `records[i + 1]`. Yet `maximal()` loads `records[i + 1]`, shifts `>> 8`, compares with `c`, and executes an inner `while` branch on every single thin run.
2. **Active-Chunk DTLB / L1D Scatter (`~90` active `1 KiB` chunks across `~80` `4 KiB` pages > `64`-entry L1 DTLB):** Can a contiguous L1D-resident write-combining stage (`WC64`: `256 × 64 B = 16 KiB` = `4` DTLB pages; `WC128`: `256 × 128 B = 32 KiB` = `8` DTLB pages, `+0 B` persistent scratch, bit-identical epoch cuts) beat direct `OutQ16` scatter while preserving Wormhole's Theorem 27.2 single-path `vmovdqu` store?

---

## 2. Native Safe-Rust A/B Confrontation (`run_c37.rs`)

Built on top of `C36-Combo` (`Wire16 + S2aFuse + S3Tag12`) and compared in the same binary against verbatim `origin/master` (`53f7ecf93`, `MasterA1` / `MasterA2` A/A control), compiled with `rustc --edition 2021 -C opt-level=3 -C target-cpu=native -C codegen-units=1` (`#![forbid(unsafe_code)]`):

- **Parity & `#703` Refusal:** `100%` byte-exact on `6/6` Wikidata blocks + `dense-300k` + `Cap::dense(300_000)` refusal (`None`). Persistent scratch unchanged at **`945.8 KiB` (`+0.00%`)**.

### 2.1 `1T` Interleaved (`50` rounds/block, `35`-round `Pre` split) & `2T` Concurrent Barrier (`35` passes)

| Candidate | `1T` Total (`µs`, ratio vs `M1`) | `1T` `Pre` (`µs`, ratio vs `M1`) | `ΔPre` vs `C36` | `2T` p10 (`µs`, ratio) | `2T` p50 (`µs`, ratio) |
|---|---:|---:|---:|---:|---:|
| `MasterA1` (`53f7ecf93`) | `10,633.3 (1.0000×)` | `6,049.5 (1.0000×)` | `+146.1` | `10,470.7 (1.0000×)` | `10,605.6 (1.0000×)` |
| `MasterA2` (A/A) | `10,648.6 (1.0014×)` | `6,054.6 (1.0008×)` | `+151.2` | `10,404.4 (0.9937×)` | `10,596.6 (0.9992×)` |
| `C36-Cmb` (prev champ) | `10,508.8 (0.9883×)` | `5,903.4 (0.9759×)` | `0.0` | `10,344.9 (0.9880×)` | `10,503.5 (0.9904×)` |
| `Cap127` (`width <= limit` fast) | `10,450.3 (0.9828×)` | `5,859.1 (0.9685×)` | `−44.4` | `10,227.0 (0.9767×)` | `10,358.9 (0.9767×)` |
| `ColdMrg` (`#[cold]` merge) | `10,486.4 (0.9862×)` | `5,910.8 (0.9771×)` | `+7.3` | — | — |
| `Pt64` (`[u64; 256]` alone) | `10,553.7 (0.9925×)` | `5,973.4 (0.9874×)` | `+70.0` | `10,368.7 (0.9903×)` | `10,511.6 (0.9911×)` |
| `WC64` (`16 KiB` 4-page WC) | `11,390.0 (1.0712×)` | `6,818.3 (1.1271×)` | `+914.8` | — | — |
| `WC128` (`32 KiB` 8-page WC) | `10,634.5 (1.0001×)` | `6,062.8 (1.0022×)` | `+159.3` | `10,389.3 (0.9922×)` | `10,613.2 (1.0007×)` |
| **`Cap+Pt64` (`Neon-C37`)** | **`10,350.8 (0.9734×)`** | **`5,774.1 (0.9545×)`** | **`−129.4`** | **`10,145.5 (0.9689×)`** | **`10,314.8 (0.9726×)`** |

Per-block `1T` ratios for **`Cap+Pt64` (`Neon-C37`)** vs `MasterA1` (A/A in parens):
`block-0: 0.972× (0.999)`, `block-2: 0.979× (1.001)`, `block-85: 0.970× (1.001)`, `block-143: 0.974× (1.000)`, `block-210: 0.969× (1.003)`, `block-207: 0.979× (1.004)` — **`6/6` clean wins!**

---

## 3. Theorems, One-Line Kills, & Promotions

- **Theorem 37.1 (Canonical-Run × `Pt64` Register-Spill Phase Transition):** Standalone `Pt64` (`+70.0 µs` `Pre`) regresses inside `maximal()`'s nested continuation loop by pushing x86-64 live GPRs over the 16-register spill cliff (matching `#719`'s `+1.3%` packed-cursor regression on EPYC). Exploiting `runs.rs`'s canonical `CAP = 127 > LIMIT = 16` invariant (`Cap127`, `−44.4 µs` alone) removes the inner continuation loop and `records[i+1]` lookahead from `95.3%` of runs, dropping GPR pressure below 16 and unlocking a **`−155 µs` super-additive synergy** (`Cap+Pt64`: **`−129.4 µs` vs `C36-Cmb`**, **`−275.4 µs` / `0.9545×` `Pre` vs `53f7ecf93`**!).
- **Theorem 37.2 (Single-Write Zero-Copy `OutQ16` Optimality):** Buffering `1.8 MiB` of ranks through an L1D/4–8-DTLB-page write-combining stage (`WC64`: `+914.8 µs`; `WC128`: `+159.3 µs`) doubles rank write bandwidth and adds `16K–55K` cold flush copies that outweigh `OutQ16`'s `1,036` DTLB misses.
- **KILL `WC64` (`+914.8 µs` `Pre`) & `WC128` (`+159.3 µs` `Pre`):** Double-writing `1.8 MiB` + cold `flush_wc` calls cost more than direct `OutQ16` scatter (Theorem 37.2).
- **KILL `ColdMrg` (`+7.3 µs` `Pre`):** Still pays the `records[i+1]` lookahead load and byte compare on all `489K` thin runs.
- **KILL standalone `Pt64` (`+70.0 µs` `Pre`):** Trips the 16-GPR spill cliff without `Cap127`.
- **PROMOTE `Neon-C37` (`Cap127 + Pt64 + Wire16 + S2aFuse + S3Tag12`, `+0 B` WS):** Cuts construction `Pre` to **`0.9545×` (`−275.4 µs`)** vs `origin/master` (`53f7ecf93`), reaching **`0.9734×` `1T`** (`6/6` wins) and **`0.9689×` p10 / `0.9726×` p50 `2T`** (vs A/A `1.0014× / 0.9937×`).

---

## 4. Artifacts & Next Step

- Reproducers archived in `docs/journal/2026-11-21-carbon-kitchen-sink-cycle37/` (`gen_c37.py`, `run_c37.rs`).
- **Cycle 38 (Neon):** Attack **Rank #2 Construction Stage (`S2b: RV Elimination`, `1,202 µs` = `22.8%` of `Nobi Pre`)** — testing **2-Byte Inline Bigram (`len=2`) & 3-Byte ASCII Inline Trigram (`len=3`) Handles inside `ID_BITS = 22` (zero rule allocation!)**, **Register-Cached Euclidean Peeling `(x, y, aw, bw, wx, wy)`**, and **1-Step Adjacent Spiral Identity Elision (`prev_x == y` / `prev_y == x`)**!
