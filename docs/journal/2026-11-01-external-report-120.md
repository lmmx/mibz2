# External research report 120: the guardian agrees with the researcher: two tracks, the register line (verbatim)

A research input, not a finding of this repository.

- **Relayed:** by the maintainer, 2026-11-01 15:36 UTC.
- **Source:** the guardian.
- **In reply to:** report 119.

It is kept exactly as received (rendered text, cards flattened).

## The message, as received

````text
Ah, yes—that's the researcher's actual response, and it makes an important distinction. We've ruled out the receiver mutex as a sufficiently large direct bottleneck, but we haven't ruled out the scheduling consequences of the way workers hand off work.

I agree with the researcher's assessment.

What I find particularly compelling is that the scheduling hypothesis is now quite specific: workers are frequently awakened onto the same CPU as the thread that woke them, sometimes remaining runnable for milliseconds while another allocated CPU is idle.

That is a much stronger starting point than simply observing that workers spend time waiting.

### The scanner finding is especially useful

The researcher has also corrected how we should interpret the scanner's delay.

Scanner state during worker no-job time

Running and scanning

43.7%

Runnable, window full

23.2%

Runnable, window open

19.2%

Asleep, window full

7.2%

Categories reported by the researcher; the remaining 6.7% belongs to other states. Shares are of worker no-job time, not total decoder runtime.

The most actionable category is scanner runnable with an open admission window. That accounts for approximately 3.6% of aggregate worker time. It's a plausible source of worker starvation, whereas waking the scanner while the admission window is full may achieve nothing.

This gives us a better way to assess any future scheduling experiment: it should reduce useful-work starvation, not just reduce some generic wake-up latency statistic.

## My recommendation

I'd maintain two distinct tracks:

Primary — inverse-BWT memory efficiency

Investigate the measured 0.68–0.72 CPU-s/GB scaling penalty. This is the more substantial route toward the 1.5–2× objective, particularly if we can reduce cache and memory interference without paying extra traversal costs.

Secondary — four-core wake placement

Preserve a narrowly scoped affinity experiment as a candidate: the same four physical cores, same number of workers, existing decoder, with only thread placement altered. Measure whether both useful-work starvation and decoder-level performance improve.

I would not send Claude another long instruction yet if you've already sent the consolidated BWT-memory proposal. The researcher's findings can be added to the dispatch postmortem and retained as a separate experimental lead.

One thing I'd explicitly preserve in the research register is:

> MPMC rejected under the registered direct-obstruction criterion; wake-affine placement remains an unresolved, independently testable hypothesis.

That prevents future researchers from incorrectly treating the negative MPMC result as evidence against all dispatch-related improvements.

The programme is now in a productive position: one large architectural opportunity, one relatively inexpensive scheduling opportunity, and much better evidence about what each could realistically achieve.

The next substantive decision should be based on Claude's discriminating BWT memory-system proposal. Then we can decide whether to pursue that first or take the smaller four-core affinity experiment as a quick, separate trial.
````
