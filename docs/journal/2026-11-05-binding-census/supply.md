#### FRONT supply
MACHINE: CPU model for every run used:
- **Dispatched occupancy run 42009406264** (master 4bfaf7cb, src identical to d1fd4405, 6 rounds): AMD EPYC 9V74.
- **Occupancy run 41976148952** (c90b3625, #180 head: window 16, the walk before #181): AMD EPYC 7763.
- **Occupancy runs on #183's branch** (fused post-walk, closed; supply code unchanged): 41986885416 (beef4e8f), Xeon Platinum 8370C; 41988588220 (9bee6b40), EPYC 7763.
- **History occupancy run 41941330659** (ee12bb9e, window 8): Xeon 6975P-C.
- **Bench runs:**
  - master 41980640481 (d1fd4405, 10 rounds): Xeon Platinum 8573C;
  - 41976148934, last attempt (c90b3625 vs 17daef23): Xeon 8370C;
  - 41977512992 artifact (eebe33f vs c9433318, both arms at window 8): Xeon 8573C;
  - 41979947814 (e3ea69fd vs eb5363c2): EPYC 9V45;
  - 41986885355 artifact (beef4e8f vs base 0769b52f, the production decoder): EPYC 9V45.
- **Walk sweeps:** 41989592674 on EPYC 9V45, 41989596336 on EPYC 9V74.
- **Every runner** has 4 CPUs on 2 cores with SMT, and an L3 shared across all 4 CPUs.

Measurement shapes:
- **Bench:** 1t is the sequential BzReader::new (examples/ab.rs:432), so the 1t rows have no supply term at all. 2t is pinned to cpus 0,2, one vCPU per core. 4t uses all four CPUs. Wall is measured in-process (ab.rs:586-590), so it includes thread spawn, start-up, the tail and join. The reported CPU excludes the sampler's 6 ms.
- **Occupancy:** 1t is pinned to one CPU; 2t and 4t are unpinned.

No local timing or build was done. Artifacts (2.7 MB) are k

- [OBSERVED] Worker time not decoding splits two ways: asleep, which equals idle vCPU (the only part supply work can recover), and runnable but queued, which equals the support threads' CPU and is not supply.
  CLAIM: At 4 threads the 4 vCPUs are saturated. Of the 6-8% of worker time that is not decoding plus the 13-14% that decode_block spends in the run queue, only 3.3-4.4% is truly unsupplied (the worker asleep). The rest is the scanner, reader and feeder holding the vCPUs. The worker-time identity closes in every window-16 run: worker run-queue equals non-worker CPU, and worker asleep equals idle vCPU time. So the window journal's 'workers decode 93%' is not 7% recoverable.
  EVID: Occupancy, full mode, 4t. Worker time split = decode CPU / worker run-queue / asleep; then non-worker CPU and idle vCPU as shares of capacity.
- r42009406264 (9V74, d1fd4405 code): 81.6% / 13.5% / 4.4%. Worker run-queue 0.1350 s against non-worker CPU 0.1346 s; asleep 0.0445 against idle vCPU 0.0448 vCPU-s. Non-worker CPU 13.4%, idle 4.5% of capacity.
- r41976148952 (7763): 81.3% / 13.9% / 4.3%. Run-queue 0.166 against non-worker 0.174; asleep 0.052 against idle 0.044.
- r41986885416 (8370C): 83.3% / 13.0% / 3.3%. Run-queue 0.198 against 0.208; asleep 0.051 against idle 0.041.
- r41988588220 (
  SHARE: Idle vCPU is 2.7-4.5% of 4t capacity; on the 9V45 bench it is 6.8-8.4%.
  RECOV: No supply-only change can recover more than the idle vCPU share. The hi bound is that share; the lo bound applies the SMT discount, lo = 2·x·m/(1+m), where x is the idle share and m is the marginal value of an SMT sibling (from the same run's 2t-pinned to 4t gain).
- Xeon 8573C: hi 2.8% (net of other processes) to 3.9% (gross); m = 0.244; lo 1.1-1.5%.
- Xeon 8370C: hi 1.8-2.9%; m = 0.159; lo 0.5-0.8%.
- EPYC 7763: hi 3.7-5.0%; m not measured; lo about 0.9-2.0%.
- EPYC 9V74: hi 3.9-4.5%; m not measured; lo about 0.9-1.9%.
- EPYC 9V45: hi 6.8-8.4%; m = 0.126; lo 1.5-1.9%.

This bound covers every item below combined.
  FALSIFIER: Either of these falsifies the claim:
- a change that alters no CPU work but cuts 4t wall by more than the run's idle vCPU share;
- an occupancy run where worker run-queue and non-worker CPU differ by more than 25%.

- [INFERRED] Scanner, reader and feeder CPU → the workers' run-queue inside decode_block → 4t wall.
  CLAIM: The largest lever behind the hidden 13-14% is the support threads' CPU, above all the scanner's. Because the box is CPU-bound, every CPU-second the scanner, reader or feeder spends comes out of the workers' vCPU time. 4t wall ≈ total CPU / (4 × 0.96).
  EVID: The derivation rests on the identity in the first claim and on 4t wall equalling (W + S + R) / 3.85. Check on the master bench (Xeon 8573C, r41980640481): workers 1.065 s, scanner 0.135 s, main/reader 0.030 s, unattributed 0.078 s; 1.308 / 3.846 = 0.340 s, against 0.3426 s observed.
- The scanner is 10.3% of process CPU at 4t: 0.135 s.
- At 1t its cost is 0.088 s (0.4185 ns/B × 209.7 MB), so co-residency and SMT inflate it 1.53×.
- The other benches agree: scanner 0.13-0.14 s on Xeon 8370C (r41976148934 head) and Xeon 8573C (r41977512992 artifact); 0.06-0.07 s on EPYC 9V45.
  SHARE: Non-worker CPU: 13.4-14.5% of 4t capacity (occupancy); the scanner alone is about 10%.
  RECOV: About 1.0% of 4t wall for each 10% cut in scanner CPU (0.0135 s / 3.85 = 3.5 ms of 343 ms).
- The reader's copy and the harness's hash (main thread, 0.03 s) bound another 0.6-0.9%.
- The feeder's 4 MiB allocate-and-fault per chunk and the scanner window's re-copy (parallel.rs:320) are worth ≤0.3-0.5% each.
- The term is 13.4-14.5% of capacity in all. Its floor is the scanner's Huffman decode, which is the scanner front's question, not this one's.
  FALSIFIER: Remove a known amount of reader CPU, for example drain without the hash. If 4t wall does not fall by about ΔCPU/3.85 within noise, the capacity model is wrong and something else binds.

- [OBSERVED] Feeder's first 4 MiB read and page faults, plus the 2 MB lookahead → scanner in its input wait → all four workers in recv.
  CLAIM: Start-up is a fixed wait of 1.8-3.4 ms per decode in which every worker idles. The scanner's first view must hold LOOKAHEAD = 2 MB, and the feeder delivers only whole 4 MiB chunks into a freshly allocated buffer, so no block can be read before about 4 MiB are read and faulted in.
  EVID: s.input (median, timeline, at 2t / 4t):
- 1.80 / 1.87 ms on Xeon 8370C (r41986885416);
- 1.83 / 1.88 ms on EPYC 9V74 (r42009406264);
- 3.34-3.41 ms on EPYC 7763 (r41976148952, r41988588220).

s.input_workers_busy is about 0 (5e-6), so input waits happen only while no worker has work: at the start.

Code: src/read.rs:13 (CHUNK = 4 MiB) and :116 (vec![0u8; CHUNK] each chunk); src/stream.rs:52 and :123 (view(base, 2 MB)); src/parallel.rs:310 (waits until from + want bytes are held); src/lib.rs:381 (decode_parallel's 4 MiB to_vec chunks).

The ramp that follows (the first chunk copied into the win
  SHARE: 0.5-1.1% of the CI 4t wall.
  RECOV: 0.5-1.1% of the CI 4t wall: 1.9-3.4 ms of 249-380 ms, recovered at near full value because whole cores idle. A small first read with a small first lookahead could add up to about 0.5% from the ramp.
- At 2t: 0.5-1.5% of 2t wall.
- At 1t: 0, since the work is serial.
- At release scale: about 0. It is a few ms per decode against 3000 s modelled.
  FALSIFIER: An arm with a 128-256 KiB first chunk and first lookahead. The claim fails if s.input stays above 0.5 ms, or if 4t off-mode wall does not fall by at least 1.5 ms.

- [INFERRED] Scanner preempted, the job queue drains, a worker waits in recv; or a job is queued but the woken worker is not yet running.
  CLAIM: The steady-state supply losses (the job handoff, and dips while the scanner reads) are all that remains at release scale. The 4t dips come from the scanner waiting in the run queue, not from its speed: unpinned at 2t, where it has a vCPU of its own, they almost vanish.
  EVID: Observed parts:
- Worker idle with a job queued (the handoff): 0.5-2.4% of worker time at 4t, about 22-56 µs a block (timeline/full: 1.7/0.8 on 7763, 0.7/0.5 on 8370C, 2.4/1.2 on 9V74).
- Worker idle while the scanner reads: 1.7-4.2% at 4t, against 0.6-1.1% at 2t unpinned (the 2t figure is about the start-up ramp).
- The scanner's wall-clock time reading: 123-211 ms at 4t, against 0.13-0.14 s of its own CPU (bench).

Derived: steady idle vCPU = total idle − start-up (about 3.5 vCPUs × s.input) − ramp − tail ≈ 1.5-2.8% of capacity.
  SHARE: Steady idle vCPU: 1.5-2.8% of 4t capacity.
  RECOV: hi 1.5-2.8% (all of the steady idle); lo 0.2-0.6% (SMT-discounted). It is the only part of this front that survives at release scale. Split condition variables would win 0.2-0.4% of CPU and are already rejected (docs/journal/2026-11-05-split-condvars.md:49-56).
  FALSIFIER: A supply-only A/B (a prefetched job per worker, or spin-then-park in recv) that beats 2.8% of 4t wall on a Xeon, 7763 or 9V74 runner. Or an occupancy run where the scanner keeps its own vCPU at 4t, as when pinned apart, and worker idle while it reads stays above 2%.

- [OBSERVED] Window full → scanner in a slot wait → workers idle.
  CLAIM: The window and the ordered output are exhausted at window 16. Slot waits and reader backpressure starve workers for well under 1% of their time. The reader is starved for 88-91% of the span and does not bind.
  EVID: 4t, window 16:
- worker idle during slot waits: 0.13-0.62% of worker time (1.9-3.1 ms of worker time);
- worker idle in a slot wait while the earliest untaken block was ready (reader backpressure): 0.08-0.34%;
- reader waiting 88-91% of the span;
- a ready block waits 2.4-3.7 ms on average before it is taken, mostly head of line: 0.44-0.50 of 0.78-0.93 MB ready-not-taken. That costs memory, not time.

History, at window 8 (r41941330659, Xeon 6975P-C): slot-wait idle was 9.4-10%, and workers decoded 75%. The 8.1-10.3% 4t gain #180 bought used that up (docs/journal/2026-11-05-window.md:100-104, 
  SHARE: 0.13-0.62% of worker time.
  RECOV: ≤0.1-0.3% of 4t wall, already counted within the steady-state bound.
  FALSIFIER: Occupancy with --slots 24 or 32 that shows w.idle_during_slot_wait_s above 0.01 worker-seconds, or a 4t wall gain above 0.5%.

- [INFERRED] The last blocks decode on 1-3 workers while the others have ended.
  CLAIM: The tail, while fewer blocks than workers remain plus the reader's catch-up, is a fixed per-decode cost of about 2-2.6 ms of wall-equivalent at 4t.
  EVID: Worker 'outside' time (before ThreadStart and after ThreadEnd): 7.9-10.5 ms of worker time at 4t, against 1.0-3.1 ms at 2t (r42009406264, r41976148952, r41986885416, r41988588220). That fits (n−1)/2 last blocks of 2.17-3.35 ms each, plus about 1 ms of the reader copying and hashing ready blocks.
  SHARE: 0.6-0.9% of worker time.
  RECOV: ≤0.3-0.8% of the CI 4t wall; about 0 at release scale. Block granularity is the floor.
  FALSIFIER: An occupancy timeline showing more than 3 ms of the 4t span after the last Issued block with every worker idle.

- [INFERRED] Faster workers → per-block handoff latency and the fixed start-up and tail grow in share → then the scanner's serial chain (Huffman decode, run-queued) binds.
  CLAIM: The scanner is not on the 4-thread critical path today. It would bind only if worker CPU fell 2.0-2.5×, which a faster walk alone cannot do short of about 5×. Before that, the supply idle grows roughly as 1 / block time, so the constraint moves toward supply and support CPU as the workers speed up.
  EVID: Observed:
- The scanner reads 49-57% of the 4t span and waits for a slot 40-47%: headroom of span over reading 1.75-2.03× (four runs), 2.7-2.9× at 2t.

Derived:
- Capacity model with the scanner's contended chain held fixed: (W/k + S) / 3.85 = R_s gives k = 2.05 (7763), 2.07 (8370C) and 2.45 (9V74).
- The bench's own model says scanner_binds_from = 7.86 workers (r41980640481), that is k = 1.97 at 4 workers.
- The walk is 1.7736 of 3.30 ns/B of worker stage time at 1t, f = 0.54, and likely 0.54-0.65 at 4t. So k = 1/(1 − f + f/s): s = 2 gives 1.37-1.48, and s → ∞ gives 2.2-2.9.
- Idle share × bl
  SHARE: Scanner headroom: 1.75-2.03×.
  RECOV: 0 today. With workers 2× faster, the predicted supply idle is 5-9% of 4t capacity, and start-up becomes 1-2% of the CI wall.
  FALSIFIER: The next merged worker speedup's bench should show 4t idle vCPU rising about as 1 / (worker CPU per block). An occupancy run after it with scanner slot-wait below 20% of the span would confirm the scanner as the next bind.

- [INFERRED] bench/report.py:787-834 (the model's overlap and wall_n_fixed).
  CLAIM: At 4t, the bench's overlap gap (wall / max(scan, workers/4) = 1.297 on Xeon 8573C) is about 90% support-thread CPU on shared vCPUs, which grows with the bytes decoded, and at most about 5 ms of fixed start-up and tail. So for the release ETA at 4t, wall_n (2998 s) is the right reading, and wall_n_fixed (2313 s) under-reads by about 20-25%.
  EVID: r41980640481 models: gap = 0.3426 − 0.266 = 0.077 s.
- The capacity model gives 0.243 / 3.846 + 1.065 × (1/3.846 − 1/4) = 0.074 s.
- The fixed parts are start-up and tail: about 2-3 ms plus 2 ms, from occupancy on other CPU models. This runner has no occupancy run.
  SHARE: The gap is 22% of the 4t wall.
  RECOV: Not a speed gain. It decides which release figure to quote.
  FALSIFIER: A doubled-corpus bench (the same blocks twice). If 4t wall does not grow by about 2× minus the fixed part (5 ms or less), the gap is not proportional.

- [OBSERVED] The same identity as at 4t on 2 vCPUs.
  CLAIM: At 1 and 2 threads there is next to nothing to recover from supply.
  EVID: - 1t: the bench's arm is sequential; the parallel decoder pinned to 1 CPU runs at CPU/wall 1.00 (all four occupancy runs).
- 2t, bench pinned to cpus 0,2: CPU/wall 1.96-1.99 (r41980640481: 2.0% idle; r41976148934: 0.5%; r41986885355: 2.3%), with run-queue 0.17-0.33 thread-seconds. The scanner is 10% of 2t CPU (0.085-0.09 s).
- 2t unpinned (occupancy): workers decode 97-98%.
  SHARE: Not applicable (1t and 2t).
  RECOV: Recoverable at 2t: ≤0.5-2.3% of 2t wall, mostly start-up (2-3 ms). At 1t: 0.
  FALSIFIER: An idle share above 3% in a pinned 2t bench run on any runner.

- NOGO Split condition variables, with targeted notifies: of=docs/journal/2026-11-05-split-condvars.md:43-56 (rejected on a wake count)
  changed: None at d1fd4405. There is still one Condvar with notify_all per block (src/parallel.rs:264, :471) and window 16. The only shift is per-block time: on the 9V74 a block takes 2.17 ms against about 2.6 ms before.
  now: Still no-go: about one wasted wake a block, 0.2-0.4% of CPU, well inside the 4t rows' ±2-5% noise. If blocks got 2× faster it would still be ≤0.8%.
  test: Apply cvcount.patch on a CI runner at 4t. Multiply wasted wakes by the measured futex wake cost; reopen only if that exceeds 1% of 4t CPU.
  score: low

- NOGO A window wider than 4 × threads (e.g. 6 or 8 a thread), or a byte-budgeted window: of=docs/journal/2026-11-05-window.md:91-94 (open after #180); split-condvars.md:57-60
  changed: Measured since: at window 16, worker idle during slot waits is 0.13-0.62% of worker time in four runs (7763 ×2, 8370C, 9V74).
  now: No-go for time: ≤0.1-0.3% of 4t wall, at a cost of 2.7 MB reserved per extra slot. A byte budget could save memory, but buys no time.
  test: Occupancy with --slots 24: reopen if w.idle_during_slot_wait_s is above 0.01 worker-seconds, or if off-mode 4t wall improves by more than 0.5%.
  score: low

- NOGO Self-scanning workers: parallel boundary search in place of the serial scanner: of=split-condvars.md:57-60 (Astra's ranks 4 and 5, bounded by the slack left at window 16)
  changed: Sharpened. The scanner's 4t effect is CPU competition, not supply (headroom 1.75-2.03×). Its reading dips are about 1.2-3.7% of worker time, and the box is saturated.
  now: No-go on CI today. Boundary search and duplicate Huffman work would land on saturated vCPUs, so the predicted change is between −0.5% and +1%. It becomes relevant once worker CPU falls about 2× and the scanner's slot-wait share drops below 20%.
  test: After any merged worker speedup, run occupancy. Reopen if scanner slot-wait falls below 20% of the span, or worker idle while the scanner reads rises above 5%.
  score: medium (deferred)

- NOGO Giving the reader priority, or waking it sooner (the window journal's 'the reader is often not running'): of=docs/journal/2026-11-05-window.md:31-36 (the hypothesis behind #180)
  changed: At window 16 the reader is still slow to be woken for 8.7-14% of the span, but the slot waits that causes idle workers for only 0.08-0.34% of their time.
  now: No-go: ≤0.3% of 4t wall.
  test: Occupancy: reopen if worker idle in a slot wait with the earliest untaken block ready exceeds 1% of worker time.
  score: low

DISPATCHED: One occupancy.yml workflow_dispatch on master: ref master = 4bfaf7cb (decoder = d1fd4405), rounds 6, run 42009406264. It finished successfully in about 7 minutes and posts no comments.

What it was to decide, stated before dispatch:
- the production decoder's own 4-thread occupancy (none existed at d1fd4405);
- whether 4-thread idle vCPU time is fixed (start-up and tail) or steady-state;
- if it landed on an EPYC 9V45, whether the 7–8% idle vCPU that the 9V45 bench shows is steady-state.

It landed on an EPYC 9V74 and matched the other runners:
- off-mode wall 0.2488 s, CPU/wall 3.846, idle vCPU 3.9% (off) and 4.5% (full);
- s.input 1.88 ms; worker outside 7.9 ms of worker time;
- workers decoding 91.7% (timeline) and 93.9% (full);
- per-block mean decode_block elapsed 2.17 ms, CPU 1.90 ms, run-queue 0.27 ms.

So the production figures are pinned on a fourth CPU model. The 9V45 split stays open.
UNKNOWNS: - **The EPYC 9V45 has twice the idle of the other runners at 4t:** 6.8-8.4% of vCPU time in the benches (r41979947814, r41986885355), against 2.7-4.5% elsewhere. No occupancy run has landed on a 9V45; the one dispatched here landed on a 9V74. Whether that idle is start-up and tail, steady handoff, or the scanner is unmeasured. The 1/block-time reading is inferred.
- **Whether idle vCPU time comes as half-idle cores or whole-idle cores.** This decides between the lo and hi bounds. No instrument records per-sibling co-idleness.
- **The SMT sibling's marginal value on the 7763 and 9V74.** Neither has a pinned-2t bench artifact here, so their lo bounds use m = 0.13-0.24 by analogy, not measurement.
- **The bench's unattributed CPU at 4t:** 0.078 s, 6% of process CPU on the 8573C. It is split unknowably between the feeder and the workers' last sampler ticks.
- **The i9-10900K at current code.** It has no occupancy run. The history (old code) is 1026 MB/s at 4 workers with the scanner at 0.05 s of CPU (docs/journal/2026-10-28-workstation-scaling.md:276-277), and the scanner capped the -1 plateau at 0.09 of 0.143 s (:284-288). There are spare cores at 4t, so CI's saturation finding does not transfer; it is not inferred here.
- **wikidata-pq's consumer** runs real work on the reader thread and writes 8 MB at a time. Reader backpressure there is unmeasured; on CI the reader only copies and hashes.
- **decode_parallel's path is not benchmarked.** It uses 4 MiB to_vec chunks, and sizes its output Vec at data.len() × 4 (src/lib.rs:378), against Wikidata's 17.7:1 ratio.
- **Code identity of the runs.** Of the occupancy runs, only r42009406264 is exactly the d1fd4405 code. The others are c90b3625 (before #181), and beef4e8f and 9bee6b40 (closed #183). Their worker cost differs by up to about 3%.
NEXT: 1. **The decisive one:** a measurement-only extension of examples/occupancy.rs. It would:
   - split idle vCPU time into three phases (before the first Ready, steady, after the last Issued);
   - record whether a core's two siblings are idle together, sampling per-CPU idle or schedstat at the marks;
   - record the scanner's, feeder's and reader's CPU from RUSAGE_THREAD at thread exit.
   
   Run it until it lands on an EPYC 9V45. That turns the lo-hi bands into point estimates and settles whether the 9V45's 7-8% idle is a steady supply target that grows as the workers speed up.
2. **A cheap causal test of the CPU-capacity model:** a bench arm that drains without hashing, removing reader CPU only. The prediction is that 4t wall falls by ΔCPU/3.85. If the wall does not move, the 4t wall is bound by something other than CPU capacity, and this front's ranking flips.
3. **Only if the CI corpus's 4t wall is itself a target:** a small first chunk with a small first lookahead, worth 0.5-1.1% of the CI 4t wall and about 0 at release scale.

#### FALSIFIER

- SURVIVES: 1. At 4t the vCPUs are saturated; only 3.3-4.4% is truly unsupplied; the worker-time identity closes twice (run-queue = non-worker CPU, asleep = idle vCPU).
  WHY: OBSERVED: saturation is real and is not an artefact of the collector. Off mode has no trace and no sampler, and its CPU/wall is 3.846 (r42009406264, 9V74), 3.801 and 3.836 (r41976148952 and r41988588220, 7763), and 3.916 (r41986885416, 8370C).

Four corrections:
(a) The 'two closures' are one equation. In every run line span = wall ± 4 µs, so asleep − idle = 4(span − wall) + (non-worker − run-queue). On the 7763: 0.1736 − 0.1658 = 0.0078 = 0.0519 − 0.0441. With 4 workers on 4 vCPUs, run-queue ≈ non-worker CPU also follows from work conservation whenever the support threads run while the workers are runnable. It confirms saturation; it does not test it independently.
(b) The run-queue inside decode_block is 11.7-12.6% of worker time (0.1170/1.0027, 0.1492/1.1967, 0.1777/1.5207, 0.1523/1.2112). The 13-14% is the whole-life worker run-queue.
(c) The collector contaminates the hi bounds.
- O
  CORR: hi = off-mode idle with the sampler excluded:
- 8573C 3.4%;
- 8370C 2.1-2.4%;
- 7763 4.1-5.0%;
- 9V74 3.9%;
- 9V45 7.0-8.4% (sampler-on bench only).

Central estimate, INFERRED by applying #180's measured conversion of 0.59-0.72:
- 8573C 2.0-2.4%;
- 8370C 1.2-1.7%;
- 9V74 2.3-2.8%;
- 7763 2.4-3.6% of CI 4t wall.

At release scale, first remove the fixed ~1.4-1.7% of capacity (start-up, ramp, tail on the 9V74).

- SURVIVES: 2. Support-thread CPU (above all the scanner's) is the largest lever; 4t wall ≈ total CPU / (4×0.96); about 1.0% of wall per 10% scanner-CPU cut.
  WHY: The mechanism is pinned by the occupancy run-queue ≈ non-worker CPU (OBSERVED). The wall model's stated check proves nothing: 1.308/3.846 reproduces the wall because 3.846 is the same run's own CPU/wall, and any split that sums to the CPU passes.

Independent support is weak. In r41979947814 (9V45), 4t wall moved −3.25±1.13%, total CPU −3.55±1.19% and attributed worker CPU −4.97±1.39%. Wall tracks total CPU more closely than worker CPU, but only at about 1σ.

The 1.0%-per-10% figure assumes that a worker taking the scanner's sibling slot costs the same CPU per block as the average. On the walk, worker-worker co-residency is expensive: 4 co-running copies run at 6.63 c/B against 2.89 alone, 1.74x aggregate on 2 cores (r41989592674, 9V45). So the figure is an upper estimate.

Two smaller errors:
- The scanner's 4t CPU of 0.135 s is floored to 10 ms ticks (utime and stime separately) and lo
  CORR: ≤1.0% of 4t wall per 10% scanner-CPU cut (upper estimate). The lower end is unpinned until the drain-without-hash arm runs.

- SURVIVES: 3. Start-up: a fixed 1.8-3.4 ms in which every worker idles (4 MiB first chunk, 2 MB lookahead); 0.5-1.1% of CI 4t wall.
  WHY: OBSERVED, and it reconciles. On the 9V74, s.input is 1.88 ms, w.idle.input is 7.5 ms of worker time (that is, 4 × 1.87 ms), and s.input_workers_busy is 5-7e-6. About 3 idle vCPUs × 1.88 ms = 0.56% of capacity; as a share of wall it is 0.76%.

The 8370C 4t timeline (r41986885416) is flagged by the tool itself: 'perturbs: its trace is not interpreted' (throughput ratio 1.0057, 1.0006-1.0108; examples/occupancy.rs:36-37). Its full mode gives the same 1.86 ms, so the figure stands.

The window copy (extend_from_slice at src/parallel.rs:320) comes after InputGot, so it belongs to the ramp, not to s.input.
  CORR: 0.5-1.1% of CI 4t wall (as B said); 0 at release scale.

- FALLS: 4. Steady-state supply losses are all that remain at release scale; the 4t dips come from the scanner waiting in the run queue, not from its speed.
  WHY: The residual reconciles. On the 9V74, off idle of 3.75-3.9%, less start-up 0.56-0.75%, ramp about 0.2-0.3% and tail about 0.65%, leaves 2.0-2.5%. The mechanism, though, is not pinned.
- The 2t comparison confounds run-queue with headroom. On the 9V74 the scanner reads a block every 0.22 ms against consumption every 0.61 ms at 2t (2.8x); at 4t it is 0.286 against 0.54 ms (1.9x).
- '123-211 ms reading vs 0.13-0.14 s CPU' pairs a 9V74 reading time (123 ms) with a Xeon CPU figure. 123 ms below 130 ms is impossible on one machine.
- The only same-CPU pairing is the 8370C: 211 ms reading against 0.14 s scanner CPU, though from different commits (beef4e8f, c90b3625). It puts about a third of the reading time off-CPU, so the run queue explains about a third of the dips at most.
- The split depends on the collector. On the 9V74, w.idle.reading is 4.15% in timeline mode and 2.65% in full mode, and
  CORR: Steady idle is 2.0-2.5% of capacity on the 9V74. At 0.59-0.72 conversion, that is about 1.2-1.8% of 4t wall: the part that matters at release scale. Cause unresolved (scanner speed vs run queue).

- SURVIVES: 5. Window and ordered output exhausted at window 16 (slot-wait idle 0.13-0.62%, 1.9-3.1 ms of worker time).
  WHY: The conclusion holds, but the numbers are off for the production code. On the 9V74 (r42009406264), w.idle_during_slot_wait_s has a median of 6.5 ms (timeline) and 4.4 ms (full), not 1.9-3.1 ms. Two single timeline runs reach 0.0112 and 0.0100 worker-seconds, so the proposed falsifier threshold of 0.01 already falls inside window-16 noise.
  CORR: ≤0.65% of worker time × 0.6-0.7 ≈ ≤0.4% of 4t wall.

- SURVIVES: 6. Tail: a fixed ~2-2.6 ms wall-equivalent at 4t.
  WHY: It reconciles. w.outside is 7.9-10.5 ms of worker time, 0.62-0.87%. That fits about 1.2-1.5 last blocks (2.17-3.35 ms each) plus the reader catching up. It is INFERRED, and harmless.
  CORR: 

- SURVIVES: 7. The scanner is not on the 4t critical path; it would bind only if worker CPU fell 2.0-2.5x; supply idle grows about as 1/block time.
  WHY: 'Not binding today' is pinned: the scanner waits for a slot 40-47% of the span. The k threshold is only a lower bound. Holding R_s (reading time) fixed keeps its run-queue share (about a third on the 8370C), which would shrink as the box unloads. With R_s at the scanner's CPU (0.14 s, 8370C), (1.2716/k + 0.2076)/3.85 = 0.14 gives k ≈ 3.8.

The 'idle × block time ≈ constant' argument cannot tell per-block handoff latency from per-decode fixed costs. On a fixed corpus, wall is proportional to block time, so both scale the same way. It also mixes sampler-on bench idle (9V45, 8370C) with occupancy idle (9V74, 7763).
  CORR: 0 today. The bind threshold is k ≈ 2.0-3.8x.

- FALLS: 8. The bench's overlap gap (1.297, 8573C) is about 90% support-thread CPU growing with bytes plus ≤5 ms fixed; wall_n (2998 s) is right, wall_n_fixed (2313 s) under-reads 20-25%.
  WHY: The decomposition is wrong. B counts the 0.078 s unattributed CPU as support. Per bench/CHANGELOG.md:151-156 and bench/report.py:323-325, unattributed is CPU the 10 ms sampler loses (threads ending between samples, below each thread's last tick), mostly the workers'.

Per round in r41980640481, the gap (median 0.0734 s) splits as:
- attributed support (scanner + main): 53.4% [35.7-60.7];
- unattributed: 26.0% [17.9-36.8];
- sampler: 2.1%;
- idle: 15.6% [12.3-45.0].

The bottom line survives. About 80% of the gap is real CPU and grows with bytes wherever it is attributed, so wall_n is the better reading. But wall_n also scales up the fixed idle (about 1.4-1.7% of capacity, measured on the 9V74, not this runner) and the sampler (0.45%, absent from a release). The model's workers_ns_per_out_byte (5.055) and scanner_binds_from (7.86) are understated by the unattributed worker CPU.
  CORR: Release 4t ≈ 2998 × 0.98 ≈ 2,940 s. wall_n_fixed under-reads by about 21%.

- SURVIVES: 9. At 1 and 2 threads there is next to nothing to recover from supply.
  WHY: Observed: 2t pinned idle is 2.03% (r41980640481), 0.48% (r41976148934 head) and 1.97-2.29% (r41986885355). The 1t runs show CPU/wall of about 1.00. Note that at 2t the sampler runs on the decoder CPUs' SMT siblings (bench/CHANGELOG.md:168-170), a small collector term on the 2t rows and on the m estimates.
  CORR: 

MISSED: 1. The idle-to-wall conversion has already been measured on CI. #180 is the only supply-only A/B. At 4t it turned 0.63 (bootstrap 0.57-0.71) of the closed idle into wall on r41976148934 (8370C: idle 14.93% → 2.44%, wall −8.87%, CPU +5.70%). From the journal's deltas, the other two runs give 0.59 (8370C) and 0.72 (7763) (docs/journal/2026-11-05-window.md:121-125). The paired CPU change of a supply-only arm measures, in aggregate, the 'half-idle vs whole-idle core' question B lists as having no instrument. It makes the lo bound (2xm/(1+m), about 0.22-0.39x) uninformative.

2. The bench's sampler is the collector artefact in B's 4t idle figures.
- At 4t it is pinned on the decoder's CPUs: sampler_cpus is 'pinned' in every 4t row.
- B's gross idle counts its CPU as idle; B's 'net of others' subtracts it twice (ab.rs:602, :988).
- others_busy is never resolvable above 1 jiffy at 4t (0.025-0.056 CPUs).
- Its measured cost to wikidata at 4t is +2.4% and +1.2% of wall, with intervals of ±4% (bench/CHANGELOG.md:162, :166). That is the same order as the whole recoverable idle.
- The 9V45's 7-8% idle exists only in sampler-on bench runs; no occupancy run has landed on a 9V45. Part of it could be the sampler preempting the scanner or reader, the supply threads. This is unmeasured.

3. The unattributed CPU (6% of 4t CPU) is a sampler artefact, mostly worker CPU. It distorts B's claim 8 and the bench model's per-byte worker rate.

4. B's identity is one equation, not two (span = wall ± 4 µs), and work conservation implies most of it when there are as many workers as vCPUs.

5. The tool's own perturbation rule.
- It excludes r41986885416's 4t timeline, which B used for 8370C figures.
- The full-mode 4t cost intervals are wide: −6.6/+3.2% on the 9V74, wider than the idle being split.
- Timeline and full modes split the idle differently by 1-1.6 percentage points (9V74: w.idle 5.7% vs 4.1%).
- The per-cause split of idle is therefore good to about ±1.5 points; only the off-mode total is clean.

6. On a fixed corpus, wall is proportional to block time. 'Idle ∝ 1/block time' therefore cannot tell steady handoff losses, which survive at release scale, from fixed start-up and tail, which do not.

7. Cross-machine pairings: a 9V74 reading time against Xeon scanner CPU, and an idle × block-time table mixing two instruments.

8. Smaller points:
- Bench CPU includes the child's process start-up, which wall excludes (ab.rs:589, startup = 0 for in-process arms). This biases idle slightly low.
- Accounting reconciliation at 4t on the 8573C: 1t CPU 0.804 s, 2t-pinned CPU 0.836 s (1.039x), 4t CPU 1.314 s (1.634x over 1t). This is consistent with SMT sharing at m ≈ 0.24-0.27 net of idle. No unexplained idle remains beyond the 3.4% (sampler excluded).
