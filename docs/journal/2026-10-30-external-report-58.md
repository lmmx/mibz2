# External research report 58: N1's memory objection resolved (verbatim)

A relayed message from the maintainer, unattributed in the relay, after the staged
before-allocation rule (`2026-10-30-n1-traffic-and-peak-reconciliation.md` §4.7).

**What it says:**
- **No longer recommend closing N1 on memory grounds.** The staged rule keeps 221 of 233
  blocks (94.8% of decoded bytes) on N1 with the shipped 7.23 MB a worker retained.
- **Both selectors choose correctly, at different costs.** P74's whole-block count was
  measured and too expensive. N1's is unmeasured and includes discarded construction
  work. N1's uses information the builder already produces, closer to the reuse principle.
- **The construction hurdle remains steep:**
  - about 1.0–1.2 ns/B on the 7763 (modelled);
  - about 2–3× better than the only measured relative (#42, untuned), which doesn't rule
    the target out but doesn't reassure either.
- **The fallback's work must be charged:** RSS preserved doesn't make rejected blocks free.
  94.8% is coverage by bytes, not a speed-up estimate; the other 5.2% may be
  disproportionately expensive.
- **Two decisions, kept apart:**
  - preserving N1 as a credible research result: yes;
  - implementing and benchmarking it next: defer until the release-level analysis. A
    narrowly bounded Gate B only if decoding materially determines release elapsed time.
- **Three of four questions closed in a useful sense:** correctness, a memory-bounded
  design, the builder budget quantified. Whether construction plus navigation wins stays
  open. Gate B stays unauthorised.

## The message, as received

This is a meaningful improvement in N1's position. The memory objection has been resolved,
and that changes my assessment.

Not enough to call N1 viable yet, but enough that I would no longer recommend closing it on
memory grounds.

The new result is quite specific: a bounded-allocation rule keeps 221 of 233 blocks,
representing 94.8% of decoded bytes, on N1 while preserving the shipped decoder's 7.23 MB
per-worker retained-memory footprint. That's a strong result for the tested corpus.

It also reveals something interesting about our broader research: both P74 and N1 have now
found ways to select their accelerated paths correctly. But the cost of making those
choices is different.

	P74	N1
Acceleration	Measured faster MTF stage	Fewer BWT navigation lookups
Selection mechanism	Count ranks before MTF	Inspect record counts before allocating the next level
Selection correctness on tested blocks	Correct	Correct
Selection cost	Measured; too expensive	Unmeasured; includes discarded construction work
Remaining obstacle	Cheap, reliable online selection	Economical record construction

The N1 selector is particularly promising because its decision uses information the
builder has already begun producing. That is closer to the reuse existing work principle
we discussed.

But the construction hurdle remains steep

The critical figures are:

N1  · builder-cost feasibility

Affordable build time

1.0–1.2 ns/B

EPYC 7763, modeled threshold

Required improvement

~2–3×

Versus prior untuned builder estimate

That is a demanding target, particularly because the shipped one-pass builder is already
efficient.

The historical #42 implementation isn't a proof that a tuned builder cannot reach the
target. But it is the only measured architectural relative, and it does not provide
reassuring evidence.

Furthermore, N1's memory-fallback mechanism must charge the cost of work already performed
before fallback. Preserving RSS is necessary, but it doesn't make those rejected blocks
free.

My updated recommendation

I would separate two decisions that we've sometimes conflated.

Is N1 worth preserving as a credible research result? Absolutely. The composition has
survived extensive correctness testing, the lookup reduction is substantial, and a
bounded-memory implementation path now exists.

Is N1 worth implementing and benchmarking next? Still uncertain, and I would defer that
decision until the release-level analysis finishes.

If decompression materially determines release elapsed time, a narrowly bounded Gate B may
be worth the effort. It would directly measure the only major unknown—the builder's
cost—and could falsify N1 quickly.

If decoding contributes little exposed wall time, then even a successful N1 builder may not
justify further development.

One subtlety: the 94.8% coverage is by decoded bytes, not an estimate of potential speedup.
The remaining 5.2% may contain disproportionately expensive inputs, and the fallbacks
themselves have unmeasured overhead. That must stay in the eventual economics.

Where we stand

We've now closed three of the four N1 questions in a useful sense: correctness has strong
experimental support, a memory-bounded design exists on the measured corpus, and the
acceptable builder budget has been quantified.

The fourth—whether the actual construction-plus-navigation implementation wins—is still
open.

This is better news than the previous report. N1 is no longer blocked by an apparently
unavoidable retained-memory increase. Its fate rests primarily on an unusually demanding
construction-speed target.

I agree with leaving Gate B unauthorised while the release-level analysis finishes. That
analysis should help determine whether resolving N1's final uncertainty has enough
practical value to justify the work.
