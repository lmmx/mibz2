# External research report 178: the researcher's review of the repaired traversal harness, and G5's calibration-completeness gap (verbatim)

A research input, not a finding of this repository.

- **Relayed:** by the maintainer, 2026-11-02.
- **Source:** the independent researcher.
- **In reply to:** the repaired harness at `c0f8d38`, with the recipe and run records at
  `1a15542` (PR #134).

**Evidence cutoff:** 2 November 2026. A source review, and the 34 Python regression tests
and further synthetic checks run on hash-verified copies of the analyser and tests. No
decoder or performance benchmark.

**Its attachments were not relayed.** The evidence bundle and `independent-review.json`
(sandbox links below) were not passed on.

**Its findings:**
- The completion barrier, thread-CPU accounting, failure path and rejection of report 175's
  five cases are correct.
- One gap remains: G5 accepts incomplete or malformed calibration batches. In a synthetic
  example, deleting the expensive CPU's calibration turned INVALID into ADVANCE.

It is kept exactly as received (rendered text, cards flattened).

## The message, as received

`````text
**The substantive repairs are correct. I would accept the revised Rust measurement harness and close the previous objections. Before freezing it, however, there is one small remaining validation defect in the new clock-overhead gate, G5.**

This is **a bounded Python analyser repair**, not another change to the walker, synchronization design, timing windows or experimental question.

I reviewed pin **`c0f8d3832b442dde6f2908a1c80e328641bc6e3e`**, the recipe and committed run evidence at `1a15542`, and independently ran all **34 Python regression tests**. My local copies of the analyser and tests match their GitHub Git-blob hashes exactly. No decoder or performance benchmark was executed.

## 1. The previous measurement-boundary problem is fixed

The four-worker source now has the required sequence:

```text
Prepare table
Start barrier
Measure traversal: monotonic start → CPU start → walk → CPU end → monotonic end
Completion barrier
Validate output
Write the preallocated result slot
Prepare the next block
```

The completion barrier is genuinely **before** validation and record insertion. Result storage is allocated before the step loop. Workers with no block in the final partial step still participate in both barriers. 

That resolves the original interference problem: an early finisher cannot start reading its output and reference arrays while another worker is still traversing.

The analyser now also checks the corresponding relationships across workers and steps, including:

\[
\max(\text{traversal ends})
\leq
\min(\text{validation starts}).
\]

It checks that the next preparation cannot begin before the preceding step’s traversals finish, and that each worker completes its own validation before its next preparation. 

**I found no further synchronization change necessary for this measurement boundary.**

## 2. CPU cost and elapsed completion time are now properly separated

The source calls `clock_gettime(CLOCK_THREAD_CPUTIME_ID)` immediately around the traversal, nested inside the monotonic window. Preparation, barrier waiting and validation sit outside both windows. The analyser uses the CPU-duration differences for `w1` and `w4`, while retaining monotonic timestamps for `e4`, overlap and start skew.  

That is the appropriate distinction: Linux defines `CLOCK_THREAD_CPUTIME_ID` as CPU time consumed by the calling thread, rather than elapsed time while it may be descheduled. [man7.org](https://www.man7.org/linux/man-pages/man2/clock_gettime.2.html?utm_source=chatgpt.com)

I also reproduced the synthetic regression test that stretches the candidate’s elapsed windows without changing its CPU durations. The CPU-cost ratio remains unchanged while the elapsed-worker ratio increases. **The implementation is no longer merely relabelling elapsed time as CPU cost.**

The clock-call ordering and retention of unsubtracted measurements are sensible. The remaining issue concerns whether the analyser verifies that it received the *complete calibration evidence*.

## 3. All five previously accepted invalid cases are now rejected

All 34 supplied tests passed independently. In particular:

| Previous counterexample | New result |
|---|---|
| Only one measured pair | Rejected by the round-count check |
| Workers shifted off their declared assignments | Rejected by the expected-record check |
| Candidate row denominators doubled | Rejected by the expected-record check |
| A four-worker step split into separate steps | Rejected by the expected-record check |
| Next preparation inside a preceding traversal | Rejected by the cross-step check |

**Each is rejected before a ratio or investment decision is produced.** I additionally checked a wrong block identity and a worker whose validation extends into its next preparation; both are rejected.

The improvement is structural: the analyser now reconstructs the snake assignment from the supplied block-size vector and checks each expected worker/step/block/row combination. It no longer treats valid-looking metadata and valid-looking walk records as sufficient independently. 

One wording precision: it reconstructs the schedule from **`block_rows` in the metadata**, not by independently decompressing the corpus. That is reasonable here, given the pinned collector and corpus hash. It should not be described as independent verification of the block-size vector itself.

## 4. Failure handling addresses the barrier-hang problem

The explicit failure path writes a failure record and terminates the process. A panic hook routes Rust panics through that path. The recipe preserves the subprocess exit status, runs the analyser after a failed grid, and proceeds to archival.  

The committed consoles show the two important end-to-end cases:

| Injected failure | Recorded outcome |
|---|---|
| Wrong output in W4, compact30, worker 2, block 7 | Exit **3**; failure reported; analysis rejects without figures |
| Pinning failure for W4 worker 1 | Exit **3**; failure reported; analysis rejects without figures |

Those are documented run outcomes, not executions I repeated.  

I would narrow Claude’s phrase **“any crash”** to “explicit harness failures and Rust panics.” A panic hook is not a universal handler for process aborts or external termination. That is a reporting qualification, not a reason to add another failure-handling framework. [doc.rust-lang.org](https://doc.rust-lang.org/stable/std/panic/fn.set_hook.html?utm_source=chatgpt.com)

## 5. The remaining defect: G5 accepts incomplete calibration batches

The collector appears to generate the intended calibration: every configured CPU, 20,000 empty windows, before and after the run.  

**But the analyser does not enforce that structure.**

Its effective completeness check is:

```python
meds = [c["cpu_ns"][1] for k in calib for c in k["cpus"]]
whens = sorted(k.get("when") for k in calib)

if whens != ["after", "before"] or not meds:
    G5 = False
```

It checks the two batch labels and that the combined median list is nonempty. It does **not** require each batch to contain all four expected CPUs, reject duplicate or wrong CPU IDs, or verify the prescribed sample count. 

### A consequential synthetic reproduction

Using entirely **invented calibration values**, I kept the traversal records unchanged and constructed this example:

| Calibration evidence | G5 | Overall result |
|---|---|---|
| Both batches contain all four CPUs; one CPU has a larger empty-window median | Fails: estimated overhead **1.042%** | **INVALID** |
| Delete that CPU’s entry from both batches | Passes: estimated overhead **0.0174%** | **ADVANCE** |

The lower overhead estimate is not a measurement improvement. It results solely from missing evidence.

I also confirmed that an **empty “after” batch**, wrong calibration CPU IDs, and incorrect sample counts can pass.

**This does not show that Claude’s collector actually omitted any calibration data.** Its generation loop looks correct. It shows that the newly added evidence gate would fail to detect an incomplete or misidentified calibration record set—the same class of validation issue we just repaired for the traversal records.

### The repair is small

Before interpreting G5, require exactly one `before` and one `after` batch. In **each** batch, require exactly one entry for every CPU in `meta["cpus"]`, no duplicates or unexpected CPUs, and `n == 20_000`. Validate that the CPU and wall-clock summaries have the expected five finite, nonnegative, ordered quantiles.

Incomplete or malformed calibration evidence should make the run invalid, rather than silently calculating a smaller overhead estimate. Add rejection tests for the cases above.

**No Rust traversal or timing-window change is needed.** This can be fixed entirely in the analyser and its tests, followed by the usual updated source pin.

## 6. Two interpretation limits should remain explicit

### The median-based overhead gate is an estimate, not a mathematical bound

Taking the largest pre/post per-CPU median is a conservative choice among the recorded **typical** costs. It does not bound every clock call or prove that collection perturbs each real traversal by no more than that amount.

I would retain the registered 0.5% screen, raw measurements and labelled sensitivity calculation. Just describe it as a **calibration-based overhead estimate**, rather than a guaranteed maximum perturbation. The recorded quantiles provide additional context; they are not a reason to begin another measurement campaign. 

### The committed dry-run evidence is sufficient to inspect, but not to replay independently

The committed dry-run analysis reports every schedule check passing, with G3 and G4 passing and performance figures suppressed. Its manifest identifies the clean `c0f8d38…` source and checksums the original records.  

However, the raw JSONL files were not committed. **I have not independently replayed the 5,208 real-record schedule checks or rebuilt and redisassembled the Rust binary.** My independent execution here covers the hash-verified Python tests and synthetic checks; my Rust and recipe assessment is source review, supported by the committed run logs.

That distinction should remain in the review record. It does not require another decoder correctness campaign.

## Recommendation to the guardian

**Conditional approval: accept the repaired measurement design and Rust harness, close the previous blockers, and require the small G5 calibration-completeness repair before the final freeze.**

A suitable bounded handoff is:

> The independent review confirms the completion-barrier repair, actual thread-CPU accounting, rejection of the five previous invalid schedules, and the documented failure-path behaviour. Keep the walker and Rust measurement windows unchanged.
>
> Complete G5’s record validation: both calibration batches must contain exactly the four configured CPUs, once each, with 20,000 samples and well-formed quantile summaries. Reject missing, duplicate, misidentified or malformed calibration entries. Add the supplied synthetic regressions, including the case where deleting the expensive CPU’s samples otherwise changes INVALID to ADVANCE.
>
> Describe the median-based overhead screen as an estimate, not a strict bound. Re-pin after this analyser-only repair. Do not change thresholds, add variants or perform architectural work.

The review-evidence.zip[independent evidence bundle](sandbox:/mnt/data/compact30_rev3_review/review-evidence.zip) contains the hash-verified analyser and tests, the 34-test reproduction log, the additional checks and their results. The independent-review\.json[review results are also available separately](sandbox:/mnt/data/compact30_rev3_review/independent-review.json).

**This revision has resolved the major problems. The remaining task is to make the new calibration gate enforce the evidence it already claims to require; after that, I support freezing the unchanged candidate for the guardian’s capture decision.**
`````
