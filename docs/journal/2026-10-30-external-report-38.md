# External research report 38: the guardian on Gate 1's result and the selector (verbatim)

A research input, not a finding of this repository. The maintainer relayed it on
2026-10-30, after external report 37 (`2026-10-30-external-report-37.md`).

**What it contains:**
- The guardian agrees with the mixed verdict.
- **It drafts an authorisation of the single selector arm,** keeping the registered
  structure: the alphabet guard, then an exact count of explicit ranks ≥ 32, with a fixed
  10% threshold and the classification's cost timed.
- **The selector's budget:** 0.268 ns/B, the Wikidata saving.
- **The instruction to stop after the selector's result,** for review.

**The coordinator's notes:**
- **The researcher's and the guardian's rules are the same rule.** A block with A ≤ 32
  symbols has no rank ≥ 32 (§3.2e(k)), so its high-rank share is 0 ≤ 10%. The alphabet
  guard only skips the count; it never changes a selection.
- **The guard's budget is small against 0.268 ns/B.** The count reads `Symbols.syms` (u16)
  once. On Wikidata that is explicit tokens (about 0.06 a byte) plus the RUNA/RUNB digits,
  a fraction of a byte of syms per output byte, which an AVX2 compare-and-count reads at
  tens of bytes a cycle. That is an estimate, and the arm measures it with the count
  inside the timing.
- **The guardian's text was relayed, not adopted.** Unlike the Gate 1 authorisation
  ("Please take the guardian's instruction here as my own", external report 35), the
  maintainer has not adopted it, so the selector arm is not started.

The text reached us as rendered text (a table flattened) and is kept as received.

## The message, as received

````text
guardian:

This is a much stronger result than the local measurements suggested. A 33.2% reduction in the complete Wikidata checkpoint-MTF stage is substantial, and the fact that the branch-free design wins so decisively is particularly interesting.

But Claude has classified it correctly: Gate 1 is mixed, not passed for promotion. The random and records controls fail their registered limits.

I would authorize the one remaining preregistered selector experiment, using a conservative rule fixed before measurement. This is exactly the situation that contingency was designed for, and we should use it rather than abandon a demonstrably promising specialization.

There’s one important detail I would change in Claude’s proposed rule: I would initially use only the fraction of explicit ranks ≥32, with a fixed 10% threshold, rather than combine that with an alphabet-size shortcut.

The reason is experimental clarity. Alphabet size and high-rank frequency are correlated but not equivalent. A single predictor makes it easier to interpret the result, and the observed corpora are far enough apart that a complicated classification rule seems unnecessary.

The remaining question is whether selecting P74 is cheap enough

The measured Wikidata checkpoint kernel takes approximately:

Arm	Time
Shipped	0.807 ns/byte
P74 branch-free	0.539 ns/byte
Saving	0.268 ns/byte

That’s the entire budget available to the selector on Wikidata before it starts eroding P74’s gain. It must examine the encoded ranks without spending too much of that saving.

I also checked the result journal. There’s a protocol detail worth preserving: the preregistered mixed-case selector specifies an alphabet guard plus an exact count of ranks ≥32. My preference for a rank-only rule should not silently remove that guard. We should retain the registered selector structure, with the alphabet guard as an early decision and the 10% rank threshold fixed before measurement.

The threshold is defensible from the available observations—1.6% high ranks on Wikidata, 5.1% on text, 76% on random and 90% on records—but 10% is a pragmatic choice, not a proven optimal cutoff. We should not tune it after looking at the result.

My recommendation to Claude

I’ve reviewed the P74 Gate 1 result and agree with the registered MIXED verdict.

I authorize the single preregistered selector arm, and nothing beyond it.

Please retain the preregistered selector structure: the alphabet guard followed by an exact count of explicit ranks ≥32 in Symbols.syms, with all classification costs included in the timing.

Fix the threshold before implementation or measurement: select the branch-free P74 checkpoint/plain kernel when high-rank tokens are at most 10% of explicit tokens; otherwise select the shipped kernel. Preserve the alphabet guard as specified in the protocol.

Use the same-binary paired methodology and the registered positive, control-regression and stopping criteria. No threshold sweeps, extra variants or fallback tuning.

Report the selector’s actual Wikidata checkpoint gain, control-corpus regressions, confidence intervals and layout-control result. Keep the stage result separate from any inferred full-decoder benefit.

Do not proceed automatically to Phase 2, N1 Gate B or i9 benchmarking. Stop after this result for review.

This is the first really convincing positive experimental evidence from the new portfolio: a substantial improvement in an entire production-equivalent stage, not just a microbenchmark.

But it’s not yet a full-decoder speedup, and the selector could still erase the advantage.

I’d use the one remaining registered experiment. If it succeeds, we have a potentially useful specialization worth evaluating in the complete decoder. If it fails, we have a clear stopping point—and N1 remains the higher-upside fundamental research avenue.
````
