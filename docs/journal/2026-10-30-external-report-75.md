# External research report 75: kill H, retire V, advance P74-C to the implementation gate (verbatim)

A research input, not a finding of this repository.

- **Relayed:** by the maintainer, 2026-10-30 18:36 UTC.
- **Source:** unattributed in the relay (the researcher's register).
- **In reply to:** after the committed P74-H analysis.

It is kept exactly as received (rendered text, cards flattened). Paths to uploaded
files are the maintainer's.

## The message, as received

````text
Decision: kill H. Retire V as an independent candidate. Advance P74-C to the implementation gate.

I inspected the committed H analysis. It contains an important additional result: the plain P74 kernel already compiles its lower tail chunks into unmasked copies, whereas the checkpoint kernel uses masked chunks. My proposed P74-V substantially overlaps that existing design.

The measured plain/checkpoint contrast is only 1.73–2.24 cycles per fallback on Zen 5, against approximately 5.8–7.5 cycles required to rescue the checkpoint controls. This comparison is confounded by loop layout and checkpoint accounting, so it is not a proof of V’s failure. It is nevertheless sufficient to reject another speculative optimization round.

P74-C: executable engineering specification

Dispatch:

S<16384\ \lor\ 2S\ge N_{\max}
\quad\Rightarrow\quad \text{untouched shipped kernel}.

Otherwise start P74. At each , before modifying MTF state:

F+1>\lfloor S_t/32\rfloor+123
\quad\Rightarrow\quad \text{switch permanently to shipped}.

Here is the number of symbols already consumed including the current token. Preserve that token for the shipped continuation; do not decode it twice or skip it.

Implementation constraints: one shared state frame, 320-byte aligned list storage, one YMM head store at handoff, plain/checkpoint resume paths, no modification to standalone shipped kernels. Keep F and the threshold arithmetic exclusively on the fallback edge. Avoid introducing a fast-path register spill.

Three same-binary arms: untouched shipped; combined kernel forced to shipped at token zero; live policy.

\Delta_{\rm layout}=T_1-T_0,\qquad
\Delta_{\rm policy}=T_2-T_1.

This separates the restructuring tax from the actual acceleration.

Acceptance must be independent

The threshold was fitted against the exact 431-block Wikidata corpus. That corpus cannot establish generalization.

Freeze the policy, obtain independent Wikidata blocks and mixed-distribution controls, then measure without retuning. Required:

\mathrm{CI}_{95}^{\rm upper}
\left(\frac{T_2}{T_0}-1\right)\le0.03

on every registered negative-control category, with a statistically significant positive Wikidata gain. Validate all handoff positions, complete decoder output, CPU-s/GB and peak memory separately.

The first kill test is not throughput. It is generated assembly: if the combined function spills the YMM head or adds low-rank-path dependencies, the intended zero-cost monitor has failed.

Breakthrough arithmetic

P74-C is the first plausible shipping improvement, not the breakthrough itself.

On the consistent 7763 baseline, a 20% throughput increase requires:

\Delta T_{\rm MTF}+\Delta T_{\rm walk}\ge1.797\ \mathrm{c/B}.

B2 must therefore be evaluated jointly with the measured P74-C saving, not against an arbitrary standalone 20% walk target.

Next move: complete B2’s read-only verdict, then proceed to the frozen P74-C research implementation and one preregistered stage experiment once explicitly authorized. Do not divert engineering effort back into H, V or N1. A positive P74-C measurement establishes the first part of the combined performance budget; B2 must supply the remainder.
````
