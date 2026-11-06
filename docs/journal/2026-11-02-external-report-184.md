# External research report 184: the guardian authorises one spinning start-barrier repair (verbatim)

A research input, not a finding of this repository.

- **Relayed:** by the maintainer, 2026-11-02.
- **Source:** unattributed in the relay (the guardian's register).
- **In reply to:** the G2 audit of the third i9 capture (report 183;
  `i9-attempt-3/g2-audit.md` on `research/compact30-walker`, PR #134).

**Evidence cutoff:** 2 November 2026. The PR journal's account of the third attempt and its
INVALID classification. The guardian notes that it did not independently reproduce the
wake-up decomposition from the raw archive.

**This authorises:** one narrowly scoped synchronisation repair.
- **The change:** only the W4 start barrier is replaced, by a correct four-participant
  spinning barrier. It must be reusable across steps and rounds, with no generation or ABA
  problem.
- **Where it may spin:** threads spin only before a traversal, never during a peer's timed
  walk.
- **Unchanged:**
  - the completion barrier and the two-barrier ordering;
  - the walker and the table preparation;
  - the timing windows and the timestamps, so that G2 measures the actual post-release
    skew;
  - the 20 paired rounds;
  - G2's 2% threshold, and every other gate and rule.
- **What must be demonstrated:**
  - correctness for an intentionally uneven arrival schedule;
  - correctness for the partial final step;
  - coherent termination under an injected output or pinning failure.
- **The protocol update:**
  - the reason for the change;
  - a registered prediction: start skew falls substantially from attempt 3, and meets the
    unchanged 2% G2 in every measured arm-round;
  - the stop rule: if G2 fails again, stop rather than iterate on barrier
    implementations.
- **To be returned:** the new pin, the tests, the untimed VM dry run and the exact proposed
  i9 recipe, for approval.

**It does not authorise:**
- a workstation run;
- reclassifying attempt 3, which stays INVALID;
- using its ratios to change thresholds;
- post-hoc adjustment, skipped steps, a threshold relaxation or a new performance arm.

It is kept exactly as received.

## The message, as received

`````text
Claude’s diagnosis is persuasive, and I support one narrowly controlled attempt to fix the synchronization problem. I would not abandon compact30 on this evidence, nor accept the invalid run retroactively.
The key distinction is now clear: the walker executed correctly, but the experiment did not achieve the simultaneous starts its own protocol required.
Claude’s audit also identifies why that matters. The four-core CPU-cost ratio is remarkably consistent across arm orders—approximately 0.841 versus 0.843—while completion time varies substantially with order, 0.832 versus 0.882. That is consistent with delayed worker wake-ups affecting elapsed completion time more than consumed CPU time.
The measured 15.8% four-core CPU reduction is still exploratory, but it is a substantial enough signal to justify repairing this specific measurement defect.
My decision: authorize the repair, not yet another i9 run
A spinning start barrier is a reasonable response to Claude’s diagnosis. It keeps the participating threads runnable instead of allowing the waiting workers to sleep before each short traversal.
But it changes how the measurement harness uses the CPU. That means it needs a new frozen capture, rather than reinterpreting the existing one.
There are three safeguards I want:

* Preserve the completion barrier. Workers must still finish all timed traversals before validation begins. A spinning start barrier must not compromise that separation.
* Keep spinning outside the measurement windows. The clock readings must occur only after the barrier releases. Record the start skew exactly as before; don't manufacture simultaneous timestamps.
* Prevent benchmark contamination. Waiting threads must spin only until the final participant arrives, not while another worker is traversing. The partial final step and failure paths must continue working.

I would also test the implementation on deliberately uneven synthetic schedules and ensure the existing schedule checks still reject the previous violations.
The specific prediction is valuable: if sleeping-thread wake-up accounts for the start skew, a spin barrier should substantially reduce G2's measured skew, particularly the first-arm penalty. Keep the registered 2% gate unchanged.
Send Claude this
I accept the archive-based G2 diagnosis and authorize one narrowly scoped synchronization repair.
Preserve attempt 3 as INVALID. Its descriptive w1, w4 and e4 ratios are not an approved result and must not be used to change the thresholds.
Replace only the four-worker start barrier with a correct four-participant spinning synchronization mechanism. Leave the completion barrier, walker, table preparation, timing windows, 20 paired rounds, G2 threshold, investment rules and other measurement gates unchanged.
The new start barrier must be reusable across steps and rounds, with no generation/ABA problem. Threads may spin only before traversal, never during a peer's timed walk. Maintain the two-barrier ordering that keeps validation and next-step preparation outside every traversal.
Demonstrate correctness for an intentionally uneven arrival schedule and for the partial final step. Test that an injected output or pinning failure still terminates coherently, rather than stranding threads.
Preserve the timestamp instrumentation so G2 measures actual post-release start skew. Add no post-hoc adjustment, skipped steps, threshold relaxation or new performance arm.
Update the protocol with the specific reason for the change and preregister this prediction: start skew should fall substantially relative to attempt 3 and meet the unchanged 2% G2 requirement in every measured arm-round. If the newly registered capture fails G2 again, stop rather than iterate on barrier implementations.
Return the new pin, tests, untimed VM dry-run and exact proposed i9 recipe for approval. No workstation run is authorized yet.
One important qualification: a spin barrier is not guaranteed to fix every scheduling delay. A thread can still be descheduled immediately after leaving it. That's why G2 remains necessary, and why this should be one bounded attempt rather than an open-ended synchronization-tuning project.
I have checked the current PR journal's description of attempt 3 and its INVALID classification. I have not independently reproduced Claude’s full wake-up decomposition from the raw archive during this response; my endorsement of the mechanism is therefore conditional on its reported audit and the existing timestamp evidence.
Nothing more needs doing on your i9 right now. Claude should implement and validate this narrow change; then we'll review its frozen recipe before deciding on the next capture.
`````
