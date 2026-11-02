# External research report 169: the guardian on routing the BLAKE3 ideas: Claude first, then the researcher on the machine code, Astra only for a concrete obstacle (verbatim)

A research input, not a finding of this repository.

- **Relayed:** by the maintainer, 2026-11-02, while the compact30 prototype was being
  finished.
- **Source:** the guardian.
- **In reply to:** the maintainer's question whether to give the researcher the BLAKE3
  ideas.

**Evidence cutoff:** 2 November 2026, before the prototype's machine code was returned
(PR #134).

**No new assignment.** The researcher's review of the compiled dependency graph is to come
after the disassembly. Astra only if that review shows a concrete obstacle or a plausible
high-value improvement.

It is kept exactly as received (rendered text, cards flattened). The maintainer's relay
said "i asked the guardian about maybe giving the researcher the ideas from blake3 and it
said yes".

## The message, as received

`````text
I wouldn't send Astra another assignment yet. The BLAKE3 study is valuable, but the most promising insights are already incorporated into Claude's approved compact30 prototype. Sending Astra another Max-effort review now risks having it theorize about questions the compiler is about to answer directly.

There is, however, one genuinely interesting technical question worth preserving.

## The potential opportunity: scheduling independent loads across lanes

BLAKE3's assembly does something more sophisticated than simply inserting prefetch instructions: it exposes independent work early, so the processor can make progress on multiple memory requests while computation continues.

For compact30, we have an analogous opportunity within each step.

Compact30's dependency structure

The two current-record addresses are independent. The next successor address still depends on their results.

The key question is:

Could issuing loads for several lanes before completing their successor updates expose more useful overlap than processing each lane to completion?

That's the most direct conceptual connection to BLAKE3's carefully arranged independent streams.

But it comes with serious trade-offs: more temporary values, register pressure, possible stack traffic, and different behaviour around RAW entries and segment termination. The shipped walker's scheduling investigation also already reached its stopping point; we should not reopen that investigation without a candidate-specific reason.

Crucially, Claude's next disassembly audit should already reveal whether this problem exists in compact30.

## What I'd do with the agents

| Agent            | Next action                                                                                                                                    |
| ---------------- | ---------------------------------------------------------------------------------------------------------------------------------------------- |
| Claude Code      | Finish the authorized compact30 prototype and its disassembly audit                                                                            |
| mibz2 researcher | Examine the resulting dependency graph and machine code independently, particularly whether field/header loads overlap across lanes            |
| Astra Max        | Revisit only if the compiled code reveals a genuine architectural obstacle or a plausible high-value improvement Claude cannot resolve locally |

The researcher would be particularly useful as a first filter. It could compare the actual dependency structure with BLAKE3's approach, distinguish compiler limitations from fundamental data dependencies, and identify whether a scheduling change would be worth Astra's attention.

I wouldn't assign even that analysis before Claude has produced the machine code.

## One more possibility, but lower priority

BLAKE3's ability to fetch future input addresses raises a broader theoretical idea: could mibz2 make future successors available earlier through auxiliary navigation metadata or multi-step links?

That is a legitimate architectural question. In fact, the project has already acknowledged the related LF²/super-alphabet approach.

But such lookahead would increase representation or construction costs and could undo compact30's storage advantage. It's not a new breakthrough discovered by reading BLAKE3, so I would not start another project around it now.

Prefetching known queued segment starts earlier is another transferable idea, but with only roughly 512 segment starts per 900,000-row block, its likely scope is much smaller than fixing a recurring per-row cost.

## My recommendation

Let Claude finish first. Then give the disassembly to the researcher before bringing Astra back.

If the compiler has preserved independent header/offset loads, sensible prefetch placement, and low state overhead, the BLAKE3 study has done its job. We proceed toward the separately authorized performance experiment.

If instead the compiler serializes avoidable loads, delays successor prefetch behind unnecessary work, or creates substantial intermediate-state traffic, that gives Astra a specific Max-effort problem:

> Can we restructure compact30's lane scheduling to expose independent memory requests earlier, with a demonstrably shorter critical path or lower execution cost, without sacrificing the validated representation and segmented semantics?

That would be an excellent use of Astra: a concrete optimization problem arising from real machine code, informed by BLAKE3, rather than another speculative review.

For now, no new assignment is necessary. We have enough research in motion, and the next piece of evidence should determine where additional intelligence is most useful.
`````
