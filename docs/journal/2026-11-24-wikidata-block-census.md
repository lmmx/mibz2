# A census of Wikidata's block structure against the Open kernel's per-block cost: ADOPT AS SPEC (construct)

*24 November 2026. A C1a diagnostic, decided by its rule fixed before the data: #145 LOG
7126445058, with the coordinator's amendments posted before the fit (the split by block identity,
the 10-over-1 contention row and the manipulability note; the m-only baseline and the 0.05 entry
bar). Costs: `emit-cost-sq/run-20261123T110350Z` on `evidence` at `a01d1f9c`, SHA-256
`cc1d465722c5cfbc80f34f04e8630a73103ac092f174a7e20ed7bf02f718a64d` (checked), base arm, 1 worker.
Features: `examples/census.rs` (feature `census`, `src/census.rs`) on wikidata ×4, SHA-256
`0b820d14…c89ce08e`, the bundle's own `inputs.sha256`. Fit: `bench/census_fit.py`. Data and
results: `docs/journal/2026-11-24-wikidata-block-census/` (`census.tsv`, `features.tsv`,
`fit.json`, `fit.txt`).*

## Why, by the book

Bach, Black, Huiberts and Kafer, *Beyond Smoothed Analysis: Analyzing the Simplex Method by the
Book* (arXiv 2510.21613), analyse an algorithm under input assumptions read off real instances
and real implementations, rather than under a worst case or a random model chosen for
tractability. The T² line taught the same thing (record 95): its synthetic pre-screen, Haar-random
cycles, said ALIVE, and the real emit said KILL. So before any further synthetic pre-screen, this
census measures which cheap properties of a real Wikidata block predict the Open kernel's
per-block cost on the i9. If a model of at most three of them predicts the cost held out, those
properties become the input specification a generator must match. If none does, pre-screens run
on real decoded blocks.

## The census

For each block the Open kernel takes (`OpenCells`' `try_run` returning `Some`), an O(n) pass
beside the kernel computes, from the block's run records and marked row alone:

- **m**, the retained rows, classified as `open::split` does at `LIMIT` 16: rows of maximal runs
  of at most 16 rows, and the marked row when it falls in a thick run;
- **the maximal runs' length histogram**: runs, and runs and rows of length 1, 2–3, 4–16,
  17–127 and 128+;
- **the strips**: the thick pieces, the marked run's two fragments counted apart;
- **the cycles of the retained map T** (cycles of LF that hold a retained row), and the retained
  rows on the marked cycle;
- **the locality** d = |T(s) − s| in retained ranks, the index the emit reads `edges` at: the
  shares below 16, 1,024, 8,192 and 65,536 (a 64-byte line, a 4 KiB page, and 32 KiB and
  256 KiB of 4-byte entries, the i9-10900K's L1d and L2), the mean log₂(1 + d), the mean d / m,
  the median and the 90th percentile.

The census's m equals the kernel's own retained count on every block, and its unit test checks
it, and T's single cycle, against `ForcedOpenCells` on a synthetic block.

**The join check passes.** Decoded at 1 worker with production's lazy workers, as the bench's
1-worker cells were, wikidata ×4 gives 932 open blocks, and the census's (n, m) equal the
bundle's 1-worker base records' block by block, in each of the 20 rounds.

**Identity.** Blocks are grouped by a hash of their run records, n and p. There are **233
distinct blocks, each exactly 4 times** (the 4 copies), so the effective sample is 233 blocks,
232 of them two-ended. The 4 blocks emitted forward (one block ×4, m = 65,112 < 65,536) are left
out; 928 blocks remain.

**The split.** Groups are numbered by first appearance; even groups fit (468 blocks, 117 groups),
odd groups scored (460 blocks, 115 groups). Every copy of a block is on one side.

## The distributions

Over the 232 distinct two-ended blocks (min, p10, median, p90, max):

| feature | min | p10 | median | p90 | max |
|---|--:|--:|--:|--:|--:|
| m | 68,411 | 87,957 | 102,142 | 168,897 | 231,600 |
| m / n | 0.076 | 0.098 | 0.114 | 0.188 | 0.257 |
| maximal runs | 47,936 | 51,104 | 56,079 | 99,429 | 179,972 |
| mean run, rows (n / runs) | 5.0 | 9.1 | 16.0 | 17.6 | 18.8 |
| rows in runs of 1–3 | 0.051 | 0.056 | 0.063 | 0.117 | 0.215 |
| rows in runs of 1 | 0.037 | 0.039 | 0.043 | 0.082 | 0.183 |
| rows in runs of 4–16 | 0.007 | 0.035 | 0.051 | 0.078 | 0.105 |
| rows in runs of 17–127 | 0.019 | 0.077 | 0.115 | 0.138 | 0.198 |
| rows in runs of 128+ | 0.620 | 0.704 | 0.769 | 0.796 | 0.859 |
| strips | 568 | 2,242 | 3,439 | 3,916 | 5,264 |
| strips / m | 0.0029 | 0.0138 | 0.0341 | 0.0412 | 0.0532 |
| cycles of T | 1 | 1 | 1 | 1 | 1 |
| d median | 13,931 | 18,272 | 20,415 | 30,520 | 49,104 |
| d p90 | 39,884 | 54,075 | 66,918 | 106,275 | 152,497 |
| d < 16 | 0.0001 | 0.0004 | 0.0007 | 0.0010 | 0.0013 |
| d < 1,024 | 0.018 | 0.031 | 0.052 | 0.061 | 0.070 |
| d < 8,192 | 0.134 | 0.185 | 0.286 | 0.307 | 0.362 |
| d < 65,536 | 0.617 | 0.749 | 0.896 | 0.933 | 0.997 |
| mean d / m | 0.246 | 0.261 | 0.268 | 0.275 | 0.290 |
| emit, ns a row (1 worker) | 4.31 | 4.61 | 4.83 | 5.09 | 5.48 |
| construct, ns a row (1 worker) | 3.72 | 4.77 | 5.90 | 6.40 | 7.29 |

Two things stand out. **T is one cycle through all m retained rows on every block**, as LF is
one cycle on a primitive block, so its cycle structure carries no information here (both cycle
features were constant and dropped). And **the locality is nearly that of a random cycle**: mean
d / m is 0.25–0.29 (a uniform random permutation gives 1/3), and only 0.07% of steps land within
a cache line. What separates blocks is not the shape of T but how much of the column the thin
rows are, and how the thick runs are cut into strips: the blocks fall into a main mode (m about
0.1n, mean run 16 rows) and a tail of denser blocks (m up to 0.26n, mean run down to 5 rows).

## The fit

**Targets**: per block, the base arm's emit and construct ns per retained row at 1 worker, each
the median over the 20 rounds.

**The search**: 22 candidate features (the 24 above, transformed as listed in
`bench/census_fit.py`: log for counts and scales, shares raw; the 2 constant ones dropped), every
subset of 1, 2 or 3: **1,793 models a target**, ordinary least squares with an intercept on the
fit half. **The correction for searching**: the model is chosen on the fit half alone, by 5-fold
cross-validation by group within it, and only that model's held-out R² is judged. The best
held-out R² over all 1,793 is reported too, against a permutation null of that maximum (200
permutations of the group-level cost vectors among groups, the whole search and choice repeated).

| target | chosen model (≤ 3) | CV R², fit half | **held-out R²** | m-only (m, log m) | gain over m-only | best of 1,793 held out | null max, p95 | ceiling |
|---|---|--:|--:|--:|--:|--:|--:|--:|
| construct | log mean run, strips / m, log d median | 0.953 | **0.964** | 0.524 | +0.440 | 0.964 (the same) | 0.051 | 0.998 |
| emit | rows in runs 4–16, log strips, log d median | 0.710 | **0.656** | 0.349 | +0.307 | 0.667 (strips / m, d < 1,024, d < 8,192) | 0.057 | 0.984 |

The ceiling is the held-out R² of each scored block's own mean over its 4 copies: the most any
function of a block's features could reach. The null's chosen model scores about 0 held out at its 95th percentile (−0.003 construct, 0.003
emit); the observed fits are far outside it (p = 1/201,
the smallest 200 permutations give). The construct model is
construct ns/row = 23.8 − 1.54 · ln(n / runs) + 65.6 · strips / m − 1.60 · ln(1 + d median).

**Gains over m-only, by feature** (m, log m and the one feature; the amendment's entry bar is
+0.05): for construct, strips / m +0.399, rows in runs of 17–127 +0.328, log strips +0.271,
run-length entropy +0.242, log mean run +0.082, d < 1,024 +0.080, d < 16 +0.054; the locality
summaries other than these add < 0.02, and log d median, though in the chosen model, adds
−0.008 alone. For emit, strips / m +0.213, rows in runs 17–127 +0.119, d < 1,024 +0.099, log mean
run +0.098, rows in runs 1–3 +0.094.

## The verdict under the rule

**ADOPT AS SPEC, on construct.** The chosen model reaches held-out R² 0.964 ≥ 0.7 with three
features. m alone does not clear 0.7 (0.524), so by the amendment the spec is m together with
each feature of the chosen model that adds at least 0.05 over m-only: **m, the mean maximal run
length (n / runs, equivalently the maximal runs, n being 900,000 to within 0.03%) and the strips
per retained row** (the spec's own model, m, log m and those two, scores 0.958 held out). The
median locality, the chosen model's third feature, does not enter (−0.008 alone over m-only).

Emit does not reach 0.7 (0.656 chosen, 0.667 at best of the 1,793), although its ceiling is 0.98:
emit cost is a stable property of the block, but no three of these summaries capture it
linearly. Under the rule emit decides nothing; it is reported.

**What a pre-screen generator must match** (for a construct-side pre-screen): the joint
distribution of m (68k–232k, median 102k, with the dense tail above 0.18n), the mean maximal run
(5–19 rows, median 16) and strips / m (0.003–0.053, median 0.034), within a tolerance its
registration states, for instance each feature's quantiles above within 10% and the three
pairwise rank correlations within 0.1. For an emit-side pre-screen the census qualifies no
generator: emit pre-screens run on real decoded blocks. A generator that draws T as a uniform
random cycle gets the locality roughly right (mean d / m 0.27 against 1/3) but nothing in this
census says that is enough for emit.

## Reported rows that decide nothing

**Contention growth.** Per distinct block, emit and construct ns per row at 10 workers over 1
(medians over copies and rounds; joined by (n, m), unique for 230 of the 232 distinct two-ended
blocks, the other 2 left out), with the same split and search. Emit grows by a median 1.12×
(p10 1.11, p90 1.14), construct by 1.12× (1.11, 1.18). Neither is predicted: held-out R² 0.002
(emit) and 0.023 (construct) for the chosen models, m-only −0.003 and 0.038, the best of 1,793
0.029 and 0.074. The growth from 1 to 10 workers is a near-constant factor across blocks; it is a
property of the machine's contention, not of the block.

**h₁, the width-1 runs** (MONODROMY record 96: G1's D ≥ h₁ on rigid jump-walk routes, met
exactly by Open). **Added after the fit**, at the coordinator's request, so it is a reported row
and not in the registered search; its share of rows, h₁ / n, was in the search as "rows in runs
of 1". h₁ (maximal runs of one row; the marked row's singleton in a thick run is not counted)
runs 33,256–165,091 (median 38,960), h₁ / m 0.30–0.83 (median 0.39), correlation with m 0.85.
Held out, alone and as a gain over m-only: on emit, h₁ 0.419 and +0.091, log h₁ 0.412 and
+0.060, h₁ / m 0.030 and +0.061; on construct, h₁ 0.186 and +0.011, log h₁ 0.199 and +0.035,
h₁ / m 0.035 and +0.024. h₁ adds little to construct beyond m; on emit it adds about as much as
the other run-length summaries, short of 0.7.

**Manipulable or only correlated.** The run-length histogram (and with it m, the mean run, h₁ and
the strips) is a property of the BWT column that an experiment can set directly: a generator can
write a column with a chosen histogram, or a real block's thick runs can be re-cut (LIMIT moved,
strips split) at fixed text, so their effect on construct can be tested causally. The locality of
T and its cycle structure are consequences of the text, not of the column's histogram alone; they
can be varied only by permuting the retained ranks (a relabelling of `edges`, which changes the
emit's memory pattern and nothing else), a valid intervention for emit's locality but not one a
text generator controls. The 10-worker growth is a property of the machine and its co-runners, a
correlate of the block at best. The fitted construct model is a correlation on 232 real blocks:
strips / m enters because the construction does per-strip work and per-run work divided over m
retained rows, which is the mechanism a causal test would check.

## Caveats

- One corpus, 232 distinct blocks from 12 windows of one dump; the dense tail is a few dozen
  blocks. A construct spec matched on Wikidata says nothing about other inputs.
- One machine (the i9-10900K), one toolchain, 1 worker. The costs are medians of 20 rounds; the
  4 copies of a block share features, so per-block noise is averaged only in the ceiling.
- The features were computed here (a shared 4-core machine), the costs on the i9; the join is
  exact by (n, m) in order and by the corpus's SHA-256, not by a timestamp.
- The permutation null has 200 draws, so p ≥ 1/201.
- The search space (22 features, linear, ≤ 3) is the one stated; a nonlinear model of the same
  features might reach 0.7 on emit. That would be a new registration.
