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
| `FCMRD`, `SCMRD`, `SFMRD`, `SFCRD` | as `SFCMRD`, with the scan, emission, construction or MTF/RLE2 real in turn (all but S, E, C, M) |
| `SECMD` | all but R: as `SECMRD`, with RLE1/CRC real |

The all-but-X arms were added at the Coordinator's decision of 26 November, before any data. **All but R is `SECMD`, not `SFCMD`.** The zero-copy oracles' `E_full` removes the spelling only where R replays the block (`supplied_codes::Mode::SuppliedFull`, PR 921). With R real, RLE1 needs emission's bytes, so the spelling runs, and `SFCMD` would be `SECMD` under another name. R's room is therefore read against `SECMRD`, with the spelling run on both sides. This was found on a trial build of PR 921, before any data. Against a zero-copy floor, they give each part's room net of its handover as a measurement, where the full matrix could only bound it (`results/full/README.md` §4).

**What F means.** `F` is emission in `E_full` (`supplied_codes::Mode::SuppliedFull`, PR 921), which skips the spelling only where R also replays the block; elsewhere the oracle falls back to `Supplied` and spells. So `F` is registered as E_full only in arms with R ideal. The harness refuses an `F` arm without R, and the analysis VOIDs the run if any `F` arm's trace calls `E_consume` at all.

**The accounting.** PR 921's `FloorAccounting` (`src/ideal/accounting.rs`) stays off. Off, each of its points is one relaxed load. On, each handover reads the clock and adds to counters shared across the workers, which would touch the timed decodes. The traces' ledgers give the same split (each handover's and `E_consume`'s time and calls) without it.

**A known effect of the stand-in: C's store is read cold.** With C's zero-copy borrow, the walk (A7) reads C's 115 MiB store cold, where the old copying oracle had warmed it. On Antigravity's 2-core VM, C ideal alone at 1 worker moved A7 from 1,003 to 1,448 ms (PR 921). So C's room against the zero-copy floor carries a cache-locality effect of the stand-in, not of construction:
- C's room is labelled as measured with a cold store;
- each arm's A7 per worker, from its traces, is reported beside the walls;
- the old copying C oracle as a reference arm at 1 and 8 workers is **not run**: PR 921 replaced C's copying replay with the borrow, so no such mode exists in this binary.

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
7. **Each part's room against the zero-copy floor:** `SFCMRD` over the arm with that part alone real (all but it ideal), the geometric mean of the blocks' ratios. For R, it is `SECMRD` over `SECMD`. It is labelled as `registration.md` labels a room:
   - **ROOM** iff its interval's upper bound is ≤ 0.95;
   - **NO ROOM** iff the interval lies within [0.98, 1.02];
   - **UNRESOLVED** otherwise.

   MASKED needs the pairs, which this run lacks. For E, the arm with emission real (`SCMRD`) runs both the walk and the spelling, so E's room here is the whole of emission made free against `E_full` (the Coordinator's decision, 26 November, before any data). Beside it, reported only and unlabelled: emission's walk against the floor with the spelling kept, `SECMRD` over `SCMRD`.
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
- any `F` arm's trace calls `E_consume` (E_full not in force);
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

Also reported only, not comparable to the i9 and unscored: PR 921's own `SFCMRD` walls on its 2-core Xeon VM, 44, 28 and 21 ms at 1, 4 and 8 workers, beside the measured walls at 1 and 8.

Its other estimates are shown beside the results, unscored: Residual A of about 273, 60–62, 51–55 and 50–54 ms at 1, 6, 8 and 10 workers, and Residual B at the other counts.

## Checks before the data

- **`results/analysis/test_zero_copy_floor.py`:** the analysis on synthetic runs with known walls, 28 checks. Every measure is recovered, the rooms against the floor among them (ROOM and NO ROOM both shown), the prediction is scored both ways, and each VOID and INCOMPLETE fires.
- **The recipe's DRY** (`just bench-zero-copy-floor dry` at f633303f, after PR 921 merged): one block, on a 4-CPU container with no PMU, so not the i9 and not evidence. It is archived as `results/dry/floor-run-20261126T104836Z.tar.gz` (sha256 2b775502…46959c4b).
  - Every phase from checks to run passed. That is 54 cells and 108 decodes, about 4.9 s a process here.
  - Every check pass was byte-identical, every ideal part was wholly replayed, and every trace's identities held. The pin held in 54 of 54 traces.
  - The removed parts read at most 0.57% of their real busy time. `E_consume` read 0.00% in every `F` arm, with no calls.
  - The zero-copy handovers were each under 4 ms summed (under 1 ms per thread at 6 workers and up), against 30–286 ms copied in the full matrix.
  - The report was VOID on the machine alone. The clock, the last level's misses and the IMC were all unread here (no PMU, no uncore).
  - Since the DRY: the prediction is left unscored when there is no interval (one block), where it had read as a miss.
