# External research report 61: the invariant accepted; option C's worst case first (verbatim)

A relayed message from the maintainer, unattributed in the relay, after report 59's reply
and the objective's correction (report 60).

**What it says:**
- **The runs-and-tokens result is a mathematical invariant,** not a correlation. The
  earlier caution was too much. Direct level-1 emission is possible without a separate
  run-detection pass, but it doesn't remove N1's doublings, so it won't rescue the builder
  alone.
- **The Gate B draft is stronger** for charging fallback blocks' wasted work.
- **P74:** the 10% threshold classifies whole blocks; it does not by itself give a safe
  mid-block policy. A block can have 4,096 low-rank tokens and then hundreds of thousands
  of high-rank ones. The question: how quickly an online policy can detect an unfavourable
  change, while monitoring stays cheap enough not to eat the fast path's gain. Compare
  periodic checkpoints, counters updated only on fallback, and the maximum loss before
  switching. None needs implementation now.
- **N1:** keep Gate B as a possible falsification test, with fallback work charged, and
  without treating correctness as established at production block sizes.
- **Neither decision depends on wikidata-pq's release-level performance.** It can proceed
  as context. mibz2's throughput, CPU and memory are the objective.
- **Preference:** investigate P74's online switching before more engineering on N1.

**The coordinator's action:** the read-only option C worst-case analysis is running. It
covers a per-symbol cost model by rank class, the adversarial worst case, the bounded
monitoring policies above and the trade-off against the controls. The release-level
analysis was stopped on the maintainer's instruction in report 60; it can be resumed as
context if wanted.

## The message, as received

Claude's latest corrections are useful, and I agree with both. They strengthen the
technical foundation of the research without pretending either candidate has passed its
performance requirements.

The runs-and-tokens result is particularly elegant. I was too cautious in questioning it
earlier.

Since MTF moves each emitted symbol to the front, the next rank is zero exactly when the
next byte of repeats the preceding byte. Consequently, nonzero ranks identify run starts,
with the stated first-position exception.

That is a mathematical invariant, not merely a correlation observed on 20 blocks.

What this means for N1

It establishes that the MTF decoder already encounters precisely the information needed to
identify 's runs. Direct level-1 emission is therefore theoretically possible without a
separate run-detection pass.

But it does not remove N1's expensive interval-doubling passes. Since those account for
most of the modeled construction work, I still wouldn't expect direct emission alone to
rescue the builder.

The Gate B draft is also stronger now that it includes work wasted on blocks that
subsequently fall back to the T-vector. Without that accounting, the memory selector could
look artificially inexpensive.

What this means for P74

I agree that the online switch needs a worst-case analysis, not just more favorable
average-case evidence.

One distinction matters here: the 10% threshold was designed to classify whole blocks. It
does not automatically produce a safe mid-block switching policy.

A block could have 4,096 low-rank tokens followed by hundreds of thousands of high-rank
tokens. A one-time prefix decision would choose P74 and remain on the slower kernel.

The relevant research question is therefore:

\text{How quickly can an online policy detect an unfavorable change,}

while keeping the monitoring cost low enough not to consume the fast path's benefit?

I'd investigate a bounded monitoring policy, rather than assume the first 4,096 tokens are
always representative. That analysis could compare periodic checkpoints, counters updated
only on fallback, and maximum loss before switching. None needs implementation at this
stage.

My recommendation

I would separate the two avenues cleanly:

P74: prioritize a read-only worst-case analysis of Option C. The measured fast-path gain
already exists, so this would address its principal remaining integration risk.

N1: keep Gate B as a possible falsification test, but don't dilute its budget by omitting
fallback work or treating correctness as fully established at production block sizes.

And importantly, I would not make either decision contingent on wikidata-pq release-level
performance. That analysis can proceed as useful context, but mibz2's own throughput, CPU
efficiency and memory requirements remain the objective.

My present preference is to investigate P74's online switching before committing more
engineering work to N1. The measured evidence is stronger, and the unresolved problem is
narrower.
