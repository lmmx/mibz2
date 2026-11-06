# External research report 45: the researcher on the three N1 analyses (verbatim)

The researcher's message, relayed by the maintainer after the three read-only N1 analyses
were started (external report 44).

**What it says:**
- **The brief is read correctly.** The three analyses are rightly separated: partial-upgrade
  correctness, four-copy sensitivity, and the traffic and peak reconciliation.
- **The notation.** The plans are mixed-depth, P^{2|4} and P^{4|8}, not six- or twelve-step
  Ψ tables. The symbols were again lost in the relay; the sentence "the intended notation
  is and , not and" is kept as received.
- **The three outcomes it looks for:**
  - **Correctness:** exact inverse BWT, adversarial segment boundaries included;
  - **Economics:** a plausible advantage with the four-copy walk amplification left
    unknown;
  - **Memory:** whether the corrected traffic and peak keep the claimed advantage.
- **A caution:** lower wall time and lower total CPU are separate achievements. N1 must
  meet both, not buy concurrency with excess CPU.
- **The Gate B revision is sensible:** cycles per record and total build cycles per decoded
  byte together.
- **Scope unchanged:** no new research task, no new experiment. Wait for the findings
  before judging Gate B.

## The message, as received

Good. Claude has interpreted the research brief correctly, and the three analyses are
appropriately separated.

One detail worth noting: the intended notation is and , not and . These are mixed-depth
plans, not six- or twelve-step Ψ tables. Claude's interpretation is correct.

The most valuable part of this round will be the four-copy sensitivity analysis. It could
reveal that N1 remains worthwhile even when its one-core construction costs are
unfavorable—or that our assumed reduction in cache contention is too optimistic.

I'm particularly interested in three outcomes:

* Correctness: whether partial upgrades and variable-length traversal reproduce the exact
  inverse BWT, including adversarial segment boundaries.
* Economics: whether N1 still has a plausible advantage when four-copy walk amplification
  is treated as an unknown rather than assumed to be 1.3×.
* Memory: whether the corrected construction traffic and peak live allocations preserve
  the claimed advantage over the shipped T-vector.

One caution for the four-copy model: reduced wall time and reduced total CPU work are
separate achievements. N1 must ultimately satisfy both, rather than trading excessive CPU
consumption for greater concurrency.

The proposed revision to Gate B's metrics is sensible. Reporting both cycles per record and
total build cycles per decoded byte prevents a misleading result in which individual
record operations are fast but the builder creates too many records.

For now, I wouldn't add another research task or alter the scope. These three
investigations address the remaining material assumptions in N1's case.

My research position: wait for their findings before judging whether Gate B is worth
authorizing. No new experiment is implied by this assessment.
