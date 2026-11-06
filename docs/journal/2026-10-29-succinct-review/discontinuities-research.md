<!--
Provenance: the research chatbot's third programme after #81, on where Ψ's discontinuities
land, written in answer to the coordinator's reading of §16; relayed by the user into the
working session on 29 October 2026. The text below is unedited, except that two interactive
illustrations it contained (one symbol's occurrence list, and clustered against spread
breaks) reached the session as their text alone, and are kept as that text.
-->

# Research response: The missing variable is where the discontinuities land

The coordinator's feedback is valuable because it exposes the weakness in our earlier theorem.

We proved:

\\[ d\le r-1 \\]

where \\(d\\) is the number of breaks in Ψ's consecutive-position behaviour and \\(r\\) is the number of BWT runs.

That bound is correct, but Claude's measurements show it is too loose to predict the observed 77.6% translation coverage.

I think we can do substantially better. In fact, we can derive an exact characterization of translation-group coverage directly from the run-length encoding of L, without constructing the full Ψ array.

This is stronger than another statistical model. It follows from the stable occurrence-order property of the BWT.

## 1. The crucial observation: run lengths become ordinal intervals

Consider the BWT last column:

`aaabbbaaaaaccbbb`

This is not meant to be an actual bzip2 block, just an illustrative string.

It contains two runs of `a`:

- The first contributes occurrence ordinals 0, 1, 2.
- The second contributes occurrence ordinals 3, 4, 5, 6, 7.

Within the Ψ list for `a`, those original L positions appear in increasing order.

One symbol's occurrence list

0

0

1

1

2

2

6

3

7

4

8

5

9

6

10

7

The jump from 2 to 6 is a Ψ translation discontinuity, corresponding to the boundary between two runs of `a` in L.

The important mathematical fact is:

Each BWT run becomes a consecutive-position interval within its character's Ψ occurrence list.

We do not need to construct Ψ to know where these intervals begin or end.

We only need the lengths of the BWT runs, grouped by character.

That gives us a direct route to predicting translation coverage.

## 2. An exact formula for fixed-group coverage

Let the BWT contain runs of character \\(c\\) with lengths:

\\[ \ell\_{c,1},\ell\_{c,2},\ldots,\ell\_{c,r_c}. \\]

The F interval of \\(c\\) begins at \\(C[c]\\).

The end of the \\(t\\)-th run's occurrence-ordinal interval is:

\\[ q\_{c,t} = C[c]+\sum\_{u=1}^{t}\ell\_{c,u}. \\]

For every nonfinal run of \\(c\\), the adjacent Ψ values at positions

\\[ q\_{c,t}-1,\quad q\_{c,t} \\]

cannot be consecutive, because they correspond to different, separated runs of \\(c\\) in L.

Therefore \\(q\_{c,t}\\) identifies the position after a translation discontinuity.

Now consider globally aligned groups of \\(B=32\\) rows.

A discontinuity at boundary \\(q\\) spoils a group only if it falls inside that group.

If:

\\[ q\bmod32=0 \\]

then the discontinuity lies exactly between two groups and spoils neither.

Otherwise, it spoils the group containing \\(q-1\\).

Thus the affected group is:

\\[ \boxed{ g(q)=\left\lfloor\frac{q-1}{32}\right\rfloor \quad\text{when }q\bmod32\ne0. } \\]

This immediately gives us something much stronger than \\(d\le r-1\\).

The number of nontranslation groups is the number of distinct fixed groups hit by these discontinuities, plus any additional groups affected by discontinuities between F-symbol intervals.

This accounts automatically for the effect Claude found: many discontinuities can be concentrated in the same 32-row group, so counting them individually greatly overstates the number of spoiled groups.

It also explains why the simple \\(r-1\\) bound can be vacuous while actual translation coverage remains high.

## 3. We can calculate coverage without building Ψ

This leads to an exact procedure.

We need only the run-length encoding of L, plus the cumulative symbol counts already used to establish F.

From BWT runs to translation-group coverage

1. Read the runs of L, retaining each run's character, start position and length.
2. For each character, accumulate its run lengths in occurrence order.
3. Convert each cumulative run endpoint into its corresponding global F position.
4. For endpoints inside a 32-row group, mark that group as nontranslation.
5. Check the boundaries between adjacent nonempty F-symbol intervals separately.
6. Count the distinct marked groups. Every unmarked full group is translation-eligible.

For the symbol-boundary checks, we must be precise. At the boundary between two F-symbol intervals, the final Ψ value of the preceding symbol and the first Ψ value of the next symbol may or may not happen to be consecutive positions in L.

We can determine that from the last occurrence position of the preceding symbol and the first occurrence position of the next symbol.

So we don't need to conservatively mark every F boundary as a failure.

With those checks, the procedure is exact.

For \\(G=\lfloor n/32\rfloor\\) complete groups, define the set of affected groups:

\\[ \mathcal B = \left\\{ \left\lfloor\frac{q-1}{32}\right\rfloor: q\text{ is a genuine Ψ break}, \ q\bmod32\ne0 \right\\}, \\]

restricted to boundaries inside complete groups.

Then:

\\[ \boxed{ \text{Translation coverage among full groups} = 1-\frac{|\mathcal B|}{G} } \\]

This is exact, not an approximation.

Crucially, \\(\mathcal B\\) can be constructed from the run-length description of L, without materializing the full Ψ permutation.

An implementation could use one bit per group to avoid counting the same group repeatedly.

For a 900,000-row block, that's 28,125 bits, only about 3.5 KiB.

The scan takes \\(O(r+\sigma'+n/B)\\) time if we initialize and count a group bitmap, or potentially \\(O(r+\sigma')\\) with a sparse set of affected groups and appropriate bookkeeping. Here \\(\sigma'\\) is the number of symbols present.

This is a substantially better characterization than the earlier theorem.

# Finding 2 — The relevant statistic is discontinuity occupancy

The previous model used the number of breaks \\(d\\).

The new model uses:

\\[ K_B=|\mathcal B|, \\]

the number of distinct groups of width \\(B\\) containing at least one internal break.

These can differ dramatically.

Suppose there are 10,000 discontinuities.

Same number of breaks, different group coverage

Clustered breaksSpread breaks

Total marked breaks

## 20

Affected 8-position groups

## 4 / 14

Illustration with eight-position groups for legibility. Orange marks represent breaks, and alternating shades indicate groups. The real analysis uses 32-row groups and excludes breaks exactly between groups.

The distinction is now obvious.

Many discontinuities occupying one group cost only one nontranslation record.

The same number spread across many groups costs many records.

This suggests the right research statistic isn't simply BWT run count, or even the number of Ψ breaks.

It's the occupancy distribution of breaks in fixed-size windows.

That is the exact structural predictor for our format coverage.

## A useful hierarchy of increasingly informative statistics

| Statistic                                 | What it tells us                                            |
| ----------------------------------------- | ----------------------------------------------------------- |
| \\(r\\), BWT run count                    | Coarse indication of repetitiveness                         |
| \\(d\\), Ψ discontinuity count            | How many translation relationships fail                     |
| \\(K\_{32}\\), affected group count       | Exactly how many 32-row groups lose translation eligibility |
| Distribution of breaks per affected group | How concentrated the discontinuities are                    |
| Longest consecutive translation stretches | How readily other group sizes could exploit the structure   |

For our fixed 32-row directory, \\(K\_{32}\\) is the statistic that matters most.

This resolves the coordinator's first research challenge more strongly than I expected: we can calculate exact translation coverage from the run structure, rather than trying to find a tighter universal upper bound based only on \\(r\\).

It doesn't predict traversal speed, but it precisely predicts which groups can use the translation encoding.

# Finding 3 — An important consistency check in Claude's reported measurements

The coordinator said the earlier \\(d\le r-1\\) bound was approximately ten times larger than the observed discontinuity counts.

There is a subtle but important distinction here.

For the standard stable Ψ mapping, the number of individual adjacent-value discontinuities is not merely upper-bounded by \\(r-1\\). It also has a strong lower bound.

If there are \\(r_c\\) runs of symbol \\(c\\), then the Ψ occurrence list for \\(c\\) has exactly \\(r_c-1\\) discontinuities between those runs.

Therefore, summing over present symbols:

\\[ \boxed{ d\_{\text{within-symbol}}=r-\sigma' } \\]

The boundaries between different F-symbol intervals contribute between zero and \\(\sigma'-1\\) additional discontinuities.

Consequently:

\\[ \boxed{ r-\sigma'\le d\le r-1 } \\]

This is stronger than our original theorem.

And it has an important implication.

With a byte alphabet, \\(\sigma'\le256\\). Therefore, for a block containing many thousands of BWT runs, \\(d\\) must be very close to \\(r\\).

It cannot be ten times smaller.

So the reported tenfold difference deserves a terminology check.

My leading explanation is that Claude may be comparing BWT run count with the number of distinct affected 32-row groups, rather than with individual Ψ discontinuities.

Those quantities are different, and a tenfold reduction through group occupancy is entirely plausible.

An alternative is that the census counts a different kind of break, such as failures among aligned 32-entry groups rather than every adjacent Ψ pair.

This is not evidence that Claude's census is wrong. The 77.6% translation result may be entirely correct.

But if Claude truly reports \\(d\approx r/10\\), where \\(d\\) means every adjacent Ψ transition failing `+1`, then that conflicts with the standard stable Ψ structure under our definitions.

That distinction is worth resolving because it determines which mathematical model applies.

I would preserve this as an explicit verification question, rather than silently accept the claim that the old bound was loose by a factor of ten.

# Finding 4 — Traversal-order clustering can also be studied analytically

The second question is harder, but not entirely beyond mathematical analysis.

Suppose we've identified all nontranslation groups.

Let \\(S\\) denote the set of Ψ row indices belonging to those groups.

Then a slow-path navigation occurs when the current row \\(j\\) belongs to \\(S\\).

The next navigation starts at:

\\[ \Psi(j) \\]

So two consecutive slow-path navigations occur precisely when:

\\[ j\in S \quad\text{and}\quad \Psi(j)\in S. \\]

This gives us an exact statistic:

\\[ \boxed{ A_2= \left|S\cap\Psi^{-1}(S)\right| } \\]

where \\(A_2\\) counts slow-to-slow transitions over the permutation.

This is potentially much more informative than the simple statement that 22.4% of rows are slow.

If slow rows were independently distributed along the traversal, we might expect roughly:

\\[ P(\text{slow next}\mid\text{slow now}) \approx0.224. \\]

But that independence is only a statistical reference model, not something guaranteed by the BWT.

A significantly larger conditional probability would indicate temporal clustering of slow-path accesses.

A smaller probability would indicate that they tend to be separated.

## The useful surprise: Ψ's translation structure lets us calculate this without explicitly walking every link

Within any maximal translation interval:

\\[ \Psi(j)=j+\delta \\]

for a fixed displacement \\(\delta\\).

Therefore, to count slow-to-slow transitions originating in that interval, we need only count rows satisfying:

\\[ j\in S \quad\text{and}\quad j+\delta\in S. \\]

Equivalently, we can intersect the slow-row intervals with shifted copies of themselves.

For a translation interval \\(I\\):

\\[ A_2(I)=|I\cap S\cap(S-\delta)|. \\]

Summing across the maximal translation intervals gives \\(A_2\\).

The necessary intervals can be derived from the BWT run structure and symbol boundaries, so this calculation need not materialize the entire Ψ table.

It could be implemented through interval-intersection or sweep-line techniques, with complexity determined by the number of translation intervals and slow-group ranges.

This is an established mathematical reduction, not a claim that we've already implemented or measured it.

It gives us a way to answer whether slow paths tend to follow other slow paths—without performing a complete decoder traversal.

# Finding 5 — What mathematics cannot tell us about 32 simultaneous lanes

The slow-to-slow statistic is useful, but it does not completely solve the coordinator's second question.

There are two different forms of clustering.

Clustering along one Ψ chain: Does a slow navigation tend to be followed by another slow navigation?

This can be studied using \\(A_2\\), and extended to longer sequences.

For example, the number of three consecutive slow visits is:

\\[ A_3= \left| S\cap\Psi^{-1}(S)\cap\Psi^{-2}(S) \right|. \\]

Clustering across 32 concurrent lanes: Do several lanes encounter slow records during the same execution window?

This is the harder problem.

It depends on:

- Which rows are chosen as segment starts.
- How segments are assigned to lanes.
- Whether the scheduler advances lanes in lockstep.
- How quickly different record formats execute.
- Whether the second-level payloads remain cache-resident.
- How the CPU overlaps outstanding memory requests.

The Ψ permutation determines the logical sequence of rows. It does not, by itself, determine their execution timing.

Consequently, there is no general reason to expect a statistic such as \\(A_2\\) to predict the precise number of simultaneously stalled lanes.

Even an exact analysis of logical traversal order would still need a scheduling and latency model.

This is where the theoretical investigation meets its evidence boundary.

# Final research conclusions for the coordinator

We've obtained a more useful result than a marginally tighter bound on \\(r\\).

## Result A: Exact translation-group characterization

For each character \\(c\\), let its BWT run lengths be:

\\[ \ell\_{c,1},\ldots,\ell\_{c,r_c}. \\]

The cumulative run endpoints, shifted into c's F interval, identify every within-symbol Ψ discontinuity.

The between-symbol discontinuities can be established by comparing the relevant first and last occurrence positions in L.

A fixed group is nontranslation exactly when it contains at least one internal discontinuity.

Thus, the exact number of nontranslation 32-row groups can be calculated from the BWT's run-length description without constructing Ψ.

This is a deterministic characterization, not a statistical prediction.

## Result B: Stronger discontinuity bounds

For \\(\sigma'\\) distinct symbols and \\(r\\) BWT runs:

\\[ \boxed{r-\sigma'\le d\le r-1} \\]

under the standard stable Ψ definition.

If the reported discontinuity count is approximately ten times smaller than \\(r\\) on full-sized blocks with many runs, it likely measures affected groups rather than individual Ψ breaks. The metric definitions should be reconciled.

## Result C: Exact first-order traversal-clustering statistic

If \\(S\\) denotes the rows requiring nontranslation payloads, then:

\\[ \boxed{ P(\text{slow next}\mid\text{slow now}) = \frac{|S\cap\Psi^{-1}(S)|}{|S|} } \\]

assuming \\(|S|>0\\) and each row is counted once.

Because Ψ is piecewise translation, the numerator can be calculated using intersections of shifted intervals, avoiding full permutation materialization.

This is a useful analytical measure of temporal slow-path clustering.

It does not predict simultaneous memory stalls across the 32 cooperative lanes.

## Result D: The limit of the analysis

We now have a precise hierarchy:

| Research question                                     | Status                                                               |
| ----------------------------------------------------- | -------------------------------------------------------------------- |
| How many individual Ψ discontinuities exist?          | Exactly characterized from BWT runs                                  |
| How many fixed 32-row groups contain discontinuities? | Exactly computable from run endpoints                                |
| Are slow visits correlated along one Ψ traversal?     | First-order correlation exactly computable by interval intersections |
| Do slow visits cluster across cooperative lanes?      | Requires segment assignment and a scheduling model                   |
| Does clustering harm throughput?                      | Requires hardware measurements                                       |

## Concise handoff

# Succinct Ψ research: discontinuity clustering findings

The previous bound (d\le r-1) can be strengthened to:

[\
r-\sigma'\le d\le r-1\
]

where (d) counts all adjacent Ψ values that do not differ by one, (r) counts BWT runs, and (\sigma') is the active alphabet size.

The exact within-symbol discontinuity positions can be derived from cumulative run lengths for each character, shifted by that character's F-interval starting position.

For globally aligned 32-row groups, only discontinuities inside a group spoil translation eligibility. Breaks exactly between groups do not.

**New result:** Translation coverage can be calculated exactly from BWT run metadata without constructing the complete Ψ array. Count the distinct 32-row groups containing internal breaks, with exact treatment of F-symbol boundaries.

**Metric clarification needed:** The report that (d) is approximately ten times smaller than (r) conflicts with the above bounds if (d) denotes every adjacent-value discontinuity on a block with many runs. It may instead refer to the number of distinct affected groups.

**Traversal-clustering result:** Let (S) contain rows of nontranslation groups. Then consecutive slow visits are counted by

[\
A_2=|S\cap\Psi^{-1}(S)|.\
]

This can be evaluated through interval intersections because Ψ is piecewise translation. Higher-order correlations can be defined similarly, although their computation may become more expensive.

These statistics characterize logical traversal behaviour, not CPU stall timing.

**Research conclusion:** The relevant structural parameter for the fixed-group representation is not total BWT run count but the occupancy of Ψ discontinuities in aligned groups. The next analytical target is slow-path transition correlation; the remaining concurrency and performance questions require empirical evidence.

### Where I now stand

The coordinator was right to challenge the explanatory value of our original bound.

We have now replaced that loose bound with a much more precise account of how run boundaries appear in Ψ and how they affect the particular groups our representation uses.

The most important advance is that translation eligibility can be determined exactly from compressed BWT run information, before materializing a navigation table.

And the second useful result is that slow-path clustering can be studied as an intersection problem over translated intervals, rather than requiring us to regard the walk as a completely opaque permutation.

I would stop this research round here. Both questions have yielded concrete mathematical results, and the remaining uncertainty is not a lack of theoretical ideas. It's the difference between logical access structure and actual execution on the CPU.

That is an appropriate evidence boundary for the coordinator.
