# Carbon Kitchen Sink (Neon) — Cycle 28: Church–Rosser Bidirectional Rauzy–Veech Induction (`BiRV`, $\mu_{\mathrm{RV}} = 2$), ASCII Bucket Skew, & `BiOp48` (`945.8 KiB = −19.28%` WS)

**Date:** 2026-11-21  
**Branch:** `research/carbon-kitchen-sink`  
**Theme:** Proving & exhaustively verifying two-ended Church–Rosser confluence of interval-exchange induction (`BiRV` via a `mid` sentinel between `terminals` and `order`), isolating Left vs Right vs Bidirectional step counts and ASCII bucket asymmetry, and confronting `BiRV`, `LeftRV`, `FusedOp`, and `BiOp48` (`48 KiB` `spelling` exact-fit) natively against `0.5.11` `master`.

---

## 1. Mathematical Construction: Church–Rosser Two-Ended Induction (`BiRV`)

In [`endpoint.rs`](../../../../src/open/endpoint.rs), `Elim` links each side as `0 <-> terminals[side] (width m) <-> order[side] (width n) <-> 0` and eliminates `order[0]` vs `order[1]` serially from the right sentinel (`prev(0, 0), prev(0, 1)`). Inserting one zero-width boundary sentinel `mid` between `terminals[side]` and `order[side]` aligns both ends of the eliminated region `[m, m + n)` identically:
- **Left front:** `(xL, yL) = (next(mid, 0), next(mid, 1))`, replacing left prefixes via `insert(prev(yL, 0), xL, 0)` (when `aw <= bw`) or `insert(prev(xL, 1), yL, 1)` (when `aw > bw`).
- **Right front:** `(xR, yR) = (prev(0, 0), prev(0, 1))`, replacing right suffixes via `insert(yR, xR, 0)` or `insert(xR, yR, 1)`.

### Theorem 28.1 (Two-Ended Church–Rosser Confluence & Step-Count Non-Monotonicity)
1. **Confluence:** Whenever `{xL, yL} ∩ {xR, yR} = ∅`, the left and right steps eliminate disjoint sub-intervals of `order` (`wl + wr <= w(xL) + w(xR) <= remaining`), commuting as block-diagonal Schur complements and converging to the **100.00% identical canonical band partition** (`verified across 25,656 exhaustive binary/ternary BWTs + 500 malformed non-BWT columns + 6 Wikidata blocks + dense-300k`; exact band counts `3204, 4244, 2446, 6788, 4219, 4596`).
2. **ASCII Bucket Asymmetry & Bidirectional Step Reduction:** Pure `LeftRV` executes **`+0.6%` to `+10.4%` MORE steps** (`+8.2%` to `+19.0%` more rules) than `RightRV` on `6 / 6` blocks because narrow control/punctuation buckets cluster at the left end of `order[1]`. Yet simultaneous `BiRV` executes **FEWER steps than BOTH `RightRV` and `LeftRV` on `4 / 6` blocks** (`0.967x` on `block-143.L`, saving `700` steps & `425` rules).
3. **L2-Resident Branch-Bound Obstruction:** Because `Elim`'s active node arena (`100–250 KiB`) fits in L2, `Elim` is governed by `50/50` unpredictable `aw <= bw` branch mispredictions rather than L3 pointer latency. `BiRV` adds the `prev(yL, 0)` left-insert indirection + collision guard + doubled code footprint (`+174.3 µs` `construct`), while `FusedOp`'s eager 4-word `len_and_val` load adds `+34.3 µs` on long-rule misses.

---

## 2. Native Safe-Rust Confrontation Against `0.5.11` `master` (`65` Pairs/Block `1T`, `45` Passes `2T`)

| Block / Metric | `A/A` (`M2/M1`) | **`BiOp48`** (`48 KiB` `spelling`) | `FusedOp` (single-load `join`) | `BiRV` ($\mu_{\mathrm{RV}}=2$) | `LeftRV` (left-only) |
|---|---:|---:|---:|---:|---:|
| `block-0.L` (`1T p10` / steps) | `1.029` | **`0.914x`** (`11,215` steps) | `0.915x` | `0.911x` (`11,528`) | `0.960x` (`12,379`) |
| `block-2.L` (`1T p10` / steps) | `1.003` | **`0.897x`** (`13,671` steps) | `0.899x` | `0.900x` (`13,680`) | `0.944x` (`14,842`) |
| `block-85.L` (`1T p10` / steps) | `1.000` | `0.904x` (`7,130` steps) | **`0.883x`** | `0.897x` (`7,064`) | `0.905x` (`7,172`) |
| `block-143.L` (`1T p10` / steps) | `0.996` | `0.858x` (`21,295` steps) | `0.858x` | **`0.854x`** (`20,595`) | `0.868x` (`21,792`) |
| `block-210.L` (`1T p10` / steps) | `0.960` | **`0.832x`** (`12,140` steps) | `0.841x` | `0.840x` (`11,986`) | `0.857x` (`12,731`) |
| `block-207.L` (`1T p10` / steps) | `1.004` | **`0.856x`** (`13,898` steps) | `0.884x` | `0.869x` (`13,761`) | `0.911x` (`14,518`) |
| **`1T` 6-Block Sum (`p10`)** | **`0.9945`** | **`0.8693x`** (`c=6110.8`) | `0.8737x` (`c=+34.3`) | `0.8717x` (`c=+174.3`) | `0.8979x` (`c=+448.1`) |
| **`dense-300k`** | `1.014` | `0.777x` | **`0.773x`** | `0.794x` | `0.780x` |
| **`2T` Concurrent (`p10 / p50`)** | **`1.0072 / 1.0012`** | **`0.8577 / 0.8621`** | `0.8824 / 0.8678` | `0.8667 / 0.8771` | `0.9066 / 0.9024` |
| **6-Blk Worker Scratch** | `1,171.7 KiB` | **`945.8 KiB` (`−19.28%`)** | **`945.8 KiB` (`−19.28%`)** | `945.8 KiB` (`−19.28%`) | `945.8 KiB` (`−19.28%`) |

---

## 3. Cycle 28 Verdicts (One-Line Kills & Promotion)

- **Kill (`BiRV` Bidirectional & `LeftRV` Left-End Rauzy–Veech Induction):** Killed (`construct` `+174.3 µs` / `+448.1 µs` vs `BiOp48`; `2T` `0.8667x` / `0.9066x` vs `0.8577x`) because L2-resident `Elim` is branch-bound (`aw <= bw`) and left-end inserts pay an extra `prev` load plus ASCII bucket skew (`+5–10%` steps in `LeftRV`).
- **Kill (`FusedOp` Eager 4-Word `len_and_val`):** Killed (`construct` `+34.3 µs`, `2T p10 = 0.8824x` vs `0.8577x`) because eagerly assembling `u128` before checking `ll + rl <= 15` wastes work on long-rule misses.
- **Promote (`BiOp48` — New Neon Champion):** Promoted as baseline (`945.8 KiB = −19.28%` scratch, `1T = 0.8693x`, **`2T = 0.8577x` p10 / `0.8621x` p50** vs A/A `1.0072 / 1.0012`).
