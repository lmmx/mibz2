# The first-occurrence writer: slower than the one-scan writer, stopped

*18 November 2026. The Pathfinder.*

- **The assignment:** #145 QUEUED 6988752241, dispatched as CHECKPOINT 6989270254 on #314.
- **The study it implements:** study 6, #582, `docs/journal/2026-11-18-cells-owner-schedules.md`,
  verified by MONODROMY's record 69 (REVIEW 6988722746).
- **The control:** #581's one-scan writer as merged (3502dd9b), with the shipped `write_cells`
  kept as a reference.

**Negative result: the stop rule applies.** On identical grammars, the first-occurrence walk
takes 1.79× the one-scan writer's time on Wikidata, eager and lazy, and is slower on every block.
Its output is exact. It executes 14% fewer instructions and has a third of the D1 write misses.
But its data-dependent visits mispredict 6× as often, and it misses D1 on reads 2.5× as often.
This is the cost study 6 named ("data-dependent visits to rules and short words") and record 69
called a timing question. No PR to master follows.

## What was built

`src/cells_first.rs`, research only, with a third writer in `examples/writer_owned.rs` (mode 4
runs it alone). It is not in master: the code is
[`bench/first-writer` at 835f297c](https://github.com/lmmx/mibz2/tree/835f297ca803d2ff4c6e56f5a8565e02880b2a71),
#587, closed unmerged as this record's evidence.
- **The walk:** the root's spelling is read from its left end, the output cursor starting at the
  rotation from the marked walk and wrapping once, after the marked byte.
- **A rule's first entry:** the current output offset is stored in its first field (the offset
  in the high half, the low half zero, which no genuine operand has). A short left operand is
  written at once, then the walk goes on to the right. A rule left operand is entered, the right
  one pushed.
- **A later visit:** reads the tag and copies at once. There are no deferred commands and no
  position table.
- **Short periods:** study 6 found the walk exact on short-period blocks, so its only guard is a
  root that is a rule. The cycle is then repeated to the block.
- **The stack:** bounded by its capacity, the rule count (R − 1 is the bound).
- **Memory safety,** on the premise the shipped writer rests on: the cursor stays below the
  period, an arena write is at most 16 bytes at a cursor below it, and a copy is made only where
  both its source and destination lie within the period.

**Tests,** all against the dense LF oracle, eager and lazy:
- every binary column through 10 rows and ternary through 7, at every row. Every block whose
  cycle has 2 rows or more takes the walk, short periods included. A one-row cycle's word is the
  marked byte itself, and falls back;
- the comb, L = 1^(n−1) 0 at row n − 2, n up to 5,000: n − 1 rules, and the stack peaks at
  exactly n − 2;
- the source-text BWTs whose copies depend on copies;
- `miri_cells_first_writer`, native here; the walk does not reach the shipped path.

## Measured

**The method:** the cloud container (Intel Xeon, 2.1 GHz), as #581's step 1.
- The fastest of 7 reps a block, the three writers interleaved.
- Before every run, uncharged, the output is poisoned and the rules restored, then each writer
  runs back to back.

| corpus | first-occurrence ÷ one-scan | ÷ shipped | per block (÷ one-scan): geometric mean, range | blocks faster |
|---|--:|--:|---|--:|
| Wikidata, eager (196 blocks) | **1.789** | 1.413 | 1.788, 1.125–2.259 | 0 |
| Wikidata, lazy (228 blocks) | **1.805** | 1.375 | 1.787, 1.351–2.376 | 0 |
| frozen block, eager | 1.647 | 1.317 | | |
| frozen block, lazy | 2.147 | 1.549 | | |

- **Bytes:** identical to the shipped writer's on every block (0 differing).
- **The deepest stack:** 23 on the frozen block (study 6's figure) and 37 over all of Wikidata.

**Callgrind,** each writer once a block, over all Wikidata blocks.
- *The setup:* simulated D1 of 32 KiB, 8-way, and LL of 20 MiB; inclusive of `memcpy` and
  `memmove`.
- *The pairing:* mode 4's profile holds the first-occurrence writer (compared) and the one-scan
  writer (the shipped path) for the same blocks.

| | instructions | data reads | data writes | D1 read misses | D1 write misses | cond. branches | mispredicted |
|---|--:|--:|--:|--:|--:|--:|--:|
| eager, one-scan | 448.6 M | 83.6 M | 61.9 M | 3.90 M | 8.27 M | 94.0 M | 1.83 M |
| eager, first-occurrence | 384.8 M | 84.6 M | 34.1 M | 9.75 M | 2.74 M | 68.0 M | **11.04 M** |
| **ratio** | 0.858 | 1.012 | 0.552 | **2.502** | 0.331 | 0.723 | **6.019** |
| lazy, ratio | 0.860 | 1.008 | 0.553 | **2.473** | 0.315 | 0.723 | **5.768** |

**Why it loses:**
- *The branches.* At each visit the walk takes one of four paths: an arena word, a tagged rule
  (a copy), a first entry with a short left operand, or one with a rule left operand. The
  sequence follows the grammar's shape, which the predictor cannot learn. There are 11.0 M
  mispredictions against 1.8 M, about 1.7 a rule against 0.3. At 15–20 cycles each, the 9.2 M
  extra cost about as much as the one-scan writer's whole run.
- *The reads.* The walk chases records in spelling order, rather than scanning them in birth
  order. D1 read misses rise 2.5×, as study 6's line model warned (rule fills rose from 143,262
  to 315,205, arena fills from 67,493 to 155,149). Its 5.70% net fill saving came from output
  writes. Here those fall by two thirds, but they are not the bottleneck.
- *The rest favours the walk,* by 14% in instructions and 45% in writes, but those cannot pay
  for the stalls.

## Disposition

**Stopped under the checkpoint's rule:** not faster than #581 on Wikidata, eager or lazy. The
one-scan writer stays.
- **What would be needed:** to compete, the walk would have to remove most of its branch
  mispredictions, for instance by a branchless dispatch over the four visit kinds, and hide its
  dependent reads. Even at the one-scan writer's misprediction rate, it would keep 2.5× the read
  misses.
- **This record doesn't pursue it.** The research code stays on its branch (#587, closed
  unmerged) as this record's evidence.

## Reproducing

From a checkout of `bench/first-writer` at 835f297c:

```sh
cargo build --release --features owned-writer --example writer_owned
target/release/examples/writer_owned 0 7 eager frozen.bz2 WIKIDATA.bz2 > ab-eager.jsonl
target/release/examples/writer_owned 0 7 lazy frozen.bz2 WIKIDATA.bz2 > ab-lazy.jsonl
for k in eager lazy; do
  valgrind --tool=callgrind --cache-sim=yes --branch-sim=yes --I1=32768,8,64 \
    --D1=32768,8,64 --LL=20971520,20,64 --callgrind-out-file=cg-$k-m4.out \
    target/release/examples/writer_owned 4 1 $k WIKIDATA.bz2
done
```

- **The first-occurrence writer:** `write_first`, inclusive.
- **The one-scan writer:** `write_owned`, inclusive, in the same profile: the shipped path's own
  call.
