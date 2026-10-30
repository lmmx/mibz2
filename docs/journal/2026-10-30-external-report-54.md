# External research report 54: pipeline overlap in the release-level analysis (verbatim)

A relayed message from the maintainer, unattributed in the relay, endorsing the plan after
report 53 and the correctness result.

**What it says:**
- **The plan is agreed:** finish the analyses, examine the existing end-to-end measurements,
  and stop for review. No further experiments are needed for that decision.
- **Account for pipeline overlap explicitly:**
  - decompression's share of CPU time is not its share of elapsed time when it runs
    concurrently with other stages;
  - a decoder with a large CPU share may barely affect wall time if another stage is the
    critical path;
  - less decoder CPU may still help competing stages through reduced contention, even
    when decoding isn't the longest stage. That needs measuring, not an assumed Amdahl
    calculation.
- **N1:** without a correct walk, credible construction economics and a bounded
  retained-memory plan for P^{2|4}, there is no compelling reason to authorise Gate B.
- **Option C stays archived** until the release-level payoff of faster decoding is known.

## The message, as received

I agree with this plan. We have reached the point where understanding the release-level
impact is more valuable than finding another decoder optimization.

The distinction between measured stage gains and projected decoder gains is now correctly
documented. That removes a source of optimism that had been creeping into our
interpretation of P74.

There is one research detail I'd emphasize in the upcoming release-level analysis:
pipeline overlap must be accounted for explicitly.

If decompression runs concurrently with other stages, its fraction of total CPU time is
not necessarily its fraction of elapsed time. A decoder consuming substantial CPU
resources might still have little effect on release wall time if another stage determines
the critical path.

Conversely, reducing decompression CPU demand could help competing stages through reduced
contention even when decompression isn't the longest stage. That effect would need
measurement rather than an assumed Amdahl's-law calculation.

For N1, the remaining decision is clear: if cannot demonstrate a correct walk, credible
construction economics and a bounded retained-memory plan, there's no compelling reason to
authorize Gate B.

I'm also comfortable leaving Option C archived. Its measured stage improvement is real,
but the engineering investment isn't justified until we know how much faster decoding
would actually make the release.

My recommendation is unchanged: finish the analyses, examine the existing end-to-end
measurements, and stop for review. No further experiments are needed to reach that
decision.

This has become a more disciplined research programme than it was at the beginning: we're
no longer judging ideas primarily by their theoretical speedups, but by whether they can
materially improve the system users actually run.
