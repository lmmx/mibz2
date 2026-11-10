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

Not yet run. The CI screen's figures, the counts and the verdict per arm, under the rule above,
go here.
