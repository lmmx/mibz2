<!--
Provenance: the coordinator chatbot's two contributions around the researcher's synthesis
(10-synthesis-research.md), relayed by the user into the working session on 29 October
2026, unedited. Part 1 is the message the coordinator drafted for the user to send with
the researcher's reply; "[Paste researcher's full response]" is its placeholder, where
the user put 10-synthesis-research.md. Part 2 is the coordinator's assessment after
reading the reply. The user sent both, and the coordinator's approval of the cooperative
integration's gates (Part 2, §5) is the one the session acts on. Tables arrived
tab-separated, and the inline figures as plain lines; both are kept so.
-->

## Part 1: the coordinator's draft message

Yes, I’d send that. I would just make the request a little more actionable and give Claude discretion over where the findings belong.

The key is to preserve the research conclusions, turn the actionable ones into tracked work, and avoid distracting Claude from cooperative-decoder integration.

I showed the researcher your intermediate commentary on the literature study, and it responded with the following synthesis. I think it contains valuable next steps worth preserving and tracking.

Please record the researcher’s response verbatim with provenance in the literature-study journal, as we’ve been doing.

For the actionable findings, use your judgment about whether they belong in existing issues, new focused issues, or the research roadmap. In particular:

* Plateau census: measure the joint distribution of plateaus, numerical span and F-bucket boundary crossings in aligned 32-row Ψ groups.
* Segment-marker audit: establish whether cooperative room-walk endpoint detection really requires per-row marker bits in the hot lookup.
* Successor-throughput benchmark: eventually compare candidate representations across 1–32 dependent chains, after selecting formats based on the census.

Please avoid duplicate issues and distinguish established results from hypotheses.

The cooperative decoder’s production integration remains our highest engineering priority. These compact-Ψ investigations should support that work, not delay it.

Here’s the researcher’s response:

[Paste researcher’s full response]

I’d slightly favour adding the census and marker audit as specific tasks under #79, or creating small linked issues if #79 has become too broad. The successor-throughput benchmark can remain a later milestone.

And one point we shouldn’t leave indefinitely unresolved: Claude is still waiting for your approval of the cooperative integration’s performance gates. We can approve the proposed targets in the same message, so it can proceed without another decision round.

That way you’re giving Claude useful research context, a sensible place to preserve it, and permission to keep shipping—not another open-ended reading assignment.

## Part 2: the coordinator on the researcher's synthesis

I agree with the researcher: the literature phase has reached a useful stopping point. We now have enough theoretical and prior-art grounding to stop designing representations on paper and start eliminating the uncertainties that determine which one is worth building.

There is, however, one architectural issue I’d elevate above the others: the segment-marker bitmap.

The plateau histogram tells us whether a compact descriptor could represent the navigation function efficiently. The marker audit tells us whether that descriptor could actually work in mibz2’s cooperative decoder without introducing a new dependency or coordination cost.

Those are different questions, and both matter.

My assessment of the three next experiments

Investigation	Value	Scheduling
Plateau, span and F-boundary census	Very high; cheap, decisive structural evidence	Do now
Segment-marker invariants audit	Very high; determines descriptor feasibility	Do next, in parallel if convenient
1–32-lane successor microbenchmark	High; settles latency versus cache footprint	After selecting candidate formats

I would keep the first two small and read-only. Neither requires a new decoder implementation.

1. The plateau census should be the next small research task

Claude has been recommending it repeatedly, and the researcher now independently agrees.

I would have Claude extend psi_census.rs to report the joint distribution, not merely a histogram of plateau counts:

* Plateaus per aligned 32-row group: 1, 2, 3–4, 5–8, 9+.
* Whether the group crosses an F-symbol boundary.
* Numerical span, including the EF-12 representability threshold.
* Per-block results as well as corpus-wide totals.

One detail is especially important: the percentage of two-plateau groups should be measured by rows and by blocks, not only as a percentage of nontranslation groups. Otherwise a promising-looking category might represent very little of the overall workload.

This census could immediately rule the two-plateau descriptor in or out as a serious candidate.

2. The marker audit could change the architecture

The researcher has identified a potentially substantial opportunity.

Proposed 64-bit directory entry

Markers · 32

Base · 20

Byte · 8

Current proposal: 32 marker bits, 20 base bits, 8 output-symbol bits, 4 format bits.

Hypothetical marker-free two-plateau entry

Two bases · 40

Split · 5

Byte · 8

Tag · 4

57 bits used, 7 spare — conditional on removing marker information from this lookup.

This is attractive, but the researcher’s caution is correct.

Moving markers outside the descriptor doesn’t automatically eliminate their cost. If every step then needs another dependent memory access to detect segment completion, we’ve potentially traded a compact table for a slower decoder.

The audit should first determine exactly when and why markers are consumed, and whether existing checkpoint or segment-length information can replace them without additional work in the hot loop.

I would not implement a marker-free design until those invariants are established.

3. One additional qualification: output bytes

The symbol-per-group result is convincing for the descriptor-based designs: most groups reside entirely inside one F bucket.

But I’d retain a qualification for the plain 24-bit direct-link alternative.

Knowing that only a small fraction of groups straddle boundaries doesn’t mean a bare three-byte-per-row table can recover the output symbol without an additional lookup. It still needs some mechanism to identify the F symbol for the current row.

A directory descriptor supplies that mechanism naturally. A bare 24-bit table doesn’t.

So I’d keep the two designs’ byte-recovery costs separate in our future benchmarks.

4. Meanwhile, cooperative production integration stays first

The research has improved our compact-Ψ programme, but it hasn’t displaced the measured opportunity from cooperative inversion.

Now that v0.1.6 is fully published, I would unblock Claude’s production integration work.

Claude proposed the following acceptance criteria:

Metric	Proposed gate	My recommendation
Full-decoder speed	≥1.15× p4	Strong success target
95% lower confidence bound	Above 1.05×	Appropriate evidence threshold for a major claim
Decoded output	Identical	Mandatory
Peak RSS	≤1.15× baseline	Reasonable budget
CPU consumption	≤1.5× baseline	Reasonable initial budget

I’d approve these as the target for a major performance win, but not make 1.15× an absolute barrier to shipping any improvement.

For example, a reproducible 1.10× full-decoder gain with acceptable memory, CPU usage and correctness would still be valuable. We shouldn’t reject that simply because a research benchmark suggested greater potential.

I’d distinguish headline success from acceptable production improvement, and pre-register both before testing.

5. What I’d send Claude now

v0.1.6 is published, so let’s move forward on production performance.

I approve proceeding with the cooperative-decoder integration programme. Use your proposed ≥1.15× full-decoder speedup with 95% lower bound above 1.05× as the headline success target, plus identical output, peak RSS ≤1.15× and CPU ≤1.5×. Please also report smaller reproducible end-to-end improvements rather than automatically rejecting them; we can decide whether to ship a more modest gain based on its full resource tradeoffs.

The literature study has now converged. Please preserve its final synthesis when convenient, but don’t let documentation work block the engineering programme.

There are two narrowly scoped compact-Ψ investigations worth doing before any new codec implementation:

1. Extend the existing Ψ census to report a joint per-block distribution of plateau count, numerical span and F-bucket boundary status for globally aligned 32-row groups. Include EF-12 eligibility and both group-weighted and row-weighted totals.
2. Audit the cooperative room walk’s segment-marker invariants. Determine whether the 32 marker bits per group must remain available in the hot navigation lookup, or whether endpoint detection can be handled from existing lane/checkpoint state without extra dependent loads or expensive coordination. Analysis first; no speculative rewrite.

These can be small independent tasks, but keep the cooperative full-decoder integration as the main engineering priority. Do not start a new compact-table prototype until the census and marker audit give us a reason to select one.

After the cooperative decoder reaches its measured production baseline, we’ll undertake matched 1–32-lane successor-throughput experiments and compare compact representations against that new baseline.

Keep shipped full-decoder results, inversion-only research results, theoretical claims and storage models clearly separated.

Overall

I think we’ve reached a productive transition.

The original literature search gave us a broad landscape of possible compression methods. Claude’s cross-checks then constrained those ideas using the actual bzip2 implementation and corpus. The researcher has now converged on two concrete questions that can be answered without building another speculative codec.

That’s genuine progress.

My proposed order is: cooperative integration as the main track; plateau census and marker audit as small supporting tasks; then targeted successor benchmarks and compact-Ψ implementation.

I’d stop the literature trawl here. The next significant discoveries are much more likely to come from measurements and code than from another paper.
