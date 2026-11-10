# 2026-11-10: The addressed-width intervention: a registration, in draft (third draft)

**Status: a draft, revised on the Optimiser's REVISE (#145 6760521690) and the Researcher's
(6762028210). Not registered, not frozen, nothing built, nothing queued. Every number that is
not a count is a proposal or a scenario. The scenarios are written before any data of this
experiment exists; the archived share-nothing run (`share-nothing/20261110T052925Z`) is
their only input.**

- **The question:** at -9 the whole process's CPU grows 3.26 times from four workers to ten,
  at -2 1.18 times. Between the two files everything changes at once. On the -2 stream, does
  spreading the table's entries over four times the bytes change that growth?
- **The intervention:** the -2 file, unchanged. The same table, its entries 4 bytes or 16
  bytes apart: 0.8 or 3.2 MB addressed a full block.
- **The design** (the Optimiser's, 6760293462): 2 widths × workers 4 and 10 × marker on and
  off: 8 cells, 2 sentinels, 20 rounds and a warm-up: 210 timed slots; 252 with two -9
  reference cells. Untimed admission is additional (§9).
- **What it is:** a bounded descriptive diagnostic of one layout's total effect and its
  interaction with the worker count, on one stream. It identifies no mechanism, gives no
  share of -9's 3.26, and decides the fate of no table.

Labels: **archived** (from the bundle), **counted** (exact for its input), **scenario** (a
number that follows from stated assumptions, unobserved), **proposed**.

## 0. What the revisions changed

**Second draft, on the Optimiser's review:**

| the first draft said | now |
|---|---|
| "the walk's instructions unchanged" | the hot recurrence can match; the complete walk is unproved, and no instruction figure is given without code and listings (§3) |
| the build dirties 1.5 → 5.9 GB a decode | **0.84 → 3.36 GB**, from the row census (§2) |
| pages 200 → 800 | 196 → 782 a full table (197 → 783 with 64-byte alignment) |
| "below 1.15 … the premise under every compact table dies" | withdrawn (§6) |

**Third draft, on the Researcher's review:**

| the second draft said | now |
|---|---|
| estimands as logs of cells | the mean of within-round log contrasts, then exponentiated; not a log of pooled means (§4) |
| "B4 − A4 does not isolate the side effects" | pages, faults, traffic, reuse and occupancy are *part of the treatment*. They defeat a capacity-only reading, not the total effect (§3) |
| "a side effect common to B4 and B10 cancels in I" | only a common *factor* on the whole process's CPU cancels. An equal added cost does not: 0.5 CPU-s on both wide cells gives exp(I) = 0.941 (§4) |
| F0, F2, F1 as rival forms with predictions | four scenarios with their assumptions; none a prediction interval (§5) |
| an interval inside 2.8 to 4.4: "the literal law not excluded" | **reversed:** it excludes the literal point if its upper end is below it (§6) |
| bands with readings | set comparisons of one interval against stated numbers (§6) |
| a round's I between 0.11 and 0.15, ±5 to 6.5% | two sensitivity scenarios, not bounds; the analysis fixed as a conditional batch-t on five groups of four (§8) |
| the marker's contamination judged on I | the marker is descriptive whatever its contrasts show; reported on E4, E10 and I; one monotonic wall clock (§7) |
| schedule, timeout, invalidity "owed" | written (§9) |

## 1. What is asked

With the stream, the feed and the symbols fixed, what is the total effect on CPU of a table
addressed over four times the bytes, at four workers and at ten, and do the two differ?

## 2. The input and the cells

**The input, counted.** `wikidata-l2` reproduced from the pinned content with
`lbzip2 -2 -n 4` (lbzip2 2.5): SHA-256 `d3bcb141…a9323045`, the archived run's.
- 1,831 blocks, 209,706,556 rows.
- **1,048 blocks of 190,000 rows or more hold 99.9% of the rows** (the median block is
  200,000). The other 783 hold 0.1%: a quarter of all blocks have 5 rows or fewer.
- So the table is 0.8 or 3.2 MB for nearly every row walked, and some 780 near-empty blocks
  pay per-block costs only.

**The cells (proposed):**

| | workers 4 | workers 10 |
|---|---|---|
| 4-byte entries, marker off | A4 | A10 |
| 16-byte entries, marker off | B4 | B10 |
| 4-byte, marker on | A4m | A10m |
| 16-byte, marker on | B4m | B10m |

- **Arm:** plain workers, `Plan::Workers`, its window of 2 × workers slots, as archived and
  named in the record.
- **Sentinels:** the default plan on the original file, first and last in a round, as archived.
- **Optional references:** -9 at 4 and 10, marker on: occupancy description only.

## 3. The 16-byte entry: the treatment, whole

**The layout.** One `u32` array of k·n entries, k = 1 or 4. Row r's entry is at index k·r;
the indices between hold nothing and are never read or written. Every row is visited once a
walk, so every line of the 16-byte arm's 3.2 MB is touched: 4 entries a line against 16.
- **The link an entry stores is the scaled index j = k·r.** The hot recurrence is then the
  same in both arms: load at `base + 4j`, the byte from the low 8 bits, j from the shift,
  the prefetch at the new `base + 4j`. A full block's j is under 2²⁰, so bit 31 stays free.
- **Not a pointer to 16-byte records indexed by 4r:** that scales twice.
- **The logical n is kept** for every count, bound, length and output.

**What is not shown equal, and must be charged from code and listings:**
- the scaled index's arithmetic at the origin and at each take;
- the segment starts: drawn as rows, flagged at scaled indices, searched at a segment's end;
- bounds and asserts on n against bounds on k·n;
- the tails, the short-block path and the assembly;
- the build: the cursor's scaling, the column's byte read at the logical slot, the end check;
- walks of more than one cycle, which must keep production's meaning.

**No instruction figure is claimed here.** Before any timing (§9): inclusive counts, matched
between the arms, of the build, the whole walk and `decode_block`; and production's build
against arm A's one-row constructor, since A is not production (a 16-wide run store cannot
span four lines, so both arms take the one-row build from one source generic over k).

**The treatment is the layout with everything it brings:**

| | 4 bytes | 16 bytes | |
|---|---|---|---|
| lines addressed a full block | 12,500 | 50,000 | |
| line-bytes the build writes a decode | 0.84 GB | 3.36 GB | counted: rows × width |
| 4 KiB pages a full table | 196 | 782 | huge-page setting recorded |
| first touch a process | 196 pages a worker | 782 | every slot is a fresh process |
| the table against a 256 KB private cache | 3 times it | 12 times it | |
| the build's 256 write streams | a line a 16 rows | a line a 4 rows | |
| resident memory a worker | | + 2.4 MB | growth peak and reuse recorded |
| workers running at once | 5.9 of 10 archived | unknown: a slower worker raises it | reported a cell |

- **These are parts of the randomized treatment, not confounders of its total effect.** The
  estimands of §4 are the layout's total effects, whatever carries them.
- **They are what forbids reading the result as shared-cache capacity alone.** Round pairing
  subtracts none of them.
- **What can confound even the total effect:** order, time and load, which no cell is
  assigned. Hence the randomized order within a round (§9), every slot retained, the order
  recorded.

## 4. Estimands (proposed)

For each retained round r, from the marker-off cells' whole-process CPU:

    e4_r  = ln CPU(B4, r)  − ln CPU(A4, r)
    e10_r = ln CPU(B10, r) − ln CPU(A10, r)
    i_r   = e10_r − e4_r

- **E4, E10 and I are the arithmetic means of these over the 20 rounds;** their exponentials
  are geometric mean ratios. Not the log of a ratio of pooled means, which is another
  quantity. Frozen.
- **I is primary.** E4 and E10 are reported whatever I is; they carry the width's cost at
  each worker count.
- **Wall** the same way, secondary: never used to choose or rescue a CPU result.
- **Raw CPU and wall of every cell are reported** beside the contrasts.
- **What cancels in I, and what does not.** A common *factor* on the whole process's CPU
  cancels. An equal *added* cost does not: with the archived A cells (0.6825 s, 0.7929 s),
  0.5 CPU-s added to both wide cells gives exp(I) = 0.941. So exp(I) below 1 can sit with a
  large width cost at both worker counts, and a factor on the workers' CPU alone need not
  cancel either, since the scanner's share differs between the cells.
- **Not adjusted for overlap or occupancy:** the width changes them, so they are downstream
  of the treatment.

## 5. Four scenarios for exp(I) (scenario; from the archived 4-byte cells)

Archived: CPU 0.6825 s at four workers and 0.7929 at ten; scanner and main 0.10 and 0.11;
workers 0.582 and 0.683; 3.64 and 5.92 workers running at once. f = 0.78. X is the old
curve, which the archived run has already failed on the whole decoder at -9
(`hypothesis-register.md` §1a). Each scenario adds the assumption that the layout's other
costs are a common factor on the workers' CPU.

| scenario | its assumptions | exp(I) |
|---|---|--:|
| S0, no interaction | the layout's total cost is the same factor at four workers and at ten | **1.00** |
| S2a, occupancy | the old X on tables in use; as many running at ten as in the archived 4-byte arm (5.92) | **1.41** |
| S2b, occupancy | the same; as many running at ten as at -9 (8.1) | **2.84** |
| S1, literal | the old X on every requested worker's table | **4.41** |

(The Researcher's recomputation from this table's rounded inputs: 1.416, 2.855, 4.414.)

- **S0 is not "no table term".** A layout whose traffic, page or fault costs grow with workers
  gives an interaction with no shared-cache term at all; and no interaction does not mean the
  table is free.
- **S2a and S2b are two points, not the ends of a prediction interval.** The width changes how
  many workers run at once; nothing bounds the 16-byte arm's count by 5.92 and 8.1. The
  occupancy idea is not refuted by a result outside them.
- **S1 is one number.** No tolerance is claimed for it: its inputs' uncertainty is not
  quantified, so excluding 4.41 excludes that number and no more.
- **The marker-on cells' occupancy is not to be put back into the old curve afterwards** and
  called a prediction.

## 6. How the interval is compared (proposed)

Let C = [L, U] be §8's conditional interval for exp(I), with its assumptions attached to
every statement below.

**Against the practical threshold of 15%:**
- U < 1.15: an interaction smaller than 15%. It says nothing about the other direction.
- C inside [1 ÷ 1.15, 1.15]: an interaction negligible both ways.
- L > 1.15: an interaction larger than 15%.
- Otherwise the 15% question is unresolved.

**Against each scenario's number p (1.00, 1.41, 2.84, 4.41):**
- p outside C: that number is excluded at the interval's level. Under its assumptions only.
- p inside C: not excluded. Never "confirmed".
- C disjoint from [1.41, 2.84]: both occupancy points are excluded; the occupancy idea is not.
- **C wholly between 2.84 and 4.41, with U < 4.41: the literal point is excluded too.**
- An end equal to a boundary counts as inside it. No interval is rounded across a boundary.

**What is never concluded:**
- a mechanism: an interval compatible with a scenario does not identify it;
- anything about compact tables in general, another stream, or -9's own inflation;
- a candidate's standing: D4-delta's F2, the chunked link's missing kernel and compact30's
  unmeasured g are where they were.

**The whole interval is reported,** with E4 and E10 and theirs. exp(E4) is a statement of the
layout's cost at four workers and nothing else: it separates no causes, and no analysis is
conditioned on it.

## 7. The marker (descriptive, always)

Per block, five edges: MTF start, build start, walk start, walk end, decode end. 9,155
reads a -2 decode, some 780 of the blocks near-empty.
- **One clock: a monotonic wall clock common to every thread.** A thread's CPU clock stops
  while the thread is not running and cannot place two workers' envelopes against each other.
- Private preallocated records, no shared written line; an overflow or a missing edge makes
  the slot invalid (§9).
- **What an envelope is:** an interval in which a worker is between two edges. Not the worker
  executing (it may be descheduled), not a table's lifetime, not cache residency.
- **Outputs a cell:** the time at each count of active build envelopes and of active walk
  envelopes, each envelope's share of the decode, workers running at once (CPU over wall),
  with their round-to-round spread.
- **The marker's effect:** E4, E10 and I from the marker-on cells, and each one's difference
  from the marker-off one, with value and conditional interval. No pass or fail is attached:
  agreement does not show the marker is harmless, and an effect on E4 and E10 can cancel in I.
- **The trace stays descriptive whatever those differences are.** No regression of time on
  overlap, no adjustment of §4 by it, no retrospective fit of any curve.

## 8. Uncertainty: fixed before data (proposed)

**20 retained rounds after one warm-up. No extension and no retry for precision.**

**The one analysis, for I (and, as description, E4, E10 and the marker's contrasts):**
1. The 20 retained rounds in chronological order, in five consecutive groups of four.
2. The five group means; their sample standard deviation s_g.
3. **The conditional batch-t interval:** the mean ± 2.776 × s_g ÷ √5 (t at 0.975, 4 degrees
   of freedom), exponentiated.
- **Its condition, stated wherever it is used:** exact only if the five group means are
  independent and Gaussian with one mean and one variance. Four rounds a group do not show
  that, and five groups can barely test it. It is no certificate against a long drift.
- **Sensitivity only, never chosen between:** the ordinary round-t interval (19 degrees of
  freedom) and the chronological plot of i_r, e4_r, e10_r. If they disagree with the batch-t,
  both are shown and the batch-t is the one compared in §6.
- **No bootstrap is offered as assumption-free.** Twenty rounds in blocks of four are about
  five blocks of information however many resamples are drawn.
- **Only I carries a threshold statement.** The others are descriptive intervals; no
  simultaneous claim is made across them.
- **Language:** every §6 statement is conditional on the batch-t's assumptions and is written
  so. If a calibrated exclusion were required, a dependence model or a randomization analysis
  would have to be justified and frozen first; this diagnostic does not attempt it.

**What precision to expect (scenario; not a bound):** if a round's i_r had a standard
deviation of 0.11 or 0.15 and rounds were independent, the round-t half-width would be 0.052
or 0.070 in log. Neither figure is established: two growths each of 0.105 can differ with a
standard deviation anywhere from 0 to 0.21, and the wide arm's own spread is unknown. With
rounds correlated at 0.5 a lag, the mean's variance is 2.8 times the independent one (the
Researcher's counterexample); the batch-t is the guard against that, within its condition.

## 9. Rules (proposed; the archived run's where not said otherwise)

**Untimed admission, before any eligible run and additional to the timed slots:**
- both arms' bytes equal `bwt::walk`'s and the blocks' CRCs on every block of the file, and
  on crafted columns with several cycles;
- §3's matched inclusive counts, and the listings read;
- the kernel's `unsafe` reviewed (B5);
- a dry decode with the marker on: every block's five edges present and ordered.

**The preflight:** the archived run's, unchanged (`2026-11-10-share-nothing-pricing.md` §2).

**The schedule:**
- a round: sentinel, the eight cells in a drawn order, sentinel: 10 slots (12 with the -9
  references);
- the order drawn once for every round, before data, from a seeded generator whose seed is a
  registered constant; the schedule's SHA-256 registered and checked by the judge;
- 1 warm-up round, run and recorded, never analysed; 20 measured rounds: 210 slots (252);
- one `ab run` a slot with the archived flags (settle 0.25 s, the busy gate at 1.5 cores, the
  sysfs state read before and after), unpinned as archived;
- each slot's position in its round, start and end recorded.

**Deadlines:** one slot 120 s with 10 s grace, as archived; the drive 3,600 s with 30 s.

**Invalidity, decided now:**
- a slot that fails, passes its deadline, decodes other than 209,700,196 bytes with the
  registered hash, or (marker on) leaves a trace with an overflow or a missing or unordered
  edge: the drive stops, the slot is recorded and not retried, **the attempt is INVALID**;
- the archived judge's admission in full (identity, sources, the executable's digest before
  and after, corpora, the preflight replayed, telemetry with explicit missingness): any item
  missing or disagreeing: INVALID;
- **no complete-rounds subset:** a missing or invalid cell voids the attempt, never the round;
- **no exclusion by outcome:** no round or slot is dropped for its sentinels, its mode, its
  clock or its value; the mode split is description, as archived.

**Every attempt is kept.** VALID, INVALID and REFUSED attempts are archived with their
outcome. A run after an INVALID one is a new attempt, registered as such, the earlier bundle
cited beside it.

**Not proposed:** a 32-byte arm, until the 16-byte one has been read.

## Reproduce (the row census)

```text
bzip2 -dc corpora/wikidata.bz2 > content          # SHA-256 3f075e30…
lbzip2 -2 -n 4 -c content > wikidata-l2.bz2       # SHA-256 d3bcb141…a9323045
# rows a block: Scanner, then mtf::mtf, the column's length; 1,831 blocks, 209,706,556 rows
```
