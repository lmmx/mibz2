# Carbon Cycle 25: Power-of-Two Safe-Rust Bounds-Check Masking (`P2Serial + P2RegBurst6`) & Fast Kill

**Date:** 2026-11-21  
**Branch:** `research/carbon-word-transport`  
**Baseline:** `origin/master` at `90e67196` (`mibz2 0.5.11` tree) & Cycle 23 `RegBurst6` (`dea87ec6`)  
**Artifact:** [c25_p2_regburst6.rs](file:///home/antigravity/lmmx/mibz2/docs/journal/2026-11-21-carbon-cycle25/c25_p2_regburst6.rs)

---

## 1. Hypothesis: Safe-Rust Power-of-Two Masking (`edges_p2`, `code_cap`) + `NoLong` Specialization

In `#![forbid(unsafe_code)]` Rust, indexing `edges[j as usize]`, `lens[code]`, `inline16[code]`, and `word_len[code]` emits panic bounds-check branches unless LLVM can prove the index is masked below slice length. Cycle 25 tested sizing `edges` to `1 << bits` and padding `inline16`/`lens`/`word_len` to `code_cap = (256 + words).next_power_of_two()` (plus specializing `!has_long` when `longest + 1 <= 16` and moving `sc_cid` out of Stage 1 GPRs) across both `P2Serial` ($m < 140\text{k}$) and `P2RegBurst6` ($m \ge 140\text{k}$).

Byte-exact parity (`warm` + `fresh` + `Plain`) passed on all `431` `wikidata.bz2` blocks.

---

## 2. Headline Native Safe-Rust A/B + A/A Results (`W = 1` and `W = 4`)

| Metric / Workload | Workers | `A/A` Control | Cycle 23 `RegBurst6` (exact sizes) | Cycle 25 `P2Serial + P2RegBurst6` |
|---|---:|---:|---:|---:|
| Worker Peak Heap (`max_KiB`, `W = 10`) | `W = 10` | `1,268.7 KiB` | **`1,275.2 KiB` (`+6.5 KiB`)** | `1,480.2 KiB` (`+211.5 KiB`) |
| `six-sum` (`6` admitted blocks, kernel) | `W = 1` | `0.9945×` | **`0.9767×`** | `0.9811×` (`11.37 / 11.16 ms`) |
| `dense-sum` (`5` dense blocks, kernel) | `W = 1` | `0.9964×` | **`0.9094×`** | `0.9116×` (`12.47 / 11.36 ms`) |
| `wiki-open233` (`233` Open blocks, kernel) | `W = 1` | `1.0069×` | **`0.9927×`** | `1.0037×` (`345.4 / 346.7 ms`) |
| `wiki-all431` (`431` blocks, kernel) | `W = 1` | `1.0070×` | **`0.9928×`** | `1.0037×` (`345.8 / 347.0 ms`) |
| `six-pass` (barrier-synced homogeneous) | `W = 4` | `1.0216×` | **`0.9292×`** | `0.9783×` (`21.80 / 21.33 ms`) |
| `dense-pass` (barrier-synced homogeneous) | `W = 4` | `1.0016×` | **`0.8498×`** | `0.8561×` (`25.78 / 22.07 ms`) |
| `wiki-all431` (barrier-synced homogeneous) | `W = 4` | `1.0024×` | **`0.9776×`** | `0.9870×` (`722.9 / 713.6 ms`) |
| `E2E BzReader` (`wikidata.bz2` full decode) | `W = 1` | `1.0025×` | **`0.9894×`** | `1.0013×` (`449.1 / 449.7 ms`) |
| `E2E BzReader` (`wikidata.bz2` full decode) | `W = 4` | `1.0096×` | **`0.9853×`** | `0.9912×` (`294.1 / 291.5 ms`) |

---

## 3. `chaingrind` Diagnostic

| Arm (`six`, `5,399,692` B) | Instructions | Proxy Cycles | `dep_past_l2_loads` | `max_chain_past_l2` | `rep_stores` | DRAM B/outB |
|---|---:|---:|---:|---:|---:|---:|
| `origin/master` (`90e67196`) | `88,300,666` | `38,485,418` | `395,446` | `2,881` | `18,528` | `0.798` |
| Cycle 23 `RegBurst6` | `104,671,163` | `36,301,541` | `407,793` | `774` | `30,816` | `0.846` |
| Cycle 25 `P2RegBurst6` | `103,541,444` | `35,924,345` | `314,592` | `1,188` | **`132,384` (`4.3×`)** | `0.871` |

---

## 4. Kill Decision

- **`P2Serial + P2RegBurst6` (power-of-two table padding):** **KILLED** — rounding `edges` to `1 << bits` grows `W = 10` worker heap by `+211.5 KiB` (`1,268.7 -> 1,480.2 KiB`) and padding `inline16`/`lens`/`word_len` to `next_power_of_two` multiplies zero-fill `rep_stores` by `4.3×`, losing `~1.0%` natively against Cycle 23 `RegBurst6` (`0.9870×` vs `0.9776×` at `W = 4`, `1.0037×` vs `0.9928×` at `W = 1`) because x86-64's branch predictor already predicts never-taken bounds checks at zero cycle cost.
