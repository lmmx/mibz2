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
