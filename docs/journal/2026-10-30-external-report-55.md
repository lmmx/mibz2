# External research report 55: the correctness result read (verbatim)

A relayed message from the maintainer, unattributed in the relay, on the partial-upgrade
correctness check (`2026-10-30-n1-upgrade-correctness.md`).

**What it says:**
- **The result separates two questions:**
  - does the partial-depth construction work? Strong evidence says yes, within the
    tested scope;
  - can it be built and used economically? Still unresolved, and the evidence remains
    unfavourable.
- **The mutation tests and the independent reference** are what make it informative.
- **The lookup rates** (0.296 and 0.150 per byte) now describe a correctness-tested walk.
  They are lookup counts, not CPU time.
- **Two corrections narrow N1's margin without invalidating it:**
  - in-place upgrades are invalid, so a separate array is required, with its allocation
    and copying;
  - the record's length must be packed into D's spare bits, a mask and shift on every
    lookup, to be counted in the break-even.
- **Status:**
  - the composition, the upgrades and the capped walk are supported or strongly
    validated;
  - full-block correctness is untested;
  - the construction economics are unfavourable and modelled;
  - the pre-allocation memory bound is pending;
  - the whole-decoder speed-up is unmeasured.
- **No change to the Gate B recommendation.** The correctness objection is removed; the
  memory and cost objections are not. The pre-allocation bound for P^{2|4} is now the
  consequential result.

**The coordinator's note on the separate array:** the cost models already assumed it. The
economics note's §4.2 (f1) says the upgrade "writes to a separate array", and the
reconciliation was briefed to count it. Only the note's §1 summary said "in place"; that
wording is corrected. The reconciliation was asked to confirm explicitly that both arrays
are counted while live, and to state the length-field packing in its layout text.

## The message, as received

Finally, some unequivocally good news for N1. This is the strongest correctness evidence
we've had for its partial-depth design, and it resolves a genuine technical uncertainty.

The result is significant: 5.65 million successful walks and approximately 7.7 million
per-row record checks, including exhaustive small inputs, adversarial examples and
independent reference implementations.

It means we can now separate two questions that were previously entangled:

* Does the partial-depth Ψ construction work? Strong evidence says yes, within the tested
  scope.
* Can we build and use it economically? Still unresolved, and the recent evidence remains
  unfavorable.

What I find particularly encouraging

The mutation testing is more informative than the raw test count. Detecting incorrect
interval boundaries, reversed words, faulty hop lengths and segment-order mistakes
demonstrates that the checker can catch exactly the kinds of errors this algorithm is
vulnerable to.

The independent reference is also valuable. Comparing against the original text rather
than a second implementation of the same Ψ logic reduces the risk of shared algorithmic
mistakes.

And the measured lookup rates now correspond to a walk that has survived substantial
correctness testing:

Partial-depth Ψ navigation

Lookups per decoded byte · measured real slices

0.296

— roughly one lookup per 3.4 bytes

0.150

— roughly one lookup per 6.7 bytes

These are lookup counts, not measured CPU time. They do not establish a decoder speedup.

Two corrections materially affect implementation

1. In-place upgrades are invalid.

The two-byte counterexample ab is decisive. The separate-array requirement is not an
implementation preference; it's necessary to preserve the source records during
composition.

That adds an allocation and copying requirement to the construction and peak-memory
economics. Any existing cost model that assumed true in-place upgrades must be adjusted.

2. The record layout needs an additional decoding operation.

Packing the record length into spare displacement bits appears feasible, but it adds a
mask and shift to every lookup.

Because the algorithm's principal advantage is reducing navigation work, even small
per-lookup overheads deserve to be included in the break-even calculation.

Neither correction invalidates N1. Both narrow the margin available for it to win.

My updated research assessment

Dimension	Status
Mathematical composition	Supported by a five-lemma argument
Correctness of partial upgrades	Strongly validated
Correctness of capped mixed-length walk	Strongly validated
Full-block implementation correctness	Not yet tested
Construction economics	Unfavorable, still modeled
Peak retained memory	Final pre-allocation bound pending
Whole-decoder speedup	Not measured

I'm pleased this came back positive, because it means N1's difficulties are primarily
economic rather than algorithmic.

That's valuable even if we eventually archive it: we would be archiving a correct,
well-characterized technique that doesn't satisfy the project's practical constraints,
rather than an unverified idea.

Does this change my Gate B recommendation?

Not yet.

It removes the correctness objection. It does not resolve the potentially excessive
retained memory or the demanding construction-cost budget.

The next result—the pre-allocation memory bound for —is now especially consequential. If
a simple rule using only block length and run count can prevent memory overruns without
diverting most of the useful workload back to the shipped decoder, then N1 retains a
credible practical implementation path.

If it cannot, a fast builder alone would not be enough.

My verdict: this is real progress, not another paper exercise. The partial-depth algorithm
has passed a serious correctness challenge, including falsification attempts that caught
actual design errors.

But the ultimate engineering question remains open. N1 has cleared its correctness hurdle;
it still has to clear its cost and memory hurdles.
