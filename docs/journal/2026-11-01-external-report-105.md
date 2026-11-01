# External research report 105: the attribution archive: a receiver-ownership convoy, and the PMU populations (verbatim)

A research input, not a finding of this repository.

- **Relayed:** by the maintainer, 2026-11-01 12:01 UTC.
- **Source:** the researcher (an independent assessment), and the guardian's reviews of it.
- **In reply to:** the i9 attribution run.

It is kept exactly as received (rendered text, cards flattened). Paths to uploaded
files are the maintainer's.

## The message, as received

`````text
hmm, I hope you have been recording these interactions there is some potential gold in here. And likewise this one I just got from the researcher:


````
# Independent research assessment

I examined the uploaded i9 archive, including `result.json`, the hardware-counter definitions, occupancy reports, and the ten raw four-worker execution timelines. I also checked the shipped queue and inverse-BWT implementations against the earlier workstation experiments.

My conclusion differs from Claude's emphasis: shared-cache pressure is a credible explanation for the CPU inflation, but the data contain unusually strong evidence of a second, software-level opportunity: the worker queue serializes blocking receivers, and the reader/scanner share a condition variable despite waiting for different conditions.

That is worth investigating before committing to N1 or another compact Ψ representation.

More importantly, the experiments establish a quantitative requirement for the longer-term breakthrough.

## 1. What 1.5–2× actually requires

The archive measures approximately 869 MB/s and 3.579 CPU-seconds per GB at four workers, implying around 3.11 occupied CPU cores.

On four physical cores, for throughput \\(X\\) and CPU cost \\(C\\),

\\[ XC\le4. \\]

At the present CPU cost, the theoretical throughput ceiling is only

\\[ \frac4{3.579}=1.118\ \mathrm{GB/s}, \\]

approximately 1.29× current throughput, even with perfect utilization.

Therefore:

| Target | Required throughput | Maximum CPU-s/GB | Required CPU-cost reduction |
| ------ | ------------------- | ---------------- | --------------------------- |
| 1.5×   | 1,304 MB/s          | 3.07             | ≥14.3%                      |
| 2.0×   | 1,739 MB/s          | 2.30             | ≥35.7%                      |

These are optimistic physical bounds, excluding unavoidable scheduling and other overhead.

Consequence: fixing thread wake-ups alone cannot achieve 1.5× at unchanged decoder work. We need both better core utilization and a genuine reduction in CPU work or memory stalls per byte.

## 2. New finding from the raw queue trace

I reconstructed the enqueue-to-claim timing for 434 blocks across each of the ten four-worker full-instrumentation rounds: 4,340 job observations.

At each enqueue event, I counted workers inside `DecodeStart`–`DecodeEnd` intervals.

| Workers actively decoding when job enqueued | Share of jobs | Median claim delay |
| ------------------------------------------- | ------------- | ------------------ |
| 0                                           | 9.0%          | 49.5 µs            |
| 1                                           | 17.2%         | 48.4 µs            |
| 2                                           | 30.1%         | 61.6 µs            |
| 3                                           | 40.7%         | 977 µs             |
| 4                                           | 3.0%          | 1,221 µs           |

Independent calculation from `occupancy/wikidata-20261026/marks/r00–r09-t4-full.tsv.gz`. Ten repeated executions of one corpus, not 4,340 statistically independent experiments. A worker outside `decode_block` is not necessarily runnable or available.

The overall median is 112.5 µs and the 90th percentile is approximately 2.12 ms.

The three-active-worker median is particularly interesting: across all ten rounds, it ranges from approximately 590 to 1,229 µs. A job can therefore wait roughly a millisecond while one worker is not actively decoding.

This warrants a specific scheduling explanation.

### The source-level mechanism

Current `src/parallel.rs` effectively does:

```
let job = {
    let rx = jobs.lock().unwrap();
    rx.recv() // blocking while holding the shared mutex
};
```

All workers share one `Mutex<Receiver<Job>>`.

Only one worker can wait inside `recv()` at once. The others may be asleep waiting for the mutex. A newly queued job cannot be claimed by those workers until the receiver owner wakes, returns, and releases the lock.

This can cause a receiver-ownership convoy. It is more specific than saying the CPU's power-saving governor causes slow wake-ups.

The archive supports the opportunity:

- Workers spend about 19.7% of their time in the instrument's receiver-lock waiting state.
- Another 9.5% is spent inside receive waiting.
- About 11.1% of aggregate worker time is classified as idle while jobs are queued.

What's not yet proven: how much delay an MPMC queue would eliminate. Some delay can come from worker scheduling, buffer acquisition or CPU sleep states rather than the mutex itself.

## 3. The cache diagnosis needs one important correction

Claude's counters support memory-related slowdown, but not yet a unique cache-capacity explanation.

Average simultaneous T-vectors, four-worker trace

# 2.19

≈7.9 MB of table storage

Peak simultaneous T-vectors

# 4

14.4 MB of table storage

The i9 has 20 MB shared L3. Other buffers, recent writes and prefetch activity consume capacity too, so these numbers do not disprove cache pressure. But they rule out the simplistic explanation that four full T-vectors are necessarily resident throughout the decode.

A second counter distinction is especially important:

| Four-worker counter            | Events per output byte |
| ------------------------------ | ---------------------- |
| `offcore_requests.all_data_rd` | 1.420                  |
| `mem_load_retired.l3_hit`      | 0.01551                |
| `mem_load_retired.l3_miss`     | 0.00361                |

These event populations differ by roughly 74×.

Intel defines `OFFCORE_REQUESTS.ALL_DATA_RD` as including demand and prefetch requests, while the retired-load events classify load instructions by their data source.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://perfmon-events.intel.com\&sz=32)

PerfMon Events Documentation+1



Therefore the reported 81% L3 hit share is not the hit ratio for all 1.42 offcore requests per byte. And the 41→58 cycles/request statistic is an outstanding-request occupancy proxy, not direct evidence that individual demand loads became 41% slower.

My competing hypothesis: software prefetch traffic and streaming writebacks compete with the T-vector for cache and uncore resources. Capacity pressure initiates the problem, but prefetch timing and write allocation may amplify it.

The jump from 0.6 to 6.9 DRAM-write bytes per output byte is consistent with substantial additional dirty-line eviction. The archive does not identify exactly which buffers produce those writes.

This matters because shrinking Ψ is not necessarily the cheapest way to reduce that pressure.

## 4. The overlooked reader-notification design

The shared `Condvar` in `src/parallel.rs` serves two logically different conditions:

- The reader waits for the next ordered output block.
- The scanner waits for an available admission slot.

When a worker publishes a completed block, it calls `notify_all()`. When the reader takes one, it also calls `notify_all()`.

Consequently, both types of waiter may awaken for a transition irrelevant to their own condition. That can produce unnecessary wake-ups, lock acquisitions and scheduler activity.

A narrower implementation would use separate notifications for ready output and freed slots. It preserves the existing ordering and bounded-capacity invariants.

I would not combine this with the job-queue replacement in the first experiment: doing both would obscure which mechanism caused the improvement.

The main uncertainty is whether the observed reader lateness is notification overhead or simply delayed scheduling on four shared cores. The raw trace cannot yet distinguish them.

## 5. Research ranking, incorporating previous negative experiments

| Rank | Candidate                                                                              | Evidence and judgment                                                                                                                                                    |
| ---- | -------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| 1    | Replace the blocking shared-receiver mutex with a genuine multi-consumer job queue     | Concrete source-level mechanism; substantial queued-work idle time; low implementation cost. Best immediate engineering experiment.                                      |
| 2    | Reduce inverse-BWT prefetch/copy/dirty-line traffic without changing Ψ's 4-byte layout | Strongest route to reducing CPU-s/GB, but the specific traffic source is unresolved. Better immediate research direction than another packed table.                      |
| 3    | Separate reader-ready and scanner-slot notifications                                   | Cheap, plausible scheduling improvement. Test separately after queue ownership is resolved.                                                                              |
| 4    | New compact/composed Ψ representation, including N1                                    | Potentially much greater upside, but construction costs and extra dependent operations remain serious barriers. Requires a demonstrated total build-plus-walk advantage. |

Historical experiments narrow this ranking substantially.

The 32-lane unrolled walk achieved an 8–13% one-copy walk improvement on Zen, but its subsequent i9 full-decoder result at four workers was 1.000 [0.987, 1.012]. That is direct evidence against assuming faster isolated traversal transfers to the deployment configuration. [Unrolled walk decoder result](https://github.com/lmmx/mibz2/blob/docs/walk-unroll-decoder-records/docs/journal/2026-11-01-walk-unroll-decoder-i9-10900K/results.md?utm_source=chatgpt.com)

The earlier 20-bit Ψ pilot had costly construction and slower traversal, while the compact Ψ experiments already demonstrated substantial per-step overhead. Those failures reject the tested representations, not every possible compact index.

Likewise, the historical phase-aware inversion semaphore helped at 8–10 workers but never exceeded the existing four-worker throughput peak. We should not revive simple inversion throttling as a novel solution.

The previous segment-assembly experiments also limit the expected direct benefit of eliminating copies to a few percent. Their possible indirect effect on cache contention remains less established, but not enough to make fusion my first investment.

### 6. The single experiment I would select

Hypothesis: the blocking ownership of `Mutex<Receiver<Job>>` is responsible for a material part of the queued-work idle time.

Prototype a true MPMC receiver, for example using `crossbeam-channel`, where workers can independently block waiting for jobs. Keep the admission window, worker count, scanner, reader, BWT kernel and output ordering unchanged.

| Criterion   | Proposed early gate                                                                                        |
| ----------- | ---------------------------------------------------------------------------------------------------------- |
| Correctness | Identical output, CRCs, errors and bounded admission                                                       |
| Mechanism   | `w.idle.jobs_queued` falls materially from \~11.1%; queue-delay tail contracts                             |
| Performance | ≥5% four-worker Wikidata throughput gain as a worthwhile engineering target                                |
| Economics   | CPU-s/GB non-increasing within measurement uncertainty; no material RSS/control regression                 |
| Failure     | Queue waits improve but decoder throughput does not, or waits do not improve despite independent receivers |

That is a new experiment, not something demonstrated by the archive. No implementation or benchmark has been performed here.

A minimal MPMC replacement is plausibly a small engineering task compared with implementing N1. Importantly, if it fails, the result cleanly redirects attention to the reader wake-up path or memory contention.

## 7. What I'd specifically pass to Astra

I would add four challenges to Guardian's proposed briefing:

A. Queue ownership is a concrete design issue, not merely a sleeping-thread hypothesis. The exact source holds `Mutex<Receiver<Job>>` across blocking `recv()`. The raw-trace conditional queue-delay analysis finds \~977 µs median delay with three of four workers actively decoding, repeated over ten rounds. Ask Astra whether MPMC dispatch is the highest-value low-risk prototype.

B. Cache attribution must separate demand loads from prefetch traffic. Stable `offcore_requests.all_data_rd` does not establish stable demand-load traffic. Ask how the software-prefetched room walker can generate nearly unchanged offcore requests but substantially more stalled cycles and writebacks at four workers.

C. The average table residency is smaller than the peak. Only 2.19 T-vectors are live on average. Ask whether the existing evidence supports LLC capacity alone, or a combined model involving eviction, prefetches, dirty writes and phase overlap.

D. Require a four-core CPU-efficiency model. For a genuine 1.5× breakthrough with four cores and non-increasing CPU-s/GB, at least \~14% CPU-cost reduction is required even under perfect utilization. An optimization that merely removes wake latency cannot satisfy that target.

### Final investment decision

I would authorize one bounded MPMC-queue experiment first, because it has a concrete mechanism and a favorable cost-to-information ratio.

But I would not mistake scheduling for the entire breakthrough. The more consequential longer-term research question remains:

Can we remove most of the roughly 0.68–0.72 CPU-seconds/GB of additional inversion cost that appears when scaling to four workers, without making single-worker BWT traversal slower?

That is approximately 19–20% of the current four-worker CPU cost. Recovering much of it, alongside better utilization, could make the 1.5× objective plausible. Reaching 2× demands deeper changes even beyond that.

For Astra: give it this queue evidence and the counter-population discrepancy alongside Claude's original archive. Ask it to adjudicate the cheap MPMC prototype versus a targeted inversion-traffic mechanism—not simply choose between “compact Ψ” and “cache contention.”
````


The guardian says:


````
The researcher has found something genuinely useful that changes our next decision. I would definitely give this to Astra before authorizing another experiment.

The most valuable contribution isn't merely recommending a new queue. It's identifying a specific, testable software mechanism that the earlier attribution analysis hadn't isolated:

```
let rx = jobs.lock().unwrap();
rx.recv() // Blocks while retaining the receiver mutex
```

That means only one worker can block directly on the channel receiver at a time. Other workers may be waiting for the mutex rather than independently waiting for jobs.

Combined with the reconstructed queue-delay distribution, this makes MPMC dispatch a credible, relatively inexpensive experiment.

But I wouldn't yet declare the mutex responsible for the observed delays.

## The strongest new evidence

| Observation                                            | Implication                                                                                         |
| ------------------------------------------------------ | --------------------------------------------------------------------------------------------------- |
| Median job-claim delay with 3 workers decoding: 977 µs | A fourth worker not decoding does not necessarily claim queued work promptly                        |
| Around 11% of worker time idle with jobs queued        | There is potential wasted worker capacity                                                           |
| One receiver mutex held across blocking `recv()`       | Concrete candidate mechanism for serialized wake-ups                                                |
| Only 2.19 T-vectors simultaneously live on average     | Cache contention deserves more nuanced analysis than assuming four large tables are always resident |
| Offcore requests include prefetches                    | The earlier demand-load interpretation requires correction                                          |

The researcher also caught an important architectural constraint: even eliminating all idle time cannot deliver 1.5× throughput at today's four-worker CPU cost.

So the MPMC idea is potentially the best next experiment, not a complete answer to the 1.5–2× objective.

## One point I'd ask Astra to scrutinize

The researcher's source-level observation is sound, but a queue-delay measurement does not directly measure mutex contention.

A worker outside `DecodeStart`–`DecodeEnd` could be executing other work, waiting for a scratch buffer, descheduled, or transitioning between jobs. And counting active workers at enqueue time doesn't prove that the fourth worker was available throughout the queue delay.

There is also an important distinction between serializing access to the receiver and serializing processing. The existing mutex does not serialize the decoding itself; it serializes which worker can wait for and receive a job.

These distinctions don't weaken the case for a bounded MPMC experiment. They make it a better experiment: we can test the proposed mechanism rather than assuming the diagnosis is settled.

## What to send Astra

I'd keep the previous architectural prompt you prepared, but add this researcher's findings and explicitly ask for an independent adjudication. You could use this as the complete next message in the existing Astra session:

We have a new independent research review of the shipped-decoder attribution archive. Before we authorize another engineering experiment, I'd like your independent architectural adjudication.

The frozen diagnostic archive and Claude's reconciliation are committed to `docs/decoder-attribution-records` (`c269245`). Please inspect the relevant source, raw traces, counters and analysis.

**The researcher's central new finding:**

The shipped worker dispatch uses a shared `Mutex<Receiver<Job>>`, with the mutex held across blocking `recv()`. Thus only one worker can wait directly in the receiver at a time; other workers may block waiting for receiver ownership.

From ten four-worker Wikidata execution traces (4,340 job observations), the researcher reconstructed enqueue-to-claim latency conditioned on how many workers were actively decoding at enqueue:

- 0 decoding: median 49.5 µs
- 1 decoding: median 48.4 µs
- 2 decoding: median 61.6 µs
- 3 decoding: median 977 µs
- 4 decoding: median 1,221 µs

The 3-active-worker result is particularly interesting. In addition, around 11% of aggregate worker time is classified as idle with queued jobs, and workers spend substantial time waiting for receiver ownership.

The researcher recommends replacing the shared blocking receiver with a genuine MPMC queue as the next small, separately measured experiment.

**Please challenge the causal inference.** A worker outside the decoder interval is not necessarily available to receive a job, and the existing mutex serializes reception, not decoding. Examine whether the source and timelines actually support a receiver-ownership convoy or whether worker availability, scheduler latency, buffer ownership or thread wake-up behavior are better explanations.

**A second queue-related proposal:**

The scanner and reader share a condition variable while waiting for different predicates: admission slots and ordered output availability. Splitting the notifications might reduce unnecessary wake-ups. The researcher proposes testing this separately from MPMC dispatch.

**Important correction to the memory interpretation:**

- Average concurrent T-vector residency in the four-worker trace is approximately 2.19 tables (\~7.9 MB); the observed peak is four tables (\~14.4 MB), against a 20 MB shared L3. Other buffers and cache traffic still matter.
- `offcore_requests.all_data_rd` includes demand and prefetch requests; retired demand-load L3-hit/miss events measure a different population.
- Therefore the 41→58 cycles/request occupancy-derived statistic is not a direct measurement of individual demand-load latency.
- Software prefetches, dirty-line eviction and intermediate-buffer writes may interact with cache capacity.
- Four-worker DRAM writes increase from 0.6 to 6.9 bytes per output byte, but their source is unproven.

Please inspect the exact PMU definitions before drawing conclusions.

**The quantitative target:**

At about 869 MB/s and 3.579 CPU-s/GB, four-core throughput cannot exceed approximately 1.118 GB/s at unchanged CPU cost, even with perfect utilization.

A 1.5× gain requires CPU cost no greater than approximately 3.07 CPU-s/GB (at least 14% lower). A 2× gain requires approximately 2.30 CPU-s/GB (at least 36% lower). Thus queue optimization alone cannot meet the eventual target.

**What I need you to decide:**

1. Is MPMC worker dispatch the best next experiment, or does the evidence support a more decisive, cheaper diagnostic first?
2. What is the smallest source-level change that would test receiver ownership without accidentally changing admission, output ordering, buffering, or worker behavior?
3. Would an MPMC queue introduce new contention, wake-up, CPU-cost or memory-allocation risks? How should those be bounded?
4. Should separate reader/scanner notifications be investigated afterward, or is there a more fundamental synchronization issue?
5. Does the corrected PMU interpretation change your ranking of inverse-BWT memory-traffic approaches versus N1 or compact representations?
6. What experiment has the highest expected information value and plausible production benefit, and what observations would falsify its proposed mechanism?

Please give one clear recommendation, with an implementation sketch, predicted observable signature, early stop criteria and the measurements needed to support a production decision.

Do not implement, modify the repo, run benchmarks or launch CI. This is a read-only architectural review. The prior negative experiments remain closed.

## My preference before hearing Astra

Near term: investigate MPMC dispatch first.

It's a specific source-level hypothesis with plausible upside and relatively limited implementation cost. It deserves a clean experiment, with latency distributions and CPU cost measured together.

Longer term: focus on reducing BWT memory-system costs.

The evidence still suggests this is where a major throughput gain must come from. But we've not established whether the best mechanism is fewer bytes moved, better prefetch timing, reduced cache footprint, or less traversal work.

I particularly like the researcher's insistence on testing the MPMC queue and condition-variable split separately. Combining both would make a positive result difficult to interpret.

I'd send this to Astra now. We've gained precisely what we wanted from the independent researcher: a concrete alternative to the existing architectural shortlist, plus corrections to the cache interpretation. Astra now has something substantive to adjudicate rather than simply ratifying Claude's earlier conclusions.
````


I asked the guardian

> shall I send your response back to the researcher for feedback, or just to Claude?

and it said


```
I'd send it to Astra next, not Claude—and I wouldn't send it back to the researcher for another full review.

We've already achieved what we wanted from the independent researcher: a genuinely new hypothesis, a quantitative analysis of the raw traces, and a challenge to our interpretation of the memory counters.

Another round with the researcher risks turning independent analysis into iterative consensus-building. I'd preserve that intellectual diversity by letting Astra evaluate the findings independently.

My preferred sequence is:

1. Astra now: Send the architectural-adjudication prompt from my previous response, including the researcher's MPMC findings. Ask Astra to challenge the hypothesis, not merely endorse it.
2. Claude afterward: Give Claude Astra's recommendation alongside the original researcher's findings. Claude can check source-level feasibility and prepare a bounded experiment.
3. Researcher again only if needed: If Astra identifies a serious disagreement or proposes a fundamentally different mechanism, ask the researcher to examine that specific contention.

One refinement: give Astra the researcher's original report as well as my prompt, if convenient. My prompt distills the findings, but Astra should have access to the original reasoning and evidence rather than only my interpretation.

We now have three perspectives: Claude's measurements, the researcher's independent interpretation, and Astra's forthcoming hardware-level adjudication. That's enough scrutiny to make the next decision without unnecessarily extending the review cycle.

So my vote: Astra next, Claude after Astra.
```
`````
