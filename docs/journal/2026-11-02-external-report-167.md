# External research report 167: the guardian on the BLAKE3 walkthrough: three audits for the compact30 prototype (dependency, state, physical access), and what not to transfer (verbatim)

A research input, not a finding of this repository.

- **Relayed:** by the maintainer, 2026-11-02, during the compact30 prototype's work.
- **Source:** the guardian.
- **In reply to:** report 163, the BLAKE3 walkthrough.

**Evidence cutoff:** 2 November 2026. A reading of the supplied guide. The guardian says it
did not audit every linked BLAKE3 backend. Its "Pasted text" markers are its citations to
the guide, kept as relayed.

**What it adds to the approved prototype (report 166):** three audits. No new performance
experiment and no change of scope.

It is kept exactly as received (rendered text, cards flattened).

## The message, as received

`````text
**This is a useful engineering reference, and it complements Astra’s recommendation rather than competing with it.** I would archive it and use a small part of it to strengthen the compact30 prototype—not adopt its entire investigation brief as another programme of work.

Its strongest lesson is **to design the representation, address dependencies and execution schedule together**. The guide treats assembly as one component of that design, rather than assuming that assembly, SIMD or more prefetches automatically produce higher performance. Pasted text Pasted text

My assessment here is about what transfers from the supplied guide to mibz2; I have not independently audited every linked BLAKE3 backend.

## 1. The most relevant lesson: distinguish a computation dependency from an address dependency

The guide explains this particularly well.

BLAKE3 cannot finish the next block’s compression before the preceding block produces its chaining value. **But it can fetch the next block’s input beforehand, because that input’s address is already known.** That is what enables the explicit load/computation overlap described in the SME2 path. Pasted text

For our dependent traversal, the distinction is:

| Question | BLAKE3’s streaming blocks | Refined compact30 |
|---|---|---|
| Is the next iteration’s input address already known? | Yes: base plus block offset | No: the current record determines the successor |
| Are multiple accesses within the current step independently addressable? | Yes | **Yes: the offset and header addresses both derive from the current row** |
| Where should we look for overlap? | Future block loads against current compression | Header/offset loads, independent lanes, and work that need not delay the next prefetch |

The compact30 column is an application of Astra’s proposed design, not something established by the BLAKE3 report.

For Claude’s disassembly audit, the useful question becomes:

> **Does the machine code expose both current-record loads independently, then make the successor available without unnecessary intervening dependencies?**

That is more specific than “try to increase MLP.” It is also already within the scope of the prototype we agreed.

There is an important limit: **same cache line does not mean one load operation**. Compact30 still needs its field and header values. Published Skylake-family measurements identify the halfword `MOVZX` as a load operation in its own right; sharing the line does not remove that execution demand. [Uops](https://uops.info/html-instr/MOVZX_R32_M16.html)

So the expected advantage is cheaper address formation and extraction—not a magically free second access.

## 2. Retaining the right representation can matter more than scheduling harder

The guide describes how BLAKE3 keeps its state in the transposed representation across blocks instead of repeatedly converting it back. Its assembly generator also avoids some register-copy instructions by changing the generator’s bookkeeping rather than physically moving values. Pasted text Pasted text

**The transferable principle is to ask whether a conversion or state update is necessary at all.**

That is closely aligned with what has just happened in compact30:

- Encoded coordinates remove repeated record-index calculations.
- Direct halfwords remove packed-bit extraction.
- Astra’s current-row F emission removes the need for carried-byte state.

Those changes are our own architectural developments, not techniques demonstrated for BWT by the BLAKE3 guide. But the guide reinforces the right way to evaluate them: trace the information required by each step, then remove unnecessary transformations before attempting more elaborate instruction scheduling.

This is why I would proceed with the refined compact30 design rather than divert Claude into a generic software-pipelining rewrite.

## 3. The most valuable additional safeguard is about what the machine actually reads

The guide’s memory-safety section is particularly relevant now.

It reports a historical case where a kernel produced correct hashes but performed full-width loads beyond the valid input extent. Guard-page tests exposed the invalid access. The lesson is explicit: **correct output is not sufficient evidence that aggressive memory accesses are valid.** Pasted text

For compact30, I would translate that into a few precise checks:

| Prototype detail | What to establish |
|---|---|
| Record layout | Actual size, alignment and header offset—not just the intended arithmetic |
| Ordinary accesses | The emitted field and header loads stay within valid initialized storage |
| Eager header loading | Reading a header before the exceptional branch is valid for ordinary, RAW and parking records |
| Parking | The parking header’s dummy RAW index is never dereferenced |
| Boundary cases | Final records, partial records, final RAW entries and segment-start transitions remain valid |

These are **applications to Astra’s design**, not new findings of defects in code that has not yet been written.

Rust’s explicit representation and alignment facilities support checking the physical layout. Array stride also follows the element’s size, including padding, so the prototype should verify both size and alignment rather than assuming one from the other. [Rust Documentation](https://doc.rust-lang.org/reference/type-layout.html)

This need not become a new verification project. It belongs in the existing correctness work: access-bound assertions, adversarial boundary cases, and available memory-safety or guard-page tests where practical.

## 4. Do not mistake intentional state storage for accidental spilling

Another useful feature of the guide is that it does not portray high-performance assembly as register-only code.

The inspected AVX2 kernel uses a 512-byte stack region for transposed message vectors. The wider AVX-512 kernel makes a different register/storage trade-off, using its vector registers for two computation states and keeping current messages in stack storage. Pasted text Pasted text

For our audit, this suggests a better distinction:

**Necessary explicit lane state** is different from **additional traffic introduced by poor register allocation or an awkward expression**.

We should not reject compact30 merely because the lane array or output-base array lives in memory. The agreed target already includes those accesses. What matters is whether the new implementation adds redundant state loads, stores, conversions or spills beyond that design.

Likewise, we should not require hand-written assembly merely because BLAKE3 uses it. The guide itself recommends establishing whether ordinary code, intrinsics or a better interface exposes the opportunity first. Pasted text

**Compile the Rust prototype, inspect what it produces, and identify a specific remaining deficiency before considering a lower-level implementation.**

## 5. What I would *not* transfer

The guide is careful about several tempting but inappropriate shortcuts.

**Do not copy the three-block prefetch distance.** Its usefulness depends on the work performed before consumption, and the guide explicitly separates stream count, SIMD width, prefetch distance and request-to-use computation. BLAKE3’s known-address input streams differ fundamentally from our successor-dependent walk. Pasted text

**Do not copy the wider backend’s concurrency policy.** The guide reports that the 32-input AVX-512 variant was selected for a particular CPU family and could be slower elsewhere. Those are author-reported observations, not a universal advantage of widening. Your i9’s documented extensions include AVX2, while the AVX-512 and SME2 sections are architectural references rather than directly applicable instruction sequences. Pasted text [Intel](https://www.intel.com/content/www/us/en/products/sku/199332/intel-core-i910900k-processor-20m-cache-up-to-5-30-ghz/specifications.html)

**Do not import the guide’s whole experimental menu.** Its closing brief is deliberately general: investigate batching, layout retention, lookahead, intrinsics, assembly and several operating conditions. That is useful for entering a new codebase, but applying it wholesale would reopen questions we have already investigated. Pasted text

The existing compact30 task is narrower and more informative.

## A small addendum for Claude

I would send the full guide for preservation, followed by this—not another large implementation brief:

> Please archive the BLAKE3 study verbatim with its pinned source revision, and retain it as implementation background for the already-approved refined compact30 prototype.
>
> Apply three lessons to that task:
>
> **Dependency audit:** show when the current row, offset address, header address, successor and successor-prefetch address become available. Verify that the two table addresses are independently derived; distinguish actual dependencies from textual instruction order.
>
> **State audit:** distinguish necessary lane-array accesses from additional spills or redundant state traffic. Do not pursue register-only code or hand-written assembly without a concrete deficiency in the compiled prototype.
>
> **Physical-access audit:** verify actual record alignment, operand widths and readable extents, including eager header loads on exceptional records and the parking record. Correct output alone does not establish valid accesses. Include the relevant boundary checks in the existing correctness work.
>
> Do not copy BLAKE3’s prefetch distances, widen the lane configuration, introduce a batching or software-pipelining sweep, or change the prototype’s scope. No new performance experiment is requested.

**My overall assessment: this study is useful chiefly because it sharpens how we implement and validate Astra’s idea.** It does not reveal a missing BLAKE3 trick that we should immediately transplant.

The next valuable evidence is still the same: **does refined compact30 compile into the intended simple access path, and does that path remain correct through real segmented traversal?** The guide helps us ask those questions better. It need not delay the prototype, and it calls for nothing additional on your workstation.
`````
