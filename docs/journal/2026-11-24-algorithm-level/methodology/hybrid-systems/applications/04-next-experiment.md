# The next experiment: use the existing traces to choose a structural change

This is an independent applied analysis plan. It does not claim that the unavailable chapters of Goebel, Sanfelice and Teel have been read or that a hybrid stability theorem proves decoder completion time.

## Decision and scope

**Decision:** does changing a boundary or ownership rule remove material whole-decoder work, or is it currently masked by another resource? Start with the already queued scanner plumbing and registered scanner direction. The five candidates are specified in [Structural levers](03-structural-levers.md).

The [active experiment programme](../../../briefs/experiment-programme.md) already contains the necessary sequence: stage 1, scanner plumbing, stage 2 interventions, public-API recalibration, joint frontier. This note supplies an analysis task inside that sequence, not another workstation grid.

## 1. Reconstruct a finite-job machine from existing evidence

Use the archived stage 1 ordered block traces (`blocks-r00..r03.tsv`) for event timing and per-block fields. Keep each run and block identity intact. The separate [service table](../../../math-targets/data/character-stage1/service.tsv) contains `round`, `k`, `rep`, `role`, `activity`, visit counts, `mean_ns` and `c2`: these are run/activity summaries, not per-block service vectors. Use them to check aggregate demand and variability; do not manufacture a block-level join or claim that a mean recovers visit order. The [report](../../../math-targets/data/character-stage1/report.txt) records 840 grid decodes passing conservation, ordering and residence identities.

Represent scanner position and mode, the ordered job queue, each worker's current block and residual service, ready results by index, next required output, the reader's current ownership, and retained allocations. Copy the actual admission predicate, including the $k+2$ floor. Preserve route/fallback costs and the original block sequence.

**Goal:** after finite valid input, consume all decoded output in order and release decoder-owned work. Corrupt input must reach the same specified error outcome. The input feeder, caller and scheduler must satisfy stated progress assumptions. A bounded occupancy trajectory alone can describe a deadlock and therefore does not establish this goal.

First replay events and accounting. Then predict event times. A replay driven by every observed completion timestamp validates bookkeeping only; it is not an independent performance prediction.

## 2. Test state reduction without assuming it

Compare an occupancy-only predictor with one that includes next-required-block status, residual work and configuration. Add measured contention or rate information only when it improves held-out prediction.

The reported mean admitted counts are 19.47 at eight workers and 18.73 at twenty; steady worker starvation rises from 3.7% to 56.1%. **Argued:** these means cannot explain the change alone. The exact same-count/different-next-result counterexample in the structural note explains why ordered state matters.

Hold out complete rounds and preserve the 431 distinct-block identities repeated four times; do not count repeats as independent workload samples. State explicitly whether measured service vectors are inputs to a scheduling replay or quantities the model predicts. Never feed the achieved service times of a held-out intervention into a claimed prediction of that intervention.

## 3. Make a prediction before the existing scanner test returns

For planning only, let $S_k$ be scanner elapsed service and $W_k$ mean worker elapsed service. Consider:

```text
estimated wall = max(S_k, W_k, reader demand, feeder demand) + gap_k
```

This is a conditional surrogate on rounded instrumented medians, not a physical lower bound. The gap is not established as a constant overhead.

At ten workers, subtracting a conjectured 15 ms from scanner service while retaining the observed 9 ms gap predicts 0.1843 s versus 0.1993 s, a 7.53% runtime reduction. At eight, worker demand immediately catches the scanner: the same calculation predicts only 0.1941 s versus 0.1944 s. Record both before evaluating the queued change.

If measured worker demand changes too, attribute that as evidence against the frozen-demand assumption rather than retroactively treating the larger gain as its prediction. Refit on the new baseline before judging worker-side ideas.

Keep the baseline identities separate. The [review decisions, section 5](../../../briefs/character-review-decisions.md#5-later-decisions) retain stage 2 on stage 1's master pin, `9621b7ad`; the scanner fix must not enter that study midway. Post-fix recalibration and later model updates use their own identified baseline. The rounded calculations above are new planning scenarios, not amendments to a frozen registration or claims that the queued implementation changes scanner demand alone.

## 4. Use existing stage 2 pulses to distinguish mechanisms

The adopted dynamical layer proposes scanner and A7 pacing over a contiguous block range, for example blocks 600–800. Use those runs to compare predicted and observed queue depletion, first starvation, ordered-output delay and recovery.

A scanner-rate model predicts depletion before worker starvation; a head-of-line explanation predicts ready results accumulating behind the missing index. A contention explanation predicts other activities' service changing during the pulse. Track these different observables, not wall time alone.

Boundary pacing is an intervention with a particular placement. It is not automatically equivalent to extra instructions or memory traffic inside the original activity. Label its achieved exposure and resource-release timing.

## 5. Exit with a construction or a rejected explanation

Accept a reduced model only if it predicts held-out event timing and queue behaviour adequately for the intended decision. Preserve the existing model-evaluation criteria and registrations. An error that changes which lever should be built matters more than an attractive aggregate fit.

Prioritise the existing scanner change. Advance a new credit or ownership construction only when its predicted benefit survives charged memory, transfers, fallbacks and reader semantics. Revisit emission after scanner improvement through the existing joint frontier.

Stage 1 is **PERTURBED at ten and twenty workers**; its recorded on/off ratios are 1.0398 and 1.0159 with wide intervals. Use these traces diagnostically and use the existing paired uninstrumented tests for deployed performance. A default worker/kernel or adaptive-mode change requires the already planned adoption registration.

The output of this task is a finite-job model, a falsifiable prediction and an implementation choice. No stability, robustness or invariance claim from an unread chapter substitutes for any of those.

