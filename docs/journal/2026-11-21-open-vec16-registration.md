# bench-open-vec16: registration

*21 November 2026. Written and frozen before any data, on `bench/open-vec16`. The recipe is
`just bench-open-vec16` (`.just/bench.just`); the verdict is `bench/open_vec16.py`'s, tested by
`bench/test_open_vec16.py`.*

**The decision it settles:** whether #668 (`research/open-vec16`) merges, that is, whether the
Open kernel's Vec16 queue reaches the decoder's default on the i9. It is an ordinary merge under
PERFORMANCE-POLICY.md, measured on the one machine where it can show: CI's bench never runs Open
(`docs/journal/2026-11-20-open-stage-attribution.md`).

## What is compared

- **Head:** `research/open-vec16` at `90ff6bfbbad43dd1088f08e5d2e0781178a0235f` (#668). It is
  master's merge-base plus #664's two commits (the fixed 16-row queue fill and the 16-byte emit)
  and #666's Vec16 queue, without #666's fused front (`invert_symbols`, `open::Fused`, the
  `runs.rs` split), which lost natively.
- **Base:** master's merge-base, `ad65b3b87fdac41cfaa65654edba4c9e654c53c6`.
- **The contrast, checked by the recipe:** the two builds' `src`, `examples`, `Cargo.toml` and
  `Cargo.lock` differ in `src/open.rs`, `src/open/b5.rs` and `src/open/checks.rs` alone, or the
  run stops. Both are built by rustc 1.99.1, and the head's harness drives the run.
- **What it predicts:** on one thread in a container, the head's Open is 347–359 ms against
  433–441 ms on master over the whole corpus (#666's journal). On the lazy route Open is about
  75% of a decode's compute at one thread, so a whole-decoder gain is expected, smaller with
  more workers sharing memory bandwidth.

## The run

- **The machine:** the maintainer's i9-10900K (10 cores, 20 CPUs, no CPU quota), under C5.
  Any other machine voids the run.
- **The corpus:** `corpora/wikidata.bz2`, the whole bench corpus (SHA-256 `82f65ace…`, 431
  blocks, 209.7 MB decoded), checked by its digest. Four copies end to end make the timed file
  (`wikidata-x4`), as the i9 recipes do, so that each run is long against process start.
- **The cells:** `ab run --arms mibz2@build-base,mibz2`, production's decoder at each build,
  unpinned on every CPU (no `--spread`, no spinner) at 6, 8 and 10 threads. That is where
  production runs `OpenCells` on the i9: 6 lazy workers at 6 threads, 8 at 8, and 7 from 10
  threads (`trimmed`, unpinned). 10 threads is production's default on the i9.
- **Rounds:** 16, each running both arms at each count, interleaved, the arms' order rotated
  (`arm_order`); a 2 s settle gate before each run.
- **The diagnostic** (decides nothing): `mibz2-workers-lazy` at each build, the workers' lazy
  route with plan and window fixed, at 6 and 8 threads, 16 rounds. It is reported as steps only.
- **Cost:** about 15–20 minutes of workstation time with the two builds, below PERFORMANCE-POLICY
  §4a's 30-minute line.

## The rule

A count's **step** is the 16 rounds' paired wall ratios, head over base: their geometric mean and
its 95% interval (Student's t on the logs, 15 degrees of freedom). FASTER if the interval lies
below 1, SLOWER if above, else INCONCLUSIVE.

- **ADOPTS:** the step at 10 threads is FASTER (its interval excludes no change), neither 6 nor
  8 threads is SLOWER, and at each of 6, 8 and 10 threads the median peak RSS over the rounds,
  head over base, is at most 1.10.
- **HELD ON MEMORY:** as ADOPTS, but peak RSS above 1.10 at some count. The Coordinator weighs
  the growth under PERFORMANCE-POLICY rule 4. (The Vec16 queues keep their buffers at full
  length, so a memory change is possible.)
- **NOT ADOPTED:** otherwise: 10 threads not FASTER, or 6 or 8 SLOWER.
- 6 and 8 threads are **supporting**: reported with their intervals, and only a loss there
  counts against adoption.
- **VOID**, with no verdict: the machine not the i9's profile; any run unsuccessful, with a
  non-finite or non-positive wall, CPU or peak RSS; any run pinned or beside a spinner; any
  record duplicated, unexpected or missing; the corpus not `wikidata-x4`; any output not
  838,800,784 bytes, or the outputs' hashes not all one. A VOID run may be repeated once, with
  the reason logged; a verdict is never re-run to change it.
- CPU time is reported beside each step. It decides nothing.

**Limits:** the corpus is Wikidata alone, the decoder's primary workload and where Open admits
97.9% of the rows. `text` is not in this run. Open's admission is the density line, unchanged
here, so the same blocks take Open in both builds.
