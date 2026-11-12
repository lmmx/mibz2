# External research report 164: the guardian on the BLAKE3 MLP study: known versus discovered addresses; compact30's access path as the target; assembly to be justified, not presumed; provenance and sourcing corrections (verbatim)

A research input, not a finding of this repository.

- **Relayed:** by the maintainer, 2026-11-02.
- **Source:** the guardian.
- **In reply to:** report 162 (the BLAKE3 session's answer), report 163 (the walkthrough,
  still pending), and the coordinator's first read of what might carry over.

**Evidence cutoff:** 2 November 2026. The guardian checked the repository's provenance and
the pinned AVX2 example. It did not run anything.

It is kept exactly as received (rendered text, cards flattened, tracking parameters
included).

## The message, as received

`````text
**This is a worthwhile direction to investigate—especially as a source of concrete implementation techniques for the new compact walker, rather than another reason to rewrite the existing one.**

I checked the repository’s provenance and the pinned AVX2 example. **It is a fork of `BLAKE3-team/BLAKE3`, with `servil` as its default branch.** The eight-stream loads and lookahead prefetches are present as described. One attribution detail: that AVX2 mechanism also appears in upstream BLAKE3, so the walkthrough should distinguish established upstream techniques from this fork’s additions.   

The opportunity is real, but there is one fundamental difference that should organize the entire comparison.

## 1. BLAKE3 knows its future addresses; mibz2 must discover them

In the inspected BLAKE3 kernel, each input stream is accessed using a base pointer and a progressing offset. Future addresses can be calculated before the current block’s hashing finishes. The quoted prefetch is indeed **192 bytes ahead of the current block’s start**: the current load uses `rdx - 64`, while the prefetch uses `rdx + 128`. 

The inverse-BWT walk is different: each loaded table entry supplies the successor needed to address the next entry. Its segmented implementation exposes several independent chains to overlap those dependencies. 

| | BLAKE3’s inspected bulk path | mibz2’s navigation |
|---|---|---|
| Next address | Calculated from a known stream base and offset | Derived from the current table entry |
| Looking several steps ahead | Does not require reading the intervening input values | Requires discovering intervening successors, or changing the representation |
| Useful transferable question | How are known loads, computation and prefetches scheduled? | How early can each successor be discovered and its request issued? |

**That does not make the comparison unhelpful. It tells us what to learn from it.**

Copying BLAKE3’s prefetch distance would not solve dependent navigation. Studying how it makes addresses available early, organizes independent work and manages state could reveal something useful.

The question I would want the walkthrough to answer is:

> **Which instructions must finish before the next useful memory request can issue—and how does this implementation keep unrelated work off that path?**

That is much more valuable than simply observing that both programs use prefetch.

## 2. The most promising connection is to compact30’s access path

For the proposed 30-entry representation, the offset address and header address are both computable from the current encoded row. Neither address requires the result of the other load. The reference implementation reflects that layout. 

That creates a focused question for the assembly study:

**Can the optimized implementation expose those independent loads early, combine their results cheaply, and issue the successor prefetch without making it wait behind unrelated output bookkeeping?**

This is a hypothesis to investigate, not a claim that the compiler currently gets it wrong.

I would look for three kinds of transferable insight:

| Technique to study | What would make it useful to mibz2 |
|---|---|
| **Load/computation interleaving** | A concrete schedule that makes independent header, field or lane accesses available earlier |
| **State placement and register use** | Removing redundant loads, stores or conversions without excessive register pressure |
| **Generated or hand-scheduled assembly** | A precise execution-path improvement that ordinary Rust compilation does not already produce |

The critical test is whether the technique removes a cost **that actually exists in our candidate**. “Use more independent streams” is not enough; the design already exposes interleaved navigation chains.

Similarly, grouping several lanes’ loads before consuming their results might be worth examining—but it could also introduce temporary-state traffic or register pressure. The walkthrough should account for the cost, not merely recommend batching.

## 3. Assembly is worth studying, but not treating as inherently superior

One useful detail in the AVX2 source is that it explicitly stages transposed message data in stack slots before the compression arithmetic consumes it. It is not a register-only kernel. 

That is a good lesson for our investigation:

> **The goal is not to eliminate every memory instruction. It is to arrange necessary work so that the processor can execute it efficiently.**

Some temporary storage may support a better overall schedule. Conversely, a shorter assembly listing can still have a worse dependency chain. We should ask what each load, store and shuffle accomplishes, rather than classifying them all as waste.

For mibz2, hand-written assembly would need to earn its maintenance cost through a specific advantage. I would first want a proposed instruction sequence, a comparison with compiler output, and an explanation of why the difference matters—not a general recommendation to “rewrite it in assembly.”

## 4. Keep the hardware comparisons disciplined

For your **i9-10900K**, the AVX2 implementation is the directly relevant starting point. Intel lists AVX2 among that processor’s instruction-set extensions, not AVX-512; the quoted Arm SME2 code is an architectural analogy rather than something we can run on this target. [Intel](https://www.intel.com/content/www/us/en/products/sku/199332/intel-core-i910900k-processor-20m-cache-up-to-5-30-ghz/specifications.html)

The wider kernels may still teach useful scheduling or code-generation techniques. But their register resources, instructions and chosen concurrency should not be transferred unchanged to the i9.

There is also a small sourcing correction in the pasted report: its closing PMU reference is for **Granite Rapids**. That is not the event specification to use for your Comet Lake processor. Linux’s event mapping places model A5 with the Skylake-family event definitions. [code.googlesource.com](https://code.googlesource.com/linux/torvalds/linux/%2B/4edcdefd4083ae04b1a5656f4be6cd83ae919ef4/tools/perf/pmu-events/arch/x86/mapfile.csv?utm_source=chatgpt.com)

The report’s broader qualification is right: source code can demonstrate that independent accesses are exposed, but **eight streams do not establish eight outstanding DRAM misses or a measured MLP benefit**.

## 5. What would make the forthcoming walkthrough genuinely actionable?

I would want its conclusion to contain a small **transfer assessment**, not just a tour of impressive code:

> **BLAKE3 technique → why it helps there → what corresponding opportunity exists in mibz2 → what prevents direct transfer → smallest check that would establish whether it helps us.**

In particular, ask it to distinguish a technique that mibz2 already uses from one that could improve compact30’s new access primitive. It should be perfectly acceptable for the analysis to conclude that a mechanism is instructive but not transferable.

For the records, Claude should preserve the transcript as received, alongside the pinned BLAKE3 commit and the distinction between upstream and fork-specific code. A later verified interpretation can sit beside it; there is no need to rewrite the original account.

**My view: this is a good complementary use of the second research chat.** Astra is designing the navigation architecture; the BLAKE3 study can supply implementation ideas and concrete examples of scheduling independent work. The valuable outcome would be a specific improvement to the candidate’s access path—not another general claim that prefetching or assembly is fast.

Nothing in this calls for another local run or changes the completed walker-ladder decision. **It gives us another place to look for a better implementation, without assuming that a hashing kernel and a dependent traversal have the same bottleneck.**
`````
