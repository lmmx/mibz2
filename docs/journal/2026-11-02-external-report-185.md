# External research report 185: the guardian approves the final registered capture with the spinning start barrier (verbatim)

A research input, not a finding of this repository.

- **Relayed:** by the maintainer, 2026-11-02.
- **Source:** unattributed in the relay (the guardian's register).
- **In reply to:** protocol revision 5 (report 184): source pin `f1dafa3`, recipe at branch
  head `5e2888f` (PR #134).

**Evidence cutoff:** 2 November 2026. A review of the implementation, the pinned recipe
and the recorded evidence in PR #134. The guardian notes that it did not repeat the stress
tests, negative controls or dry runs.

**This authorises:** one capture, `just bench-c30-traversal`, on the workstation at branch
head `5e2888f`. Upload the archive whether the run is valid or invalid.

**The stop rule:**
- **If G2 passes:** the complete registered decision is evaluated, on w4 and e4.
- **If G2 fails:** the capture is INVALID, and this comparison stops, with no further
  barrier modification.
- **Attempt 3 stays INVALID** whatever happens next.

It is kept exactly as received.

## The message, as received

`````text
Approved. The revised spin barrier is ready for one final registered i9 capture.
I inspected the implementation, the pinned recipe and the source changes in [PR #134](https://github.com/lmmx/mibz2/pull/134?utm_source=chatgpt.com). The repair addresses the specific synchronization defect found in attempt 3 without changing compact30's traversal algorithm or the experimental decision rules.
The implementation has the properties we required:

* Reusable synchronization: an atomic generation counter prevents one step's release from being mistaken for another's.
* No spinning during traversal: the existing completion barrier remains, and every worker must finish its current step before moving to the next.
* Unmodified timing boundaries: traversal timestamps are still taken after the start barrier releases.
* Unchanged validity rules: G2 still requires start skew at or below 2%. The CPU-cost and completion-time thresholds remain fixed.
* Failure handling: the existing process-wide failure path remains in place.

Claude also reports successful stress testing across 300,000 steps, negative controls and untimed dry runs. I reviewed the code and recorded evidence rather than independently repeating those executions.
Run the approved capture
From `~/dev/mibz2`:

```
git switch research/compact30-walker
git pull --ff-only
git rev-parse --short HEAD
just bench-c30-traversal
```

The branch head should be `5e2888f`. The recipe runs the frozen source pin `f1dafa3` using Rust 1.97.0. Your installed toolchain and existing worktree are handled automatically.
Upload the resulting `target/c30-traversal-<UTC>.tar.gz`, whether the analysis declares it valid or invalid.
The stopping rule matters
This is one authorized repeat with a corrected measurement design, not permission to keep adjusting the experiment until compact30 passes.
If G2 passes, we evaluate the complete registered decision: four-core CPU cost `w4` and lockstep completion time `e4`.
If G2 fails again, we preserve the evidence, classify the capture INVALID and stop this particular comparison without further barrier modifications.
Attempt 3 remains INVALID regardless of what happens next. Its approximately 15.8% lower four-core CPU cost is an encouraging exploratory observation, not a substitute for a valid result.
We've now addressed a demonstrated synchronization defect rather than adjusting the acceptance threshold. The next result should settle whether compact30 earns direct-constructor work under the agreed investment policy.
`````
