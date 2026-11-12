# External research report 134: the guardian on the researcher's last contribution: a bounded minimal-walker diagnostic beside FOR-14 (verbatim)

A research input, not a finding of this repository.

- **Relayed:** by the maintainer, 2026-11-01, together with report 133 (which it follows).
- **Source:** the guardian. The maintainer's question is kept in place.
- **In reply to:** report 132.

**Evidence cutoff:** 1 November 2026. A plan amendment, not a result: the minimal-walker
comparison is to be planned, not built, and no timed i9 capture is authorized.

It is kept exactly as received (rendered text).

## The message, as received

`````text
i asked the guardian chatbot:

> Should we incorporate this into our plan? The researcher chat hit its limit so this is its last contribution

it said:

Yes. I'd incorporate the researcher's final contribution, but as a parallel diagnostic for the architectural decision—not as a replacement for Astra's FOR-14 proposal.

In fact, I think it identifies a missing piece in our programme: we have measured how fast different walkers run, but we haven't established a reliable performance ceiling for the existing 32-lane walker on the actual i9 hardware.

That distinction could save us considerable wasted effort.

Astra asks whether a fundamentally smaller navigation representation can repay its additional instructions. The researcher asks whether the current representation is already near the processor's practical limit, or whether software scheduling and bookkeeping leave substantial performance untapped.

Those questions complement each other. And the second can help interpret a negative result from the first.

## What the researcher adds

The proposed experiment is an idealized walker ladder: begin with a minimal pointer-chasing loop over a real Wikidata T-vector, then reintroduce the operations required by the actual decoder.

Locate where the walker loses performance

Minimal dependent traversal

Hardware-facing reference

Real BWT links, minimal per-step work, verifiable checksum

Lane bookkeeping and segment boundaries

Determine cost of maintaining the real traversal state

Prefetch scheduling

Separate demand-miss behaviour from prefetch-mediated overlap

Output stores and segment semantics

Find out where useful hardware throughput is lost

Production walker

Compare with actual decoder-stage cost and full-pipeline performance

The researcher's central caution is sound: the historical single-chain result of 38.55 cycles per byte and the 32-lane result of 4.49 cycles per byte are not a clean same-conditions comparison. Dividing them to infer memory-level parallelism would be invalid.

Similarly, a low demand-load outstanding-miss count can be misleading in a software-prefetched walker. We would need to distinguish demand traffic from prefetch traffic and confirm exactly what each available counter measures.

The historical AMAC and pointer-chasing literature is useful here as experimental methodology, not as proof that the i9 shares a particular hardware concurrency limit.

## How I'd integrate it without expanding the research indefinitely

I'd preserve one main architectural bet and one bounded diagnostic.

| Work                             | Purpose                                                                                             | Priority              |
| -------------------------------- | --------------------------------------------------------------------------------------------------- | --------------------- |
| FOR-14/raw record feasibility    | Test whether a substantially smaller navigation representation can be efficient                     | Primary               |
| Minimal versus production walker | Quantify how much performance remains recoverable through instruction scheduling and memory overlap | Supporting diagnostic |
| P84 direct builder               | Only if compact navigation leaves enough construction budget                                        | Conditional           |
| Full neighbour harness           | Defer                                                                                               | Not authorized        |
| P87 production validation        | Retain as small engineering work                                                                    | Parked                |

I would make two changes to the researcher's original proposal.

First, don't immediately build the full 10-lane-count × multiple-kernel ladder. Start with 1, 8, 16 and 32 lanes, then add intermediate points only if a meaningful transition appears. Otherwise we're creating another sizeable investigation before establishing whether the reference is informative.

Second, measure both single-core and four-core behaviour. A minimal walker on an otherwise quiet processor might be extraordinarily fast yet tell us little about a four-worker decoder under shared-cache contention. A useful idealized reference should ultimately reproduce the relevant concurrency environment, or explicitly acknowledge that it cannot.

The minimal walker must also do enough observable work to prevent the compiler from optimizing away traversal, preserve a comparable number of actual link steps, and distinguish pure pointer-chasing performance from the cost of constructing the T-vector.

### The decision tree I want

Is the idealized walker materially faster than the shipped walker under comparable conditions?

No

Little demonstrated headroom from removing bookkeeping or changing lane scheduling alone.

Favour fewer costly memory accesses, better representation, or changes outside traversal.

Yes

There may be recoverable software overhead or inadequate overlap.

Add decoder operations back progressively and identify where the gap first appears.

Neither result automatically proves that FOR-14 will succeed or fail. A smaller representation changes both access locality and instruction work.

But this diagnostic can tell us whether the current walker is leaving obvious execution efficiency on the table, which would affect our assessment of how much extra work a compact representation can afford.

## What I'd tell Claude

Please incorporate the independent researcher's final architectural contribution into the research plan as a narrowly bounded supporting diagnostic.

The main priority remains Astra's FOR-14/raw feasibility investigation. Do not replace it with another broad walker-optimization programme.

The researcher identifies an important missing reference: we have not established the practical hardware-facing performance ceiling for the current 32-lane walker on the i9-10900K.

Its historical 38.55 cycles/byte single-chain and 4.49 cycles/byte 32-lane figures were not obtained under sufficiently comparable conditions to infer achieved memory-level parallelism.

Please add a short plan for a **minimal-real-T-vector walker comparison**:

1. Use a genuine extracted Wikidata T-vector, not merely a random permutation. Preserve valid link dependencies and a verifiable count of traversed rows.
2. Compare a minimal walker with progressively more production semantics, including lane state, prefetch, segment boundaries and output stores.
3. Initially examine a small number of lane counts, rather than automatically running the full proposed sweep.
4. Audit the relevant i9 performance counters before selecting them. Distinguish demand misses, software-prefetch traffic, achieved overlap, memory stalls and execution-resource limits. Do not infer useful outstanding requests from speedup ratios.
5. Include a way to test whether conclusions from an isolated core remain valid under representative four-core competition.
6. State an explicit stopping rule: if the minimal walker has no material advantage under comparable conditions, do not commission another walker scheduling rewrite.
7. Keep this as a bounded diagnostic with a direct architectural consequence, not a new open-ended performance investigation.

For the immediate next work, continue with the FOR-14 current-corpus census, physical record specification and untimed correctness checks as authorized.

Do not implement the full walker ladder or perform any timed i9 capture yet. Return its proposed minimum scope and decision rules alongside the FOR-14 feasibility preparation.

Preserve the researcher's final contribution as an external research note with its literature references and caveats intact.

## My final prioritization

I wouldn't send this back for another long round of research discussion. The independent researcher has left us a useful methodological insight, and Astra has already selected a promising architecture.

We should now act on those two contributions without letting either become an unlimited investigation.

My recommendation is:

1. Proceed with Claude's FOR-14 census, exact record specification and VM correctness work.
2. Add the researcher's minimal-walker ceiling test to the experimental plan, but do not build it yet.
3. Review the compact format and the small hardware-ceiling test together before deciding how to allocate the next i9 capture.

One important qualification: a fast idealized walker does not automatically imply a full-decoder opportunity, just as a smaller navigation table doesn't automatically imply a faster decoder. The whole-decoder cost model remains the final arbiter.

What I particularly like about the researcher's last contribution is that it gives us a way to learn from failure. If FOR-14 loses, we should be able to say whether the problem was packing instructions, additional dependency cost, exception handling, construction expense, or simply insufficient benefit from reduced memory footprint.

That is substantially better than another result labelled “compact navigation: negative.”

We have a concrete architectural candidate, a cheap falsification path, and now a complementary method for understanding the processor's limitations. That's enough planning. The next valuable evidence should come from carefully bounded implementation and measurement.

---
`````
