# The open kernel's dense admission: the Mechanisms line's cycle 19, ported to 0.5.10

The Mechanisms line's cycle 19 candidate (`AhtEdgesMux16`,
`docs/journal/2026-11-21-carbon-cycle19-unified-synthesis.md` on
`research/carbon-word-transport`, built on cycles 17 and 18) was measured against 0.5.8. Master
is 0.5.10, which already ships the Open memory diet (#692): the queues in `out` (OutQ16) and the
elimination's nodes and rules in `edges`. This port takes only what 0.5.10 lacks, in three
commits, onto this kernel's own construction (its `endpoint.rs` elimination, not the cycle's
R4):

1. **The dense band.** `OpenCells` also opens run-route blocks above `LazyCells`' density line,
   up to r/n 0.28, when the first pass finds at most min(256,000, 0.3 n) retained rows and
   16,384 thick strips (the cycle 18 structural gate). The queues check the cap in their cold
   path only. Past it a sticky flag stops every further chunk, and the block goes to
   `LazyCells` as before. The blocks the count gates admit keep no cap and the same route, and
   the sparse gate stays.
2. **Spirals past a width certificate.** A spiral is a run of steps that peels one node against
   the same partner. It is contracted to one step: a power by squaring for the word and q times
   the partner's width for the width (the cycle 18 contraction, Theorem 19.2a's certificate).
   Blocks whose nodes are all at most 32,768 rows wide run the plain loop unchanged.
3. **EdgesMux's remainder.** The strips' image order (the cycle's `u32_scratch`) is kept in
   `edges`' first `ns` words, before the arena. The cycle's `tops` and `bots` are this
   kernel's nodes, in `edges` since #692.

## Baseline

The corpus (`corpora/wikidata.bz2`, 82f65ace…) has 431 blocks: 383 on the run route (233 full,
150 tiny, of 4 to 1,185 rows) and 48 on the column route. 0.5.10 opens 228 of them, every one
`LazyCells`' count gates admit. It refuses five full blocks between the lines (41, 56, 165, 270
and 278, r/n 0.151 to 0.206), which go `LazyCells` → Segmented, and every tiny one. The six
authenticated blocks (`data/admitted-bwt` 41c03a5b, each checked against `manifest.json`'s
SHA-256) are all among the admitted 228. So the dense blocks this change is about are the five.

## After

The kernel opens 233 blocks: all 233 full run-route blocks. The five are opened within the cap
(199,189 to 231,600 retained rows against 256,000, and 568 to 2,729 strips). No block of the
corpus fails the width certificate, so the contracting loop never runs on it.

The tiny blocks stay where they were, as do the cycle's MicroLF route (n ≤ 2,048) and its
claim of 383. Their fallback is a plain walk of at most 4.7 KiB. This kernel's queues, on a
block under a chunk, are all overflow chunks, which the kernel then keeps.

## Measured (this container: 4 vCPUs; not the i9)

`results.txt` has every table. The harness (`admit_harness.rs`, `build.sh`, `drive.py`) is
#692's in-process A/B:
- master's src/open.rs, endpoint.rs and outq.rs are kept unchanged in the same test binary as
  `admit_ref`;
- decodes are alternated;
- `m0.bin`, master's tree built as the candidate, is the A/A control.

Heap is a thread-local count of a pass-through to the system allocator, in the measurement
binary only.

- **Per-worker kernel heap**, each worker taking its round-robin share of the 431 blocks:

  | Workers | Max per worker, KiB | Sum, KiB |
  |---|---|---|
  | 4 | 7,128.7 → 1,342.0 (0.188×) | 22,337 → 5,040 |
  | 10 | 6,804.5 → 1,268.8 (0.186×) | 39,401 → 11,548 |

  A worker that met a refused block held Segmented's T-vector and scratch (5.6 MiB). The image
  order's move took 27–28 KiB off the maximum.
- **One thread, admitted blocks:** no change. The six are at 1.003–1.009, against an A/A of
  0.993–1.019.
- **One thread, the five newly admitted blocks:** slower. The open kernel takes 1.12–1.43× the
  time Segmented took (block 165 at 1.427 [1.413, 1.440], block 270 at 1.120 [1.111, 1.128]).
  The A/A is within 0.987–1.019.
- **Four concurrent workers over the whole corpus** (its own mix of admitted and refused
  blocks):
  - the five blocks: 0.925 [0.909, 0.942], against an A/A of 1.004 [0.994, 1.015];
  - a whole pass: 1.003 [0.991, 1.016], against an A/A of 1.007 [1.000, 1.013]. The five are
    under 4% of a pass.
- **End to end** (`examples/bzcat`, fresh processes, `e2e.py`):

  | Threads | Peak RSS | Wall |
  |---|---|---|
  | 8 | 0.741 [0.714, 0.768] | 0.978 [0.928, 1.031] |
  | 10 | 0.783 [0.754, 0.812] | 1.002 [0.957, 1.050] |
  | 4 (not `OpenCells` off the i9: the decoder's A/A) | 1.002 | 0.944 [0.867, 1.029] |

## Parity

Every block of the corpus and the six decode byte for byte as the plain inverse does, through
reused and fresh kernels, in both arms. `bzcat` at 4, 8 and 10 threads gives `bzip2 -dc`'s
SHA-256 (3f075e30…). The moonshot correctness gate passes (338 cases, 0 failed), as does
`cargo test --release`.

New tests (`dense.rs`, `spirals.rs`, `mux.rs`) cover:
- the gates' edges;
- the cap exactly: taken at a block's own counts, refused one below either;
- the sticky refusal: no chunk is taken past the cap;
- the route against the uncapped counts;
- the contracted loop's lists, widths, bands and words against the plain loop's, on spirals,
  arbitrary columns and every small column;
- the certificate's edge;
- each region of `edges` poisoned after its last read (exact) and before it (the controls
  break).

Two or three hand mutations per commit were each caught:
- `>=` for `>` at the cap;
- the sticky check removed;
- the strip cap removed;
- the spiral's partner check dropped;
- its count off by one;
- its word order swapped;
- the order overlapping the arena;
- the order misplaced.

## What did not transfer

- **Speed on the dense blocks.** The cycle's 0.42–0.46× cycles and 0.02–0.06× DRAM on blocks
  40..278 are chaingrind's contention proxy at 2–2.5 MiB of LLC a worker. Natively, here, the
  open kernel loses to Segmented on those blocks at one thread and gains 7.5% at four workers.
  This container's cache is not the i9-10900K's 20 MB shared L3. Only CI's bench and an i9 run
  settle the contended case.
- **Heap.** The cycle's 9,979 → 1,346 KiB was against 0.5.8. On 0.5.10 the diet already took
  the admitted blocks' scratch to about 1.3 MiB, so the gain here is the refused blocks'
  Segmented fallback: 6.8–7.9 MiB → 1.27–1.35 MiB a worker.
- **EdgesMux's 432 KiB.** Most of it is on master since #692 (the nodes in `edges`). The image
  order is the remainder, at 27–28 KiB.
- **The contracted loop** never runs on Wikidata. It is a bound on spirals for the blocks the
  kernel admits, not a gain. The sparse gate stays, where the cycle dropped it.
- **Not ported:** SentEmit, `L = 14`, MicroLF. None of them is the admission.
