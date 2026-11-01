# Dispatch causality, amendment 1: the scheduler evidence enforced (validation only)

**Status:** registered before any i9 capture, at pin `48f8cda`. It was prompted by the
independent researcher's review and the maintainer's instruction (reports 110–112).

**What it leaves unchanged:**
- the hypotheses;
- the classes and thresholds: 5% materiality, 80% repeatability, 90% coverage, 99%
  scheduler state, 3% perturbation, 20 µs alignment;
- the schedule, the tiers and the expansion rule;
- the decoder. `src/` is untouched, and C6 still holds.

## 1. The summary's validity omitted the scheduler checks (the researcher's finding: confirmed)

**The defect:**
- The expansion check (`quality`) required, of each of the first three w = 4 runs:
  - every TID seen by the kernel (`threads_in_sched`);
  - scheduler state known for 99% of queued-work nondecoding time (`sched_coverage`).
- The final `summary()` checked only coverage, perturbation, alignment and completeness.
- So a run added by the expansion that failed either check could still have reached an
  apparently valid decision. §6 requires the checks on every interpreted run.

**The correction:**
- One function, `run_checks`, now serves both the expansion and the decision.
- The decision requires, of every w = 4 run, the expansion's included:
  `threads_in_sched_all`, `sched_coverage_all` and, from §2 below, `trace_complete_all`.
- **With no w = 4 runs, the evidence is invalid** (`runs_present`). Before, `all()` over an
  empty list read as valid, and the decision came out as an ordinary negative.
- The summary gives each run's checks and trace problems, and an explicit `outcome`:
  - `promote`;
  - `do not proceed`;
  - `invalid evidence`, which is never conflated with the negative result.

## 2. Trace loss and incomplete scheduling streams were not detected (verified, and worse than suspected)

The researcher asked whether lost kernel events could vanish in the parser. Checked on the
preparation VM:

**Loss records were invisible.**
- With `perf record -m 1`, perf lost 25,095 samples in 272 loss records.
- `perf script` without `--show-lost-events` prints nothing for them, so the parser could
  not have known.
- **The correction:**
  - The capture now scripts with `--show-lost-events`.
  - It also keeps `perf report --stats` of the same data, where perf attributes each loss
    record's samples to its event. On the VM the per-event sums matched the loss records
    exactly.
  - The trace is **unsound** (`trace.ok = false`) if any of these hold:
    - scheduler or anchor events are lost;
    - the script's and the statistics' loss counts disagree;
    - the statistics are missing;
    - any line is unreadable.
  - **Futex loss** marks only the futex mapping (`futex_ok`).
  - **`power:cpu_idle`** is recorded but feeds nothing, so its loss is reported and is
    irrelevant to the decision.

**Lines were dropped.**
- When perf cannot resolve an event's sample tid, it prints `:-1 -1`. The parser's
  pattern rejected those lines.
- Every VM capture dropped 6 `sched_switch` lines this way: our threads' exit switches,
  after their `ThreadEnd`, so the dry run's classifications were unaffected.
- **The correction:**
  - These events are kept, since their payload names both tasks, and counted.
  - Any line still unreadable is counted, and makes the trace unsound.

**The VM's kernel does not emit switches out of idle on CPUs 1–3.** It is a 6.18
Firecracker guest. Shown three independent ways:
- in every capture;
- in perf's raw dump: no sample at all from the idle task on CPUs 1–3;
- in ftrace: on CPUs 1 and 3, 3,061 and 4,080 switches into idle and none out.

**The consequence:**
- A thread switched in from idle on those CPUs looked to the reconstruction as woken but
  never running. Its running time was counted as **runnable**: class D2, scheduling.
- On the dry run's w = 4 capture, 19 ms of the 77 ms "runnable" was this artifact.

**The correction, in two parts:**
- **The stream's own consistency.** If one of our threads is switched out while not known
  to be running, or switched in while running, the interval since its last transition
  becomes **unknown** (`unresolved.sched`), never runnable.
  - These are counted as `missed_switch_in` and `missed_switch_out`. Their effect reaches
    the registered 99% `sched_coverage` gate.
  - On the VM capture, `sched_coverage` falls from 1.000 to 0.860 and coverage from
    0.943 to 0.803.
- **Idle pairing.** On every allocated CPU, switches into idle and out of it must pair,
  within one for the capture's edges. A run that fails is unsound.
  - The runner now probes this before any capture, with a short uninstrumented w = 1
    decode under `sched_switch`.
  - A kernel that fails stops a real run, and the failure is recorded in
    `stream-probe.json`. A dry run continues, and every one of its runs is refused.

## 3. Reported apart, never a criterion

**`receiver_associated_exploratory`** is the registered receiver obstruction plus the
runnable time in the receiver mutex's wait phase (woken from it, or preempted in it, not
yet running), as a share of aggregate worker time.
- It is the maintainer's and the researcher's R_associated.
- It cannot promote MPMC. The D1 rule reads R_direct alone.

## 4. Tests (`bench/test_dispatch.py`, 24 cases; run by the runner before any capture)

**Parsing and statistics:**
- loss records are counted;
- unresolved-tid events are kept;
- unreadable lines are counted;
- the per-event loss is read, and missing statistics are detected.

**Reconstruction:**
- a complete stream gives R/Q/S;
- a missed switch-in leaves its interval unknown;
- a missed switch-out does the same;
- a missing wakeup makes its sleep unknown;
- unpaired idle switches are detected.

**The trace's soundness:**
- a clean trace is sound;
- scheduler loss is unsound;
- futex-only loss spares the scheduler;
- loss the script never printed, missing statistics and unreadable lines are each
  unsound.

**The summary:**
- complete evidence reaches `do not proceed` (R = 1%) and `promote` (R = 6%);
- each of these gives `invalid evidence`, naming the check: the eighth run's scheduler
  coverage at 95%, a thread unseen by the kernel, trace loss on the tenth run, and no runs;
- the associated share at 11% with R = 1% stays `do not proceed`.

**The expansion:**
- complete runs expand;
- trace loss stops it (exit 3);
- no runs never expand.

**A real capture:** the VM's tier-B w = 4 dry-run capture (`bench/fixtures/dispatch/`) is
refused for idle pairing on CPUs 1–3 and scheduler coverage below 99%. Three of them
summarise to `invalid evidence`.
