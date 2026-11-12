# External research report 40: the selector arm's authorisation (verbatim)

The maintainer's instruction of 2026-10-30: "take this advice from the guardian", together
with the researcher's matching advice. The maintainer adopted both.

**What it decides:**
- **The single selector arm is authorised,** under Gate 1's mixed-result step:
  - P74 free when A ≤ 32 (the count skipped), or when at most 10% of explicit ranks are
    32 or more;
  - otherwise the shipped kernel;
  - both kernels, with the classification timed.
- **Reported:** per-block selections, decoded-byte coverage, and the paired intervals.
- **After it:** stop for review.
- **Not authorised:** Phase 2, N1 Gate B, production integration, the i9, tuning, or
  further variants.

**The coordinator's response:**
- **The rule is frozen first,** committed to `2026-10-30-p74-gate1-preregistration.md`
  ("The selector arm") before any of its code. It fixes the zero-explicit-token case
  (E = 0 → P74) and the exact integer test (10·F ≤ E).
- **"Decoded bytes" is defined there** as the stage's output (`ll`, before RLE1).

## The instruction, as received

````text
take this advice from the guardian:

Claude is handling this correctly. It’s distinguishing research recommendations from your authorization, and it’s preserving the protocol before making any changes.

I recommend authorizing the selector experiment. We have a measured 33.2% improvement in Wikidata’s checkpoint-aware MTF stage, and the remaining question is whether a cheap per-block decision can preserve that advantage while avoiding regressions on random and records.

There’s no compelling reason to abandon P74 before completing the mixed-result procedure we explicitly preregistered.

Claude’s clarification about the alphabet guard is also correct: for blocks with at most 32 symbols, ranks of 32 or greater are impossible. The guard is an inexpensive shortcut around the classification scan, not a different selection criterion.

You can send this:

I explicitly authorize the single P74 Gate 1 selector experiment under the registered mixed-result procedure.

Freeze and commit this selection rule before implementation or measurement:

* If the block uses at most 32 symbols, choose P74 branch-free without scanning the ranks.
* Otherwise, count explicit MTF ranks ≥32 in Symbols.syms. Choose P74 branch-free if their proportion is at most 10%; otherwise choose the shipped kernel.
* Apply the same selection policy to both plain and checkpoint kernels.
* Include the complete classification cost within the timed stage.

Use the same-binary paired comparison, correctness checks, negative controls, confidence intervals and iteration limits already registered. Do not tune the threshold or add further variants.

Report per-block selection counts, decoded-byte coverage, measured stage costs, paired confidence intervals and layout-control results.

After this single CI experiment, stop and report the registered verdict for review.

This authorization does not extend to P74 Phase 2, N1 Gate B, production integration or any i9 benchmarking.

One thing worth emphasizing: a successful selector test would be an important result, but still not a shipping decision. Phase 2 must establish that the complete four-worker decoder improves while CPU-seconds per GB and memory stay within their registered limits.

And we should keep the larger breakthrough programme moving intellectually. P74 has now demonstrated meaningful scope for reducing work in the front end; N1 remains the more radical prospect for reducing inverse-BWT navigation itself.

My recommendation is to complete this one bounded experiment, then make the next decision from measured evidence.

===

the researcher says to as well:

Claude has handled this correctly. The experimental protocol is frozen, and it makes sense to require your explicit authorization rather than infer it from the Guardian’s or my recommendations.

My research recommendation remains to approve the single selector experiment. The measured 33.2% Wikidata checkpoint-stage improvement justifies testing whether we can retain the gain while avoiding the random and records regressions.

The proposed rule is well defined:

\text{Select P74 if } A\le32 \quad\text{or}\quad 10F\le E

where is the used alphabet size, counts explicit ranks of at least 32, and counts all explicit ranks. For zero explicit tokens, the alphabet guard or an explicitly specified default must handle the case without division.

Everything else should remain as preregistered.

If you’d like to authorize it, you can send Claude this:

I explicitly authorize the single P74 Gate 1 selector experiment, under the existing mixed-result provision.

Freeze the threshold in the preregistration before implementation: select the branch-free P74 kernel when the used alphabet has at most 32 symbols, or when at most 10% of explicit MTF ranks are 32 or higher. Otherwise select shipped MTF.

Include all classification costs inside the measured stage. Define the zero-explicit-token case before measurement.

Use the same-binary comparison, registered paired statistics, negative controls, correctness checks and iteration limits. Report the per-block selections and the resulting confidence intervals.

After this single CI experiment, stop and report the registered verdict for review.

This does not authorize Phase 2, N1 Gate B, i9 benchmarking, further tuning or production changes.

One point worth emphasizing: this experiment is not a search for the optimal threshold. Ten percent is a fixed, defensible classification rule chosen before measurement. If the selector fails the registered criteria, that result should stand.

We have a promising stage-level result, a credible explanation for why it doesn’t generalize, and one bounded opportunity to address the regression. That’s a worthwhile final test of P74 Gate 1.
````
