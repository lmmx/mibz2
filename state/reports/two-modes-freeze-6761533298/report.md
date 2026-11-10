# PR318: one conditioning row, then every assigned treatment

Inbox [6761533298](https://github.com/lmmx/mibz2/pull/162#issuecomment-6761533298), re-review of **7af2b970cf97b5414a766359c8363c419fd45124**. Instructions: docs/team at **a2da49638ac54d7763de1c167bd054f67ffdc763**.

**REVISE the gap admission before freeze.** The previous witnesses are repaired and all 86 source tests pass locally. One residual exemption allows treatment records to disappear from the primary analysis while the complete attempt remains VALID. Keep the experiment; this needs a small checker correction, not another design.

## R1. Conditioning is a position, not an exemption attached to any label

At bench/two_modes.py:1185, the judge requires a conditioning child first; at 1187 it counts all remaining records as treatments. But the loop at 1194 skips the schema/index/registered-slot check for **every** record labelled C. The G/Z interval bounds also apply by label. Finally gaps_reading excludes every C record.

Using the reviewed source's complete synthetic replay fixture and actual judge:

| Mutation | Exit / label / problems | Primary counts |
|---|---|---|
| none | 0 / VALID / [] | G30, Z30 |
| change only first treatment's condition from G to C | 0 / VALID / [] | G29, Z30 |
| change every treatment's condition to C | 0 / VALID / [] | G0, Z0 |

The primary outcome correctly becomes INSUFFICIENT; **this does not fabricate a positive gap/slow association**. The defect is narrower: a protocol-broken attempt is admitted VALID instead of INVALID. Missing assigned treatments are not valid observations lacking a slow class. The fixed order and counts are part of admission, not merely a display floor.

Require C exactly at record 0 with the conditioning schema/index/endpoints; every subsequent record must have its own expected index, schema and fixed G/Z slot. Reject a later C regardless of whether its child succeeded. Check the full 30/30 treatment counts (or the exact six-slot DRY prefix) at admission. Add both reproduced witnesses as negative tests. This also closes the conditioning-label escape from the timing-policy checks.

The normal run_gaps driver does emit one C followed by the assigned slots; this is a replay-validator defect, not evidence of a corrupt native run.

## The two requested rulings

**1. No new full DRY solely to write its results into §5.** I fetched the GitHub comparison e5885772...7af2b970: only the registration's §5 changes, recording the completed DRY (24 additions, 3 deletions). Executable recipe, driver, probe and other inputs do not change in that comparison. Requiring a new DRY every time its record is written would create a self-referential loop.

Keep the original DRY identity at e5885772 and its original source digests. Attach the exact comparison as the explanation for carrying plumbing evidence forward; freeze the new head with its own honest source digests. Do not rewrite the old bundle to claim it ran at the new head or relax exact identity admission. This is a scope-limited equivalence judgement for executable plumbing, not evidence that the new commit itself ran.

The reported bundle SHA-256 is b01d8dbfd2129687336dca44c4d53f887c26604a09d259621aa58e9e222b3164. I verified the committed DRY record and source delta, **not the archive's bytes or the reported .999 coverage**: no immutable downloadable bundle reference was supplied in this assignment/record. Publish that object reference and check its digest and original judgement before treating the raw DRY as independently verified.

For the new admission-only repair in R1, rerun the fixture suite and these counterexamples, then check the retained valid DRY records with the repaired validator under explicit cross-version provenance. That is checker regression evidence; do not relabel the old bundle as a native run of the repaired head. Another full DRY is necessary if the repair changes execution, collection or identity plumbing in a way the retained evidence does not exercise. Recording §5 alone is not such a change.

**2. Synthetic pass 2 is not native SMT validation; disclose it, but do not add a new experiment merely for this review.** The full DRY exercised the paths available on its host; pass 2's sibling placement and native collector interaction remain unexercised. The synthetic tests establish expected record handling, not host scheduling, affinity or concurrent-probe operation. The existing bounded, diagnostic-only single attempt can retain that limitation once R1 is closed. Its first target pass 2 must pass the registered native placement, coverage, containment, status and identity checks. Failure is retained as INVALID/refusal under the frozen contract; it is not permission to retry pass 2 selectively or reinterpret pass 1 as a completed two-pass attempt. No production qualification follows from either pass.

## Accepted repairs and the operational sum

Source and 86 passing tests support the specific previous repairs: finite positive timing, nonnegative CPU and Z intervals; ordered retained launch/reap/spawn endpoints; actual initial conditioning child; exact rest/deadline fields; 1–4 slow observations explicitly insufficient; and subwindow support assessed within each third. The assigned reap-to-launch policy now matches the measured endpoints. These close the earlier witnesses, not every possible malformed record.

The code computes the operational total correctly:

    supervised calls including grace + both cleanup phases = 8190 s
    six collectors                                        =  210 s
    fixed rest                                            =   60 s
    total                                                 = 8460 s

Correct the registration's stale **7965 s** supervised subtotal to **8190 s**. The total does not need increasing on that account. This remains an operational ceiling excluding the explicitly named unsupervised bookkeeping; do not call it a strict wall-time guarantee. I checked current CI metadata: 19 successes, two skipped jobs, no failed check.

## Evidence, limits and next move

Reproduce by fetching the paths in source-manifest.json at the exact reviewed head, preserving directories, and placing probe.py beside bench/. Run python bench/test_two_modes.py, then python probe.py. The probe imports the source's synthetic Recheck fixture, invokes the actual CLI judge and asserts the reported false admissions. It builds and times no decoder. probe-results.json records its output.

Expected: previous repairs permit a bounded freeze. Observed: those repairs pass, with one remaining admission escape and a documentation subtotal error. **No native performance evidence or mechanism identification was added.** Correct the narrow exemption, demonstrate INVALID for both witnesses and VALID for the intact fixture/retained DRY records, preserve old/new identity provenance, then freeze the diagnostic. Native SMT remains an explicitly untested path, not an undisclosed validation claim.

Migration guards passed before receipt; channel routing audit returned no new misroutes. Research only: own team-state/researcher branch and one REVIEW on #145. No implementation, merge, workstation run or automation change.
