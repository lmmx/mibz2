<!--
Provenance: the research chatbot's "Phase II" research agenda, 29 October 2026, written
after the ψ census (2026-10-29-succinct-review.md §9) and relayed by the user into the
working session to be recorded ("Record the research agenda from the researcher"). The
text below is unedited, including its own formatting, escapes and citation links. It is a
proposal, not a measurement or a decision; the coordinator's response, which sets the
priorities, is in coordinator-phase-2.md beside it.
-->

# mibz2 succinct research agenda — Phase II: From compressibility to navigability

The new census changes what we should be researching.

Until now, we've been asking whether the structure of Ψ could support a compact representation. Claude's results make that question much less speculative.

The reported findings are striking:

# 99.51%

Rows fit FOR-14 or EF-12

# 77.6%

Rows permit translation

# 16.12

Modeled bits per row

These are reported census results, not independently reproduced measurements. Nevertheless, assuming the census is correct, they justify a real change in research priority.

My new position: the primary research problem is no longer whether Ψ can be compacted. It is how much decoding machinery we can eliminate while preserving that compactness.

And I've found a particularly relevant paper that should reshape our next literature investigation.

## 1. The new paper I'd put at the centre of our research

[Improved and Extended Locating Functionality on Compressed Suffix Arrays](https://users.dcc.uchile.cl/~gnavarro/ps/jda15.2.pdf?utm_source=chatgpt.com) — Simon Gog, Gonzalo Navarro and Matthias Petri (2015 preprint).

This is much closer to our precise problem than the general Barbay–Navarro compressed-permutation theorem.

The authors directly investigate how to encode Ψ using compressed increasing integer lists. They compare techniques including partitioned Elias–Fano and patched frame-of-reference compression, and discuss handling transitions between increasing runs.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://users.dcc.uchile.cl\&sz=32)

users.dcc.uchile.cl



That is a remarkable overlap with our work.

Their objectives are primarily compressed suffix-array operations, not one-shot high-throughput BWT inversion. But the underlying representation problem is extremely similar.

It means we should not simply treat FOR-14/EF-12 as an isolated invention. We can now compare it with a developed family of Ψ encodings.

A second important reference is [Partitioned Elias–Fano Indexes](https://pages.di.unipi.it/rossano/assets/pdf/papers/SIGIR14.pdf?utm_source=chatgpt.com) by Ottaviano and Venturini (2014). This work explicitly studies splitting monotone lists into locally encoded chunks to exploit clustering while preserving efficient random access.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://pages.di.unipi.it\&sz=32)

pages.di.unipi.it



The particularly relevant observation is that ordinary Elias–Fano does not automatically exploit long runs of consecutive values efficiently. The authors identify local clustering as a reason to partition sequences.

That sounds familiar for a reason: Claude's 77.6% finding tells us that local consecutive structure is abundant in the tested Ψ.

So the literature now points toward piecewise-monotone, locally encoded, directly addressable Ψ as our central subject.

## 2. The five research questions I'd prioritize

I. What exactly is the structure of the 77.6% translation population?

Is it dominated by long runs, short repeated patterns, or a particular class of BWT inputs?

We need to understand the distribution of maximal translation lengths, not just coverage by fixed groups of 32.

A crucial mathematical distinction: a 32-entry group with consecutive Ψ destinations is not necessarily the same object as a maximal BWT run. For Ψ, the corresponding occurrence positions in L must be consecutive.

Research deliverable: a model of translation-run length, tail distribution and how it relates to BWT run count.

II. What is the simplest possible 64-byte Ψ record?

Not the smallest record. The simplest.

We should compare translation, frame-of-reference, Elias–Fano and patched FOR under an identical 512-bit budget.

Count exact loads, shifts, masks, bit-select operations, branches and serial dependencies to recover one arbitrary entry.

Research deliverable: complete record layouts and per-format navigation dependency graphs.

III. Can we eliminate format-dependent branching?

A 77.6% translation fast path sounds attractive, but a branch on every navigation step isn't free.

Could translation groups be encoded as a special case of the same arithmetic used for FOR, allowing one common extraction path?

Alternatively, could a record's format determine the decoding operation using small branchless arithmetic?

Research deliverable: a unified representation or evidence that explicit format specialization is cheaper.

IV. Can the next pointer be recovered without a second dependent memory access?

This is our fundamental hardware constraint.

A 64-byte record may contain all the necessary information, but the processor still has to decode it.

The objective is to make the critical path approximately:

\\[ \text{record load}\rightarrow\text{register operations}\rightarrow\Psi(j) \\]

rather than:

\\[ \text{load}\rightarrow\text{load}\rightarrow\text{select}\rightarrow\Psi(j). \\]

Research deliverable: a navigation cost model that distinguishes critical-path latency from total instructions and memory traffic.

V. How does compressed Ψ behave under fundamentally different input distributions?

The existing census is encouraging, but the corpus has a highly unusual bimodal block-size distribution.

We should study the representation's behaviour on random, highly repetitive, low-alphabet, structured and adversarial inputs.

In particular, what is the worst-case fraction of exception records for a fixed 32-row grouping?

Research deliverable: distribution-dependent space bounds and exception-rate bounds, not merely corpus averages.

## 3. A new idea: the 77.6% result may let us simplify the decoder rather than merely compress it

Here's something I want us to investigate carefully.

Suppose a record contains 32 consecutive Ψ values:

\\[ \Psi(32b+i)=p_0+i,\qquad 0\le i<32. \\]

The obvious representation is:

- A 20-bit base.
- A format marker.
- Segment flags.

But why retain an entire 64-byte record for these groups?

Because fixed-size addressing is valuable. Yet we might be able to encode translation groups in the same 64-byte format as ordinary groups without introducing distinct decoding machinery.

For example, a generalized FOR record could use:

\\[ \Psi(j)=base+f\_{\text{record}}(j\bmod32) \\]

where:

\\[ f\_{\text{record}}(i)= \begin{cases} i&\text{translation}\\\ d_i&\text{FOR}\\\ \text{EFdecode}(i)&\text{EF}. \end{cases} \\]

The question becomes whether we can design \\(f\\) so that the common case is cheap without requiring a complicated dispatch.

A subtle possibility is to make translation a degenerate FOR representation with an implicit sequence of offsets.

Then the format selection might simply control whether an offset is extracted or taken directly from the row's low five bits.

That sounds straightforward, but we'd need to establish whether the format-dependent select operation introduces more latency than a predicted branch.

The important thing is that we should optimize the next-address calculation, not just the compression algorithm.

## 4. Two research tracks are now worth keeping distinct

Track A — Fastest compact Ψ

Primary

The goal is a directly indexed 64-byte record, cheap single-entry decoding, and near-16-bit storage.

This is our most credible new performance-research direction.

Track B — Succinct Ψ approaching entropy

Exploratory

The goal is to exploit translation runs, monotone lists and adaptive partitioning to compress well below 16 bits per row.

This is the deeper succinct-data-structure problem, with a higher risk of additional dependent loads.

One very important distinction: Track B does not have to outperform Track A to be valuable.

It may establish a new space–navigation trade-off or expose techniques that can be brought back into the fixed-record design.

For example, partitioned Elias–Fano may reveal how to handle exceptional spans more efficiently even if variable-length records themselves are unsuitable for the hot traversal.

## 5. A research hypothesis we can now make quantitative

For the fixed-record design, suppose total throughput is constrained by three overlapping resources:

\\[ T\_{\text{step}}\gtrsim \max\left( T\_{\text{issue}}, T\_{\text{memory}}, T\_{\text{dependency}} \right). \\]

Where:

- \\(T\_{\text{issue}}\\) measures the instruction/µop throughput needed per navigation.
- \\(T\_{\text{memory}}\\) measures sustainable memory traffic and misses.
- \\(T\_{\text{dependency}}\\) measures serial progress along each chain, after accounting for independent lanes.

This is a simplified lower-bound model, not a complete processor-performance equation.

The census changes our estimates of the first two terms:

Memory: A nominal 16-bit representation approximately halves the randomly accessed table footprint compared with the current 32-bit representation, before exceptions and metadata.

Instructions: With 77.6% translation-eligible rows, a large fraction of steps may avoid packed-field extraction and in-register select.

Dependency: Still unknown, and potentially the most decisive term.

That means we have strong new reasons to believe the representation could be compact, and a plausible reason that decoding might be inexpensive. But we don't yet know the cost of getting the next address onto the CPU's load-address-generation path.

This is precisely the performance frontier I want us to characterize.

## 6. One concern I would elevate: exact encoding correctness

Before treating the 16.12-bit estimate as a settled result, I want a complete accounting of what must be recoverable from each record.

Our original 512-bit layouts include a 32-bit segment-marker bitmap. That is convenient, but it isn't information intrinsic to Ψ.

Likewise, the original layout reserves an output symbol. A single symbol describes all rows only when the group lies entirely within one F interval.

Mixed-symbol groups therefore need either:

- A different symbol-recovery path.
- Separate symbol metadata.
- Or an exception representation.

For every record, we need to recover:

\\[ (\Psi(j),\ \text{segmentMarker}(j),\ F[j]) \\]

or establish that one of those fields can be recovered independently without introducing another costly memory dependency.

This is important because a representation that stores Ψ efficiently but requires an additional unpredictable access to obtain the output symbol is not equivalent to the current T-vector.

I'd keep this triple as the formal definition of the record's required functionality.

There's also a further architectural subtlety: the 64-byte record is the unit of addressing, not necessarily the unit of loading. Its metadata and selected packed offsets may require separate load instructions, even though they occupy the same line.

Cache-line residency, load count and load dependency depth must be tracked separately.

## 7. The literature research I would conduct next

My recommended reading order has changed.

First priority

[Gog, Navarro & Petri — Improved and Extended Locating Functionality on Compressed Suffix Arrays](https://users.dcc.uchile.cl/~gnavarro/ps/jda15.2.pdf?utm_source=chatgpt.com)

Especially the treatment of Ψ as increasing lists and comparisons of alternative list encodings.

This is the strongest direct prior-art connection we've uncovered.

Second priority

[Ottaviano & Venturini — Partitioned Elias–Fano Indexes](https://pages.di.unipi.it/rossano/assets/pdf/papers/SIGIR14.pdf?utm_source=chatgpt.com)

Study why local clustering makes ordinary EF inefficient, and how partitioning changes its random-access cost.

Third priority

[Ferragina & Pucci — Generalized Elias–Fano project](https://gef.di.unipi.it/?utm_source=chatgpt.com)

Explore newer Elias–Fano variants, especially those designed for fast random access and runs in high-order bits. The claimed throughput and space advantages should be treated as workload-specific until independently evaluated.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://gef.di.unipi.it\&sz=32)

Compressed Indexing for Arbitrary Sequences



The first paper is particularly intriguing because it evaluates multiple methods specifically on Ψ. We may discover that some of our proposed encoding choices already have a close analogue in compressed suffix-array engineering, saving us from repeating dead ends.

# My recommendation for the next research checkpoint

Rather than broaden the agenda further, I'd now want a Ψ representation atlas: a compact mathematical description of the measured Ψ groups, their exact encoding possibilities and their navigation costs.

It should answer four questions:

1. Which kinds of Ψ groups occur, and how frequently, across individual full-sized blocks?
2. Which groups can be represented by translation, FOR, EF or patched FOR, with exact accounting of bytes and exceptions?
3. Which layouts can recover the next index from one directly addressed record with no additional unpredictable memory access?
4. How do the resulting layouts compare in estimated bits per row, decoding dependencies, and construction requirements?

That would turn the census from an encouraging feasibility result into a representation-design foundation.

The most exciting question now is whether 77.6% translation coverage allows a common navigation path whose CPU cost approaches the current direct-pointer load, while the other 22.4% is handled by slightly more expensive compressed formats.

If the answer is yes, we could be looking at something more interesting than the original 24-bit compromise: a physically compact Ψ representation designed from the beginning around out-of-order execution and memory-level parallelism.

Our research stance should now be: stop treating Elias–Fano as the proposed solution, and treat it as one component in a broader search for the fastest cache-line-addressable representation of Ψ.

The census has earned that change.
