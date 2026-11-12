# External research report 32: the move from the burn-down to Gate 1, and the pause (verbatim)

A research input, not a finding of this repository. The maintainer relayed these
messages on 2026-10-30, after external report 31 (`2026-10-30-external-report-31.md`).

**What it contains:**
- **The researcher's transition note:** three targets in order (P74's compiled kernel,
  N1's Gate B, full-decoder viability), three outcome classes for P74, and a
  pre-experiment falsification review as the last analytical task.
- **The Gate 1 recommendation, with its boundaries:**
  - production unchanged;
  - both variants;
  - the shipped kernel and a layout control;
  - negative controls;
  - a disassembly read;
  - no i9;
  - promotion only on four-worker, CPU-s/GB and RSS criteria.

  It is stated as "a recommendation, not authorization".
- **The scope clarification:**
  - the earlier "yes" was a recommendation;
  - the boundary is the complete checkpoint-aware stage;
  - three controls: a layout rebuild, all three rank paths, and promotion kept separate
    from the stage result.
- **The pause:** the maintainer's own line, and the researcher's reply.

**The coordinator's response:**
- The burn-down is closed formally in §9.
- Gate 1 is pre-registered in `2026-10-30-p74-gate1-preregistration.md` and **not
  started**. Nothing is implemented, built or measured. It waits on the maintainer's
  explicit authorisation.

The text reached us as rendered text (tables flattened) and is kept as received.

## The transition note, as received

````text
Yes—but we’ve finished burning down the theoretical uncertainty. What’s left is burning down the performance risk.

All 84 original questions have dispositions, the literature review is complete, and P74’s correctness has been verified across all alphabet sizes.

I see three worthwhile targets remaining, in priority order.

Priority	Target	Question to settle
1	P74 compiled-kernel economics	Does the register-held MTF actually outperform the shipped loop?
2	N1 Gate B — builder economics	Can the multi-step Ψ builder overcome its much higher modeled construction traffic?
3	Full-decoder viability	Do any gains survive four workers, CPU-seconds/GB, and peak-RSS constraints?

My recommendation: P74 first

Claude’s latest research has made P74 unusually testable.

The benchmark should compare the branching and branch-free register-held implementations against shipped 0.2.1, using identical stored symbol streams and matched runs.

There are three outcomes worth distinguishing:

* Positive: meaningful checkpoint-aware MTF savings, with no significant random or records regression. Advance to full-decoder evaluation.
* Mixed: Wikidata wins but controls regress. Evaluate the per-block selector against its own overhead and whole-decoder requirements.
* Negative: the compiler, branch prediction or call boundaries consume the theoretical advantage. Close P74 as a correct but uneconomic design.

N1 should follow with its builder-cycle gate before spending effort optimizing its walker. A cheaper resident table is not enough if constructing it increases total decode time.

Is there more purely analytical work?

A little, but I would not manufacture new rounds for their own sake. The best remaining analytical task is a pre-experiment falsification review: make sure each benchmark compares the correct boundaries, includes the appropriate negative controls and has a predetermined stopping criterion.

Beyond that, the evidence really does have to come from running code.

My vote: close the theoretical burn-down formally, preserve the research artifacts, and move to P74’s controlled CI gate. Leave the i9 untouched.

We’ve reached a useful transition: from asking could this algorithm work? to asking does this algorithm earn its place in the decoder?
````

## The Gate 1 recommendation, as received

````text
Excellent. The latest checks strengthen P74 considerably, especially the finding that every Wikidata fallback fits within three fixed 32-byte vector moves. That removes memmove from the relevant workload entirely.

The remaining library-call rate—32.5 calls per 1,000 explicit tokens—is also much more favorable than the original 329.

This means P74’s remaining risks are concentrated in branch prediction, register allocation, and the actual throughput of the checkpoint-aware loop.

I’m satisfied that the analytical phase is complete. Further theoretical rounds would offer diminishing returns.

My recommendation is to approve Gate 1: P74’s controlled CI benchmark, with these boundaries:

* Production decoder unchanged.
* Both branching and branch-free variants included.
* Shipped kernel and code-layout control included.
* Wikidata, random and records negative controls included.
* Compiled machine code inspected for register spills.
* No i9 runs.
* No promotion without improved four-worker full-decoder throughput and non-regressing CPU-seconds/GB and peak RSS.

Gate B for N1 remains second.

One distinction worth preserving: removing memmove from the Wikidata fallback doesn’t establish that the same vector-only fallback is valid for arbitrary 256-symbol inputs. The random and records controls must exercise the unrestricted case correctly.

The research decision is now clear: Gate 1 is the highest-value next step.

I recommend authorizing that scoped CI experiment. This message is a recommendation, not authorization or initiation; neither gate has been started.
````

## The scope clarification, as received

````text
Agreed on the scope of P74 Gate 1 only. The intended sequence is sound: formally close the research burn-down, commit the experimental pre-registration, then implement and measure the isolated CI benchmark.

One clarification: my previous “yes” was a recommendation, not explicit authorization. Your message now establishes the intended scope, and nothing in it authorizes N1 Gate B or an i9 run.

The crucial point when inspecting the existing CI harness is to preserve its measurement boundaries. The P74 comparison must include the complete checkpoint-aware MTF stage—RLE2 expansion, symbol mapping, byte counting and checkpoint generation—not merely the SIMD update loop.

I’d also preserve three controls in the pre-registration:

* Code layout: compare against a rebuilt shipped kernel as well as the existing baseline, so binary-layout changes aren’t mistaken for optimization gains.
* Workload correctness: exercise all three rank paths, particularly unrestricted high-rank fallbacks on random and records.
* Promotion: distinguish a positive MTF-stage result from a qualifying four-worker full-decoder improvement, including CPU-seconds/GB and peak RSS.

The latest 32.5-call figure applies to the measured Wikidata workload after proposed inlining; it must not be generalized to arbitrary inputs.

The next decision is no longer theoretical. It’s whether the CI harness can deliver a fair, reproducible stage-level comparison before any full-decoder evaluation. That is exactly the right thing to inspect first.
````

## The pause, as received

````text
> ok we are going to pause now and I am going out. we will return to this later

Sounds good! Enjoy your time out.

We’ve reached a natural stopping point: all 84 research questions have been dispositioned, and P74 is ready for its controlled CI experiment.

When we return, we’ll pick up with P74 Gate 1, keeping production unchanged, N1 Gate B unapproved, and the i9 untouched.

===

Claude it looks like you are busy, so finish up what you’re doing and then I will come back to this later
````
