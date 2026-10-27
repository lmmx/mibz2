# The four-thread ceiling: two cores, not four (#39, #53, #54)

The whole decoder gained only 1.5–1.9× from four workers on CI's 4-vCPU runners. This
entry records how the explanation changed as the occupancy instrument (#53) and its
placement arms (#54) measured it.

Labels:
- **measured:** a CI figure, with its run;
- **source:** a primary source, cited;
- **hypothesis:** ours, untested.

## 1. The window was the leading suspect; the trace demoted it

- **Believed:** the admission window (`2 × threads` slots, released in order) starved the
  workers: decode head-of-line blocking or output backpressure.
- **Changed by (measured, #53, two runs on an EPYC 9V74):**
  - at 4 threads, workers decode 80% of the time and are idle 18–19%;
  - only 7–8% of the workers' time is idle during slot waits, an upper bound on what a
    wider window could recover;
  - the largest idle share, 10–11%, falls while the serial scanner reads a block.
- **Now:** the 8/16/32-slot counterfactual is low on the list. The trace attributes each
  instant to one state, so overlapping waits are never added twice.

## 2. The cost is on a CPU, not waiting for one

- **Believed:** scheduling delay on shared runners could be inflating block times.
- **Changed by (measured):**
  - per-block CPU time rises 1.5–1.6× from 2 to 4 threads;
  - run-queue wait (`schedstat`, validated by oversubscription on every run) is about 6%
    of a block's time;
  - steal is 0%.
- **Now:** CPU time counts memory stalls as running, so this shows an on-CPU cost, not
  which resource causes it.

## 3. The 4 vCPUs are 2 cores (measured)

- **Seen:** the instrument records the topology the guest is given. Both runner models
  seen (EPYC 9V74 and 7763) report 4 CPUs on **2 physical cores**, with SMT siblings
  0-1 | 2-3. The curve bends exactly there:
  - 1.81–1.87× at 2 threads;
  - +7% more at 4.
- **Caveat:** the guest's topology may not be the host's. What follows is shown for the
  topology reported, as placement effects.

## 4. The pre-registered placement test: SMT is a large cost (measured, #54)

- **The test:** 2 threads, the whole process pinned (scanner, feeder and reader with the
  workers), 12 rounds paired:
  - `packed` puts both on one core's siblings, `[0,1]`;
  - `spread` puts them on two cores, `[0,2]`.
- **The verdict, off-mode throughput:** spread over packed was:
  - **+40.5% (+37.5%, +43.6%)** on an EPYC 9V74 ([run](https://github.com/lmmx/mibz2/actions/runs/40594623915));
  - **+39.6% (+37.5%, +41.6%)** on an EPYC 7763 ([run](https://github.com/lmmx/mibz2/actions/runs/40594990404)).

  Every round agreed.
- **The explanation:**
  - per-block CPU is about 1.43–1.46× higher on siblings;
  - workers decode the same share of the time in both placements;
  - run-queue wait is the same.
- **Mechanism (source, a hypothesis for this walker):** AMD's Software Optimization Guide
  for Family 19h (56665) lists how SMT siblings share a core:
  - L1 and L2 caches are competitively shared;
  - the load queue and memory request buffers are watermarked;
  - the store and retirement queues are statically partitioned.

  The 32-lane room walk exists to keep many independent loads in flight. A sibling
  therefore caps the memory-level parallelism each walker can expose, not only its cache.
  The placement test can't separate this from cache or execution-unit sharing, since all
  of them change together.

## 5. SMT is most of the ceiling, not all of it (measured)

- **Pre-registered:** if SMT were the whole story, 4 threads (two sibling pairs) would give
  about 2× packed's 2 threads.
- **Measured (EPYC 7763, one run):**
  - **1.62× (1.59, 1.65)**;
  - the A/A check (the same four CPUs in two orders) was +1.5% (−0.5%, +3.5%).
- **Split of the shortfall** (arithmetic on the verdicts, not a counter): 4 threads reach
  0.58 of two independent cores per pair.
  - SMT sharing gives a factor of 0.72, 61% of the shortfall on a log scale.
  - Full occupancy gives 0.81, the other 39%.
- **What the trace says about the 19% (explanation only;** the packed 2-thread arm's traced
  modes perturbed it by about 1%, so its figures are indicative):
  - CPU per block is 14% higher with both cores busy, at the same two workers a core.
    That points to a resource shared beyond the core (L3, memory, the host); the clocks
    can't say which.
  - Workers decode 80% instead of 86% of the time, most of the added idle falling while
    the scanner reads (11% against 6%).
  - Together: 0.88 × 0.93 ≈ 0.82.

## Outcome, and what is open

- **Established:** spreading two decoder threads over separate cores, rather than packing
  them onto SMT siblings, raises end-to-end throughput by about 40%. Replicated on two
  EPYC models (+40.5% and +39.6%).
- **Falsified:** that SMT sharing alone explains the four-thread ceiling. The registered
  prediction (about 2× from one packed core to two) measured 1.62×.
- **Unresolved:** the remaining loss coincides with higher CPU per block and with workers
  idle while the scanner reads. Neither is shown to be a bottleneck that can be
  recovered.
- **Settled for these runners:** the "1.5–1.9× from four workers" is mostly the runners'
  topology. Four vCPUs are two cores' worth. No decoder change follows from that.
- **Open, for a machine with more cores** (`just bench-occupancy` on the 20-core
  workstation):
  - whether the full-occupancy remainder grows with core count;
  - if it does, whether it is the serial scanner (rising idle while it reads; lead #10)
    or a shared resource (CPU per block rising with little idle; #42's pre-registered
    contention test).
- **Open:** the EPYC 9V45's weaker 1.50×. No run has landed on one yet.
- **Not claimed:**
  - that the scanner or a shared resource causes the remainder: each needs its own A/B.
    A symptom in a trace doesn't by itself reopen #10 (speculative Huffman in the
    workers) or #42 (the move structure under contention); a paired experiment does;
  - that SMT is bad for mibz2: the fourth worker still adds throughput.
- **Lesson:** a speedup's denominator should be physical cores, read from the machine,
  not its advertised vCPUs. The instrument now records both.
