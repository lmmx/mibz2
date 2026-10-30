# External research report 62: dense directives on option C and N1 (verbatim)

Three relayed messages, in order. Each opens with the maintainer's own line where there is
one. The rest is unattributed in the relay.

**What they decide:**
- **P74 option C first,** as an online decision algorithm with bounded regret:
  R ≲ δ·W/n + H/T_T(n) + M. They require:
  - event-driven monitoring (a counter on the fallback path only, a periodic check at an
    existing boundary) with an instruction-level accounting;
  - the denominator identified: symbols consumed ≠ explicit tokens, since RUNA/RUNB occupy
    positions in `Symbols.syms`;
  - handoff at a clean token or completed-run boundary, preserving the list, the cursor,
    the pending run, the output cursor, the checkpoint state and the count invariants;
  - fitted costs treated as estimates, not bounds;
  - the one-time prefix's exposure shown to be structurally unbounded;
  - predictors compared offline: F, a rank-weighted count, a debt counter.

  The bar for a pre-registered implementation experiment: monitoring cost under the 3%
  control budget, bounded adverse exposure on representative and adversarial inputs, and
  a statistically significant decoder-level gain.
- **N1:** reconsider only with a structural reduction in doubling work, given as records
  read + written + dependent queries per doubling, against 22.5 B/B. Otherwise close it
  without Gate B. The budget is about 1.0–1.2 ns/B, fallback waste included.
- **N3:** a small independent analysis. Avoided work must exceed the added MTF cost.
- **Style:** dense, mathematical engineering decisions.

**The coordinator's actions:**
- the option C analysis was given the requirements (fitted ≠ bounds, the unbounded
  prefix, denominators, handoff state, predictors, adversarial streams, instruction-level
  monitoring);
- N1's work-count question is answered in `2026-10-30-n1-fused-construction.md`: no
  substantial reduction (5.3–7.2 against 4.5–8.4 instructions per byte); N1 archived.

## Message 1, as received

> ok talk more densely now. mathematics and engineering decisioning only. must work intensively to improve. hard and concise. go

Research directive: optimize net decoder cost, not isolated kernels

Two candidates remain. P74 has measured acceleration and an expensive selection mechanism.
N1 has proven lookup reduction and an unmeasured, probably expensive constructor. They
warrant different investments.

My priority is P74 Option C, but not another ad hoc implementation. First derive a
switching policy with bounded regret and negligible steady-state overhead.

1. P74: turn online switching into a bounded-cost algorithm

Let be the number of explicit MTF tokens, their ranks, and

h_i=\mathbf1q_i\\ge32.

Write and for P74 and shipped costs. Define the excess cost of choosing P74:

d_i=c_P(q_i)-c_T(q_i).

On Wikidata, ; on random and records, . A whole-block rank threshold estimates the sign
but requires an expensive census.

The design problem is online regret: minimize the excess cost of the chosen kernel
relative to the cheaper choice for each block, including monitoring and handoff.

A concrete candidate: bounded-window, one-way switching

* Start in P74.
* At 4,096 explicit tokens, inspect the observed high-rank fraction.
* Switch to shipped if that fraction exceeds 10%.
* Otherwise continue in P74, but monitor subsequent high-rank activity in bounded windows.
* Switch permanently to shipped when a later window exceeds the threshold.

This differs deliberately from the failed whole-block selector. It needs a new
preregistration.

The key parameter is not just the initial 4,096-token probe. It is the maximum additional
exposure to an unfavorable distribution.

Suppose the additional cost of a fallback-heavy interval is at most times shipped cost,
the policy reacts within tokens, and handoff costs . For a block of approximately
equal-cost tokens, the excess contribution from one late unfavorable transition is bounded
approximately by

R \lesssim \delta\frac{W}{n}+\frac{H}{T_T(n)}+M,

where is monitoring overhead as a fraction of shipped time.

With , , , the first term is about 0.091%.

That is attractive, but not a universal guarantee: 20% is an observed workload-level
disadvantage, not a bound on individual ranks or arbitrary rank sequences. Multiple shifts
in distribution, short blocks and expensive individual fallbacks require separate
treatment.

The critical engineering question

Can monitoring be driven by high-rank events rather than by an operation on every token?

A promising structure is:

* Ordinary fast path: no additional classification branch.
* High-rank fallback: increment .
* Existing token or checkpoint progress: provide the denominator .
* At infrequent checkpoints: compare integer counts and decide whether to switch.

This uses information exposed by work already being performed.

But is not necessarily free. A symbol-stream cursor counts RUNA/RUNB symbols as well as
explicit tokens. We must either reuse an existing explicit-token counter or charge for
maintaining one. Substituting the encoded-symbol position changes the statistic and its
threshold.

I'd require an instruction-level accounting before treating this design as cheap.

2. A stronger switching criterion than an arbitrary 10%

The 10% threshold was reasonable for the frozen selector experiment. It need not be optimal
for an entirely new online policy.

The economically correct decision is

\text{switch when }\widehat{\Delta T}_{\rm remaining}>H,

where estimates the future cost of staying with P74 relative to shipped.

The original rank threshold implicitly assumes excess cost is adequately predicted by the
number of ranks at least 32.

That assumption needs testing.

A potentially better predictor is a weighted fallback cost:

S=\sum_{q_i\ge32}w(q_i)-\alpha E,

where approximates the cost of the tail operation at rank , and represents P74's saving
per ordinary token.

However, this is only useful if calculating is effectively free inside the existing
fallback path. Otherwise we're rebuilding the expensive classifier in another form.

Research choice: compare simple fallback counts, rank-weighted counts and the original 10%
rule offline using existing traces. Select the simplest predictor whose errors are
acceptably bounded. No tuning against CI performance after implementation.

3. N1: reduce construction work or stop

The main construction budget is approximately

B_{\rm N1}\le1.0\text{–}1.2\ \mathrm{ns/decoded\ byte}

on the 7763 for , subject to the modeled walk cost.

The only historical related builder implies 16–27 cycles per record, versus roughly
6.3–8.8 affordable across the corpus.

Therefore, ordinary instruction tuning is not a convincing research thesis by itself. We
need a concrete structural reduction in doubling work.

The promising research question is whether the existing counting, prefix and merge passes
can be fused or reorganized while preserving linear complexity and bounded memory.

A viable N1 proposal should specify:

\text{records read}+\text{records written}+\text{dependent queries}

per doubling, and demonstrate an improvement relative to the current 22.5 B/B construction
model.

If no new organization removes a substantial fraction of those operations, I'd close N1
without Gate B. Its correct representation and bounded-memory selector can remain
archived.

4. The engineering decision

Candidate	Next research question	Criterion for further investment
P74 online switch	Can we bound unfavorable-prefix exposure while charging near-zero monitoring overhead?	Credible path to ≤3% control regression and positive full-decoder benefit
N1 construction	Can we structurally eliminate a substantial part of doubling work?	Concrete route to the 1.0–1.2 ns/B builder budget
N3 direct level-1 emission	Can MTF-produced run boundaries replace shipped work?	Avoided work exceeds extra MTF-stage cost, measured at decoder boundary

My ranking: P74 first; N1 only upon a new construction insight; N3 as a small independent
analysis.

For P74, I would derive the regret bound, examine counter availability in the actual loop,
quantify handoff cost, and construct adversarial rank streams before any benchmark. For
N1, I'd demand a new work-count argument, not another optimistic cycle model.

The objective remains faster mibz2 decoding on actual multicore CPUs, with no unacceptable
regressions in CPU efficiency or memory. Wikidata is one corpus among several, not the
criterion for whether the research matters.

The immediate problem to solve is P74's online switching economics. It has the clearest
measured benefit and the shortest plausible route to a deployable improvement.

## Message 2, as received

Agreed. The scope is now correct: optimize mibz2 itself, and evaluate P74 Option C as an
online decision algorithm with a measurable cost.

The planned analysis is useful. I would impose four technical requirements on its
interpretation.

1. Corpus-fitted rank costs are not adversarial bounds.

A model fitted from three aggregate corpus timings is underdetermined: rank costs, branch
prediction, run emission and inter-token memory dependencies are confounded.

Report fitted costs and residuals, but do not use the fitted maximum as a guaranteed worst
case. The adversarial analysis should distinguish a conditional cost bound from an
empirical estimate.

2. A one-time prefix decision has unbounded relative exposure to a distribution shift.

For a 4,096-token low-rank prefix followed by a long high-rank suffix, a one-way decision
to remain in P74 can incur nearly the entire high-rank penalty.

Increasing the probe to 16,384 tokens doesn't fix that structural failure.

Bounded monitoring is therefore essential if the aim is robust behavior across arbitrary
rank sequences.

3. Monitoring must use the correct denominator.

symbols_consumed is not equivalent to explicit_tokens_consumed: RUNA/RUNB occupy positions
in Symbols.syms without being ordinary MTF ranks.

If the denominator changes, so does the classification rule. That's permissible for a new
experiment, but it must be identified as a new statistic and calibrated or bounded
independently.

The most promising implementation is an event-driven high-rank counter combined with
periodic inspection of an already-maintained progress counter—provided the actual assembly
confirms negligible additional fast-path cost.

4. Handoff must preserve the complete decoder state.

The switching proof must cover the MTF list, encoded-symbol cursor, pending RLE2 run,
output cursor, checkpoint state and output-count invariants. Switching at a clean token or
completed-run boundary is preferable to supporting arbitrary internal handoffs.

One deeper question for the analysis

Can we avoid predicting the whole remainder altogether?

A policy that monitors cumulative excess cost rather than a fixed high-rank percentage
might switch when P74's observed disadvantage exceeds a budget.

For example, maintaining a debt counter only on fallback events could permit low-rank
periods to accumulate credit and high-rank periods to consume it. The switch occurs when
estimated debt crosses a fixed threshold.

This could be more robust to distribution changes than a single prefix classification,
although it still needs a defensible cost model and may fail to react to a costly suffix
until some loss has accumulated.

I would treat that as an analytical comparison, not an additional experimental arm.

Research decision: finish the existing Option C analysis. If it demonstrates a plausible
monitoring cost below the 3% control budget and bounded adverse exposure on representative
and adversarial inputs, a separately preregistered implementation experiment becomes
defensible.

If it doesn't, P74 remains archived. No reason to reopen N1 or authorize another benchmark
in the meantime.

## Message 3, as received

> please do not spread out your writing. dense mathematical engineering decisioning. concise and hard. go

Understood. I'll keep subsequent research responses dense: equations, measured evidence,
assumptions, falsification criteria and engineering decisions. No decorative formatting or
extended recaps.

Current priority: P74 Option C. Derive monitoring overhead , switching cost , and
adverse-prefix exposure . Require on controls, while preserving a statistically
significant decoder-level gain. Corpus-fitted costs are estimates, not worst-case
guarantees. Favor event-driven monitoring if its fast-path cost is demonstrably
negligible.

N1: Gate B remains unauthorized. Reconsider only with a credible construction route below
the whole-corpus budget of approximately – ns/B, including fallback waste and
retained-memory constraints.

Decision principle: optimize total decoder cost, reject unjustified performance
extrapolations, and spend implementation effort only where a falsifiable model predicts
sufficient net gain.
