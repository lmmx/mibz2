# External research report 159: the guardian on the three VM-side results: the census strengthens the 30-entry design; RAW rows are a storage frequency; P87 and the dispatch addendum at their boundaries; the Astra handoff (verbatim)

A research input, not a finding of this repository.

- **Relayed:** by the maintainer, 2026-11-02, with reports 160–161. All three are exact
  slices of one relayed message, split at its section boundaries.
- **Source:** the guardian.
- **In reply to:** the coordinator's results for report 158's three tasks: the dispatch
  addendum (`945460d`), P87's cleanup (`7c40491`, PR #128) and the 30-entry census
  (`1fbc0a0`, PR #132).

**Evidence cutoff:** 2 November 2026. A reading of the committed census journal and
reference implementation, PR #128 and the dispatch addendum. The guardian did not rerun the
census or the test suites.

It is kept exactly as received (rendered text, cards flattened). The maintainer's
introduction to the relay was: "I sent Astra the two promp parts attached. The guardian
helped me write them". The two prompt parts are reports 160 and 161.

## The message, as received

`````text
Claude did well. All three tasks produced useful results, and it stopped at the agreed boundary. The most important development is that the 30-entry layout now has corpus-tested encoding and reference correctness—not just a promising paper design.

I checked the committed census journal and reference implementation, the updated P87 PR, and the dispatch addendum. I have not rerun the census or test suites myself.

1. The census strengthens the architectural proposal

The result is a favourable storage trade-off:

Representation	Logical bytes per BWT row
Shipped T-vector	4.000
FOR-14	2.076
Proposed 30-entry layout	2.164

On the primary corpus, the new layout needs only about 4.2% more logical storage than FOR-14, while retaining approximately a 46% reduction against the shipped table. Its largest full block occupies 1,953,600 logical bytes, below the proposed 2,108,096-byte bound for the relevant alphabet size.

That is the substantive result: we do not appear to need FOR-14’s expensive packed-field extraction to retain most of its storage advantage.

It does not prove that the simpler accesses will be fast. But it removes a significant objection to trying them: the wider fields have not destroyed the compactness benefit on the actual corpus.

The correctness evidence is stronger than a size census alone

The program independently constructs Ψ and F by counting scatter, checks the encoded-coordinate and sentinel invariants, and walks the resulting records against the production implementation’s output. The recorded run passed both ordinary and marked-start round trips across all 1,513 blocks.

This means the proposal has progressed through three distinct stages:

Mathematical design → executable reference encoding → corpus-tested output equivalence.

What remains is the performance implementation: the ordinary loop, lane state, RAW parking, segment termination and assembly integration. The journal explicitly leaves those unvalidated.

One small qualification to preserve

Treat 0.72% RAW rows as a storage-frequency result, not yet as the measured slow-path frequency of an optimized segmented walker.

The reference code counts RAW rows while constructing the encoding. Its correctness walk follows one chain for the required output length; it does not measure the dynamic RAW-access frequency of a multi-lane implementation. The journal’s statement equating rows with visits should therefore remain conditional on the relevant traversal coverage, rather than becoming a general performance assumption.

That needs a wording clarification, not another census or workstation run.

2. P87’s cleanup addressed the actual integration problem

The P87 changes are appropriately narrow. The updated record now distinguishes the allocation-free Segmented path from the separate-buffer Plain path, adds differential checks between the two forms of segmented inversion, and retains fallback and error-reuse checks. The development-machine tests are reported passing.

That is better than simply changing the failing assertion from “140,004” to “0.” It keeps the memory-accounting contract meaningful for both implementations.

Two boundaries remain:

The 68.9 versus 72.5 MB ledger figures are reservation-accounting figures, not a new RSS measurement. And fixing the integration tests does not establish the target-machine production benefit.

P87 is now better prepared for its separate production-validation decision. It does not need another theoretical redesign, but its comparison protocol, control precision and machine-code audit are still outstanding.

3. The wake-placement evidence is now preserved properly

The dispatch addendum contains the replay method, input hashes, per-run results and the distinction between the approximately 0.225-core capacity-capped opportunity and the 0.178-core support-demand sensitivity result. It expressly avoids calling these a guaranteed recoverable range or a prediction of affinity-policy performance.

That is the right endpoint for this piece of analysis. We have a reproducible opportunity estimate; further progress now requires choosing and eventually evaluating a policy, not refining the same percentage indefinitely.

The next step is a focused Astra review

I agree with Claude: Astra’s review of the 30-entry design is next. We should now give it the executable reference and census, not merely the original theoretical sketch.

Its assignment should be narrower than “find us a breakthrough”:

Does this specific layout deserve one optimized, compile-and-correctness-only prototype—and what would cheaply falsify its execution-cost advantage?

Astra should inspect the encoding and slow-path invariants, then assess whether the intended direct halfword access survives in a realistic 32-lane implementation. It should also decide whether k/j timing would answer a necessary question first, rather than automatically adding those experiments because the candidates already exist.

Updated handoff for Astra

The proposed 30-entry compact layout now has an executable reference and corpus census in PR #132, commit 1fbc0a0b609ed9584fdafcaf9668308ccbd0358d, journal docs/journal/2026-11-02-compact30-census.md, implementation examples/census30.rs.

Across the established 11 corpora, all specification checks and both scalar reference round trips passed on 1,513 blocks. On primary Wikidata, logical storage is 2.164 bytes per row: approximately 4.2% above FOR-14, while retaining approximately 46% reduction against the shipped table. RAW rows comprise approximately 0.72% of stored rows; do not assume this is the dynamic multi-lane exception rate.

This validates the reference encoding, coordinate transformation, sentinel separation and output equivalence on the corpus. It does not validate an optimized segmented walker, parking behaviour, code generation, construction economics or speed.

Please review the actual reference implementation and census before deciding whether to authorize one bounded optimized prototype. Concentrate on ordinary-path dependencies, lane state, RAW/start/parking transitions, alignment and any hidden coordinate conversions. Assess the direct-construction budget, but do not implement a constructor.

Decide whether the 30-entry prototype is now the most informative next step, whether an existing k/j comparison is a necessary prerequisite, or whether a specific flaw warrants stopping. Do not expand this into an encoding sweep.

The frozen FOR-14 loss and walker-ladder decisions remain unchanged. No new i9 run is authorized.

Where we stand

The research has advanced without another performance trial. We have removed uncertainty about the new representation’s actual size and basic correctness, repaired P87’s integration obligations, and preserved the scheduling opportunity quantitatively.

The next uncertainty is more specific than before: can the compact layout’s simpler access primitive be realized cheaply in the production-shaped walker?

That is a worthwhile question for Astra. Nothing needs running on your workstation now, and there is no reason to start PR monitoring or another implementation track merely to keep Claude occupied.

Here is the complete prompt for your existing Astra conversation, on Max reasoning. It assumes Astra already knows the earlier FOR-14 work, but gives it the new evidence, precise technical questions and a clear decision to make.
`````
