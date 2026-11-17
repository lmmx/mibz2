---
title: "Technical notes: modelling the cache to rightsize workers"
description: "Technical notes to the post on modelling the cache to rightsize workers: the contention–parallelism trade-off and its sensitivity, what the contention-associated overhead is made of, the full ledger of frozen predictions, the CPU bound's history, and topology and transfer."
---

← [Modelling the cache to rightsize workers](2026-11-17-modelling-the-cache-to-rightsize-workers.md)

# Technical notes: modelling the cache to rightsize workers

These notes hold the detail that [the main post](2026-11-17-modelling-the-cache-to-rightsize-workers.md) leaves out to keep its argument moving. They use its notation: $W$ workers, $N$ logical CPUs in the process's affinity set, the normalised cost per byte $c(W) = a + b\,W^2$ with $a \approx 0.422$ and $b \approx 0.0068$, and the footprint $F$. Measured figures are on the i9-10900K, decoding four copies of an 11.9 MB Wikidata extract with the lazy-letters kernel, unless a row says otherwise. Its sections can be read in any order.

## 1. The trade-off at the optimum

With every worker kept busy, the wall time goes as $c(W)/W = a/W + b\,W^{p-1}$ for a cost $c(W) = a + b\,W^p$. Setting the derivative to zero gives $a = (p-1)\,b\,W^p$, so at the continuous optimum the contention-associated overhead $b\,W^p$ is $1/p$ of the total: half at $p = 2$, two thirds at 1.5, 40% at 2.5. It is a fitted term, not counted stall cycles.

The fits over 6 to 9 workers at 10 threads, and what each says at 7 workers:

| exponent *p* | *a* | *b* | continuous optimum | overhead share at 7 | cost ratio *c*(7)/*a* |
|---:|---:|---:|---:|---:|---:|
| 1.5 | 0.300 | 0.0248 | 8.37 | 60.5% | 2.53 |
| 1.75 | 0.370 | 0.0129 | 8.02 | 51.2% | 2.05 |
| 2 | 0.422 | 0.0068 | 7.88 | 44.1% | 1.79 |
| 2.5 | 0.494 | 0.0020 | 7.70 | 34.4% | 1.52 |

The range is sensitivity to the model's form, not a confidence interval. The cost ratio is what the model would charge with the overhead removed and nothing else changed. It is not a speedup anyone has measured or can expect: removing contention by any other means would change other costs too. At 7 workers the share is under half because 7 is below the quadratic's optimum of 7.9; under that fit alone, 8 workers would cost $0.10715$ against 7's $0.10789$.

!!! info "A familiar curve, a different cause"

    Throughput proportional to $W / (a + b\,W^2)$ has the shape of [Gunther's Universal Scalability Law](https://arxiv.org/abs/0808.1431): a rational function that rises, peaks and then falls (retrograde scaling). The resemblance is in the shape only. In the USL the quadratic term models coherency between processors; here it is fitted to interference in a shared cache.

## 2. What the contention-associated overhead is made of

**A smaller footprint at 7 workers.** A variant of the lazy-letters kernel that stores its rules in 12 bytes instead of 16 shrinks each worker's footprint. With 9 workers past the cores it took 0.98 to 0.99 of the wall time. With 7 it cut LLC misses per byte to 0.926 at 12 threads, with 1.6% more instructions, and no change in wall time was detected: 1.012 (0.998, 1.028), 0.996 (0.987, 1.005) and 1.002 (0.985, 1.018) of the 16-byte kernel's at 12, 16 and 20 threads. That is consistent with the saved misses overlapping other work and being offset by the extra instructions. It doesn't identify what the overhead is.

**The cycles per miss avoided.** Across the 10-thread arms, the cycles saved per LLC miss avoided come to about 589 from 10 workers to 7. That is a slope across arms, not a cost per miss: it moves with the cost of the misses that remain, with overlap, and with every other cost that changes with the count. A conditional calculation is all it supports: *if* each removed miss carried 390 to 490 unhidden cycles, the cost of an uncontended trip to memory at an assumed 80 to 100 ns, *and* nothing else changed, fewer misses would account for 66% to 83% of the saving. Neither assumption is established. The counters run below measured a loaded miss latency of about 595 cycles, already outside that range, with about 2.2 misses in flight at once; dividing one by the other doesn't give the exposed critical-path cost either. The remainder can't be assigned to anything by subtraction. Large miss reductions need not cut cycles in proportion when memory-level parallelism changes, as [Qureshi and Patt (2006)](https://doi.org/10.1109/MICRO.2006.49) show, and stall counters overlap, as [Yasin (2014)](https://doi.org/10.1109/ISPASS.2014.6844459) warns.

**The two hypotheses, scored.** Before the counters run, the model's [record 47](https://github.com/lmmx/mibz2/blob/0e7c6888bcba2ce8dd3056c587a23be958d6de4f/docs/journal/monodromy/2026-11-17-contention-counters.md) froze predictions for two readings: queueing in the L3 and the ring that links it to the cores (Q), and main-memory latency rising with load (D). [Record 51](https://github.com/lmmx/mibz2/blob/c7ea4a5c5e8447fbdfd407c6f0ae07b6669bd87e/docs/journal/monodromy/2026-11-17-miss-cost-scoring.md) scored them, from 10 workers to 7 at 10 threads:

| frozen prediction | measured | status |
|---|---|---|
| Q: L3-hit latency falls 10% to 25% | derived from demand-read occupancy: 91.0 → 65.2 cycles, −28% | direction right, size outside the range |
| Q: L3-miss latency falls 5% to 15% | 594 → 552 cycles, −7% | met |
| D: L3-miss latency falls 25% or more | −7% | not met |
| main-memory traffic at 10 workers: 4 to 9 GB/s under Q, 60% of peak or more under D | not measured; demand L3-miss reads alone are 1.8 GB/s, not total traffic | untested |
| Q: stalls on L3 misses at most 45% of the cycle cut | 43% | met |
| Q: stalls on L2 misses at least 70% of the cut | 46% | not met |
| L3-hit loads per byte flat within 5% | 0.908 | not met |
| L3-miss loads per byte, 7 against 10, about 0.52 | 0.390 | not met |

Q is not refuted, and D is unsupported but not formally refuted, since its traffic half wasn't measured. Neither is identified as the mechanism. The record fits the derived L3-hit latency as about 44.8 cycles plus a term proportional to the L3-miss reads per byte, which would mean each miss also slows the hits around it; that is a model on four points, not a measurement of cause.

## 3. The ledger of frozen predictions

Each prediction below was frozen before its run. Where a later record corrected a model, both versions are kept.

| record | frozen prediction | measured | verdict |
|---|---|---|---|
| 41 | a capacity cliff: misses flatten from 7 workers to 6 | fell a further 28% | refuted |
| 42 | ring or bandwidth contention with flat misses | misses fell 41% from 8 workers to 6 | refuted (this rival only) |
| 43 | 10 threads: 7 workers, at 0.93 (0.90, 0.98) of 10's time | 7, at 0.885 | count right, gain outside the interval |
| 44 | counts at 8, 6 and 4 threads, with 11 ratio intervals | counts right; 4 of 11 intervals missed; its near-saturated utilisation refuted | mixed |
| 45 | `-3`, 10 threads: the best count is 9 or 10 | 7 | refuted |
| 45 | `-3`: 7 workers no better than 0.98 of 10's time | 0.978 (0.973, 0.983) | borderline |
| 45 | `-3`, if bandwidth-bound: 7 at about 0.885 | 0.978 | refuted |
| 45 | CI runner: 4 workers, and the reference plan stays | not yet run | pending |
| 46 | 8, 6 and 4 threads: 6, 5 and 4 workers; utilisation flat near 0.76 | 6, 5 and the reference 4; 0.742, 0.725, 0.740 | right |
| 46 | rival idle term: 3 workers at 4 threads; utilisation 0.56, 0.67, 0.74 | the reference 4; flat | refuted |
| 46 | `-3`, 10 threads: 9 workers, 7 at about 1.18 of 10's time | 7 best, at 0.978 | refuted |
| LOG | 0.7 of the threads, rounded: 6, 4 and 3 | penalties of 12% at 6 threads and about 11% at 4 | refuted |
| LOG | 0.7 of the threads, floored: 5, 4 and 2 | fails at all three counts | refuted |
| 47 | Q and D, the counters run | see section 2 | mixed |
| 48 | `-9` misses at 4 workers match `-3`'s at equal fullness | 0.00368 against 0.0045 (0.0035, 0.0058) | met |
| 48 | the same at 3 workers | 0.00225 against 0.0034 (0.0027, 0.0042), 34% low | refuted |
| 49 | eager kernel at the trimmed counts | the eager-trim run is queued | pending |

The 8-thread count was observed before record 46's CPU bound was written; the low-thread sweep re-measured it and tested 6 and 4 threads afresh.

**The low-thread sweep in full.** Each arm against the reference plan at its count, 10 paired rounds, pinned one CPU a core:

<figure class="bf-fig">
<p class="bf-title">Below 8 threads: 5 workers win at 6 threads, and nothing beats the reference 4 at 4</p>
<svg viewBox="0 0 600 380" role="img" aria-label="The low-thread sweep against the reference plan. At 6 threads: 4 lazy workers 1.077, 5 lazy workers 0.963, 6 lazy workers 1.040. At 4 threads: 2 lazy workers 1.533, 3 lazy workers 1.113, 4 lazy workers 1.098."><text x="4.0" y="18.0" class="bf-tm" text-anchor="start" style="font-size:14px">wall time against the reference plan at the same thread count (lower is faster)</text><path d="M70 283.0H470" class="bf-grid"/><text x="62.0" y="288.0" class="bf-tm" text-anchor="end" style="font-size:14px">0.96</text><path d="M70 256.0H470" class="bf-grid"/><path d="M70 229.0H470" class="bf-rule"/><text x="62.0" y="234.0" class="bf-tm" text-anchor="end" style="font-size:14px">1.00</text><path d="M70 202.0H470" class="bf-grid"/><path d="M70 175.0H470" class="bf-grid"/><text x="62.0" y="180.0" class="bf-tm" text-anchor="end" style="font-size:14px">1.04</text><path d="M70 148.0H470" class="bf-grid"/><path d="M70 121.0H470" class="bf-grid"/><text x="62.0" y="126.0" class="bf-tm" text-anchor="end" style="font-size:14px">1.08</text><path d="M70 94.0H470" class="bf-grid"/><path d="M70 67.0H470" class="bf-grid"/><text x="62.0" y="72.0" class="bf-tm" text-anchor="end" style="font-size:14px">1.12</text><text x="103.3" y="334.0" class="bf-tm" text-anchor="middle" style="font-size:15px">2</text><text x="186.7" y="334.0" class="bf-tm" text-anchor="middle" style="font-size:15px">3</text><text x="270.0" y="334.0" class="bf-tm" text-anchor="middle" style="font-size:15px">4</text><text x="353.3" y="334.0" class="bf-tm" text-anchor="middle" style="font-size:15px">5</text><text x="436.7" y="334.0" class="bf-tm" text-anchor="middle" style="font-size:15px">6</text><text x="270.0" y="360.0" class="bf-tm" text-anchor="middle" style="font-size:15px">lazy-letters workers</text><path d="M270.0 125.0 L353.3 278.9 L436.7 175.0" style="fill:none;stroke:var(--bf-blue);stroke-width:3"/><circle cx="270.0" cy="125.0" r="6" style="fill:var(--bf-blue);stroke:var(--bf-blue);stroke-width:2.5"><title>4 workers: 1.077</title></circle><path d="M353.3 299.2V260.0" style="stroke:var(--bf-ink);stroke-width:2"/><circle cx="353.3" cy="278.9" r="6" style="fill:var(--bf-blue);stroke:var(--bf-blue);stroke-width:2.5"><title>5 workers: 0.963</title></circle><circle cx="436.7" cy="175.0" r="6" style="fill:var(--bf-blue);stroke:var(--bf-blue);stroke-width:2.5"><title>6 workers: 1.040</title></circle><path d="M186.7 76.4 L270.0 96.7" style="fill:none;stroke:var(--bf-amber);stroke-width:3"/><circle cx="186.7" cy="76.4" r="6" style="fill:var(--bf-amber);stroke:var(--bf-amber);stroke-width:2.5"><title>3 workers: 1.113</title></circle><circle cx="270.0" cy="96.7" r="6" style="fill:var(--bf-amber);stroke:var(--bf-amber);stroke-width:2.5"><title>4 workers: 1.098</title></circle><path d="M103.3 94.0V46.7" style="stroke:var(--bf-amber);stroke-width:3"/><path d="M97.3 56.2l6 -8 6 8" style="fill:none;stroke:var(--bf-amber);stroke-width:3"/><text x="115.3" y="60.2" class="bf-tm" text-anchor="start" style="font-size:14px">2: 1.533</text><text x="353.3" y="308.9" class="bf-t-strong" text-anchor="middle" style="font-size:16px;fill:var(--bf-blue)">6 threads: 5 workers, 0.963</text><text x="426.7" y="161.0" class="bf-tm" text-anchor="end" style="font-size:14px">6 lazy: 1.040</text><text x="280.0" y="147.0" class="bf-tm" text-anchor="start" style="font-size:14px">1.077</text><text x="176.7" y="98.4" class="bf-tm" text-anchor="middle" style="font-size:14px">1.113</text><text x="282.0" y="101.7" class="bf-tm" text-anchor="start" style="font-size:14px">1.098</text><path d="M488 120H512" style="stroke:var(--bf-blue);stroke-width:3"/><text x="518.0" y="125.0" class="bf-t" text-anchor="start" style="font-size:15px">6 threads</text><path d="M488 150H512" style="stroke:var(--bf-amber);stroke-width:3"/><text x="518.0" y="155.0" class="bf-t" text-anchor="start" style="font-size:15px">4 threads</text><text x="488.0" y="200.0" class="bf-tm" text-anchor="start" style="font-size:14px">1.00 = reference</text><text x="488.0" y="220.0" class="bf-tm" text-anchor="start" style="font-size:14px">eager plan,</text><text x="488.0" y="240.0" class="bf-tm" text-anchor="start" style="font-size:14px">one worker</text><text x="488.0" y="260.0" class="bf-tm" text-anchor="start" style="font-size:14px">a thread</text></svg>
<figcaption>Measured on the i9, decoding four copies of the Wikidata extract, 10 paired rounds a point, the process limited to one hardware thread a core; the bar is the 95% interval over the rounds. At 6 and 4 threads the reference plan runs the older (eager) kernel, one worker a thread, so the 6-worker point at 6 threads and the 4-worker point at 4 threads measure the lazy-letters kernel alone. The 2-worker point at 4 threads, 1.533, is off the chart.</figcaption>
</figure>

| threads | reference plan | arm | wall |
|---:|---|---|---:|
| 8 | lazy 8 workers, window 32 | lazy 6/24 | 0.925 (0.918, 0.933) |
| 8 | | lazy 5/20 | 0.997 (0.984, 1.010) |
| 6 | eager 6 workers, window 24 | lazy 6/24 (kernel control) | 1.040 |
| 6 | | lazy 5/20 | 0.963 (0.948, 0.977) |
| 6 | | lazy 4/16 | 1.077 |
| 4 | eager 4 workers, window 16 | lazy 4/16 (kernel control) | 1.098 |
| 4 | | lazy 3/12 | 1.113 |
| 4 | | lazy 2/8 | 1.533 |

At 6 threads the 0.963 includes a change of kernel; the lazy kernel alone costs 4% there, so the count's own share is about $0.963 / 1.040 \approx 0.93$, a descriptive ratio, not a paired result.

**The `-3` hot-set ratio.** The 0.355 first used was the ratio of the workers' total buffers at the end of a block (5.97 MB against 16.81 MB). Counting the kernel's per-letter store directly gives a hot-set ratio of 0.377, and the exponent of the footprint in the contention term then comes out at 2.03, 2.23, 1.85 and 2.05 from the four slope ratios (cycles and misses, over 6–9 and 7–10 workers). The cache bound at `-3` is about 22.

## 4. The CPU bound and the tie-break

**Per-worker CPU utilisation.** A worker with a CPU to spare is busy 0.93 of the time. With $\nu$ the non-worker threads' CPU use and $\iota$ a residual idle capacity, the workers get $\min(0.93\,W,\ N - \nu - \iota)$ CPUs. At 10 threads, with $\nu$ counted and $\iota = 0.125\,N$, that reproduces the measured utilisation: 0.767, 0.851, 0.930, 0.930 and 0.930 for 10 down to 6 workers, against 0.763, 0.857, 0.931, 0.925 and 0.931.

!!! info "A correction to this term"

    An earlier version of the busy term was derived from the wall times and the cycles, and the review of the runs pointed out that with a flat clock it is close to an identity, so its match was not independent evidence. The term is now built from each thread's counted processor time: a separate measurement, but still calibrated to these runs.

**The rule's history.** Record 46 wrote the CPU bound as $(N - \nu - \iota)/0.93$: 8.2 at 10 threads and 6.6 at 8. Record 49 replaced the counted $\nu$ with $\nu \approx 0.16\,W$, which gives $0.80\,N$; it also replaced the physical-core caps with logical CPUs, since counting SMT siblings as fractional cores would cap a 2-core, 4-thread runner at 2. Its first tie-break, the larger of two candidates within 1%, gave 7 at 8 threads and 8 at 10 against the measured 6 and 7, so it was reversed to the smaller, a choice fitted to those two counts. [Record 50](https://github.com/lmmx/mibz2/blob/c7ea4a5c5e8447fbdfd407c6f0ae07b6669bd87e/docs/journal/monodromy/2026-11-17-rule-scope.md) found that with its own $\nu$ the rule gives 3 at 4 threads, against the reference plan's measured 4, hence the exception at 4 threads or fewer. Against the fitted model's own joint optimum, composing the bounds costs at most 0.7% at the measured counts; the model itself is the larger error.

## 5. Topology and transfer

The cache bound sums over LLC-sharing domains: $\sum_d \min(N_d,\ 0.8 \times L_d / F)$, with $N_d$ the logical CPUs of the affinity set in domain $d$. The sum holds only if workers are spread across domains, which soft affinity per domain would arrange. The CPU bound stays process-wide. SMT enters through each worker's cost $c(W)$, not through the caps.

On the free GitHub x86 runners sampled so far (AMD EPYC 7763, 9V74 and 9V45: 2 cores, 4 logical CPUs, one 32 MiB L3) the cache bound is about 12.8 and never binds, and the rule gives $m = 3.2$. "Within one worker" can only fail there on an outcome the existing data already rule out, so the runners are judged on three frozen refuters instead: 3 workers beating 4 by more than 2%; 2 workers coming within 3% of 4; or any lazy-letters plan beating the reference plan by more than 2%.

The open topology question is the one the main post ends on. The i9 check reads the topology through the affinity set; a 10-thread run pinned to CPUs 0 to 9 sees 10 logical CPUs and no trim, while every sweep behind the trims ran pinned that way. Whether the decoder should read its topology machine-wide (present and online CPUs) or through its affinity set waits on the unpinned run.

---

**Research records:** the [main post's](2026-11-17-modelling-the-cache-to-rightsize-workers.md) list, and the model's records [45](https://github.com/lmmx/mibz2/blob/c7ea4a5c5e8447fbdfd407c6f0ae07b6669bd87e/docs/journal/monodromy/2026-11-17-workers-machine-model.md), [46](https://github.com/lmmx/mibz2/blob/c7ea4a5c5e8447fbdfd407c6f0ae07b6669bd87e/docs/journal/monodromy/2026-11-17-workers-cpu-budget.md), [47](https://github.com/lmmx/mibz2/blob/c7ea4a5c5e8447fbdfd407c6f0ae07b6669bd87e/docs/journal/monodromy/2026-11-17-contention-counters.md), [48](https://github.com/lmmx/mibz2/blob/c7ea4a5c5e8447fbdfd407c6f0ae07b6669bd87e/docs/journal/monodromy/2026-11-17-level3-scoring.md), [49](https://github.com/lmmx/mibz2/blob/c7ea4a5c5e8447fbdfd407c6f0ae07b6669bd87e/docs/journal/monodromy/2026-11-17-worker-rule.md), [50](https://github.com/lmmx/mibz2/blob/c7ea4a5c5e8447fbdfd407c6f0ae07b6669bd87e/docs/journal/monodromy/2026-11-17-rule-scope.md) and [51](https://github.com/lmmx/mibz2/blob/c7ea4a5c5e8447fbdfd407c6f0ae07b6669bd87e/docs/journal/monodromy/2026-11-17-miss-cost-scoring.md); the [12-byte rules at 7 workers](https://github.com/lmmx/mibz2/pull/145#issuecomment-6944610173); the [simple candidate rules and their scoring](https://github.com/lmmx/mibz2/pull/145#issuecomment-6943635048).

**Further reading:**

- N. J. Gunther, ["A General Theory of Computational Scalability Based on Rational Functions"](https://arxiv.org/abs/0808.1431), 2008: the Universal Scalability Law.
- M. K. Qureshi and Y. N. Patt, ["Utility-Based Cache Partitioning"](https://doi.org/10.1109/MICRO.2006.49), MICRO 2006: shared-LLC capacity and miss rates.
- C. Xu, X. Chen, R. P. Dick and Z. M. Mao, ["Cache Contention and Application Performance Prediction for Multi-Core Systems"](https://doi.org/10.1109/ISPASS.2010.5452065), ISPASS 2010.
- A. Yasin, ["A Top-Down Method for Performance Analysis and Counters Architecture"](https://doi.org/10.1109/ISPASS.2014.6844459), ISPASS 2014.
- R. L. Mattson, J. Gecsei, D. R. Slutz and I. L. Traiger, ["Evaluation Techniques for Storage Hierarchies"](https://doi.org/10.1147/sj.92.0078), IBM Systems Journal, 1970.
