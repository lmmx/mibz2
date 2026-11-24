# N-13: Supplied Capability 2 (Outside Review §5.2) — Compressed `DSA` (`SA[i] - SA[i-1]`) with Paid Anchors and Kempa Theorem 5.1 Audit

*Cycle 3 supplied-capability measurement answering Coordinator Note #3 / Outside Review §5.2 (`origin/claude/sharp-meitner-cx0v78` `e48f904`): measure `DSA` compressibility `g_{Δ,N}`, `LF`-invariance breaks, distinct values & entropy on `h₁` and `m₁₆`, and audit Kempa Theorem 5.1 against `K = 2` and `S ≤ openPeakScratch`.*

**Script:** [`research/neon/scripts/screen_cycle3_supplied.py`](scripts/screen_cycle3_supplied.py) (exact `DSA[i] = SA[i] - SA[i-1]` census on all 6 admitted blocks of `origin/data/admitted-bwt`, plus automated verification of the round-2 arithmetic counterexample `T = [0,2,4,6,8,7,5,3,1]`).

**Verdict (narrowed in N-17):** **CLOSED for literal-terminal / symbol-frequency representations of `DSA` (at (c) Viability) and for Kempa Theorem 5.1's `O(n/τ + rτ)` suffix-sampling algorithm (at (b) Passes & workspace)** — `tested failure within defined scope`. **Does NOT close non-literal arithmetic, algebraic, or transducer coordinate representations.**
- `DSA[LF(i)] ≠ DSA[i]` breaks at **`1.000 × r`** (every BWT run boundary on 6/6 blocks `[measured]`), and `DSA` has **`887,651–899,575` equal-value runs** out of 900,000 rows (`98.6–99.96%` singletons `[measured]`).
- Restricted to `h₁` width-1 rows, **`90.7–94.2%` of all `DSA` values are distinct** (`H₀ = 14.79–16.30 bits`, near `log₂(h₁) = 15.1–16.6 bits` `[measured]`). Restricted to **Stream N (`nonhex_w1`)**, **`86.9–93.4%` are distinct** (`H₀ = 11.69–13.59 bits` `[measured]`). This lower-bounds any **literal-terminal** grammar or zero-order symbol coder over `DSA` values (`[proved]`).
- **Scope boundary (`[proved]`, round-2 counterexample verified in `test_n13_counterexample`):** Distinctness and `H₀` do **not** bound arithmetic or formulaic descriptions. For the valid single-cycle byte string `T = [0, 2, 4, 6, 8, 7, 5, 3, 1]` (`L = [1, 3, 0, 5, 2, 7, 4, 8, 6]`, `p = 0`, `SA = [0, 8, 1, 7, 2, 6, 3, 5, 4]`), all `n = 9` cyclic `DSA` values `[-4, 8, -7, 6, -5, 4, -3, 2, -1]` are distinct (`H₀ = log₂ 9`), yet `SA[2j] = j` and `SA[2j+1] = 8 - j` has an `O(1)`-size arithmetic formula! Thus N-13 rules out literal-terminal `DSA` compression and Kempa Theorem 5.1's RLBWT sampling on `𝓕`, **not** arbitrary arithmetic or transducer coordinate systems.
- Kempa Theorem 5.1 at optimal `τ = √(n/r) ∈ [2.6, 4.2]` requires **`429k–681k` words** of workspace and **`23.5M–29.0M` operations** (`26–32 × n` `[argued]`), failing `K = 2` and `S ≤ openPeakScratch`.

## 0. The book (METHOD.md §6a)

1. **Input features relied on:**
   - Exact BWT cyclic `SA`, `LF`, `r`, `h₁`, `h₁N` (`nonhex_w1`), and `m₁₆` on all 6 admitted blocks (`origin/data/admitted-bwt`, `41c03a5b`).
2. **Real code path:**
   - Baseline `OpenCells` (`LIMIT = 16`, `K = 2`, `S ≤ openPeakScratch`, `d(D, c) ≤ d(Open, c)`).
3. **Assumptions knowingly unrealistic:**
   - Measuring `DSA` distinct-value counts and Shannon `H₀` entropy on an **oracle-supplied** `SA` array sets a lower bound on **literal-terminal** `DSA` grammars and symbol coders only (zero acquisition cost assumed); it does **not** lower-bound arithmetic or transducer descriptions of `SA` (see the `T = [0,2,4,6,8,7,5,3,1]` witness above).

## 1. Exact `DSA` compressibility & Kempa Theorem 5.1 audit on the 6 admitted blocks (`[measured]`)

| Block | `r` | `LF`-breaks (`DSA[LF(i)] ≠ DSA[i]`) | `DSA` runs (of 900k) | `w1` distinct `DSA` (`%h₁`) | `w1` `H₀` (bits) | `nonhex_w1` distinct (`%h₁N`) | `nonhex_w1` `H₀` (bits) | `m₁₆` distinct (`%m₁₆`) | Kempa 5.1 opt `τ` / ws (words) / ops |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---|
| `block-0.L` | 51,184 | 51,184 (`1.00r`) | 887,651 | 32,198 (`91.7%`) | 14.79 | 4,950 (`90.4%`) | 11.69 | 43,962 (`48.9%`) | `4.2` / `429k` / `23.5M` |
| `block-2.L` | 54,532 | 54,532 (`1.00r`) | 894,023 | 32,279 (`94.2%`) | 14.87 | 5,485 (`91.3%`) | 12.04 | 47,495 (`42.4%`) | `4.1` / `443k` / `23.8M` |
| `block-85.L` | 79,375 | 79,375 (`1.00r`) | 899,575 | 51,532 (`92.8%`) | 15.57 | 8,996 (`93.4%`) | 13.02 | 71,662 (`45.8%`) | `3.4` / `535k` / `25.9M` |
| `block-143.L` | 108,335 | 108,335 (`1.00r`) | 897,583 | 67,537 (`93.1%`) | 15.96 | 12,756 (`93.4%`) | 13.45 | 94,875 (`48.2%`) | `2.9` / `624k` / `27.8M` |
| `block-210.L` | 128,732 | 128,732 (`1.00r`) | 897,031 | 88,069 (`90.7%`) | 16.30 | 13,453 (`93.3%`) | 13.59 | 112,787 (`55.2%`) | `2.6` / `681k` / `29.0M` |
| `block-207.L` | 87,111 | 87,111 (`1.00r`) | 897,073 | 57,221 (`91.7%`) | 15.67 | 10,962 (`86.9%`) | 13.06 | 76,175 (`56.0%`) | `3.2` / `560k` / `26.4M` |

## 2. What N-13 proves and what it does not (`[proved]` + `[measured]` + `[argued]`)

1. **Literal-terminal `DSA` representations fail viability on `h₁` and `nonhex_w1` (`[measured]` + `[proved]`):**
   - Because every width-1 run is a BWT run boundary (`lf_breaks = 1.000 × r` on 6/6 blocks `[measured]`), `DSA` changes value at every width-1 run. Across `h₁`, `90.7–94.2%` of `DSA` integers are distinct (`86.9–93.4%` on `nonhex_w1`), and `43,962–112,787` are distinct on `m₁₆` (`> h₁` on 6/6 blocks).
   - For any **literal-terminal** representation (a straight-line grammar, dictionary, or run-length/entropy coder whose terminals are literal `DSA` integer values), vocabulary size alone satisfies `g_Δ ≥ |distinct(DSA|_{h₁})| ≥ 0.907 · h₁` (`[proved]`), failing `g_Δ + s ≪ h₁` even with zero anchors `s` charged.
2. **Arithmetic / transducer coordinate representations are NOT closed by distinctness or `H₀` (`[proved]`):**
   - Verified witness (`test_n13_counterexample` in `screen_cycle3_supplied.py`): `T = [0, 2, 4, 6, 8, 7, 5, 3, 1]` yields `L = [1, 3, 0, 5, 2, 7, 4, 8, 6]`, `p = 0` (single 9-cycle), `SA = [0, 8, 1, 7, 2, 6, 3, 5, 4]`, and cyclic `DSA = [-4, 8, -7, 6, -5, 4, -3, 2, -1]` — `9/9` distinct values (`100%` distinct, `H₀ = 3.17` bits), yet `SA[2j] = j`, `SA[2j+1] = 8 - j` is an `O(1)`-size arithmetic map.
   - This falsifies any blanket jump from `distinct(DSA) ≈ h₁` to "no compact coordinate description of `SA` exists." (Conversely, the witness does not supply an arithmetic description on Wikidata; any future arithmetic/transducer proposal would be tested directly against the exact conjugacy certificate `f(o) = 0`, `f(LF(x)) = f(x) + 1 mod n`, not via `H₀(DSA)`.)
3. **Kempa Theorem 5.1 fails workspace `S` and pass budget `K = 2` on `𝓕` (`[argued]`):**
   - Kempa (SODA 2019, Thm 5.1) constructs `SA`/`ISA` support from RLBWT in `O(n/τ + rτ)` workspace and `O((n/τ)(log(rτ) + log(n/τ)) + τ² r log(rτ))` time.
   - Because `r/n = 0.057–0.143` on the 6 admitted blocks (vs `~10⁻⁴` on pangenomes), balancing `n/τ = rτ` forces `τ = √(n/r) ∈ [2.6, 4.2]`, giving `429k–681k` words of workspace and `23.5M–29.0M` operations (`26–32 × n`, `≫ K = 2` passes).

## 3. Obstruction table entry (narrowed in N-17, all six fields)

| Field | Value |
|---|---|
| **Framework** | Literal-terminal compressed `DSA` (`SA[i] - SA[i-1]`) & Kempa Thm 5.1 RLBWT suffix sampling (`N-13`, narrowed in `N-17`) |
| **Route class** | **R2** (if `DSA` were compactly acquirable) |
| **Gate failed** | **(c) Viability** for literal-terminal `DSA` representations (`90.7–94.2%` distinct on `h₁`, `86.9–93.4%` on `nonhex_w1`, `lf_breaks = 1.000r`) and **(b) Passes / workspace** for Kempa Thm 5.1 (`23.5M–29.0M` ops, `429k–681k` words) |
| **Witness** | All 6 admitted blocks (`screen_cycle3_supplied.py`): `32,198–88,069` distinct `DSA` values on `h₁` (`H₀ = 14.79–16.30` bits), `887,651–899,575` `DSA` runs out of `900,000`. Scope-boundary counterexample against over-generalisation: `T = [0,2,4,6,8,7,5,3,1]` (`9/9` distinct `DSA`, yet `SA[2j]=j`, `SA[2j+1]=8-j`) |
| **Scope** | **Literal-terminal / symbol-frequency representations** of adjacent `DSA[i] = SA[i] - SA[i-1]` (distinct-symbol vocabularies, literal straight-line grammars, zero-order entropy coders, equal-`DSA` run-length coders) and **Kempa Theorem 5.1** suffix sampling on `𝓕`. **Explicitly excluded:** non-literal arithmetic, piecewise-formula, or finite-state transducer descriptions of `SA` |
| **Escape condition** | Leave literal-terminal `DSA` values for a non-literal arithmetic / transducer coordinate map `f` (certified by `f(o) = 0`, `f(LF(x)) = f(x) + 1 mod n`) or a non-adjacent coordinate system whose description size and evaluation work stay `o(h₁)` on `𝓕` |
| **Reach** | **Local to literal-terminal `DSA` representations and Kempa Thm 5.1 on `𝓕`** (not global over all coordinate systems) |
| **Strength** | **Measured exact** on 6 admitted blocks (`DSA` distinctness, runs, `H₀`) + **proved** (vocabulary lower bound for literal-terminal coders; arithmetic counterexample `T = [0,2,4,6,8,7,5,3,1]`) + **argued** (Kempa Thm 5.1 scaling) |
| **Transferable insight** | First-order distinctness and `H₀` lower-bound only literal-terminal encodings, never algorithmic or arithmetic coordinate laws; conversely, because `SA` satisfies the exact `O(n)` verifiable conjugacy equation `f(o) = 0`, `f(LF(x)) = f(x) + 1 mod n`, any future coordinate-change candidate can be audited directly by exact conjugacy residual rather than `DSA` histogram statistics |

## 4. Postmortem (METHOD.md §6a)

1. **Input assumptions:** Exact BWT cyclic `DSA`, `LF`-breaks, distinct counts, and `H₀` on all 6 admitted blocks.
2. **Model assumptions:** Zero acquisition cost assumed for literal-terminal `DSA` viability test; round-2 review correctly caught that distinctness/`H₀` cannot close arithmetic coordinate formulas (`T = [0,2,4,6,8,7,5,3,1]`).
3. **Method:** Exact `O(n)` `DSA` census + automated witness verification (`test_n13_counterexample`) + Kempa Theorem 5.1 parameter scaling.
4. **Outcome (N-17):** Scope cleanly narrowed to literal-terminal `DSA` representations and Kempa Theorem 5.1 on `𝓕`; no new framework screen opened.

