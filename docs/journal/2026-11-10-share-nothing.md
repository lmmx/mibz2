# 2026-11-10: The step nobody listed: cooperation. What a small table makes unnecessary

**Status: a reading of the record and of counts already archived, and a design direction. No
new run, no decoder change. Handed to the Coordinator.**

- **The question** (Louis): is there a step the decoder takes that a different formulation
  makes unnecessary, rather than faster?
- **The answer:** the cooperative layer. Teams, pipelined plans, shared builds and shared
  walks exist for one reason: a level-9 block's table is 3.6 MB and four of them fill the
  i9's cache.
- **Counted:** the default ten-thread plan executes 38.0 instructions a decoded byte against
  one thread's 26.7. Cooperation costs 42% more work for the same bytes, 76% more in the walk.
- **Already measured on the workstation, and not read this way:** with plain independent
  workers, the CPU a decode costs grows 3.50× from 4 to 10 workers at 3.6 MB a table, 1.44×
  at 1.2 MB and 1.10× at 0.4 MB.
- **So:** a table under about 1.2 MB a block, with the parallel scan, lets ten workers each
  run the one-thread decoder and share nothing. Designs C, D and E are that table.

Labels: **read** (from the record), **counted** (callgrind, exact for the input, bias stated),
**model** (derived, not observed).

## 1. What cooperation costs (counted)

Instructions a decoded byte, `wikidata.bz2`, the simulated runs of the bandwidth ledger
(`2026-11-10-bandwidth-titration.md`):

| | one thread | 10 independent workers | the default plan (three teams of three) |
|---|--:|--:|--:|
| walk | 10.1 | 10.1 | 17.8 |
| build | 6.2 | 6.2 | 9.2 |
| MTF | 3.9 | 3.9 | 4.4 |
| scanner | 3.5 | 3.5 | 3.5 |
| everything else | 3.0 | 3.1 | 3.1 |
| total | 26.7 | 26.8 | 38.0 |

- **Independent workers do one thread's work exactly.** The default plan does 42% more.
- **The walk is where it goes:** 76% more instructions when three threads share a table.
- **Bias:** valgrind runs one thread at a time, so a loop that polls for work may count
  differently than on ten cores. The table's shape, not its second digit, is the finding.

## 2. What a small table buys (read)

`2026-10-28-workstation-scaling.md` §9, the i9, plain workers, the corpus recompressed at four
block sizes. CPU-seconds for the whole decode, exact:

| level | table a block | 4 workers | 10 workers | growth |
|---|--:|--:|--:|--:|
| -9 | 3.6 MB | 0.834 | 2.921 | ×3.50 |
| -6 | 2.4 MB | 0.729 | 1.930 | ×2.65 |
| -3 | 1.2 MB | 0.704 | 1.012 | ×1.44 |
| -1 | 0.4 MB | 0.791 | 0.874 | ×1.10 |

- **That section was read for where the knee sits.** Read for what a table size is worth, it
  says: at 1.2 MB a block, ten independent workers pay 44% more CPU than four; at 0.4 MB, 10%.
- **And it already ran faster that way.** At -3, eight plain workers gave 1,729 MB/s, with the
  decoder of 28 October, before the scanner bound them. Today's cooperative default gives
  about 1,500 on level-9 input.
- **The scanner is the other wall,** as that section's title says. One scanner caps the
  decoder at 9 to 11× one thread on level 9 and 4.9× on level 1
  (`2026-11-10-parallel-scan.md`).

## 3. The design direction (model)

**Make a level-9 block look like a level-2 block to the cache.**

| table | bytes a row | a 900 k block | ten live |
|---|--:|--:|--:|
| today's | 4 | 3.6 MB | 36 MB |
| E, the line table | 2 | 1.8 MB | 18 MB |
| D, k-step cells at k = 4 (Wikidata) | 1.6 | 1.44 MB | 14.4 MB |
| C, runs and a directory (Wikidata) | 0.83 | 0.75 MB | 7.5 MB |

The knee sits at 14.4 MB live. C and D at k = 4 are under it at ten workers on data with long
runs; E is under it at eight on text.

**Then the plan is the simplest one there is.** Each worker finds its own block (the parallel
scan), reads it, emits its runs, fills its table, walks it, and hands the bytes to the ordered
sink. Nothing is shared but the input and the sink.

**What becomes unnecessary:**

| step | why it exists | what removes it | its share today |
|---|---|---|--:|
| the last column | the build reads it | the run emitter | 12.8% of cold writes |
| the build | the walk needs Ψ in F order | a table filled from runs (A, C, D, E) | 6.2 instructions a byte |
| shared builds and walks | four tables fill L3 | a table under 1.2 MB | 11.3 instructions a byte |
| the team plans, their queues and waits | the same | the same | ten threads 68% busy in the fast mode (1.08 CPU-seconds in 0.159 s) |
| the scanner thread | block ends are unknown | the bit-aligned finder | a cap of 9 to 11× |
| segment assembly | lanes start at unknown text positions | RLE1 read from the segments in chain order (#183) | 15.2% of cold writes |

**Model, level-9 Wikidata on the i9, today's kernels otherwise:** ten workers at one thread's
400 MB/s, less the all-core clock (about 0.94) and the 1.1 to 1.4× of §2: 2.7 to 3.4 GB/s,
against about 1.5. The emitter and the k-step walk multiply that; they are not counted in it.

## 3a. Correction after #323: which table, if any (model, with E's count applied)

Design E was built and falsified (`2026-11-10-design-e-result.md`): 48 instructions a row
against the room walk's 10.4, a one-thread inversion at 3.67×. The gate of this entry's §4.1
is a one-thread walk within 1.44× of the plain walk, about 15 instructions a row. Read against
it, with E's as-built step (22 to 26) and bookkeeping (about 22) in place of this author's
estimates:

| table | a step is | bytes a step | instructions a row | the gate |
|---|---|--:|--:|---|
| E, the line table | compare, two popcounts, a base | 1 | 48 (counted, #323) | fails, 4.6× |
| C, runs and a directory | a directory popcount, an entry, an add | 1 | about 45 (model) | fails, the same way |
| D, k-step cells, k = 2 | C's step, a 2-byte store | 2 | about 23 | fails |
| D, k = 4 | C's step, a 4-byte store | 4 | about 12 | passes, narrowly |
| D, k = 8 | C's step, an 8-byte store | 8 | about 6 | passes |

- **C is withdrawn as a walk table.** Its step is E's in kind and yields one byte. It keeps its
  use where the fill's cost is what matters (§5's peeks).
- **Only a table that emits several bytes a step can clear the gate,** because the step itself
  cannot be brought near the T-vector's three instructions. That leaves D.
- **D is not yet a design that can be built to the gate.** Two things are open:
  1. **Lanes.** A lane that steps k rows of text at a time cannot land on a marker that lies
     inside its step, and `2026-11-10-backward-table.md` §4a did not say how lanes end.
     A way: for each marker row m, the k rows LF⁻ʲ(m), j < k, are each made a cell of their
     own while the cells are doubled, flagged with j. A lane that lands on a flagged cell
     takes the first j bytes of its word and ends there, joined to m's segment. That is at
     most 512 k cells more, and one flag test a step.
  2. **The build.** Doubling is a loop a cell with a data-dependent trip count, the shape of
     E's build, which ran 1.83× the scatter's time at 0.82× its instructions. Counted on
     paper: 0.197 n cell-steps to k = 4 and 0.34 n to k = 8 on Wikidata, at about 30
     instructions each: 6 to 10 a row against the build's 4.9.
- **So the inversion, build and walk together, is about 18 instructions a row at k = 4 and 16
  at k = 8, against production's 15.3:** near parity on paper, from an author whose last paper
  count was low by half. At k = 4 the table is 1.44 MB and ten of them sit exactly at the
  knee; at k = 8 it is 2.25 MB and six fit.
- **What stands of this entry:** §1 and §2 as readings: what cooperation costs and what a
  small table is worth. **What does not:** §3's claim that C and E are that table.
- **What the pricing run (§4.2) now decides:** whether the prize is large enough to take on
  D's two open problems. It prices the table nobody has, not any design.

## 3b. The prize, measured (read: `par-scan/20261110T030608Z`, the i9, VALID)

The parallel-scan diagnostic ran on the workstation. Bundle SHA-256 `140a0e5b…bab4d152`,
checked. Plain workers, one process, five rounds a cell, medians:

| input | arm | 1 | 4 | 8 | 10 | CPU-seconds, 1 → 10 |
|---|---|--:|--:|--:|--:|--:|
| level 9 (3.6 MB a table) | scanner | 455 | 1,145 | 741 | 631 MB/s | 0.58 → 3.69 |
| level 9 | finder | 409 | 1,127 | 744 | 619 | 0.57 → 3.63 |
| level 1 (0.4 MB a table) | scanner | 413 | 1,524 | 1,600 | 1,598 | 0.70 → 0.82 |
| level 1 | finder | 348 | 1,255 | 2,429 | 2,563 | 0.66 → 0.84 |

- **Small tables and the finder: 2.56 GB/s today,** on level-1 input, with CPU-seconds nearly
  flat. At ten workers the wall is the CPU divided by ten (0.84 s, 0.082 s): the cores are
  full and nothing else binds.
- **Large tables: the knee, as before.** Level 9 peaks at four workers and its CPU grows 6.4×.
- **One scanner caps level 1 at 1.6 GB/s.** The finder removes the cap: 1.6× at ten workers.
- **The gate, from these figures and no longer from §2's 1.44×.** A level-9 block costs 0.57
  CPU-seconds here, 70% of it the build and walk. With a small table whose inversion costs g
  times production's, and level 1's CPU growth (1.13 to 1.27), ten cores give about
  3.0 GB/s ÷ (0.3 + 0.7 g):

  | inversion, g | 1.0 | 1.2 | 1.44 | 2.0 | 2.5 | 3.67 (E) |
  |---|--:|--:|--:|--:|--:|--:|
  | GB/s at ten (model) | 3.0 | 2.6 | 2.3 | 1.8 | 1.5 | 1.0 |

  Break-even with the cooperative default (about 1.5 GB/s) is near g = 2.5. E at 3.67 loses, as
  its result said. A table at 1.2 would be worth 2.6 GB/s. This assumes the table is small
  enough to behave as level 1's does; the `lbzip2 -2` run measures that.
- **Corrected (the register's close reading): the break-even above mixes two harnesses.** The
  3.0 GB/s comes from long-lived processes run back to back; the default plan's 1.5 GB/s from
  one fresh process a run. Four plain workers on level 9 read 1,145 MB/s here and 1,016 to
  1,021 there. On one harness the break-even is nearer g = 2.1 than 2.5. `par_scan`'s `lib`
  arm in the same sweep would give the default plan's figure directly.
- **And at 1.2 MB a table the old ladder has ten workers behind eight** (§2's run: 1,729 MB/s
  at eight, 1,604 at ten, at level 3). Level 1's 0.4 MB scales from eight to ten; whether
  0.9 to 1.4 MB does is what the `lbzip2 -2` and `-3` cells measure.
- **No second mode in this bundle.** 180 cells in three long-lived processes, and no cell's
  rounds differ by 2×: level 1 at ten workers reads 81, 83, 81, 82, 83 ms. The two modes of
  `2026-11-10-two-modes.md` were seen with one fresh process a run and cooperative plans. This
  harness differs in both, so it separates neither; it is an observation for the discriminator.

## 4. Falsifiers, cheapest first

1. **Free:** §2 is the experiment, already run. A reading of it that fails: if C's or D's walk
   costs more than 1.44× the plain walk a byte at one thread, the cooperative plan on the
   plain table still wins at ten.
2. **One workstation run, existing arms:** plain workers on `lbzip2 -2` Wikidata (0.8 MB a
   block, C's footprint) at 2 to 10, with today's decoder. It prices §3's model before any
   table is built. The scanner binds it near 5×, so it reads CPU-seconds, as §2 did.
3. **The registered one:** the same sweep on level 9 once a small table and the parallel scan
   exist.

## 5. Two answers that need no inversion at all (model, not built)

For a consumer that does not want the bytes, the walk itself is the unnecessary step.

- **Counts.** A block's byte histogram is the MTF stage's `counts`, less the RLE1 count bytes.
  Those follow four equal bytes, and the rows whose four preceding bytes are equal are whole
  cells of LF⁴: found in the runs' time, not the rows'. So the decoded length, the line count
  and any byte's count come from the scanner and the emitter alone: on Wikidata about a
  quarter of a decode's work.
- **Peeks.** From `orig_ptr`, LF gives a block's last bytes and Ψ its first. With a table
  whose fill costs the runs (C), the first or last N bytes of every block cost the front end
  plus N steps. That is enough to find record boundaries for a parallel consumer, or to
  sample a dump.

Neither helps `bzcat`. Both are a different product from the same parts.

## Reproduce

```text
# §1: instruction totals from the ledger's simulated runs
valgrind --tool=callgrind --cache-sim=yes --D1=262144,8,64 --LL=20971520,20,64 \
  target/release/examples/ab child ARM THREADS corpora/wikidata.bz2
#   ARM THREADS: mibz2 1 | mibz2-workers 10 | mibz2-pipe3-both 9   (evidence: 2026-11-10-bandwidth-titration/)
callgrind_annotate --show=Ir callgrind.out.*
# §2: docs/journal/2026-10-28-workstation-scaling.md §9 (blocks-20261028T015837Z)
```
