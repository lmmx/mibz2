<!--
Provenance: the research chatbot's closing note on the literature study, and the
coordinator chatbot's reply to it, relayed by the user into the working session on
29 October 2026, unedited. The user introduced them with "alright just you me and the
coordinator now": the researcher's part in the programme ends here. The coordinator's
"57 bits" line arrived as LaTeX source and is kept so. Its reply closes the theoretical
investigation and sets the engineering programme: the cooperative integration first, the
census and the marker audit as two small tasks under #79, and the successor-throughput
benchmark only if they support compact descriptors.
-->

## The researcher's closing note

Excellent. We’ve got a solid theoretical foundation and, more importantly, two clear questions that can decisively narrow the design space.

I’d carry these into the discussion:

* Plateau census: How many nontranslation groups have 2, 3–4, 5–8 or 9+ plateaus? Include F-boundary crossings and span distributions.
* Marker audit: Can the 32 per-row segment-marker bits be removed from the descriptor without increasing hot-loop latency?
* Conditional next step: If the results favour compact descriptors, compare their dependent-successor throughput against the existing T-vector across 1–32 lanes.

The especially interesting possibility is that two-plateau groups might fit entirely in a 64-bit descriptor, alongside the output byte, if segment markers can be handled elsewhere.

That would give us a genuinely new architectural direction to investigate.

Enjoy the discussion. I’m keen to see what the measurements reveal.

## The coordinator: closing the literature phase

Good. I think we can now close the theoretical investigation and run a focused engineering programme with Claude.

The researcher has left us with three falsifiable questions, rather than another set of speculative representations. That’s exactly where I wanted us to arrive.

I’d make one distinction in how we schedule them.

The plateau census and marker audit should happen now, because they’re inexpensive and can eliminate entire design branches. The successor-throughput benchmark should be conditional on what they find.

For the marker audit, I especially want Claude to examine whether segment endpoints can be recovered from the checkpoints and lane state already maintained by the cooperative walker. We need to establish not merely that the marker bits can be stored elsewhere, but that doing so doesn’t introduce a dependent read or additional synchronization into the critical path.

The two-plateau descriptor is the exciting possibility:

40\text{ bits (bases)}+5\text{ (split)}+8\text{ (byte)}+4\text{ (tag)}
=57\text{ bits}.

That leaves seven bits in a 64-bit word. But it’s only a viable architecture if the marker audit passes and the census shows enough two-plateau groups to justify the added complexity.

There’s also a useful implementation discipline here: the plateau census should report actual groups, including F-boundary exceptions, rather than just modelled aggregate savings.

What I’d have Claude do

I wouldn’t send another long research programme. I’d give Claude a compact direction:

The researcher and I have concluded the literature phase. It’s now just us working through the engineering evidence.

Please proceed with the cooperative-decoder production integration as the main priority. I approve using the proposed ≥1.15× full-decoder gain as the headline success target, with identical output, ≤1.15× peak RSS and ≤1.5× CPU. Report smaller reproducible gains too, rather than rejecting them automatically.

For #79, please schedule two small, independent investigations:

1. Extend the existing census with a joint distribution of plateau count, span and F-bucket boundary status for aligned 32-row groups.
2. Audit the cooperative room walk’s segment-marker invariants, specifically whether endpoint detection can leave the hot descriptor without adding dependent loads, synchronization or expensive lane-state updates.

Record the findings in the existing issues and journal, avoiding duplicate tracking.

Only if these results support compact descriptors should we build a matched dependent-successor benchmark across 1–32 lanes, comparing them with the current T-vector.

No additional literature research is needed. Focus on evidence, integration and shipping.

My preferred next checkpoint is Claude reporting the cooperative integration plan and the two small research findings. After that, we can make a concrete choice about whether to prototype the two-plateau descriptor or return to FOR/EF.

The aim is no longer to find more clever representations. It’s to make mibz2 faster, with each experiment answering a question that matters to the shipped decoder.
