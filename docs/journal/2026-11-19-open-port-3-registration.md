# open-port-3: the proportionate validation of the open port for 0.5.7

*19 November 2026. The Pathfinder's registration, frozen before any of Part B's data, for one
urgent review by the Researcher through the Coordinator.*

- **Assigned:** #145 QUEUED 7013818037, under Louis's authority (LOG 7013839409): "Ship it
  autonomously please yes put simply, no marathon runs for ill founded reasons".
- **The criteria are the Coordinator's** (QUEUED 7013818037). This document implements them and
  states what they cost and what they give up. It changes none of them.
- **What stands:**
  - the first registration's NOT ADOPTED (`open-port/run-20261119T125538Z`), permanently;
  - #635 is shelved and will not be run; its branch is kept as a record.
- **The change:** the port, #630 (`perf/open-kernel`), at
  9743bbe0d51ba451b8b8ab9e6fd731c238940052, against v0.5.6 (530ed404). Its correctness evidence
  (B5, D115 and Miri) is unchanged.
- **The recipe:** `just bench-open-port-3`, on `bench/open-port-3`. The scorer is
  `bench/open_port3.py`, and its checks are `bench/test_open_port3.py` (35, in CI).
- **One attempt.** It is archived as `open-port-3/run-*.tar.gz`, with the guard, the timing marker,
  the queue entry and the ready tag, as before.

## 1. The decision question

**Does shipping `OpenCells` in 0.5.7 cost any guard workload a wall-time regression that matters,
while keeping its Wikidata gain?**

The first attempt answered the gain (FASTER on wikidata-x4 and on held-out Wikidata at every
count). It did not answer the guards. Four guard cells failed G's point bound of 1.02 and none was
SLOWER:
- ho-code-l1 at 8 and 10 threads;
- random at 10 threads;
- ho-code-l9 at 20 threads.

All four were on corpora where no block opens. This attempt asks whether that reflects a real,
common cost, or noise in single cells.

## 2. The causal argument: what the port changes on a block it does not open

From the diff 530ed404 → 9743bbe0 (the Pathfinder's path facts, LOG 7013573891; memo §2; and the
Researcher's mechanisms, REVIEW 7013742538):
1. **The gate.** A refused run-route block costs `admits()`: about four compares and two
   multiplies (`src/open.rs:404–409`). It then delegates to the shipped `LazyCells`
   (`src/open.rs:425`), which repeats its own density and sparse checks.
2. **The monomorph.** The worker loop is a new instance,
   `StreamDecoder::launch<with_inverse<OpenCells>>`, in place of `with_inverse<CellsVariant<1>>`.
   Its source is the same and its code is a separate copy.
3. **The scratch.** An unused `open::Kernel` sits inline in each worker's `Scratch`, about 380 B.
   It shifts the offsets of the later fields. It allocates nothing on a block that does not open.
4. **The layout.** The binary's text grows by 44 KB (+2.4%), partly from the harness's two new
   arms (`examples/ab.rs`, +25 lines). Every function's address can move.

Mechanism 1 adds instructions. Mechanisms 2–4 can change misses and mispredictions with no
change in instructions. Part A measures both, on the decoder's real path. Part B measures what
any of them costs in wall time on the i9.

## 3. Part A: the cachegrind causal diagnostic (container, done before this registration)

**Result: no VOID, and every one of the 124 triggers attributed.** On these blocks the decoder
executes the same instructions as v0.5.6 and opens nothing. Every trigger is either the
measurement's own run-to-run spread or the `ab` harness's code. Below the triggers, the decoder
shows layout-scale miss differences, which are real and consistent but small. Part A cannot bound
the native wall time; Part B does.

**What ran:**
- *The builds:* both container builds of the registered commits, rustc 1.99.1 with the release
  profile (`ab` sha256 bfed8098… for the port, 18bb5d96… for v0.5.6), under valgrind 3.22's
  `cachegrind --cache-sim=yes --branch-sim=yes`.
- *The path:* `ab child mibz2 T`, the production entry point (`BzReader::parallel`, the
  production plan), at 8, 10 and 20 threads, 5 repeats per build. Here, on 4 CPUs, the plan ran
  8, 10 and 20 workers.
- *The i9's worker count:* `ab child mibz2-workers-lazy 7` runs the shipped lazy route at exactly
  7 workers, the i9's plan at 10 and 20 threads, with 3 repeats per build.
- *The inputs:* the nine B1 corpora, the first registration's bytes (verified against the
  manifest).
- *The total:* 324 runs, 36 cells.

**The VOID checks pass:**
- *Outputs:* every run's output size and digest is the corpus's registered one.
- *Opened blocks: none.* Every opened block calls the open kernel's out-of-line `open::emit`
  (`src/open.rs:361`). In all 162 port runs, `emit` and `Kernel::induced` have zero instructions.
- *The detector works:* a positive control on a 9 MB Wikidata sample, whose blocks the gates
  admit, gives `emit` 32.0M instructions and `induced` 28.0M (`part-a/positive-control.txt`).

**The 124 triggers, on whole-program counts at the Coordinator's thresholds** (Ir beyond 0.1%,
a miss or mispredict count beyond 2%; 396 cell-events). Each has a written attribution in
`part-a/summary.json`, generated from the per-function split (`attribute_triggers.py`):

| class | triggers | events | what moved |
|---|---|---|---|
| within the same-binary spread | 84 | Ir, I1mr, D1mw, DLmr, DLmw, Bcm | No larger than the same binary's range over its own repeats. Valgrind serialises the threads, so the harness's time-driven sampler, the pool's waits and the order of first touches vary from run to run. Whole-program Ir varies by 0.1–1.6% between repeats of one binary, and LL misses (rare cold misses) by far more. |
| harness | 38 | Bim (x1.3–x3.0), I1mr | The `ab` binary's own sampler: `ab::run_delay`, `ab::sample_tasks`, and the /proc parsing in std's split and UTF-8 code. The two `ab` builds differ (+25 lines of arms, 44 KB more text), so these functions sit at other addresses in the simulated predictor and I1. That is a few hundred to about 2,000 events, at most 0.0018% of the run's cost by cachegrind's model. |
| runtime | 2 | DLmr | libc's `memcpy` LL misses, just past the same-binary range. Records at 20 threads: +65,758 against a range of 65,532. ho-code-l1 at 7 workers: −2,781 against 1,611. At most 0.098% of the run's cost. |

**The decoder itself** (the mibz2 crate's functions, its worker monomorph included;
`part-a/decoder.txt`):
- **Ir: x0.99996 to x1.00008 in 35 of the 36 cells.** The other cell is ho-code-l1 at 7
  workers, +0.069%. That is all in `bwt::room_segments` and `memcpy`, whose counts move by ±0.2%
  between repeats of one binary there: the work splits differently across idle workers. It is no
  added code. The gate's own instructions (§2.1, about 6 per block) are below this resolution.
- **Systematic miss differences, the same at every thread count, with the same instructions:**
  1. *D1 write misses in `bwt::room_segments`* (LazyCells' segment fill): random +2.0%,
     records +2.3%, ho-prose-l9 +0.5%, onebyte +0.5%, pattern +0.4%, but text −1.7%,
     ho-code-l9 −1.7% and runs −2.2%. This is the data-layout mechanism (§2.3–2.4).
  2. *LLi misses* +11 to +23 (+1–3%) of about 700–1,100 cold code misses: code layout (§2.4).
  3. *Indirect mispredictions in the worker loop:* +53 on text and +68 to +104 on pattern
     (about 200 each), mixed in sign elsewhere: code layout in the simulated predictor (§2.2,
     §2.4).
- **Their cost by cachegrind's model** (Ir + 10 per L1 miss or mispredict + 100 per LL miss):
  the decoder's total is x0.9984 to x1.0015 in the 25 cells whose same-binary range is under 1%.
  The largest systematic term, records' +135k D1 write misses, is about 0.02% of its decoder
  cost.

**What Part A shows, and what it does not:**
- *It shows* that, on blocks it does not open, the port runs the same decoder instructions and
  opens nothing. Its effects are layout effects whose size in a simulated machine is under 0.2%.
- *It does not bound native wall time* (the Researcher's point, REVIEW 7013742538):
  - cachegrind serialises the threads, so it models no contention and no SMT;
  - its caches are this host's geometry (32 KB I1, 48 KB D1, a 264 MB LL), not the i9's
    (32 KB, 32 KB, 20 MB L3);
  - it has no prefetchers, µop cache or frequency effects.

  Part B measures the wall time.

## 4. Part B: one i9 attempt

**What runs** (`open_port3.py plan`; `ab run`, the port's build against v0.5.6's, each its own
binary run as the harness's child; paired rounds in the harness's ABBA order, unpinned, at the
registered settle of 2 s):

| part | file | corpora | threads | rounds | records |
|---|---|---|---|---|---|
| B1, zero-route guards | `b1.jsonl` | text, random, records, runs, pattern, onebyte, ho-code-l1, ho-code-l9, ho-prose-l9 | 8, 10, 20 | 16 | 864 |
| B2, admitted guards | `b2.jsonl` | ho-wikidata-l1, ho-structured-l1, ho-structured-l9, ho-prose-l1 | 8, 10, 20 | 16 | 384 |
| B3, benefit | `b3.jsonl` | wikidata-x4, ho-wikidata-l9 | 8, 10, 20 | 12 | 144 |
| B4, sustained | `b4.jsonl` | wikidata-x16, back to back (settle 0) | 10 | 8 | 16 |

**The estimate: about 60 minutes on the i9.**
- *Timed part, about 48.5 minutes:* 1,392 settled records. The first attempt's decision run took
  2.07 s a record (1,096 records, 12:58:02 to 13:36:08, settle included). The 16 back-to-back
  records add about 20 s.
- *Setup, about 10 minutes:* both builds from scratch in a new `target/open-port-3`, then the
  corpora.
- *Dropped:* the first recipe's context arms (gate and crossover) and its census. They decided
  nothing and took 14 minutes.

The run is shorter than the 75–90 minutes allowed. The rounds are the Coordinator's, and they are
not padded.

**The inputs are the first registration's frozen 18,** checked against
`bench/open-port-inputs.sha256` before any timing. These are the same compressed bytes Part A
measured, so Part A's routing (no block opens on the B1 corpora) holds for the i9's inputs. The
code is the same and routing depends only on the bytes.

**The rule** (each cell: x_i, the rounds' paired log ratios of wall, the port over v0.5.6):
- **B1 and B2, pooled.** The estimand is the mean of the cells' mean log ratios, each cell
  weighted equally. Its 95% interval is a stratified bootstrap: rounds are resampled with
  replacement within each cell, 20,000 resamples, and the 2.5th and 97.5th percentiles taken. The
  generator is fixed (splitmix64, seed 7013818037), so the verdict is reproducible on any Python.
  **Pass:** the upper bound, as a ratio, is at most **1.01**.
- **B1 and B2, credible regression.** A cell is flagged when its one-sided lower bound,
  m − t(1 − 0.05/k, 15)·s/√16, lies above ln 1.02. Here k is 27 for B1 and 12 for B2. **Any flag
  fails.**
- **Memory, every guard cell (B1 and B2):** peak RSS, the median of the port's peaks over the
  median of v0.5.6's, at most **1.10**.
- **B3, benefit:** every one of the 6 cells is a gain under Holm at family α 0.05 (one-sided t, as
  P1 was), and every cell's peak RSS is at most **1.00**.
- **B4, sustained:** not SLOWER; the two-sided 95% t-interval's lower bound is not above 1. Memory
  is reported but not judged, as the first registration's S was.
- **Part A:** no VOID, and every trigger carries a written attribution. The scorer reads
  `part-a/summary.json`, committed beside this registration before timing. The recipe copies it
  into the bundle.

**The verdict:**
- **ADOPT** only if Part A, B1, B2, B3, B4 and the memory caps all hold.
- **VOID** if the inputs, binaries, outputs or routing mismatch. Specifically: any of
  `open_port.py`'s VOIDs in any file (a record missing, duplicated, failed or outside the roster;
  a field of the wrong type; a wall, RSS or output that is not a finite positive number; the
  builds or the 18 inputs not the registered ones; the machine not the i9; the Wikidata corpora's
  observed workers not the shipped 8/7/7; a dry run); a corpus's output, or a build's arm,
  differing between files; or a Part A VOID. A VOID allows one repair of untimed setup only.
- **NOT ADOPTED** otherwise, with every failing condition named. The Open campaign then ends, and
  #630 is closed with the evidence kept.

## 5. Why the pooled-plus-flag check is proportionate, and what it gives up

**What it bounds.** The mechanisms of §2 that act on every block are common: the monomorph, the
scratch layout and the code layout act on every worker and every block. A common effect moves
every cell together, and the pooled bound sees it with 432 (B1) or 192 (B2) paired rounds. The
per-cell flag catches a single cell whose regression is clearly above 2%.

**The operating characteristics, before any data** (`open-port-3-registration/oc.py` and `oc.txt`):
- *How:* each cell's 16 rounds are drawn from the first attempt's 12 residuals for that cell
  (centred: "own"), or from the part's residuals pooled ("part"). 2,000 simulations each, with
  the bootstrap at 2,000 resamples in the simulation and 20,000 in the scorer.
- *What it found:*

| | B1 (27 cells; residual sd median 0.036, max 0.126) | B2 (12 cells; median 0.040, max 0.075) |
|---|---|---|
| **no effect:** the pooled bound fails | 0.015 own, 0.017 part | **0.073 own, 0.070 part** |
| **no effect:** any cell flagged | 0.035 own, 0.000 part | 0.000, 0.000 |
| every cell +0.5%: the pooled bound fails | 0.45, 0.43 | 0.56, 0.59 |
| every cell +1.0%: the pooled bound fails | 0.967, 0.970 | 0.968, 0.968 |
| every cell +1.5%: the pooled bound fails | 1.000 | 1.000 |
| one cell +2%: its flag (least / most noisy cell) | 0.008 / 0.071 | 0.067 / 0.001 |
| one cell +5%: its flag (least / most noisy cell) | 1.000 / 0.203 | 1.000 / 0.042 |
| one cell: the slowdown its flag catches with power 0.80 | x1.032 to x1.167 | x1.031 to x1.097 |

- **The false NOT ADOPTED rate under no effect is about 0.12** under the own-cell model, treating
  the three as independent: 1 − 0.985 × 0.965 × 0.927. Most of it is B2's pooled bound (0.073).
  Its 12 cells give the bound less room than B1's 27 at the same 1.01 margin, and its admitted
  cells are spikier.
- **These criteria were set after the first attempt's data was seen.** Applied to that attempt's
  12 rounds, they give:
  - *B1:* pooled 0.9995 [0.9937, 1.0051], with no cell flagged;
  - *B2:* pooled 0.9923 [0.9858, 0.9990], with no cell flagged.

  This attempt is prospective only for its own new data (C1a). Its rule is frozen here, before
  that data, and is not relaxed after it (C2).

**What it gives up, accepted under Louis's direction:**
- *Per-cell power at a true +2% is far lower than #635's.* A single cell with the first attempt's
  median spread (sd 0.036) is flagged with 80% power only at about +6%. The noisiest cell (sd
  0.126) needs about +15%.
- *Single-cell regressions of 2–5% will usually pass.* That is the price of not running about 7
  hours. A common effect of 1% fails the pooled bound with probability about 0.97.
- *The flag's false-alarm rate is modelled, not exact.* It rests on the first attempt's
  heavy-tailed residuals: 0.035 for B1 under the own-cell model.

## 6. The preflight

`just bench-open-port-3 9743bbe0d51ba451b8b8ab9e6fd731c238940052 1` (DRY) ran in the container
at e4d60735, the recipe as frozen, and is archived as
`open-port-3-registration/preflight/run-20261119T160851Z.tar.gz`:
- both builds made, and the 18 inputs built and verified ("the builds and all 18 inputs are the
  registered ones");
- every corpus decoded once by each build, with the same output size and digest for all 16;
- the scorer's 35 checks pass;
- no timing was kept.

The timed loop's path handling was checked separately: `ab run` records `text` for the
`gen/text.bz2` symlink, and both arms succeed.

## 7. Reproduce

- **The scorer's checks:** `python3 bench/test_open_port3.py` (35 checks; in CI).
- **The plan:** `python3 bench/open_port3.py plan`.
- **Part A:** `docs/journal/2026-11-19-open-port-3-registration/part-a/` holds:
  - *the runs:* `run.sh` and `run7.sh`, one per line of `jobs.txt` and `jobs7.txt`, through
    `xargs -P4`;
  - *the analysis:* `analyse.py`, `attrib.py`, `attribute_triggers.py` and `cgfn.py`. Extract
    `raw.tar.xz` beside them, then run them in that order;
  - *the outputs:* `summary.json` (which the scorer reads), `attribution.json`, `table.txt`,
    `attrib.txt`, `decoder.txt` and `positive-control.txt`;
  - *the raw cachegrind and child outputs:* `raw.tar.xz` (sha256 464d6264…7f0e).
- **The operating characteristics:** `python3 oc.py RUNS.jsonl`, with RUNS the first attempt's
  `runs.jsonl` from its evidence bundle (sha256 fb4771f5…a523).
