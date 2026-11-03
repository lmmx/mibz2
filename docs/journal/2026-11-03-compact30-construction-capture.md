# Compact30: the registered construction-cost capture on the i9 — INVALID

**The registered outcome is INVALID: a validity gate failed, so there is no decision.** The
capture ran under report 200's authorisation with the pinned recipe at `14d9ecb` (protocol
draft 4), consuming the verified dry-run archive `f75b00e8…40f9f9`. It completed all 20
rounds, every correctness gate and the memory, fault and identity gates passed, and three
validity gates failed:

- **G1 (stability), W1 only:** production's per-round one-worker CPU cost varies by
  max ÷ min 1.227 (warm) and 1.223 (cold) against the 1.05 limit.
- **G2 (simultaneous start, per step, report 197):** one all-full W4 step out of 4,560 had a
  start skew of 4.12%, against the 2% limit.

No rerun was made, and nothing was changed. The guardian decides what follows.

## The run

- **When and where:** the i9-10900K, 2026-11-03 at 17:29 UTC, from `/home/louis/dev/mibz2`:

      just bench-c30-construction target/walk-decoder/work/corpora 20 capture target/c30-construction-20261103T172121Z-dry-run.tar.gz

- **The archive**, kept here as
  [c30-construction-20261103T172906Z.tar.gz](2026-11-03-compact30-construction-capture/c30-construction-20261103T172906Z.tar.gz):
  5,301,416 bytes, sha256 `47c61c87c7433cf738a7ffb9ea03905384fbe28d21c00ea17f0a60953bfed5f3`
  (matching the console), 69 files and `MANIFEST.json`, commit `14d9ecb`. It holds the
  dry-run evidence archive and its checksum.
- **Preconditions** (by the recipe, before any build): the corpus hash; the frozen commit in
  its clean worktree; rustc 1.97.0 (`2d8144b78`), LLVM 22.1.6; 94 analysis and 6 listing
  tests passed; the timed path's machine code is the reviewed listings (G3 yes); workers on
  CPUs 1–4; the barrier test passed; the dry run's evidence verified.
- **The collector** ran round −1 and rounds 0–19 and exited 0. The six peak-RSS processes
  ran.
- **Independent replay:** the pinned analyser (`bench/c30_construction.py` at `14d9ecb`),
  run here on the archive's own records, RSS files and dry-run evidence, reproduces the
  archived `c30-analysis.json` exactly (kept as
  [c30-analysis-replayed.json](2026-11-03-compact30-construction-capture/c30-analysis-replayed.json)).

## The checks

- **Schedule:** S0–S10 all true.
- **Gates that passed:** G3 identity, G4 allocation, G5 clock overhead (largest share 0.21%),
  G6 exact output, G7 round −1, G8 one-pass, G9 memory, G10 major faults; G1 for P's W4 cost
  and W4 completion time in both regimes.
- **Gates that failed:**

  | gate | criterion | measured |
  |---|---|---|
  | G1 warm, W1 cost | P's per-round max ÷ min ≤ 1.05 | 1.2274 (min round 12, 0.4430 ns/row; max round 0, 0.5437) |
  | G1 cold, W1 cost | the same | 1.2232 (min round 10, 0.4430; max round 2, 0.5419) |
  | G2 | every all-full W4 step's start skew ≤ 2% | 1 of 4,560 steps over: round 7, warm, P, step 1, 4.12%; the next largest in that pass 0.11%; the row-weighted aggregate's largest 0.09% |

- **What G1's failure looks like:** P's warm W1 cost is bimodal across rounds, at about
  0.521 ns/row in most rounds and 0.443–0.468 in rounds 6, 8, 12, 13, 14, 16 and 18, with
  intermediate values in others. The cold W1 pattern is similar. C's W1 cost varies by 1.054
  (warm) and 1.042 (cold), and both arms' W4 cost and completion time by 1.022–1.028. G1 as
  registered tests P only. The cause is not established here.
- **Memory:** estimate 1,855,326,234 bytes; verified dry-run peak 1,769,914,368; required
  1,855,326,234 plus the 2 GiB margin; `MemAvailable` 130,761,359,360; capture peak RSS
  1,773,305,856 (0.956 of the estimate); preflight and postflight OK.
- **Faults:** no major faults. Minor faults, reported only: warm P 0, warm C 0, cold P
  2,438, cold C 218.

## The figures the invalid run recorded (no decision)

Under the protocol (§7) an invalid run's figures are reported without a decision. All are
C ÷ P, as the geometric mean of the 20 paired rounds with a 95% t interval:

| ratio | point | 95% interval |
|---|---:|---:|
| warm b4 (four-worker CPU cost) | 1.3158 | [1.3108, 1.3208] |
| warm e4 (four-worker completion) | 1.3641 | [1.3597, 1.3685] |
| warm b1 (one-worker CPU cost) | 1.4473 | [1.4045, 1.4913] |
| cold b4 | 1.3357 | [1.3279, 1.3435] |
| cold e4 | 1.3873 | [1.3786, 1.3960] |
| cold b1 | 1.4208 | [1.3781, 1.4647] |

- **By round:** the warm four-worker cost ratio lies between 1.2945 and 1.3357 in every one
  of the 20 rounds (cold: 1.2979 to 1.3572).
- **By block size (warm):** full blocks b4 1.3151 [1.3101, 1.3201] and e4 1.3639; short
  blocks b4 1.7535 [1.7037, 1.8048] and e4 1.5314; the one mixed step e4 1.3528.
- **The registered thresholds** were b4 upper ≤ 1.06714 and e4 upper ≤ 1.02 for PASS, and
  either lower bound above its threshold for FAIL. These figures don't decide anything,
  because the run is INVALID.

## What this record does not establish

- It doesn't decide PASS, FAIL or INCONCLUSIVE: the registered rule makes any gate failure
  INVALID, with no decision, whatever the figures.
- G1's failure is in P's one-worker cost, a reported (non-deciding) measure, but G1 as
  registered covers it, and its bimodality is unexplained.
- G2's failure is a single step, the case report 197 accepted the per-step rule might reject.
- Nothing here bears on the closed traversal result, the audited constructor, or production.
