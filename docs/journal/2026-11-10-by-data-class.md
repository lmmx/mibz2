# 2026-11-10: By data class: where a decode's time is, four ideas counted out, two that stand

**Status: counts and local timings, indicative (RULES C5). Four candidate ideas tested against
their own premises and dropped. Two findings handed to the Coordinator as designs. Nothing in
the decoder changes.**

- **The question:** after the run emitter, the backward tables and the parallel scan, what is
  the next step change, and for which data?
- **Measured:** the record's stage split is Wikidata's. On ordinary text the front end
  (move-to-front and the build) is 39% of a decode, and on incompressible data move-to-front
  alone is 50%.
- **Stands, 1:** the run emitter is worth more on text than on Wikidata. Its stage is 3.0× the
  production MTF's there, and with a table filled from the runs a text decode loses 21 to 30%.
- **Stands, 2:** a table of one cache line a 32 rows (2 bytes a row, one access a step), for
  data whose runs are short. It halves the live table, which is what sets the i9's knee.
- **Counted out:** overlapping MTF chains, overlapping scanner chains, a pair table filled
  from the runs, and contracting the walk by sequential passes. Each failed a count below.
- **Added to the two-modes design:** one long decode, the cheapest test of whether the slow
  mode belongs to short processes.

Labels: **measured** (local, one container, indicative), **counted** (instruction or event
counts, exact for the input), **model** (derived, not observed).

## 1. The stage split depends on the data (measured)

`phases`, cycles a decoded byte at a nominal 3 GHz, one thread, the same session:

| stage | `wikidata` (17.7×) | `text` (5.05×) | `records` (1.00×) |
|---|--:|--:|--:|
| scanner | 1.35 (11%) | 3.17 (16%) | 9.29 (27%) |
| MTF and RLE2 | 1.84 (15%) | 5.47 (27%) | 17.18 (50%) |
| build | 1.75 (15%) | 2.50 (12%) | 2.96 (9%) |
| walk | 6.54 (55%) | 7.68 (38%) | 4.63 (13%) |
| RLE1, CRC | 0.48 (4%) | 1.48 (7%) | 0.28 (1%) |
| total | 11.96 | 20.30 | 34.34 |
| runs a row | 0.072 | 0.300 | 0.997 |

- **The walk is over half of a decode only on Wikidata.** On text the three stages before it
  are 55%.
- **Text costs 1.7× Wikidata a byte, and records 2.9×.** The difference is in the scanner and
  MTF, which scale with symbols and not with rows.
- **So a walk-only gain is worth 55% of itself on Wikidata, 38% on text and 13% on records.**

## 2. Stands: the run emitter by class (measured)

`run_front time`, the emitter's first stage against production `mtf::mtf`, ns a row:

| | `wikidata` (recorded) | `text` | `records` |
|---|--:|--:|--:|
| production MTF | 0.423 | 3.00 | 8.84 |
| the emitter, stage 1 | 0.173 | 0.987 | 9.34 |
| ratio | 2.4× | 3.0× | 0.95× |

- **On text the emitter is 3.0× the MTF stage,** more than on Wikidata: 6.9 cycles a record
  against production's 20.9 (ns × the TSC's 2.1 GHz).
- **Model, a text decode:** MTF 5.47 → 1.80, and the build 2.50 → 1.94 with the one-step table
  filled from runs (`lf_fill`: 0.78 of the build on text). Total 20.30 → 16.07, −21%. With a
  table whose cost is the runs' (designs C and D), up to −30%.
- **On records it gains nothing.** Every symbol is a literal at a high index.
- **For the Coordinator:** the emitter's production gate should read text as well as Wikidata.
  The larger gain is there.

## 3. Stands: the line table, design E (counted, then model)

**The design.** One 64-byte line a 32 consecutive rows, in the LF direction:
- the 32 bytes of L;
- a 32-bit mask of the rows that are the first of their byte in the line;
- up to 9 bases of 3 bytes, in first-occurrence order: C(c) plus the copies of c before the
  line.

A step from row i, at offset o in its line, with c = L[i]:
- `eq` = the line's rows equal to c (one compare and a mask);
- its slot = the first-occurrence bits below `eq`'s lowest bit, counted;
- LF(i) = the slot's base + the bits of `eq` below o, counted.

About 16 instructions and one line, against the plain table's 10.5 and one line of 16 entries.
A line with more than 9 distinct bytes keeps 8 and an index into an overflow list: a second
access for the rows whose byte is past the eighth.

**Counted** (`examples/line_census.rs`: text's 25 blocks, every sixteenth of Wikidata's,
every fourth of records'), distinct bytes in aligned 32-row lines:

| | mean | lines with at most 8 | at most 9 | at most 12 |
|---|--:|--:|--:|--:|
| `wikidata` | 2.26 | 93.0% | 94.0% | 96.2% |
| `text` | 5.12 | 79.0% | 83.0% | 92.9% |
| `records` | 30.5 | 0% | 0% | 0% |

- **2 bytes a row,** half the plain table, for text and Wikidata. Records keeps the plain
  table: the choice is made a block, from the count of runs.
- **Its fill is sequential:** the 32 bytes, then a base a distinct byte from one counter a
  byte. About 1.6 instructions a row on text (model), against a build of 2.5 cycles a byte.
- **What it buys, model:** the i9's knee sits at a live table of 0.7 × L3 (four plain tables).
  At 2 bytes a row that is eight. On text, where designs C and D have little to remove
  (below), this is the member of the family that moves the knee.
- **What it costs:** half again the instructions a step. On a core whose L2 holds the table
  (1 MB and up) more steps hit L2; on the i9 at one thread it is neutral at best.
- **Prior art:** fixed-block boosting of FM-indexes (Kärkkäinen and Puglisi, 2011) uses the
  same local scarcity of distinct symbols, for space. Nothing here is claimed as new but the
  use.
- **Cheapest falsifier:** the laned walk on this table against the room walk, one thread,
  text: slower by more than the build saves. **Gate:** a registered thread sweep on text.

## 4. Counted out

### 4a. Overlapping MTF chains

**The claim that would have carried it, and is true:** a move-to-front step commutes with any
relabelling of the list. So a block's symbols cut into K chunks decode independently from the
identity list; chunk k's true bytes are its outputs read through the list at its start, and
that list is the previous chunk's composed with its final one. One 256-byte lookup a run, no
second pass.

**The premise that fails:** that MTF waits on a chain.
- **Text, counted:** 53 instructions and 0.67 simulated mispredictions a move (callgrind,
  6.60 M moves a pass).
- **Text, measured against them:** a move takes 14 to 25 cycles for any core clock from 2.1 to
  3.7 GHz. 53 instructions at four a cycle are 13, and 0.67 mispredictions at 15 cycles are
  10. Nothing is left for a chain to explain, and lanes share a misprediction's flush.
- **Records, counted:** 79 instructions a move, in 17 nominal cycles: bound by instructions.
- **What remains:** the relabelling lets a team's threads share one block's MTF. No plan
  measured so far has MTF as the stage a team waits on.

### 4b. Overlapping scanner chains across blocks

- **The idea:** with the block finder, one core reads two or three blocks' bit streams in
  lockstep.
- **Counted, text:** `read_item` 270 M instructions a pass against 79 M nominal cycles. At
  least 2.7 instructions a cycle at any clock up to 3.7 GHz.
- **Bound:** under 1.5× on the scanner, under 5% of a text decode. Not pursued.

### 4c. A pair table filled from the runs

- **The claim, true and checked:** a run's rows go under LF to consecutive rows, so
  Q[i] = L[i], L[LF(i)], LF²(i) is filled from the run list with one cursor a byte, and its
  walk takes two bytes a step. `lf_fill`: byte-identical on 431, 25 and 22 blocks.
- **Measured, the fill as written:** 5.4× the build on text, 3.8× on Wikidata, 5.1× on
  records. The one-step fill gained 3× from wide stores, so perhaps 2× the build is reachable.
- **Bound:** it removes at most half the walk and adds a build. On text that is 3.8 cycles a
  byte saved for 2.5 spent if the fill reaches twice the build: 6% of a decode. As written it
  is a loss. Not handed.
- **The k-step cells on text, counted** (`phrase_census`): r₂ 0.449 n, r₄ 0.626 n, r₈ 0.797 n.
  Design D's table is 2.5 bytes a row at k = 2 and 4.6 at k = 4. D is a lever for data with
  long runs. On text it is not.

### 4d. Contracting the walk by sequential passes

- **The observation, true:** a pass that reads X[LF(i)] in row order is 256 sequential
  cursors. So one round of list contraction costs about a build, not a random pass.
- **Bound:** a round removes a third of the rows at best (an independent set of a cycle chosen
  locally), so it saves a third of the remaining accesses for a pass over all of them. A build
  costs about a third of a walk here. Nothing is gained at any depth.

### 4e. A line-local pair table

- **Counted:** distinct pairs in 16-row lines: text 5.66 mean, 86.9% of lines at most 10.
- **Dominated:** 4 bytes a row and two bytes a step, against the dense pair table's 5 bytes a
  row with no overflow and no compare. And the dense pair table is 4c.

## 5. Records: MTF at high indices (open, small)

- **Counted:** 79 instructions a move; every index is past the two registers' 32.
- **Tried on paper:** the list as a bitmap of live positions in the output of moves, a move
  being a select from the newest end. About 50 instructions with the words in registers, and
  a chain of about 15 cycles. With 4a's relabelling to overlap chains, 1.4 to 1.6× at most.
- **Worth:** under a quarter of a decode of data bzip2 does not compress. Recorded, not
  handed.

## 6. For the two-modes design: one long decode

A bzip2 file may be streams end to end. `wikidata.bz2` fifty times over is one input of
10.5 GB decoded, 7 to 15 s at 10 threads.
- **Read:** throughput in 100 ms windows, from the decoder's own byte count.
- **Steady at the fast mode's rate:** the slow mode belongs to 0.15 s processes (the ramp,
  packing, or pages), and the harness needs decodes back to back.
- **Falls to the slow mode's and stays:** stop-clock or the power limit, and every burst
  figure overstates the decoder.
- **Alternates:** the mode's period, which no 0.15 s run can show.

It needs no probe and no counter. It goes first.

## 7. What this changes

| data | binds now | the lever | sown |
|---|---|---|---|
| long runs (JSON, XML, logs) | the walk, 55% | k-step cells (D), runs and directory (C) | 2026-11-10-backward-table |
| text, source | MTF and build, 39%; walk, 38% | the emitter (−21 to −30%); the line table at many threads | here |
| incompressible | MTF, 50%; scanner, 27% | none found worth its cost | here, §5 |
| any, at 10 threads | the live table against L3; the scanner | a table of 2 bytes a row or less; the parallel scan | here, and 2026-11-10-parallel-scan |

**Limits.** One container, an EPYC with a 2.1 GHz TSC and an unknown core clock: every cycle
figure is nominal. `text` and `records` are the bench's synthetic corpora. callgrind's branch
predictor is simpler than a core's.

## Reproduce

```text
cargo build --release --examples
target/release/examples/ab gen DIR 8                      # text.bz2, records.bz2
target/release/examples/phases FILE.bz2 3 3.0             # §1
target/release/examples/run_front time FILE.bz2 3         # §2
target/release/examples/line_census FILE.bz2 [EVERY]      # §3, §4e
target/release/examples/lf_fill FILE.bz2 5                # §4c
target/release/examples/phrase_census FILE.bz2 6          # §4c, the cells
valgrind --tool=callgrind --branch-sim=yes target/release/examples/phases FILE.bz2 1 3.0   # §4a, §4b
```
