# 2026-11-10: The pool's own wait budget, three arms: registration

10 November 2026. **Registered before any timing of these arms**, on CI or anywhere else. This
entry is committed before the CI screen below runs; its result is appended under "Result"
afterwards and changes nothing above it.

The brief is the Optimiser's "Buildable brief A" (`team-state/optimizer` branch,
`state/records/reviews/inbox-6756316971/report.md`): separate the cost of the pool's own waits
from the refuted bimodality hypothesis (`2026-11-09-spin-audit.md`), opt-in, with the default
unchanged.

## The hypothesis

A cooperative pool's two waits, a helper's between runs (`pool_helper_idle`) and the caller's
for its helpers to end a run (`pool_caller_join`), spin up to `SPINS` (2,000) checks before they
park. On the 4-vCPU VM, 94–100% of those spins belonged to waits that parked anyway
(`2026-11-09-spin-audit.md`, section 3), about 1.2 M spins a decode of the bench's Wikidata
blocks at three teams of three, and as many at one team of three: about 41% of all the decode's
spins. A smaller budget saves that wait work.

- **Prediction:** less CPU, with wall neutral or better if most of those waits park anyway.
- **What it can cost:** the waits that used to end while spinning now park, and each costs a
  futex wait and an unpark, and the wake-up's latency. A caller's join is on the team's critical
  path: if its wake latency is exposed, wall rises. On the VM about a third of the caller's
  joins ended without parking.
- **Its size, bounded before timing:** the CPU an arm can save is at most the spins it removes
  times one spin's CPU cost on the machine measured, before the parks it adds. Arm B removes all
  the pool's spins (about 1.2 M a decode on the VM); arm C leaves at most 63 a wait, about 44 k
  over the 3×3 decode's 705 pool waits. On the VM, at its idle-thread estimate of about 9 ns a
  spin, 1.2 M spins is about 0.011 CPU-s a decode. That is the VM's figure, not the CI
  runner's or the i9's, and the counts step below prints the runner's own estimate, labelled as
  one. The spin audit's 3.3 M bound covers every wait, the walk's included, and is not credited
  to these arms. **This bound is small:** on the CI corpora an improvement may not be
  resolvable, and that would be an informative null, not a reason to search further.

## The arms

Construction-time, research only, never selected by the default (`parallel::Levers::pool`, a
`parallel::PoolWait`):

| arm | `ab` arm | the pool's two waits | everything else |
|---|---|---|---|
| A | `mibz2-pool2000` | budget 2,000 (`coop::SPINS`, the default's) | as `mibz2-pipe3-both` |
| B | `mibz2-pool0` | budget 0: park at the first unmet check | the same |
| C | `mibz2-pool64` | budget 64 | the same |

- **The budget is the pool's comparison, unchanged:** a wait spins while its count of checks
  is below the budget, so A spins up to 1,999 times, C up to 63 and B none. B falls through
  the same park protocol (the condition is checked, then `park`, re-checked after every
  return).
- **The plan:** each arm is `mibz2-pipe3-both`, the pipelined teams with the default's levers
  (park and small): THREADS / 3 teams of 3 (at least one), with the default's window,
  `SLOTS_PER_THREAD` a team. At 9 threads that is the default decoder's own three teams of
  three (`PIPELINED_PLAN`).
- **What stays fixed:** the walk's waits (`BARRIER_SPINS`, park, and the separate `spin2k`
  control, which is not this experiment), the task partition, the front and back buffers,
  the kernels and the source. Only the pool's budget differs between arms. The plain teams'
  pools (`bwt::Cooperative`) keep `SPINS` in every arm.
- **Exactly three arms:** `PoolWait` offers these and no other budget. Every attempted arm is
  retained; there is no search for a lucky budget.

**A deviation from the brief, disclosed:** the brief asks for the comparison first on the
service plan (#316). The service plan is not on master, so these arms run on master's
pipelined teams. A service-plan comparison is a separate registration once that plan exists
on master. Transfer from one to the other is not assumed.

## What was built

- `src/coop.rs`: a pool's budget is a field of its shared state, set when the pool is made
  (`Pool::pipelined_with(k, spins)`); `Pool::new` and the default's pipelined pools pass
  `SPINS`, so the default's waits are what they were. Park and unpark, the epoch's
  publication, `done`'s, the caller's registration, cancellation and the join before reuse
  are unchanged. The one addition beside them is the `wait-stats` counter of wakes, which
  compiles to nothing without the feature.
- `src/parallel.rs`: `Levers::pool` (a `PoolWait`, default `Spin2000`), passed to the
  pipelined worker as a value. `PIPELINED_LEVERS` sets `Spin2000`.
- `coop::wait_stats` (feature `wait-stats`) also counts **wakes**: the unparks aimed at each
  pool site's waiters (the walk's unparks are not attributed to a site, so its sites show 0).
- `examples/ab.rs`: the three `mibz2-pool*` arms; built with `wait-stats`, a run's record also
  carries its wait counts (`waits`). `examples/wait_audit.rs`: `--pool 2000|0|64`, each line
  naming its arm (`pool_wait`, `pool_spins`) and each site's wakes.

## The falsifiers, in place before any timing

All are in CI's test job or Miri job, on this pull request.

- **Lost wake-ups** (`coop::tests::pool_wait_arms_lose_no_wakeup`): for budgets 2,000, 0, 64
  and 1, at 2, 3 and 4 threads, 6,000 runs each. Between runs the caller goes on at once, or
  spins up to 4,000 times, yields or sleeps a few microseconds; each thread's call does a
  different amount of work. So a run's epoch often lands as a helper's spins run out, and a
  helper's `done` as the caller's do. Each set has a 60 s deadline on a thread of its own: a
  hang fails the test instead of hanging it.
- **The negative controls** (`pool_wait_stress_catches_an_unpark_before_the_epoch`,
  `..._before_done`): the same stress on a pool whose publication is deliberately broken
  (test builds only, `coop::Broken`):
  - the run's unparks sent before its epoch is published;
  - a helper's unpark of the caller sent before it counts itself done.

  Each widens its race window with a 2 ms sleep. Under every budget, each must hang within
  5 s, and both do: the stress sees a lost wake-up when there is one.
- **Unchanged output:**
  - `tests/pipelined.rs` runs every test, for each lever set, with arms B and C added (the
    corpora, every level and shape, small blocks, block-CRC, primary-index and
    move-to-front errors, truncation, a reader dropped mid-stream, cancellation). Each is
    held to the default decoder and libbzip2: the same bytes, the same errors, at the same
    block.
  - The fuzz smoke's pipeline cases (`tests/common`'s `pipeline_candidates`) gain the 3×3 team
    with arms B and C. They are held to the plain parallel decoder exactly, and to libbzip2.
  - On the bench corpora, the CI screen's "Same bytes" step fails if any run of any arm decoded
    a corpus to other bytes or an error. The counts step fails likewise across arms and plans.
- **Miri** (`coop::tests::miri_pool_wait_arms`, in the targeted suite `cargo miri test --lib --
  miri_`): each budget's pool, every index once, the helpers' writes visible after the run, a
  helper's panic resumed, the pool dropped while its helpers park. It runs under the race
  detector, with Miri's check that every thread ends. The changed code touches the pool's
  atomics' loops, and that suite covers `coop`.

## The measurement

`.github/workflows/pool-wait.yml`, one runner (`ubuntu-latest`), triggered once by the label
`bench:pool-wait` on this pull request.

- **Measured inputs:** the pull request's head when labelled, recorded in the run (`HEAD_SHA`)
  and in its artifact's name. A later change to `src/**`, `examples/ab.rs`, `tests/common` or
  the Cargo files invalidates the result for any later head.
- **Workload:** the standing bench's corpora (`ab gen corpora 4`: text, random, onebyte,
  pattern, runs, records) and the 12 MB of Wikidata blocks (`bench/wikidata.json`), built and
  checked as `bench.yml` builds them.
- **Grid:** the standing bench's thread grid on this runner, 1, 2 and its CPUs (4), with
  `--spread`, and 9 threads (the default's own three teams of three). That makes 7 corpora × 4
  thread counts = 28 cells, each with wall and CPU.
- **Rounds:** 18, three full cycles of `ab run`'s arm-order rotation over three arms, so each
  arm runs in each position equally often. That is 3 arms × 28 cells × 18 = 1,512 decodes.
  - **Why 18:** the brief's prospective screen was 20 rounds; 18 is the nearest balanced
    count.
  - **It is not powered for a small effect:** with paired log-ratio SD σ, the interval's
    half-width is about 2.1σ/√18 ≈ 0.5σ, so at σ = 0.03 it resolves about 1.5%. The CPU bound
    above may sit below that.
- **The figures:** whole-decoder wall (from opening the file to its last byte) and CPU (user +
  system, every thread of the process). Each arm is reported over arm A, cell by cell, as
  `ab summary --ref mibz2-pool2000` reports it: the geometric mean of the paired ratios by
  round, and its two-sided 95% t interval over the rounds. Peak RSS is reported, median by
  cell.
- **Counts, untimed, after the timing:** `wait_audit` (built separately with `wait-stats`) on
  the Wikidata blocks, each arm at 1×3 and 3×3, 3 rounds. It gives the waits, the parks, the
  spins, the wasted spins and the wakes by site: the mechanism, never a time.

## The decision rule

STEWARDSHIP's standing acceptance route, unchanged (`docs/STEWARDSHIP.md`, "What the steward
may do without asking", at 3e1a7ec3), quoted:

> **Changes that touch measured code** (`src/**`, `Cargo.toml`, `Cargo.lock`, the bench's inputs)
> merge only when all of these hold:
> - the tests and parity checks pass;
> - CI is green on the head that will merge;
> - the CI paired A/B bench (`.github/workflows/bench.yml`) shows an end-to-end improvement
>   beyond noise, meaning its 95% interval excludes zero in the beneficial direction;
> - no other figure regresses unacceptably.

How it applies here, fixed before the data:

- **This pull request** changes no default, so it merges, if at all, as a measurement change:
  CI green, and `bench.yml`'s head-over-base shows nothing changed that wasn't meant to. On CI's
  runners the default decoder is production's workers (fewer than `COOP_MIN_CORES` physical
  cores), so its paired bench should show no change.
- **An arm "passes" this screen** when, with that arm as head and arm A as base:
  1. at least one cell's wall or CPU ratio has its 95% interval wholly below 1 (an
     improvement beyond noise);
  2. no cell's wall or CPU ratio has its 95% interval wholly above 1. "Unacceptably" is fixed
     here, before the data, as any regression beyond noise, at any cell, in wall or CPU;
  3. no correctness check failed ("Same bytes", the counts' outputs, the tests).

  Peak RSS is reported and not judged; the arms allocate nothing different.
- **A pass makes the arm a candidate, nothing more.** Changing the default is a later pull
  request that must itself meet the route above on `bench.yml`, on the runner's default decoder.
  This screen's 56 intervals an arm carry no multiplicity correction, so a single cell's pass is
  weak evidence.
- **The rejection condition the Optimiser set:** if the saved spins merely expose wake latency,
  the arm is rejected. That shows as wall up while CPU falls: at any cell, a wall ratio whose
  interval is wholly above 1 while its CPU ratio is below 1. This holds whatever any other cell
  shows.
- **Only oversubscribed cells:** at 1 and 2 threads a team of three, the decoder's scanner
  and the reader share one or two CPUs, and at 9 threads nine decoding threads share four. If an
  arm's improvements are only there, the result is CI-only: no i9 promotion follows from it.
- **No i9 timing,** of any arm, until the two-modes diagnostic has reported and been reviewed.
  Any workstation run after that is a separate registration under RULES C5.

**The protocol:**
- **One attempt:** the first run that writes `runs.jsonl` is the attempt, whatever it shows,
  with no retry to pass.
- **A setup failure** before any timing (CI-INCIDENTS class B: the toolchain, the corpora's
  download) may be re-run once at the same head, with its reason logged here.
- **A correctness failure** in the screen rejects the arm whose output differed, whatever its
  timing.

## Result

**One attempt, valid:** the Pool-wait arms run
[42740378038](https://github.com/lmmx/mibz2/actions/runs/42740378038), at `02cc5f89` (the
registration's commit). The runner was an AMD EPYC 7763: 4 vCPUs on 2 cores, no CPU quota, no
cpufreq policy visible, `perf_event_paranoid` 4. No setup failure and no re-run. Every run of
every arm gave the same bytes on every corpus, without error, and the counts' outputs agreed
across arms and plans.

**Verdict under the registered rule: arms B (0) and C (64) both pass the screen.** Each has
improvements beyond noise, no regression beyond noise in wall or CPU at any of the 28 cells,
and no cell meets the rejection condition. Neither changes any default: a pass makes each a
candidate only (see "The decision rule"). Per arm:
- **B (0):** 13 of the 56 intervals are improvements, none are regressions, and none meet the
  rejection condition.
- **C (64):** 18 improvements, no regressions, no rejection.

### The figures: arm / A, geometric mean of 18 paired rounds [95% t interval]

Below 1 is better. Marked **bold** where the interval is wholly below 1.

| corpus | threads | B wall | B CPU | C wall | C CPU |
|---|--:|--:|--:|--:|--:|
| wikidata | 1 | **0.925 [0.898, 0.953]** | **0.925 [0.899, 0.953]** | **0.955 [0.927, 0.983]** | **0.954 [0.927, 0.983]** |
| wikidata | 2 | **0.966 [0.941, 0.992]** | **0.957 [0.940, 0.974]** | **0.961 [0.937, 0.985]** | **0.956 [0.935, 0.977]** |
| wikidata | 4 | 1.001 [0.988, 1.015] | **0.962 [0.953, 0.972]** | 1.012 [0.996, 1.028] | **0.969 [0.955, 0.984]** |
| wikidata | 9 | **0.956 [0.919, 0.995]** | **0.950 [0.911, 0.990]** | 0.974 [0.935, 1.014] | 0.965 [0.927, 1.005] |
| text | 1 | 0.988 [0.968, 1.008] | 0.988 [0.969, 1.008] | **0.981 [0.965, 0.996]** | **0.981 [0.966, 0.996]** |
| text | 2 | 0.973 [0.928, 1.020] | 1.005 [0.971, 1.040] | 0.985 [0.932, 1.041] | 1.006 [0.954, 1.060] |
| text | 4 | 0.989 [0.968, 1.010] | **0.979 [0.960, 0.998]** | 0.988 [0.970, 1.007] | **0.969 [0.948, 0.990]** |
| text | 9 | 0.999 [0.947, 1.053] | 0.986 [0.947, 1.026] | 0.996 [0.969, 1.023] | 0.986 [0.959, 1.014] |
| random | 1 | **0.972 [0.950, 0.995]** | **0.973 [0.951, 0.996]** | **0.969 [0.948, 0.991]** | **0.970 [0.948, 0.991]** |
| random | 2 | 1.002 [0.950, 1.057] | 0.999 [0.941, 1.061] | 0.976 [0.932, 1.022] | 0.970 [0.940, 1.001] |
| random | 4 | 1.015 [0.982, 1.049] | 1.012 [0.973, 1.052] | 1.004 [0.989, 1.018] | 1.004 [0.979, 1.029] |
| random | 9 | 1.002 [0.935, 1.074] | 1.006 [0.938, 1.081] | 0.977 [0.928, 1.028] | 0.974 [0.922, 1.029] |
| records | 1 | 0.995 [0.975, 1.014] | 0.994 [0.975, 1.014] | 0.991 [0.968, 1.013] | 0.991 [0.969, 1.013] |
| records | 2 | 0.974 [0.940, 1.010] | 0.988 [0.950, 1.028] | 1.002 [0.969, 1.036] | 0.988 [0.949, 1.028] |
| records | 4 | 0.995 [0.978, 1.012] | 0.991 [0.968, 1.014] | 1.007 [0.973, 1.042] | 1.000 [0.968, 1.032] |
| records | 9 | 0.992 [0.968, 1.016] | 0.990 [0.972, 1.008] | 0.991 [0.967, 1.017] | 0.992 [0.972, 1.011] |
| pattern | 1 | **0.944 [0.919, 0.969]** | **0.946 [0.922, 0.970]** | **0.948 [0.929, 0.968]** | **0.952 [0.933, 0.971]** |
| pattern | 2 | **0.924 [0.859, 0.993]** | 0.943 [0.887, 1.003] | **0.924 [0.863, 0.989]** | **0.936 [0.878, 0.999]** |
| pattern | 4 | 1.028 [0.971, 1.088] | 0.973 [0.934, 1.013] | 1.013 [0.981, 1.045] | **0.964 [0.933, 0.996]** |
| pattern | 9 | 0.991 [0.932, 1.054] | 1.001 [0.940, 1.066] | **0.965 [0.945, 0.985]** | **0.963 [0.947, 0.980]** |
| onebyte | 1 | 0.974 [0.921, 1.030] | 0.973 [0.919, 1.029] | 0.992 [0.919, 1.071] | 0.991 [0.915, 1.074] |
| onebyte | 2 | 0.986 [0.962, 1.011] | 0.982 [0.960, 1.006] | 0.987 [0.960, 1.014] | **0.975 [0.962, 0.988]** |
| onebyte | 4 | 0.946 [0.840, 1.066] | 0.943 [0.838, 1.060] | 0.920 [0.827, 1.024] | 0.916 [0.824, 1.018] |
| onebyte | 9 | 0.993 [0.970, 1.017] | 0.982 [0.960, 1.004] | 0.998 [0.982, 1.015] | 0.987 [0.972, 1.003] |
| runs | 1 | 0.935 [0.833, 1.048] | 0.940 [0.841, 1.050] | 0.936 [0.831, 1.054] | 0.938 [0.837, 1.051] |
| runs | 2 | 1.013 [0.972, 1.056] | 1.016 [0.968, 1.067] | 0.999 [0.975, 1.024] | 1.000 [0.979, 1.021] |
| runs | 4 | 0.928 [0.838, 1.028] | 0.932 [0.848, 1.024] | 0.957 [0.846, 1.082] | 0.963 [0.855, 1.084] |
| runs | 9 | 1.015 [0.942, 1.093] | 1.032 [0.954, 1.117] | 0.995 [0.969, 1.022] | 0.999 [0.975, 1.023] |

Arm A's Wikidata medians: 0.270 s wall and 0.736 CPU-s at 4 threads; 0.305 s and 1.057 CPU-s at
9. The artifact `pool-wait-02cc5f89…` holds `runs.jsonl`, `summary.md`, `counts.jsonl` and
`env.json`.

### The counts (untimed, the Wikidata blocks, mean of 3 rounds)

| plan | arm | pool spins (helper idle + caller join) | caller-join parks | all spins |
|---|---|--:|--:|--:|
| 1×3 | A | 1.08 M | 254 | 2.28 M |
| 1×3 | B | 0 | 341 | 1.11 M |
| 1×3 | C | 40 k | 340 | 1.22 M |
| 3×3 | A | 1.18 M | 290 | 2.83 M |
| 3×3 | B | 0 | 254 | 1.63 M |
| 3×3 | C | 38 k | 260 | 1.66 M |

- **Wakes are the same in every arm:** 466 caller-join wakes, and 468–472 helper-idle wakes, a
  decode. Wakes are unparks sent, so the protocol's traffic doesn't change; what changes is how
  often a waiter is asleep to receive one.
- **The runner's spin estimate:** `wait_audit`'s idle-thread figure was about 22.7 ns a spin.
  It is an estimate on this runner, not a measured saving.

### What it shows

- **The one uncrowded cell agrees with the bound.** At 4 threads (one team of three, the
  scanner and the reader on 4 vCPUs), Wikidata's CPU fell by 3.8% (B) and 3.1% (C), about
  0.028 and 0.023 CPU-s.
  - The bound priced before timing, on this runner's estimate, is 1.08 M removed spins × 22.7
    ns ≈ 0.025 CPU-s. The saving is about the size the mechanism allows, and no more.
  - Wall there did not move beyond noise: B 1.001 [0.988, 1.015], C 1.012 [0.996, 1.028].
    C's central wall is up while its CPU is down, the rejection condition's direction, but
    within noise. It is noted, not judged.
- **The wall gains are in the oversubscribed cells,** where a spinning waiter takes a CPU from
  a thread with work:
  - 1 and 2 threads, where three decoding threads, the scanner and the reader share one or two
    CPUs;
  - 9 threads, where nine decoding threads share four CPUs.
  
  By the registration, a wall gain seen only there is CI-only and promotes nothing on the i9.
- **The CPU gains reach the uncrowded cell, the wall gains don't.** At 4 threads CPU improves
  beyond noise on Wikidata (B, C), text (B, C) and pattern (C). Wall improves at no 4-thread
  cell. So the evidence of a non-oversubscribed benefit is a CPU saving at unchanged wall.
- **The plain teams were not measured.** Every arm here is the pipelined plan; the plain teams'
  pools keep `SPINS`.
- **The default's bench** ([42740293712](https://github.com/lmmx/mibz2/actions/runs/42740293712),
  head over base) detected no change on its four assessed measures, as expected: the runner's
  default decoder has no pools. One of its 42 per-cell intervals, `records` wall at 4 threads
  (−4.4%), excluded no change. The workers' code path is unchanged, so it is read as chance.

**Not done, by the registration:** no i9 timing, until the two-modes diagnostic has reported and
been reviewed. A default change would be a separate pull request: arm B or C as the pipelined
teams' pool budget, under the standing route on `bench.yml`. Since the runner's default has no
pools, it would need a target argument (the target-gated route) rather than this screen.
