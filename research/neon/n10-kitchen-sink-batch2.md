# N-10: Remaining kitchen-sink E2 screens — Krohn–Rhodes, Dimension groups (`K₀`), Fourier/Optimal transport, Learned coordinates & Compressed sensing

*Cycle 2 batch screen of the remaining algebra, spectral, transport, and learning frameworks from `docs/journal/2026-11-20-kitchen-sink.md` under frozen parameters `K = 2` (also checked at `K = 3`), `δ = 1/2`, `S ≤ openPeakScratch`, and §1a clauses 1–6.*

**Script:** [`research/neon/scripts/measure_cycle2.py`](scripts/measure_cycle2.py) (Part C: exact `|SA[x+1] - SA[x]|` adjacent-row text-position jump distribution on all 6 admitted blocks of `origin/data/admitted-bwt`).

**Verdict:** All 4 framework families are **CLOSED** (`tested failure within defined scope`). Combined with `N-3`–`N-8`, **11/11** screened kitchen-sink and reviewer frameworks are now closed with explicit obstructions.

## 0. The book (METHOD.md §6a)

1. **Input features relied on:**
   - Single-cycle property (`cycles = 1` on all 232 census blocks `#805` and all 6 admitted blocks).
   - Exact `SA[x] = (n - 1 - lf_pos[x]) mod n` jump statistics on all width-1 and non-hex width-1 rows across the 6 admitted blocks (`measure_cycle2.py`).
   - Exact `ν′(x) = 6,903–14,314` and `|U| ≤ 1–354` from `N-9`.
2. **Real code path:**
   - Baseline `OpenCells` (`LIMIT = 16`, `#805`).
3. **Assumptions knowingly unrealistic:**
   - None; all proofs hold for arbitrary single-cycle `Column` and all measurements are exact on the 6 admitted blocks.

## 1. Framework 8: Krohn–Rhodes decomposition & modular-orbit (`pos mod d`) semigroup scans

- **Bridge:** View `LF` as a transformation semigroup/automaton and factor via Krohn–Rhodes into reset semigroups (parallel-prefix free) and simple groups, or solve modular position congruences `pos(x) mod d` for prime-power divisors `d | n` (e.g., `900,000 = 2^5 · 3^2 · 5^5`).
- **(a) Escape & (b) Passes — NO `[proved]`:**
  1. **Zero reset component:** Because `LF` is a bijection on `[n]` (a single `n`-cycle on every block of `𝓕`), the transformation monoid `⟨LF⟩` is the **cyclic group `ℤ/nℤ`**, containing **no** reset (aperiodic) factor at all! Krohn–Rhodes therefore reduces to factoring `ℤ/nℤ ≅ ∏ ℤ/p_i^{e_i}ℤ` (Chinese Remainder Theorem on `pos(x) mod p_i^{e_i}`).
  2. **Modular coordinates `pos(x) mod d` are still `L`-complete and blind-flip obstructed:**
     - Computing even `pos(x) mod 2` (2-coloring the `LF` cycle from `origPtr`) is **`L`-complete under uniform `TC⁰`** (same reduction as Gamma's G1: parity of path length in a cycle), ruling out `O(1)`-depth formulas.
     - Under Record 101 Lemma 1, any valid flip `(i, m, j)` shifts `pos(x)` outside `origPtr`'s arc by `±|A_k|` — which changes `pos(x) mod d` whenever `d ∤ |A_k|`. So coarse-aggregate rules `Λ_agg` cannot determine `pos(x) mod d` either!
     - Computing `LF^d` directly by composition is inside `𝓡` (`c₁ ≥ h₁` by G1).

## 2. Framework 9: Dimension groups (`K₀` of AF algebras), Fourier analysis on `S_n`, Spectral methods, and Optimal transport

- **9a. Dimension groups and `K₀` of AF algebras (Effros–Handelman–Shen; Giordano–Putnam–Skau):**
  - **(a) Escape — NO (non-identifiability + acquisition) `[proved]`:** The ordered dimension group `K₀(B)` of a Bratteli diagram depends only on the **abelianised incidence matrices `M_k`** (forgetting the Vershik linear order `≤` on edges). Two columns with identical abelianised incidence / run-transition counts can have distinct texts (e.g., `L = "caab"` vs `L' = "cbaa"` have identical byte counts and run-length multiset `{2, 1, 1}` yet decode to `"aabc"` vs `"abac"`). Moreover, acquiring the Bratteli matrices `M_k` from `(L, p)` is inside `𝓡` (`N-3`).
- **9b. Fourier analysis on `S_n` / Spectral invariants of `P_LF`:**
  - **(a) Escape — NO (conjugacy invariance) `[proved]`:** Every single-cycle `LF` on `n` rows lies in the **exact same conjugacy class of `S_n`** (the class of `n`-cycles). Therefore **every** class function of `LF` (character values `χ_λ(LF)`, spectrum `{e^{2πik/n}}`, characteristic polynomial `λ^n - 1`, traces `Tr(P_LF^k)`) is **identically constant across all `(n - 1)!` single-cycle permutations** and carries zero bits of order information! Meanwhile, Fourier/spectral invariants of the `(256 × 256)` bigram/transition matrix or block histograms lie inside `Λ_agg`, closed by Record 101 + `N-9` (`ν′ = 6,900–14,300`).
- **9c. Optimal transport (Monge–Kantorovich coupling):**
  - **(a) Escape — NO (`R1` / `E1`) `[proved]`:** In 1D, the monotone optimal transport plan between the unsorted column `L` and sorted `F` (within each byte class) IS **literally `LF(i) = C[L[i]] + rank_{L[i]}(i)`**! Optimal transport recovers single-step successors (`E1`), NOT global text positions (`pos(x)`), and was already closed in MONODROMY Record 99 §2.

## 3. Framework 10: Learned piecewise-linear coordinates (PGM-index) and Compressed sensing / sparse error recovery on `SA[x]`

- **Bridge:** Approximate the global coordinate `pos(x) = SA[x]` by a piecewise-linear model `f(x)` with bounded error `ε` (PGM-index, Ferragina–Vinciguerra) or a sparse representation in a Fourier/wavelet basis (`SA = Φ α + e`), then resolve exact residuals in `K = 2` passes.
- **(a) Escape:** A sound exact piecewise-linear / sparse-basis positional coordinate with `≪ h₁` parameters WOULD escape `𝓡` (R2).
- **(b) Passes & (c) Viability — NO `[proved]` + `[measured]`:**
  1. **Acquisition (`(b)`):** Fitting a PGM-index or sparse representation of `SA[x]` requires knowing `SA[x]` first (which IS the inversion, Record 84); any coarse-aggregate surrogate is blocked by Record 101 + `N-9`.
  2. **Viability (`(c)`) — `SA[x]` has zero piecewise-linear smoothness or sparsity on width-1 runs (`measure_cycle2.py`) `[measured]`:**

| Block | `w1` `\|SA[x+1] - SA[x]\|` median | `w1` mean `/ n` | `w1` jumps `≤ 64` | `nonhex_w1` median | `nonhex_w1` mean `/ n` |
|---|---:|---:|---:|---:|---:|
| `block-0.L` | 223,111 | 0.298 | 1.53% | 176,272 | 0.256 |
| `block-2.L` | 222,348 | 0.296 | 1.61% | 163,383 | 0.247 |
| `block-85.L` | 198,450 | 0.281 | 0.68% | 149,854 | 0.250 |
| `block-143.L` | 206,210 | 0.277 | 0.99% | 183,797 | 0.258 |
| `block-210.L` | 178,236 | 0.256 | 0.44% | 153,281 | 0.236 |
| `block-207.L` | 211,405 | 0.288 | 1.86% | 102,865 | 0.218 |

  - Adjacent width-1 rows jump by a median of **`178k–223k` bytes** (`103k–184k` even on Stream N `nonhex_w1`!), close to a uniform random permutation's `mean/n = 0.333`, with `< 1.9%` within 64 bytes!
  - Therefore **any** piecewise-linear model of `SA[x]` with error `ε ≤ 64` requires `≥ 0.98 · h₁` segments (violating viability `≪ h₁` AND `S ≤ openPeakScratch`), and the exact residual vector `e(x) = SA[x] - f(x)` has sparsity `Θ(h₁)` (non-sparse), killing compressed sensing on `SA[x]` even if `f` were supplied free.

## 4. Obstruction table entries

| Framework | Route | Gate failed | Witness | Scope | Escape condition | Reach | Strength |
|---|---|---|---|---|---|---|---|
| Krohn–Rhodes / `pos mod d` scans | R1 / E2-blocked | **(a)/(b)** | `⟨LF⟩ = ℤ/nℤ` (zero reset part); `pos mod 2` is `L`-complete; Record 101 flips shift `pos mod d` | Semigroup cascades and modular-orbit scans on single-cycle `LF` | Non-coarse-aggregate global rule beyond `O(1)` depth solving `pos mod d` without `LF^d` composition | Global for `⟨LF⟩` cascades on primitive blocks | Proved (group structure) + argued (`L`-completeness, Rec 101) |
| Dimension groups `K₀`, `S_n` Fourier, Optimal transport | R1 / non-identifiable | **(a) Escape** | `caab` vs `cbaa` (same `K₀`/counts, different text); all `n`-cycles conjugate in `S_n`; 1D OT = `LF` (`E1`) | Abelianised BV invariants, `S_n` class functions, 1D Monge–Kantorovich coupling | Non-abelian, non-conjugacy-invariant positional invariant reading fine order | Global for `S_n` class functions and 1D OT | Proved |
| Learned piecewise-linear (PGM) & Compressed sensing on `SA[x]` | R2 (if sparse) | **(b) Passes** (needs `SA` first) & **(c) Viability** (zero smoothness/sparsity) | 6 admitted blocks: `w1 \|SA[x+1]-SA[x]\|` median `178k–223k` (`nonhex_w1` `103k–184k`), `< 1.9%` within `≤ 64` | Piecewise-linear / sparse-basis approximations of `SA[x]` in row order | Re-index `SA` by a cheaply computable non-row permutation where `SA` becomes smooth (which IS finding global order) | Global for row-order smooth/sparse models of `SA[x]` on `𝓕` | Argued (Rec 84/101) + measured exact on 6 admitted blocks |

## 5. Postmortem (METHOD.md §6a)

1. **Input assumptions:** Exact `SA[x]` jump measurements on 6 admitted blocks; `cycles = 1` verified on all 238 blocks of `𝓕`.
2. **Model assumptions:** All reductions use exact algebraic/combinatorial identities (`⟨LF⟩ = ℤ/nℤ`, `S_n` conjugacy, 1D OT = stable sort) and Record 101.
3. **Method:** Algebraic obstruction proofs + exact empirical jump census.
4. **Outcome:** Closes 4 more framework families cleanly (11/11 total closed across Cycles 1–2). No premise survives as a positive construction; feeds directly into `N-11`'s formal Stream-N premises.
