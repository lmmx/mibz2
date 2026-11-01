# External research report 106: measure receiver ownership and scheduling before any MPMC prototype (verbatim)

A research input, not a finding of this repository.

- **Relayed:** by the maintainer, 2026-11-01 12:49 UTC.
- **Source:** Astra.
- **In reply to:** the attribution archive, its erratum and report 105.

It is kept exactly as received (rendered text, cards flattened). Report 106 and 107 arrived
in one message, separated here at the relay's `guardian:` label.

## The message, as received

````text
**Measure receiver ownership and scheduling next; do not prototype MPMC yet.** The corrected backlog analysis weakens its causal case substantially. My reanalysis of the raw traces also finds scheduler evidence that the published 2.7% figure does not cover.

The longer-term investment should target **inverse-BWT memory-system cost**, initially preserving the four-byte representation. That is where the demonstrated CPU-cost opportunity lies. Neither the queue nor a particular replacement representation has yet earned an implementation recommendation.

I inspected the pinned [attribution archive and erratum](https://github.com/lmmx/mibz2/blob/33e0e4ebac554bae03795bfb76e86440b33bae6b/docs/journal/2026-11-01-decoder-attribution-i9-10900K/analysis.md), report 105, relevant source and analysis scripts, raw counter CSVs, and all ten four-worker full traces. The compressed traces were accessible, and I reproduced the queue tables. I did not independently reprocess the binary `perf.data` files; the symbol-level attribution remains the archive’s result.

**The queue evidence supports delayed service, but does not yet identify receiver ownership as its cause.**

The worker’s relevant sequence is:

```text
publish previous result; return symbols; notify
    ↓
JobLock
    ↓ acquire receiver mutex
JobRecv
    ↓ recv returns
release receiver mutex
    ↓
JobGot
    ↓ acquire shared-state lock; obtain output buffer
DecodeStart
    ↓ decode
DecodeEnd
```

This ordering is explicit in [the archived worker loop](https://github.com/lmmx/mibz2/blob/33e0e4ebac554bae03795bfb76e86440b33bae6b/src/parallel.rs#L415).

If three workers are genuinely between `DecodeStart` and `DecodeEnd`, **none of those three owns the receiver mutex**. The fourth worker therefore cannot, at that instant, be prevented from acquiring it by one of the decoding workers.

There are nevertheless two concrete histories to distinguish.

1. **A genuine receiver convoy requires multiple workers outside decoding.** Worker A owns the mutex and sleeps in `recv`; B waits for the mutex. Jobs arrive while A is delayed. B cannot receive them even if B could otherwise run. A must return and unlock before B can proceed. MPMC could remove this ownership dependency.

2. **A three-decoding-worker snapshot can inherit an earlier contention delay.** Previously A owned the receiver and D waited for it. A received a job, unlocked, woke D, and entered decoding alongside B and C. D is now runnable but has not run. Another job arrives, but D has not reached `recv` and is not a channel waiter. An already-running worker may finish and acquire the unlocked mutex first.

The second history fits the zero-backlog observation without requiring a mutex owner to be blocking D at enqueue time. A related possibility is that D was preempted immediately before acquisition—or after acquisition but before `JobRecv`.

Replacing the receiver might change these wake-up histories. It does not follow that it would eliminate their scheduling delay.

The backlog correction is decisive: **80% of three-active-worker observations had older unclaimed jobs**. Their approximately 977 µs median includes ordinary FIFO service waiting. Furthermore, “unclaimed” means “no `JobGot` recorded”; a receive may have completed before a preempted worker records that marker.

The zero-backlog split remains useful, but its interpretation changes:

| Last recorded state | What it establishes | What it does not establish |
|---|---|---|
| `JobRecv` | The worker acquired receiver ownership and reached the receive call | That it was asleep, runnable, or executing at enqueue |
| `JobLock` | It reached the next acquisition attempt | That another worker currently owns the mutex |

For the `JobLock` subset specifically, previous-result publication and symbol return already precede the marker. Output-buffer acquisition follows `JobGot`. Those operations cannot simply be invoked to explain this subset’s entire claim delay.

**There is additional scheduler evidence already in the archive.**

Using cumulative `wait_ns` and `cpu_ns` at successive `DecodeEnd` and `DecodeStart` markers, I obtained:

| Scope, pooled across ten executions | Elapsed worker time | Runnable-but-not-running time |
|---|---:|---:|
| Between successive decodes | 2.639 s | 0.884 s — **33.5%** |
| 120 distinct inter-decode gaps containing the 149 zero-backlog `JobLock` observations | 0.447 s | 0.348 s — **78.0%** |

These are ratios of pooled sums, not independent-sample confidence estimates. Crucially, the second row covers **the encompassing gaps**, not just enqueue-to-claim time. It does not prove that 78% of each observed claim delay is scheduler delay.

One concrete example occurs in round 0:

- `mibz2-0` records `JobLock` at 60.963 ms.
- Job 115 is enqueued at 66.644 ms, with three workers decoding and no older observed backlog.
- Another worker claims it at 67.158 ms.
- `mibz2-0` reaches `JobRecv` at 67.496 ms and claims job 117.
- Its encompassing 6.537 ms inter-decode gap contains **5.385 ms of run-queue wait**, and only about 35 µs of CPU time.

Across the zero-backlog cases, the initially nondecoding worker claims the job in **194/198 `JobRecv` cases**, but only **60/149 `JobLock` cases**.

This strengthens a scheduling/acquisition-history explanation. It does not distinguish earlier mutex blocking, wake-to-run delay and later acquisition competition precisely.

The published **2.7% run-queue figure is measured inside decoding intervals**. Its registered negative result does not exclude substantial run-queue delay during dispatch. Also, runnable delay is not automatically recoverable idle-core time: the scanner, consumer or other workers may occupy the allocation.

**The reader notification concern remains separate and less specifically diagnosed.**

`ReadWait` means the reader is inside the wait/predicate loop; it does not mean the kernel has put it to sleep. “Waiting while next output is ready” can include:

- publication followed by a delayed notification;
- wake-up processing;
- runnable-but-not-running delay;
- reacquisition of the shared-state mutex;
- execution between checking the predicate and recording `Taken`.

The worker records `Ready` **before** calling `notify_all()`. A preemption between those operations can therefore contribute to apparent reader lateness.

The shared condition variable can cause irrelevant wakes, but normally has two relevant waiter roles—the scanner and reader—not a four-worker herd. Their predicates are checked under the mutex, so this structure does not itself establish a lost-wakeup correctness bug.

The distinguishing evidence is:

```text
result published → notification → reader made runnable
                 → reader scheduled → shared lock reacquired → Taken
```

Prompt wake-up followed by delayed scheduling implicates scheduling. Prompt scheduling followed by delayed acquisition implicates the shared lock. Repeated wakes with a false predicate establish unnecessary notifications; they still need a material cost before justifying separate condition variables.

I would not change notifications in the receiver experiment.

**The corrected PMU interpretation is substantially right, with several remaining qualifications.**

I checked Intel’s [event definitions for the Skylake-family events used by model A5](https://github.com/intel/perfmon/blob/main/SKL/events/skylake_core.json).

| Evidence | Supported interpretation | Remaining uncertainty |
|---|---|---|
| Cycles/B +30.1%, instructions/B +1.3% | Substantial additional execution delay with similar retired instruction counts | Instruction counts do not measure transferred bytes, all kernel costs, or individual dependencies |
| Inversion owns approximately 78–83% of added CPU | Inversion is the principal CPU-efficiency target | Its individual memory mechanisms remain unresolved |
| Stable `OFFCORE_REQUESTS.ALL_DATA_RD` per byte | Similar counts in that mixed request population | Demand, prefetch and page-walk composition may change |
| Outstanding/request ratio 41 → 58 | Higher occupancy-derived residence proxy for those transactions | Not individual demand-load latency |
| Retired-load L3 misses rise | More retired loads have L3 misses as their source classification | Their hit ratio cannot describe all offcore requests |
| DRAM writes rise strongly | More write traffic reaches the memory controller during the measured window | Source buffers, eviction cause and deferred-writeback timing are unidentified |
| Traffic below sustained bandwidth | Aggregate bandwidth saturation is not demonstrated | Bank conflicts, read/write turnaround, queueing and latency pressure remain possible |

Two details particularly matter:

- `L1D_PEND_MISS.PENDING` includes prefetch-allocated fill buffers **after demand touches them**. Its conditional occupancy ratio is neither total prefetch occupancy nor total useful MLP. Approximately unchanged occupancy does **not** exclude an overlap/resource limitation.
- `FB_FULL` counts requests blocked by unavailable fill buffers, including loads, stores and software prefetches. It is not a count of distinct misses or directly a fraction of cycles.

The broad stall counters support load-related execution delay. They do not assign every coincident stall uniquely to memory. The hypothesis that demand loads wait on prefetched lines is credible; `FB_HIT` and suitable demand-latency/address samples would help, but neither alone proves that prefetching is harmful.

Likewise, **2.19 active T-vector phases is not measured LLC residency**. Older dirty lines can remain resident after a phase ends, and time averages hide expensive overlap bursts. That statistic defeats the simplistic “four tables always resident” account without defeating cache-capacity pressure.

My best-supported memory diagnosis is therefore: **concurrency changes the cost of serving inversion’s accesses, with substantial additional load-related stalls and memory-controller traffic**. Capacity, prefetch interactions, write allocation, dirty eviction and request scheduling remain competing and potentially interacting causes. The archive’s stronger wording—“capacity and uncore contention, not MLP”—is not established.

**The preferred next action is one bounded dispatch-mechanism capture.**

The hypothesis to test is:

> Receiver ownership or its serial wake handoff prevents a worker that could otherwise receive work from doing so for a material portion of queued-work delay.

This is more specific than testing whether a different queue happens to benchmark faster.

Minimum scope:

- Keep the current channel, receiver mutex, condition variable, worker count, admission window, affinity and power settings.
- Add diagnostic-only, preallocated per-thread markers around send, receiver acquisition, receive return and receiver release. Record OS thread IDs.
- Correlate those markers with `sched_waking`/`sched_wakeup`, `sched_switch`, and futex operations identified by address. Distinguish the receiver mutex, channel parking mechanism and shared condition variable.
- Include publication/notification markers to interpret the existing reader observation, without changing its notification policy.
- Avoid per-event `/proc` reads, allocation or a new shared logging lock.

This preserves block ordering, cancellation, error propagation, bounded admission, shutdown and memory ownership because it changes no dispatch semantics. The release of the receiver guard must remain at the same source-level point.

Use send/receive **brackets**, rather than replacing one supposedly exact timestamp with another. The channel’s linearization point remains inside the call. If a thread is preempted within a bracket, classify the timing as ambiguous unless scheduler evidence resolves it. A successful receive before the post-send marker is legitimate.

The mechanism decisions should be fixed before capture:

| Observation | Consequence |
|---|---|
| Jobs are available; a nonowner attempts reception; another worker retains receiver ownership while delayed | Confirms an ownership obstruction that MPMC can plausibly remove |
| Successive workers require serial receiver-mutex wakes before receiving a burst, despite available execution capacity | Supports the convoy hypothesis |
| Receiver is free through most delayed intervals and the relevant worker is runnable but unscheduled | Falsifies ownership as the main explanation for those intervals |
| Queue delay is mainly older-job service time, with no available worker | Rejects its interpretation as dispatch overhead |
| Delay occurs after `JobGot` or before the next `JobLock` | Assign it to buffer/publication/shared-lock work, not receiver ownership |
| A delay coincides with CPU idle exit | Supports investigating sleep-state effects; does not establish them from the governor name alone |

For a bounded decision framework, I would prespecify:

- **Start with three diagnostic repetitions**, expanding to ten only if events correlate reliably.
- Require explanations covering at least **90% of queued-work nondecoding time**; leave unresolved intervals explicit.
- Treat more than **3% tracing-induced change in throughput or CPU-s/GB** as a reason to reduce instrumentation before interpreting timing quantitatively.
- Promote MPMC only if receiver-specific obstruction is repeatable and material—for example, at least **5% of aggregate worker time**, with CPU availability and FIFO waiting separately accounted. That is an investigation threshold, not a predicted throughput gain.
- If receiver-specific obstruction is small and scheduling explains the tail, stop queue-replacement work.

Use the frozen Wikidata primary, a one-worker reference and existing correctness controls. Report throughput, CPU-s/GB and peak RSS for instrumented and uninstrumented executions. Trace storage must have a fixed, disclosed budget; instrumented RSS is not a production memory result. The previously problematic control timelines should not be interpreted without passing their perturbation checks.

A later implementation would need its own acceptance criteria: correct output and failure behavior, throughput improvement, nonincreasing CPU-s/GB and no peak-RSS regression. A diagnostic can succeed by rejecting MPMC without producing any speedup.

**The secondary architectural investment should be reducing inversion’s traffic while retaining inexpensive navigation.**

The observed additional inversion cost is approximately **0.68–0.72 CPU-s/GB**, roughly 19–20% of the four-worker total. Recovering a substantial fraction is consequential. It is also difficult: attaining 1.5× requires at least a 14% CPU-cost reduction even with perfect utilization.

The strongest concrete direction to keep in view is **avoiding full intermediate materialization between segment assembly and RLE1 consumption**, preserving the existing four-byte link/byte lookup. That avoids the extra navigation and construction costs which defeated pair walking and compact Ψ. It would require correct RLE state across segment boundaries and preservation of efficient output/CRC processing.

But the historical assembly measurements bound its **direct** benefit to a few percent. Its case for larger gains depends on an unproved external effect: fewer dirty intermediate lines must make concurrent inversion cheaper. Reduced write traffic without reduced integrated CPU cost would falsify that larger claim.

I would therefore investigate the source and phase timing of writeback/load pressure before constructing that pipeline. I would not launch another prefetch-hint sweep: previous i9 results already found no benefit from L2 hints and losses from disabling prefetch or using NTA.

Compact Ψ and N1 remain alternatives if traffic reduction proves insufficient. They must demonstrate total construction-plus-walk savings, including retained buffers and fallback distributions. N1’s simulated visit reduction is not a hardware-load or cycle reduction; its existing fused-builder model remains unvalidated.

Perfect occupancy at today’s CPU cost gives only **1.29×**. Recovering all observed scaling inflation down to the one-worker 2.707 CPU-s/GB, together with perfect occupancy, would give approximately **1.70×**—a conditional ceiling, not a forecast. Reaching 2× requires further efficiency beyond that baseline.

The handoff I would give Claude is:

> Prepare a diagnostic-only dispatch capture; do not replace the queue or split notifications yet.
>
> First reproduce the inter-decode scheduler accounting from the existing ten full traces: approximately 33.5% runnable delay across all inter-decode gaps, and 78% across the 120 distinct gaps containing zero-backlog `JobLock` observations. Keep these encompassing-gap measurements separate from enqueue-to-claim latency.
>
> Instrument send brackets, receiver acquisition, receive return and receiver release with preallocated thread-local records and OS TIDs. Correlate with scheduler and address-identified futex events. Preserve existing lock lifetimes and all decoder semantics. Record publication/notification boundaries without changing notification behavior.
>
> Test whether actual receiver ownership or a serial wake chain blocks an otherwise available receiver. Separate FIFO backlog, runnable delay, blocked time, publication and buffer acquisition. Do not count all nondecoding time as idle CPU capacity.
>
> Prespecify tracing-overhead limits, ambiguity handling and a materiality threshold before any separately authorized capture. Retain correctness checks and report throughput, CPU-s/GB and RSS. If the receiver is generally free during the problematic delays, stop MPMC work and report the scheduling mechanism instead.
>
> Keep inversion traffic reduction as the next architectural research direction. P74-C and the unrolled-walker experiments remain closed.

**The deciding observation is actual receiver obstruction with another usable receiver available.** Without it, MPMC would test a package of scheduling changes rather than the causal mechanism currently claimed.
````
