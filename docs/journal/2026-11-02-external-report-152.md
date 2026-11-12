# External research report 152: the researcher's top tier after the walker ladder: P87's production follow-up, wake placement, packed-state compact navigation (`k`, `j`); the concurrency question as support (verbatim)

A research input, not a finding of this repository.

- **Relayed:** by the maintainer, 2026-11-02.
- **Source:** the independent researcher of reports 138–149.
- **In reply to:** the maintainer's question, quoted below.

> alright so using the records, can you give a "top tier" listing (ignoring the discarded things, they are just adding noise) of which items we now update to see as our most promising candidates for further work? I feel like we have alluded to this but not yet fully itemised them. we had that massive list of ideas earlier in the project and have listed off some here but I want to see the most promising ones enumerated now.

**Evidence cutoff:** 2 November 2026, after the walker-ladder capture and report 151. A
reading of the records; no new run.

It is kept exactly as received (rendered text, cards flattened).

## The message, as received

`````text
## The updated top tier

**I would now keep three candidates in the top tier: P87’s production follow-up, wake placement within the four-core budget, and the packed-state compact walker represented by `k` and `j`.**

I went back through the [original research portfolio and hypothesis register](https://github.com/lmmx/mibz2/blob/9723f06847b88a10ba93d07c3f35680e58766015/docs/journal/2026-10-30-research-portfolio-register.md), then reconciled those entries with their later results. This is a shortlist for the current objective: **better decoding throughput and CPU efficiency within the same four-physical-core allocation**.

My order below reflects **the expected return from the next bounded piece of work**, taking evidence, possible benefit and remaining effort together. The largest speculative upside is a different question.

| Priority | Candidate | Why it belongs in the top tier | What kind of opportunity it is |
|---|---|---|---|
| **1** | **P87: reuse the dead last-column buffer in the production decoder** | A modest benefit was measured in the diagnostic; the production implementation already exists and passes correctness checks. | **The most mature incremental engineering candidate.** |
| **2** | **Improve wake placement and pipeline utilization within the existing core budget** | The dispatch records identify runnable workers waiting while an allocated CPU is idle. There is a specific utilization problem to target. | **The strongest remaining utilization hypothesis.** |
| **3** | **P38/P40: cheaper compact navigation, starting with packed-state `k` and `j`** | The compiler has emitted substantial reductions in extraction and lane-state work, while retaining the compact representation. | **The strongest remaining concrete architectural candidate, with substantial performance risk.** |

Those are three different reasons to invest. **P87 has the clearest evidence of a small benefit; wake placement has a concrete observed opportunity; compact navigation has the more ambitious potential but the least established net benefit.**

## 1. P87 production follow-up: finish the most mature modest improvement

### The actual proposal

After constructing the navigation table, reuse the now-dead last-column buffer, `ll`, for the reconstructed bytes. The production candidate also avoids allocating the separate `rle` buffer.

The implementation is already in [PR #128](https://github.com/lmmx/mibz2/pull/128). Its decoder changes are small—29 lines across the two production source files—and its correctness coverage includes reused scratch space, different block sizes, parallel decoding, cancellation and error paths. The production-shaped candidate remains parked and untimed. Avoiding `rle` removes a buffer of approximately **0.9 MB per worker at maximum block size**; the production effect on peak RSS still needs validation.

### Why I rank it first for the next bounded engineering investment

The earlier diagnostic supplies actual positive evidence on the primary four-worker Wikidata workload:

| Diagnostic result | Estimate | 95% interval |
|---|---:|---:|
| Throughput change | **+1.35%** | +0.29% to +2.41% |
| CPU cost per GB | **−1.71%** | −2.37% to −1.06% |

These are diagnostic A/B results, rather than a measured improvement of the production candidate over the shipped decoder. The registered engineering decision remains **NOT DEMONSTRATED**, because the other workload controls did not establish their required no-harm conditions. The concurrency-specific mechanism result also remains inconclusive against its materiality threshold. [P87’s primary result and closure](https://github.com/lmmx/mibz2/blob/a0094cfa0cef985e9f57fa9b384656b7ae5bcaad/docs/journal/2026-11-01-p87-four-core/i9-10900K/analysis.md)

Nevertheless, this is the strongest combination of **measured benefit, implementation readiness and limited remaining engineering scope** among the survivors.

### How the walker result changes its priority

It makes me more comfortable treating P87 as an independent, modest optimization. Its value does not depend on finding a large hidden bookkeeping problem in the walker.

The new result also helps keep its ambition realistic: output-free walkers still suffer substantial concurrency penalties, so buffer reuse should not be presented as the main solution to concurrent traversal cost. It can still be worthwhile on its own.

### The next useful work

**Complete the production validation proposal already associated with PR #128:** review the production implementation and machine code, finish the comparison protocol, and ensure the controls are capable of answering the no-harm questions.

That is a concrete completion task. The existing diagnostic is already finished; its result does not need to be rediscovered.

**My assessment:** the best candidate for a small production improvement, subject to its remaining validation.

## 2. Wake placement: recover useful execution time inside the four-core allocation

### The actual proposal

Improve how the decoder’s workers, scanner and reader are woken and placed, so that ready work can use an available allocated core promptly.

This is **pipeline and operating-system scheduling**, distinct from the walker’s internal lane scheduling. It needs a specific policy whose effect can be traced through the whole decoder.

### Why it remains promising

The dispatch investigation identified **5.65% of aggregate worker time** in a particularly relevant condition:

> A worker was runnable in the receiver-acquisition phase while at least one allocated CPU was idle.

The longer waits often involved a worker placed on the CPU of the thread that woke it, then waiting behind that thread while another allocated CPU became available. The study explicitly preserves wake placement as an **unresolved, independently testable hypothesis**. [Dispatch analysis, especially §§4 and 8](https://github.com/lmmx/mibz2/blob/4ef5f36d5cf0193d9183b8f599d1105f7719d66d/docs/journal/2026-11-01-dispatch-causality-i9-10900K/analysis.md)

That is more specific than “the pipeline sometimes goes idle.” It identifies a plausible interaction and the circumstances in which it costs time.

The **5.65% is an observed worker-time category, not a predicted throughput gain**. Recovering it requires a schedule that uses the available core without imposing a larger cost elsewhere. The scanner and reader also need execution time, and the study’s different delay categories cannot simply be added into one savings estimate.

### How the walker result changes its priority

The ladder does **not** undermine this candidate. Its small P/M gap concerns work inside the traversal kernel. It does not establish that the full decoder already makes good use of every allocated core.

However, the concurrency finding adds an important requirement: **getting more workers running can make each worker more expensive**. A placement policy must therefore improve completed decoding throughput and CPU efficiency, rather than merely reducing wake latency.

That makes this a coupled utilization problem. More activity is useful only when it produces enough additional completed work.

### The next useful work

**Specify one bounded placement or wake policy against the existing traces.** It should explain:

- Which observed delay it addresses.
- How workers, scanner, feeder and reader all fit within the same four-core allocation.
- Why the improved placement is expected to outweigh any loss of locality or increased contention.
- Which whole-decoder outcomes would establish that it helped.

The records do not yet justify selecting a particular implementation merely because it sounds plausible.

**My assessment:** the strongest remaining lead for improving utilization, with credible observational evidence and no demonstrated intervention benefit yet.

## 3. Packed-state compact navigation: retain `k` and `j` as one tightly bounded architectural candidate

### The actual proposal

Use the compact navigation records while making the per-lane state and extraction sequence cheaper.

The key improvement already implemented is to carry the row and output byte in a single token. That removes a real lane-state load and store and simplifies how the next state is formed. The two concrete alternatives are:

| Candidate | Ordinary-step instructions | Loads | Stores | Lane state | Distinguishing change |
|---|---:|---:|---:|---:|---|
| **`k`** | **31** | **4** | **2** | **32 bytes** | Packed token and simpler record addressing |
| **`j`** | **27** | **4** | **2** | **32 bytes** | `k`, plus explicit multiplication for the packed-field index |

The BMI2 baseline used 37 instructions, five loads, three stores and 40 bytes of lane state. Both new arms passed the recorded release and debug correctness checks, including exceptional rows and awkward segment boundaries. Neither has been timed. [FOR-14 journal, packed-token check](https://github.com/lmmx/mibz2/blob/9d22ac9164d0bba46a4942cf4c8001e40d75eca4/docs/journal/2026-11-01-for14-stage1.md#8a-the-packed-token-check-reports-138139-compile-and-correctness-only)

### Why it still belongs in the top tier

There are two substantive pieces of evidence behind it:

1. **Changing the representation can change the concurrency behaviour.** The earlier compact-table experiment moved the best-concurrency point from four blocks to six, with accompanying cache-hit evidence supporting the relevance of table footprint. [Compact-Ψ Stage A result](https://github.com/lmmx/mibz2/blob/9d22ac9164d0bba46a4942cf4c8001e40d75eca4/docs/journal/2026-10-29-compact-psi.md#5-the-result-measured-the-i9-b99b92b-2026-10-29)
2. **The new implementation removes actual work.** The load/store reduction and smaller state are present in compiled code. Their benefit is still unknown, but the mechanism is concrete.

That earns this family a place above speculative formats whose only support is a paper instruction estimate.

### How the walker result changes its priority

It sharpens the question considerably:

> **Can a compact representation reduce concurrent traversal cost enough to pay for the extraction and state work it adds?**

The ladder shows that the concurrent traversal penalty persists even after much surrounding production work is removed. It also shows little demonstrated headroom from ordinary cleanup of the shipped walker. Those findings make a representation-specific improvement relevant, while making a generic rewrite less attractive. [Recorded assessment of the successful capture](https://github.com/lmmx/mibz2/blob/1abeb0cbd669ba64691ed28504bda3c493acf3f2/docs/journal/2026-11-02-external-report-149.md)

They do **not** establish that compactness will win, that cache capacity is the dominant limit, or that the issued-µop figure provides spare capacity for extra extraction work.

I would retain **both `k` and `j`**, giving `k` particular attention as the cleaner hypothesis: it removes the load/store pair without adding `j`’s longer multiplication dependency. A shorter listing does not settle which will run faster.

### The next useful work

**Finish the architectural assessment of these two concrete candidates and define the evidence needed for them to earn further investment.**

A direct compact constructor—**P84**—belongs behind that decision. It is not an independent top-tier project at present. Navigation must first establish a plausible savings budget, and construction must then fit inside it.

I would also keep any cheaper extraction or slightly different record layout within this **one candidate family**, rather than treating every encoding variation as another promising project.

**My assessment:** the strongest remaining concrete architectural lead, but a high-risk one. Its code-generation improvement is established; a net performance improvement is not.

## The supporting research question: what makes concurrent traversal more expensive?

There is one research question I would keep alongside those three, without counting it as a fourth optimization:

**P117 and the related concurrency question: why does essentially unchanged traversal work require substantially more cycles when several copies run?**

In the matched execution-counter records, instructions per row changed by less than **0.03%**, while cycles per row rose by approximately **37–43%** across A, B and P. That makes the question consequential. The capture still does not distinguish cache residency, request servicing, prefetch interactions and other shared-resource effects sufficiently to select a specific remedy. [Matched-counter analysis](https://github.com/lmmx/mibz2/blob/1abeb0cbd669ba64691ed28504bda3c493acf3f2/docs/journal/2026-11-02-external-report-149.md) · [Interpretation safeguards](https://github.com/lmmx/mibz2/blob/1abeb0cbd669ba64691ed28504bda3c493acf3f2/docs/journal/2026-11-02-external-report-150.md)

Its value is to improve the cost model for concrete candidates, particularly compact navigation. It should support a decision about an implementation, rather than become an open-ended measurement programme.

**The current portfolio therefore has one mature modest improvement, one specific utilization hypothesis, and one substantial architectural gamble.** That is the top tier I would carry forward now. None yet establishes a route to a standalone 1.5× improvement, and this ranking does not authorize another workstation run.
`````
