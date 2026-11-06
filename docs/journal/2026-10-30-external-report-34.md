# External research report 34: the guardian on reading Gate 1's result (verbatim)

A research input, not a finding of this repository. The maintainer relayed it on
2026-10-30, after external report 33 (`2026-10-30-external-report-33.md`).

**What it contains:** the guardian, now holding the researcher's three details.
- **Validation is not attribution.** Both P74 arms carry the same RLE2 and fallback
  changes. A win validates the combined kernel; it doesn't say how much comes from
  register residency and how much from fewer `memset` calls. That is no reason to change
  the protocol, only a reason not to over-claim afterwards.
- **The two arms deserve equal weight.**
- **The disassembly check matters as much as the source design.**
- **The outcome is judged against the frozen criteria,** not the predicted 20–32%.
- **A negative result is informative** without proving register-resident MTF impossible.
- **Still not an authorisation.**

**The coordinator's note:** the attribution limit is added to the pre-registration's
open points (item 6) as a limit on interpretation. No arm or rule is added.

The text reached us as rendered text and is kept as received.

## The guardian's message, as received

````text
The researcher is right. Those three details materially sharpen how we should interpret P74 Gate 1, particularly if the results are mixed.

I’m caught up on them now, and I’d preserve one overarching distinction: P74 has strong mechanistic motivation, but it has not yet demonstrated a performance improvement.

What matters most when reviewing Gate 1

1. The dependency hypothesis is testable, but the result may not isolate its cause.

The shipped disassembly demonstrates overlapping MTF-list stores and loads that cannot use ordinary store-to-load forwarding. The historical 1.058× store-ordering result strengthens the hypothesis that this dependency matters.

But even if P74 wins, we cannot automatically attribute the entire improvement to eliminating that dependency.

P74 also changes RLE2 handling, call frequency, register pressure and fallback operations. The experiment tests a combined kernel design, not one isolated instruction-level intervention.

That’s acceptable for an engineering feasibility gate. We simply need to distinguish performance validation from causal attribution.

2. The branching and branch-free designs deserve equal consideration.

The approximately 84.6% one-shuffle coverage is encouraging, but branch prediction can negate much of the expected saving.

A branch-free variant may perform more instructions on common inputs while avoiding costly control-flow mistakes.

So I wouldn’t select a winner by counting shuffles or branches. The comparison needs to include the entire MTF kernel, precisely as preregistered.

3. The RLE2 call boundary may determine whether register residency survives.

This is arguably the most important implementation detail beyond the MTF shuffle itself.

Reducing modeled library calls from approximately 329 to 32.5 per 1,000 explicit Wikidata tokens could change the viability of maintaining the list head in a YMM register.

However, reducing calls is not automatically equivalent to reducing work. Inline vector operations have their own costs, particularly for larger runs and uncommon ranks.

The disassembly check therefore matters just as much as the source-level design.

One additional concern I’d keep in mind

The two P74 variants share the same RLE2 and fallback changes. If both substantially outperform production, we will know the combined approach works, but we won’t know how much improvement comes from register residency versus avoiding memset calls.

That isn’t a reason to alter the frozen Gate 1 protocol.

It is a reason to avoid making an overly specific causal claim afterward. If the result is genuinely strong and reaches the full-decoder promotion stage, a later attribution experiment could be useful—but only if the added information would change an engineering decision.

Where I stand

I still favour authorizing P74 Gate 1. Its cost is bounded, its controls are strong, and it addresses an identified machine-code inefficiency.

But I’d judge its outcome against the frozen criteria, not the researcher’s predicted 20–32% MTF-stage improvement.

A negative result would be informative: it would tell us that removing the suspected dependency, alongside the specified associated changes, did not deliver enough improvement under the tested conditions. It would not prove that register-resident MTF is mathematically or architecturally impossible.

And I continue to regard N1 as the more ambitious breakthrough candidate. The relevant novelty is its proposed multi-byte decoding and construction mechanism, not the already-published underlying run partition.

The researcher’s final point is especially important: understanding the experiment and authorizing it are separate decisions. There is no need to send Claude another message simply to update me. I’m ready to review Gate 1 whenever you decide to run it.
````
