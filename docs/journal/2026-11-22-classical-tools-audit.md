# Classical tools in Kitchen Sink cycles 46 and 47: weight, not mentions

*Date:* 2026-11-22
*Sources:* `research/carbon-kitchen-sink` at `f4b18ee6e` (the cycle 46 and 47 journals and the
16 Rust files under their journal directories, 27,819 lines), `origin/master` at `3296195e3`
(`src/`, the merged PRs' journals, `docs/team/research-charter/FRONTIER.md`).
*Scope:* read-only audit. Nothing was rebuilt or rerun. Every classification below rests on the
code and journal text cited, not on the journals' verdict columns.

## Result

Cycles 46 and 47 report pre-1950 shares of 31/53 (58.5%) and 35/49 (71.4%) of the tools they
tried, with 67.7% and 91.4% of the classical ones "surviving". Weighed by what the surviving code
and claims actually depend on, the shares are much smaller:

| | Cycle 46 | Cycle 47 | Both (rows) |
|---|---:|---:|---:|
| Pre-1950 rows as tagged | 31 | 35 | 66 |
| **Load-bearing** | **6** | **3** | **9** |
| Analogy | 5 | 7 | 12 |
| Relabel | 14 | 15 | 29 |
| Decorative | 6 | 10 | 16 |
| Tagged pre-1950 share of all tools | 58.5% | 71.4% | 64.7% |
| **Load-bearing pre-1950 share of all tools** | **11.3%** (6/53) | **6.1%** (3/49) | **8.8%** (9/102) |
| Load-bearing share of the pre-1950 rows | 19.4% | 8.6% | 13.6% |

Of the nine load-bearing rows:

- four credit code that was already in the baseline before the cycle, all of it already shipped
  on master: the first-return construction (Kakutani), the spiral quotient step (filed as
  Lagrange–Legendre continued fractions), and the cycle-decomposition tiling checks (Cauchy,
  counted in both cycles);
- two serve synthetic inputs only: Euclid's algorithm in the two-letter Sturmian solver and the
  `gcd(w)` periodic guard, and Ostrowski numeration. Neither fires on any Wikidata block;
- one supports a negative result with no code: the birthday bound behind the GUID wall;
- one serves the test harness only: Hierholzer's algorithm generates the de Bruijn and Eulerian
  test texts;
- one is a performance model: Whitworth's circle division predicts the multi-cursor walk depth.
  The walk it models (`DualFwd`) is blocked on the i9.

**No pre-1950 result introduced in cycles 46–47 is load-bearing in a mechanism that measured
faster on real data.** The two promoted candidates, F-Inline3 and Spec4-SPRT-L, owe nothing to
the mathematics they are filed under (§4).

**"Survived" mostly meant exact or law-checked, not faster.** Of cycle 46's 21 surviving
classical rows, 2 attach to a mechanism that measured faster on real blocks than its base:
Spec1-TinyDense (Dirichlet, a size threshold: −0.13% wall on 431 blocks, −0.9% instructions)
and Spec2-Adaptive (Dantzig–Wald, a record-count threshold: faster on 6 non-Wikidata blocks,
1.010–1.016× slower on Wikidata). Both are relabels. Of cycle 47's 32, 13 attach to the three
mechanisms that measured faster somewhere: F-Inline3, the Spec1/Spec3/Spec4 selector, and DualFwd
(on a 105 MiB-L3 Xeon only). One of the 13 is load-bearing, and only as a model (Whitworth).
The other "survivals" are baseline code given a name, byte-exact harnesses, identities that hold
by construction, or diagnostics.

## 1. Method

The classes are the brief's, applied to each pre-1950 row of the two journals' deduplicated
catalogues (cycle 46 §3.1 Part A, cycle 47 §3 Part A). Row numbers below are the journals' own.

- **Load-bearing:** the result supplies a construction, invariant, bound or proof step without
  which the surviving code or claim would not exist or would not be correct. A code line or
  theorem is named.
- **Analogy:** it motivated a direction, but the mechanism as implemented does not use it. A
  hypothesis tested and falsified goes here: it shaped a direction that was then dropped.
- **Relabel:** the code is an ordinary engineering technique given a historical name. The test
  is whether a competent engineer would write the same line without knowing the citation:
  a threshold, a counter, a parameter sweep, a histogram, a regression, an equality check,
  path compression, cache blocking, XOR-linked neighbours.
- **Decorative:** cited or computed, and nothing depends on it: an identity that holds by
  construction, a certificate redundant with a direct count, a name with no code.

Each row gets exactly one class, chosen by its role in what the cycle kept. Where a row's
"survival" is pre-existing baseline code, that is said, because the cycle contributed nothing
to it. A spot check of the post-1950 catalogues shows the same inflation there (for example
"Sleator–Tarjan online algorithm zoo" for an `if` chain dispatching on block size, "free-monoid
truncated operad" for the 16-byte inline dictionary, "greedy knapsack" for `LIMIT = 16`). So
the tagged era split is confounded on both sides. The post-1950 rows were not classified
formally.

## 2. Cycle 46: the 31 pre-1950 rows

Files are under `docs/journal/2026-11-22-carbon-kitchen-sink-cycle46/` on the research branch.

| # | Tool (as filed) | Tag | Journal verdict | Class | Evidence |
|---:|---|---|---|---|---|
| 1 | Euclid's algorithm, continued fractions | 300 BCE | Survived (guard, Sturmian) | Load-bearing (synthetic only) | `gcd(w)` early abort of `ExactPeriodicFast` (`p3/analogue_c_chain.rs`); two-letter Euclid solver (`p2/analogue_b_stepanov.rs`). On Wikidata `gcd(w) = 1` and 0.00% of RV steps have `q >= 2`: it never fires |
| 2 | Montmort / von Mises birthday collisions | 1700s | Barrier | Load-bearing (negative claim, no code) | Why 122-bit GUID contexts shatter after about ⌈log₁₆ N⌉ = 4–5 hex digits (D6: GUIDs are 28.7% of emission). The "no local shortcut" generalisation rests on it |
| 3 | Euler characteristic | 1750s | Diagnostic | Decorative | Component and cycle counts; nothing downstream |
| 4 | Lagrange–Legendre continued-fraction spiral contraction | 1790s | Survived (baseline) | Load-bearing (pre-existing, shipped) | The spiral step `q = (aw - 1) / bw` and `power` by squaring, `src/open/endpoint.rs:357–380` on master (#703). 0/431 Wikidata blocks trigger it; the cycle added nothing |
| 5 | Gauss modular progression | 1800s | Specialist | Relabel | `x = (p + t q) mod n` applied after a runtime check that all shifts are equal (`p1/stepanov_analogue_a.rs:210–256`); the Christoffel-BWT theorem it alludes to is checked, not used. Inapplicable on Wikidata |
| 6 | Cauchy–Cayley cycle decomposition | 1810s | Survived | Load-bearing (elementary, baseline) | Short-cycle tiling `repeat(n, out)` (`src/open.rs:1029` on master) and the `tot == n`, `cur == k0` closure checks. Referee bug #1 (a missing `cur != k0` corrupted 37.5% of malformed doubled columns) shows the check matters |
| 7 | Return-time Dirichlet series | 1830s | Diagnostic | Relabel | A sum of reciprocal return times; the τ = 1 share is a histogram |
| 8 | Dirichlet pigeonhole zero-strip partition | 1830s | Survived (Specialist) | Relabel | `n < 32_768` routes tiny blocks to a plain forward-FL walk (Spec1-TinyDense) |
| 9 | Cayley–Jordan ℤ/2 involution (`XOR-BiEmit`) | 1850s | Survived (baseline) | Relabel | Storing `prev ^ next` and reading a cycle from both ends: an XOR-linked list. Already shipped (`src/open/outq.rs:238` `wire_xor`) |
| 10 | Christoffel words, modular shift classes | 1870s | Falsified | Analogy | Constant shift per run tested; `D_mod / r` = 90–95% |
| 11 | Faulhaber / Dedekind–Rademacher sum | 1870s | Diagnostic | Decorative | A correlation figure, unused |
| 12 | Jordan canonical form of `U_JJ` | 1870s | Survived (diagnostic) | Relabel | The code takes lengths of consecutive τ = 1 runs along the orbit (`p1/stepanov_analogue_a.rs:2098–2122`); no matrix and no Jordan form |
| 13 | Boltzmann–Gibbs partition function | 1870s | Diagnostic | Decorative | Unused |
| 14 | Neumann series (Theorem C1-1) | 1877 | Survived (theorem) | Decorative | The "Kac–Neumann balance" (mean return word length `n/m`) holds by construction: the return words partition the text |
| 15 | Poincaré Betti numbers | 1890s | Diagnostic | Relabel | Component and cycle-rank counts of a graph |
| 16 | Pareto–Zipf width tail | 1890s | Diagnostic | Relabel | A power-law fit to the width histogram |
| 17 | Minkowski convex envelope on `(ns(L), m(L))` | 1890s | Survived (Foundational) | Relabel | A sweep of `L` over 8 values. Its finding (`L* = 16` on Wikidata, 24–32 elsewhere) is real and fed Spec4 |
| 18 | Jordan–Wielandt primitive block systems | 1892 | Closed | Analogy | Global deck quotient ruled out; no code |
| 19 | Lebesgue width-threshold split `w > 16` | 1900s | Survived (baseline) | Relabel | `LIMIT = 16`, already shipped |
| 20 | Perron–Frobenius stationarity | 1900s | Survived (exact law) | Decorative | `‖πT − π‖ ≤ 3e-16` checks a flow conservation that holds by construction |
| 21 | Thue maximal-factor classification | 1906 | Survived (diagnostic) | Relabel | A census of repeated factors |
| 22 | MacMahon descents | 1915 | Diagnostic | Relabel | A descent count |
| 23 | Weyl / Erdős–Turán discrepancy | 1916 | Falsified | Analogy | `ET(G_w1) ≈ 0` hypothesis refuted |
| 24 | Block Schur complement | 1910s | Survived (baseline) | Analogy | A retrospective algebraic reading of `endpoint.rs`'s interval-exchange elimination; no matrix in the code |
| 25 | Ostrowski numeration | 1922 | Specialist (local) | Load-bearing (synthetic only) | Exact O(log n) coordinate on 5,627 Sturmian BWTs (`p2/analogue_b_stepanov.rs`); `r(Δτ) ≈ n` on Wikidata |
| 26 | Mayer–Vietoris sequence | 1920s | Falsified / exact law | Relabel | Cycle-rank bookkeeping over a thick/thin split; the finding (84–95% of β₁ in the thin part) is a count |
| 27 | Bohr almost-periodic functions | 1925 | Falsified | Analogy | Additive period smeared by variable statement lengths |
| 28 | "Streaming bucket rank monoid" (`OutQ`) | 1930s | Survived (baseline) | Relabel | The shipped per-byte queues (OutQ16, 2026, `src/open/outq.rs`). Cycle 47's referee moved the same component to the 2020s |
| 29 | Whitehead elementary subdivision | 1930s | Obstruction | Decorative | Unused |
| 30 | Kakutani–Rokhlin first return, Kac's lemma | 1940s | Survived (baseline, Foundational) | Load-bearing (pre-existing, shipped) | The Open kernel is the first return to whole LF strips (`src/open.rs:1–16`; proof `docs/journal/2026-11-18-open-return/proof.md`). The cycle's own use, A-Tube, lost 1.46–2.64× |
| 31 | Dantzig–Wald break-even; Shannon entropy | 1940s | Survived (Foundational, bound) | Relabel | Spec2's `records < 60_000` threshold; Shannon's bound sizes a refuted table (DSID) |

## 3. Cycle 47: the 35 pre-1950 rows

Files are under `docs/journal/2026-11-22-carbon-kitchen-sink-cycle47/`.

| # | Tool (as filed) | Tag | Journal verdict | Class | Evidence |
|---:|---|---|---|---|---|
| 1 | Euler degree balance, Eulerian circuits | 1736 | Survived | Analogy | The framing of Analogue F. The 2-unique solver (`p2/analogue_f_construct.rs:1831`) is an order-2 context table `next_after[(b, a)]` whose exactness follows from the BWT's sort order. It runs only for `n <= 65_536` and rejects 12/12 real blocks |
| 2 | Bayes' theorem | 1763 | Survived | Decorative | No prior or posterior in the code |
| 3 | Gauss least squares | 1809 | Survived (law) | Relabel | An offline 4-coefficient regression (`spec/headroom_oracle_c47.rs:1713`, `:1841–1866`), printed; no coefficient reaches the selector |
| 4 | Laplace–De Moivre normal approximation | 1812 | Survived | Decorative | "Sets the 39,500 margin": the margin is a literal in `sprt_select_limit`; no variance is computed |
| 5 | Cauchy cycle decomposition | 1815 | Survived | Load-bearing (elementary) | `try_short_periodic` (`tot == n && x == p && n % d == 0`) and forward-FL multi-cycle tiling, passing the 338-case `#729` gate and 250 malformed columns |
| 6 | Galois coset decomposition (`TileRev`) | 1830s | Falsified | Relabel | Cache-blocked transpose; lost 1.46–1.60× in S4 |
| 7 | Dirichlet pigeonhole partition | 1830s | Survived (Specialist) | Relabel | Cycle 46 row 8 again |
| 8 | Kirchhoff matrix-tree theorem | 1847 | Survived | Decorative | Tree-count entropy, a diagnostic |
| 9 | Cayley permutation action, dihedral reversal | 1854 | Survived | Analogy | Solved Analogue G on Fibonacci words; on real blocks 99.97–100% of rows fail ρπρπ = id, and the existing XOR adjacency is used |
| 10 | Smith normal form | 1861 | Survived (certificate) | Decorative | Certifies the tower's last level is one directed cycle; `m_D = 1` is read off directly |
| 11 | Chebyshev–Markov inequality, Markov–Dirichlet moment bound | 1867 | Survived (key) | Relabel | `md_w1 = 2 (recs − ns) − m` is the count that a thin run of width ≥ 2 holds ≥ 2 rows. As implemented with capped records it is off by 2Δ₁₂₇ (the referee's correction). No tail bound in the code |
| 12 | Jacobi–Perron continued fraction | 1868 | Survived (law) | Decorative | A contrast figure in the FQ2 tower |
| 13 | Jordan centralizer, ℤ/2 involution | 1870 | Survived (baseline) | Relabel | Cycle 46 row 9 again |
| 14 | Hierholzer's Eulerian circuit | 1873 | Survived (Foundational) | Load-bearing (test harness only) | `generate_debruijn_hierholzer` (`p2/analogue_f_diagnose.rs:352`) builds the B(σ, k) and balanced Eulerian test texts. Its decoder uses (E4, F-HierFast: contracting degree-(1,1) chains, i.e. path compression) both lost |
| 15 | Frobenius–Stickelberger torsion | 1879 | Survived (law) | Decorative | A diagnostic of the FQ2 tower |
| 16 | Poincaré–Birkhoff–Kac sections | 1890–1947 | Survived | Relabel | `(m & 7) == 0` subsampling; `Σ|W| = n` is cycle 46 row 14's identity |
| 17 | Pareto–Zipf stratification (Law 1) | 1890s | Survived (law) | Relabel | The width histogram (41% of ribbon yield at `w = 2`, a real finding) |
| 18 | Minkowski convex envelope | 1890s | Survived | Relabel | Cycle 46 row 17 again |
| 19 | Flye Sainte-Marie line-graph doubling | 1894 | Survived (Promoted) | Analogy | "Motivates" F-Inline3; nothing in the packing uses line graphs (§4) |
| 20 | Frobenius companion reduction, sparse transpose | 1896 | Survived / falsified | Relabel | Ribbon rewiring (E4, lost) and a tiled transpose (G3a, lost) |
| 21 | Burnside–Miller orbit-stabiliser | 1897 | Survived (Foundational) | Relabel | E3's one-cache-line `sl[k] == e0 + k` check; G4's two forward cursors |
| 22 | Whitworth–Stevens random circle division | 1897 | Survived (law) | Load-bearing (performance model) | Expected longest arc H_K/K predicts the K-cursor walk's critical path (K = 2: 0.750 predicted, 0.725 and 0.886 measured). DualFwd wins on a 105 MiB-L3 Xeon and is blocked at 10 workers on the i9 by +879 KiB per worker |
| 23 | Thue repeated-factor stratification | 1906 | Survived (census) | Relabel | Ribbon census |
| 24 | König matching | 1916 | Survived / falsified | Relabel | The exact-match strip test `aw == bw` |
| 25 | Schur complement, Gaussian elimination | 1917 | Survived (baseline) | Analogy | Cycle 46 row 24 again |
| 26 | Menger connectivity | 1927 | Survived (diagnostic) | Relabel | Component census |
| 27 | Schreier coset graphs, transversals | 1927 | Survived (proved) | Analogy | A name for the FQ2 tower, built from Kruskal (1956) spanning-forest chords and iterated first returns; the obstruction rests on measured counts |
| 28 | Fisher linear discriminant | 1936 | Survived | Decorative | The rule is an axis-aligned box `recs <= 57_000 && ns <= 2_400 && md_w1 <= 26_000`, not a discriminant direction |
| 29 | Whitney line-graph isomorphism | 1932 | Survived (theory) | Decorative | No code |
| 30 | Neyman–Pearson lemma | 1933 | Survived | Decorative | No likelihood ratio in the code |
| 31 | Hall marriage, flush containment | 1935 | Falsified | Analogy | Zero-skip S4 lost +480 µs |
| 32 | BEST theorem | 1941 | Survived | Decorative | Entropy figure |
| 33 | Wald's sequential probability ratio test | 1945 | Survived (Promoted) | Relabel | E1 aborts after 64 probes with fewer than 4 hits (`p1/ribbon_kernel.rs`, `wald_window`, `wald_min_hits`); Spec4 takes two fixed samples (§4). No likelihood ratio, no Wald boundaries, no error rates |
| 34 | Birkhoff–von Neumann decomposition | 1946 | Falsified | Analogy | A 1-local purity test (35–40% max-entry mass) refuted a k = 1 shortcut |
| 35 | de Bruijn and Good sequences | 1946 | Survived (Promoted) | Relabel | F-Inline3 (§4) |

Several rows repeat across the cycles (Dirichlet, Cauchy, Minkowski, Schur, the XOR involution),
so the cumulative 66/102 counts some tools twice. Deduplicated, the nine load-bearing rows are
eight distinct results.

## 4. The two promoted candidates

### F-Inline3 (filed as de Bruijn 1946, Flye Sainte-Marie 1894): relabel

The change (`p2/analogue_f_construct.rs:159–188`, `:393–470`) widens the handle's payload field
from 22 to 24 bits. A word of length 1–3 then sits in the low 24 bits of the `u32` handle, its
length in the top 8 (`SATURATED = 255`), and the rule ids start at 0 instead of above an implicit
literal and bigram range (`RULE_BASE = 0`). A join whose total length is at most 3 ORs the two
payloads with a shift and writes no arena rule.

This is small-string inlining in a tagged handle. No shift register, de Bruijn graph or line
graph appears in the code; "order 3" means "3 bytes". Its correctness rests on two counting
facts master's `endpoint.rs:85–89` already states: every id is below 256 + 2 × 900,000, under
2²¹, and lengths at or above the saturated field fall back to the rule's stored length. Any
engineer looking at a 21-bit id bound in a 22-bit field would see the spare bits.

The engineering value is real and modest:

- arena rules −19.3% on the six Wikidata blocks and −13.6% on the six others;
- the S2a + S2b + S3a stages −5.6% and −4.2%, winning on all twelve blocks;
- in-harness wall on the 233 full blocks 0.971–0.990× against R5-Head;
- on `#729`, instructions 0.920 against R5-Head's 0.930, and L2 traffic 24.9 against
  25.5 B/B.

`#729` wall at W = 2 is 0.956 [0.935, 0.978], inside R5-Head's own 0.955 [0.930, 0.981]. So
the scorer shows no wall gain over the base it modifies. That base, R5-Head, was not adopted on
the i9: the 10-thread wall was 1.000 [0.983, 1.017]. DRAM traffic at W = 10 stays at 4.91 B/B.
Master still has `ID_BITS = 22`, so the change could be ported to master and measured on CI as
an ordinary construction PR. The historical label adds nothing to it.

### Spec4-SPRT-L (filed as Wald 1945, Laplace, Chebyshev–Markov, Gauss): relabel

`sprt_select_limit` (`spec/cand_c47_selector_moonshot.rs:177–222`) strides over the capped run
records. It takes 64 samples, then 256, and counts records of width 1 and widths 17–32. It
returns `L = 16` at once if the scaled width-1 estimate exceeds 39,500 or more than two of the
first 64 samples are in 17–32. Otherwise it picks 32, 24 or 16 from the 256-sample counts,
against 33,500 and 3 or 5.

That is a two-stage fixed-size sampling plan with hand-set integer cut-offs. It has none of an
SPRT's parts: no likelihood ratio, no boundaries from chosen error rates, no sequential stopping
beyond the single early exit. The Laplace margin and the Chebyshev bound do not appear in the
code. The least-squares fit runs offline and none of its coefficients is used. The post-S1 μ
rule it embeds (Spec3: Markov–Dirichlet, Neyman–Pearson, Fisher, Bayes) is the box in cycle 47
row 28 over a counting bound (row 11). Any engineer choosing a strip limit from a sample of the
run records would write this code.

The value is a cross-corpus selector, and it is unproven out of sample:

- 0.946–0.996× on the six non-Wikidata blocks at 1T, 2W and 4W (the referee's rerun 0.949–0.950×), winning on all six;
- 0.986× at 1T on the 431 Wikidata blocks, a tie at 2W;
- on `#729`, W = 1 0.950 [0.905, 0.997] and W = 2 0.960 [0.937, 0.983], inside R5-Head's own
  intervals.

The thresholds were chosen on the same six non-Wikidata blocks and 233 Wikidata blocks they are
scored on, so the "92.5% of the oracle" is in-sample. A held-out non-Wikidata corpus is the
missing test.

## 5. Pre-1950 results in shipped code on master

Searching `src/` for the results and the merged PRs' journals (#565's open return, #692, #703,
#705, #714/#719, #739) finds three pre-1950 results doing work in shipped code. All three
predate cycles 46–47.

| Result | Where it ships | Weight |
|---|---|---|
| First return / induced map (Poincaré 1890, Kakutani 1943) | The Open kernel: "the inverse BWT as the first return to a union of whole LF strips" (`src/open.rs:1–16`, `OpenCells`, since 0.5.7) | The kernel's construction. Reached through Rauzy–Veech induction (1979) and the Astra open-return line; the proof (`docs/journal/2026-11-18-open-return/proof.md`) is self-contained and cites neither |
| Euclid's division step and exponentiation by squaring (antiquity) | Spiral contraction: `q = (aw - 1) / bw` steps taken at once, the word by `power` (`src/open/endpoint.rs:357–380`; `CERTIFIED_WIDTH` at `:98–108`; #703) | A worst-case bound (at most `SPIRAL_RULES` rules a spiral). No Wikidata block fails the width certificate, so on the corpus it never runs |
| Cycle decomposition of a permutation (Cauchy 1815) | A short marked cycle repeated to `n` bytes (`src/open.rs:1029`, `repeat`) | Correctness on periodic and malformed columns; elementary |

The rest of the shipped machinery is post-1950 or engineering: Rauzy–Veech induction
(`src/cells.rs`, 1979/1982); the bzip2 format's Huffman (1952), CRC (1961), MTF and BWT
(1986, 1994); counting sort (1954); XOR-linked neighbours (`src/open/outq.rs`); OutQ16 queues
in `out`; the Endpoint-in-`edges` overlay (#692, buffer aliasing filed under Chaitin 1982);
Sattolo's shuffle (1986) in the `dose` instrument.

Christoffel and Sturmian words, the two pre-1950 entries among the blog's eleven landmark tools,
reach master only as test columns ("Euclid-like" columns in `src/cells_lazy_test.rs:117`) and in
analysis.

So the pre-1950 share that has produced shipped code is three results, all foundational
concepts with elementary proofs. That is consistent with the blog's 2/11 and does not move with
cycles 46–47.

## 6. Negative results worth keeping

These stand on measurements, whatever their labels:

- **The GUID wall.** Width-1 GUID rows are 53.6–67.1% of width-1 retained rows and 28.7% of
  emission on the six blocks. Random 122-bit ids leave no local shortcut without a format
  change.
- **The copy sector is real but too expensive to exploit.** Three independent mechanisms
  (A-Tube, U_iso/MK1D, LPTQ) cut retained hops by 16–58% on `sha1` and repeated text. All
  three lose 1.14–2.64× end to end, because acquisition costs more than emission saves.
- **The copy-ribbon cost barrier.** Ephemeral ribbons win on the synthetic Analogue E
  (0.80–0.84×) and lose on Wikidata (1.17–1.22×). An L2-resident trace costs 10.1–10.5 ns per
  saved hop, against the 5.79 ns/row the i9 emits at, and 57.5% of the yield is at widths 2–3.
- **S4 scatter is already hidden.** Tiled, zero-skip and write-combining S4 variants lose
  1.32–2.43× in S4, because `edges` fits in L2 and out-of-order execution overlaps the scatter.
- **DualFwd** is faster on a 105 MiB-L3 Xeon (0.94–0.96× at 1T) but needs +879 KiB per worker,
  past the i9's 20 MiB L3 at 10 workers.
- **The plateau.** Every construction win in cycles 36–47 leaves `#729`'s W = 10 DRAM traffic at
  4.91 B/B, consistent with R5-Head's flat 10-thread wall on the i9. Instruction cuts in
  construction do not reach production width.
- **FQ2.** Iterated first-return cutting ends at one cycle in depth 5–6. The second level blows
  up 11.9–31.8× in pieces, all of it geometric, so `Open<16>` is the only level with O(H)
  compression.
- **Per-block tuning headroom.** Tuning `(L, μ)` per block in hindsight saves only 3.17% on the
  431 Wikidata blocks, against 12.8–13.1% on the six non-Wikidata blocks (in-sample). Global
  `L = 4` loses 26.5%; the percolation `L_c ≈ 16` hypothesis is falsified (`L_c` = 112–9,628).
- **Two forward-FL bugs** were caught by the referee and the `#729` gate (76/338 failures from
  a backward-LF walk; a missing cycle-closure check). Both justify the gate.

## 7. A better metric

1. **Report three numbers, not one.**
   - *Search breadth:* the era and field spread of what was consulted. Keep it as a prompt for
     where to look; it is not a result.
   - *Causal contribution:* the tools whose removal breaks a kept artefact, each with the code
     line or proof step it supplies.
   - *Engineering value:* the measured end-to-end effect of each mechanism, on the
     authoritative bench, against the shipped base, with its interval.
2. **Make the tag pass the relabel test.** A tool is tagged with the date of the mathematics the
   code uses. A threshold, counter, sweep, histogram, regression, equality check or cache
   blocking is tagged "engineering", with no era. The referee applies the test, one row at a
   time, with the line of code beside it.
3. **Split "survived" four ways and count only new work.** The four are exact (byte-exact gate),
   law (an identity or count checked), faster (real corpus, interval excluding parity, against
   the shipped base) and shipped. Code already in the baseline earns the cycle no credit, and
   rows are deduplicated across cycles.
4. **Weigh, Weiss-style.** For each kept result, give the share of its measured gain, or of its
   proof's steps, that each tool carries. A result that rests on a threshold and a sample owes
   its classical tags nothing.
5. **Drop the quota as a gate.** A one-third pre-1950 rule rewards naming old results, and the
   cycle 47 share rose to 71.4% while its load-bearing share fell to 6.1%. Keep classical-first
   as advice for where to search, and gate on causal contribution and measured value. Score
   selectors on held-out corpora before they are promoted.
