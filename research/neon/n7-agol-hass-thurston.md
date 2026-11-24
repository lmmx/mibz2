# N-7: E2 Screen 5 — Agol–Hass–Thurston interval-pairing reduction

*Directed kitchen-sink screen for Stream N (E2 open cell) under frozen parameters `K = 2` (also checked at `K = 3`), `δ = 1/2`, `S(c) ≤ openPeakScratch c`, and anti-cheat clauses 1–5 (`origin/claude/sharp-meitner-cx0v78` `4d7d83b`, `main-theorem.md` §1–1a).*

**Script:** [`research/neon/scripts/screen_cycle1_frameworks.py`](scripts/screen_cycle1_frameworks.py) (exact AHT pairing count `k = r`, width-1 pairing share `h₁/r`, and initial self-overlapping run / periodic-contraction count on all 6 admitted blocks of `origin/data/admitted-bwt`).

**Verdict:** **CLOSED at (a) Escape** — `tested failure within defined scope` (`R1` inside `𝓡`: every AHT move is `Restrict`, `Compose`, or iterated `Compose` in `𝓡`, giving `c₁ ≥ h₁` by G1) **and doubly closed at (c) Viability** (AHT scales with the pairing count `k = r`, where `62.8–75.4%` of pairings on `𝓕` already have width 1, and initial self-overlapping runs occur on only **2 of 509,269 runs / 20 of 5.4M rows = `0.00037%`** across the 6 admitted blocks).

## 0. The book (METHOD.md §6a)

1. **Input features relied on:**
   - Maximal runs `R_1, ..., R_r` and their `LF` images `LF(R_i)` as `k = r` orientation-preserving integer interval pairings on `[0, n)` across the 6 admitted blocks (`origin/data/admitted-bwt`, `41c03a5b`) and 232 census blocks (`#805`, `h₁/r = 0.7163` pooled).
   - Exact self-overlap census (`R_i ∩ LF(R_i) ≠ ∅` and periodic quotient `q = ⌊w_i / |t_i|⌋`) on all 509,269 runs of the 6 admitted blocks (`screen_cycle1_frameworks.py`).
2. **Real code path:**
   - `Cells` (`src/cells.rs`, right-end Rauzy–Veech induction) and `OpenCells` (`src/open/endpoint.rs`, endpoint induction with `eliminate::<true>` spiral contraction `O11`).
3. **Assumptions knowingly unrealistic:**
   - The self-overlap census measures initial (`t = 0`) self-overlaps; PROOF-PROGRAMME §5.4 and `src/open/endpoint.rs` (Theorem 19.2a / `CERTIFIED_WIDTH`) already record that `0.00%` of RV steps on Wikidata have `q ≥ 2` during execution as well.

## 1. Directed four-gate screen (stop at first "no")

### (a) Escape: Does Agol–Hass–Thurston interval-pairing reduction escape `𝓡`? — **NO** `[proved]`

In Agol, Hass and Thurston (*The computational complexity of knot genus and spanning area*, Trans. AMS 2006, §3), an interval-pairing instance consists of `k` pairs of equal-width integer intervals `(A_i, B_i)` on `[1, N]` paired by translation (or reflection). A BWT column `(L, p)` with `r` maximal runs is an exact orientation-preserving instance with `k = r` translation pairs `(R_i, LF(R_i))` on `[0, n)`.

**Theorem AHT-1 (AHT is strictly inside `𝓡`) `[proved]`:**
Every elementary operation of AHT (both the original component-counting version and any word-carrying extension) is a rule of `𝓡`:
1. **Interval splitting** (cutting a wider pairing `(B_j, A_j)` at an endpoint of `A_i`): **`Restrict`** in `𝓡`.
2. **Transmission** (transporting `A_i` across `(B_j, A_j)` to its translated image inside `A_j`): **`Compose`** in `𝓡` (with `Mirror` when transmitting in the `Ψ` direction).
3. **Periodic contraction / truncation** (replacing `q = ⌊w / d⌋` consecutive self-overlapping transmissions by one division step): **`q`-fold iterated `Compose`** in `𝓡` (identically Open's spiral contraction `O11`, `eliminate::<true>` with binary-squaring `power`, `src/open/endpoint.rs`).
4. **Static-pair removal** (`A_i = B_i`): closed-cycle elimination (identically Open's `O9` when `X = Y`).

Therefore, any word-carrying AHT decoder is a derivation in `𝓡` (a non-rightmost scheduling of Rauzy–Veech / endpoint induction), and by Record 98 Theorem G1 (on single-cycle columns, repaired per `N-1`) and Record 92 Theorem 4 it incurs **`c₁ ≥ h₁`** (and total transmissions `≥ r`). In particular, a width-1 pairing `(R_i, LF(R_i))` (`w_i = 1`) can never be contracted by periodic truncation and must be transmitted individually (`+1` in `c₁`). Thus AHT is **R1 (reshaping inside `𝓡`, the salami trap)**.

*(Screening stops at (a). For Rauzy–Veech hindsight, we also verify (c) Viability directly on `𝓕`:)*

### (c) Viability note (diagnostic) — **NO** `[measured]`

**Parameter cost scales with:** AHT runs in `O(k · polylog(N))` time, where `k` is the **number of interval pairings** and `N` is the total width. AHT is transformative in 3-manifold topology because normal surfaces have `k = O(100)` bands and `N = 2^{1000}` (`k ≪ N`), so periodic contractions collapse exponentially long spirals.

On `𝓕`, the regime is the exact opposite: **`k = r`** is large (`51k–129k` on the 6 admitted blocks; `48k–180k` on the 232 census blocks), **`62.8–75.4%` of all pairings already have width 1**, and self-overlapping contractions almost never exist:

| Block | Pairings `k = r` | Width-1 pairings `h₁` | `h₁ / k` | Initial self-overlapping runs | Self-overlap rows | `q ≥ 2` runs | Max `q` |
|---|---:|---:|---:|---:|---:|---:|---:|
| `block-0.L` | 51,184 | 35,116 | 68.6% | **0** | 0 | 0 | 0 |
| `block-2.L` | 54,532 | 34,262 | 62.8% | **0** | 0 | 0 | 0 |
| `block-85.L` | 79,375 | 55,504 | 69.9% | **0** | 0 | 0 | 0 |
| `block-143.L` | 108,335 | 72,578 | 67.0% | **1** | 18 | 1 | 3 |
| `block-210.L` | 128,732 | 97,054 | 75.4% | **0** | 0 | 0 | 0 |
| `block-207.L` | 87,111 | 62,407 | 71.6% | **1** | 2 | 1 | 2 |

Across all **509,269** runs (`5,399,692` rows) of the six admitted blocks, only **2 runs (20 rows, `0.00037%`)** self-overlap initially, and `h₁` is `63–75%` of `k`. Thus AHT's cost parameter `k = r` is strictly **larger** than `h₁`, failing viability by construction.

## 2. Obstruction table entry

| Field | Value |
|---|---|
| **Framework** | Agol–Hass–Thurston interval-pairing reduction (`N-7`) |
| **Route class** | **R1** (reshaping inside `𝓡`: general-schedule accelerated Rauzy–Veech induction) |
| **Gate failed** | **(a) Escape** (Theorem AHT-1: every move is `Restrict`/`Compose` in `𝓡`, `c₁ ≥ h₁` by G1) and diagnostic **(c) Viability** (`k = r ≥ h₁`, `63–75%` width-1 pairings, `0.00037%` self-overlap rows) |
| **Witness** | Any single-cycle column with `h₁ > 0` for G1; quantitatively, all 6 admitted blocks (`k = 51,184–128,732`, `h₁/k = 62.8–75.4%`, only 2/509,269 runs initially self-overlapping) and 232 census blocks (`h₁/r = 71.6%` pooled) |
| **Scope** | All interval-pairing reduction algorithms (AHT, Dunfield–Thurston train-track splitting, Erickson–Nayyeri, Schaefer–Sedgwick–Štefankovič) operating on the `r` translation pairings `(R_i, LF(R_i))` of a BWT column |
| **Escape condition** | Drop interval-pairing reductions on the `r` runs: replace individual width-1 pairings by a non-interval global grouping (`subject size ≥ 2`) whose count is `≪ h₁` on `𝓕` |
| **Reach** | Bounded (all interval-pairing / train-track splitting reductions on run pairings) |
| **Strength** | Proved (Theorem AHT-1 + Record 98 G1) and measured (exact census on 6 admitted + 232 census blocks) |

## 3. Postmortem (METHOD.md §6a)

1. **Input assumptions:** Exact counts over all 509,269 runs of the 6 admitted blocks (`screen_cycle1_frameworks.py`) and `#805` census.
2. **Model assumptions:** Directly embeds every AHT move in `𝓡` (`main-theorem.md` §2).
3. **Method:** Operational reduction of AHT to `𝓡`, plus direct measurement of `k = r`, `h₁/k`, and self-overlaps.
4. **Outcome:** Cleanly closed at gate (a) and doubly closed at (c). Together with `N-3`–`N-6`, all five initial kitchen-sink frameworks are now resolved with exact witnesses and measurements.
