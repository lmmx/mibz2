# 2026-11-10: The addressed-width intervention: a registration, in draft

**Status: a draft for the Optimiser's price and the Researcher's review. Not registered, not
frozen, nothing built, nothing queued. Every threshold here is a proposal. Predictions are
written before any data of this experiment exists; the archived share-nothing run
(`share-nothing/20261110T052925Z`) is their only input.**

- **The question:** at -9 the workers' CPU grows 3.26 times from four workers to ten, at -2
  1.18 times. Between the two files everything changes at once: the table, every other buffer
  a block owns, the block count, the feed. Do the *table's addressed bytes alone* carry the
  inflation?
- **The intervention:** the -2 file, unchanged. The same table, its records 4 bytes or 16
  bytes apart: 0.8 or 3.2 MB addressed a worker. Nothing else about a block changes.
- **The design** (the Optimiser's, #145 6760293462): 2 widths × workers 4 and 10 × marker on
  and off: 8 cells, 2 sentinels, 20 rounds and a warm-up: 210 slots; 252 with two -9
  reference cells.
- **What it can show:** whether addressed table bytes are *sufficient* for the inflation on
  this block stream. It cannot show what share of -9's 3.26 they are.

Labels: **archived** (from the bundle), **model** (a prediction, unobserved), **proposed**.

## 1. The hypothesis and its rivals

H: with block count, feed, symbols and every other buffer fixed, quadrupling the bytes a
worker's table is addressed over raises the growth of CPU from four workers to ten.

| form | what contends | says about the 16-byte arm |
|---|---|---|
| F0, no table term | something else a -9 block owns: its 900,000-row column, symbols, segments, output | grows as the 4-byte arm does |
| F2, occupancy | tables being built or walked at that moment (`hypothesis-register.md` §1a) | grows more, by how many workers run at once |
| F1, literal | every requested worker's table, the old X | grows most |

F0 is the form no one has tested, and the one a smaller table cannot help.

## 2. The cells (proposed)

| | workers 4 | workers 10 |
|---|---|---|
| 4-byte records, marker off | A4 | A10 |
| 16-byte records, marker off | B4 | B10 |
| 4-byte, marker on | A4m | A10m |
| 16-byte, marker on | B4m | B10m |

- **Input:** `wikidata-l2` of the archived run (SHA-256 `d3bcb141…a9323045`, 1,831 blocks).
- **Arm:** plain workers, `Plan::Workers`, its window of 2 × workers slots, as archived.
- **Sentinels:** the default plan on the original file, twice a round, as archived.
- **Optional references:** -9 at 4 and 10, marker on: occupancy only, no estimate.
- **Order:** balanced within a round, the archived judge's schedule and bootstrap.

## 3. The 16-byte record: an addressed-bytes intervention (proposed)

- **The record:** entries 16 bytes apart from a 64-byte-aligned base. Bytes 0 to 3 hold the
  entry; bytes 4 to 15 are never read or written. Every record is visited once a walk, so
  every line of the 3.2 MB is touched: 4 entries a line where the 4-byte table has 16.
- **The same instructions in the walk:** the entry holds its link times four
  (`byte | (link · 4) << 8`; a -2 block has under 2¹⁸ rows, so the scaled link is under 2²⁰
  and the start flag's bit stays free). The walk's load is then `base + (entry >> 8) · 4`,
  the 4-byte arm's addressing mode exactly.
- **One kernel for both arms:** build and walk generic over the stride, the same source,
  both arms compiled from it. The 16-wide run store cannot span four lines, so both arms take
  the one-row build. The 4-byte arm is therefore *not* production's build; production's own
  figures at these cells are the archived run's.
- **Before any timing:** both arms' bytes equal `bwt::walk`'s on every block; instructions a
  row counted for build and walk in each arm, and the listings read for a widened multiply,
  a spill, or a changed prefetch.

**What changes besides the bytes in cache (to be charged, not assumed away):**

| side effect | 4 bytes | 16 bytes | read by |
|---|---|---|---|
| lines addressed a block a worker | 12,500 | 50,000 | the intervention itself |
| 4 KiB pages a table | 200 | 800 | B4 against A4; huge-page setting recorded |
| lines the build dirties a decode | 1.5 GB | 5.9 GB | B4 against A4: traffic at a size L3 holds |
| the build's 256 write streams | a line a 16 rows | a line a 4 rows | the build's counted instructions and B4 |
| resident memory a worker | | + 2.4 MB | reported |
| run stores in the build | off in both | off in both | the kernel is one |

- **B4 against A4 is the control for all of them:** four tables of 3.2 MB are 12.8 MB, inside
  a 20 MB L3. What the width costs there is traffic, pages and density at a size that fits.
- **It is not a pure capacity test.** At ten workers capacity and write traffic rise together.
  The interaction (§4) subtracts what four workers show, on the assumption that traffic's
  cost a worker does not itself grow with workers. That assumption is stated, not tested.
- **The feed binds at -2:** 5.9 workers ran at once at ten in the archived run. A slower
  16-byte worker raises that. So CPU is primary and wall secondary, and workers running at
  once (CPU over wall, less the scanner and main) is reported for every cell.

## 4. Estimands (proposed; marker-off cells only)

Paired within round, on log whole-process CPU, as the archived judge:
- **E4** = B4 − A4 and **E10** = B10 − A10: the width's effect at each worker count.
- **I** = E10 − E4: the width-by-workers interaction. exp(I) is the 16-byte arm's growth
  from four to ten over the 4-byte arm's.
- The same on wall, secondary.
- **Marker contamination:** the same three from the marker-on cells; their difference from
  the marker-off ones is a diagnostic. If it moves I materially the trace is rejected for
  attribution and the marker-off result stands alone.
- **Not adjusted for overlap:** width changes overlap, so overlap is downstream of the
  treatment.

## 5. Predictions for exp(I) (model; from the archived 4-byte cells)

Archived: CPU 0.6825 s at four workers and 0.7929 at ten; scanner and main 0.10 and 0.11;
workers 0.582 and 0.683; 3.64 and 5.92 workers running at once. f = 0.78. X is the old curve.
A side-effect factor common to B4 and B10 cancels in I.

| form | table bytes it counts, 4 and 10 workers | workers' CPU factor | exp(I) |
|---|---|---|--:|
| F0 | none | 1, 1 | **1.0** |
| F2, as many running as in the 4-byte arm | 9.4, 16.4 MB | 1.12, 1.65 | **1.4** |
| F2, as many running as at -9 (8.1) | 9.4, 24.3 MB | 1.12, 3.49 | **2.8** |
| F1 | 12.8, 32 MB | 1.30, 6.27 | **4.4** |

And for E4, before side effects: F0 1.00, F2 1.10, F1 1.26.

## 6. What each outcome kills (proposed bands; the Researcher's to set)

| exp(I), interval inside | reading |
|---|---|
| below 1.15 | addressed table bytes do not make the knee on this stream. F1 and F2 die. The premise under every compact table dies with them: the candidates lose their reason, and what a -9 block holds *besides* its table becomes the question. |
| 1.15 to 1.4 | a table term, weaker than the old X at any occupancy. The old curve is dead as a curve; a small table is worth a fraction of -9's inflation. |
| 1.4 to 2.8 | consistent with F2. The compact-state line keeps its premise; the literal law stays dead. |
| above 3.5 | the literal law holds on this stream, and §1a's correction was the feed's doing. |
| an interval across a boundary | inconclusive. Said so, and not read toward either side. |

- **E4 above about 1.5:** the side effects at a fitting size are as large as the thing sought.
  I is then reported with that warning and no form is declared.
- **Marker on against off moving I:** the trace is descriptive only.
- **No outcome qualifies a candidate.** D4-delta's F2, the chunked link's missing kernel and
  compact30's unmeasured g stand where they are.

## 7. The marker (descriptive)

Per block, a worker's clock at: MTF start, build start, walk start, walk end, decode end.
Private preallocated records; an overflow voids the trace. Outputs a cell: the time spent
at each count of workers in build and in walk envelopes, workers running at once, and each
envelope's share of a worker's time. No law is fitted to it. (A within-block regression of
walk time on overlap was this author's first proposal; the Optimiser's counterexample, that
a walk's duration sets its own averaging window, stands against it.)

## 8. Precision (model, from the archived run)

The archived per-round log growth has a standard deviation near 0.05 at each level (one -2
round stood at 0.79 against a median of 0.15; the judge's bootstrap carries such rounds). I is a
difference of four paired cells: about 0.1 a round, 0.022 over 20 rounds, an interval of
about ±0.045 in log, ±5% on exp(I). The bands of §6 are 15% and wider. This is arithmetic on
another experiment's variance, not a power claim.

## 9. Not in this draft

The slots' duration, the timeout and the invalidity policy (the archived judge's are the
starting point); the kernel's code; a 32-byte arm, which would put 6 running workers past L3
and is not proposed until 16 bytes has been read.
