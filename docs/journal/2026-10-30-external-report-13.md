# External research report 13: research burn-down rounds 8–9, exactness of the join, unsplit parents, segment boundaries (verbatim)

A research input, not a finding of this repository. The maintainer relayed it on
2026-10-30, continuing external report 12 (`2026-10-30-external-report-12.md`). The
researcher had read the corrected §3.1 in PR #117.

**What it contains:**
- **Exactness:** C_{a+b} = C_a ∪ Ψ^{−a}(C_b), so the image-partition join produces the
  minimal records directly, with no coalescing. It follows that
  m_{2a} = m_a + |Ψ^{−a}(C_a) \ C_a| ≤ 2m_a − 1.
- **Unsplit parents:** at least 2m_a − m_{2a} parents stay whole on doubling. That is at
  least about 69% of Ψ⁴ parents on the preliminary figures.
- **Segments:** at k = 8, at most 7 × 512 = 3,584 scalar-remainder bytes a 900 KB
  block, under 0.4%. On that basis the researcher concludes the cooperative segments
  need no redesign.
- **Builder work** split into mandatory parent processing and work caused by splitting.

**The coordinator's checks** are in `2026-10-30-research-portfolio.md` §3.1:
- the census's measured child-count distribution;
- one correction to the segment argument. The shipped room walk finds a segment's end
  by reaching a flagged row; it doesn't know the segment's length (the marker audit,
  #100: "lengths are unknowable without walking"). So the remainder must be enforced
  when the records are built: each record is capped so that it never passes a segment's
  start row. The bound is the same, by a different mechanism.

Two further parts follow round 8:

**Round 9.**
- The researcher accepts the round-7 correction.
- It proposes masked final-word stores at segment tails, which would need no fallback
  decoder, and separates construction peak RSS from resident table size.

**The researcher's acceptance** of the coordinator's reply to round 9:
- the masked-tail scheme needs each lane to know its remaining segment length, which
  the shipped walk does not;
- the construction-time caps stand, provided they meet the segment-terminal invariant
  the researcher states.

The coordinator checked that invariant by brute force. Walking records alone from every
segment start emits exactly that segment's bytes and stops exactly on the next flagged
row: 10,188 segments, 0 failures.

This is source material. Its claims are checked there, not taken as findings.

The text reached us as rendered text and is kept as received.

## The report, as received

````text
## Round 8: one more simplification—and a hard limit on the decoder's extra work

I've read Claude's corrected §3.1 in PR #117. The revised proof, the 1→2→4→8 construction and the run-native starting point are now consistent.

There are two useful consequences we haven't fully exploited.

### 1. The doubling join already produces the minimal partition

We previously included an optional coalescing pass after composing intervals.

For mibz2's stable Ψ construction, that pass is unnecessary.

Let \\(C_a\\) be the exact breakpoint set for a complete \\(a\\)-byte output-producing record, and let \\(C_b\\) be the corresponding set for \\(b\\) bytes.

The partition identity gives:

\\[ C\_{a+b} = C_a\cup\Psi^{-a}(C_b). \\]

The image-partition join splits the level-\\(a\\) source intervals at precisely those preimages of level-\\(b\\) boundaries.

So its resulting intervals are exactly the level-\\((a+b)\\) minimal records.

There are no redundant interior boundaries to remove, apart from representation-specific endpoint conventions.

This matters because the builder no longer needs to compare adjacent output words or displacements to determine whether records can be merged.

It can calculate the number of output descriptors in the counting pass, allocate exactly that many, and fill them once.

The exact recurrence is:

\\[ \boxed{ m\_{a+b}=m_a+ \left|\Psi^{-a}(C_b)\setminus C_a\right| } \\]

where \\(m_k=|C_k|+1\\).

For doubling:

\\[ \boxed{ m\_{2a}=m_a+ \left|\Psi^{-a}(C_a)\setminus C_a\right| \le 2m_a-1. } \\]

That's not merely a capacity bound. It is the number of descriptors the merge must produce.

It also means that Claude's direct-computation cross-checks can use record boundaries and descriptors as exact equality tests, with no canonicalization or coalescing phase.

### 2. Most Ψ⁴ intervals may survive doubling to Ψ⁸ without splitting

The first three blocks give approximately:

\\[ m_4=76\text{–}85\text{k},\qquad m_8=99\text{–}113\text{k}. \\]

Consider the \\(m_4\\) parent intervals. Each produces at least one Ψ⁸ interval. Every additional child increases the total record count by one.

Therefore, if \\(s\\) parent intervals split into two or more children,

\\[ s\le m_8-m_4. \\]

So the number of parents that do not split is at least

\\[ \boxed{2m_4-m_8.} \\]

For an illustrative paired block with \\(m_4=80{,}000\\) and \\(m_8=105{,}000\\):

\\[ \boxed{\text{at least }55{,}000\text{ of 80,000 parents remain intact.}} \\]

That's at least 68.75%.

Even using the least favourable combination of Claude's preliminary ranges, the lower bound remains above 50%.

This creates a concrete opportunity within the existing construction, rather than a new research branch.

A substantial majority of level-four records may be convertible into level-eight records with no subdivision.

The output still requires the second level's word and displacement, but the join machinery can recognize the unsplit case from interval endpoints.

The key measurement to extract from Claude's existing census is the distribution of child counts per parent:

\\[ \\#\\{e_i=1\\},\quad \\#\\{e_i=2\\},\quad \\#\\{e_i\ge3\\}. \\]

We already have a rigorous lower bound on the first quantity. The exact distribution would tell us whether a specialized no-split construction path is worthwhile.

No additional corpus traversal is necessary if the join already records parent counts.

## 3. Cooperative segment boundaries are not a serious obstacle

The shipped cooperative walk divides a block into approximately 512 segments.

An eight-step jump can be used for every complete group of eight output positions within a segment. The remainder can use the ordinary one-step method or shorter composed jumps.

For a segment of length \\(\ell\\), the scalar remainder is

\\[ \ell\bmod8\le7. \\]

Across 512 segments, the maximum number of scalar-remainder bytes is

\\[ 512\times7=3{,}584. \\]

For a 900,000-byte block:

\\[ \boxed{\frac{3584}{900000}<0.4\\%.} \\]

This is an upper bound on bytes requiring boundary handling, not on execution overhead; a fallback's setup may cost more than the bytes themselves.

But it removes a possible algorithmic objection: we do not need to redesign the 512-segment cooperative decomposition to accommodate eight-step navigation.

The current segmentation can remain intact. Each lane advances by eight until fewer than eight output positions remain in its current segment.

Any fallback must preserve the exact original-row position and output orientation, but that is a correctness matter rather than a structural impediment.

## 4. A more useful builder metric: work per additional output-producing boundary

The existing cost model counts total records processed.

We can now split that into two different costs:

\\[ \text{builder work} = \text{mandatory parent processing} + \text{work caused by splitting}. \\]

For doubling \\(a\to2a\\):

- Every parent interval requires some processing.
- Exactly \\(m\_{2a}-m_a\\) extra children are created.
- At least \\(2m_a-m\_{2a}\\) parents are copied forward without splitting.

The second term is where the combinatorial complexity lives.

This gives us a more discriminating analysis than saying that the builder processes approximately 200,000 records across three levels.

For example, if the parent records are mostly unsplit, a construction can use a cheap direct path for most of them and invoke the intersection loop only around actual crossings.

But I would be cautious: detecting whether a parent is unsplit still requires examining its image interval against the next partition. We must not count the shortcut as free.

The objective is to avoid repeatedly materializing child descriptors when no subdivision occurs—not to pretend the necessary interval checks disappear.

## 5. The decoder's core cost model is now unusually specific

For an eight-step record, the hot path is conceptually:

One Ψ⁸ decoding iteration

Unlike the existing T-vector walk, this performs approximately one descriptor lookup for eight output bytes.

The remaining questions are now tightly bounded:

- Does the rank directory require one cache-line access per jump, and does the following record access introduce another costly dependency?
- Can the eight-byte output word be emitted efficiently in the correct order?
- Does the builder's additional work fit within the cycles saved by the shorter walk?
- Does the smaller resident table improve four-worker cache behaviour, rather than merely kernel throughput?

The last question matters because on the representative blocks a roughly 1.2–1.4 MB record payload plus a 225 KB directory is considerably smaller than the 3.6 MB T-vector. That's a potentially valuable residency improvement, but it does not prove a corresponding speedup.

## Updated burn-down

| Existing issue                       | Status after this round                                                    |
| ------------------------------------ | -------------------------------------------------------------------------- |
| N1: minimal partition after doubling | Solved: intersection join is exact; coalescing unnecessary                 |
| N1: children per parent              | Bounded: at least \\(2m_a-m\_{2a}\\) parents remain unsplit                |
| N1: segmented walk compatibility     | Bounded: fewer than 0.4% fallback bytes at \\(k=8\\), 512 segments, 900 KB |
| N1: builder operation count          | Sharpened into mandatory-parent and subdivision costs                      |
| N1: actual construction cycles       | Still open                                                                 |
| N1: rank-indexed walk latency        | Still open                                                                 |
| N3: full-decoder benefit             | Still dependent on N1 and front-end integration                            |

### The message to pass to Claude

> Round 8 derives two useful refinements from the corrected partition identity. First, \\(C\_{a+b}=C_a\cup\Psi^{-a}(C_b)\\), so the image-partition join constructs the exact minimal level-\\((a+b)\\) records; no post-join coalescing is necessary for the stable Ψ. Second, since each level-\\(a\\) parent has at least one child, at least \\(2m_a-m\_{2a}\\) parents remain unsplit. With preliminary \\(m_4\approx80k\\) and \\(m_8\approx105k\\), at least \~69% of Ψ⁴ parents survive intact. Please include child-count distribution in the builder model if it is already available from the join. Finally, eight-step jumps need at most \\(7\times512=3,584\\) scalar remainder bytes per 900 KB block, under 0.4%, so existing cooperative segment boundaries need no redesign. The unresolved issues are construction cycles and the latency/throughput of rank-plus-record lookup, not correctness or segment compatibility.

The meaningful progress is that N1 now has fewer escape hatches. We know what the records are, how many exist, how to construct them, how they interact with the cooperative segmentation, and where the remaining costs must come from.

The only thing I would not do is call it a performance breakthrough yet. We have a credible candidate for one—and an increasingly short list of reasons it might fail.
````

## Round 9, as received

````text
Claude's correction is right. I made an ordering error in round 7, and the smallest counterexample, \\(\Psi=[1,0]\\), exposes it cleanly.

For the doubling join, the two orderings are:

- Source order: group children by their first parent \\(i\\), in the source order of the level-\\(a\\) intervals.
- Destination order: group children by their second parent \\(j\\), in the image order of the level-\\(a\\) intervals.

The second prefix sum must therefore follow the destination-order permutation of the parent records, not their source-order indices.

Claude's 10,440-case test is useful evidence that the corrected construction works. More importantly, the interval-translation invariant explains why it works, so the correction doesn't depend on those tests alone.

The no-sort, no-bitvector, no-search construction survives. Only the destination-offset ordering changes.

I would preserve both the failed version and the corrected version in report 12, as Claude has done. The failure is an instructive example of why a bijective mapping gives us two valid interval orderings, not one interchangeable ordering.

## Round 9: segment boundaries can be handled without a fallback decoder

There's another simplification worth passing to Claude, and it removes a dependency I left open in round 8.

We previously allowed up to seven scalar remainder bytes per cooperative segment. But we need not decode those bytes using the original Ψ representation at all.

An eight-step record emits an eight-byte word and provides the row after eight transitions. Suppose a segment has three bytes remaining.

We can:

1. Look up the ordinary Ψ⁸ record for its current row.
2. Write only the first three bytes of that record's output word, in decoding order.
3. Discard the other five bytes and the resulting successor.

That's correct because the segment's output is defined by the first three transitions. The final row state is irrelevant once the segment is complete.

Final partial output word

A

B

C

D

E

F

G

H

&#x20;Write first three bytesDiscard remaining five

This means the decoder can use the same eight-step table for every output position, including segment endings.

For segment lengths \\(\ell_1,\ldots,\ell\_{512}\\), the total number of table lookups is

\\[ N_8=\sum\_{s=1}^{512}\left\lceil\frac{\ell_s}{8}\right\rceil. \\]

Since the segments together contain \\(n\\) bytes,

\\[ \boxed{ \frac n8\le N_8<\frac n8+512. } \\]

For \\(n=900{,}000\\), that's between approximately 112,500 and 113,012 lookups.

The overhead from partial endings is therefore less than 0.46% of the ideal eight-byte lookup count, and there is no requirement to retain the T-vector or smaller Ψ¹/Ψ²/Ψ⁴ tables merely to finish segments.

This result assumes the cooperative segments are independent output ranges, as in the existing segmented walk. Any implementation must use the correct byte ordering and avoid writing across the output boundary.

### Why this matters

It strengthens N3, the proposed run-native pipeline.

Previously, a completely T-vector-free decoder still seemed to need a fallback mechanism for segments whose lengths weren't multiples of eight.

That requirement disappears.

The only exceptional operation is a bounded partial store at the end of each segment—not an alternative navigation algorithm.

It also gives us a precise test for the eventual walker: verify every segment's bytes, including lengths 1–7, without using the old Ψ table.

## Another useful constraint: the two ordered views needn't survive decoding

Claude's corrected construction needs two ordered views during building. But the finished rank-indexed Ψ⁸ decoder needs only:

- The source-ordered records.
- The row-to-record directory.
- The cooperative segment-start information.

The destination-ordered view is a temporary construction aid.

Therefore the peak-memory model should distinguish:

\\[ \text{construction peak RSS} \\]

from

\\[ \text{resident decoding table size}. \\]

A 1.2–1.4 MB final record payload does not mean the construction peak is equally small. Conversely, temporary destination-order arrays can be released or reused before the walk.

That distinction matters particularly for four-core laptops, where four simultaneous block decoders could multiply the temporary memory cost.

## Current dispositions

| Research issue                       | Result                                       |
| ------------------------------------ | -------------------------------------------- |
| Round 7 destination ordering         | Corrected and independently tested by Claude |
| Construction sorting                 | No longer necessary                          |
| Construction searches                | No longer necessary                          |
| N1 segment-boundary handling         | Solved with masked final word stores         |
| Need to retain Ψ¹ as fallback        | No, not for segment endings                  |
| Destination-order arrays during walk | Unnecessary                                  |
| Construction CPU cost                | Still open                                   |
| Ψ⁸ lookup latency                    | Still open                                   |
| Full-decoder gain                    | Still unmeasured                             |

The useful message for Claude is:

> One more N1 simplification: cooperative segment tails do not require a scalar Ψ fallback or retained lower-level tables. For each segment, decode full eight-byte records normally; at the final partial word, look up one Ψ⁸ record and write only the remaining 1–7 bytes. Discard the unused output and successor. The number of lookups is \\(\sum_s\lceil\ell_s/8\rceil\<n/8+512\\), under 0.46% above the ideal count for a 900 KB block. The completed walker retains only the source-ordered Ψ⁸ records and rank directory; the destination-ordered views are temporary construction data. Please include masked-tail correctness cases and peak construction RSS separately from resident table size.

The emerging picture is increasingly clear: the mathematical obstacles to an eight-byte CPU inversion path are largely gone. The real uncertainty is whether the two-pass interval builder and rank-plus-record walker have the right constant factors to outperform mibz2's remarkably efficient T-vector.

And that is a much narrower uncertainty than we started with.
````

## The researcher's acceptance of the coordinator's reply to round 9, as received

````text
Claude is right. I made an invalid assumption about the shipped walk: the lanes don't know their remaining segment lengths. They discover segment termination by encountering a flagged row.

My proposed masked-tail scheme therefore cannot be applied directly to the existing cooperative walk. Knowing that at most seven bytes could remain isn't the same as knowing when to stop.

Construction-time caps are the better direction. They let the shortcut representation respect the existing segmentation without asking the walker to predict its endpoint.

There's an important distinction to preserve in report 13, though:

The ≤3,584 bound counts potentially exceptional output bytes, not necessarily the number of additional records or their construction cost.

For 512 segments and eight-step jumps, the arithmetic bound is:

\\[ 512(8-1)=3{,}584. \\]

But we still need to establish how the capped records handle an early termination encountered partway through a jump. The construction must supply enough information to stop at precisely the flagged row, without emitting bytes belonging to the next segment or losing the next segment's starting state.

I would make that a direct correctness invariant:

> Starting from any valid cooperative segment start, repeated capped-record operations must emit exactly that segment's bytes, encounter its terminal marker exactly once, and never traverse beyond it.

And I'd distinguish two possible costs: extra construction work to impose the caps, and extra decoding work caused by shorter terminal jumps.

The mathematics of the ordinary Ψ⁸ records remains intact. This is a boundary condition imposed by mibz2's actual cooperative execution model, not a refutation of the composed representation.

I agree with Claude's correction and would retain the construction-time approach, provided the segment-terminal invariant is verified against the existing flagged walk.
````
