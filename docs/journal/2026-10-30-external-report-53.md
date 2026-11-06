# External research report 53: the decoder-level scaling and P74's economics (verbatim)

A relayed message from the maintainer, unattributed in the relay, after the coordinator
scaled the salvage table's stage gains to the whole decoder.

**What it says:**
- **The scaling is accepted.** P74 projects to about 5.8% of decoder time (Zen 4) and 4.1%
  (Zen 5); #114 to about 1%. These are estimates from stage shares, not end-to-end decoder
  benchmarks.
- **Time and throughput are not interchangeable:** a 5.8% cut in time is about 6.2% more
  throughput (1/(1 − 0.058) ≈ 1.062), with the other stages unchanged.
- **A wording correction:** the figures are *projected decoder gains* from applying the
  measured stage improvements, not strict upper bounds ("ceilings") on every possible P74
  implementation. They hold for the measured implementations, with stage costs assumed
  unchanged, and exclude classification, switching and integration costs.
- **Option C:** it has the best experimental evidence of the surviving ideas, but is not an
  obvious next engineering project. A mid-block switch, with checkpoint and RLE2 state
  preserved, correctness proved, adversarial distributions tested and several
  architectures benched, is substantial work for 4–6%. Keep it in the archive and don't
  reopen P74.
- **No Gate B and no P74 experiment** on the current stage gains alone. Let the two N1
  analyses finish, close the round, then establish whether bzip2 decoding limits the
  release's elapsed time.
- **The hypothesis to test next:** the shipped decoder may already be efficient enough for
  the intended workload, and further stage-local optimisation has diminishing returns.
  Not proved.

**The coordinator's note:** the correction stands. "Ceiling" was the coordinator's word in
the conversation; it does not appear in a committed journal note. Later notes use
"projected decoder gain".

## The message, as received

Claude's correction is important. It changes how much further investment in P74 I would
recommend.

We have a real optimization, but its maximum plausible contribution to the full decoder is
modest. And once we consider the complete Wikidata release pipeline, its value could be
smaller still.

That doesn't invalidate the research. It changes the economic justification for pursuing
it.

The full-decoder perspective

Improvement	Measured stage gain	Estimated whole-decoder reduction
P74, Zen 4	33.2%	~5.8%
P74, Zen 5	19.2%	~4.1%
Shipped one-pass build (#114)	6%	~1%

These are estimates derived from stage shares, not end-to-end decoder benchmarks.

There's one mathematical distinction worth preserving: reducing decoder execution time by
5.8% corresponds to approximately 6.2% higher decoder throughput, assuming the other
stages are unchanged. The two figures shouldn't be interchanged.

Even these estimates are optimistic for P74 because they exclude classification, switching
and integration costs.

Does this change my view of Option C?

Yes, somewhat.

Earlier, I described P74's online switch as our strongest unshipped opportunity. I still
think it has the best experimental evidence of the surviving ideas, but I no longer think
that makes it an obvious next engineering project.

A theoretical 4–6% reduction in decoder time is useful. In a highly optimized library, it
can be worthwhile.

But implementing a mid-block switch, preserving checkpoint and RLE2 state, proving
correctness, testing adversarial rank distributions and benchmarking across architectures
is substantial work.

If decompression isn't on the release pipeline's critical path, that work may have little
practical payoff.

So I would preserve Option C in the research archive rather than reopen P74 now.

One further caution

I agree with Claude's arithmetic, but would adjust one word: ceiling.

The 4–6% figures are better described as projected decoder gains from applying the
measured stage improvements. They aren't strict mathematical upper bounds on every
possible P74 implementation.

A different implementation or workload might perform better. Conversely, the measured
stage gain may not transfer intact when integrated into the full decoder.

For the specific measured implementations and assumed unchanged stage costs, however, the
calculations are appropriate.

What remains worth doing?

My priority would now be to establish the release-level performance opportunity before
another optimization experiment.

That means finding out whether bzip2 decoding actually limits the elapsed time of the
wikidata-pq pipeline. If the release is predominantly constrained by another stage, even a
perfect decoder offers limited improvement.

The two pending N1 analyses should still finish, because correctness and pre-allocation
memory bounds determine whether its mathematical work is worth preserving as a practical
candidate.

But I would not authorize Gate B or another P74 experiment on the strength of the current
stage gains alone.

My present recommendation is to close this research round after the outstanding N1
results, then evaluate the actual end-to-end bottleneck.

The most consequential discovery may be that the shipped decoder is already efficient
enough for the intended workload, and that further stage-local optimization has
diminishing returns.

That's not a claim we've proved yet. It's the hypothesis I would test next—before
committing more time to speculative decoder architectures.
