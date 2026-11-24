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

---

## Cycle 2 Prior-Art Addendum (Enrichment Along Composition + Cross-Stage + Further Contraction Senses)

Additional sources audited before proposing Cycle 2 candidates (`E1–E3`, `S7–S9`):

7. **`docs/journal/2026-11-24-algorithm-level/`** (pinned at `efac4a340723e2b26ba66a91105ab6b4c14e1e22`):
   - `ESSENCE.md` (updated): adds Principle 2 — **Enrichment along composition** (lifting into a richer algebra, cf. AD `(a, a')`), beside Contraction along composition.
   - `docs/journal/2026-11-24-algorithm-level.md` §§35–37: Louis's JAX / `page-dewarp` history (`value_and_grad`, `maxcor 10 -> 100`, `vmap`), outside verdict, and opening of the cross-stage track.
   - `CROSS-STAGE.md`: shared algebraic object / enrichment across `Huffman -> MTF -> L -> LF -> Emit`; explicitly flags external reports 5–7 (chunk MTF `(O, P)`), Edwards–Vishkin, occurrence counts alongside MTF, and Carbon cycle 38 as already known.
   - `jax-idea-verdict.md`: "look for transformations that are secretly the same computation, expressed in different coordinates."
8. **`origin/claude/sharp-meitner-cx0v78`** (MONODROMY records 114 §6 & 115, channel entries #22–#23):
   - **Record 114 §6 (`NEEDS M-21b`):** withdraws frequency-rank landmark estimate after channel #23 (`1.3–1.9%` of `n`) and registers a falsifiable count-targeted rule (`rho in {8, 16, 32}`, `k <= 4`, threshold `< 0.5%` of `n` on `>= 4/6` blocks).
   - **Record 115 (`2026-11-24-enrichment.md`, `NEEDS M-22`):** lists free by-products (`(L, F)`, `M_2` prefix histogram, gap `g(x)`, run extent, cache-line neighbours), argues no cheap rider cuts `Q`, and proposes serial per-symbol gap streams emitted from MTF (unmeasured: `NEEDS M-22`).
9. **Additional repo prior art audited so nothing is presented as new:**
   - **`origin/research/carbon-kitchen-sink` Cycle 4 (`ba9e533`):** ALREADY proved & verified the downstream **Post-BWT `RLE1 ∘ CRC32` 5-state transducer over `ℕ ⋊ GF(2)[x]/(P_32)`** (`23–83` triggers/block) AND Tropical LCP reducibility duality. Cited as Escape Hatch I in `E3`, NOT repeated as new!
   - **`origin/research/carbon-wormhole` Cycle 4 (`39e317a`, `Bidi-Open`):** Launched **2** single rays from `mark` (halving single-chain depth `m -> m/2`), distinct from `S7`'s multi-excursion `Q_thin = 0` sorted-merge wave from `∂_thick^±`.
   - **`src/run_fed.rs` (`2026-11-08`):** Serial MTF emitting `Run { c, p, len, k }` for `T`-vector (not chunk-parallel, not gap-encoded).

| Cycle 2 Item | Principle | Relation to Prior Art | What Was Missed / What Is New Here |
|---|---|---|---|
| **E1 (`cross-stage-mtf-monoid`)** | Principle 2 (Cross-Stage) | Revisits external reports 5–7 `(O, P)` MTF, `src/run_fed.rs`, and MONODROMY 115 §3 & §5 (`NEEDS M-22`). | Lifts `(O, P)` into the strictly associative semidirect monoid `(\mathbb{N}^\Sigma \times \mathbb{Z}_{\min}^\Sigma \times \mathbb{Z}_{\max}^\Sigma \times \mathbb{N}) \rtimes S_\Sigma` unifying parallel MTF, `C[c]`, Open `IN`, and MONODROMY's gap streams into ONE zero-`L` `O(log G)` scan; measures active support `K_50`; delivers exact `NEEDS M-22` figures (full `L` and retained `M_16`) with 8/8 zero-`L` byte-exact decode. |
| **E2 (`prefix-histogram-m2`)** | Principle 2 (In-Stage `ℕ^Σ`) | Answers MONODROMY 115 §4's open question on uses of free `M_2[a, b]`; extends Cycle 1 S6 IFS. | Builds exact `P = 2, Q_acq = 0` monochromatic-image 2-step bypass (8/8 byte-exact), quantifies genuine retained-to-retained savings on `M_16` vs `0%` on novel hex, and measures `9–13×` free 2-gram IFS error reduction (`2.7–2.9` bits/row). |
| **E3 (`monotone-envelope-dichotomy`)** | Principle 2 (Enrichment Theory) | Sharpens MONODROMY 115 §2's `[A]` and Neon N-18/N-21; cites `carbon-kitchen-sink` C4 as Escape Hatch I. | Proves & verifies the **Exact `1/3` Pigeonhole-Pin Law** (dynamic 1-Lipschitz monotone rank-envelope forces `33.3%` of random lone rows without reading `tt`) AND **Theorem E3.2 (Algebraic Enrichment Dichotomy)** via explicit 8/8 contiguous 3-rotation witnesses leaving every commutative monoid + `M_2` invariant (`dLF = 3`). |
| **S7 (`pairing-obstruction-bidir-wave`)** | Principle 1 | Directly settles MONODROMY 113–115's #1 open candidate (Relation C pairing `π`); extends Cycle 1 S5 & contrasts with `wormhole` C4 `Bidi-Open`. | Proves **Theorem S7.1 (Pairing-Oracle Obstruction)** (`34/40` hex digits carry zero bits from `π` via verified `π' == π` witness) AND introduces the **Bi-Directional `LF × Ψ` Sorted-Merge Wave** (both frontiers in 1 `IN` scan, `F_{t+1}` emerges directly sorted with zero bucketing, halving `D_wave` to `24–43` and setting new best `Q_thin = 0` volume `P_bytes = 14.8–54.8`). |
| **S8 (`count-targeted-landmarks`)** | Principle 1 | Directly executes MONODROMY 114 §6's pre-registered `NEEDS M-21b`; extends Cycle 1 S1B. | Evaluates MONODROMY's pre-registered `< 0.5%` gate across all 6 blocks + novel hex (8/8 byte-exact) AND combines row-axis `M_16` + count-targeted text-axis landmarks + bi-directional `Y_k` meeting for `< 0.1%` depth collapse. |
| **S9 (`unitig-endpoint-merge`)** | Principle 1 | Closes the exact open gap from `carbon-cycle1-critique.md` on Carbon C-4 (de Bruijn unitigs; kitchen-sink C7 Thm 7.1). | Keeps `L_min = 16` thick side cheap, replaces Carbon C-4's `qsort` + `38–93 h_1` thick probes with `Q = 0` ordered-frontier `k`-mer acquisition & sorted endpoint merge, and proves **Theorem S9.1 (the `k >= log_16 n` Pass-Wall)**: `0%` unique at `k = 2` (`P = 1`), `P >= 5 > K = 2` required on novel hex. |
| **R1 (`renormalisation-leads`)** | Channel #25 (`LEADS-dynamics.md`) | Directly tests Lead 1 (Feigenbaum renormalisation $\mathcal{R}_Y$ and $\mathcal{R}_{\mathrm{sq}}$) & Lead 2 (Fork dual numbers, answered by E1–E3); cites Gessel–Reutenauer (1993) / Crochemore–Désarménien–Perrin (2005), `carbon-kitchen-sink` C7 Thm 7.2 (`r(LF^2)`), and MONODROMY 113 Sense D. | Measures the exact minimum BWT alphabet size $q_{\min}(\sigma) = 1 + \mathrm{desc}(\sigma^{-1})$ under $\mathrm{LF}^2$, first-return $\mathcal{R}_Y(\mathrm{LF})$, and Open's retained map $R(M_{16})$ across all 8 blocks — proving quantitatively why squaring and first-return sections leave the small-alphabet BWT class immediately on novel random hex. |

---

## Cycle 3 Prior-Art Addendum (The Closure Problem, Channel `#26–#27`)

Sources audited before Cycle 3 (`CL1–CL4`):

10. **`docs/journal/2026-11-24-algorithm-level/`** (pinned at `c1eee5da`):
    - `chatgpt-closure.md` & journal §42: Uses Open's factorization $R = P \circ \mathrm{LF}_v$ (`2,148` per-byte `+1`-segments on `block-0.L`, `m = 89,874`, `h_1 = 35,116`), shows one in-place M-17 Gauss–Seidel sweep fragments it `21–24×` (`44.8–52.3k` segments) and bloats flat words `0.90 -> 2.51 MB`, and poses the **Closure Problem**: find a representation of composed retained-row transitions whose compactness survives a second composition across all 6 blocks, keeps word payloads cheap, handles periodic/multi-cycle columns, and builds without following transitions in emit order.
    - `neon-movement-critique.md` & `neon-movement-cycle2-critique.md` & channel `#26`: Cycle 1–2 audits (noting streaming retained-row glue in `src/open.rs` and requiring striking superseded Cycle-1 claims in `README.md`, `S4`, `S5`, completed in `c2f7e0b`).
11. **`origin/claude/sharp-meitner-cx0v78`** (MONODROMY record 116, `2026-11-24-predictable-touch-bound.md`):
    - Proves counting cannot bound `K = 2` passes, proves Theorem O ($Q \ge n / (2(k+1))$ in $\mathcal{S}_k$ via 3-necklaces), and proposes (§4) constructing a minimum-FAS 2-digit key $\kappa$ on the 256-node hex de Bruijn graph to beat `e = 2.718` in one sweep (unconstructed in record 116).
12. **Prior-art contrast for `CL1`:**
    - **Neon N-20 C4 (`neon-main-theorem`) & Astra LMS:** Ran unimodal arches on raw `LF` (`3.18–9.03 h_1` links on the 6 blocks; `LIMIT=1` had a hidden thick-row LF walk). `CL1` runs on Open's genuine endpoint-contracted $R = P \circ \mathrm{LF}_v$ (`0` hidden walks), replays the compact $B_P$ schedule (`16–43 KB`) forward and backward with zero non-$P \circ \mathrm{LF}_v$ reads, uses a zero-copy 8B binary word DAG, and verifies periodic + 200 multi-cycle corrupt columns.

| Cycle 3 Item | Relation to Prior Art | What Is New Here |
|---|---|---|
| **CL1 (`closure-mirror-factored`)** | Reproduces `chatgpt-closure.md` on `block-0.L` to the exact integer (`2,148 -> 44,814 / 46,671 / 52,319`); contrasts with M-17 GS (record 110) and raw-LF arches (Neon N-20 C4). | Mirror-factored two-sweep (`D_1 ↑` then `A_1 ↓`) keeping the $B_P$-segment $P \circ \mathrm{LF}_v$ schedule intact: survives the **second** composition with **zero fragmentation** (`650–1,939` cache16 segments = `44–133×` below M-17 GS), uses the exact same `m` total successor reads as 1 M-17 sweep, cuts emit links `10–12%` below M-17 (`0.67–1.08 h_1` on Wikidata, `0.325 h_1` on hex), and replaces `2.5–3.0 MB` flat words with an `8m` zero-copy binary DAG. |
| **CL2 (`closure-obstruction-anatomy`)** | Uses Gessel–Reutenauer / Crochemore–Désarménien–Perrin rising-sequence duality (`q_min(T) = 1 + desc(T^{-1})`) to test closure beyond Sweep 2. | Proves **Theorem CL2.1** ($\mathrm{seg}_{+1}(T, \ell) \ge q_{\min}(T)$ for *every* labelling $\ell$, equality via $\ell^*$), dissects ChatGPT Pro's `24×` jump (`first_byte` vs `last_byte` vs `A_1`/`D_1`), and measures exact $q_{\min}$ on surviving sections $R_{\mathrm{gs}}$, $R_1$, $R_2$, $R^2$, and $R_{\mathrm{exc}}$ across all 8 blocks — proving a **third** exact-`+1` monotone-cursor sweep is representation-independently blocked ($q_{\min}(R_2) = 0.19–0.25\,|M_2|$). |
| **CL3 (`debruijn-fas-thin-retention`)** | Directly constructs MONODROMY 116 §4's unbuilt 2-digit de Bruijn FAS key $\kappa$ on $B(16, 2)$ and answers Channel `#27`'s sharp enrichment question on thin runs (`src/runs.rs`). | Builds explicit $\kappa$ hitting `0.6599` descents (`17/4,096` below Thm O's `2/3` ceiling) and confirms 1-sweep M-17 breaks $e$ to `3.256` bytes/link on hex; sweeps $L_{\min} \in \{2, 3, 5, 9, 17\}$ with `CL1` across 6/6 blocks, proving eager thin-strip contraction loses in total $Q$ (`3.2–5.5×` more `Q_acq_strips` per saved row). |
| **CL4 (`endpoint-schedule-acquisition`)** | Directly satisfies `chatgpt-closure.md` §5's third condition and answers Channel `#27`'s enrichment question; extends `src/open.rs`'s streaming glue (`neon-movement-critique.md`). | Builds `CL1`'s `(target_rank_start, seg_len, full_wid)` schedule (`2,039–5,552` pieces, `23.9–65.1 KB`) purely by `2,446–6,788` linear endpoint merges of `ret_lf_intervals[b]` against `M_intervals ∪ final_strips` — `Q_acq_glue = 0` (cutting total acquisition probes `30–72%`), NEVER allocating `R[0..m)` or `R_word_id[0..m)` (8/8 byte-exact). |



