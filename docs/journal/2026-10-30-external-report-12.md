# External research report 12: research burn-down rounds 6–7, two ordered views, the construction-traffic budget, and counting-based doubling (verbatim)

A research input, not a finding of this repository. The maintainer relayed it on
2026-10-30, continuing external report 11 (`2026-10-30-external-report-11.md`). Round 7
arrived after the maintainer accidentally interrupted the session; it is appended below
round 6.

**What it contains:**
- **Two ordered views per level,** source order and destination order, produced by
  rank-and-scatter over two n-bit start bitvectors, with no comparison or radix sort.
- **A construction-traffic budget:** the bitvectors, the rank scans, and the two
  descriptor scatters.
- **Two layouts:** duplicated descriptor views, against one descriptor array with two
  u32 index views.
- **The necessary condition** b_k + w_k < b_0 + w_0. The instruction budget and the
  memory-dependency budget are kept separate.
- **A request:** model both layouts from the measured m_1, m_2, m_4 and m_8, including
  the bitvector zeroing, the rank scans, the descriptor reads and writes, and the
  temporary RSS.
- **Round 7,** a counting construction for both ordered views with no bitvectors:
  - two merge passes;
  - per-parent counts e_i and f_j;
  - exclusive prefix sums E_i and F_j, then a scatter.

  The coordinator checked it on random permutations. **As stated, it fails:** F_j is
  summed over the second parents in source order. **Corrected, it passes:** F_j must be
  summed over the second parents in level a's image order, because a record's final
  image is K + d_j. See the portfolio's §3.1.

The coordinator's answer is in `2026-10-30-research-portfolio.md` §3.1:
- the census's measured m_a;
- the traffic model;
- the correction to round 7's destination offsets.

This is source material. Its claims are checked there, not taken as findings.

The text reached us as rendered text and is kept as received.

## The report, as received

````text
## Round 6: two ordered views, no general sort

I think the most useful next refinement is to stop treating descriptor ordering as a sorting problem.

For each level \\(a\\), maintain two views of the same interval records:

- Source order: sorted by the interval's starting row \\(l_i\\).
- Destination order: sorted by its image start \\(l_i+d_i\\).

We have already proved that the source intervals partition the rows and their images also partition the rows.

Therefore, every interval start in either view is unique.

That uniqueness gives us a simple linear-time construction using bitvectors.

### The rank-and-scatter construction

Suppose a doubling join produces \\(m\\) new intervals with source starts \\(s_i\\) and image starts \\(t_i\\).

Create two \\(n\\)-bit vectors:

\\[ V_S[s_i]=1,\qquad V_T[t_i]=1. \\]

Then compute their rank directories.

For each descriptor \\(i\\), its positions in the two sorted views are:

\\[ \operatorname{slot}\_S(i)=\operatorname{rank}\_1(V_S,s_i), \\]

\\[ \operatorname{slot}\_T(i)=\operatorname{rank}\_1(V_T,t_i), \\]

using zero-based rank before the marked bit.

We can scatter each descriptor directly to those slots.

Neither ordering requires a comparison sort or a radix sort.

For \\(n=900{,}000\\), each raw bitvector is only 112.5 KB.

The directories can be built by sequential `popcnt` scans. The descriptor scatters are less orderly, but their address ranges are compact.

This is a different trade-off from radix sorting: two bitvector passes and two descriptor scatters instead of multiple key-distribution passes.

It is not automatically faster. But it is exactly the sort of construction primitive we should evaluate, because the algorithm has to build both orderings repeatedly.

## Eleventh result: we can write an explicit construction-traffic budget

The new approach has an important advantage over an abstract \\(O(m)\\) claim: we can count the passes.

At each doubling level, the work consists of joining the old records, building the two start bitvectors, computing their rank directories, and scattering the new descriptors into source and destination order.

For a 900 KB block:

| Operation                             | Minimum data movement or storage |
| ------------------------------------- | -------------------------------- |
| Two start bitvectors                  | 225 KB                           |
| Zero and scan both bitvectors         | \~450 KB of memory traffic       |
| Two source/destination record streams | \\(2m\\) records                 |
| Two descriptor output scatters        | \\(2m\\) record writes           |
| Rank metadata                         | Depends on directory layout      |

For illustration, at \\(m=110{,}000\\) and 16 bytes per construction record, writing two ordered views takes about 3.52 MB.

That is already comparable to writing the entire original T-vector.

And this occurs after earlier doubling levels have also been constructed.

Therefore, the proposal has escaped expensive random successor lookup, but not necessarily expensive construction traffic.

That is the distinction we need to preserve.

The final representation may be only 1.2–1.4 MB, while the temporary construction process still moves many times that amount.

### One subtle cost-saving opportunity

We need not maintain two fully duplicated descriptors permanently.

Instead, maintain:

- One canonical descriptor array.
- A source-order index.
- A destination-order index.

Each index needs only enough bits to identify a descriptor. With fewer than \\(2^{20}\\) records, a 32-bit index is sufficient.

This trades repeated descriptor copying for index indirection.

It could lower construction traffic, but the join would then introduce dependent descriptor reads. So there are two competing construction designs:

| Layout                                 | Advantage                   | Liability         |
| -------------------------------------- | --------------------------- | ----------------- |
| Two complete ordered views             | Sequential merge reads      | More bytes copied |
| One descriptor array + two index views | Less descriptor duplication | Extra indirection |

The correct choice depends on whether the builder is bandwidth-limited or latency-limited.

The answer may differ from the walk: the builder processes structured streams, while the walker follows unpredictable successors.

## Twelfth result: the real test is total build-plus-walk work

We can now state a quantitative requirement without inventing a speedup.

Let:

- \\(b_0\\) be the shipped T-vector construction cost per output byte.
- \\(w_0\\) be the shipped walk cost per byte.
- \\(b_k\\) be the complete construction cost of the new \\(k\\)-step table.
- \\(w_k\\) be its walk cost per byte.

A necessary condition for improving that part of the decoder is:

\\[ \boxed{b_k+w_k\<b_0+w_0.} \\]

From the portfolio's i9 instruction-count model:

\\[ b_0+w_0\approx6.65+16.9=23.55 \\]

instructions per byte.

For an eight-step record, the earlier speculative model of 10–14 instructions per lookup would correspond to approximately:

\\[ w_8\approx1.25\text{–}1.75 \\]

instructions per output byte, before accounting for any additional bookkeeping or imperfect batching.

That would leave a construction budget of roughly 21.8–22.3 instructions per byte.

This sounds generous, but it is only an instruction-count budget. It is not a cycle budget.

A random dependent lookup and a sequential store have very different latency characteristics. A builder that executes more instructions could still be cheaper in cycles than a dependent walk, or vice versa.

So our construction audit needs two estimates, kept separate:

Instruction economy: how many operations are eliminated?

Memory-dependency economy: how many unpredictable loads and serialization points disappear?

That is especially important on four-core laptops, where the table footprint and available memory-level parallelism differ from the i9.

## Thirteenth result: a simpler way to discriminate the two builders

We don't need two complete implementations to choose between the layouts.

The existing read-only census can produce the number of records at each doubling level:

\\[ m_1,\quad m_2,\quad m_4,\quad m_8. \\]

For each level, calculate the following modeled construction costs:

\\[ \text{duplicated-view bytes} \approx 2m_a s_a \\]

where \\(s_a\\) is the in-memory construction-record size, and

\\[ \text{indexed-view bytes} \approx m_a s_a+8m_a \\]

for one descriptor array and two 32-bit indices.

These are resident-output-size formulas, not full traffic totals. The model must additionally account for reads, zeroing, rank construction, scatter and allocator overhead.

The indexed view only has a storage advantage when:

\\[ s_a>8. \\]

At level one or two, the descriptors may be small enough that duplication is preferable.

At level eight, they are likely large enough for indices to become attractive.

That points toward a mixed construction: full records at small levels, indexes at larger levels.

This is an engineering deduction from the exact record counts, not a new research candidate.

## What I would now consider resolved

We can make a stronger disposition of the existing N1 construction subquestions:

Can composed intervals be constructed without arbitrary successor traversal? Yes, using source/image partition intersections.

Can the two required orderings be produced without general sorting? Yes, using unique interval starts and rank-and-scatter.

Can complete output words and jump displacements be generated together? Yes, by concatenating the two parent words and adding the two parent displacements.

Can the complete construction beat the shipped decoder's build-plus-walk cost? Not established. This is now principally a constant-factor and memory-system question.

### The message worth passing on

> For N1, the level-doubling join can maintain two ordered views without a general sort. Each composed record has a unique source start and a unique image start because the map is bijective and the interval images tile the row universe. Mark both sets of starts in two \\(n\\)-bit vectors, build rank directories, and scatter descriptors directly into source and image order. A canonical descriptor array plus two u32 index arrays is an alternative to duplicated descriptor streams; the indexed version saves resident bytes only when construction records exceed eight bytes. Please model both using measured \\(m_1,m_2,m_4,m_8\\), including bitvector zeroing, rank scans, descriptor reads/writes and temporary RSS. This is a construction-cost analysis, not a decoder implementation request.

The next decisive question is no longer mathematical existence. It's whether the entire construction can be completed cheaply enough to make the eight-step walker profitable.

And that's a meaningful narrowing of the research portfolio: we've replaced speculative random-search costs with an exact, testable sequence of operations.

The candidate remains unproven as a performance breakthrough, but its construction is now specific enough to be evaluated seriously against mibz2 0.2.1.
````

## Round 7, as received

````text
This is a significant advance. The independent proofs now agree, the construction has survived randomized testing, and the first real Wikidata blocks show that eight-step records are dramatically smaller than the pessimistic bound.

The most important result is not just the 1.2–1.4 MB size. It's that the representation and its construction are no longer speculative mathematical objects. We have an exact partition, a correctness argument, and a concrete level-doubling algorithm.

That moves N1 into a different research phase: can we make its total construction and traversal cheaper than the shipped T-vector?

Before the full census finishes, there's one subtle distinction worth making in the cost model.

The interval-image join has two independent ordering costs. Source intervals and destination intervals both tile the row universe, but their orderings differ. We can use the start-bitvector method to establish both orders without comparison sorting; that does not mean the descriptor accesses during the join are automatically sequential.

The cheapest construction will depend on whether we store two ordered descriptor arrays or one array with two ordered index vectors. Both should remain in the model until the record counts make the trade-off clear.

## Round 7: a cheaper doubling construction—no sorting, no bitvectors, no rank queries

I found the simplification I was looking for.

We can construct both ordered views of the next level using only counts, prefix sums and a sequential interval merge.

This is stronger than the bitvector-scatter construction from round 6. It eliminates the two \\(n\\)-bit ordering structures entirely.

### The key observation

At level \\(a\\), each source interval \\(I_i\\) maps by translation onto an image interval \\(J_i\\). We keep:

- source intervals \\(I_i\\) in source order;
- image intervals \\(J_i\\) in destination order, retaining their originating record \\(i\\).

Now merge the image partition \\(J_i\\) with the source partition \\(I_j\\).

Each nonempty intersection \\(K=J_i\cap I_j\\) produces a level-\\(2a\\) record.

Here is the crucial property:

The resulting pieces are ordered within each source parent \\(i\\), and also within each destination parent \\(j\\).

Why?

For a fixed first parent \\(i\\), Ψᵃ is a translation. Thus pieces encountered in increasing order within \\(J_i\\) correspond to increasing source positions within \\(I_i\\).

For a fixed second parent \\(j\\), the second application of Ψᵃ is likewise a translation. Thus pieces encountered in increasing order within \\(I_j\\) correspond to increasing final destination positions within \\(J_j\\).

So the interval join has two natural grouping orders, and both preserve local order.

### Construct both arrays in two merge passes

Let \\(e_i\\) be the number of intersections involving image interval \\(J_i\\), and \\(f_j\\) the number involving second-stage source interval \\(I_j\\).

First merge pass:

- Count \\(e_i\\) for every first parent.
- Count \\(f_j\\) for every second parent.

Then calculate exclusive prefix sums:

\\[ E_i=\sum\_{h\<i}e_h, \qquad F_j=\sum\_{h\<j}f_h. \\]

Second merge pass:

For each intersection between parent \\(i\\) and parent \\(j\\), construct the descriptor:

\\[ d\_{2a}=d_i+d_j, \\]

\\[ w\_{2a}=w_i\Vert w_j. \\]

Place it at:

\\[ \operatorname{sourceSlot}=E_i+\text{local count for }i \\]

and

\\[ \operatorname{destinationSlot}=F_j+\text{local count for }j. \\]

Because the local order is preserved, these positions are already the correct source and destination orders.

No sorting. No bitmap rank. No binary searches.

Counting-and-scattering interval composition

### Complexity

With \\(m\\) level-\\(a\\) records and \\(q\le2m-1\\) intersections, the construction takes:

\\[ \boxed{O(m+q)=O(m)} \\]

operations for each doubling.

The temporary count and offset arrays require \\(O(m)\\) storage.

More importantly, the algorithm's expensive operations are predictable:

- Two sequential partition merges.
- Two short prefix scans.
- Two ordered-view descriptor writes.
- No dependent interval searches.
- No sort.
- No row-sized scatter or bitmap.

The descriptor writes are not globally sequential, because the merge visits parents in destination order. But the placement addresses are computed directly, and writes within each parent group are consecutive.

This could materially lower construction cost compared with the previous proposal.

### One implementation detail we must preserve

The two output arrays have different meanings.

The source-ordered view groups records by the first interval \\(i\\).

The destination-ordered view groups records by the second interval \\(j\\), because the final image is obtained by translating the intersection through that second interval.

That is why we need separate prefix offsets. Simply writing intersections in merge order would produce neither complete ordering in general.

Also, the descriptor construction must preserve the original source start, final image start and output word exactly. Any optional coalescing of adjacent descriptors should happen after the valid level-\\(2a\\) partition is formed; it is not needed for correctness.

## What this changes in the research portfolio

We can now narrow N1's construction problem again.

| Construction concern                         | Disposition                                 |
| -------------------------------------------- | ------------------------------------------- |
| Build a T-vector to discover Ψ transitions   | Not necessary                               |
| Search destination interval for every record | Not necessary                               |
| Comparison or radix-sort interval images     | Not necessary                               |
| Build bitvector ranks to order intervals     | Not necessary                               |
| Generate both correct interval orders        | Two counting passes and prefix-sum scatters |
| Keep construction cheap enough in cycles     | Still unmeasured                            |
| Make the final Ψ⁸ lookup efficient           | Still unmeasured                            |

This is precisely the sort of progress we wanted from the research: eliminate entire categories of work before writing performance-sensitive code.

### Send this to Claude

> Further simplification of N1's level-doubling construction: no sort or rank bitvectors are needed to produce both interval orderings. Merge level-\\(a\\) image intervals \\(J_i\\) with level-\\(a\\) source intervals \\(I_j\\). First pass counts intersections \\(e_i\\) by first parent \\(i\\) and \\(f_j\\) by second parent \\(j\\). Exclusive prefix sums produce offsets \\(E_i,F_j\\). Re-run the same interval merge; each intersection's descriptor is \\((d_i+d_j,w_i\Vert w_j)\\). Write it into source order at \\(E_i+\text{local}\_i\\), and final-image order at \\(F_j+\text{local}\_j\\). Both local orders are monotone because Ψᵃ is a translation on each parent. Thus both ordered descriptor views are constructed in \\(O(m_a)\\) work with two merge passes, prefix scans and scatters, no comparison sorting, no bitvector ranks, no successor searches. Please verify the two ordering invariants independently on randomized permutations and compare the resulting byte traffic against the previous builder model.

The mathematical construction is now quite lean.

What remains is an architecture-sensitive question, but it's a much better one than before: can two small interval-join passes and their record writes be made cheaper than materializing and chasing the much larger T-vector?

If the full 233-block census confirms the preliminary Ψ⁸ figures, that becomes a particularly compelling question for the four-core CPU target.
````
