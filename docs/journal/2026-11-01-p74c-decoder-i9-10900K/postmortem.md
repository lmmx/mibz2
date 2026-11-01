# P74-C i9-10900K run: read-only postmortem of the archive

**What this is, and isn't:**
- It reads only the committed archive in this directory (`raw.tsv.gz`, `summary.json`)
  and the pre-trial RSS note (`../2026-10-31-p74c-decoder-rss-note.md`).
- Nothing was rerun, rebuilt or re-timed.
- **The registered verdict, NEGATIVE, stands, and nothing here re-decides it.**
  - The figures below are descriptive.
  - The intervals marked "plain t" are two-sided 95% t-intervals on the paired logs,
    without the registered ρ₁ correction. They are not gate figures.
  - The registered figures are in `results.md`.
- **Status:** P74-C is archived at decoder level. No rerun, variant, tuning or
  threshold change follows from this.

Arms: A is the shipped decoder, B is P74-C, A′ is a second run of A (the A/A null).
Each figure has 20 paired rounds per cell. Throughput is bytes over wall time; RSS is the
child's `ru_maxrss`.

## 1. Peak RSS: the NEGATIVE is the pre-registered 32-byte mechanism

**The deciding row:** `mixed-independent` (2.0 MB, 3 blocks, of which 2 bypass P74 and
1 runs P74-C).

| w | B − A peak RSS, median | Rounds with B > A | B − A minor faults, median | A max | B max | A median |
|--:|--:|--:|--:|--:|--:|--:|
| 1 | +160 kB | 19 / 20 | +220 pages (≈ 880 kB) | 20,172 | 20,332 | 20,012 |
| 2 | **+876 kB** | **19 / 20** | +171 pages (≈ 684 kB) | 26,312 | 25,512 | 24,476 |
| 3 | −2 kB | 7 / 20 | 0 | 24,832 | 24,864 | 24,700 |
| 4 | −6 kB | 8 / 20 | 0 | 24,944 | 24,844 | 24,608 |

*(Measured; RSS in kB.)*

**It matches the mechanism the pre-trial note stated and tested before the run:**
- **The w = 2 offset:** +876 kB is one 900,000-byte `ll` buffer (879 KiB). The note
  measured +0.83 MB locally.
- **The w = 1 offset:** +160 kB, against the note's +0.16 MB.
- **It is absent at w ≥ 3.** With 3 blocks over 3 or more workers, no worker decodes a
  second block, so no worker meets a bypassed block followed by a P74-C block.
- **The note's intervention:** a scratch build in which both kernels reserved `max + 32`
  cut the excess to +0.24 MB at w = 2 and to 0 at w = 1. So the excess comes from
  integration, not from the P74 algorithm or its selector.

**Why the two RSS rules disagree (0.970 against a paired 1.030):**
- A's w = 2 maximum, 26,312 kB, is a single round-13 outlier, 1.8 MB above A's median.
- The max-over-rounds rule compares that outlier with B's maximum, and reads B as
  smaller.
- The paired rule sees the +876 kB offset that recurs in 19 of 20 rounds.
- The NEGATIVE is the paired rule doing its job, not a statistical artefact.

**The other RSS flags:**
- **`random` w = 4,** max rule 1.035: one B outlier of 66,820 kB against B's other
  rounds. The median B − A is −58 kB, and B > A in only 9 of 20 rounds.
  - P74 never runs on `random`: all 9 blocks bypass, as table 2 shows.
  - So this is noise; the kernel can't cause it.
- **`text` w = 3,** max rule 1.028: the median B − A is 0, and B > A in 5 of 20 rounds.
  This is also noise.

**On Wikidata, the realloc happens but is diluted:**
- B − A minor faults are +130 to +780 pages in every Wikidata cell. Wikidata has 204
  bypassed and 230 P74-C blocks interleaved, so the bypass → P74-C reuse happens.
- Peak RSS doesn't move beyond noise: the median B − A is between −418 and +524 kB, on a
  250–315 MB peak.
- **Minor faults are a more sensitive detector of this defect than peak RSS.**

## 2. Corpus size, P74 engagement and the regressions

The kernel selections per corpus come from the run's selection counts. The figures are
the B/A throughput point estimates (geometric means of the paired ratios).

| Corpus | MB | Blocks | P74-C blocks | w1 | w2 | w3 | w4 |
|---|--:|--:|--:|--:|--:|--:|--:|
| hug-* (5) | 0.4 | 1 | 1 (100%) | 1.11–1.13 | 1.10–1.16 | 1.12–1.14 | 1.11–1.15 |
| rank-* (10) | 0.1 | 1 | 1 (100%) | 1.06–1.11 | 1.06–1.12 | 1.00–1.10 | 1.03–1.09 |
| split-q75, q90 | 0.4–0.5 | 1 | 1 | 1.06–1.07 | 1.04–1.06 | 1.03–1.06 | 1.05–1.07 |
| split-q05–q50 | 0.6–0.8 | 1 | 1 | 0.99–1.02 | 0.99–1.02 | 0.97–1.03 | 1.00–1.03 |
| text | 12.3 | 13 | 13 (100%) | 1.057 | 1.056 | 1.031 | 1.012 |
| wikidata-20261026 | 207 | 434 | 230 (53%) | 1.043 | 1.017 | 1.022 | **1.036** |
| wikidata-fitted | 210 | 431 | 233 (54%) | 1.032 | 1.021 | 1.018 | 1.013 |
| wikidata-heldout | 236 | 473 | 262 (55%) | 1.031 | 1.011 | 1.018 | 1.019 |
| mixed-independent | 2.0 | 3 | 1 (33%) | 0.983 | 0.995 | 1.005 | 1.010 |
| random | 8.0 | 9 | **0** | 0.995 | 1.003 | 0.999 | **0.987** |
| records | 9.6 | 11 | **0** | 1.007 | 1.000 | 1.000 | **0.972** |

*(Measured. Ranges cover the corpora in each family.)*

**What it shows (deduced):**
- **The gain follows engagement and the block's run structure, not corpus size.**
  - The single-block crafted corpora, which have the short-run, low-rank MTF structure
    P74 targets, gain 4–16% at every w.
  - Wikidata, with P74-C on about 54% of its blocks, gains 1–4%.
  - The decoder-level gain is real and measurable. It is too small at w = 4 to clear the
    registered 1.02 lower bound.
- **The two control regressions that made gate 3 INCONCLUSIVE cannot be the kernel's.**
  - P74-C is selected on 0 of 9 `random` blocks and 0 of 11 `records` blocks. On those
    corpora B runs the shipped kernel, apart from the selector's bypass test.
  - Their w = 4 point estimates, 0.987 and 0.972, appear only at w = 4. At w = 1–3 they
    are 0.995–1.007.
  - A′/A for the same cells is 0.993 [0.977, 1.009] and 0.991 [0.972, 1.011] (plain t).
    These are A/A excursions of the same order.
  - Their intervals include 1.00 (plain t: `records` [0.938, 1.007], `random`
    [0.969, 1.005]).
  - Reading: these are w = 4 noise on small corpora, where B/A and A′/A move the same
    way. They aren't a cost of the bypass test.
- **The CPU-s/GB INCONCLUSIVEs (`random` 1.001, `records` 1.006) are on the same two
  fully bypassed corpora.** The same reading applies.

## 3. Three workers against four

**Wikidata scaling (A, median MB/s):**

| | w1 | w2 | w3 | w4 | w4 / w3 |
|---|--:|--:|--:|--:|--:|
| wikidata-20261026 | 353 | 666 | 759 | 781 | 1.03 |
| wikidata-fitted | 360 | 672 | 764 | 800 | 1.05 |
| wikidata-heldout | 375 | 694 | 768 | 819 | 1.07 |

*(Measured.)*

**Small corpora (w4 / w3 median throughput, each arm):**
- `text`: 0.95, 0.91, 0.95 (A, B, A′).
- `random`: 0.98 for each arm.
- `records`: 0.96–0.97.

**CPU cost:** A's CPU-s/GB rises from w = 3 to w = 4: Wikidata 3.69 → 4.00 (+8%), text
6.26 → 6.71.

**Reading (deduced):**
- **w = 4 is a saturated regime on this allocation.** The four workers share CPUs 1–4
  with the scanner and feeder threads. Wikidata gains only 3–7% from the fourth worker;
  the small corpora lose throughput from it.
- **The cost of that contention is noise.** Every arm sees it: A, B and A′ alike.

**The primary cell, wikidata-20261026 at w = 4, per round:**
- **Round 1 is an outlier.** A's wall time is 304 ms, against a median of 265 ms. That
  round's B/A is 1.193 and its A′/A is 1.127.
- **B/A and A′/A are correlated across rounds,** at r = 0.62. They share the denominator,
  so a slow A round inflates both.
- **There is early-round drift:**
  - corr(round, B/A log) = −0.42;
  - corr(round, A′/A log) = −0.32.
- **Without round 1** (descriptive only, not a registered exclusion):
  - B/A is 1.028 [1.009, 1.047];
  - A′/A is 1.019 [1.003, 1.036] (plain t).
- **Against the other control arm,** B/A′ at w = 4 is 1.011 [0.992, 1.030] (plain t),
  while B/A′ at w = 3 is 1.028 [1.011, 1.045].
- **Reading:**
  - The registered A/A row that excluded 1 here (1.025) signals real excess variance at
    w = 4.
  - The primary 1.036 is partly A-arm noise. The underlying w = 4 gain is likely about
    1–3%: the same band as the fitted and held-out sets, 1.013 and 1.019.
  - The primary was registered at w = 4, the noisiest and most saturated cell. At
    w = 3, B/A is 1.022 [1.010, 1.034] (plain t) on a cell whose A′/A is
    0.994 [0.976, 1.013].
  - This is recorded as a lesson for choosing future primaries. It is not a reason to
    re-decide P74-C.

## 4. Conclusions

1. **The RSS NEGATIVE is fully explained** by the pre-registered integration defect, the
   32-byte `ll` reservation mismatch. It is not the algorithm's.
   - The evidence: the +876 kB offset in 19 of 20 rounds, its absence at w ≥ 3, and the
     note's intervention.
   - Fixing it is a production change and a new project. See
     `../2026-11-01-buffer-capacity-contract.md`.
2. **The control and CPU INCONCLUSIVEs** fall on corpora where P74-C never runs, only at
   w = 4, with A/A excursions of the same size. They are not evidence of a kernel cost.
3. **The primary INCONCLUSIVE** reflects a real but small (≈ 1–4%) decoder-level gain on
   Wikidata, measured in the noisiest, most contended cell, with round-1 and early-round
   drift.
4. **The meaningful decoder-level gains** are 11–16% on the hug corpora and 6–11% on the
   rank corpora, at every w.
5. **None of this licenses a rerun.** The registered outcome is NEGATIVE, and P74-C
   stays archived.

**Lessons for future registered decoder trials, recorded here, not applied to P74-C:**
- **Check buffer-capacity contracts across kernels before timing.** Assert that a reused
  buffer's capacity is unchanged across an alternation of kernels.
- **Report minor faults beside peak RSS.**
- **Weigh a primary at the saturation knee (w = 4 here) against its A/A variance** when
  choosing it.
- **Use the paired RSS rule.** It caught a real offset that the max rule missed.
