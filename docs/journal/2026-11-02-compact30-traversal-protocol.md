# Compact30 against the shipped walk: a traversal-comparison protocol (DRAFT for review, not frozen)

- **Status:** proposed, as report 173 asks.
- **Not authorised:** timing, a harness change for timing, a recipe or any workstation
  command. If the guardian approves this draft, the harness and recipe will be written,
  dry-run on the VM (untimed) and frozen, then handed over for the i9.
- **The candidate:** the refined compact30 walker (`examples/c30walk.rs`, `room::<false>`)
  at the commit that adds this draft. Its validation is in the walker journal's §8, and
  its ordinary step is the reviewed 21 instructions.

## 1. The question

At four cores, does compact30's whole traversal make up for its added execution work, by
enough absolute margin to justify building a direct constructor?

**The traversal, on each side:**
- start selection;
- marking and restoration;
- the room walk with its exceptional paths and tail;
- assembly.

**What decides it, and what doesn't:**
- The decision is on the absolute ratio against the shipped walk at four cores.
- The one-core ratio is reported but does not decide.
- Beating FOR-14, or scaling better from one core to four, decides nothing (report 173).

## 2. The arms (one binary, both arms in the same process)

| arm | what is timed | the table it reads | written just before (untimed) |
|---|---|---|---|
| **P** (control) | `mibz2::bwt::walk_segments_room` | the shipped 4-byte T-vector | by `mibz2::bwt::build` (production's build) |
| **C** (candidate) | `C30::walk::<false>` | compact30 records and RAW groups | by a copy from the block's prebuilt oracle image into the worker's arena |

**What P times:**
- production's 511 random starts plus the first;
- START flags set and cleared in the table;
- the 32-lane room walk with L1 prefetch;
- the tail and `SegScratch::assemble`.

This is the same function the decoder calls (`Segmented::invert` = `build` +
`walk_segments_room`), unchanged.

**What C times:**
- the same random starts mapped through φ, plus φ(orig_ptr);
- saving and marking the starts, and restoring them afterwards;
- the room walk with RAW parking and resumption;
- the tail and assembly.

**The matched boundary:** each arm's timing starts after its table is written and ends
when the block's inverse-BWT output is assembled.
- **Both tables are freshly written by the worker just before its walk,** as in the
  decoder, where the walk follows the build.
- **The candidate's writer is a copy, not a constructor.** Its cache state after writing
  approximates a constructor's but is not one, and nothing about construction cost is
  measured.
- **Excluded from both arms:** construction (P's build and C's oracle and copy), MTF,
  RLE1, CRC and the scanner.

**The output** is the same inverse-BWT bytes (production's convention). It is checked
exactly, both arms against each other and against production's, on every block in an
untimed round −1.

## 3. The workload and the conditions

**The corpus:** the primary `wikidata-20261026.bz2` (SHA-256 `cf13f439…`), its blocks
decoded to last columns once, before timing. It has 434 blocks, 230 full; the short ones
use each arm's plain walk.

**One core (W1):**
- one worker, pinned;
- it walks every block in order, under arm P then arm C, with the order alternating by
  round.

**Four cores (W4):**
- four workers pinned to four distinct physical cores;
- each takes a quarter of the blocks;
- they start each arm's pass together from a barrier, so all four run the same arm at
  once;
- every worker records its timing window.

**Rounds:** 20 paired rounds plus an untimed round −1.
- **A round** is a full pass of the corpus under each arm, in alternating order.
- **Round −1** warms the binary and checks every output, and is not used.
- No timing change rides on any other run, and no workstation setting is changed.

**The measure:**
- **Per arm and round:** the summed walk time over the blocks, divided by the summed rows,
  in ns a row. At W4 that is the summed worker time over the summed rows.
- **The ratio:** w = C ÷ P, paired by round.
- **The estimate:** the geometric mean of the 20 round ratios, with a 95% interval from a
  t interval on the logs.
- **Two ratios:** w4 (primary) and w1 (reported).

## 4. Validity gates (G), checked before any decision is read

| gate | condition | if it fails |
|---|---|---|
| G0 output | every block, both arms, exact against production's output in round −1, and each round's output hash equal to round −1's | invalid: report, no decision |
| G1 stability | P's per-round ns a row, max ÷ min, ≤ 1.05 at W1 and at W4 | invalid: report, no decision |
| G2 overlap | at W4, every arm-pass's four worker windows overlap ≥ 95% of the longest | invalid: report, no decision |
| G3 identity | the binary's SHA-256 and `room::<false>`'s instructions match the frozen listing; the corpus hash matches | invalid: no run |

## 5. The construction budget and the decision rules (fixed before any run)

**The planning model** (report 165's, from the earlier phase reconstruction):

> S ≈ 0.592(1 − b) + 1.847(1 − w) CPU-s per decoded GB

- **b** is a direct constructor's cost relative to production's build.
- **w** is the traversal-plus-assembly cost relative to the shipped walk's.
- **Its assumptions:**
  - the coefficients are approximate phase costs, not constants;
  - the rest of the decoder is unchanged;
  - traversal costs measured on this boundary carry into the decoder.

  The last could fail if construction changes the traversal's cache state, or phases
  overlap differently.

**The minimum useful screen:** S ≥ 0.25 CPU-s/GB. That is about 7% of the decoder's
present CPU cost, and smaller than the 1.5–2× aim.

**The construction budget a measured w leaves:**

> b_max(w) = 1 + (1.847(1 − w) − 0.25) / 0.592

| w | b_max: a constructor may cost up to this × production's build |
|--:|--:|
| 0.70 | 1.51 |
| 0.75 | 1.36 |
| 0.80 | 1.20 |
| 0.865 | 1.00 |
| 0.90 | 0.89 |
| 1.00 | 0.58 |

**The decision rules,** on w4's 95% interval [w4−, w4+]:

1. **ADVANCE:** w4+ ≤ 0.865.
   - Even at its interval's worst, compact30's traversal leaves a direct constructor a
     budget of at least production's build cost (b_max(w4+) ≥ 1).
   - This authorises a separately registered constructor design and budget, b ≤ b_max(w4+).
     It does not authorise integration.
2. **STOP:** w4− > 0.865.
   - Even at its interval's best, the traversal leaves a constructor less than production's
     build cost. Production's build is already a fast single pass with a 16-equal-byte
     path, so no credible constructor beats it.
   - Compact30 stops at this checkpoint: the record and code stay, with no further
     compact30 work.
3. **INCONCLUSIVE:** otherwise (the interval straddles 0.865).
   - No automatic rerun and no extra rounds.
   - Report w4, its interval, b_max at both ends, and w1. The guardian decides.

**Read in every outcome, deciding nothing:**
- w1, with its interval;
- each arm's ns a row at W1 and W4;
- each arm's W4 ÷ W1 scaling.

## 6. What it cannot show

- **A production-decoder result:** MTF, RLE1, CRC, the scanner and the pipeline are absent.
- **Construction cost.**
- **Peak RSS** or any memory measure.
- **Other machines or corpora.**
- **Whether a better compact30 schedule exists.** The code is fixed at the reviewed
  listing.
- **Why a result came out as it did.** No counters are collected. A counter capture would
  be a separate registration.

## 7. The records and the cost

- **Records:** one directory under `target/` per run, with a `MANIFEST.json` (commit,
  binary and corpus hashes, the command and the exit status) and one tarball, by the
  convention of report 142.
  - per-round, per-arm, per-worker times and rows;
  - the four-core windows;
  - the round −1 output hashes.
- **i9 time:** a rough estimate, not a measurement.
  - Each round walks the corpus's 207 M rows once per arm.
  - At the 1–3 ns a row such walks have shown, that is a few seconds a round at W1 and
    less at W4.
  - With the untimed table writes, 20 rounds of each come to a few minutes.
