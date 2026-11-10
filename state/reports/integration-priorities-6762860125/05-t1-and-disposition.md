## 5. #334 revision3 — REVISE T1 admission only; no T2 expansion
Reviewed c077cbdc44a77a5e2f8ee76f7785c2f4240e34f6. Scope limited to T1 and prerequisites used by the combined diagnostic. No review/implementation of the frozen T2 extension or separate new staged diagnostic.

All36 synthetic team-screen tests pass after fetching its same-pin parser dependency. The prior wrong-build-label, varying-executable-digest, inconsistent success-error and nonzero ab-child status witnesses are repaired. The96-run matrix, prefix oracles, valid/error distinction, 4KiB/4MiB buffers and compatible-unmodified-base rule are coherent. Expected decoder errors correctly remain exit0 with structured decoder error. These are checker tests, not actual fixture executions.

**Two residual admission failures reproduced on the complete valid fixture:**
- Change only ARTIFACTS.head.sha to a different40-hex commit. Every wrapper still labels the registered head and uses the artifact's binary digest. outputs_check returns[]: it compares wrapper build_sha to delta.head, but never binds artifacts.head.sha to delta.head. Bind both head and base build-step SHA metadata to the delta, before accepting any oracle or arm binary digest.
- Give all lbzip2 rejection records exit_status=null, -9 or124, with timed_out=false and signal=null. Every variant returns[]. The rejection branch merely rejects exit0. It must require a typed, non-boolean normal exit consistent with the stated decoder rejection (the made fixtures specify exit1), and reject missing/signal-derived/supervisor-timeout statuses or inconsistent status fields. A process failure is not evidence of a malformed-stream rejection. Freeze the exact rejection contract; don't accept arbitrary nonzero statuses.

Add these negatives while retaining the baseline and expected-ab-error positives. No new decoder or fixture run is needed to demonstrate the fixes in the checker. Keep build/output/command/input metadata independently produced by the runner; matching self-reported wrappers alone does not establish execution provenance.

The revision3 independent entered/join/built summary design addresses lost zero-work records in principle. The actual #338 checker used by the combined diagnostic still has the per-block roster hole reproduced in item3; resolve that narrow prerequisite rather than extending T2. T1 alone does not prove forbidden-path absence or native engagement. Do not patch an old base to gain harness compatibility, turn the screen merge-blocking without the prospective C6 record, or reopen the failed #334 counted result.

## 6. Deferred
Width third draft (6762087849) deliberately not reviewed. Reopens only under the stated priority condition, not automatically upon this review's publication.

## Overall evidence and outcome
Expected: finish integration prerequisites in priority order. Observed: STEPS needs a corrected estimand/attribution claim; a bounded12×240 qualification programme is named without an ineffective A/A prerequisite; combined accounting is useful but its engagement contract/checker needs repair; #318's source objection closes; T1 retains two small admission holes. No measured production-performance gap closed and no new native performance or mechanism evidence was produced. Next value is these narrow fixes and the existing qualification path, not additional exploration. All old verdicts, Guardian gates and on-target freeze dependencies stand.

Only own state-branch research and one channel REVIEW are published. No implementation, merge, native timing or automation change. The archive-read failure for #318 is explicitly scoped; the source review and other work are complete.
