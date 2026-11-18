# The moonshot scorer

One command that checks a candidate inverse-BWT kernel against mibz2 0.5.6's, for a research
agent working in its own container:

```sh
bash bench/moonshot/corpus.sh                 # once: corpora/wikidata.bz2, the bench corpus
just moonshot-score path/to/candidate.rs      # or: python3 bench/moonshot/moonshot.py score ...
```

The objective it serves: **1.25× the sustained throughput of 0.5.6 on Wikidata, at 7–10
workers on an i9-10900K, with no extra peak memory.** A container can't measure that. The
scorer gives three kinds of evidence, side by side and **never combined into one number**:

1. **Correctness, a hard gate.** Any mismatch fails the score, and nothing is timed.
2. **Container wall time.** The candidate against 0.5.6's kernel, paired, at every worker count
   the container has, with peak RSS.
3. **A contention proxy.** A cache and dependency model of one worker, at the i9's worker counts,
   with three separate quantities per output byte (transfers, dependent-miss chains, overlap),
   calibrated against i9 records and checked on held-out arms.

**Only CI's paired bench and an i9 workstation run establish production performance.**
- *CI's bench:* a PR that touches `src/` gets CI's paired bench (`.github/workflows/bench.yml`)
  automatically. To get it, port the candidate into the crate behind a research feature,
  as `open-return` and `memory-lever` were.
- *The i9:* the Coordinator queues i9 runs (`bench/workstation-queue.txt`, CLAUDE.md
  "Workstation recipes"). Ask on #145 with the PR and the comparison wanted.

A container's wall time can mislead. The open-return kernel `opend` was 9–21% slower on a
4-CPU cloud VM, yet 6–8% faster on the i9 at 8–10 workers (MONODROMY record 71).

## Plugging in a candidate

A candidate is a Rust source file that defines `Candidate`, a type implementing
`mibz2::bwt::InverseBwt`: the shipped workers' block-decode entry. It returns a block's
RLE1-coded text from its last column, or from its capped run records, exactly as LazyCells does.
Copy [`candidates/template.rs`](candidates/template.rs) and run:

```sh
just moonshot-score path/to/my_kernel.rs
```

- **How it is built.** `build.rs` compiles the file as the module `crate::candidate` of
  `bench/moonshot` (the file is named by `MOONSHOT_CANDIDATE`, which the recipe sets).
- **Code from inside mibz2 compiles unchanged.** `crate::bwt`, `crate::runs` and
  `crate::Kernels` are mibz2's. A sibling file can be a submodule (`#[path = "x.rs"] mod x;`).
- **The run records.** They are `r.records()` (capped runs, `byte << 8 | length`, in L order).
  This is public only under the crate's research feature `moonshot`, which this harness turns
  on. Without the feature, `src/` compiles exactly as shipped.
- **How it is timed.** It runs in the very workers 0.5.6 runs:
  `BzReader::parallel_workers_with::<Candidate>(r, W, 4W, k)`. The baseline is the same call
  with `LazyCells`. So the two arms differ in the kernel alone, as record 71 §4.1 asks.
- **Worked examples.** [`candidates/lazy_cells.rs`](candidates/lazy_cells.rs) is the null
  candidate, 0.5.6's own kernel: it must pass and time at 1.0 within its interval.
  [`candidates/opend.rs`](candidates/opend.rs) is opend, the open-return decoder compiled from
  its journal behind LazyCells' density line.

The harness's reference arms ([`src/arms.rs`](src/arms.rs)) are:
- `lazy`: LazyCells, 0.5.6's kernel;
- `cells`: eager Cells, the i9's `oce` arms;
- `open`: the open decoder on every run-route block;
- `opend`: the same decoder under the density line.

## 1. The correctness gate

`moonshot check` ([`src/check.rs`](src/check.rs)) checks the output byte for byte against
libbzip2 (tests/common's oracle). It also checks every block's CRC and every stream's combined
CRC again, recomputed from the candidate's output with the bytewise reference CRC. These checks
are apart from the decoder's own.

**What it decodes:**
- tests/common's samples (text, runs, random, few symbols, `abab`, period 3, all bytes) at
  levels 1 and 9;
- the crate's corpora (its own text, random, one byte, a pattern, runs, records);
- multi-stream and mixed-level inputs, an empty stream among them;
- crafted blocks no encoder writes (tests/decode.rs' generator, 300 cases, 60 with `--quick`):
  - random columns, whose LF permutation is several cycles;
  - **periodic** columns, one short cycle repeated;
  - 1 to 900,000 rows, any primary index;
  - blocks that end inside a run, which must be refused as 0.5.6 refuses them;
  - a multi-block stream mixing periodic and multiple-cycle blocks;
  - Cells' witnesses;
- the Wikidata bench corpus;
- 600 mutated inputs (bits flipped, bytes set, cuts, deletions; 120 with `--quick`). On these,
  the candidate must give 0.5.6's result exactly, its bytes or its error kind, and must never
  panic.

**How it decodes each valid input:**
- sequentially through `decode_block`, on the detected and the scalar kernels;
- by the shipped workers with the candidate at 1, 2 and every CPU.

## 2. Container timing

`moonshot run` makes one decode in a fresh process. The input is read before the clock starts;
the workers are built and the output drained inside it. The run reports wall, CPU and peak RSS
(`VmHWM`). How the runs are arranged:
- **Workers:** every count from 1 to the container's CPUs (`--workers` to choose).
- **Rounds:** `--rounds` paired rounds. Each round's order rotates the arms, so each arm takes
  each position equally often.
- **The gap:** a 2 s settle between runs (`--settle 0` runs them back to back). Any gap shorter
  than 2 s is refused, because MONODROMY record 68 §4 found a 0.25 s gap causes slow starts.
- **The reading:** the geometric mean of the rounds' paired ratios (candidate over lazy), with a
  95% t interval on the logs. The verdict is FASTER, SLOWER or INCONCLUSIVE. The speedup is
  1/ratio, so the target is a ratio of 0.800 or less.
- **Peak memory:** the median peak-RSS ratio.
- **opend:** `--with-opend` times it as a third arm.

The container is noisy. The null candidate's 3-round score on this 4-CPU development machine
gave ratios from 0.946 to 1.079, with intervals up to [0.81, 1.44]. Use more rounds on a busy
machine, and expect an identical kernel to land outside its interval now and then.

## 3. The contention proxy

[`chaingrind/chaingrind.c`](chaingrind/chaingrind.c) is a Valgrind tool, built out of tree
against the installed Valgrind's static libraries ([`chaingrind/build.sh`](chaingrind/build.sh);
Debian and Ubuntu's `valgrind` package ships them).

**What it runs.** `moonshot proxy --warm` decodes one Wikidata block on one thread, the worker's
`decode_block`, between two client requests, in a worker's steady state:
1. **The block once, unseen.** A kernel's buffers grow only when a block needs more than they
   hold, with a zero-filled reallocation that Valgrind runs a byte a store. A real worker's
   buffers reach their size within its first few blocks. Measuring a growth once made one
   block 6.5× slower than its steady state (block 47: 50.5 against 7.7 cycles a byte).
2. **The warm block** (block 1), which fills the tool's caches and counts nothing.
3. **The block, measured.** Dirty lines left at the end are charged net of those found at the
   start.

**The sample.** The bench corpus has 233 full blocks and 198 tiny ones (the dump windows' stream
tails, a few hundred bytes each). A score decodes 48 full and 16 tiny blocks
(`--proxy-blocks 48+16`), spread evenly, one Valgrind process each, in parallel. Each stratum's
mean is weighted by its count in the corpus. **Every interval is a paired bootstrap over the
blocks** (the same resampled blocks for the candidate and lazy). The per-block ratios are wide,
0.70 to 1.23 for opend over lazy, so with 10 full blocks the ratio's 90% range was
[0.90, 1.01]. That was the old 16-block sample, and why its fit moved.

**A cache model.**
- L1 is 32 KiB, 8-way. L2 is 256 KiB, 4-way. The LLC is 20 MiB divided by W, the i9's L3
  shared by W workers, 16-way.
- Replacement is true LRU. Stores write-allocate, except fast-string (`rep movs`, `rep stos`) and
  non-temporal stores: they write whole lines with no read for ownership, as Intel cores do.
- An L2 streamer prefetches four lines ahead on sequential misses.
- Each line carries its fill time, so a demand access to a line still in flight waits for it.

**A timing model, one value at a time.** Every IR temporary, guest register byte and memory
word carries the cycle it becomes ready.
- **Dispatch:** 4 wide. An instruction can't dispatch until the instruction 224 before it has
  retired (the ROB).
- **Latencies:** loads take the latency of the level that served them. Address arithmetic,
  widening and narrowing take 0 cycles, multiplies 3, divides 20, other operations 1.
- **Misses:** at most 12 misses past L1 are outstanding (the fill buffers).
- **Stores:** a 56-entry store buffer drains in order, once each line is owned.
- **Forwarding:** a store's data reaches a later load of the same address.

**Chains.** Each value also carries the loads on the path that made it late. That path counts
only while it is binding, meaning an input became ready after the instruction dispatched.

**What it reports**, per output byte, in three groups never combined:

| group | quantities |
|---|---|
| transfers | L1↔L2 and L2↔L3 bytes; DRAM bytes (demand, prefetch and writebacks) |
| dependent chains | dependent DRAM links (DRAM loads whose address waited on an earlier DRAM load: record 71's serially dependent L3-miss chains); dependent past-L2 links (the same for loads served by L3 or DRAM); the longest of each chain; dependent loads at any level |
| overlap | DRAM memory-level parallelism (the loads' total latency over the time at least one is outstanding); for each dependent DRAM link, the DRAM loads outstanding beside it; DRAM loads per dependent link |

Beside the three groups the report gives the model's cycles and instructions per byte, and each
arm's **DRAM-latency sensitivity**: how much its cycles a byte rise, in per cent, for 100
cycles more DRAM latency. It is measured, not inferred: the first 16 full blocks are run again
at the DRAM latency plus 150 cycles.

**How contention enters.**
- **The LLC's share:** 20 MiB / W.
- **The L3-hit latency:** 42 + κ·(W − 1) cycles. κ is the one fitted parameter.
- **The DRAM latency follows each arm's own bandwidth.** This is a fixed point (`solve`):
  - *The latency curve:* the i9's L3-miss latency as a queue in the memory controller's load,
    idle + k·U/(1 − U), with U the share of its 33.39 GB/s ceiling. It is fitted to
    bench-imc-ten's four (GB/s, latency) readings, which come from another arm (`occ`), never
    from the arms the scorer is checked on. It gives 438 cycles idle, 495 at lazy's 20 GB/s
    and 600 at 27 GB/s.
  - *Lazy's bandwidth:* its measured 20.0 GB/s at 7 workers (open-return, dram.txt), scaled to
    each W by its modelled bytes a second.
  - *An arm's bandwidth:* lazy's at that W, times the arm's DRAM bytes a byte and its rate,
    both relative to lazy's. The process's traffic outside `decode_block` (input, scan, output
    copy) is the constant c0: the i9's 8.0 B/B for lazy less the model's.
  - *The solution:* an arm's cycles at a latency are its measured cycles plus its sensitivity
    times the difference. The latency where the curve and the arm's bandwidth agree is
    unique, and bisection finds it.
  - *Why:* an arm that moves more bytes waits longer for each of them, as eager Cells does at
    28 GB/s against lazy's 20.
- **The predicted ratio over lazy at each W** is the arm's cycles a byte over lazy's, each at
  its own latency, with the workers assumed saturated.

**Each prediction shows what drives it:**
- the instruction ratio (work);
- the ratio at the nominal latencies (cache and dependency behaviour at equal contention);
- the contention feedback (what the arm's own bandwidth does to its latency);
- each arm's DRAM latency and sensitivity;
- the three groups above.

**What the model leaves out:** no other cores' traffic, coherence, physical indexing or slice
hashing; LRU, not the i9's adaptive replacement; no branch mispredictions (control dependences
are ignored, as a predicted branch would ignore them).

**Out of calibration.** At each W, if any of the candidate's access-pattern statistics falls
outside the range the four reference arms span (widened by 10%), the report prints OUT OF
CALIBRATION with the statistics that fall outside, and says its prediction there is unreliable.
The reference arms' statistics are in `calibration.json` (`reference`), from
`moonshot.py calibrate --write`. The statistics watched are:
- the DRAM-latency sensitivity;
- dependent DRAM and past-L2 links;
- L2, L3 and DRAM bytes;
- past-L2 and DRAM loads per byte;
- instructions per byte.

## Calibration, and what it shows

[`calibration.json`](calibration.json) holds the model's constants, the i9 cells, and the
reference arms' metrics and predictions (`reference`, `reference_predictions`). The cells
come from the open-return bundle (evidence fa9707e9, `open-return/run-20261118T210422Z`:
`verdict.txt`, `dram.txt`; 12 rounds, unpinned, Wikidata ×4).

**The fit.** κ is fitted on one cell, opend 7/28 against lazy at 7 workers (measured 0.940). Over
the grid 0, 2, 4 and 6, `moonshot.py calibrate` gives 0.956, 0.971, 0.982 and 0.992, so
**κ = 0, at its bound.** A rising L3-hit latency only hurts opend in this model, which has about
twice lazy's dependent past-L2 links. So no parameter is effectively fitted on timing: the
fitted cell is missed by 0.016. Record 71 puts opend's kernel-only effect, corrected for the
reference's construction cost, at about 0.946, which is 0.010 from the prediction.

**The held-out check** (48 full + 16 tiny blocks, 400 paired bootstrap resamples):

| cell | threads | role | i9 | handed over (v1) | v2 without feedback | **v2** [95%] | side | in interval | DRAM ratio i9 / v2 |
|---|---|---|---|---|---|---|---|---|---|
| opend 7/28 | 10 | fitted | 0.940 | 0.940 | 0.924 | **0.956** [0.922, 0.986] | yes | yes | 1.035 / 1.144 |
| open 7/28 | 10 | held out | 0.915 | 0.958 | 0.895 | **0.924** [0.857, 0.979] | yes | yes | 0.975 / 1.100 |
| oce 7/28 (eager) | 10 | held out | 1.268 | 1.154 | 1.298 | **1.397** [1.229, 1.578] | yes | yes | 1.723 / 1.724 |
| opend 8/32 over 7 workers | 10 | held out | 0.919 | 0.876 | 0.868 | **0.931** [0.895, 0.967] | yes | yes | 1.152 / 1.279 |
| open 8/32 | 8 | held out | 0.929 | 0.939 | 0.844 | **0.905** [0.822, 0.966] | yes | yes | 0.954 / 1.041 |
| opend 8/32 | 8 | held out | 0.944 | 0.916 | 0.883 | **0.947** [0.916, 0.975] | yes | yes | 1.012 / 1.090 |
| oce 7/28 over 8 workers | 8 | held out | 1.300 | 1.206 | 1.320 | **1.420** [1.250, 1.625] | yes | yes | – / 1.469 |

- **Same side of 1:** every cell.
- **Inside its 95% interval:** every cell.
- **Pairs at the same thread count in the i9's order:** 9 of 9 (v1: 6 of 9; v2 without the
  feedback: 8 of 9, open 7/28 against opend 8/32).
- **Is the held-out ranking right? Yes, at both thread counts:** open < opend < 1 < eager
  Cells, and opend 8/32 over 7 workers below opend 7/28.

**What was wrong in v1, and why.** These are diagnoses, not tuning: each was found in the
harness or the sample, before κ was refitted.
1. **Buffer growth inside the measured region.** LazyCells grows its buffers by a zero-filled
   reallocation when a block needs more than they hold. Valgrind runs glibc's `rep stosb` a
   byte at a time. Whichever block triggered it carried about 10 MB of byte stores into the
   store-buffer model: on block 47, 44 of lazy's 50.5 cycles a byte. This is why eager Cells'
   loss was underpredicted (1.15) and the fit was unstable.
2. **The sample.** Opend/lazy per-block ratios run from 0.70 to 1.23. Ten full blocks (the
   16-block sample's) give a 90% range of [0.90, 1.01]. This is why κ moved from 8 to 4 and
   why open against opend came out in the wrong order.
3. **A cold cache with all its dirty lines charged:** about 8.6% of lazy's cycles and 22% of
   its DRAM bytes, but under 2% of opend's.
4. **One DRAM latency for every arm.** The 549 and 649 cycles were the `occ` arm's, at 25 and
   29 GB/s, not lazy's at 20.

**The feedback, an honest note.**
- *Why it is in v2:* the bandwidth-latency term was added after v1's eager miss. That miss
  turned out to be item 1, not the missing term.
- *Why it stays:* it rests on measurements (bench-imc-ten's curve, lazy's measured
  bandwidth), not on these arms' timing, and it ranks 9 of 9 against 8 of 9 without it.
- *What it costs:* it overpredicts eager (1.40 against 1.27). It also penalises the open
  kernels for a DRAM ratio the model overpredicts.
- *How the run judges it:* the i9 calibration run measures each arm's own L3-miss latency,
  which tests the term directly.

**The mechanism: the model and record 71 disagree, and the i9 run can tell them apart.**
- *The model's account at 7 workers:* opend wins with 27% fewer instructions (17.5 against
  24.0 a byte) and half the L2 and L3 traffic. Its dependent past-L2 links are *more*, not
  fewer (0.057 against 0.032 a byte). Its DRAM-latency sensitivity is the same as lazy's
  (8.6% against 8.8% for 100 cycles).
- *Record 71's reading (labelled I):* "more base work … a much smaller latency-contention
  coefficient". The instruction counts here are exact, and they contradict the "more base
  work".
- *The predictions differ in W:*
  - the model: opend at **0.87 over lazy at 1–4 workers**, its advantage shrinking to about
    0.95 at 7–10 as the feedback grows (opend moves more DRAM bytes);
  - record 71: a crossover, with opend slower at low W and its gain growing with W.
- *The i9 calibration run* times W = 1, 2, 4, 7, 8 and 10 with stall and latency counters, and
  will show which.

**Known limits.**
- **DRAM bytes.** The open kernels' DRAM ratio is overpredicted: 1.10–1.14 against 0.98–1.04
  measured. Eager Cells' is right (1.724 against 1.723).
  - *Ruled out:* fast-string no-RFO stores, tested and kept, which save under 0.1 B/B.
  - *Untested suspects:* LRU in place of the i9's scan-resistant adaptive replacement, which
    would spare a streaming kernel's working set; and the streamer's overshoot.
- **κ at its bound.** The L3 term contributes nothing; L3 contention is in the model only as
  the LLC's share.
- **Intervals.** They are wide, about ±4% for opend and ±13% for eager Cells at 7 workers,
  because the per-block ratios spread. A candidate's 2–3% effect is below the proxy's
  resolution: use the proxy as a screen for gross regressions and for the direction and
  mechanism of a gain, not for a few per cent.
- **Absolute scale.** Lazy models at 10.5 cycles a byte at 7 workers. The i9's 0.312 s for
  838.8 MB is about 12.8 cycles a byte a worker at 4.9 GHz. The model omits the scan,
  input and output copy.
- **The chain counts of records 72–75** (0.2–0.25 dependent stages a byte for lazy, by the
  algorithm) are not this model's machine-level counts (1.49 dependent loads a byte at any
  level, 0.032 past L2). Astras A and C aren't `InverseBwt` kernels, so the scorer can't
  run them.
- **Historical bundles.** The `evidence` branch's other i9 bundles predate 0.5.6's kernels
  (eager-trim, 8426934a), or time other arms (imc-ten's `occ`, used only for the latency
  curve). None is used for timing calibration.

### The i9 calibration run

`just bench-moonshot-calibrate` is a frozen workstation recipe.
- **What it runs:** lazy, opend, open and eager Cells in the shipped workers, kernel-only (all
  four through `parallel_workers_with`).
- **The arrangement:** 1, 2, 4, 7, 8 and 10 workers; unpinned; 10 paired, rotated rounds; a
  2 s settle; Wikidata ×4.
- **What it records:** each run's `perf stat` counters:
  - cycles and instructions;
  - `cycle_activity.stalls_l3_miss`;
  - `l1d_pend_miss.pending` and `.pending_cycles`;
  - `offcore_requests(_outstanding).l3_miss_demand_data_rd`, the mean L3-miss latency as
    bench-imc-ten measured it, per arm.
- **Its checks first:** every arm passes the quick gate on the i9. Every counter is checked
  before anything is built.
- **What it leaves:** only its bundle. It deletes its build and corpora. It takes about
  20 minutes.
- **How it is used:** the scorer is frozen at the commit the #145 LOG names, with κ = 0 from the
  open-return bundle. **Every cell of the run is held out.** The predictions, made before any
  data, are `calibration.json`'s `reference_predictions`. It decides nothing in production.
- **A dry run:** `just bench-moonshot-calibrate 10 2 4 1,2,4,7,8,10 1` is untimed, two rounds,
  no counters. DRY is the 5th positional argument: `just` has no keyword arguments, so
  `DRY=1` alone would be taken as ROUNDS, and the recipe refuses it.

## Files

| file | what |
|---|---|
| `moonshot.py` | the scorer: `score`, `report`, `calibrate`, `sweep`; statistics and the report |
| `test_moonshot.py` | its tests (statistics, the gate's verdict, the settle rule, the proxy's reading, the range flag) |
| `src/main.rs`, `src/check.rs`, `src/arms.rs` | the harness: the gate, one timed run, the proxy's driver; the reference arms |
| `build.rs` | the candidate, compiled as `crate::candidate` |
| `chaingrind/` | the Valgrind tool and its build |
| `calibration.json` | the model's constants, the i9 cells, the reference arms' metrics and predictions |
| `corpus.sh` | the Wikidata bench corpus |
| `candidates/` | the template and the worked examples |

A score writes `bench/moonshot/target/moonshot/score-<time>/`: `summary.txt`, `summary.json`,
`check.jsonl`, `runs.jsonl`, `proxy.json` and `proxy-runs.jsonl` (every chaingrind run's counters;
`moonshot.py calibrate --again DIR` reads a calibration's again without running anything).
