# bench-block-reader-on-wormhole: registration

*22 November 2026. Written and frozen before any data, on `bench/block-reader-on-wormhole`. The
recipe is `just bench-block-reader-on-wormhole` (`.just/bench.just`). The verdict is
`bench/block_reader_on_wormhole.py`'s, a thin wrapper around `bench/parscan_c37.py`
(`--decision block-reader`), tested by `bench/test_block_reader_on_wormhole.py`. The recipe, its
checks, the arms and the rule are bench-parscan-c37's Decision 2, BLOCK-READER
(`docs/journal/2026-11-22-parscan-c37-registration.md`), unchanged. Only the builds differ.
Decision 1 is left out.*

**The decision it settles:** whether #745 (`perf/block-reader-c37`, the default path's block
reader from cycle 37) still holds on master now that Wormhole's plateau stack (#739, merged at
`3296195e`) is in it. Both were measured on the i9, but separately:
- the block reader on `96a24d3e`, in bench-parscan-c37's Decision 2: ADOPTS, 1 thread 0.977
  [0.968, 0.987], 6 0.976, 8 0.884, 10 0.932 [0.923, 0.941], peak RSS within 1.02 (evidence
  `5fed1ffc`, `parscan-c37/run-20261122T082138Z`);
- #739 alone, in bench-wormhole-plateau.

Composition has to be shown, not assumed. #745 merges only on this run's ADOPTS.

## The hypothesis

**What changed.** #745's head `7b901b67` is #741's block reader with master `3296195e` merged
in. Against that master, the decoder's source differs only in these files:
- `src/huff.rs` (new);
- `src/bits.rs`, `src/block.rs`, `src/stream.rs` and `src/runs.rs`, each byte-identical to
  `d57ec321`, the build Decision 2 measured;
- one line in `src/lib.rs` (`mod huff;`).

The merge with #739 was automatic: #739 touched none of the block reader's files. In `src/lib.rs`
the two changes are in different places. Every file #739 changed is master's, byte for byte, at
#745's head. The recipe checks the contrast.

**Why it should compose.** The block reader works in the serial scanner: block headers and
Huffman symbols. #739 works on the workers' side and the output: run-route output, in-place
RLE1, tiled CRC, the 12 MiB ready cap and the ready ring. They touch different stages. At 1 thread
the decoder runs both in one thread, so the 1-thread gain should carry as before.

From 6 threads up, the scanner is a serial stage beside the workers. #739 made the workers and
the output faster, which makes the scanner's share of the wall larger. So the block reader's
gain at 10 threads may be as large as before or larger, if the scanner bounds the wall more
often. It may also be smaller, if #739 moved the bound elsewhere, to the ready ring or the
writer. That is what the composition report shows. It is a hypothesis.

**The risk.** On Wormhole's master the parallel counts' gains (0.884 at 8, 0.932 at 10) may
shrink to noise. They are supporting counts, so that would not block adoption; only a loss
would. A loss at 6 to 10 would mean the two changes interact badly, for example in the scanner's
handoff into the ready ring.

## What is compared

- **Head:** #745's head, `perf/block-reader-c37` at
  `7b901b6789d98158d8f006e061599214365dc44b`, arm `mibz2`.
- **Base:** master at `3296195e3104967d800e105d44c4c43d7fc01f14` (#739's merge), arm `mibz2`
  (`mibz2@build-base`). It is an ancestor of HEAD, and the recipe checks that.
- **The contrast, checked by the recipe:** the builds' `src`, `examples`, `tests`, `Cargo.toml`,
  `Cargo.lock`, `build.rs` and `rust-toolchain.toml` may differ only in `src/lib.rs`,
  `src/bits.rs`, `src/block.rs`, `src/huff.rs`, `src/runs.rs` and `src/stream.rs`, or the run
  stops. At these SHAs, exactly those six differ. The bundle keeps the list and the diff.
- **Should master move before the run:** the run still measures #745 against `3296195e`, and the
  Coordinator weighs any later master change to `src` at the merge.

## The run

- **The machine:** the maintainer's i9-10900K (10 cores, 20 CPUs, no CPU quota), under C5.
- **The corpus:** `corpora/wikidata.bz2`, checked by `bench/wikidata.json`'s SHA-256, four
  copies end to end (`wikidata-x4`).
- **The cells:** `ab run --arms mibz2@build-base,mibz2` at 1, 6, 8 and 10 threads. Above one
  thread unpinned on every CPU, with no spinner; at one thread on ab's one-CPU pin, both arms
  alike.
- **Rounds:** 16 paired rounds, interleaved, the arms' order rotated, with a 2 s settle.
- **Cost:** about 15 minutes with the two builds.

## The rule

Decision 2's (R4's step), unchanged. A count's **step** is the 16 rounds' paired wall ratios,
head over base: their geometric mean and its 95% interval (Student's t on the logs). It is FASTER
if the interval lies below 1, SLOWER if above, and INCONCLUSIVE otherwise.

- **ADOPTS:** all three hold:
  - the step at 1 thread is FASTER (its interval's upper end below 1.0);
  - none of 6, 8 and 10 threads is SLOWER;
  - at 10 threads, the median peak RSS over the rounds, head over base, is at most 1.10.
- **HELD ON MEMORY:** as ADOPTS, but with peak RSS above 1.10 at 10 threads. The Coordinator
  weighs it under PERFORMANCE-POLICY rule 4.
- **NOT ADOPTED:** any other outcome: 1 thread not FASTER, or any of 6, 8 and 10 SLOWER.
- **VOID**, with no verdict, if any of these holds:
  - the machine is not the i9's profile;
  - any run is unsuccessful, or has a non-finite or non-positive wall, CPU or peak RSS;
  - a run above one thread is pinned, a one-thread run is not on a single CPU, or a run is beside
    a spinner;
  - any record (arm, threads, round) is duplicated, unexpected or missing;
  - the corpus is not `wikidata-x4`;
  - any output is not 838,800,784 bytes, or the outputs' hashes are not all one.

  A VOID run may be repeated once, with the reason logged. A verdict is never re-run to change
  it.
- **Reported beside each step, deciding nothing:** CPU time, and the median peak RSS at the base
  and the head in MiB, with their difference.
- **The composition, reported and deciding nothing:** the 10-thread step here, g [lo, hi],
  beside Decision 2's 0.9320 [0.9233, 0.9407] on `96a24d3e`, and g / 0.9320 [lo / 0.9320,
  hi / 0.9320]. The interval is this run's alone; the prior is treated as fixed. Near 1, the
  block reader's 10-thread gain is the same on Wormhole's master: the two gains compose,
  multiplying. Above 1, Wormhole's stack absorbed part of it. Below 1, the two reinforce.

**Limits:**
- **The corpus:** Wikidata alone.
- **What a result separates:** an ADOPTS shows that the block reader's gain holds on master with
  #739. It does not separate the block reader from `takes_runs`' early exits, which ship
  together. The combined stack against the last release is the release-scaling run's, not this
  one's.
- **The head's branch:** the recipe fetches HEAD from `perf/block-reader-c37` if the workstation
  lacks it. A later push to #745 is not what this run measures, and needs its own pin.
