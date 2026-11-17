FROM: pathfinder
TO: coordinator, researcher
RE: After #510 (8/32 over production's 9/36 past the cores; LOG 6931719814), #514 (7/28 over 8/32; LOG 6942020517) and #515 (7/28 at 10 threads and 6/24 at 8 over production within the cores; LOG 6943194948): the plan's choices confirmed directly against production, on held-out corpora. Its registration, frozen before any data. `just bench-workers-confirm`, on `bench/workers-confirm`, stacked on #515 (which carries #514)
TYPE: REVIEW

**What it decides:** whether the worker counts the last three runs chose beat production when each meets production directly, and whether they hold on corpora that took no part in choosing them. It confirms three choices, each against production's own plan at its count:

| group | candidate | judged at | the choice it confirms | prior estimate over production |
|---|---|---|---|---|
| past the cores | `mibz2-occ-7-28` (7 lazy workers, 28 in flight) | 12, 16, 20 threads, unpinned | #514's, after #510: a chain of two runs, 9 → 8 → 7 | **0.92**, the chain's product (LOG 6942020517: 0.918) |
| at 10 threads | `mibz2-occ-7-28` | 10, pinned one CPU a core | **#515's choice at 10** (LOG 6943194948) | **0.885** [0.882, 0.889], #515's measured step |
| at 8 threads | `mibz2-occ-6-24` (6 lazy workers, 24 in flight) | 8, pinned, a spinner on a free core | **#515's choice at 8** (LOG 6943194948) | **0.929** [0.922, 0.935], #515's measured step |

The reference at every count is production's `mibz2`: nine lazy workers with 36 in flight past the cores, ten at 10 threads, eight at 8, as #510, #514 and #515 ran it.

**What is new here:**
- *Past the cores,* 7/28 has never met production directly. Its case rests on the product of two steps, each measured against 8/32. This run measures the end-to-end step.
- *At 10 and 8 threads,* the cells repeat #515's comparisons on #515's corpora, in an independent run. **These cells are confirmations of #515's choices on held-out corpora.** 7 and 6 were chosen on Wikidata, with text as the only guard. Here both also face two corpora that took no part in any choice.

## The run (`just bench-workers-confirm`)

- **One build, all three arms in it**, by rustc 1.99.1 (`RUSTUP_TOOLCHAIN=1.99.1`, the repository's pin, checked), with `rustc -vV`, `cargo -vV` and the binary's SHA-256 recorded. The arms are #514's and #515's: `ab`'s `mibz2-occ-N-S` is `BzReader::parallel_workers(N, S, lazy = true)`, lazy letters held below 8 workers (`the_occupancy_arms` unchanged).
- **Each candidate runs only where it is judged.** Two `ab run`s, production in both:
  - `mibz2` and `mibz2-occ-7-28` at 10, 12, 16 and 20 threads;
  - `mibz2` and `mibz2-occ-6-24` at 8 threads.

  Their records are joined into one `judged.jsonl`, read whole. Every ratio pairs the two arms' runs in the same round of the same `ab run`. Running 6/24 at 10 to 20, or 7/28 at 8, would cost about 40% more runs, for cells nobody would judge.
- **The pinning and the spinner, as #514 and #515 had them:**
  - at 8 and 10 threads, `--spread`, one CPU on each of that many cores;
  - past the cores, unpinned;
  - a spinner on a free core at 8 threads alone (`--hold-cpu auto --hold-max-threads 9`). `HOLD_CPU=off` runs none, and the checker is told so.
- **Cells:** **10 paired rounds**, as #514 and #515. 12 were considered and dropped to keep the run inside the budget (below). No controls: the arms differ at every judged count.
- **The masks:** each record's `cpus` and `held_spinner`, plus `ab env`, `lscpu -e` and `Cpus_allowed_list` in the bundle.

### The corpora

| corpus | what it is | role |
|---|---|---|
| `wikidata-x4` | 4 copies of the Wikidata sample (`bench/wikidata.json`, Wikimedia's own -9 blocks) | **the gain family**, as #510, #514 and #515 |
| `text` | `ab gen` at scale 16 (the crate's source text, repeated; libbzip2 -9) | a loss and memory guard, as before |
| `records` | `ab gen` at scale 16: 9.6M fixed-width 32-bit binary records, low-run and near-incompressible (38.4 MB → 38.3 MB at -9) | **held out**: a loss and memory guard |
| `text-l1` | `text` decoded and recompressed at **-1** by Python's `bz2` (libbzip2): 100k blocks, 480 of them | **held out**: a loss and memory guard |

**Why these two held-out corpora.** Neither took part in choosing 7 or 6. Both are made on the workstation from what the recipe already has, with no download: `records` comes out of the same `ab gen` call that makes `text`, and `text-l1` takes a few seconds of Python from it. The inputs' SHA-256 are recorded. Each differs from the gain family in one of the two ways that could reverse a choice:
- *`records`, the content:* binary, low-run and high-entropy. It gives about 1 byte of output per compressed byte, against about 18 for Wikidata JSON. The decode's balance moves from the inverse BWT's output walk towards the Huffman and MTF front end, which fewer workers could starve.
- *`text-l1`, the block size:* #514's and #515's counters put the gain on a smooth L3 capacity effect: fewer workers, fewer live T-vectors sharing the 20 MB L3. At -1 a block's T-vector is a ninth of a -9 block's (400 KB, not 3.6 MB), so production's workers' hot set already fits the cache. If the capacity reading is right, fewer workers have least to gain there and most to lose in parallelism. That is the cell most likely to find a loss.

Considered and not chosen:
- *The Wikidata sample at -1* (`bench/blocks.py`'s level sweep, #140). It isolates the block size on the gain family's own content, but it is not a different kind of input, and it needs another lbzip2 pass over 840 MB.
- *`ab gen`'s `random`, `onebyte`, `pattern` and `runs`.* These are degenerate, or near-empty decodes (`onebyte` is 81 bytes compressed), where the wall is the harness's.

## The rule (`bench/two_arm.py --confirm`, #514's and #515's, production the reference)

- **A cell's step:** the rounds' paired wall ratios, the candidate over `mibz2` at that count: geometric mean and 95% interval (Student's t on the logs). FASTER below 1, SLOWER above, else INCONCLUSIVE.
- **Gain:** Holm's one-sided tests over **the five Wikidata cells of the three groups together** (7/28 at 12, 16, 20 and at 10; 6/24 at 8), family α = 0.05.
- **Loss screen, each group:** no judged cell of it SLOWER on any of the four corpora. It finds only the losses its intervals resolve.
- **Memory gate, each group:** at every judged cell of it, all four corpora, the median over the rounds of each run's peak RSS, the candidate over `mibz2`, at most 1.10.
- **A verdict for each group:**
  - **CONFIRMS:** Holm rejects at **every** count of the group (all three past the cores; the one cell at 10; the one cell at 8), no cell of it SLOWER on any corpus, and the memory gate passed.
  - *CONFIRMS IN PART* (past the cores alone): a gain at one or two of 12, 16 and 20, not all three; no SLOWER cell; the memory gate passed.
  - *HELD ON MEMORY, NOT CONFIRMED:* a gain and no SLOWER cell; the memory gate failed.
  - *A FINDING, NOT CONFIRMED:* a gain, and a cell SLOWER on some corpus.
  - *NOT CONFIRMED:* no gain.

  The groups are read apart: a loss at 8 leaves the verdict at 10, and past the cores, as they are.
- **VOID** (exit 1, no verdict), before any verdict is computed:
  - *the records:* the file missing or a line not JSON; a key duplicated, outside the roster, or missing; a run not `success` or with an `err`; wall, CPU or peak RSS not finite and positive; an output's size or hash missing, or a corpus's outputs differing. The roster is 400 at R = 10: production at 5 counts, 7/28 at 4 and 6/24 at 1, each on 4 corpora. A candidate's record at a count where it is not judged is outside it.
  - *the machine* (`ab env`): not 10 cores and 20 CPUs with each core's CPUs listed and no quota;
  - *a record's pinning:* at 8 and 10, not one CPU on each of that many cores; past the cores, not unpinned;
  - *the spinner:* at 8, missing, malformed, or on a core of the decode's; at 10 and past, recorded at all.
  - *the groups:* a judged count in no group, a count in two groups of one arm, a group's arm not a candidate, a guard that is the main corpus or named twice, a control count, a prediction that names no group or is not a positive step.

**The pre-registered consistency checks (reported, never a veto):** each group's prior estimate should lie within each of its Wikidata cells' 95% intervals:
- past the cores, the chained **0.92** (the Coordinator's requirement), at 12, 16 and 20;
- at 10, #515's **0.885**;
- at 8, #515's **0.929**.

The verdict prints, for each, inside or OUTSIDE at each count. Outside changes no verdict and no exit code. Past the cores it would mean the two steps do not compose: 9 → 7 is not the product of 9 → 8 and 8 → 7 on this machine. At 10 or 8 it would mean the independent run did not reproduce #515's step.

**The false-positive cost, stated:** Holm over five cells holds the family's error at 0.05. CONFIRMS past the cores asks for a rejection at all three counts, more than an adoption does.

**The changes to `bench/two_arm.py`** leave every earlier reading as it was:
- `--guard` may be repeated. Every guard enters the loss screen and the memory gate, none the Holm family.
- `--confirm` reads the groups and verdicts above. Without `--judge-at` it reads one candidate at every judged count as one group. With it, each `--judge-at` is a group, and an arm may head several.
- `--predict STEP`, or `ARM=COUNTS=STEP` naming a group, prints the consistency lines.

`bench/test_two_arm.py` (CI, 111 checks: #515's 81 unchanged, and 30 for this run) covers:
- *the one-group reading,* each verdict: CONFIRMS with the prediction inside, and with it outside at one count; CONFIRMS IN PART; a held-out cell SLOWER, `records` and `text-l1` each; a held-out corpus over the memory gate; no gain;
- *the grouped reading, this run's shape:* all three groups CONFIRM with every prediction inside; a held-out loss at 8 leaves 10 and past the cores confirmed; no gain at 10 with its prediction OUTSIDE, and the -1 file over the memory gate past the cores;
- *its VOIDs:* a candidate run where it is not judged; a judged cell missing; no spinner at 8; unpinned at 10; a count in two groups; a judged count in no group; a prediction naming no group, a bare prediction with groups, a group predicted twice; a held-out corpus's records missing, or run but not named; a guard that is the main corpus, or named twice; a control count; `--predict` without `--confirm`.

Several guards without `--confirm` give the adoption verdicts as before.

**End to end on this VM** (1 copy, 2 rounds): every stage ran: the build, the inputs, both `ab run`s (80 runs), the joined records, the summary and the bundle. Every corpus's outputs were byte-identical between the arms. The verdict VOIDs as it must, on the machine record, and on the spinner at 8 threads: on 4 CPUs, 8 threads leave no core free.

```
records 80; roster 80 (mibz2 and mibz2-occ-7-28 and mibz2-occ-6-24; wikidata-x1 and text and records and text-l1; judged [8, 10, 12, 16, 20], controls []; 2 rounds)
VOID: the records do not make the registered run (17 problems); no verdict
  env.json: not the registered profile: cores 4, not 10; cpus 4, not 20; core_cpus ['0', '1', '2', '3']: not 10 cores
  judged.jsonl:65: ('wikidata-x1', 'mibz2', 8, 0) ran beside a spinner sharing a decoder core
  ...
```

7/28's records read with the VM's profile (`--cores 4 --cpus 4`) go through every line to the two groups' verdicts and consistency lines. The VM's figures decide nothing.

**On the workstation:** about 21 minutes, inside the Coordinator's 25.
- *The judged run:* 400 runs, each behind `ab run`'s 2-second settle.
  - What it is calibrated on: #514's 240 runs at 12 to 20 threads took about 11 minutes (about 2.75 s a run, half of them Wikidata ×4). #515's 200 at 8 and 10 took about 11 (about 3.3 s a run).
  - Per round, about 112 s: Wikidata ×4 at about 3 s a run past the cores and about 3.8 s within them, and the small corpora (`text`, `records`, `text-l1`) at about 2.6 s each. So about 19 minutes.
- *The inputs and the build:* one to two minutes. `ab gen` at scale 16 if its files are absent (about half a minute of libbzip2 -9), `text-l1` (seconds), and the build (`ab` unchanged since #515, so incremental).
- *12 rounds* would take about 24 minutes, too close to the budget. 10 is #514's and #515's count.

It queues after bench-workers-at-cores.

Not certified here, and still separate obligations: exact-head correctness, CI, and policy review of any adoption.
