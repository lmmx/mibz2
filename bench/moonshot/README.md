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

**What it runs.** `moonshot proxy` decodes 16 sampled Wikidata blocks on one thread, the
worker's `decode_block`, between two client requests. Before the region it decodes one block
outside it, to size the buffers. Inside the region, the tool does three things.

**A cache model.**
- L1 is 32 KiB, 8-way. L2 is 256 KiB, 4-way. The LLC is 20 MiB divided by W, the i9's L3
  shared by W workers, 16-way.
- Replacement is true LRU. Stores write-allocate.
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
| transfers | L1↔L2 and L2↔L3 bytes; DRAM bytes (demand, prefetch, writebacks and the end's drain) |
| dependent chains | dependent DRAM links (DRAM loads whose address waited on an earlier DRAM load: record 71's serially dependent L3-miss chains); dependent past-L2 links (the same for loads served by L3 or DRAM); the longest of each chain; dependent loads at any level |
| overlap | DRAM memory-level parallelism (the loads' total latency over the time at least one is outstanding); for each dependent DRAM link, the DRAM loads outstanding beside it; DRAM loads per dependent link |

Beside the three groups the report gives the model's cycles and instructions per byte. The
predicted ratio over lazy at each W is the model's cycles per byte against lazy's, with the
workers assumed saturated. It is one proxy quantity among the others.

**Contention enters in three ways only:**
- the LLC's share, 20 MiB / W;
- the DRAM latency: the i9's measured L3-miss latency (bench-imc-ten: 549 cycles at 7 workers,
  649 at 10), linear in W and floored at 250 cycles. This is measured, not fitted;
- the L3-hit latency: 42 + κ·(W − 1) cycles. κ is the one fitted parameter.

**What the model leaves out:** no other cores' traffic, coherence, bandwidth limit, physical
indexing or slice hashing; no branch mispredictions (control dependences are ignored, as a
predicted branch would ignore them).

**Out of calibration.** At each W, if any of the candidate's access-pattern statistics falls
outside the range the four reference arms span (widened by 10%), the report prints OUT OF
CALIBRATION with the statistics that fall outside, and says its prediction there is unreliable.
The statistics watched are dependent DRAM and past-L2 links, L2, L3 and DRAM bytes, past-L2 and
DRAM loads per byte, and instructions per byte.

## Calibration, and what it shows

[`calibration.json`](calibration.json) holds the model's constants and the i9 cells. The cells
come from the open-return bundle (evidence fa9707e9, `open-return/run-20261118T210422Z`:
`verdict.txt`, `dram.txt`; 12 rounds, unpinned, Wikidata ×4).

**The fit.** κ is fitted on one cell: opend 7/28 against lazy at 7 workers, measured 0.940.
`moonshot.py calibrate` scans a grid of κ values and keeps the nearest, which gives **κ = 4**.
That is 66 cycles of L3-hit latency at 7 workers. Every other cell is held out.

The 16-block score of the null candidate on this machine (`moonshot.py score
candidates/lazy_cells.rs`), against the i9:

| cell | threads | role | i9 measured | proxy predicted | same side of 1 | DRAM ratio i9 / proxy |
|---|---|---|---|---|---|---|
| opend 7/28 | 10 | fitted | 0.940 | 0.940 | yes | 1.035 / 1.172 |
| open 7/28 | 10 | held out | 0.915 | 0.958 | yes | 0.975 / 1.173 |
| oce 7/28 (eager) | 10 | held out | 1.268 | 1.154 | yes | 1.723 / 1.648 |
| opend 8/32 over 7 workers | 10 | held out | 0.919 | 0.876 | yes | 1.152 / 1.321 |
| open 8/32 | 8 | held out | 0.929 | 0.939 | yes | 0.954 / 1.140 |
| opend 8/32 | 8 | held out | 0.944 | 0.916 | yes | 1.012 / 1.140 |
| oce 7/28 over 8 workers | 8 | held out | 1.300 | 1.206 | yes | – / 1.421 |

**What it shows, plainly:**
- **opend over LazyCells at 7–10 workers: yes.** The proxy ranks opend faster at 7, 8 and 10
  workers (0.940, 0.916, 0.863), as the i9 did.
  - *But not for record 71's reason.* In the model, opend's dependent past-L2 links per byte are
    about twice lazy's (0.052 against 0.023), and its DRAM bytes are higher. Its predicted gain
    comes from fewer instructions (17.3 against 23.9 a byte) and half the L2 and L3 traffic.
  - *It also predicts a gain at 1–4 workers* (about 0.90–0.91). Record 71 reads the VM's loss as
    a crossover. The model does not reproduce one.
- **Held out, every cell is on the right side of 1.** Eager Cells is slower and open is faster.
- **The held-out ranking is partly right: 6 of 9 pairs at the same thread count.**
  - *Wrong:* open against opend. The i9 measured open faster (0.915 against 0.940 at 10
    threads, 0.929 against 0.944 at 8); the proxy predicts the reverse.
  - *The magnitudes are off.* Eager Cells' loss is underpredicted (1.15 against 1.27), and the
    worker-count step of opend 8/32 over 7 workers is overpredicted.
- **DRAM traffic.**
  - *The open kernels:* their DRAM ratio is overpredicted (1.14–1.17 against 0.95–1.04).
  - *Eager Cells:* about right (1.65 against 1.72).
  - *Absolute:* lazy's modelled DRAM at 7 workers is 6.0 B/B for one worker's `decode_block`
    alone, against the i9's whole-process 7.97–8.10 B/B. Records 74 and 75 put the scanner,
    input, output copy and the rest of the process at about 2.9 B/B.
- **The cycle scale is plausible.** Lazy at 7 workers models at 12.1 cycles per byte. The i9's
  0.312 s for Wikidata ×4 (838.8 MB) at 7 workers is about 12.8 cycles per byte a worker,
  assuming a 4.9 GHz all-core clock.
- **The chain lengths in records 72–75 can't be reproduced here.** Those records count about
  0.2–0.25 dependent stages per byte for lazy Cells and about 0.57 for Astra C's cursor, by the
  algorithm.
  - *Different definitions.* This model's "dependent loads at any level" is about 1.4 per byte
    for lazy, because it counts every machine load whose address waited on a load: stack
    reloads and table walks included. Its past-L2 links, 0.023 per byte, count only the links
    that reach L3.
  - *Not in the harness.* Astras A and C exist as journal prototypes, not as `InverseBwt`
    kernels, so the scorer can't run them.
- **Historical bundles mined, and why most are not used.** The `evidence` branch holds 95 i9
  bundles.
  - *eager-trim* (8426934a) measured eager over lazy at 7 workers at 1.091 and at 5 workers at
    0.964: the eager/lazy crossover in W.
  - *Why not used:* its kernels predate 0.5.6's admission and one-scan writer, which moved the
    same pair to 1.27. Only bundles built on 0.5.6's kernels calibrate this model: the
    open-return run and, when it lands, the opend sweep.

**The reading.** The proxy gets the sign of every held-out cell right, and it ranks opend above
LazyCells at 7–10 workers. It does not get the open/opend order, the low-W crossover, or the
mechanism record 71 proposes. Treat its predictions as a screen for gross regressions (a
candidate like eager Cells), not as an estimate of a few percent.

**A further i9 run** (`just bench-moonshot-calibrate`, below) and the opend sweep's bundle would
let κ, and perhaps a second parameter, be fitted on W-dependence. They would also test the
low-W prediction directly.

### The i9 calibration run

`just bench-moonshot-calibrate` is a frozen workstation recipe.
- **What it runs:** lazy, opend, open and eager cells in the shipped workers, kernel-only.
- **The arrangement:** 1, 2, 4, 7, 8 and 10 workers; unpinned; 10 paired rounds; a 2 s settle;
  Wikidata ×4.
- **What it records:** each run's `perf stat` stall counters, `cycle_activity.stalls_l3_miss`
  among them.
- **Correctness:** every arm passes the quick gate on the i9 first.
- **What it leaves:** only its bundle. It deletes its build and corpora.
- **How it is used:** κ is refitted on lazy and opend at 7 workers only. Every other cell is a
  held-out check, timing and counters alike. Its rule is posted on #145 before any data. It
  decides nothing in production.
- **A dry run:** `just bench-moonshot-calibrate DRY=1` is untimed, two rounds, no counters.

## Files

| file | what |
|---|---|
| `moonshot.py` | the scorer: `score`, `report`, `calibrate`, `sweep`; statistics and the report |
| `test_moonshot.py` | its tests (statistics, the gate's verdict, the settle rule, the proxy's reading, the range flag) |
| `src/main.rs`, `src/check.rs`, `src/arms.rs` | the harness: the gate, one timed run, the proxy's driver; the reference arms |
| `build.rs` | the candidate, compiled as `crate::candidate` |
| `chaingrind/` | the Valgrind tool and its build |
| `calibration.json` | the model's constants and the i9 cells |
| `corpus.sh` | the Wikidata bench corpus |
| `candidates/` | the template and the worked examples |

A score writes `bench/moonshot/target/moonshot/score-<time>/`: `summary.txt`, `summary.json`,
`check.jsonl`, `runs.jsonl` and `proxy.json`.
