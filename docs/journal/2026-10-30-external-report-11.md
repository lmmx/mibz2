# External research report 11: research burn-down rounds 3–5 (verbatim)

A research input, not a finding of this repository. The maintainer relayed it on
2026-10-30, continuing external report 10 (`2026-10-30-external-report-10.md`). It has
three parts, in the order received.

1. **Round 3.**
   - The researcher's exhaustive check: 58,212 cases, every primitive cyclic string over
     three symbols up to length 8, with k = 1…6. The union equals the minimal
     output-producing partition in every case.
   - A periodic counterexample (`aaaa` with one tie order).
   - A proof of the equality for a stable, bucket-monotone Ψ. Within one F-bucket,
     Ψ(y) − Ψ(x) ≥ y − x, so a separation that opens cannot close while the words
     agree. That gives M_k = 1 + Σ min(k, g_i).
   - The Ψ⁸ storage threshold, Σ max(g_i − 8, 0) ≳ 619,000.
   - Construction raised to the principal risk.
2. **Round 4.**
   - |B| = r − 1, so the one-step intervals are L's runs in F order.
   - A run-native propagation of the breakpoint sets: LF at sorted positions,
     C[c] + q_c + (p − a), by a merge-scan of the L-runs. Its outputs are already sorted
     per character, so a concatenation and a merge replace a general sort, in
     O(r + |C_k| + σ) per round.
   - The descriptors (displacement and word) left as the open construction problem.
   - The traversal-clustering implication of the preliminary Ψ⁸ size: at least about
     84–89% of gaps shorter than 8.
3. **Round 5.**
   - **The interval-image tiling theorem:** for any permutation represented as
     translations on disjoint intervals, the images partition the rows.
   - **Doubling as a two-pointer intersection** of the image partition and the source
     partition. The displacements and words are composed in the join.
   - **The refinement bound** R_{2a} ≤ 2R_a − 1.
   - **Ordering** by a 10-bit two-pass radix sort, or by a bitmap and rank directory.
   - **The researcher's own check:** 5,750 random cases of a reference implementation.
   - **The construction bound:** O(r + m_1 + m_2 + m_4 + m_8). The open problem left is
     the constant factors.

The coordinator's checks and answers are in `2026-10-30-research-portfolio.md` §3.1:
- the census on the bench's 233 full blocks;
- the descriptor construction by image splitting, checked by brute force.

These are source material. Claims are checked there, not taken as findings.

The text reached us as rendered text and is kept as received.

## 1. Round 3, as received

````text
## Research burn-down, round 3: the composition theorem needs one more correction

I independently checked the partition mathematics by enumerating small cyclic BWTs. This exposed an important distinction in Claude's latest result.

The union

\\[ U_k=\bigcup\_{t=0}^{k-1}\Psi^{-t}(B) \\]

is a valid conservative refinement when \\(B\\) contains the one-step navigation and output-symbol boundaries. But it is not generally the minimal partition needed to produce a \\(k\\)-byte word and its successor.

The difference is that two adjacent starting rows can temporarily diverge under Ψ and subsequently reconverge into adjacent destinations. A conservative partition remembers the earlier divergence; the minimal partition can discard that boundary when the emitted words also agree.

Consequently, my previous gap formula remains exact for the conservative union:

\\[ |U_k|=\sum_i\min(k,g_i), \\]

but not necessarily for the minimal output-producing partition.

This is useful rather than merely inconvenient. We now have two independent counts:

- \\(M_k\\): the minimum number of contiguous records needed for a constant output word and translated \\(k\\)-step successor.
- \\(U_k\\): the number of records from the conservative preimage-union construction.

The excess \\(U_k-M_k\\) measures the cost of insisting that all intermediate transitions remain regular.

That is an actual design choice: a minimal representation may be smaller, but its construction could require more work.

## An unexpected finding from exhaustive small-block tests

I tested the exact output-producing partition against the conservative preimage union on every primitive cyclic string over a three-symbol alphabet, up to length eight, with shortcut lengths \\(k=1\ldots6\\).

INDEPENDENT EXHAUSTIVE CHECK

## 58,212

String / shortcut-length cases

## 0

Cases where partitions differed

## 8

Maximum string length

For these nonperiodic cyclic strings, the conservative union always equalled the minimal output-producing partition in my test.

That suggests my earlier warning about conservative overpartitioning may be too pessimistic for ordinary, nonperiodic BWT blocks.

I also found a genuine counterexample for periodic strings. For `aaaa`, cyclic rotations tie, and with one conventional ordering of those rotations the partitions differ at \\(k=2\\).

Thus we have a sharper question about assumptions, not another open-ended hypothesis:

Does the equality hold for primitive cyclic strings because of the order-preserving structure of the BWT, and fail only when cyclic rotations are not unique?

I have not proved that general statement. Exhaustive testing up to length eight is evidence, not a theorem.

But it is an excellent lead: if a proof follows from the stable-order properties of Ψ, we can recover an exact minimal-record formula for the normal case, instead of treating it as merely an upper bound.

One important caution: ordinary bzip2 blocks can contain periodic content, so any production algorithm must still be correct for those cases. We cannot make an implementation assumption just because Wikidata blocks are unlikely to be perfectly periodic.

## We can now prove the equality under a precise BWT condition

The experiments suggest a simple proof. The key is that Ψ expands, rather than contracts, distances between rows in the same F-symbol bucket.

For a conventional stable BWT mapping, suppose \\(x\<y\\) and \\(F[x]=F[y]=c\\). The positions \\(\Psi(x)\\) and \\(\Psi(y)\\) are the corresponding occurrences of \\(c\\) in the last column.

They occur in the same order, and there are at least \\(y-x\\) occurrences between their ordinal positions. Hence:

\\[ \boxed{\Psi(y)-\Psi(x)\ge y-x.} \\]

Now take two initially adjacent rows, \\(j\\) and \\(j+1\\).

Suppose their first \\(k\\) emitted symbols are identical. At every step up to \\(k\\), both trajectories occupy the same F-symbol bucket.

Therefore their row separation cannot decrease:

\\[ 1=\Delta_0\le\Delta_1\le\cdots\le\Delta_k, \\]

where

\\[ \Delta_t=\Psi^t(j+1)-\Psi^t(j). \\]

If they ever stop being adjacent, they cannot become adjacent again while their emitted symbols remain identical.

If their emitted symbols ever differ, they must remain in different output-word classes.

This establishes the missing implication:

> For a stable, order-preserving Ψ, a boundary introduced by any intermediate step cannot disappear from the final output-producing partition.

Consequently, under that condition,

\\[ \boxed{ M_k = 1+\left|\bigcup\_{t=0}^{k-1}\Psi^{-t}(B)\right| } \\]

where \\(M_k\\) counts the minimal output-producing intervals, and \\(B\\) includes one-step translation and F-symbol boundaries.

And the earlier gap identity becomes an exact formula for the minimal record count:

\\[ \boxed{ M_k=1+\sum_i\min(k,g_i). } \\]

This is stronger than what I could justify at the start of this session.

It also identifies precisely why periodic cyclic-sort counterexamples deserve special treatment: an arbitrary tie order among identical rotations need not preserve the stable correspondence between F and L.

The next verification is implementation-specific: does mibz2's actual construction maintain this stable Ψ property for every supported bzip2 block, including periodic rotations? If not, the conservative partition remains correct, while minimality is not guaranteed.

We can distinguish those cases with an invariant check rather than guess.

## Eighth result: an exact decision threshold for eight-step records

We can now express Ψ⁸ feasibility directly in terms of the gap histogram.

For the representative 900,000-row block, with approximately 56,000 one-step boundaries, the eight-step representation needs about 281,000 records or fewer to fit the 3.6 MB T-vector budget, including the proposed directory.

Since

\\[ |U_8|=\sum_i\min(8,g_i) =n-\sum_i\max(g_i-8,0), \\]

the storage condition is approximately

\\[ \boxed{ \sum_i\max(g_i-8,0)\ge619{,}000. } \\]

In plain English: after accounting for the first eight positions in every breakpoint gap, at least about 619,000 row positions must lie beyond those first eight positions.

This is a surprisingly stringent requirement.

It's not sufficient for the breakpoints merely to be unevenly distributed. They must form a traversal pattern with substantial long gaps that can support the larger shortcuts.

We can now distinguish the \\(k=4\\) and \\(k=8\\) candidates more decisively:

| Candidate        | Mathematical disposition                               | Principal remaining obstacle                               |
| ---------------- | ------------------------------------------------------ | ---------------------------------------------------------- |
| Ψ²               | Record budget compatible with the representative block | Existing pair-walk execution costs                         |
| Ψ⁴               | Record budget compatible even near the upper bound     | Construction and dependent lookup latency                  |
| Ψ⁸               | Requires sufficiently long traversal gaps              | Representation budget, then construction and execution     |
| Variable \\(k\\) | Potentially more flexible                              | Selecting and addressing mixed-length records economically |

These are storage dispositions, not measured performance results.

## Ninth result: construction is now the highest-risk assumption in N1

The portfolio proposes computing breakpoint refinements from L's runs, then composing descriptors by doubling, without materializing the T-vector.

That is attractive. But the low construction cost is not established by the record-count bound.

In particular, a construction can use only \\(O(|U_k|)\\) records and still be slow if each record requires a dependent interval search.

We need to separate three quantities already conflated in some models:

\\[ \text{Construction cost} = \text{discover boundaries} + \text{resolve destinations} + \text{materialize descriptors}. \\]

The first may be cheap if the breakpoint sets can be propagated from runs.

The second could be the killer: composing two interval descriptors requires finding which other interval contains the first descriptor's destination.

The third is mostly sequential memory traffic.

A small representation does not imply a cheap builder.

This also explains why the existing Ψ² results remain important. They demonstrate that reducing the number of navigation steps can fail even when the resulting traversal has some advantage.

The most valuable next theoretical step for N1 is not another compression scheme. It's a construction algorithm with a credible bound on the number and locality of destination-resolution operations.

Until then, the projected decoder speedup is not justified by the interval-count result alone.

## Revised register dispositions from this round

Research assessment only; no GitHub changes made.

| Existing item                | New disposition                                                                               |
| ---------------------------- | --------------------------------------------------------------------------------------------- |
| N1/T1: preimage overlap      | Exact gap-histogram identity; under stable Ψ, exact minimal output-producing count            |
| N1: mathematical correctness | Proof conditional on stable, bucket-monotone Ψ; periodic tie handling must be checked         |
| N1: Ψ⁴ space                 | Representative 56k-boundary regime passes a worst-case fixed-record storage budget            |
| N1: Ψ⁸ space                 | Exact gap-tail condition derived                                                              |
| N1: construction             | Elevated to the principal unresolved engineering risk                                         |
| #63–#66                      | Row-space clustering established in aggregate; traversal-space statistics now sharply defined |
| K2 / #25                     | Citation resolved; inversion relevance remains separate                                       |
| N3: run-native pipeline      | Remains conditional on an economical N1 builder                                               |

### The next questions to eliminate

I would focus our next round on the construction, specifically whether the destinations of composed intervals can be located with a streaming procedure, rather than one random interval lookup per record.

If we can construct Ψ⁴ descriptors from the existing run stream with near-sequential access and roughly \\(O(r)\\) or \\(O(|U_4|)\\) inexpensive operations, N1 becomes much more credible.

If every descriptor requires multiple dependent searches, then its compactness may be irrelevant.

That is the decisive fork.

And the work so far has materially sharpened the research: we've moved from an incorrect identity, to a counterexample, to a conservative construction, to a gap formula, to a conditional proof of exactness and a concrete storage threshold.

We are no longer debating whether multi-step intervals exist. We're approaching the question of whether they can actually be built and traversed cheaply enough to beat mibz2 0.2.1.
````

## 2. Round 4, as received

````text
## Round 4: we can eliminate the first construction problem

The new Ψ⁸ measurements are encouraging. If the full 233-block census confirms 1.2–1.4 MB per block, then representation size is no longer the obvious objection to N1.

I have now derived something more important for the CPU decoder:

We can generate the composed breakpoint sets directly from the run-length encoding of the BWT last column, without materializing the 900 KB last column or the 3.6 MB T-vector.

This is an exact construction, not a performance result.

### 1. A stronger identity for the one-step partition

Let \\(L\\) be the BWT last column and let \\(r\\) be its number of maximal equal-symbol runs.

For each symbol \\(c\\), the rows belonging to its F-column bucket correspond, in order, to occurrences of \\(c\\) in \\(L\\).

Within a single run of \\(c\\) in \\(L\\), their Ψ successors are consecutive, so Ψ is a translation.

Whenever consecutive occurrences of \\(c\\) lie in different L-runs, Ψ ceases to be a translation across their boundary.

Including boundaries between F-symbol buckets gives:

\\[ \boxed{|B|=r-1} \\]

for the non-wrapping row-boundary convention, assuming the usual stable BWT mapping.

This means the safe one-step intervals correspond exactly to the individual runs of \\(L\\), rearranged into F-bucket order.

We don't need to discover their boundaries through pointer chasing. They're already encoded in the run stream.

### 2. The inverse transition is particularly convenient

Define the conventional LF mapping:

\\[ \operatorname{LF}(p) = C[L[p]]+\operatorname{rank}\_{L[p]}(L,p), \\]

using zero-based occurrence rank.

This is the inverse of our Ψ mapping.

To build the next conservative breakpoint set, we need:

\\[ C\_{k+1}=C_k\cup\operatorname{LF}(C_k). \\]

The challenge appears to be evaluating LF at many irregular positions.

But suppose the positions in \\(C_k\\) are sorted.

We can merge them with a scan of the L-runs.

For a run of symbol \\(c\\), starting at position \\(a\\), let \\(q_c\\) be the number of previous occurrences of \\(c\\). Every requested position \\(p\\) inside that run satisfies:

\\[ \boxed{\operatorname{LF}(p)=C[c]+q_c+(p-a).} \\]

That is just arithmetic.

No random access to \\(L\\). No T-vector lookup. No rank query.

We only need the run boundaries and 256 counters.

The produced LF positions will generally not be sorted globally, so we need to reorder or merge them before the next iteration. But that is a separate, tractable problem.

This establishes a concrete, stream-based route for propagating the exact breakpoint sets from the compressed run representation.

### 3. We can eliminate the general sort too

The LF results are ordered within each character.

As we scan \\(L\\) from left to right, the occurrence count \\(q_c\\) increases for each character \\(c\\). Therefore the LF positions generated for that character are strictly increasing.

And the F buckets themselves are ordered by character.

So we can generate the next breakpoint frontier by:

1. Scanning the current sorted breakpoint positions alongside the L-runs.
2. Appending each computed LF destination to its character's output sequence.
3. Concatenating the 256 character sequences in F-bucket order.
4. Merging that sorted result with the existing breakpoint set, removing duplicates.

There is no comparison sort and no full-length permutation construction.

For each propagation round, the work is bounded by

\\[ \boxed{O(r+|C_k|+\sigma)} \\]

with \\(\sigma\le256\\), assuming the L-runs and current sorted frontier are available.

For a fixed shortcut length \\(k\\), discovering the boundaries therefore takes

\\[ \boxed{O\left(kr+\sum\_{t\<k}|C_t|+k\sigma\right)} \\]

time and linear auxiliary storage in the number of represented boundaries.

This is an algorithmic bound on arithmetic and sequential processing. It doesn't establish a low cycles-per-byte cost, and it excludes constructing the final jump descriptors.

But it settles a major part of N1's construction question:

The breakpoint sets can be generated in linear work per round using the run stream, with predominantly sequential accesses.

That's materially stronger than the earlier proposal to perform arbitrary LF lookups at every boundary.

### 4. The next outstanding problem is smaller

After obtaining the final intervals, we still need, for each interval \\(I\\):

\\[ \left( \Psi^k(\min I)-\min I,\quad F(j),F(\Psi(j)),\ldots,F(\Psi^{k-1}(j)) \right). \\]

The output word is constant within the interval under the established stable-mapping conditions.

But computing the jump displacement and its bytes remains a separate construction problem.

Our breakthrough candidate now has a cleaner decomposition:

| Construction component                   | Current mathematical status               |
| ---------------------------------------- | ----------------------------------------- |
| Find one-step boundaries                 | Derived directly from L-runs              |
| Propagate boundaries for \\(k\\) steps   | Linear merge/scatter by character         |
| Remove duplicate propagated boundaries   | Sorted merge                              |
| Emit \\(k\\)-step destinations and words | Still needs an efficient algorithm        |
| Build rank-indexed lookup metadata       | Streaming bitvector/popcount construction |
| Decode efficiently                       | Requires an actual CPU cost comparison    |

This removes the need to investigate expensive general-purpose sorting for breakpoint propagation.

It does not remove the need for destination resolution during descriptor construction.

## Tenth result: the reported Ψ⁸ size implies strong clustering in traversal order

Claude reports that the first three full blocks suggest approximately 1.2–1.4 MB of eight-step records. That's preliminary; the 233-block census is still running.

Let's see what that would imply if the figure refers to the 12-byte descriptor payload, and the block has approximately 56,000 one-step boundaries.

The implied number of records is approximately:

\\[ 100{,}000\text{ to }117{,}000. \\]

Thus:

\\[ \frac{|C_8|}{|B|} \approx1.8\text{ to }2.1. \\]

Under the no-overlap upper bound, that ratio could approach eight.

Instead, it appears much closer to two.

Recall:

\\[ |C_8|=\sum_i\min(8,g_i). \\]

Every gap contributes at least one. Every gap of length eight or greater contributes eight.

Therefore:

\\[ \\#\\{i:g_i\ge8\\} \le \frac{|C_8|-|B|}{7}. \\]

Under the preliminary figures, at least approximately 84–89% of the gaps between successive breakpoint encounters must be shorter than eight traversal steps.

That's a very different phenomenon from the earlier observation that translation boundaries cluster in adjacent BWT rows.

It suggests the boundaries also cluster along the actual decoding trajectory.

And since the mean traversal gap is about

\\[ 900{,}000/56{,}000\approx16, \\]

the gap distribution must be strongly skewed: numerous very short gaps coexist with much longer uninterrupted stretches.

Illustration: same mean gap, radically different distributions

If the 1.2–1.4 MB figure includes the rank directory, the record count is smaller still, and the clustering implication is stronger. The calculation above deliberately uses the more conservative interpretation.

### The interesting consequence

This structure suggests a new way to evaluate the already-recorded variable-length shortcut option.

Instead of simply asking whether every starting row supports a long shortcut, we should ask how much of the actual decoding traversal lies inside the long gaps between breakpoint encounters.

Those long regions may be precisely where the multi-step representation can avoid the most dependent operations.

However, a long gap between breakpoint encounters doesn't itself guarantee a cache-efficient descriptor lookup, and a variable-length scheme must still know where to stop.

So this is a strong structural finding, not yet a decoder algorithm.

## What we've actually burned down

Breakpoint discovery no longer requires general sorting or construction of Ψ.

Derived an exact LF-from-runs method, with per-character ordered outputs and linear merging.

The composition census has an exact structural interpretation.

The gap histogram determines every conservative partition size. Claude has independently cross-checked the identity.

The apparent Ψ⁸ size reveals traversal-space clustering.

Conditional on the preliminary block measurements, the breakpoint-gap distribution is highly skewed, rather than merely row-space clustered.

The hard remaining construction question is descriptor generation.

We still need a cheap way to produce the multi-step destinations and output words, preferably using ordered joins over the same run representation.

I would send Claude this specific result:

> The breakpoint-propagation stage of N1 admits a run-native construction. Given sorted breakpoint positions and the L-run stream, merge-scan to evaluate LF at those positions. Within each L-run, LF is \\(C[c]+q_c+(p-a)\\). Collect results into 256 per-character sequences; each sequence is already sorted, and concatenating in F-bucket order gives a globally sorted frontier. Merge with the old frontier to obtain the next safe partition. This avoids general sorting and a materialized T-vector. Please distinguish this solved boundary-construction problem from the still-open task of computing the \\(k\\)-step displacements and output words for every record. Also check the reported Ψ⁸ sizes against the traversal-gap distribution: if 1.2–1.4 MB is descriptor payload for approximately 56k one-step boundaries, at least approximately 84–89% of breakpoint gaps must be shorter than eight traversal steps.

The next research step is now clear: can we exploit the same occurrence-order preservation to construct the complete Ψ⁸ descriptors using sequential joins rather than random pointer chasing?

That is where I would concentrate next. It targets the remaining expensive operation, builds directly on the mathematics we've established, and could determine whether N1 becomes a real CPU decoder breakthrough rather than merely a compact representation.
````

## 3. Round 5, as received

````text
# Round 5: the descriptor-construction problem collapses to an interval join

Claude is checking exactly the right invariant. I can establish it more generally:

If a partition represents a permutation as translations on disjoint intervals, the images of those intervals necessarily partition the entire row universe.

This is true for any permutation—not just a BWT permutation.

That gives us a way to construct the complete multi-step descriptors without performing a destination search for every resulting record.

## 1. The interval-image tiling theorem

Suppose a level-\\(a\\) table consists of \\(m\\) records:

\\[ I_i=[l_i,h_i),\qquad \Psi^a(j)=j+d_i\quad(j\in I_i). \\]

Each record also stores the \\(a\\)-byte output word \\(w_i\\).

The source intervals partition \\([0,n)\\).

Because \\(\Psi^a\\) is bijective, the image intervals

\\[ J_i=[l_i+d_i,h_i+d_i) \\]

are pairwise disjoint and together cover \\([0,n)\\).

Therefore, sorting the image intervals by their starting positions produces a second exact partition of the rows.

No gaps, no overlapping image intervals, and no duplicate destination ownership.

I also checked this property independently on random permutations partitioned into maximal translation intervals. It held in the tested cases, as expected from bijectivity.

The implication is stronger than Claude's proposed one-search-per-parent construction.

## 2. Doubling becomes an intersection of two partitions

To build \\(\Psi^{2a}\\), consider two interval partitions:

- The level-\\(a\\) source partition, already sorted by row.
- The level-\\(a\\) image partition, sorted by destination row, with each image retaining a pointer to its source record.

Intersect these two partitions using a standard two-pointer merge.

For an image interval \\(J_i\\) and source interval \\(I_j\\), let

\\[ K=J_i\cap I_j=[u,v). \\]

If this intersection is nonempty, its originating source interval is

\\[ K-d_i=[u-d_i,v-d_i). \\]

On that interval:

\\[ \boxed{\Psi^{2a}(x)=x+d_i+d_j.} \\]

And the complete output word is simply:

\\[ \boxed{w\_{2a}=w_i\Vert w_j.} \\]

That is, concatenate the bytes emitted during the first \\(a\\) steps with those emitted during the next \\(a\\) steps.

Every output record now has its exact source interval, jump displacement and output word.

No destination interval search is required during the join.

The only ordering problem is sorting the image intervals before the merge.

## 3. A linear bound on the number of candidate records

Here's the useful complexity result.

Two partitions of \\(n\\) rows, each containing \\(m\\) intervals, have at most

\\[ \boxed{2m-1} \\]

nonempty intersections.

Why? Their common refinement has boundaries drawn from the union of their two sets of \\(m-1\\) internal boundaries.

There are at most \\(2m-2\\) distinct boundaries, hence at most \\(2m-1\\) intervals.

Therefore:

\\[ \boxed{R\_{2a}\le 2R_a-1.} \\]

This is true for the intersection construction before coalescing adjacent identical descriptors.

It implies that doubling the shortcut length cannot produce a combinatorial explosion beyond twice the previous interval count.

More importantly, once the image intervals have been ordered, the entire join requires \\(O(R_a)\\) operations.

This includes generating the successor displacements and output words—the two components that were still unresolved in the previous round.

### I tested the full construction

I built a small independent reference implementation that:

1. Generates exact level-\\(a\\) interval records for a random permutation and symbol assignment.
2. Sorts their destination intervals.
3. Intersects them with the level-\\(a\\) source partition.
4. Produces the corresponding level-\\(2a\\) displacements and output words.
5. Sorts and coalesces adjacent identical descriptors.
6. Compares the result with direct repeated permutation evaluation.

It passed 5,750 random test cases, including various permutation sizes and shortcut lengths.

This is stronger than testing only BWT examples: the algorithm depends on interval translations and bijectivity, not on the BWT's more specialized properties.

The tests are not a production correctness proof, but the interval argument above supplies that proof under the stated assumptions.

## 4. We can avoid comparison sorting as well

The remaining operation is ordering the image intervals and the final output records.

Their keys are row positions in \\([0,n)\\), where \\(n\le900{,}000\\).

We have two plausible linear-time choices.

Option A: small radix sorts. The interval starts require only 20 bits. A two-pass radix sort using 10-bit digits can order them in \\(O(m)\\) work, with small histograms and sequential passes.

Option B: bitmap ordering. Since image-interval starts are unique, mark them in an \\(n\\)-bit vector. A rank directory then assigns each marked start its sorted position. Scatter the corresponding descriptor to that position.

For a 900,000-row block, the raw bitmap is only about 113 KB. It can be reused between construction phases.

Neither option requires building the 3.6 MB T-vector.

Nor does either require a binary search for each output record.

There is still a serious performance distinction: these are linear-time algorithms, not necessarily cheap algorithms. Sorting or scattering hundreds of thousands of descriptors could cost more than today's simple T-vector construction.

But the asymptotic obstacle is gone.

## 5. The resulting construction

For \\(k=2,4,8\\), the scheme is:

Repeat the middle three operations for level 1→2→4→8. Every join constructs both the multi-byte output word and the composed successor.

If \\(m_a\\) is the number of level-\\(a\\) records, the construction through level eight requires work proportional to

\\[ \boxed{O(r+m_1+m_2+m_4+m_8)} \\]

assuming linear-time ordering and a direct run-native level-one construction.

The proportionality constants include repeated descriptor reads, writes and sorting passes. Those constants are critical to whether the method beats mibz2.

### What this burns down

This directly advances the existing questions rather than opening more:

| Existing research question                                       | What we've established                                                                     |
| ---------------------------------------------------------------- | ------------------------------------------------------------------------------------------ |
| N1: can descriptors be built without random successor lookups?   | Yes. The interval-image merge constructs complete descriptors.                             |
| #53 / P84: can a compact structure be constructed directly?      | A concrete run-native, multilevel construction exists; its practical cost remains unknown. |
| #44: can several successor operations be replaced by arithmetic? | Yes, on an interval record, a \\(k\\)-step successor is one displacement addition.         |
| N3: can we avoid materializing L and the T-vector?               | Mathematically feasible for this construction; integration costs remain unresolved.        |
| N1: can record count grow uncontrollably during doubling?        | No. Each doubling produces at most \\(2m-1\\) candidate records before coalescing.         |

I would not close the full engineering questions #53 or N3 yet: they ask for economical implementation, not merely existence.

## The next obstacle is finally the right one

We've eliminated three suspected fundamental obstacles to multi-step Ψ records:

- Their correctness can be expressed through exact interval invariants.
- Their size can be measured and appears promising in Claude's preliminary full-block census.
- Their complete descriptors can be built by a linear-work interval join.

That leaves a hard but sharply defined performance problem:

Can the builder's sorting, merging and descriptor writes be made cheap enough that the reduced dependent navigation pays for them within a single 900 KB block?

This is where the work becomes architecture-sensitive. The current T-vector build is fast, and earlier attempts lost precisely because their extra construction cost overwhelmed their navigation savings.

The correct next cost model should therefore count actual bytes moved and records processed at each level, not just asymptotic operations or the final table size.

For each block, Claude's census can provide \\(m_1,m_2,m_4,m_8\\). Those numbers are enough to estimate the read/write volume of a specific radix-sort-and-merge construction before any decoder implementation.

I'd send Claude the tiling theorem, the \\(2m-1\\) refinement bound, and the two-way interval join. This is a direct extension of the one-search-per-parent approach Claude has already verified—and it removes even that search requirement.

No decoder code, no i9 experiment, and no claim of a measured speedup are needed to establish it.

This is meaningful progress: the proposed multi-step representation now has a complete mathematical construction, and the remaining uncertainty is its cost on the hardware we actually care about.
````
