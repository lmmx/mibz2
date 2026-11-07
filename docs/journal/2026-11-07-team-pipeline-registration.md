# 2026-11-07: The pipelined team, registered before any workstation run

**Status:** registered, not run. This entry and the prototype it tests are frozen at one
commit, the one the recipe names (`frozen=` in `just bench-team-pipeline`,
`.just/bench.just`, the commit after this entry's last change). The recipe records, in the
attempt's directory, whether the code and this entry still match it (`NOT-FROZEN.txt` if
they don't). Nothing here changes the
default decoder: the pipelined team is opt-in (`StreamDecoder::pipelined`,
`BzReader::parallel_pipelined`, both `#[doc(hidden)]`).

It tests H1 of the knee entry (docs/journal/2026-11-07-knee-mechanism.md, as revised at
d95ca1ea): pipeline each team, so the leader's serial stages overlap the helpers' parallel
ones.

## The prototype as built (`research/team-pipeline`)

**A pipelined team of k threads**, one run of its pool per block (`coop::invert_pipelined`):
- **Helpers 1..k−1:** block i's fused build, cut into k − 1 shares (`coop::FusedBuild`); the
  last helper to finish its share makes the end check and sets up the walk
  (`coop::RoomWalk::new`); all walk it, segments taken from one counter
  (`512 × (k − 1)` segments); the last thread to leave the walk assembles block i.
- **The leader, meanwhile:**
  1. block i − 1's RLE1 and CRC, from its assembled buffer, and its result published;
  2. the next job, only if one is queued (`try_lock`, `try_recv`: it never waits for one);
  3. that job's move-to-front into the second front set;
  4. then it **joins block i's walk** if the walk is still going.
- **Buffers a team keeps:** two front sets (`ll`, `counts`, `checkpoints`), two assembled
  buffers, one table, one segment scratch, one pool, and a one-core kernel's small set.

**Deviations from the brief, each with its reason:**
- **The leader joins the walk** once its own work is done. Otherwise a team of 2 would
  leave one thread inverting alone and its leader idle for most of each block.
- **A block below `MIN_COOP` doesn't drain the pipeline.** The leader decodes it whole,
  inside the run, on its own one-core buffers (a table of at most 1 MB). The reason: the
  bench's Wikidata sample has 198 of its 431 blocks below `MIN_COOP` (the trace's counts:
  233 team runs, 431 blocks), so draining at each would have measured mostly the drain.
- **The leader takes a job only if one is already queued.** A team never waits for the
  scanner while it holds an unpublished block. That wait could deadlock: the scanner
  waits for a slot the reader frees only once it has that block.

**The waits, and what they cost:**
- **A helper waiting for the build's end** (`BuildWait*` marks): 4,096 spins, then
  `yield_now` in a loop. It never parks.
- **The leader waiting to join the walk:** the same wait.
- **The pool's own waits:** between runs, and the leader waiting for the helpers at the end
  of a run. These are `coop::Pool`'s: 2,000 spins, then `park`.

So a waiting thread is never CPU-free. That cost is in the CPU figures registered below.

**The tests** (`tests/pipelined.rs`, `coop::tests`):
- **Byte for byte the default decoder's output and error** (kind, block, bit), at teams of
  2, 3 and 4, on:
  - the crate's corpora;
  - every sample at levels 1, 3 and 9;
  - several streams, with blocks below `MIN_COOP` among full ones;
  - one full block, one small block, an empty stream, and no input.
- **The default decoder** checked against libbzip2 on the same corpora.
- **Errors:**
  - a block CRC in the middle of a file (the right block, the blocks before it delivered);
  - a bad primary index;
  - truncations;
  - a move-to-front error on the block a leader staged while the one before was in flight
    (the overlap asserted by the decoder's own count, `pipeline_counts`).
- **Cancellation:** a reader dropped mid-stream on every plan, under a watchdog.
- **The split build and walk** at share counts 1, 2 and 3, against `bwt::build` and
  `bwt::walk`; the pipelined run itself; the leader's panic. All also under Miri, on small
  blocks (`miri_split_build_and_walk`).

## The arms

One `ab run`. Arms `mibz2`, `mibz2-team3`, `mibz2-pipe3`, `mibz2-team4` and `mibz2-pipe4`, at
10 and 12 threads, each with the default's window (`SLOTS_PER_THREAD` blocks a team), on the
bench's pinned Wikidata sample (`bench/wikidata.json`, SHA-256 `82f65ace…`). The registered
cells, and the plan each run's threads must show:

| cell | plan | what it is |
|---|---|---|
| `mibz2@10` | 3 × 3 | today's default on the i9 (the policy, `Plan::for_threads(10)`) |
| `mibz2-team3@10` | 3 × 3 | cooperative, not pipelined |
| `mibz2-pipe3@10` | 3 × 3 | pipelined |
| `mibz2-team4@12` | 3 × 4 | cooperative, not pipelined |
| `mibz2-pipe4@12` | 3 × 4 | pipelined |

`ab` crosses every arm with every thread count, so the run also has `mibz2@12` (3 × 3 on the
i9), the arms of 3 at 12 threads (4 × 3) and the arms of 4 at 10 (2 × 4). They are reported
and never judged.

**H2's placement arm is deferred.** Pinned against unpinned, with the actual mapping of
threads to CPUs recorded, needs either a per-thread affinity record in the decoder or a
pinned arm in `ab`. `ab --spread` only pins a run whose threads fit the physical cores (so
not 3 × 4 at 12 on 10 cores), and it can't be interleaved with an unpinned arm in the same
rounds. Every arm here runs unpinned, as the default does.

## The comparisons

Each comparison is a speed ratio: the reference's wall time over the arm's, paired by round,
so above 1 means the arm is faster. It is reported as the geometric mean over the rounds with
its 95% t-interval. CPU and peak RSS are reported beside it, never instead of it.

**The causal comparisons** (the pipeline alone: same plan, threads, window and placement):
- **C1:** `mibz2-pipe3@10` against `mibz2-team3@10`;
- **C2:** `mibz2-pipe4@12` against `mibz2-team4@12`.

**The product comparison** (kept apart: it changes the plan as well as the pipeline):
- **P1:** `mibz2-pipe4@12` against `mibz2@10`, today's default at the budget it is used at.
- P1 also runs two more decoding threads than its reference, on the same 10 cores. A P1
  gain without a C2 gain is a plan effect, not a pipeline one.

**The noise control:**
- **N1:** `mibz2-team3@10` against `mibz2@10`. The two cells run the same plan, so N1's
  interval must lie within 1 ± 0.03.

## Rounds and order

- **20 rounds.** Each round runs every arm once at each thread count.
- **The order is counterbalanced, not random.** Within a thread count, `ab`'s `arm_order`
  rotates the arms one step a round and reverses the order in every other cycle of 5 rounds.
  Over 20 rounds (two full cycles of 10), each arm runs in each position 4 times.
- **The thread counts are not counterbalanced:** 10 always runs before 12 within a round.
  That leaves a fixed position between P1's two cells; C1, C2 and N1 are each within one
  thread count.
- **No randomisation is used,** and none is claimed. The order is `ab`'s deterministic
  rotation, which every earlier workstation run used.

## Validity (checked by `bench/team_pipeline.py report`, not by the reader)

A run is **VALID** only if all of these hold:
- every run of every cell gave the same output hash and length, and none failed;
- each registered cell ran rounds 0–19 once each, and every run's threads show its plan
  (`mibz2-N` leaders and `mibz2-coop-*` helpers, counted from `ab`'s `thread_cpu`);
- N1's interval lies within [0.97, 1.03].

An invalid run has no outcome. A run with uncommitted changes (`git-status.txt`), or with
`NOT-FROZEN.txt` present, is not this registration's run.

## The outcome map

**The timed outcome of C1, C2 and P1, from the speed ratio's 95% interval [lo, hi]:**

| interval | outcome |
|---|---|
| lo > 1.02 | **GAIN** |
| 1.00 < lo ≤ 1.02 | **SMALL GAIN** (above 1, below the threshold: not a reason to adopt) |
| hi < 1.00 | **REGRESSION** |
| 0.97 ≤ lo and hi ≤ 1.03, the interval containing 1 | **NULL** |
| anything else | **INCONCLUSIVE** (the interval spans gain and harm) |

**Null and regression are registered outcomes, not failures of the run.** The knee entry's
model doesn't bound α. At α ≥ 1.6 it predicts 1.01× or less.

**The traces.** Occupancy passes of each registered plan, pipelined and not, 6 rounds each,
at the default (unpinned) placement:
- `--teams 3` and `--pipe 3` at 10 threads;
- `--teams 4` and `--pipe 4` at 12 threads.

The pipelined passes record the new phase marks (src/trace.rs: `Mtf*`, `BuildSetup*`,
`Helper*`, `Build*`, `BuildWait*`, `WalkSetup*`, `Walk*`, `Assemble*`, `Rle*`, `Run*`). From
them the summary gives:
- each phase's summed time and count (`pipe.ms.*`, `pipe.n.*`);
- `pipe.overlap.cross_thread`: the blocks whose move-to-front, on the leader, intersects a
  helper's build or walk of another block. This is the cross-block overlap, shown from the
  marks' clock;
- the same for the RLE1 and CRC (`pipe.overlap.rle_cross_thread`);
- the same-thread containment (`pipe.overlap.same_thread`).

**What an outcome may be read as, with the traces** (the reviewer's map, made numeric):

| observation | interpretation allowed |
|---|---|
| a pipelined trace incomplete, or its median `pipe.overlap.cross_thread` 0 | the implementation or the instrument is unresolved: no mechanism verdict, whatever the timing |
| overlap shown, C1 or C2 a GAIN | the pipeline helps on this machine and workload. The mechanism still needs stage evidence: the pipelined trace's `pipe.ms.build` + `pipe.ms.walk` per team run, against the non-pipelined trace's inversion time per block |
| overlap shown, NULL or REGRESSION, the build and walk per run longer than the non-pipelined inversion | added service cost cancels the overlap (fewer threads on the table, the leader's late join); locate it with the phase times |
| overlap shown, NULL or REGRESSION, the build and walk unchanged, `pipe.ms.build_wait` or the leader's `pipe.ms.rle` + `pipe.ms.mtf` per run above the build and walk | pipeline overhead, or the leader's serial stage, limits it |
| INCONCLUSIVE | inconclusive, not a falsification |

**The model's reading** (reported, not judged): C1's ratio gives the α implied at H = 0
(speed = C0 / max(S, α · P0), with C0 = 1.056 ms and S = 0.40 ms, so P0 = 0.656 ms). It is
valid only where α · P0 > S.

**Also reported, never judged:**
- CPU ratios: the waits spin and yield, so a pipelined arm can spend more CPU at the same
  wall time;
- peak RSS: about 2 MB more a team, from the second `ll` and `checkpoints` set, the second
  assembled buffer and the small table;
- the unregistered cells.

## Indicative only: this session's 4-core machine

Not evidence for the registration. This was one shared machine with 4 cores and a large
L3, so tables never compete for the cache; it isolates the serial-overlap effect alone. The
runs were 10 paired rounds at 4 threads on the same sample.

`ab`'s wall ratio is the arm's over the reference's, so below 1 means faster:

| | prototype | arm against its twin | wall ratio [95% CI] | CPU ratio [95% CI] |
|---|---|---|---|---|
| 1 × 4 | small blocks drained | `pipe4` / `team4` | 0.925 [0.881, 0.971] | 1.078 [1.042, 1.115] |
| 2 × 2 | small blocks drained | `pipe2` / `team2` | 1.087 [1.005, 1.176] | 1.145 [1.086, 1.207] |
| 1 × 4 | as registered | `pipe4` / `team4` | 1.054 [0.904, 1.230] | 1.172 [1.076, 1.276] |
| 2 × 2 | as registered | `pipe2` / `team2` | 1.093 [0.986, 1.213] | 1.073 [1.022, 1.127] |

**The second pair was noisier.** Other processes kept a median of 0.6–1.1 cores busy
during it, against 0.1–0.25 during the first, so its intervals are wide. Its best walls
are pipe4 0.497 s against team4 0.526 s, and pipe2 0.390 s against team2 0.376 s.

**What these runs suggest, and no more:**
- At 2 × 2, pipelining costs wall time and CPU here.
- At 1 × 4, the first pair's 7.5% gain didn't hold to a clear interval in the second.

The i9 run decides.

## What this run can't tell

- Whether a gain comes from the overlap or from the leader's extra walking thread. The
  pipelined team walks with up to k threads, but only after the leader's serial work.
- Placement effects (H2, deferred).
- Any machine but the i9-10900K.
