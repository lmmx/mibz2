# External research report 102: after both negative trials: Astra's architectural review, framed and reviewed (verbatim)

A research input, not a finding of this repository.

- **Relayed:** by the maintainer, 2026-11-01 09:42 UTC.
- **Source:** the guardian (a prompt for Astra and two reviews), and Astra's architectural review.
- **In reply to:** the walker's i9 trial and its postmortem.

It is kept exactly as received (rendered text, cards flattened). Paths to uploaded
files are the maintainer's.

## The message, as received

````text
Guardian (prompt for Astra):


```
Yes. I think that's exactly the right evolution in how we use Astra.

We've been treating Astra primarily as a machine-code specialist: Claude identifies an algorithm, Astra examines the generated instructions, and Claude implements the suggested improvements.

That's useful, but it underuses Astra's reasoning capabilities.

I'd now involve Astra earlier, at the architectural decision stage, while still expecting it to ground its recommendations in actual hardware constraints and measured results.

The two recent experiments make a compelling case for doing that.

## What Astra needs to understand

| Experiment   | What worked                                                          | What failed                                                                  |
| ------------ | -------------------------------------------------------------------- | ---------------------------------------------------------------------------- |
| P74-C        | \~20% MTF-stage gain on i9; +3.6% decoder throughput on new Wikidata | Strict acceptance criteria, controls and RSS                                 |
| Unrolled BWT | 12.3% faster walker stage on EPYC 7763                               | Essentially zero four-core decoder improvement on i9; records CPU regression |

We have improved two significant components without achieving the production-level breakthrough we're pursuing.

That suggests the next question should be which architectural resource or algorithmic cost is actually worth attacking, not simply which loop can be made shorter.

And Astra's prior work gives it valuable context: it now knows the walker assembly, register constraints, prefetch behaviour, 32-chain scheduling and the limits of unrolling.

## What I'd ask Astra now

I'd continue in the same BWT superoptimization chat. Unlike the first assignment, continuity is valuable here because Astra can reconsider its original prediction in light of the new evidence.

We now have the production-equivalent i9-10900K results for the fully unrolled 32-lane BWT walker you analysed.

I want you to broaden your role from machine-code optimizer to independent performance-architecture researcher. Your task is to help decide what mechanism is worth engineering next, not merely how to optimize an already-selected implementation.

**New experimental evidence**

The unrolled walker achieved the intended assembly signature: approximately 9.53 fused macro-ops per ordinary lane step, versus 14.4 for the original. On EPYC 7763, the walker-stage time improved by 12.3%, with a measured B/A ratio of 0.877 [0.863, 0.891].

However, the registered full-decoder experiment on an Intel i9-10900K, with the entire process restricted to four physical cores, was negative.

At four workers:

- New Wikidata dump: throughput ratio 0.9996 [0.9874, 1.0120].
- Fitted Wikidata: 1.0076.
- Same-dump held-out Wikidata: 1.0125.
- Records: throughput approximately 2.1% slower and CPU-seconds/GB approximately 1.8% higher.
- Correctness, thread affinity, structural assembly and A/A calibration all passed.

The P74-C experiment, separately, achieved approximately 3.6% higher four-core throughput on new Wikidata and about 1.6% lower CPU-seconds/GB, but failed its registered acceptance criteria. That experiment is closed.

Both experiments are archived under their original verdicts. Do not propose rerunning, retuning or retroactively changing their acceptance gates.

**Your research assignment**

First, revisit your original unrolling recommendation in light of this evidence. Explain which parts of your machine-code model remain supported, which predictions did not transfer to Intel full decoding, and what measurements would distinguish plausible explanations.

Then identify the most promising *next architectural mechanism* for substantially increasing full-decoder throughput, under these constraints:

- Target a 1.5–2× eventual throughput improvement, while maintaining or reducing CPU-seconds per GB and controlling peak RSS.
- Optimize for four physical cores as well as AMD/Intel portability.
- Consider reducing dependent BWT traversal work per output byte, improving T-vector representation or construction, and addressing scanner/worker pipeline limits.
- Account for the possibility that the pipeline is bottlenecked elsewhere once one stage becomes faster.
- Treat N1's partial-upgrade Ψ traversal as an existing research candidate, not an unexplored invention. Compare it against other mechanisms and its known construction and RSS costs.
- Avoid proposing another minor unrolling, register-allocation or prefetch variant unless evidence suggests it can produce a material decoder-level gain.

**Deliverables**

1. A causal interpretation of the two negative full-decoder experiments, explicitly distinguishing measurement, hypothesis and unknowns.
2. The two or three strongest architectural directions, ranked by expected full-decoder benefit, implementation cost, memory cost and probability of success.
3. One recommended next experiment with a precisely defined hypothesis, minimal implementation scope, early falsification criteria and expected machine-code or memory-access signature.
4. The most important missing evidence that would change your recommendation.

Use the repository's existing research notes and archived experiment results. Do not invent performance measurements.

This is a read-only research decision. Do not implement code, modify branches, run benchmarks or launch CI. I want independent architectural judgment before committing further engineering effort.

## Why I think this is worth the Astra quota

Astra previously made a credible estimate of the walker-stage improvement. The measured Zen 3 result fell within that prediction.

What its analysis did not establish was how much of that improvement would survive full decoding under Intel's scheduling and resource constraints.

That distinction is precisely what we now want Astra to investigate.

I would particularly value an answer that challenges the assumption that the next breakthrough must come from further reducing walker instructions.

For example, N1's multibyte traversal could reduce the number of dependent accesses per decoded byte, potentially changing the fundamental latency problem. But its additional construction work and memory footprint may make it uneconomic.

An excellent architectural review would expose that trade-off quantitatively—and perhaps identify a better compromise.

My recommendation: send Astra the new results and ask it to choose the next engineering hypothesis. Don't spend its next call polishing the unrolled walker.

Let Astra help determine what is worth building, Claude build it, and us review whether the resulting evidence justifies continuing.
```


Guardian (upon seeing Claude Code review of the i9 run):


```
Yes, this materially changes what I'd tell Astra. I'm glad Claude finished the postmortem before you sent the prompt.

The most important discovery is that the unrolled walker's performance problem starts at one worker on Intel, before four-worker contention enters the picture. And the existing pair-walk experiment gives Astra a particularly useful counterexample to simplistic proposals for reducing dependent loads.

I would keep the architectural-research prompt you've already pasted, but append the following evidence before sending it.

**Important additional evidence from the completed i9 postmortem — incorporate this before making your architectural recommendation.**

The full-decoder results have been independently reproduced from the frozen archive. The exact binary, correctness checks, thread allocation and A/A calibration passed.

**1. The unrolled walk's weak Intel result begins at one worker.**

- On the i9-10900K, the unrolled implementation saves only approximately 1.5% decoder CPU at one worker (estimated at \~0.2 cycles per decoded byte) and improves throughput by 1.7%.
- At four workers, decoder CPU savings fall to approximately 0.6%, and throughput is essentially unchanged.
- On Zen 3 at one worker, the arm saved 6.8% CPU for +8.1% throughput; Zen 5 saved 13.2% CPU for +17.9%. Zen 4 and another Intel Xeon were closer to the i9.
- The i9 has no independently measured walker-stage speedup. The estimated 3–4% walker improvement assumes a 35–45% stage share and must not be treated as directly measured.

This means the disappointing four-worker result cannot be attributed entirely to parallel scheduling. The same generated-code transformation appears to have much less benefit per core on Intel.

**2. Four-worker scaling exposes a separate limit.**

- CPU cost per decoded byte rises approximately 30% from one to four workers in both arms.
- Only about 3.15 of the four allocated physical cores are busy on average.
- The existing archive cannot distinguish scanner/feeder serialization from memory-system stalls. Per-thread CPU accounting and hardware counters are missing.

Do not assume that adding more memory-level parallelism inside worker threads improves the four-worker pipeline.

**3. A multibyte traversal counterexample already exists.**

The earlier pair-walk implementation, which retrieves two output bytes per dependent traversal load, was tested on the i9 against v0.1.4:

| Threads                       | 1     | 4     | 6     | 8     | 10    |
| ----------------------------- | ----- | ----- | ----- | ----- | ----- |
| Pair-walk relative throughput | 0.69× | 0.82× | 0.99× | 1.11× | 1.12× |

The pair walk substantially loses at our 1–4-worker target, despite winning at high concurrency. It reduces memory pressure at the expense of additional per-core work.

Do not simply recommend multibyte traversal without explaining how a new design avoids this known four-worker failure. N1's partial-upgrade Ψ traversal may differ materially, but its construction cost, memory footprint and four-core economics must be assessed independently.

**4. Distribution-specific regression remains unexplained.**

The unrolled walker increases `records` CPU cost by approximately 1.3–1.8% at every worker count, including one worker, while `random` remains approximately neutral. The cause is not established.

**Implications for your assignment**

Please revisit your earlier machine-code recommendation without defending it reflexively. Distinguish the Intel per-core limitation from the four-core pipeline limitation.

Before committing to a new architecture, assess whether the most valuable next step is actually a bounded diagnostic experiment: a one-core Intel walker-stage comparison and/or per-thread CPU and memory-stall measurements of the shipped four-core decoder.

If the evidence is insufficient to select a new architecture confidently, say so. Rank the best remaining hypotheses, but recommend measurement rather than speculative implementation where appropriate.

The registered unrolled-walker experiment is closed as negative. No rerun, retuning, production modification or new benchmark is authorized by this request.

### Why this changes the research direction

We now have evidence for two distinct bottlenecks:

Per-core efficiency

The same unrolling approach delivers substantially different benefits across CPU designs. That points toward microarchitectural limits, code layout, cache behavior or instruction delivery—not yet a single established cause.

Parallel pipeline scaling

At four workers, total CPU efficiency deteriorates while some core capacity remains unused. We need to distinguish scheduling and serial-stage bottlenecks from memory stalls.

And there's an important strategic consequence: the earlier pair-walk result makes “reduce dependent loads per byte” an insufficient optimization thesis by itself. We have direct evidence that doing so can increase per-core costs enough to lose badly on the intended deployment.

That doesn't rule out N1, but it significantly raises the burden on its construction economics and execution design.

My recommendation: append this evidence to your existing Astra prompt and send it in the same conversation, at Extra High effort.

I would not add further directions from the issue tracker yourself. Astra already has permission to inspect them. Let it decide whether the next step is a new algorithm or a diagnostic experiment that would prevent us from spending weeks engineering the wrong thing.

This additional evidence is more valuable than another long list of proposed optimizations.
```


Astra codex:

```
**I recommend a bounded diagnostic of the shipped decoder before engineering another walker.** The evidence supports two distinct limitations: little benefit from unrolling on one i9 worker, and substantial additional inefficiency when scaling to four workers. Their causes remain insufficiently separated to choose an architecture confidently.

I withdraw the earlier implication that removing walker bookkeeping was a strong candidate for a material Intel decoder improvement. It was a useful machine-code hypothesis, and the AMD results support it on some processors. It did not deliver the required result on the target configuration. Both registered experiments remain closed as negative.

I reviewed the open issues, [research portfolio and register](https://github.com/lmmx/mibz2/pull/117), subsequent N1 analyses, production pipeline, and both frozen experiment archives. The distinction between an open issue and an unimplemented mechanism matters here:

| Mechanism | What the repository establishes |
|---|---|
| Faster Huffman lookup, run-lane construction, room walking | Already implemented and measured. |
| MTF checkpoints and cooperative fused construction | Shipped; their larger-core economics do not establish an opportunity on four physical cores. |
| One-pass T-vector construction, #114 | Already shipped. Any new builder must compete with this baseline, not the older two-pass construction. |
| Pair walking | Implemented and rejected for the relevant configuration. Its i9 ratios of **0.69× at one worker and 0.82× at four** are a strong counterexample to “fewer traversals means faster decoding.” |
| Precopy and run-move approaches | Implemented and unsuccessful in their tested forms. These are warnings about construction and per-visit overhead, rather than mathematical refutations of every composed traversal. |
| 24-bit compact Ψ, [#112](https://github.com/lmmx/mibz2/pull/112) | Implemented. Smaller tables moved the concurrency knee, but per-core overhead prevented progression. Byte recovery from a separate descriptor has already been tried. |
| Parallel speculative scanner, [#10](https://github.com/lmmx/mibz2/issues/10), and full front-end fusion, [#8](https://github.com/lmmx/mibz2/issues/8) | Remain unresolved. Earlier scanner deferral relied on older measurements. |
| N1 partial-upgrade Ψ | Existing theory, census, correctness simulation and construction analysis. No demonstrated native builder-plus-walker speedup. |
| Unrolled walker and P74-C | Completed negative full-decoder experiments; supersede earlier portfolio optimism. |

**The unrolling result narrows the original model.** The transformation achieved its structural purpose: approximately 14.4 → 9.53 fused macro-ops per ordinary lane step. The EPYC stage improvement establishes that this overhead could matter. It did not remove the central recurrence:

`lane row → dependent T load → successor row → next T load`

Nor did it remove most lane-state traffic, output stores or prefetch traffic. Consequently, it reduced one resource demand while leaving several others largely intact.

The [i9 postmortem](https://github.com/lmmx/mibz2/blob/270333c2d32fe16dc24ae7ab64f3697d87939418/docs/journal/2026-11-01-walk-unroll-decoder-i9-10900K/analysis.md) establishes:

- At one worker, decoder CPU falls only about **1.5%**, with throughput up **1.7%**. The approximately **0.2 c/B** saving is a clock-based estimate.
- At four workers, CPU savings fall to approximately **0.6%**, with essentially unchanged throughput.
- `records` costs more CPU even at one worker.
- Zen 3 and Zen 5 benefit substantially more, while Zen 4 and the tested Xeon are much closer to the i9.

Thus, “AMD benefits, Intel does not” is also too coarse a model.

Several explanations remain plausible:

| Hypothesis | Observation that would distinguish it |
|---|---|
| The i9 was already constrained mainly by load/address resources or dependent memory accesses | A small directly measured walker gain, fewer retired instructions, but little reduction in walker cycles; memory/backend pressure remains dominant. |
| Larger code or changed integration offsets the bookkeeping saving | Increased front-end delivery stalls, instruction-cache/ITLB pressure or branch recovery, especially on `records`. |
| The isolated walker improves, but surrounding integrated code becomes more expensive | A substantial isolated-stage gain alongside a small integrated gain, with the offset attributable outside the walker. |

The fidelity note’s roughly **2.5 c/step Intel load/address throughput floor** is a static lower bound, not proof of the active bottleneck. Likewise, the estimated **3–4% i9 walker improvement is not a measurement**.

A one-core Intel stage comparison would resolve that particular uncertainty. It would explain the closed result; it would not change its verdict. I assign higher decision value to understanding the shipped four-core decoder first.

**The four-worker problem requires two accounts: occupied CPU time and unoccupied time.** A thread stalled on a cache miss still accumulates CPU time. Memory stalls can explain higher CPU-seconds/GB, but cannot by themselves explain idle cores; they would need to induce starvation, imbalance, blocking or a longer drain.

The production [pipeline](https://github.com/lmmx/mibz2/blob/45b366d1455a487f1655def689ebd8c87a31f135/src/parallel.rs#L369) exposes a specific alternative to scanner saturation. Admission stops when `issued - taken` reaches the slot limit, and `taken` advances in output order. A slow early block can leave later outputs ready, admission blocked, and some workers idle. That is **a hypothesis supported by the control structure**, not a measured explanation.

Other possibilities include input waits, consumer work, scheduler contention, startup/drain effects and shared-cache pressure. Four workers also do not mean four total process threads.

The arithmetic shows why this distinction matters. Using the same archived i9 baseline:

\[
\text{throughput}=\frac{\text{average occupied cores}}{\text{CPU-seconds per GB}}
\]

- Filling 3.15 occupied cores to four, at unchanged CPU cost, gives only **1.27×**.
- At four occupied cores, **1.5×** requires CPU cost to fall from 3.90 to approximately **3.30 CPU-s/GB**.
- **2×** requires approximately **2.48 CPU-s/GB**, below the observed one-worker cost of 2.99.
- Recovering the one-worker CPU cost *and* occupying four cores would yield approximately **1.65×**. This is a conditional calculation, not a prediction; those improvements may interfere with each other.

The 1.5× target might therefore be reachable through improved scaling economics. The 2× target demands additional work reduction.

**P74-C supplies a different lesson.** Its [postmortem](https://github.com/lmmx/mibz2/blob/e9df8ab33dff2ac3a5f1846c4601c8df3e8b8bab/docs/journal/2026-11-01-p74c-decoder-i9-10900K/postmortem.md) supports a modest, distribution-dependent decoder benefit, insufficient for the registered criteria. It also identifies a repeatable RSS failure associated with the 32-byte reservation mismatch when buffers cross between bypass and P74 paths: approximately one additional 900 KB buffer at two workers on `mixed-independent`.

That is stronger causal evidence than the unexplained unroll regression. P74’s small-control regressions occur where its kernel never engages and have comparable A/A excursions; they do not establish the same per-core regression seen with unrolling. The engineering lesson is to evaluate retained allocation capacity across mixed paths, not just each kernel’s nominal scratch requirement. None of this supplies a saving that can be credited to a future design.

**My provisional architectural ranking follows.** This ranks suitability for the eventual target, rather than claiming measured expected speedups. No remaining candidate currently supports a credible 1.5–2× forecast.

| Rank | Direction | Full-decoder opportunity | Engineering and memory cost | Confidence |
|---|---|---|---|---|
| **1** | **Reduce total inversion work through construction-aware composed Ψ**, starting conceptually with N1 \(P^{2\mid4}\) | Highest algorithmic upside: amortize traversal, state handling and output operations across several bytes. Could eventually combine with run-native input construction. | High implementation cost. Construction can erase the walk saving; allocation must be capped before expansion. | Low confidence in a large gain; strongest remaining work-reduction hypothesis. |
| **2** | **Change pipeline work ownership and eliminate intermediate traffic**, conditional on attribution | Parallel block discovery/Huffman ownership could remove a serial limit; fusion could remove symbol-buffer traffic. Admission or ordering changes could address a different measured bottleneck. | Medium to high cost. Exact block joining and bounded buffering are essential. Fusion may lower RSS; speculative work or deeper queues may increase it. | Moderate chance of a useful gain if the corresponding bottleneck is observed; currently low confidence in 1.5× Wikidata. |
| **3** | **Compact Ψ with cheap byte recovery and direct construction** | Potentially reduce cache-capacity amplification at four workers. | High representation/build complexity; extra decoding dependencies can outweigh footprint reduction. Final tables shrink, but construction peak must also be controlled. | Low confidence in a large four-core gain after #112; improves in rank if counters show strong cache-capacity pressure. |

For **N1**, the distinction from pair walking must be economic, not terminological. It proposes sparse interval records and partial upgrades, rather than simply paying for a dense two-step representation. The existing simulation reports approximately **0.296 visits per BWT byte for \(P^{2\mid4}\)** and **0.150 for \(P^{4\mid8}\)**. These are visit counts, not hardware-load counts or speedups: rank/directory access, record access, boundary handling and construction remain.

The [best-known fused construction analysis](https://github.com/lmmx/mibz2/blob/9723f06847b88a10ba93d07c3f35680e58766015/docs/journal/2026-10-30-n1-fused-construction.md) estimates **5.3–7.2 instructions per byte**, with rank-heavy accesses whose achieved IPC is unknown. It explicitly concludes that the revised organization has not earned a construction experiment through a substantial reduction in operations.

Memory needs equal scrutiny. The [reconciliation](https://github.com/lmmx/mibz2/blob/9723f06847b88a10ba93d07c3f35680e58766015/docs/journal/2026-10-30-n1-traffic-and-peak-reconciliation.md) distinguishes the approximately **1.04 MB median \(P^{2\mid4}\) walk structure** from retained worker memory. Uncapped constructions can exceed the shipped footprint. Even the fused design reaches approximately **4.1 MB excluding L** on its maximum census block. Preallocation admission and reuse of a bounded arena are therefore part of the mechanism.

N1 avoids the pair-walk failure only if:

\[
\text{extra construction CPU}
<
\text{walk CPU saved}
\]

at **one and four workers**, including fallback blocks and actual buffer retention. Fewer visits alone establish none of that. The corrected four-copy model and untraced processor baselines should not be used to predict current i9 economics.

For **pipeline work ownership**, I would challenge the assumption that the old scanner deferral still settles the question. It does not—but neither does 3.15-core occupancy prove scanner saturation. A busy Huffman producer, a producer blocked on input, and a producer blocked by ordered admission require different mechanisms. Moving more work into the serial scanner would be particularly unattractive without that distinction.

For **compact Ψ**, any successor must avoid repeating #112’s expensive byte recovery. Keeping the byte with the link or its cache line is a meaningful architectural distinction, but remains unproven. Smaller occupied storage does not guarantee fewer transferred cache lines or shorter dependencies.

**The next proposed experiment is shipped-decoder resource attribution on the i9.** I have not run it or changed anything.

Its explicit hypothesis would be:

> More than half of the one-to-four-worker increase in CPU-seconds/GB originates in BWT construction and walking, with increased memory-related execution cost; meanwhile, scanner service capacity does not already prevent a 1.5× throughput improvement.

This tests whether the first-ranked inversion direction deserves engineering priority.

The minimal scope would be:

1. **Use the frozen shipped-A implementation**, at one and four workers within the same four-physical-core allocation. Use new Wikidata as primary, with `records` and `random` as controls. No algorithm variants, worker sweep or acceptance-gate changes.
2. **Account for every role:** scanner, feeder, workers and consumer. Record CPU time, runnable delay and blocked time, together with block issue/completion/consumption.
3. **Reuse the existing trace facilities** for input waits, slot waits, job receipt and ordered consumption. Keep any trace-enabled equivalent separate from uninstrumented counter collection and check its perturbation. Attribute worker execution to build, walk and remaining decode work.
4. **Collect a small set of supported, nonmultiplexed counters:** instructions and cycles per byte, effective frequency, cache/memory stalls and misses, plus outstanding-miss/fill-buffer indicators. Interpret Intel events on Intel; later AMD validation needs equivalent questions, not assumed equivalent raw events.

The expected signatures and early decisions would be fixed beforehand:

| Observation | Decision |
|---|---|
| Build+walk accounts for most added CPU cost, with increased cycles and memory-stall indicators per byte but similar retired work | Supports prioritizing inversion representation/work reduction. |
| Build+walk accounts for **less than half** the added CPU cost | Falsifies the experiment’s primary attribution hypothesis. Investigate the measured owner instead. |
| Frequency change explains the increase while cycles per byte remain similar | Reject the memory-amplification explanation. |
| Workers wait for jobs while the scanner actively decodes Huffman near its service ceiling | Promote parallel front-end ownership. |
| Scanner waits on full slots while later outputs are ready and an earlier block is outstanding | Promote investigation of ordering/admission and block imbalance. Adding scanner parallelism would not address that cause. |
| Most unused capacity belongs to startup/drain | Reject a sustained scanner-saturation explanation based on average occupancy alone. |

Scanner CPU service time per output byte gives an optimistic single-producer capacity bound. If that bound is already **below 1.5× current throughput**, leaving the scanner unchanged cannot meet the target, regardless of walker improvement. Conversely, ample service headroom plus negligible job starvation would demote scanner parallelization.

These are proposed diagnostic decision rules, not revisions to either closed experiment.

**The evidence most likely to change my ranking is:**

- Integrated i9 build/walk CPU attribution at one and four workers, including frequency. This determines whether inversion dominates the recoverable cost.
- A timeline separating scanner service, input starvation, ordered-admission blockage and drain. This can move pipeline work to first place.
- A directly measured one-core Intel walker comparison with phase-local counters. This would explain the unroll’s weak transfer and `records` regression, without reopening its verdict.
- Eventually, native N1 construction-plus-walk CPU and retained-memory measurements on both processor families. Current visit counts and instruction models cannot substitute for these.

**The next thing to test is the shipped decoder’s resource attribution.** My leading architectural hypothesis is construction-aware reduction of inversion work. Its immediate falsifier is that build and walk explain less than half the four-worker CPU inflation, or that measured scanner capacity already blocks the desired throughput. Until that distinction is established, committing to N1, compact storage or more worker-side parallelism would be premature.
```


Guardian:


```
This is the most valuable architectural assessment Astra has given us so far. I agree with its recommendation: measure the shipped decoder's resource use before committing to another major optimization.

What makes this response different is that Astra has stopped treating a faster inner loop as the objective. It's asking which resource actually prevents `mibz2` from reaching substantially higher throughput on four cores.

And it identifies two distinct problems that our earlier experiments couldn't resolve:

1. Worker efficiency: why does CPU cost per decoded byte increase so much between one and four workers?
2. Pipeline utilization: why are only approximately 3.15 of the four allocated cores occupied on average?

Those problems may require entirely different solutions.

## The most important new hypothesis

The pipeline's ordered-output admission mechanism is particularly interesting.

Astra noticed that admission is bounded by the number of issued but not yet consumed blocks. Because consumption advances in output order, one unusually slow early block could prevent admission of additional work even when later blocks have finished.

That creates a possible sequence:

Hypothetical ordered-admission bottleneck

Illustration of Astra's hypothesis, not an observed execution trace.

This could explain idle capacity without the scanner itself being too slow. But it remains a hypothesis until we have issue, completion and consumption timestamps.

That distinction is important because parallelizing Huffman decoding wouldn't fix an admission bottleneck caused by ordered output.

## Which direction should we pursue?

Astra's ranking makes sense, but I'd separate long-term potential from what is ready to engineer.

| Direction                                          | Potential                                      | Decision today                                             |
| -------------------------------------------------- | ---------------------------------------------- | ---------------------------------------------------------- |
| Construction-aware composed Ψ / N1                 | Potentially large reduction in traversal work  | Keep as leading algorithmic candidate; don't implement yet |
| Scanner, feeder and ordered-admission architecture | Could recover unused cores and improve scaling | Measure first — possibly the highest near-term value       |
| Compact traversal representations                  | Could reduce cache pressure                    | Deprioritize unless cache measurements support it          |
| More scalar walker superoptimization               | Limited demonstrated Intel benefit             | Stop for now                                               |

The most encouraging point is Astra's estimate that recovering both one-worker CPU efficiency and full four-core utilization could theoretically approach 1.65× throughput.

That isn't a forecast: the two improvements might interfere with each other. But it shows why pipeline-level work deserves attention alongside reducing BWT operations.

It also makes the next diagnostic economically meaningful. We aren't collecting counters for curiosity; we're deciding whether to invest in N1, front-end parallelism or an ordered-admission redesign.

## What I would authorize next

One bounded shipped-decoder resource-attribution experiment, independent of both closed optimization trials.

I'd have Claude prepare it with these constraints:

- Shipped decoder only, no experimental kernels.
- New-dump Wikidata primary, random and records as controls.
- One and four workers on the same four-physical-core allocation.
- Per-thread CPU, runnable/blocked time, scanner and feeder service, block issue/completion/consumption events.
- Small, architecture-appropriate hardware-counter groups, without relying on multiplexed measurements.
- Explicit assessment of measurement perturbation from tracing.
- Prespecified decision rules before collecting results.

One extra point: the diagnostic must distinguish CPU time spent executing but stalled from time a thread is not scheduled or is blocked. Those are different bottlenecks and require different remedies.

I'd also make the instrumented version an observational tool, not treat its wall-clock performance as directly interchangeable with the uninstrumented decoder.

### What to send Claude

Astra has completed its independent architectural review of the closed P74-C and unrolled-walker experiments.

Its recommendation is to stop selecting further optimization architectures until we establish the shipped decoder's actual one-to-four-worker resource limitations on my i9.

I agree. Please prepare one bounded, preregistered resource-attribution experiment using the shipped decoder only. Do not start timing yet.

Focus on two separate questions:

1. Why does CPU-seconds per decoded GB increase by approximately 30% between one and four workers?
2. Why does the four-core configuration average only about 3.15 occupied cores?

The leading hypotheses are inversion build/walk memory amplification, serial scanner/feeder limitations, ordered-admission blocking caused by slow earlier blocks, and startup/drain effects.

Use the separately dated Wikidata dump as primary, with random and records as controls. Compare one versus four workers inside the same four-physical-core allocation.

Prepare per-thread CPU and scheduling accounting, scanner/feeder service timing, block issue/completion/consumption timelines, and supported Intel hardware counters. Keep tracing and uninstrumented counter collection separate, and characterize tracing perturbation.

Before implementation, show me:

- Which existing instrumentation can be reused;
- The exact metrics and their attribution limitations;
- The smallest instrumentation changes required;
- How each possible result changes our next engineering decision;
- A frozen protocol and proposed i9 command.

Keep the decision rule that build+walk explaining less than half the added CPU cost falsifies the leading inversion-attribution hypothesis. Also calculate scanner service capacity and test for ordered-admission blocking and startup/drain effects.

No P74-C or unrolled-walker changes, no new performance optimization, no CI or i9 timing, and no production-default changes. Stop after presenting the proposed diagnostic for approval.

## One strategic adjustment

I would now give Astra a broader role at decision points, rather than calling it after every optimization.

Claude can build the instrumentation and perform the accounting. Once we have the actual utilization and bottleneck measurements, Astra can evaluate the evidence and help identify the most promising architecture.

That is likely a better use of its quota than continuing to review incremental assembly changes.

My recommendation is to send the Claude instruction above and pause new optimization engineering. We have two completed, carefully controlled trials and a very specific uncertainty that separates our remaining architectural options. Resolve that uncertainty, then commit to the next substantive mechanism.
```
````
