# N-6: E2 Screen 4 — Discrete Morse theory and algebraic Morse reduction (Forman, Sköldberg, Perseus)

*Directed kitchen-sink screen for Stream N (E2 open cell) under frozen parameters `K = 2` (also checked at `K = 3`), `δ = 1/2`, `S(c) ≤ openPeakScratch c`, and anti-cheat clauses 1–5 (`origin/claude/sharp-meitner-cx0v78` `4d7d83b`, `main-theorem.md` §1–1a).*

**Script:** [`research/neon/scripts/screen_cycle1_frameworks.py`](scripts/screen_cycle1_frameworks.py) (width-1 functional-graph neighborhood census on all 6 admitted blocks of `origin/data/admitted-bwt`).

**Verdict:** **CLOSED at (a) Escape** — `tested failure within defined scope` (`R1` inside `𝓡` for 1-complex/2-complex discrete and algebraic Morse collapses over `ℤ⟨Σ⟩` / Fox calculus; single-row pivots over `ℤ[t, t⁻¹]` charged 1 per width-1 row under Record 100 §1, and iterative linear solvers for `(P_LF - I) pos = 1` require `Ω(n)` / `Ω(√n)` passes due to spectral gap `2 sin(π/n) ≈ 7 × 10⁻⁶`). Perseus 6 implementation techniques (256-entry LUT, prefetch, index-order compression) are `R0` engineering tuning.

## 0. The book (METHOD.md §6a)

1. **Input features relied on:**
   - The functional graph `G = ([n], {(x, LF(x))})` and the origami 2-complex of the six admitted blocks (`origin/data/admitted-bwt`, `41c03a5b`).
   - Exact width-1 neighborhood classification `(prev_is_w1, next_is_w1)` across all `h₁ = 34,262–97,054` width-1 runs on the 6 admitted blocks (`screen_cycle1_frameworks.py`).
2. **Real code path:**
   - `OpenCells` (`src/open.rs`, `src/open/endpoint.rs`), which FRONTIER.md L130 already identifies as a discrete Morse collapse of `G` with thick-strip rows matched away and retained rows `S` as critical 0-cells.
3. **Assumptions knowingly unrealistic:**
   - None in the escape reduction; Record 98 G1 (repaired for single-cycle columns per `N-1`) and Record 100 §1 (single-row charge on all fact types) govern.

## 1. Directed four-gate screen (stop at first "no")

### (a) Escape: Does discrete or algebraic Morse reduction give facts outside `𝓡`? — **NO** `[proved]`

We check all four mechanisms proposed in FRONTIER.md L130 (Forman 1998; Sköldberg 2006; Mrozek–Batko; Mischaikow–Nanda / Perseus 6):

1. **1-complex discrete Morse collapse and algebraic Morse reduction over `ℤ⟨Σ⟩` (Fox calculus) `[proved]`:**
   - On the directed 1-complex `G = ([n], {(x, LF(x))})` with edge `(x, LF(x))` weighted by `L[x]` in the free monoid ring `ℤ⟨Σ⟩`, collapsing a matched vertex-edge pair `(x, (x, LF(x)))` replaces `y → x → z` by `y → z` with gradient-path label `w(y, x) · w(x, z)`.
   - Collapsing a single width-1 vertex `x` is literally `Compose` on width-1 subject `{x}` in `𝓡` (`+1` in `c₁`).
   - Collapsing an interval `[a, b)` of adjacent vertices in one collective step (a thick-strip collapse) is valid iff all `x ∈ [a, b)` share the same translation shift `t` and emitted word `w` — i.e., iff `[a, b)` is a rigid fact `(I, h, t, w) ∈ 𝓡`.
   - **Reductions vs coreductions** (Mrozek–Batko): matching `x` with its *incoming* edge `(Ψ(x), x)` instead of its outgoing edge `(x, LF(x))` applies `Mirror` + `Compose` in the `Ψ`-direction (Backward Open) — still inside `𝓡`, `c₁ ≥ h₁` by G1.
   - **Filtered matchings** (Mischaikow–Nanda): any filtration restricting which cells are matched merely chooses a different critical set `S` (the retained set of Record 92 Model R), which is covered by Record 92 Lemma 1 & Theorem 4 and Record 98 G1 (`c₁ ≥ h₁`).

2. **2-complex discrete Morse collapse on the `(LF, +1)` origami `[proved]`:**
   - Lifting `G` to the origami 2-complex (unit squares = 2-cells, horizontal/vertical edges = 1-cells, cone points = 0-cells): collapsing a 2-cell across a free vertical edge merges `x` with `LF(x)` (`Compose` in `𝓡`); collapsing across a free horizontal edge merges `x` with `x + 1`, which preserves word equality iff `x` and `x + 1` lie in the same maximal run (`Join` in `𝓡`). Hence 2-complex Morse collapses remain inside `𝓡`.

3. **Algebraic Morse / totally unimodular linear algebra over `ℤ[t, t⁻¹]` for orbit times `pos(x)` `[proved]` + `[argued]`:**
   - Weighting each edge by `t` (forgetting letters) yields the totally unimodular linear system `(P_LF - I) · pos = 1` (mod `n`, anchored at `pos(p) = 0`), whose exact solution is the positional vector `pos(x)`.
   - **Direct elimination (Schur complement / algebraic Morse pivots):** Eliminating a width-1 vertex `x` adds row `x` to row `Ψ(x)`, a single-row pivot on `{x}` charged `1` in `c₁` under Record 100 §1 (anti-cheat clause 1). Eliminating thick strips in blocks yields Open's Schur complement on the `m × m` retained system `(P_{T_S} - I) · pos_S = len_S`, where every width-1 run remains an individual uncoupled equation (`c₁ ≥ h₁`). Note that even inside a single run `R = [a, b)`, `pos(x + 1) - pos(x)` is **not** constant, so `pos` does not collapse to one scalar unknown per run.
   - **Iterative / Krylov / `K`-pass matrix-vector solvers:** Because `LF` is a single `n`-cycle, the eigenvalues of `P_LF - I` are `e^{2πik/n} - 1` (`k = 0, ..., n - 1`), with smallest nonzero modulus `|λ_1| = 2 sin(π/n) ≈ 6.98 × 10⁻⁶` at `n = 900,000` (condition number `Θ(n)`). Any `K`-pass local polynomial/iterative solver (`K = 2` or `3`) propagates positional information by at most `K` steps along the cycle, leaving global order unresolved (`Ω(n)` passes for Richardson/Jacobi, `Ω(√n)` for Chebyshev/CG).

4. **Diagnostic measurement on Perseus "apparent pairs" (`screen_cycle1_frameworks.py`) `[measured]`:**
   Even if isolated width-1 runs (surrounded on both sides by runs of width `≥ 2`, the functional-graph analogue of Perseus's locally-settled apparent pairs) could somehow be settled cheaply, they are a small minority on `𝓕`:

| Block | `h₁` | Isolated `(¬w1, ¬w1)` | Chain entry `(¬w1, w1)` | Chain exit `(w1, ¬w1)` | Interior `(w1, w1)` |
|---|---:|---:|---:|---:|---:|
| `block-0.L` | 35,116 | 12.1% | 15.0% | 15.0% | **58.0%** |
| `block-2.L` | 34,262 | 13.6% | 15.1% | 15.1% | **56.3%** |
| `block-85.L` | 55,504 | 11.2% | 15.7% | 15.7% | **57.4%** |
| `block-143.L` | 72,578 | 12.0% | 15.5% | 15.5% | **57.0%** |
| `block-210.L` | 97,054 | 9.4% | 15.0% | 15.0% | **60.5%** |
| `block-207.L` | 62,407 | 12.1% | 15.7% | 15.7% | **56.4%** |

   Over **56–60%** of width-1 runs sit in the interior of multi-step width-1 chains, and even isolated width-1 runs still incur 1 width-1 event in `c₁`.

## 2. Obstruction table entry

| Field | Value |
|---|---|
| **Framework** | Discrete Morse theory & algebraic Morse reduction (Forman, Sköldberg, Mrozek–Batko, Perseus 6) (`N-6`) |
| **Route class** | **R1** (reshaping inside `𝓡`: exact vocabulary for Open and Backward Open); Perseus implementation tricks are **R0** |
| **Gate failed** | **(a) Escape** (Morse collapses = `Compose`/`Mirror`/`Join` in `𝓡`; linear pivots charged by Record 100 §1; iterative linear solvers fail **(b) Passes** with condition number `Θ(n)`) |
| **Witness** | Any single-cycle column with `h₁ > 0` for G1; spectral gap `|λ_1| = 2 sin(π/n) ≈ 6.98 × 10⁻⁶` at `n = 900,000`; and 6 admitted blocks (56.3–60.5% of `h₁` in interior `w1 → w1 → w1` chains, only 9.4–13.6% isolated) |
| **Scope** | All 1- and 2-complex discrete Morse collapses, coreductions, filtered matchings, algebraic Morse reductions over `ℤ[t, t⁻¹]` or `ℤ⟨Σ⟩`, and `O(1)`-pass iterative solvers on `(P_LF - I) pos = 1` |
| **Escape condition** | Solve `(P_LF - I) pos = 1` (or obtain multi-row positional relations) via a non-pivot, non-iterative, non-rigid global operator whose subject covers `≥ 2` width-1 rows simultaneously within `K = 2` passes |
| **Reach** | Bounded (all Morse / Schur-complement reductions on `G` and `O(L)`, and `O(1)`-pass Krylov solvers on `P_LF - I`) |
| **Strength** | Argued (reduction to Record 98 G1 + Record 100 §1; exact spectrum of `P_LF - I`) and measured (6 admitted blocks) |

## 3. Postmortem (METHOD.md §6a)

1. **Input assumptions:** Exact width-1 chain topology measured on all 6 admitted blocks.
2. **Model assumptions:** Relies on Record 98 G1 (repaired per `N-1`) and Record 100 §1.
3. **Method:** Exact dictionary between Morse/algebraic-Morse operations and `𝓡`'s rules, spectral analysis of the totally unimodular incidence matrix `P_LF - I`, and neighborhood census.
4. **Outcome:** Cleanly closed at gate (a); Perseus 6's engineering tricks remain valid `R0` pot-shot ideas outside the major shot. No premise `P_N` survives.
