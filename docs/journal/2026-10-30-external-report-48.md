# External research report 48: the guardian on the reconciliation's interim findings (verbatim)

The guardian's message, relayed by the maintainer after the interim report and the
researcher's report 47.

**What it says:**
- **N1 is promising but economically high-risk,** not a likely breakthrough awaiting
  implementation. Two attractive assumptions are gone: a large four-copy advantage, and
  lower overall memory.
- **The biggest setback** is the four-copy correction, from about 31 to about 10–16 cycles
  per record. Since the four-copy slowdown is mostly SMT sharing, smaller tables probably
  can't make up for an expensive builder.
- **Still positive:** the partial-upgrade construction.
- **Not final yet:** the build ratios and memory comparisons, until the post-#114
  accounting arrives.
- **No Gate B now. Four questions once the pending results arrive:**
  1. Does the partial-upgrade construction pass independent correctness checks?
  2. Is a builder below roughly 9 cycles per element across the corpus credible, given
     #42?
  3. Does the build-plus-walk advantage stay meaningful against the current shipped
     decoder?
  4. Is there a credible memory plan for high-r blocks without raising retained worker
     RSS?

  If the answers stay favourable, a small, tightly pre-registered Gate B could be worth
  running as a falsification experiment.
- **The guardian's leaning:** about 65% negative, 35% positive, as a qualitative judgement.
  Keep the partial-upgrade design, finish the two pending analyses, and commit no further
  implementation effort.

**The coordinator's action:** none new. All four questions are covered by the running
correctness check and the reconciliation, with its report-47 additions. The final report
answers them in this order.

## The message, as received

More negative than positive for N1's original performance thesis. But not negative enough
to abandon N1 yet.

The latest analyses have removed two of N1's most attractive assumptions: that four
concurrent decoders would give it a large additional advantage, and that its compact
navigation table would reduce overall memory consumption.

That's significant. I would now classify N1 as promising but economically high-risk,
rather than a likely breakthrough awaiting implementation.

How the evidence has shifted

Finding	Direction	Significance
Partial-depth tables and reduce construction work	Positive	Still a credible route to faster navigation
Full P⁸ construction traffic is ~22.5 B/B, rather than ~17 B/B	Negative	The original builder model was optimistic
Four-copy pass threshold falls from ~31 to ~10–16 cycles per record	Major negative	Removes much of the hoped-for concurrency advantage
Four-copy slowdown largely reflects SMT contention	Negative	Smaller tables may not solve the dominant bottleneck
Full P⁸ high-water memory could reach 9.5–10.7 MB	Major negative	Weakens the lower-RSS argument
Whole-corpus builder budget is tighter than median-block budget	Negative	Raises the bar for a useful builder
Partial-upgrade correctness check	Pending	A necessary mathematical and implementation gate

The biggest setback is the four-copy correction

The original estimate suggested N1 could tolerate approximately 31 cycles per construction
element at four copies.

That was unusually attractive because the earlier #42 implementation suggested a much
higher construction cost than the optimistic N1 model.

Now we're looking at approximately 10–16 cycles per element for a strong four-copy result,
depending on assumptions.

That puts N1 back in essentially the same difficult territory as the one-thread experiment.

And the historical evidence that looked like cache contention now appears substantially
explained by two workloads sharing each physical core through SMT.

In other words, N1 probably cannot rely on smaller tables to compensate for an expensive
builder. The builder needs to be genuinely efficient.

There is still a real opportunity

The positive finding I wouldn't discount is the partial-upgrade construction.

A representation that offers four-byte or occasionally eight-byte navigation without fully
materializing those deeper tables may reduce the amount of work substantially.

And the four-copy analysis has not established that N1 fails. It has established that the
earlier optimistic explanation for why it would succeed was unsupported.

A tuned builder at roughly 8–10 cycles per element might still be competitive on some
machines and distributions. We don't know whether that is achievable.

There is also an important unresolved wrinkle: the shipped baseline changed in PR #114, and
some historical comparisons used the older two-pass builder. Claude was right to flag that.
Until the revised accounting arrives, I wouldn't treat the newest build ratios or memory
comparisons as final.

What I would do next

I would not authorize Gate B immediately. Claude is already finishing the correctness
checker and revising the shipped-side accounting. Those results could materially affect
our decision.

Once they arrive, I would apply a tougher standard:

* Does the partial-upgrade construction pass independent correctness checks?
* Is a builder cost below roughly 9 cycles per element across the corpus credible, given
  the historical #42 evidence?
* Does the prospective build-plus-walk advantage remain meaningful against the current
  shipped decoder?
* Is there a credible memory plan for high-run-count blocks without increasing retained
  worker RSS?

If the answers remain favorable, a small, tightly preregistered Gate B could be worthwhile
as a falsification experiment—not because we're confident N1 will succeed, but because
the construction-cost uncertainty is decisive.

My present leaning: approximately 65% negative, 35% positive for N1 as an economically
viable decoder improvement. That's a qualitative judgment, not a statistical probability.

The research process itself is producing valuable results: it's identifying and
correcting optimistic assumptions before they become expensive implementation work.

But for our original ambition of a substantial reduction in full-decoder cost, N1 now
looks considerably less likely to deliver than it did before these corrections. I would
preserve the partially upgraded design, finish the two pending analyses, and avoid
committing further implementation effort until we see them.
