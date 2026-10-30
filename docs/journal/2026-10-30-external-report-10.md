# External research report 10: the researcher's review of PR #117 and research burn-down rounds 1–2 (verbatim)

A research input, not a finding of this repository. The maintainer relayed it on
2026-10-30. The researcher had studied PR #117 directly, and the maintainer asked the
researcher and the coordinator to work together from here. It has three parts, in the
order received.

1. **The review of PR #117.** It gives a counterexample to the portfolio's §3.1(a)
   identity. For the cyclic BWT of `aabc`, the union of preimages is {0, 1, 2}, while
   Ψ²'s own breakpoints are {1, 2}. Ψ² is a translation on rows 0–1, yet their two-byte
   words differ. The researcher's fix is to keep the union as a safe refinement C_k. It
   also gives recommendations on the open decisions.
2. **Burn-down round 1.**
   - Question #25 / contradiction K2 is resolved: Gagie's "Reportable Shortcuts with
     RLBWTs" exists, and is search-oriented.
   - A storage lower bound: R_k ≥ d_k, so the payload is ≥ (4 + k)·d_k.
   - Its limit at k = 2.
   - The fragmentation factor R_k / d_k.
   - The 47 open questions grouped into research packages.
3. **Burn-down round 2.**
   - Row-space concentration of breaks, from the register's aggregates: at least about
     4.4 breaks per impure 32-row group.
   - **The traversal-gap identity:** |C_k| = Σ_i min(k, g_i) over the gaps between
     successive breakpoints along Ψ, with its first and second differences.
   - The correction that the four-step storage budget holds even in the worst case
     (k|B|).
   - A simpler T1, built on the gap histogram.

The coordinator's checks are in `2026-10-30-research-portfolio.md`, §0 and §3.1:
- the counterexample is confirmed by brute force;
- the union C_k is shown to equal exactly the minimal record partition;
- the gap identity is confirmed on real blocks;
- the T1 census is run on the bench's blocks.

These are source material. Claims are checked there, not taken as findings.

The text reached us as rendered text. Its citation markers are kept as received: the
`[image](…)` favicon links, the bare site names after them, the `&#x20;` entities and the
`utm_source` parameters.

## 1. The review of PR #117, as received

````text
Claude has delivered a substantial piece of work in [mibz2 PR #117](https://github.com/lmmx/mibz2/pull/117?utm_source=chatgpt.com). I've read the portfolio document directly. The historical reconciliations, experimental boundaries and research queue are much more rigorous than the earlier proposals.

However, I found a mathematical error in the portfolio's highest-ranked breakthrough candidate, N1, and I'd correct it before approving that candidate's proposed analysis.

This doesn't kill the idea. It changes the construction that needs to be investigated.

## A counterexample to the exact breakpoint identity

The portfolio claims:

\\[ B_k=\bigcup\_{t=0}^{k-1}\Psi^{-t}(B). \\]

That equality is not generally true, even for a valid cyclic BWT.

Consider the string `aabc`. Its cyclic rotations give:

\\[ F=[a,a,b,c],\qquad \Psi=[1,2,3,0]. \\]

Including both Ψ translation breaks and F-symbol boundaries, the initial breakpoint set is:

\\[ B=\\{1,2\\}. \\]

For \\(k=2\\):

\\[ \Psi^2=[2,3,0,1], \\]

whose breakpoints, including F-symbol boundaries, are still:

\\[ B_2=\\{1,2\\}. \\]

But the proposed union is:

\\[ B\cup\Psi^{-1}(B)=\\{0,1,2\\}. \\]

So the asserted equality fails.

More importantly, the two-byte output words for rows 0 and 1 differ:

\\[ (F[0],F[\Psi(0)])=(a,a), \\]

\\[ (F[1],F[\Psi(1)])=(a,b). \\]

Yet those rows lie in the same translation interval of Ψ².

Therefore:

An interval on which Ψ² is a translation does not necessarily have a constant two-byte output word.

That contradicts a central assumption in §3.1(c) of the portfolio.

The solution is to maintain a refined partition that tracks both intermediate transitions and output-symbol boundaries. The union construction may be useful as a conservative refinement, but its size cannot simply be identified with the number of maximal Ψᵏ translation intervals.

## What I would change in PR #117

I'd retain N1 as an open structural hypothesis, but make its mathematical foundation more precise.

Define a safe multi-step partition \\(C_k\\) by:

\\[ C_k=\bigcup\_{t=0}^{k-1}\Psi^{-t}(B), \\]

where \\(B\\) includes one-step translation discontinuities and F-symbol boundaries.

This is a conservative refinement that can guarantee compatible intermediate transitions and constant output words. It need not be the minimal partition of Ψᵏ itself.

That distinction has practical consequences:

- The proposed \\(k|B|\\) upper bound remains useful for the safe refinement.
- The census should count the refined records required for correct output, not merely Ψᵏ translation runs.
- The construction-cost model must use those record counts.
- The claimed free byte recovery is conditional on the refined partition, not on Ψᵏ translation intervals alone.
- The rank-indexed lookup design still deserves investigation, but its projected instruction and construction costs remain hypotheses.

### My recommendations on Claude's proposed decisions

| Decision                         | Recommendation                                                |
| -------------------------------- | ------------------------------------------------------------- |
| T1: Ψ composition census         | Approve after correcting the partition definition and proof   |
| T2: MTF token census             | Approve                                                       |
| T3: Existing occupancy artifacts | Approve                                                       |
| T5: Scanner-marker statistics    | Approve                                                       |
| P1: Decoder-free application arm | Worth approving separately as a measurement-only change       |
| #93 superseded by #114           | Close as superseded, preserving the historical record         |
| i9 experiment queue              | Keep unlaunched pending the results of the read-only analyses |

I would also ask Claude to distinguish the maximum possible saving from the cost of realizing it. N1's projected 10–18.5 instructions per byte is an interesting model, but it isn't yet evidence of a work-reducing implementation.

The four-core CPU target remains the right one. None of this suggests pursuing GPU execution.

## The message I'd give Claude

The portfolio is ready for review, but §3.1 contains a mathematical error that should be corrected before T1.

The asserted identity \\(B_k=\bigcup\_{t\<k}\Psi^{-t}(B)\\), where \\(B_k\\) denotes the actual translation and F-bucket breakpoints of Ψᵏ, is false. The valid cyclic BWT of `aabc` provides a counterexample: \\(F=[a,a,b,c]\\), \\(\Psi=[1,2,3,0]\\), and for \\(k=2\\), the actual \\(B_2=\\{1,2\\}\\), while the proposed union is \\(\\{0,1,2\\}\\).

Furthermore, Ψ² is a translation on rows 0 and 1, but their two-byte output words are `(a,a)` and `(a,b)`. Thus constant output words do not follow from the maximal translation intervals of Ψᵏ.

Please preserve the union as a conservative safe refinement \\(C_k\\), correct the proof and terminology, and make T1 count the records actually required to recover all output bytes correctly. Revisit the record-size and construction-cost models accordingly. This is a mathematical correction, not a rejection of N1.

Once corrected, I support the read-only T1, T2, T3 and T5 analyses. I would keep new decoder implementations and i9 experiments on hold pending those results.

The broader news is encouraging: 165 recorded ideas, 84 research questions, and 63 historical contradictions constitute a serious research foundation.

But today's counterexample is exactly why we needed a review before implementation. Our leading candidate had an appealing theorem whose stronger form was incorrect.

The good news is that a useful weaker statement survives, and it still supports a potentially interesting multi-byte navigation structure.

I would correct the theorem, run the censuses, and let the actual structure of Wikidata determine whether N1 deserves its top ranking.

That's how we pursue a breakthrough without talking ourselves into one.
````

## 2. Burn-down round 1, as received

````text
I've begun working through the 84 recorded questions in PR #117. The first useful result is that we can reduce several seemingly independent questions to one structural measurement, and another historical question can already be answered from a primary source.

## First closure: the supposedly missing 2026 RLBWT paper exists

The register's question #25 and contradiction K2 concern Travis Gagie's Reportable Shortcuts with RLBWTs, arXiv:2609.06596. An earlier literature note said the title could not be located.

The paper exists, with a revised version dated 12 October 2026. Its mechanism is to precompute shortcuts for recognizable pairs of strings, allowing an RLBWT backward search to skip matching stretches. The reported speedups are for pattern searches, not decoding a bzip2 block.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://borowiecki.dev\&sz=32)

borowiecki.dev



That gives us a concrete disposition:

- K2: resolved—the citation is genuine.
- Question #25: the bibliographic uncertainty is resolved, but whether the mechanism transfers economically to inversion remains open.
- Engineering priority: low until there is a construction that avoids the paper's reference-index preprocessing and recognition overhead.

No new hypothesis needed.

## Second result: a lower bound for the proposed multi-byte Ψ records

The portfolio's leading candidate, N1, proposes interval records containing a \\(k\\)-step successor and the \\(k\\) emitted bytes.

Let \\(d_k\\) be the number of distinct cyclic \\(k\\)-byte strings represented among the block's BWT rows.

Any partition whose records each have a single constant \\(k\\)-byte output word must have at least \\(d_k\\) records:

\\[ \boxed{R_k\ge d_k.} \\]

This remains true regardless of how cleverly the interval boundaries are selected.

For the particular proposed representation—four bytes of displacement and \\(k\\) bytes of output per record—we obtain:

\\[ \boxed{\text{record payload}\ge (4+k)d_k\text{ bytes}.} \\]

That excludes the rank directory and any additional metadata.

For a 900,000-byte block, this yields an immediate space-only test against the 3.6 MB direct T-vector:

| \\(k\\) | If distinct \\(k\\)-grams exceed… | Records alone exceed 3.6 MB |
| ------- | --------------------------------- | --------------------------- |
| 2       | 600,000                           | Yes                         |
| 4       | 450,000                           | Yes                         |
| 8       | 300,000                           | Yes                         |

These are necessary space conditions for that specific fixed-record design, not performance thresholds or impossibility results for other encodings. A different format could compress the words, displacements or records.

But this is useful because the corpus can establish \\(d_k\\) without constructing any Ψ powers.

The resulting research order is clearer: test the representation's information budget before investing in a sophisticated composition census.

## Third result: we can sharpen the N1 feasibility test further

There is an important asymmetry between \\(k=2\\) and larger shortcuts.

With a byte alphabet, the number of possible distinct two-byte strings is at most

\\[ 256^2=65{,}536. \\]

Therefore the \\(k=2\\) lower bound cannot reach the 600,000-record threshold above. Distinct bigrams alone cannot falsify Ψ² on storage grounds. The actual interval structure is what matters there.

For \\(k=4\\) and \\(k=8\\), however, distinct output words could be numerous enough to make a fixed-record representation uneconomical.

This yields a better decision sequence for N1:

1. Measure distinct cyclic \\(k\\)-grams for \\(k=2,3,4,8\\), along with the actual safe interval counts.
2. Compare the records required by the safe partition with the theoretical minimum imposed by distinct output words.
3. Quantify the excess records caused by transition-boundary fragmentation.
4. Only then evaluate descriptor lookup and construction costs.

The excess is scientifically interesting:

\\[ \boxed{\text{fragmentation factor}=\frac{R_k}{d_k}}. \\]

It measures how much more complex the interval representation is than a hypothetical representation needing just one record per distinct output word.

It doesn't by itself predict runtime, but it separates output diversity from navigation discontinuity—two different reasons a representation might become expensive.

## Fourth result: a correction to the portfolio's mathematical framing

The portfolio currently uses the exact identity

\\[ B_k=\bigcup\_{t=0}^{k-1}\Psi^{-t}(B). \\]

As I noted in the earlier review, this is incorrect if \\(B_k\\) means the actual discontinuities of the composed map. Intermediate discontinuities may cancel under composition.

The union can instead be retained as a conservative partition used to ensure that each intermediate mapping and output symbol behaves predictably.

That means T1 should compute at least two separate quantities:

- Minimal composed-map complexity: the actual translation intervals of \\(\Psi^k\\).
- Safe output-producing complexity: the refined intervals needed to recover the complete \\(k\\)-byte output word.

The difference tells us how much representation complexity is introduced by requiring correctness for every emitted byte, rather than only the final successor.

This is not just a proof correction. It could materially alter N1's estimated memory and construction costs.

## How I'd burn down the remaining 47 open questions

The inventory contains numerous questions that share the same underlying uncertainty. I would organize work around those dependencies rather than process the rows sequentially.

| Research package                        | Existing question numbers             | What would settle them                                                                    |
| --------------------------------------- | ------------------------------------- | ----------------------------------------------------------------------------------------- |
| Multi-step Ψ structure                  | #37, #38, #44, #76, plus portfolio N1 | Distinct \\(k\\)-grams, safe interval counts, record budget and dependency model          |
| Exceptional navigation costs            | #43, #45, #51, #71–#75                | One defined family of fixed-line layouts, disassembly and dependent-load cost measurement |
| Run and discontinuity statistics        | #63–#69                               | Joint corpus census of runs, boundaries, alignment, segmentation and slow-path clustering |
| Construction and front-end work         | #11, #53, plus P74                    | Existing fusion audit, explicit MTF-token census, read-only construction-cost model       |
| CPU memory-level parallelism            | #2–#4, #47–#48, #84                   | Compatible counter evidence tied to the same kernel and hardware                          |
| Architecture portability and contention | #77–#82                               | Existing experimental records first; new matched runs only where necessary                |
| Literature housekeeping                 | #25, #28–#33, #36                     | Primary-source verification and identification of any genuinely untested mechanisms       |

These are packages of existing questions, not newly invented research projects.

I would also avoid assigning every open question equal urgency. Questions about old citation discrepancies should be settled quickly where possible, but they should not displace work on the algorithms with the greatest potential to change decoding cost.

## My revised priorities

1\. T1 — Corrected multi-step Ψ feasibility

Highest scientific value

Resolve whether the proposed output-producing interval representation can fit and be constructed economically. This is the decisive uncertainty behind N1 and N3.

2\. T2 — Actual MTF token economics

The register already establishes that explicit rank-zero tokens do not occur in the worker's MTF stream and that the current loop has a store-forwarding dependency. Measure eligible batch coverage and how much of the existing CPU cost is actually removable. This narrows N2, N2b and P74 together.

3\. Structural clustering census

Use the existing #99 census data to examine where exceptional navigation steps cluster. This could decide several compact-Ψ questions simultaneously and tell us whether the two-tier design needs a staged scheduler at all.

4\. Historical and literature resolution

Settle low-cost bibliographic discrepancies, retain architecture-specific negatives, and remove questions whose answer is already present in the repository.

### What we can claim at this point

I've examined the actual 84-question inventory and the corresponding portfolio, verified one unresolved citation, derived a representation-specific storage lower bound, identified its limitation for bigrams, and sharpened the key mathematical distinction between composed navigation and correct multi-byte output.

I have not run the corpus censuses, inspected hardware counters or produced a measured speedup. So I wouldn't mark any of the principal algorithmic questions resolved yet.

But we are getting closer to doing what you asked: reducing a long list of unresolved questions into a smaller number of decisive, evidence-producing investigations.

The next meaningful milestone is to determine whether N1 has a viable representation budget at all. If it doesn't, we should be willing to remove it from the top engineering rank, rather than spend weeks optimizing a beautiful mathematical construction that cannot beat the existing decoder.

And if it does, we will have a much stronger reason to investigate its lookup and construction algorithms.

The objective is to make the register shrink as our understanding grows—not reward ourselves for making it longer.
````

The maintainer added: "I want you two to work together now".

## 3. Burn-down round 2, as received

````text
## Research burn-down, round 2

A strong result emerges from combining two measurements already in the register. It addresses questions #63–#66 about run structure and clustering.

The portfolio records approximately 56,000 Ψ translation runs in a full-size Wikidata block, while roughly 77.4% of aligned 32-row groups are pure translations.

Those numbers imply something stronger than simply saying the data is repetitive:

The translation discontinuities are highly concentrated in a minority of row groups.

For 900,000 rows, there are approximately 28,125 aligned 32-row groups. If 77.4% are pure, only about 6,356 groups contain an internal translation discontinuity.

Now consider the roughly 56,000 translation runs, implying approximately 56,000 discontinuities across the block. Some can fall exactly between groups, but there are at most 28,124 such boundaries.

Consequently, even under the most conservative assignment, at least about 27,900 discontinuities must lie inside the 6,356 impure groups.

That gives:

\\[ \boxed{\text{At least about }4.4\text{ internal breaks per impure 32-row group.}} \\]

If very few breaks lie exactly on group boundaries, the mean is closer to 8.8.

This is a mathematical implication of the reported aggregate figures, subject to those figures describing comparable blocks. It does not require a new benchmark.

Why does this matter? A random placement of approximately 56,000 boundaries among 900,000 positions would produce a very different distribution: only around 14% of 32-row groups would be expected to contain no internal breakpoint.

The reported figure is about 77%.

So a uniform-breakpoint model is fundamentally wrong for this workload.

This strengthens the case for concentrating representation complexity in exceptional groups rather than spending equal resources on every group.

It also narrows the research questions:

- \#63: average run length alone cannot determine fixed-group purity.
- \#65: alignment may matter, but cannot by itself explain the observed concentration.
- \#66: strong clustering is implied by the aggregate counts; what remains is its distribution across blocks and whether that clustering is exploitable cheaply.

It does not establish that an exceptional-group decoder would be fast. That is precisely what #112 cautions against.

## Fifth result: the entire Ψ-composition census reduces to a gap distribution

This is the strongest deduction of this round.

Let \\(B\\) be the safe one-step breakpoint set, including the boundaries necessary for output-symbol recovery, and define

\\[ C_k=\bigcup\_{t=0}^{k-1}\Psi^{-t}(B). \\]

The set \\(C_k\\) is the conservative refinement we need for the proposed multi-byte records.

Now follow Ψ in traversal order, rather than row order.

Mark every visited row belonging to \\(B\\). Let

\\[ g_1,g_2,\ldots,g_m \\]

be the cyclic distances between successive marked rows, where \\(m=|B|\\).

For a single-cycle permutation,

\\[ \sum\_{i=1}^{m}g_i=n. \\]

The exact size of the safe refinement is:

\\[ \boxed{|C_k|=\sum\_{i=1}^{m}\min(k,g_i).} \\]

### Why this is true

Each breakpoint contributes itself and its \\(k-1\\) predecessors along the Ψ cycle.

If consecutive breakpoints are separated by \\(g_i\\) steps, the predecessor coverage inside that gap has length \\(\min(k,g_i)\\).

Those gap contributions are disjoint. Summing them counts the union exactly.

No probabilistic assumption is involved.

For multiple Ψ cycles, the same identity holds by summing the gaps separately around every cycle that contains breakpoints.

This yields several immediate consequences.

First: the entire \\(k=1\ldots16\\) record-count curve can be calculated from a single histogram of distances between breakpoint encounters during traversal.

Second: the overlap that could make multi-step records economical is controlled by breakpoint clustering in Ψ traversal order, not merely clustering among adjacent BWT rows.

That distinction is critical.

Our earlier 77.4% pure-group result proves substantial clustering in row space. It tells us essentially nothing about how close those breakpoints are along the decoding trajectory.

Those are two different structures.

Third: we can measure the marginal cost of one more shortcut step exactly:

\\[ \boxed{ |C\_{k+1}|-|C_k| = \\#\\{i:g_i>k\\}. } \\]

In words: adding another step creates one new breakpoint per traversal gap longer than the current shortcut length.

This tells us precisely when the record count stops growing rapidly.

Fourth: the second difference is:

\\[ \boxed{ |C\_{k+2}|-2|C\_{k+1}|+|C_k| = -\\#\\{i:g_i=k+1\\}. } \\]

So the safe-partition growth curve is discretely concave. Every drop in its slope identifies the number of traversal gaps of a particular length.

This is stronger than a worst-case \\(k|B|\\) bound. It explains exactly what makes the bound tight or loose.

### What this resolves

The portfolio's composition question can now be reformulated without adding a new research item:

> What is the distribution of distances between one-step breakpoints along Ψ?

That single distribution determines the safe-refinement record count at every shortcut length, subject to the chosen breakpoint definition.

It does not determine construction time, memory latency, output-word compressibility or the number of lookups actually executed by the decoder. Those remain separate issues.

But we no longer need to speculate about breakpoint-preimage overlap.

Its combinatorics are settled.

## Sixth result: a correction to the previous storage argument

The gap identity lets us rule out one of my own earlier feasibility concerns.

For an approximately 900,000-row block with \\(m\approx56{,}000\\) safe one-step breakpoints:

\\[ |C_4|\le4m\approx224{,}000. \\]

With eight-byte records, the payload is at most approximately 1.79 MB.

Adding the portfolio's roughly 225 KB rank directory brings the proposed structure to about 2.02 MB, below the existing 3.6 MB T-vector.

Thus, for blocks with that breakpoint count, the proposed four-step representation does not fail the simple storage budget even in the worst case.

My earlier threshold of 450,000 distinct four-grams was mathematically valid as a necessary condition, but it is unreachable under this breakpoint bound. It therefore provides no useful rejection test for these particular blocks.

That correction matters. We shouldn't let a formally valid but vacuous bound drive research priorities.

For \\(k=8\\), the situation is different.

The nominal record budget is 12 bytes per interval. To remain below approximately 3.6 MB, including the rank directory, we need:

\\[ |C_8|\lesssim281{,}000. \\]

Yet the worst-case bound is around 448,000.

Therefore eight-step records require substantial overlap between breakpoint preimages to stay within that storage target. The new gap identity tells us exactly how to measure that overlap.

### A direct engineering implication

For the 56,000-breakpoint regime:

- Ψ²: storage looks feasible under this representation, but the existing pair-walk experiments show that feasible storage is not sufficient for speed.
- Ψ⁴: storage also looks feasible; construction and lookup cost become the principal uncertainties.
- Ψ⁸: the traversal-gap distribution is a genuine storage discriminator.
- Variable-length shortcuts: the gap distribution could help explain where longer records are representable, but it does not yet prove that a walker can exploit them efficiently.

## Seventh result: the cheapest composition census is now much simpler

The original T1 plan called for repeatedly constructing breakpoint-preimage sets and counting their unions.

That remains a valid correctness cross-check, but it is no longer necessary as the primary counting method.

A cheaper read-only analysis can:

1. Obtain the correct one-step breakpoint set.
2. Traverse the existing Ψ cycle once, recording the number of steps between successive breakpoints.
3. Build the gap-length histogram.
4. Calculate the complete safe-partition size curve using \\(\sum\min(k,g_i)\\).
5. Cross-check selected \\(k\\) values against explicit composition on a small test block.

The most useful output isn't merely \\(|C_4|\\). It's the gap survival curve—the fraction of breakpoint gaps longer than each \\(k\\).

That directly answers how much additional structure each extra shortcut step costs.

It is especially valuable because the same 900 KB block can have strong row-space clustering and almost no traversal-space clustering. Only the latter reduces the safe multi-step partition.

## Updated research dispositions

| Existing research item               | Progress from this round                                                                         | What remains                                            |
| ------------------------------------ | ------------------------------------------------------------------------------------------------ | ------------------------------------------------------- |
| #63, average run length vs occupancy | Mathematically narrowed: run count alone cannot determine aligned-group purity                   | Joint block-level statistics                            |
| #65, boundary alignment              | Alignment alone cannot account for all observed concentration under the representative counts    | Actual mod-32 boundary histogram                        |
| #66, clustering                      | Aggregate figures demonstrate row-space concentration                                            | Traversal-space clustering and block variation          |
| N1 / T1, breakpoint composition      | Exact safe-partition counting identity derived                                                   | Corpus gap histogram, construction and lookup costs     |
| N1, storage                          | Four-step record payload is bounded below the T-vector for the approximate 56k-breakpoint regime | Larger-run blocks and realistic metadata                |
| N3, run-native construction          | Better-defined representation budget                                                             | Whether construction from run records is actually cheap |
| #25 / K2, Reportable Shortcuts       | Citation identified; mechanism classified as search-oriented                                     | Any transferable inversion-specific result              |

I would not relabel the entire cluster as answered yet. But the central combinatorial uncertainty in T1 is now reduced to an exact, independently checkable statistic.

### Where this leaves the breakthrough search

The portfolio had N1 ranked first because interval composition might have enough overlap to make multi-byte records economical.

We can now say something sharper:

Four-step records may fit comfortably even without overlap, but their success will depend on making record construction and lookup cheaper than the dependent T-vector operations they replace.

Eight-step records have an additional requirement: enough traversal-space clustering to prevent excessive record growth.

That separates two previously entangled uncertainties—representation size and execution cost—and gives us a much better basis for deciding what to investigate.

My strongest recommendation is to amend T1 around the traversal-gap identity, retain a small explicit correctness cross-check, and concentrate subsequent mathematical work on constructing and querying the four-step records without rebuilding an expensive per-row structure.

We have not demonstrated a decoder speedup. But we have made substantive progress toward deciding whether the leading candidate can deliver one, and we have corrected an earlier false lead along the way.

That's the standard I'd maintain as we burn through the remaining questions: fewer unresolved mechanisms, sharper bounds and no inflated breakthroughs.
````
