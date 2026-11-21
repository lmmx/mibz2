# bench-open-r4: registration

*21 November 2026. Written and frozen before any data, on `bench/open-r4`. The recipe is
`just bench-open-r4` (`.just/bench.just`); the verdict is `bench/open_r4.py`'s, tested by
`bench/test_open_r4.py`. The recipe, the rule and the verdict script are bench-open-lanes's
(#676, `docs/journal/2026-11-21-open-lanes-registration.md`), themselves bench-open-vec16's
(#669), with only the builds changed.*

**The decision it settles:** whether #679 (`research/open-r4`) merges, that is, whether the
Open kernel's R4 endpoint elimination, alone, with the serial walk and LIMIT 16, reaches the
decoder's default on the i9. It is an ordinary merge under PERFORMANCE-POLICY.md, measured on the
one machine where it can show: CI's bench never runs Open
(`docs/journal/2026-11-20-open-stage-attribution.md`).

## Why this variant

#675 combined three changes: the walk in eight lanes into 4 KiB pages, R4-DGIM's tapes, and
LIMIT 24 with a 24-row queue fill. Natively, on one thread, the combination took Open to 0.872×
of 0.5.8's. On the i9 (`open-lanes/run-20261121T023127Z`) it was **NOT ADOPTED**:

| threads | step, head/base | label |
|---|---|---|
| 6 | 0.950 | FASTER |
| 8 | 1.049 | SLOWER |
| 10 | 1.012 | INCONCLUSIVE |

So the one-thread kernel gain reverses as workers multiply. The likely cause is contention for
the shared cache and memory bandwidth, from two sources:
- the lanes' eight concurrent walks and their pages;
- LIMIT 24's more retained rows and wider tables.

That cause is plausible, not shown (`docs/team/research-charter/CONTEXT.md`'s ledger).

**This run tests R4 alone, with the serial walk, at LIMIT 16.** R4 shrinks the elimination's
working set (16-byte top slots, 8-byte bottom slots, no `bottom` array), and it leaves the walk
and the retained rows as 0.5.8 has them. Of #675's parts, it is the one least likely to suffer
contention, and the one #675's ledger entry names as untested.

## What is compared

- **Head:** `research/open-r4` at `b1e34269feb61608ff28e26028bf9f199a1f87a7` (#679). It is
  master's src plus one commit, which ports e53fc0a2's R4 elimination
  (`src/open/r4.rs`, unchanged) with:
  - LIMIT 16 and the 16-row queue fill kept;
  - the serial emit kept;
  - no `lanes.rs` and no page pool;
  - `endpoint.rs` behind the test switch `Kernel::endpoint_lists`;
  - no `unsafe`.
- **Base:** master's head when the recipe was written,
  `bb98c9958c5d5158e3d23d1da1939b747eef6391`. That is v0.5.8's source with #678, the lazy kernel
  at 4 threads unpinned, which changes nothing at 6, 8 or 10 threads. It is #679's merge-base
  with master, so the head's history contains it.
- **The contrast, checked by the recipe:** the two builds' `src`, `examples`, `Cargo.toml`,
  `Cargo.lock` and `build.rs` may differ only in `src/open.rs` and files under `src/open/`,
  or the run stops.
  - At these SHAs the changed files are `src/open.rs`, `src/open/b5.rs`, `src/open/checks.rs`
    and the new `src/open/r4.rs`. The bundle keeps the list as `src-changed.txt`.
  - Both builds use rustc 1.99.1, and the head's harness drives the run.
- **What it predicts:** a whole-decoder gain smaller than the kernel's, and smaller still with
  more workers sharing memory bandwidth. Open is most of a lazy decode's compute at one thread.
  The native evidence is one thread in a shared container (#679): R4/master on the Open stage,
  over the whole corpus, in 6 interleaved rounds:
  - 0.946, 1.026, 0.954, 0.939, 0.977, 0.928;
  - median about 0.95, faster in 5 of 6 rounds.

  For comparison, #675 measured 0.925 for R4 with LIMIT 24 and the serial walk.
- **Memory:** LIMIT is unchanged, and `open::b5`'s worst-case scratch test passes with the R4
  tapes counted. The tapes are kept beside `endpoint.rs`'s scratch, which the head no longer
  fills outside the tests. No peak RSS change beyond noise is expected; the memory gate checks
  this.

## The run

- **The machine:** the maintainer's i9-10900K (10 cores, 20 CPUs, no CPU quota), under C5.
  Any other machine voids the run.
- **The corpus:** `corpora/wikidata.bz2`, the whole bench corpus (SHA-256 `82f65ace…`, 431
  blocks, 209.7 MB decoded), checked by its digest. Four copies end to end make the timed file
  (`wikidata-x4`), as the i9 recipes do, so that each run is long against process start.
- **The cells:** `ab run --arms mibz2@build-base,mibz2`, production's decoder at each build,
  unpinned on every CPU (no `--spread`, no spinner), at 6, 8 and 10 threads. Production runs
  `OpenCells` on the i9 at each of these:
  - 6 threads: 6 lazy workers;
  - 8 threads: 8 lazy workers;
  - 10 threads (production's default on the i9): 7 workers, `trimmed`, unpinned.

  Since #678, production also runs `OpenCells` at exactly 4 threads unpinned. That count is
  not in this run, which keeps bench-open-lanes's grid so the two results compare directly.
- **Rounds:** 16. Each round runs both arms at each count, interleaved, with the arms' order
  rotated (`arm_order`) and a 2 s settle gate before each run.
- **The diagnostic** (decides nothing): `mibz2-workers-lazy` at each build, the workers' lazy
  route with plan and window fixed, at 6 and 8 threads, 16 rounds. It is reported as steps only.
- **Cost:** about 15–20 minutes of workstation time with the two builds, below
  PERFORMANCE-POLICY §4a's 30-minute line.

## The rule

A count's **step** is the 16 rounds' paired wall ratios, head over base: their geometric mean
and its 95% interval (Student's t on the logs, 15 degrees of freedom). A step is FASTER if the
interval lies below 1, SLOWER if it lies above 1, and INCONCLUSIVE otherwise.

- **ADOPTS:** all three hold:
  - the step at 10 threads is FASTER (its interval lies below 1);
  - neither 6 nor 8 threads is SLOWER;
  - at each of 6, 8 and 10 threads, the median peak RSS over the rounds, head over base, is
    at most 1.10.
- **HELD ON MEMORY:** as ADOPTS, but with peak RSS above 1.10 at some count. The Coordinator
  weighs the growth under PERFORMANCE-POLICY rule 4.
- **NOT ADOPTED:** any other outcome: 10 threads not FASTER, or 6 or 8 SLOWER.
- 6 and 8 threads are **supporting**: they are reported with their intervals, and only a loss
  there counts against adoption.
- **VOID**, with no verdict, if any of these holds:
  - the machine is not the i9's profile;
  - any run is unsuccessful, or has a non-finite or non-positive wall, CPU or peak RSS;
  - any run is pinned or beside a spinner;
  - any record is duplicated, unexpected or missing;
  - the corpus is not `wikidata-x4`;
  - any output is not 838,800,784 bytes, or the outputs' hashes are not all one.

  A VOID run may be repeated once, with the reason logged. A verdict is never re-run to change
  it.
- CPU time is reported beside each step. It decides nothing.

**Limits:**
- **The corpus:** Wikidata alone, the decoder's primary workload and where Open admits 97.9%
  of the rows. `text` is not in this run.
- **Admission:** Open's admission is the density line, unchanged here, so the same blocks take
  Open in both builds. R4 applies to every block Open takes.
- **What a result separates:** a NOT ADOPTED here, with #675's, would leave the one-thread
  gain unexplained at scale. It would not by itself separate R4's own cost under contention
  from a gain too small to show at 16 rounds.
