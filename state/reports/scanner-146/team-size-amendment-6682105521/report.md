# Team-size amendment: buffering repaired; helper tracing can follow

Researcher · 2026-11-06 · [inbox 6682105521](https://github.com/lmmx/mibz2/pull/162#issuecomment-6682105521)

Reviewed commit: `6f3183d96f2e8fc1d59751e1279be062fc09d897` (PR #219). Instruction pin: `0479b380f5d6309f3fd0f23e0ad6dfade4b7a5c0`. This follows [the original review](https://github.com/lmmx/mibz2/pull/145#issuecomment-6682038133); it is a source review, not a new experiment or implementation.

## Answers

**Yes: the new teamK arms close the specific matched-window confound. No: helper-side tracing need not precede the i9 throughput screen.** The screen can answer whether the complete configuration is useful without identifying its microarchitectural cause. The added occupancy pass is useful for choosing a follow-up, provided its labels describe what it actually measures. Do not let a search for a complete causal explanation indefinitely postpone the screen.

## 1. Window closure verified in source

[ab.rs](https://github.com/lmmx/mibz2/blob/6f3183d96f2e8fc1d59751e1279be062fc09d897/examples/ab.rs#L448) routes team2/3/4 through the same explicit team-plan machinery as coopK, but supplies `SLOTS_PER_THREAD * plan.workers()`. The older coopK arms retain two slots per team. [read.rs](https://github.com/lmmx/mibz2/blob/6f3183d96f2e8fc1d59751e1279be062fc09d897/src/read.rs#L87) and [parallel.rs](https://github.com/lmmx/mibz2/blob/6f3183d96f2e8fc1d59751e1279be062fc09d897/src/parallel.rs#L532) confirm that the default requests four slots per requested thread and rescales by team count.

| Arm on the registered i9 plan | Teams × k | Actual window |
|---|---:|---:|
| mibz2@8 | 4 × 2 | 4 × 8 × 4 / 8 = 16 |
| mibz2-team2@8 | 4 × 2 | 4 × 4 = 16 |
| mibz2-team3@12 | 4 × 3 | 4 × 4 = 16 |
| mibz2-team4@16 | 4 × 4 | 4 × 4 = 16 |

The [.just recipe](https://github.com/lmmx/mibz2/blob/6f3183d96f2e8fc1d59751e1279be062fc09d897/.just/bench.just) uses the teamK arms for timings and counters. [occupancy.rs child](https://github.com/lmmx/mibz2/blob/6f3183d96f2e8fc1d59751e1279be062fc09d897/examples/occupancy.rs#L130) rescales its default `4*T` slots by teams/T, giving 16 at each of the three forced four-team plans too.

Thus team2@8 is now a legitimate same-plan, same-window control. Equal results are expected, not guaranteed: construction through different entry points and instrumentation can still introduce differences. With different team counts, total window capacity still changes as four slots per team; those comparisons remain complete plan comparisons. For fixed four-team comparisons, the window is constant. K-dependent segmentation remains explicitly part of the intervention.

The VM check reported by the Coordinator is consistent with this source reconstruction; I did not independently rerun it.

## 2. The occupancy pass is useful, but three causal labels need narrowing

**Known allowed CPUs, not known per-thread placement.** The instrument constructs a CPU list from the sibling map and launches the whole child under `taskset -c` ([placement](https://github.com/lmmx/mibz2/blob/6f3183d96f2e8fc1d59751e1279be062fc09d897/examples/occupancy.rs#L1708), [launch](https://github.com/lmmx/mibz2/blob/6f3183d96f2e8fc1d59751e1279be062fc09d897/examples/occupancy.rs#L995)). It does not bind each leader/helper to an individual CPU or record each thread's execution placement. Scheduler choice and migration remain inside the mask; scanner, feeder and reader share it.

Assuming the full ten-core/twenty-logical-CPU i9 topology is allowed:

| Team threads / logical CPUs in mask | spread physical cores | packed physical cores |
|---:|---:|---:|
| 8 | 8 | 4 |
| 12 | 10 | 6 |
| 16 | 10 | 8 |

This is a valid whole-process topology intervention at fixed plans. It changes available physical resources, cache placement and support-thread contention together. It does not uniquely measure helper sibling cost, nor distinguish that from shared-service capacity. At 12 and 16 threads even spread permits siblings. Describe the result as sensitivity to the allowed-CPU topology. A later per-thread assignment/observation would be needed for stronger attribution.

**InvertStart–InvertEnd is the whole inversion call, not exclusively the shared build/walk.** [decode_block](https://github.com/lmmx/mibz2/blob/6f3183d96f2e8fc1d59751e1279be062fc09d897/src/lib.rs#L331) brackets the call to `invert_checkpointed`; [Cooperative::invert_checkpointed](https://github.com/lmmx/mibz2/blob/6f3183d96f2e8fc1d59751e1279be062fc09d897/src/bwt.rs#L231) performs `self.seg.assemble(out)` on the leader before returning. The interval includes serial assembly, setup and synchronization as well as cooperative work. Therefore:

- decode span minus inversion span is elapsed time **outside the inversion call**;
- it is not all leader-only work, because leader-only assembly lies inside inversion;
- neither duration is automatically useful CPU service: descheduling and waiting contribute.

This limits the inference if inversion stays dominant. It does not invalidate the trace or demand immediate instrumentation. A separate assembly boundary is a cheap conceptual next discriminator if serial work inside inversion becomes the question.

**A job-wait span is not automatically scanner starvation.** [worker](https://github.com/lmmx/mibz2/blob/6f3183d96f2e8fc1d59751e1279be062fc09d897/src/parallel.rs#L421) marks waiting on a shared receiver lock as well as receiving. The instrument already distinguishes idle coincident with scanner input/read/slot waits and queued-job handoff. Use that decomposition. Full-window backpressure, lock/handoff delay and a slow reader must not be called a slow scanner simply because a worker is idle.

The instrument's application queues also do not measure line-fill or uncore request queues. The registration's proposed “queue occupancy per request” remains a future hardware diagnostic, not a field furnished by occupancy.rs.

## 3. When helper-side tracing becomes worth adding

Proceed with the matched-window screen. Keep these conditional follow-ups:

- **Candidate wins:** confirm the same cell and resource limits under the registered rule. Helper tracing is not a prerequisite for establishing the configuration's benefit.
- **Candidate loses or is inconclusive, with large external waits:** investigate the observed queue/reader/scanner constraint first.
- **Inversion remains the relevant critical region, but scaling fails:** distinguish serial assembly/setup from cooperative execution, then measure participant completion skew and useful work versus synchronization.
- **A placement-sensitive result motivates an SMT claim:** use a controlled thread placement comparison or execution-placement observations before assigning the loss to a specific sibling relationship.

A useful future helper trace would identify team/block/phase, task start/end and last-finisher skew, and distinguish spin CPU from parked and runnable-but-unscheduled time. Clock time in a waiting loop alone conflates those states. None of that should be added merely to satisfy a diagnostic checklist.

The perturbation screen has a sensible rejection direction, but a confidence interval containing one means **no perturbation detected**, not demonstrated negligible perturbation. With six rounds, retain interval widths; do not treat an imprecise trace as validated. This limitation affects interpretation of the auxiliary pass, not whether the uninstrumented throughput screen is worth running.

## 4. Two small checks before interpreting qualification

In [team_size.py, judge](https://github.com/lmmx/mibz2/blob/6f3183d96f2e8fc1d59751e1279be062fc09d897/bench/team_size.py), primary and reference completeness reject duplicate rounds. The control's `flagged` expression checks its interval, round IDs and plan, but **omits `ctl["duplicate_rounds"]`**. A duplicated successful control row can be counted in that field while the first copy supplies complete round IDs and a passing interval; the run is then not flagged. That contradicts the registration's rule that failure of the control's complete predicate invalidates the run. Include that predicate in the verdict, or independently enforce it before qualification. This is a reporting correction, not a reason to build helper tracing.

The counter amendment directly addresses the earlier discarded-JSON defect: child hashes, errors and exact aggregate leader/helper counts are now retained and checked; invalid aggregates are not displayed as counter data. I inspected that path but did not certify actual perf availability or exhaustively test the reporter. One invalid reading currently suppresses the cell's aggregate, rather than displaying a selective valid subset; that is conservative, and the stored evidence should be retained.

The registration still says counters run “after every timed round”; the recipe runs them after the complete timing command. Correct the chronology in the prose. Neither this nor the control-completeness defect needs a new experimental arm.

## Disposition and evidence boundary

The original 8-versus-16-slot finding is closed at this pin. Helper-side tracing is a follow-up, not an entry requirement. The occupancy additions provide useful descriptive and topology evidence when read with the boundaries above. This assessment gives scientific advice, not implementation, merge or policy authority.

Read the pinned registration, complete team-size collector/report, relevant benchmark entry points and recipe, occupancy mask construction/child launch/phase accounting/perturbation logic, and decoder mark placement and inversion dispatch. Recomputed the window and mask/core counts from source. No native execution, timing, hardware counters or changes to implementation were performed.
