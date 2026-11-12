# External research report 177: the guardian approves one compact30 traversal capture at `c0f8d38`, and the blog handoff (verbatim)

A research input, not a finding of this repository.

- **Relayed:** by the maintainer, 2026-11-02.
- **Source:** the guardian.
- **In reply to:** the repaired harness at `c0f8d38`, with the recipe and run records at
  `1a15542` (PR #134).

**Evidence cutoff:** 2 November 2026. A reading of the revised source, analyser, recipe and
committed dry-run and failure records. The guardian did not rebuild the executable or rerun
its suites.

**This authorised:**
- one i9 capture, `just bench-c30-traversal`, at the pin `c0f8d38` (branch head `1a15542`);
- the blog bundle, as separate docs-only work.

**It was superseded in part by report 179,** which holds a capture that has not yet started
until the G5 calibration-completeness repair (report 178) is done.

It is kept exactly as received (rendered text, cards flattened). The maintainer's note
naming the blog, which the message refers to, follows it.

## The message, as received

`````text
The repairs address the blockers. I approve one compact30 traversal capture at the new pin, `c0f8d38`. This is now the point to obtain performance evidence—not begin another design round.
I checked the revised source, analyser, recipe and committed dry-run/failure records. I have not rebuilt the Rust executable or rerun its correctness suites.
What is now ready
Validation happens after all workers finish traversing. The actual four-worker loop now puts the completion barrier before output checking and result recording. Measurement storage is allocated beforehand. This removes the large validation reads from neighbouring workers’ timed traversals.
CPU cost and elapsed completion time are genuinely separate measurements. The harness reads thread CPU time around the walk and retains monotonic timestamps for synchronization and completion. Barrier waiting and output validation are outside both traversal windows.
The analyser checks the complete schedule before calculating ratios. It enforces the registered round count, block/worker/step assignment, row denominators, placement, barrier ordering and pass order. The regression suite specifically includes the researcher’s five counterexamples and the early-validation case.
The committed evidence demonstrates both acceptance and rejection paths. The dry run passes its schedule, identity and allocation checks; the injected output failure produces an explicit INVALID result with no performance figures. The recipe preserves and archives failures rather than treating them as unsuccessful performance outcomes.
The i9 still has to pass the runtime stability, synchronization and clock-overhead gates. The development-machine dry run cannot establish those in advance.
Run this on the i9
First update and inspect the branch:

```
git fetch origin research/compact30-walker
git switch research/compact30-walker
git pull --ff-only
git rev-parse --short HEAD
git status --short
```

The reviewed branch head is `1a15542`. Untracked earlier results are fine; there should be no unexpected tracked modifications. The branch’s recipe deliberately runs `c0f8d3832b442dde6f2908a1c80e328641bc6e3e` in its own pinned worktree—the two commit identifiers serve different purposes.
Then:

```
just bench-c30-traversal
```

No sudo, no `tee`, and no additional probe. The recipe checks the corpus, builds the pin, runs the analysis tests and machine-code comparison, then packages the capture under:

```
target/c30-traversal-<UTC>.tar.gz
```

Upload that single archive afterward, including if the run fails a gate. Do not change thresholds or automatically rerun a failed capture.
What the result will decide
The question is now concrete:
Does compact30 reduce four-core traversal CPU cost enough to justify developing its real constructor, without exceeding the accepted completion-time regression?
For a valid capture, the registered advance condition requires the upper confidence bound of `w4` to be at most 0.865, and that of `e4` to be at most 1.02. A sufficiently weak result stops this investment; an interval crossing a threshold is inconclusive, not an invitation to add rounds until it passes.
Even an ADVANCE would be a traversal result under the specified table preparation, not a production-decoder speedup. Construction remains excluded, and the candidate’s copied oracle image does not reproduce a future constructor’s cache state. Those limitations are already recorded; they do not require another repair before this screen.
This is also the suitable break for the blog
You can now give your bundle directly to Claude. The experimental code is pinned, and your next action is independent of its editorial work.
Keep that handoff simple:
The compact30 capture is approved at its existing pin. Leave the experimental source, protocol and recipe unchanged. Please work on this blog bundle in a separate docs-only branch or worktree, following its supplied editorial instructions. Do not incorporate unreviewed compact30 performance claims. When I return with the capture archive, verification of that evidence takes priority.
No need to send the bundle here.
So the sequence is now: run the frozen experiment locally; let Claude work on the separate blog; then verify the capture before deciding anything further about compact30.
`````

## The maintainer's note on the blog, as relayed

`````text
by the blog, it is talking about a blog i prepared which i told it about earlier:

> ok so not to interrupt what we're doing here but i had some materials prepared for a blog post on benchmarking MLP vs ILP, based on a chat on twitter between daniel lemire and someone else discussing that. i had the files ready to give claude but i dont wanna interrupt this more important work... claude is busy right now but i have the files (some reports and an instruction on how to tailor them to more 'blog'-like shape) ready for when there is a suitable break in the sequencing. as the guardian i will leave it up to you
`````
