# PR317 delta: PROCEED within the reviewed diagnostic scope

Inbox 6757686244; previous REVIEW 6757069602 and Guardian D079. Reviewed delta from `c2f461491b97a563ca3f8669b5c3d7e14eeff07f` to ready head `efc1bd88c73b07d917617eddfdf008dffc691e3d`. Measured freeze `243051248de80ebc88066c8f2db3b3852c47d1f8`; freeze-pointer commit `0b4d3c17277ef1b3b97410cdb96f23be3a093483`. Specs/rules pin `a2da49638ac54d7763de1c167bd054f67ffdc763`.

**PROCEED on the requested delta.** The changes close the three concrete readiness defects: M28's untested failed call, the supervisor's unbounded/error-discarding cleanup path, and VALID/DRY surviving failed finalisation. Earlier acceptances carry over only for untouched work. This review does not lift D079 on the Guardian's behalf, authorise agent timing, queue/tag the recipe, or qualify production performance.

## Independent checks and findings

### CPU-time failure path

The private `cpu_micros_by(GetRusage)` contains the real zero-initialised buffer, call, return-code check and timeval validation. Production `cpu_micros` selects libc through OnceLock; the injection selector selects the failing stand-in. The tests do not merely copy the predicate. The new stand-in returns -1 without writing, so removing `rc != 0` leaves zero timevals, which satisfy the unchanged timeval checks and yield 0. Both should-panic tests would then fail: the mutation has an explicit distinguishing input.

Success is separately anchored: (1 second + 500 microseconds) + (2 seconds + 250 microseconds) = **3,000,750 microseconds**. This also distinguishes the seconds-times-1000 mutant. Negative seconds, out-of-range microseconds, and unknown selectors each reach their intended rejection. The function-pointer seam adds no new unconstrained buffer writer: the private call sites select libc or the file's stand-ins, with the existing ABI/layout obligations unchanged.

The recipe refuses a nonempty inherited PAR_SCAN_INJECT before measured work, unsets it otherwise, and scopes the synthetic failure to one DRY cell. It checks nonzero exit, no emitted cell, the expected panic message, and INVALID through the same `knee_outcome` used by measured cells. The injection itself remains visibly marked as nonmeasurement.

Evidence boundary: source argument independently re-derived. Ready-head Rust CI job [129595407618](https://github.com/lmmx/mibz2/actions/runs/42745168254/job/129595407618) confirms all six new named tests pass natively. Names beginning `miri_` in this job are not proof of execution under Miri. The reported Miri and native mutation runs, and full DRY at 0b4d3c17, were not rerun here; their scope remains reported evidence. No Rust compiler is available in this local runtime. That limitation does not obscure the concrete M28 distinction above.

### Supervisor cleanup and reaping

All paths now funnel through `finish`. If cleanup failed, it returns 125 **before invoking wait**. Otherwise it calls `wait(timeout=REAP)`; TimeoutExpired also yields 125, taking precedence over both 124 and 143. Normal successful exit still returns the child's status; clean signal handling retains 143.

Independent `probe_finish.py` exercises all three requested outcomes (normal, deadline, signal) with failed cleanup and with an unreapable child. All six return 125; failed cleanup makes **zero wait calls**, and the other cases pass timeout=10.0. This is a controlled mock, not a claim to create a real unkillable process.

All ten supplied supervisor test functions also passed locally, including live child/grandchild cases. Because pytest is unavailable, `run_tests.py` invokes the actual source functions using TemporaryDirectory and unittest.mock fixture substitutes, with signal-handler restoration. This is not presented as a pytest invocation.

The bounded-refusal promise is appropriate. Kernel-blocked tasks may survive; descendants creating new sessions are explicitly outside the model. There is a small wording precision: `stop_group` may spend up to REAP checking group emptiness, and `finish` then independently permits REAP for wait. The code establishes **separate bounded phases**, not a universal single ten-second budget measured from KILL. Interpret the documentation on that basis; a future wording cleanup should say so. This does not restore the original unbounded-wait defect. Blocking kernel/filesystem operations are not a hard real-time guarantee.

### Finalisation

The new helper writes the proposed outcome, runs the bundler under the supervisor, and on nonzero status writes INVALID plus FINALISATION-FAILED.txt, sets any existing archive/temp archive aside under a non-.tar.gz suffix, and returns 2. The caller reads the revised outcome and exits 2. The helper is included among measured inputs.

All four supplied finalisation tests passed locally: success, timeout from both VALID and DRY, archive-created-before-timeout, and bundler failure. The archive-created case verifies the ordinary archive is gone and submit.sh refuses the failed suffix. These checks use the actual helper and bundle/submit scripts. They cover the reported stale-outcome defect.

This is not a promise that failed storage writes or tasks surviving KILL can be made harmless by renaming a path. On exit125 no measured work may continue; retain the directory and inspect the survivors. The return-code failure and submit refusal are essential, not merely the human-readable marker. No new universal storage/signal guarantee is inferred.

### Identity, CI and scope

GitHub's frozen-to-ready comparison lists exactly:
- `bench/par-scan.frozen` (the freeze pointer);
- `tests/pipelined.rs` (corpora(1) to corpora(2) for the middle-CRC fixture).

Neither belongs to the enumerated measured-input list. That independently corroborates measured identity through the ready head. The test change increases the corpus for a test needing at least four blocks; no decoder source change appears in the delta.

Ready-head check summaries show successful applicable checks, with Release and Publish crate intentionally skipped. The Rust log was read for native test results; this is not an audit of every CI job or an independent rerun of Miri/fuzz.

Registration §5/§10 now distinguish the **18,600-second sum of step deadlines** from expected or whole-run duration, and observed RSS from enforced memory caps. The scope is honest: small queries/cleanup remain outside that sum, and the fixed-input diagnostic is not licensed for arbitrary files.

## Consequence

Expected: close narrow readiness failures without reopening untouched scientific/safety acceptances. Achieved: real failed-call distinguishability, no wait after failed cleanup, bounded remaining reap, consistent error status, and no ordinary submit candidate after tested bundling failure. No decoder speed or machine mechanism has been measured.

The next useful step is the registered diagnostic once the Guardian rules on D079 and the Coordinator's existing readiness requirements are met. Another speculative redesign would add little information here. This PROCEED is specific to the reviewed exact delta and does not decide PR318/PR316 or production adoption.

## Reproduction and provenance

- `run_tests.py` executes the 14 supplied Python/shell test functions. `replayed-tests.json` records all 14 PASS; `replayed-tests.stderr` preserves supervisor diagnostics.
- `probe_finish.py` and `probe-results.json` record the independent six-case control-flow checks.
- Reproduce by placing the ready-head bench/bounded.py, par_scan_finish.sh, test_bounded.py, test_par_scan_finish.py, bundle.sh and submit.sh under bench/ beside these scripts, then running `python run_tests.py` and `python probe_finish.py`.
- Source identities, comparison lists, CI summary and selected native-test log lines are in evidence.json. No workstation timing was performed.
- Fresh migration head and manifest digest matched before receipt. Pinned coord.audit on fresh channel snapshots returned no routing violations. Inbox 6757210413 was already finished and was not replayed. QUEUED 6757715565 is not this assignment and was not started.
