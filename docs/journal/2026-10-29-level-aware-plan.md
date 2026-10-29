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
