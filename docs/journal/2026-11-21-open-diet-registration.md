# bench-open-diet: registration

*21 November 2026. Written and frozen before any data, on `bench/open-diet`. The recipe is
`just bench-open-diet` (`.just/bench.just`); the verdict is `bench/open_diet.py`'s, tested by
`bench/test_open_diet.py`. The recipe, the rule and the verdict script are bench-open-r4's
(#680, `docs/journal/2026-11-21-open-r4-registration.md`), themselves bench-open-lanes's (#676)
and bench-open-vec16's (#669). Two things differ: the builds, and one added report, each count's
median peak RSS in MiB.*

**The decision it settles:** whether #692 (`perf/open-wormhole-diet`, "perf(open): memory diet,
Endpoint in edges and queues in out") merges, that is, whether the Open kernel's memory diet
reaches the decoder's default on the i9. It is an ordinary merge under PERFORMANCE-POLICY.md,
measured on the one machine where it can show: CI's bench never runs Open
(`docs/journal/2026-11-20-open-stage-attribution.md`).

## The hypothesis

#692 makes two changes, both in `src/open/` only, in safe Rust, with no allocator or routing
change:
- **W1:** the endpoint elimination's nodes, rules and bottom coordinates live in `edges`' own
  buffer, which they are dead before the wiring writes. `edges` is sized exactly.
- **OutQ16:** the 256 `Vec<u32>` byte queues are replaced by `u16` ranks in 1 KiB chunks of
  the block's own `out` buffer, which is dead until the walk.

Together they cut a worker's persistent Open scratch, the sum of every Vec the kernel owns
after the six admitted blocks, from 4,236 KiB to 1,200 KiB: **0.283×**.

**Claim:** at master's 4.2 MiB, 7 to 10 workers' scratch (29–42 MiB) overflows the i9-10900K's
shared 20 MiB L3. At 1.2 MiB it does not (8.2–11.7 MiB). The kernel then stops competing for
the shared cache and memory bandwidth, so the decoder gets faster where production runs
`OpenCells` with many workers. That cause is a hypothesis, not shown.

## The native evidence (#692's body)

Measured in-crate in a shared 4-core container (a Xeon with 2 MiB of L2 per core and a 260 MiB
shared L3), each figure a ratio against master's kernel in the same process, 12 processes per
arm, 95% t intervals:
- **Scratch:** 4,236 → 2,750 KiB with W1 (0.649×), → 1,200 KiB with W1 and OutQ16 (0.283×).
  Per block, a fresh kernel: 0.31–0.38× of master's.
- **One thread:** W1 neutral, within about 1% of the A/A control. W1 with OutQ16 is neutral to
  about 2% slower on three of the six blocks (block-0 1.041 against A/A 1.017).
- **Four concurrent workers:** W1 neutral (1.005× per pass). W1 with OutQ16 is **0.960×
  [0.934, 0.987] per pass**, against A/A 0.991 [0.969, 1.014]. The gain is on the blocks with
  the most retained rows (block-85 0.955, block-143 0.944, block-210 0.956).
- **Parity:** all six admitted blocks byte-exact against the plain inverse at each commit, the
  counts identical to master's, `cargo test --release` in full, new tests for the `Wide` links,
  the overflow chunks and the 2^16 epochs.

**Why the container can't decide it:** it has 4 cores and a 260 MiB L3, so it cannot reproduce
the i9's 20 MiB L3 at 7 to 10 workers. Its four-worker figure shows L2 and memory pressure only.
The C ledger's ten-worker contention proxy (0.852–0.886× cycles) was not measurable there.

## What is compared

- **Head:** #692's head, `perf/open-wormhole-diet` at
  `79ab1af89e1cccedf23135f8006ea45999650632`. It is master's src at its branch point plus two
  commits (736b1c480 W1, 218e8ab1e OutQ16) and a third that adds the native harness under
  `docs/journal/2026-11-21-open-memory-diet/`, which is not compiled.
- **Base:** master's src, `d425d772c0a0e6b82e9626b36bbfa4b28a6c9613`, #692's merge-base with
  master. It carries v0.5.9 (22afc64d). Its Open kernel is 0.5.8's, identical to
  bench-open-r4's base bb98c995 in `src/open.rs` and `src/open/**`.
- **Why this base and not master's head:** master's head when the recipe was written,
  `db634667fbd7bfa6062a68b04f4dc68680c7a8ac`, is not an ancestor of HEAD: master has moved
  since #692 branched, by docs alone (#689, #691). bench-open-r4's ancestry check
  (`git merge-base --is-ancestor BASE HEAD`) is kept unchanged, so the base is the merge-base,
  which HEAD's history contains. Its `src`, `examples`, `Cargo.toml`, `Cargo.lock` and
  `build.rs` are identical to db634667's, so the base build is master's code as it stands.
- **The contrast, checked by the recipe:** the two builds' `src`, `examples`, `Cargo.toml`,
  `Cargo.lock` and `build.rs` may differ only in `src/open.rs` and files under `src/open/`,
  or the run stops.
  - At these SHAs the changed files are `src/open.rs`, `src/open/b5.rs`,
    `src/open/checks.rs`, `src/open/endpoint.rs` and the new `src/open/outq.rs`. The bundle
    keeps the list as `src-changed.txt`.
  - Both builds use rustc 1.99.1, and the head's harness drives the run.
- **What it predicts:** a whole-decoder wall gain at 10 threads (7 workers) and 8 threads
  (8 workers), where the scratch crosses the L3 on master and not at the head. At 6 threads
  (6 workers, 25 MiB on master) the cliff is nearer its edge, so a smaller gain or none. At one
  thread the native figure is neutral to slightly slower, so a gain here comes from the shared
  cache, not from fewer instructions. For scale: bench-open-r4 on the same grid
  (`open-r4/run-20261121T092824Z`) measured 0.956, 0.982 and 0.985 [0.970, 1.001] at 6, 8 and
  10 threads, NOT ADOPTED by 0.001 at 10, memory flat.
- **Memory, reported and expected to fall:** the scratch cut is about 3.0 MiB per worker
  running Open, so about 18, 24 and 21 MiB less resident at 6, 8 and 10 threads (6, 8 and 7
  workers), if those pages are all touched. The verdict prints each count's median peak RSS at
  the base and the head, in MiB, and their difference. Only the 1.10 gate decides anything.

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

  Production also runs `OpenCells` at exactly 4 threads unpinned (#678). That count is not in
  this run, which keeps bench-open-r4's grid so the results compare directly.
- **Rounds:** 16. Each round runs both arms at each count, interleaved, with the arms' order
  rotated (`arm_order`) and a 2 s settle gate before each run.
- **The diagnostic** (decides nothing): `mibz2-workers-lazy` at each build, the workers' lazy
  route with plan and window fixed, at 6 and 8 threads, 16 rounds. It is reported as steps
  only.
- **Cost:** about 15–20 minutes of workstation time with the two builds, below
  PERFORMANCE-POLICY §4a's 30-minute line.

## The rule

bench-open-r4's, unchanged. A count's **step** is the 16 rounds' paired wall ratios, head over
base: their geometric mean and its 95% interval (Student's t on the logs, 15 degrees of
freedom). A step is FASTER if the interval lies below 1, SLOWER if it lies above 1, and
INCONCLUSIVE otherwise.

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
- **Reported beside each step, deciding nothing:** CPU time, and the median peak RSS at the
  base and the head in MiB with their difference.

**Limits:**
- **The corpus:** Wikidata alone, the decoder's primary workload and where Open admits 97.9%
  of the rows. `text` is not in this run.
- **Admission:** Open's admission is the density line, unchanged here, so the same blocks take
  Open in both builds. The diet applies to every block Open takes.
- **What a result separates:** an ADOPTS shows the whole-decoder gain, not its cause. Peak RSS
  falling and the wall gain growing with workers would be consistent with the L3 claim; neither
  shows it. The two commits are measured together: a result does not separate W1 from OutQ16
  (natively W1 alone was time-neutral).
- **The head's branch:** the recipe fetches HEAD from `perf/open-wormhole-diet` if the
  workstation lacks it. A force-push that drops 79ab1af8 from every ref on origin would stop
  the run at the fetch, not change what it measures.
