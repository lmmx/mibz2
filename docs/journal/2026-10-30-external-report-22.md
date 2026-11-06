# External research report 22: research burn-down rounds 21–22, P74's distinction from #25, layout C's hurdle, the counting gate and Gate B (verbatim)

A research input, not a finding of this repository. The maintainer relayed it on
2026-10-30, after external report 21 (`2026-10-30-external-report-21.md`). It has two
parts, in the order received.

1. **Round 21:**
   - P74 against the shipped `mtf_avx2` and the rejected #25. #25 held 16 entries
     (84.8% of explicit tokens on the latest census); P74 holds 32 (98.4%) and removes
     the per-token load and store.
   - The algebra of a register-held 32-byte update.
   - The structural test to apply: the AVX2 critical path across the 128-bit lanes.
   - Layout C's screening inequality, w/s > 3.3 + 0.7·g/s.
2. **Round 22:**
   - **The counting gate for layout C:** strict-interior counts, Σe_i = Σf_j = m_2a,
     and output equality with the two-pass builder.
   - **The cap subtlety:** the counts are over the uncapped partition.
   - **A recommendation** that a tightly bounded Gate B experiment be put to the
     maintainer, with the i9 unlaunched unless approved.

**The coordinator's checks** are in `2026-10-30-research-portfolio.md` §3.1(k) and §3.2d:
- **The counting gate:** passes on 8,160 random doubling levels.
- **P74's register-held update:** three instructions, `vperm2i128`, two `vpshufb` and
  `vpblendvb`, with per-rank index and select vectors drawn from a 2 KB table off the
  dependency chain. Exact on 20,000 random states for every rank 0–31.

This is source material. Its claims are checked there, not taken as findings.

The text reached us as rendered text (some formula symbols and table cells lost) and is
kept as received.

## 1. Round 21, as received

````text
Burn-down round 21: P74 survives, but we can now define exactly what would make it different from #25

The three completed analyses give us three firm conclusions: #74 is answered negatively, layout C is the least traffic-heavy modeled N1 builder so far, and N2/N2b fail their pre-registered criteria.

I inspected the shipped src/mtf.rs and the historical #25 record. That reveals a specific distinction worth preserving.

1. The shipped MTF still round-trips its head through memory

For an explicit MTF rank below 32, the current AVX2 loop:

* Loads the selected value v from the list in memory.
* Loads the current 32-byte head.
* Loads the preceding 32-byte window, shifted by one byte.
* Blends the two vectors.
* Stores the updated vector and then writes the new front byte.

The source also explicitly documents a store-forwarding dependency between updates.

So the surviving P74 hypothesis should not simply be “use AVX2 for small ranks.” That’s already shipped.

The useful distinction is:

Keep the 32-byte MTF head live in a SIMD register across consecutive explicit tokens, rather than reloading and storing it for every move.

The new census says that 98.4% of explicit tokens have rank at most 31. This makes that representation much more credible than it would be on the records and random controls.

But this is still not a demonstrated optimization.

2. How it differs from the rejected #25

The historical #25 candidate kept the MTF head in two registers, covering the first 16 entries. Its recorded result was approximately −1.6% at one thread, +1.0% at four, and an 11% regression on random data.

P74’s potential distinction is therefore not register residency alone. #25 already tested that principle.

The actual differences would be:

Dimension	Rejected #25	Surviving P74 hypothesis
Register-held prefix	16 entries	32 entries
Supported hot ranks	First 16	First 32
Wikidata explicit-token coverage	~84.8% from latest census	98.4%
High-rank handling	Existing experimental fallback	Must preserve an efficient fallback
Dominant proposed saving	Register-held short moves	Remove current 32-byte load/store dependency across tokens
Performance evidence	Measured negative/mixed	None yet

The extra 13.6 percentage points of rank coverage help. But the more important proposed saving is removing the specific memory dependency present in today’s implementation.

One essential test remains at the algorithmic level: can we perform rank extraction, insertion at the front and shifting across the 128-bit halves of a 256-bit register without replacing the memory dependency with an equally expensive chain of SIMD permutations?

That’s where P74 could fail before an implementation is justified.

3. A complete register-held MTF update is possible

Let contain the first 32 MTF symbols in order, with a separate array holding the remaining 224.

For a rank , define:

v=Vq.

The new head is exactly

V’0=v,V’i=Vi-1\quad(1\le i\le q),V’i=Vi\quad(q<i<32).

The tail is unchanged.

This is an exact 32-byte register-local state transition. It requires a variable-index extraction and a masked shift, but no access to the 224-byte tail.

For , let be the selected tail element. The new state is:

V’=\[v,V\[0\],V1,\ldots,V30]

and the tail becomes the previous tail shifted one position up to the selected element, with the former inserted at its beginning.

This establishes that register residency can be maintained correctly across both paths.

The implementation must still handle the AVX2 128-bit lane boundary and extract the selected byte efficiently. On AVX2, there is no single general 32-byte variable-index byte permutation. That makes instruction selection central to the outcome.

Research disposition: P74 is algebraically feasible and distinct from the shipped implementation at the memory-dependency level. It is not yet proven distinct enough from #25’s failed cost structure to justify a performance claim.

4. Layout C: the exact cost disadvantage we must overcome

The builder census gives:

Median modeled construction traffic

T-vector

5.0 B/B

Layout A/B

23.1 B/B

Layout C

17.0 B/B

These are logical byte-traffic models, not measured memory bandwidth or CPU time.

Layout C saves 26.4% against A/B, but still moves 3.4 times as many bytes as the T-vector builder.

Of its 17 B/B, approximately 2.8 B/B are indexed gathers. That’s about 2.52 MB of gather traffic per 900 KB block.

We can formulate a break-even condition without guessing machine latencies.

Let:

* : effective time cost per sequential byte.
* : effective time cost per indexed-gather byte.
* : effective time cost per scattered-write byte.

Use the crude decomposition of layout C into 14.2 B/B sequential traffic and 2.8 B/B gathers, and the T-vector into one sequential-read byte plus four scattered-write bytes.

Ignoring arithmetic and overlap, layout C’s traffic cost is lower only if

14.2s+2.8g<s+4w.

Therefore,

\boxed{\frac ws>3.3+0.7\frac gs.}

This is a screening inequality, not a cycle prediction. Real memory operations overlap and have cache-line granularity.

It does, however, tell us something concrete: if gathered bytes are appreciably more expensive than sequential bytes, the T-vector’s scattered writes must be substantially more expensive to compensate.

And construction arithmetic is additional work that this inequality hasn’t charged to layout C.

This is why we should not infer that 17 B/B is close to competitive merely because it’s below 23 B/B.

5. The four-core constraint makes layout C’s tail important

Under the current 4.5 MB peak-build-memory cap:

Graded representation choices, 233 Wikidata blocks

This is more conservative than the resident-table selector, which chose Ψ⁸ for all 233.

Layout C’s median peak is 4.2 MB, but its 90th percentile remains 7.2 MB and its worst observed case 10.1 MB.

Consequently, lower traffic does not eliminate the need for a peak-memory-aware fallback.

One design implication is that the format selector should not be tied to one construction layout. The best combination of shortcut length and construction layout can depend on the block’s structural counts. But choosing by predicted CPU cost requires actual hardware evidence; byte totals alone cannot provide it.

6. Research conclusions from the completed analyses

Research item	Status	What remains
#74 lane bunching	Answered negatively on 12 blocks with a faithful scheduler model	Nothing further for the structural question
#53 builder representation	Layout C reduces modeled traffic by 26%	Dependent gathers, intersection arithmetic and CPU cost
#45/#48 adaptive representation	Peak-constrained selection demonstrated	CPU-optimal choice
N2	Stopped under memoization uniqueness rule	No further work on that route
N2b	Stopped at 7.1% batch coverage	No further work on that route
P74	Register-held 32-byte state transition feasible	SIMD critical path, fallback cost, comparison with #25

The next burn-down target

P74 has one remaining structural test that could settle whether it is worth keeping alive: construct the shortest plausible AVX2 dependency graph for a rank-0–31 transition, including variable-byte extraction, shift across the 128-bit lane boundary, insertion, and output-symbol recovery.

Compare that graph directly with the shipped load/blend/store sequence and the historical #25 kernel.

If P74 merely substitutes several serialized shuffles for the current store-forwarding delay, its promise may disappear. If it removes the memory dependency with a short register-only chain, it remains a credible candidate—subject to actual timing and the negative controls.

For N1, layout C now has an explicit economic hurdle. It would need to outperform a very efficient five-byte-per-output-byte T-vector construction despite 17 modeled bytes, indexed gathers and additional interval arithmetic.

No implementation or i9 experiment is implied. We’ve closed #74, rejected N2 and N2b on evidence, and narrowed both surviving avenues to specific CPU-cost questions rather than broad speculation.
````

## 2. Round 22, as received

````text
The new results settle the builder-layout question much more decisively. The counting pass was avoidable; the two-view storage penalty was manageable; neither was the fundamental obstacle.

The best modeled compromise is now:

* 17.5 B/B construction traffic, down from 23.1 B/B.
* 1.1 B/B of indexed gathers.
* 4.23 MB median peak construction memory.
* A peak-aware fallback for blocks that exceed the 4.5 MB reference.

That is genuine progress, but it still isn’t evidence of lower CPU cost than the shipped 5 B/B T-vector builder.

I agree with Claude that this particular research branch has reached a decision boundary. We should not commission more byte-traffic variants merely to avoid measuring cycles.

However, before recommending Gate B, there’s one mathematical condition in the new bitvector-counting method that deserves an explicit correctness check.

Round 22 — verify the counting-pass elimination

Let be a level- source interval and let

J_i=\Psi^a(I_i)

be its image interval.

The number of children generated from first parent is

\boxed{
e_i=1+
\#\{x:\ x\text{ starts a level-}a\text{ source interval},\
\min J_i<x<\max J_i\}.
}

The important word is strictly.

A source-interval start coinciding with the left endpoint of does not create another child. Nor does a start at its exclusive right endpoint.

Likewise, for each second-parent image interval , its child count is one plus the number of first-parent image starts strictly inside .

That gives two exact count arrays from the two start bitvectors, with no interval join needed.

There is one subtlety: these are counts over the uncapped composition partition. If construction-time segment caps introduce additional record boundaries, those must be accounted for separately or incorporated into the appropriate boundary bitvectors. Otherwise, the count arrays could be correct for Ψ⁸ composition but insufficient for the final capped representation.

What I would require before marking the builder construction completely settled

A deterministic assertion for each doubling level:

\sum_i e_i=m_{2a}

and

\sum_j f_j=m_{2a}.

The first counts must come from the second partition’s source-start bitmap within first-parent image intervals. The second counts must come from the first partition’s image-start bitmap within second-parent image intervals.

Then verify that the resulting source- and image-ordered outputs match the original two-pass intersection builder, including exact interval endpoints, displacement and output words.

This is a correctness gate, not another performance experiment.

If those assertions already pass in Claude’s checker, I would mark the no-second-join counting construction resolved.

Should we authorize Gate B now?

Yes, as a research decision—but not an unrestricted implementation campaign.

There is no longer a productive reason to keep exploring construction byte models before answering the CPU question. Layout C already provides a concrete best-known candidate, and we’ve measured the distributions needed to evaluate it.

I would define the next experiment narrowly: compare the shipped T-vector builder with the corrected bitvector-counting builder, using matched input blocks and the same CPU. Record builder cycles, memory traffic counters where available, logical traffic, allocation peaks and parity. Include the two-view construction as a diagnostic reference, but don’t turn every possible view mixture into a separate optimization project.

The i9 should remain unlaunched until you explicitly approve it. A smaller local or CI study can screen the construction mechanics without prejudging laptop performance.

The outcome must be allowed to be negative. If the best layout still loses materially in construction cycles, that substantially lowers N1’s expected full-decoder benefit and could close this branch without building a complete decoder.

The broader 84-question burn-down

We also have an important inventory milestone: according to Claude’s reconciled ledger, every original row now has a disposition or a specifically assigned pending primary-source check.

The final independent literature review still matters. Once #28–#33, #36 and #73 are reconciled, the research inventory will be classified, but that does not mean every conditionally unresolved performance question has been experimentally answered.

I’d keep these two milestones separate:

Inventory completion: every question has an evidence-grounded disposition.

Research completion: every decision-relevant uncertainty has either been answered or demonstrated not to affect the choice.

We’re close to the first. The builder’s cycle cost is one of the most important remaining uncertainties for the second.

My conclusion: close the bitvector construction mathematics, stop further layout speculation, and put a tightly bounded Gate B experiment up for your decision—while continuing the remaining literature and CPU questions.
````
