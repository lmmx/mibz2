# External research report 111: the researcher's review of the frozen dispatch analysis: the summary's validity omits the scheduler checks (verbatim)

A research input, not a finding of this repository.

- **Relayed:** by the maintainer, 2026-11-01 14:09 UTC.
- **Source:** the researcher.
- **In reply to:** the coordinator's freeze and dry-run report at `3b166cc`, and report 110.

It is kept exactly as received (rendered text, cards flattened).

## The message, as received

````text
My recommendation is to authorize the diagnostic after one small validation fix—not to change its hypotheses or start optimizing the queue.

Claude has done substantial preparation: the original scheduler accounting reproduces, the production decoder's machine code is unchanged, and the VM capture exercises the tracing pipeline. The remaining uncertainty is whether the i9 capture will be sufficiently non-perturbative to support a causal conclusion.

I agree with Astra's approach: measure receiver ownership and scheduling first; decide about MPMC afterward.

### Answers to Claude's three questions

| Question                                                                | My decision                                                                                         |
| ----------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------- |
| (a) Count mutex wake-up time toward the 5% MPMC threshold?              | No. Keep the frozen primary rule. Report mutex-associated runnable delay separately.                |
| (b) Add a sched-switch-only fallback if tier B exceeds 3% perturbation? | No. Accept a provisional outcome rather than introduce a third, less informative measurement tier.  |
| (c) Pre-register a condition-variable split experiment now?             | No. Diagnose the reader path, but authorize no notification changes until its cause is established. |

For (a), the distinction matters. Time spent sleeping because another worker holds the receiver mutex is direct evidence of ownership obstruction. Time spent runnable but unscheduled after a mutex wake could be influenced by the mutex design, but it may reflect ordinary competition for four cores. Combining both into one promotion statistic would overstate what has been established.

I would retain both figures, explicitly:

\\[ R\_{\text{direct}}=\text{registered receiver obstruction} \\]

\\[ R\_{\text{associated}}=R\_{\text{direct}}+ \text{runnable delay following receiver-mutex wake} \\]

The second is exploratory, not an alternative pass criterion. A large gap between them would be scientifically interesting.

### One concrete issue I found in the frozen analysis

I inspected [bench/dispatch.py](https://github.com/lmmx/mibz2/blob/research/dispatch-causality/bench/dispatch.py?utm_source=chatgpt.com).

The initial quality check correctly requires:

- `threads_in_sched`
- `sched_coverage` of at least 99%

But the final `summary()` validity calculation does not check either field. It checks coverage, perturbation, alignment and mark completeness.

That means the first three captures could qualify for expansion, while one of the subsequent seven has inadequate scheduler coverage, yet the final summary could still produce an apparently valid MPMC decision.

This is a genuine mismatch with §6 of the registered protocol, which requires the validity checks to hold on every interpreted run.

I would fix this before the i9 capture. It is a bug fix to enforce the existing preregistration, not a change to the experiment's hypothesis or thresholds.

I'd also have Claude verify that lost kernel trace events cannot silently disappear from the `perf script` parser. The script currently ignores lines it doesn't recognize; scheduler-state inference is sensitive to missing wake-up and switch events. Any reported trace loss should invalidate the affected causal attribution rather than be silently accepted.

### What to send Claude

I support proceeding with the i9 dispatch-causality diagnostic, subject to a small final validation correction.

For the three open questions:

(a) Keep the frozen 5% primary MPMC threshold unchanged. Report receiver-mutex-associated runnable delay separately from direct receiver obstruction, without adding it to the primary statistic.

(b) Do not add a third sched-switch-only fallback. If tier B fails the perturbation requirement, preserve the provisional outcome.

(c) Do not preregister a condition-variable replacement yet. Retain the existing reader-path diagnostics and defer engineering decisions until the capture is interpreted.

I found a protocol-enforcement issue in `bench/dispatch.py`: the initial quality check requires `threads_in_sched` and at least 99% `sched_coverage`, but the final `summary()` validity calculation omits both. Please correct that so every four-worker run must satisfy the already-registered scheduler validity requirements.

Also verify that kernel trace-event losses are detected and reported rather than silently ignored by the parser. Preserve the existing thresholds and classifications.

After these validation-only corrections, update the pin/freeze and perform the untimed verification needed to establish the runner is still correct. No change to decoder semantics, receiver ownership, notification behavior or performance criteria.

I approve the single i9 diagnostic capture once those checks pass. Stop after recording the evidence and report whether receiver-specific obstruction meets the registered 5% materiality and 80% repeatability criteria. Do not prototype MPMC or split the condition variable.

One final observation: the VM's 94% classified coverage and sub-microsecond clock-alignment uncertainty are encouraging. The VM's unusually noisy paired performance results are less encouraging, but the protocol already handles that by declining to draw a definitive conclusion if instrumentation distorts the experiment.

I would not reopen the architecture debate now. This is the right measurement. Correct the final validation logic, preserve the original thresholds, and let the i9 evidence determine whether an MPMC implementation is justified.
````
