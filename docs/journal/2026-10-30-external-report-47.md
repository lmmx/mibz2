# External research report 47: the researcher on the reconciliation's interim findings (verbatim)

The researcher's message, relayed by the maintainer after the interim report on the
traffic and peak reconciliation and the four-copy note.

**What it says:**
- **A material deterioration in N1's engineering case:**
  - construction traffic is higher;
  - the shipped build is one pass now;
  - P⁸'s retained high-water mark is large;
  - the four-copy factor came from stale code;
  - the whole-corpus target is tighter (8.8 against 10.2 cycles per record, about −13.7%).
- **Not yet defeated.** The figures for P⁸ must not be carried over to the cheaper plans.
  The shipped 4.5 MB baseline waits on the current-master reconciliation. Bytes moved are
  not cycles.
- **The four-worker advantage is now a hypothesis with almost no direct N1 support.** The
  earlier large four-copy threshold should no longer weigh in the decision.
- **Two answers needed before recommending Gate B:**
  1. Can the best partial-depth plan meet the peak-RSS requirement under a realistic
     sequence of high-r blocks? Measured as the maximum simultaneously live, retained
     capacity per plan, on equivalent decoder boundaries.
  2. The exact whole-corpus build-cost threshold against the current shipped build, in
     nanoseconds per decoded byte, with the median cycles per record secondary. Every
     baseline figure labelled by its source commit.
- **The researcher's verdict:** N1 is high risk. Its recommendation to the guardian is not to
  authorise Gate B on the current evidence. Be prepared to close N1 if the corrected figures
  leave no credible route.

**The coordinator's action:** both answers fall inside the reconciliation's tasks 2 and 3,
and were passed to it as additions. Nothing new was started.

## The message, as received

I think your concern is justified. This is a material deterioration in N1's engineering
case, not merely another conservative correction.

The most serious finding is the memory footprint. N1 was supposed to offer a smaller
navigation structure without materially increasing peak RSS. We now have evidence that the
construction process may require substantially more retained memory than the shipped
implementation.

The second serious finding is that the builder is competing against a more efficient
shipped implementation than some of the earlier models assumed.

That said, I would not declare N1 technically defeated yet. Some findings invalidate
earlier claims; others narrow the implementation that could conceivably succeed.

What has changed

Finding	Effect on N1
P⁸ construction traffic revised upward to 21.7–25.6 B/B	Negative; construction is less economical than modeled
Current shipped build uses a single pass	Negative for N1's relative construction advantage
P⁸ high-water memory estimated at 9.5–10.7 MB	Major problem for the RSS constraint
Four-copy shipped-build slowdown measured on stale code	Earlier contention model loses support
Whole-corpus builder target tightens from 10.2 to 8.8 cycles/record	Makes the pass criterion substantially harder
N1 walk may use a smaller active working set	Still potentially favorable, but unmeasured

The direction is now unmistakable: the plausible N1 design space is narrowing, and the
optimistic assumptions are being removed faster than the evidence of actual speed
improvements is accumulating.

1. Memory is now N1's biggest structural problem

The distinction between resident table size and peak allocation matters enormously.

Earlier, N1 looked attractive because its navigation representation was smaller than the
shipped T-vector. But the latest analysis reveals three different quantities:

* Active navigation table: potentially 1.0–1.5 MB, versus approximately 3.6 MB shipped.
* Median construction peak: approximately 4.03 MB for P⁸.
* Worst retained capacity: approximately 9.5–10.7 MB for P⁸, according to the current
  analysis.

Those are not interchangeable.

If workers retain their allocations, a high-rank block can permanently raise a worker's
capacity requirement for the remainder of the decode. On four workers, the additional
retained capacity could be substantial, depending on allocation reuse and which workers
encounter pathological blocks.

That undermines the original RSS argument.

However, the reported 9.5–10.7 MB is for P⁸, not necessarily for the cheaper candidate. We
should not transfer that figure to every N1 design. Nor should we accept the old 4.5 MB
shipped baseline until the current-master reconciliation is complete.

The right test is the maximum simultaneously live, retained allocation capacity for each
candidate plan, on equivalent decoder boundaries—not an attractive median.

2. The construction economics are worse, but not conclusively fatal

The new whole-corpus correction matters:

\frac{8.8}{10.2}\approx0.863.

That is a 13.7% reduction in the allowable cost per record for the proposed pass
criterion.

The earlier reference estimate of 16–27 cycles per element from the untuned move-structure
builder already sat above the desired 10.2-cycle threshold. With an 8.8-cycle whole-corpus
target, the required improvement becomes more demanding.

There is also a subtle modeling issue worth keeping separate.

The new 21.7–25.6 B/B traffic estimate reflects a larger construction workload, but it does
not automatically mean execution time increases by the same proportion. Data may be reused
from cache, and instruction dependencies can dominate. We should not convert bytes moved
directly into cycles without measurement.

Likewise, the new one-pass shipped build strengthens the baseline, but some earlier
measured shipped stage timings may already include that optimization. A stale code
inspection and a stale benchmark are different problems. Claude should label every
baseline figure by its actual source commit before recalculating break-even.

For that reason, the precise new speedup opportunity remains uncertain. The direction of
the corrections is negative; their combined quantitative effect is not yet settled.

3. The four-worker advantage is no longer something I'd rely on

This is where the cumulative evidence is particularly damaging.

The original four-copy argument was that the smaller Ψ table would avoid much of the
shipped walk's slowdown.

We now know that:

* The earlier model contained an arithmetic error.
* At least one benchmark was attributed to the wrong microarchitecture.
* Much of the observed slowdown appears attributable to SMT resource sharing.
* The shipped build's four-copy contention factor came from older code.
* No corresponding four-copy slowdown has been measured for the N1 walker.

Consequently, the 4-worker performance advantage is now a hypothesis almost entirely
unsupported by direct N1 measurements.

A smaller active table may still help cache locality. But the earlier large four-copy pass
threshold should no longer influence the investment decision.

4. What I would change about our research recommendation

Previously, I was leaning toward a tightly preregistered Gate B after the correctness
check.

I'm less enthusiastic now.

I would finish the correctness checker and current-master reconciliation, then insist on
two additional answers before recommending Gate B:

* Can meet the project's peak-RSS requirement under a realistic sequence of high-rank
  blocks? If it cannot, fast construction is insufficient for promotion.
* What is the exact whole-corpus build-cost threshold against the current shipped
  implementation, expressed in nanoseconds per decoded byte? That is the correct economic
  boundary; the median cycles-per-record target is only a secondary diagnostic.

If either is clearly unfavorable, Gate B may not be worth the engineering expense.

If both remain plausible, a small builder-cost falsification experiment would still be
reasonable, but with a deliberately strict stopping rule.

My updated verdict

N1 research assessment

High risk

What remains in its favor

The mathematical construction and partial-upgrade architecture remain interesting. A
reduced number of dependent navigation lookups is a credible algorithmic advantage.

What now works against it

Construction complexity, tighter builder economics, potentially unacceptable retained
memory, and the loss of the assumed four-worker cache advantage.

What could still change the conclusion

A correct partial-upgrade implementation with bounded allocation capacity and a
substantially cheaper builder than the prior move-structure prototype.

I also want to correct the tone of my previous assessment. I said N1 was showing genuinely
promising signs. At that point the cheaper partial-depth plans were encouraging, but the
cumulative evidence now supports a more skeptical view.

The mathematics is promising. The performance engineering is not yet.

My recommendation to the Guardian would be not to authorize Gate B on the current
evidence. Finish the two outstanding analyses, establish whether the best partial-depth
candidate can meet RSS and construction-cost constraints, and only then decide whether one
final falsification experiment is worth the effort.

If the corrected figures leave no credible route to the original four-worker throughput,
CPU and memory objectives, we should be prepared to close N1 rather than preserve it
indefinitely because the idea is elegant.

So yes: I share your concern. N1 isn't dead, but it is now a high-risk candidate that needs
to earn its next experiment.
