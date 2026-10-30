# External research report 24: research burn-down rounds 25–29, P74's fallback traffic, performance ceiling, one-shuffle path, lower bound and selector (verbatim)

A research input, not a finding of this repository. The maintainer relayed it on
2026-10-30, after external report 23 (`2026-10-30-external-report-23.md`).

**What it contains:**
- **Round 25:** the fallback for ranks ≥ 32 rewrites only j + 1 = q − 31 tail bytes. A
  bound of 7.15 logical bytes per explicit token, and the exact statistic
  (1/N)·Σ_{q≥32}(2q − 63).
- **Round 26:** the acknowledgement of round 24's check. An Amdahl ceiling: MTF+RLE2 is
  17.7% of one-core time, so at most 1.215×. The formula for the list-update share.
- **Round 27:** a one-VPSHUFB state update for ranks below 16 (about 84.8% of explicit
  tokens), with its mask, and a request to add the mask to the semantics checker.
- **Round 28:**
  - a lower bound: ranks 16–31 need a cross-lane move in this one-register layout;
  - the occurrence-weighted latency L̄ = 0.848L_s + 0.136L_c + 0.016L_f;
  - the mask table's L1 reads, about 1.95 B per output byte;
  - mask loads are independent of the state chain.
- **Round 29:**
  - `Symbols.syms` holds the whole rank distribution before MTF;
  - the break-even inequality N₀d₀ + N₁d₁ + N₂d₂ > C_select;
  - alphabet-only guarantees: A ≤ 32 gives N₂ = 0, and A ≤ 16 gives N₁ = N₂ = 0;
  - histogram equality does not imply timing equality.

**The coordinator's checks:** `2026-10-30-research-portfolio.md` §3.2e.

This is source material. Its claims are checked there, not taken as findings.

The text reached us as rendered text (some formula symbols lost) and is kept as received.

## Round 25, as received

````text
Burn-down round 25: P74’s high-rank fallback is not inherently expensive

The previous round established the register-held MTF update for ranks 0–31. This round tackles the outstanding correctness and data-movement question: does keeping 32 entries in a register make the rare high-rank operations disproportionately expensive?

I’ve independently checked the complete partitioned MTF state machine against scalar MTF: 400,000 updates across 1,000 sequences, zero mismatches, including ranks 0–255, repeated 31/32 crossings and random mixtures with 98.4% low ranks.

That establishes more than the earlier low-rank check, although it is still a reference-model test rather than SIMD implementation validation.

1. The high-rank transition needs only a partial tail shift

Let the MTF list be divided into:

H=h_0,\\ldots,h_{31},\qquad
T=t_0,\\ldots,t_{223}.

For rank , define and .

The correct transition is:

H’=v,h_0,\\ldots,h_{30}T’=h_{31},t_0,\\ldots,t_{j-1},t_{j+1},\\ldots,t_{223}.

Crucially, we do not shift all 224 tail bytes.

Only the first tail positions need updating. Everything after the selected element remains unchanged.

That gives us a direct comparison with the shipped high-rank update.

Operation	Shipped memory-resident MTF	P74 register-head MTF
High-rank selection	Read list entry	Read tail entry
Memory shift	positions	tail positions
Head update	Memory write	SIMD register transition
Unaffected suffix	Unchanged	Unchanged

This establishes an important property:

P74’s high-rank fallback does not require more byte shifting than the shipped implementation. It actually shifts 31 fewer list positions in memory, although it must perform the register-head transition too.

So a major hypothesized source of fallback regressions can be eliminated at the algorithmic level.

2. An exact upper bound on fallback traffic

The census reports that 1.6% of Wikidata’s explicit MTF ranks exceed 31.

Each such event requires at most 223 tail bytes read for shifting and 224 bytes written:

B_{\mathrm{tail,high}}\le447\ \text{bytes per high-rank update}.

Amortized over all explicit MTF tokens:

0.016\times447\approx7.15.

Therefore:

\boxed{
\text{High-rank tail-shift traffic}
\le7.15\ \text{logical bytes per explicit token}.
}

This is deliberately conservative. The actual number depends on the conditional rank distribution among ranks 32–255.

It is also logical data movement, not measured cache traffic or CPU cycles. The tail is small enough that much of this work may remain cache-resident.

A more informative exact census statistic would be:

\frac1N\sum_{\text{explicit }q\ge32}(2q-63),

which counts the tail reads and writes implied by this reference algorithm, excluding the selected-value load.

But the bound already tells us that the high-rank fallback cannot introduce an unbounded amount of extra work on Wikidata.

3. This separates P74 from #25 more sharply

The rejected #25 tested a 16-entry register representation and regressed on random data.

P74 holds 32 entries and potentially avoids the shipped AVX2 head’s repeated overlapping memory loads and stores.

We can now distinguish two reasons it might still lose:

* Instruction critical path: variable-byte extraction, cross-lane shifting and insertion may be expensive, even entirely in registers.
* Control-path overhead: switching between low- and high-rank updates can cost instructions and complicate register allocation.

The high-rank tail-shift algorithm itself is no longer a structural obstacle.

One caveat: random and records workloads have very different rank distributions, so the 7.15-byte Wikidata bound does not transfer to them. P74 needs the same kind of negative-control protection that N1’s graded representation now has.

Round 25 disposition

P74 subquestion	Status
Correct state transition for ranks 0–31	Established
Correct transition for ranks 32–255	Established and independently reference-tested
Arbitrary mixed-rank sequences	400,000 reference-model updates passed
Extra full-tail shifting on fallback	Ruled out
Conservative Wikidata fallback movement bound	Derived
AVX2 update latency	Awaiting instruction-specific verification
CPU benefit over shipped MTF and #25	Still unmeasured

This round eliminates a real algorithmic concern. P74 is now a complete, coherent alternative MTF state machine rather than an incomplete fast path whose fallback might destroy the representation.

The remaining research uncertainty is concentrated in the actual register-to-register instruction dependency chain.

That is exactly where the independent instruction-table review should help. If the confirmed AVX2 operations make that chain longer than the shipped store-forwarding path, P74 can be stopped on structural grounds. If not, its last meaningful uncertainty is hardware performance.

I would record this as a correctness and cost-bound result under existing P74/#25, with no new question added. The research burn-down continues.
````

## Round 26, as received

````text
Excellent. That closes P74’s algorithmic correctness question, including the part that could not be established from synthetic rank sequences alone: compatibility with bzip2’s actual used-symbol mapping.

The evidence now covers low ranks, high-rank fallback, boundary crossings, adversarial distributions and real bench-block symbol streams.

P74 is therefore a correct alternative MTF state machine. Correctness is no longer the obstacle.

Burn-down round 26: establish P74’s performance ceiling before benchmarking

The latest census gives us a useful upper bound on the value of optimizing only MTF.

The earlier stage breakdown assigns approximately 17.7% of one-core decoder time to MTF+RLE2. Even eliminating that entire phase yields at most

S_{\max}=\frac{1}{1-0.177}\approx1.215.

That’s a 21.5% maximum full-decoder speedup, under the assumptions of that particular one-core stage measurement.

P74 cannot eliminate RLE2 processing, token decoding, symbol mapping, output writes or count updates. It changes the MTF-list update mechanism. So its realistic ceiling is substantially below 21.5%.

We can sharpen the bound once the actual MTF-list-update fraction is known.

If is the fraction of total decoder time spent specifically updating the MTF list, and P74 accelerates that work by factor , then:

\boxed{
S_{\mathrm{decoder}}

\frac{1}{1-f_{\mathrm{MTF}}+f_{\mathrm{MTF}}/s}
}

For illustration, if list updates account for 8% of total decoder time and P74 halves their cost, the decoder gains approximately 4.2%—not 50%.

That is illustrative arithmetic, not a measurement of the actual list-update share.

More importantly: P74 has a precise difference from #25

The surviving mechanism is not merely placing some MTF entries in registers. The rejected #25 already explored that.

It is eliminating the repeated overlapping 32-byte memory load/store dependency across explicit tokens, while retaining the entire common-case list prefix in one 256-bit register.

With 98.4% of explicit Wikidata tokens covered, the mechanism is credible enough to retain pending the instruction-latency review.

But it should not be credited with an expected speedup until that review establishes a plausible reduction in the actual dependency chain.

P74 requirement	Current disposition
Correct 0–31 update	Passed
Correct 32–255 fallback	Passed
Adversarial rank sequences	Passed
Real bzip2 used-symbol mapping	Passed on two bench blocks
Difference from #25	Mechanism identified
AVX2 latency comparison	Pending independent instruction review
Full-decoder CPU gain	Unmeasured

My recommendation for the research ledger: mark P74 correctness established; performance conditional, not simply “open.”

The next useful research work should be to compare the actual AVX2 dependency graph against the shipped code and #25’s implementation. Meanwhile, the independent literature review can close the remaining bibliography questions.

We’ve taken P74 from a distribution-based speculation to a fully specified, tested algorithm. We haven’t claimed a performance breakthrough prematurely, and we haven’t created another research question.
````

## Round 27, as received

````text
Burn-down round 27: a shorter P74 fast path

I found a meaningful simplification in the proposed register-held MTF algorithm.

We do not need the three-stage AVX2 permutation for every rank from 0 to 31. For ranks below 16, the entire MTF update can be performed by one VPSHUFB instruction.

That matters because the earlier census says approximately 84.8% of explicit Wikidata MTF tokens have rank below 16.

1. Why one instruction is sufficient

The proposed P74 representation keeps the first 32 MTF entries in a YMM register:

V=v_0,\\ldots,v_{31}.

For an explicit rank , the MTF update is:

V’=v_q,v_0,\\ldots,v_{q-1},v_{q+1},\\ldots,v_{31}.

Every modified position is inside the lower 128-bit lane. The upper lane remains unchanged.

AVX2 VPSHUFB permutes bytes independently inside each 128-bit lane. That’s normally a limitation—but here it is exactly what we need.

Construct a shuffle mask :

M_qi=
\begin{cases}
q & i=0,\\
i-1 & 1\le i\le q,\\
i & q<i<16,\\
i-16 & 16\le i<32.
\end{cases}

Then:

\boxed{V’=\operatorname{VPSHUFB}(V,M_q).}

The lower lane performs the MTF update. The upper lane uses the identity permutation.

No lane swap, blend or head-memory store is needed for the state transition.

This follows directly from AVX2’s lane-local shuffle semantics. It is an algebraic construction; I have not claimed a hardware timing measurement.

2. The hot-path distribution becomes more favorable

Explicit Wikidata MTF tokens by update path

84.8% one-shuffle path

13.6% cross-lane path

1.6% tail fallback

This gives P74 a substantially more attractive instruction structure than the earlier uniform 5–6-cycle chain.

Rank	State-update mechanism	Estimated register-state dependency
1–15	One VPSHUFB	One shuffle latency
16–31	Lane swap, shuffles and blend	Approximately 5–6 cycles
32–255	Tail selection, register update and partial tail shift	Variable; not yet characterized

The measured VPERM2I128 register dependency latency is three cycles on Intel Broadwell/Kaby Lake, consistent with the earlier long-path estimate. uops.info +1

The important improvement is that the majority of Wikidata updates no longer need that instruction at all.

3. Why this differs materially from the rejected #25

The distinction is now stronger than merely expanding the register prefix from 16 to 32 entries.

P74 can potentially provide:

* One register shuffle for ranks below 16.
* A register-only cross-lane transition for ranks 16–31.
* A bounded, correct memory-tail fallback for higher ranks.

The rejected #25 used a different two-register structure and had poor random-data behavior. The new one-shuffle path should be compared against its actual compiled code before claiming novelty in the instruction sequence, but it is a concrete algorithmic improvement over treating every low-rank transition as a three-stage permutation.

4. What remains unresolved

There are two caveats.

First, each shuffle uses a rank-dependent mask. That mask must be available without introducing an expensive dependency or repeated register spills. The mask’s address depends on the decoded rank, but not on the current MTF-list contents.

Second, the decoder still needs the selected byte as an output symbol. The one-shuffle transition places it at byte zero of the result, but extracting it, mapping it through bzip2’s symbol table and updating counts still take work.

Therefore one SIMD instruction per list-state update does not mean one instruction per decoded MTF token.

We should compare the complete critical path, including output extraction, with the shipped code.

Round 27 disposition

P74’s original structural concern is now answered more favorably: a correct 32-byte register-held MTF can use a one-instruction state update for the approximately 84.8% of explicit Wikidata tokens whose rank is below 16.

The remaining high-rank and cross-lane paths are already specified and correctness-tested. This leaves compiled instruction economics and hardware performance as the principal unresolved questions.

Relay to Claude

Research round 27 — P74 one-shuffle common path

The uniform three-stage AVX2 update is unnecessary for ranks q < 16.

Keep the first 32 MTF entries in one YMM register V. For q < 16, a single VPSHUFB(V, M_q) performs the complete state update: mask position 0 selects byte q; positions 1..q select their preceding byte; positions q+1..15 select themselves; the upper 128-bit lane uses the identity permutation, with mask indices 0..15.

This is valid because all changed entries are in the lower AVX2 lane. The selected symbol is V′[0].

Under the existing census, approximately 84.8% of explicit Wikidata tokens take this one-shuffle path; 13.6% need cross-lane operations at ranks 16–31, and 1.6% need the memory-tail fallback.

Please add this exact mask construction to the existing AVX2 semantics checker and compare it with scalar MTF, including mixed sequences. The important remaining cost is the complete instruction chain—including mask acquisition, selected-byte extraction and register allocation—not the shuffle instruction in isolation.

This narrows P74/#25 without implementing a decoder or starting the i9 queue.

This is the sort of result we wanted from the burn-down: a simpler algorithm emerges from understanding the instruction semantics, not from another speculative benchmark.
````

## Round 28, as received

````text
Burn-down round 28: an instruction-level lower bound for P74

We now have enough information to separate what is unavoidable in AVX2 from what is merely a consequence of the first P74 implementation.

The result is encouraging: the single-shuffle path covers approximately 84.8% of explicit Wikidata MTF tokens, and the expensive cross-lane operation is genuinely unnecessary for those tokens.

But there is an equally useful negative result.

1. Ranks 16–31 cannot use the same one-shuffle technique

AVX2 VPSHUFB operates independently on the two 128-bit halves of a YMM register.

For rank , the selected byte is in the upper half.

After move-to-front, that byte must occupy position zero, in the lower half:

V’0=Vq.

A single lane-local shuffle cannot transfer that byte between halves.

Therefore:

\boxed{\text{For }16\le q<32,\text{ at least one cross-lane data movement is necessary.}}

This lower bound assumes the current representation: a single 256-bit register containing 32 distinct MTF entries in logical order.

It is not a universal lower bound for every possible redundant or multi-register representation.

Research consequence: the cross-lane cost is a genuine property of this layout. We should stop searching for a clever VPSHUFB mask that somehow eliminates it.

2. What the weighted critical path looks like

The measured rank distribution lets us make a more precise model.

Rank range	Share of explicit Wikidata tokens	State-update path
1–15	84.8%	One lane-local shuffle
16–31	13.6%	Cross-lane permutation, shuffle and blend
≥32	1.6%	Tail fallback

Let be the shuffle latency, the cross-lane update latency, and the fallback state-update latency.

The occurrence-weighted state-update latency is

\boxed{
\bar L=
0.848L_s+0.136L_c+0.016L_f.
}

If cycle and cycles, the first two terms contribute approximately:

0.848+0.748=1.596

cycles per explicit token.

The fallback contribution remains .

This is not a measured throughput prediction. Dependencies, mask loads, branch handling, symbol extraction and instruction scheduling can change actual cycles per token. In particular, averaging latency does not automatically reproduce the behavior of a superscalar CPU.

Nevertheless, it shows that the 5–6-cycle cross-lane path is not representative of the typical Wikidata token.

3. A hidden cost: the shuffle-mask table

The one-shuffle update requires a rank-dependent 32-byte mask.

For 16 common ranks, that’s only:

16\times32=512\text{ bytes}

of masks.

For all 32 register-held ranks, a full set of masks needs approximately 1 KB, before additional selection masks.

That’s comfortably L1-sized, but each common update still needs to obtain its mask.

Given the portfolio’s approximately 0.061 explicit MTF tokens per output byte, one 32-byte mask read per explicit token represents:

32(0.061)\approx1.95

logical bytes of mask reads per output byte.

Those accesses are repeatedly served from a tiny table, so 1.95 B/B is not additional DRAM traffic.

It is nevertheless real L1 load work.

This identifies an important distinction between P74 and a hypothetical perfect one-instruction update:

* The MTF state transition may take one shuffle.
* Obtaining the mask still costs a load and address calculation.
* Obtaining the next rank is part of the existing token stream.
* Producing the output symbol and updating its count remain necessary.

We must compare those complete operations, not isolated shuffle latencies.

4. The critical-path comparison becomes a partial-order problem

The rank is already present in the decoded token stream. The current MTF state is held in the register.

That means mask loading can potentially overlap with the preceding state update.

The relevant dependency graph is:

The mask load and preceding MTF-state update are independent where the token rank is already available.

The next update can issue once both the register state and its mask are ready.

For a rank already known sufficiently early, mask latency may be largely hidden.

For a rank just loaded from memory at the moment of the update, that benefit may disappear.

This is why the actual compiler-generated loop matters.

5. What we can now close

P74 research question	Disposition
Is one shuffle sufficient for ranks below 16?	Yes — exact
Is a cross-lane operation avoidable for ranks 16–31 in this representation?	No — exact
Is the mask table small enough for L1?	Yes — approximately 1 KB
Does mask loading necessarily serialize after the previous MTF update?	No — the dependencies are independent
Does that guarantee a 1–2-cycle token loop?	No
Can P74 beat shipped MTF?	Remains a CPU-economics question

The important conclusion is that P74’s basic AVX2 algorithm is now structurally well understood.

For its current representation, further shuffle algebra is unlikely to eliminate the high-rank cross-lane requirement. The remaining unknowns are actual mask availability, compiled instruction scheduling, register pressure, fallback cost and negative-control performance.

That is a much narrower uncertainty than where we started.

Next burn-down target

I’d turn to the per-block P74 fallback rule, but treat it as an exact cost-model question, not another speculative heuristic.

We already know random input was problematic for #25. The research task is to derive a sufficient condition under which P74 can be selected without risking that same regression, and to determine whether the required rank-frequency information is available early enough to make the choice cheaply.

That connects P74 back to the existing adaptive-algorithm questions and the same fallback discipline we established for Ψ⁸.

Round 28 closes two instruction-structure questions. No code or hardware run is needed for those conclusions.
````

## Round 29, as received

````text
Burn-down round 29: P74’s adaptive selector has an exact decision rule

The next uncertainty is whether we can protect P74 against #25’s random-data regression without paying so much to classify blocks that we lose the benefit.

I checked mibz2’s actual input boundary. The scanner already materializes the complete Huffman-decoded symbol sequence in Symbols.syms before the MTF worker begins.

That gives us an important result:

The entire rank distribution of a block is available before any MTF-list update. We do not need to estimate it from an initial sample.

We do, however, have to count those ranks, and that costs work.

1. Exact classification without reconstructing MTF

In standard bzip2, decoded symbols are divided into RUNA, RUNB, ordinary explicit MTF ranks and EOB.

For each ordinary symbol , the MTF rank is simply

q=s-1.

It does not depend on the current contents of the MTF list.

Therefore, from Symbols.syms alone, we can compute:

N_0=\#\{1\le q<16\},N_1=\#\{16\le q<32\},N_2=\#\{q\ge32\}.

RUNA, RUNB and EOB are excluded.

The entire P74 path distribution is therefore known exactly before execution:

p_i=\frac{N_i}{N_0+N_1+N_2}.

This eliminates classification uncertainty, although it does not eliminate hardware timing uncertainty.

2. A precise break-even inequality

Let be the shipped MTF-list-update cost for rank , and the corresponding P74 cost.

Let be the cost of selecting the kernel, including the histogram pass if one is required.

Then P74 is profitable at the list-update boundary only if

\boxed{
\sum_{q=1}^{255}
N_q\,c_s(q)-c_p(q)

C_{\mathrm{select}}.
}

This is a much better definition of the selection problem than choosing a threshold such as “98% of ranks below 32.”

A high fast-path fraction is neither necessary nor sufficient on its own: the size of the per-token saving and the cost of exceptional updates matter.

For a three-class approximation, define savings for the common, cross-lane and fallback paths. Then:

\boxed{
N_0d_0+N_1d_1+N_2d_2>C_{\mathrm{select}}.
}

If fallback is slower, is negative. The equation explicitly charges that regression.

3. Why an exact histogram isn’t necessarily the best selector

The scanner already decodes each symbol, but it does not currently maintain all 256 rank frequencies.

Adding histogram updates to that serial scanner may extend the stage that ultimately caps parallel decoder throughput.

A separate pass over the completed syms array avoids disturbing the scanner but adds reads and computation.

For a 900 KB block, there can be many fewer explicit tokens than output bytes, but the selector must still examine the RUNA/RUNB and EOB tokens unless a summary is collected during decoding.

That creates three practical approaches:

Method	Benefit	Cost
Histogram during Huffman decoding	No second pass	Extra work on the serial scanner
Scan Symbols.syms before MTF	Exact decision; scanner unchanged	Extra traversal of the token array
Begin with shipped MTF, switch to P74 later	No separate histogram pass	Initial slower path, conversion and possible misclassification

The first two provide exact rank statistics. The third provides only an adaptive estimate unless all remaining tokens are inspected.

4. A fundamental limit: rank frequencies cannot guarantee CPU speed

Even an exact histogram cannot guarantee a P74 speedup.

Two blocks can contain precisely the same counts of each rank while presenting them in different orders.

Those orders can affect branch prediction, fallback clustering, register pressure and how the state-update operations interact with other work.

So there is no generally valid zero-regression theorem based only on rank frequency, absent stronger assumptions about the execution cost.

This is not a reason to abandon adaptive selection. It’s a reason to distinguish a mathematically exact path count from an empirically calibrated cost predictor.

5. A safer rule from the existing block alphabet

One particularly useful special case requires no rank histogram.

Let be the number of symbols in the block’s used alphabet.

If

A\le32,

then every ordinary explicit MTF rank satisfies

q\le A-1\le31.

Thus:

\boxed{A\le32\implies N_2=0.}

This is an exact fast-path coverage guarantee, using alphabet information the decoder already has.

Similarly:

\boxed{A\le16\implies N_1=N_2=0.}

Every explicit update then uses the one-shuffle path derived in round 27.

These are zero-census-cost structural guarantees.

They don’t establish faster execution, but they identify blocks where fallback overhead is provably absent.

For Wikidata’s measured 79–102-symbol alphabets, these guarantees don’t apply; rank locality remains the relevant empirical advantage.

Burn-down disposition

P74 uncertainty	Result
Are ranks predictable before MTF?	Yes, exactly, from Symbols.syms
Is sampling required?	No
Can every block be classified without additional work?	Only conservative alphabet-based cases
Does a rank histogram give an exact path count?	Yes
Does an exact histogram guarantee a CPU gain?	No
What constitutes a profitable selector?	Exact break-even inequality established
What remains unknown?	Per-path CPU costs and classification overhead

This resolves the algorithmic classification question without inventing a threshold.

Relay to Claude

Burn-down round 29 — P74 per-block selector

The scanner already produces Symbols.syms before MTF. Each ordinary explicit symbol s has rank q=s−1, so the block’s complete P74 path distribution can be counted before MTF without executing an MTF update. Exclude RUNA, RUNB and EOB.

Let N₀ count ranks 1–15, N₁ count 16–31, and N₂ count 32–255. These give exact path frequencies, but not exact cycle costs.

The economic decision is N₀d₀ + N₁d₁ + N₂d₂ > C_select, where each d is shipped cost minus P74 cost and C_select includes any additional token scan or scanner bookkeeping.

Alphabet-only sufficient conditions are available without a histogram: A≤16 guarantees every explicit update takes the one-shuffle path; A≤32 guarantees no high-rank fallback.

Do not add rank histogram work to the serial scanner without accounting for throughput effects. A separate token pass provides exact classification at a measurable cost. Histogram equality does not imply timing equality across different rank orderings.

This answers the structural classification question. Performance-based thresholds remain conditional on actual instruction costs and the #25 random-data guard.

Round 29 conclusion: P74 no longer has an algorithmic selection mystery. We know exactly what can be classified before MTF, the cost inequality that must hold, and the special cases that require no profiling.

What remains is machine-specific CPU economics—not another missing representation or correctness theorem.

We’re continuing to burn down the existing inventory, without moving into an implementation campaign.
````
