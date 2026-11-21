# bench-open-lanes: registration

*21 November 2026. Written and frozen before any data, on `bench/open-lanes`. The recipe is
`just bench-open-lanes` (`.just/bench.just`); the verdict is `bench/open_lanes.py`'s, tested by
`bench/test_open_lanes.py`. The recipe, the rule and the verdict script are bench-open-vec16's
(#669, `docs/journal/2026-11-21-open-vec16-registration.md`, adopted at
`open-vec16/run-20261121T004904Z`), with only the builds changed.*

**The decision it settles:** whether #675 (`research/open-bp4k`) merges, that is, whether the
Open kernel's eight-lane walk into 4 KiB pages and its R4 endpoint elimination at LIMIT 24 reach
the decoder's default on the i9. It is an ordinary merge under PERFORMANCE-POLICY.md, measured on
the one machine where it can show: CI's bench never runs Open
(`docs/journal/2026-11-20-open-stage-attribution.md`).

## What is compared

- **Head:** `research/open-bp4k` at `8ec1a92ced6e03200561c133b16fc88e07a053d5` (#675). It is
  master's src plus two commits: the walk in eight lanes, `Pages` (BP4K: one pass, the marked
  segment in place, the others into 4 KiB pages of dead `out` or an overflow, moved in Kahn
  order) from 4,096 retained rows (80f9cb51); and R4-DGIM's `prev`-only tapes with rule R4's
  overwrite, words of at most 16 bytes straight into `inline16`, and LIMIT 24 with a 24-row
  queue fill (e53fc0a2). A third commit (8ec1a92c) is its journal.
- **Base:** master's src, `70b18ca4411fa10c9f29b80821c835358dde2f9e` (#668, the Vec16 queues;
  v0.5.8's source). Master's head differs from it in no file under `src/`, and in `Cargo.toml`
  and `Cargo.lock` only by the version number (0.5.7 to 0.5.8), so the base is master's decoder.
  It is #675's merge-base with master, so the head's history contains it.
- **The contrast, checked by the recipe:** the two builds' `src`, `examples`, `Cargo.toml` and
  `Cargo.lock` differ in `src/open.rs` and files under `src/open/` alone, or the run stops. (At
  these SHAs: `src/open.rs`, `src/open/b5.rs`, `src/open/checks.rs`, and the new
  `src/open/lanes.rs`, `src/open/lanes_checks.rs` and `src/open/r4.rs`; the list is kept in the
  bundle as `src-changed.txt`.) Both are built by rustc 1.99.1, and the head's harness drives the
  run.
- **What it predicts:** on one thread in a container, the head's Open is 0.872× of #668's
  [0.820, 0.928] over the whole corpus, and front+Open 0.879× (#675's journal,
  `docs/journal/2026-11-21-open-lanes.md` on its branch). Open is most of a lazy decode's compute
  at one thread, so a whole-decoder gain is expected, smaller with more workers sharing memory
  bandwidth.
- **Memory:** the page pool reserves about 256 KiB more `out` per worker (`out` is reserved at
  n + 16 + 256 KiB; on the admitted blocks the overflow takes 184–216 KiB). At 10 workers that
  is about 2.5 MiB; on a peak RSS of tens of MB per run it should stay under the 1.10 gate, but
  that is what the memory gate checks.

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

- **ADOPTS:** the step at 10 threads is FASTER (its interval lies below 1), neither 6 nor 8
  threads is SLOWER, and at each of 6, 8 and 10 threads the median peak RSS over the rounds,
  head over base, is at most 1.10.
- **HELD ON MEMORY:** as ADOPTS, but peak RSS above 1.10 at some count. The Coordinator weighs
  the growth under PERFORMANCE-POLICY rule 4. (The page pool's extra 256 KiB of `out` a worker is
  the memory change expected.)
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
here, so the same blocks take Open in both builds. The lanes start from 4,096 retained rows, so
a block below that takes the serial walk in the head too; R4 and LIMIT 24 apply to every block
Open takes.
