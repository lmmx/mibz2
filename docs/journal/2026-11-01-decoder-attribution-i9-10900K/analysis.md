# The shipped decoder's resource attribution on the i9-10900K: validation and interpretation (read-only)

**What this is:**
- It reads the archive in this directory and nothing else. Nothing was rerun or retuned.
- `result.json` is kept byte for byte. The frozen analysis (`bench/attribution.py` at pin
  `68956ca`), rerun on the raw files, reproduces it exactly. The binaries match their
  recorded hashes.
- **Labels:**
  - **measured:** read from the counters, the trace or the samples;
  - **deduced:** follows from measured figures;
  - **plausible:** consistent with them but not shown;
  - **unresolved:** the archive can't say.

**The run (measured):**
- `just bench-attribution i9-10900K`, at head `b8cc070`, pin `68956ca`.
- **Setup:**
  - CPUs 1–4, siblings 11–14;
  - rustc 1.97.0;
  - 10 timeline rounds, 5 counter repetitions and 3 sample repetitions per (corpus, w).
- **Before measuring, every check passed:**
  - **C6:** 104 `mibz2` functions identical to master's.
  - **Corpora and correctness:** every corpus matches its frozen hash, and decodes to its
    frozen output at w = 1 and 4.
  - **Events:** all 18 counted without multiplexing, in 7 whole groups, at 100% coverage
    over 240 passes. Both sampled events were real, with no fallback.
- **Duration:** the timeline took 33 s; counters and samples took 104 s.

## 1. The registered outcomes, as `result.json` records them

| | Status | Share |
|---|---|---|
| H0 clock | not supported | 5.9% of the CPU-s/GB rise |
| H1 inversion | **attribution holds (R1); the memory signature is not shown (R3)** | R1 0.665 (samples), 0.775 (with copies), 0.832 (trace) |
| H2 scanner | not supported (demoted): ceiling 3,093 MB/s, against 869 MB/s | — |
| H3 head-of-line | not supported: 10.4% of worker idle time, against 25% | — |
| H4 startup and drain | not supported: 6.9%, against 50% | — |
| H5 run queue | not supported: 2.7% of decoding time | — |
| Q2's largest owner | `steady.handoff`, 34.7%. **Not one of H2–H5** | — |

## 2. Validity, item by item

| Check | Result | Effect on the figures |
|---|---|---|
| Correctness, parity | pass, all corpora | — |
| Tracing perturbation | primary: none at either w. `records` w4 timeline perturbs | `records`' w4 timeline is not interpreted |
| This analysis against the instrument | primary within 0.003, `records` 0.007. `random` 0.057 | `random`'s timeline is not interpreted (below) |
| Counter coverage | 240 of 240 passes complete, 100% enabled | — |
| Sampled events | both genuine (`evlist`) | — |
| **Sampling overhead** | **flagged: record wall 2.2× (w1) and 4.7× (w4) the counting wall** | **an artefact of the check, not perturbation** (below) |
| **Idle siblings** | **flagged: 12 at 2.3% and 14 at 2.9% busy, limit 2%** | **background, not a placement fault** (below) |

**The sampling overhead (deduced):**
- Every `perf record` process took 1.26 s, whatever the corpus or w: 1.263–1.268 s on all
  six cells. That is perf's fixed start and teardown, which includes collecting build-ids
  at the end.
- **The decode under sampling**, first sample to last, took 0.559 s at w1 and 0.243 s at
  w4. The counting passes' whole process took 0.576 s and 0.267 s (ratios 0.97 and 0.91).
- So sampling at about 2,000 samples a second per core didn't slow the decode measurably,
  and the sampled shares are not discredited by overhead.
- **Their statistical error:** 1,400–1,800 samples a run, 3 runs per cell, gives a
  standard error of about 0.7 points on a share near 0.5.
- **The check as registered** compared process walls, so it measured perf's constant.
  It stays flagged, as registered.

**The siblings (measured, then deduced):**
- Over M1–M3, the siblings were 1.1–2.9% busy.
- The twelve CPUs neither allocated nor siblings were 1.2–3.8% busy, mean 2.3%. The
  allocated CPUs were 25–30% busy.
- **The siblings were no busier than unrelated idle CPUs.** Decoder threads leaking onto a
  sibling would show as several times that.
- **The likely sources** are the desktop and the harness's own unpinned processes (`perf`'s
  writer and its reports, Python, the sudo keep-alive). The environment snapshot shows
  one long-lived process at 26% on CPU 17, a CPU outside the allocation; the same PID
  appears in the two earlier i9 runs.
- **Not an allocation violation:** every run was pinned to CPUs 1–4 by `taskset`.
- **The machine-wide memory-controller counts are not materially contaminated:**
  - at w4, the idle-paired background was 0.9% of the decode's DRAM reads and 0.5% of its
    writes;
  - at w1 it was 6.7% and 7.4%, of a much smaller traffic;
  - the analysis subtracts it in both cases.
- **The flag stays as registered.** The 2.9% sibling, with SMT sharing costing about 1.45×
  CPU per block on a shared core (`2026-10-27-four-thread-ceiling.md`), could move the
  allocated cores' CPU by well under 1%. That is far below every threshold here.

**`random`'s disagreement (measured):**
- It is in 2 of its 20 runs, both at w4. Those runs last 40 ms over 9 blocks, and the two
  definitions of the span's ends differ by 1–2 ms there.
- It is an edge effect of a tiny corpus. Under the rule, `random`'s timeline is not
  interpreted. Its counters are.

**A labelling defect in `result.json` (disclosed; no rule uses it):**
- R3's `in_flight` and `fb_full` fields are per-byte counts of
  `l1d_pend_miss.pending` and `.fb_full`. They are not misses in flight.
- The occupancy is `pending / pending_cycles`, given in §4.

## 3. Q1: why CPU-s/GB rises 32% from one worker to four

**Measured (primary, the uninstrumented decode, G0):**

| | w1 | w4 | Change |
|---|--:|--:|--:|
| CPU-s/GB | 2.707 | 3.579 | +32.2% |
| Cycles per output byte | 13.46 | 17.51 | +30.1% |
| Instructions per output byte | 32.63 | 33.06 | +1.3% |
| IPC | 2.42 | 1.89 | −22% |
| Effective clock (GHz) | 4.97 | 4.89 | −1.6%: 5.9% of the CPU-s/GB rise |

**So the decoder does almost the same work in about 30% more cycles.** The clock explains
6% of the rise; cycles per byte explain 94%.

**Where those cycles go: the two attributions, reconciled (measured).** Shares of the added
CPU-s/GB:

| Stage | Samples × counters (uninstrumented) | Trace phases (CPU time at the marks) |
|---|--:|--:|
| build | 9.9% | 11.1% |
| walk (`room_segments`) | 56.6% | — |
| `memmove` (mostly the segments' assembly, part of the walk, plus RLE1's output) | 11.0% | — |
| kernel | 9.4% | — |
| **walk phase as traced** (`WalkStart`→`InvertEnd`: the walk, its assembly's copies, and any kernel time within it) | ≈ 56.6 + 11.0 + part of 9.4 | **72.2%** |
| RLE1 and CRC | 5.8% | 7.3% |
| MTF | 1.1% | 4.0% |
| reader, scanner, feeder | 4.7% | 3.7% |
| Added total, CPU-s/GB | +0.87 | +0.84 |

- **The two methods agree:**
  - the traced build's CPU-s/GB is within 3% of the uninstrumented one at each w (2.77
    against 2.71; 3.62 against 3.58);
  - the stages line up once the copies and kernel time are placed: build within 1.2
    points; walk with its copies 67.6% against 72.2%, the rest kernel time inside the
    phase.
- **The spread between 0.665, 0.775 and 0.832 is classification, not disagreement.**
  - 0.665 is the walk and build symbols alone, a lower bound.
  - 0.775 adds the C library's copies.
  - 0.832 is the whole inverse-BWT interval.
- **The best-supported figure: the inverse BWT (build, walk and its assembly) carries
  about 78–83% of the added CPU cost.** Workers carry 96% of it (trace roles). The kernel's
  9.4% can't be split from this archive: perf resolves only addresses, without the i9's
  kernel symbols.

## 4. The memory evidence, without summing nested counters

**Measured, primary, per output byte unless stated, w1 → w4:**

| Counter | w1 | w4 | What it is |
|---|--:|--:|---|
| `stalls_total` (cycles) | 2.95 | 6.31 | cycles with no µop executing |
| ⊃ `stalls_mem_any` | 2.25 | 5.18 | …with a load outstanding at any level |
| ⊃ `stalls_l3_miss` | 0.10 | 1.13 | …with a demand load missing L3 outstanding |
| L3 misses per KB | 0.22 | 3.61 | ×16 |
| L3 hit share | 0.989 | 0.811 | |
| Data reads past L2 per KB | 1,454 | 1,420 | unchanged |
| Cycles per read past L2 (outstanding / requests) | 41 | 58 | +41%; a queueing and latency proxy, not a latency |
| Demand L1D misses in flight when ≥ 1 (`pending / pending_cycles`) | 2.19 | 2.17 | unchanged |
| Cycles with ≥ 1 demand L1D miss pending (share of cycles) | 12% | 30% | |
| Fill buffer full (events per 1,000 cycles) | 146 | 177 | +22% |
| DRAM reads per output byte (machine-wide, less the idle pair) | 1.6 B | 9.8 B | |
| DRAM writes per output byte | 0.6 B | 6.9 B | |
| DRAM traffic during the decode (reads + writes) | 0.8 GiB/s | 11.8 GiB/s | |
| Top-down: retiring / back end | 0.56 / 0.29 | 0.44 / 0.44 | |

**The three stall counters are nested** (l3_miss ⊂ mem_any ⊂ total). They are never added.
Of the 4.05 added cycles per byte:
- 3.36 (83%) are execution stalls;
- 2.93 (72%) are stalls with a load outstanding;
- 1.03 (25%) are with a demand L3 miss outstanding;
- so about 1.9 cycles per byte (47%) are memory stalls with **no** L3-miss demand load
  outstanding. Those are loads waiting on L2 or L3 hits, or on lines already in flight.

**What this establishes (deduced):**
- **Most of the added cycles are memory-related stalls.** That is 72% by the broad counter
  at constant instructions.
- **Not extra work:** instructions per byte are within 1.3%.
- **Not the clock.**
- **Not more traffic leaving L2:** requests past L2 per KB are unchanged.
- **The same requests take longer.** The cycles per request past L2 rise 41%, and 16× as
  many demand loads miss L3.

**What it doesn't establish:**
- **DRAM bandwidth saturation is unlikely.** About 11.8 GiB/s is under half the 26–27 GiB/s
  this machine sustained in `2026-10-28-workstation-scaling.md` §2. Saturation isn't
  shown.
- **Demand misses in flight don't rise** (2.2 at both w), while fill-buffer-full events
  are frequent and rise.

**Why R3's narrow metric misses most of it (plausible):**
- The walk issues each lane's next T-vector line as a software prefetch, one hop ahead.
  Its demand load then usually finds the line in L1, or in flight in a fill buffer. So the
  walk's waiting shows under `stalls_mem_any`, not as an L3-miss demand stall.
- **The L3-miss stall samples agree:** the walk holds 0% of them at w1 and 4% at w4,
  against 46% of all cycles.
- **What does hold the L3-miss stalls at w4:**
  - the copies (`memmove`), 44% of them;
  - the reader's drain, 13%;
  - RLE1 and CRC, 10%;
  - build, 11%.
- **DRAM writes** rise from 0.6 to 6.9 B per output byte. The decode's intermediate
  buffers (the T-vector, segment buffers, the output and RLE1's) stay in cache at one
  worker and go to DRAM at four. That is the mechanism `2026-10-28-workstation-scaling.md`
  §7 hypothesised, now measured at 1–4 workers.
- **R3's registered outcome stands as written:** the memory signature is not shown. Its
  metric was specified narrowly for a software-prefetched walk.

**The best-supported diagnosis for Q1:**
- Four workers' inverse-BWT working sets compete for the shared 20 MB L3: four 3.6 MB
  T-vectors, plus segment, output and RLE buffers.
- **Each worker's walk and build** then run about 30% more cycles at the same
  instructions. They wait longer on in-flight and L3-served lines.
- **Their streaming copies** miss to DRAM.
- **This is capacity and uncore-latency contention**, not DRAM bandwidth, not MLP, and not
  extra work.

**Unresolved:**
- The L2-hit / L3-hit split of the stalls: `cycle_activity.stalls_l2_miss` wasn't
  collected.
- How many demand loads hit an in-flight fill: `mem_load_retired.fb_hit` wasn't
  collected.
- Per-stage counter breakdowns: the stall samples are of one event.

## 5. Q2: the idle capacity at four workers

**Measured, primary, w4, shares of worker time (timeline mode, interpretable):**
- **Decoding:** 70.0%. Idle 29.9%.
- **Cores busy:** 3.14 of 4 (full mode).
- **Where the idle time goes (this analysis's split):**
  - handoff 11.0%;
  - scanner reading 7.4% steady, plus 0.9% at startup;
  - reader-bound slot waits 6.3%;
  - head-of-line 3.1%;
  - startup and drain together about 2%.
- **Run-queue wait:** 2.7% of decoding time.

**The instrument's own split (measured):**
- **Workers:** 19.8% of their time waiting for the jobs' lock, and 9.5% in `recv`.
- **The reader:** consuming 9.4% of the span and waiting 90.5%. For **19.2% of the span
  it waits while its next block is already ready.**
- **A queued job** waits a median 112 µs (90th percentile 2.1 ms) between `Enqueued` and a
  worker's `JobGot`.
- **A worker** waits on average 431 µs between `JobLock` and `JobRecv`.

**What the two largest categories are (deduced):**
- **Handoff (11.0% of worker time)** is real dispatch latency, not a classification
  artefact.
  - The workers share one `Mutex<Receiver>`. One waits in `recv` while the others queue on
    the lock.
  - So a burst of jobs is handed out one wake at a time, and idle workers wait with work
    queued.
  - It is small at one worker (1.6%), and grows with the number of workers.
- **The reader-bound slot waits (6.3%)** are reader wake latency, not a slow reader.
  - The reader is busy only 9.4% of the span. But it sleeps 19.2% of the span while its
    next block is ready.
  - The scanner then waits for a slot that the reader's late wake holds.
  - The bench's consumer, a 4 MiB drain into a hash, is cheap. A heavier consumer, such
    as wikidata-pq's split, would turn this into a genuine reader cost.

**Together, wake and dispatch latency hold about 17% of worker time** (11.0 + 6.3). That is
more than the scanner's reading (7.4%) and head-of-line (3.1%) together.

**Its opportunity, as an upper bound and not a prediction:**
- if that idle time became decoding, worker decoding time could rise by up to about 25%
  (17.3 / 70.0);
- but with w + 3 threads on four CPUs, and a shared-cache cost that grows as more of them
  decode, the realisable part is unknown.

**Unresolved:**
- Whether the latency is software dispatch (the mutex chain, condvar wakes) or the idle
  cores' C-state exit. The governor is `powersave`, and a 112 µs median is long for a
  futex wake alone.
- No C-state or wake-up counters were collected.

**The corpus is not too short:** startup and drain hold 6.5–6.9% of idle time (H4 not
supported). The 207 MB corpus reflects the steady state well enough for these questions.

## 6. The controls (counters only; their timelines are not interpreted)

| | CPU-s/GB, w1 → w4 | Cycles per byte | Instructions | Clock | L3 hit share |
|---|---|--:|--:|--:|---|
| `records` | 13.25 → 15.26 (+15%) | +14% | +3.9% | −0.9% | 0.93 → 0.80 |
| `random` | 12.59 → 14.60 (+16%) | +15% | +4.7% | −0.4% | 0.93 → 0.82 |

The same direction as the primary, at half the inflation.

## 7. What the data justify

**Established (measured, and deduced from measurements):**
1. **The CPU-cost inflation from one to four workers is in the inverse BWT.**
   - 78–83% of it, by two independent methods;
   - at constant instructions;
   - as memory-related stalls from shared-cache contention;
   - with the intermediate buffers spilling to DRAM.
2. **The scanner doesn't bind.** Its ceiling is 3.1 GB/s, 2.4× the 1.5× target.
3. **Neither head-of-line nor startup and drain is a dominant owner.**
4. **The largest owner of the idle capacity is wake and dispatch latency:** the handoff
   and the reader's wake, about 17% of worker time together. No registered hypothesis
   anticipated it.

**What this means for the choice of architecture:**
- **The direction is justified:** reduce the inverse BWT's shared-cache footprint and its
  traffic at four workers. This is the lever on CPU-s/GB.
- **The data favour footprint and traffic over raw MLP:** misses in flight are flat, the
  copies hold the L3-miss stalls, and DRAM writes rise twelvefold.
- **A specific mechanism is not justified yet.** The candidates are a compact T-vector,
  fewer intermediate buffers (assembly in place, fused RLE1), or N1's work reduction.
  The missing counters (§4) bear directly on that choice.
- **The second lever is separate and cheap:** dispatch and wake latency, worth up to
  about 17% of worker time.
- **Before engineering it, one narrow measurement would settle whether it is software or
  power state:** the timeline alone, rerun with C-states limited (`/dev/cpu_dma_latency`
  held at 0). That isn't authorised here and isn't run.
