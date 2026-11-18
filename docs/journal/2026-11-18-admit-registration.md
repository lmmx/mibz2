# 2026-11-18 · Pathfinder · Wider admission for the lazy kernel: its i9 adoption registration, frozen before data

Direction: the Coordinator's QUEUED 6973253342 on #145, and its reply to the Pathfinder's REVIEW 6973651863 ("build `bench-admit`").
- **The question:** can Cells stop writing its intermediate pre-RLE output? The answer was no, about 0.15 B of the 11.0 B of DRAM traffic per output byte.
- **What the census found instead:** the blocks Cells refuses carry about a third of the decoder's DRAM traffic.

This registers the run that tests admitting them. It is frozen here, before any data, for the Researcher's one review.

## What it decides

Whether production's lazy kernel admits blocks up to r/n 0.15 with a budget of 0.32·n (`bwt::WideLazyCells`), in place of r/n 0.10 and 0.16·n.
- *What stays the same:* the induction and the writer are master's own code. The sparse gate, the plan, the window and the threads are master's.
- *What adoption would change:* only which blocks the lazy kernel admits.

## Why: the census

**The method:** cachegrind on `ab child … 1`, rustc 1.99.1, with the last level set to 3 MiB, a worker's share of the i9's 20 MiB L3 at seven workers.
- *What it counts:* a write-miss is a later write-back and costs an RFO read first, so DRAM ≈ 2 × write-misses + read-misses.
- *The model against the i9's memory controller* (`knee/run-20261117T230729Z`, seven workers): 11.2 B per output byte against the measured 11.0 (writes 4.37 against 4.0, reads 6.82 against 7.0).

**Where the refused traffic is:**
- *The T-vector walk's reads:* 1.40 B per output byte, for the 15.9% of Wikidata's bytes that Cells refuses. That is about a third of the decoder's DRAM traffic.
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
  - *What the i9 ADOPTS covers:* the measured profile only. A default changed on every host goes through the CI performance policy and B5 review, or a forward C6 registration.
  - *What stays as master's:* eager Cells (below `LAZY_FROM_THREADS`), the teams and the service core keep master's admission.

`bench/test_two_arm.py` checks the reader: 60 checks in CI, including the unpinned registration and two guards.

## Deciding nothing

**The memory controller,** with `kernel.perf_event_paranoid` 0 or less, else skipped and said so:
- *What:* `uncore_imc/data_reads/` and `data_writes/` system-wide, at 8 and 10 threads unpinned, on Wikidata ×4, 5 reps (`bench/counters.py --events imc --unpinned`).
- *The baseline:* each rep first counts the machine idle for a second, and the report takes the idle rate times each run's elapsed time off that run's bytes. It reports DRAM bytes read, written and in all per output byte, for each arm and as wide over master.

`bench/test_counters.py` checks it with a stub perf: 11 checks in CI, including scaled MiB summed over two controllers, the idle rate taken off, an event not counted, and an idle record missing.

## Predictions (M, frozen here; they don't enter the verdict, and get scored)

1. **DRAM at seven workers** (10 threads): 11.0 → **9.0 B per output byte [8.4, 9.9]**. Writes 3.6 [3.4, 3.9]; reads 5.4 [4.8, 6.1]. As wide over master: 0.81 [0.75, 0.90].
2. **DRAM at eight workers** (8 threads): wide over master 0.81 [0.75, 0.90], from the knee run's 12.4 B.
3. **Wall:**
   - Wikidata at 10 threads 0.97 [0.93, 1.01], at 8 threads 0.96 [0.92, 1.00];
   - text and random 1.00 [0.98, 1.02] at both, since their blocks keep master's route;
   - peak RSS 1.00 [0.95, 1.06].
4. **The knee:** within `mibz2-wide`, wall(8 threads, eight workers) over wall(10 threads, seven workers) on Wikidata ≤ 1.00 [0.96, 1.03]. In the knee run, eight workers lost to seven by 2.4%.

## Checks before the run

- **Tests:** `cells::wide_tests`, three of them:
  - the gates and budget at their lines;
  - 4,000 arbitrary columns across both lines: wherever the lazy kernel admits, the wide arm admits with the same bytes, and wherever the wide arm alone admits, forced Cells' bytes;
  - Cells' own admission (`attempt`, its budget by variant): between the lines the lazy kernel refuses for density and the wide arm doesn't; past 0.15 both refuse; below both lines, the wide arm admits whatever the lazy kernel admits.
- **The library's tests and the integration tests,** clippy (`--all-targets --all-features -D warnings`) and fmt, under 1.99.1.
- **End to end on a 4-core VM:** VOID on the machine record, as it must be off the i9, with the DRAM diagnostic skipped there and saying so.
- **On the workstation:** about 7 minutes. That is 120 judged runs (the Wikidata ×4 runs at about 0.4 s), 20 counter runs and 5 idle seconds, and two builds.
