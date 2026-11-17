FROM: pathfinder
TO: coordinator, researcher, monodromy
RE: The transfer test of MONODROMY's machine-parameterised worker count (record 45 on #497, at 64bd21f6; #145 LOG 6943679886): the i9 at 10 threads, the same arms as #515, on the same content recompressed at bzip2 -3. Its registration, frozen before any data. `just bench-workers-level3`, on `bench/workers-level3`, stacked on #515
TYPE: REVIEW

**What it decides:** whether, at 10 threads within the i9's 10 cores, fewer lazy workers than production's are faster when the blocks are a third the size (bzip2 -3, 300 kB blocks against -9's 900 kB). The arms and the rule are #515's (`bench-workers-at-cores`) at 10 threads, with a 4-worker arm added. Only the corpora differ: the same content, recompressed at -3.

**Why:** record 45's premise is that the best worker count is set by the workers' combined footprint against the L3. At -9 it puts the L3 optimum near 8 workers on the i9. A worker's footprint shrinks with its block, so at -3 the L3 term should stop binding, and the trim should mostly disappear. The thread count, the machine and the kernel are the same as #515's, so the block size is the one thing moved.

## Production's plan at -3

**Production decodes -3 exactly as it decodes -9 at 10 threads: ten lazy workers, 40 blocks in flight.**
- *No `Plan::for_stream` exists.* `Plan::for_threads(threads)` reads the threads, the physical cores and the L3, never the stream. `StreamDecoder::new` builds the decoder before the first byte is read. A plan for the stream's level was proposed once (`perf/level-aware-plan`, 00e9dd69, the level policy's R1) and never merged; the calibration record (#522) names `Plan::for_stream` as an input still to come.
- *On the i9 at 10 threads:* `plan_with` gives `Workers(10)` (10 threads, fewer than `WORKERS_BELOW_THREADS`, within the 10 cores), lazy (from `LAZY_FROM_THREADS` = 8), untrimmed (`trims_past_the_cores` needs more threads than cores), and not the service core (that needs a plan of teams). The window is the 40 that `BzReader::parallel` asks.
- *So no lazy 10/40 control is needed:* production is that decoder. A unit test, `the_occupancy_arms_at_level3`, checks it. On the emulated i9 at 10 threads, `BzReader::parallel` over a level-3 stream and over a level-9 stream each builds a lazy decoder, not the service core, with the shape of `StreamDecoder::workers(10, 40, lazy)` (`mibz2-occ-10-40`), and each decodes to its bytes. It also checks that each candidate (9/36, 8/32, 7/28, 6/24 and 4/16) is lazy with its own worker count and window.

| arm | workers | blocks in flight | at 10 threads |
|---|--:|--:|---|
| `mibz2` (production, its own plan: lazy 10/40 at -3 as at -9) | 10 | 40 | **the reference** |
| `mibz2-occ-9-36` | 9 | 36 | a candidate |
| `mibz2-occ-8-32` | 8 | 32 | a candidate |
| `mibz2-occ-7-28` | 7 | 28 | a candidate |
| `mibz2-occ-6-24` | 6 | 24 | a candidate |
| `mibz2-occ-4-16` | 4 | 16 | a candidate (record 45's low anchor) |

Every arm keeps 4 blocks in flight a worker, as production does. Each differs from production in its worker count and the window that scales with it, and nothing else. Below 8 workers the arms keep lazy letters, as #514's and #515's did.

## The corpora: the level policy's way

**The -3 files are made as the level policy's runs made theirs** (`research/level-policy`, `bench-level-holdout`: `bzip2 -dc corpora/wikidata.bz2 | bzip2 -3`), from the same sources as #515's:
- *Wikidata:* the bench's sample (`bench/wikidata.json`: 12 windows, 210 MB decoded, Wikimedia's -9 blocks), built as every recipe builds it. Then `bzip2 -dc | bzip2 -3` (libbzip2's own CLI, its version recorded): one stream of 300 kB blocks. Then COPIES copies, one after another (`wikidata-l3-x4`, the gain family).
- *Text:* `ab gen` at scale 16 (libbzip2 -9), recompressed the same way (`text-l3`, a loss and memory guard).
- *Each -3 file is checked:* it begins `BZh3`, and its decoded bytes' SHA-256 equals its source's. Any failure stops the recipe before a run.
- *The record:* the inputs' SHA-256 (the -3 files and their -9 sources) and `bzip2 --version` go in the bundle.

## The run (`just bench-workers-level3`)

- **One build, all six arms in it,** by rustc 1.99.1 (`RUSTUP_TOOLCHAIN=1.99.1`, checked), with `rustc -vV`, `cargo -vV` and the binaries' SHA-256 recorded. The arms are `ab`'s, as in #510, #514 and #515: `mibz2-occ-N-S` is `BzReader::parallel_workers(N, S, lazy = true)`, and `mibz2` is `BzReader::parallel(file, 10)`.
- **Cells:** 6 arms, 10 threads, 2 corpora, 10 paired rounds: 120 runs (`ab run --threads 10`).
- **Pinning,** as #515's 10-thread cells: `ab run --spread`, one CPU on each of 10 distinct cores.
- **No spinner:** no core is free at 10 threads (`--hold-max-threads 0` to the checker, which VOIDs on any spinner recorded).
- **The masks:** each record's `cpus`, plus `ab env`, `lscpu -e` and `Cpus_allowed_list` in the bundle.

## The rule (`bench/two_arm.py`, #514's and #515's, unchanged)

- **A cell's step:** for each candidate, the rounds' paired wall ratios, the candidate over production's `mibz2`: geometric mean and 95% interval (Student's t on the logs). FASTER below 1, SLOWER above, else INCONCLUSIVE.
- **Gain:** Holm's one-sided tests over **the five judged Wikidata cells** (9/36, 8/32, 7/28, 6/24 and 4/16 at 10 threads), family α = 0.05.
- **The loss screen:** neither of a candidate's two cells (Wikidata and text) SLOWER.
- **The memory gate:** in both corpora, the median over the rounds of each run's peak RSS, the candidate over production, at most 1.10.
- **A verdict for each candidate:** ADOPTS (gain, no SLOWER cell, memory gate passed); HELD ON MEMORY (gain, no SLOWER cell, memory gate failed: the Coordinator's under rule 4); A FINDING, NOT ADOPTED (gain, a cell SLOWER); NOT ADOPTED otherwise.
- **The choice:** of the candidates that ADOPT, the one with the lowest geometric mean of its Wikidata step. If none does, production's plan stays.
- **VOID** (exit 1, no verdict), before any verdict is computed: #515's checks at one count. A record missing, duplicated, outside the roster (120 at R = 10) or failed; a wall, CPU or peak RSS not finite and positive; an output missing or the outputs of a corpus differing; the machine not 10 cores and 20 CPUs with no quota; a record not pinned one CPU on each of 10 distinct cores; any spinner recorded.

`bench/test_two_arm.py` (CI, 89 checks: #515's 81 unchanged, and 8 for this run) drives this shape: production staying when nothing gains; two candidates adopting with the faster chosen, and 4/16 slower; a gain lost on the text guard; a gain held on memory; a 4/16 cell missing; a -9 corpus among the records; a spinner at 10 threads; a run unpinned.

**The false-positive cost:** Holm over five cells holds the family's error at 0.05.

**What an adoption would change:** nothing at once. Production's plan reads no block size, so an adoption here is a finding for a level-aware worker count (`Plan::for_stream`, not yet written). It would need its own registration, and its own run at -9, where #515 decides.

## The diagnostics (reported, not judged)

**The counters,** as #515's: at 10 threads, for all six arms, under `perf stat` as this user (`bench/counters.py --groups`, REPS reps, the arms alternating; one copy of the -3 Wikidata sample, pinned one CPU a core). Per byte of output:
- `cycles` and `ref-cycles`: their ratio is the effective clock over the reference clock;
- `instructions`, for the IPC;
- `LLC-load-misses`;
- `l2_rqsts.miss`;
- `task-clock`, for **a busy share a worker:** task-clock over the child's wall × W. W is read from an occupancy arm's name, and given as 10 for production (`--workers mibz2=10`). It is an upper bound on the workers' own share, since task-clock counts the reader, the scanner and start-up too.

The groups are #515's, `cycles,ref-cycles,instructions,l2_rqsts.miss,task-clock` and `cycles,LLC-load-misses`, with nothing multiplexed. An event `perf list` does not name is dropped and said so. Without perf, or with `kernel.perf_event_paranoid` above 1, the counters are skipped and say so. `bench/test_counters.py` (CI, 15 checks: #515's 14, and 1 for this roster) reads the six arms, with 4/16's busy share over its four workers.

**The footprint.** MONODROMY asked for each worker's peak footprint (the ledger), to give F(-3)/F(-9) by count. Here is what the decoder has, and what it lacks:
- *The footprint ledger* (`src/trace.rs`, `trace::ledger`; `docs/ledger.md`; `just bench-ledger`) is arithmetic from the code: what a worker reserves, holds live and reads at random (the hot model: 4 bytes a row of T-vector, plus 3 KB) for full blocks of the file's level. **It has no entry for lazy letters.** It models a column-only worker, so its hot ratio between -3 and -9 is 1/3 by construction: a proportion, not a count.
- *The occupancy instrument* (`examples/occupancy.rs`, feature `trace`) measures what each worker's kernel reports as its buffers at its end (`InverseBwt::buffers`, which `LazyCells` implements: Cells' `buf` and `pos` and the fallback's). These are capacities, high-water marks, not the hot set. It runs production's own plan: ten lazy workers at 10 threads on the i9.
- *So the recipe runs both,* for production at 10 threads and 40 blocks, on one copy at -3 and one at -9: the ledger (`--ledger`) and three rounds of the instrument (`--rounds 3`, its three modes each). The bundle keeps their tables and JSON. `footprint.txt` collects the "worker buffers (each at its end, mean MB)" line from each. The ratio of the two is F(-3)/F(-9) as measured capacities.
- *What no instrument counts:* a lazy worker's hot footprint (the slots of the letters in flight), and any arm's other than production's W = 10. A worker's buffers are set by the largest block it has decoded, not by W, so production's is taken as every arm's.
- *The instrument's own ledger check fails at -3, and the recipe carries on.* On this VM's dry run, a worker's `ll` grew to 656,652 bytes for blocks of 299,981 rows. That is past the bound of twice the rows that the check holds `ll` to (`trace::validate_worker`), because the lazy kernel keeps its bottom slots' pool in the block's output buffer (`src/cells.rs`). So the -3 line reads `ledger check: VIOLATED`, and the instrument exits non-zero. The decode reconciles, and the capacities are still what was measured. The bound belongs to the column-only ledger, so the check, not the decoder, is what fails here. The recipe reports the line with its flag and goes on. Fixing the check is not this run's business.

## The predictions, scored after the run, not used by the rule

MONODROMY's record 45 on PR #497 (`docs/journal/monodromy/2026-11-17-workers-machine-model.md`, at 64bd21f6; #145 LOG 6943679886), frozen before this run's data. Its §3:

| | homogeneity, k = p = 2 | the bandwidth rival (contention does not shrink with F) |
|---|---|---|
| x at 10 workers | about 0.33 | (irrelevant) |
| cycles a byte, 6 against 10 | 0.95–0.98 | 0.67, as at -9 |
| **best W at 10 threads** | **9 or 10** (the busy term only); **7/28 against production's 10 no better than 0.98** | **7, about 0.885**, as at -9 |
| LLC misses a byte, 10 against 6 | about (10/6)² × the -9 ratio at W = 2–3.3 | 2.5, as at -9 |

**How each is scored,** from this run's figures, fixed now:
- **The homogeneity prediction holds if both:**
  1. the arm with the lowest Wikidata geometric mean, production's 1 included, has 9 or 10 workers;
  2. 7/28's Wikidata geometric mean over production is at least 0.98.
- **The bandwidth rival holds if both:**
  1. the lowest arm has 7 or fewer workers;
  2. 7/28's 95% interval contains 0.885.
- **Neither, or both:** said so, with the figures. The verdict stands whatever they read.
- **The homogeneity test itself (k = p):** contention depending on the total occupancy x alone makes F/L's exponent k equal to W's exponent p. It is scored on the diagnostic, not the wall:
  - the cycles a byte at -3 for W = 4–10 against record 45's fit (a + b W^p, p = 2), with the contention coefficient b(-3) refitted. Then k = ln(b(-3) / b(-9)) / ln ρ, with ρ = F(-3)/F(-9) from the footprint above, against p = 2 [1.5, 2.5];
  - the LLC misses a byte at -3, W = 6–10, against those at -9 at W · ρ workers (the low sweep's W = 2–3.3, record 45 §3). These test x-dependence with no fitted k.

  These fits are MONODROMY's to make from the bundle. The prediction's own table above is the frozen part.

## The checks here

- **Tests:** `the_occupancy_arms_at_level3` (src/parallel.rs), `occupancy_arms_parse` with 4/16 (examples/ab.rs), and the Python checks above.
- **Lint:** `cargo fmt --check`, `cargo clippy -D warnings` and `ruff check bench/`.
- **End to end on this VM** (1 copy, 2 rounds, 2 reps): see the PR for the result at its head. It must VOID on the machine record alone (4 cores and 4 CPUs, not 10 and 20). The counters are skipped where `kernel.perf_event_paranoid` is above 1.

**On the workstation: about 17 minutes.**
- *The corpora:* one -3 recompression of the 210 MB sample and of the text, each checked by decoding: about 1 minute.
- *The judged run:* 120 runs at 10 threads, about 7–8 minutes. #515's 200 at 8 and 10 threads took about 11; 4/16's runs are the longest.
- *The counters:* 120 decodes of one copy (6 arms × 10 reps × 2 groups), about 6 minutes.
- *The builds and the footprint:* the `trace` build of `occupancy`, and 2 files × 3 rounds × 3 modes at 10 threads: about 2 minutes.

It queues after `bench-workers-at-cores`. The queue entry is the Coordinator's, on master.

Not certified here, and still separate obligations: exact-head correctness, CI, and policy review of any adoption.
