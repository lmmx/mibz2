# External research report 5: the guardian's follow-up, the researcher's response and the guardian's review (verbatim)

Research inputs, not findings of this repository. The maintainer relayed them on
2026-10-30, after external report 4 (`2026-10-30-external-report-4.md`) and the
coordinator's first reading of it. There are three parts:

1. **The guardian's follow-up.** It is a further literature search on the same vein:
   bzip2 decoding on GPUs (2024), the construction of the 2012 super-alphabet,
   composition of reverse MTF and composable Ψ.
2. **The researcher's response** to the coordinator's first reading. It corrects that
   reading on two points:
   - the k·r bound on Ψ^k's breakpoints is a worst case, not a prediction;
   - #25 does not settle the question of composing reverse MTF.

   It also flags the #93/#114 contradiction.
3. **The guardian's review** of the researcher's response. It arrived as a separate
   message, and it includes the guardian's own reading of PRs #93 and #114.

The maintainer asked for parts 1 and 2 to be recorded verbatim. Part 3 is recorded
verbatim with them, so the exchange stays in one place.

They are source material for the research portfolio. Their claims are checked against the
primary sources and against this journal's history in the portfolio's register. They are
not taken as findings. They are also not authorisation to implement or benchmark
anything: each part says so.

The text reached us as rendered text. Citation markers are kept as received: the
`[image](…)` favicon links, the bare site names after them, the `&#x20;` entities and the
`utm_source` parameters.

## 1. The guardian's follow-up, as received

````text
# Searching for the inverse-BWT breakthrough: what the precedents actually enable

The deeper literature search strengthens the case for pursuing a breakthrough, but changes where I would look for it.

The most promising common principle is not simply parallelism or sequential scanning. It is finding a compact representation of what many dependent operations collectively accomplish.

That could mean replacing several Ψ transitions with one operation, representing the effect of a sequence of MTF instructions, or avoiding navigation altogether for a substantial reconstructed fragment.

There are also two important findings beyond the attached research note:

1. A 2024 bzip2 GPU decoder provides substantially more recent, implementation-backed precedent for inverse BWT and inverse MTF than the earlier theoretical literature.
2. The original 2012 super-alphabet paper has a construction detail worth examining very carefully: its optimized method constructs the two-character navigation representation from adjacent-symbol counts, rather than necessarily building Ψ² through a second round of arbitrary pointer gathering.

Neither proves a new mibz2 optimization exists. Both sharpen the novelty audit.

## 1. A new primary precedent: complete bzip2 decompression on GPUs

[Weißenberger and Schmidt, Massively Parallel Inverse Block-sorting Transforms for bzip2 Decompression on GPUs (2024)](https://doi.org/10.1145/3673038.3673067?utm_source=chatgpt.com)

The authors report parallel algorithms for inverse BWT and inverse MTF, and publish an implementation handling standard bzip2 streams. Their reported figures include up to 6.1× inverse-BWT speedup over a multicore CPU implementation, and combined inverse-BWT/MTF throughput around 2,400 MB/s on an A100 GPU.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://bibbase.org\&sz=32)

bibbase.org+1



Those are GPU results, not CPU predictions, and the paper's end-to-end server throughput is not comparable directly with mibz2's single-machine numbers.

The useful artifact is the actual [bzip2gpu source repository](https://github.com/weissenberger/bzip2gpu?utm_source=chatgpt.com).

A closely related older GPU paper, Parallel Lossless Data Compression on the GPU, described a parallel reverse-MTF formulation but left its implementation to future work. Its later thesis treatment implemented a reverse-MTF approach, reporting that reverse-MTF plus Huffman remained slower than the CPU counterpart. So the literature history is more developed than either “MTF is intrinsically serial” or “a scan makes it fast.”&#x20;

[image](https://www.google.com/s2/favicons?domain=https://escholarship.org\&sz=32)

escholarship.org+1



Breakthrough enabling condition: a representation of reverse-MTF state transitions that supports inexpensive composition and a cheap CPU execution path.

The critical questions for the 2024 code audit are:

- How does it divide inverse MTF into independent work?
- How much intermediate state is stored per chunk?
- Is its advantage fundamentally dependent on GPU-wide parallelism?
- Is there a narrower, work-reducing component that could benefit a four-core CPU?

I haven't inspected its CUDA kernels deeply enough to establish which of those mechanisms is present. But this is an unusually relevant source because it solves the actual decompression stages, rather than their forward transforms.

## 2. The 2012 super-alphabet construction merits exact reconciliation

In [Slashing the Time for BWT Inversion](https://www.cs.helsinki.fi/u/tpkarkka/publications/dcc2012.pdf?utm_source=chatgpt.com), Kärkkäinen, Kempa and Puglisi give two distinct ways of constructing the two-character representation.

Their explanatory pseudocode computes the next character and second successor by following first-order links. But the paper's optimized implementation avoids retaining the original LF array: it computes the pair information using character counts and constructs the second-order mapping through those pair counts.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://www.cs.helsinki.fi\&sz=32)

cs.helsinki.fi



This matters enormously given mibz2's historical result:

| Existing mibz2 evidence                                        | What remains to determine                                                     |
| -------------------------------------------------------------- | ----------------------------------------------------------------------------- |
| Pair walk construction: approximately 7.1–7.6 c/B              | Was this the same algorithmic construction as the optimized published method? |
| Existing table build: approximately 2.5 c/B in that comparison | Could a different construction avoid the observed cost?                       |
| Earlier Ψ² experiments failed overall                          | Was the failure intrinsic to the representation, the builder, or both?        |

There is no basis yet to declare this an untested win. In particular, even a cheap construction might leave an unacceptably large table.

But the existence of two construction pathways means the original failure must be attached to the exact pathway tested, not generalized to all super-alphabet methods.

This is a precise, historically grounded novelty condition.

## 3. The mathematical opportunity in reverse MTF is more specific than “parallelize MTF”

The GPU precedents and the work-optimal PRAM treatment by [Edwards and Vishkin (2014)](https://www.sciencedirect.com/science/article/pii/S0304397513007615?utm_source=chatgpt.com) suggest considering compositions of state transformations. The PRAM paper explicitly uses associative operations for several decompression stages, although its strong theoretical bounds do not guarantee efficiency on ordinary CPUs.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://www.sciencedirect.com\&sz=32)

ScienceDirect+1



There is a useful algebraic observation.

A move-to-front instruction at rank \\(r\\) acts on the current 256-element list as a permutation of positions. Consequently, a whole sequence of rank instructions induces a single permutation of those positions.

So, in principle, the effects of a chunk can be summarized independently of the incoming list.

That gives us an associative composition operation.

The problem is its price: a general chunk summary may require a 256-entry permutation, and composing two summaries may cost hundreds of operations.

A 256-byte summary that costs hundreds of operations is not a breakthrough when the original chunk contains only a few cheap MTF operations.

But that gives us an unusually clear research question:

Do real bzip2 MTF-index sequences admit a substantially smaller, faster-to-compose transformation summary than a general 256-element permutation?

There are concrete possible enabling conditions:

- Chunks contain very few distinct nonzero ranks.
- Most operations affect only a small prefix of the list.
- The composition can be represented as a few exceptional moves plus an identity-like remainder.
- RLE2 zero-run behavior allows large spans to be handled without ordinary MTF state updates.

The last point demands care: a rank-zero MTF operation does not change the list, and bzip2's RLE2 coding deliberately exploits zero runs. mibz2 already combines RLE2 and MTF processing, so any proposal must prove that it avoids work the implementation still performs.

An ideal first study would examine the actual post-Huffman token stream and calculate, per chunk, the minimum representation cost for its resulting MTF-state permutation. No new decoder required.

If those summaries remain effectively 256-element permutations, I'd deprioritize the idea for four-core CPU performance despite its theoretical elegance.

If large chunks routinely admit tiny summaries, that would be a genuinely interesting result.

## 4. The strongest Ψ opportunity remains structural shortcutting—not generic list ranking

The attached note correctly distinguishes contiguous Ψ translations from multi-step translations. A single translation interval does not automatically support a cheap Ψ² transition. That requires a compatible interval after the first mapping.

The 2012 super-alphabet paper and the 2026 length-capped move-structure paper illuminate different opportunities.

The former demonstrates the value of reducing dependent memory accesses; the latter establishes that structured permutations admit efficient traversal with compact run-based representations. But the 2026 result is an amortized linear-time result, not a claim of sublinear total decoding work or low per-step latency on mibz2.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://www.cs.helsinki.fi\&sz=32)

cs.helsinki.fi+1



The new opportunity would have to combine those strengths:

Use interval structure to eliminate consecutive dependent transitions, without paying for a full second-order table.

For each possible shortcut length \\(k\\), the census should determine:

| Quantity                                  | Why it matters                            |
| ----------------------------------------- | ----------------------------------------- |
| Fraction of actual traversal covered      | Determines potential usefulness           |
| Dependent transitions actually eliminated | Determines the true saving                |
| Total descriptor bytes                    | Determines cache-residency cost           |
| Lookup dependencies and branches          | Determines hot-path overhead              |
| Cost of recovering all \\(k\\) bytes      | Prevents hidden symbol-decoding penalties |
| Construction work per input byte          | Prevents repeating the Ψ² failure         |

For the read-only census, I would also distinguish an oracle upper bound from an implementable result.

An oracle might choose the best nonoverlapping shortcut at every point in the traversal for free. That establishes how much dependent work is theoretically avoidable.

The actual proposed representation must then pay to recognize eligible positions, retrieve metadata, emit characters and construct itself.

If even the oracle upper bound is small, the hypothesis is weak.

If the oracle removes most navigation operations but the proposed representation is still expensive, the representation—not the mathematical phenomenon—is the next research problem.

That is a useful way to preserve fundamental mathematical discoveries without pretending an implementation is already economical.

## 5. Why generic streaming inversion is less compelling

The original external-memory paper by Ferragina, Gagie and Manzini establishes scan-based inversion using sorting and fragment assembly, but its objective is minimizing expensive external-memory access with tightly constrained working space. It does not give a few-pass, CPU-efficient decoder for a 900 KB resident block.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://www.researchgate.net\&sz=32)

researchgate.net+1



A related theoretical warning comes from [Cotumaccio (2025), On Inverting the Burrows–Wheeler Transform](https://drops.dagstuhl.de/entities/document/10.4230/OASIcs.Grossi.17?utm_source=chatgpt.com): BWT inversion has an \\(\Omega(n\log n)\\) lower bound in the comparison model, whereas integer-alphabet settings admit linear-time algorithms. The lower bound does not rule out a linear-time, streaming algorithm for bzip2's byte alphabet. It does explain why generic sorting-based approaches should not be presumed cheap.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://drops.dagstuhl.de\&sz=32)

drops.dagstuhl.de



The relevant breakthrough condition is therefore very demanding:

> Can we assemble long output fragments and determine their global ordering without paying either a full pointer traversal or an expensive general sorting procedure?

I'd keep this as a scientific investigation, but below structural shortcuts and front-end state composition on the initial engineering ranking.

## 6. The updated breakthrough shortlist

1\. Composable Ψ shortcuts

Highest initial priority for a mathematical breakthrough

Establish whether a large fraction of dependent navigation steps can be removed economically. This has the closest connection to mibz2's existing structural evidence.

2\. Work-reducing reverse-MTF composition

Highest-priority newly surfaced front-end question

Study whether actual Wikidata MTF sequences admit compact transformation summaries. Audit the 2024 GPU implementation as concrete prior art, but target reduced CPU work rather than GPU-style parallel depth.

3\. Reconstructing the optimized super-alphabet builder

High historical-reconciliation priority

Determine whether the published pair-count construction was actually evaluated in mibz2 and whether it changes the recorded 7.1–7.6 c/B construction penalty.

4\. Work-reducing fragment reconstruction

High scientific upside, less certain engineering path

Investigate whether repetitions and BWT-specific structure permit large fragments to be reconstructed or ordered without proportional successor traversal.

These rankings are hypotheses about research value, not demonstrated performance.

One final observation from the literature: the strongest algorithms often change the representation of a sequence of operations, not merely their execution schedule. The key question is whether the effect of many expensive operations has a much cheaper description.

That's the enabling condition I'd explicitly hunt for across both inverse BWT and inverse MTF.

## A concise addition to Claude's research corpus

I've completed a deeper literature search for breakthrough enabling conditions. Please add these findings to the portfolio audit as research evidence, not authorization to implement anything.

1. **New primary precedent:** Weißenberger and Schmidt (ICPP 2024), *Massively Parallel Inverse Block-sorting Transforms for bzip2 Decompression on GPUs*. Its actual bzip2-compatible implementation is at [https://github.com/weissenberger/bzip2gpu](https://github.com/weissenberger/bzip2gpu) . Unlike some earlier GPU work, it implements inverse BWT and inverse MTF. Please inspect the algorithms and source, especially whether any part reduces total work rather than simply exposing GPU parallelism.
2. **Reverse-MTF composition:** A sequence of MTF rank instructions induces a permutation of the 256 list positions, so chunk transformations are composable. A general 256-entry summary may be prohibitively expensive. The useful open question is whether actual bzip2 token sequences admit much smaller summaries. Check whether existing RLE2/MTF fusion already exploits zero runs. A read-only corpus analysis of chunk-summary complexity could falsify this before implementation.
3. **Super-alphabet construction:** In Kärkkäinen–Kempa–Puglisi (2012), the optimized two-character construction uses pair counts and avoids retaining the original LF array, unlike the explanatory pointer-gathering pseudocode. Audit whether the failed mibz2 Ψ² experiments actually tested this construction. Do not label it novel unless that distinction survives inspection of the existing journal and code.
4. **Composable Ψ:** Distinguish oracle shortcut coverage from economically usable shortcuts. Measure nonoverlapping dependent transitions eliminated across real traversals, and account separately for descriptor bytes, byte reconstruction, fast-path access dependencies and construction cost. If the oracle saving is low, stop; if the oracle saving is large but representation overhead dominates, preserve that as an open data-structure problem.
5. **Streaming inversion:** Ferragina–Gagie–Manzini establish external-memory scan-based inversion, not a cheap constant-pass in-memory bzip2 decoder. Retain this as a scientific lead, but don't prioritize generic sorting or list ranking without evidence that BWT structure removes enough work.

Useful sources:

- [https://doi.org/10.1145/3673038.3673067](https://doi.org/10.1145/3673038.3673067)
- [https://github.com/weissenberger/bzip2gpu](https://github.com/weissenberger/bzip2gpu)
- [https://www.cs.helsinki.fi/u/tpkarkka/publications/dcc2012.pdf](https://www.cs.helsinki.fi/u/tpkarkka/publications/dcc2012.pdf)
- [https://www.sciencedirect.com/science/article/pii/S0304397513007615](https://www.sciencedirect.com/science/article/pii/S0304397513007615)
- [https://drops.dagstuhl.de/storage/00lipics/lipics-vol371-sea2026/html/LIPIcs.SEA.2026.9/LIPIcs.SEA.2026.9.html](https://drops.dagstuhl.de/storage/00lipics/lipics-vol371-sea2026/html/LIPIcs.SEA.2026.9/LIPIcs.SEA.2026.9.html)

Please reconcile these against H1–H5, E1–E11, the GPU-related research already inventoried, and all historical negative experiments. Keep any genuinely novel mechanisms in the breakthrough column, with explicit falsification conditions. No decoder implementation or i9 experiments yet.

My overall conclusion: the attached research note was on the right track, but the strongest new contribution from this deeper search is the concrete inverse-MTF precedent and the need to audit the exact optimized Ψ² construction. The best chance of a breakthrough still appears to lie in finding a compact summary of many serial operations—but the decisive question is whether that summary is cheap enough to beat performing them individually on the actual Wikidata workload.
````

## 2. The researcher's response, as received

````text
Claude has done something valuable here: it has challenged our proposed novelty, rather than simply adding another appealing idea to the portfolio.
But I think two of its conclusions need refinement before they become permanent entries in the research register.
1. The \(kr\) bound is correct in spirit—but it doesn't settle interval composition
For a permutation Ψ represented by piecewise translations, let \(B\) be the set of positions where the translation rule changes.
The discontinuities of \(\Psi^k\) must lie among the original boundaries and their preimages under earlier powers:
\[ B_k\subseteq\bigcup_{t=0}^{k-1}\Psi^{-t}(B). \]
Because Ψ is a permutation, each preimage set contains at most \(|B|\) positions. Consequently,
\[ \boxed{|B_k|\leq k|B|}. \]
Up to boundary conventions, this establishes the linear growth bound Claude describes. It's consistent with the interval-exchange and move-structure literature.
pmc.ncbi.nlm.nih.gov

But this is an upper bound, not a prediction of what happens on Wikidata.
If the preimage sets overlap substantially, the number of distinct boundaries could grow much more slowly.
Conversely, if they spread across the block, the useful translation intervals could quickly become short.
That distinction is precisely why the composition census has scientific value.
It would tell us whether the actual BWT permutation exhibits more compositional structure than the generic worst-case bound suggests.
Crucially, an interval representation of \(\Psi^k\) does not automatically recover the \(k\) output bytes. Those have their own information cost, which might eliminate the apparent advantage.
So I'd classify interval composition as related to the previously rejected move structure, but not experimentally settled by it.
We need a materially different lookup and construction mechanism before reopening implementation work. However, the read-only structural question remains legitimate.
2. The reverse-MTF question is not closed by #25
This is the larger issue.
Claude says two-symbol Huffman decoding shipped and the MTF experiment in #25 was rejected, then suggests that parallel scan formulations probably belong in the experimentally inferior category.
I don't think that conclusion follows.
#25 tested a particular MTF implementation optimization, not an algebraic reformulation of MTF decoding.
These are substantially different hypotheses.
The new question concerns the effect of an entire sequence of MTF operations on the list state.
And there is a structural property worth investigating before dismissing it.
An MTF operation at rank \(r\) changes only the first \(r+1\) positions of the list. Positions beyond \(r\) remain unchanged.
Therefore, for a chunk of MTF operations with maximum rank \(m\), the induced permutation affects only the first \(m+1\) positions.
That gives a simple exact bound:
\[ \boxed{\text{affected prefix length}\leq1+\max(r_1,\ldots,r_q)}. \]
Moreover, rank-zero operations leave the list unchanged.
This creates a genuine opportunity for compact chunk summaries if the post-RLE2 MTF ranks have sufficiently small maxima over useful chunks.
The catch is that an occasional large rank can expand the affected prefix dramatically. And even a compact state-transition summary doesn't by itself produce the decoded symbols without work.
A useful read-only census would measure the rank maxima, number of distinct positions affected and composition costs over chunks of different lengths.
So I'd retain this as an open mathematical question, with a clear falsification test.
It is not established by the rejected register-based implementation.
3. The most consequential finding might actually be in the contradictions list
Claude reports that #114 merged a design previously held back in #93, and that its own benchmark classified it as worse in CPU cost.
If that finding survives reconciliation, it deserves attention.
Why? Because our stated objective is higher full-decoder throughput at equal or lower CPU-seconds per GB.
A change can improve wall time while increasing total CPU work, but that's a different point on the performance frontier.
I'd want the register to establish whether #114's outcome reflects:

* A different test boundary.
* A throughput–CPU trade-off knowingly accepted.
* A regression introduced after the original experiment.
* Or an inconsistent acceptance decision.

We shouldn't assume which until Claude finishes the audit. But this is potentially more actionable than another literature lead.
4. What I'd communicate back
Two corrections for the portfolio audit:
Interval composition: I agree that this is closely related to the move-structure work in #42. But the bound of at most k·r translation discontinuities for Ψ^k is a worst-case upper bound, not evidence that the boundaries actually grow at that rate on Wikidata. The unresolved mathematical question is how much overlap occurs among the successive preimages of the breakpoints, and whether useful multi-step translations survive. Keep a read-only composition census open, while requiring a genuinely novel construction and hot-path cost model before any implementation.
Reverse MTF: Please don't classify state-composition or scan-based inverse MTF as experimentally inferior solely because #25's register-based MTF implementation was rejected. That experiment tested a different mechanism. A rank-r MTF operation changes only the first r+1 list positions, so an entire chunk affects at most the prefix determined by its maximum rank. Real post-RLE2 rank distributions could make compact chunk summaries possible. Whether they are cheap to compose and capable of reducing total CPU work remains untested. Keep it as a structural hypothesis with a cheap census and a strict work-cost test.
I agree that GPU-style parallelism alone is not evidence for a CPU breakthrough, and that the published Ψ² construction must be reconciled with the existing port.
One other item deserves careful review: the reported #93/#114 contradiction involving CPU cost versus the merged design. Please preserve the original benchmark boundaries and acceptance criteria before assigning a verdict.
No new implementation or workstation experiment is implied by these corrections.
My assessment of Claude's progress
I like the direction of the audit. It has already identified historical inconsistencies, recovered a missing-inventory conclusion, and started separating genuinely new questions from previously tested mechanisms.
But we should be cautious about premature mathematical closure.
An earlier failed implementation is strong evidence against that implementation. It is not automatically a theorem about every algorithm using a related mathematical property.
Equally, a mathematical property is not sufficient justification for building a decoder.
That's the balance we want.
The most interesting remaining opportunities are precisely those where we have both an exploitable mathematical structure and an unresolved question about whether it can reduce real CPU work.
I would keep the interval-composition and reverse-MTF investigations alive at the read-only stage.
And I'd regard the emerging contradictions register as a major achievement in its own right: it's making our next experiments more reliable before we've spent another minute of workstation time.
````

## 3. The guardian's review of the researcher's response, as received

````text
The researcher is right to push back here. Claude is doing excellent historical reconciliation, but it's beginning to conflate a failed implementation with a failed mathematical direction. That's precisely what our novelty conditions were intended to prevent.

I see three important distinctions.

### 1. Interval composition is related to M2, but it isn't experimentally settled by M2

Claude is right that both approaches exploit piecewise translations of Ψ. That makes the M2 results directly relevant.

But a k-step representation could have a different ratio of lookup cost to useful output. The fact that M2 required approximately 41 instructions per step doesn't establish that a compositional shortcut must pay 41 instructions for each original step.

If a descriptor advances several bytes with one lookup, its cost is amortized differently. Equally, the descriptor could be much more complicated and lose even more badly.

The researcher's mathematical point is correct: the bound

\\[ |B_k|\le k|B| \\]

doesn't predict actual breakpoint growth. It establishes a maximum. The overlap among breakpoint preimages—and the resulting distribution of usable shortcut lengths—is still an empirical question.

My classification: related prior work, unresolved structural hypothesis, implementation conditional on novel cost evidence.

### 2. Reverse-MTF composition is not refuted by #25

This is the clearer error in Claude's assessment.

A register-based implementation of conventional MTF updates and a chunk-level algebraic representation of their combined effect are different algorithms.

The rank-prefix observation is exact: an MTF update at rank \\(r\\) affects only positions \\(0\\) through \\(r\\). Hence a sequence whose maximum rank is \\(m\\) acts as the identity outside the first \\(m+1\\) positions.

That doesn't make composition profitable. In particular, computing the output symbols still matters, and large ranks could destroy compactness.

But it gives us a mathematically specific and cheaply testable hypothesis.

My classification: open structural hypothesis; no decoder implementation until the summary-size and total-work tests pass.

### 3. The #93/#114 contradiction deserves independent scrutiny

This may be the most consequential issue in the current audit.

If a construction change was rejected for CPU cost and later merged despite that finding, we need to understand whether the comparison changed, whether an improvement elsewhere outweighed the extra CPU, or whether the acceptance logic was inconsistent.

That's not evidence yet that #114 should be reverted.

But given our breakthrough objective—higher throughput at equal or lower CPU-seconds per GB—it's exactly the sort of discrepancy the new register must resolve before prioritizing new work.

## 4. The #93/#114 discrepancy is real, but the PRs reveal an important distinction

I checked the two pull requests directly.

[PR #93](https://github.com/lmmx/mibz2/pull/93?utm_source=chatgpt.com) — original one-pass build

Registered an end-to-end improvement requirement and a particular CI build-stage threshold. Its original design removed the initialization pass and wrote whole table entries directly.

[PR #114](https://github.com/lmmx/mibz2/pull/114?utm_source=chatgpt.com) — later one-pass build

Used the same core idea, but included additional correctness handling and different stated acceptance criteria: faster build-stage timing, no slower thread count, and output parity.

That establishes at least one genuine discrepancy: the registered acceptance rules were not identical.

It does not, by itself, establish a CPU-efficiency regression. We still need to reconcile the specific benchmark that reported “Worse (CPU cost)” against the later CPU-seconds measurements.

A faster build stage and higher CPU usage can coexist, particularly when other work becomes exposed or concurrency changes. The verdict must use matched boundaries and hardware.

So I would classify this as acceptance-criteria drift requiring reconciliation, rather than evidence that the shipped constructor was mistakenly accepted.

## What I would communicate back

Thanks for preserving the researcher's report and reviewing it critically. I agree with the novelty discipline, but please retain two distinctions in the register.

**Interval composition:** the bound of at most k·r breakpoints is a worst-case upper bound, not a measurement of breakpoint growth or usable shortcut coverage on Wikidata. Keep the read-only composition census as an open structural question. Classify any implementation as conditional on a materially different construction, byte-recovery method and hot-path cost model from M2. Do not use M2's 41-instruction result as a universal per-shortcut lower bound.

**Reverse MTF:** #25's failed register-based implementation is not an experiment on chunk-level composition of MTF transformations. An operation of rank r changes only the first r+1 list positions, and a chunk affects only the prefix determined by its maximum rank. Keep compact chunk summaries as an unresolved mathematical hypothesis, subject to rank-distribution, representation-size, output-recovery and CPU-work analysis. Do not classify it as experimentally inferior solely by association with #25.

I also checked mibz2 PRs #93 and #114. Their original acceptance criteria differ, although their underlying one-pass construction is closely related. Please reconcile the actual CPU measurements and benchmark boundaries before determining whether there was a resource-efficiency regression or an inconsistent merge decision.

Preserve these as audit findings, not instructions to implement or benchmark anything yet. The portfolio should distinguish failed implementations from unsolved mathematical opportunities while maintaining strict performance plausibility tests.

My view: the researcher has correctly identified two hypotheses that should survive the historical audit, without yet earning implementation approval. And Claude has surfaced a potentially valuable process finding in #93/#114 that deserves a precise, evidence-based resolution.

This is the productive tension we want between the researcher and Claude: one protects mathematical possibility, the other protects the accumulated negative evidence. The portfolio should preserve both, rather than allow either to prematurely close the question.
````
