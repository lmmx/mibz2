<!--
Provenance: the research chatbot's synthesis at the end of the literature study. The user
showed it the working session's commentary on deep dives 4–7 (the cross-checks in
`../2026-10-29-literature-study.md`), and this is its reply, relayed by the user into the
working session on 29 October 2026, unedited. The coordinator's notes on it are in
10-synthesis-coordinator.md. Some inline mathematics was dropped or survived as LaTeX
source; both are kept as received. The inline figures arrived as plain lines. Tables
arrived tab-separated and are kept so.
-->

This is excellent feedback from the coordinator. And having read all five updates together, I think they reveal something important:

Our literature programme has converged. We are no longer short of promising mathematical ideas. We are short of a very specific set of measurements that will tell us which ideas are worth pursuing.

I also appreciate that the coordinator has been checking our theoretical claims against mibz2’s actual implementation rather than simply accepting them. Several of those checks materially improve our research.

1. The biggest development: our remaining uncertainty has become very small

Across Sadakane, Partitioned Elias–Fano, Gog–Navarro–Petri and patched integer coding, we kept arriving at the same question:

What do the 22–23% of nontranslation groups actually look like?

We know that approximately 77.4% of rows in full-sized blocks belong to groups where all 32 Ψ values are consecutive.

But the remaining groups might have dramatically different structures.

Three possibilities for a nontranslation group

A. One discontinuity

Two long consecutive sequences, joined by one jump.

B. A handful of discontinuities

Several short consecutive sequences; perhaps still cheap to encode.

C. Many discontinuities

Highly irregular positions, potentially better suited to FOR or Elias–Fano.

Schematic patterns, not measured corpus distributions.

If most nontranslation groups look like A, our two-plateau hypothesis becomes much more interesting.

If most look like C, further plateau-specific research probably has little value.

And the coordinator has identified that the existing census already collects enough underlying information to answer this.

That’s the decisive next research result, not another literature review.

2. The marker question is potentially even more consequential

This is the observation from the coordinator that most changes my thinking.

Our proposed 64-bit directory descriptor allocates:

Field	Bits
Segment markers	32
Base address	20
Output byte	8
Format tag	4
Total	64

We have been treating those 32 marker bits as unavoidable.

But are they?

The segment markers exist because of how mibz2 schedules cooperative traversal, not because they are intrinsic to the BWT permutation.

If they can instead be represented through lane state, segment-length information or another cheap mechanism, we could reclaim up to half the descriptor.

That leads to an intriguing result:

A two-plateau group requires approximately 45 bits of navigation information: two 20-bit destination bases and a five-bit split position.

Add an eight-bit output symbol and a four-bit format tag:

45+8+4=57\text{ bits}.

It could fit in a single 64-bit descriptor if the 32 marker bits were no longer required there.

That would mean both translation groups and two-plateau groups could be navigated without a secondary payload load.

But there is a very important qualification: we have not established that marker information can be eliminated from the hot lookup without increasing instructions, dependency latency or checkpoint complexity.

The coordinator is right to identify this as a fundamental architectural question.

The hypothetical benefit is enormous; the implementation constraint is real.

3. The output-byte issue is much closer to settled

The coordinator’s inspection of src/bwt.rs confirms a useful detail.

Although the current walker obtains the output byte from the loaded T-vector entry, that byte is mathematically determined by the current row’s F interval.

For a group contained entirely within one F-symbol interval, every row has the same output symbol.

Therefore a compressed group can store the symbol just once.

Only groups crossing F-symbol boundaries require special treatment.

For bzip2, there are at most 255 such boundaries, so among 28,125 complete groups, fewer than 1% can be affected.

That’s a strong structural advantage.

It means output-byte recovery is no longer a major exception-handling concern for the compact architectures.

It does not mean the common-case symbol extraction is free, but we can keep it local to the descriptor rather than requiring another unpredictable table lookup.

4. The published evidence is now surprisingly coherent

I would summarize our findings as follows.

Idea	Evidence
Ψ has monotone character-specific lists	Mathematical fact
Consecutive Ψ blocks can be represented by translation	Established prior art
Most full-block rows in our corpus are translation-eligible	Measured
FOR-14 and EF-12 cover almost every row in the existing census	Measured format suitability
Fixed cache-line Elias–Fano is practical	Published implementation
Select within a machine word can be efficient	Published implementations and CPU instructions
Directly constructed compressed Ψ can beat fused #81	Unknown
Two-plateau descriptors avoid payload loads	Conditional on marker treatment and plateau prevalence
A smaller table improves cooperative throughput	Unknown until measured

There is no need to keep rediscovering the same representational ideas.

The literature has already established their fundamental legitimacy.

Our distinctive question is how to integrate them with an unusually fast, cooperative, one-shot inverse-BWT decoder.

5. What I’d change about our research agenda

I now see three genuinely decisive research results ahead of us.

First: the plateau census

This is the highest-value missing piece of data.

It should tell us how often each category appears among globally aligned 32-row groups:

Number of plateaus	What we learn
1	Existing translation fast path
2	Potential direct two-base descriptor
3–4	Potential compact multi-plateau payload
5–8	Could support plateau encoding or FOR
9+	Likely benefits less from plateau specialization

Groups crossing F-symbol boundaries should be counted separately.

I would also preserve the span distribution, because it lets us evaluate the precise EF-12 representability condition.

The important result will be the joint distribution of plateau count, span and boundary status.

These properties aren’t interchangeable.

A group can have very few plateaus but a huge numerical span, or many plateaus within a relatively small span.

Second: the segment-marker audit

This is now the most interesting unanswered architectural question.

We need to distinguish two things:

* The information required to identify segment endpoints.
* The information that must be available at each hot-loop navigation step.

If endpoint detection can be handled without per-row marker bits in the record, the descriptor design changes fundamentally.

But the answer has to come from the exact scheduling invariants of the room walker.

I would not assume that the marker bitmap can simply disappear because it contains relatively little information.

Third: the successor-throughput experiment

This is where the literature eventually has to face the CPU.

The right experiment isn’t a random-access benchmark over independent keys.

It’s multiple dependent Ψ chains, with the number of independent lanes varied from one to 32.

That would distinguish:

* Isolated successor latency.
* Throughput from memory-level parallelism.
* The cost of compressed record decoding.
* The benefit of reduced cache footprint.
* The additional latency from exceptional payload accesses.

It is the experimental counterpart to everything we’ve read.

One correction to our earlier excitement

I agree with the coordinator’s clarification of #81.

The 31.3% construction improvement is against the given-counts cooperative baseline, which has its own overhead. The reported 1.643× comparison against production refers to the broader inversion-only configuration, not solely to the build phase.

Those results should not be conflated.

This matters when discussing how much room remains for compressed construction to improve performance.

Our new compact formats need to compete against the best relevant fused pipeline, measured on the same workload and with the same included stages.

Where this leaves us

I think the coordinator has done exactly what we needed: converted our literature exploration into a small number of falsifiable research questions.

The most exciting possible outcome is now quite specific.

Suppose the census reveals that a large share of the nontranslation groups contain only two plateaus.

And suppose the marker audit finds a cheap way to handle segmentation without a 32-bit marker bitmap in every descriptor.

Then we might be able to represent substantially more than 77% of the Ψ table with directly navigable 64-bit descriptors, without additional payload lookups.

That would be a much stronger architecture than our original two-tier proposal.

On the other hand, if most nontranslation groups contain many discontinuities, or marker elimination proves expensive, we have a clear reason to return to FOR/EF payloads.

Either result would be useful.

My conclusion: the literature phase has succeeded, and the next intellectual breakthrough is likely to come from the plateau histogram and marker invariants rather than another paper.

I’d preserve the research synthesis, but I wouldn’t finalize its recommendations until those two facts are known. They determine whether our most ambitious descriptor architecture is a realistic candidate or merely an attractive bit-budget calculation.

