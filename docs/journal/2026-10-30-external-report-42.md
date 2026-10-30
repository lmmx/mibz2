# External research report 42: the guardian and the researcher on the selection architectures (verbatim)

A research input, not a finding of this repository. The maintainer relayed it on
2026-10-30, after the read-only assessment (`2026-10-30-p74-selection-architecture.md`).
The guardian's text holds two messages, separated by "…". The first predates the selector
arm's result.

**What it contains:**
- **Both favour closing PR #118** under the negative verdict, and stopping P74
  implementation work.
- **B (scanner counting) deprioritised** by both.
- **A (AVX2 census):** the stronger next experiment, if any.
- **C (online switch):** the stronger candidate for analysis. It needs a bounded-loss
  rule (δ·p/n plus the switch and classification costs), short-block economics, handling
  of prefix misclassification, an exact handoff boundary, and E's per-token cost.
- **No experiment authorised.** N1 gets an independent priority decision.

**The coordinator's notes:**
- The assessment's reading is corrected: C has the lowest classification overhead in
  principle, which doesn't yet make it the cheapest design.
- **PR #118 stays open:** both texts recommend closing it, subject to the maintainer's
  own authorisation, which hasn't been given.

The text reached us as rendered text (tables and formulas flattened) and is kept as
received.

## The messages, as received

````text
the guardian says:

I agree with the researcher, and I think we’re now interpreting P74 at exactly the right level of caution.

The preliminary result is neither a breakthrough nor a failure. It’s evidence of a potentially valuable specialization with a potentially very expensive fallback.

The distinction between workload dependence and its underlying cause is especially important. We know that random data exercises high-rank MTF operations much more frequently, but we don’t yet know how much of the regression comes from vector-tail shifts, register handling, branches or RLE2 behavior.

There is, however, one additional issue I’d watch when the CI results arrive.

The selector must preserve the checkpoint-kernel result

The registered mixed-case procedure introduces a per-block selector. The relevant comparison isn’t simply whether selection protects random and records from regression.

It must also preserve enough of P74’s Wikidata checkpoint-stage improvement after paying for classification.

That is potentially harder than it appears because the selector operates on the MTF token stream before decoding, and counting ranks adds an entire pass over those tokens unless the information is already collected elsewhere.

So there are two independent economic questions:

* Can the selector avoid sending fallback-heavy blocks through P74?
* Can it do so cheaply enough that the Wikidata gain survives?

The first looks plausible from the dramatically different rank distributions. The second remains unmeasured.

A small statistical caution

I’d also keep the CI decision tied to the ten paired rounds we authorized.

An 8–12% local speedup is encouraging, but the actual registered decision depends on the paired CI estimates, confidence intervals, layout control and negative-corpus limits.

The selector should only be evaluated if the registered mixed condition is established. We shouldn’t introduce it just because the preliminary results make it attractive.

What would constitute a meaningful success?

If P74 ultimately passes with the selector, it would establish that the decoder can profitably select between different MTF implementations according to block characteristics.

That would be a useful architectural result, even if the absolute full-decoder gain is modest.

But I wouldn’t describe it as proving that the register-resident kernel has universally lower computational cost. It would be a conditional performance improvement.

And if the selector fails, we should close the experiment without further tuning, preserving the positive Wikidata result as a conditional finding.

For now, my recommendation is unchanged: let Claude finish CI, follow the frozen decision tree, and bring us the raw paired results.

The preregistration has already protected us from making an emotional decision in either direction.
…

I agree with the researcher, and I think its latest assessment improves our decision-making in one important respect: Option C has the greater architectural upside, but Option A currently has the stronger experimental case.

Those are different things. We shouldn’t confuse the most elegant design with the most promising next measurement.

My assessment

	A: AVX2 census	C: Online switch
Main advantage	Exact classification, straightforward integration	No separate scan
Main weakness	Classification still costs memory traffic	Wrong-kernel prefix and state handoff
Evidence	Gate 1 identified a specific, measurable counting bottleneck	Architectural hypothesis only
Primary risk	Exceeding the 3% control budget	Misclassification, transition overhead, short blocks
Research priority	Strong candidate for a narrow future experiment	Strong candidate for analytical investigation

I agree that B should be deprioritized. Adding work to the serial scanner could increase the critical-path cost even if it makes the MTF stage look cheaper.

One issue I’d sharpen in Option C

The researcher’s approximate loss model,

\text{regression}\approx\delta\frac{p}{n},

is useful for a fallback-heavy block if the selector switches after tokens and the remaining work runs at shipped-kernel speed.

But it omits the fixed switching overhead and the possibility that the prefix is unrepresentative.

A more complete first-order model is:

\text{relative overhead}
\approx
\delta\frac{p}{n}
+
\frac{C_{\text{switch}}+C_{\text{classification}}}{T_{\text{shipped}}}.

Even that is conditional: it doesn’t capture a rank-local prefix followed by a fallback-heavy suffix that never triggers another switch.

I’d therefore question whether C should use a one-time decision at all. An online monitor with a bounded sampling budget might detect later distribution changes. But that introduces its own overhead and a new decision policy. It’s a research question, not something to add to the current implementation.

There’s also an important distinction between an exact classifier and a safe classifier. A’s whole-block census implements the registered threshold exactly. It does not prove that 10% is the optimal performance boundary for every possible block.

Where I differ slightly from Claude

Claude called C the most economical design. That’s premature.

What we’ve actually established is:

* A needs a faster implementation of a known decision rule.
* C needs a decision rule, transition mechanism and cost model.
* Neither has demonstrated that it can satisfy the full-decoder CPU-efficiency requirements.

If we ultimately choose the next performance experiment based on confidence of yielding an interpretable answer, I’d currently favor A.

If we choose the next research investigation based on potential for a new architectural mechanism, I’d favor C.

That division keeps both possibilities alive without starting another P74 tuning campaign.

What I recommend now

I would explicitly authorize closing PR #118 under its registered negative verdict, then stop P74 implementation work.

The read-only feasibility assessment of C that we discussed remains worthwhile, particularly its state handoff, short-block economics and response to changing rank distributions. But I would not authorize either A or C as an implementation experiment yet.

And I’d make an independent priority decision about N1. We shouldn’t allow incremental P74 refinements to indefinitely displace the more ambitious Ψ construction research.

The lasting P74 finding is strong: on the tested Zen 4 and Zen 5 CI machines, the branch-free register-held implementation substantially improves Wikidata’s MTF stage. Its general-purpose deployment failed because the fallback and selection economics did not satisfy the registered controls.

That is a negative engineering gate with a positive architectural discovery—not a reason to reopen the frozen experiment.

===

the researcher says:

Yes. I have a fairly strong research recommendation here, and it differs slightly from Claude’s preference.

I would close P74 Gate 1 formally, retain P74 as a proven workload-specific optimization opportunity, and investigate Option C analytically before authorizing another benchmark.

I would not immediately authorize an AVX2 counting experiment, and I would not move the count into the serial scanner.

The distinction is important: Gate 1 has failed its registered acceptance criteria, but the two CI runs uncovered something valuable enough to preserve.

My assessment of the three options

Approach	Research assessment	Recommendation
A. Standalone AVX2 census	Smallest change; preserves exact whole-block classification, but still rereads the encoded symbols	Credible fallback
B. Scanner-side census	Eliminates the extra pass but adds work to the serial pipeline stage and changes the API	Deprioritize
C. Mid-block adaptive switch	Potentially eliminates the census entirely, but introduces new correctness and prediction questions	Best candidate for read-only research

Claude’s conclusion that C is the most economical design is plausible. It is not yet demonstrated that C is the most economical decoder.

There are several reasons I would examine it before spending more CI time.

The central opportunity in C

The first selector’s weakness was that it paid to classify every symbol, including on blocks that ultimately used shipped MTF.

A mid-block selector could instead use information gathered during normal decoding, ideally only along a path that already has to process high ranks.

If a block is fallback-heavy, its classification becomes obvious quickly. If it’s rank-local, there is little classification work to perform.

That is an attractive asymmetry.

But C’s chief risk isn’t classification overhead. It’s the cost of making the wrong decision during the prefix, and possibly paying for a state conversion after doing considerable work in the slower representation.

The catch with Option C: the prefix must pay for itself

Claude’s read-only assessment is useful, and I agree that C deserves serious investigation. But I would challenge one assumption before declaring it the best design: avoiding an extra scan does not automatically make online switching cheaper than a vectorized whole-block census.

The online scheme deliberately processes the beginning of every block using P74. On a fallback-heavy block, that means knowingly executing the slower kernel before switching.

We can bound the damage.

Let be the block’s explicit-token count, the number of explicit tokens processed before switching, and the relative cost disadvantage of P74 on that workload.

Ignoring conversion and decision overhead, the fractional regression is approximately

\boxed{\delta\frac{p}{n}}.

For a 900,000-token random block, a 20% P74 penalty over an initial 16,000-token probe would contribute approximately 0.36% to the MTF cost, comfortably below the 3% control threshold.

But that bound deteriorates on short blocks. If a block contains fewer than 16,000 explicit tokens, the probe might consume the entire block before making a decision.

This leads to three research requirements for C.

1. A bounded-loss decision rule

A fixed-size probe is not enough by itself. The design needs to account for short blocks and specify when switching is no longer worthwhile.

More importantly, a prefix histogram cannot guarantee that the remainder of the block has the same distribution. A rank-local prefix followed by a random-like suffix could make C keep P74 on an expensive remainder.

So C offers excellent potential performance on the measured corpora, but it is not a universally safe selector merely because its probe is cheap.

2. Exact state conversion

The head and tail can be materialized as the shipped 256-byte list. That part follows from P74’s correctness proof.

However, switching mid-loop must preserve more than the list: the current symbol cursor, pending RLE2 state, output position, cumulative counts and checkpoint boundary all have to remain coherent.

The simplest proof obligation is to switch only at a point where the current explicit-token update and any pending run have been fully consumed. Then the shipped kernel can resume from the same logical state.

This is a new integration question, but not a new MTF-theory problem.

3. Correct accounting for the classifier

Claude describes as a counter incremented only inside the slow fallback branch. That is attractive.

But , the explicit-token count, still needs to be maintained for the threshold. If that requires additional work on every explicit token, C is not literally free of classification cost.

Its advantage is that the count can stop once the decision is made, rather than requiring a pass over the entire encoded block.

A comparison I would preserve for the Guardian

	Option A: AVX2 census	Option C: online switch
Classification	Exact whole block	Prefix estimate
Extra read of syms	Yes	No
Risk of wrong classification	No, relative to fixed rule	Yes
Short-block behavior	Predictable scan cost	Potentially high probe fraction
State conversion	None	Required
Historical Gate 1 comparability	Strong	Weaker
Engineering complexity	Lower	Higher
Potential lowest overhead	Good	Excellent

My research preference is C for theoretical investigation, but A for the next narrowly scoped performance experiment if the Guardian prioritizes certainty and interpretability. C needs a stronger decision rule and short-block loss analysis before its apparent advantage is credible.

A may already be cheap enough to cross the measured 3% control budget, although its memory-bandwidth estimate has little margin. That makes it an unusually direct falsification test.

I would not advocate starting either without a new pre-registration.

What I would pass to the Guardian

I agree that scanner-side counting is the least attractive option because it adds work to the serial pipeline and changes the Symbols API.

I would retain both A and C as distinct research candidates rather than immediately declare C the winner.

A (standalone AVX2 census) has the strongest experimental continuity with Gate 1 and preserves exact whole-block selection. Its estimated 1–2% control overhead is plausible but close to the 3% limit.

C (online switching) has the more interesting architecture: no second scan and a potentially tiny classification cost. Before authorizing implementation, I’d want a read-only derivation of its short-block penalty, prefix-misclassification risk, exact state-conversion boundary and overhead of counting explicit tokens.

For a fixed probe of p explicit tokens and a fallback-heavy block containing n explicit tokens, the approximate regression from running the wrong kernel before switching is δp/n, where δ is P74’s relative slowdown. That is small on 900k-token random blocks but need not be small on short blocks.

I recommend closing Gate 1 under its frozen negative verdict and preserving A/C as separate, unapproved hypotheses. Do not reopen Gate 1 or reinterpret its failure.

My overall advice: close PR #118 with its negative result, subject to your authorization through the coordinator. Preserve the P74 discovery, but don’t let the encouraging Wikidata stage gain turn into an indefinite tuning programme.

And I would keep N1 in view. P74 has now demonstrated a genuine fast-path advantage; the remaining difficulty is making it safe and cheap across workloads. N1 still offers the more fundamental opportunity to change the cost of BWT inversion, and deserves an independent priority decision rather than waiting indefinitely behind P74 refinements.
````
