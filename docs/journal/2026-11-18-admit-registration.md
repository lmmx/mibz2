# 2026-11-18 · Pathfinder · Wider admission for the lazy kernel: its i9 adoption registration, frozen before data

Direction: the Coordinator's QUEUED 6973253342 on #145, and its reply to the Pathfinder's REVIEW 6973651863 ("build `bench-admit`").
- **The question:** can Cells stop writing its intermediate pre-RLE output? The answer was no, about 0.15 B of the 11.0 B of DRAM traffic per output byte.
- **What the census found instead:** the blocks Cells refuses carry about a third of the decoder's DRAM traffic.

This registers the run that tests admitting them. The history:
- first posted as REVIEW 6974203358;
- corrected by the Researcher's REVIEW 6974376850: a targeted correction, then PROCEED with the 10-round wall design.

This file carries all six corrections:
1. the guards bound to their registered digests;
2. a pre-flight that VOIDs before any timing;
3. the scope, with the newly admitted blocks as their own stratum;
4. the traffic share;
5. the knee, matched by round;
6. the IMC reader's controller checks.

## What it decides

Whether production's lazy kernel admits blocks up to r/n 0.15 with a budget of 0.32·n (`bwt::WideLazyCells`), in place of r/n 0.10 and 0.16·n.
- *What stays the same:* the induction and the writer are master's own code. The sparse gate, the plan, the window and the threads are master's.
- *What adoption would change:* only which blocks the lazy kernel admits.

## Why: the census

**The method:** cachegrind on `ab child … 1`, rustc 1.99.1, with the last level set to 3 MiB, a worker's share of the i9's 20 MiB L3 at seven workers.
- *What it counts:* a write-miss is a later write-back and costs an RFO read first, so DRAM ≈ 2 × write-misses + read-misses.
- *The model against the i9's memory controller* (`knee/run-20261117T230729Z`, seven workers): 11.2 B per output byte against the measured 11.0 (writes 4.37 against 4.0, reads 6.82 against 7.0).

**Where the refused traffic is** (correction 4):
- *The refused route as a whole:* the T-vector route the refused blocks take carries about a third of the modelled traffic, counting its writes, their RFOs and its reads: about 3.24 of 11.2 B per output byte. Those blocks are 15.9% of Wikidata's bytes.
- *The T-vector walk's reads alone:* 1.40 B per output byte, about 12.5%.
- *Which blocks those bytes are:* 13.7% of all bytes are 32 full 900 KB blocks at r/n 0.097–0.148, needing 0.161–0.241 cells a row. The admitted blocks lie at r/n 0.059–0.094 and need at most 0.156.
- *What stays refused:* another 2.1% is 5 blocks at r/n 0.15–0.3. The other 150 refused blocks are stream tails of 24 to 60 bytes.

**The guards' blocks:**
- *Text* (`ab gen` at scale 16) lies at r/n 0.28 and up, so all 49 of its blocks stay refused.
- *Random bytes* take the column path and stay refused too.

**Why not admit everything:** forced admission (`ForcedLazyCells`, no gates and a budget of n) costs 4.1× the wall on text and 4.0× on random at one thread, as those blocks need 0.58–1 cells a row. Wide admission leaves both alone.

**Wide admission's counts,** per output byte of `corpora/wikidata.bz2`, on the binary the i9 runs:

| | production | wide admission |
|---|--:|--:|
| DRAM, the model (writes + RFO + reads) | 11.41 | **9.27 (−18.8%)** |
| writes | 4.52 | 4.12 |
| reads, RFO included | 6.90 | 5.15 |
| instructions | 27.45 | 29.58 (+7.8%) |

**Wall and memory on this VM** (one thread, best of 3): Wikidata 0.702 s against 0.721; text 0.308 against 0.311; random 0.574 against 0.591.
- *Peak RSS, one worker:* Wikidata 41.1 MB against 38.0; text and random within 10%, lower or level.
- *Peak RSS at 8 and 10 workers:* medians of 158 against 166 MB and 192 against 191 MB.
- *The outputs:* byte-identical everywhere.

## The run (`just bench-admit`)

**The arms:** production's decoder from two builds, both by rustc 1.99.1 (`RUSTUP_TOOLCHAIN=1.99.1`; the recipe refuses without it).
- *`mibz2@build-master`:* master at `MASTER` (the recipe's default, the commit this branch has merged; checked).
- *`mibz2-wide`:* this commit's `BzReader::parallel_lazy_as::<WideLazyCells>`, production's plan, window and threads with the lazy kernel's admission widened.
- *Recorded:* both builds' `rustc -vV` and `cargo -vV` (required to agree), the master worktree's identity and clean source, and both binaries' SHA-256.

**The cells:** 10 paired rounds, unpinned on the whole machine (`ab run` without `--spread`), with no spinner.
- *Judged:* 8 and 10 threads asked. On the i9 that is production's eight lazy workers with 32 in flight, and its seven with 28 (`trims_to_seven`, unpinned).
- *Corpora:* 4 copies of the Wikidata sample (`wikidata-x4`, the gain family); the text corpus and random bytes (`ab gen` at scale 16) as loss and memory guards.
- *Ten rounds is a time cap.* An unresolved gain is NOT ADOPTED, with no extension after a near miss. At the fused-finish run's noise (paired log SD 0.020 to 0.030), ten rounds have 81–99% single-cell power for a true 0.970 at the first Holm threshold; at SD 0.05, 41% (the Researcher's derivation).

**The guards are bound to their registration** (correction 1):
- *Fresh inputs:* the recipe generates text and random afresh for every run (`ab gen` at scale 16 into a new directory) and never reuses earlier files. The text corpus is the checkout's own `src`, `tests`, `examples` and README, so it is tied to this commit.
- *The registered digests,* which the pre-flight checks:
  - `text.bz2` `088478a5c9980c3ad26706aa7a3e2d287148b3b0860bcd0e6f23fe28e9dc9262`;
  - `random.bz2` `f24831b171e595ef5e93d9f3f56fd267ec3122cfa482f55cf6c73547c0af20e1`.

**A pre-flight that VOIDs before any timing** (`bench/admit_preflight.py`, correction 2). The recipe stops and keeps the bundle unless all of these hold:
- *the machine:* `ab built mibz2,mibz2-wide 8,10` reports, in both arms at both counts, the measured i9 (`measured_i9`, read as `StreamDecoder::new` reads it) with this process's affinity every CPU, 0–19;
- *the plans built:* eight workers with 32 in flight at 8 threads and seven with 28 at 10, the same in both arms. The kernels are `CellsVariant<1>` (the lazy kernel) in `mibz2` and `CellsVariant<4>` (wide) in `mibz2-wide`. The reference's plan is read from this build's `StreamDecoder::new`; this branch changes no plan code, only adds two accessors;
- *the census on the exact inputs* (`ab census`, recorded):
  - the Wikidata sample: 196 blocks both admit, 32 the wide arm alone, 155 neither, 48 the column path, none the lazy kernel alone;
  - text and random: no block admitted by either;
  - no block's two outputs differing;
- *the guards' digests* above.

`bench/test_admit_preflight.py` checks the pre-flight: 13 checks in CI.

## The rule (`bench/two_arm.py`, as #506's, with `--unpinned` and two guards)

- **A cell's step:** the rounds' paired wall ratios, `mibz2-wide` over `mibz2@build-master`: geometric mean and 95% interval (Student's t on the logs). FASTER below 1, SLOWER above, else INCONCLUSIVE.
- **Gain:** Holm's one-sided tests over the two Wikidata cells, family α = 0.05; established if Holm rejects at one or more.
- **Loss screen:** no judged cell SLOWER, on Wikidata, text or random.
- **Memory gate:** at every judged cell on every corpus, the median over the rounds of each run's peak RSS, wide over master, at most 1.10.
- **Verdicts:**
  - *ADOPTS:* gain, no SLOWER cell, memory gate passed.
  - *HELD ON MEMORY:* gain and no SLOWER cell, memory gate failed; the Coordinator's under rule 4.
  - *A FINDING, NOT ADOPTED:* gain, but a cell SLOWER.
  - *NOT ADOPTED:* otherwise.
- **VOID** (exit 1, no verdict), before any verdict is computed:
  - *the records:* the file missing or a line not JSON; a key duplicated, outside the roster, or missing (120 at R = 10); a run not `success` or with an `err`; wall, CPU or peak RSS not finite and positive; an output's size or hash missing, or a corpus's outputs differing;
  - *the machine* (`ab env`): not 10 cores and 20 CPUs with each core's CPUs listed and no quota;
  - *a record's pinning:* any record not unpinned (`all`), since the run is registered unpinned;
  - *a spinner* recorded anywhere.
- **What an adoption means:** `LazyCells` takes wide admission, so the lazy kernel admits to r/n 0.15 with a budget of 0.32·n, wherever production runs lazy workers.
  - *What the i9 ADOPTS covers (correction 3):* aggregate performance on the measured workload (the Wikidata ×4 mix, with the two guards) and profile only. A default changed on every host goes through the CI performance policy and B5 review, or a forward C6 registration.
  - *What it does not establish:* no-SLOWER is no proof of no harm. The guards keep master's route and don't exercise the newly admitted blocks, so a mixed Wikidata gain could hide a regression in that stratum. A dense-block no-harm claim would need its own treatment-sensitive cell, loss margin and precision plan, frozen before data. None is made here.
  - *What stays as master's:* eager Cells (below `LAZY_FROM_THREADS`), the teams and the service core keep master's admission.

**The strata, reported** (correction 3): the census above is recorded with the run, giving the newly admitted blocks (32, 28.8 M rows of 209.7 M) as their own stratum. It also gives separately the 196 blocks both arms admit, where the larger budget still changes the scratch layout (the rules' and arena regions sized for 0.32·n cells, not 0.16·n), and the 155 neither admits.

`bench/test_two_arm.py` checks the reader: 61 checks in CI, including the unpinned registration, two guards, and the matched-round pair.

## Deciding nothing

**The knee, matched by round** (correction 5): on Wikidata ×4, each arm's wall at 8 threads over its wall at 10, matched by round with its 95% interval, and the wide arm's over master's (`bench/two_arm.py --pair 8,10`). A crossover can also come from slowing seven workers, so this is a separate question from the verdict.

**The memory controller,** with `kernel.perf_event_paranoid` 0 or less, else skipped and said so:
- *What:* `uncore_imc/data_reads/` and `data_writes/` system-wide, at 8 and 10 threads unpinned, on Wikidata ×4, 5 reps (`bench/counters.py --events imc --unpinned`).
- *The baseline:* each rep first counts the machine idle for a second, and the report takes the idle rate times each run's elapsed time off that run's bytes. It reports DRAM bytes read, written and in all per output byte, for each arm and as wide over master.

- *Every controller verified* (correction 6): an event counted on several memory controllers is summed, and one failed entry fails the sum; a later valid entry never overwrites it. Every run, idle ones included, must count each event on as many controllers. Otherwise the DRAM figures are unscorable, and the wall verdict stands.

`bench/test_counters.py` checks it with a stub perf: 13 checks in CI, including scaled MiB summed over two controllers, the idle rate taken off, an event not counted, a first controller's failed entry before a valid one, a controller missing from some runs, and an idle record missing.

## Predictions (M, frozen here; they don't enter the verdict, and get scored)

1. **DRAM at seven workers** (10 threads): 11.0 → **9.0 B per output byte [8.4, 9.9]**. Writes 3.6 [3.4, 3.9]; reads 5.4 [4.8, 6.1]. As wide over master: 0.81 [0.75, 0.90].
2. **DRAM at eight workers** (8 threads): wide over master 0.81 [0.75, 0.90], from the knee run's 12.4 B.
3. **Wall:**
   - Wikidata at 10 threads 0.97 [0.93, 1.01], at 8 threads 0.96 [0.92, 1.00];
   - text and random 1.00 [0.98, 1.02] at both, since their blocks keep master's route;
   - peak RSS 1.00 [0.95, 1.06].
4. **The knee:** within `mibz2-wide`, wall(8 threads, eight workers) over wall(10 threads, seven workers) on Wikidata ≤ 1.00 [0.96, 1.03]. In the knee run, eight workers lost to seven by 2.4%. This forecast is kept and scored as frozen. The Researcher notes that the knee run's 1.024 with these point forecasts (0.960 at 8, 0.970 at 10) implies 1.0134, not parity.

## Checks before the run

- **Tests:** `cells::wide_tests`, three of them:
  - the gates and budget at their lines;
  - 4,000 arbitrary columns across both lines: wherever the lazy kernel admits, the wide arm admits with the same bytes, and wherever the wide arm alone admits, forced Cells' bytes;
  - Cells' own admission (`attempt`, its budget by variant): between the lines the lazy kernel refuses for density and the wide arm doesn't; past 0.15 both refuse; below both lines, the wide arm admits whatever the lazy kernel admits.
- **The library's tests and the integration tests,** clippy (`--all-targets --all-features -D warnings`) and fmt, under 1.99.1.
- **End to end on a 4-core VM:** the pre-flight stops it before timing, on the machine and the plans alone (not the i9, 4 CPUs, ten workers at 10 threads), with the census and the guards' digests as registered.
- **On the workstation:** about 7 minutes. That is 120 judged runs (the Wikidata ×4 runs at about 0.4 s), 20 counter runs and 5 idle seconds, and two builds.
