# The zero-copy floor: the decoder's residual with the oracles' copies removed (registration)

*A C1a diagnostic, registered before its data: it decides nothing, and nothing is adopted. Assigned by the Coordinator's QUEUED 7188035640 on #145 (thread 1 of `../plan.md`). Recipe `just bench-zero-copy-floor`, the Pathfinder's, on #909's harness. The zero-copy oracles are Antigravity's (`antigravity/prompt-build-zero-copy.md`, from its `IDEAL-PARTS-2.md` §3). A change after the data is a new registration with its reason.*

## Why

The full matrix's floor (all six parts ideal over all real: 0.299, 0.387, 0.578 and 0.721 at 1, 6, 8 and 10 workers; `results/full/README.md` §4) is mostly the oracles' own handover copies:
- at 1 worker, the handovers sum to the all-ideal wall;
- from 8 to 10 workers, the all-ideal wall rises as the threads copy replayed buffers at once.

So the matrix could not say what the decoder's own residual is. Antigravity's zero-copy oracles hand the recorded outputs over without copying them:
- lock-free stores;
- R's bytes shared rather than copied;
- C's and E's structures borrowed.

They also split emission's handover into the codes' delivery (`HandE_deliver`) and the word spelling (`E_consume`), and add `E_full`, which removes the spelling too. This run measures that floor and its parts directly.

## The binary

- **master with Antigravity's zero-copy oracles** (their PR, at the head the recipe is tagged on);
- **#909's harness:** `examples/character.rs --one` with the record pass, the timed decode and the check pass; the passes hashed off the reader; the per-decode timeout;
- **the research features:** `character`, `scan-oracle` (with `supplied-codes`), `ideal-construction`, `ideal-mtf`, `ideal-rle1` and `ideal-delivery`.

The recipe refuses a head without the zero-copy modes (a `git grep` for `E_full`, `HandE_deliver` and `E_consume`).

## The design

**Nine arms,** every handover zero-copy, at 1, 6, 7, 8, 9 and 10 workers:

| arm | what is ideal |
|---|---|
| `none` | nothing: the decoder as shipped |
| `SECMRD` | all six parts. Emission ideal on the supplied codes: its chase removed, its word spelling (`E_consume`) run |
| `SFCMRD` | all six, emission in `E_full`: its word spelling removed too |
| `SFCMR` | as `SFCMRD`, with delivery real (all but D) |
| `FCMRD`, `SCMRD`, `SFMRD`, `SFCRD`, `SFCMD` | as `SFCMRD`, with the scan, emission, construction, MTF/RLE2 or RLE1/CRC real in turn (all but S, E, C, M, R) |

The all-but-X arms were added at the Coordinator's decision of 26 November, before any data. Against a zero-copy floor, they give each part's room net of its handover as a measurement, where the full matrix could only bound it (`results/full/README.md` §4).

**Fixed across every cell, as in the full matrix:**
- the scanner is pinned to the first CPU of the core whose pinned calibration runs fastest, with every other thread masked off that core;
- the 100 ms pre-spin on every CPU;
- the record pass (all six stores, held in every arm) and the check pass, outside every clock.

**Size and order:**
- 54 cells, each once a block in 8 randomised blocks;
- the ledgers are on in an extra decode of each cell in the first 4 blocks;
- 648 decodes in all, every wall a decode with the ledgers off;
- each decode is limited to 120 s, and one past it is killed and recorded as failed.
- **Estimated time:** 25 to 35 minutes on the i9 (about 2.3 s a process; the first full run came in a third over its estimate).

**The counters, every decode:**
- cycles, reference cycles, and the last level's load and store misses (perf group 5);
- the memory controllers' read and write traffic (the uncore IMC's `data_reads` and `data_writes`), system-wide over the timed decode, where the machine allows it. Where it can't be opened (no such PMU, or `perf_event_paranoid` above 0 without CAP_PERFMON), each decode records why, and the last level's misses and the clock (cycles over reference cycles) stand in.
- The recipe says at its start that `sudo sysctl kernel.perf_event_paranoid=0` enables the IMC counters, and carries on without them otherwise. Whether to set it is Louis's call.

**Left out, for the budget:** the unpinned link.

## The measures (all reported, none decided on)

At each count k, the arms are paired within a block. Every figure is given with its 95% t-interval over the blocks (df 7), or over the traces (df 3) for the ledgers' figures.

1. **The floor:** `SECMRD` over `none`, the geometric mean of the blocks' ratios.
2. **Residual A:** the `SECMRD` wall (ms): emission's word spelling plus the decoder's plumbing.
3. **Residual B, the plumbing residual:** the `SFCMRD` wall (ms), what is left with every part's work and every copy removed.
4. **E_consume** on the wall: `SECMRD` − `SFCMRD`, paired by block (ms). Also, from the `SECMRD` traces, the `E_consume` ledger entry, summed and per worker.
5. **Delivery's share with everything else free:** `SFCMR` − `SFCMRD`, paired by block (ms).
6. **The handover,** from the `SECMRD` traces, each oracle's own ledger entry, summed over its threads and per thread: `hand_S`, `HandE_deliver`, `hand_C`, `hand_M` and `hand_R`. Also their per-thread sum on the workers, and its share of Residual A.
7. **Each part's room against the zero-copy floor:** `SFCMRD` over the arm with that part alone real (all but it ideal), the geometric mean of the blocks' ratios. It is labelled as `registration.md` labels a room:
   - **ROOM** iff its interval's upper bound is ≤ 0.95;
   - **NO ROOM** iff the interval lies within [0.98, 1.02];
   - **UNRESOLVED** otherwise.

   MASKED needs the pairs, which this run lacks. For E, the arm with emission real (`SCMRD`) runs both the walk and the spelling, so E's room here is the whole of emission's work against `E_full`.
8. **The decomposition** at each k: Residual A = E_consume (on the wall) + Residual B, with the handover (6) reported beside it as the copies' part of the plumbing.
9. **Beside them:**
   - each arm's clock (cycles over reference cycles × 3.7 GHz);
   - the last level's misses per decode;
   - the memory controllers' traffic per decode and over its wall (GB/s), where read;
   - peak RSS;
   - the all-real and all-ideal walls against the full matrix's (`results/full/`).

## THE C1a RULE

**VOID** (the whole run), registration.md's rules:
- the machine is not the i9-10900K;
- any cell's timed decode or trace is missing, duplicated or failed. This includes one killed at the timeout, and one whose ideal parts were not wholly replayed, which the harness fails;
- any decode's output is not the registered size (4 × 209,700,196 bytes copied to the caller, or under D discarded in order), or its check pass is not byte-identical to the record pass's;
- any decode has a major fault;
- a removed part shows more than 5% of its real busy time in the ideal arms' traces, at any count. The real figure is the `none` arm's. Each part's own activities, per clarification 5 as corrected:
  - S, the scanner's A2 and A3;
  - E, the workers' A7, and in the `F` arms also `E_consume`, against `SECMRD`'s `E_consume`;
  - C, A6a, A6b and A6c;
  - M, A5;
  - R, A9;
  - D, the reader's `copy_out` (in the arms with D ideal);
  - each part's ideal arms are every arm but `none` with it ideal, the all-but-X arms included;
- the pin is broken in more than 10% of the pinned traces.

**INCOMPLETE** if any traced decode fails an identity.

## The prediction, scored

**Antigravity's Residual B at 1 worker, 14–19 ms** (`IDEAL-PARTS-2.md` §3.1, its table's "Est. Residual B" row at k = 1; marked `[ARGUED]` there). It is scored as in `results/slice-a/scoring-neon-1.tsv`:
- **hit** iff the measured `SFCMRD` wall at 1 worker has a 95% interval overlapping [14, 19] ms;
- **miss** otherwise.

Its other estimates are shown beside the results, unscored: Residual A of about 273, 60–62, 51–55 and 50–54 ms at 1, 6, 8 and 10 workers, and Residual B at the other counts.

## Checks before the data

- **`results/analysis/test_zero_copy_floor.py`:** the analysis on synthetic runs with known walls, 26 checks. Every measure is recovered, the rooms against the floor among them (ROOM and NO ROOM both shown), the prediction is scored both ways, and each VOID and INCOMPLETE fires.
- **The recipe's DRY on the container:** one block, after rebasing onto Antigravity's merged PR.
