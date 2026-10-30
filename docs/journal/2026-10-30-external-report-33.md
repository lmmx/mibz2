# External research report 33: the guardian's catch-up review, and the researcher's notes for it (verbatim)

A research input, not a finding of this repository. The maintainer relayed both on
2026-10-30, after external report 32 (`2026-10-30-external-report-32.md`).

**What it contains:**
- **The guardian's review of PR #117 and the Gate 1 pre-registration**, after a repository
  update. It recommends authorising Gate 1 only, and drafts an authorisation text.
- **The researcher's assessment of that review.** The researcher reviewed it from the
  journals the maintainer sent (an attachment, not reproduced here). It finds no
  material misunderstanding, and lists details for the guardian to keep in view.

**Neither is an authorisation.** The guardian's draft is a suggestion to the maintainer,
and the researcher notes that sending it only to bring the guardian up to date is not
needed. Gate 1 stays not started until the maintainer authorises it.

**The coordinator's checks:**
- The guardian's figures match the register: 165 candidate ideas (P-numbers up to 181)
  and 63 contradictions.
- Its "projected 3–6% one-core" is the pre-registration's own figure, wider than the
  model's 4.6–5.8%, which the pre-registration now explains (its "Open points for
  review").
- "The compiled-code bottleneck is identified" holds for the shipped kernel only: no
  P74 machine code exists yet.

The text reached us as rendered text (cards flattened) and is kept as received.

## The guardian's review, as received

````text
Welcome back. You’re caught up—and the project has moved considerably further than when we last discussed the reverse-MTF research.

I checked mibz2 PR #117⁠￼ and read the P74 Gate 1 preregistration⁠￼ directly.

The big development is that the exploratory phase has become a finite experimental programme. The research has produced two sharply defined next tests, rather than another expanding list of possible optimizations.

Where we stand

Research-question burn-down

84 / 84

Every recorded question has a disposition; this does not mean every scientific uncertainty has been resolved.

Candidate ideas inventoried

165

Historical contradictions catalogued

63

New experimental gates authorized

0

The two immediate candidates serve different purposes.

P74 — Register-resident inverse MTF

Immediate, inexpensive experiment

Hold the first 32 MTF entries in an AVX2 register to remove a suspected store-to-load dependency. The projected 3–6% one-core decoder gain is still a model, not a measurement.

Gate 1 compares shipped, branching and branch-free variants in the same binary, including both the ordinary MTF and cooperative checkpoint kernels.

N1 — Multi-byte Ψ navigation

Higher theoretical upside, larger construction risk

The mathematical representation can cover multiple output bytes per lookup, and the reported Wikidata table sizes look promising.

Its problem is construction traffic. Gate B must show that the representation can be built economically before a traversal implementation is justified.

My assessment of P74 Gate 1

The preregistration is unusually thorough. In particular, I like four decisions Claude made:

* It includes the checkpoint kernel, which the earlier CI stage table omitted despite production cooperative decoding using it.
* It tests all alphabet sizes and the costly fallback paths, rather than optimizing Wikidata’s common ranks and ignoring random inputs.
* It includes a same-binary comparison and layout-noise control, addressing an actual source of historical benchmark ambiguity.
* It limits iterations to diagnosed code-generation fixes, preventing a negative result from becoming an open-ended tuning expedition.

The positive threshold is a meaningful improvement of at least 5% in the checkpoint MTF stage, with no unacceptable regressions on other corpora. Even success at this gate only authorizes a separate full-decoder evaluation; it does not authorize production integration.

One subtle point: P74 is not the breakthrough itself. It is a reasonably inexpensive test of a causal performance hypothesis. It could produce a useful improvement while N1 remains the more ambitious attempt to change the underlying cost model.

That’s a healthy balance between engineering and fundamental research.

What I recommend now

I would authorize P74 Gate 1.

The questions that would have justified delaying it have largely been answered: the candidate has a correctness argument, the historical overlap has been assessed, the compiled-code bottleneck is identified, and the benchmark has been preregistered before implementation.

It requires no i9 time and does not change the production decoder during Gate 1.

I’d leave N1 Gate B unauthorized for the moment, so we can inspect its builder cost model independently and avoid confusing the two experiments.

If you want to send Claude a clear authorization, use:

I’ve reviewed the research synthesis and the P74 Gate 1 preregistration in PR #117.

I authorize P74 Gate 1 only, under the frozen protocol in docs/journal/2026-10-30-p74-gate1-preregistration.md.

Proceed with the same-binary shipped/branching/branch-free comparison, both plain and checkpoint MTF kernels, correctness controls, compiled-code inspection and the registered CI measurements.

Keep the production decoder unchanged during Gate 1. Observe the preregistered stopping rules and iteration limit, and report the measured checkpoint-stage result separately from any full-decoder inference.

This does not authorize P74 Phase 2, N1 Gate B, any i9 benchmarking, or other decoder experiments. After Gate 1, report the evidence and proposed next decision before proceeding further.

One final point: the failure of the P74 checker wait loop is not a failed research experiment. The checker finished and its results were saved; only the pgrep-based waiting mechanism was defective. No rerun is needed on that account.

My view is that the programme is ready to resume experimental work—carefully, but decisively. The inventory has done its job. We now have well-defined hypotheses that can actually be falsified, without turning the next week into another uncontrolled optimization sweep.
````

## The researcher's notes for the guardian, as received

````text
The Guardian’s assessment is sound. It has recovered the important research conclusions from our earlier session, and I don’t see a material misunderstanding that needs correcting before it continues.

In particular, it correctly distinguishes between completing the 84-question research inventory and actually resolving every performance uncertainty. It also understands that P74 is a promising but unproven optimization, while N1 carries greater potential upside and substantially greater construction risk.

There are, however, three details from our work that I would make sure the Guardian retains, especially if it’s going to review experimental results.

1. P74’s most important evidence is causal, not just its projected gain. The shipped 0.2.1 disassembly confirms overlapping memory accesses that cannot use ordinary store-to-load forwarding. The earlier 1.058× store-ordering experiment supports the importance of that dependency, but does not isolate it conclusively. P74 is testing whether removing that dependency actually improves the complete MTF loop.

2. The branch-free variant is a serious competitor, not merely a fallback design. Our one-shuffle path covers roughly 84.6% of explicit Wikidata tokens, but Claude’s simple branch-predictor model predicts enough mispredictions to consume much of its theoretical advantage. The two variants must be judged experimentally, without assuming the one-shuffle version wins.

3. The RLE2 call boundary is a potentially decisive detail. The shipped kernel calls memset frequently. Inlining short runs reduces modeled library-call frequency from about 329 to 49 per 1,000 explicit tokens. The later fixed-vector fallback analysis brings that down to approximately 32.5 on Wikidata. This is important because external calls complicate keeping the MTF head resident in a YMM register.

The Guardian may already know these details from the repository. Its summary doesn’t contradict them; it simply doesn’t mention them.

What I’d send the Guardian

Your assessment matches the researcher’s conclusions. A few details from the earlier research sessions are worth keeping in view when independently reviewing P74 Gate 1:

* The shipped 0.2.1 disassembly confirms overlapping MTF-list loads and stores that prevent ordinary store-to-load forwarding. The historical 1.058× store-ordering result is supporting evidence, not definitive causal proof.
* The branching and branch-free P74 variants are genuine competing hypotheses. The one-shuffle path covers approximately 84.6% of explicit Wikidata tokens, but modeled branch mispredictions could consume much of its advantage.
* RLE2 calls create an ABI preservation cost for the register-held MTF head. Inlining short runs and using fixed-size vector tail shifts substantially reduces the projected call frequency.
* The eventual acceptance criterion is not simply faster MTF. It is improved four-worker full-decoder throughput without worsening CPU-seconds/GB or materially increasing peak RSS.
* N1’s s-run partition and rk bound predate our research (Kärkkäinen, Kempa and Puglisi, 2012). Any novelty claim should be restricted to the specific multi-byte inversion and construction approach unless further literature supports more.

Please continue to challenge the causal inference and benchmark interpretation independently rather than treating the preregistration or the researcher’s projected gains as proof of performance.

One other thing: the Guardian’s suggested authorization is appropriately scoped, but you don’t need to send that authorization merely to bring it up to date. Whether to authorize Gate 1 is a separate decision for you and Claude.

My conclusion: The Guardian is sufficiently caught up to perform its role. Sending the additional context above would sharpen its review, but there’s no reason to repeat the entire research history.
````
