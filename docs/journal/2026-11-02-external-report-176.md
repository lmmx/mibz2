# External research report 176: the guardian holds the compact30 i9 capture and authorises the harness repairs (verbatim)

A research input, not a finding of this repository.

- **Relayed:** by the maintainer, 2026-11-02.
- **Source:** the guardian.
- **In reply to:** the researcher's harness review (report 175), on the harness at `b12b49b`
  and the recipe at `65f7169` (PR #134).

**Evidence cutoff:** 2 November 2026. The guardian's own run of report 175's synthetic
checks, and a reading of the pinned source. No decoder or performance benchmark.
- The guardian's reproduction log and results (linked below as sandbox files) were not
  relayed. The researcher's bundle, which they reproduce, is kept with report 175.

**This authorises, together:**
1. validation and result recording after the completion barrier, preallocated records,
   recorded boundaries, and injected output and pinning failures that end the run
   coherently with evidence and an archive;
2. per-thread CPU durations for the worker-cost ratios and the constructor budget, with
   monotonic timestamps for completion time and synchronisation, and CPU IDs for
   placement, and an assessment of clock-reading overhead (raw values kept, nothing
   subtracted);
3. validation against the complete registered schedule before any ratio, 20 measured
   rounds outside labelled dry runs, and report 175's five cases, validation-overlap cases
   and failure-path cases as rejection tests;
4. the accepted design kept, with clarified names.

Then a recheck of the candidate's machine code, a new source pin, passing regression tests
and an untimed end-to-end dry run.

**It does not authorise:** walker tuning, an alternate layout, a constructor, or the i9
performance run.

It is kept exactly as received (rendered text, cards flattened).

## The message, as received

`````text
**Hold the i9 run. I agree with the researcher: the compact30 walker remains accepted, but this version of its measurement harness is not ready to freeze.** I would authorize the repairs together, without changing the representation, walker or experimental question.

I independently ran the attached synthetic checks. **All 14 original tests passed—and all five invalid cases were accepted as valid and produced ADVANCE, exactly as reported.** The regenerated results match the researcher’s supplied results. No decoder or performance benchmark was executed. synthetic-reproduction.log[Reproduction log](sandbox:/mnt/data/compact30_harness_review/synthetic-reproduction.log)

## 1. The measurement-boundary problem is a genuine blocker

I checked the pinned Rust source. A worker stops its timer, validates its output, returns, and appends a result **before reaching the completion barrier**. An early finisher can therefore read its output and reference buffers while another worker is still traversing. 

The important principle is:

> **Work outside one thread’s timer is not necessarily outside another thread’s measurement environment.**

That matters particularly here because the experiment is investigating concurrent memory-system costs. The validation traffic is an uncontrolled additional workload, and different completion patterns could expose the two arms differently. We do not know its magnitude or direction.

The repair should establish this ordering:

```text
All workers prepare their tables
                 ↓
            Start barrier
                 ↓
     Timed traversal and assembly
                 ↓
          Completion barrier
                 ↓
     Validate output and record results
                 ↓
       Prepare the following step
```

Uneven finishes remain a legitimate result. **The early finisher should wait, not start a large comparison while its neighbour is being measured.**

The researcher’s failure-path concern is also justified by the source: assertions can terminate a participant before a fixed-participant barrier. The harness needs a tested way to fail the run without leaving peers waiting indefinitely. 

## 2. Measure CPU cost directly rather than silently substituting elapsed time

The researcher is right to tighten my earlier wording. I accepted summed worker elapsed durations as an approximation, but **that approximation should not become a measured CPU-cost claim**.

The recorded traversal timestamps use `Instant`, and the analyser sums their differences. That measures elapsed time. Rust documents `Instant` as using a monotonic clock on Unix; Linux separately provides `CLOCK_THREAD_CPUTIME_ID` for CPU consumed by the calling thread.  [doc.rust-lang.org](https://doc.rust-lang.org/std/time/struct.Instant.html)

Also, `sched_getcpu()` returns a **CPU identifier**, not a duration. [man7.org](https://man7.org/linux/man-pages/man3/sched_getcpu.3.html)

I support the researcher’s preferred resolution:

| Quantity | Use |
|---|---|
| **Summed thread CPU duration around traversal** | CPU-cost ratio and conditional constructor budget |
| **Monotonic start/end timestamps** | Completion time, synchronization and overlap |
| **CPU identifier** | Placement verification |

Clock-reading overhead needs a documented treatment, particularly for the tiny blocks. That does not require another PMU campaign or collector redesign; it requires checking whether the measurement calls materially affect the intended aggregate result.

The existing elapsed measurements remain useful. They simply need the right name and purpose.

## 3. The analyser must check that the records describe the registered experiment

The reproduced denominator case is especially clear:

| Synthetic case | Reported cost ratio | Decision |
|---|---:|---|
| Original invented measurements | 0.950 | STOP |
| Same durations, candidate row counts doubled | 0.475 | **ADVANCE** |

This does **not** show that Claude’s Rust harness currently emits wrong row counts. It shows that the analyser would not catch that error. gate-review-results.json[Reproduced counterexamples](sandbox:/mnt/data/compact30_harness_review/gate-review-results.json)

The underlying problem is that the checks validate separate pieces without fully connecting them. The analyser verifies block completeness and assignment metadata, but does not establish that each recorded worker, step, block and row count matches the declared assignment. It also trusts the metadata’s round count, allowing a one-pair run to receive a zero-width interval and an experimental decision. 

The repair is to derive the expected record set from the frozen protocol and assignment, then validate it **before calculating performance ratios**. It should reject missing, unexpected, duplicated or misassigned records—not merely check that the file looks internally plausible.

## What can stay unchanged

I would retain the **21-instruction candidate, one-/four-core comparison, lockstep workload, one-block allocation tolerance, 0.865 investment screen and 1.02 completion-time tolerance**.

The researcher’s remaining wording corrections are appropriate:

**`e4` is summed lockstep traversal completion time**, not whole-capture or decoder throughput. And **0.865 is a rounded investment threshold**, not an exact guarantee that the constructor budget is at least 1. These do not justify a new design or threshold debate.

The 2% tolerance should simply say what it permits: completion time within a predeclared 2% regression allowance—not proof of zero trade-off.

## Send Claude this

> **Hold the i9 capture. I accept the researcher’s harness review and authorize the following repairs together. The reviewed compact30 walker and its architecture must remain unchanged.**
>
> I independently reproduced all five synthetic counterexamples in the attached evidence bundle. Preserve that bundle and turn the cases into rejection tests.
>
> **1. Separate postprocessing from every worker’s timed traversal.** Move output validation and result insertion/allocation after the completion barrier. Preallocate measurement storage. Record enough boundaries to verify that no validation starts before every active traversal in that step has ended, and that no next-step preparation overlaps the preceding step’s traversals.
>
> Test this with an intentionally uneven synthetic schedule. Also inject output and pinning failures: the run must terminate coherently, retain failure evidence and produce its archive rather than strand peers at a barrier. Do not add a new concurrency framework unless necessary for that bounded requirement.
>
> **2. Record actual thread CPU durations.** Use per-thread CPU time for the worker-cost ratios and constructor-budget calculation. Retain monotonic timestamps for completion time and synchronization, and CPU IDs for placement checks. Exclude barrier waiting, validation and table preparation from the traversal CPU window.
>
> Document the clock-call ordering and assess clock-reading overhead, including its aggregate effect across short blocks. Preserve raw measurements; do not silently subtract an assumed overhead. No PMU collection or workstation-policy change is requested.
>
> **3. Validate against the complete registered schedule before computing ratios.** Enforce 20 measured paired rounds outside explicitly labelled dry runs. Verify each expected `(round, mode, arm, worker, step, block, rows)` record, correct row denominators, exact step membership, declared partial-step participation, placement and cross-step ordering. Reject insufficient samples rather than manufacturing a zero-width confidence interval.
>
> Add the researcher’s five cases, validation-overlap cases and failure-path cases to the regression suite. An invalid record set must not reach an ADVANCE/STOP performance decision.
>
> **4. Preserve the accepted design and clarify its names.** Keep the deterministic allocation, lockstep condition, rounded 0.865 investment screen, 1.02 completion-time tolerance and cache-preparation limitation. Describe `e4` as the sum of each step’s earliest recorded traversal start to last traversal end. Report the actual constructor budget at the confidence-interval endpoints.
>
> Recheck the candidate’s machine code after the harness revision, then return a new source pin, passing regression tests and an untimed end-to-end dry run. Continue packaging both successful and failed runs under `target/`.
>
> **No walker tuning, alternate layout, constructor implementation or i9 performance run is authorized by this instruction.**

**This is not a setback in compact30’s architecture.** The researcher found specific ways the experiment could measure the wrong environment or accept the wrong records. Those are worth fixing before the first result, not explaining away afterward.

Nothing needs running locally. The next approval should depend on demonstrated rejection of these failure cases and a clean measurement path—not simply a larger count of passing tests.
`````
