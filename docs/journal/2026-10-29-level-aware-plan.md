# The level-aware plan: R1 in the decoder (pre-registered)

- **What this implements:** the level policy's rule R1. It was held out at levels 1 and 2
  and repeated at 3 (`2026-10-29-level-policy.md`): on 6 or more physical cores, a stream of
  level 1–3 gets teams of 2 on all the physical cores the threads can use.
- **Unchanged:** levels 4–9 and inputs without a readable header. The -4 and -5 results
  remain boundary evidence and do not widen the rule.
- **Machines below 6 physical cores** plan workers at every level, as before.

## The design

**1. The level, from the input.**
- `BzReader::parallel*` reads the input's first 4 bytes on the calling thread, before it
  returns. `decode_parallel` reads them from its slice. The Python bindings go through
  these two.
- Those bytes, or the error reading them, go to the decoder before the rest. So the decode
  sees the input whole, and its errors are unchanged: tested for an empty input, short
  inputs, a non-bz2 input and a source failing at its first read.
- `StreamDecoder::new` and `parallel_planned` are unchanged.
- `StreamDecoder::for_stream(.., level)` and `Plan::for_stream(threads, level)` are new.

**2. The plan:**
- At level ≤ 3, where `Plan::for_threads` plans teams and all the cores would hold more:
  `Teams { teams: cores / 2, k: 2 }`, 5 on the i9.
- Otherwise exactly `for_threads`' plan. The unit test `the_level_policy` pins every case
  on the i9's topology.

**3. Levels 1 and 2, named for what they are.**
- Their blocks are below `bwt::MIN_COOP` rows, so a team inverts each on one core, and its
  helper never starts. They are one-core workers, five of them where `for_threads` plans
  four.
- The plan stays `Teams` so that a later large-block stream in a concatenation does
  cooperate (point 4). The documentation of `Plan::for_stream` says so.

**4. Mixed-level concatenations.**
- The scanner reads each stream's own header, and each block is read against its own
  stream's level, so any mixture decodes. Only the plan is chosen from the first stream.
- When R1 plans more teams than `for_threads`, a gate (`parallel::Gate`) admits at most
  `for_threads`' team count (4 on the i9) of blocks above 300 000 rows (level 3's most)
  into their inverse BWT at once.
- **Where the gate sits:** after the block's MTF stage (when its length is known) and
  before its table is built. The pass is held through the inversion and given back on
  unwinding.
- So a -9 stream after a -3 one keeps today's live tables. A -9 first stream plans as
  today.

**5. What the gate doesn't bound** (measured below, not assumed):
- **Table capacity:** each worker keeps its table's capacity from block to block. Five
  workers can each hold a 3.6 MB table's allocation, though at most four are in use.
- **Pre-inversion buffers:** each worker's MTF output and symbols exist before its pass.
- **The window:** it scales to the teams, 10 blocks in flight at 20 threads against 8.

All three can raise peak RSS on the mixed corpus.

## Correctness (this PR's tests)

- **`the_level_policy`:** the plans and gates on the i9's topology at every level and
  thread count; no topology, or below 6 cores, gives workers; a large L3 already at a team
  per core pair gives no change; and the header parser.
- **`the_gate`:** 8 threads × 200 passes never put more than the permits inside. A pass
  held through a panic is given back.
- **`mixed_levels_decode_whole`:** -3 then -9, and -9 then -3, decode to the serial
  decoder's bytes in all of these settings:
  - teams under a gate of one permit (so large blocks must wait for each other);
  - workers under the same gate;
  - `BzReader::parallel` and `decode_parallel` at 20 threads.

  Finishing at all is the no-deadlock check.
- **`the_header_read_keeps_the_errors`:** see point 1.
- **The rest:** every existing test passes, with clippy and rustdoc clean.

## The A/B (fixed now; the i9, `just bench-level-aware`)

**The harness:** the release-gate harness (`bench/coop_decoder.py`), whole decoder,
20 rounds, unpinned.
- **Arms:** `mibz2` (the default, `BzReader::parallel`: this PR), `mibz2-auto`
  (`Plan::for_threads`: today) and `mibz2-workers`.
- **Threads:** 4, 6, 8, 10 and all CPUs.
- **The 4-core pass** (`mibz2` against workers on 4 physical cores) runs on -1 and on the
  native corpus.
- **Scoring:** `bench/level_aware.py`, `mibz2` against `mibz2-auto` paired by round at all
  CPUs.

| corpus | role | gate |
|---|---|---|
| -1, -2, -3 | the rule | ≥ 1.05, lower bound > 1.00, RSS ≤ 1.15×, CPU ≤ 1.5× |
| -6, native, -9 then -3 | unchanged (the same plan as today) | lower bound ≥ 0.97, RSS ≤ 1.05× |
| -3 then -9 | mixed, the gate engaged | lower bound ≥ 0.97 (no slower), RSS ≤ 1.15×, CPU ≤ 1.5× |
| every corpus | | one output hash, no errors |
| 4 cores, -1 and native | unchanged (workers below 6 cores) | `mibz2` against workers at 4: lower bound ≥ 0.97 |

**Predictions:**
- **-1, -2 and -3:** close to the held-out figures, 1.150, 1.216 and 1.075, since the
  default now runs that arm's plan and window.
- **The unchanged corpora:** 1.00 within noise.
- **-3 then -9:**
  - The -3 part gains about as -3 does, and the -9 part runs at about today's speed, with
    4 large inversions at once and the fifth team waiting.
  - Overall 1.00–1.05, with RSS up to about 1.10× (one more table's capacity, a wider window).
  - A result below 0.97, or RSS over 1.15×, rejects the gate as designed.

**The PR merges only if every gate is met.** A failure on the mixed corpus alone would
mean redesigning the gate, not dropping R1. That outcome goes back to the user with the
figures.

## The result (measured, the i9, 00e9dd6, 2026-10-29/30): the shipping gate is not met

**The run:** `just bench-level-aware`. Its 25 files are in `2026-10-29-level-aware-plan/`: seven
`coop-decoder-*` report sets, the two `-4cores` passes, `level-aware-scores.md` and the
console.
- Every run of every arm gave its corpus's one output hash, with no errors or deadlocks.
- **The base now includes #114** (the one-pass build), so today's plan at -1 is about 4%
  faster than in the held-out run (1,357 MB/s against 1,305).

**The registered gates** (`bench/level_aware.py`; `mibz2` against `mibz2-auto` at 20 threads,
paired by round):

| corpus | role | throughput | RSS | CPU | gate |
|---|---|--:|--:|--:|---|
| -1 | the rule | 1.092 (1.072–1.111) | 1.05 | 1.01 | **met** |
| -2 | the rule | 1.195 (1.180–1.210) | 1.11 | 1.03 | **met** |
| -3 | the rule | 1.092 (1.065–1.119) | 1.13 | 1.11 | **met** |
| native | unchanged | 0.995 (0.979–1.011) | 1.00 | 1.00 | **met** |
| -9 then -3 | unchanged | 0.997 (0.990–1.005) | 1.02 | 1.00 | **met** |
| -3 then -9 | mixed | 0.986 (0.974–0.997) | 1.14 | 1.18 | **met** |
| -6 | unchanged | 0.983 (0.956–1.009) | 1.00 | 1.02 | **not met** (lower bound below 0.97) |
| 4 cores, -1 | unchanged | 1.000 (0.988–1.012), against workers | | | **met** |
| 4 cores, native | unchanged | 1.008 (0.993–1.023), against workers | | | **met** |

**Verdict, as registered: not every gate is met, so #115 does not merge on this run.**

### The -6 control, investigated (the criteria unchanged)

**1. What differs at -6:** the code path is today's.
- The plan, window and kernels are `for_threads`' (pinned by `the_level_policy`), and there
  is no gate.
- The arm differs from `mibz2-auto` only by the 4-byte header read and its 4-byte first
  chunk. The scanner copies every chunk into its window either way, so that costs one more
  4-byte copy.
- **No mechanism for a 1.7% slowdown was found in the code.**

**2. The rounds (post hoc, for the record; not a verdict):**
- One round dominates: round 12, at 0.791 (the new arm 0.157 s against its usual
  0.128 s). The machine's other processes were busy 0.83 of the time in that run, the
  highest of the 20 rounds; the paired `mibz2-auto` run read 0.21.
- **Without that round:** 0.994 (0.981–1.007), n = 19.
- **The run was noisy throughout.** Its reference intervals were wide, `mibz2-auto`
  against workers at 4: 1.161–1.294. Even with no true difference, a lower bound of 0.97
  is close to what this corpus's noise allows.
- None of this changes the registered verdict. Dropping a round after the fact is not an
  allowed analysis.

**3. What would settle it** (a proposal, not yet registered; the user decides):
- Repeat the -6 control alone (same arms, 20 rounds, same gate), with nothing else
  running.
- The failed run stays in the record either way.
- **A repeat meeting the gate** would show the first as a disturbed run. **A second
  failure** would be treated as a real regression and investigated before any merge.

### The mixed case passes its gate, at measurable costs

-3 then -9 meets its registered gate, but not by a wide margin, and its costs are real:
- **Throughput:** 0.986 (0.974–0.997). The interval excludes no change: a measured 1.4%
  slowdown. That is worse than the predicted 1.00–1.05.
- **Peak RSS:** 1.14× (97 MB against 85), just under the 1.15 budget.
- **CPU:** 1.18× (2.36 s against 2.01).

**Expected against measured:**
- **Expected:** the -3 half alone gains about 9%, so it predicts about +4% overall if the
  -9 half ran as today.
- **Measured:** −1.4%. So the -9 half ran slower than today's plan.

**Hypotheses** (not measured here):
- **The fifth team's worker waits at the gate** holding a block, while that team's
  helper spins before it parks. That would account for the CPU.
- **Head-of-line blocking:** the waiting block can be the reader's next.

**What it means:** a file that starts with a low-level stream and goes on to large blocks
decodes about 1% slower than today, with 14% more memory and 18% more CPU. That is rare
input, but the rule's cost there is not nothing. A design that re-plans, or spawns
helpers only when they can run, could remove it; it is not designed here.

### The -6 repeat (registered before it runs; the user's decision, 2026-10-30)

- **The run:** `just bench-coop-decoder --file corpora/wikidata-bz6.bz2 --rounds 20 --arms
  mibz2-workers,mibz2-auto,mibz2 --gate mibz2 --no-small`, on the i9, at the PR's head, with
  nothing else running.
- **The scoring:** `bench/level_aware.py`, the unchanged gate as registered: `mibz2` against
  `mibz2-auto` at 20 threads, lower bound ≥ 0.97, RSS ≤ 1.05×, one output hash.
- **The first run stays in the record as not met,** whatever the repeat shows. **Amended
  by the user before the run:** a passing repeat adds evidence but doesn't erase the
  failure, and a second failure doesn't by itself prove a regression in the code.
- **Both runs are scored separately and pooled** (40 paired rounds, the same statistic),
  and the shipping decision weighs them together. Neither is dropped, and nothing in
  either run is excluded.
- **If the pooled lower bound is below 0.97, or the repeat fails alone,** the -6 path is
  investigated before any merge, starting from what differs: the header read and the
  first chunk.
- **The mixed case's cost** (above) is a separate decision, not settled by this run. The
  user does not accept it before it is understood: it is investigated from the existing
  traces and code first, with no code change and no broad new benchmark.

### The mixed case's cost, investigated from the run's own records (no new run, no code change)

**What the records hold:** each run's CPU by thread (`thread_cpu`), its busy threads and
its run queue, from `ab`'s sampler. -3 then -9, 20 threads, medians over 20 rounds:

| | new (5 teams) | today (4 teams) |
|---|--:|--:|
| per-worker CPU | 0.23–0.25 s, all five even | 0.25–0.26 s each |
| workers, total | 1.205 s | 1.025 s |
| per-helper CPU | 0.15–0.17 s, all five even | 0.16–0.17 s |
| helpers, total | 0.78 s | 0.65 s |
| busy threads | 13 | 11 |
| wall | 0.282 s | 0.279 s |

**Readings:**
1. **No team is stuck at the gate.** The extra CPU is spread evenly over all five workers
   and five helpers; no worker or helper stands out as idle or as waiting. So the waits
   pass between whichever workers arrive fifth.
2. **Spinning helpers are not the main cost.**
   - From the code: a helper spins at most 2,000 `pause` iterations (`coop::SPINS`, about
     75 µs) between jobs, then parks.
   - On this file's 934 blocks over five teams, that bounds the spin at about 0.07 CPU-s,
     against the 0.35 measured.
   - The hypothesis that the waiting team's helper spins is not supported as the main
     cause.
3. **The cost is all in the -9 half** (derived: -3 alone against the mix, assuming the
   halves add):

   | | -3 half (from -3 alone) | the whole mix | so the -9 half |
   |---|--:|--:|--:|
   | wall, new − today | −9.7 ms | +2.9 ms | **+12.6 ms** (about +8% of today's 0.154 s) |
   | CPU, new − today | +0.105 s | +0.354 s | **+0.25 s** (about +23% of today's 1.09 s) |

   The gate admits only 4 large inversions at once, but the -9 half doesn't run as today's
   4 teams do.
4. **Where the code says the difference is:**
   - A worker passes the gate after its block's MTF stage. So while four large blocks are
     inverted, the fifth worker still runs a large block's MTF stage. That reads its
     symbols (up to 1.8 MB) and writes its 0.9 MB last column, and the worker then holds
     both while it waits.
   - So the gate bounds the tables, but not the other per-block working data, nor the
     thread count: 13 busy threads on 10 physical cores, against 11.
   - **Two candidate causes:** extra working data beside four 3.6 MB tables, at the cache
     knee the knee journal measured; and threads beyond the physical cores, on SMT
     siblings. These records can't separate them; they have no counters.
5. **The memory, accounted** (an estimate from the module's documented sizes, not
   measured):
   - +12 MB of peak RSS is about the fifth worker's buffers (about 7 MB a worker:
     `parallel.rs`' module documentation) plus two more blocks in the window (10 against
     8, about 2.8 MB each with symbols and output).

**The smallest change the evidence points to** (proposed, not made):
- Gate a large-block stream's blocks **when a worker takes the job**, before its MTF
  stage, not before its inversion. The scanner knows each block's stream level
  (`Event::Block`), so the job can carry it.
- Then during a large-block stream at most 4 workers hold large blocks, as today's plan
  runs. The fifth worker waits holding nothing, and its helper parks.
- **It addresses both candidate causes at once:** no extra per-block working data and no
  extra busy threads during the large blocks.
- **Predictions if it is made:**
  - the -9 half returns to today's speed and CPU (each within noise of 1.00);
  - the mix's overall result becomes the -3 half's gain, about +4%;
  - RSS stays above today's by the fifth worker's retained buffers and the window: an
    estimated 1.05–1.10×.
- **Unchanged by it:** -1, -2 and -3 alone, since a level ≤ 3 stream passes freely, and
  every control, since the gate is only built under R1.
- **Its test:** a focused A/B of the same arms on -3 then -9, -3 alone and -9 then -3.
  The -6 repeat can run in the same session, since the change doesn't touch that path.

## Revision 2: the gate taken when a worker takes the block (approved by the user, 2026-10-30)

**The record so far stands:**
- the first A/B (00e9dd6) did not meet its shipping gate, at -6;
- the mixed case met its gate at a measured cost;
- the investigation above.

A revised design needs its own validation. A pass of it does not make the first run's
verdict a pass.

**What changed** (the smallest change the investigation pointed to):
- **Each job carries its stream's level,** from the scanner (`Event::Block`).
- **The gate admits at most `for_threads`' team count** (4 on the i9) of workers at once on
  blocks of streams above level 3. A worker takes the pass when it takes such a block,
  before any of the block's work (MTF, table, RLE1, CRC), and holds it until the block is
  done.
  - It is given back on an error, and after a caught panic, when the pass drops.
  - Blocks of streams at level 3 or below pass freely.
- **So during a large-block stream** at most 4 workers hold large blocks, as today's plan
  runs. The fifth waits holding nothing but the job's symbols, which the scanner had made
  anyway, and its helper parks.
- **Removed:** the gate in `decode_block`, after MTF, and its field in `Scratch`.
  `decode_block` is as on master again.

**What is not established:** which of the two candidate causes (the extra per-block
working data, or the extra busy threads) the first design's cost came from. The change
addresses both. Neither is presented as proven.

**Tests:**
- all of revision 1's, updated: `the_gate` now on levels; the policy table's bound is
  now `(level 3, 4 permits)`;
- **`gated_errors_end_the_decode`:** a bad block in a -9 stream, after a -3 one and first,
  under a one-permit gate on teams and on workers. It ends with the serial decoder's error,
  within a minute, so the passes are given back on the error path and nothing deadlocks;
- every test passes; clippy, fmt and rustdoc are clean.

### Its validation (registered before it runs)

**The run:** `just bench-level-aware-revised`, on the i9, at the revision's head, with
nothing else running.
- **Arms:** the same as before (`mibz2`, `mibz2-auto`, `mibz2-workers`), 20 rounds,
  threads 4–20.
- **Corpora:** -3 then -9, -3 alone, -9 then -3, and -6.
- **Scoring:** `bench/level_aware.py`, at 20 threads:

| corpus | role | gate (unchanged from the first run) |
|---|---|---|
| -3 then -9 | mixed | lower bound ≥ 0.97, RSS ≤ 1.15×, CPU ≤ 1.5× |
| -3 | the rule | ≥ 1.05, lower bound > 1.00, RSS ≤ 1.15×, CPU ≤ 1.5× |
| -9 then -3 | unchanged | lower bound ≥ 0.97, RSS ≤ 1.05× |
| -6 | unchanged, the repeat | lower bound ≥ 0.97, RSS ≤ 1.05×; scored alone, and pooled with the first run's (`--pool`) |
| every corpus | | one output hash, no errors |

**Hypotheses** (recorded, not acceptance criteria):
- **-3 then -9:** the -9 half runs at about today's speed and CPU. The whole file gains
  about the -3 half's share, roughly +4%, with RSS about 1.05–1.10×.
- **-3 alone:** close to the first run's 1.092. The change doesn't touch a level ≤ 3
  stream's path beyond the level each job carries.
- **-9 then -3 and -6:** about 1.00.

**Not re-measured in this session** (the user's scope: no broader sweep):
- **-1, -2, the native corpus and the 4-core pass.** Their measured results are the first
  run's, on 00e9dd6.
- The revision changes their path only by the level each job carries, and (-1, -2) by a
  pass that every block of theirs is admitted without.
- **For the merge decision:** CLAUDE.md asks that the merged head be the one measured, or
  that its later commits change nothing the bench measures. This revision does change
  every path's code, slightly. So whether the first run's figures stand for those corpora
  is put to the user with the results, not assumed.

### The final confirmation (registered before revision 2's results; the user's condition)

**The rule:** the first run's -1, -2, native and 4-core results support the change, but do
not validate revision 2's head. **#115 merges only after a confirmation on the head to be
merged,** run only if revision 2's focused validation passes.

**The run:** `just bench-level-aware-confirm`.
- -1, -2 and the native corpus, with the same arms, 20 rounds and threads 4–20.
- The 4-core pass on -1 and on the native corpus.

**The gates, unchanged from the first run:**

| corpus | role | gate |
|---|---|---|
| -1, -2 | the rule | ≥ 1.05, lower bound > 1.00, RSS ≤ 1.15×, CPU ≤ 1.5× |
| native | unchanged | lower bound ≥ 0.97, RSS ≤ 1.05× |
| 4 cores, -1 and native | unchanged | `mibz2` against workers: lower bound ≥ 0.97 |
| every corpus | | one output hash, no errors |

**The merge decision then weighs the evidence together:**
- the first run, recorded as not met;
- revision 2's focused validation;
- the -6 repeat, alone and pooled;
- this confirmation.

No single run decides it alone.

## Revision 2's validation (measured, the i9, 2591867, 2026-10-30): not met, and the mixed case worse

**The run:** `just bench-level-aware-revised`. Its 14 files are in
`2026-10-29-level-aware-plan/revision-2/`. Every run gave one output hash, with no errors.

| corpus | role | `mibz2` / `mibz2-auto`, 20 threads | RSS | CPU | gate |
|---|---|--:|--:|--:|---|
| -3 then -9 | mixed | **0.945 (0.930–0.959)** | 1.15 | 1.04 | **not met** |
| -3 | the rule | 1.078 (1.060–1.097) | 1.13 | 1.11 | met |
| -9 then -3 | unchanged | 0.999 (0.991–1.007) | 1.00 | 1.00 | met |
| -6, the repeat | unchanged | 0.985 (0.971–1.000) | 1.01 | 1.02 | met, alone |
| **-6, pooled with the first run** (40 rounds) | unchanged | **0.984 (0.969–0.999)** | 1.00 | 1.02 | **not met** |

**Verdicts, as registered:**
- **Revision 2's validation is not met.** The mixed case is worse than revision 1's 0.986.
- **The -6 control is not met when pooled.** Its lower bound is 0.969, 0.001 short.
- The first run's not-met verdict stands. **#115 does not merge.**

**The hypotheses registered with it, scored:**
- **The mixed file +4% with RSS 1.05–1.10×: wrong.** It is −5.5%, with RSS 1.15×.
- **The -9 half back to today's: half right.** Its CPU is back; its time is worse (below).
- **-3 alone near 1.092:** 1.078, near.
- **-9 then -3 and -6 near 1.00:** 0.999 and 0.985.

### The two analyses (from these records; no new run)

**1. The mixed file, split** (-3 alone subtracted, assuming the halves add):

| the -9 half, new − today | revision 1 | revision 2 |
|---|--:|--:|
| wall | +12.6 ms (about +8%) | **+23.9 ms (about +15%)** |
| CPU | +0.25 s (about +23%) | **−0.02 s (about today's)** |

- **The -3 half gains about as before:** −8.3 ms here, against −9.7 ms in revision 1.
- **So the loss is still in the -9 half, and doubled,** while its CPU returned to today's.
- **The additivity assumption is a limit:** these records can't show what happens at the
  boundary between the streams, or whether the -3 half suffers inside the mix.

**2. The five workers' CPU is now uneven:** 0.18, 0.205, 0.22, 0.23 and 0.24 s, against
0.25–0.26 s each in today's plan and 0.23–0.25 s each in revision 1. Some workers spent much
more of the decode not working.

**3. What the code says the difference is** (a hypothesis to test, not a finding):
- **Revision 2:** a worker takes the next job from the shared queue, then waits for a pass
  holding it. Jobs are taken in stream order, so the waiting block is often the oldest in
  flight.
  - The reader takes blocks in order, so it waits on that block.
  - That block's whole decode starts only after another block's ends.
  - The window (10 blocks) fills behind it, and the scanner waits for a slot.
- **Revision 1:** the waiting block had its MTF stage done before it waited, so it finished
  sooner once admitted. That costs more CPU (the extra work, the extra busy thread) and
  less time.
- **Today's plan:** a worker takes a job only when it is free to start it, so no job waits.
- **Why this is the leading hypothesis:** both gates hold a taken job they can't start. The
  order of their time losses (revision 2 worse than 1, both worse than today) and of their
  CPU (revision 1 above today, 2 at today's) fit it.
- **Not shown:** that the reader's waits, rather than something else, account for the
  +23.9 ms. These records have no per-block timeline.

**4. The -6 control,** separately from the mixed case:
- **The two runs agree:** 0.983 and 0.985. Pooled, the interval excludes no change.
- **So the first run is not simply disturbed:** a slowdown of about 1.5% at -6 now has
  some support.
- **Its cause is unknown.** The code at -6 differs from today's plan only by the header
  read, its 4-byte first chunk and the level each job carries; no mechanism was found in
  any of them.
- **One explanation not ruled out:** a difference between the harness's two arms that
  predates the PR. `mibz2` decodes through `BzReader::parallel`; `mibz2-auto` decodes
  through `parallel_planned`, with its slots set by `ab`.

### What would settle each question (proposed; nothing run, nothing redesigned)

- **Revision 2's waits:** one traced decode of -3 then -9 per design, with the decoder's
  `trace` feature, on the i9. Its marks (`JobGot`, `DecodeStart`, `DecodeEnd`, `Ready`,
  `ReadWait`, `Taken`) give each block's wait at the gate and each wait of the reader.
  A diagnostic of seconds, not a benchmark.
- **The -6 gap:** an A/A run on master, `mibz2` against `mibz2-auto` at -6, without this
  PR's code. If master shows the same 0.98, the gap belongs to the arms, not to the PR.

## The diagnostics (the user's decision, 2026-10-30; not shipping gates)

**Their standing:**
- **#115's shipping decision is made:** the automatic level-aware plan is a failed shipping
  experiment, and #115 does not merge. The small-block speedups stay a valid research
  result.
- **These two runs are diagnostic.** Neither is a gate, neither is scored against one, and
  neither reopens the decision, whatever it shows. A plausible explanation of a failure
  does not make the failed design pass.
- **Nothing else is run:** no gate is redesigned, no new optimisation is tried, no further
  sweep is made.

**The run:** `just bench-level-aware-diagnose` (`bench/level_aware_diagnose.sh`), on the
i9, about 8 minutes. Each build is made from its own commit in a worktree of its own.

### 1. The traced decode of -3 then -9 (`examples/occupancy.rs`, the `trace` feature)

- **The builds:** today's plan (master), revision 1 (00e9dd6: the gate before the
  inversion) and revision 2 (this branch: the gate when a worker takes the block).
- **The run:** `wikidata-mix39` at all CPUs, 6 rounds per build. Each round runs the
  decode untraced, with the timeline, and with full tracing, in rotated order.
- **The question:** do the gated designs' extra wall times come from blocks held in the
  in-order output: a worker holding a job it can't start, so the reader waits behind it?

**What the records can show:**
- **Revision 2's gate wait:** the share of the span its workers spend between taking a job
  and starting its decode (the `buffer` state, which today's plan keeps near zero).
- **Revision 1's gate wait:** inside the decode, so in each block's off-CPU time (`full`:
  elapsed less CPU per block).
- **The in-order output:** the reader's waiting share (`r.waiting`, and `next_ready`, its
  waits with the next block ready); the bytes held behind the head of the line
  (`q.ready_mb_mean.head_of_line`) and the longest hold (`q.hold_ms_max`); the scanner
  waiting for a slot whose earliest block is still decoding (`s.slot_earliest_decoding`).
- **The workers' idle time,** by cause (`w.idle.*`).

**How it will be read (fixed now):**
- **The hypothesis has support** if both gated designs show their gate wait (revision 2 in
  `buffer`, revision 1 off-CPU), and the reader's waits or the head-of-line holds rise
  over today's in the same order as their wall-time losses (revision 2 above revision 1,
  both above today's).
- **It is weakened** if the gate waits are small against the span, or if the reader's waits
  and the holds don't rise over today's. The loss would then lie elsewhere (cache sharing
  among more teams, say), and is left unexplained.

**What it can't establish:**
- **The tracer's records are whole-file:** they don't split the -3 half from the -9 half,
  in which alone the gate engages.
- **Tracing perturbs timing.** The tool reports its own perturbation against the untraced
  rounds; a design's reading counts only where that perturbation is small.
- **Six rounds give a mechanism's direction and rough size, not a measured effect.** They
  are no substitute for the A/B, which stands as measured.

### 2. The A/A on master (`ab` built from master)

- **The run:** master's `mibz2` (`BzReader::parallel`) against its `mibz2-auto`
  (`parallel_planned`, with its slots set by `ab`) on -6 at 4, 6, 8, 10 and all threads,
  20 rounds, the A/B's arms and settings. No code of #115's is in the build.
- **The question:** did the -6 gap (0.983 and 0.985, pooled 0.984 (0.969–0.999)) predate
  #115?
- **Scored without gates** (`bench/level_aware.py --no-gates`): the tables only.

**How it will be read (fixed now):**
- **The gap predates #115** if master's ratio at all CPUs has its interval wholly below 1,
  near 0.98. The gap would then lie between the harness's two arms, not in the PR.
- **The gap is not the arms'** if master's interval lies wholly above 0.985 (above the
  pooled point) and includes 1. It would then stay with #115, unexplained.
- **Otherwise, inconclusive.** An interval of about ±1.5%, like those of the earlier runs,
  can include both 0.984 and 1.

**What it can't establish:**
- **A gap on master explains the -6 result only as the arms' difference.** It doesn't say
  which part of that difference (the slots, the reader or the call path) is responsible.
- **No gap on master doesn't locate a cause in #115's code:** none has been found there.
- **One run of 20 rounds.** The earlier -6 runs show how much one run can differ from
  another.
