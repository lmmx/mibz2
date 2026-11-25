# bench-kernel-choice: registration

*25 November 2026. Written and frozen before any data, on `bench/kernel-choice` (from master
f8edc1ac). The recipe is `just bench-kernel-choice` (`.just/bench.just`). The verdict is
`bench/kernel_choice.py`'s, tested by `bench/test_kernel_choice.py`. The plan probe is
`bench/kernel_choice_probe.rs`. The recipe, rule and verdict script are bench-scanner-plumbing's
(#850, `docs/journal/2026-11-25-scanner-plumbing-registration.md`), with the same step, checks,
fail-soft phases and median peak RSS report. What differs: the counts, the three judged counts,
and a plan check in place of the workers check. The change is meant to change the plan, so the
check requires each build to build exactly its registered plan. There is no diagnostic phase.*

**The decision it settles:** whether #860 merges. #860 changes two production defaults on the
measured i9 unpinned:
- lazy workers at every count from 4 threads asked;
- 8 workers from 10 threads asked, in place of 7.

It is an adoption registration under RULES C6 (a production default), judged under
PERFORMANCE-POLICY.md. CI's bench can't see the change. CI runs `ab` at 1, 2 and 4 threads on
runners that aren't the i9, and no i9 cell applies there, so CI builds the same decoders before and
after (its 4 threads are eager `Workers(4)` both times). CI still checks correctness and that
nothing moved off the i9. This run measures the change where it acts, on the i9 (C5).

## Which head a run measures

As bench-scanner-plumbing:
- **`HEAD_REF`** is a branch on origin, default `perf/lazy-from-four` (#860).
- **`HEAD_SHA`** is a full SHA, or empty for the branch's tip when the run starts. The queue entry
  pins #860's head: `args=perf/lazy-from-four,<sha>`.
- **The base** is the head's merge-base with `origin/master`, computed by the recipe.

The bundle records what was measured: `builds.txt`, `head-commits.txt`,
`src-head-against-base.diff`.

## The contrast, checked by the recipe

Between the base and the head:
- **May differ:** `src/`, `tests/` and `examples/`. #860 touches `src/parallel.rs` and
  `tests/service_default.rs`.
- **Must be identical:** `examples/ab.rs`, `Cargo.toml`, `Cargo.lock`, `build.rs` and
  `rust-toolchain.toml`. Neither build may carry its own `examples/kernel_choice_probe.rs`.
- **Must differ at all:** the head must not already be on master.

Both builds use rustc 1.99.1. The head's harness drives the run, and the base runs as
`mibz2@build-base`.

## The mechanism, from the code (src/parallel.rs at f8edc1ac)

The default decoder's kernel below 8 workers comes from two places:
- **`Plan::lazy()`:** lazy only from `LAZY_FROM_THREADS` = 8 workers (lines 2763, 2857).
- **The i9's measured cells:** unpinned, `unpinned_cell` (line 3369) has explicit arms at 4 and
  6 threads alone. They were added by 0.5.8's and 0.5.7's recalibrations (#678, #642).

So on the i9 unpinned:
- 3, 5 and 7 fall through to `planned` → `workers(n, slots, k, false)`: eager.
- 4 and 6 are lazy through their cells; 8 and 9 through `Plan::lazy`.
- From 10, `unpinned_trims` takes `trim_cell`'s 7 lazy workers with 28 blocks.

#860 replaces the two arms with one range, 4 to 7 threads, the plan's own workers on the lazy
kernel (`UNPINNED_LAZY_FROM` = 4). From 10 it gives 8 lazy workers unpinned
(`UNPINNED_FROM_TEN` = 8), with the window scaled to them. The pinned cells and every other
machine are unchanged.

**The plans, as the probe must record them** (`BzReader::parallel(r, t)`, 4 blocks a thread
asked; workers / window / kernel):

| threads | base (master) | head (#860) |
|---|---|---|
| 3 | 3 / 12 / eager | 3 / 12 / eager |
| 4 | 4 / 16 / lazy | 4 / 16 / lazy |
| 5 | 5 / 20 / eager | **5 / 20 / lazy** |
| 7 | 7 / 28 / eager | **7 / 28 / lazy** |
| 8 | 8 / 32 / lazy | 8 / 32 / lazy |
| 10 | 7 / 28 / lazy | **8 / 32 / lazy** |
| 12 | 7 / 28 / lazy | **8 / 32 / lazy** |

The base's column is bench-kernel-count's probes at master 16aaee3b (journal §76, table 3), whose
`src/parallel.rs` is f8edc1ac's. 3, 4 and 8 are the same at both builds: those are controls, where
the step should be level.

## The prediction (stated before the data; it is not the rule)

From journal §76 (`bench-kernel-count`, kernel-count/run-20261125T022939Z, the same machine,
corpus and placement; 12 rounds, not paired base-to-head as here):

| threads | wall head/base | peak RSS head/base | from §76 |
|---|---|---|---|
| 3 | ≈ 1.00 | ≈ 1.00 | the same decoder |
| 4 | ≈ 1.00 | ≈ 1.00 | the same decoder |
| 5 | ≈ 0.83 | ≈ 0.42 | lazy 5 0.2892 s, 50.7 MB, against the API's eager 0.3485 s, 121.7 MB |
| 7 | ≈ 0.62 | ≈ 0.39 | lazy 7 0.2147 s, 57.2 MB, against the API's eager 0.3446 s, 147.3 MB |
| 8 | ≈ 1.00 | ≈ 1.00 | the same decoder |
| 10 | ≈ 0.94–0.97 | ≈ 1.04–1.06 | the API (7 workers) at 1.066 [1.005, 1.130] of lazy 8 |
| 12 | ≈ 0.96 | ≈ 1.04–1.06 | the API at 1.044 [1.012, 1.076] of lazy 8 |

CPU at 10 and 12 is predicted about +5%: lazy 8 against lazy 7 is 1.811 s against 1.725 s.

## The rule

**A count's step** is the 16 rounds' paired wall ratios, head over base: their geometric mean and
its 95% interval (Student's t on the logs, 15 degrees of freedom). It is FASTER if the interval
lies below 1, SLOWER if above 1, and INCONCLUSIVE otherwise.

- **ADOPTS:** all three hold:
  - the steps at **5, 7 and 10 threads** (judged) are all FASTER;
  - none of **3, 4, 8 and 12 threads** (supporting) is SLOWER;
  - at each of 3, 4, 5, 7, 8, 10 and 12 threads, the median peak RSS over the rounds, head over
    base, is at most 1.10.
- **HELD ON MEMORY:** as ADOPTS, but with peak RSS above 1.10 at some count. The Coordinator
  weighs the growth under PERFORMANCE-POLICY §2 rule 4.
  - **Rule 4, accepted by rule,** applies when all of these hold at each count over the gate:
    - a primary wall-time gain of at least 5%, beyond noise, in this paired run on the target
      machine;
    - the growth, in percent, is at most twice that gain;
    - the absolute growth is at most 64 MB of peak RSS per process;
    - no other corpus grows by more than 10%.
  - **What the verdict prints** at each count over the gate: whether the first three hold, with
    the gain, the growth in percent and in MB. The fourth is out of this run's reach. The change
    acts only on the i9, and CI's bench shows it changes nothing elsewhere.
  - **Otherwise:** the growth needs the PR's own justification, and the Coordinator weighs it.
  - **Not expected:** the predicted growth is +4–6%, at 10 and 12 only.
- **NOT ADOPTED:** any other outcome: 5, 7 or 10 not FASTER, or 3, 4, 8 or 12 SLOWER.
- **VOID**, with no verdict, if any of these holds:
  - the machine is not the i9's profile (an i9-10900K with 10 cores, 20 CPUs and no CPU quota);
  - any run is unsuccessful, or has a non-finite or non-positive wall, CPU or peak RSS;
  - any run is pinned or beside a spinner;
  - any record is duplicated, unexpected or missing;
  - the corpus is not `wikidata-x4`;
  - any output is not 838,800,784 bytes, or the outputs' hashes are not all one;
  - the plan check fails (next section).
- **Reported beside each step, deciding nothing:**
  - each build's plan;
  - the observed workers;
  - CPU time;
  - the median peak RSS at the base and the head, in MiB, with their difference.

**The plan check** replaces bench-scanner-plumbing's "median workers differ between base and
head", since here the workers at 10 and 12 are meant to change (7 to 8). A run is VOID unless:
1. **Each build's probe is its registered plan,** at every count: workers, window and kernel,
   and neither the service core nor the pipelined teams. The base must build master's plan and
   the head #860's, as in the table above. The probe (`kernel_choice_probe BUILD T`, copied into
   each build as an example) is run in the timed runs' placement (unpinned), once a build and
   count, before the timed run (`plan.jsonl`).
2. **Each build's timed runs used its probe's workers:** the median over the rounds of
   `observed_threads`' `mibz2-N` threads, at each count and build. So the probe is what was
   timed. The kernel itself is not visible in a thread's name, so the probe records it.

**Why 5, 7 and 10 are judged.** They are where #860 changes the decoder at the thread counts its
users ask for:
- 5 and 7 are the odd counts that built eager;
- 10 is the i9's own core count, the first count the trim applies at.

12 also changes (7 to 8 workers), but it is supporting. Its predicted gain (≈ 4%) is the smallest
and sits nearest the noise. Requiring it would test the same mechanism as 10 a second time. 3, 4
and 8 are controls, where both builds build the same decoder. The choice is made before the data,
from the code and §76.

**What each outcome does:**
- **ADOPTS:** #860 merges at the measured head once its CI is green. CI's paired bench must show
  nothing moved off the i9 (PERFORMANCE-POLICY §2, "a change built for a specific machine").
- **HELD ON MEMORY:** the Coordinator weighs it under rule 4, with the trade the verdict reports.
- **NOT ADOPTED:**
  - #860 does not merge as it stands;
  - the result is written up as a negative result (journal, PR comment);
  - since #860 makes two changes, the Coordinator may split it and measure each part in a run of
    its own: the lazy kernel at 5 and 7, and the trim at 8. A split run is a new registration,
    not a re-run.
- **VOID:** repeated once, with the reason logged.

## The run

- **The machine:** the maintainer's i9-10900K (10 cores, 20 CPUs, no CPU quota), under C5. Any
  other machine voids the run.
- **The corpus:** `corpora/wikidata.bz2`, the whole bench corpus, checked by
  `bench/wikidata.json`'s SHA-256. Four copies end to end make the timed file (`wikidata-x4`).
- **The cells:** `ab run --arms mibz2@build-base,mibz2`, the public API at each build, unpinned on
  every CPU (no `--spread`, no spinner), at 3, 4, 5, 7, 8, 10 and 12 threads.
- **Rounds:** 16. Each round runs both arms at each count, interleaved, the arms' order rotated,
  with a 2 s settle gate before each run.
- **Fail-soft:**
  - each phase's status goes to `phases.tsv`: resolve, the two builds, inputs, machine, plan, the
    timed run and the verdict;
  - a phase whose prerequisite failed is SKIPPED;
  - the EXIT trap always bundles, and the recipe exits 1 if any phase failed;
  - the run leaves exactly one `target/*/*.tar.gz`, its bundle.
- **Cost:** 224 timed decodes, at about 0.2–0.5 s each plus the 2 s settle. That is about
  9–10 minutes timed, and about 15–20 minutes with the two builds and the corpus. It is below
  PERFORMANCE-POLICY §4a's 30-minute line.

## Limits

- **The corpus:** Wikidata alone, the primary workload, as in §76. `text` is not in this run.
- **Two changes, one verdict:** the lazy kernel at 5 and 7, and the trim at 10 and 12, are
  judged together, since #860 ships them together. The counts separate them: 5 and 7 move only by
  the kernel, 10 and 12 only by the worker count. So a mixed result says which part failed.
- **Above 12:** not measured. The trim also applies at 16 and 20, the Python binding's default
  on this machine. §76 measured up to 12. §76 also found that 8, 9 and 10 lazy workers tie at the
  top, so 8 is predicted to hold there, unmeasured.
- **Pinned:** not measured. #860 leaves the pinned cells as they were.
- **The scanner work interacts with the trim** (§76): a faster scanner moves the balance above 8.
  Each change is priced at the count it will run at. This run prices the trim on master's
  scanner.
- **The parameterised head:** the recipe checks the contrast's files and each build's plan, but
  not what a head's diff does inside `src/`. The Coordinator reads `src-head-against-base.diff`
  in the bundle against #860 before accepting a verdict for it.
