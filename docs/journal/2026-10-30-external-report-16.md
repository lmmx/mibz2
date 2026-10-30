# External research report 16: research burn-down rounds 10–12, the structural, compact-Ψ and MLP clusters (verbatim)

A research input, not a finding of this repository. The maintainer relayed it on
2026-10-30, after external report 15 (`2026-10-30-external-report-15.md`) and the
maintainer's instruction to finish the burn-down before anything else. It has three
parts, in the order received.

1. **Round 10: #63–#69 and #71.**
   - **Result A:** an exact formula for pure 32-row groups from the run intervals.
   - **Result B:** a weak lower bound on impure groups from the run count, at least 900
     a block.
   - **Result C:** row-space and traversal-space summaries do not determine each other.
   - **Result D:** a segmented amortisation bound,
     an + S·Δ_Φ (#69).
   - **Result E:** #67 and #68 separated.
   - **#71:** load independence is a property of the address dependencies.
2. **Round 11: #37–#48 and #58.**
   - Ψ(j) = select_c(L, j − C[c] + 1), with its run form (#39, #46);
   - a claimed worst case of 100% impure groups, from an alternating last column (#58);
   - the boundary-information bound log₂ C(n−1, r−1) ≈ 38 KB (#37);
   - the interval-location trade-off common to #38, #43 and #44.
3. **Round 12: #2–#4, #77–#84 and P126–P133.**
   - lanes as an upper bound on outstanding successor requests (#4);
   - a window-capacity model (#3);
   - the non-transfer of CI amplification (#77, #78, #80, #82);
   - the timing fingerprint's identifiability (#81);
   - prefetch attribution (#2);
   - the USL as description, not extrapolation (P132).

The coordinator's measurements and checks are in `2026-10-30-research-portfolio.md`:
§3.1 holds Gate A's census and controls, and §9 holds the burn-down ledger. One
correction is recorded there. Round 11's alternating last column (ab)^{n/2} is not the
BWT of any text: its stable Ψ has several cycles of unequal length (3 cycles at n = 4,
14 at n = 64). The conclusion holds by a valid example instead: the bench's `random` and
`records` blocks measure 0.000 pure 32-row groups.

These are source material. Their claims are checked there, not taken as findings.

The text reached us as rendered text and is kept as received.

## 1. Round 10, as received

````text
## Research burn-down, round 10: settle the structural questions first

We are not moving to implementation yet. I agree that the 84-question register deserves a much more thorough examination, including questions unrelated to N1.

I've returned to the original question inventory in PR #117. The next cluster is #63–#69: BWT-run distributions, 32-row group occupancy, alignment, clustering and the validity of amortized navigation bounds under segmentation.

There is a useful mathematical result here: three of these questions are not independent. They are different measurements of the same ordered run-boundary sequence.

### Result A — exact 32-row occupancy from run boundaries

Let the maximal one-step translation intervals have row ranges

\\[ I_i=[s_i,e_i). \\]

Because mibz2's stable Ψ gives us precisely the L-run-derived intervals in F order, we already know these ranges from the run representation.

An aligned group of 32 rows is a pure translation group exactly when it lies entirely inside one interval.

The number of complete aligned 32-row groups contained in interval \\(I_i\\) is

\\[ \boxed{ P_i= \max\left( 0,\left\lfloor\frac{e_i}{32}\right\rfloor -\left\lceil\frac{s_i}{32}\right\rceil \right). } \\]

Thus

\\[ \boxed{P\_{32}=\sum_i P_i.} \\]

This is an exact formula, requiring no T-vector, no Ψ walk and no timing.

It establishes the following dispositions:

- \#63, average run length versus occupancy: average length alone is insufficient. The ordered run lengths and their cumulative alignment determine occupancy exactly.
- \#65, alignment modulo 32: its effect is directly calculable from the same intervals. We do not need a separate conceptual model.
- \#64, symbol-specific boundary patterns: each F-bucket contains the intervals corresponding to L-runs of that symbol, so the same calculation can be broken down by symbol without additional decoding.

The empirical parts—whether symbols differ systematically, and how much alignment matters on Wikidata—still require the corpus data. But the mathematical relationship is settled.

### Result B — the number of runs gives surprisingly weak occupancy guarantees

This explains why the old average-run-length reasoning was misleading.

For a 900,000-row block there are approximately 28,125 aligned groups of 32 rows. With approximately 56,000 one-step translation intervals, there are 55,999 internal translation boundaries.

A boundary can lie either inside a 32-row group or between two groups.

There are only 28,124 between-group positions.

So at least

\\[ 55{,}999-28{,}124=27{,}875 \\]

boundaries must lie inside groups.

Each 32-row group has at most 31 internal boundary positions. Hence at least

\\[ \boxed{\left\lceil\frac{27{,}875}{31}\right\rceil=900} \\]

groups must be impure.

But the actual preliminary occupancy figure indicates approximately 6,356 impure groups.

Both are compatible with the same run count.

The average run length tells us the overall boundary density; it does not determine how those boundaries are distributed among row groups.

That addresses part of #66 as well: a more repetitive block with fewer runs can nevertheless have worse aligned-group occupancy than another block with more runs, depending on how boundaries cluster and align.

This is not just a theoretical curiosity. It means a representation-selection heuristic based only on \\(r/n\\) can mispredict the number of exceptional records.

Research disposition: #63 is mathematically answered in the limited sense that the relevant predictor is the ordered boundary sequence, not the average run length. The empirical predictive strength of simpler summaries remains to be measured.

### Result C — the occupancy question and N1's gap question require different orderings

There are now two exact structural summaries of the same breakpoint set:

\\[ \text{Row-space structure}\longrightarrow P\_{32} \\]

\\[ \text{Traversal-space structure}\longrightarrow |C_k|=\sum_i\min(k,g_i). \\]

The first determines aligned-group purity for compact Ψ formats.

The second determines the number of multi-byte records for N1.

Neither summary determines the other.

This gives us a clean way to settle #66 without confusing two phenomena that earlier reports tended to merge:

A reduction in BWT runs can improve both structures, but it does not guarantee identical improvements in row-group occupancy and multi-step shortcut coverage.

It also gives the corpus census more scientific value. Once both summaries exist per block, we can determine whether an inexpensive row-space statistic predicts traversal-space shortcut complexity. Until that correlation is measured, the claim remains unestablished.

## Result D — #69: the full-cycle amortized guarantee does not automatically survive segmentation

Question #69 asks whether a full-cycle amortized bound for interval navigation remains valid when mibz2 divides traversal among 512 cooperative segments.

We can settle the general mathematical issue.

Suppose an interval-navigation algorithm has a potential function \\(\Phi\\) satisfying

\\[ c_t\le a+\Phi(x_t)-\Phi(x\_{t+1}), \\]

where \\(c_t\\) is the cost of a navigation step.

Over a complete cycle, the potential telescopes:

\\[ \sum_t c_t\le an. \\]

The starting and ending potentials cancel.

But for \\(S\\) separately processed segments, we instead obtain

\\[ \sum_t c_t \le an+\sum\_{s=1}^{S} \left[\Phi(\text{start}\_s)-\Phi(\text{end}\_s)\right]. \\]

If the potential has range \\(\Delta\_\Phi\\), this gives

\\[ \boxed{\text{segmented work}\le an+S\Delta\_\Phi.} \\]

For mibz2, \\(S\approx512\\).

This proves a useful point:

A whole-cycle amortized guarantee transfers to segmented traversal only with an additional boundary term—or a stronger invariant that cancels that term.

That is exactly the issue #69 was concerned about.

We can therefore classify the general question as answered mathematically, while leaving the representation-specific value of \\(\Delta\_\Phi\\) unresolved.

For the N1 eight-step records, this particular amortization objection is less relevant: each lookup advances a bounded number of output positions, and construction-time caps handle segment ends. Its work can be counted directly instead of relying on a full-cycle potential argument.

For older move-structure and interval-splitting candidates, however, the issue remains important.

## Result E — #67 and #68 are distinct optimization questions

The register asks:

- \#67: Can segment boundaries be chosen to make interval navigation cheaper?
- \#68: Can interval splitting cut occasional long navigation paths?

These are related but not equivalent.

Changing cooperative segment starts changes the traversal positions assigned to lanes. It may improve load balance or the timing of exceptional operations without changing the total number of expensive interval transitions.

Splitting an interval changes the representation itself. It may reduce the worst-case number of internal navigation operations, but increases metadata and possibly lookup work.

Therefore:

\\[ \boxed{\text{less critical-path work}\ne\text{less total CPU work}.} \\]

For our four-core objective, #67 should be evaluated mainly as a scheduling and critical-path question, while #68 needs a total-work accounting.

The existing #42 move-structure failure remains relevant: shorter navigation paths are not valuable if selecting the next interval costs more than the avoided steps.

Neither question is experimentally closed, but their acceptance criteria can now be separated without adding new hypotheses.

## A further distinction for #71: independence of loads is a property of addresses, not layout names

The register asks whether the low and high parts of a compact Ψ value can load independently.

There is a precise answer at the dependency-graph level.

If both addresses are calculable directly from the current row \\(j\\),

\\[ a\_{\mathrm{low}}=f(j),\qquad a\_{\mathrm{high}}=g(j), \\]

the CPU can issue the two loads independently.

If the second address depends on the first load's result,

\\[ a\_{\mathrm{high}}=g(j,\operatorname{load}(a\_{\mathrm{low}})), \\]

the loads are serialized.

This is a necessary architectural distinction, and it can be established for each proposed layout by inspecting its address calculations—before benchmarking anything.

But independent loads are not free: they still consume load slots, cache bandwidth and miss-handling resources.

So #71's feasibility is a code/layout question; its profitability is a measurement question. We should not leave both lumped together as one vague uncertainty.

## Updated burn-down sheet for this round

| Question                                | Disposition                                                     | Remaining uncertainty              |
| --------------------------------------- | --------------------------------------------------------------- | ---------------------------------- |
| #63 — Run length and occupancy          | Exact occupancy formula established; average alone insufficient | Empirical predictive power         |
| #64 — Symbol-specific boundary patterns | Exact per-symbol decomposition available                        | Corpus distribution                |
| #65 — Mod-32 alignment                  | Exact calculation from existing intervals                       | Measured alignment effects         |
| #66 — Repetitiveness and clustering     | Row-space and traversal-space effects distinguished             | Cross-block correlations           |
| #67 — Segment-boundary choice           | Scheduling mechanism isolated                                   | Actual critical-path gain          |
| #68 — Interval splitting                | Total work distinguished from path length                       | Cost of a particular layout        |
| #69 — Full-cycle amortization           | General segmented bound established                             | Potential range for each structure |
| #71 — Independent low/high loads        | Exact dependency condition established                          | Codegen and hardware payoff        |

No empirical outcomes were invented, and I haven't changed any register statuses on GitHub.

What we have done is reduce eight loosely formulated questions to three concrete evidence sources: the existing ordered L-run intervals, the Ψ-traversal gap distribution, and the address-dependency structure of proposed compact layouts.

That's the approach I'd continue across the other open clusters: resolve what follows from mathematics and existing evidence first, leaving only genuinely independent uncertainties for measurement.

The next unburned cluster with similar potential is #37–#48, the compact-Ψ representation and information-budget questions. In particular, I want to distinguish what can be ruled out by information bounds from what failed merely because a particular layout introduced an expensive dependent load.

We stay in research burn-down until the inventory is genuinely exhausted—not until one attractive candidate becomes ready to implement.
````

## 2. Round 11, as received

````text
## Burn-down round 11: compact Ψ, rank/select, and the limits of compression

The completed 233-block census changes N1's status: space feasibility has passed on the measured Wikidata corpus. It does not establish universal space bounds or CPU speed, but it removes a major reason for rejecting the candidate.

I'm returning to questions #37–#48 and #58, which are still recorded as open or partly resolved.

### Result 1 — Question #39 has an exact reduction to select on L

The fundamental inverse-BWT operation can be written without explicitly storing Ψ.

Let \\(C[c]\\) be the number of characters smaller than \\(c\\) in the BWT block. For a row \\(j\\) in the F-bucket of symbol \\(c\\),

\\[ \boxed{\Psi(j)=\operatorname{select}\_c(L,j-C[c]+1)} \\]

where `select` returns the position of the specified occurrence of \\(c\\) in L, using one-based occurrence numbering.

That identity is exact for mibz2's stable BWT permutation.

Now suppose L is stored as runs. For every character \\(c\\), let its successive runs have lengths \\(a_1,a_2,\ldots\\), starting at positions \\(p_1,p_2,\ldots\\).

Define cumulative counts

\\[ A_h=\sum\_{i=1}^{h}a_i. \\]

To find Ψ(j), choose the first run \\(h\\) with

\\[ A_h\ge j-C[c]+1. \\]

Then

\\[ \boxed{\Psi(j)=p_h+j-C[c]-A\_{h-1}.} \\]

So an alphabet-partitioned representation already gives the exact ingredients needed for one-step navigation:

- Identify the F-symbol bucket.
- Find the containing occurrence interval within that symbol's run list.
- Compute the destination by addition.

This answers the representational part of #39 and #46. F's contiguous buckets do allow a direct algebraic reduction to per-character select. But they don't eliminate the remaining predecessor/select search, which is the critical CPU expense.

It also explains why compressibility alone isn't enough. The representation may be tiny, yet still require an expensive search followed by a dependent descriptor read.

The outstanding question is no longer whether alphabet-partitioned select maps onto Ψ. It does, exactly. It is whether that lookup can compete with the shipped table's direct indexing.

### Result 2 — Question #58: the structural worst case is 100% exceptional groups

The register has measured FOR/EF exception rates on seven inputs, with rates at most 2% in those tests. But it still asks about the worst case.

We can settle the translation-purity worst case without another corpus.

Take an even-length BWT last column alternating between two symbols:

\\[ L=ababab\ldots \\]

Every L-run has length one.

The corresponding stable Ψ maps are translations only on singleton intervals. Every adjacent pair of rows has a one-step translation boundary, except where an accidental cross-bucket adjacency may preserve translation.

For \\(n\\) sufficiently large, every aligned group of 32 rows contains internal breaks.

Therefore:

\\[ \boxed{\text{Worst-case fraction of impure 32-row groups}=100\\%.} \\]

This is not the same as proving that every row requires a FOR-14 or EF-12 exception. Those encodings have their own numeric range and layout rules.

Disposition for #58: the worst-case translation-group impurity is answered. The worst-case exception rate of each proposed encoding still needs a layout-specific argument. The seven-input corpus maximum must not be presented as a worst-case guarantee.

### Result 3 — Question #37: the requested redundancy number has no meaning without an access-time model

The register asks:

> What is the minimum redundancy above \\(nH_0\\) for class 1–2 access to Ψ?

Here \\(H_0\\) is the zero-order entropy of L.

The theoretical issue is that \\(nH_0\\) describes the compressibility of a symbol sequence, not the cost of answering its select queries on a CPU.

There are compressed rank/select representations with asymptotic access guarantees, but those guarantees depend on their word-RAM operations, indexing structure and redundancy allowance. They do not imply that an actual query takes one or two cache accesses. The classical rank/select literature explicitly studies these space–time trade-offs.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://www.sciencedirect.com\&sz=32)

ScienceDirect



We can nonetheless derive a useful representation-specific lower bound.

Suppose a decoder stores a bitvector identifying the boundaries of \\(r\\) intervals among \\(n\\) rows.

There are

\\[ \binom{n-1}{r-1} \\]

possible arrangements of those boundaries.

An unrestricted encoding capable of representing every such arrangement must use at least

\\[ \boxed{\log_2\binom{n-1}{r-1}} \\]

bits in the worst case.

For approximately \\(n=900{,}000\\), \\(r=56{,}000\\), this is roughly 38 KB of boundary information.

That's much smaller than a straightforward 112.5 KB boundary bitmap, or the roughly 225 KB interleaved rank-directory representation discussed for N1.

But this is only a combinatorial boundary-encoding bound. It excludes displacement values, output words and the redundancy needed for fast queries. Also, not every arbitrary boundary arrangement need correspond to a valid BWT, so it is not a tight lower bound for the restricted family of BWT permutations.

The important deduction is:

Boundary compression is not our principal theoretical obstacle. Fast location of the containing interval is.

For N1, the measured descriptor payload and directory occupy less than the existing T-vector, so pushing boundary metadata toward its information-theoretic minimum may offer very little benefit if it adds another dependent memory access.

I'd retain #37 as an open time-space question, but remove the implication that minimizing redundancy above \\(nH_0\\) alone identifies a faster decoder.

### Result 4 — Questions #38, #43 and #44 share one unavoidable dependency

A run-native record gives:

\\[ \Psi(j)=j+D_i \\]

once the containing interval \\(i\\) is known.

So arithmetic eliminates the successor-value lookup, but not the interval-location problem.

A decoder must obtain \\(i\\) through one of three broad strategies:

- Compute it from the row using a direct directory.
- Search or select it from compressed boundaries.
- Carry interval state forward and update that state after each jump.

The first spends space, the second spends lookup operations, and the third makes interval maintenance part of the dependency chain.

This is the essential trade-off behind #38, #43 and #44.

A 64-byte fixed record may fit in one cache line, but that doesn't prove its lookup is comparable to `u32` direct indexing. It can still require multiple extraction instructions and an additional dependent load.

A translation descriptor needs no second payload lookup if its displacement is already in the selected record. That resolves the existence question in #44, but not its performance criterion.

Disposition: #44 is answered algebraically for interval records; #38 and #43 remain economically open.

## A more useful classification of this cluster

| Question                                   | Status after analysis               | Remaining decisive evidence                                  |
| ------------------------------------------ | ----------------------------------- | ------------------------------------------------------------ |
| #37 — Redundancy above \\(nH_0\\)          | Information-budget framing narrowed | Concrete access model and representation-specific redundancy |
| #38 — Run records                          | Exact arithmetic established        | Lookup cycles and total decoder cost                         |
| #39 — Alphabet-partitioned select          | Exact reduction established         | CPU cost of predecessor/select                               |
| #43 — Fixed-line versus direct load        | Open                                | Dependency depth and compiled instruction sequence           |
| #44 — Arithmetic successor                 | Possible and constructed            | Total cost of locating the interval                          |
| #45 — Hybrid or single format              | Open                                | Measured costs across block distributions                    |
| #46 — Exploit F-buckets                    | Exact select formula established    | Whether direct indexing can avoid select overhead            |
| #47–48 — MLP and space/throughput frontier | Open                                | Comparable CPU measurements                                  |
| #58 — Worst-case exceptional structure     | 100% impure groups possible         | Encoding-specific worst-case exception rates                 |

This round meaningfully narrows nine existing questions, without treating performance as settled by algebra.

The completed 233-block census gives us a strong empirical result for N1's storage gate. The remaining compact-Ψ questions should now be judged against that result rather than starting over with generic compression theory.

The wider lesson is that we have become very good at reducing the number of stored transitions. The hard problem is reducing the number and latency of operations required to find the next useful transition.

Next in the original inventory are #2–#4 and #77–#84: hardware prefetch, memory-level parallelism, the four-core versus i9 knee, and whether CI contention measurements predict real-machine performance. Those are worth burning down from the existing counter records and falsifiable hardware models, not by guessing new microarchitectural optimizations.

We remain in research mode. No implementation or workstation run has been initiated.
````

## 3. Round 12, as received

````text
## Research burn-down, round 12: memory-level parallelism and the reliability of our performance evidence

I've returned to the actual register, specifically #2–#4, #77–#84 and P126–P133.

The first important distinction is this: having 32 software walk lanes does not mean the CPU has 32 outstanding cache misses. And a slowdown under concurrent decoding does not, by itself, prove that memory-level parallelism has been exhausted.

These distinctions let us retire several overly strong interpretations without requiring new experiments.

### 1. Question #4 — 32 lanes versus 32 outstanding misses

In a cooperative Ψ walk, every lane has a dependency chain:

\\[ j_t\longrightarrow\operatorname{load}(\Psi[j_t]) \longrightarrow j\_{t+1}. \\]

One lane normally supplies at most one unresolved successor dependency at a time. With \\(L\\) lanes:

\\[ \boxed{M\_{\mathrm{successor}}\le L} \\]

where \\(M\_{\mathrm{successor}}\\) counts outstanding successor requests attributable to those lanes.

For \\(L=32\\), 32 is an upper bound on independently pending successor requests, not a prediction that 32 memory misses are in flight.

The actual number can be lower because of cache hits, prefetch completion, issue limits, fill-buffer limits and instructions between loads. Software prefetches also need separate accounting.

This means the original question can be split cleanly:

- Mathematical/architectural part: answered. Lane count bounds available independent chains but does not equal measured miss concurrency.
- Empirical part: still open. Actual outstanding miss occupancy needs compatible PMU evidence on the target CPU.

Crucially, the existing lane sweeps don't tell us the exact number of concurrent misses. They tell us which implementation performs better.

### 2. Question #3 — did the room walk free instruction-window capacity?

The room walk reduced per-step instruction overhead. That can allow more independent iterations to occupy a finite out-of-order instruction window.

But it need not increase useful memory-level parallelism.

Suppose the old walker required \\(u_0\\) in-flight micro-operations per lane and the new walker requires \\(u_1\\). If an effective instruction-window budget is \\(W\\), a rough capacity model is:

\\[ L\_{\mathrm{window}}\lesssim\frac{W}{u}. \\]

Reducing \\(u\\) may raise this ceiling.

However, usable concurrency is also bounded by memory request resources and the number of runnable independent chains:

\\[ \boxed{ L\_{\mathrm{effective}} \lesssim \min(L,\ L\_{\mathrm{window}},\ L\_{\mathrm{memory}}). } \\]

This is a conceptual resource model, not a measurement of the i9.

The register already records that some 8- and 16-lane alternatives were slower at high worker counts. That argues against assuming fewer lanes are preferable merely because the walker now uses fewer instructions.

Disposition for #3: the proposed mechanism is physically plausible but unproven. The existing results cannot distinguish instruction-window headroom from cache pressure or scheduling effects. Its remaining question is counter attribution, not another theoretical investigation.

### 3. Questions #77–#80 — CI contention is not a reliable substitute for the workstation's scaling curve

The register already contains enough evidence to reject one tempting assumption.

Its early-warning section reports that lbzip2's one-to-four-worker amplification varied from approximately 1.11× to 1.42× across runners. It also records different cache topology, processor generations and test conditions.

Thus a single contention ratio is not a stable property of a decoder.

Let \\(T_A(p)\\) be decoder A's throughput at concurrency \\(p\\), and define its amplification as

\\[ A_A(p)=\frac{T_A(1)}{T_A(p)/p}. \\]

An amplification measures how much throughput each worker loses relative to its own one-worker baseline.

But knowing \\(A_A(2)\\) does not determine \\(A_A(10)\\).

For example, two decoders can have identical two-worker amplification while one crosses the shared-L3 capacity at six workers and the other at ten.

This isn't a statistical subtlety. The two-worker observation simply contains insufficient information about where later cache-capacity transitions occur.

Disposition:

- \#78: the proposition that lbzip2 has one stable amplification baseline is contradicted by the existing measurements. It should be marked answered negatively.
- \#77 and #80: a two-core fingerprint is not mathematically sufficient to determine the ten-core knee or preserve decoder rankings across machines. A useful empirical predictor could still exist, but its validity has not been established.
- \#82: the recorded cross-run controls cannot be treated as matched comparisons unless their runner, decoder version and measurement protocol coincide.

That narrows this group substantially. A two-core CI result remains useful for discovering regressions on that environment, but not as a decisive filter for cache-sensitive alternatives intended for the i9 or four-core physical hardware.

### 4. Question #81 — the contention fingerprint cannot identify its own mechanism

The register explicitly asks whether a decoder's fingerprint reflects more memory-level parallelism or simply less computation per byte.

Consider a simplified worker time:

\\[ t=t\_{\mathrm{compute}}+t\_{\mathrm{memory}}. \\]

Two implementations might show the same one-worker and two-worker times even though:

- One reduces computation but leaves memory traffic unchanged.
- The other improves memory overlap while increasing computation.
- A third changes instruction scheduling, cache misses and prefetch traffic simultaneously.

The measured elapsed-time pair gives too few independent observations to distinguish these explanations.

This is an identifiability problem.

It means #81 has a definite methodological answer: a timing fingerprint alone cannot attribute the mechanism.

The remaining empirical question is which resource dominates the observed fingerprint on a particular machine. That requires independent measurements—such as instructions, memory requests, stall cycles or controlled changes to memory footprint.

The existing record doesn't justify selecting one mechanism over the others.

### 5. Question #2 — hardware prefetch mechanisms: separate overlap from attribution

The register records no controlled Intel prefetcher-on/off comparison. It also notes that Intel's adjacent-line prefetcher is a machine-configuration question, not a practical library-wide switch.

There is nevertheless an important logical distinction.

If software prefetch improves decoding, it establishes that the software-prefetch version performs better under the tested conditions. It does not establish which hardware prefetcher was inactive, which prefetches overlapped, or whether the benefit came from changes in instruction scheduling rather than only reduced miss latency.

Three different effects can coexist:

- Hardware prefetch predicts some accesses independently.
- Software prefetch requests data for future dependent iterations.
- Out-of-order execution overlaps the requests with useful instructions.

Those mechanisms are neither mutually exclusive nor individually identifiable from elapsed time.

So #2 remains open as hardware attribution, but there is no missing mathematical mystery to resolve. Its practical priority should be low unless a proposed algorithm depends on a specific prefetcher behavior.

For example, N1 does not need to assume Intel's prefetchers will recognize Ψ⁸ trajectories. Its relevant advantage is reducing the number of dependent navigation steps.

### 6. P132 — a three-point contention model cannot validate extrapolation

The register raises the Universal Scalability Law (USL) as a possible contention descriptor.

A common normalized form is

\\[ S(p)=\frac{p}{1+\alpha(p-1)+\beta p(p-1)}, \\]

where \\(\alpha\\) represents a contention term and \\(\beta\\) a coherency-like quadratic penalty in the model.

There is a straightforward limitation: fitting parameters to a small number of measurements does not validate the model's behavior outside that range.

Even a perfect fit at one, two and four workers cannot demonstrate the existence or location of a cache-residency knee at six, eight or ten workers. A workload may undergo a change in cache-hit probability that the smooth two-parameter model doesn't represent.

I would resolve P132's use as an extrapolation rule negatively. It can remain a descriptive summary of measured scaling, but it should not be used to rank unmeasured decoder designs or predict the i9's knee.

This is a decision we can make from the model's limitations and the existing evidence, without another benchmark.

## Round 12 disposition sheet

| Existing question                             | Research disposition                                    | Still requires evidence?                    |
| --------------------------------------------- | ------------------------------------------------------- | ------------------------------------------- |
| #2 — Intel hardware prefetchers               | Mechanism not attributable from current timings         | Yes, only if relevant to a design           |
| #3 — Instruction-window headroom              | Plausible mechanism; no causal attribution              | PMU evidence                                |
| #4 — 32 lanes vs 32 misses                    | Answered conceptually: not equivalent                   | Exact occupancy                             |
| #77 — CI amplification predicts i9 knee       | No guaranteed transfer; empirical predictor unvalidated | Paired multi-concurrency curves             |
| #78 — Stable lbzip2 amplification             | Answered negatively by recorded variation               | No, unless seeking a standardized reference |
| #80 — Two-core fingerprint preserves rankings | Not guaranteed; historical evidence insufficient        | Matched ranking comparisons                 |
| #81 — What causes the fingerprint?            | Timing-only attribution impossible                      | Independent resource measurements           |
| #82 — lbzip2/pbz2 controls                    | Old unmatched measurements insufficient                 | Matched controls only                       |
| P132 — USL extrapolation                      | Reject as a decisive extrapolator                       | No                                          |
| #84 — Fill-buffer headroom                    | Existing lane sweeps narrow its relevance               | Counter attribution remains                 |

This is not ten new questions. It's a classification of the original questions into what is answered, what cannot be inferred from existing evidence, and what would require a genuinely discriminating measurement.

## What this changes about our overall research direction

The memory-level parallelism cluster was in danger of becoming an endless investigation into CPU counters.

We can reduce it to one core decision:

Do we need to know the exact cause of the shipped walk's memory stalls to evaluate a fundamentally different decoder?

For N1, mostly no. Its essential claim is that one eight-byte transition can replace eight dependent one-byte transitions. That can be tested against the shipped path without first resolving every Intel prefetcher mechanism.

For incremental optimizations to the existing cooperative walk, those details matter much more.

That distinction tells us how to prioritize the remaining work without deleting legitimate open questions.

### Burn-down impact

I would recommend three register status changes based on this round:

- \#78: Open → Answered (negative). The existing amplification measurements vary enough to refute a stable universal baseline.
- \#81: Open → Answered at the identification level. Timings alone cannot distinguish computation reduction from greater memory overlap; architecture-specific attribution remains open.
- P132: Open → Rejected as a predictive extrapolation rule, retaining descriptive use.

I'd retain #2, #3, #77, #80, #82 and #84 as unresolved empirical questions with explicit prerequisites.

The next remaining substantial cluster is the front end and whole-decoder accounting: questions #11, #15, #19–#21, plus the unresolved scanner, fusion and library-versus-kernel gap. That area may have more direct potential for reducing actual CPU work than further study of contentious cache counters.

We're still burning down the original inventory. No implementation is needed to make these distinctions, and none of them changes the project's CPU-first objective.
````
