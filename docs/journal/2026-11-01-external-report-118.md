# External research report 118: the guardian on the coordinator's postmortem: wake-affine placement is new evidence (verbatim)

A research input, not a finding of this repository.

- **Relayed:** by the maintainer, 2026-11-01 15:36 UTC.
- **Source:** the guardian.
- **In reply to:** the coordinator's dispatch postmortem (`docs/dispatch-causality-records`
  at `3a954fe`).

It is kept exactly as received (rendered text, cards flattened).

## The message, as received

````text
No, you hadn't sent me this particular Claude report before. You'd already shared the initial i9 capture and the researcher's independent interpretation, but this is Claude's subsequent, more detailed postmortem.

And it contains an important new finding.

Claude has identified a plausible explanation for the substantial scheduling delay we observed:

A worker releases the receiver mutex, wakes another worker, and then starts decoding. Linux frequently places the newly woken worker on the same CPU as the worker that woke it—even though another allocated CPU may be idle.

The resulting worker can sit runnable for approximately 2.36 ms while useful decoding proceeds elsewhere.

### What's new here

| Finding                                                                       | Why it matters                                                                       |
| ----------------------------------------------------------------------------- | ------------------------------------------------------------------------------------ |
| 1.90% direct receiver obstruction                                             | Confirms the registered decision not to proceed with MPMC                            |
| 9.95% mutex-associated waiting, including runnable delay                      | Shows a much larger scheduling interaction than direct lock obstruction alone        |
| 87% of long receiver-phase waits associated with placement on the waker's CPU | Suggests wake placement is part of the mechanism                                     |
| 70% of long-wait time overlaps an idle allocated CPU                          | Raises the possibility of improving utilization without changing decoding algorithms |
| Scanner runnable but not running for 36% of the run                           | Suggests scanner scheduling deserves attention too                                   |
| Admission window full 60% of the time                                         | Shows bounded admission and ordered consumption interact with scheduling             |

These latter figures are exploratory, not registered causal proof. In particular, a fifth CPU for the scanner would change the hardware budget, so it wouldn't be a valid four-core production improvement by itself.

### Does this change our direction?

Somewhat, but I wouldn't abandon the BWT work.

The strongest new hypothesis is no longer simply “the receiver mutex is slow.” It's that mutex handoff and Linux wake-affine scheduling interact poorly with the decoder's thread arrangement.

That could potentially be addressed by changing worker placement or wake-up behavior without replacing the queue.

However, the separate inverse-BWT scaling penalty remains roughly 0.68–0.72 CPU-s/GB. Recovering that is still essential for approaching the 1.5–2× goal.

I'd make one adjustment to the consolidated Claude instruction we prepared: ensure Claude incorporates this new postmortem into its dispatch closure, rather than simply reproducing the frozen verdict again.

Otherwise, the next task remains a discriminating inverse-BWT memory-system proposal.

I would keep a small scheduling-affinity experiment as a credible secondary option, not dismiss it. If a future four-core experiment can distribute workers more effectively while preserving the same total CPU budget, that could be an unusually inexpensive improvement.

Bottom line: this report is new to me. It strengthens the case that scheduling placement may be a real optimization opportunity, while leaving the registered MPMC rejection intact. I would preserve both conclusions rather than treating the dispatch investigation as entirely uninteresting.
````
