# External research report 158: the guardian authorises three bounded VM-side tasks: preserve the capacity-aware dispatch replay, P87's integration cleanup, and a census of the 30-entry layout; the Astra paragraph corrected (verbatim)

A research input, not a finding of this repository.

- **Relayed:** by the maintainer, 2026-11-02.
- **Source:** the guardian.
- **In reply to:** the coordinator's checks of reports 153–157, the capacity-aware wake
  replay, and its three proposed VM-side tasks.

**Evidence cutoff:** 2 November 2026. A check of PR #128's CI report; the new replay was not
independently reproduced. Authorises VM-side work only: no i9 capture and no optimised
walker.

It is kept exactly as received (rendered text, cards flattened).

## The message, as received

`````text
This is useful progress. I would give Claude the go-ahead for all three proposed VM-side tasks, with clear limits. They preserve evidence, fix a known integration issue and test the new layout’s assumptions without spending more workstation time.

The most important new result is the capacity-aware wake analysis. The most important remaining qualification is that the older compact-table measurements do not predict the new layout’s cache behaviour.

1. The wake opportunity survived a meaningful challenge

According to Claude’s replay, replacing the original waiting-worker total with

E=\min(\text{idle CPUs},\text{runnable workers},\text{queued jobs})

changes the estimate from 0.226 to 0.225 average core-equivalents. That is useful: the concern that several waiting workers were being counted against the same single idle CPU apparently makes almost no difference in these traces.

The support-thread adjustment gives a second view, around 0.178 core-equivalents. I would preserve both—not turn them into a guaranteed recoverable range.

These are opportunity estimates on the recorded schedule, not simulations of the proposed affinity policy. Once worker masks change, job claims, support-thread execution and phase overlap can all change. Excluding intervals with competing support demand is a sensitivity analysis, not a proven lower bound on the improvement.

Similarly, the quoted 6–8% is conditional arithmetic, not a throughput forecast. The ladder demonstrates a substantial one-to-four-copy cost increase; it does not measure the marginal cost of recovering another 0.18–0.23 cores of activity inside this decoder.

The immediate task is to commit the replay script and results. They should not remain solely in a temporary scratch directory. I have not independently reproduced this new calculation, so preserving its exact inputs and method is particularly important.

2. P87 should receive the small correctness-and-accounting cleanup

I support fixing the ledger, adding the direct separate-buffer/in-place comparison, and documenting the success contract.

The candidate’s actual implementation constructs the table and then passes ll as the walk’s destination. That is the source-level basis for the transformation.

One wording correction: Rust guarantees the relevant borrowing constraints, not that an algorithm has read every required byte correctly. The borrow checker prevents conflicting live references in safe code; the algorithm and its contracts establish that the table contains everything needed before ll is overwritten. Those are complementary arguments, not interchangeable proofs. 

The layout concern is legitimate—but not yet diagnosed

I checked the CI report: it does contain the −1.4% build-stage and +3.2% RLE1-stage changes Claude quotes, alongside the positive whole-decoder result.

That warrants caution. It does not establish that code layout caused those changes, or that layout explains P87’s measured benefit.

Performance sensitivity to seemingly incidental experimental changes is a documented problem. But identifying it in this case requires more than observing that an unchanged algorithm timed differently. 

I would therefore record:

The stage results reveal unexplained between-build sensitivity. They are not attribution evidence for P87’s reuse mechanism.

A deliberately perturbed base build could be a useful diagnostic, but one perturbation cannot establish the full range of possible layout effects. It might move performance a lot, a little or not at all. Do not let that automatically grow into another substantial benchmark project.

For now, fix P87’s integration and contracts. Leave the production-validation design as a separate decision, with the existing no-harm requirements intact.

3. Approve the 30-entry census—but correct the proposed Astra paragraph

The census is exactly the sort of inexpensive preparation that can strengthen tomorrow’s review.

It should answer what the new grouping actually does on the corpus: ordinary and RAW groups, reasons for fallback, bytes per row, per-block distribution, and whether all encoded coordinates and reserved values satisfy the specification.

It should not implement an optimized walker or direct constructor yet.

The historical comparison is relevant. The compact-Ψ record shows a four-block walk ratio of 0.839 for v/o, followed by 0.969 for u/v: changing byte recovery and packing had measurable costs, despite the representation’s memory advantages.

But this part of Claude’s proposed Astra brief goes too far:

“At about 8.4 MB live over four blocks, it would sit in the hit-share range u4 measured.”

We cannot infer that. The approximately 8.4 MB figure comes from applying the proposed logical-storage bound to four qualifying full blocks. It is not a measured resident footprint or a predicted cache-hit rate.

Likewise, removing the older format’s named F-directory and packing operations does not automatically recover their measured percentage costs. The new design still has header access, state handling, exceptions and potentially a different request rate.

The older journal itself reports that u generated 26% more reads past L2 per byte at four blocks, with part of that increase unexplained by the counters. That is another reason not to transfer its cache behaviour solely by matching table sizes.

Better wording for Astra

The historical four-block compact-Ψ result demonstrates that improved memory behaviour can coexist with slower traversal when access overhead is too high. The proposed 30-entry layout removes the old F-directory lookup and packed-bit extraction sequences, but retains other costs that must be evaluated.

Its storage bound supports a materially smaller logical representation; it does not establish its cache-hit rate, resident footprint or performance. Please compare the actual access primitives and dependencies, rather than transferring the old format’s measured gains or penalties directly.

That preserves the useful precedent without turning it into a forecast.

What I would send Claude

Please proceed with the three bounded VM-side tasks below. No i9 capture or new optimized walker is authorized.

1. Preserve the capacity-aware dispatch analysis first. Commit the replay script, exact source/input references and results as a descriptive post-hoc addendum. Include the uncapped, capacity/job-capped and support-demand-filtered calculations, with their denominators and per-run distributions. Preserve uncertainty guards. Label these as opportunity estimates on the recorded schedule—not a simulation of singleton worker affinity, a guaranteed recoverable range or a throughput prediction. Keep the original registered dispatch verdict unchanged.

2. Complete P87’s integration cleanup. Correct the memory ledger and its tests for the segmented in-place path while retaining correct expectations for other kernels. Add the separate-buffer versus in-place differential checks and document both success and fallback contracts. Run the relevant development-machine tests. Do not weaken assertions merely to make CI pass, alter decoding algorithms, merge the candidate or start production timing.

Record the CI stage differences as unexplained between-build sensitivity, not established code-layout causation. A possible layout control belongs in the later validation proposal; it is not authorization for a new benchmark matrix.

3. Run a census of the proposed 30-entry layout. Fetch and hash-check the established corpus as needed. Record each block’s row count, used alphabet, ordinary/RAW group counts and fallback reasons, partial-group and parking overhead, and total logical bytes. Check encoded-coordinate limits, sentinel separation and the proposed storage bound against the actual data. Keep this to census/reference validation—no optimized traversal, direct builder or performance claims.

Please correct the Astra brief: four tables’ logical size does not establish their cache-hit range, and removing the old lookup/packing sequences does not automatically recover their historical percentage costs.

Archive the new material separately from frozen experiments, then return the three results. Keep k/j untimed and the new layout unimplemented as a performance candidate.

This is a good use of the pause before Astra is available: preserve the new wake evidence, finish P87’s small integration obligations, and replace assumptions about the new representation’s corpus behaviour with actual counts.

It advances all three candidates without reopening closed experiments or launching three competing implementation programmes.
`````
