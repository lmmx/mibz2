<!--
Provenance: the coordinator's reading of the research chatbot's discontinuity programme
(discontinuities-research.md), relayed by the user into the working session on 29 October
2026. The text below is unedited, except that one interactive piece it contained (two kinds
of clustering) reached the session as its text alone, and is kept as that text.
-->

This is the strongest mathematical result from the latest research cycle. The researcher has gone beyond describing why the 77.6% translation coverage might occur: it has derived a way to calculate that coverage exactly from BWT runs, without constructing the full Ψ table.

More importantly, it has identified a possible inconsistency in Claude's previous measurements. I would prioritise resolving that inconsistency before asking for any new experiments.

## 1. The researcher has found a genuinely useful result

Previously, we had the bound:

\\[ d\le r-1 \\]

where \\(d\\) is the number of discontinuities in Ψ and \\(r\\) is the number of runs in the BWT last column.

Claude verified the bound, but reported that it was roughly ten times looser than the measured quantity.

The researcher has now sharpened the mathematics:

\\[ \boxed{r-\sigma'\le d\le r-1} \\]

where \\(\sigma'\\) is the number of distinct symbols.

This follows because each additional run of a particular character creates exactly one discontinuity in that character's Ψ occurrence list. Boundaries between different character classes account for at most another \\(\sigma'-1\\) discontinuities.

I agree with this derivation under our established stable-Ψ definition.

And it reveals something important: for blocks with many thousands of runs, \\(d\\) cannot be ten times smaller than \\(r\\).

So something needs clarifying.

### The apparent contradiction

The likely explanation is that Claude was comparing BWT runs with affected 32-row groups, not with individual discontinuities.

Those are different quantities:

| Quantity      | Meaning                                            |
| ------------- | -------------------------------------------------- |
| \\(r\\)       | Number of BWT runs                                 |
| \\(d\\)       | Number of individual Ψ translation breaks          |
| \\(K\_{32}\\) | Number of distinct 32-row groups containing breaks |

Many individual discontinuities can occupy the same group.

Consequently, \\(K\_{32}\\) can be dramatically smaller than \\(d\\), even though \\(d\\) remains close to \\(r\\).

We need Claude to confirm which quantity was actually measured. This is a terminology and validation check, not evidence that its census is necessarily wrong.

## 2. We can now calculate translation coverage without building Ψ

This is the most practically useful discovery.

Each BWT run corresponds to a consecutive interval within one character's Ψ occurrence list.

Using the cumulative run lengths of each character, we can determine exactly where Ψ's translation property breaks.

Then, for globally aligned groups of 32 rows, we simply determine which groups contain at least one internal break.

A break exactly between two groups spoils neither.

The result is:

\\[ \boxed{ \text{Translation coverage} = 1-\frac{K\_{32}}{\lfloor n/32\rfloor} } \\]

for complete groups, with partial groups handled separately.

The calculation requires BWT run information, character counts and a small group bitmap. It does not require constructing the full permutation.

For a 900,000-row block, that bitmap would occupy approximately 3.5 KiB.

That's useful because it gives us an exact structural feasibility test for the two-tier descriptor design.

It could eventually help choose a representation during construction, although whether that decision is worth making dynamically remains unproven.

## 3. The second result is even more interesting for performance research

The researcher identifies a way to measure whether slow-path navigation tends to follow another slow-path navigation.

Let \\(S\\) be the set of rows belonging to nontranslation groups.

Then:

\\[ A_2=|S\cap\Psi^{-1}(S)| \\]

counts rows whose current navigation requires a payload lookup and whose next navigation also requires one.

This gives:

\\[ P(\text{slow next}\mid\text{slow now}) = \frac{A_2}{|S|} \\]

when each row is counted once and \\(S\\) is nonempty.

That's an exact measure of one-step slow-path correlation.

It is more informative than knowing that 22.4% of rows belong to nontranslation groups.

Why? Because two consecutive dependent payload accesses may be worse than two isolated ones, especially for pointer-chasing.

The researcher also shows how piecewise translation permits this statistic to be calculated using interval intersections, without explicitly following every Ψ link.

That is a useful theoretical reduction.

However, there is a limit.

Two different kinds of clustering

Within a traversal chain

The researcher can calculate whether a slow visit tends to be followed by another slow visit.

Across simultaneous cooperative lanes

Whether several lanes stall together depends on segment assignment, scheduling, memory latency and hardware execution.

That second problem cannot be settled from the permutation alone.

This distinction is excellent. It prevents us from confusing an exact mathematical correlation with a predicted CPU speedup.

## 4. What this changes in our work

I would make one small adjustment to the post-release compact-Ψ research plan.

Before implementing either the fixed-line hybrid or the two-tier directory, we should add a structural validation step.

It should establish:

- The exact relationship between \\(r\\), \\(d\\) and \\(K\_{32}\\), including reconciliation of the tenfold discrepancy.
- Whether the run-derived group classification matches Claude's existing Ψ census for every block.
- Per-block translation coverage and its distribution across the full-sized blocks.
- The first-order slow-path correlation \\(A_2\\), plus a comparison with the simple independent-access reference model.

This work is relatively narrow. It can reuse the existing corpus and census infrastructure.

It should not become a new open-ended research project.

### Why I would do this before building a decoder

We already know the 16-bit representation fits the measured data remarkably well.

What we don't know is whether the two-tier design's extra payload lookups will undermine traversal throughput.

If the researcher is correct, we can now calculate an important part of that risk before constructing an expensive experimental decoder.

That makes the eventual microbenchmark more targeted.

## 5. What I'd send Claude — after the release

The researcher's latest cycle has produced two useful mathematical results for #79's future compact-Ψ work.

Please preserve the report when convenient, without interrupting the current release preparation.

First, the discontinuity bound has been strengthened to:

(r-\sigma' \le d \le r-1),

where (d) counts individual adjacent Ψ transitions that are not +1, (r) counts BWT runs and (\sigma') is the active alphabet size.

This raises a question about the earlier statement that the bound was approximately 10× looser than the measured discontinuity count. Please check whether the census actually compared (r) with the number of affected 32-row groups, (K\_{32}), rather than with individual breaks (d).

Second, the researcher derived an exact method for calculating translation-group coverage from BWT run endpoints, without materializing Ψ. This can provide an independent check against `psi_census.rs`.

It also proposed the one-step slow-path correlation statistic:

(A_2=|S\cap\Psi^{-1}(S)|),

where (S) contains rows requiring nontranslation payloads.

These are useful additions to the future compact-representation evaluation. Please record them with their definitions and limitations.

Do not start new experiments or alter the release scope. Once the release and cooperative integration priorities are settled, we can decide whether a small structural census extension is warranted.

## My final assessment

I think this cycle has genuinely advanced the research.

It produced:

1. A stronger theorem about Ψ discontinuity counts.
2. An exact construction-independent calculation of fixed-group translation eligibility.
3. A measurable correlation statistic for consecutive slow-path accesses.
4. A falsifiable discrepancy in how the earlier census was described.

Of these, the fourth deserves the most immediate attention when Claude has capacity. Reconciling definitions is easy compared with implementing a decoder, and it protects the integrity of everything built on the census.

The first three improve the eventual experimental design without forcing us to invent another representation.

My scheduling decision remains unchanged: ship the release, integrate cooperative decoding, and then use these results to make the compact-Ψ experiments more discriminating.

The researcher's latest contribution gives us better tools to decide which compressed representation deserves implementation. It doesn't yet justify changing the shipping order.
