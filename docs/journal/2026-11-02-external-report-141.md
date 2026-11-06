# External research report 141: the guardian on report 140: five narrow repairs to the walker probe, no i9 run until they are ready (verbatim)

A research input, not a finding of this repository.

- **Relayed:** by the maintainer, 2026-11-02.
- **Source:** the guardian.
- **In reply to:** report 140.

**Evidence cutoff:** 2 November 2026. Instructions: no workstation setting changed, no timed capture, no new experimental variant.

It is kept exactly as received (rendered text, cards flattened, tracking parameters
included).

## The message, as received

`````text
Excellent diagnosis from the researcher. I agree with all three fixes, and I would send this directly to Claude.

This is considerably more useful than another speculative counter explanation: the researcher has traced the failure through the actual launcher, identified a second bug that would probably have appeared after fixing the first, and reproduced the transport behaviour without needing your i9.

### What went wrong

1\. `sudo` stripped the FIFO environment variables

Perf started with counters disabled, but the walker never received `WALKER_CTL` and `WALKER_ACK`, so it never sent `enable`.

2\. Perf's ACK contains a trailing NUL

The current reader accepts the first `ack\n`, but the leftover NUL can corrupt the second acknowledgement. This is a separately identified, not-yet-triggered defect.

3\. Invalid counts appeared as “not multiplexed”

Perf displayed `100.00%` alongside `<not counted>` because the counters never ran. That is an unavailable measurement, not a valid unmultiplexed one.

The researcher's additional observation about the four-core result vector growing inside the measured window is also important. It could contaminate the exact comparison we're trying to make between a minimal walker and the shipped implementation.

## What I'd authorize Claude to do

The repair should remain narrow. No workstation setting changes, no timed captures, and no additional experimental variants.

The independent researcher has identified the cause of the failed `walker-counter-probe`, plus two further issues in PR #131. Please use its complete review as the basis for the following repairs.

**1. Repair the sudo environment handoff.**

The probe at `a245a455` starts `sudo perf` with `-D -1`. Perf receives its FIFO paths from command-line arguments, but the walker receives `WALKER_CTL` and `WALKER_ACK` from environment variables that are stripped by sudo. The observed `controlled: false`, `Events disabled`, no `Events enabled`, and `<not counted>` for every counter are consistent with that failure.

Pass the variables explicitly to the workload after perf's `--`, using an argument-vector construction such as:

`sudo perf stat ... -- env WALKER_CTL=... WALKER_ACK=... taskset ... walker_ladder ...`

Apply the same correction to `WALKER_BARRIER` in the four-worker path. Make missing control settings a hard error whenever the run requires perf control.

**2. Repair ACK framing.**

The researcher checked perf 6.8.12 and found that its FIFO acknowledgement is emitted as `ack\n\0`. Reading only through the newline leaves a NUL before the next acknowledgement. Parse consecutive ACKs correctly, without accepting malformed acknowledgements.

Regression-test two successive commands using both `ack\n` and `ack\n\0`.

**3. Repair probe validity.**

Report `<not counted>` plus zero running time as invalid, regardless of a displayed `100.00%`. Multiplexing should be labelled unassessed when no valid count exists.

Validate both the 20-walk interval and its corresponding zero-walk control. Preserve raw perf output and make shared control failures explicit rather than displaying thirty misleading availability results.

**4. Add transport-level regression tests.**

Test the real command constructor through an environment-filtering wrapper; verify FIFO and barrier settings reach the child. Include the exact failed perf CSV shape and a good measured run paired with a failed control-only run.

**5. Fix the four-core result-vector growth before a full capture.**

The deadline-based copies are launched with `reps=1` but can push many results into a vector reserved for only two. Prevent allocation growth inside measured windows, and test the correction.

Preserve the failed probe and original records unchanged. Use the VM and stand-in perf for validation. Do not disable the NMI watchdog or change workstation policy. Do not launch another i9 probe or any timed walker capture.

Return the corrected source pin, passing tests, and one revised probe command for review.

### My assessment of the researcher

This is exactly the role we wanted the new Pro researcher to fill. The analysis goes beyond identifying a symptom: it establishes a plausible failure chain, distinguishes what the archive proves from what was reproduced separately, and catches the next failure before it happens.

It also reinforces an important principle for our investigation: benchmark infrastructure must itself be treated as code requiring correctness tests. Otherwise increasingly sophisticated performance counters can produce increasingly convincing-looking nonsense.

The current architecture work remains unchanged: FOR-14's frozen implementation is stopped, the packed-token `k`/`j` candidates are compiled and correctness-tested but untimed, and the walker-ladder capture is still unauthorised.

I would have Claude make these repairs now. No need for another i9 command until the corrected probe is ready.
`````
