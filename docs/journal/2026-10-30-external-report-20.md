# External research report 20: research burn-down rounds 16–19, conditional rows, lane bunching, and the builder's layouts and peak memory (verbatim)

A research input, not a finding of this repository. The maintainer relayed it on
2026-10-30, after external report 19 (`2026-10-30-external-report-19.md`). It has three
parts, in the order received.

1. **Round 16:**
   - the six conditional rows: #5 toward answered, #18 toward superseded;
   - the graded selector as bound propagation, m_k ≤ m_2k ≤ 2m_k − 1, with
     certified rejection and acceptance on size.
2. **Rounds 17–18:**
   - #74 posed as an exact lane-coincidence statistic, X(t) = Σ_l q(a_l + t), with its
     variance in terms of the cyclic autocorrelation;
   - a challenge to the 23 B/B builder: why it takes two passes, the 2sm′ write bound
     for two full views, and the alternative of one descriptor array plus a u32
     permutation.
3. **Round 19:**
   - split-excess accounting, Σ(e_i − 1) = m_2a − m_a;
   - the skewed children distribution;
   - **resident-space success is not peak-memory success:** the two-view builder's
     modelled peak is 8.2 MB at p90 and 12.2 MB at the worst block;
   - per-block selection with three constraints.

The coordinator's measurements answering these are in `2026-10-30-research-portfolio.md`
§3.1(i) and §9:
- the #74 lane simulation, which answers it;
- three builder layouts with traffic and peak memory;
- the selector under resident and peak constraints together.

This is source material. Its claims are checked there, not taken as findings.

The text reached us as rendered text (some formula symbols lost in round 19) and is kept as
received.

## 1. Round 16, as received

````text
## Burn-down update: the ledger is finally catching up

Claude's corrected count gives us the clearest picture yet of the original 47 open questions.

Research coverage of the original open questions

# 10

Answered since baseline

# 20

Materially narrowed

# 17

Not yet addressed

Claude's corrected research-coverage ledger, not yet a formal change to the original register's answer statuses.

That means 30 of the 47 originally open questions have received substantive research attention, with 17 untouched. The six conditional questions remain a separate group.

More importantly, the adaptive representation result is a real advance:

- All 233 Wikidata blocks select Ψ⁸.
- 203 can be accepted early from the Ψ⁴ bound.
- The `text` control selects Ψ⁴ at approximately 2.45 MB.
- High-run-count controls reject the compact route at level one.

This converts N1's corpus-specific space advantage into a graded per-block selection method. It does not remove the adverse 23 B/B construction-traffic model, but it means we no longer have to force one shortcut length onto every block.

## Burn-down round 16: six conditional questions

The conditional questions deserve particular attention because some of them are conditional on the original experiment, not necessarily still scientifically uncertain.

Here is the exact original set from the register:

| Question                                                             | Existing evidence                                                               | Recommended treatment                                                                  |
| -------------------------------------------------------------------- | ------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------- |
| #5 — Do optimal hints/lane counts differ at one versus four threads? | CI four-copy walk says yes; i9 full decoder says no                             | Answered as machine- and boundary-dependent; no universal tuning rule                  |
| #12 — Parallel speculative scanner                                   | Scanner can become a throughput bottleneck                                      | Retain conditional on measured scanner saturation and marker correctness               |
| #18 — libsais walk parameters in Ψ²                                  | The Ψ² candidate failed its larger economic test                                | Superseded for the existing Ψ² design, unless an independently cheaper builder appears |
| #62 — Compact Ψ competitive with shipped decoder                     | Older compact layouts have limited modeled upside; Ψ⁸ introduces a new approach | Retain, but distinguish the older compact-Ψ family from N1                             |
| #83 — Same-process looping twin                                      | Previously near, but not clearly past, its screening threshold                  | Defer until it can change a design decision                                            |
| #84 — Fill-buffer headroom at 4–10 workers                           | Smaller lane counts already performed worse; counters absent                    | Retain as attribution-only, not a prerequisite to other research                       |

Two conclusions stand out.

\#5 need not remain scientifically open merely because the answer differs between machines. The question asks whether optimal parameters can differ. Existing evidence already says they can. A universal policy is a different question.

\#18 shouldn't perpetually reopen a failed Ψ² candidate. Fine-tuning a walk that is dominated by its construction cost does not establish a path to a competitive full decoder. It remains relevant only if that underlying economic constraint changes.

I recommend those two as formal closure or supersession candidates, while retaining the others with narrower conditions.

## A further exact result: graded selection is a bound-propagation problem

Claude's graded Ψ¹/Ψ²/Ψ⁴/Ψ⁸ selector is more than a useful heuristic.

For the uncapped minimal partitions, doubling gives

\\[ m\_{2k}\le 2m_k-1. \\]

Combined with monotonicity,

\\[ \boxed{m_k\le m\_{2k}\le2m_k-1.} \\]

This provides two logically different decisions before constructing level \\(2k\\).

Let \\(b\_{2k}\\) be its bytes per descriptor, \\(D\_{2k}\\) its directory overhead, and \\(B\\) the memory budget.

Guaranteed rejection:

\\[ b\_{2k}m_k+D\_{2k}>B \\]

because the next level cannot have fewer records.

Guaranteed acceptance on size:

\\[ b\_{2k}(2m_k-1)+D\_{2k}\le B. \\]

Everything between those bounds genuinely requires more information.

For the 900 KB T-vector budget, with \\(b_8=12\\) and approximately 225 KB of directory overhead, the simple upper-bound acceptance test is roughly

\\[ m_4\le140{,}626. \\]

The exact threshold depends on directory accounting and segment-cap metadata. The bound above applies to the uncapped partition; caps need to be included explicitly in a certified acceptance test.

This explains why accepting 203 blocks before fully constructing Ψ⁸ is meaningful: the selector can make certain decisions from proofs, not predictions.

There's also an important limit: a proof that Ψ⁸ fits says nothing about whether Ψ⁸ is the fastest choice. Ψ⁴ could still win if its construction is substantially cheaper.

Thus the graded selector now has a rigorous storage dimension, while its CPU-optimal dimension remains unresolved.

## Suggested ledger change from this round

The corrected breakdown remains:

10 answered

20 narrowed · 17 untouched · 6 conditional separately

I'd move #5 toward answered and #18 toward superseded, contingent on Claude's register rules. I would not automatically promote the other four conditionals.

The next step in the burn-down is to examine the 17 untouched questions individually, starting with those answerable from existing measurements or mathematics, rather than commissioning another experiment.

We're making measurable progress on both fronts: the new algorithm is being subjected to stronger economic constraints, and the original research inventory is actually being reduced.
````

## 2. Rounds 17–18, as received

````text
This is a substantial improvement in the burn-down. Claude has reconciled the inventory, measured the structural questions, and kept the adverse N1 construction result visible. I accept the corrections.

The next useful contribution is to challenge the remaining uncertainties rather than repeat settled results. I'll leave #28–#33, #36 and #73 with the independent literature agent, and concentrate on #74 and the builder's 23 B/B cost.

## Round 17 — #74: do expensive lookups bunch across the 32 cooperative lanes?

The new census establishes that Ψ boundaries cluster strongly, both in row order and along traversal. But that does not establish whether expensive lookups occur simultaneously across lanes.

Those are different statistical properties.

We can settle the distinction precisely and give Claude a read-only analysis that answers #74 from the existing blocks.

### 1. An exact model for lane bunching

Represent a full Ψ traversal as a cyclic sequence of length \\(n\\). Define

\\[ q(t)= \begin{cases} 1 & \text{if the lookup at traversal position }t\text{ uses the expensive path},\\\ 0 & \text{otherwise}. \end{cases} \\]

Let \\(a_1,\ldots,a\_{32}\\) denote the traversal offsets of 32 active lanes.

For a synchronized round in which all lanes advance once, the number of expensive lookups is

\\[ \boxed{X(t)=\sum\_{\ell=1}^{32}q(a\_\ell+t).} \\]

This gives us exactly the quantity #74 asks about.

The average is determined by the expensive-lookup fraction, but its variation depends on how the lane starts align with the expensive positions.

Let

\\[ p=\frac1n\sum_tq(t) \\]

and define the cyclic autocorrelation

\\[ R(d)=\frac1n\sum_tq(t)q(t+d). \\]

Then, averaging over traversal phase,

\\[ \boxed{ \operatorname{Var}(X)= 32p(1-p) + 2\sum\_{\ell\<m} \left[R(a_m-a\_\ell)-p^2\right]. } \\]

This is exact for the fixed-offset, synchronized, fully active model.

It reveals why the new 84.8% short-gap statistic cannot by itself close #74.

That statistic measures local traversal clustering. Lane bunching depends on autocorrelation at differences between lane starting offsets, which may be much larger than one or two steps.

Two blocks can have the same expensive-lookup fraction and similar short-gap distributions, yet very different simultaneous lane costs.

### 2. How to answer #74 from existing data

I would compare three schedules using identical expensive-lookup flags:

| Schedule                                            | Purpose                                                                   |
| --------------------------------------------------- | ------------------------------------------------------------------------- |
| Actual cooperative starts and segment assignments   | Measures what mibz2 would encounter                                       |
| Independently phase-shifted lane sequences          | Breaks cross-lane alignment while preserving each lane's local clustering |
| Randomized expensive flags with the same prevalence | Measures the simpler independence baseline                                |

For each schedule, record the mean, variance, 95th/99th percentile and maximum number of expensive lookups per active lane round.

The particularly valuable statistic is

\\[ \boxed{ P(X\ge 8) } \\]

for the 32-lane case, alongside the full distribution. Eight is a diagnostic threshold, not a hardware limit.

There is one implementation detail to respect: the shipped walk does not have 32 permanently active, equally long segments. Lanes discover flagged endpoints and acquire further work.

Therefore the fixed-offset formula is the analytical control, not a complete simulation of the shipped scheduler. The actual simulation must reproduce segment turnover and count active lanes.

A second limitation is that a count of expensive operations isn't automatically a CPU-stall prediction. Their latency, overlap and cache-hit behavior matter. Nevertheless, it answers the structural part of #74 directly.

Research disposition: #74 is reducible to an exact traversal-schedule statistic. The existing short-gap measurements do not yet answer it.

## Round 18 — challenge the 23 B/B builder model

I would resist the temptation to search for clever ways to delete the builder's first counting pass before checking what it costs.

The current construction uses two passes for an important reason: an intersection record must be placed in two different orders.

One output view is ordered by source row. The other is ordered by image row.

A single interval-join pass naturally encounters the intersections in image order. It does not automatically know where each intersection belongs in source order without knowing the preceding child counts for that source parent.

This makes the first pass more than an accidental implementation inefficiency.

### A necessary distinction

There are at least three notions of “one-pass construction”:

1. One pass over the original L-run stream.
2. One pass per interval-composition level.
3. One pass over the data while constructing both ordered output views.

The existing algorithm satisfies a useful form of the first property but not the others.

Eliminating one merge pass is not free if it introduces a general sorting operation, a predecessor structure or extra random writes.

### A lower-bound observation

Suppose a doubling level produces \\(m'\\) records, and both ordered views contain complete records of size \\(s\\).

If both views are physically materialized independently, their payload alone requires at least

\\[ \boxed{2sm'} \\]

bytes of writes.

This is only a layout-specific lower bound, not an information-theoretic lower bound on construction. It can potentially be avoided by storing one descriptor array plus an index permutation for the second order.

That gives us a concrete comparison:

| Construction representation                    | Main trade-off                                          |
| ---------------------------------------------- | ------------------------------------------------------- |
| Two complete descriptor views                  | Simpler sequential access; more data copied             |
| One descriptor view + 32-bit index permutation | Less duplicated payload; extra indexed gathers          |
| One descriptor view + compact permutation      | Less storage; more extraction work                      |
| Recompute one ordering on demand               | Less retained state; repeated intersections or searches |

For eight-byte and twelve-byte records, a four-byte permutation can be significantly smaller than a duplicate record array.

But it can also turn sequential descriptor access into indexed gathering, potentially trading bandwidth for latency.

The correct way to evaluate this is to compare per-level modeled bytes read and written, preserving the distinction between sequential streams and scattered accesses.

This challenges the 23 B/B builder without assuming we can eliminate its work.

I would also reject a universal conclusion that fewer bytes necessarily wins: the shipped T-vector builder's irregular writes and the composed builder's ordered passes can have very different CPU costs.

## What I would send Claude

### Research rounds 17–18: #74 and the two-view builder

**#74 — lane bunching.** The census's short traversal gaps do not establish simultaneous exceptional operations across 32 lanes. For expensive-lookup indicator q(t) on the Ψ cycle and fixed lane offsets a_l, the per-round exceptional count is X(t)=Σ_l q(a_l+t). Its variance is 32p(1-p)+2Σ\_{l\<m}[R(a_m-a_l)-p²], where R is cyclic autocorrelation.

A read-only test can use the existing block and segment information. Reproduce actual cooperative segment turnover; measure the distribution of exceptional lookups per active round. Compare against independently phase-shifted lanes and shuffled flags. Keep structural simultaneity separate from claims about hardware stalls.

**Builder cost — challenge before implementation.** The \~23 B/B construction traffic is adverse and remains the current model. The two-pass join exists because source and image views need different child ordering. Model an alternative with one complete descriptor array plus a u32 permutation, instead of two complete descriptor arrays. Account for sequential accesses versus indexed gathers, per-level traffic and peak temporary memory. Do not label fewer modeled bytes as fewer measured cycles.

**Ledger:** #74 remains conditionally unresolved until lane-coincidence statistics exist. The alternative builder layout is a cost-model comparison under existing #53, not a new research question. The 233-block Ψ⁸ storage result and the adverse construction model remain unchanged.

Do not initiate production code changes or i9 experiments.

## Progress assessment

The next phase of the burn-down should be about closing independent uncertainties, not just assigning dispositions.

Claude's reconciliation is important: ten of the originally open questions are now answered, twenty have defined evidence gaps, and the rest are being resolved or independently audited. That's a much firmer position than an unstructured backlog of 47 open questions.

My immediate priorities are #74's actual lane-coincidence calculation, the indexed-view builder cost model, and the pending MTF token census. All three can materially change our understanding using existing data.

We're still in the burn-down. The objective remains to exhaust the research inventory before moving to implementation.
````

## 3. Round 19, as received

````text
Burn-down round 19: the new child-count results change the builder analysis

The updated portfolio contains an important result we haven’t exploited yet: 94.2% of Ψ⁴ parents survive the Ψ⁴→Ψ⁸ composition without splitting, at the median of the 40-block sample.

That is much stronger than the previous mathematical lower bound of approximately 69%.

But the same census reveals something more significant: a few parents produce thousands of children. At the final doubling level, one parent produced as many as 2,093 children in the sample.

This means the composition work is highly skewed. We should not design or cost it as though each interval produces roughly the same number of children.

1. Most parents survive, but a much smaller fraction of output records come from them

Using the census medians as an illustrative block—not claiming they describe the same individual block—we have:

m_4\approx84{,}287,\qquad m_8\approx109{,}726.

If 94.2% of the parents are unsplit, approximately 79,400 parents contribute one child each. The remaining 4,900 parents contribute approximately 30,300 children.

So:

94.2%

Ψ⁴ parents surviving intact, sampled median

~27.6%

Ψ⁸ records produced by splitting parents, illustrative calculation

The distinction matters.

An unsplit-parent fast path can simplify the majority of parent processing, but cannot eliminate most output-record writes. The split minority still accounts for a substantial portion of the final table.

A traffic model that assumes only 6% of the work remains is therefore unjustified.

2. A sharper invariant: every split has an exact accounting cost

For parent , let be its number of children.

Then

m_{2a}=\sum_i e_i

and consequently

\boxed{\sum_i(e_i-1)=m_{2a}-m_a.}

This identity holds regardless of how concentrated the splitting is.

For the illustrative median counts:

m_8-m_4=25{,}439.

So the final composition introduces approximately 25,439 additional records beyond the one-child-per-parent baseline.

That gives us a useful separation:

* : unavoidable parent processing under this representation.
* : additional output records caused by refinement.
* The number and sizes of splitting parents: determine where the extra work concentrates.

Unlike the parent-count percentages, the first two are exact for each measured block.

Research payoff: existing #53 can now be modeled using base-copy work plus split-excess work, instead of treating all records as equally expensive.

3. Why the long tail matters for a different construction strategy

The measured maxima—3,161, 3,439 and 2,093 children at successive doubling levels—suggest a useful asymmetry:

A simple branch for unsplit parents should cover the common case. A streaming interval-intersection loop should handle the rare large splits.

This avoids per-parent fixed-size buffers, which would be extremely wasteful for mostly unsplit parents and unsafe for the long tail.

But it does not prove a faster builder. We must still produce the correct source and image ordering, and any additional indexing may introduce random accesses.

The stronger conclusion is that the census has now supplied the exact structural quantities needed to compare builder representations without speculating about a typical parent.

4. The more serious result: resident-space success is not peak-memory success

The latest portfolio gives these construction-memory figures:

Point in Wikidata distribution	Ψ⁸ peak live memory	Shipped reference, approximately 4.5 MB
Median	4.7 MB	+0.2 MB
90th percentile	8.2 MB	+3.7 MB
Worst observed	12.2 MB	+7.7 MB

These are per-block modeled live-memory figures, not measured process RSS.

Nevertheless, the conclusion is quite strong:

The present two-view Ψ⁸ builder does not satisfy the project’s memory objective at the upper end of the measured Wikidata distribution, despite every final Ψ⁸ table fitting under 3.6 MB.

The earlier statement that Gate A passes needs to be kept narrowly defined: the resident representation passes. The current construction memory model does not.

For four simultaneously active blocks, the upper-tail scratch exposure could be significant. We shouldn’t multiply 12.2 MB by four and call that measured process RSS, but we should account for concurrent construction lifetimes explicitly.

5. This changes the representation-selection objective

The graded selector currently chooses a representation using a storage budget:

S_k\le B.

But our actual objective has at least three constraints:

\boxed{
\begin{aligned}
S_k&\le B_{\mathrm{resident}},\\
P_k&\le B_{\mathrm{peak}},\\
T_{\mathrm{build},k}+T_{\mathrm{walk},k}
&<T_{\mathrm{baseline}}.
\end{aligned}
}

Here is peak live construction memory.

These constraints aren’t interchangeable.

A block may have a Ψ⁸ table that fits comfortably but a Ψ⁸ builder whose temporary views make it undesirable. A Ψ⁴ option might be superior even though Ψ⁸ passes the final-table space gate.

This is an existing #45/#48/#53 trade-off, not a new research item.

In particular, all 233 blocks choosing Ψ⁸ under a resident-space rule does not imply all 233 should choose Ψ⁸ under the complete decoder objective.

6. What we have burned down in this round

Question	New resolution or constraint
#53 — compact construction	Exact split-excess accounting; 94% unsplit-parent fast path is justified structurally
#45 — hybrid versus one format	Per-block selection must include peak construction memory, not just resident bytes
#48 — space/throughput frontier	Final representation size alone is insufficient to define the frontier
#51 — non-translation loads	Splitting distribution identifies where complex construction occurs, not its lookup latency
N1 / N3	The current builder has an upper-tail memory problem in addition to its 23 B/B traffic problem

My recommendation is to challenge the duplicate-view memory requirement next, before optimizing its cycle count.

The specific question is whether a single descriptor store plus two orderings of lightweight indices can reduce the high-end 12.2 MB live-memory figure without replacing sequential construction with prohibitively expensive indexed gathers.

That is falsifiable from the existing per-level counts and record sizes. It doesn’t require a production decoder or a workstation run.

Status: the builder is now our leading target for mathematical and cost-model reduction. We’ve extracted an exact split-work identity, identified a strongly skewed workload, and found a memory constraint that the final-table census alone concealed.

We keep burning down the existing register.
````
