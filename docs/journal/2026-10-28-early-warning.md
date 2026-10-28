# The warning we had and didn't read: resource-aware measurement on every pull request

The workstation investigation (`2026-10-28-workstation-scaling.md`, #39) found that mibz2
peaks at 4–5 workers on a 10-core machine and falls past them. This entry asks why the
development process didn't see that coming, what would have seen it, and what now runs
on every pull request so the next such trade-off is visible early.

Labels, as in the other entries: **measured** (a figure from a run, CI or local, said
which); **source** (a primary source, cited); **hypothesis** (ours, untested).

## 1. The warning was in master's data from its first bench record

- **Believed:** CI's 4 vCPUs can't see a many-core scaling problem, so it was invisible
  until the workstation.
- **Changed by (measured, master's 18 records on the bench-results branch;** `retro.py`
  in this entry's folder recomputes it). CPU per byte at 4 threads over 1 on the Wikidata
  blocks, on the same runner and in the same rounds:

  | runner | mibz2 | lbzip2 | mibz2 over lbzip2, paired |
  |---|--:|--:|--:|
  | EPYC 7763 (5 records) | 1.48–1.69× | 1.12–1.29× | 1.24–1.50× |
  | EPYC 9V74 (7) | 1.71–1.83× | 1.11–1.22× | 1.45–1.58× |
  | EPYC 9V45 (3) | 1.75–2.31× | 1.16–1.26× | 1.39–1.98× |
  | Xeon Platinum 8370C / 8573C (3) | 1.48–1.68× | 1.29–1.42× | 1.05–1.30× |

  The table in `results-table.md` gives every record with its interval.

  - Every record's interval for mibz2 over lbzip2 excludes 1.
  - The very first record (2a68359, the commit that introduced the bench) already shows 1.44×
    (1.41–1.46).
  - Across commits on one runner model the figure is stable (9V74: 1.45–1.58). So the
    walk changes (#21's 32 lanes and prefetch, #24, #32, #37) neither caused it nor made
    it worse, which agrees with the version ladder on the workstation (section 6 there).
- **Why it was missed:** the bench reported each build's wall and its CPU against the
  base (head over base), and each decoder's CPU against mibz2's. It never reported a
  build's CPU per byte against its own lower thread count.
  - The four-thread-ceiling entry did measure it ("per-block CPU time rises 1.5–1.6×
    from 2 to 4 threads", and "CPU per block 14% higher with both cores busy… a resource
    shared beyond the core").
  - It attributed most of it to SMT (correctly, 61% of the shortfall), and left the
    remainder under "Unresolved".
  - **No control decoder was put beside it.** lbzip2 on the same runners pays 1.1–1.3×,
    and a comparison against it would have separated "the runner's SMT, for any decoder"
    from "mibz2's own sensitivity". The control was in every master run, as an arm.
- **Refined by #69's first bench (measured, EPYC 9V74, 10 rounds):**

  | mibz2's amplification over 1 thread | |
  |---|--:|
  | 2 threads, one CPU on each of the 2 cores | **1.06× (1.05–1.07)** |
  | 4 threads, on both cores' SMT siblings | **1.78× (1.76–1.79)** |

  - On these runners nearly all of the 1→4 rise is **sharing a core**. Two separate
    cores cost mibz2 6%.
  - So the historical excess over lbzip2 is mostly a sensitivity to an SMT sibling: two
    32-lane walks on one core's fill buffers and L2 [inference; the research notes cite
    about 10 outstanding L1 misses a core on Skylake].
  - That is a related but distinct mechanism from the workstation's cross-core knee:
    L3 capacity, then queueing past the LLC.
  - Both say the same thing about the decoder: it buys its speed with memory-level
    parallelism, and anything that shares the resources behind that parallelism costs
    it more than it costs a slower decoder.
  - The cross-core part on 2 cores is small, which is what a 2-core runner can show.
    The first master run with lbzip2 at 2 spread threads gives the control for it.
- **Now:**
  - this was a reporting and review failure more than a missing instrument;
  - the signal is CI-visible, cheap and specific enough to put on every pull request,
    on two conditions: SMT separated from it (2 spread against 4), and a control beside it
    (master's other decoders);
  - on 2 cores it flags sensitivity to sharing a core's memory resources. It cannot by
    itself show the cross-core knee.
- **Not claimed:**
  - that lbzip2 is a clean control. It is a different program, slower per byte, and so
    issues fewer misses per second.
  - that a 1→4 amplification predicts the 10-core knee's position. It flags sensitivity;
    the workstation located the knee.

## 2. A bug in our own CPU accounting

Premonition 4 of the brief was "measure the wrong CPU time". It had happened.

- **Measured (local, and in the workstation's files):** `ab`'s per-thread split sampled
  `/proc` every 50 ms. mibz2's workers end when the jobs run out, before the reader's
  last block, so each lost up to 50 ms. The split was short by:
  - 0.10–0.16 s on CI's 4-thread Wikidata runs (about 14% of the workers);
  - 0.31–0.35 s at 10 workers on the workstation.
- **What it fed:** CI's "where mibz2's CPU goes" table, the whole-dump model's worker
  time, and my block-size comment on #39, whose worker figures were corrected there.
  Exact totals, 4 → 10 workers: ×3.50 at -9, ×2.65 at -6, ×1.44 at -3, ×1.10 at -1.
- **The process total was always right:** getrusage over every thread, with a program
  arm's children included, as Google Benchmark's `MeasureProcessCPUTime` does for code
  that spawns threads.
- **A second bug:** a thread whose name read back empty as it ended was keyed twice.
  Found locally, in a run whose split exceeded the process's total.
- **Fixed in #69:**
  - sampling every 10 ms (the sampler's own CPU is recorded: about 0.01 s a second);
  - threads keyed by id;
  - the remainder shown as unattributed (4–6%);
  - a hard check that this build's split never exceeds the total.

## 2b. A third bug, found by the A/A run: head always ran first

- **Measured (CI dispatch, the same commit as head and as base, 10 rounds, EPYC 9V74):**
  the A/A test the new report was meant to pass. It didn't.
  - 7 of 42 head-over-base rows excluded no change, against about 2 by chance.
  - Head came out 1.8% faster at 1 thread (−3.2%, −0.4%) and 2.3% faster at 2 (−4.1%,
    −0.5%), and 2.4% cheaper in CPU at 4 threads (−3.5%, −1.3%).
  - The assessment said "Better (CPU cost)" for no change at all.
- **The cause:** the harness rotated the arms' order a step each round, then reversed it
  every other round. With two arms the reversal undid the rotation. The run's log shows
  `mibz2` (head) before `mibz2@base/ab` in all 10 rounds, at every thread count. It was
  so on every pull request's bench since 1.0.0, though the comment says "the builds'
  order alternating".
  - The stage bench (bench/stages.py) did alternate.
  - wikidata-pq's bench pairs a forward and a reverse job each round, and is unaffected.
- **Fixed in #69** (`arm_order`): over 2 × arms rounds each arm runs in each position
  equally often, tested for 1–8 arms. A re-run A/A on the fixed harness is in section 6.
- **What it means for past decisions (an audit of the merged decoder PRs):**
  - the bias only ever favoured head, so every rejection stands; a rejected change was
    as bad as reported, or worse;
  - #12 (−10.7% / −16.6% at 1 / 4 threads), #21 (−13.2% / −8.3%) and #32 (−16.4% /
    −15.5%) stand by a wide margin;
  - #37's 1-thread −5.8% is corroborated by the stage bench, which alternated (walk
    −14.2%). Its 4-thread −2.7% (−4.1%, −1.3%) is within the bias;
  - #24's 4-thread −4.7% (−8.1%, −1.1%) is within reach of the bias. Its mechanism
    stands: the scanner's own CPU fell from 0.16 to 0.10 s;
  - workstation comparisons run as two arms of one `ab run` carry the same order, small
    against their 20%+ effects.
- **Lesson:** a paired harness needs an A/A run as a standing test of itself, not only a
  description of its design. This one passed review because its comment said what its
  code didn't do.

## 3. What now runs on every pull request (#69)

**What is measured:**
- 1, 2 and 4 threads.
- The 2-thread runs are pinned to one CPU on each of two cores (`ab run --spread`), so
  on CI's 2-core runners they are free of SMT. The 4-thread runs share siblings, as
  before.
- Per run:
  - the CPUs it had;
  - the threads' run-queue wait (schedstat's `run_delay`: runnable, not running);
  - the cgroup's throttling;
  - CPU PSI (an environment signal, never read as memory contention).
- `ab env` records the cores, the quota and a check that the run-queue counter counts.

**The report's resources table (Wikidata):**
- CPU per byte;
- **amplification** over 1 thread, paired by round;
- parallel efficiency;
- best thread count;
- run-queue wait;
- unattributed CPU.

**On a pull request:**
- head's amplification over base's;
- head's best thread count against base's;
- a one-line **assessment** from four comparisons declared in advance: best against
  best, CPU per byte at 4 threads, amplification, and peak RSS.

  Each moves only where its 95% interval excludes no change.

**The assessment's outcomes:**
- better, worse, a trade-off (no build dominates; weigh it for the deployment), no change,
  or inconclusive (parity, accounting or throttling);
- and **"needs a machine with more cores"** when amplification rose, memory grew, or the
  1- and 4-thread walls moved in opposite directions.

**Hard failures:** only wrong output and broken accounting fail CI. Performance flags
are advisory.

**False-positive rate:** with four comparisons at 95%, an unchanged build flags one
about one run in five. The report says so.

**Cost:** about 40 s a PR run.

**The acceptance question:** would a reviewer of an ordinary optimisation PR see a
resource or contention trade-off? The assessment line is the answer the report gives
first, before any table. *(The first real sample, from #69's own bench, is below in
section 6.)*

## 4. Layers in their own PRs

- **Footprint and queue ledger** (layer C, its PR): allocated against live against the
  walk's hot set (a model), by category, as functions of block size, threads and
  slots. Queue figures carry exact definitions:
  - reader starved (the decoder is behind);
  - scanner backpressured with the earliest block ready (the reader is behind);
  - scanner waiting on a decoding block (the workers are);
  - workers idle.

  The instrument is opt-in, behind the `trace` feature. The default build is unchanged.
  Every figure is summed from events, never sampled, and reconciled against independent
  anchors: decoded bytes against those read, and the end state empty. That is the lesson of
  section 2, applied before the ledger could repeat it.

  **Merged as #70. What it showed (local, 4 threads):**
  - **Hot is about 13% of reserved.** The hot model is 9.3 MB on average and 14.4 MB at
    most, against about 111 MB reserved and 74 MB peak RSS.
  - **Each worker reserves 13.8–17.7 MB for a 3.6 MB hot T-vector.** The T-vector doubles
    when a later block is a few rows longer, and the segment buffers ratchet to 3–4×
    their floor. Neither costs resident or hot memory.
  - **Completed output buffers wait about one decode** (ready→taken 3.35 ms, 0.35 MB on
    average).
  - **So the footprint lever is the T-vector per block × blocks in its phase, and nothing
    else.** #8's fusion can cut the phase's traffic (the last column's write and re-read) but not its hot set, and the output window can't shrink it either.
  - **The scanner's apparent reader-bound backpressure is mostly the reader's wake-up**
    (7.5 of 10.4 points are handoff). The reader is starved 83% of the span.
- **Contention fingerprint** (layer B, its PR; periodic and on demand, not on every
  PR): the decoder pinned to one core, a competitor pinned to another. The competitors:
  - none;
  - compute, the control;
  - a cache stressor at ½ of a *measured* effective LLC;
  - a DRAM streamer;
  - a twin decoder.

  Stressors are calibrated and validated per machine. Rounds are interleaved and
  paired, and the report says "inconclusive" when isolation fails. It detects
  sensitivity and never names a cause.

  **#71. What it showed (CI, EPYC 9V74, victim on CPU 0, competitor on CPU 2):**
  - **Calibration:** C_eff came out at 16.8 MB on one runner and 12.8 MB on another, from
    a pointer chase rather than sysfs. Every stressor passed its validation. The bubble
    stayed in cache (1.96 ns per access against 1.65 in cache and 11.97 out). The streamer
    reached 37–40 GB/s against 86–96 GB/s resident.
  - **Head on the PR run:** stream 1.69× slowdown (1.64× over compute); cache 1.07× over
    compute; twin 1.10× over compute, with two copies giving 1.76× one copy's
    throughput.
  - **Retrospective** (one dispatch, every release interleaved, 8 rounds):

    | | v0.1.1 | v0.1.2 | v0.1.3 | v0.1.4 | v0.1.5 | head |
    |---|--:|--:|--:|--:|--:|--:|
    | solo MB/s | 184 | 229 | 273 | 313 | 348 | 349 |
    | stream slowdown | 1.30× | 1.39× | 1.45× | 1.48× | 1.47× | 1.47× |
    | against head's, paired | 0.89× (0.86–0.92) | 0.95× (0.93–0.98) | 0.99× | 1.01× | 1.00× | |

    - **It would have flagged** a rise in sensitivity to a streaming co-runner at v0.1.2
      (the flagged walk) and v0.1.3 (32 lanes with prefetch), the changes the
      workstation's knee is attributed to. The figure has been flat since.
    - **The confound it can't remove:** each release is also faster solo. With the same
      stall time per byte, a faster decoder spends a larger share of its time stalled, so
      its ratio rises. "More memory-level parallelism" and "less compute per byte" read
      alike here.
    - **The cache and twin arms didn't separate the versions** on that runner (1.00–1.02×
      over compute).
    - **The A/A pair (head against its base) flagged nothing.**
  - **Between runners the figures move** (the stream arm 1.64× on one and 1.47× on
    another; the cache arm sensitive on one, under the floor on the other). Compare
    builds within one interleaved run, never across runs.

## 5. What each early signal could and could not have told us

| signal | runs on CI | would it have flagged this? |
|---|---|---|
| CPU per byte, 1 → 4 threads | yes | **A sensitivity, from the first record**, once beside a control decoder (section 1): mostly to sharing a core (SMT), a cousin of the cross-core knee, not the knee itself. Alone, SMT explained it away. |
| 1 / 2 / 4 with 2 spread across cores | yes | Not recorded historically. On #69's runs, 2 spread cores cost mibz2 6% (section 6): the cross-core part is small on 2 cores. Whether it was larger for older versions is open, and #69's report now tracks it per PR. |
| Co-runner fingerprint | yes (2 cores) | **Yes, the stream arm**, at v0.1.2 and v0.1.3 (section 4). It is confounded with solo speed, so it is a question to ask, not a verdict. The cache and twin arms: no. |
| Footprint ledger | yes | It doesn't detect; it prices. 3.6 MB of hot T-vector per worker against an unknown, shared L3 says "measure scaling" before any figure moves. |
| The knee's position (4–5 of 10) | **no** | Needs ≥ 6 physical cores: the workstation, or a larger runner. |
| Which shared resource (L3 capacity, then queueing past the LLC) | **no** | Needs PMU counters (#39 sections 7–8) or controlled interventions (block size, section 9). |
| Whether wikidata-pq's release is hurt | **no** | Needs the release-shaped split-stage comparison (#65). |

## 6. The first sample

#69's own bench (EPYC 9V74, 2 cores × 2 siblings, 10 rounds) is an A/A for the decoder:
head and base differ only in the harness.

- **Assessment:** "No change detected on the four measures". That is right.
- **The resources table (head):**
  - 356 → 612 → 677 MB/s at 1 / 2 / 4 threads;
  - CPU 2.82 → 2.97 → 4.98 ns/B;
  - amplification 1.06× and 1.76×;
  - efficiency 85% and 48%.
- **Head's amplification over base's:** +0.1% (−0.7%, +0.9%) at 2 threads and −0.9%
  (−2.4%, +0.6%) at 4. So an amplification change of about ±1–2% is resolvable on these
  runners in 10 rounds.
- **What the run caught in the harness itself:**
  - the 10 ms sampler, as first written, cost the 1-thread runs +1.0% (+0.4%, +1.7%) in
    wall and CPU. It shared the pinned CPU and was counted in the run;
  - a follow-up commit moves it off the run's pinned CPUs and subtracts its own CPU
    (RUSAGE_THREAD).

  The instrument was checked by the same paired comparison it reports, as instruments
  should be.
- **The sampler's own cost, measured** (`base_sample_ms=0`: the same commit sampled
  against unsampled, dispatched):
  - **Wikidata within noise:** +0.6%, +0.7%, +1.2% (−0.1%, +2.4%) at 1/2/4 threads, CPU
    at 4 threads +0.2%.
  - **12 ms single-threaded runs were not:** +4.7–17% in wall (`runs`, `onebyte`),
    plausibly because reading another task's /proc entries locks that task's run queue
    [inference].
  - **So a 1-thread in-process run now has no sampler at all.** Its split is its own
    RUSAGE_THREAD and its run-queue wait its own schedstat.
  - **Parallel runs keep a sampler** that leaves the run's pinned CPUs before the clock
    starts, and whose own CPU is subtracted from the run's.
  - **On 2-core runners, the only CPUs left are the workers' SMT siblings:** a residual
    within noise on the decision corpus.
- **The acceptance test, A/A on CI** (one commit dispatched as head and as base, 10
  rounds):

  | | head-first harness (e89921a) | fixed order (1e2c5fb) |
  |---|---|---|
  | rows excluding no change | 7 of 42 | **0 of 42** |
  | wikidata wall, 1 / 2 / 4 threads | −1.8%, −2.3%, −0.8% | −0.6%, −0.0%, +0.4% |
  | assessment | "Better (CPU cost)" | "No change detected … at this run's precision" |

  The intervals also narrowed (wikidata CPU at 4 threads ±0.7%, against ±1.1%). The
  order effect had been adding spread as well as bias.
- **On a 9V45 runner** (#69's bench after the fix): mibz2's best is 2 threads on separate
  cores, 826 MB/s. 4 threads on SMT siblings give 808 MB/s at 2.11× the CPU per byte.
  Even on CI, the best worker count is the physical core count (#65).
- **Run-queue wait:** 0.56 threads at 2 (two workers, the scanner and the reader on two
  CPUs) and 0.88 at 4. That is the decoder's shape, recorded so that a co-runner
  comparison never mistakes it for interference.

## 7. Reading: what we used, and what we took from it

**Practitioner methods** (each used for one decision in #69 or the fingerprint):
- **Google Benchmark's user guide**
  ([docs](https://github.com/google/benchmark/blob/main/docs/user_guide.md)) [source]:
  - `MeasureProcessCPUTime()` against the main thread's CPU, for code that spawns its
    own threads. Ours is process-wide, via getrusage;
  - `UseRealTime()`;
  - `ThreadRange` sweeps;
  - `--benchmark_enable_random_interleaving`, which reduces run-to-run variance. Ours
    rotates arms each round and pairs by round.
- **Brendan Gregg, the USE method** ([page](https://www.brendangregg.com/usemethod.html))
  [source]: utilization, saturation and errors for each resource. In the ledger, each
  queue gets a utilization (workers decoding) and a saturation reading (scanner
  backpressured, reader starved), never one without the other.
- **Gregg, thread state analysis** ([page](https://www.brendangregg.com/tsamethod.html))
  [source]: executing, runnable and sleeping are different states. CPU time counts a
  memory stall as executing, so run-queue wait is recorded separately (schedstat) before
  a slowdown is read as interference.
- **Gregg's Linux performance checklist**
  ([USE on Linux](https://www.brendangregg.com/USEmethod/use-linux.html)) [source]:
  `/proc/PID/schedstat`'s second field, `run_delay`, for CPU saturation per thread.
  Ours is checked by oversubscribed spinners before it is trusted.
- **Gregg's benchmarking questions**
  ([methodology](https://www.brendangregg.com/methodology.html), active benchmarking)
  [source]: did it even happen, does it reproduce, does it matter? The assessment's
  inconclusive state covers "did it happen". Its false-positive note covers "does it
  reproduce".

**Measurement and interference research** (research notes in this entry's folder, each
claim marked source or inference):
- **CPI²** (Zhang et al., EuroSys 2013) [source]: CPI against a job's own distribution
  as the interference signal, with a mean + 2σ outlier rule and samples ignored below
  0.25 CPU-s/s. [Inference] At constant clock and instructions per byte, our CPU
  amplification is that signal.
- **Bubble-Up / Bubble-Flux, iBench, Heracles' antagonists, Cache Pirating / Bandwidth
  Bandit** [source]: the stressor designs and the validation the fingerprint follows.
  The Pirate checks it holds the cache it claims, Bubble-Flux pairs on and off to
  cancel drift, and iBench separates "tolerated" from "caused".
- **Abedi & Brecht** (ICPE 2017) [source]: naive comparisons call two identical systems
  38% different. Randomized interleaved trials are the defence, which the bench already
  uses.
- **Laaber et al.** (EMSE 2019) and CodSpeed's measured coefficients of variation
  (2.66% hosted, 0.56% bare metal) [source]: why paired intervals, not fixed percentage
  thresholds.
- **On GitHub's runners** [source]: perf counters are unavailable, and the L3 size that
  sysfs and CPUID report is the host socket's. The fingerprint therefore calibrates its
  cache size by timing.
- **Gunther's USL** [source]: at least six points, and subset fits miss peaks. We don't
  extrapolate a knee from 1/2/4.

**Leads recorded, not pursued tonight:**
- **libsais 2.12.0's multi-threaded inverse of one bzip2 block** [source, released
  2026-10-18]: fewer live T-vectors without idle cores (#65).
- **A deterministic CI proxy:** cachegrind's or drcachesim's last-level misses per
  output MiB at a fixed LL size [inference, from their manuals].
- **The second knee:** DDR4's four-activate window may cap random reads near the
  20-worker throughput [inference: a consistency check, not a test].

## Open, needing a many-core machine or wikidata-pq's pipeline

- **Does a rise in CI's 2-spread amplification predict a lower knee on 10 cores?** One
  test: build two variants that differ only in per-worker footprint, run both under
  #69's report and on the workstation's `bench-scaling`, and compare.
- **Is lbzip2's 1→4 amplification a stable runner baseline?** It varies by runner model
  (1.11–1.42×). It is a control, not a constant.
- **Whether #65's cap helps the release, in the split stage.**
- **Whether the fingerprint's sensitivity on 2 cores orders the decoders the same way
  the workstation's knee does.**

## Open, answerable on CI

- **The fingerprint's confound.** Does the stream arm's rise at v0.1.2 and v0.1.3 come
  from more memory-level parallelism, or only from less compute per byte?
  - **A test:** a victim with a fixed, known amount of compute added per block (the same
    misses, more CPU per byte). If its stream ratio falls toward v0.1.1's, the ratio is
    tracking speed, not memory behaviour.
  - Dispatch-only, at no cost to the PR bench.
- **lbzip2's and pbz2's 2-spread amplification,** from the first master run after #69. It
  is the control for mibz2's 1.06–1.14×.
- **A twin that loops inside one process,** if the decoder gets much faster: the twin's
  CPU is 0.93–0.98 against a 0.9 screen.

## What the tools now say, in one place

For an ordinary decoder PR, a reviewer sees, in order:
1. **The decoder table,** head over base, now with the order alternating.
2. **The resources assessment:** better, worse, a trade-off, no change at this run's
   precision, or inconclusive. It also says when a change needs a machine with more cores.
3. **The resources table:** CPU per byte, amplification, efficiency and placement.

**Periodically** (a dispatch, or master): the fingerprint's slowdowns beside each
co-runner, and any build's stream sensitivity against head's.

**On demand** (`just bench-ledger`, the occupancy instrument): the footprint and the
queues, from events, reconciled.

**Each layer's findings tonight:**
- The resources report found the order bias (A/A), the sampler's cost (sampled against
  unsampled) and SMT's share of the amplification (2 spread against 4).
- The fingerprint found that the historical rise came at v0.1.2 and v0.1.3, with a
  confound named.
- The ledger found the T-vector is the only footprint lever.

None of them can locate a many-core knee. That still needs the workstation.
