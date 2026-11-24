# bench-character, stage 1: the decoder's character across 1–20 workers (registration)

*Registered before any data. Brief: `docs/journal/2026-11-24-algorithm-level/briefs/character.md` at c57640c4 (§1–§4, §7a's operational laws, §8), on `docs/journal-algorithm-level`; assignment by the Coordinator, 24 November. Stage 1 of two: the baseline with the public API's sweep. Stage 2 (the interventions, the fractional factorial, the models) is a recipe of its own, after this one's data. A C1a diagnostic: its rule is posted on #145 before the data, and it decides nothing.*

## The question

What is the parallel decoder, at each count from 1 to 20 workers? Treat it as a network:
- **activities**, each block's work, routed;
- **buffers**, with admission's feedback through the ordered reader;
- **resources** the threads share.

At each count, stage 1 measures:
- where each thread's time goes;
- what each block demands of each activity;
- what each buffer holds;
- which limit binds, and where the limit moves.

It checks that the account is complete by the operational laws, not by a sum of coefficients. It gives k\*, which the gated parallel scan's registration uses (`briefs/scanner.md` part 2).

## The contract (brief §1)

- **Mechanism, the primary.** Production's lazy workers at exactly k (`BzReader::parallel_workers`, lazy: `OpenCells` with its admission and fallback; the window W = 4k; the ready bytes' cap 12 MiB, with its floor of k + 2). This is `mibz2-workers-lazy`'s decoder, not `mibz2-workers-open`.
- **The public API, secondary and unperturbed.** `BzReader::parallel(r, t)` as shipped, at t = 1…20 asked. Recorded per decode: what it built (threads, window, lazy, service).
- **The corpus.** 4 copies of `corpora/wikidata.bz2` (checked by bench/wikidata.json's SHA-256), in that order. Each decode's process reads the file into memory before the clock starts, and the decoder reads it from there: resident, with no file-system read in the clock. The output goes to the caller's thread in 4 MiB reads and is discarded (the decoder checks every block's CRC). The clock runs from the decoder's construction to its last byte read.
- **Lifetimes.** Each decode in a fresh process (bench-scan-profile's lesson: in one process, glibc's per-thread arenas keep the earlier decodes' heap). Each decode builds and drops its own decoder. An observation is 3 such decodes at one count, back to back, after 1 s of settle; its figures are their mean.
- **Build, kernels, allocator.** Master at 9621b7ad with the `character` research feature, rustc 1.99.1, release. Kernels by `Kernels::detect()`. The allocator is unchanged.
- **Frequency and power.** Observed, not held: the governor, the driver, EPP, `no_turbo` and transparent huge pages are recorded.
- **Placement.** Unpinned on all 20 logical CPUs. Pinned runs are stage 2's targeted checks.
- **Topology.** Recorded once (`lscpu`, `lscpu -e`). NUMA is dropped.

## The instrument (src/character.rs; examples/character.rs)

**The hooks, and their cost when off.** The feature's hooks are in src/parallel.rs, read.rs, block.rs, lib.rs and open.rs. Off, each hook is one relaxed atomic load. On, each thread keeps its own ledger, with no lock shared between threads while decoding. The ledger has three kinds of event:
- **an activity's begin and end**, nested as a stack, so each thread's timeline is partitioned. At every instant a thread is in the activity on top of its stack, or in its residual;
- **a block's points**, each with its place: issued, enqueued, taken by a worker, routed, ready (with its bytes), taken by the reader. Also the scanner's waits, marked by what binds: the window, or the ready bytes' cap;
- **the clocks.** Each event has its time. Each thread's CPU time is read at its waits' begins and ends and at its start and end. A CPU-time read at every activity is a system call, too dear for the scanner, about 1.6% of its time. So each stretch between waits has its wall and CPU time. At a thread's start and end, its scheduler counters (`schedstat`) and its migrations are read.

**The activities** (brief §2's tags):

| tag | where | from … to |
|---|---|---|
| A0 | feeder | a chunk's buffer taken, the input copied into it (its wait to hand it on apart, `feed_wait`) |
| A1 | scanner | its waits for input (`in_wait`), and the window's maintenance: swap, compaction, append and return (`in_maint`) |
| A2 | scanner | the item's magic, header, bitmap, selectors, code lengths and tables (`block::read_item_with_scratch`, to the tables' end) |
| A3 | scanner | the symbols' buffer prepared, and the Huffman decode |
| A4 | scanner and workers | admission wait (`slot_wait`, window or cap); issue and send (`issue`); the output buffer from the pool (`pool`); publish (`publish`) |
| A5 | worker | MTF/RLE2 and the run records (`runs::takes_runs` + `runs::front`), or the column route's MTF |
| A6a | worker | Open's first pass to its admission (`construct_capped`, the out buffer's sizing included) |
| A6b | worker | Open's set-up, induction and dictionary (`finish`, to the wiring) |
| A6c | worker | Open's retained-edge wiring (XOR or plain) |
| A7 | worker | Open's emission (`walk`) |
| A8 | worker | inversion other than Open: on the run route, the fallback (LazyCells' gate, then Segmented's build and walk); or the column route's inversion |
| A9 | worker | RLE1 and the CRC (`rle::unrle_in_place`, tiled, whole; or the column route's `finish`) |
| A10 | reader | each read call (`read_wait` within it: waiting for the next block in order) |

**Each block's route** is recorded once:
- Open admitted outright (1/64 ≤ r/n < 0.15);
- Open admitted under its dense cap;
- Open refused after its first pass;
- Open skipped;
- the column route.

**The buffers' live occupancy** is kept apart from the ledgers:
- admitted, issued − taken;
- queued;
- executing;
- ready, in blocks and in bytes.

Each change is timed under the buffers' own lock, with one clock reading shared with its block's point. That clock reading is what makes Little's law exact below. For each buffer: its integral, its high-water mark and its arrivals.

**The harness** reads every thread's ledger after the decode, in-process. It writes:
- one JSON line a decode: the process figures, each thread's states (per phase), CPU, waits, scheduler and migrations, the occupancy, the routes and the identities;
- every block's row: its route, n, r, its bytes, its timestamps and its demand at each activity (scanner and worker), and its reader wait.

**The phases** (brief §1: steady, start-up and drain apart):
- start-up, to the first block taken by the reader;
- steady, to the last block issued;
- drain, to the end.

## Completeness: the operational laws (brief §4, §7a)

Every decode with the ledgers on checks each of these:
1. **threads:** k workers, 1 scanner, 1 feeder and 1 reader, each with its ledger.
2. **timelines partitioned:** each thread's events in time order, its activities nested (each end closes the activity on top), the stack empty at the end, and its states summing to its life.
3. **flow:** every block issued, enqueued, taken by a worker, routed, ready and taken by the reader exactly once.
4. **forced flow:** each activity's visits equal its routes' blocks:
   - A5, pool, A9, publish and issue: every block;
   - A6a: Open's attempts;
   - A6b, A6c and A7: Open's admissions;
   - A8: the refused, skipped and column blocks.
5. **conservation:** every buffer empty at the end, with its arrivals the blocks (enqueues − dequeues = occupancy).
6. **admission:** 0 ≤ issued − taken ≤ W at every instant, from the ledger and from the live high-water mark.
7. **Little's law:** each buffer's live occupancy integral equals its blocks' summed residence, exactly. Each change and its point share a clock reading, so the two are the same sum of integers unless an event is missing or doubled. The preflight decodes held exactly, a discrepancy of 0 in every buffer over 36 decodes at 3, 10 and 20 workers.
8. **utilisation:** every worker activity is charged to a block. A worker's time outside a block is its wait for a job alone.

The report also gives the utilisation and bottleneck laws from the medians:
- the throughput X;
- each stage's demand per block, D;
- U = X·D;
- the bound 1/max D.

The control-coefficient sum is stage 2's, and only as a conditional diagnostic.

## The run

- **The grid.** Every count from 1 to 20, in **12 randomised complete rounds**. Each round is every count once, in an order drawn from seed 20261124 (`random.Random(seed·1000 + r)`), never a rising sweep. An observation is 3 decodes with the ledgers on: 240 observations, 720 decodes. Every block's row is kept for the first decode of each observation in rounds 0–3 (80 observations × 1,724 blocks), the rest as totals.
- **The ledgers' cost** (brief §3), interleaved so as to share the drift. After every second round comes one pair at 1, 10 and 20 workers, in a drawn order:
  - 3 decodes with the ledgers off and 3 on;
  - the arm first alternating by pair;
  - 6 pairs in all.
- **The counters** (brief §3), with the ledgers off. At the anchors 1, 6, 8, 10, 12 and 20, in a drawn order:
  - **group 1:** cycles, instructions, ref-cycles, branches and branch misses;
  - **group 2:** LLC load misses, `l2_rqsts.miss`, dTLB load misses and `cycle_activity.stalls_mem_any` (Comet Lake's raw codes).

  Each group runs 3 times at each anchor: 36 observations. The counters are opened in-process before the decoder, inherited by its threads, enabled for the decode alone and read once its threads are joined, with each counter's time enabled and running (its coverage). In the preflight, the software group's inherited counts agreed with `getrusage` over the same decode: task clock 0.508 s against 0.509 s, and 576 context switches against 576. Where no hardware PMU counts, the runs are skipped and that is recorded.
- **The memory controller,** where `perf stat -a` can count `uncore_imc/data_reads/` and `data_writes/` (a `perf_event_paranoid` of 0 or less, or root), at each anchor 3 times, around the whole decode's process. It counts system-wide, so the input's read is included; as uncore bytes they are not attributable per stage. Skipped and recorded where it cannot count; no sudo is asked.
- **The public API:** 3 rounds, each asking for every count from 1 to 20 once in a drawn order, one decode each.
- **The structural features,** untimed:
  - the census (`examples/census.rs`, in a build of its own) of the open kernel's blocks on one copy;
  - the harness's `--features` pass: each block's route, n, r, bytes and h₁ (the first-order entropy of its decoded bytes).

  The four copies are the same 431 blocks, so the report joins the features to the block rows by place mod 431. The census is kept beside them for the math-targets join: its rows are the Open blocks in place order, 233 distinct.
- **Every decode's process figures:**
  - wall, bytes, user and system CPU;
  - minor and major faults, voluntary and involuntary switches;
  - the base, peak and time-weighted mean resident memory (and its kB-seconds);
  - with the ledgers on, each thread's migrations.
- **The time:** about 20 minutes on the i9 with the build:
  - the grid: about 4.5 minutes of decoding, 4 of settle and 2 of process starts;
  - the cost check: about 2 minutes;
  - the counters and the memory controller: about 2;
  - the public API: about 1.5.

## The report and the TSVs

`bench/character.py` reports, at each count (the rounds' medians of each observation's mean):
- **the process:** wall (with the 95% half-width), throughput, CPU, the decode's own peak and mean memory, faults, switches and migrations;
- **where the time goes:**
  - the scanner's busy time, A2, A3, maintenance, issue, input wait, and admission wait by what binds (window or cap);
  - the mean worker's busy time, its starvation, and each activity A5–A9, pool and publish;
  - the reader's wait, and the feeder's A0;
- **the phases:** start-up, steady and drain, and the steady state's shares:
  - the scanner's utilisation and admission-wait share;
  - the mean worker's utilisation and starvation share;
- **the buffers:** each one's time-averaged occupancy and high-water mark;
- **the laws:** each identity's count of decodes holding and failing; the utilisation and bottleneck table;
- **k\* and the near-optimal set,** and each count's limit with the limit's moves;
- **the ledgers' cost,** the counters, the memory controller and the public API;
- **each route's blocks:** their count, n, r/n and h₁, and their demand per row at each activity at 1, 10 and 20 workers.

It writes into the bundle's `tsv/`, for the journal's math-targets project (`math-targets/data/`, with their script, once read):
- `observations.tsv`: one row a decode;
- `threads.tsv`: one row a thread a grid decode, its states per phase;
- `per_k.tsv`, `cost.tsv`, `pmu.tsv`, `imc.tsv` where counted, and `api.tsv`.

Beside them in the bundle: the block rows (`blocks-r00.tsv` … `blocks-r03.tsv`), `features.tsv` and `census.tsv`.

## THE C1a RULE (it decides nothing)

**k\*** = min{k : the median over rounds of Δ_k > 0}, where:
- Δ_k = the scanner's busy time less the mean worker's busy time, each observation's mean over its decodes;
- the scanner's busy time is its life less its waits;
- a worker's busy time is its life less its wait for jobs.

The interval: 2,000 resamples of the 12 rounds with replacement (seed 20261124), k\* recomputed in each; the 2.5% and 97.5% quantiles, and how many resamples have none. Where no count's median is positive: none in 1..20.

**The near-optimal set:** the counts whose median wall is within 2% of the best median wall.

**Each count's limit**, on the steady phase's medians:
- **SCANNER-LIMITED** iff the scanner's utilisation is ≥ 0.85 and its admission-wait share ≤ 0.05;
- else **WORKER-LIMITED** iff the mean worker's utilisation is ≥ 0.85 and its starvation share ≤ 0.05;
- else **ADMISSION-LIMITED** iff the scanner's admission-wait share is > 0.10 and the mean worker's utilisation < 0.85: the scanner held by admission while the workers are not saturated (brief §2's feedback through the ordered reader);
- else **MIXED**.

The report gives the limits' moves across k.

**The labels:**
- **INCOMPLETE** (reported, labelled) if any grid decode fails an identity;
- **PERTURBED** (reported, labelled) at a count among 1, 10 and 20 where the ledgers' cost passes the bound. The bound is passed when the geometric mean of the pairs' on/off wall ratios exceeds 1.01, or when its 95% interval lies wholly above 1.005 (the brief's "about 0.5–1%"). If so, the tracing is to be reduced before stage 2.

**VOID** (no report but its reasons) if any of these:
- the machine is not the i9-10900K;
- a registered decode is missing, duplicated or failed;
- an output is not 4 × 209,700,196 bytes;
- a grid window is not 4k, or a grid decode ran with the ledgers off;
- a round's order is not a permutation;
- a decode lacks its memory or scheduling figures.

## What stage 1 leaves to stage 2

- **The pacing.** Stage 2's budgets are fixed fractions of each activity's unperturbed per-block time measured here, never recomputed from slowed runs.
- **The models' inputs and checks:**
  - the per-block service vectors in original order (the block rows) drive the discrete-event simulation;
  - the per-count demands drive the mean-value analysis;
  - the identities above are the models' checks.
- **The rest:**
  - the fit and held-out counts (brief §7: fit {1, 2, 4, 6, 8, 10, 12, 16, 20}, held out the rest);
  - the 2^(9−4) fractional factorial at 8 and 10;
  - the targeted checks.

## Checks

- **`tests/character.rs`** (with the feature) covers the crate's test corpora at levels 1 and 9, and three streams end to end, at 1, 3 and 8 workers:
  - the bytes are unchanged;
  - every thread's ledger starts, ends, keeps time order and nests;
  - every block's points come once each;
  - every buffer is empty at the end, its arrivals are the blocks, and Little's law holds exactly;
  - with the ledgers off, nothing is recorded.
- **`bench/test_character.py`:** 25 checks of the report's rules on synthetic records: k\* and its absence; each limit; the cost bound; INCOMPLETE; each VOID.
- **CI** runs both, then the harness and its driver on the repository's own text (15 blocks, the refused and column routes among them). Every decode's identities must hold in the harness's own check.
- **A container smoke run** (4 CPUs, 1 copy, 4 rounds of 2; not evidence for the i9) held every identity in all 160 decodes but one. That one was Little's law before its point and its change shared a clock reading, and it is what led to the shared reading.
