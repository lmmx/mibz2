# External research report 60: the objective is mibz2 in its own right (verbatim)

The maintainer's instruction, quoted, and the reply it was given, relayed together.

**The maintainer's instruction:** "the point isnt about our wikidata-pq critical path, we
are trying to accelerate mibz2 in its own right, take that hedge back".

**What changes:**
- **The objective** is mibz2's own decompression throughput on real multicore hardware,
  without worse CPU efficiency or materially more memory. Wikidata is a benchmark workload,
  not the sole justification. Whether wikidata-pq's critical path exposes a gain is not an
  acceptance gate.
- **Withdrawn:** the hedges in reports 52–59 and in the coordinator's replies that made P74
  or N1 conditional on release-level exposure ("if decoding materially affects release
  elapsed time", "the release-level analysis decides whether either is worth pursuing").
- **The release-level analysis is stopped** before reporting. Nothing from it is
  committed.
- **Unchanged:**
  - negative controls stay: a Wikidata gain that badly degrades random or records data is
    not automatically a good general-purpose optimisation;
  - P74 is judged on correctness, throughput across representative inputs, CPU, memory
    and complexity;
  - N1 is judged on its construction cost;
  - the full decoder on four physical cores and mixed inputs should drive promotion.
- **The reply's position:** option C deserves serious consideration for a separately
  pre-registered experiment on mibz2 itself. N1's next useful experiment would test
  construction cost. These are recommendations. Neither experiment is authorised by this
  report.

**The coordinator's action:** the option C worst-case analysis (report 59's added task),
queued behind the release-level result, is now unblocked and started as read-only analysis.

## The message, as received

> well no the point isnt about our wikidata-pq critical path, we are trying to accelerate mibz2 in its own right, take that hedge back pls

You're right. I withdraw that hedge. I let the wikidata-pq release context displace the
actual research objective, and that was a mistake.

The goal is to make mibz2 itself faster: higher decompression throughput on real multicore
hardware, without worsening CPU efficiency or materially increasing memory consumption.

Whether wikidata-pq happens to expose that improvement on its critical path is irrelevant
to whether the optimization is worthwhile for mibz2. Wikidata is a benchmark workload, not
the sole justification for the project.

That changes my recommendation:

* P74: A projected 4–6% improvement in full-decoder execution time is independently worth
  investigating. We should judge Option C on correctness, throughput across representative
  inputs, CPU cost, memory and implementation complexity—not on release ETA.
* N1: Its roughly 70–85% reduction in navigation lookups is worth preserving, but
  construction economics remain the decisive obstacle. A tightly bounded builder
  experiment may be justified on mibz2's own performance merits.
* Benchmarking: Four physical cores, mixed input distributions and complete decoder
  measurements should drive promotion decisions. The wikidata-pq pipeline is useful
  context, not an acceptance gate.

I would still insist on rigorous negative controls. A specialization that improves
Wikidata while badly degrading random or records data is not automatically a good
general-purpose decoder optimization.

But we should not require an improvement to move an unrelated pipeline's wall clock before
considering it valuable.

So my revised research position is that P74 Option C deserves serious consideration for a
separately preregistered experiment on mibz2 itself, regardless of the pending
release-level analysis. N1 remains a higher-risk candidate whose next useful experiment
would test construction cost.

Thank you for correcting the objective. I'll keep mibz2's own performance as the primary
criterion from here.
