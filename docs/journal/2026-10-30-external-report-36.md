# External research report 36: the guardian and the researcher on Gate 1's local signal (verbatim)

A research input, not a finding of this repository. The maintainer relayed four messages
on 2026-10-30, while Gate 1's CI run (lmmx/mibz2#118) was pending: the guardian, the
researcher, and each on the other.

**What they agree on:**
- **No action until CI reports;** no tuning, and the protocol unchanged.
- **Three propositions kept apart:**
  - the shipped dependency exists (established);
  - the register-held state machine works and compiles without head spills
    (established);
  - it is faster (unproven, with local evidence pointing both ways).
- **A loss is not a blanket verdict.** A loss of the combined kernel isn't a rejection
  of register-resident MTF; the fallback may be the part that fails.
- **The selector runs only on a true mixed result:** Wikidata meeting the positive rule
  with a control failing. Its own figures include its classification cost.
- **N1 is independent of P74's outcome.**

**The coordinator's note, a correction to its own report.** The coordinator's message
before this one said the shipped `memmove` "beats the chunked tail shift" on random and
records. That over-states the local evidence: it is a hypothesis. Two other candidates
fit the same local numbers, and none is isolated:
- **Branch mispredictions** on the records corpus's rank mix.
- **Failed store forwarding inside the tail.** When fallbacks come close together, as
  on random data (about 87.6% of tokens), a chunk's unaligned load at p − 1 overlaps the
  previous fallback's chunk stores. That is the same mechanism P74 removes from the head,
  reappearing in the tail.

Gate 1 doesn't attribute between them (open point 6 of the pre-registration).

The text reached us as rendered text (tables flattened) and is kept as received.

## The messages, as received

````text
guardian:

Yes, that’s a discouraging start for P74. But there’s an important distinction: the experiment appears to have been implemented correctly and is already exposing the weaknesses the preregistration was designed to detect.

The most significant new information is actually quite specific: Claude now suspects the shipped memmove outperforms P74’s chunked vector-tail shift on high-rank inputs. That is more concrete than our earlier, generic concern about contention or fallback overhead.

The local findings suggest two different performance regimes:

* Wikidata: P74 may benefit from keeping the MTF head in a register, although the checkpoint results vary and the improvement isn’t yet established.
* Random and records: the cost of P74’s high-rank fallback may overwhelm its fast-path gains.

This does not mean the original store-to-load dependency was imaginary. It means removing that dependency may be worthwhile only where the resulting representation doesn’t make other operations more expensive.

And importantly, Claude has respected the key boundaries: no production decoder change, correctness checks passed, disassembly inspected, one permitted code-generation fix, and CI measurement running under the registered protocol.

One point worth watching

The experimental design bundles two changes: register-resident MTF updates and a different high-rank fallback. Consequently, a large regression on random data does not, by itself, establish that register residency is unsuitable for random data. It might establish that the particular fallback implementation is unsuitable.

That distinction matters for our research register, even if the frozen stopping rules prevent further tuning.

I’ve also checked PR #118⁠￼. It remains an open, research-only comparison, explicitly not intended for merging into production. No verdict is established by the information available so far.

My recommendation is to let the CI result decide. If Wikidata clears the threshold but the controls fail, the registered selector gets its single chance. If that fails, close P74 and retain the mechanistic findings.

And don’t lose sight of N1. P74 was always a relatively modest front-end opportunity, whereas the multi-byte Ψ representation is our more ambitious route toward changing decoding’s fundamental cost. A negative P74 result would be disappointing, but it wouldn’t undermine the central breakthrough programme.

===

researcher:

This is a disappointing start for P74, but there’s an important distinction: the experiment is functioning, and the optimization is encountering a genuine performance challenge. That’s precisely what Gate 1 was designed to establish.

I’m particularly interested in one finding from Claude’s report:

The shipped memmove beats the chunked tail shift on random and records.

That gives us a more concrete explanation than our earlier hypothesis that high-rank fallbacks were simply too frequent.

What the evidence says so far

Finding	Research interpretation
Register head remains resident on the fast path	The central code-generation requirement is satisfied
Wikidata checkpoint kernel improves locally	The optimization has potential on its intended workload
Random and records regress substantially	The replacement fallback is potentially uneconomic
Shipped memmove outperforms chunked shifts locally	The new tail-update mechanism is a plausible contributor
Tests detect deliberately introduced errors	Useful evidence that the correctness tests aren’t trivially passing
CI results pending	No registered performance verdict yet

The interesting scientific lesson is that less memory movement does not necessarily mean fewer CPU cycles.

Our earlier analysis showed that P74 moves fewer logical bytes during high-rank fallback. But an optimized memmove can process a short region very efficiently, whereas several vector chunks, masks, blends and address calculations may cost considerably more instructions.

The result challenges the assumption that eliminating the fallback’s library call would necessarily be beneficial.

One caution about causal attribution

We should preserve Claude’s observation as a hypothesis supported by local measurements, not a fully isolated causal conclusion. The two P74 variants share several changes, including short-run emission and fallback logic, so the existing comparison doesn’t tell us precisely how much of the regression comes from each.

That matters when documenting why P74 succeeds or fails.

What I would do now

I agree with the decision to stop tuning and leave the frozen experiment alone.

There is no reason to authorize another implementation variant, alter the stopping criteria, or run the i9.

The CI result should answer whether the checkpoint-kernel improvement is reproducible, whether the negative-control regressions exceed their registered limits, and whether the pre-registered selector test becomes appropriate.

If CI confirms the local pattern, P74 may still be viable as a per-block optimization, but only if the selector passes its own measured gate. Otherwise, the correct research outcome is to close it as an uneconomic implementation despite its proven correctness.

One positive point worth keeping: the disassembly confirms that the register-held fast path can exist in real compiled Rust code. That result remains valuable even if this particular kernel loses overall.

For now, I would preserve the findings, wait for the registered CI verdict, and make no changes.

P74 has not failed scientifically. Its general-purpose performance hypothesis is being tested—and the fallback may be the part that falsifies it.

===

researcher on the guardian:

The Guardian’s assessment is sound, and I agree with its central distinction: P74’s general-purpose performance may fail without falsifying the benefit of register-resident MTF on rank-local workloads.

There is one research point I’d sharpen.

We currently have evidence for three separate propositions:

* The shipped dependency exists. Disassembly confirms the overlapping loads and stores that motivated P74.
* The alternative state machine works. Correctness tests and compiled-code inspection support the register-held design.
* The alternative is faster. This remains unproven, with preliminary evidence pointing in opposite directions across workloads.

Those propositions should remain separate in the final research record.

In particular, I wouldn’t yet state that the shipped memmove is the cause of the random and records regressions. It is a plausible explanation, but the present experiment changes multiple operations simultaneously. The Guardian correctly recognizes that attribution limitation.

One important distinction for the CI verdict

The preregistered mixed-result procedure matters here.

If CI establishes a qualifying Wikidata improvement alongside control regressions, the single per-block selector arm is permitted under Gate 1. However, that selector must meet the same performance requirements with its classification cost included.

We must not report the unscreened P74 arm’s Wikidata gain as though it were the selector’s gain.

Equally, a negative result for the present P74 implementation should be recorded as a negative result for this combined kernel design, not a proof that every possible register-resident MTF implementation is slower.

That distinction preserves the scientific value of the work without justifying an unregistered tuning campaign.

My research assessment

P74 correctness

Established

Fast-path machine-code feasibility

Established

Workload-dependent performance

Preliminary evidence

Causal attribution of regressions

Unresolved

Registered Gate 1 verdict

Pending CI

And yes, the Guardian is right about N1: its underlying research is independent of P74. A negative P74 result would not weaken the mathematics or potential upside of multi-byte Ψ navigation.

For now, I see no need to send the Guardian further corrections or additional research instructions. The decisive contribution from me will be to scrutinize the actual CI results against the frozen rules when they arrive.

The experiment is producing useful evidence already. We shouldn’t mistake an unfavorable observation for wasted research.

===

guardian on the researcher:

I agree with the researcher’s assessment. The most consequential finding so far is that P74 has achieved its intended machine-code structure, but that structure may not be economical across different workloads.

That’s a useful distinction. The fast path keeps the MTF head in a register, while the expensive fallback may negate the benefit on high-rank inputs.

One nuance deserves particular attention: fewer logical bytes moved is not necessarily less computational work. An optimized memmove can outperform a sequence of vector masks, blends and shifts, even when those operations appear more efficient on paper.

I would preserve three conclusions separately:

* Established: P74’s fast path compiles without the unwanted head spills, and its correctness tests pass.
* Preliminary: Wikidata benefits locally, while random and records suffer large regressions.
* Unresolved: How much of either result is attributable to register residency, RLE2 handling or the high-rank fallback.

The last point matters because a failed combined kernel should not become a blanket rejection of register-resident MTF in the research portfolio.

I also see no reason to modify the experimental protocol. The registered mixed-result procedure already provides one controlled opportunity to evaluate a per-block selector.

There’s one statistical subtlety I would watch when CI reports: the selector should only be tried if Wikidata actually satisfies the registered positive criterion and the controls fail. Large random-data regressions alone do not establish a mixed verdict.

For now, the right action is no action. Let CI produce the paired estimates, apply the frozen decision rules, and retain the mechanistic findings regardless of the result.

And strategically, this reinforces the value of having two independent gates. P74 is investigating a relatively narrow MTF optimization; N1 remains the more radical attempt to reduce the fundamental cost of inverse-BWT navigation. The outcome of one does not decide the other.
````
