# External research report 148: the guardian on the walker capture: valid; D1 holds; the four-copy penalty persists in reduced kernels; the physical cause unresolved (verbatim)

A research input, not a finding of this repository.

- **Relayed:** by the maintainer, 2026-11-02, with reports 148–150 together.
- **Source:** the guardian.
- **In reply to:** the walker-ladder capture at 29a0215 and the coordinator's §8.

**Evidence cutoff:** 2 November 2026, after the capture. An independent check of the archive and recalculation; no new run.

It is kept exactly as received (rendered text, cards flattened, tracking parameters
included).

## The message, as received

`````text
**The capture is valid, and it gives us a useful answer: the shipped walker is already close to the reduced reference on one core, while running several copies still makes each substantially more expensive.**

That points away from a large gain through ordinary loop/bookkeeping cleanup. It does **not** yet identify the physical cause of the concurrency penalty.

I verified the archive SHA-256 and all three manifest checksums, checked the raw perf records against their parsed values, and independently recalculated the main results. My independent-review\.json[independent checks and calculations](sandbox:/mnt/data/walker_ladder_review/independent-review.json) agree with the recorded decisions.

## 1. The measurement passed

| Check | Result |
|---|---:|
| Recorded source | `29a0215`, clean tracked tree |
| Measurement records | **618**, matching the planned configuration counts |
| Counter collection | All expected events counted; **100% reported running time** |
| Correctness, control and result-capacity checks | Passed throughout |
| Shipped-walker timing spread | **2.58%**, within the 5% gate |
| Minimum four-core window overlap | **99.44%**, above the 95% gate |
| Maximum control-only cycle cost | **0.012%** of its corresponding A32 window |

The archive’s summary says `"valid": true`, with no G0 failures. My independent checks reproduce that result. This is no longer an instrumentation troubleshooting exercise—we have usable measurements. walker-ladder-20261102T004502Z-summary.json[Recorded summary](sandbox:/mnt/data/walker_ladder_review/walker-ladder-20261102T004501Z/walker-ladder-20261102T004502Z-summary.json)

## 2. The main result: only a small gap to the reduced reference

These are **cycles per BWT row**, at 32 lanes on one core. They are not whole-decoder cycles per final output byte.

| Rung | What it measures | Cycles/row |
|---|---|---:|
| **A** | Counter-terminated traversal, without software prefetch | 9.763 |
| **B** | A with software prefetch | 6.659 |
| **C** | Flag-terminated traversal with prefetch | 6.363 |
| **D** | Output-producing reduced walker | 6.840 |
| **M** | Reduced flag-terminated walker without output stores | **6.293** |
| **P** | Shipped segmented walk, including assembly | **6.897** |

These values are recalculated from the walker-ladder-20261102T004502Z.jsonl[raw measurement records](sandbox:/mnt/data/walker_ladder_review/walker-ladder-20261102T004501Z/walker-ladder-20261102T004502Z.jsonl).

The central comparison is:

\[
\frac{P_{32}}{M_{32}}
=\frac{6.897}{6.293}
=\mathbf{1.096}.
\]

**The shipped implementation costs only 9.6% more cycles per row than the best reduced 32-lane reference.**

That triggers the registered **D1 decision: no walker-scheduling rewrite on this evidence**. It is comfortably inside the protocol’s 15% band. walker-ladder-20261102T004502Z-summary.json[Recorded decision](sandbox:/mnt/data/walker_ladder_review/walker-ladder-20261102T004501Z/walker-ladder-20261102T004502Z-summary.json)

Two qualifications are essential.

First, **M is not a usable decoder**: it omits required output work. The 9.6% gap is not a saving we can simply collect by writing better Rust.

Second, **M is not a mathematical hardware minimum**. This result discourages a large rewrite of the current scheduling and bookkeeping; it does not prove that a different algorithm or memory representation cannot outperform it.

Nevertheless, this is a consequential result. The idea that a large, obvious bookkeeping tax is hiding inside the current one-core walker has become substantially less convincing.

### Prefetch is doing useful work

The A→B contrast reduces cost from **9.763 to 6.659 cycles/row**: about **31.8% fewer cycles**, equivalent to **46.6% higher traversal throughput** in that particular comparison. independent-review\.json[Independent calculations](sandbox:/mnt/data/walker_ladder_review/independent-review.json)

That is not a new decoder speedup—the shipped walker already prefetches. It demonstrates that the mechanism we already rely on is valuable in this test.

The lane-count results also show little difference between 16 and 32 lanes for the shipped walker: **6.763 versus 6.897 cycles/row**. That is descriptive evidence of diminishing returns here, not authorization to change production to 16 lanes or begin another sweep.

## 3. The larger finding: concurrency still hurts without the production pipeline

The four-core section runs independent copies of the kernels, rather than the full scanner/feeder/reader pipeline.

| Kernel, 32 lanes | One-core cycles/row | Four-core cycles/row | Increase |
|---|---:|---:|---:|
| **A: no prefetch, no output stores** | 9.763 | 13.594 | **39.2%** |
| **B: prefetch, no output stores** | 6.659 | 9.196 | **38.1%** |
| **P: shipped walk and assembly** | 6.897 | 10.000 | **45.0%** |

Elapsed time per row rises by approximately 48%, 47% and 55%, respectively. Those are per-copy costs, not claims that aggregate four-core throughput is lower than one-core throughput. independent-review\.json[Independent four-core comparison](sandbox:/mnt/data/walker_ladder_review/independent-review.json)

**This is the finding I would emphasize to Astra and the researcher.**

A substantial concurrency penalty appears even in kernels that do not emit decoded output and do not use the production job queue. Therefore, a slowdown of this kind **does not require** the production dispatch machinery or segment-assembly traffic to appear.

That narrows the explanation. It does not establish that this simplified workload reproduces the original full-decoder penalty through exactly the same mechanism. Nor does it separate cache residency, request servicing, prefetch interactions and other concurrency effects.

The increase in **cycles** also shows that a lower clock frequency cannot explain the whole elapsed-time increase.

### One important limit on the four-core conclusion

The registered D5 comparison passed: its ratio of ratios is **1.055**, within the 15% transfer band.

But **D5 compares P with B—not P with M**. The four-core section did not run M. We therefore must not write:

> “The production walker is within 9.6% of the minimal reference at four cores.”

That was demonstrated only in the single-core section. The frozen collection and analysis make this distinction explicit. 

## 4. The counters do not identify one dominant hardware limit

The counters are usable, but the registered diagnostic signatures do not justify a simple “execution-bound” or “miss-capacity-bound” verdict.

### Issue pressure: below the registered warning level

For P32, the summary reports:

| Quantity | Recorded share |
|---|---:|
| Issued µops relative to four per cycle | **57.1%** |
| Retirement slots relative to four per cycle | **54.0%** |
| Front-end undelivered slots | **1.23%** |

D3 consequently reports **no classification**: the issued-work ratio is below its 70% warning threshold. walker-ladder-20261102T004502Z-summary.json[Recorded counter results](sandbox:/mnt/data/walker_ladder_review/walker-ladder-20261102T004501Z/walker-ladder-20261102T004502Z-summary.json)

This does **not** mean the compact walker can add another 43% of work for free. `UOPS_ISSUED.ANY` counts allocation from the register-allocation machinery to the reservation station; it is not a measurement of spare capacity in every execution resource. [GitHub](https://github.com/torvalds/linux/blob/master/tools/perf/pmu-events/arch/x86/skylake/pipeline.json?utm_source=chatgpt.com)

### The registered memory-overlap signature did not trigger

For the prefetched B rung:

| Quantity | 16 lanes | 32 lanes |
|---|---:|---:|
| Demand-associated pending-buffer ratio | 1.354 | 1.612 |
| Fill-buffer-full events per cycle | 0.123 | 0.164 |

The pending-buffer ratio increased by about 19%, rather than plateauing within 10%; the fill-buffer event rate increased but did not double. Thus D4’s specific plateau-and-pressure signature was **not observed**. walker-ladder-20261102T004502Z-summary.json[Recorded D4 result](sandbox:/mnt/data/walker_ladder_review/walker-ladder-20261102T004501Z/walker-ladder-20261102T004502Z-summary.json)

That is a failure to establish the proposed diagnosis—not proof that memory-resource constraints are absent.

### The event-specific safeguard mattered

For P32, the summary flags the L3-miss-stall event’s control/window ratio at approximately **0.62**, and the retired-load L3-miss ratio at approximately **0.47**. Those event counts are too close to the separate control measurements to support a strong claim about the underlying L3 behaviour. They were retained, qualified and **not subtracted**. walker-ladder-20261102T004502Z-summary.json[Control comparisons](sandbox:/mnt/data/walker_ladder_review/walker-ladder-20261102T004501Z/walker-ladder-20261102T004502Z-summary.json)

We should not turn these small counts into “everything fits in L3” or “cache capacity is irrelevant.”

## 5. What this changes in the research programme

My interpretation is:

**A generic rewrite of the shipped walker’s loop and lane bookkeeping is now a lower-value bet.** The reduced reference did not reveal a large single-core gap.

**The concurrent cost of doing the traversal remains important.** Even output-free kernels become substantially more expensive when four copies execute together.

**The precise physical explanation remains unresolved.** This experiment has narrowed the software-overhead explanation more strongly than it has distinguished the competing memory-system explanations.

There is also a scope limit to preserve: this is **one repeatedly walked, warmed 900,000-row block**, not complete decoding of the corpus. It cannot provide a full-decoder speedup estimate.

### What about FOR-14 `k` and `j`?

This capture neither rescues nor rejects those improved candidates. They were not measured.

The original FOR-14 loss remains unchanged. The compiled packed-state improvement remains a separate piece of evidence. But neither a 27-instruction listing nor the 57% issue-pressure figure is enough to predict whether `j` will win.

I would send these results to the ongoing Astra and researcher assessments, with the distinction:

> The existing walker shows little demonstrated single-core bookkeeping headroom, while concurrent copies still incur a large per-row cost. A compact successor must improve those economics without introducing too much additional execution work; this capture does not establish that it can.

## Next action

**No further workstation run is needed now.** Have Claude archive and independently reproduce this capture’s results, preserving the D1 decision and the scope limitations.

A suitable handoff is:

> The walker capture passes its registered gates. Please reproduce the analysis and archive a postmortem without changing thresholds or commissioning another run.
>
> Preserve the central result: P32 is 1.096× the best reduced 32-lane reference on one core, triggering D1’s no-scheduling-rewrite decision.
>
> Also report the substantial four-copy cost increase in A, B and P. Distinguish this from proof of a particular cache mechanism or reproduction of the full-decoder bottleneck.
>
> D3 and D4 do not establish a bottleneck classification. Retain the event-specific control warnings. State explicitly that M was not measured at four cores, so D5 does not transfer the P/M comparison.
>
> Keep the frozen FOR-14 result unchanged and `k`/`j` untimed pending the architectural assessment.

My independent_check.py[audit script](sandbox:/mnt/data/walker_ladder_review/independent_check.py) and independent-review\.json[recalculated results](sandbox:/mnt/data/walker_ladder_review/independent-review.json) are available alongside the original archive.

**This experiment has done something useful: it argues against spending the next substantial effort polishing the existing walker’s bookkeeping, while showing that the concurrency penalty persists even after much of the surrounding decoder has been removed.**
`````
