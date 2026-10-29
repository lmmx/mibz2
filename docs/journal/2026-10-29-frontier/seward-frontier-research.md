<!--
Provenance: a conversation between the user and the research chatbot on Seward's 2001
space–time trade-off and whether mibz2 can shift it outward, relayed by the user into the
working session on 29 October 2026, unedited. The user's two prompts are quoted; the
researcher's two answers follow each. The chat copy turned the researcher's figures into
their captions and labels only, and its links into titles followed by the site's name;
both are kept as received.
-->

I said to the succinct researcher:

> Okay, so I've left the agent like validating things. Can we just go back to that 2001 paper though? Was it Seward which talked about the trade-off? So there was this diagram we made at one point when we were discussing all of this, in which we said that there was like a merge, merge TL and index F. And there was basically like a Pareto frontier between them, between throughput and memory use. And we hypothesized an improvement because Seward largely evaluated how to invert an individual block within a particular memory budget. But our recent experiments have additionally exposed shared cache capacity as a performance constraint across concurrent blocks. A representation that's slower for one isolated block might become faster overall if its smaller footprint keeps several independent tables in L3. And this raised the particularly interesting possibility here that instead of simply moving along Seward's frontier, less memory in exchange for less speed, we might be able to shift the frontier outward. Do you still think that we've like established anything towards this goal? And can we track this in any way if it's something that we want to move towards?

it said

===

Yes. And I think revisiting Seward now, after the literature review and the recent mibz2 measurements, makes our original hypothesis substantially more interesting.

We have established evidence for the mechanism that could shift the frontier outward. We have not yet demonstrated the shift itself.

That distinction matters, because there’s already a genuine research result here—and a very clear way to establish the stronger claim.

1. What Seward’s frontier represented

The paper was Julian Seward’s 2001 Space-time Tradeoffs in the Inverse B-W Transform⁠.

He examined different representations of the information needed to invert a BWT block, trading memory against the work performed during navigation.

Two important examples were:

* mergedTL: keep the next-position information and character together, giving fast navigation at higher memory cost.
* indexF: avoid storing the character alongside every next-position link by recovering it from the first-column character intervals, reducing storage at the price of extra lookup work.

The paper goes further into lower-memory variants. Seward also argued that BWT inversion speed would be fundamentally constrained by servicing cache misses. ResearchGate +1

Our original illustration was approximately this:

The original single-block trade-off

Moving left saves memory, but tends to slow each lookup.

That is a fair description of the historical design problem, although the curve is our interpretation, not an exact frontier measured by Seward.

2. Our new idea changes the objective

Instead of asking:

How quickly can we invert one block with a particular memory allowance?

We’re asking:

How many bytes per second can the machine decompress when several independent blocks are active and sharing a finite cache?

This introduces an effect that a single-block speed–memory plot doesn’t capture.

Suppose a compact representation makes individual navigation 8% slower, but reduces the working set enough that four active blocks can remain cache-resident instead of two or three.

Then the whole machine might become faster, despite every individual compressed lookup doing more work.

That’s the proposed frontier shift.

Our hypothesis: concurrency changes the attainable frontier

A crucial nuance: to establish a genuine Pareto improvement, the comparison must use the same performance metric and workload. We cannot simply put Seward’s isolated-block throughput and our multicore throughput on the same axis and declare one superior.

What we can establish is that a new compact design achieves higher machine-level throughput at lower table memory usage than our current 32-bit representation under identical concurrent workloads.

That would be a real outward shift in the mibz2 system-level frontier.

3. What we’ve actually established already

I would separate our evidence into four findings.

A. Shared cache capacity is a real constraint in our workload.

Our full-sized bzip2 blocks each need approximately 3.6 MB for the current 32-bit T-vector.

On the i9-10900K, with 20 MB of shared L3, the measurements exposed a performance knee as several full tables became active. Four such tables alone occupy approximately 14.4 MB, before other working data.

This is the most important direct evidence supporting our original idea.

B. There is substantial, measured redundancy in Ψ.

The 32-row census found approximately 77.4% translation-eligible rows in full blocks.

Our fixed-line design is modelled around 16.16 bits per row, compared with 32 bits for the current T-vector.

That corresponds to roughly 1.82 MB per full block instead of 3.6 MB, before any additional allocation or construction overhead.

This supports the feasibility of reducing the footprint. It does not yet establish fast lookup.

C. The literature offers credible low-latency compressed formats.

Uniform Ψ blocks, local FOR, small Elias–Fano bitvectors, and cache-line-contained access are all established techniques.

This removes much of the theoretical uncertainty about whether such a representation is plausible.

It does not remove the hardware uncertainty around dependent Ψ navigation.

D. Construction efficiency has already improved materially.

#81 demonstrates that eliminating construction overhead can improve combined build-and-walk performance.

That gives us a stronger baseline and a useful construction principle.

But #81 itself is not evidence that compressing the navigation table improves throughput.

Taken together, these results establish something stronger than idle speculation:

We have independently observed both a cache-capacity problem and substantial compressibility in the structure causing that problem.

What we haven’t established is that the additional decoding cost is small enough for the latter to solve the former.

There is also an interesting precedent worth acknowledging: Kärk­käinen and colleagues’ Slashing the Time for BWT Inversion⁠ (2012) already investigated cache-efficient inversion and parallel recovery. So the broad idea that cache and concurrency change inverse-BWT performance is not new. Our narrower opportunity lies in the particular combination of compact Ψ representation, cooperative scheduling and simultaneous block residency. University of Helsinki

4. The concrete threshold we’re trying to cross

Here’s the simple capacity picture for full-sized 900,000-row blocks.

Navigation-table footprint across simultaneous blocks

Decimal MB, for full-sized blocks. The processor’s nominal 20 MB L3 is not a guarantee that 20 MB of navigation tables can reside there: other data, cache associativity and access patterns matter.

This is why our idea is interesting.

A compact representation could bring six simultaneously active full-block tables down from 21.6 MB to approximately 10.9 MB.

Whether that translates into faster decoding depends on measured cache misses, decoding overhead and scheduling. But it’s a substantial change in the working-set regime, not a tiny incremental saving.

And it tells us where to look for a frontier shift: near the concurrency levels where the current representation starts suffering from cache pressure.

A compressed representation might lose at one active block and win at four or six. That would be an especially compelling confirmation of our original hypothesis.

5. How I would track the frontier

I’d make this a named research objective rather than treating it as an incidental benefit of compact Ψ.

Research objective: shift the inverse-BWT throughput–memory frontier

Hypothesis

Reducing the per-block navigation working set allows more independently decoded blocks to remain in shared cache, increasing whole-machine decompression throughput enough to offset the extra work required to decode compact Ψ records.

Current status: Mechanism supported; end-to-end frontier improvement not yet demonstrated.

I would track it in three parts.

Question	Measurement	What would count as evidence
Does compactness improve residency?	L3 misses, memory traffic, active table footprint versus block concurrency	A reproducible reduction in cache pressure near the known knee
Does it improve system throughput?	Complete build-plus-walk MB/s at matched concurrency	Compact format overtakes direct T-vector as concurrency increases
Is it a true Pareto improvement?	Throughput against peak memory across all tested configurations	A compact configuration uses less memory and achieves higher throughput than the relevant direct-table alternatives

The third standard is deliberately demanding.

For example, beating a 4-block direct-table configuration with a 4-block compressed configuration is encouraging. But it isn’t the complete frontier comparison if a 3-block direct configuration achieves even better throughput.

We should compare each candidate against the best available direct-table configuration, not an artificially disadvantaged one.

The most revealing visualization

I’d maintain two related plots:

Throughput vs concurrency

Does the compact format overtake as concurrency increases?

Throughput vs peak memory

Does the achievable speed–memory frontier actually move outward?

Direct table

Compact Ψ

Illustrative shapes only—not benchmark data.

For the experiments, I’d distinguish navigation-table footprint, peak RSS, and total allocated working memory rather than mixing them. Table footprint is the proposed mechanism; peak memory and throughput are the externally meaningful outcomes.

I’d also keep build and walk timings separate, even while using full decode throughput as the ultimate objective. That will reveal whether we have a good compression scheme held back by its constructor, or a constructor that’s fast enough but whose decoder is inherently too expensive.

6. The result that would convince me most

Imagine the eventual measurements looked like this:

* At one active full block, direct 32-bit navigation is faster.
* At three active blocks, the formats are competitive.
* At five or six active blocks, compact Ψ wins because substantially more of its working set remains in L3.
* The compact format’s peak memory remains lower, and its complete build-plus-walk throughput exceeds the best direct-table configuration.

That pattern would be unusually persuasive.

It would demonstrate not just that we’ve compressed a table successfully, but that the optimal representation depends on the number of concurrent decompressions competing for cache.

It might even justify an adaptive implementation that uses a direct table at low concurrency and a compact table at high concurrency. That’s a possible consequence, not something we need to commit to now.

Conversely, if compact Ψ reduces L3 misses but loses overall because of dependent decoding instructions or costly construction, we’d have falsified the performance hypothesis while confirming the memory model. That would still be a useful result.

7. My assessment after the literature review

I would describe our progress this way:

Cache-capacity constraint

Measured

Substantial Ψ redundancy

Measured

One-cache-line compressed lookup

Feasible on paper

Direct compressed construction

Not established

Faster whole-machine decoding at lower memory

Not yet demonstrated

I’d call this a well-supported architectural hypothesis with a clear experimental test, rather than a demonstrated optimization.

But it’s not merely speculative anymore. The measurements establish both the bottleneck and an unusually large opportunity to reduce the working set responsible for it.

That is considerable progress toward the original idea.

And I think this is a worthwhile overarching goal to preserve in the journal, independent of whether the winning implementation uses FOR, Elias–Fano, plateau descriptors, 24-bit links or something we haven’t thought of.

The research question is bigger than any one codec:

Can we turn the memory saved by compressed Ψ into greater aggregate inverse-BWT throughput, instead of merely accepting slower navigation in return for lower memory use?

That is the proposed departure from Seward’s original trade-off—and it’s now precise enough to measure and potentially prove experimentally.

===

I said

> Yes, exactly, um that sounds really compelling Okay, I see that you're suggesting we record this with more plots. So I guess rather than, so I guess what we're after then is, is to show these plots as yet another benchmarks page, I suppose. And then to update these with each version and each PR perhaps, yeah. Well, ideally just each version, yeah. But maybe certain PRs, I don't know. Okay, yeah that sounds definitely doable. I can pass that over to my coordinator agent then

it said

===

Yes. I think a dedicated Throughput–Memory Frontier benchmarks page would be an excellent way to make this a lasting part of mibz2’s performance story.

And I’d favour releases as the primary historical record, with selected PRs appearing as experimental candidates. That gives us a stable, readable history without turning the page into a stream of transient benchmark results.

I’d pass the coordinator something like this:

Proposal: Throughput–Memory Frontier benchmarks

Create a benchmark page tracking whether mibz2 can improve aggregate decompression throughput while reducing its navigation-table memory footprint.

Research hypothesis: Compact Ψ representations may be slower for an isolated block but faster under concurrent decompression because more active navigation tables fit in shared cache. The goal is to establish whether we can shift the system-level throughput–memory Pareto frontier outward, rather than merely move along Seward’s original space–time trade-off.

Two principal plots

1. Throughput vs. concurrent blocks: Compare complete build-plus-walk throughput across concurrency levels for each representation. Look for cache-residency crossover points.
2. Throughput vs. memory: Plot throughput against measured peak memory, showing the nondominated configurations and whether later versions improve the frontier.

Where practical, also record navigation-table footprint, build and walk time separately, L3 miss statistics and CPU model.

Historical tracking

* Releases: Permanent reference points, tagged with the mibz2 version and benchmark environment.
* Selected PRs: Optional experimental overlays, clearly distinguished from released results.
* Comparability: Keep workload definitions, full-block counts, benchmark commands, hardware, concurrency and measurement methodology recorded. Do not join incomparable machines or workloads into one frontier.
* Reproducibility: Preserve machine-readable benchmark results alongside the rendered plots.

What counts as success?

A new configuration should demonstrate higher aggregate throughput at lower measured memory usage than the relevant existing configurations under the same workload and hardware.

An improvement at fixed concurrency is useful evidence, but the actual frontier should compare the best configurations across concurrency levels.

The page should distinguish established benchmark results from theoretical footprint estimates and proposed designs.

One subtlety worth emphasizing: the frontier is hardware-specific. An L3-driven improvement on the i9-10900K may appear at a different concurrency level—or not appear at all—on another processor.

So I’d keep separate frontiers by CPU, rather than mixing measurements.

And there’s a nice narrative payoff here. The page could eventually show three generations of progress:

Seward’s trade-off → mibz2’s cooperative traversal → compact Ψ exploiting shared-cache residency.

The third stage is still a hypothesis, but a living benchmark page would let us document precisely when, and whether, it becomes a measured result.

That seems worth preserving even if our first compact representation doesn’t win.

===
