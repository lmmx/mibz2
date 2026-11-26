# Smooth and striated: what determines the trajectory?

26 November 2026. A further cross-reading prompted by the maintainer's question about the claim-lifecycle diagram in the first frontier blog post (#959). The distinction in that diagram is operationally useful, but philosophically too narrow. This note extends [the earlier cross-reading](deleuze-guattari-cross-reading.md), especially §3.2, rather than replacing its findings or changing permissions.

## Two distinctions, not one

The frontier calls docs, probes and examples smooth, and production, queue and protected changes striated. This classifies **integration risk**. It does not tell us how research is organised or how it discovers its objects. The former distinction is an engineering convention; the latter is the philosophical question.

In *A Thousand Plateaus*, plateau 14 opens by saying the two spaces "exist only in mixture" (p. 474). The authors retain an abstract distinction precisely to investigate concrete mixtures and the asymmetrical passages between them. Their technological model contrasts woven cloth with felt; patchwork develops a different organisation through successive additions of fabric (pp. 475–477). The musical model takes up Boulez's metric and nonmetric space-time, including their transformations (pp. 477–478). The maritime model distinguishes movement between designated locations from movement that constitutes its own significant locations (pp. 478–479). The sea can be charted, and striated spaces can generate new smooth ones. None of this makes smoothness inherently liberatory (p. 500).

The most useful formulation here is that "the points are subordinated to the trajectory" (p. 478). A claim can emerge from following a phenomenon rather than exist beforehand as the destination of an assigned journey.

These are source-based distinctions, not a claim that a software repository literally instantiates the political, aesthetic or mathematical ontology of the book. The following is an engineering interpretation.

| Mode of discovery | Low-risk integration | High-consequence integration |
|---|---|---|
| Predetermined points organise the movement | A formulaic blog outline followed to completion | Implementing a specified feature under a fixed acceptance test |
| Movement discovers or changes its points | Cold exploration yields a new question outside the board | A novel algorithm emerges from following material constraints, then meets exactness and safety gates |

The axes are independent. Rigorous proof can be open-ended in its formation; self-merge permission does not make a prescribed writing task smooth. A new algorithm may be investigated freely while its adoption remains tightly controlled. The interesting passage is how a discovery takes determinate form without foreclosing what can be discovered next.

## What the first afternoon already shows

- [#952](https://github.com/lmmx/mibz2/pull/952) overturned the purported 48 ms delivery floor by reading primary traces. A measurement apparatus had made its own cold-store traffic appear to be a property of the decoder. The map needed changing, not merely following. The correction was then connected to cache contention and future worker-side work by [#954](https://github.com/lmmx/mibz2/pull/954).
- [#962](https://github.com/lmmx/mibz2/pull/962) found the tiny non-Open blocks and corrected the assumed Huffman lookup rate during a cold exploration. A new research direction emerged from the investigation, rather than being the object of a preassigned build.
- [#960](https://github.com/lmmx/mibz2/pull/960) turned a probe into a concrete, frozen paired-oracle registration; [#968](https://github.com/lmmx/mibz2/pull/968) subsequently supplied a prototype under that registration. Local specification and verification can sustain an extensible research programme. The prototype and its workstation result are not the same thing.
- The first-day saturation incident ([#955 review](https://github.com/lmmx/mibz2/pull/955#pullrequestreview-6078377169), [#957 integration](https://github.com/lmmx/mibz2/commit/f88024a9ae9091d04afa2cd244b5d84283c5e8d9)) is the opposite passage: an ownership mechanism meant to enable independent activity impeded finishing when all slots were occupied. This is a specific liveness defect, not evidence that locking is intrinsically undesirable.
- The conversation about the blog diagram became a research trajectory in its own right. The maintainer did not assign a new role or a predefined deliverable; a question changed what was worth thinking about. [#964](https://github.com/lmmx/mibz2/pull/964) records conversational participation and removes editorial approval waits. A less regimented publishing process is useful, but is not by itself proof of a smoother intellectual process.

## The chart must be revisable

"The surface is the map" works as a counter to prose routing and speculation. Yet the measured surface is a representation constructed by a particular experimental apparatus, set of corpora, decomposition and questions. It is not the whole territory.

The measured map can make movement possible: paired oracles decouple work that would otherwise be masked. It can also capture movement: if the only available question is which existing row has the largest room, discoveries that invalidate the rows become invisible. The proper response to a surprising result is sometimes to change the coordinates, not to improve a score inside them. The existing cold exploration and synthesis practices are useful in this respect. Treat the one-in-five exploration cadence as a provisional resource heuristic, not a philosophical definition of smoothness.

The patchwork example also suggests that contributions need not share one global form in order to fit together. Exact evidence, provenance, independent checks and narrow interfaces make heterogeneous findings joinable. This differs from requiring every participant to narrate the same plan. The musical analogy suggests another possibility: the timing of contributions may follow discoveries, with bursts and intervals of inactivity, rather than a uniform rate of completed claims. These analogies suggest questions, not new compulsory stages.

## Consequence: investigate the passages, not the labels

No new process or permission change is proposed here. In particular, do not weaken byte-exactness, preregistration, memory safety, ownership or resource bounds. The existing repair of lease handoff remains separate.

The substantive hypothesis is that research is better organised when evidence can reshape its own questions, while consequential adoption retains explicit gates. Look for the following in ordinary deltas and subsequent work, without installing another reporting requirement:

- Did a contribution originate a question or change a premise, rather than merely close a card?
- Did the changed premise actually alter a subsequent experiment, implementation or negative-result record?
- Did structured measurements open further independent lines, or did the workflow suppress an observation because it was not on the board?
- Could useful work be integrated without requiring the original participant or the maintainer to return?

Counterevidence would include unbounded speculation, repeated discoveries already in the journal, unreproducible claims, increased integration delay, or no validated improvement over a simpler restarted-solo workflow. A few anecdotes do not establish a productivity advantage. The deferred matched-budget pilot is still needed for causal comparison. The above is an interpretive lens for evidence already being left, not an additional adoption test.

One implication for the public article's diagram: its two branches depict different **verification regimes**, not a complete mapping of D&G's spaces. A later editorial pass could label them as such, show how investigation changes claims, and place review/revision of published prose on a feedback path. Do not confuse the diagram with the organisation it depicts.

**Let discovery determine the organisation of work, rather than letting the organisation of work determine discovery.** Rupture should interrupt dependence on a session, not the continuity of an insight; the next participant can begin from this note, disagree with it, or carry it somewhere else.

## Source

Gilles Deleuze and Félix Guattari, *A Thousand Plateaus: Capitalism and Schizophrenia*, trans. Brian Massumi, University of Minnesota Press, 1987, plateau 14, pp. 474–500. [Chapter scan](https://lthdigital.wordpress.com/wp-content/uploads/2020/03/deleuze_guattari_-_the_smooth_and_the_striated.pdf), especially pp. 474–479 and 500. The two short quoted phrases are from pp. 474 and 478; the application to repository work is our interpretation. See also [the existing account](deleuze-guattari-account.md) §14 and [cross-reading](deleuze-guattari-cross-reading.md) §3.2.
