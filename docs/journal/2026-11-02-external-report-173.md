# External research report 173: the guardian accepts the researcher's conditional go: correct the count to 21, validate the uncounted walker on fresh scratch and repeated walks, then prepare one traversal comparison protocol (verbatim)

A research input, not a finding of this repository.

- **Relayed:** by the maintainer, 2026-11-02.
- **Source:** the guardian.
- **In reply to:** report 172, the researcher's review of the compact30 prototype.

**Evidence cutoff:** 2 November 2026. The guardian checked the researcher's recount and
recomputed its JSON arithmetic. It did not rerun the executable.

**This authorises:**
- the instruction-count correction;
- the strengthened validation of `room::<false>`;
- recording the final identity;
- preparing one traversal-comparison protocol.

**It does not authorise:** timing, a direct constructor, production integration, a new
workstation command, or any change to the traversal design.

It is kept exactly as received (rendered text, cards flattened). Its two citations are to
report 172's attachments.

## The message, as received

`````text
**I agree with the researcher’s conditional go. Compact30 has earned preparation of a performance comparison—but first correct the instruction count and close the specific test-coverage gap. Neither requires another architectural redesign.**

## 1. The count is 21, and I need to correct my earlier assessment

The attached execution-order listing contains **eight instructions before the ordinary-path branch and thirteen in the ordinary body and loop control: 21 in total**. I checked that recount. I previously repeated Claude’s 19-instruction claim despite the displayed listing containing all 21; that was my error. ordinary-path-recount

The corrected comparison is:

| Ordinary full-loop step | Instructions | Data loads | Stores |
|---|---:|---:|---:|
| Shipped reference | 16 | 3 | 2 |
| Compact30 | **21** | **4** | **2** |

**The gap is five instructions, not three.** That matters for the record, but it does not overturn the implementation decision: 21 is still within Astra’s stated 19–21 expectation, and the intended removal of packed extraction, carried-byte state and directory lookup remains.

I also agree that **we should not now chase 19 by changing the implementation**. In particular, leave the self-move alone for this checkpoint. A reporting mistake must not silently become a new optimization objective.

## 2. The correctness gap is real, narrow and repairable

The researcher distinguishes this correctly from a decoding failure.

Both compiled versions have been compared against production, but the existing sequence gives the audited, uncounted version scratch buffers that the counted version has already grown. It therefore does not independently demonstrate cold-growth coverage for the implementation we intend to measure. The explicit marker-restoration comparison also applies only to the counted version. review-calculations

The needed sequence is straightforward:

```text
Fresh candidate records and scratch
    ↓
Run the uncounted walker
    → check output
    → check complete record restoration
    ↓
Run the uncounted walker again, without rebuilding
    → check output
    → check complete record restoration

Run the counted transition audit separately,
without using it to prepare the uncounted walker's scratch.
```

This tests two different obligations: **starting correctly from fresh scratch** and **leaving the representation reusable afterward**.

The existing transition accounting remains useful. I independently recomputed the arithmetic in the attached JSON: RAW arrivals, resumptions and waiting turns balance; segment initialization plus replacement matches completed segments; and the logical-storage formula reconciles. That verifies those calculations—not the executable or the full correctness suite. review-calculations

## 3. The review supports measurement, not another BLAKE3 or Astra assignment

The researcher found no concrete dependency defect requiring a new schedule. Its assessment is that independently addressed field/header loads, successor formation and output bookkeeping have the intended structure.

**That is the useful endpoint of the static review.** It does not prove that the processor overlaps the loads effectively, or that the smaller table compensates for the additional execution work. Those are now appropriate measurement questions.

There is no reason to ask Astra to revisit the architecture because the count is 21. Nor should we start batching loads across lanes simply because BLAKE3 demonstrates that technique elsewhere.

The next step is to repair the validation sequence, preserve the candidate, and prepare its first performance comparison.

## Send Claude this

> **The independent review gives compact30 a conditional go for performance-experiment preparation. Please make the two narrow corrections first, without changing the traversal design.**
>
> **Correct the ordinary-path count to 21.** The executed path contains eight instructions at `15f5b`–`15f78` and thirteen at `15f30`–`15f59`. Update the journal, PR description and comparison tables. Retain four data loads, two stores and one prefetch. Record this as an instruction-count correction, not a change in the binary or a performance result.
>
> Do not try to reach 19 instructions, remove the self-move, introduce another variant or change the loop schedule as part of this repair.
>
> **Strengthen validation of the actual uncounted implementation.** Run `room::<false>` first on fresh scratch, including cases that require segment-buffer growth. Compare its output against the independent reference and check complete record restoration after the call. Run it again on the same records without rebuilding, and repeat both checks.
>
> Keep the counted implementation’s transition audit on a separate instance so that it cannot pre-grow or otherwise prepare the audited implementation’s scratch. Explain how the growth-forcing cases exercise the uncounted version; do not use the counted version’s growth totals as a substitute.
>
> Rerun the relevant established and adversarial correctness suites in release and debug configurations. Preserve the original results and add the strengthened validation separately.
>
> **Record the final identity.** Capture the source commit, compiler/LLVM versions, flags and binary hash. Recheck the ordinary and exceptional machine-code paths against the reviewed candidate. Report any substantive code-generation difference caused by the harness changes.
>
> **After those checks pass, prepare one separately registered traversal comparison.** Use matched one-core and four-core comparisons against the shipped conventional walker. Include candidate setup, marking/restoration, RAW handling, tails and assembly at the corresponding traversal boundary. Keep oracle construction explicitly excluded and distinguish that traversal result from a production-decoder result.
>
> State the construction-cost budget and decision rules in advance. Beating old FOR-14 or showing better scaling is not sufficient: the result must establish whether there is enough absolute traversal benefit to justify direct-construction work.
>
> Return the corrected record, validation evidence and proposed protocol for review. **No timing, direct constructor, production integration or new workstation command is authorized yet.**

**My disposition: accept the researcher’s review, authorize the narrow repair and protocol preparation, and keep the architecture fixed.** This is not another setback or another round of speculative design. It is the final validation cleanup before asking the hardware the question that static analysis cannot answer.
`````
