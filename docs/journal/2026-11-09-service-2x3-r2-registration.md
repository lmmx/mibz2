# The service core at two teams, revision 2: the same registration, with the machine's state logged

9 November 2026. **DRAFT, for the Researcher's review.** Registered before any run, and not yet
frozen for the workstation: the Coordinator freezes it after review. Nothing here sets a ready tag
or adds a queue entry.

This is the next run of the service core's registered design (#258,
`2026-11-08-service-2x3-registration.md`, with its revision for HOLD 6729358962). It adds one
thing: the machine's clock and thermal state, logged beside every record as diagnostic
telemetry. That is option (b) of Guardian D064 ([REVIEW
6737091245](https://github.com/lmmx/mibz2/issues/145#issuecomment-6737091245); `state/decisions.md`
on `team-state/guardian`), and only (b). The Coordinator's LOG 6737988315 applied it as written.
It folds in the Researcher's REVIEW
[6738754551](https://github.com/lmmx/mibz2/issues/145#issuecomment-6738754551) (report
`state/reports/clock-state-6736096739/report.md` at 5129f318 on `team-state/researcher`): PROCEED
with recorded-only telemetry and the unchanged full-data analysis.

**It depends on #298** (`bench/telemetry-validity`, bench 3.4.0): the recorder's validity fixes
and its untimed check. This branch carries the same change as its commit 5080dee6, cherry-picked
from #298's c754acc8. It is to be frozen only after #298 is reviewed, and re-based on it if #298
changes.

## What stays exactly as registered

Everything that defines the experiment is #258's, by reference and byte for byte:

| | as registered in #258 | here |
|---|---|---|
| decoder (`src/`, Cargo files) | `research/service-core` at 9ef3c17d, measured at 3e201246 | identical (`git diff 7aeb5bf9 HEAD -- src Cargo.toml Cargo.lock` is empty) |
| cells (arms) | `mibz2`, `mibz2-svc2`, `mibz2-team2x3`, `mibz2-pipe2x3`, `mibz2-team3`, `mibz2-team3-twin`, at THREADS = 10 | identical |
| grid and repeats | one `ab run`, interleaved by round, arms rotated as `ab` does, 20 rounds; 6 trace rounds of svc2 at 10 | identical. No ABBA order, no repeated references, no extra rounds |
| run order | `ab`'s `arm_order(round, 6)`: deterministic, no seed, a step's rotation each round and reversed every other cycle | identical, so frozen with the harness. Each record's position, predecessor and gap are logged (below) |
| estimand | the paired geometric-mean ratio of wall, and of CPU, over the 20 rounds, with its 95% t-interval | identical |
| judged family and multiplicity | P1, C1, P2 judged; S1–S5 and R1 reported, never judged | identical |
| gates | the outcome map (GAIN lo > 1.02, NULL within [0.97, 1.03], REGRESSION hi < 1), the extra-thread rule, N1 within 1 ± 0.03, the saturation thresholds, the registered order of the overall reading | identical |
| validity and analysis population | every check of `bench/service_2x3.py`; every record of every registered cell enters | identical: `bench/service_2x3.py` and `bench/service_core.py` are unchanged from 7aeb5bf9 |
| corpus | the registered sample, `82f65ace…`, 209,700,196 decoded bytes | identical |

**Every pair and every record enters the unchanged analysis.** The report reads none of the
telemetry. `bench/test_service_2x3_r2.py` checks that the report's result and its Markdown are
identical with and without the telemetry fields, with a halved clock in a third of the rounds,
and with every telemetry reading missing.

**This run's reading stands on its own.** It is a second, separate run of a registered design.
It never replaces #258's VALID reading (P1 1.045 [1.017, 1.073]; B4). It is never pooled with
#258 or with #271's reported D-a2w8 (1.073 [1.041, 1.106]). If they differ, both are reported.

## What changes, and nothing else

| | change | why |
|---|---|---|
| T1 | the harness (`examples/ab.rs`, bench 3.3.0 from #290, and bench 3.4.0 here) records the machine's state beside each run; the decoder is untouched | D064 (b) |
| T2 | `ab run ... --machine-state` in the recipe | D064 (b) |
| T3 | one read-only preflight before round 0 (F1–F4 below), including the recorder's own untimed check; the same readings after the last round | D064, "Pre-run check"; the Researcher's REVIEW |
| T4 | `bench/report.py machine` writes `machine/machine-state.md`, run by run; `bench/service_2x3_diag.py` writes `machine/diagnostics.md` and `.json`: every record in run order, and every registered comparison's pairs with their clock states | the diagnostic tables (below) |
| T5 | a new recipe, `bench-service-2x3-r2`, its own frozen file `bench/service-2x3-r2.frozen`, its own directory `target/service-2x3-r2/`; the measured inputs add `bench/report.py` and this file | so #258's recipe, frozen at 7aeb5bf9, stays as it ran |

**An altered test condition, stated as D064 asks.** With `--machine-state` each run's child opens
four perf counters (cycles, instructions, reference cycles, task clock), inherited by every thread
it starts. The kernel then clones them for each new thread and saves and restores them at each
context switch of a counted thread, inside the timed region.
- **Every arm runs under it**, N1's twins included. So N1 still measures noise under this
  condition.
- **Its size is untested on real hardware:** the dev VM has no PMU, and #290's caveat stands. It
  can differ between arms, since they differ in threads and in hand-offs: svc2's hub switches
  more often. The records carry each run's context switches (`ctxsw_*`) so its scale is visible.
  Nothing corrects for it.
- **Outside the timed region,** `ab telemetry-check` (about 1 s), the parent's sysfs reads, the child's counter set-up and read-out,
  and the 10 s samples before and after are additional work. They are read-only and outside every
  wall the report reads.

## Telemetry (diagnostic, never judged)

Each field is a value or null with a `<field>_why` reason. **Missing is never imputed:** never zero,
never a nominal clock, never evidence of a settled machine. Units are in the names.

| field (per record) | source | interval and scope | units |
|---|---|---|---|
| `cycles`, `instructions`, `ref_cycles`, `task_clock_s` | `perf_event_open` in the child, pid 0, any CPU, `inherit` | enabled just before the wall's clock starts, disabled just after it stops. Covers the child and every thread it starts after the counters open, each followed across CPUs (migrations included); not the sampler thread (`perf_coverage`). User and kernel mode where `perf_event_paranoid` ≤ 1, else user only (`perf_scope`) | counts; s |
| `perf_times_ns` | each counter's own time enabled and time running | the same | ns, raw, never scaled |
| `perf_running` | the least of running/enabled over the counters | the same; below 1 means multiplexed | ratio |
| `perf_multiplexed` | every counter that ran less than its enabled time | the same | names |
| `perf_ctl_errors` | every failed enable or disable (counter, request, errno); that counter's value is then null with the failure as its reason | the region's edges | list |
| `perf_skew_ns` | the enable sequence's and the disable sequence's duration, and the order | the region's edges: the counters' intervals differ by up to this | ns |
| `eff_ghz` | cycles / task clock | the same. **Null** where cycles or the task clock was multiplexed, and null where cycles are user-mode only (the dev VM's kernel showed its task clock doesn't honour the user-only flag: `task-clock-modes`). It is descriptive: cycles over it gives back the task clock, so it is no frequency correction | GHz (descriptive) |
| `cpus_allowed` | the child's `/proc/self/status`, `Cpus_allowed_list` | read just before the region | CPU list |
| `cpus_seen` | each thread's last CPU (`/proc/<pid>/task/*/stat`, field 39) at each 10 ms sample, the sampler's own excluded | the region. **A sample, not an account:** a thread can run on a CPU between samples and never be seen there | CPU list |
| `ctxsw_voluntary`, `ctxsw_involuntary` | `getrusage(RUSAGE_SELF)`, before and after | the region, plus the enable and disable calls; the sampler's switches included | counts |
| `machine_pre`, `machine_post` | sysfs, read by the parent before the child is spawned and after it exits | one moment each: `t_unix_ns` (CLOCK_REALTIME, the reading's start) and `read_s` (its duration). `t_spawn_unix_ns` and `t_exit_unix_ns` give the run's own span | see below |
| … `cpu_khz` | each allowed CPU's `cpufreq/scaling_cur_freq`, by CPU id | that moment | kHz |
| … `policies` | each cpufreq policy holding an allowed CPU: its id, `affected_cpus`, driver, governor, EPP | that moment | strings |
| … `turbo`, `turbo_src` | `intel_pstate/no_turbo`, else `cpufreq/boost`, with the file read | that moment | bool |
| … `pkg_temp_c`, `temp_src` | thermal zone `x86_pkg_temp`, else hwmon `coretemp` "Package id N", else `k10temp` Tctl, with the sensor named | that moment | °C |
| … `platform_profile` | `/sys/firmware/acpi/platform_profile` | that moment | string |
| … `freq_mhz`, `freq_cpus` | the mean of `cpu_khz` over the allowed CPUs (3.3.0's field, kept) | that moment | MHz |

**Per attempt,** in `machine/`:
- `pre-sysfs.tsv` and `post-sysfs.tsv`: each listed file's path and its raw contents, or
  UNREADABLE, with the time read. The files: every policy's CPUs, driver, governor, EPP,
  current, minimum and maximum frequencies and base frequency; every CPU's `scaling_cur_freq`;
  `intel_pstate`'s status, turbo and performance limits; `cpufreq/boost`; SMT; the platform
  profile and its choices; every thermal zone's type and temperature; every hwmon's name and
  temperature labels and inputs; `perf_event_paranoid`; `nmi_watchdog`.
- `pre-sample.jsonl` and `post-sample.jsonl`: `ab sample --count 10 --interval 1`, exactly 10
  readings 1 s apart, each a full state as above.
- `preflight.txt` (below) and `machine-state.md`.
- `env.json`'s `machine` field: the cpufreq driver, governor, EPP and turbo over every policy,
  `perf_event_paranoid`, and what perf can count here (cycles, scope, instructions, reference
  cycles, task clock), or why not.

**What the telemetry may be used for.** Description, beside the registered result: tables of
the records as they are.
- **No frequency-normalised ratios:** the judged and reported ratios stay wall and CPU. Cycles,
  cycles per byte and `eff_ghz` are never substituted for them.
- **No strata:** no comparison is recomputed within a clock or temperature band.
- **No exclusion:** no record, pair or round is flagged out, deleted or replaced.
- **No extra rounds.**
- **No veto:** no whole-attempt quality constraint. None is approved (D064 left its rule,
  threshold, baseline, missing-data behaviour and k unspecified). One would be a separate
  registration under C2/G3, never applied to this run.
- **No causal reading.** A slowed record shown beside a lower clock reading is *consistent with*
  a lower effective clock, not proof of it.
  - Endpoint sysfs readings describe their moments, not the 0.15 s between them.
  - `scaling_cur_freq` may be a requested state (the kernel's CPUFreq documentation).
  - A mean over allowed CPUs is not a workload-weighted frequency.
  - Unvalidated counter output is no gate (D064).

## The diagnostic tables (never judged)

`bench/service_2x3_diag.py`, after the registered report, which it never feeds:
- **In run order:** every record, with its round, its position in the round, the cell before
  it, the gap since that cell's child exited, its wall and CPU, cycles over reference cycles,
  its clock state, `eff_ghz`, the endpoint readings and its counter faults. This is the
  carryover inspection the REVIEW asks for: predecessor and position, shown, not modelled.
- **Each registered comparison's pairs, round by round** (P1, C1, P2, S1–S5, R1, N1): the
  pair's log speed and log CPU as the report forms them, and each member's clock state. The
  pair's state is arm then reference: HH, HL, LH, LL or unknown.
  - **Counts only per state, never a ratio, interval or mean within a state.** No
    stratification, even one declared here. Conditioning on clock can remove part of a real
    treatment effect, since an arm's own power draw can lower its clock (the REVIEW's
    witness: [0.9, 0.9, 1.8, 1.8] has geometric mean 1.273, and its high-clock pairs alone
    give 0.9).
  - These states are never competing verdicts.
- **A record's clock state, fixed here before any run and independent of any outcome:**
  - **L:** cycles over reference cycles below 1, i.e. the counted threads ran below the
    reference rate. The two counters are opened alike, so their mode scope matches.
  - **H:** that ratio at 1 or above.
  - **unknown:** either counter missing, failed or multiplexed. Unknown is never imputed.

  The state describes the counted threads' on-CPU time, aggregated over them. It is not a
  critical-path frequency, not a cause, and not a quality rule. Reference cycles grow with
  elapsed on-CPU time, so this is a rate comparison, not a correction.

## The untimed validation, before the frozen attempt

The REVIEW's next value is an untimed check of the recorder's errors and coverage before any
recorded-only attempt. It comes in two steps.
1. **`just bench-telemetry-check`** (#298), run on the workstation first, untimed, as its own
   bundle. It times no decoder and is never timing evidence. It runs `ab telemetry-check`:
   - a failed enable and disable;
   - a counter that can't be opened;
   - the multiplexing predicate, and live multiplexing under 24 extra cycle counters;
   - the user-only fallback, forced, in software and on the real counters;
   - the task clock's user-only semantics;
   - inheritance: a thread started before the counters open is not counted, one started in the
     region is, and nothing outside it is.

   Each case is PASS, FAIL, OBSERVED or UNAVAILABLE. Its bundle shows what the real PMU does
   before this registration is frozen. Whether a result there changes this registration is
   for review, not an automatic step.
2. **The same check again as the preflight's F4,** in the recorded-only attempt itself, so the
   attempt's own bundle shows the recorder working on that machine at that commit.

Locally, on the dev VM (no PMU, no cpufreq), `ab telemetry-check` gave:
- PASS: control-failure, missing-counter, multiplex-predicate, user-only-fallback, inheritance
  (0.108 s against 0.100 expected);
- UNAVAILABLE: the three hardware cases;
- OBSERVED, task-clock-modes: 1.000.

## The preflight (bounded, read-only, before any timed run)

After the build, the corpus, `ab env` and `lscpu`, and before round 0. It changes no setting and
writes nothing outside the attempt's directory. It takes about 10 s, fixed: its length doesn't
depend on what it reads.

| | check | refuses when |
|---|---|---|
| F1 | the raw sysfs snapshot is written to `machine/pre-sysfs.tsv` | the file can't be written |
| F2 | `env.json` has the machine-state probe (`machine.perf`) | it is absent: this is not the telemetry build |
| F3 | `ab sample --count 10 --interval 1` gives its 10 readings in `machine/pre-sample.jsonl` | it fails, or gives other than 10 readings |
| F4 | `ab telemetry-check` writes `machine/telemetry-check.jsonl` | a case FAILs: the recorder records wrongly on this machine. UNAVAILABLE is recorded, never refused on |

- **One preflight, one attempt.** It refuses only where the telemetry tools themselves fail, or
  the recorder's own check FAILs. It never refuses on a value: an
  unavailable counter, a missing sensor, a governor, a temperature or a clock is recorded.
  No threshold on the machine's state is registered, and none may be added to this run.
- **A refusal is archived.** The outcome is `REFUSED: PREFLIGHT: <check>`, `preflight.txt` says
  which check refused and why, and the attempt is bundled and submitted like any other.
- **No retry, no settling, no cooldown.** The recipe runs the preflight once and never repeats it.
  It never calls `ab settle` (see "The harness against D064" below). It doesn't wait for a state
  of the machine or change one. A refused or failed attempt is kept and reported. Any further
  attempt is Louis's, as every attempt is (C5): a refusal authorises no rerun, and changed code is
  no permission to retry.
- **After the last round,** the same snapshot and sample are taken and recorded. They never
  refuse: a failure there is noted in `machine/` and the run goes on to its report.
- **Incomplete diagnostics are archived as they are.** A failed post-run sample, machine table
  or diagnostics file is noted (`machine/*-failed.txt`) and bundled. It never changes the
  registered reading. A diagnostic objective that fails goes back to review, not to a new run.

## The harness against D064

- **`ab settle` is not used.** As built (#290), it samples until the clock is within 3% and the
  temperature within 2 °C over 5 samples, or 120 s pass.
  - Its wait therefore depends on the machine's state: it waits, boundedly, for a favourable one.
  - It keeps only its last reading and a count, not its samples.
  - `ab sample` takes its place: a fixed count at a fixed interval, every sample kept, nothing
    judged.
- **`ab run`'s per-run wait for other processes is unchanged.** Before each run, `ab run` samples
  the other processes' load for 2 s. It waits in 20 s steps while they keep more than 1.5 cores
  busy, or one CPU more than half, recording `waited_s`.
  - That loop has no bound.
  - It is the registered harness's, as in #258, and it waits on other processes' load, not on
    the clock or temperature.
  - Changing it would change the registered method. It is an open question below.
- **#290's labels.** Its report table called cycles over the task clock "eff. clock" and the
  endpoint mean "clock". Bench 3.4.0 relabels them: "cycles/task-clock GHz", "scaling_cur_freq
  mean". It keeps the mean as recorded and adds the raw per-CPU readings.
- **#290's `eff_ghz` in user-only scope** divided user-mode cycles by a task clock whose scope
  isn't shown to match. 3.4.0 makes it null there, with its reason. On the dev VM's kernel the
  task clock ignores the user-only flag.
- **#290's other gaps** (the REVIEW, fixed in #298):
  - ignored ioctl results;
  - a multiplexing rule worded more broadly than its code;
  - only a rounded minimum running share;
  - no boundary skew.
  See #298's table.
- **Allowed against observed CPUs.** An unpinned 10-thread run's allowed CPUs are all 20. 3.4.0
  records them (`cpus_allowed`) apart from the CPUs its threads were seen on (`cpus_seen`).
- **Not validated:** the counters on real hardware, their overhead and their differential
  perturbation. D064 records that #290 was not code- or PMU-audited. This run is the first real
  use of the counters on the i9. Their output is descriptive until validated, and no gate.

## Size and run time

As #258: 6 cells × 20 rounds = 120 runs, about 4.3 minutes, plus the build check, corpus and
trace. Add about 20 s for the two samples and the check, so about 7 minutes once built (11 from
a cold build).

## Local runs

Local runs on the shared 4-core machine are correctness checks only, never timing evidence. The
recipe's DRY run (`just bench-service-2x3-r2 20 1`) is untimed, and the report never calls it
VALID.

Done for this draft:
- `cargo test --example ab`: 13 tests, including the fake-sysfs trees for the raw readings, the
  software-event stand-in for inheritance, the region and the scope rule, and
  `the_recorder_check_passes_where_it_can`;
- `ab telemetry-check` (above);
- `bench/test_report.py`, `bench/test_service_2x3.py`, `bench/test_service_2x3_r2.py`. The last
  includes the diagnostics: run order, predecessor and gap; pair states counted, never
  averaged; unknown never imputed;
- shellcheck of the recipe body;
- an untimed DRY run on a small local `.bz2`, outside the registration. It checks the recipe's
  path, preflight, telemetry fields and bundle only. The VM has no cpufreq, sensors or PMU, so
  every reading is null with its reason.

## Open questions for the Researcher's review

1. **The run's value under C3.** Against #258 (VALID, P1 1.045) and #271's reported replication,
   this run adds:
   - a third reading of P1, C1 and P2 at the same runtime;
   - the first real-hardware record of the counters on a design whose last two runs showed no
     slowed mode.
   Is that worth a workstation run beside the open work (the starvation diagnostic, #293; the
   revised #270)?
2. **The counters' perturbation.** Is running every arm counted acceptable as a uniform,
   registered condition? Or does D064's "documented and tested" need a separate, pre-registered
   counted-against-uncounted diagnostic first, with its own information objective (C1),
   incapable of qualification?
3. **`ab run`'s unbounded wait on other processes' load** (above). Leave it as registered, or
   bound it in a separate revision? Bounding it changes the method (C6).
4. **The preflight's refusals.** F1–F4 refuse only on tool failure or a recorder FAIL. Should an unavailable PMU
   (`perf_event_paranoid` above 2, or no PMU) refuse instead, so that no attempt runs without
   counters? A refusal would then depend on a host setting that no agent may change.
5. **The base.** This branch is `research/service-core-2x3` (3e201246) with #290 cherry-picked, so
   the runtime stays the one D057 cleared. A merge of master would bring the guard (#278) and the
   M12 unwind changes into both the default and the service core: a changed runtime (D057). Is
   keeping 3e201246's runtime right for the "next qualifying run"? Or should that run measure
   master's guarded runtime, under a registration of its own?
6. **The clock-state cut.** L/H is cycles over reference cycles against 1, the reference rate.
   It is fixed before the run, depends on no outcome, and needs no tuning. But it reads "below
   the base clock", not "slowed". On the i9-10900K (base 3.7 GHz, all-core turbo up to 4.9), a
   halving from turbo reads L, but a drop from 4.9 to 3.8 GHz still reads H. Is a cut at 1 the right diagnostic? A different fixed cut needs a
   number named before the run, and none is chosen from timings here.
7. **The untimed `just bench-telemetry-check` on the i9:** should it be a precondition of the
   freeze, as written above? A FAIL there, or an UNAVAILABLE PMU, would bring this registration
   back to review.
