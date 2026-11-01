# External research report 86: taking over as independent reviewer; the protocol is ready (verbatim)

A research input, not a finding of this repository.

- **Relayed:** by the maintainer, 2026-10-31 10:36 UTC.
- **Source:** the guardian.
- **In reply to:** the P74-C full-decoder protocol.

It is kept exactly as received (rendered text, cards flattened). Paths to uploaded
files are the maintainer's.

## The message, as received

````text
guardian:

Good. I'll take over as your independent research reviewer, with Claude handling implementation and experiments. We can simplify the communication considerably: Claude builds and measures; I review the engineering decisions and evidence; you authorize major steps.

My assessment of Claude's latest message is positive. The full-decoder protocol is ready in principle. I would not introduce another round of changes before freezing it.

There are two technical points I want to keep an eye on, but neither justifies delaying the work.

### 1. The four-core experiment is now measuring the right thing

The primary question is whether P74-C improves the actual decoder on a previously unseen Wikidata dump.

PRIMARY ACCEPTANCE TEST

# ≥ 2%

Lower confidence bound on throughput gain

4 workers

New October 26 Wikidata dump, with the complete decoder confined to four physical cores.

CPU efficiency, memory and negative controls must also pass their own gates.

This is considerably more informative than another MTF-stage experiment.

The one subtlety is that the CPU and RSS gates should measure what we actually care about in production, not merely what happens to be statistically convenient. Claude's current definitions are acceptable as frozen engineering criteria, provided the results include the raw paired distributions.

For RSS, I would interpret the strict maximum rule conservatively: a failure could reflect a real memory increase or high-water-mark variability, and the reported data should distinguish those possibilities.

### 2. Astra's assignment is exactly where I'd spend the quota

I support the proposed BWT walker investigation. It's narrow, technically demanding and has a clear deliverable.

But I would make one mental adjustment to the question.

The estimate of 8.6 effective chains is a performance diagnostic, not proof that only 8.6 memory requests are in flight. Astra should not feel compelled to find a particular register spill or software dependency to explain it.

It's possible the limiting factor is fundamentally the processor's ability to overlap dependent loads, rather than a correctable compiler defect.

A successful Astra review might therefore conclude that there is no credible 1.2–1.5 c/B machine-code improvement available from restructuring this loop. That would itself be a valuable result.

I would rather obtain that conclusion than spend our limited quota on a superficially clever rewrite that damages memory-level parallelism.

### What happens next

Claude already has the necessary instructions and is preparing the frozen trial. I wouldn't send another message now.

When Claude reports the pinned commit and i9 command, I'd check three things before you run it: that the code really matches the frozen P74-C policy, that the entire process is restricted to four physical cores, and that the shipped and experimental modes share the same production scheduling path.

After the benchmark, we'll distinguish a genuine performance failure from a merely inconclusive statistical gate, without rewriting the stopping rules.

And when Astra's evidence pack is ready, we'll use one focused specialist review on the BWT walk—not a broad request for more ideas.

My recommendation: let Claude finish the freeze, then run the full-decoder experiment. This is the most consequential P74 measurement we've undertaken, and it has a clean path to an actual shipping decision.

> ok so it is gonna run, and then... what, it tells me something to send to astra codex and/or run on my local i9? i can run stuff locally in the next ten minutes or so ONLY, otherwise it has to be codex today until about ten hours from now

Yes, exactly. Claude is preparing two separate handoffs, but your ten-minute i9 window changes the priority.

We should prioritize getting the i9 benchmark running now. Astra can be handled entirely through Codex later today.

Here's how it should work:

1\. Your i9 — time-sensitive

Claude finishes the frozen full-decoder runner and gives you a pinned commit SHA plus a one-command invocation. You run it locally, as you did for P74-C. Ideally the runner completes unattended and preserves its result archive.

2\. GitHub CI — Claude handles

Claude launches the exploratory CI benchmarks. They can run independently of your workstation, but cannot establish four-physical-core performance.

3\. Astra Codex — no local action needed

Claude prepares the BWT assembly evidence pack, then gives you a focused prompt and the file or repository reference. You pass that to Astra for read-only analysis. No i9 access is required.

One important practical point: the i9 benchmark doesn't need to finish within your ten-minute window if you can leave the machine running. You only need to start it, provided the runner is fully prepared and the machine can remain available.

If the machine must be shut down or become busy after ten minutes, however, don't start a run that cannot finish. The protocol prohibits selective reruns, so we should avoid a preventable interrupted experiment.

### I'd message Claude right now

Important scheduling constraint: I have access to my i9 workstation for only approximately the next ten minutes. After that, I cannot run local commands for about ten hours, although you can continue working through Codex and GitHub CI.

Please prioritize freezing the full-decoder protocol and producing the pinned i9 one-command runner immediately if it can be made genuinely ready without skipping essential correctness or audit requirements.

If the full i9 benchmark can run unattended after I launch it, say so and provide the exact command and expected artifact location. If not, don't rush or compromise the freeze; proceed with CI and Astra preparation instead, and we'll run the i9 later.

The Astra BWT evidence pack is not time-critical and can follow. Don't spend time optimizing or reworking P74-C before the full-decoder trial.

My recommendation: send that scheduling note now. It is not changing the research protocol; it's ensuring Claude knows which part of the work has a rapidly closing execution window.

If the i9 runner isn't fully ready, we lose ten hours of calendar time, not scientific progress. We absolutely shouldn't compromise a carefully frozen experiment just to catch the window.
````
