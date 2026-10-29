<!--
Provenance: the research chatbot's second multi-cycle programme (ten cycles to a fixed point,
"as before"), started by the user after #81's cooperative result, relayed into the working
session on 29 October 2026. The text below is unedited, except that two interactive widgets
it contained (a modelled-storage chart in cycle 6 and an Amdahl slider in cycle 8) reached
the session as their text alone, and are kept as that text.
-->

# Research cycle 1 — What #81 tells us about compressed construction

The first question is whether the 31.3% build improvement changes how we should encode Ψ.

It does, but not quite in the obvious way.

## A. Construction is not just a prelude to navigation

In the existing full-table representation, a scan of the BWT produces absolute navigation entries.

For a link in the convention we've been studying:

\\[ \Psi(C[c]+k)=\operatorname{select}\_c(L,k+1) \\]

where \\(k\\) is zero-based.

This means we can construct Ψ by scanning the BWT left to right, assigning every occurrence to its position in the appropriate F interval.

That scan has a useful property:

Every new occurrence of a particular character is larger than the previous occurrence of that character.

So a compressed builder can assemble monotone Ψ groups without sorting them.

A conceptual streaming builder is:

```
count the bytes in L
compute each character's F interval

for each position i in L:
    c = L[i]
    k = next occurrence ordinal for c
    j = C[c] + k

    append i to the compressed group containing Ψ[j]
```

Once a group of 32 positions is complete, its encoder can decide whether to store:

- A translation descriptor.
- A frame-of-reference payload.
- An Elias–Fano payload.
- An exceptional raw payload.

There is no need to build the entire flat Ψ permutation first.

But the word append hides a difficulty: the scan interleaves up to 256 symbol streams. We need per-symbol accumulation buffers, and parallel workers introduce boundary ownership problems.

A sequential version needs at most \\(256\times32\\) pending positions, approximately 32 KiB of `u32` position storage.

That's small enough to make the streaming concept credible, though not necessarily fast.

## B. Why parallel construction is harder

Suppose two workers scan separate portions of L.

Both may contribute occurrences to the same 32-entry Ψ group.

If they try to write a compressed record concurrently, they need some way to coordinate the record's construction.

The promising general solution is to compute per-worker, per-symbol occurrence ranges.

Each worker can count its symbols, then calculate where its occurrences belong in the final F intervals.

The resulting destination ranges are disjoint, even though adjacent workers may contribute to the same compressed group.

This suggests a strategy:

1. Establish per-worker symbol counts and prefix offsets.
2. Give each worker ownership of complete compressed groups where possible.
3. Handle groups crossing worker boundaries separately.
4. Emit the final compressed records without constructing a full `u32` Ψ array.

There is a crucial qualification: making group ownership nonoverlapping without scanning L again or buffering extra data is not automatic. We must count that cost explicitly.

The encouraging observation is that the number of group boundaries is much smaller than the number of Ψ values.

At 900,000 rows:

\\[ \left\lceil\frac{900000}{32}\right\rceil=28,125 \\]

So construction coordination could potentially operate at the level of tens of thousands of groups rather than hundreds of thousands of individual pointers.

## C. The important lesson from #81

The reported #81 result is:

# 1.313×

3×3 construction speedup

# 1.102×

Combined build + walk speedup

These results support the importance of reducing construction work, but they don't yet establish whether the gain came mostly from fewer instructions, less traffic, better locality or synchronization effects.

Therefore, I would not predict a particular benefit from compressed construction.

I would instead adopt this principle:

The compressed structure must be constructed in its final form, and its construction algorithm must be evaluated as a first-class part of the representation.

The critical research distinction is between compressing an existing Ψ array and constructing compressed Ψ directly from L.

The latter is our real target.

# Research cycle 2 — Reconsidering the exceptional records

The earlier representation discussion devoted considerable attention to Elias–Fano.

But now that we know only approximately 0.49% of rows reportedly require exceptions, I think we've overcomplicated the exceptional case.

We can simplify it.

## A. A raw fallback might be perfectly adequate

Each Ψ value needs only 20 bits for a maximum-size bzip2 block.

Therefore 32 arbitrary positions can be encoded in:

\\[ 32\times20=640\text{ bits}=80\text{ bytes} \\]

No monotonicity assumption is needed.

For an exceptional group, we could simply store 32 packed 20-bit positions.

A corresponding group-level descriptor would retain the segment markers and format information.

This would recover arbitrary Ψ entries with fixed positional extraction and no rank/select operations.

The extra 16 bytes above a 64-byte record might occasionally touch another cache line, but the reported exception population is tiny.

Under a row-weighted 0.49% exception rate, the number of affected rows in a full 900,000-row block would be about 4,410, corresponding to approximately 138 complete 32-row groups.

Even if those groups were stored in larger, padded 128-byte records, the total additional storage would be modest.

The key is that we don't need an elegant succinct representation for a rare exceptional case.

We need one that's correct, simple and cheap enough.

This is a general compression-engineering principle: preserve the simplicity of the common case, and tolerate some redundancy in the tail.

## B. A more useful three-format architecture

I'd now simplify the two-tier candidate to three fundamental modes:

| Mode             | Description                 | Navigation               |
| ---------------- | --------------------------- | ------------------------ |
| Translation      | 32 consecutive destinations | Base + slot              |
| Compact monotone | FOR or EF payload           | Load and decode          |
| Raw exception    | Arbitrary 20-bit values     | Direct packed extraction |

That is a cleaner architecture than continuously adding specialized exception types.

The next question is whether FOR and EF need to remain distinct, or whether one could encode all ordinary monotone groups efficiently enough.

Claude's census suggests FOR-14 covers 98.11% of rows already, while EF-12 adds only 1.40%.

That raises an attractive possibility: perhaps one FOR format plus rare raw exceptions is faster than a three-way translation/FOR/EF decoder, even if it uses somewhat more space.

This is a cost-model question, not an assumption.

We should compare:

- Translation + FOR + EF + raw.
- Translation + FOR + raw.
- Translation + raw.

The last is likely too large, but it provides a simple bound on how much the cleverer encodings are actually buying.

# Research cycle 3 — A useful theorem connecting translation coverage to BWT runs

This is the strongest mathematical deduction from the new work.

The 77.6% translation result isn't an isolated empirical curiosity. It can be related directly to the number of runs in the BWT.

Let \\(r\\) be the number of maximal runs of identical characters in L.

For example:

`aaabbccccaa`

has four runs.

Now consider Ψ as the concatenation of the sorted occurrence-position lists for each character.

Within one character's list, successive Ψ values are consecutive exactly when those occurrences were adjacent in L.

Every discontinuity within a character's list corresponds to a gap between two runs of that character.

If character \\(c\\) appears in \\(r_c\\) runs, its list has \\(r_c-1\\) such discontinuities.

Summing over all present characters gives:

\\[ \sum_c(r_c-1)=r-\sigma' \\]

where \\(\sigma'\\) is the number of distinct symbols present.

At the boundaries between character intervals in F, there are at most another \\(\sigma'-1\\) discontinuities.

Therefore the total number of places where the translation rule can fail is bounded by:

\\[ \boxed{d\le r-1} \\]

where \\(d\\) counts positions \\(j\\) for which:

\\[ \Psi(j+1)\ne\Psi(j)+1. \\]

This is a particularly useful result.

## What does it tell us about 32-row groups?

A group of 32 entries is translation-eligible if it contains no such discontinuity between its entries.

Each discontinuity can spoil at most one group of 32, assuming globally aligned, nonoverlapping groups.

Therefore:

\\[ \boxed{ \\#\text{nontranslation groups}\le r-1 } \\]

and the number of translation-eligible full groups is at least:

\\[ \max\left(0,\left\lfloor\frac n{32}\right\rfloor-(r-1)\right). \\]

This is a conservative guarantee; it can be much weaker than the actual coverage because several discontinuities may fall into one group, while others fall exactly between groups and spoil none.

The significance is that translation coverage is related to BWT run structure, one of the central quantities in modern repetitive-text indexing.

And we can measure \\(r\\) from L using a single sequential scan.

### Why this matters

It gives us a principled way to predict which data benefits from translation records.

Small \\(r\\): large runs, comparatively few translation discontinuities, high potential coverage.

Large \\(r\\): many short runs, more nontranslation groups, less benefit from translation descriptors.

This is not a complete performance predictor, but it gives us a mathematical bridge between the representation and properties of the input.

It also connects our work to run-length BWT indexing without requiring the expensive interval-navigation mechanisms that lost in earlier experiments.

Research conclusion: translation-record suitability can be studied through the distribution of Ψ discontinuities and BWT runs, not just by blindly measuring compression ratios.

# Research cycle 4 — The real variable is discontinuity density

The new theorem suggests that we should stop thinking of the corpus as simply “77.6% translation-friendly”.

Instead, characterize the discontinuity process.

Define:

\\[ D[j]= \begin{cases} 1 & \Psi(j+1)\ne\Psi(j)+1\\\ 0 & \text{otherwise} \end{cases} \\]

This is a bitvector of translation breaks.

Then a 32-entry group beginning at \\(a\\) is translation-eligible precisely when:

\\[ \operatorname{rank}\_1(D,a+31)- \operatorname{rank}\_1(D,a)=0. \\]

Here the first 31 adjacent transitions within the group are checked.

This is a surprisingly direct reappearance of succinct bitvectors.

We began by asking whether rank/select could replace Ψ navigation. It was a poor match for the hot loop.

But rank over a discontinuity bitmap could be useful during representation construction and analysis, where it determines the most economical encoding of groups.

That's a very different and more natural application.

## A new way to visualize the representation

Ψ as long translations separated by discontinuities

Conceptual illustration: consecutive-value stretches require only an initial destination and an offset. Breaks require more information.

The important statistical quantities are now:

- Total discontinuities \\(d\\).
- Number of 32-row groups touched by discontinuities.
- Distribution of distances between discontinuities.
- Per-block translation coverage.
- Whether discontinuities cluster or are spread relatively evenly.

That last distinction is extremely important.

Two Ψ permutations with the same number of discontinuities can have very different 32-group translation coverage.

If breaks cluster into a few groups, most groups remain simple.

If breaks are evenly distributed, many more groups need payloads.

So \\(r\\) is a useful structural predictor, but the spatial arrangement of discontinuities relative to group boundaries determines the fixed-record encoding cost.

# Research cycle 5 — A new representation model using discontinuity entropy

We can take the previous idea one step further.

Suppose we know the positions of all discontinuities in Ψ.

Then Ψ is piecewise translation:

\\[ \Psi(j)=b_k+(j-a_k) \\]

for each maximal translation interval beginning at \\(a_k\\), with destination base \\(b_k\\).

In principle, we could represent Ψ using:

- Translation-interval starts.
- Translation base values.
- A data structure for identifying which interval contains \\(j\\).

This is related to compressed permutations with strict ascending runs and run-length BWT move structures.

The difficulty is the predecessor query needed to identify the interval.

That predecessor operation is precisely what can create extra dependent memory accesses.

Our fixed-group design avoids it by imposing a regular grid.

So we have two representation philosophies:

| Design                   | How it identifies a translation interval |
| ------------------------ | ---------------------------------------- |
| Maximal translation runs | Search for preceding breakpoint          |
| Fixed 32-row groups      | Compute `j >> 5`                         |
| Two-tier descriptors     | Compute `j >> 5`, then inspect mode      |
| Direct T-vector          | Index `j` directly                       |

The fixed 32-row grid sacrifices some compression to eliminate predecessor search.

That is the core reason it may be faster than a theoretically more compact run-based representation.

A useful new objective is therefore:

\\[ \text{Minimize representation size} \\]

subject to:

\\[ \text{translation region identified by arithmetic, not search}. \\]

This is more specific than simply targeting entropy-compressed permutations.

It defines a class of directly addressed, locally compressed functions.

# Research cycle 6 — Is the 64-bit descriptor the right basic unit?

The new structural understanding lets us evaluate the two-tier proposal more seriously.

For each group of 32 Ψ values, the proposed descriptor stores:

\\[ 32\text{ marker bits}+20\text{ address bits}+8\text{ symbol bits}+4\text{ mode bits}=64. \\]

For a translation group, this is sufficient to determine the next row, the marker and the output symbol, provided the group is uniform in F.

The resulting base cost is:

\\[ \frac{64}{32}=2\text{ bits per row}. \\]

This is a remarkably economical directly addressed structure.

But almost half the descriptor is occupied by segment markers.

And that raises a question: should the marker representation be integrated with the navigation encoding at all?

## A. Segment markers have very low information content

Your algorithm chooses a relatively small number of segment starts.

If there are 512 marked positions among 900,000 rows, specifying their locations requires only approximately:

\\[ \log_2{900000\choose512}\approx6.27\text{ Kbits}. \\]

That's under 1 KiB.

By comparison, a 32-bit bitmap for every 32-row Ψ group consumes:

\\[ 28125\times32=900000\text{ bits} \\]

or 112.5 KB.

So the descriptor's marker bitmap is highly redundant.

But it's redundant for a reason: we can test a marker without performing another memory lookup.

That redundancy may be exactly what makes the representation fast.

This is an important example of the central principle we've developed:

The information-theoretic minimum isn't necessarily the fastest executable representation.

## B. A useful alternative: separate metadata from the pointer answer

One possible alternative would use a 32-bit descriptor for the translation base and format, with markers stored elsewhere.

But then each navigation step might require another memory access to determine whether it has reached a segment boundary.

Since segment testing happens in the hot loop, that could be a poor trade.

Another possibility is to derive termination from the lane's known remaining segment length or another piece of traversal state. Whether that's practical depends on how your segmentation algorithm determines endpoints and lengths.

Without the precise current segment invariants, we cannot assume that approach would work.

For now, I favor the 64-bit descriptor as a research baseline because it keeps marker information local to the navigation answer.

## C. We can quantify the size advantage

Let:

- \\(p\\) be the fraction of 32-row groups that are translations.
- \\(b\\) be the number of bytes used by the directory entry.
- \\(s\\) be the average payload bytes for nontranslation groups.

Then the approximate physical cost per row is:

\\[ \boxed{ S=\frac{8b+8(1-p)s}{32} } \\]

in bits per row, excluding exceptions and alignment overhead.

For our proposed 8-byte descriptors and 64-byte payloads:

\\[ S=2+16(1-p). \\]

With \\(p=0.776\\):

\\[ \boxed{S\approx5.584\text{ bits per row}} \\]

before exception overhead.

This is the origin of our earlier \~5.6-bit estimate.

How translation coverage changes modeled storage

0 bits5 bits10 bits15 bits20 bits0%20%40%60%77.6%90%100%

Model assumes one 8-byte descriptor per group and one 64-byte payload per nontranslation group. It is not a measured allocation.

The implication is interesting: the two-tier representation is highly sensitive to translation coverage, whereas the fixed 16-bit representation is not.

This may eventually justify selecting a representation based on properties of each block.

But choosing that policy would be a later systems experiment, not something established by the mathematics.

# Research cycle 7 — Could the decoder select a representation using BWT statistics?

We now have two useful measurements that can be obtained while scanning L:

1. The symbol-frequency distribution.
2. The BWT run structure.

The first determines the F intervals.

The second gives information about potential translation groups.

This suggests that some representation decisions might be made using statistics the decoder already computes or could compute with very little additional work.

For example, blocks with long BWT runs may be excellent candidates for translation compression, while nearly random blocks might be better handled by the direct table.

But there is a catch.

The exact fraction of translation-eligible fixed groups isn't determined by the total BWT run count alone. It depends on where the discontinuities fall relative to group boundaries.

Fortunately, once the occurrence lists are being constructed, that information is straightforward to determine.

This leads to a new research question:

Can the builder choose the best representation while constructing Ψ, without first materializing Ψ or performing a separate census pass?

In principle, yes.

A group encoder only needs to inspect its 32 values to determine which format fits.

The hard part is predicting whole-block performance from the resulting format distribution.

For example, a highly compressible block might be small enough in the two-tier format to fit in cache, but might also have exceptional accesses that introduce substantial latency.

A less-compressible block might not benefit enough from compaction to offset decoding overhead.

The choice could depend on both the data and concurrent decoder load.

That suggests an eventual adaptive representation strategy, but it would require measured performance models for each candidate.

For the research agenda, I would preserve this as an architecture-selection question, not assume automatic adaptation will help.

# Research cycle 8 — A more rigorous performance ceiling

The #81 result makes it important to avoid optimizing a representation in isolation.

Suppose the current fused build-and-walk pipeline consists of build time \\(B\\) and traversal time \\(W\\).

A new representation changes both:

\\[ B\rightarrow B', \qquad W\rightarrow W'. \\]

It wins only if:

\\[ \boxed{B'+W'\<B+W} \\]

for equivalent work and the same concurrency constraints.

This sounds trivial, but it leads to a useful threshold analysis.

Define:

\\[ \alpha=\frac{B}{B+W} \\]

as the current construction fraction.

If the compressed builder is \\(s_B\\) times as fast and the compressed walk is \\(s_W\\) times as fast, then the combined speedup is:

\\[ \boxed{ S=\frac{1}{\alpha/s_B+(1-\alpha)/s_W} } \\]

We can explore how sensitive the result is to the two stages.

For illustration, take \\(\alpha=0.45\\), close to the previously reported construction share in one cooperative configuration. That is not a fresh measurement of the #81-fused baseline.

Explore the build/walk trade-off

Compressed build speed

1.10×

Compressed traversal speed

0.95×

Baseline time spent in construction

45%

Predicted combined pipeline speedup

# 1.012×

1.2% faster than baseline

Simplified Amdahl-style calculation. Real multicore cache contention, overlapping stages and scheduling can violate this additive model.

The point is not to predict the actual result from a toy equation.

It's to establish how much traversal overhead the construction improvement can afford.

For example, if a compressed builder is 20% faster but traversal is 10% slower, the combined result can still be positive when construction accounts for a sufficiently large fraction of total time.

But #81 has already accelerated construction, changing that fraction and making additional walk overhead less attractive than before.

The stronger the existing fused builder becomes, the more demanding our compact representation must be about navigation cost.

This is a meaningful shift in the optimization economics.

# Research cycle 9 — What could invalidate the entire design?

Before reaching a fixed point, I want to make the strongest adversarial case against our own proposal.

There are four potential failure modes.

## 1. The census may be optimizing the wrong unit

The reported 77.6% is a row-weighted fraction of translation-eligible 32-row groups.

For a uniform 32-row grouping, row-weighted and group-weighted coverage agree, apart from the final partial group.

That's helpful.

But the coverage needs to correspond to the actual grouping used by the final descriptor layout.

If the census was performed on character-aligned groups while the proposed directory uses global groups indexed by `row >> 5`, the translation percentage could change.

Likewise, the symbol-boundary correction rate is only directly comparable if it uses the same group alignment.

This is one of the most important specification-level checks.

## 2. The compact builder might lose the fusion advantage

The current #81 construction win came from removing redundant work in a direct-array builder.

A compressed builder could reintroduce:

- Per-symbol buffering.
- Additional counters.
- Conditional format decisions.
- Record assembly.
- Synchronization at group boundaries.
- Scattered writes to compressed payload storage.

It could therefore save memory yet substantially increase construction instructions.

There is no guarantee that a compressed builder inherits #81's 31.3% construction gain.

This is probably the largest risk to whole-pipeline performance.

## 3. The two-tier payload path could interfere with MLP

A 22.4% dependent-payload frequency sounds manageable.

But the actual penalty depends on when the slow cases occur.

Suppose several lanes encounter nontranslation groups simultaneously.

Each must complete its descriptor load before it can issue the payload load. Those payload loads may miss cache, delaying the next Ψ address.

A lockstep scheduler might suffer more than the average latency suggests.

Alternatively, with enough lanes, other traversal chains may cover the delay.

The mixture formula alone cannot distinguish those outcomes.

So I would want the temporal clustering of slow paths during actual traversal measured, rather than just the fraction of slow rows in Ψ order.

This is an important new distinction.

Ψ storage order and Ψ traversal order are different.

The census measures structure in storage order. Performance depends on the order in which those structures are visited.

## 4. The compressed table may become instruction-bound

The current direct walk is highly optimized.

Even translation entries require:

- Address computation.
- A descriptor load.
- Format extraction.
- Base extraction.
- Offset arithmetic.
- Marker handling.
- Output-byte recovery or extraction.

Some operations can overlap, but none are free.

The directory may fit comfortably in cache, yet its decoding instructions could prevent the processor from fully exploiting memory-level parallelism.

In that case, shrinking the structure dramatically might improve cache statistics without improving throughput.

This is exactly why the instruction-count anomaly in Seward's paper is relevant, but not decisive.

# Research cycle 10 — The fixed point

At this stage, the mathematical and architectural questions have converged.

I no longer see a strong reason to invent additional record formats before obtaining the next measurements.

We have three meaningful classes of representation and a small number of decisive questions.

The current representation frontier

| Representation        | Storage goal           | Principal risk                          |
| --------------------- | ---------------------- | --------------------------------------- |
| Direct packed links   | 24 bits/row            | F recovery + packing overhead           |
| Fixed-line hybrid     | \~16 bits/row          | Register decoding after record load     |
| Translation directory | \~5–6 bits/row modeled | Second dependent loads and construction |

I would preserve all three because each occupies a different point in the possible space–time–parallelism frontier.

No further representation brainstorming is likely to improve our confidence as much as examining actual format distributions, construction traffic and navigation dependencies.

There is, however, one more useful conclusion before the handoff.

## The real breakthrough may be a direct-addressed compressed function

We've been calling this succinct permutation research, which is mathematically appropriate.

But the structure we are converging on has a more general description.

We have a function:

\\[ f:[0,n)\rightarrow[0,n) \\]

whose values are mostly piecewise translations.

We want to represent it compactly while allowing extremely fast random evaluation:

\\[ f(i) \\]

The fixed-group approach stores the function in regular blocks, with direct arithmetic addressing.

The two-tier approach stores immediately computable answers in a small descriptor array and only occasionally consults larger payloads.

This is a form of locally compressed, directly indexed function representation.

That abstraction matters because it connects our research to more than BWT inversion:

- Compressed graph navigation.
- Monotone integer indexes.
- Posting-list representations.
- Piecewise-affine mappings.
- Compact transition tables.
- Cache-aware lookup structures.

The BWT gives us a particularly favourable source of piecewise translation, but the architectural technique is more general.

I would not claim this abstract design is novel without a dedicated prior-art search. Still, it's a useful way to frame the research independently of the original compression format.

# Final research verdict and coordinator handoff

Research programme completed to its current evidence limit · 29 October 2026

## What changed because of #81?

The strong fused-construction result made us ask a more demanding question:

Can compressed Ψ be constructed directly, cheaply and in a form that is immediately navigable, rather than building a full permutation and compressing it afterward?

The resulting investigation produced four conclusions.

1\. Direct compressed construction is algorithmically plausible. A left-to-right BWT scan generates monotone Ψ occurrence lists naturally. The main uncertainty is efficient parallel group assembly without losing #81's fusion gains.

2\. Elias–Fano may be unnecessary for the rarest cases. Given the reported 0.49% exception population, simple raw 20-bit pointers may be a better fallback than elaborate variable-length encoding.

3\. The 77.6% translation result has a mathematical explanation. Translation discontinuities are bounded by the number of BWT runs. This connects the observed coverage to established run-length BWT theory and gives us a predictor for representation suitability.

4\. The two-tier descriptor is our strongest new architectural hypothesis. Translation groups can recover Ψ directly from an 8-byte descriptor; other groups use compact payload records. Its illustrative footprint is around 5.6 bits per row, but its actual traversal speed remains entirely unproven.

The strongest new insight is the connection between directly addressable translation groups and the distribution of BWT-run discontinuities.

## What we can now settle analytically

| Question                                                           | Conclusion                                          |
| ------------------------------------------------------------------ | --------------------------------------------------- |
| Can Ψ be generated in monotone occurrence order?                   | Yes, by a sequential scan of L with symbol counters |
| Can a translation group return its next row from one descriptor?   | Yes                                                 |
| Can arbitrary exception groups be represented without EF?          | Yes, using packed absolute positions                |
| Is translation coverage related to BWT run count?                  | Yes; the discontinuity count is at most \\(r-1\\)   |
| Can a fixed group be addressed without predecessor search?         | Yes, through arithmetic indexing                    |
| Does the two-tier scheme guarantee a throughput improvement?       | No                                                  |
| Can the full compressed builder inherit #81's gains automatically? | No                                                  |

## What genuinely remains to discover

These are the research questions that cannot be resolved responsibly from the current evidence.

- Per-block structure: Does the reported 77.6% translation coverage remain high across individual full-sized blocks and unfamiliar inputs?
- Construction: Can a parallel compressed builder avoid full intermediate arrays and expensive group-assembly synchronization?
- Dependency cost: Can the translation-directory common path approach the latency and instruction throughput of a direct `u32` load?
- Slow-path clustering: Do nontranslation groups appear in bursts during actual Ψ traversal, or can 32 lanes hide their latency?
- Real footprint: What is the allocated size after payloads, alignment, exception records and symbol handling?
- Best-concurrency throughput: Does the compact representation actually improve complete inversion when each design is allowed its optimal worker and lane configuration?

Those questions now dominate further theoretical speculation.

## Research handoff

# mibz2 — Succinct Ψ Research Phase II

**Research handoff, 29 October 2026**

## Central hypothesis

Directly constructed, compact Ψ representations may improve whole-decoder throughput by reducing construction traffic and shared-cache footprint while preserving memory-level parallelism.

The strongest new candidate is a two-tier, directly indexed translation directory.

## New mathematical result

Let (r) be the number of runs in the BWT last column L. Let (d) count the discontinuities between consecutive Ψ entries:

[\
d=#{j:\Psi(j+1)\ne\Psi(j)+1}.\
]

Then:

[\
d\le r-1.\
]

Consequently, for globally aligned groups of 32 Ψ values, at most (r-1) groups can be nontranslation groups.

This gives a structural relationship between translation coverage, BWT runs and the potential footprint of the compressed representation.

## Candidate representations

**A. 24-bit direct links:** preserve one direct pointer lookup.

**B. Fixed 64-byte hybrid records:** 32 positions per record, using translation, FOR-14, EF-12 and raw exceptions.

**C. Two-tier translation directory:** one 64-bit descriptor per group of 32 rows; translation groups give the answer directly, nontranslation groups reference payloads.

For C, the proposed descriptor contains a 32-bit segment-marker bitmap, a 20-bit translation base or payload index, 8-bit symbol metadata and 4-bit mode field. Mixed-symbol groups need exact alternative symbol recovery.

Using the reported 77.6% translation coverage and 64-byte nontranslation payloads gives a conditional estimate of approximately 5.584 bits per row before exceptions, padding and other overhead.

## Construction research

A scan of L emits Ψ destinations in monotone order within each character class.

A possible compressed builder maintains partial 32-entry groups and emits records directly. Parallel implementations require per-worker occurrence offsets and careful handling of groups crossing worker or symbol boundaries.

The existing fused #81 builder is the performance baseline. Direct compressed construction must not assume it inherits the #81 speedup.

## Decisive measurements

1. Per-block translation and discontinuity distributions, including BWT run counts.
2. Exact group formats, payload counts and physical allocated bytes.
3. Construction cost including buffering, memory traffic and synchronization.
4. Per-format next-address dependency latency and instruction count.
5. Slow-path frequency and clustering in actual Ψ traversal order.
6. Full decoder throughput under each representation's best concurrency configuration.

## Prior-art boundaries

Seward's `indexF`, compressed Ψ representations, uniform blocks, partitioned Elias–Fano, patched frame-of-reference and cache-line Elias–Fano all have established prior art.

The research opportunity is a systems-specific space–time–parallelism study of direct compressed construction and navigation for cooperative BWT inversion.

## Decision boundary

The current theoretical investigation has reached its evidence limit.

Further useful advances require the raw group census, exact codec layouts or performance measurements. More speculative format variants should not be treated as stronger hypotheses without that evidence.

**Principal question:** Can a small, directly addressed Ψ descriptor provide sufficiently cheap common-case navigation that its substantial cache-footprint advantage outweighs the cost of exceptional dependent payload accesses and compressed construction?

## My final conclusion

The original research asked whether succinct data structures could make mibz2 faster.

We've now reached a more precise and ambitious question:

> Can we avoid storing most inverse-BWT pointers individually, without surrendering the ability to follow them at extraordinary speed?

The latest investigation has sharpened the answer considerably.

For the reported corpus, many pointers appear to follow simple translation rules. Those rules could be encoded in small, directly indexed records. We have identified a mathematical connection to BWT runs, a plausible direct-construction algorithm, and a set of representation choices whose performance trade-offs are now explicit.

The surprising possibility is that the fastest compact representation may not resemble a traditional succinct index at all.

It may instead resemble a tiny array of directly executable translation descriptors, with compressed records reserved for the cases that don't fit.

That's where the research currently converges.

The programme has reached its natural fixed point: further theory alone is unlikely to settle the remaining uncertainties. The next meaningful discoveries must come from examining the actual corpus and the behaviour of these representations on the target processor.
