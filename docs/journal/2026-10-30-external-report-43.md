# External research report 43: the guardian's simplification, adopted by the maintainer (verbatim)

The guardian's message, relayed by the maintainer, followed by the maintainer's instruction
"no do as it advises", which adopts it.

**What it decides:**
- **Close PR #118** with a short results comment linking the journal. Gate 1 is negative
  and complete.
- **Authorise the read-only option C feasibility analysis:**
  - explicit tokens per block;
  - how well prefixes represent the block;
  - short-block penalties;
  - a precise handoff boundary for both kernels;
  - the cost of the explicit-token denominator;
  - one-way against reversible switching;

  with measured observations kept apart from estimates.
- **Then make N1's construction economics the primary analysis priority:** the dominant
  costs, the feasible construction mechanisms, and the most useful falsifiable next
  hypothesis. This does not authorise N1 Gate B.
- **No implementation or benchmark** of A or C, and no reopening of Gate 1. Phase 2,
  production, the i9 and new experiments stay unauthorised. wikidata-pq #97 is untouched.

**The coordinator's actions:**
- **PR #118 closed** with the results comment. The comment's server-appended footer was
  removed, and the PR's activity subscription ended.
- **The option C analysis** is appended to `2026-10-30-p74-selection-architecture.md`.

## The guardian's message, as received

````text
Claude is drawing the authorization boundary correctly. And now that it’s just the two of us reviewing the research, I’d simplify the decisions.

My recommendation is to close P74 Gate 1, authorize the read-only investigation of C, and make N1 the primary research priority. I would not authorize another P74 implementation experiment yet.

The important distinction is between finishing P74 responsibly and letting P74 consume the entire research programme.

How I’d decide the three questions

1. Close #118 — yes. Its registered verdict is negative, and the result is documented. Keep the journal and the substantial Wikidata stage gain, but don’t leave the experimental PR open as though another amendment might rescue Gate 1.

2. Read-only C analysis — yes, narrowly. The most valuable work is measuring how representative early MTF-rank prefixes are of the remaining block, and establishing a correct handoff boundary. I would want the analysis to distinguish actual token-sequence evidence from conclusions inferred only from whole-block averages. No implementation or benchmarking.

3. N1 — elevate its priority. N1 has a potentially larger architectural payoff, but its construction-cost estimate is a serious obstacle. I’d prioritize determining whether the multi-byte Ψ table can be built economically, rather than continuing to refine P74 selectors indefinitely. That does not authorize N1 Gate B.

There’s one extra requirement I’d add to C’s analysis: account for the asymmetry of switching direction. Switching from P74 to shipped is potentially feasible after materializing the list; switching back later could require reconstructing the register-held state. A one-way switch is simpler, but it cannot recover if the rank distribution changes again. The research should quantify whether that limitation matters on real blocks.

Suggested instruction to Claude

I explicitly authorize closing PR #118 with the short results comment and journal link previously described. The P74 Gate 1 verdict remains NEGATIVE, and the experiment is complete.

I also authorize the read-only Option C feasibility analysis under the standing research mandate. Use existing code and available corpus data only. Assess explicit-token counts per block, prefix representativeness, short-block penalties, a precise handoff boundary for both kernels, the cost of maintaining the explicit-token denominator, and the implications of a one-way versus reversible switch. Separate measured observations from estimates.

Do not implement or benchmark Option C or Option A, and do not reopen P74 Gate 1.

Make N1 the primary research-analysis priority after this bounded C assessment. Focus on the construction economics of its multi-byte Ψ representation: identify the dominant costs, compare feasible construction mechanisms, and establish the most useful falsifiable next hypothesis. This is analytical work only, not authorization for N1 Gate B.

Phase 2, production integration, i9 experiments and any other new performance experiment remain unauthorized. Leave wikidata-pq issue #97 untouched.

That gives Claude useful work without opening a new optimization campaign.

My strategic preference is to preserve P74 as a demonstrated workload-specific opportunity, then shift the main research effort back toward N1’s larger potential payoff.
````

## The maintainer's instruction, as received

````text
no do as it advises
````
