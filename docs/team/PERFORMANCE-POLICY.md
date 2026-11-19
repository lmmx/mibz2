# Performance trade-off policy

*Set by Louis on 10 November 2026, in the Coordinator's session. It is his rule (RULES D), and it applies from this date forward.*

> "Correctness, memory safety, and reproducible evidence remain mandatory. Perfect performance across every workload, complete causal explanations, and exhaustive experimental certainty do not."
>
> "Optimise for useful speedups merged per unit of engineering effort. Every further investigation needs a specific, near-term integration decision. Stop allowing process to become the product."

## 1. Never negotiable

A change merges or becomes a default only with all of these:

- **Correctness:**
  - byte-identical output on every bench corpus;
  - its equivalence and differential tests pass;
  - corrupt input gives the same error as before.
- **Memory safety:** new `unsafe` code meets B5. That means a written safety argument; Miri, differential, fuzz and mutation testing; and an adversarial review by an agent other than the implementer.
- **Reproducible evidence:** the figures come from CI's paired bench, or from a workstation run under C5, archived with its digest.
- **Green CI** on the exact head merged.

## 2. Performance: net useful gain, weighted to what the decoder is for

The decoder exists mainly for Wikidata dumps and text-like data.

| class | corpora |
|---|---|
| **primary** | `wikidata`, `text` |
| **secondary** | `records`, `random`, `pattern`, `onebyte`, `runs` |

A performance change is **accepted** on its CI paired bench when all of these hold:

1. **A primary gain:** at least one primary corpus is faster in wall time beyond noise (its 95% interval excludes zero) at one or more thread counts.
2. **No primary loss:** no primary corpus is slower beyond noise at any thread count.
3. **Bounded secondary losses:**
   - **Up to 5% slower** (point estimate, wall time) at any thread count: accepted, and listed in the PR.
   - **More than 5%, up to 10%:** accepted only if the PR shows the primary gain at that thread count is at least twice the loss. It must also say why the workload matters less.
   - **More than 10%:** needs Louis's sign-off. The Coordinator may defer or abandon such a change without asking.
4. **Memory:** peak RSS grows by no more than 10% on any corpus, or the growth is traded for speed:
   - **Accepted by rule:** more than 10% is accepted when all of these hold:
     - a workstation run on the target machine (C5, paired) shows a primary wall-time gain of at
       least 5%, beyond noise, on the corpus and at the thread counts where memory grows;
     - the growth, in percent, is at most twice that gain at each of those thread counts;
     - the absolute growth stays small for an ordinary laptop: at most 64 MB of peak RSS per
       process;
     - no other corpus grows by more than 10%.

     The PR states the trade: the gain, the growth in percent and in MB, and the run that measured
     both.
   - **Otherwise:** growth beyond 10% needs the PR's own justification for the target machine, and
     the Coordinator weighs it, as before.

   (Louis, 18 November 2026: the 10% line is a trigger for weighing, not a ban. The first case was
   WideLazyCells: 16.2% less wall time for 13.9% more peak RSS at 8 threads, and 12.2% for 15.9%
   at 10, +25 MB on Wikidata only. LOG 6975512468.)

A change built for a specific machine, gated on its hardware, is judged on that machine (C5) for its gain. On every other machine it must show that it stays inactive. A deterministic check that the route is not taken, plus a neutral CI bench, settles that. Statistical non-inferiority is not required where the code path is not taken.

**Adaptive dispatch is fair; changing the measurement is not** (Louis, 19 November 2026: "it is ok to detect properties of the input and switch approach accordingly but not to artificially modify the benchmark etc").
- **Allowed:** choosing a kernel or route per block from properties the decoder observes at run time, such as run density or run structure. LazyCells' density line is an example. The rule should predict which route is cheaper, fitted to measured per-block cost, not to which corpus a block came from. It must hold on held-out data that wasn't used to fit it.
- **Not allowed:** changing what is measured so that a candidate passes. That covers adding, dropping or reweighting corpora, loosening a guard or bound, swapping the reference, or moving a threshold after its data has been seen.
- The bench corpora, their classes above and the registered guards stay as they are. A dispatch rule passes or fails on them unchanged.

**One bench run decides.** A regression flagged in a secondary corpus is weighed under rule 3, not re-run to settle it. Only a flagged primary regression gets one re-run; if it shows again, it stands.

## 3. Statistical tests

The standing CI paired bench and sign test are pre-approved. A **new** test needs one review (Researcher); the Guardian is involved only on a HOLD. When a test is shown unsound, it is replaced by a simpler sound one. It is never just dropped, and never kept on trust.

Prefer designs that are small and exact: paired runs, sign tests, a deterministic route check. Avoid large replicated programmes.

## 4. Investigations

Every experiment names the integration decision it will settle, and when. An instruction-count result is followed promptly by a wall-clock measurement. If the change is faster, it moves towards integration; if not, it is dropped.

Complete causal explanations are welcome, but they are never a precondition for merging a measured gain.

### 4a. Decision quality: the cost of evidence must match the decision (Louis, 19 November 2026)

**The rule:** any experiment costing more than **30 minutes of workstation time** must state three
things in its registration, before its data:
1. **The decision it changes.** Say what will be done on each outcome: pass, fail and
   inconclusive.
2. **The causal relevance of each condition it tests.** Name the mechanism by which the change
   could affect that measurement, and give the evidence for it (static path, routing census,
   deterministic counts).
3. **Why cheaper evidence won't do.** Cover the static, deterministic, CI and already-recorded
   evidence.

**Who is accountable:** the **Coordinator**.
- The Researcher rules on statistical validity, and the Guardian on correctness and safety.
- The Coordinator decides whether the remaining uncertainty is worth the cost. A registration
  that satisfies every specialist can still fail this test.
- When a review round raises an experiment's cost by more than half, the Coordinator re-asks
  the decision question before accepting the increase.

**Why this rule exists:** the open port (#630), 19 November 2026.
- *The first attempt:* its single registered attempt failed four guard point bounds, all on
  corpora that route no block to the new kernel. Its verdict, NOT ADOPTED, stands.
- *The successor (#635):* it then grew, through correct review rounds, from 65 minutes to about
  7 hours of i9 time, to measure layout-scale noise on code paths the change barely touches.
  No one asked what decision those hours would change. Louis stopped it.

**Causal-equivalence evidence:** for a corpus whose execution path the change leaves
demonstrably near-unchanged, the evidence is:
- **What it consists of:**
  - a static path analysis, quoting `file:line`;
  - a routing census showing the new route is not taken;
  - byte-identical output;
  - deterministic instruction, cache and branch counts (cachegrind) against the baseline.
- **What it settles:** the question of *mechanism*, so these corpora don't need per-cell
  powered wall-time guards. Any trigger is attributed in writing first.
- **What it does not replace:**
  - **a short pooled paired wall-time check on the same corpora.** A machine-level
    difference (code layout, an unused inline field, a new monomorph) is real deployed cost,
    so it is bounded in pooled form. A single cell clearly beyond the guard bound is still
    flagged.
  - **the usual guards where the new code executes:** correctness, memory, primary, held-out
    and sustained.
- **Its limits, stated in each use:**
  - Simulated counts aren't native timing.
  - Zero routing is not "identical": the scratch, layout and fallback checks still differ.
  - The pooled wall check has less per-cell power than a powered per-cell design. It is
    proportionate when, and only when, the causal analysis finds no mechanism specific to
    that cell.
- **When it applies:** from 0.5.7 on, prospectively. **It never converts an earlier failed
  registration:** the open port's first NOT ADOPTED stands permanently.

## 5. History

- **Earlier verdicts stand as recorded.** #324's registered no-harm rule failed (`random` 1t +5.3%, `records` 1t +2.1% wall, run 3 of 42747668401), and that verdict is not relabelled.
- **#324 itself** is judged under this policy on its merged head's fresh CI bench.
- **Why the numbers are what they are:** the 5% and 10% tiers were chosen with #324's figures known, following Louis's statement that its trade-off is acceptable. They are not presented as blind.
