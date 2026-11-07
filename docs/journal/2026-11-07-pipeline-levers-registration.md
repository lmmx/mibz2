# 2026-11-07: The pipelined team's two levers, registered before any workstation run

**Status:** registered, not run. This entry, the levers it tests, the recipe and the report
are frozen at one commit: the one `bench/pipeline-levers.frozen` names, the commit after this
entry's last change.
- `just bench-pipeline-levers` refuses to run if any of its measured inputs differ from that
  commit, or if the tree is dirty. The measured inputs are the sources, Cargo's files,
  `examples/`, `bench/pipeline_levers.py` and the scripts it uses, `.just/bench.just`, and
  this entry.
- The report never calls a run VALID unless it is the registered one: 20 rounds, a clean
  tree, no `NOT-FROZEN.txt` or `DRY-RUN.txt`, and `commit.txt` the checkout's head.
- **Nothing here changes the default decoder, or the registered pipe.**
  - Each lever is a separate opt-in flag (`parallel::Levers`, through
    `StreamDecoder::pipelined_levers` and `BzReader::parallel_pipelined_levers`, all
    `#[doc(hidden)]`).
  - `StreamDecoder::pipelined`, the prototype registered at f8aa9dc2, calls the same code
    with both levers off. Each lever is a const generic (`PARK`, `SMALL`), so with both off
    the code is the prototype's: the same waits, and one job taken per run.

It follows the registered run's result (`2026-11-07-team-pipeline-result.md`). There,
pipelining gave +4.2% at 3 × 3 on 10 threads and +3.2% at 3 × 4 on 12, for 8–11% more CPU.
Only 147 of 431 blocks overlapped across threads, and 198 of the 431 are below `MIN_COOP`.
It named the two levers tested here.

## The levers as built (`research/team-pipeline-levers`)

### Lever A, park (`Levers::park`)

**What it replaces:** a thread waiting for the build's end (`walk_ready`). That is a helper
at the helpers-only barrier, or the leader waiting to join the walk. In the prototype it
spins 4,096 times, then calls `yield_now` in a loop. It never parks, so it is never
CPU-free.

**What it does instead:** the pool's own wait, between runs.
1. It spins `coop::SPINS` (2,000) times, checking the flag.
2. It lists itself in the run's `sleepers` (a mutex-guarded list of `Thread`s, its capacity
   reserved before the run).
3. It parks until the flag is set.

Whoever sets the flag (the build's last helper, in `set_walk_ready`) stores it with
release ordering, then takes the lock and unparks everyone listed.

**No wake-up is lost** (the argument is in `Relay::wait_walk_ready`):
- The waiter lists itself under the lock, then loads the flag (acquire).
- If its lock came first, the setter finds it listed and unparks it. An unpark that comes
  before the park makes the park return at once.
- If the setter's lock came first, the setter's store happened before the waiter's lock, so
  the waiter sees the flag and never parks. Its entry stays in the list, harmlessly: the
  list belongs to the run and is dropped with it.
- A spurious wake-up, or a token left over from an earlier unpark, only makes it check the
  flag again. The same holds for the pool's own parks.

**Unchanged:** the walk itself, the segment counter, the last walker's assembly, and the
pool's own waits. Neither the output nor the errors change.

**What it should do:** cut the CPU that the build waits spend yielding while the build's
other shares, or the walk set-up, run. The cost is a wake-up latency (a futex wake, a few
microseconds) where a waiter had already parked. The question registered for it is whether
CPU falls while the wall holds.

### Lever B, small blocks keep the pipeline primed (`Levers::small`)

**What it replaces:** in a run, the leader takes at most one job.
- If that job's block is below `MIN_COOP`, the leader decodes it whole, on its own one-core
  buffers, and stages nothing.
- So the next run's leader finishes its block alone (the pipeline drained), and the next
  job's move-to-front runs with no helper work beside it.
- Each small block costs about one block of overlap.

**What it does instead:** after a small block, the leader takes the next queued job too. It
keeps going until it stages a block for the team, or finds no job queued.
- The front set it stages into is free again, because the small block's decode has ended and
  its result is published.
- So after a run that met a small block, the next run usually has a block staged, and its
  leader's move-to-front overlaps that run's build and walk.

**Why this design**, and not giving small blocks to the helpers:
- **It reuses the prototype's one safe path.** It is the same `try_job` (which never waits)
  and the same `stage` into the same front set, so it adds no new state, buffer or hand-off.
- **It keeps the invariant that prevents deadlock.** A leader never waits for a job while it
  holds an unpublished block: `try_job` only takes a job that is already queued, so the
  scanner can never be waiting on a slot that this team holds.
- **Errors keep their identity.** A failed move-to-front is published at its own place and
  bit, as before. The reader sees it in block order whatever order the blocks finished in.
- **The alternative needs more machinery.** Letting the helpers take small blocks would need
  a second job channel into the pool, a buffer set per helper, and a rule for when helpers
  stop. It would add RAM, protocol and tests for a block 1/16 the size or less. It is not
  built.

**What it does not fix:**
- While the leader decodes small blocks, the helpers, once their block is inverted, still wait
  idle in the pool.
- The inverted block i waits unpublished until the next run's leader finishes it. With
  lever B, the leader may decode several small blocks first, so block i's publication can
  come later. That is a latency the reader may see (it reads in order), bounded by the
  queue. It is measured, not assumed away.

**What it should do:** raise the cross-thread overlap count, and the wall gain with it.

### Both (`park` and `small`)

The two levers are independent. A waits in the helpers' barrier and the leader's join; B
changes how many jobs the leader takes. So their combination is a third arm, registered with
the same comparisons.

## The tests

Run on this branch. All are in CI's test job.

**Every lever setting against the default** (`tests/pipelined.rs`): none, park, small and both,
at teams of 2, 3 and 4, on 1 or 2 teams, with tight and wide windows. Each test checks the
bytes byte for byte, and the error by kind, block and bit:
- the crate's corpora;
- every sample at levels 1, 3 and 9;
- several streams, with small blocks among full ones;
- one full block, one small block, an empty stream, and no input;
- a block CRC in the middle of a file;
- a bad primary index;
- truncations;
- a move-to-front error on the block a leader staged while the one before was in flight.

**New tests:**
- **`small_blocks_keep_the_pipeline_primed`:** full blocks and small ones alternating, 13
  blocks on one team. As registered, 6 jobs are taken overlapped. With lever B it is 12, as
  the design predicts (every job but the first). The test asserts lever B's count is higher,
  and that the bytes are the same.
- **`an_mtf_error_after_a_small_block`:** a full block, a small one, then a block whose
  move-to-front fails. With lever B, the leader stages the failing block in the run that
  decoded the small one (asserted by `pipeline_counts`'s error count). The bytes and the
  error match the default.
- **Cancellation (`dropped_mid_stream`):** a reader dropped mid-stream, on every plan and
  every lever setting, under a 60-second watchdog. Also a push-form decoder, cancelled by
  its reader while its writer writes, with every lever setting.

**Unit tests** (`coop::tests`):
- `split_build_and_walk_are_build_and_walk` and `pipelined_leader_late_or_early` run both
  waits (`PARK` false and true), with a leader that is early or 30 ms late.
- `miri_parked_wait_loses_no_wakeup`: three waiters against the setter, which sets the flag at
  once, after a yield, or after a sleep. A lost wake-up would hang the test.

**Miri:**
- `cargo +nightly miri test --lib -- coop::tests::miri_` passes, all 10 tests.
- That includes `miri_split_build_and_walk`, which now runs the parked waits too, and the new
  parked-wait test.
- The parked-wait test also passes over 16 seeds (`-Zmiri-many-seeds=0..16`), and the split
  test over 4.
- Miri found one bug, in the test, not the code: an assertion that the list of waiters ends
  empty. It needn't (above).

**Lints:** `cargo clippy --all-targets -D warnings`, with and without `--features trace`, and
`cargo fmt --check`.

## The arms

There are two `ab run`s, one for each plan, appended to one `runs.jsonl`. Each is interleaved
by round with `ab`'s counterbalanced rotation. Every comparison is within one plan and one
thread count, so the two runs needn't share rounds. The sample is the bench's pinned
Wikidata sample (`bench/wikidata.json`, SHA-256 `82f65ace…`), and every arm uses the
default's window (`SLOTS_PER_THREAD` blocks a team).

| run | cell | plan | what it is |
|---|---|---|---|
| 10 threads | `mibz2@10` | 3 × 3 | today's default on the i9: the noise control's reference |
| | `mibz2-team3@10` | 3 × 3 | cooperative, not pipelined |
| | `mibz2-pipe3@10` | 3 × 3 | the registered pipe: no lever |
| | `mibz2-pipe3-park@10` | 3 × 3 | lever A |
| | `mibz2-pipe3-small@10` | 3 × 3 | lever B |
| | `mibz2-pipe3-both@10` | 3 × 3 | both |
| 12 threads | `mibz2-team4@12` | 3 × 4 | cooperative, not pipelined |
| | `mibz2-pipe4@12` | 3 × 4 | the registered pipe |
| | `mibz2-pipe4-park@12` | 3 × 4 | lever A |
| | `mibz2-pipe4-small@12` | 3 × 4 | lever B |
| | `mibz2-pipe4-both@12` | 3 × 4 | both |

There are no unregistered cells. The report refuses any cell beyond these. Every arm runs
unpinned, as the default does.

## The comparisons

Each comparison has two co-primary figures, both paired by round. Each is reported as the
geometric mean over the rounds, with its 95% t-interval [lo, hi]:
- **speed:** the reference's wall over the arm's, so above 1 means the arm is faster;
- **CPU:** the reference's CPU over the arm's, so above 1 means the arm spends less.

Peak RSS is reported beside them.

**The causal comparisons**, each lever against the pipe it changes (the same plan, threads,
window and placement):

| | lever | arm | against |
|---|---|---|---|
| A1 | park | `mibz2-pipe3-park@10` | `mibz2-pipe3@10` |
| A2 | park | `mibz2-pipe4-park@12` | `mibz2-pipe4@12` |
| B1 | small | `mibz2-pipe3-small@10` | `mibz2-pipe3@10` |
| B2 | small | `mibz2-pipe4-small@12` | `mibz2-pipe4@12` |
| AB1 | both | `mibz2-pipe3-both@10` | `mibz2-pipe3@10` |
| AB2 | both | `mibz2-pipe4-both@12` | `mibz2-pipe4@12` |

**The noise control:** N1, `mibz2-team3@10` against `mibz2@10`. The two cells run the same
plan, so N1's speed interval must lie within 1 ± 0.03.

**Reported, never judged:**
- R1 and R2: the registered C1 and C2 replicated (`pipeK` against `teamK`);
- each lever against `teamK`: how much of the pipeline's gain it keeps or adds;
- `mibz2-pipe4-park@12` against `mibz2@10`: the product view.

## Rounds and order

- **20 rounds** in each `ab run`. Within a run, `ab`'s `arm_order` rotates the arms one step a
  round, and reverses the order every other cycle.
- **The order is not balanced over 20 rounds.** The 10-thread run has 6 arms and the 12-thread
  run 5, so a full balanced cycle is 12 or 10 rounds. The 12-thread run's 20 rounds are two
  full cycles; the 10-thread run's are one cycle and two-thirds of another.
- **The 12-thread run always follows the 10-thread one.** No comparison crosses the two.
- **No randomisation is used,** and none is claimed.

## Validity (checked by `bench/pipeline_levers.py report`, not by the reader)

A run is **VALID** only if all of these hold:
- every run of every cell gave the same output hash and length, and none failed;
- each registered cell ran rounds 0–19 once each, and every run's threads show its plan
  (`mibz2-N` leaders and `mibz2-coop-*` helpers, counted from `ab`'s `thread_cpu`);
- no cell beyond the registered ones;
- the run is the registered one (the frozen inputs, a clean tree, not a dry run, the head);
- N1's interval lies within [0.97, 1.03].

An invalid run has no outcome.

## The outcome map

**Each figure's outcome, from its 95% interval [lo, hi]:**

| interval | speed outcome | CPU outcome |
|---|---|---|
| lo > 1.02 | **GAIN** | **CPU SAVING** |
| 1.00 < lo ≤ 1.02 | **SMALL GAIN** | **SMALL CPU SAVING** |
| hi < 1.00 | **REGRESSION** | **CPU COST** |
| 0.97 ≤ lo and hi ≤ 1.03, the interval containing 1 | **NULL** | **NULL** |
| anything else | **INCONCLUSIVE** | **INCONCLUSIVE** |

**Lever A's registered question (A1, A2): does CPU fall while the wall holds?** "Holds" means
the speed interval lies wholly above 0.98, so any wall loss is at most 2%, about half of C1's
gain.

| CPU interval | speed interval | answer |
|---|---|---|
| lo > 1 | lo > 0.98 | **YES:** CPU falls, the wall holds |
| lo > 1 | hi < 1 | **TRADE:** CPU falls, the wall regresses |
| lo > 1 | otherwise | **CPU FALLS,** the wall not shown to hold |
| hi < 1 | any | **NO:** CPU rises |
| neither | hi < 1 | **NO:** the wall regresses, CPU not shown to fall |
| neither | otherwise | **NO:** CPU not shown to fall |

A YES on both A1 and A2 is the case for making park the pipe's wait. A YES on one plan only
is reported as plan-dependent.

**Lever B (B1, B2):**
- Its question is speed: GAIN or SMALL GAIN, with its CPU outcome beside it. CPU is co-primary:
  a speed GAIN with a CPU COST is a trade-off to weigh, not an outright win.
- **Its mechanism has to be shown in the traces before any speed outcome is read as lever B's
  doing.** Its median `pipe.overlap.cross_thread` must exceed the pipe's at the same plan by
  at least 10% of the blocks (43 of 431).
  - If it does: the mechanism is **SHOWN**.
  - If not: **NOT SHOWN**, and then no speed outcome of B is attributed to more overlap.
  - If either trace is missing or incomplete: **UNRESOLVED**.

**Both (AB1, AB2):** both questions apply, A's to the CPU figure and B's to speed.
- If both levers act independently, AB's speed is about B's and its CPU about A's.
- An AB outcome worse than either lever alone on its own figure (speed for B, CPU for A, each
  interval wholly below that lever's mean) is reported as an **interaction**.

**Null and regression are registered outcomes, not failures of the run.**

| observation | interpretation allowed |
|---|---|
| A: CPU NULL, speed NULL | the waits' yielding cost too little CPU on this machine to show. Park is not falsified elsewhere |
| A: CPU SAVING, speed REGRESSION | the parked waiters' wake-up latency costs wall: a spin longer than 2,000, or a park only once the build's remaining work is long, would be the next step |
| B: mechanism SHOWN, speed GAIN | more overlap per block makes the pipe faster: the gain's attribution as in the prototype's registration (the overlap, or the leader's extra walking) |
| B: mechanism SHOWN, speed NULL or REGRESSION | the extra overlap is cancelled: by the helpers' idle time while the leader decodes small blocks, or by block i's later publication. The phase times (`pipe.ms.*`) locate it |
| B: mechanism NOT SHOWN | the leader rarely finds a job queued after a small block on this machine (the scanner behind), or the instrument misses it: no verdict on B |
| INCONCLUSIVE | inconclusive, not a falsification |

## The traces

Occupancy passes of every pipelined arm, 6 rounds each, at the default (unpinned) placement:
`--pipe 3` at 10 threads and `--pipe 4` at 12, each with no lever and with `--levers park`,
`small` and `both` (8 passes).

From the phase marks (src/trace.rs: `Mtf*` to `Run*`), the report gives, per pass:
- the blocks and the move-to-front count (`pipe.n.mtf`);
- the cross-thread MTF overlap (`pipe.overlap.cross_thread`), as corrected in #247's review:
  a leader is credited only with its own team's helpers' build or walk of the block its
  enclosing run inverts;
- the same-thread containment (`pipe.overlap.same_thread`);
- the RLE1 and CRC overlap (`pipe.overlap.rle_cross_thread`);
- the summed build-wait time (`pipe.ms.build_wait`), which lever A changes the nature of but
  shouldn't lengthen;
- the build and walk time (`pipe.ms.build` + `pipe.ms.walk`).

The prototype's result is the reference: 147 of 431 blocks overlapped cross-thread at 3 × 3,
140.5 at 3 × 4.

## The recipe

`just submit bench-pipeline-levers` runs `.just/bench.just`'s `bench-pipeline-levers`, modelled
exactly on `bench-team-pipeline`:
- one new `mkdir`'d directory per attempt (`target/pipeline-levers/<UTC stamp>`);
- `console.log` written with `tee`;
- an EXIT trap that writes `outcome.txt` and packs the directory with `bench/bundle.sh`,
  however the attempt ends;
- the freeze checks (`bench/pipeline-levers.frozen`, the measured inputs, a clean tree);
- the corpus built from the pinned windows;
- the two `ab run`s, the 8 traces, and the report's verdict.

It takes about 35 minutes. `DRY=1` is the untimed check of the recipe itself (one round, one
trace round, no settling). It is never evidence, and the report never calls it VALID.

## Indicative only: this session's 4-core machine

**Not evidence for the registration.** This was one shared machine with 4 cores, and others
kept about 0.1 cores busy. The runs were 10 paired rounds at 4 threads on the same sample.
Ratios are as registered above, with the reference the pipe without levers (speed and CPU
above 1 favour the lever).

| plan | arm | speed [95% CI] | CPU [95% CI] | median wall, CPU (s) |
|---|---|---|---|---|
| 1 × 4 | `pipe4` (reference) | | | 0.648, 1.856 |
| 1 × 4 | `pipe4-park` | 0.963 [0.908, 1.021] | 1.062 [0.991, 1.137] | 0.662, 1.771 |
| 1 × 4 | `pipe4-small` | 1.118 [1.020, 1.226] | 1.034 [0.947, 1.129] | 0.578, 1.822 |
| 1 × 4 | `pipe4-both` | 1.060 [0.967, 1.163] | 1.071 [0.974, 1.178] | 0.605, 1.794 |
| 1 × 4 | `team4` | 0.920 [0.859, 0.984] | 1.003 [0.928, 1.084] | 0.704, 1.901 |
| 2 × 2 | `pipe2` (reference) | | | 0.495, 1.713 |
| 2 × 2 | `pipe2-park` | 0.959 [0.875, 1.051] | 1.060 [0.961, 1.169] | 0.506, 1.598 |
| 2 × 2 | `pipe2-small` | 1.017 [0.916, 1.129] | 1.002 [0.897, 1.120] | 0.473, 1.655 |
| 2 × 2 | `pipe2-both` | 1.042 [0.959, 1.133] | 1.083 [0.994, 1.180] | 0.455, 1.519 |
| 2 × 2 | `team2` | 1.077 [0.977, 1.186] | 1.118 [1.009, 1.239] | 0.443, 1.467 |

**What these runs suggest, and no more:**
- Park saves about 6% CPU, but neither interval excludes 1. It may cost a few percent of wall.
- Small is the one interval above 1, at 1 × 4 (+12%); it is unclear at 2 × 2.
- Both looks like small's speed with park's CPU, within wide intervals.
- At 2 × 2 the non-pipelined team is still the fastest here, as in the prototype's own
  indicative runs.

The i9 run decides.

## What this run can't tell

- Whether lever B's gain comes from the overlap itself or from the leader walking more
  often. That is the prototype's own open attribution question.
- Placement effects (H2, deferred).
- Any machine but the i9-10900K.
- Corpora with a different mix of small and full blocks. Lever B's effect scales with the
  share of small blocks, 198 of 431 here.
