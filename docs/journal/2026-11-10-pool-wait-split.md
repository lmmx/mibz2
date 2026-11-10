# 2026-11-10: The pool's two waits split, three arms: registration

10 November 2026. **Registered before any timing of these arms**, on CI or anywhere else. This
entry is committed before the CI screen below runs; its result is appended under "Result"
afterwards and changes nothing above it.

The brief is the Optimiser's review of #321 (#145 comment 6757343704; report at
[`1bac98df…/state/records/reviews/inbox-6757179363/report.md`](https://github.com/lmmx/mibz2/blob/1bac98dff722313c280e610b05fe1dd83a4ba174/state/records/reviews/inbox-6757179363/report.md)),
"Next construction": split the pool's helper-idle budget from its caller-join budget, three
fixed arms, no budget search, and charge the helper's launch latency too. It follows
#321's screen ([`2026-11-10-pool-wait-budgets.md`](2026-11-10-pool-wait-budgets.md), run
42740378038), whose arms both passed. Under the Guardian's D080, #321 is not
measurement-only and does not merge. This branch is built on #321's and stands under the
same ruling.

## The hypothesis

A cooperative pool has two waits:
- **`pool_helper_idle`:** a helper waits between runs for the next one.
- **`pool_caller_join`:** the caller waits for its helpers to end a run.

Both spin up to `SPINS` (2,000) checks before they park. In #321's counts on the Wikidata
blocks, helper-idle was **72.5% (1×3) and 72.3% (3×3) of the default's pool spins**.
- **Helper-idle is the bulk of the spinning:** 783 k spins a decode at 1×3 and 854 k at 3×3,
  against the join's 297 k and 328 k (both2000). On the 4-vCPU VM, 94–100% of the pool's
  spins were in waits that parked anyway (`2026-11-09-spin-audit.md`).
- **The caller's join is on the team's critical path.** Under both0, its parks rose from 254
  to 341 a decode at 1×3.

**The hypothesis:** helper0/join2000 removes most of the pool's removable spin work (the
helper-idle share). It keeps the caller's short completion wait, and so it recovers any
caller-wake penalty both0 pays.
- **Prediction:**
  - CPU close to both0's saving, less the join's retained spins: at 1×3 about 297 k spins, a
    gross scale estimate of 0.0067 CPU-s on #321's runner;
  - wall no worse than both0's, and better wherever both0's join wake was exposed.
- **What it can cost:**
  - **The helpers' launch latency is unchanged from both0's.** Every helper that would have
    caught the next run while spinning now parks, so the run waits for a futex wake before
    the helper claims its call. This is the cost the Optimiser asked to be charged. It is
    measured directly below (wake-to-work latency), not inferred.
  - **The join's restored spins are CPU** that both0 does not spend.
- **The Optimiser's falsifiers, adopted:**
  - **CI falsifier:** helper0/join2000 fails to remove the helper-idle spins, introduces a
    correctness or liveness failure, or shows significant wall harm against both2000.
  - **"Keep both0":** if helper0/join2000 does not recover wall against both0, and simply
    adds caller CPU, keep both0. No further budget is searched.
  - **Mechanism falsifier:** if the join's waits and latency do not recover despite the
    restored budget, the caller-spin explanation is inadequate, whatever the timing shows.
    The test is how the join's waits end (immediate, ended spinning, parked) and its wake
    latency, compared between both0 and helper0/join2000.

## The arms

Construction-time, research only, never selected by the default (`parallel::Levers::pool`, a
`parallel::PoolWait`). Each pool's two budgets are immutable values fixed when it is made
(`coop::Budgets`):

| arm | `ab` arm | helper-idle budget | caller-join budget | everything else |
|---|---|--:|--:|---|
| both2000 | `mibz2-pool2000` | 2,000 (`coop::SPINS`) | 2,000 | as `mibz2-pipe3-both` |
| both0 | `mibz2-pool0` | 0 | 0 | the same |
| helper0/join2000 | `mibz2-poolh0j2000` | 0 | 2,000 | the same |

- **both2000 is the default's,** exactly: `Budgets::DEFAULT` is `SPINS` at both waits, and
  every pool the default decoder makes (`Pool::new`, the default's pipelined teams,
  `PIPELINED_LEVERS`) takes it.
- **both0 is #321's pool0, unchanged.**
- **#321's pool64 is not offered:** `PoolWait` has exactly these three arms. The Optimiser
  found pool64 had no evidenced latency benefit, and "no budget search".
- **A budget is the pool's comparison, unchanged:** a wait spins while its count of checks is
  below its budget, then parks. A budget of 0 parks at the first unmet check, through the
  same protocol: the condition is checked, then `park`, re-checked after every return.
- **The plan:** each arm is `mibz2-pipe3-both`: the pipelined teams with the default's levers
  (park and small), max(1, THREADS / 3) teams of 3, with the default's window,
  `SLOTS_PER_THREAD` a team.
- **What stays fixed:** the walk's waits (`BARRIER_SPINS`, park), the task partition, the
  publication order, notifications, the buffers, the kernels and the source. Only the
  pool's two budgets differ between arms. The plain teams' pools (`bwt::Cooperative`) keep
  `SPINS` at both waits in every arm.

## What was built

- **`src/coop.rs`:**
  - `Shared::spins` becomes `Shared::budgets` (`Budgets { idle, join }`). The helper's loop
    reads `idle`, the caller's `Wait` reads `join`.
  - Park and unpark, the epoch's publication, `done`'s, the caller's registration and the
    join before reuse are unchanged.
- **`src/parallel.rs`:** `PoolWait::Helper0Join2000` replaces `Spin64`. `PoolWait::budgets`
  maps an arm to its `Budgets`, passed to the pipelined worker as a value.
- **The wake-to-work latency (counting build only, feature `wait-stats`):**
  - **At `pool_helper_idle`:** from the run's publication to the helper's claim of its call.
    The publication is the caller's clock read just before it advances the epoch and
    unparks the helpers. The claim is the helper's read of the run's task, just before it
    calls it. This is the launch latency a helper adds to a run.
  - **At `pool_caller_join`:** from the run's last helper's end to the caller's wait ending.
    The last helper's end is the latest of the helpers' clock reads, each taken just before
    it counts itself done (a `fetch_max`). This is the join's wake latency.
  - **Each measured wait is classed by how it ended:** immediate, spun (ended inside its
    budget) or parked. Its latency goes into a log-linear histogram, four buckets an octave
    (`wait_stats::latency`). `wait_audit` prints each class's count, mean, p50, p90, p99
    and maximum. Each quantile is its bucket's upper bound, an overestimate of at most a
    quarter octave.
  - **None of it is in a timed build:** the `Shared` fields, the clock reads and the
    histograms are all `#[cfg(feature = "wait-stats")]`, and the timed `ab` and the default
    bench build without the feature.
  - **The counting build reads `Instant` twice a measured wait** (tens of nanoseconds), so its
    latencies carry that cost and its untimed schedule. They describe the mechanism, not
    the timed builds' wall.
- **`examples/ab.rs`:** the arm `mibz2-poolh0j2000`. `mibz2-pool64` is gone.
- **`examples/wait_audit.rs`:** `--pool 2000|0|h0j2000`; each line names `idle_spins` and
  `join_spins`, and carries the two pool sites' latencies.

## The falsifiers, in place before any timing

All are in CI's test, fuzz-smoke or Miri jobs, on this pull request.

- **Lost wake-ups** (`coop::tests::pool_wait_arms_lose_no_wakeup`), for five budget pairs at
  2, 3 and 4 threads, 6,000 runs each:
  - both2000, both0 and helper0/join2000;
  - helper2000/join0, the other mixed pair;
  - 1 at both waits.

  Between runs the caller goes on at once, spins, yields or sleeps a few microseconds, and
  each thread's call does a different amount of work. So a run's epoch often lands as a
  helper's spins run out, and a helper's `done` as the caller's do. Each set has a 60 s
  deadline, on a thread of its own.
- **The negative controls, kept**
  (`pool_wait_stress_catches_an_unpark_before_the_epoch`, `..._before_done`): the same
  stress on a pool whose publication is deliberately broken, under each of the five budget
  pairs:
  - the run's unparks sent before its epoch;
  - a helper's unpark of the caller sent before it counts itself done.

  Each must hang within 5 s under every pair, and each does locally: the stress sees a lost
  wake-up at either wait, whichever budget that wait has.
- **The split reaches its own waits** (`wait_stats_split_budgets_and_latency`, the counting
  build, in test.yml's new "Counting-build tests" step):
  - at helper0/join2000 the helpers' idle wait never spins, and the caller's may;
  - at helper2000/join0 the caller's join never spins;
  - every helper's claim and every caller's join records exactly one latency.

  `latency_buckets` checks the histogram's buckets.
- **Unchanged output:**
  - `tests/pipelined.rs` runs every test with both0 and helper0/join2000 beside the default's
    levers (both2000). That covers the corpora, every level and shape, small blocks,
    block-CRC, primary-index and move-to-front errors, truncation, a reader dropped
    mid-stream, and cancellation, each held to the default decoder and libbzip2.
  - The fuzz smoke's pipeline cases (`pipeline_candidates`) carry the 3×3 team with both0 and
    with helper0/join2000. They are held to the plain parallel decoder exactly, and to
    libbzip2.
  - On the bench corpora, the screen's "Same bytes" step fails on any differing output; the
    counts step fails likewise across arms and plans.
- **Miri** (`coop::tests::miri_pool_wait_arms`, in `cargo miri test --lib -- miri_`): each of
  the five budget pairs' pools. It checks:
  - every index runs once;
  - the helpers' writes are visible after the run;
  - a helper's panic is resumed;
  - the pool can be dropped while its helpers park.

  It runs under the race detector, with Miri's check that every thread ends.

## The cells, described correctly

**The runner:** `ubuntu-latest`. #321's runner (`env.json`) had **4 logical CPUs on 2
physical cores** (sibling sets 0,1 and 2,3), an AMD EPYC 7763. This run's `env.json` records
its own, and the result states it.

| requested threads | plan built | decoding workers | what shares the CPUs |
|--:|---|--:|---|
| 1 | one pipelined team of three (1×3) | 3 | the team, the scanner and the reader, on 1 logical CPU (`--spread`) |
| 2 | 1×3 | 3 | the same, on 2 logical CPUs |
| 4 | 1×3 | 3 | the same, on 4 logical CPUs on 2 physical cores |
| 9 | three teams of three (3×3, the default's own plan) | 9 | nine workers, the scanner and the reader, on 4 logical CPUs on 2 physical cores |

**No cell is uncrowded:**
- requested threads of 1, 2 and 4 all force the same 1×3 plan, and differ only in the
  logical CPUs it is given;
- even at 4, three decoding workers and the decoder's own feeder roles share two physical
  cores' four hardware threads.

The cells are named by requested threads and plan: 1 (1×3), 2 (1×3), 4 (1×3, 4 logical / 2
physical) and 9 (3×3).

## The measurement

`.github/workflows/pool-wait-split.yml`: #321's screen workflow, with its arms replaced and
the latencies added. It runs once, on one runner, triggered by the label
`bench:pool-wait-split` on this draft pull request.

- **Measured inputs:** the head when labelled, recorded in the run (`HEAD_SHA`) and in its
  artifact's name. A later change to `src/**`, `examples/ab.rs`, `examples/wait_audit.rs`,
  `tests/common` or the Cargo files invalidates the result for any later head.
- **Workload:** the standing bench's corpora (`ab gen corpora 4`: text, random, onebyte,
  pattern, runs, records) and the 12 MB of Wikidata blocks (`bench/wikidata.json`), as
  #321's screen ran them.
- **Grid:** requested threads 1, 2, the runner's logical CPUs (4) and 9, with `--spread`.
  That is 7 corpora × 4 = 28 cells, each with wall and CPU, the same grid as #321's, so the
  two screens are comparable cell by cell. The Optimiser's smaller grid (2 plans × 2
  corpora) is contained in it, as Wikidata and text at 4 (1×3) and 9 (3×3).
- **Rounds:** 18, three full cycles of `ab run`'s arm-order rotation over three arms. That is
  3 × 28 × 18 = 1,512 decodes.
  - **The reference and controls are fresh:** both2000 and both0 are run concurrently with
    helper0/join2000, interleaved, not reused from #321's run.
  - **Its resolution is limited:** the paired interval's half-width is about 0.5σ, about
    1.5% at σ = 0.03. helper0/join2000's expected difference from both0, the join's retained
    spins, may sit below that.
- **The figures:** whole-decoder wall and CPU (user + system, every thread). Two summaries are
  reported, each as `ab summary` reports it: the geometric mean of the paired ratios by
  round, with its two-sided 95% t interval over the rounds.
  - **The screen:** each arm over both2000 (`--ref mibz2-pool2000`).
  - **The secondary contrast:** each arm over both0 (`--ref mibz2-pool0`). This is
    helper0/join2000 against both0, the "recovery" question.
  - **Peak RSS** is reported, median by cell, and not judged.
- **Counts and latencies, untimed, after the timing:** `wait_audit` (the counting build) on
  the Wikidata blocks, each arm at 1×3 and 3×3, 3 rounds. By site, it records:
  - the waits, and how they ended (immediate, ended spinning, parked);
  - the spins and the wasted spins;
  - the parks and the wakes;
  - at the two pool sites, the wake-to-work latency distribution of each class.

## The decision rule

STEWARDSHIP's standing acceptance route, unchanged (`docs/STEWARDSHIP.md`, "What the steward
may do without asking", at 63907bc7), quoted:

> **Changes that touch measured code** (`src/**`, `Cargo.toml`, `Cargo.lock`, the bench's inputs)
> merge only when all of these hold:
> - the tests and parity checks pass;
> - CI is green on the head that will merge;
> - the CI paired A/B bench (`.github/workflows/bench.yml`) shows an end-to-end improvement
>   beyond noise, meaning its 95% interval excludes zero in the beneficial direction;
> - no other figure regresses unacceptably.

How it applies here, fixed before the data:

- **This pull request does not merge,** whatever the screen shows (Guardian D080: #321's
  construction, which this extends, is not measurement-only). **A pass is a screen pass
  only:** no merge, no promotion and no default change follow from it. A default change
  would be its own pull request under the route above, with a target argument, since the
  runner's default decoder has no pools.
- **An arm passes this screen** when, with that arm as head and both2000 as base (the earlier
  screen's rule, as #321 registered it):
  1. at least one cell's wall or CPU ratio has its 95% interval wholly below 1 (an
     improvement beyond noise);
  2. no cell's wall or CPU ratio has its 95% interval wholly above 1. **"Unacceptably" is
     fixed as #321's screen fixed it:** "any regression beyond noise, at any cell, in wall or
     CPU";
  3. no correctness check failed ("Same bytes", the counts' outputs, the tests).
- **The Optimiser's rejection condition:** saved spins that merely expose wake latency
  reject the arm. That shows as **wall up while CPU falls**: at any cell, a wall ratio over
  both2000 whose interval is wholly above 1 while its CPU ratio is below 1. This holds
  whatever any other cell shows.
- **The secondary contrast (helper0/join2000 over both0)** ranks the arms; it is not a gate.
  It is read, fixed now, as:
  - **Recovers:** at a cell, a wall ratio whose interval is wholly below 1 means
    helper0/join2000 recovered a caller-wake penalty there, beyond noise.
  - **Keep both0:** at no cell is the wall interval wholly below 1, and some cell's CPU
    interval is wholly above 1. That is the Optimiser's case of no wall recovery, only
    added caller CPU.
  - **Unresolved:** every interval includes 1. That is reported as such, an informative
    null, with no further budgets tried.
- **The mechanism check (counts and latencies),** read beside the timing:
  - helper0/join2000's helper-idle spins must be 0, or the arm is broken;
  - its join must end inside its spin budget more often than both0's, with fewer parks;
  - its join's wake latency must fall against both0's.

  If the join's ending and latency do not recover, the caller-spin explanation is reported as
  inadequate, whatever the timing shows.
- **Oversubscribed cells:**
  - **1 and 2 (both 1×3):** three decoding workers and the decoder's feeder roles share one
    or two logical CPUs.
  - **9 (3×3):** nine share four.

  At 4 (1×3), the plan has four logical CPUs, but on 2 physical cores, so it is not spare
  capacity either. Any result here is CI-only: no i9 promotion follows from it.
- **No i9 timing,** of any arm, until the two-modes diagnostic has reported and been
  reviewed. Any workstation run after that is a separate registration under RULES C5, run by
  Louis.

**The protocol:**
- **One attempt:** the first run that writes `runs.jsonl` is the attempt, whatever it shows,
  with no retry to pass.
- **A setup failure** before any timing (CI-INCIDENTS class B) may be re-run once at the same
  head, with its reason logged here.
- **A correctness failure** in the screen rejects the arm whose output differed, whatever its
  timing.

## Result

(Appended after the one attempt; nothing above changes.)
