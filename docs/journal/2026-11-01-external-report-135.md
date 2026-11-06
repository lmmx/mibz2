# External research report 135: the guardian on FOR-14 stage 1: one i9 confirmation, then the walker-ceiling diagnostic; and, on the maintainer's question, a bounded BMI2 code-generation test first (verbatim)

A research input, not a finding of this repository.

- **Relayed:** by the maintainer, 2026-11-01.
- **Source:** the guardian. The maintainer's question ("but wait, didnt we previously find
  ways to use fewer instructions…") and the guardian's answer to it are kept in place,
  after the review.
- **In reply to:** the FOR-14/raw stage-1 record (lmmx/mibz2#130).

**Evidence cutoff:** 1 November 2026, after the stage-1 record and before any i9 run of it.
A review and its authorizations, not a result: one frozen i9 FOR-14 confirmation; a bounded
BMI2 code-generation test of the `h` arm (no format change), measured as a separately
identified candidate if it improves materially; and the minimal-walker diagnostic's VM
preparation, returned for review before any i9 capture.

It is kept exactly as received (rendered text, cards flattened, tracking parameters
included).

## The message, as received

`````text
## FOR-14: we've learned something important, even though it looks likely to fail

Claude's work is a productive outcome of Astra's proposal.

The most valuable discovery isn't that FOR-14 compresses the table to 52% of its original size. We already suspected that would be possible.

It's that obtaining that compression while preserving correct BWT navigation currently requires much more execution work than the shipped representation.

Table footprint

# −48.1%

3.60 → 1.87 MB

Ordinary-step instructions

# 16 → 38

About 2.4× as many

VM walk speed, H/O

# 0.61–0.66×

Indicative, not an i9 result

I've reviewed Claude's actual [Stage 1 record in PR #130](https://github.com/lmmx/mibz2/pull/130?utm_source=chatgpt.com). The physical format is substantially more concrete than Astra's original sketch, the census agrees with the historical data, and the correctness coverage is extensive.

I agree with stopping work on a direct builder for now. Implementing P84 when the optimistic navigation path already looks this expensive would be a poor use of engineering time.

However, I would make one correction to the reasoning before we declare the architecture conclusively rejected.

## The stopping bound is conditional, not a physical law

Claude's argument is:

\\[ C\_{h,4}\geq C\_{h,1}=\frac{1.236}{W_1} \\]

where \\(W_1\\) is FOR-14's one-worker speed relative to the original walker.

Under the assumptions that construction and other phases are unchanged, and that the compact walk cannot become cheaper per byte at four workers than at one, the thresholds follow:

| Objective                             | Necessary one-worker speed ratio |
| ------------------------------------- | -------------------------------- |
| Break even on traversal plus assembly | 0.669×                           |
| Recover 0.25 CPU-s/GB from that phase | 0.774×                           |

The arithmetic is sound given those assumptions.

But two qualifications matter:

1. Four-worker cost need not always exceed one-worker cost. That is a reasonable expectation for competing memory-intensive workers, not an unconditional physical law. Changes in phase overlap, cache state, scheduling and measurement context can affect the relationship.
2. A future direct builder could change the economics. The 0.774 threshold assumes no construction saving. If compact construction were materially cheaper, it could compensate for some navigation regression. Astra didn't establish that possibility, and the current implementation gives no positive evidence for it, but we shouldn't present 0.774 as a universal bound for every compact representation.

This doesn't rescue FOR-14. It simply means the result should be characterized correctly:

A working, half-sized representation whose current navigation implementation has a strongly unfavorable execution-cost model, making further construction work unjustified without new evidence.

That's a convincing engineering stop, even without claiming mathematical impossibility.

## Would I run the optional three-minute i9 confirmation?

Yes, I would authorize that one narrowly scoped run, provided its existing frozen protocol and calibration checks are sound.

Not because I expect FOR-14 to win. I don't.

I'd run it because we have reached the point where a cheap measurement could replace a prediction that currently carries a consequential architectural decision.

The current rejection rests on a model fitted from earlier `v` and `u` measurements, plus indicative timings from a VM with a radically different cache environment.

If the i9 confirms a one-worker ratio near 0.60, the decision becomes substantially stronger. If it surprises us with 0.80, we've learned that the instruction-count model was misleading—and that matters for future succinct representations.

The cost is sufficiently low and the decision sufficiently important to justify one confirmation.

I would not authorize optimization of the `h` walker, another format or a direct builder as part of that run.

## Then pivot to the researcher's hardware-ceiling diagnostic

The researcher's last contribution is becoming more valuable.

FOR-14 has exposed the essential question:

> How expensive is the work required to recover a navigation link, and how much of that expense can the i9 overlap with memory latency?

The proposed minimal-walker ladder can investigate that directly rather than guessing from instruction counts.

I would now rank the immediate work as follows:

1\. Confirm or reject FOR-14's predicted stop

One frozen three-minute i9 comparison, no new variants.

2\. Prepare the minimal-walker diagnostic

Determine whether instruction throughput, outstanding memory requests or decoder bookkeeping impose the practical limit.

3\. Choose the next architecture from the measured bottleneck

Don't automatically switch to another compressed representation or to R2. Let the diagnostic change our ranking.

There is one caveat for the minimal-walker experiment too: a bare pointer chase is an intentionally optimistic reference, not a physically attainable decoder. It removes necessary output and termination work. The interesting quantity is where the gap opens as those operations return, not simply how fast the bare loop runs.

## What I'd tell Claude

The FOR-14 Stage 1 record is accepted as a successful feasibility investigation with an unfavorable predicted performance result.

Do not implement a direct builder, another packed format, or further FOR-14 optimization.

I agree that the 38-instruction common path is a serious obstacle. Please retain one qualification in the permanent record: the 0.669 and 0.774 one-worker thresholds are conditional on the phase-cost model, unchanged construction costs, and the assumption that four-worker navigation cost cannot fall below its one-worker cost. They are useful planning bounds, not universal hardware laws.

I authorize **one frozen i9 FOR-14 confirmation**, using the existing `just bench-for14` protocol, provided the source pin, corpus hashes and calibration checks pass. Do not change its registered gates after seeing the result.

After that, stop FOR-14 under its registered decision unless its outcome explicitly earns further investigation. No automatic second trial.

Separately, begin preparing the researcher's **minimal real-T-vector walker diagnostic** on the development VM, within its one-engineer-day preparation budget. Its job is to distinguish execution overhead from memory-request limitations, not to become another open-ended lane-count optimization project.

Before any i9 capture for that new diagnostic, return its source pin, correctness evidence, counter availability audit, protocol and stopping rules for review.

Keep P87 production, R2/P85, wake-placement and the neighbour harness on hold until we have interpreted the walker-ceiling evidence.

## The bigger picture

I wouldn't regard this as another frustrating failure.

Astra identified an architectural possibility; Claude rapidly turned it into an exact representation; the corpus census confirmed the information-theoretic opportunity; and the generated machine code exposed the practical price.

That is precisely the kind of cheap architectural falsification we wanted.

But I also wouldn't keep repeating this exercise across a succession of encodings. FOR-14 may be the point where we stop treating navigation compactness as our default route to a breakthrough.

The next valuable question is no longer whether we can encode the table in fewer bytes. We can.

It's whether the processor has a large remaining execution opportunity that a better algorithm could exploit—or whether our next major gain must come from eliminating some of the required work altogether.

My recommendation: one short i9 confirmation, then make the hardware-ceiling diagnostic our next architectural priority.

---

> but wait, didnt we previously find ways to use fewer instructions if we tried, is there any chance we could get it down ??

Yes. I think you're right to challenge the stop here. We have previously found that changing how the walker is expressed can substantially alter the generated instruction count. And Claude's own FOR-14 work went from 48 instructions per ordinary step to 38 with one change to the control flow.

That is a reduction of roughly 21% without changing the mathematical representation.

So I would distinguish two conclusions:

- The current FOR-14 implementation is too expensive. The evidence strongly supports that.
- FOR-14 cannot be implemented cheaply enough to compete. We have not established that.

The second conclusion is stronger than the evidence permits.

There is a particularly important clue in Claude's journal: it estimates that BMI2's variable-shift instruction, `shrx`, might bring the common step down to around 27 instructions, but this version was never compiled. The existing implementation uses a baseline x86-64 target and pays heavily for extracting 14-bit packed offsets.

That is an actual untested implementation hypothesis, not merely a vague hope that a compiler might do better.

And there's a concrete reason to try, rather than simply hoping for better code generation.

Claude's audit attributes approximately 12 of the 22 extra instructions to extracting a 14-bit offset. That makes extraction the obvious target.

| Walker                  | Instructions per ordinary step | Evidence                   |
| ----------------------- | ------------------------------ | -------------------------- |
| Shipped `o`             | 16                             | Measured disassembly       |
| FOR-14 first version    | 48                             | Measured disassembly       |
| FOR-14 sentinel version | 38                             | Measured disassembly       |
| FOR-14 with BMI2 `shrx` | \~27                           | Claude's untested estimate |

The i9-10900K supports BMI2, so the last idea is technically relevant to the actual target machine.

The important catch is that 27 instructions does not guarantee a sufficient speedup. The previous `v` walker used 28 instructions and achieved just 0.721× the control at one worker, below Astra's provisional 0.774 threshold. But instruction count alone doesn't determine performance: dependencies, µops, loads, stores and how the compiler schedules them all matter.

I'd authorize one bounded code-generation experiment before closing FOR-14:

1. Compile the existing `h` arm for the actual i9 instruction set, including BMI2.
2. Inspect whether the packed-offset extraction becomes substantially cheaper. Compare instruction and µop counts, the dependency chain and any new spills.
3. Run the existing correctness checks, without changing the record format.
4. If code generation improves materially, use the already frozen i9 comparison to measure its actual performance—but distinguish this as a separately identified candidate, not a silent alteration of the frozen original arm.

I'd aim for a common path around 24–28 instructions, though speed and dependency depth—not an arbitrary instruction target—should ultimately determine whether it's viable.

I would not authorize another general rewrite, a new encoding format or a lane-count sweep. The experiment should be limited to finding out whether we're paying an avoidable instruction tax for packed-field extraction.

One subtle point: if we change the generated machine code, the previous 38-instruction stop analysis no longer applies unchanged. And if we change the benchmark binary after freezing it, the new version needs its own recorded identity and decision procedure.

My revised recommendation: don't spend days optimizing FOR-14, but don't close it while its largest identifiable instruction cost has an untested, target-supported remedy.

A small BMI2 code-generation test is worth doing. If it still leaves a costly dependent path, we can reject this implementation with much greater confidence. If it cuts the cost sharply, we may have rescued the most promising compact-navigation design we've found.

That is exactly the kind of targeted architectural optimization I'd allow before moving on to the minimal-walker diagnostic.
`````
