# External research report 21: research burn-down round 20, indexed descriptor views (verbatim)

A research input, not a finding of this repository. The maintainer relayed it on
2026-10-30, after external report 20 (`2026-10-30-external-report-20.md`).

**What it contains:** one descriptor array plus a u32 image-order permutation in place of
two full views, with the corrected round-7 invariant kept.
- **The saving:** a third of intermediate payload, 24m → 16m bytes.
- **The caution:** two image-order scans can erase the write saving. The design is
  proven smaller, not proven faster.
- **The asks:** a per-level mixed layout, and whether folding the counting pass into the
  previous level's scatter is possible without secretly performing the join twice.

**The coordinator's answer** (`2026-10-30-research-portfolio.md` §3.1(j)):
- **Layouts:** the one-copy layout is layout B of §3.1(i), published earlier. The round's
  conclusion matches that model: under two passes, all eight per-level mixes move exactly
  23.1 B/B.
- **Folding:** that is layout C. A level's children counts are popcounts of start
  bitvectors marked during the previous level's scatter, so there is no second join.
  It moves traffic to 17.0–17.9 B/B for every mix.
- **Layout choice:** layout only trades gathers against peak memory.

This is source material. Its claims are checked there, not taken as findings.

The text reached us as rendered text (some formula symbols and table cells lost) and is
kept as received.

## Round 20, as received

````text
Burn-down round 20: eliminating duplicate descriptor storage

The portfolio’s current builder materializes two complete ordered views at intermediate levels. That is not mathematically necessary.

We can represent both orderings using one descriptor array and one permutation array. This preserves the corrected two-pass construction without introducing comparison sorting or predecessor searches.

It addresses the memory part of #53 directly and narrows the construction-cost questions behind #45 and #48.

1. A one-copy representation

At level , store:

* records[i]: the complete Ψᵃ descriptor, in source-row order.
* image_order[t]: a 32-bit index into records, arranged in image-row order.

The two iteration orders become:

\text{Source order: } records0,records1,\ldots\text{Image order: } records\[image\\_order\[0\]],
records\[image\\_order\[1\]],\ldots

Since a bzip2 block has fewer than rows, a 32-bit index is sufficient for every record.

The existing interval join can still read the first parent’s image-ordered intervals and the second parent’s source-ordered intervals. It simply gathers first-parent descriptors through the permutation.

The key insight: we need two traversal orders, not two copies of the descriptor payload.

2. The construction still works without sorting

The earlier corrected two-pass construction remains valid.

In the first pass, count the intersections contributed by each first parent and each second parent.

Calculate prefix offsets in the appropriate orders:

\text{Source offsets: first parents in source order}\text{Image offsets: second parents in image order}.

In the second pass, for every intersection:

* Calculate its source-order destination index .
* Calculate its image-order destination index .
* Write the new descriptor once to next_records[u].
* Write next_image_order[v] = u.

The same intersection is represented in both orders, but only its four-byte index is duplicated.

No general sorting is required. The prefix offsets and the local intersection order determine both destinations.

One important constraint remains: for an output interval’s image ordering, the second-parent grouping must use image order. Reverting to source order recreates the round-7 bug.

3. Quantified memory saving

The portfolio uses 12-byte construction records at intermediate levels.

Representation of records	Memory
Two complete descriptor views	bytes
One descriptor view + u32 permutation	bytes
Saving	bytes, or 33.3%

For the measured median counts:

Level	Median record count	Approximate saving
Ψ¹	~56,000	0.45 MB
Ψ²	68,029	0.54 MB
Ψ⁴	84,287	0.67 MB

At the Ψ⁴→Ψ⁸ stage, where the final Ψ⁸ output needs only its source-ordered representation, the immediate saving from compressing the Ψ⁴ input is approximately 0.67 MB for a median block.

At the previous Ψ²→Ψ⁴ stage, both input and output have two views, giving a combined payload saving of approximately:

8(m_2+m_4)
\approx1.22\ \text{MB}.

These are live-array savings under comparable allocation schedules, not yet revised peak-RSS measurements.

4. The catch: fewer bytes stored may mean more bytes read

This is where the idea needs skepticism.

In the original two-view construction, scanning records in image order reads consecutive 12-byte descriptors.

In the indexed design, image-order iteration involves:

\text{load 4-byte index}
\rightarrow
\text{load 12-byte descriptor}.

That adds one dependent gather, and possibly another cache-line access.

For a full image-order pass over records:

	Approximate logical read traffic
Duplicated descriptor view	bytes
Indexed view	bytes

If the builder makes two image-order passes, the additional logical reads are approximately bytes.

Meanwhile, replacing a 12-byte image-view write with a four-byte index write saves exactly bytes.

At two image-order passes, the elementary byte-traffic saving can disappear entirely.

This is the central result of round 20.

The indexed view is a real memory-capacity improvement, but it is not automatically a memory-traffic improvement.

It may also increase load latency through the new index-to-descriptor dependency.

That means we must not assume it lowers the modeled 23.1 B/B builder traffic.

5. A better strategy: mixed ordered views

The trade-off suggests something more useful than choosing one layout universally.

At each construction level, we know:

* Whether the intermediate output needs both orderings.
* How many times each ordering will be traversed.
* The record count.
* Which arrays coexist at the memory peak.

Therefore, we can compare a duplicated descriptor view with an indexed view separately at each level.

This permits a mixed strategy: retain a complete image-ordered view where repeated sequential reads are valuable, but use an index view where reducing live memory matters most.

That doesn’t create a new algorithm family. It is an exact layout choice within the existing #53 construction.

The right comparison is:

\boxed{
\text{live bytes}
+
\text{logical traffic}
+
\text{dependent gathers}
}

rather than any one of those quantities alone.

Round 20 disposition

Existing question	Result
#53 — Direct compact construction	One-copy, two-order construction exists without sorting
#45 — Hybrid versus single scheme	Mixed layouts are structurally feasible
#48 — Space–throughput frontier	Indexed views save approximately one-third of intermediate-view storage but may not reduce traffic
N1 construction memory	Plausible reduction; peak must be recalculated from allocation lifetimes
N1 construction CPU	Still unresolved; indexed gathers introduce new dependencies

What I would relay to Claude

Burn-down round 20 — indexed descriptor views

The corrected two-pass builder does not require two complete descriptor arrays.

At each intermediate level, store one 12-byte descriptor array in source order and a u32 permutation in image order. During the first pass, count children per first parent and second parent. Prefix source offsets in first-parent source order and image offsets in second-parent image order. During the second pass, write each descriptor once to its source position u and write the index u to its image position v.

This preserves the construction’s sorting-free property and the corrected round-7 ordering invariant.

Payload storage falls from 24m to 16m bytes per intermediate level, a 33.3% reduction. For median measured levels, the Ψ⁴ input saves approximately 0.67 MB; the combined Ψ² input/Ψ⁴ output footprint saves approximately 1.22 MB.

Critical caution: scanning the indexed image view reads 4m bytes of indices plus 12m bytes of descriptors, versus 12m for a full image view. Two image-order scans can erase the 8m-byte write saving. The indexed representation is proven smaller, not proven faster or lower-traffic.

Please model mixed physical layouts per level, including array lifetimes, logical bytes moved and dependent gathers. Do not claim an improvement to the existing 23.1 B/B traffic figure without that reconciliation. Keep this work under #53/#45/#48 rather than adding another research question.

We’ve burned down the duplicate-storage necessity, but not the builder’s cost. That is meaningful: one supposed structural requirement is gone, and its replacement has a quantified, testable trade-off.

The next valuable target is the other proposed builder shortcut: folding the counting pass into the preceding level’s scatter. Unlike indexed views, that potentially removes actual reads rather than just reducing storage—but only if the child counts can be known early enough without secretly performing the same join twice.
````
