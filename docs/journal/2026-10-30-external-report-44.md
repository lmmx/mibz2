# External research report 44: the maintainer, the guardian and the researcher on N1's construction economics (verbatim)

The maintainer's message, relayed after the N1 construction-economics note
(`2026-10-30-n1-construction-economics.md`). It holds three texts: the guardian's reading
with its suggested instruction ("What I'd send Claude"), then, after "the researcher
said", the researcher's message, and a further assessment.

**Symbols lost in the relay.** Several plan names arrived blank or altered, and are kept
as received below:
- "The partially upgraded representations, and ," and "My current preference is to keep
  as the first prospective construction experiment" lost P^{2|4} and P^{4|8};
- "P^6" and "P^12" are a rendering or transmission artefact, not a mathematical
  identity: they are not sixth- or twelfth-power Ψ tables. Their figures (0.277 elements
  and about 0.296 lookups per byte; 0.150 lookups per byte) identify them as the
  partial-upgrade plans P^{2|4} (two-byte records, eligible unsplit intervals upgraded to
  four bytes) and P^{4|8} (four-byte records, eligible unsplit intervals upgraded to eight
  bytes). Report 46 asks for this to be recorded so;
- "If a level- interval has displacement ," reads as level-a, d_I, J and a-byte words.

**What it asks, all read-only analysis:**
1. **Correctness of the partial upgrades** (P^{2|4} and P^{4|8}): exhaustive small cases
   and randomised adversarial cases. Check both the records' Ψ composition and the exact
   complete output of the variable-length segmented walker, including hop caps, segment
   boundaries, emission order, split parents and singleton intervals. Record the
   coverage's limits.
2. **The four-copy case:**
   - what establishes the 5.0 → 10.4 c/B walk regression, and which mechanism the
     evidence supports (cache capacity, bandwidth, address aliasing or another);
   - how much evidence there is for N1's assumed 1.3× factor;
   - a sensitivity surface instead of a fixed factor, over the builder's cost per element
     c_e, N1's walk amplification and the shipped walk's amplification at four copies;
   - net build plus walk in CPU cycles per decoded byte and in wall time, for four
     concurrent workers.
3. **Reconcile the construction-traffic models** (22.5 against 17.0–17.5 B/B) access by
   access, and make the peak-memory comparison symmetric: L and temporaries on both sides.
4. **Revise the prospective Gate B criteria:**
   - the median per-element cost apart from the aggregate, corpus-weighted construction
     cost (total cycles over total bytes);
   - per-block distributions;
   - a builder pass does not establish traversal performance; it permits testing it.

Throughout:
- measurements, mathematical deductions and estimates kept apart;
- the assumptions whose failure would reverse N1's projected advantage named;
- the EPYC 7763 as the continuity reference, Zen 3, 4 and 5 as sensitivity points, none
  standing for the four-worker laptop target.

**What it does not authorise:** Gate B, Gate C, a Rust kernel, a benchmark, CI, the i9 or
production integration. Report the findings and stop for review. The Gate B decision is
the maintainer's, after these findings.

## The message, as received

The researcher and I are aligned on N1's next steps. I'd adopt its three-part research
programme without adding another round of debate.

I particularly agree with the shift in objective:

N1 should optimize the total cost of constructing and traversing Ψ, not maximize the
number of bytes emitted per lookup.

That's an important change. A full eight-byte table can win the traversal benchmark while
losing the decoder benchmark because building it costs too much.

The partially upgraded representations, and , give us a plausible way around that
trade-off.

One refinement I'd make

The four-copy sensitivity analysis should use CPU-seconds per decoded byte as well as wall
time.

At four concurrent workers, a compact table could substantially reduce contention and
improve throughput without proportionally reducing total CPU consumption. Both matter for
the project's release objectives.

I would want the analysis to expose a surface of possible outcomes rather than a single
assumed 1.3× amplification:

* Builder cost per record element, .
* N1 walk amplification under four simultaneous copies.
* Shipped walk amplification under the same conditions.
* Net build-plus-walk cost, both in aggregate CPU cycles and elapsed time.

That will tell us whether N1 needs an exceptionally fast builder or can tolerate a
moderately expensive one because its traversal scales better.

A second refinement: the correctness checker should test not only the algebra of upgraded
intervals, but the byte order and exact segmented output of the mixed-length walk. A
mathematically correct Ψ composition can still be implemented with incorrect hop caps or
emission order.

And I agree that the 22.5 versus 17–17.5 B/B construction-traffic discrepancy and the
omitted L buffer must be reconciled before making memory-efficiency claims.

What I'd send Claude

I adopt the researcher's proposed N1 research sequence.

Please continue with the three bounded, read-only investigations:

1. Validate P^{2|4} and P^{4|8} partial-upgrade correctness with exhaustive small cases
   and randomized adversarial cases. Verify both record-level Ψ composition and exact
   complete output from the variable-length segmented walker, including hop caps and
   boundaries.
2. Build a four-copy sensitivity analysis using measured shipped stage figures. Vary
   builder cost, N1 walk amplification, and shipped walk contention rather than fixing
   the N1 amplification at 1.3×. Report both total CPU cost per decoded byte and
   wall-time implications for four concurrent workers.
3. Reconcile the two construction-traffic estimates (22.5 versus 17–17.5 B/B) access by
   access, and make peak-memory comparisons symmetric, including L and temporary
   allocations on both sides.

Clearly separate measurements, mathematical deductions, and estimates. Identify
assumptions whose failure would reverse N1's projected advantage.

This is analysis only. Do not start Gate B, build a Rust kernel, benchmark, run CI or use
the i9. Report the findings and stop for review.

I would then make the Gate B decision based on those findings.

My current preference is to keep as the first prospective construction experiment,
because it is the cheapest meaningful test of the new builder mechanism. remains the more
ambitious candidate if the construction economics prove favorable.

The potentially transformative result would be demonstrating that a compact
variable-depth Ψ representation reduces four-worker construction-plus-navigation CPU cost,
not merely that it performs fewer lookups.

That's the question worth spending the next research effort on.

the researcher said

This is a significant development for N1. The construction-economics review has changed
the question from "Can Ψ⁸ make inversion faster?" to "What's the cheapest depth of Ψ
composition that actually pays for its construction?"

And I think the report makes a compelling case that full Ψ⁸ is probably not the right
initial target.

I would pursue the remaining research in this order, without authorizing Gate B yet.

1. First: close the correctness gap in partial upgrades

The strongest candidates are now and , which upgrade records only where doing so
introduces no new interval splits.

The underlying reasoning is sound. If a level- interval has displacement , and its image
lies entirely inside another level- interval , then

\Psi^{2a}(x)=x+d_I+d_J

throughout .

Its emitted word is the concatenation of the two corresponding constant -byte words.

That gives us a short correctness proof for each upgraded record.

However, the composition of individually correct records is not automatically a correct
variable-hop walker. We must also verify that the mixed-length traversal respects segment
ends, emits bytes in the right order, and neither skips nor duplicates an output position.

The offline checker should compare the complete reconstructed output, not merely each
record's formula.

I'd make this the immediate next research action. It's cheap, read-only, and closes the
one explicitly unverified step in the best N1 designs.

2. Second: analyze the four-copy case independently

This is where I would put the most intellectual effort.

The report's most consequential numbers are:

N1 builder-cost thresholds

Cycles per record element, modeled

CPU	One-core pass	Four-copy pass	One-core kill
EPYC 7763	10.2	31	22.4
Zen 4	9.9	31	21.8
Zen 5	6.1	19	14.2

Pass thresholds are model-derived, not observations. The four-copy values rely on assumed
contention factors.

These figures suggest something quite important: N1 might be mediocre at one core but
valuable under four concurrent workers, because its compact navigation table may suffer
less from cache contention.

But the four-copy case currently assumes the N1 walk slows by only 1.3× while the shipped
T-vector walk has historically slowed much more.

That's exactly the assumption I would try to falsify next.

A smaller representation does not guarantee a proportionally smaller working set at every
cache level, or a smaller dependent-load latency penalty. The relevant variables include
lookup locality, active cache lines, memory-level parallelism and the total work required
per decoded byte.

I would construct a sensitivity analysis showing the break-even N1 walk amplification as
a function of the measured builder cost and shipped four-copy stage times. That can be
done without implementing the walker.

3. Third: scrutinize the construction-cost model before Gate B

There are two warnings in the report that deserve attention.

The first is the traffic discrepancy: 22.5 B/B in the new reconstruction versus
17.0–17.5 B/B in the earlier portfolio. That's a material difference. The accounting
should be reconciled access by access before using either figure as a reliable estimate.

The second is peak memory: excluding the 0.9 MB last-column buffer from N1 while
retaining it in the shipped baseline produces an unfair comparison. Both paths need a
symmetric live-allocation accounting.

Neither discrepancy necessarily kills N1, but both affect the claim that it satisfies the
project's CPU and RSS constraints.

My recommended research sequence

P74 Gate 1: closed

Negative under the frozen criteria; preserve its measured fast-path findings. No further
P74 tuning is implicitly authorized.

N1 partial-upgrade correctness

Offline exhaustive/randomized checker, including variable-length segmented traversal.

N1 four-copy sensitivity

Derive the contention and lookup-cost conditions under which it beats shipped at four
workers.

N1 model reconciliation

Resolve the construction-traffic difference and peak-memory accounting on identical
boundaries.

Gate B decision

Only after the above analysis should the coordinator consider authorizing a measured
builder experiment.

One additional research judgment: I would not choose Zen 5 as the sole reference machine.
Its faster shipped walk makes N1's one-core economics look less favorable, but the
project's four-worker laptop objective cannot be inferred from one cloud
microarchitecture. Zen 3, Zen 4 and Zen 5 provide useful sensitivity points; none
substitutes for the ultimate target.

Suggested response to Claude

The N1 construction-economics analysis materially changes our research priorities. I
agree that full P⁸ is unlikely to be the best first candidate; the partial-upgrade plans
P^{2|4} and P^{4|8} deserve priority.

Please continue read-only research in three parts:

1. Brute-force the partial-upgrade records and mixed-length segmented walker against the
   exact Ψ reference, including adversarial small inputs and segment boundaries.
2. Derive a four-copy sensitivity model showing the maximum affordable N1 walk
   amplification for different builder costs, rather than assuming the 1.3×
   amplification.
3. Reconcile the 22.5 versus 17.0–17.5 B/B construction-traffic models and account
   symmetrically for L and other live allocations in the peak-RSS comparison.

Keep measured facts separate from models, and identify any assumption whose failure
reverses the N1 verdict.

This is research only. No Gate B, new production implementation, CI or i9 experiment is
authorized by this message.

My research conclusion: N1 is still alive, but its most promising form has changed. We
should stop treating eight-byte jumps as the goal in themselves. The goal is now the best
construction-plus-navigation trade-off at four concurrent workers—and the partially
upgraded tables may be the better answer.

My assessment: N1 has become more promising—but its construction model needs one more
round of scrutiny before Gate B

This is a substantial research result. The analysis has moved beyond asking whether
multi-byte Ψ traversal could be faster. It identifies which table structures might be
worth constructing, what construction cost they can tolerate, and which costs would make
N1 uneconomic.

I've read the full construction-economics journal, and my recommendation is:

Do the inexpensive correctness proof and investigate four-copy economics next. Then
decide whether to authorize Gate B.

I would not start building the P^6 kernel just yet.

The most important discoveries

P^6 — best economical starting point

Promising model

0.277 construction elements per output byte at the median; approximately 0.296 lookups
per output byte.

P^12 — stronger traversal

0.150 lookups/byte

More construction work, but substantially fewer dependent lookups.

Full P⁸ — poor construction economics

78% unused

In the sampled walks, most constructed records are never executed.

The strongest idea isn't necessarily building an eight-byte lookup table. It's upgrading
unsplit intervals without constructing their deeper descendants.

That gives N1 an adaptive-depth representation with a more attractive balance between
construction and navigation.

The record-count evidence is relatively solid. The cost predictions are much less
certain.

Three concerns I would raise before Gate B

1. The builder's cost per element isn't the only deciding unknown.

The report calls the decisive quantity, but the benefit also depends on the cost per
lookup, .

The construction model uses an inferred 16–24 instructions per N1 lookup and assumes it
can sustain the shipped walk's architecture-dependent IPC. That is plausible, but
unmeasured.

A builder could meet its 10-cycle target and still produce a disappointing decoder if N1
traversal is substantially slower than predicted.

I would therefore treat a Gate B pass as permission to test traversal, not evidence that
N1 will win.

2. The reported median construction costs could conceal expensive blocks.

At the 90th percentile, the number of construction elements per output byte roughly
doubles. The same implementation could be excellent on typical Wikidata blocks and
uneconomic on high-run-count blocks.

A final Gate B should report more than the median : it should include the total
construction cycles divided by total decoded bytes across the corpus, as well as
per-block distributions. Otherwise, a seemingly successful median result could hide a
disappointing aggregate.

3. The four-copy model is too important to leave largely assumption-driven.

The report cites a previous four-copy increase in walk cost from 5.0 to 10.4 cycles/byte.

That's potentially enormous. It could mean N1's smaller working set is valuable even if
its one-thread build cost is merely acceptable.

But the model assumes N1's four-copy walk incurs only a 1.3× penalty, rather than
measuring it. That's the central uncertainty of N1's most relevant deployment scenario.

I would make a read-only analysis of that assumption a priority.

The four decisions

Question	My recommendation
Authorize Gate B now?	Not yet. Worth doing after the low-cost checks below
Investigate the four-copy regression?	Yes. High priority, read-only
Brute-force-check the unsplit upgrade?	Yes. Immediate priority
Optimize against Zen 5 or EPYC 7763?	Use EPYC 7763 as the continuity reference; treat Zen 5 as an important robustness constraint

The last decision matters. Zen 5 has a much cheaper existing inverse-BWT walk, so N1 has
less room to improve. But optimizing exclusively for Zen 5 would risk discarding an
approach that performs especially well under the four-way contention relevant to the
target release.

I would not adopt an architecture-specific pass/kill threshold without recording which
platforms the intended release actually needs to serve.

What I would ask Claude to do next

I've reviewed the N1 construction-economics analysis. I want two bounded, non-performance
research checks before deciding whether to authorize Gate B.

First, validate the mathematical correctness of the P^{2|4} and P^{4|8} unsplit-parent
upgrades using small exhaustive or brute-force Python models. Include interval
boundaries, split parents, singleton intervals, segment caps and exact output
reconstruction. Record the limitations of the coverage. Do not implement a Rust builder or
benchmark anything.

Second, investigate the historical four-copy walk regression as read-only research.
Determine which measurements establish the 5.0 → 10.4 c/B increase, whether cache
capacity, bandwidth, address aliasing or another mechanism is supported, and how much
evidence exists for N1's assumed 1.3× contention factor. Distinguish measured facts from
extrapolation.

Please also revise the prospective Gate B criteria to distinguish median per-element cost
from aggregate corpus-weighted construction cost, and make clear that a builder-cost pass
does not establish N1 traversal performance.

Do not launch Gate B, Gate C, any i9 experiment or any production integration. Report the
findings and stop for review.

One larger strategic observation

The report's most exciting number isn't the hypothetical 0.4× build-plus-walk cost. That
depends on assumptions we haven't validated.

It's the evidence that an aggressively compact, variable-depth Ψ representation can avoid
constructing much of the unused full-depth table.

That is a more concrete and potentially valuable direction than full P⁸. It's also a
meaningfully different construction opportunity from the earlier approaches that failed.

My inclination is to pursue N1, with P^6 as the first prospective builder test and P^12
as the important higher-performance candidate.

But we're not yet at the point of declaring a breakthrough. The cost model has identified
a plausible route; the next measurements still have to establish whether it exists in
actual machine code—and, eventually, whether it wins in the complete four-worker decoder.
