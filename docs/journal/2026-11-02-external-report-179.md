# External research report 179: the guardian accepts the G5 gap and authorises the analyser-only repair (verbatim)

A research input, not a finding of this repository.

- **Relayed:** by the maintainer, 2026-11-02.
- **Source:** the guardian.
- **In reply to:** the researcher's review of the repaired harness (report 178).

**Evidence cutoff:** 2 November 2026. The guardian's own reading of the pinned analyser. The
guardian did not rerun the new synthetic tests.

**This authorises:**
- the G5 calibration-completeness repair in the analyser and its tests, with the
  researcher's rejection cases;
- the wording corrections: G5 a calibration-based overhead estimate; the schedule rebuilt
  from the recorded `block_rows`; failure handling covering explicit harness failures and
  Rust panics;
- an updated pin.

**On the capture:**
- A capture not yet started waits for this repair.
- One already run is kept and replayed offline with the corrected analyser, with the
  collection pin and the analysis pin recorded separately. It is not rerun because of
  the repair.

**It does not authorise:** walker work, timing variants, counters or threshold changes.

It is kept exactly as received (rendered text, cards flattened).

## The message, as received

`````text
**The researcher is right. This is a real gap in G5’s validation—but importantly, it is not another defect in the measurement procedure.**

I checked the pinned analyser. It verifies the `before`/`after` labels and that the combined median list is nonempty, but not that each batch contains all four expected CPUs and the prescribed samples. Removing a CPU’s calibration can therefore lower the estimated overhead without being rejected. 

**I missed that completeness check when approving the capture.** The correction should stay exactly as narrow as the researcher proposes.

## First: whether you have already started matters

**If you have not started the i9 capture, hold it until this analyser-only repair is complete.**

**If the capture is already running or finished, do not interrupt it, delete it or automatically rerun it.** Preserve the archive. The repaired analyser can check its calibration records offline. If both complete calibration batches are present and all gates pass, this omission alone does not require collecting the measurements again.

That distinction matters: the defect is that the analyser **could accept missing evidence**, not that the collector has been shown to produce bad measurements. The reviewed collector already iterates over the configured CPUs and records their calibration summaries. 

## What needs changing—and nothing more

The existing measurement design remains accepted: completion barrier before validation, actual thread CPU time, separate elapsed-time measurements, and the complete traversal-schedule checks.

G5 needs the same discipline applied to its own inputs:

| Calibration requirement | Required validation |
|---|---|
| Batches | Exactly one `before` and one `after` |
| CPUs in each batch | Every configured CPU exactly once; no missing, duplicate or unexpected IDs |
| Samples | Exactly 20,000 per CPU |
| Summaries | Five finite, nonnegative, ordered quantiles for both CPU and wall time |
| Invalid evidence | Explicit rejection before using the medians or issuing an investment decision |

The researcher’s terminology correction is also appropriate. **The largest observed median is an overhead estimate, not a mathematical upper bound on every clock call.** Keep the 0.5% screen and the raw measurements; correct the description rather than changing the experiment.

I have verified the source omission here, **not independently rerun this new synthetic test suite**.

## Send Claude this

> **Please make the researcher’s bounded G5 repair before any not-yet-started capture. The Rust collector, walker, synchronization and measurement windows remain accepted and unchanged.**
>
> Validate calibration structure before extracting medians: exactly one `before` and one `after` batch; in each, exactly the CPUs listed in `meta["cpus"]`, once each; `n == 20_000`; and five finite, nonnegative, ordered quantiles for both CPU and wall time.
>
> Missing, duplicate, wrong-CPU, wrong-sample-count or malformed entries must produce an explicit INVALID result, not a reduced overhead estimate or an unhandled parser exception.
>
> Add the researcher’s rejection cases, particularly the example where deleting the expensive CPU’s calibration otherwise changes INVALID into ADVANCE. Confirm that complete valid calibration leaves the existing calculations and decisions unchanged.
>
> Describe G5 as a **calibration-based overhead estimate**, not a guaranteed maximum perturbation. Preserve the threshold, unsubtracted primary measurements and separately labelled sensitivity calculation.
>
> Keep the reporting qualifications: schedule reconstruction uses the recorded `block_rows`, and failure handling covers explicit harness failures and Rust panics—not every possible process termination.
>
> Return the analyser/test diff, regression results and updated pin. This needs a focused validation review, not another architecture or protocol-design round.
>
> **If the user has already run the approved collector, preserve that archive and replay the corrected analyser offline. Record the original collection pin and corrected analysis pin separately. Do not request a rerun merely because the analyser was repaired.**
>
> No new walker work, timing variants, counters or threshold changes.

**This should briefly take precedence over the blog, then Claude can return to the separate docs task.** There is no need to withdraw the blog assignment or ask you to move its bundle.

The previous major blockers are closed. What remains is making the calibration gate check the evidence it already claims to require—and then evaluating the capture, whether it has already happened or is still waiting.
`````
