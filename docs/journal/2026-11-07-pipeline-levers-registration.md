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
- **An earlier freeze, at bf08ec55, is superseded. Nothing was run under it.**
  - A stand-in Researcher's review (an independent read of the branch, relayed by Louis) found
    two faults in it, both fixed here.
  - **The spin budget was confounded with parking.** That park arm spun 2,000 times before
    parking, against the pipe's 4,096 before yielding. Lever A now spins 4,096, and a control
    arm separates the budget.
  - **Jobs taken were close to being read as overlap.** That overlap is now the traces'
    corrected count, beside the leader's join delay, which is now traced. Jobs taken are
    context only.
- **Nothing here changes the default decoder, or the registered pipe.**
  - Each lever is a separate opt-in flag (`parallel::Levers`, through
    `StreamDecoder::pipelined_levers` and `BzReader::parallel_pipelined_levers`, all
    `#[doc(hidden)]`).
  - `StreamDecoder::pipelined`, the prototype registered at f8aa9dc2, calls the same code
    with every lever off. Each lever is a const generic (`PARK`, `SMALL`, and the spin budget
    `SPIN`), so with all of them off the code is the prototype's: the same waits, and one job
    taken per run.
  - The trace build gains two marks on the leader (`LeadStart`, `LeadEnd`), in every pipelined
    arm alike. The timed arms are built without the trace feature.

It follows the registered run's result (`2026-11-07-team-pipeline-result.md`). There,
pipelining gave +4.2% at 3 × 3 on 10 threads and +3.2% at 3 × 4 on 12, for 8–11% more CPU.
Only 147 of 431 blocks overlapped across threads, and 198 of the 431 are below `MIN_COOP`.
It named the two levers tested here.

## The levers as built (`research/team-pipeline-levers`)

### The wait for the build's end, and its two factors

**The wait:** a thread waiting for the build's end (`walk_ready`). That is a helper at the
helpers-only barrier, or the leader waiting to join the walk. The prototype spins
`coop::BARRIER_SPINS` (4,096) times, then calls `yield_now` in a loop. It never parks, so it is
never CPU-free.

**Two factors** set the wait, each its own flag:
- **the spin budget** (`SPIN`): 4,096, as registered, or 2,000 (`coop::SHORT_SPINS`, the pool's
  own `SPINS`) with `Levers::spin2k`;
- **what follows the spins** (`PARK`): a yield in a loop, as registered, or a park (lever A,
  `Levers::park`).

**The arms registered here, and what each comparison isolates:**

| arm | spins | then | against the pipe (4,096, yield), it isolates |
|---|---|---|---|
| `pipeK` | 4,096 | yield | (the reference) |
| `pipeK-park` (lever A) | 4,096 | park | **parking alone**, at the same budget |
| `pipeK-spin2k` (the control) | 2,000 | yield | **the spin budget alone**, still yielding |

The fourth cell of the factorial (2,000 spins, then park: the superseded park arm) is built
(`Levers { park, spin2k }`) and tested, but not registered as an arm. `park` against `spin2k`
(PS1, PS2) changes both factors, and is reported, never judged.

### Lever A, park (`Levers::park`)

1. A thread spins 4,096 times, checking the flag.
2. It lists itself in the run's `sleepers` (a mutex-guarded list of `Thread`s, its capacity
   reserved before the run).
3. It parks until the flag is set.

Whoever sets the flag (the build's last helper, in `set_walk_ready`) stores it with release
ordering, then takes the lock and unparks everyone listed.

**No wake-up is lost** (the argument is in `Relay::wait_walk_ready`):
- The waiter lists itself under the lock, then loads the flag (acquire).
- If its lock came first, the setter finds it listed and unparks it. An unpark that comes
  before the park makes the park return at once.
- If the setter's lock came first, the setter's store happened before the waiter's lock, so
  the waiter sees the flag and never parks. Its entry stays in the list, harmlessly: the list
  belongs to the run and is dropped with it.
- A spurious wake-up, or a token left over from an earlier unpark, only makes it check the
  flag again. The same holds for the pool's own parks.

**Unchanged:** the walk itself, the segment counter, the last walker's assembly, and the
pool's own waits. Neither the output nor the errors change.

**What it should do:** cut the CPU that the build waits spend yielding once their spins run
out. The cost is a wake-up latency (a futex wake, a few microseconds) where a waiter had
already parked. The question registered for it is whether CPU falls while the wall holds.

### Lever B, small blocks keep the pipeline primed (`Levers::small`)

**What it replaces:** in a run, the leader takes at most one job.
- If that job's block is below `MIN_COOP`, the leader decodes it whole, on its own one-core
  buffers, and stages nothing.
- So the next run's leader finishes its block alone (the pipeline drained), and the next job's
  move-to-front runs with no helper work beside it.
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

**Its known counterexample:** a leader that takes several small blocks in one run ends its own
work later. So it joins the walk later, or finds it over, and the walk runs with one thread
fewer. More jobs taken can then mean less throughput. Also:
- while the leader decodes small blocks, the helpers, once their block is inverted, wait idle
  in the pool;
- the inverted block i waits unpublished until the next run's leader finishes it, a latency
  the reader may see (it reads in order), bounded by the queue.

These are measured, not assumed away (the evidence below).

### Both (`park` and `small`)

Lever A (4,096 spins, then park) together with lever B. A acts on the waits; B changes how many
jobs the leader takes and so when it joins. They could interact through the leader's join
wait, so their combination is a third arm, registered with the same comparisons.

## What counts as evidence for lever B

**The decisive evidence is three things, all of them required:**
1. **The overlap:** the traces' corrected same-team cross-thread count
   (`pipe.overlap.cross_thread`). A leader is credited only when its move-to-front
   intersects its own team's helpers' build or walk of the block its enclosing run inverts.
2. **The phases, the leader's join above all:**
   - `pipe.ms.lead`: the leader's own work inside the runs, from the new `LeadStart` and
     `LeadEnd` marks;
   - `pipe.ms.join_delay`: summed over the team runs, how long after the build's end
     (`WalkSetupEnd`, on a helper) the leader ended its own work and turned to join;
   - `pipe.n.join_late`, `pipe.n.join_missed` and `pipe.n.join_runs`: the runs whose leader
     joined late, the runs whose leader found the walk over and never walked, and all runs.
   - So the counterexample (more jobs taken, a later join) shows in these figures.
3. **Wall and CPU,** from the timed runs.

**`pipeline_counts()` is context only, never evidence.** It counts jobs taken during a team
run, not useful overlap. The tests use it only to show that lever B's code path runs:
`small_blocks_keep_the_pipeline_primed` takes 6 jobs during runs as registered and 12 with
lever B. That is a check that the code does what it says, not a measure of overlap.

## The tests

Run on this branch. All are in CI's test job.

**Every lever setting against the default** (`tests/pipelined.rs`). The settings are: none,
park, spin2k, small, park with small, and park after the short spin. Each runs at teams of 2,
3 and 4, on 1 or 2 teams, with tight and wide windows. Each test checks the bytes byte for
byte, and the error by kind, block and bit:
- the crate's corpora;
- every sample at levels 1, 3 and 9;
- several streams, with small blocks among full ones;
- one full block, one small block, an empty stream, and no input;
- a block CRC in the middle of a file;
- a bad primary index;
- truncations;
- a move-to-front error on the block a leader staged while the one before was in flight.

**New tests:**
- **`small_blocks_keep_the_pipeline_primed`:** full and small blocks alternating, on one team.
  The bytes are the same, and lever B's code path is shown to run (the jobs taken, as context,
  above).
- **`an_mtf_error_after_a_small_block`:** a full block, a small one, then a block whose
  move-to-front fails. With lever B, the leader stages the failing block in the run that
  decoded the small one. The bytes and the error match the default.
- **Cancellation (`dropped_mid_stream`):** a reader dropped mid-stream, on every plan and
  every lever setting, under a 60-second watchdog. Also a push-form decoder, cancelled by its
  reader while its writer writes, with every lever setting.

**Unit tests** (`coop::tests`):
- `split_build_and_walk_are_build_and_walk` and `pipelined_leader_late_or_early` run all four
  waits (yield or park, after 4,096 or 2,000 spins), with a leader that is early or 30 ms
  late.
- `miri_parked_wait_loses_no_wakeup`: three waiters against the setter, which sets the flag at
  once, after a yield, or after a sleep. A lost wake-up would hang the test.

**Miri:**
- `cargo +nightly miri test --lib -- coop::tests::miri_` passes, all 10 tests.
- That includes `miri_split_build_and_walk`, which now runs the parked waits too, and the new
  parked-wait test.
- The parked-wait test also passes over 16 seeds (`-Zmiri-many-seeds=0..16`).
- `miri_split_build_and_walk` passed over 4 seeds before the spin budget became a parameter.
  Under Miri both budgets are 4 spins, so its waits are unchanged.
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
| | `mibz2-pipe3@10` | 3 × 3 | the registered pipe: 4,096 spins, then yield |
| | `mibz2-pipe3-spin2k@10` | 3 × 3 | the spin-budget control: 2,000 spins, then yield |
| | `mibz2-pipe3-park@10` | 3 × 3 | lever A: 4,096 spins, then park |
| | `mibz2-pipe3-small@10` | 3 × 3 | lever B |
| | `mibz2-pipe3-both@10` | 3 × 3 | levers A and B |
| 12 threads | `mibz2-team4@12` | 3 × 4 | cooperative, not pipelined |
| | `mibz2-pipe4@12` | 3 × 4 | the registered pipe |
| | `mibz2-pipe4-spin2k@12` | 3 × 4 | the spin-budget control |
| | `mibz2-pipe4-park@12` | 3 × 4 | lever A |
| | `mibz2-pipe4-small@12` | 3 × 4 | lever B |
| | `mibz2-pipe4-both@12` | 3 × 4 | levers A and B |

There are no unregistered cells. The report refuses any cell beyond these. Every arm runs
unpinned, as the default does.

## The comparisons

Each comparison has two co-primary figures, both paired by round. Each is reported as the
geometric mean over the rounds, with its 95% t-interval [lo, hi]:
- **speed:** the reference's wall over the arm's, so above 1 means the arm is faster;
- **CPU:** the reference's CPU over the arm's, so above 1 means the arm spends less.

Peak RSS is reported beside them.

**The causal comparisons**, each arm against the pipe (the same plan, threads, window and
placement), with what each isolates:

| | arm | against | isolates |
|---|---|---|---|
| A1, A2 | `pipe3-park@10`, `pipe4-park@12` | `pipe3@10`, `pipe4@12` | parking, at the same spin budget |
| S1, S2 | `pipe3-spin2k@10`, `pipe4-spin2k@12` | the same | the spin budget (4,096 to 2,000), still yielding |
| B1, B2 | `pipe3-small@10`, `pipe4-small@12` | the same | lever B |
| AB1, AB2 | `pipe3-both@10`, `pipe4-both@12` | the same | levers A and B together |

**The noise control:** N1, `mibz2-team3@10` against `mibz2@10`. The two cells run the same
plan, so N1's speed interval must lie within 1 ± 0.03.

**Reported, never judged:**
- R1 and R2: the registered C1 and C2 replicated (`pipeK` against `teamK`);
- each arm against `teamK`: how much of the pipeline's gain it keeps or adds;
- PS1 and PS2: park against spin2k, two factors at once;
- `mibz2-pipe4-park@12` against `mibz2@10`: the product view.

## Rounds and order

- **20 rounds** in each `ab run`. Within a run, `ab`'s `arm_order` rotates the arms one step a
  round, and reverses the order every other cycle.
- **The order is not balanced over 20 rounds.** The 10-thread run has 7 arms and the 12-thread
  run 6, so a full balanced cycle is 14 or 12 rounds. Neither run's 20 rounds is a whole number
  of cycles.
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

**Lever A's registered question (A1, A2): does parking, at the same spin budget, make CPU fall
while the wall holds?** "Holds" means the speed interval lies wholly above 0.98, so any wall
loss is at most 2%, about half of C1's gain.

| CPU interval | speed interval | answer |
|---|---|---|
| lo > 1 | lo > 0.98 | **YES:** CPU falls, the wall holds |
| lo > 1 | hi < 1 | **TRADE:** CPU falls, the wall regresses |
| lo > 1 | otherwise | **CPU FALLS,** the wall not shown to hold |
| hi < 1 | any | **NO:** CPU rises |
| neither | hi < 1 | **NO:** the wall regresses, CPU not shown to fall |
| neither | otherwise | **NO:** CPU not shown to fall |

A YES on both A1 and A2 is the case for parking in the pipe's wait. A YES on one plan only is
reported as plan-dependent.

**The spin-budget control (S1, S2):** both figures' outcomes, as above.
- If S is NULL on both figures, the budget doesn't matter between 2,000 and 4,096 spins, and
  A's effect is parking's.
- If S moves a figure, the budget matters. The superseded design's park arm (2,000 spins, then
  park) would have mixed the two effects, which is why it was replaced. The fourth factorial
  cell is not run, so no interaction between the budget and parking is estimated.

**Lever B (B1, B2):**
- **Its mechanism must be shown in the traces** before any speed outcome is read as lever B's
  doing. Its median `pipe.overlap.cross_thread` must exceed the pipe's at the same plan by at
  least 10% of the blocks (43 of 431).
  - If it does: the mechanism is **SHOWN**.
  - If not: **NOT SHOWN**, and then no speed outcome of B is attributed to more overlap.
  - If either trace is missing or incomplete: **UNRESOLVED**.
- **Its counterexample, a later join, is checked in the same traces.** It counts as a **LATER
  JOIN** when B's median summed `pipe.ms.join_delay` is at least 1.2 times the pipe's and at
  least 5 ms more. `pipe.n.join_missed` is reported beside it.
- **Speed: GAIN or SMALL GAIN, with its CPU outcome beside it.** CPU is co-primary: a speed
  GAIN with a CPU COST is a trade-off to weigh, not an outright win.

**Both (AB1, AB2):** lever A's question applies to the CPU figure, and lever B's mechanism and
join checks to the traces and speed.
- If the levers act independently, AB's speed is about B's and its CPU about A's.
- An AB outcome worse than either lever alone on its own figure (speed for B, CPU for A, each
  interval wholly below that lever's mean) is reported as an **interaction**.

**Null and regression are registered outcomes, not failures of the run.**

| observation | interpretation allowed |
|---|---|
| A: CPU NULL, speed NULL | the waits' yielding after 4,096 spins costs too little CPU on this machine to show. Park is not falsified elsewhere |
| A: CPU SAVING, speed REGRESSION | the parked waiters' wake-up latency costs wall: a longer spin, or a park only once the build's remaining work is long, would be the next step |
| S: a figure moves | the spin budget is itself a lever. A's reading is unaffected (A is at the pipe's budget), but the budget's best value is open |
| B: mechanism SHOWN, no LATER JOIN, speed GAIN | more overlap per block makes the pipe faster, without the leader's join suffering. The gain's attribution is as in the prototype's registration (the overlap, or the leader's extra walking) |
| B: mechanism SHOWN, LATER JOIN, speed NULL or REGRESSION | the counterexample: the extra overlap is paid for by the leader's later join (and the helpers' idle time). A cap on the jobs a leader takes in one run would be the next step |
| B: mechanism SHOWN, no LATER JOIN, speed NULL or REGRESSION | the extra overlap is cancelled by something else: block i's later publication, or the helpers' idle time. The phase times (`pipe.ms.*`) locate it |
| B: mechanism NOT SHOWN | the leader rarely finds a job queued after a small block on this machine (the scanner behind), or the instrument misses it: no verdict on B |
| INCONCLUSIVE | inconclusive, not a falsification |

## The traces

Occupancy passes of every pipelined arm, 6 rounds each, at the default (unpinned) placement:
`--pipe 3` at 10 threads and `--pipe 4` at 12, each with no lever and with `--levers spin2k`,
`park`, `small` and `both` (10 passes).

From the phase marks (src/trace.rs: `Mtf*` to `Lead*`), the report gives, per pass:
- the blocks and the move-to-front count (`pipe.n.mtf`);
- the cross-thread MTF overlap (`pipe.overlap.cross_thread`), as corrected in #247's review;
- the same-thread containment (`pipe.overlap.same_thread`);
- the RLE1 and CRC overlap (`pipe.overlap.rle_cross_thread`);
- the summed build-wait time (`pipe.ms.build_wait`), and the build and walk time;
- the leader's own work (`pipe.ms.lead`) and its join: `pipe.ms.join_delay`, and the late,
  missed and total runs.

The prototype's result is the reference: 147 of 431 blocks overlapped cross-thread at 3 × 3,
140.5 at 3 × 4.

## What no outcome here establishes: α

The result entry's model reading, C1 = max(S, αP0) + H, fitted α from C1's speed ratio. **α
depends on the handoff cost H assumed:** 1.545 at H = 0 ms, 1.468 at H = 0.05 ms, and 1.392 at
H = 0.10 ms. H isn't measured.
- So no outcome of this run establishes α, nor a change in α.
- The report gives no α, and any reading of one from this run is a model's, not a measurement.

## The recipe

`just submit bench-pipeline-levers` runs `.just/bench.just`'s `bench-pipeline-levers`, modelled
exactly on `bench-team-pipeline`:
- one new `mkdir`'d directory per attempt (`target/pipeline-levers/<UTC stamp>`);
- `console.log` written with `tee`;
- an EXIT trap that writes `outcome.txt` and packs the directory with `bench/bundle.sh`,
  however the attempt ends;
- the freeze checks (`bench/pipeline-levers.frozen`, the measured inputs, a clean tree);
- the corpus built from the pinned windows;
- the two `ab run`s, the 10 traces, and the report's verdict.

It takes about 40 minutes. `DRY=1` is the untimed check of the recipe itself (one round, one
trace round, no settling). It is never evidence, and the report never calls it VALID.

## Indicative only: this session's 4-core machine

**Not evidence for the registration.** This was one shared machine with 4 cores. Others' load
rose during the run (a load average of about 3.4 at its end). The runs were 10 paired rounds at
4 threads on the same sample, with the arms as registered here (park at 4,096 spins). Ratios
are as registered above, with the reference the pipe without levers (speed and CPU above 1
favour the arm).

| plan | arm | speed [95% CI] | CPU [95% CI] | median wall, CPU (s) |
|---|---|---|---|---|
| 1 × 4 | `pipe4` (reference) | | | 0.649, 1.905 |
| 1 × 4 | `pipe4-spin2k` | 1.007 [0.944, 1.075] | 1.009 [0.928, 1.099] | 0.656, 1.950 |
| 1 × 4 | `pipe4-park` | 0.939 [0.888, 0.992] | 1.017 [0.950, 1.088] | 0.684, 1.861 |
| 1 × 4 | `pipe4-small` | 1.135 [1.029, 1.252] | 1.061 [0.956, 1.178] | 0.589, 1.842 |
| 1 × 4 | `pipe4-both` | 1.082 [1.001, 1.170] | 1.093 [0.996, 1.200] | 0.606, 1.769 |
| 1 × 4 | `team4` | 0.969 [0.899, 1.044] | 1.056 [0.980, 1.138] | 0.678, 1.855 |
| 2 × 2 | `pipe2` (reference) | | | 0.491, 1.697 |
| 2 × 2 | `pipe2-spin2k` | 0.999 [0.929, 1.074] | 1.004 [0.939, 1.073] | 0.503, 1.740 |
| 2 × 2 | `pipe2-park` | 1.032 [0.948, 1.124] | 1.142 [1.026, 1.271] | 0.484, 1.504 |
| 2 × 2 | `pipe2-small` | 1.147 [1.027, 1.281] | 1.137 [1.010, 1.279] | 0.437, 1.539 |
| 2 × 2 | `pipe2-both` | 1.121 [0.956, 1.315] | 1.165 [0.983, 1.382] | 0.471, 1.599 |
| 2 × 2 | `team2` | 1.097 [1.022, 1.178] | 1.150 [1.061, 1.246] | 0.448, 1.465 |

**What these runs suggest, and no more:**
- The spin budget alone shows nothing here at either plan.
- Park's effect looks plan-dependent: about 14% less CPU at 2 × 2, but a slower wall at 1 × 4.
- Small is above 1 at both plans (+13–15%).
- Both looks like small's speed with somewhat less CPU, within wide intervals.
- A first draft's runs (park at 2,000 spins, in the same session) agreed on small and differed
  on park. That is noise here, or the budget: the i9 run decides.

## What this run can't tell

- Whether lever B's gain comes from the overlap itself or from the leader walking more often.
  That is the prototype's own open attribution question.
- The interaction between the spin budget and parking (the fourth factorial cell isn't run).
- α (above).
- Placement effects (H2, deferred).
- Any machine but the i9-10900K.
- Corpora with a different mix of small and full blocks. Lever B's effect scales with the
  share of small blocks, 198 of 431 here.
