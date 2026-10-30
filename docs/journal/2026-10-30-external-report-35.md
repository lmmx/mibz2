# External research report 35: Gate 1's authorisation (verbatim)

The maintainer's instruction of 2026-10-30: the guardian's text, which the maintainer
adopted as their own ("Please take the guardian's instruction here as my own").

**What it decides:**
- the six open points in `2026-10-30-p74-gate1-preregistration.md`, with 10 paired
  rounds and no second alignment build;
- **P74 Gate 1 is authorised, and only Gate 1,** under the amended and frozen protocol;
- production stays unchanged;
- no phase 2, no N1 Gate B, no i9.

**The coordinator's response:** the pre-registration is amended and committed before any
P74 code is written or measured.

## The instruction, as received

````text
Please take the guardian’s instruction here as my own:

===

Claude has identified two genuine protocol choices that deserve a decision before Gate 1 is authorized: statistical power and how strongly to control for code-layout effects.

My inclination is to strengthen the experiment modestly rather than expand it into another research campaign. P74 is inexpensive enough that we shouldn’t risk an inconclusive result simply because we economized on a few CI rounds.

I would leave the prediction width, branch-model caveat, microarchitecture limitation and attribution caveat as documented uncertainties. None requires changing the experimental arms.

My decisions on the six open points

Point	Recommendation
Prediction width	Keep 3–6%. Clearly marked as model-based.
Statistical power	Increase to 10 paired rounds, retaining two passes per round.
Code layout	Keep the same-binary comparison as the primary test. Don’t require a second, differently aligned build for Gate 1.
Branch model	Keep both variants. Let the measurements decide.
Zen 3 versus Comet Lake	Accept the limitation. No i9 benchmark at this stage.
Causal attribution	Record the limitation. Don’t add another implementation arm.

Why I’d increase the rounds

Five rounds can be adequate for a large effect, but the registered threshold is 5%, and the modeled stage improvement is potentially close enough to the decision boundary that measurement uncertainty matters.

Ten paired rounds should provide a more useful confidence interval for little additional cost. It won’t guarantee a decisive result if run-to-run variance is large, but it’s a reasonable investment.

I’d retain rounds as the independent statistical units rather than treating both passes as ten independent samples under the old protocol.

Why I wouldn’t add a second alignment build yet

The checkpoint-kernel layout concern is legitimate. The same-binary design eliminates much cross-build variation, but cannot eliminate the effects of function placement.

However, a differently aligned build introduces another experimental dimension. Requiring a positive result in both builds would also change the estimand from performance under the specified build to robustness across two layouts.

For Gate 1, I’d prefer a simpler controlled experiment, with disassembly inspection and a specific interpretation limit.

If the measured improvement is extremely close to the threshold, or the compiler produces suspiciously different code between arms, we should record that uncertainty rather than retrospectively alter the protocol.

The important thing is to make these choices before implementation and measurement, not after seeing the results.

Suggested instruction to Claude

I’ve reviewed the six open points appended to the P74 Gate 1 preregistration.

My decisions are:

* Keep the deliberately widened 3–6% one-core prediction.
* Increase the stage comparison to 10 paired rounds, retaining two passes per round and treating rounds—not passes—as the independent observations.
* Retain the same-binary comparison and existing plain-kernel layout control. Do not add a mandatory second alignment build for Gate 1; document the checkpoint-kernel layout limitation.
* Keep both branching and branch-free arms without privileging either based on the simple predictor model.
* Accept that this gate measures Zen 3, not the i9.
* Preserve the causal-attribution limitation; do not add another experimental arm.

Please update and commit the preregistration with these decisions before implementing or measuring anything.

I authorize P74 Gate 1 only, under that amended and frozen protocol.

Production decoder code must remain unchanged during this gate. No P74 Phase 2, N1 Gate B or i9 experiment is authorized.

After the Gate 1 measurements, report the paired results, confidence intervals, layout-control outcome, correctness and disassembly findings, and the registered verdict before proposing any further action.

I would approve that version of Gate 1. It’s a relatively cheap, well-bounded experiment with a credible mechanism and clear stopping rules.

And it leaves the larger breakthrough ambition intact: P74 could improve the front end, while N1 remains the more radical possibility of changing the cost of inverse-BWT navigation itself.
````
