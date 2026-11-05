# P87's production candidate on current master: registration

Registered before any measurement of this branch. The decision rule below is fixed here: it may
be amended only in a commit made before the first bench comment on this PR is posted, and every
amendment says what changed and why.

## The candidate

- **Branch:** `perf/p87-in-place`, which merges `research/p87-production` (7c404919) into master
  08e7fc3b. The merge is clean.
- **The change:** `Segmented` inverts in place, writing the walk's output over `ll`, whose last read
  is the build. A worker using it never allocates `rle`. Every other kernel returns false from
  `invert_in_place` and keeps the separate output buffer, unchanged.
- **Contracts and tests:** see `2026-11-02-p87-production-integration.md`:
  - the in-place success and fallback contracts;
  - the differential test against `Segmented::invert` into its own buffer, on every sample shape at
    levels 1 and 9;
  - the fallback and error-reuse tests;
  - the corrected trace ledger.

## Prior record, read in full

- **The four-core diagnostic** (research/p87-buffer-reuse a0094cfa, analysis §§1–6). It reused `ll`
  with equal allocation, on the i9.
  - Primary w4 throughput +1.35% [+0.29, +2.41]; CPU per GB −1.71% [−2.37, −1.06].
  - **Mechanism INCONCLUSIVE:** D = 0.078 [0.042, 0.114] crosses the 0.10 bound.
  - **Engineering NOT DEMONSTRATED:** five no-harm controls were inconclusive, and none harmed.
  - It is closed and is not reopened here.
- **#128 on CI** (EPYC 7763, bench 3.2.0, against 13e9854):
  - at 0c121c5, w4 wall −1.9% [−3.2, −0.7];
  - at 7c40491, w4 wall −0.7% [−2.1, +0.7], CPU −1.4% [−2.5, −0.3], peak RSS −6.2%, with no row
    slower;
  - its stage table showed unexplained between-build shifts, and the phase harness doesn't call
    `invert_in_place`.
- **The Optimizer's ranking** (review/compact30-instruction-ledger dc5e9501, `reviews/inbox-6645643541/report.md` §1)
  puts this first. It asks for:
  - a current-master comparison;
  - byte-exact controls;
  - no-harm criteria registered first.
- **The Researcher's causal assessment** (3843e7da) and Guardian D021–D023. #128's CPU and RSS
  findings don't isolate destination reuse, because the production change also drops an
  allocation.

**What this is.** A production validation of a recorded candidate on a new baseline, not a new
construction and not a repeat of the diagnostic. Two things could change the earlier picture:
- master now carries #157's walk;
- the production intervention also removes `rle`, which the diagnostic didn't test.

It can't attribute any effect to a mechanism, and it claims none.

## What is measured

1. **Whole-decoder instructions** (charter §5.6, STEWARDSHIP): Callgrind on `decode_block`, both
   arms built from the same driver, rustc 1.97.0, fat LTO, one codegen unit. Seven blocks of
   `wikidata-independent.bz2`: full blocks 1, 41, 233 and 400, and tiny blocks 0, 97 and 472. The
   output length and CRC must match in every block.
2. **The CI bench** on this PR's head: the paired A/B of bench 3.2.0, 10 rounds on one runner,
   against the merge base with master.
3. **One re-run of the bench on the same head,** as a second, independent run. The runner may differ;
   each run is read on its own, not pooled.

## The decision rule

- **Correctness (required):**
  - CI green, Miri included;
  - same bytes on every corpus in both runs;
  - the tests named above pass.
- **Instructions (required):** `decode_block` at or below master's on every full block. The in-place
  path removes an allocation and changes a destination, so the expectation is equal to within a
  few thousand instructions, not a large saving.
- **Benefit (required, one of):**
  - on `wikidata` at 4 threads, the CPU change excludes zero on the faster side in at least one
    run, and its point estimate is not slower in the other;
  - or peak RSS at 4 threads is lower beyond noise in both runs.
- **No harm (required):** no corpus × thread-count wall row is flagged slower in both runs. A row
  flagged slower in one run only is reported and weighed, not waved through.
- **Power, stated before seeing the data:** control rows' 95% intervals in recent runs are ±1–5%. A
  harm smaller than a row's half-width can't be excluded by these runs, and this PR doesn't
  claim to exclude it.
- **Falsified, and closed with a write-up, if any of these holds:**
  - the instruction count exceeds master's on a full block;
  - a control row is slower beyond noise in both runs;
  - neither benefit condition holds.

A pass meets STEWARDSHIP's merge gate. It doesn't establish P87's mechanism, and it doesn't meet
the four-worker campaign objective by itself.
