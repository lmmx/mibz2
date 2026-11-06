# External research report 171: the guardian on the compact30 prototype: a significant milestone; the researcher's independent review next; a possible frozen i9 traversal comparison afterwards (verbatim)

A research input, not a finding of this repository.

- **Relayed:** by the maintainer, 2026-11-02, with report 170. The maintainer then sent its
  handoff to the researcher, whose reply is pending.
- **Source:** the guardian.
- **In reply to:** the compact30 prototype, PR #134 at `85b87fb`, and its journal.

**Evidence cutoff:** 2 November 2026. A reading of PR #134 and its journal.

**This authorises nothing new.** The code stays frozen at `85b87fb` pending the
researcher's review. A frozen i9 traversal experiment is mentioned only as a possible next
step after that review.

It is kept exactly as received (rendered text, cards flattened, tracking parameters
included). The relayed text included a chart, flattened here to its labels.

## The message, as received

`````text
This is a significant milestone. Astra predicted a 19–21-instruction ordinary path, and Claude has produced a working, production-shaped segmented walker with an actual 19-instruction path.

I checked [PR #134](https://github.com/lmmx/mibz2/pull/134?utm_source=chatgpt.com), currently at commit `85b87fb4f21bf52fe9f327dcb2a371ddeddc983d`, and its journal. The reported assembly and correctness results are documented there.

Ordinary-step comparison — compiled instructions

010203040Original FOR-14Packed kPacked jCompact30Production

Lower is fewer static instructions, not necessarily faster.

Compact30 has reduced the ordinary-step instruction gap against production from 22 instructions to just three, while retaining a navigation table around 46% smaller in logical bytes.

That is a much stronger case than the original FOR-14 prototype had.

### What particularly encourages me

The machine code confirms the intended structural simplifications:

- Both offset and header addresses derive directly from the current encoded row.
- There is no packed-bit extraction or F-directory lookup.
- The carried byte has disappeared from lane state.
- There are no additional spills in the ordinary loop.
- The successor prefetch comes before the output-base load and byte store.

That last point is especially satisfying after the BLAKE3 investigation: the compiler has already kept output bookkeeping off the successor-prefetch path.

The correctness evidence is also substantial: 244 release configurations across the established corpora and adversarial inputs, including forced-RAW cases. The candidate's measured transition counts now give us an actual dynamic exceptional-path frequency for this implementation, rather than an assumption derived from the storage census.

### What is still unknown

The 19-instruction result does not imply that compact30 will outperform production.

It still uses four data loads rather than three, with two table loads instead of one. Even when those table accesses hit the same cache line, they consume execution resources.

The RAW path also introduces a parked turn before resumption. Its observed frequency is low on Wikidata, but its cost is not yet measured.

And this is still an oracle-built table. Construction economics remain an independent hurdle.

We have proven that the intended architecture compiles and reconstructs correctly across the tested cases. We have not proven that it is fast.

## The next step: give the researcher the actual artifacts

The researcher has already agreed to review them. Send this:

Claude has finished the authorized compact30 prototype. Please now perform the independent review we agreed.

**PR #134:** [https://github.com/lmmx/mibz2/pull/134](https://github.com/lmmx/mibz2/pull/134)

**Pinned commit:** `85b87fb4f21bf52fe9f327dcb2a371ddeddc983d`

**Implementation:** `examples/c30walk.rs`

**Journal:** `docs/journal/2026-11-02-compact30-walker.md`

**Machine code and records:** `docs/journal/2026-11-02-compact30-walker/`

The refined walker compiles to 19 ordinary-step instructions, four data loads, two stores and one successor prefetch, using rustc 1.97.0 / LLVM 22.1.6, baseline x86-64.

Correctness passed across all 11 established corpora and the adversarial set, including forced-RAW cases, with exact output against production. The implementation reports 0.713% RAW arrivals per emitted byte on primary Wikidata and preserves the census's 2.1642 logical bytes per row. No timing has been performed.

**Please independently inspect the actual code and disassembly, not just Claude's summary.**

Focus on the three agreed questions:

1. **Correctness and transitions:** Audit current-row F emission, original-primary-row initialization, ordinary/RAW starts, parking and resumption, mid-pass replacement, one-byte segments, tail behaviour and marker restoration. Identify any serious correctness or memory-access obligation that the existing tests do not establish.
2. **Execution dependencies:** Reconstruct the actual ordinary-step dependency graph from the compiled instructions. Distinguish independent field/header addresses from dependent successor generation. Check whether output bookkeeping is off the successor-prefetch path, whether loads can overlap across lanes, and whether the branch structure, register use or slow-path transitions introduce a significant hidden cost. Use the BLAKE3 study only where it identifies a concrete transferable mechanism.
3. **Decision:** Does the prototype satisfy its compile-and-correctness objectives sufficiently to earn one separately registered performance comparison, or is there a specific defect or execution-cost obstacle that needs addressing first?

Please distinguish structural code-generation facts from performance predictions. Do not infer speed from the 19-instruction listing or assume a smaller table improves cache behaviour.

Avoid additional implementation proposals unless a concrete defect justifies one. Return a clear recommendation and the evidence supporting it. This is a source and machine-code review; no repository changes or benchmark runs are requested.

## My recommendation

Send that to the researcher now. Do not send another assignment to Astra yet.

If the independent review confirms the implementation and its dependency structure, I would be inclined to authorize one carefully frozen i9 traversal experiment, comparing compact30 directly with the shipped walker under one-core and four-core conditions.

That would be the first real performance test of this substantially improved design, with construction explicitly excluded and separately budgeted.

The measurement would need a predeclared stop rule. Improving substantially over the failed FOR-14 result would not be enough on its own: compact30 must demonstrate a credible path to outperforming the conventional representation after construction costs.

For now, I would leave Claude's code frozen at its current checkpoint and obtain the independent review.

We've progressed from a mathematical proposal to a corpus-validated format to a compiled, production-shaped 19-instruction walker. The next critical unknown is no longer whether it can be implemented efficiently in principle. It's whether that implementation is fast enough on the actual hardware.
`````
