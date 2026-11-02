# External research report 150: the guardian on report 149: accept the capture; tighten the coordinator's interpretation (the eight-lines reading, D3, D5, the P–M contrast) (verbatim)

A research input, not a finding of this repository.

- **Relayed:** by the maintainer, 2026-11-02, with reports 148–150 together.
- **Source:** the guardian.
- **In reply to:** report 149 and the coordinator's §8.

**Evidence cutoff:** 2 November 2026. An interpretation amendment only: no rerun, new counters or collector migration.

It is kept exactly as received (rendered text, cards flattened, tracking parameters
included).

## The message, as received

`````text
**The two reviews agree on the experiment’s main result, but the researcher’s interpretation is more defensible. Claude’s postmortem adds a plausible hardware explanation and then treats parts of it too confidently.**

That does not undermine the capture. It means we should distinguish **a successful measurement** from **a settled explanation of the bottleneck**.

I rechecked the principal results in the uploaded archive. The registered outcome is clear: the shipped walker is within 9.6% of the reduced reference on one core; D3 provides no bottleneck classification; and D4’s proposed miss-capacity signature did not occur. walker-ladder-20261102T004502Z-…

## What we should accept

**The repaired perf setup is adequate for closing this diagnostic.** Its principal measurements passed the collection and repeatability checks, and the cycle and execution-counter control costs are small. The archive supports keeping this backend rather than making a Rust migration a prerequisite for further interpretation. independent-review walker-ladder-20261102T004502Z-…

**There is no demonstrated case for a substantial rewrite of the existing walker’s scheduling and bookkeeping.** P32 costs 6.897 cycles per row against M32’s 6.293, satisfying the registered stopping rule. Since M omits required output work, even that difference is not a readily available optimization. walker-ladder-20261102T004502Z-…

**Prefetch is valuable in this experiment.** The A→B contrast reduces cycles per row by about 31.8%. That is a meaningful intervention result, not merely an interpretation of a counter. independent-review independent-review

Those findings are enough to make a research decision. We do not need to manufacture a more precise hardware diagnosis to justify stopping generic walker cleanup.

## Where Claude’s wording needs tightening

### 1. “About eight lines in flight” is not established

Claude divides roughly 52–55 cycles per row for one lane by roughly 6.3–6.9 for 32 lanes. Those measurements exist, but their quotient is a **throughput ratio**, not directly a count of outstanding cache-line requests. independent-review

To convert it into occupancy, we would need assumptions about the requests generated per row and their service latency under concurrency. The one-lane cost also contains instructions, address dependencies and loop work—not exclusively a cache-line service time.

The relationship one would need is:

\[
\text{average outstanding requests}
=
\text{request arrival rate}
\times
\text{average request residence time}.
\]

Neither input can simply be replaced by the corresponding whole-loop measurement without justification.

**Using the same binary fixes the earlier comparability problem; it does not turn speedup into an occupancy measurement.**

The measured demand-associated occupancy is a different quantity. Intel’s definition includes a prefetch-allocated buffer only after a demand access encounters it, so it cannot supply the missing total-prefetch occupancy either. [GitHub](https://raw.githubusercontent.com/intel/perfmon/main/SKL/events/skylake_core.json)

I would retain “approximately eight” only as an explicitly assumption-dependent model—not as the explanation now established by the experiment.

### 2. “The shipped walk isn’t issue-bound” is stronger than D3 says

The recorded result is **57.1% of the four-per-cycle reference, below the registered warning threshold**. The analysis explicitly concludes that no classification follows. walker-ladder-20261102T004502Z-…

Intel defines this event as µops allocated from the RAT to the reservation station. It is not a measure of spare capacity across every execution port or dependency chain. [GitHub](https://raw.githubusercontent.com/intel/perfmon/main/SKL/events/skylake_core.json)

Therefore, neither of these conclusions is justified:

> Extra instructions are the established main limitation.

> Extra instructions cannot be an important limitation.

Claude is right that FOR-14’s old instruction-count model has not been confirmed. But replacing it with “the extra instructions probably delay the next prefetch” introduces another hypothesis, not a measured explanation.

That distinction also matters for `k` and `j`: removing a load/store pair could help even without saturating aggregate issue width, while shortening an instruction listing could still lose by lengthening an address dependency.

### 3. “The one-core conclusions transfer” needs its exact scope

The researcher is correct here: **D5 compares P with B, not P with M.** There are no four-core M measurements in this capture. The recorded 1.055 ratio of ratios supports the stability of the P/B relationship within the chosen tolerance; it does not establish that the P/M gap remains 9.6% under four-core contention. independent-review independent-review

Similarly, the 0.604-cycle P–M difference is a **net kernel contrast**, not a clean measurement of the output store alone. P includes its output machinery and assembly. The difference can also reflect how that work interacts with the rest of execution. walker-ladder-20261102T004502Z-… independent-review

### 4. “The penalty lives in L3/memory service” remains a leading explanation

The strongest observation is that concurrent copies of **A and B also slow down substantially**, despite omitting the production pipeline and decoded-output stores. That makes the full decoder’s job queue or assembly traffic unnecessary for producing a substantial slowdown in this simplified setting. independent-review

But it does not uniquely locate that slowdown in L3 request servicing, cache residency, prefetch resources or another shared-resource interaction.

Nor does the fill-buffer-full event identify average occupancy or the dominant cause of elapsed time. Intel defines it as occurrences of requests needing an unavailable fill buffer, including load, store and software-prefetch requests. It is not a percentage-of-time-full measurement. [GitHub](https://raw.githubusercontent.com/intel/perfmon/main/SKL/events/skylake_core.json)

**Memory-system interaction is a well-motivated architectural hypothesis. “We have identified its specific limiting resource” is not yet warranted.**

## This still narrows the search meaningfully

We should not let these qualifications obscure what the experiment achieved:

| Question | What this capture supports |
|---|---|
| Must we replace the perf backend to trust the main comparison? | **No.** |
| Is there a large demonstrated single-core bookkeeping opportunity? | **No; the registered stopping rule fires.** |
| Does prefetch improve this traversal? | **Yes, substantially in the A/B contrast.** |
| Can concurrency make traversal expensive without the full production pipeline? | **Yes.** |
| Have we measured eight total requests in flight or proved fill-buffer saturation is dominant? | **No.** |
| Have we resolved which memory architecture would deliver the big gain? | **Not yet.** |

The measured comparisons and scope limits support that distinction. walker-ladder-20261102T004502Z-… independent-review independent-review

## What I would tell Claude

This needs a **short interpretation amendment, not another experiment or another large review cycle**:

> Accept and close the valid walker-ladder capture under its registered decisions. Keep the repaired perf backend.
>
> Please tighten the descriptive conclusions: the single-lane/32-lane speed ratio does not independently measure eight lines in flight; D3 does not establish that execution resources are nonlimiting; D5 transfers P/B only, not P/M; and the P–M difference is a net contrast rather than an isolated output-store cost.
>
> Preserve fill-buffer limitation and slower memory service as hypotheses, clearly separated from measurements. The demonstrated architectural finding is that a substantial concurrency penalty persists in reduced walkers without the production pipeline.
>
> No rerun, new counters or collector migration is requested. Send this corrected interpretation to the ongoing Astra/researcher assessment of `k` and `j`, without treating either their success or failure as predicted by this capture.

**My overall view: Claude handled the experiment well; the researcher is doing the necessary job of keeping the explanation within the evidence.** We can accept the result and move forward without pretending it answers every hardware question.

Nothing further needs running on your workstation now.
`````
