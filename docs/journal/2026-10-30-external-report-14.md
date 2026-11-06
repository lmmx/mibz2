# External research report 14: the researcher's burn-down summary for 30 October 2026 (verbatim)

A research input, not a finding of this repository. The maintainer relayed it on
2026-10-30, after external report 13 (`2026-10-30-external-report-13.md`).

**What it contains:** the researcher's summary of the day's burn-down.
- **The inventory, unchanged:** 84 questions (31 answered, 6 conditional, 47 open), 165
  ideas and 63 contradictions. The register has not yet been recounted against the
  day's results.
- **N1's state, step by step,** from the corrected record definition to the open
  economic test.
- **A ranking** of the remaining work.
- **Clusters** of the 47 open questions. The researcher estimates that about 8–12 have
  been substantially narrowed but not closed.
- **A split** into what is closed mathematically, what is substantially narrowed, and
  what is still open.
- **The next sequence:**
  1. finish the census;
  2. the builder's byte volume and peak memory;
  3. the segment-cap audit;
  4. novelty, then a decision on whether N1 has earned a tightly scoped implementation
     experiment.

**Where its items are answered:** in `2026-10-30-research-portfolio.md` §3.1.
- **Items 1–3:** the full census, the builder model, the segment-cap invariant.
- **Item 3 in particular:** the invariant was checked by brute force after this
  summary's source text was written; see report 13.

A follow-up message is appended after the summary. In it the researcher moves the
segment caps out of the speculative category and asks for the walker's executed lookup
count H = Σ h_s alongside the resident record count. The coordinator's answer, checked
by brute force:
- under construction-time caps, a lane reaches a capped row only on its final hop;
- so H = Σ_s ⌈ℓ_s/8⌉ exactly, which is less than n/8 + 512;
- every one of the 10,188 walked segments took exactly ⌈ℓ/8⌉ lookups.

This is source material. Its claims are checked there, not taken as findings.

The text reached us as rendered text (some chart and card labels flattened) and is kept
as received.

## The summary, as received

````text
# mibz2 research burn-down — 30 October 2026

Mathematical construction established

# We've moved from searching for a mechanism to evaluating one.

The strongest progress is on N1: multi-step Ψ inversion. We now have a mathematically justified eight-byte decoding representation, an exact record-count formula, a construction that avoids general sorting and per-record successor searches, and preliminary measurements on real Wikidata blocks.

But we have not yet demonstrated a faster decoder. The two major unresolved performance questions are the cost of building the records and the cost of looking them up.

## 1. The burn-down numbers

Original 84-question inventory

Answered 31

Conditional 6

Open 47

# 84

Deduplicated questions in the register

# 165

Registered ideas and hypotheses

# 63

Historical contradictions catalogued

Those are the official inventory counts, not an updated closure tally. I checked the current accessible PR #117 branch; the register has not yet been retallied against all our recent results.

What has materially changed is the depth of the answers. Several questions previously marked open now have a mathematical solution, leaving only implementation-cost or measurement conditions.

I would not simply subtract ten or fifteen from the 47 open questions: some of our breakthroughs answer subquestions of an existing entry rather than closing that entry entirely.

## 2. N1: where we've made the biggest advance

Multi-step Ψ inversion

Leading candidate

1. Define the correct records — resolved

   A record gives a constant \\(k\\)-byte output word and a \\(k\\)-step successor translation. The false claim about Ψᵏ's own breakpoints was corrected.
2. Count the records exactly — resolved

   The minimal output-producing partition satisfies

   \\[ |C_k|=\sum_i\min(k,g_i) \\]

   for the stable Ψ used by mibz2, with the appropriate endpoint convention.
3. Establish space feasibility — promising empirical evidence

   Three full Wikidata blocks had approximately 76–85k four-step records and 99–113k eight-step records. The latter use roughly 1.2–1.4 MB of descriptor payload, versus 3.6 MB for the current T-vector.
4. Construct multi-step descriptors — algorithmically resolved

   Build level-one records from L-runs, then compose levels 1→2→4→8 using intersections of source and image interval partitions.
5. Avoid sorting and random destination searches — resolved

   The corrected counting construction uses two merge passes, parent counts, prefix sums and scatter. Destination offsets are accumulated in the second parents' image order, not source order.
6. Handle cooperative segment termination — design identified, verification outstanding

   The shipped walk discovers segment ends through flagged rows. Construction-time capping is the proposed solution; my earlier known-length tail scheme was invalid.
7. Beat the shipped decoder — open

   Construction cycles, rank-directory lookup latency, full-decoder throughput, CPU-seconds/GB and peak RSS are not measured.

This is the clearest picture of our progress. At the beginning, we had an idea about composing translations. Now we have a concrete algorithm family with a short, falsifiable list of remaining costs.

## 3. What the first real-block figures suggest

Indicative records per 900 KB Wikidata block

Low end

High end

030K60K90K120KΨ⁴Ψ⁸

Preliminary ranges from three full blocks; not the completed 233-block census.

The 233-block census is still running according to Claude's latest report. These are measured preliminary ranges, not a corpus-wide result.

These figures support three conclusions:

- Space is substantially more encouraging than the worst-case bound.
- Breakpoints cluster in traversal order, not merely adjacent-row order.
- Doubling does relatively little splitting: using the paired counts, a substantial fraction of four-step parent intervals survive unchanged as eight-step intervals.

But the design could still lose completely if constructing 100,000–110,000 descriptors and querying the rank directory costs more than today's direct T-vector build and walk.

## 4. The remaining research, ranked by impact

| Priority | Existing work                        | State             | What would settle it                                                                |
| -------- | ------------------------------------ | ----------------- | ----------------------------------------------------------------------------------- |
| 1        | N1 builder economics                 | Open              | Actual per-level bytes moved, scratch RSS, cycles                                   |
| 2        | N1 Ψ⁸ walker                         | Open              | Rank-plus-record dependency cost against shipped walk                               |
| 3        | Construction-time segment caps       | Conditional       | Correctness across all flagged segment boundaries and cap overhead                  |
| 4        | N3 run-native pipeline               | Conditional on N1 | Eliminate L materialization and T-vector construction without increasing total work |
| 5        | T2: MTF token census / P74           | Open              | Eligible batches and avoidable store-forwarding stalls                              |
| 6        | N0: application ceiling              | Open              | Decoder-free wikidata-pq split measurement                                          |
| 7        | T3/T5: occupancy and scanner markers | Open              | Existing trace analysis and bitstream statistics                                    |
| 8        | T4: literature novelty               | Open              | Move-r, Movi, SEA 2026 and KKP construction comparison                              |
| 9        | Architecture questions               | Conditional       | Paired four-core and i9 tests after a viable design exists                          |

The ordering is deliberate. We've already invested considerable mathematical effort in N1. It has now earned a proper economic test.

I would not divert the primary effort into old compact-table variants, MTF micro-optimizations or thread policy until that test tells us whether N1 can deliver a genuinely large gain.

## 5. How much of the broader 84-question list are we actually reducing?

The 47 still-open questions are not 47 independent scientific mysteries.

They fall into a smaller number of clusters:

| Cluster                                       | Existing question numbers           | Overall status                                      |
| --------------------------------------------- | ----------------------------------- | --------------------------------------------------- |
| Ψ structure, compact navigation, construction | #37–39, #43–48, #51, #53–54, #63–76 | Major mathematical progress; performance unresolved |
| MLP, prefetch, scheduling and memory pressure | #2–4, #15, #19–21, #77–78, #80–84   | Mostly machine-specific measurement                 |
| Front-end and fusion                          | #11, plus P74/P78 and N2            | Mechanisms understood, costs unresolved             |
| Literature and external methods               | #25, #28–33, #36                    | Mostly reading and novelty verification             |
| Remaining corpus and portability questions    | #58 and selected others             | Corpus-dependent or architecture-specific           |

My estimate is that roughly 8–12 of the recorded open questions have been substantially narrowed by the recent Ψ work, but not necessarily closed according to their original acceptance criteria.

That's a research assessment, not a revised official tally.

A useful distinction is emerging:

MATHEMATICAL QUESTIONS

## Fewer left

Exact partitions, record bounds, interval-image tiling and construction existence are now understood.

PERFORMANCE QUESTIONS

## Still decisive

CPU cycles, memory-dependency latency, construction traffic and full-decoder behavior remain unmeasured.

## 6. What I would consider genuinely closed versus still conditional

Closed at the mathematical level: the wrong original identity; the correct minimal record partition; the gap-counting relation; interval-image tiling; no-search descriptor composition; no-sort two-view construction with Claude's ordering correction; and the \\(2m-1\\) doubling bound.

Substantially narrowed: whether Ψ⁴/Ψ⁸ records fit; whether construction can avoid the T-vector; whether intermediate Ψ navigation is necessary to emit output words; whether multi-byte inversion can coexist with cooperative segmentation.

Still wide open: whether this algorithm beats mibz2, whether it reduces CPU-seconds/GB on four-core hardware, how much temporary RSS it uses, and whether the run-native integration delivers its projected decoder-level gains.

The existing Ψ², move-structure and compact-Ψ failures remain relevant evidence. We haven't invalidated those experiments; we've developed a materially different construction and lookup path that those experiments did not test.

## 7. The immediate burn-down sequence

First: finish and reconcile the 233-block census, including \\(m_1,m_2,m_4,m_8\\), child counts, gap survival curves and space distribution.

Second: calculate exact builder read/write volume and peak temporary memory under the corrected two-pass counting construction.

Third: audit the proposed construction-time segment capping against the actual flagged-walk invariants.

Fourth: verify novelty and then decide whether N1 has earned a tightly scoped implementation experiment. No i9 queue launch is implied.

## Bottom line

Our burn-down is stronger in substance than in question-count statistics.

We began with 165 ideas, 84 recorded questions and an uncertain hypothesis about Ψ composition. Through independent proofs, counterexamples, exhaustive checks and actual Wikidata measurements, we've turned our leading hypothesis into a complete mathematical construction with a plausible small representation.

We're now concentrated on two decisive unknowns:

\\[ \boxed{\text{Can we build it cheaply?}} \\]

\\[ \boxed{\text{Can we walk it cheaply?}} \\]

If both answers are yes, N1 could substantially alter the decoder's cost model. If either is no, we'll have enough evidence to reject the implementation economically without discarding the underlying mathematics.

That's exactly where I want a breakthrough-oriented research programme to be: fewer speculative mechanisms, more hard results, and increasingly decisive tests.

The relevant live record is [PR #117](https://github.com/lmmx/mibz2/pull/117?utm_source=chatgpt.com). I haven't changed it or launched any experiments.
````

## The researcher's follow-up on segment capping, as received

````text
That's another important correctness result. Construction-time segment capping now has independent validation, so we can move it out of the speculative category.

The 10,188 randomized segment-start checks establish that, in the tested cases, capped Ψ⁸ records reproduce each segment's bytes and terminate at the correct flagged row. The observed maximum of five extra records per start is also encouraging, although the theoretical bound of 14 remains the safer planning figure.

I'd record the state of N1 as follows:

| Component                          | Status                                         |
| ---------------------------------- | ---------------------------------------------- |
| Minimal output-producing partition | Mathematically established                     |
| Exact record-count identity        | Established and cross-checked                  |
| Level-doubling composition         | Established and randomized-tested              |
| No-sort, no-search construction    | Established, with corrected ordering           |
| Cooperative segment capping        | Randomized correctness checks passed           |
| Eight-step representation size     | Promising on three blocks; full census pending |
| Builder CPU cost                   | Unknown                                        |
| Walker CPU cost                    | Unknown                                        |
| Full-decoder performance           | Unknown                                        |

### One further deduction worth preserving

The segment-capping result has a consequence for how we evaluate the walker.

We should distinguish the number of records stored from the number of record lookups actually performed.

Those are not interchangeable:

- The final record count determines representation size and much of construction cost.
- The number of executed shortcuts determines navigation work.
- Segment caps can increase the latter without substantially increasing the former.

For each segment, let \\(h_s\\) be the number of capped-record operations needed to reach its terminal marker. The actual lookup count is:

\\[ H=\sum\_{s=1}^{512}h_s. \\]

The relevant quantity for performance is:

\\[ \boxed{\text{mean output bytes per record lookup}=\frac{n}{H}.} \\]

For a perfect eight-byte walker, that approaches eight. Capping, shorter terminal records and any implementation-specific bookkeeping reduce it.

This is more informative than simply saying that the segment-boundary overhead is bounded by 3,584 bytes.

Claude's correctness checker already counts record operations, so it may be able to report \\(H\\) without another expensive analysis.

That would give the future walker cost model a measured dependent-lookup count, rather than assuming exactly one lookup per eight decoded bytes.

I would include three values in the final census report: resident record count, construction record traffic, and executed capped-record lookups per output byte. Together, they connect the mathematical representation to the CPU cost model.

The larger conclusion is unchanged: we've removed another correctness obstacle without altering the shipped cooperative design.

We are now very close to having a complete, falsifiable economic model of N1. The full corpus census and measured construction costs will tell us whether that mathematical progress translates into a credible performance opportunity.
````
