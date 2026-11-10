# 2026-11-10: The addressed-width intervention: a registration, in draft (second draft)

**Status: a draft for the Researcher's review, revised on the Optimiser's REVISE (#145
6760521690). Not registered, not frozen, nothing built, nothing queued. Every threshold here
is a proposal. Predictions are written before any data of this experiment exists; the archived
share-nothing run (`share-nothing/20261110T052925Z`) is their only input.**

- **The question:** at -9 the workers' CPU grows 3.26 times from four workers to ten, at -2
  1.18 times. Between the two files everything changes at once: the table, every other buffer
  a block owns, the block count, the feed. On the -2 stream, does quadrupling the bytes the
  table is addressed over change that growth?
- **The intervention:** the -2 file, unchanged. The same table, its entries 4 bytes or 16
  bytes apart: 0.8 or 3.2 MB addressed a full block. Nothing else about a block is meant to
  change, and §3 lists what does.
- **The design** (the Optimiser's, 6760293462): 2 widths × workers 4 and 10 × marker on and
  off: 8 cells, 2 sentinels, 20 rounds and a warm-up: 210 timed slots; 252 with two -9
  reference cells. Untimed admission is additional (§9).
- **What it can show:** the size of one layout's interaction with the worker count, on one
  stream. Not what share of -9's 3.26 the table is, and not the fate of every compact table.

Labels: **archived** (from the bundle), **counted** (exact for its input), **model** (a
prediction, unobserved), **proposed**.

## 0. What the revision changed

| the first draft said | now |
|---|---|
| "the walk's instructions unchanged" | the hot recurrence can match; the complete walk is unproved, and no instruction figure is given without code and listings (§3) |
| "B4 − A4 reads the side effects at a size L3 holds" | it does not isolate them: other buffers, private caches, mapping, traffic, pages and faults move with the width at four workers too, and scale with workers (§3) |
| the build dirties 1.5 → 5.9 GB a decode | **0.84 → 3.36 GB**, from the row census (§2); the first figure took all 1,831 blocks as full |
| pages 200 → 800 | 196 → 782 a full table (197 → 783 with 64-byte alignment) |
| "below 1.15 … the premise under every compact table dies" | below 1.15 excludes a 15% interaction of *this layout* with workers on *this stream*. Nothing wider (§6) |
| bands with nothing between 2.9 and 3.5 | complete (§6) |
| E4 "above about 1.5", scale unstated | every estimand in log, its factor beside it (§4) |
| "a standard deviation near 0.05" | archived: 0.105 at -2, 0.058 at -9, 0.112 for the paired difference (§8) |

## 1. The hypothesis and its rivals

H: with the stream, the feed and the symbols fixed, a table addressed over four times the
bytes raises the growth of CPU from four workers to ten.

| form | what contends | says about the 16-byte arm |
|---|---|---|
| F0, no table term | something else a -9 block owns: its 900,000-row column, symbols, segments, output | grows as the 4-byte arm does |
| F2, occupancy | tables being built or walked at that moment (`hypothesis-register.md` §1a) | grows more, by how many workers run at once |
| F1, literal | every requested worker's table, the old X | grows most |

## 2. The input and the cells

**The input, counted.** `wikidata-l2` reproduced here from the pinned content with
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
- **Sentinels:** the default plan on the original file, twice a round, as archived.
- **Optional references:** -9 at 4 and 10, marker on: occupancy only, no estimate.

## 3. The 16-byte entry: an addressed-bytes intervention

**The layout.** One `u32` array of k·n entries, k = 1 or 4. Row r's entry is at index k·r;
the indices between hold nothing and are never read or written. Every row is visited once a
walk, so every line of the 16-byte arm's 3.2 MB is touched: 4 entries a line against 16.
- **The link an entry stores is the scaled index j = k·r.** The hot recurrence is then the
  same in both arms: load at `base + 4j`, the byte from the low 8 bits, j from the shift,
  the prefetch at the new `base + 4j`. A full block's j is under 2²⁰, so bit 31 stays free.
- **Not a pointer to 16-byte records indexed by 4r:** that scales twice.
- **The logical n is kept** for every count, bound, length and output.

**What is not shown equal, and must be charged from code and listings:**
- the scaled primary index's own arithmetic at the origin and at each take;
- the segment starts: drawn as rows, flagged at scaled indices, found again at a segment's
  end by a search over them;
- bounds and asserts on n against bounds on k·n;
- the tails, the short-block path and the assembly;
- the build: the cursor's scaling, the column's byte read at the logical slot, the end check;
- walks of more than one cycle, which exist for crafted columns and must keep production's
  meaning.

**No instruction figure is claimed here.** Before any timing (§9): inclusive counts, matched
between the arms, of the build, the whole walk and `decode_block`; and production's build
against the new one-row constructor of arm A, since A is not production (a 16-wide run store
cannot span four lines, so both arms take the one-row build from one source generic over k).

**What changes with the width besides bytes in the shared cache:**

| | 4 bytes | 16 bytes | |
|---|---|---|---|
| lines addressed a full block | 12,500 | 50,000 | the intervention |
| line-bytes the build writes a decode | 0.84 GB | 3.36 GB | counted: rows × width (the bound N ≤ 1.25 D gives 1.05 and 4.19) |
| 4 KiB pages a full table | 196 | 782 | huge-page setting recorded |
| first touch a process | 196 pages a worker | 782 | every slot is a fresh process: paid in every slot |
| the table against a 256 KB private cache | 3 times it | 12 times it | both exceed it; by different margins |
| the build's 256 write streams | a line a 16 rows | a line a 4 rows | |
| resident memory a worker | | + 2.4 MB | growth peak and reuse across blocks recorded |

- **B4 against A4 does not isolate these.** At four workers the width also moves private-cache
  misses, pages, faults, write traffic and what the other buffers are evicted by; and each of
  those scales with workers in its own way.
- **So I (§4) is the interaction of this layout with the worker count, from every cause.**
  Shared-cache capacity is one candidate reading of it. The design cannot make it the only one.
- **The feed binds at -2:** 5.9 workers ran at once at ten in the archived run, and a slower
  16-byte worker raises that. CPU is primary, wall secondary, and workers running at once is
  reported for every cell.

## 4. Estimands (proposed; marker-off cells only)

On whole-process CPU, paired within round, each given as a log and as its factor:
- **E4** = ln(B4 ÷ A4), **E10** = ln(B10 ÷ A10): the width's effect at each worker count.
- **I** = E10 − E4. exp(I) is the 16-byte arm's growth from four to ten over the 4-byte arm's.
- The same on wall, secondary.
- **A width that doubles the cost at both worker counts gives exp(I) = 1.** I says nothing
  about a cost that does not interact with workers; E4 and E10 carry that, with §3's caveat.
- **Marker contamination:** the same three from the marker-on cells. If their difference from
  the marker-off ones moves I materially, the trace is rejected for attribution and the
  marker-off result stands alone.
- **Not adjusted for overlap:** width changes overlap, so overlap is downstream of the
  treatment.

## 5. Predictions for exp(I) (model; from the archived 4-byte cells)

Archived: CPU 0.6825 s at four workers and 0.7929 at ten; scanner and main 0.10 and 0.11;
workers 0.582 and 0.683; 3.64 and 5.92 workers running at once. f = 0.78. X is the old curve.
Each form is asked what it says *of this layout*; a side effect common to B4 and B10 cancels
in I, and one that grows with workers does not.

| form | table bytes it counts, 4 and 10 workers | workers' CPU factor | exp(I) | exp(E4) from the table term alone |
|---|---|---|--:|--:|
| F0 | none | 1, 1 | **1.0** | 1.00 |
| F2, as many running as in the 4-byte arm | 9.4, 16.4 MB | 1.12, 1.65 | **1.4** | 1.10 |
| F2, as many running as at -9 (8.1) | 9.4, 24.3 MB | 1.12, 3.49 | **2.8** | 1.10 |
| F1 | 12.8, 32 MB | 1.30, 6.27 | **4.4** | 1.26 |

## 6. What each outcome licenses (proposed bands; the Researcher's to set)

Every reading is of this layout, on this stream, at these two worker counts.

| the interval for exp(I) lies | reading |
|---|---|
| below 1.15 | no interaction of 15% or more. F2 and F1, as predictions for this intervention, are rejected. It does not say a compact table is worthless: a width cost that does not interact shows in E, not I; and -9's stream is another stream. |
| within 1.15 to 1.4 | an interaction, below what the old X gives at any occupancy. The old curve is rejected here as a curve. |
| within 1.4 to 2.8 | consistent with F2. The literal law stays rejected. |
| within 2.8 to 4.4 | above F2 at -9's running count; the literal law not excluded. |
| above 4.4 | more than the literal law. A side effect that grows with workers is suspected before any form is. |
| across a boundary | inconclusive between the two readings it spans, and said so. |

- **exp(E4) above 1.5:** the width's cost at four workers is as large as the interaction
  sought. I is reported with that warning and no form is named.
- **Marker on against off moving I:** the trace is description only.
- **No outcome qualifies or disqualifies a candidate.** D4-delta's F2, the chunked link's
  missing kernel and compact30's unmeasured g stand where they are.

## 7. The marker (descriptive)

Per block, a worker's clock at five edges: MTF start, build start, walk start, walk end,
decode end. 9,155 reads a -2 decode, some 780 of the blocks near-empty.
- Private preallocated records, no shared written line; an overflow voids the trace.
- What the edges are: build-active and walk-envelope intervals (setup, traversal and
  assembly inside the walk's), not a table's lifetime and not cache residency.
- Outputs a cell: the time spent at each count of workers in build and in walk envelopes,
  workers running at once, each envelope's share of a worker's time, with uncertainty
  clustered by round. No law is fitted. (A within-block regression of walk time on overlap
  was this author's first proposal; the Optimiser's counterexample, that a walk's duration
  sets its own averaging window, stands against it.)

## 8. Precision (model, from the archived run)

Archived standard deviations of a round's log growth: 0.105 at -2, 0.058 at -9, and 0.112
for their paired difference (the Optimiser's replay). I is a difference of two -2 growths: a
round's I has a standard deviation between about 0.11 and 0.15, depending on how far pairing
cancels. Over 20 rounds that is 0.025 to 0.033, an interval of ±0.05 to ±0.065 in log.
- The 1.15 boundary is 0.14 in log: two to three such half-widths from 1. Enough to tell 1.0
  from 1.4; marginal for a true value near 1.15.
- Arithmetic on another experiment's variance. Not a power claim; the rounds are the
  Researcher's to set.

## 9. To complete before a freeze (not in this draft)

**Untimed admission, additional to the 210 slots:**
- both arms' bytes equal `bwt::walk`'s and the blocks' CRCs on every block of the file, and
  on crafted columns with several cycles;
- §3's matched inclusive counts and the listings read;
- the kernel's `unsafe` reviewed (B5) before any eligible run;
- the marker's records complete and ordered on a dry decode.

**Rules to write, starting from the archived run's:**
- a slot's timeout, and what a timed-out slot does to its round;
- identity: the binary, the input and the content pinned by SHA-256, each slot's output
  hashed;
- invalidity: which failures void a slot, a round, the attempt; none decided after the data;
- the schedule: balanced order within a round, its seed, the warm-up, the busy gate;
- the bootstrap and its block length, as the archived judge's.

**Not proposed:** a 32-byte arm (6.4 MB a full block, past L3 with six workers running),
until the 16-byte one has been read.

## Reproduce (the row census)

```text
bzip2 -dc corpora/wikidata.bz2 > content          # SHA-256 3f075e30…
lbzip2 -2 -n 4 -c content > wikidata-l2.bz2       # SHA-256 d3bcb141…a9323045
# rows a block: Scanner, then mtf::mtf, the column's length; 1,831 blocks, 209,706,556 rows
```
