# Prior-Art Audit for `research/neon-movement`

Before proposing or building any sense of "moving together", we audited the following branches and journals so that no existing object is presented as new, and every revisited object states explicitly what was missed.

## Sources Audited

1. **`docs/journal/2026-11-24-algorithm-level/`** (`origin/docs/journal-algorithm-level`, pinned at `b404cdf4f`):
   - `ESSENCE.md`: Hilbert's 24th problem movement — walk → Rauzy–Veech is contraction along composition (a "moves together" relation + composition rule + recursion). All senses pursued in parallel; cost rules as referee, never generator.
   - `docs/journal/2026-11-24-algorithm-level.md` §§33–34: Council decision on real machine cost `(Q, D, P, S)` under `Π` with acquisition included, plus the essence restatement.
   - `council/COMPARISON.md`: Draft agreed formulation (`Π`, `K = 2`, `s_max = |Σ| + c_0`, `(Q, D, P, S)`, bridge theorem, exactness on all columns).
   - `carbon-cycle1-critique.md` & `neon-cycle5-critique.md`: The failure mode to avoid — squeezing row-adjacency harder while omitting thick acquisition, hiding `LF` walks in `build_retained_map`, or relocating `Q` into uncharged sorts / `P ≫ K` / `S = Θ(n)` word materialisation.
2. **`origin/claude/sharp-meitner-cx0v78`** (MONODROMY records 112–114, `channel/coordinator-to-neon.md` entries #15–#21):
   - **Record 112 (`spec-pi-and-bridge.md`):** Specification `Π` (`LOADC`/`STOREC`/`NEXT` free in `Q`; `LOAD`/`STORE`/`SEEK` cost 1 in `Q`; `D` = longest dependent chain of charged accesses; `P` = sweep count; `K = 2`, `s_max = |Σ| + c_0`). Bridge holds with `κ = 1` for walk/Open/lanes, `κ = 1/2` for `T²`, and fails as a universal constant for in-place sweep composition (`Q ≥ ½ min(a⁺, a⁻)`).
   - **Record 113 (`moving-together.md`):** Formal scheme `(𝒞, ι_C, τ, w)` with S1–S5; survey of Senses A (same word / tunnelling / copy plans), B (same byte order-kept / `T²` / sweeps), C (same template offset / pairing problem `rank_g(w)`), D (Pell recurrence among induction steps). Withdraws record 110's near miss on Open's retained map under the byte-cap ruling.
   - **Record 114 (`more-senses.md`):** Senses E–L (landmark induction, distance-to-landmark order, alphabet reorder, co-lex order, bit planes, predicted address, output-side streaming, fingerprint). Works up Sense E (`Y = I_c` landmark induction) and posts `NEEDS M-21` for the six admitted blocks.
3. **`origin/research/carbon-new-basis`** (Cycle 1, C-1..C-4):
   - **C-1 (boundary chords):** Single-row chord chains via Gauss–Seidel sweep + thick induction (`Q_tot = 20–78 h₁`, `P = 10–38`).
   - **C-2 (radix-fibre `k`-cones):** `LF^k` on subalphabet (`P = 64–137`), essentially `T²` kernel.
   - **C-3 (polychromatic quotient slabs):** Hex quotient `[0-9a-f] → HEX` with rank directory; Theorem QS-1 proves height `h = 1` cannot beat `m₁₆`; consumer is row-by-row LF walk.
   - **C-4 (de Bruijn unitig–interval duality):** Per-row 8-mer name + uniqueness certificate `V₁`; hits name→address wall (`Q_tot = 38–93 h₁` at `L = 1`, `P = 110–145`).
4. **`origin/research/carbon-kitchen-sink`** (Cycles 1–12 & 45–55):
   - **Cycle 5:** Origami Galois rigidity, Durand return cylinders, Bayer–Diaconis riffle-shuffle mixing on the GUID wall.
   - **Cycle 6:** Kailath–Kung–Morf displacement commutators `[D, P_LF]` and Forman–Sköldberg algebraic Morse theory.
   - **Cycle 7:** Theorem 7.1 Burnside–BEST de Bruijn unitig condensation (prior art for C-4); Theorem 7.2 **Dyadic IET exponentiation `LF^{2^j}` via pointer-free stream merges** (`r(LF^2) = 1.21–1.25 r`, saturating at `n` by `j = 10`, totalling `12.87–14.56 n` records); Theorem 7.3 Fourier Cauchy-like resolvents (`r` active generators, 0 cancellations on 6/6 blocks); line 106 hex-quotient proposal (prior art for C-3).
   - **Cycle 8:** Post-BWT wreath factorisation and functorial non-commutativity `π(BWT(T)) ≠ BWT(π(T))`.
   - **Cycles 9–11:** Non-abelian `SL_2` sheaves, directed FVS (`0.3%` of `r`) vs non-planar diamond braiding.
   - **Cycle 12:** Theorem 12.1 Zorich acceleration of Open-gap exchanges; Theorem 12.2 `99.986%` free-monoid derivation-tree rigidity.
5. **`origin/research/neon-main-theorem`** (Cycles 1–5, N-0..N-22):
   - **N-8:** Novel hex (~65% of `h₁`) vs repeated hex (~15–20%) vs non-hex on the 6 blocks.
   - **N-12 / N-16:** Copy-plan ceilings (`0.54–0.64 h₁` supplied); N-20 Witness 20.1 proves offline `copy(dst, src, len)` acyclicity cannot be acquired locally.
   - **N-20 C1–C4:** Φ-sibling copy (`0%` of `J₁`), LZ77 oracle, Gauss–Seidel sweeps (`M-17`), and **C4 single-level unimodal orbit arches** (equivalent to Astra/SA-IS LMS cuts; `0.366 h₁` on synthetic hex at `K = 3`, `3.18–9.03 h₁` on raw LF of the 6 blocks; `LIMIT=1` figure omitted thick walk).
   - **N-21:** `|J₁ ∩ U_O| = 1` on scalar moments `(H, A, A_2, origPtr)`.
6. **`origin/research/neon-bipolar-contraction`** (Cycles 1–5):
   - Bipolar/half-bipolar contraction, bucketed sequential Schur elimination (BSSE), functional in-forest contraction (UFC), thin-excursion spelling conservation, bit-sliced implicit backward Open (BSO).

## How Each Sense in `research/neon-movement` Relates to Prior Art

| Sense | Relation to Prior Art | What Was Missed / What Is New Here |
|---|---|---|
| **S1 (`text-axis`)** | Revisits MONODROMY 114 Sense E (landmark induction, `NEEDS M-21`), `toy-transfer` lanes, Bauer–Cox–Rosone wave inversion, and Neon N-12/C1 copy plans. | (i) First exact measurement & byte-exact decode of recursive landmark induction (`k=1,2,3`) on all 6 blocks + novel hex (`NEEDS M-21`); (ii) **orthogonal two-axis composition** (row-axis thick-strip contraction FIRST, then text-axis landmark induction on `M`); (iii) forward-`Ψ` `F`-interval word sharing without `src < dst` blindness. |
| **S2 (`relabelling`)** | Avoids already-killed post-BWT alphabet quotient (Carbon C-3 / kitchen-sink C7–C8). Connects to Baker (1993) parameterized matching and MONODROMY 114 Sense G. | (i) **Period-`p` Strided / Arithmetic-Progression Rauzy–Veech**: exact affine transport `t ↦ base + μ·t` on periodic lone-row patterns (`a,b,a,b,...`) and constant-stride progressions; (ii) **monotone ramp** runs (`c_0 + j`); (iii) **Parameterized `S_Σ`-equivariance** (exact Bell-number collapse `16^k → B_k` + proved Equivariance Obstruction Theorem quantifying rank discrepancy). |
| **S3 (`pell-recurrence`)** | Revisits kitchen-sink Cycle 7 Theorem 7.2 (`LF^{2^j}` on all `n` rows) and MONODROMY 108/109/112 doubling/list ranking; Chiang et al. (1995) external-memory list ranking. | Kitchen-sink C7 ran `LF^{2^j}` on raw `LF` (`n` rows, `13n` records). Here we apply pointer-free radix-sort-merge doubling `R_{j+1} = R_j ∘ R_j` to the **`20×`-smaller retained map `R`** (`m ≈ h₁`) **without word materialisation** (exact `Q=0, D=0` limit at `j = ⌈log₂ m⌉`, plus `j = 1, 2, 3` hybrid Pareto curve), alongside a Zorich-Pell cycle-acceleration audit. |
| **S4 (`independent-set`)** | Revisits Neon N-20 C4 (single-level arches) & Astra LMS, and introduces **Miller–Reif (1985) / Cole–Vishkin (1986) recursive independent-set contraction**. | Neon C4 stopped at 1 level (`m → m/3`) and never recursed. Iterating independent-set contraction `V_0 ⊃ V_1 ⊃ ...` (both Local-Minima `~3^{-k}` and Cole–Vishkin bit-difference) on `R` gives **`O(m)` geometric total streamed work (`~1.5 m`)** instead of S3's `O(m log m)` (`~16 m`), with `Q = 0` via radix-sort-merge. |
| **S5 (`frontier-wave`)** | Revisits MONODROMY 104 (ordered frontier), 113 §3 (Relation C pairing), and `neon-bipolar-contraction` C4 (thin excursions). | MONODROMY 113 assumed scanning `n`-byte `L` per round and knowing template boundaries. Here **`∂_thick` (exits from Rauzy–Veech thick strips)** is an **automatic, template-free wave front**, and each synchronous round merges sorted `F_t` against **`IN` (only `r ≈ 0.06–0.14 n` run records)** — yielding `Q = 0, D = 0` at `P_bytes = d·r/n` and a hybrid `(Q, P)` frontier. |
| **S6 (`ifs-discrepancy`)** | Generalizes MONODROMY 114 Sense J ("predicted address") via **Iterated Function System (IFS) contraction theory** and Brownian-bridge discrepancy. | Proves the **IFS Contraction-vs-Boundary-Error Theorem**: early address errors decay geometrically as `|Σ|^{-k}` (vanishing by `k = 5`), yet the final step's `O(√(n/|Σ|))` discrepancy cannot be contracted because `L[y]` is unsmoothed across adjacent rows. Quantifies piecewise-linear IFS residuals on novel hex vs Wikidata macro-regions. |
