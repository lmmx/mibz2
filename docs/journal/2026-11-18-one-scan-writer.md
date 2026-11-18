# The one-scan writer: a native prototype, measured writer-only

*18 November 2026. The Pathfinder.*

- **The assignment:** #145 QUEUED 6986629119, dispatched as CHECKPOINT 6986665685 on #314, with
  the measurement addendum LOG 6986965565.
- **The study it implements:** study 5, #579, `docs/journal/2026-11-18-cells-live-reuse.md`,
  verified by the Researcher's REVIEW 6986491194 and MONODROMY's record 67 (REVIEW 6986559098).

**Step 1, the writer alone, passes.** On the same finished grammars, the one-scan writer takes
0.785 to 0.813 of the shipped writer's time on Wikidata, eager and lazy. Its bytes are identical
on every block of every corpus. The stop rule ("if the new writer isn't faster on identical
grammars, stop") does not apply, so step 2 follows: the writer in the shipped path, for CI's
paired bench.

## What was built

- **`src/cells_owned.rs`:** the schedule as study 5 states it.
  - *The guard:* full period (the cycle is the whole block) and the marked word the last stored
    rule. One test a block; any other block goes to the shipped `write_cells`, unchanged.
  - *The scan:* the stored rules, last to first. Each rule is placed at its position. Its arena
    children are written. Each rule child is either owned (its one position set) or, where the
    next stored rule carries the same descriptor on the same side, a deferred copy command.
  - *The commands:* stored over the rule records already read, from the high end down. The
    next rule's descriptors are carried in registers, never re-read (record 67's note).
  - *The copy pass:* the commands run in increasing parent order, each copying its child's bytes
    from the child's position.
  - *Memory safety:* every access the writer adds is checked locally, so no access depends on
    the schedule's proof (the file's safety argument).
- **The comparison harness** (research feature `owned-writer`, `examples/writer_owned.rs`).
  - *How it runs:* after each Cells induction, both writers run on that block's grammar,
    interleaved, the fastest of R reps each, with the rule records restored before every rep
    (uncharged, since the one-scan writer writes its commands over them). The bytes are
    compared, block by block.
  - *Kernels:* `eager`, `lazy`, `forced-eager` or `forced-lazy`.
  - *What it reports:* full-period and fallback blocks apart, and the one-scan writer's scan and
    command pass timed apart.
  - *Modes 1–3* run one writer, the other, or neither, for counter profiles.

## Correctness

- **The guard's reach:** every full-period block of two rows or more takes the one-scan
  schedule. The tests check this exactly, by counting each block's LF cycle through the primary.
  - *What else falls back:* blocks whose cycle is shorter than the block, the case study 5's
    proof does not cover, and one-row blocks.
  - *The word is the last rule:* this holds on every full-period block tested. No full-period
    block falls back.
- **The tests, all against the dense LF oracle:**
  - every binary column through 10 rows and every ternary column through 7, at every row, eager
    and lazy;
  - study 5's two witnesses at the 16-byte cutoff, every row: 11 0^33 (one cycle, a 17-byte
    copy from a later address) and 11 0^34 (period 18 in 36 rows, the fallback);
  - text-like and random columns of 64 to 70,000 rows on reused allocations;
  - **copies that depend on copies:** the BWT columns of 12–30 KB slices of the repository's
    own sources, whose long repeated lines nest surviving long words in the owned occurrences
    of others (`owned_writer_nested_copies`);
  - `miri_cells_owned_writer` under Miri.
- **On real data:** bytes identical on every block in the comparison below, with zero blocks
  differing in any corpus or kernel.

- **Mutation check** ([`mutate.py`](2026-11-18-one-scan-writer/mutate.py)): eight mutations of the writer, each
  alone, against the writer's tests:
  - the survival certificate inverted;
  - no wrap;
  - the commands run last parent first;
  - no command stored;
  - an owned position not set;
  - the root one place off;
  - the carried descriptors not updated;
  - the guard admitting a shorter cycle.

  The first round missed "the commands run last parent first": no small column nests copies.
  The nested-copies test was added for it, and every mutation is caught.

## Writer time

- **The machine:** the cloud container, an Intel Xeon at 2.1 GHz with 4 cores. The writer runs
  on one thread a block, so these are microarchitectural results, not the i9's.
- **The method:** the fastest of 7 reps a block, interleaved, summed over the blocks.
  - *Before every writer run, uncharged:* the output is filled with a poison byte, so no writer
    can pass on bytes the other left. Then the rule records are restored, last written, as an
    induction leaves them.
  - *Why that order:* poisoning after the restore evicts the rules, which the shipped writer
    reads twice. That biases the ratio towards the one-scan writer (0.75 against 0.81 in a
    trial).
- **The Wikidata sample** is the pinned 82f65ace… stream. Admission gives exactly study 5's
  populations: 196 eager blocks and 228 lazy, all full period.

| corpus | kernel | blocks | shipped | one-scan | ratio (sum) | per block: geometric mean, median, range | blocks slower |
|---|---|--:|--:|--:|--:|---|--:|
| Wikidata | eager | 196 | 73.29 ms | 59.56 ms | **0.813** | 0.811, 0.809, 0.639–0.926 | 0 |
| Wikidata | lazy | 228 | 103.65 ms | 81.35 ms | **0.785** | 0.789, 0.790, 0.640–1.052 | 1 |
| frozen block | eager | 1 | 297.0 µs | 236.8 µs | 0.797 | 28,710 rules, 9,628 commands | |
| frozen block | lazy | 1 | 289.0 µs | 231.3 µs | 0.800 | (study 5's counts exactly) | |

An earlier run without the poison fill gave the same picture: 0.812 and 0.792 on Wikidata,
0.824 and 0.821 on the frozen block.

**Forced Cells on the other corpora** (fastest of 5, before the poison fill; every block full
period except onebyte's second; re-checked with the poison, 0 blocks differing):

| corpus | eager ratio | lazy ratio |
|---|--:|--:|
| text | 0.700 | 0.708 |
| random | 0.717 | 0.709 |
| pattern | 0.640 | 0.635 |
| records | 0.692 | 0.690 |
| runs | 0.712 | 0.712 |
| onebyte | 0.554 | 0.571 |

**Periodic and fallback blocks, apart** (the addendum's item 2):
- *Across all these corpora,* one block falls back: onebyte's second, a 2-byte block whose
  cycle is shorter than the block. It costs 0.02 ms under both writers. The fallback is the
  shipped writer after one test, so the fallback path cannot regress.
- *On Wikidata* no block falls back.

## The two phases, and what the ledger left unpriced

**Wall time by phase** (the addendum's item 1), in-process timestamps at the phase boundary,
summed over every call:

| kernel | scan (and the walk to the row) | the deferred copy pass |
|---|--:|--:|
| eager | 74.9% | 25.1% |
| lazy | 77.2% | 22.8% |

**Counts by phase** (callgrind):
- *The setup:* simulated D1 of 32 KiB, 8-way; LL of 20 MiB, 20-way; branch simulation on.
- *What is counted:* inclusive of callees (`memcpy` and `memmove`), each writer once a block,
  over all Wikidata blocks.
- *These are simulated counts,* not the i9's.

| eager | instructions | data reads | data writes | D1 read misses | D1 write misses | cond. branches | mispredicted |
|---|--:|--:|--:|--:|--:|--:|--:|
| shipped | 597.8 M | 140.6 M | 139.3 M | 6.38 M | 8.00 M | 184.3 M | 2.63 M |
| one-scan | 517.4 M | 87.6 M | 137.2 M | 3.90 M | 9.49 M | 169.6 M | 1.84 M |
| · its scan | 325.1 M | 69.6 M | 25.3 M | 2.82 M | 4.48 M | 49.8 M | 1.72 M |
| · its copy pass | 89.1 M | 18.0 M | 8.9 M | 1.07 M | 3.40 M | 16.7 M | 0.11 M |
| **ratio** | **0.866** | **0.623** | 0.985 | **0.610** | **1.187** | 0.920 | **0.698** |

| lazy | instructions | data reads | data writes | D1 read misses | D1 write misses | cond. branches | mispredicted |
|---|--:|--:|--:|--:|--:|--:|--:|
| shipped | 673.1 M | 173.7 M | 105.2 M | 8.12 M | 8.71 M | 161.6 M | 3.41 M |
| one-scan | 570.7 M | 108.5 M | 102.2 M | 4.88 M | 10.57 M | 142.4 M | 2.36 M |
| **ratio** | **0.848** | **0.624** | 0.972 | **0.601** | **1.214** | 0.881 | **0.693** |

The rest of the one-scan writer's count, outside its two phases, is common to both writers:
the walk to the row, the position table's fill, and the output buffer's first sizing.

**The checkpoint's three unpriced items:**
- **Branch mispredictions on the certificate** (31% of long references survive): they do not
  cost the writer.
  - The scan mispredicts 1.72 M of 49.8 M branches.
  - The whole writer mispredicts 0.70× as often as the shipped one, whose minimum and NIL tests
    mispredict more.
- **The store and copy order** (36% of copies read from later addresses): this is where the new
  writer pays.
  - D1 write misses rise 1.19× eager and 1.21× lazy. The copy pass is 36% of them, its
    destinations scattered.
  - Read misses fall to 0.61×, so all D1 misses fall to 0.93× (13.39 M against 14.38 M, eager).
- **Register pressure:** `owned_scan` is 103 instructions with 25 stack references.
  - Most are saves and reloads around the `memcpy` call `copy_small` makes for an arena word of
    variable length, as the shipped writer makes too.
  - A 16-byte over-copy would avoid the call, but is unsafe here: the one-scan order is not
    increasing in position, so it would overwrite bytes already written.

**The bench method** (the addendum's item 3): no timed run follows an idle gap. Each block's
writers run back to back, interleaved, the fastest of the reps.

## What it does not show

- **Not the i9:** these are one machine's figures.
- **The writer's share of whole-kernel time** is study 5's i9 figure, 19% of eager kernel time.
  At 0.81× of it, the whole-kernel gain would be about 3.6%, which CI's paired bench (step 2)
  measures directly.
- **Contention at 7–10 threads:** not measured. The workstation run is step 3, if CI shows a
  gain.

## Reproducing

```sh
cargo build --release --features owned-writer --example writer_owned
python3 docs/journal/2026-11-18-open-return/frozen.py WIKIDATA.bz2 frozen.bz2
target/release/examples/writer_owned 0 7 eager frozen.bz2 WIKIDATA.bz2 > ab-eager.jsonl
target/release/examples/writer_owned 0 7 lazy frozen.bz2 WIKIDATA.bz2 > ab-lazy.jsonl
for k in eager lazy; do for m in 1 2; do
  valgrind --tool=callgrind --cache-sim=yes --branch-sim=yes --I1=32768,8,64 \
    --D1=32768,8,64 --LL=20971520,20,64 --callgrind-out-file=cg-$k-m$m.out \
    target/release/examples/writer_owned $m 1 $k WIKIDATA.bz2
done; done
```

- **The shipped writer's counts:** `write_cells` in mode 1 less `write_cells` in mode 2. Mode 1
  runs it twice a block, once compared and once shipped.
- **The one-scan writer's counts:** `write_owned`, `owned_scan` and `owned_copies` in mode 2,
  inclusive.
