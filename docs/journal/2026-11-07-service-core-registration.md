# The service core: registration

7 November 2026. Registered before any run. The prototype is on `research/service-core`
(from `research/team-pipeline`, #247). It is opt-in and doc-hidden:
`StreamDecoder::service`, `BzReader::parallel_service`, the `mibz2-svcK` arms. The default
decoder and the pipelined and lever paths are unchanged.

**How it is frozen.** `bench/service-core.frozen` names the commit this registration, the
prototype, the recipe (`just bench-service-core`) and the report (`bench/service_core.py`)
are frozen at. The recipe refuses to run if any measured input differs from it, and the
report calls nothing VALID but the registered run.

## The question

In the 3 × 3 plan, each team's leader does the block's move-to-front and checkpoints, then
later its assembly, RLE1, CRC and publish. In the pipelined team (#247), the leader overlaps
that work with its helpers. That gained 3–4%, for 8–11% more CPU, partly because the leader
leaves the build and the walk.

**The idea:** take the serial work out of the teams.
- K dedicated service workers (1 or 2) do every block's serial stages, serving every team:
  - preparing: the move-to-front and its checkpoints, for blocks to come;
  - finishing: the assembly, RLE1, CRC and publish, for blocks walked.
- Each team's k threads only build and walk.

**The causal question:** at an equal budget of decode threads, is a plan with service
workers faster than the 3 × 3 team (`team3`) and the pipelined team (`pipe3`)? And at what
CPU cost?

## The design

**Threads:**
- the scanner, as in every decoder;
- a feeder, `mibz2-svc-feed`. It moves the scanner's jobs into the hub, so a service worker
  waits on one condition for either kind of work. It never sits in the jobs' `recv` while a
  walked block waits for it: the reader's next block could then be stuck behind a scanner
  waiting for the reader's slot.
- K service workers, `mibz2-svc-N`;
- the teams, `mibz2-N`, each with its pool's helpers, `mibz2-coop-*`.

**The team's run** (`coop::invert_team`) is `bwt::Cooperative`'s inversion without its
assembly:
- `build_fused` and `walk_room` as they are: two pool runs of `FusedBuild`'s and
  `RoomWalk`'s shares (the new/share/finish split from #247), with the pool's own waits
  between them;
- the segments left for the service to assemble.

So a service-core team builds and walks exactly as a `Cooperative<3>` team does. The
pipelined run's in-run barrier, and the levers' parked wait for it, aren't needed.

**Two queues in a hub**, each bounded by buffers that are reused, never allocated per block:
- **Prepared** (service → teams): blocks whose move-to-front is done, each in a front set
  (`ll`, counts, checkpoints). There are teams + lookahead + K front sets, with the
  lookahead at one block a team.
- **Finishing** (teams → service): blocks walked, each in a back set (the walk's segments,
  or the bytes of a block too short to segment). There are two back sets a team.

**The service's priority policy (registered):** finishing first, but the prepared queue is
never left empty while a block can be prepared. In order, a service worker:
1. prepares, if the prepared queue is empty and a job and a front set are free;
2. else finishes the walked block earliest in the stream, if any;
3. else prepares, if the prepared queue is below its lookahead and a job and a front set
   are free;
4. else ends, once the jobs are closed and no block is in flight, or waits.

Why this order:
- Finishing first frees back sets for the teams and publishes the reader's next block, so
  the finish backlog can't grow while a block is ready to finish.
- Rule 1 keeps the teams' 9 threads from idling behind the service's one thread. A team
  idle for a prepared block wastes three cores; a block waiting to finish holds one back
  set.
- The lookahead caps the prepared blocks at one a team, so preparing can't run far ahead
  and take the front sets and the L3 while blocks wait to finish.

**Small blocks** (below `MIN_COOP`, 256 Ki rows; 198 of the sample's 431 blocks) are
prepared and finished by the service like every block. They are inverted whole by a team's
first thread, on the one-core kernel, with its helpers idle: exactly what a `Cooperative`
team does today. The alternative, inverting them on the service worker, was rejected:
- The model below makes the service the scarce resource.
- A block's inversion costs about 2.8 times its serial stages (1.57 against 0.57 ns a
  byte).
- So small blocks on the service would saturate it first.
- Keeping them on a team also keeps the comparison with `team3` matched.

**Kept:**
- **Ordered output:** every result goes through `ready`, as a worker's does.
- **Errors with their block and bit:** a move-to-front error, a bad primary index, RLE1
  and CRC errors, all published by the service as `Error { block, bit }`.
- **The cancellation drain:** a cancelled decode drops its blocks wherever they are and
  returns their buffers, and the feeder drains the jobs, so the scanner never waits.
- **Panic → `Internal`:** a panic in any stage is caught where it happens and becomes that
  block's `Internal` error. The stages are a service worker's preparing or finishing, and a
  team's build or walk on any pool thread. The buffers the stage held are replaced with new
  ones, never lost, so the buffer bounds can't shrink into a deadlock.
- **The end-of-stream flush:** each thread ends once the jobs are closed and every block
  taken has been published or dropped.

**Memory**, an estimate rather than a measurement. Each block in flight now holds a front set and a
back set, not one workspace. That is about +4 front sets (~1 MB each) and +3 segment sets
(~2 MB touched each) against `team3`, about +10 MB of peak RSS (67 → ~77 MB). Measured
peak RSS is reported for every cell.

## The model (before any run)

**Its inputs:**
- The CI stage harness (`examples/phases.rs`, bench run 42310037840; knee journal E4),
  single-threaded:
  - the serial stages: MTF 0.41, assembly 0.04, RLE1 0.05 and CRC 0.07 ns a byte, so
    **S = 0.57 ns a byte**;
  - build 0.48 and walk 1.09 ns a byte.
- The i9's `team3`: about **1.39 GB/s** of aggregate decode (E1, 1,382–1,399 MB/s at 3
  teams).
- A team spends **29–33%** of each block outside the inversion call (E4, measured at
  4 × k).

**These are CI-derived extrapolations, not measurements of this design.** S was measured
on one thread on CI, not on the i9 with nine other busy threads, and E4's shares were
measured at another plan. Every figure below inherits that.

**The service's demand and capacity:**
- Demand at an aggregate rate X is X · S cores. At today's 1.39 GB/s that is **0.79 of a
  core**.
- Capacity is K / S: **1.75 GB/s at K = 1, 3.51 GB/s at K = 2**.

**The teams' capacity, relieved of the serial work:** X_team3 over its inside share (0.67–0.71).
- **3 teams: 1.96–2.07 GB/s.**
- **2 teams** (svc2 at 10): E1's 2-team per-table rate, 558–583 MB/s, gives
  **1.61–1.74 GB/s**.

**The bound** is X ≤ min(teams' capacity, K / S), against `team3`'s 1.39 GB/s:

| arm | plan | teams' bound GB/s | service bound GB/s | X bound GB/s | speed bound vs team3 | service utilisation at the bound |
|---|---|---|---|---|---|---|
| svc1 at 10 or 12 | 3 × 3 + 1 | 1.96–2.07 | 1.75 | **1.75** (service-bound) | ≤ 1.26 | **1.00** |
| svc2 at 12 | 3 × 3 + 2 | 1.96–2.07 | 3.51 | 1.96–2.07 (team-bound) | ≤ 1.41–1.49 | 0.56–0.59 each |
| svc2 at 10 | 2 × 3 + 2 | 1.61–1.74 | 3.51 | 1.61–1.74 (team-bound) | ≤ 1.16–1.25 | 0.46–0.50 each |
| svc1-t2 | 3 × 2 + 1 | about 1.96–2.07 (E3: team size barely matters at a fixed team count) | 1.75 | **1.75** (service-bound) | ≤ 1.26 | **1.00** |

**Sensitivity to the service's cost on the i9.** A factor f is applied to S for contention:
its move-to-front sharing the L3 and memory with nine busy threads.

| f | K = 1 capacity GB/s | svc1's speed bound vs team3 | K = 2 capacity GB/s |
|---|---|---|---|
| 1.0 | 1.75 | 1.26 | 3.51 (teams bind) |
| 1.2 | 1.46 | 1.05 | 2.92 (teams bind) |
| 1.4 | 1.25 | **0.90 (a regression)** | 2.51 (teams bind) |

**Queueing near saturation.**
- **A block's service time:** the sample's 431 blocks decode to about 210 MB, about
  0.49 MB a block. At S, a block costs the service about 0.28 ms: 0.20 ms preparing and
  0.08 ms finishing.
- **At K = 1:** with deterministic service (M/D/1, a rough guide; arrivals are bursty, not
  Poisson), the mean queue wait is ρ · s / (2 (1 − ρ)). That is 0.56 ms at ρ = 0.8 and
  1.26 ms at ρ = 0.9.
- **Against the inversion:** a team's block takes about 0.73 ms (1.49 ns a byte × 0.49 MB).
  So at K = 1 a block's queue wait is about as long as its inversion. The prepared queue's
  lookahead of one block a team is what has to absorb the bursts.
- **At K = 2,** per-server ρ is about 0.6, and the waits are a few tenths of the block.

**What the model predicts:**
- K = 1 runs at or near saturation at any gain worth having (ρ ≥ 0.8). Its gain over
  `team3` is at most about 26%, and plausibly small, null or negative once the service's
  i9 cost and the queueing are counted.
- K = 2 is team-bound, not service-bound.
- So **K = 2 is registered as a co-equal arm, not an option**: C3 and C4 below are judged
  like C1 and C2. D1 (K = 2 against K = 1 at the same three teams) is the saturation's own
  causal test.

**Not in the model:**
- the cross-core traffic of the hand-offs: `ll` written by the service and read by a team,
  the segments the other way;
- the hub's synchronisation;
- the extra live bytes in the L3 from queued front and back sets (the knee's footprint
  effect, E2).

All three would lower the gain.

## Budgets, and the threads every arm runs

**The budget is the decode threads:** the teams' threads plus the service threads. Every
causal comparison is at an equal budget. ab's THREADS is the budget, and no arm runs more
decode threads than it.

- **How the controls use their budget:** `team3` and `pipe3` run THREADS / 3 teams of 3.
  At 10 that is 3 × 3, 9 decode threads: teams of 3 can't use the tenth.
- **The svcK arms:** K service threads, and teams of T (3, or 2 in `svc1-t2`), as many as
  fit the rest of the budget, at most 3 (the three-table knee, and today's policy above 9
  threads).

**Every arm also runs** the scanner (`mibz2-scan`), the reader's feeder (`mibz2-read`) and
the consumer (ab's own thread). The svcK arms add `mibz2-svc-feed`, blocked in `recv`
except for one wake a block. None is counted in the budget, the same way for every arm.

**Disclosed: arms with more decode threads than their reference.**

| cell | plan | decode threads | other threads | tables at once |
|---|---|---|---|---|
| mibz2@10, mibz2@12 | 3 × 3 (the policy) | 9 | 3 | 3 |
| team3@10, pipe3@10 | 3 × 3 | 9 | 3 | 3 |
| team3@12, pipe3@12 | 4 × 3 | 12 | 3 | 4 |
| svc1@10, svc1@12 | 3 × 3 + 1 | 10 | 4 | 3 |
| svc2@10 | 2 × 3 + 2 | 8 | 4 | 2 |
| svc2@12 | 3 × 3 + 2 | 11 | 4 | 3 |
| svc1-t2@10, @12 | 3 × 2 + 1 | 7 | 4 | 3 |

- **Comparisons where the arm has more decode threads:**
  - C1 and C2: 10 against 9;
  - D1: 11 against 10;
  - P1: 10 against 9;
  - P2: 11 against 9.
- **Physical cores:** on the i9 (10 cores, 20 hardware threads, nothing pinned), svc1@10's
  10 decode threads can occupy every physical core, where team3@10's 9 leave one free.
  Every cell at 12, and svc1@10 with its other threads, runs more busy threads than
  physical cores, so SMT siblings are shared.
- **At 12,** the controls run four tables (4 × 3), past the three-table knee. C3 and C4
  compare against that plan, as the budget gives it, and say so.

**The extra-thread rule (registered).** Where the arm runs more decode threads than its
reference (C1, C2, D1, P1, P2):
- **Beyond its threads:** a speed gain counts as more than the extra thread only if its
  interval's lower bound is above the threads' ratio: 10/9 = 1.111 for C1, C2 and P1,
  11/10 = 1.100 for D1, and 11/9 = 1.222 for P2. That reads the reference as if speed
  scaled linearly with threads, its most generous reading.
- **Otherwise:** a gain that doesn't clear the ratio is recorded as an equal-budget gain
  from using one more decode thread. It goes in the product reading, labelled so, and
  never counts as a causal gain of the architecture.
- **D1:** the extra thread is K's own effect, so a D1 gain is read as the saturation test
  below.

## The cells

ab arms at budgets 10 and 12, each budget its own `ab run`, interleaved by round, 20 rounds:

| budget | arms |
|---|---|
| 10 | `mibz2`, `mibz2-team3`, `mibz2-pipe3`, `mibz2-svc1`, `mibz2-svc2`, `mibz2-svc1-t2` |
| 12 | `mibz2`, `mibz2-team3`, `mibz2-pipe3`, `mibz2-svc1`, `mibz2-svc2`, `mibz2-svc1-t2` |

- The window is SLOTS_PER_THREAD (4) blocks a team in every team and service arm, as the
  teamK arms keep it.
- At most 4 · teams + K blocks are in the hub's buffers at once. The window caps them at
  12 at 3 teams, so a svc arm differs from `team3` in its threads, not its window.
- **Traces:** occupancy passes of `--svc 1`, `--svc 2` and `--svc 1t2` at 10 and 12
  threads, 6 rounds each, with the `t.svc.*` figures.

## The registered comparisons

Speed is the reference's wall over the arm's; CPU is the reference's CPU over the arm's.
Above 1, the arm is faster, or spends less. Each comparison is paired by round, with the
geometric mean and its 95% t-interval over the 20 registered rounds. **Speed and CPU are
co-primary**, each with its own outcome.

**Causal** (equal budget, judged):
- **C1:** svc1@10 against team3@10;
- **C2:** svc1@10 against pipe3@10;
- **C3:** svc2@12 against team3@12;
- **C4:** svc2@12 against pipe3@12;
- **D1:** svc2@12 against svc1@12. K's effect at the same three teams of three: the
  saturation's causal test.

**Product** (against today's default at the same budget, kept apart from the causal ones):
- **P1:** svc1@10 against mibz2@10;
- **P2:** svc2@12 against mibz2@12.

**Reported, never judged:**
- **S1–S2:** svc1@12 against team3@12 and pipe3@12;
- **S3–S4:** svc2@10 against team3@10 and pipe3@10;
- **S5–S6:** svc1-t2 against team3 at 10 and 12;
- **S7:** svc2@10 against svc1@10;
- **R1–R2:** #247's pipe3 against team3, replicated.

## The outcome map (numeric)

On a ratio's 95% interval [lo, hi], speed and CPU each:

| outcome | condition | CPU's wording |
|---|---|---|
| GAIN | lo > 1.02 | CPU SAVING |
| SMALL GAIN | 1 < lo ≤ 1.02 | SMALL CPU SAVING |
| NULL | lo ≥ 0.97, hi ≤ 1.03, and 1 inside | NULL |
| REGRESSION | hi < 1 | CPU COST |
| INCONCLUSIVE | anything else (the interval spans gain and harm, or is too wide for NULL) | INCONCLUSIVE |

- **A speed gain also gets the extra-thread label** (above), where the arm has more decode
  threads.
- **The architecture's reading:**
  - **For it:** C1 or C3 a GAIN, with CPU not a CPU COST. For C1, the gain must be BEYOND
    ITS THREADS.
  - **A trade:** a speed gain with a CPU COST. It is reported as such, for Louis to weigh.
  - **NULL or REGRESSION in C1, C2, C3 and C4 together:** the idea fails on this machine.
    It is recorded in the journal and on the queue's "Rejected" list.

## The registered failure modes, and the counterexample

From each svc trace, as medians over its complete rounds (`t.svc.*`, from the marks alone,
matched by block):
- **Service utilisation:** each service thread's busy time (its life less its idle waits)
  over the run's span. It is reported per thread and per arm, beside wall and CPU, so the
  headroom left shows whatever the speed outcome.
- **Prep starvation:** `team.idle_prep_share`, the teams' idle time waiting for a prepared
  block over teams × span. At **0.10 or more** it is PREP STARVATION.
- **Finish backlog:** either of these is FINISH BACKLOG:
  - `queue.finish.median_ms`, the walked block's wait for a service worker, at **0.5 ms or
    more**;
  - `team.idle_back_share`, the teams idle with a block prepared but no back set free,
    because the finishing queue holds them all, at **0.05 or more**.
  Its consequences show in the backlogs (`backlog.finish_mean` and `_max`) and in the
  scanner's slot waits, which occupancy reports.
- **Service saturation (the registered counterexample):** SATURATED when utilisation max
  ≥ **0.90**, and PREP STARVATION or FINISH BACKLOG.

**Reading the counterexample:**
- **SATURATION BINDS AT K = 1:** svc1@12 SATURATED, and D1 a speed gain.
- **SATURATION MEASURED, NOT SHOWN TO BIND:** svc1@12 SATURATED, and D1 not a gain.
- **NOT SATURATED at K = 1:** neither.

A C1 or C2 NULL or REGRESSION with svc1@10 SATURATED is the model's prediction coming true,
not an unexplained null.

**The service-queue waits are traced separately,** each block's wait by place:
- **the jobs:** the scanner's `Enqueued` to a service worker's `JobGot`;
- **the prepared queue:** `Prepared` to a team's `TeamGot`;
- **the finishing queue:** a team's `Walked` to a service worker's `SvcGot`.

Each has its mean, median and p90.
- **The output backlog:** blocks waiting in each queue, time-averaged and at their most.
  Blocks finished but not yet taken by the reader are occupancy's existing ready-and-untaken
  figures.
- **The service worker's stages:** `ms.mtf`, `ms.assemble` and `ms.rle`.
- **The teams:** build, walk, walk set-up and in-run wait.

## Validity (the report checks each)

The run is VALID only if all of these hold:
- the registered 20 rounds;
- a clean tree;
- the frozen inputs unchanged (no `NOT-FROZEN.txt`);
- not a dry run;
- `commit.txt` the checkout's head;
- every run the same output bytes, none failed;
- every registered cell's rounds 0–19 once each, every run with its registered plan (team
  leaders, helpers and service threads counted from ab's per-thread CPU);
- no other cell;
- every registered trace complete, with at least one complete round;
- **the noise control N1:** team3@10 against mibz2@10 (the same 3 × 3 plan) within
  1 ± 0.03.

An INVALID run judges nothing.

## What this design does not separate

**The team's run differs from team3's in its assembly only.** Its build and walk are
`Cooperative`'s, two pool runs as today. So C1 compares where the serial work runs, plus:
- the hub's synchronisation;
- the cross-core hand-offs;
- the extra thread.

The extra-thread rule handles the last.

**C2 against pipe3:** pipe3's leader joins the walk, and its waits spin and yield (#247).
The service core's teams use the pool's park waits. So C2 is a comparison of designs, not
of one factor.

**What is not measured:** the L3 footprint of the queued front and back sets. Peak RSS is
reported, and RSS is not cache residency.

## Local runs

Local runs on the shared 4-core machine are correctness checks only, never timing
evidence: 10 threads can't be tested here. The recipe's `DRY=1` run is untimed, and the
report never calls it VALID.
