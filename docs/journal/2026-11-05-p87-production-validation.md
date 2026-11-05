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

*Amended once, at 00:20 UTC on 5 November, after Guardian D024 (6645964170). At that time the
bench run (41940043718) was still in progress, with no artifacts. The only bench comment on #164
was its "Running" placeholder, with no figures. So no measured result was available, and none
informed the amendment. The original rule is in d6401b55.*

- **Correctness (required):**
  - CI green, Miri included;
  - same bytes on every corpus in every bench run;
  - the tests named above pass.
- **Instructions (required):** `decode_block` at or below master's on every full block.
- **Benefit (required):** on `wikidata` at 4 threads, the CPU change or the wall change excludes
  zero on the faster side in at least one run, and the same figure's point estimate is not slower
  in the other. Peak RSS is reported as a secondary figure. It doesn't satisfy this condition on
  its own.
  - **"RSS lower beyond noise"** means the bench's own flag (**less**) on the peak RSS column.
- **No harm (required), per run, each run read under its own conditions:**
  - a wall row (any corpus × thread count) flagged **slower** in a run is a harm signal. A second
    run that is inconclusive on that row doesn't cancel it;
  - each harm signal is settled by one more run on the same head: if the row is flagged slower
    again, it is a harm, and the rule fails;
  - if it isn't flagged slower again, the row is reported as unresolved, with all three runs, and
    the PR doesn't claim to have excluded harm on it;
  - at most three runs are made in all.
- **Power, stated before seeing the data:** control rows' 95% intervals in recent runs are ±1–5%. A
  harm smaller than a row's half-width can't be excluded by these runs, and this PR doesn't
  claim to exclude it.
- **Falsified, and closed with a write-up, if any of these holds:**
  - the instruction count exceeds master's on a full block;
  - a harm is confirmed as above;
  - the benefit condition fails.

**What a pass means:**
- the registered conditions are met;
- and if no row is left unresolved, STEWARDSHIP's conditions on measured code are met too: an
  end-to-end improvement beyond noise, and no regression shown.

It doesn't establish P87's mechanism or a steady-state instruction saving. It doesn't meet the
four-worker campaign objective by itself.

## Measured: whole-decoder instructions (added after the registration above)

Inclusive Ir of the driver's `measured` (one `decode_block` on a fresh `Scratch`), from
`callgrind_annotate --inclusive=yes`. Both arms: `driver/` with only the `mibz2` path differing,
rustc 1.97.0, fat LTO, `codegen-units = 1`, line tables. Input `wikidata-independent.bz2`, sha256
`7f5c9a0044e5a305…`. Command: `valgrind --tool=callgrind --toggle-collect=c30cg::measured c30cg cg
FILE BLOCK 1`. The profiles, stdout (length and CRC) and consoles are in `cg/`.

| block | master 08e7fc3b | perf/p87-in-place | change | output |
|---|--:|--:|--:|---|
| 0 (54 B) | 7,573 | 7,296 | −277 | identical |
| 1 (899,971 B) | 20,184,756 | 20,184,356 | −400 | identical |
| 41 (900,000 B) | 19,480,510 | 19,480,110 | −400 | identical |
| 97 (56 B) | 7,942 | 7,672 | −270 | identical |
| 233 (899,978 B) | 19,989,585 | 19,989,153 | −432 | identical |
| 400 (899,993 B) | 19,490,584 | 19,490,042 | −542 | identical |
| 472 (70 B) | 8,780 | 8,510 | −270 | identical |

**The instruction condition holds:** the candidate is at or below master on every block. The
difference is a few hundred instructions a block, consistent with the allocation it removes. A
single decode on a fresh scratch includes `rle`'s first growth, so this is not a steady-state
saving, and none is claimed.

## Measured: the two bench runs, and the verdict

**Timing of the amendment.** The registration allowed amendments "before the first bench comment
on this PR is posted". The bench workflow posts a "Running" placeholder before it measures: one
was posted at 00:16:10 UTC, and the amendment (ef28ad32) came at about 00:20. So the literal
cutoff had passed. When the amendment was made, the run was still in progress, with no figures
and no artifacts, and nothing measured informed it (Researcher 6645977057; Guardian D024). The
verdict below is read under both the original rule (d6401b55) and the amended one.

**The runs.** Bench 3.2.0, 10 paired rounds, against master 08e7fc3b, each on a runner with two
physical cores and four logical CPUs (`env.json`).
- **The measured source is the same in both:** ee12bb9e differs from ef28ad32 only in
  `examples/occupancy.rs`, which the bench doesn't build. That is equivalence of source, not a
  checked identity of the executables.
- **The archive:** each run's report, `result.json`, `env.json`, `ab-summary.md`, and the
  sample-level `runs.jsonl` and `stages.jsonl`, are in `bench-run1-ef28ad32/` and
  `bench-run2-ee12bb9e/`, copied from the Actions artifacts.

| `wikidata` | run 1, ef28ad32, Xeon Platinum 8573C | run 2, ee12bb9e, Xeon 6975P-C |
|---|--:|--:|
| 1 thread, wall | +0.0% (−0.6, +0.7) | +0.0% (−0.2, +0.2) |
| 2 threads, wall | −0.2% (−1.8, +1.4) | −1.1% (−4.8, +2.8) |
| 4 threads, wall | **−2.2% (−4.0, −0.5) faster** | −1.1% (−2.3, +0.2) |
| 4 threads, CPU | −1.3% (−2.9, +0.3) | −0.7% (−1.9, +0.5) |
| 4 threads, peak RSS | 71 → 66 MB (−6.8%, **less**) | 71 → 66 MB (−6.8%, **less**) |

- **Correctness:** same bytes on all 21 rows of both runs, and CI green at ee12bb9e, Miri and
  occupancy included.
- **Instructions:** at or below master on all seven blocks (above).
- **Benefit:**
  - **amended rule:** 4-thread wall is faster beyond noise in run 1. Run 2's estimate is
    favourable but inconclusive, and not slower. **Met;**
  - **original rule:** 4-thread CPU isn't beyond noise in either run. Peak RSS is lower beyond
    noise in both. **Met.**
- **No harm:** no wall row is flagged slower in either run, so there is no harm signal awaiting a
  repeat, and no third run. **Met under both rules.** This is the absence of a detected harm, not a
  demonstration of safety.
- **What isn't excluded.** Several rows have wide upper bounds, and a harm of that size on them
  isn't excluded:
  - run 1: `random` 1 thread +6.4% (upper +17.7%), `records` 1 thread +2.6% (+9.3%), `pattern`
    4 threads −3.0% (+5.1%);
  - run 2: `onebyte` 2 threads +0.7% (+9.2%), `pattern` 2 threads −3.4% (+8.0%), `runs` 2 threads
    +1.2% (+6.4%), and `pattern` 4 threads CPU +2.7% (+10.2%).
- **The stage table** flags RLE1 slower in both runs: +2.6% (+2.0, +3.2) and +3.9% (+3.1, +4.8).
  - Run 1's stage total is also slower, +1.2% (+0.1, +2.4); run 2's isn't, −0.1% (−0.3, +0.2).
  - The stage harness, `examples/phases.rs`, times RLE1 over a separately walked buffer, so its
    timed stages don't measure production's in-place path.
  - But after the first timed pass, its `main` calls production `mibz2::decode` for parity, and
    that path does invert in place. So the untimed run of the changed code could affect later
    passes, through cache, allocation or thermal state. That is a hypothesis, not an identified
    cause (Researcher 6646303348).
  - #128's integration record saw the same RLE1 shift (+3.2%). It is recorded the same way here:
    an unexplained between-build sensitivity, neither dismissed as noise nor shown unrelated to
    the change.
  - **Next, if it matters:** audit the retained phase binaries, callers and layout first, without
    timing. Only then, if needed, record first and later passes separately around the parity call.

**Verdict.** The registered conditions hold under both rules, with no flagged harm awaiting a repeat,
so STEWARDSHIP's gate on measured code is met. The amendment was disclosed, but it was made after
the literal cutoff, so it is not retroactively preregistered. #164 merged as e26e3d9c.
- **Claimed:** a faster `wikidata` decode at 4 threads in one run, favourable but inconclusive in
  the other, with less memory in both.
- **Not claimed:**
  - why it is faster: the change moves the output buffer and removes an allocation together;
  - a steady-state instruction saving;
  - that harm is excluded on the rows listed above;
  - that the four-worker campaign objective is met.
