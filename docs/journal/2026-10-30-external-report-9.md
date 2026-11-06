# External research report 9: back to the BWT itself, trajectories, and multi-byte inverse-BWT records (verbatim)

A research input, not a finding of this repository. The maintainer relayed it on
2026-10-30, after external report 8 (`2026-10-30-external-report-8.md`), and it is
recorded verbatim as the earlier reports were. It has three parts, in the order
received:

1. **A redirection:** away from general CPU optimisation (SIMD, the four-rank MTF
   table), back to inverse BWT as a structured permutation problem. The readings named
   are Kärkkäinen–Puglisi 2011, Baier 2018 (tunnelling) and Brown–Langmead 2026.
2. **The researcher's deductions** from those papers:
   - translation structure and copying are two views of synchronised trajectories;
   - recognising a shortcut at every row in the hot loop may be what failed;
   - the obstruction is row-space regularity against traversal-space regularity.
3. **"A concrete mathematical result: multi-byte inverse-BWT transitions":**
   - intervals on which Ψ, …, Ψ^k are translations, refined at the F-bucket boundaries,
     carry a constant k-byte output word;
   - the bound R_k ≤ 1 + k(r + σ − 2);
   - variable-length records;
   - a read-only oracle over the actual cycle.

It is source material for the research portfolio. Its claims are checked against the
portfolio's §3.1, where the coordinator derived the same output-word result
independently, and against the register. They are not taken as findings. It is not
authorisation to implement or benchmark anything.

The text reached us as rendered text. Its citation markers are kept as received: the
`[image](…)` favicon links, the bare site names after them, the `&#x20;` entities and the
`utm_source` parameters.

## The report, as received

````text
You're right. We've drifted from fundamental algorithms for decoding existing bzip2 blocks into general-purpose CPU optimization literature.

SIMD is a possible implementation technique, not the research question. And a four-rank MTF lookup table, however elegant, isn't the sort of breakthrough we're primarily hunting.

I'd bring us back to the intersection of three subjects:

1. Inverse BWT as a structured permutation problem.
2. Reconstruction of text without following every permutation edge.
3. Algorithms that exploit BWT runs, repeated substrings and interval mappings to reduce total decoding work.

In particular, I think we should return to the literature on BWT tunnelling, s-runs and repeated-path reconstruction—but read it with a different question from the previous Precopy experiments.

Not how do we implement Precopy more efficiently?

Rather: What structural information is already present in a standard BWT block that might allow us to reconstruct an entire substring without following its individual Ψ transitions?

That's much closer to our real objective.

## The book I'd actually pull down next

Cache Friendly Burrows-Wheeler Inversion, Kärkkäinen and Puglisi, 2011.

[Full paper](https://www.cs.helsinki.fi/u/tpkarkka/publications/ccp2011.pdf?utm_source=chatgpt.com)

We've cited it many times, but I'd like to reread its chain-detection construction and invariants, alongside the later 2012 Precopy algorithm.

The reason is subtle. The paper doesn't merely observe that repetitive text contains repetitions. It detects parallel LF paths during inversion and replaces later random traversals with copying. That's a direct example of reducing the amount of expensive work in the precise operation mibz2 performs.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://citeseerx.ist.psu.edu\&sz=32)

citeseerx.ist.psu.edu



I'd then pair it with two more focused readings:

- [Baier — On Undetected Redundancy in the Burrows–Wheeler Transform](https://arxiv.org/abs/1804.01937?utm_source=chatgpt.com) (2018), to understand when these parallel paths can be recognized as larger structural objects.
- [Brown and Langmead — Bounding the Average Move Structure Query for Faster and Smaller RLBWT Permutations](https://pmc.ncbi.nlm.nih.gov/articles/PMC13523032/?utm_source=chatgpt.com) (2026), especially the inversion section, to distinguish amortized improvements in LF navigation from genuinely skipping LF operations.&#x20;

  [image](https://www.google.com/s2/favicons?domain=https://pmc.ncbi.nlm.nih.gov\&sz=32)

  pmc.ncbi.nlm.nih.gov



The question I want to carry through all three is:

Could we identify a cheaply representable family of entire Ψ trajectories, rather than individual Ψ transitions?

That would unite the strongest parts of our research—translation intervals, run structure, compositional shortcuts and repeated-path reconstruction—without retreating to general SIMD tricks.

And we have a particularly valuable constraint: our existing negative Precopy and M2 results tell us what any new construction must outperform.

So yes, back to our realm: the combinatorics of the Burrows–Wheeler transform itself, with existing bzip2 blocks as the non-negotiable input.
````

````text
You're right. Recommending papers we've already read isn't getting us anywhere.

The interesting question is what their mathematics tells us that our failed experiments haven't exhausted.

I think there are three useful deductions.

1\. Translation structure and copying are two views of the same phenomenon. Kärkkäinen–Puglisi's parallel LF paths, Baier's tunnels and our Ψ translation intervals describe related forms of synchronized permutation trajectories. But they exploit them differently: M2 represents transitions compactly; Precopy tries to avoid revisiting trajectories altogether.

The unexplored question isn't whether those structures exist. We know they do. It's whether we can discover and exploit them collectively, avoiding the per-byte overhead that defeated M2 and Precopy.

2\. The failure may be in paying to recognize a shortcut at every row. Both approaches introduce machinery into a hot loop that was already very efficient. A potentially different algorithm would identify reusable regions during construction and hand the walker longer, preclassified tasks. But doing so might merely move the same work into preprocessing. We'd need an amortization proof over one 900 KB block.

3\. The missing piece is a cost model for reconstructing trajectories. A BWT run gives adjacent rows with adjacent successors. It does not automatically give consecutive characters in decoding order. The distinction between row-space regularity and traversal-space regularity is the critical obstruction.

So I'd stop collecting generic references and ask a much sharper question:

> Can we use the run structure of Ψ to derive a small set of long, output-producing trajectories, with less total work than explicitly following all \\(n\\) links?

That is the point where those three papers meet—and where our experiments leave a genuinely difficult question.

The most promising synthesis is that the next breakthrough may depend on treating repeated trajectories as a single object during construction, rather than accelerating their detection during decoding.

That suggests a specific, falsifiable direction: identify maximal families of rows whose Ψ trajectories remain parallel for several steps, then determine whether each family can be represented once and reused to reconstruct multiple output substrings.

The 2011 Precopy work exploits this locally. Baier's tunnelling work identifies related structures globally. Move structures encode their transitions. What remains uncertain is whether there is a cheap enough construction that combines those advantages for an ordinary bzip2 block. The literature confirms the underlying interval regularity but not that proposed combination's profitability.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://pmc.ncbi.nlm.nih.gov\&sz=32)

pmc.ncbi.nlm.nih.gov



I would investigate three quantities together: the total number of distinct trajectory families, their useful lengths, and the number of row visits that can actually be avoided after construction. Those would tell us whether the proposal has any hope of outperforming mibz2's existing cooperative walk.

But I also want to be careful about novelty. The existing Precopy and move-structure research already gets close to this idea. Unless we can identify a substantially cheaper construction or reconstruction method, it isn't a new algorithm—it's a restatement of one we've tested.

So my conclusion from those papers is not that we should try Precopy again. It's that they expose a precise missing mechanism: amortizing the discovery of repeated traversal structure over enough output to eliminate more work than discovery costs.

That's a worthwhile theoretical question. It is not yet a breakthrough result.

And you're right to call out the repeated reading recommendations. We should now look for genuinely new constructions in adjacent research, rather than keep circling these same three papers.
````

````text
# A concrete mathematical result: multi-byte inverse-BWT transitions

I've found a more promising formulation than our earlier ideas about simply compressing Ψ or composing its intervals.

We can represent several successive inverse-BWT transitions as a single interval operation that also contains all the output bytes.

The important part is the output bytes. That was the missing piece in our earlier composition discussions, and a major obstacle in #112.

This construction follows from the piecewise-translation property used in move structures. The literature establishes that property for BWT permutations, but the multi-byte representation and cost analysis below are deductions for our problem—not a claimed published speedup.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://pmc.ncbi.nlm.nih.gov\&sz=32)

pmc.ncbi.nlm.nih.gov+1



## 1. The starting point

Let Ψ be mibz2's inverse-BWT navigation permutation.

On a translation interval \\(I\\),

\\[ \Psi(j)=j+d_I. \\]

The existing move-structure work stores the displacements and determines which interval contains the current row.

Its problem on our workload was the cost of locating and navigating intervals for every emitted byte.

Now consider not one transition, but \\(k\\) transitions:

\\[ j,\Psi(j),\Psi^2(j),\ldots,\Psi^k(j). \\]

Suppose there is a source interval \\(I\\) on which every power of Ψ up to \\(k\\) acts as a translation.

Then we have constants \\(d_0=0,d_1,\ldots,d_k\\) satisfying

\\[ \Psi^t(j)=j+d_t \quad\text{for all }j\in I,\quad 0\le t\le k. \\]

That gives a single \\(k\\)-step successor:

\\[ \boxed{\Psi^k(j)=j+d_k.} \\]

But can we output all \\(k\\) characters without following the intermediate links?

Yes, provided we further partition the interval by output-symbol boundaries.

## 2. The output-word theorem

Let \\(F\\) be the first column of the sorted BWT matrix, so the output symbol associated with a row is determined by its F-bucket. Under our Ψ convention, the sequence emitted by \\(k\\) forward steps can be expressed using

\\[ F(j),F(\Psi(j)),\ldots,F(\Psi^{k-1}(j)). \\]

The precise output orientation may need reversing for bzip2 reconstruction, but that doesn't affect the argument.

The F column consists of at most 256 contiguous symbol buckets.

On an interval where \\(\Psi^t(j)=j+d_t\\), the \\(t\\)-th output symbol becomes

\\[ F(j+d_t). \\]

That is constant until \\(j+d_t\\) crosses an F-bucket boundary.

So if we subdivide the source interval wherever any of the \\(k\\) output positions crosses such a boundary, the entire \\(k\\)-byte output word is constant throughout each resulting interval.

We can therefore associate a record with an interval:

\\[ \boxed{ I\longmapsto (d_k,\ c_0,c_1,\ldots,c\_{k-1}) } \\]

such that for every row \\(j\\) in that interval, the record gives both:

- the next row after \\(k\\) transitions;
- all \\(k\\) emitted bytes, in their correct order.

There is no need to recover a symbol from an additional per-row F lookup during the shortcut.

This is an exact representation theorem, subject to constructing the correct interval partition.

## 3. A useful bound on the number of records

Suppose Ψ has \\(r\\) translation intervals, and the alphabet contains \\(\sigma\\) symbols.

First consider the boundaries needed to make

\\[ \Psi,\Psi^2,\ldots,\Psi^k \\]

act as translations on every interval.

If \\(B\\) denotes the original translation-breakpoint set, the breakpoints of successive compositions are contained in

\\[ B,\Psi^{-1}(B),\ldots,\Psi^{-(k-1)}(B). \\]

Thus the total number of translation breakpoints is bounded by approximately

\\[ k(r-1). \\]

Now we need the output word to remain constant.

Within an interval where \\(\Psi^t\\) acts as a translation, the function

\\[ j\mapsto F(\Psi^t(j)) \\]

can change only when the translated row crosses an F-bucket boundary.

Because \\(\Psi^t\\) is a permutation, each of the \\(\sigma-1\\) F-bucket boundaries can introduce at most one additional breakpoint across the entire partition for that step.

Across \\(k\\) output positions, this adds at most

\\[ k(\sigma-1) \\]

boundaries.

Combining the bounds gives:

\\[ \boxed{ R_k\le 1+k(r+\sigma-2) } \\]

where \\(R_k\\) is the number of intervals sufficient to encode the \\(k\\)-step successor and its complete output word.

This is a worst-case upper bound, not an estimate of actual Wikidata structure.

It is also a constructive statement: partition at the relevant preimages of translation and symbol boundaries, and the required records exist.

### What this means for a 900 KB block

Taking the previously observed approximate figure \\(r=56{,}000\\) and \\(\sigma\le256\\):

| Shortcut length | Upper bound on records | Average rows per record at that bound |
| --------------- | ---------------------- | ------------------------------------- |
| 2               | \~112,509              | \~8.0                                 |
| 4               | \~225,017              | \~4.0                                 |
| 8               | \~450,033              | \~2.0                                 |
| 16              | \~900,065              | \~1.0                                 |

These numbers are derived bounds, not measured record counts.

They show why this could either be interesting or fail spectacularly.

If the bounds are nearly attained, long shortcuts become too fragmented. If repeated preimages overlap substantially, the actual number of records might be much smaller.

That is the first essential uncertainty.

## 4. Why this formulation is different from M2 and the previous compact-Ψ experiments

Our failed M2 structure represented a one-step permutation and paid interval-lookup overhead per decoded byte.

The proposed multi-byte records would represent a different operation:

One interval lookup, multiple emitted bytes

At \\(k=4\\), one descriptor could provide four bytes and advance the row by four Ψ steps.

The potential advantages are:

- No dependent lookup for the three skipped intermediate successors.
- No separate per-byte symbol-directory access.
- Output bytes arrive together and could potentially be written as a word.
- Interval metadata may be shared across multiple starting rows.

This directly targets the mechanisms that made #112 and #42 unsuccessful.

But there are equally serious liabilities.

Finding the containing interval may still cost more than a direct T-vector lookup. If we require a large auxiliary row-to-interval map, the memory advantage can disappear.

Constructing the records may be expensive. The benefit must pay back within one block, not across repeated queries to an index.

The output words may be repetitive but the interval descriptors may not be. Compressing them further could add yet another dependent load.

And multi-step segments must respect the cooperative walk's boundaries. A shortcut cannot silently skip a segment endpoint or corrupt output ordering.

None of those costs is settled by the theorem.

## 5. A potentially decisive alternative: variable-length records

The fixed-\\(k\\) bound suggests something else.

We might not want Ψ⁸ everywhere.

Perhaps a source interval admits eight safe steps, another four, another two, and another only one.

Instead of forcing every row into a uniform Ψ⁸ representation, imagine records containing:

\\[ (\text{interval},\ k,\ \text{successor displacement},\ \text{output word}). \\]

Here \\(k\\) varies.

That would let the representation exploit long composable trajectories where they exist without paying the full breakpoint explosion everywhere.

This is the part I find most interesting.

The right objective would not be maximizing the number of long intervals. It would be minimizing the total cost of reconstructing one complete traversal:

\\[ \text{construction cost} + \text{descriptor lookup cost} + \text{output cost} + \text{fallback cost}. \\]

A read-only oracle can tell us how much potential such variable-length shortcuts have before we attempt any implementation.

The oracle would choose nonoverlapping shortcuts along the actual permutation cycle, using exact output-word intervals, and count the number of dependent transitions eliminated.

Then we would impose realistic descriptor-size and lookup-cost constraints.

If the oracle cannot eliminate a large fraction of transitions, this direction is unlikely to produce the breakthrough.

If it can, the unsolved problem becomes very specific: how to index those multi-byte records cheaply enough that a four-core CPU benefits?

## 6. The research result—and its limits

I want to be precise about what we've achieved here.

Established mathematically

Piecewise translations of Ψ can be composed and refined into intervals that carry both a multi-step successor and a constant output word. The record-count bound above follows from the breakpoint structure.

Still unknown

Whether actual Wikidata blocks have enough repeated compositional structure, whether the records can be built cheaply, and whether finding a record costs less than the transitions it replaces.

Not demonstrated

A faster decoder, lower CPU-seconds per GB, or a performance breakthrough. No such result can be claimed without the structural census and implementation measurements.

The result is related to established move structures and super-alphabet inversion; I have not established that the construction is novel in the published literature.

But it gives us an unusually concrete research target.

We are no longer merely asking whether Ψ² intervals compose.

We have an exact way to ask:

> Can the inverse-BWT permutation be partitioned into cheaply addressable records that each reconstruct several bytes, so the decoder executes substantially fewer dependent memory operations?

And unlike the earlier interval-composition proposal, the output bytes are now part of the representation itself.

That is the most substantive mathematical result I've found in this line of investigation so far.

It is not yet the breakthrough performance result we're seeking. The decisive next step is a read-only census of the actual multi-step record partition and its viable cost—not another general literature survey, and not another implementation built on hope.
````
