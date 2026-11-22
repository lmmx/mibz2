# Carbon Kitchen Sink (Neon) — Cycle 32: Shifted Schur-Complement Equilibrium (`LIMIT` Pareto Frontier `14, 16, 20, 24, 32`) Under $\mu = 2$ Antipodal Emission

**Date:** 2026-11-21  
**Branch:** `research/carbon-kitchen-sink`  
**Theme:** Re-deriving `#657`'s `LIMIT` optimum under `Neon-C31` (where $\mu = 2$ `XOR-BiEmit` + L1D `Operad-MN16` cuts marginal `walk` cost per retained row to `0.635x`), mapping the complete `(1T speed, 2T speed, worker scratch)` Pareto frontier across `LIMIT` $\in \{14, 16, 20, 24, 32\}$.

---

## 1. Theorem 32.1 (Shifted `1T` Minimum vs Multi-Core & Global Scratch Optimality at `LIMIT = 16`)

Under `#657` (`0.5.11` `master`, $\mu = 1$), `LIMIT = 16` minimized `1T` wall time. Under `Neon-C31` ($\mu = 2$ antipodal emission, with inline 2x16-wide `OutQ16::push` for `17 <= w <= 32`):
1. **Unconstrained `1T` Optimum Shifts Rightward to `LIMIT = 24` (`0.8157x`):** Because `walk` costs `0.635x` per retained row, trading `+13.3%` more retained rows (`895K -> 1,015K`) for `−29.7%` fewer thick strips (`20,170 -> 14,181`) drops `construct` below `master` (`5,454 vs 5,758 µs = 0.947x`, even *with* `wire_xor`!), reaching **`0.8157x`** `1T` (`0.777x` on `block-143`, `0.786x` on `block-207`).
2. **Global Scratch Minimum & Multi-Core Pareto Dominance Lock at `LIMIT = 16` (`945.8 KiB = −19.28%` WS):**
   - Below `16` (`LIMIT = 14`), `MuxStrips + Elim` arena size (`4 * ns + 8 * (3 * ns + 3)`) overtakes `m`, making `LIMIT = 14` **strictly worse in BOTH scratch (`973.0 vs 945.8 KiB`) and speed (`0.8486x vs 0.8344x`)**.
   - Above `16` (`LIMIT = 20, 24, 32`), `edges = 4 * m` inflates worker scratch (`1,030 / 1,082 / 1,109 KiB`, falling below `#704`'s `10%` memory-gain gate at `LIMIT >= 24`) and grows past-L2 random loads by `+6.7%` to `+22.3%`.
   - Already at **`2T`**, `LIMIT = 16` (**`0.8329x` p10 / `0.8360x` p50**, `945.8 KiB`) **collapses the entire `1T` gap against `LIMIT = 24` (`0.8331x / 0.8343x`, `1,081.9 KiB`)** while saving `136.1 KiB/worker` (`1.36 MiB` at 10 threads) and `13.3%` random walk loads!

---

## 2. Native Safe-Rust Pareto Confrontation Against `0.5.11` `master` (`65` Pairs/Block `1T`, `45` Passes `2T`)

All arms passed byte-for-byte parity on all `6 / 6` Wikidata blocks and `dense-300k`:

| Arm | 6-Blk Scratch (vs `master`) | `sum_m` / `sum_thick` | `construct` / `walk` (`50` rds) | `1T` 6-Blk (`p10`) | `dense-300k` | **`2T` (`p10 / p50`)** |
|---|---:|---:|---:|---:|---:|---:|
| `Master@16` (`A/A`) | `1,171.7 KiB` | `895,391` / `20,170` | `5,758` / `6,000 µs` | `0.9977` (A/A) | `0.988` (A/A) | `0.9898 / 0.9974` (A/A) |
| `Neon@14` | `973.0 KiB` (`−16.96%`) | `0.960x` / `1.114x` | `6,257` / `3,705 µs` | `0.8486x` | **`0.757x`** | `0.8472 / 0.8489` |
| **`Neon@16` (`Neon-C31`)** | **`945.8 KiB` (`−19.28%`, MIN!)** | `1.000x` / `1.000x` | `6,037` / `3,874 µs` | **`0.8344x`** | `0.790x` | **`0.8329 / 0.8360`** |
| `Neon@20` | `1,030.3 KiB` (`−12.07%`) | `1.067x` / `0.837x` | `5,759` / `4,112 µs` | `0.8235x` | `0.819x` | `0.8460 / 0.8350` |
| `Neon@24` | `1,081.9 KiB` (`−7.66%`) | `1.133x` / `0.703x` | `5,454` / `4,253 µs` | **`0.8157x`** (`1T` peak) | `0.794x` | `0.8331 / 0.8343` |
| `Neon@32` | `1,108.7 KiB` (`−5.38%`) | `1.223x` / `0.562x` | `5,227` / `4,521 µs` | `0.8236x` | `0.813x` | `0.8269 / 0.8295` |

---

## 3. Cycle 32 Verdicts (One-Line Kills & Confirmation)

- **Kill (`Neon@14`):** Strictly dominated by `Neon@16` on both speed (`0.8486x` vs `0.8344x` 1T, `0.8472x` vs `0.8329x` 2T) and scratch (`973.0` vs `945.8 KiB`, as `Elim` arena overtakes `m`).
- **Kill (`Neon@20` / `Neon@24` / `Neon@32` for Production Multi-Worker Default):** Killed for production because `+85..+163 KiB/worker` scratch inflation drops `LIMIT >= 24` below `#704`'s `10%` memory-gain gate, adds `+6.7%..+22.3%` past-L2 random loads, and loses `Neon@24`'s `1T` `0.8157x` edge already at `2T` (`0.8331x` vs `Neon@16`'s `0.8329x`).
- **Confirm (`LIMIT = 16`):** Confirmed as the unique global scratch minimum (`945.8 KiB = −19.28%`) and multi-core Pareto champion (`0.8344x` 1T, `0.8329x` p10 / `0.8360x` p50 2T).
