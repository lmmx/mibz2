# External research report 136: the guardian on the FOR-14 i9 confirmation and the walker counter audit: a decisive loss, and a validation gap in the ladder's driver (verbatim)

A research input, not a finding of this repository.

- **Relayed:** by the maintainer, 2026-11-01, with reports 136–139 together.
- **Source:** the guardian.
- **In reply to:** the FOR-14 i9 confirmation (lmmx/mibz2#130, §7a) and the walker counter audit (lmmx/mibz2#131).

**Evidence cutoff:** 1 November 2026, after the FOR-14 i9 confirmation and the counter audit, before the ladder's regrouping was reviewed. A review and instructions: no new capture is authorized.

It is kept exactly as received (rendered text, cards flattened, tracking parameters
included).

## The message, as received

`````text
**The frozen FOR-14 implementation loses decisively on the i9. The counter audit also did its job: the events are available, but none of the proposed event bundles can currently be collected without multiplexing.** These are separate outcomes—the counter problem does not invalidate the FOR-14 timing result. for14-analysis walker-audit-20261101T232121Z

I checked the uploaded raw rows and independently recalculated the central paired walk ratios. They agree with the reported analysis.

## 1. FOR-14: this is a substantial loss, not a marginal failed gate

Here, `h` is the frozen FOR-14 implementation and `o` is its conventional-table control.

| Measured walk comparison | FOR-14 speed relative to control | Throughput difference | Equivalent time for the same work |
|---|---:|---:|---:|
| One core: `h1/o1` | **0.475×** [0.473, 0.477] | **52.5% lower** | **2.10× as long** |
| Four cores: `h4/o4` | **0.688×** [0.682, 0.693] | **31.2% lower** | **1.45× as long** |

The time multipliers are reciprocals of the measured speed ratios. Both intervals are far below parity. for14-analysis

The control also passed the registered comparison against the production walker:

- `o1/p1`: **1.005×**.
- `o4/p4`: **0.991×**.

Both calibration gates passed, and the run records all six arms passing their output checks. This is not a case where a badly mismatched control explains the apparent loss. for14-analysis for14-console

### We no longer need the disputed stopping bound to see the problem

The frozen analysis emits its `STOP` using the conditional one-worker threshold of 0.774. Preserve that verdict exactly. But the measured **four-core loss itself** is now sufficient reason not to build a production constructor for this implementation. We don't need to assert that four-worker cost can never fall below one-worker cost to reach that engineering decision. for14-analysis

That is a stronger position than we had before this run:

> **This particular FOR-14 walker is demonstrably slower under the tested four-core conditions—not merely predicted to be slower from its instruction count.**

### The forecast was too optimistic

The measured one-worker ratio, **0.475×**, is worse than Claude's earlier 0.59–0.66 estimate and below the previously stated 0.50–0.70 prediction range. The record should therefore say that the model predicted the direction correctly but underestimated the loss—not that its quantitative calibration was confirmed. for14-analysis

This matters for the continuing instruction-optimization research: **raw instruction count is still not a calibrated performance model.**

## 2. There is useful information in the smaller four-core loss

FOR-14 goes from 0.475× the control at one core to 0.688× at four. Its **relative disadvantage narrows under concurrency**. for14-analysis

That is consistent with the two implementations responding differently to the concurrent memory environment. A smaller working set is one plausible contributor. But this run did not collect the counters needed to establish that explanation, and different execution bottlenecks could also produce different scaling.

The defensible conclusion is:

> FOR-14 scales better relative to this control than its isolated performance suggests, but not nearly enough to overcome its higher navigation cost.

This is not a vindication of the format, but neither is it evidence that footprint reduction has no value.

### Don't mistake the combined-phase result for a decoder result

The reported four-core **build-plus-walk** ratio is 0.875×, which looks less severe than the walk-only loss. However, FOR-14's “build” is an optimistic copy of **precomputed records**, not a direct compact-table constructor. The real record-construction work occurs outside the timing. for14-analysis 

Consequently:

**“12.5% slower in this build-plus-walk harness” is not a measurement of a production FOR-14 decoder.** The report explicitly excludes MTF and identifies the boundary as the T-vector phase, not full decompression. for14-coop-20261101T231824Z

## 3. What this means for Astra and the new researcher

**Give both agents these measurements now. I would not cancel their already-running, bounded rescue assessments.**

Their task has become much more concrete: determine whether there is a *substantially cheaper implementation*, rather than speculate about whether the current implementation might succeed on the i9.

From the observed four-core ratio, the candidate would need approximately

\[
\frac{1}{0.6877}-1 \approx 45.4\%
\]

**more walk throughput just to reach the control**, before accounting for a real compact constructor. That is calculated from this run, not a universal requirement for every future representation. for14-analysis

That sets a serious bar for any rescue proposal. A credible response should identify a concrete change to extraction, lane state, dependencies or layout—not merely predict that a slightly shorter instruction listing will suffice.

My disposition would be:

**Stop this frozen implementation and its direct-builder work. Let the two existing research assessments finish. Consider another candidate only if they identify a materially different, testable execution path.**

That preserves the value of their work without quietly turning a clear negative result into an unlimited optimization exercise.

## 4. The counter audit: available events, unsuitable bundles

The audit reports **all 22 requested events available and counting individually**. But every proposed bundle—`exec`, `mlp`, `stalls`, `pf_tlb`, and `loads`—is multiplexed. The recorded running percentages range from approximately **65% to 86%**. walker-audit-20261101T232121Z walker-audit-20261101T232121Z

This is not evidence that the i9 lacks the required profiling facilities. It means the current collection arrangement cannot keep all those requested events counting continuously.

With multiplexing, Linux schedules events onto the available hardware counters for portions of the measurement. That can introduce errors when workload behaviour changes during collection—particularly when deriving ratios from events that were not measured over the same intervals. The perf documentation recommends grouping related events and notes both counter-assignment restrictions and counter reservations. [man7.org](https://man7.org/linux/man-pages/man1/perf-list.1.html)

Your audit records `nmi_watchdog=1`. The watchdog can reserve a hardware counter, making it a relevant possible contributor—not a demonstrated explanation for every multiplexed bundle. I would first have Claude reduce and re-probe the bundles rather than immediately change your workstation's watchdog setting. walker-audit-20261101T232121Z [man7.org](https://man7.org/linux/man-pages/man1/perf-list.1.html)

**The next ladder capture should use smaller, verified combinations.** Events used together in a ratio should be collected simultaneously where the hardware permits. If a required pair cannot be scheduled together, the protocol must state that limitation rather than silently manufacture a simultaneous metric from unrelated passes. Linux event groups are specifically designed to schedule their members together. [man7.org](https://man7.org/linux/man-pages/man2/perf_event_open.2.html)

## 5. I found one concrete validation gap before the ladder freeze

I inspected the pinned `bench/walker_ladder.py`, not just the audit summary.

**The four-core collection path does not retain the counters' running percentages, and the final multiplexing validity check examines only the single-core records.**

In the code, the four-core path saves counter values as `totals_with_setup`, while the summary checks:

```python
any(x.get("multiplexed") for x in one)
```

Here, `one` explicitly excludes four-core records. Thus a four-core measurement could multiplex without triggering that validity check. 

This has **not** corrupted a completed ladder experiment—the ladder capture has not happened. It is a specific validation fix to make before freezing it.

I would require the same event-completeness and running-percentage checks across single-core runs, four-core runs, and the setup-only runs used for subtraction. The existing driver also submits comma-separated event lists; those are not automatically simultaneous perf groups merely because the Python dictionary calls them “groups.”  [man7.org](https://man7.org/linux/man-pages/man1/perf-list.1.html)

## What to send Claude

> The frozen FOR-14 i9 confirmation is complete. Please archive the raw rows, console and analysis unchanged, and reproduce the registered result.
>
> The measured walk ratios are h1/o1 = 0.475 [0.473, 0.477] and h4/o4 = 0.688 [0.682, 0.693]. Both control calibration gates passed. Preserve the STOP verdict, but distinguish the conditional one-worker bound from the directly measured four-core regression. Record that the one-worker prediction was optimistic.
>
> Do not implement a direct builder or rerun the frozen candidate. Send these results to the ongoing Astra and researcher rescue assessments. Any successor must identify a substantially cheaper execution path and receive separate authorization.
>
> The walker audit found all events countable individually, but every proposed bundle multiplexed. Prepare smaller compatible event groups and a revised availability probe; retain raw perf output and running percentages.
>
> Please also fix the pinned driver's four-core validation gap: the four-core path currently omits running percentages, and the summary checks multiplexing only in single-core records. Apply completeness and multiplexing validation consistently to all relevant measurements, including setup-subtraction runs. Add regression tests demonstrating rejection of a multiplexed four-core capture.
>
> Use actual perf grouping for counters that must be measured simultaneously, subject to hardware compatibility. Do not silently substitute cross-run ratios or change workstation settings.
>
> Return the revised event plan and validation results before the timed walker-ladder capture. No new performance capture is authorized by this message.

**Nothing further needs running on your i9 right now.** We have a clear negative result for the frozen FOR-14 implementation, a stronger numerical target for the ongoing rescue work, and a specific collection problem to fix before the hardware-ceiling experiment.
`````
