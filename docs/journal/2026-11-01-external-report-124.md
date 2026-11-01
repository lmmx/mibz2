# External research report 124: the guardian on using Astra: an independent architectural proposal in parallel (verbatim)

A research input, not a finding of this repository.

- **Relayed:** by the maintainer, 2026-11-01 15:36 UTC.
- **Source:** the guardian.
- **In reply to:** the maintainer's question: "are we to put anything to Astra that requires
  its expertise? it says 5 hour limit 100% left now, weekly limit 57% left (resets in 5
  days 4h)".

It is kept exactly as received (rendered text, cards flattened).

## The message, as received

````text
Yes. I think Astra has a valuable role right now—but I would use it for an independent architectural assessment, not another general review of the research programme.

With 100% of the five-hour allowance available and 57% of the weekly allowance remaining, we have enough capacity for a substantial reasoning task. We should spend it where Astra's expertise in algorithms, CPU architecture and machine-code behaviour can change our next decision.

The opportunity is to have Astra independently design a discriminating experiment, in parallel with Claude, before either proposal influences the other.

That gives us two independently developed approaches to the same difficult problem.

## What I would ask Astra

I'd like an independent, deep architectural analysis of the next optimization opportunity in `lmmx/mibz2`, focusing on inverse-BWT memory-system behaviour under four-worker decoding.

Please approach this as a CPU microarchitecture and algorithm-design problem, not a general performance review. I want you to challenge our causal assumptions and identify the smallest experiment capable of changing an engineering decision.

**Context**

Our target is 1.5–2× full-decoder throughput on four physical cores, without increasing CPU-seconds/GB or peak RSS.

The shipped i9-10900K attribution experiment showed:

- Four-worker decoder CPU cost: approximately 3.579 CPU-s/GB.
- One-to-four-worker CPU inflation: approximately 32.2%.
- Cycles/byte inflation: approximately 30.1%.
- Retired instructions/byte inflation: approximately 1.3%.
- Inverse BWT and its associated work explain approximately 78–83% of the incremental CPU cost, or 0.68–0.72 CPU-s/GB.
- L3 misses and DRAM writes rise substantially under concurrency, but observed DRAM bandwidth is below saturation.
- `OFFCORE_REQUESTS.ALL_DATA_RD` includes prefetch traffic; it is not a reliable demand-load count.
- The 32-lane BWT walker uses a four-byte T-vector/Ψ representation and software prefetch to overlap dependent memory accesses.

**Relevant negative results**

- A faster standalone unrolled BWT walker did not improve four-worker full-decoder throughput on the i9.
- MTF improvements did not clear full-decoder gates.
- Compact Ψ representations have historically incurred extra per-core traversal/construction costs.
- Earlier prefetch and copy-elimination experiments have not established a substantial full-decoder gain.
- A newly completed, valid dispatch experiment rejected MPMC receiver replacement under its registered criterion (1.90% direct obstruction versus 5% required). Separately, wake-affine scheduling remains an exploratory, untested opportunity. Do not redirect this analysis into a dispatch redesign.

**Your task**

Independently assess five competing explanations for inverse-BWT's concurrency penalty:

1. T-vector cache residency and eviction.
2. Prefetch interference, including fill-buffer, cache and request-resource pressure.
3. Intermediate-buffer write allocation, dirty-line eviction and writebacks.
4. Interference between concurrent T-vector construction and traversal.
5. Uncore/memory-request scheduling effects.

These may interact; do not assume they are independent or mutually exclusive.

For each, explain:

- What our existing measurements actually establish.
- Which inferences are unsupported by the available counters.
- Which alternative mechanism could produce the same measurements.
- What observation or intervention would distinguish the explanations.

Pay particular attention to whether increased DRAM writes and L3 misses are causes of the extra latency or merely consequences of working-set pressure.

**The central challenge**

Design **one preferred, narrowly bounded experiment** that can distinguish the most consequential competing mechanisms.

I want a genuinely discriminating intervention or measurement, not another collection of correlated performance counters.

Specify:

- The hypothesis and its strongest alternative.
- Expected signatures under each explanation.
- The minimum necessary instrumentation and Intel i9-10900K PMU events, including event-semantic limitations.
- Confounders introduced by the measurement itself.
- The controls required for valid one-versus-four-worker comparison.
- What outcomes would support retaining the four-byte T-vector with reduced traffic, changing build/walk overlap, or revisiting composed/compact Ψ.
- The plausible recoverable CPU-s/GB relative to the measured 0.68–0.72 incremental inversion penalty.

If you believe the leading hypotheses cannot be distinguished cleanly in one experiment, say so and recommend the smallest useful first decision instead. Do not claim more causal certainty than the hardware supports.

**Important constraint**

Do not implement an optimization, launch benchmarks, or assume new measurements exist.

Please return a concrete experiment proposal and an explicit engineering-decision matrix. Be willing to reject our favored memory explanations if the evidence warrants it.

Claude Code is independently developing a proposal for the same problem. Your analysis should stand on its own; we'll compare the two afterward.

## Why I'd use Astra now rather than later

There's a genuine advantage in getting two independent proposals before we choose the experiment:

- Claude has direct repository knowledge, experimental tooling and the accumulated implementation history.
- Astra can challenge the microarchitectural interpretation and identify whether the proposed intervention actually distinguishes causes.
- The researcher can audit the resulting experiment for confounding, hidden assumptions and decision validity.
- I can reconcile the proposals into one executable research decision without forcing agreement where evidence is missing.

One practical point: give Astra access to the attribution records and relevant source if possible. The prompt is enough to orient it, but exact PMU event definitions, walker code and buffer lifetimes are crucial to making the recommendation implementable. Without those, its answer should be treated as an architectural hypothesis rather than a verified analysis of this implementation.

I wouldn't spend the remaining weekly allowance on repeated high-effort reviews of the same explanation. One strong independent architectural pass now, followed by a targeted adjudication only if Claude and Astra materially disagree, is a much better use of it.

My recommendation: send this to Astra now, in parallel with Claude's work. This is precisely the sort of problem for which its extra reasoning depth should earn its cost.
````
