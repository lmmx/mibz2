# bench-emit-cost: registration

*22 November 2026. Written and frozen before any data, on `bench/emit-cost`. The recipe is
`just bench-emit-cost` (`.just/bench.just`); the report is `bench/emit_cost.py`'s, tested by
`bench/test_emit_cost.py`. The workstation conventions (one new timestamped directory per attempt,
its console log, the machine's records, `bench/bundle.sh`, nothing in the home directory) are
bench-open-stack's (#720).*

**Diagnostic only.** This run decides no production default and no merge. Its one use is to
choose which research candidates for the open kernel's emit stage are worth porting: LazyTunnel,
BSAC, both or neither.

## The question

On the i9-10900K (256 KiB of L2 a core, 20 MiB of shared L3), what is the open kernel's emit
time per walked step, that is, per retained row (`m` at `LIMIT` 16, "m16"), at 1, 6 and 10
decoding workers, on master?

Research measured Emit(Q) = a + b·Q with b = 4.2 ns a step on a Xeon with 2 MB of L2, where
the walk's `edges` array fits in L2. On the i9, `edges` (351 to 798 KiB a block) spills L2, so b
there is unknown. The candidates both cut the emit's cost per step, at a porting cost that differs
between them. Whether either is worth that cost depends on b on the machine where production runs
`OpenCells`.

## What is measured

**The build:** one build, the recipe's own checkout at its ready tag. It is master at
`69961392739a455561cbdfe544a1529c53c57fbe` (the PIN) plus:

- the research feature `emit-cost` (`src/emit_cost.rs`, with four feature-gated statements in
  `Kernel::try_run`, `src/open.rs`, and a gated `pub mod` in `src/lib.rs`). The one ungated
  change there is the construction's `?` written as `let … else { return None }`, the same code;
- its harness `examples/emit_cost.rs`;
- the report, its test, and this file.

The recipe checks that PIN is an ancestor of the checkout and that the source differs from PIN
in `src/open.rs`, `src/lib.rs` and `src/emit_cost.rs` alone. It writes that diff into the bundle.
Without the feature the kernel compiles exactly as at PIN. With it, each block the open kernel
takes reads the clock three times, before the construction, between the construction and the
emit, and after the emit. It then pushes one record onto a mutex-guarded vector: two
`Instant::now()` calls and an uncontended lock a block, a few hundred nanoseconds against an emit
of about a millisecond. The recipe builds with rustc 1.99.1.

**No existing instrumentation fits.** `phase-counters` counts Cells' induction and writer, not
the open kernel. `walk-stats` covers the cooperative teams' walks. `trace` is the parallel
decoder's timeline, block by block, not stage by stage. `moonshot` only exposes the run records.
The stage-attribution harness (`docs/journal/2026-11-20-open-stage-attribution/attrib.rs`) times
the whole inverse on one thread, construct and emit together. #719's per-stage counters
(`docs/journal/2026-11-22-open-construction/build_harness.py`) are source patches applied to a
disposable worktree, single-block and outside the decoder. The open kernel is a private module,
so construct and emit can only be split from inside it.

**The workload:** the whole Wikidata corpus (`corpora/wikidata.bz2`, checked by its SHA-256),
four copies end to end (`wikidata-x4`, 838,800,784 decoded bytes). It is read into memory once,
so no decode touches the disk. Each decode runs production's lazy workers,
`BzReader::parallel_workers(data, w, SLOTS_PER_THREAD × w, Kernels::detect(), true)`. The kernel
is `OpenCells`, the window production's, and every worker decodes its own blocks concurrently:
the decoder's own scanner, workers and window, with only the worker count forced. That is
`ab`'s `mibz2-workers-lazy` arm.

**The grid:** w = 1, 6 and 10 workers, unpinned on every CPU. There are 20 rounds. Each round
decodes once at each count, the order rotated round by round, with 2 s of sleep before each
decode. That makes 60 decodes, about 4 minutes on the i9, and about 7 with the build.

**Per block** (every block the open kernel takes, in each decode):

- `n` rows;
- `m` retained rows;
- `construct_ns`, the wall time of `construct_capped`: the first pass, the elimination and the
  dictionary;
- `emit_ns`, the wall time of `walk`, forward or two-ended, with `out`'s resize and the repeat
  of a short cycle;
- which emitter ran.

A block the first pass refuses (past `Cap::dense`, then `LazyCells`) records its refused pass with
`m` 0 and is excluded from b. The walk visits each retained row once on the marked cycle, so its
step count Q is m. The exception is a periodic block, whose marked cycle is shorter than m: it
walks fewer steps and repeats its period. The recorder does not count steps, so that the emit
loop stays as shipped. Such a block can only lower b, and periodic blocks are rare in Wikidata's
JSON. The harness writes one line per decode with its wall time and output bytes.

## The report

The report comes from `bench/emit_cost.py`, standard library only.

- **b at each count:** for each decode, b = Σ emit_ns / Σ m over its open blocks. The report
  gives the mean over the 20 rounds and its 95% interval (Student's t).
- **The regression at each count:** emit_ns = a + slope·m by least squares, pooled over every open
  block of every round, giving a in µs and the slope in ns a row. It also gives the per-round
  slopes' mean and 95% interval.
- **Construct at each count:** µs a block, and ns a block row (Σ construct_ns / Σ n), each with
  its 95% interval over rounds.
- **The rest:** the decodes' median wall, the open, two-ended and refused block counts, and the
  retained rows a decode.

## The thresholds (frozen before the data)

At 10 workers (the judged count), b's mean over rounds decides:

| b at 10 workers | Class |
|---|---|
| b > 15 ns | **PORT BOTH** (LazyTunnel and BSAC) |
| 8 ns ≤ b ≤ 15 ns | **PORT THE CHEAPER** only |
| b < 8 ns | **SHELVE BOTH** |

- **An interval across a threshold:** if b's 95% interval contains 8 or 15, the class is marked
  STRADDLES. The mean still decides.
- **The pooled slope:** its band is reported beside the class. b is Σ emit / Σ m as the question
  defines it, which carries the intercept a spread over the rows. Where the slope's band differs
  from b's, b still decides, and the report shows both.
- **1 and 6 workers:** they are reported and decide nothing. They show how much of b at 10 is
  contention for L3 and memory rather than the L2 spill alone.

**VOID** (no class, exit 1) if any of these fails:

- the machine is an i9-10900K (lscpu);
- every (round, workers) decode is present once, and no unexpected one;
- each decode wrote 838,800,784 bytes;
- each decode had at least one open block;
- the blocks' (n, m, refused) are the same multiset in every decode. The construction is
  deterministic whatever the worker count, so a difference means a broken record.

`bench/test_emit_cost.py` checks each class on synthetic records, along with the straddle mark,
the regression's recovery of a known a and slope, and each VOID case. CI runs it, and the
recorder's test (`cargo test --release --features emit-cost --lib emit_cost::`).

## The harness, checked

A smoke check on the development VM (a 4-CPU Xeon with 2 MiB of L2 a core, not the i9) proves
the harness works, and is not evidence for the class. It decoded one window (`w00`, 37 blocks,
22 taken by the open kernel, all two-ended), 1 round at 1 worker, with all 19,800,013 bytes
right.
