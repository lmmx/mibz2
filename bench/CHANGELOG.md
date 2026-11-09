# Changelog

The benchmark (`bench/`, `examples/ab.rs`, `examples/phases.rs`, `.github/workflows/bench.yml`):
how mibz2 is measured, not what is measured. Changes to the decoder are what the
benchmark measures, and never change its version.

Its version is `BENCH` in `bench/report.py`, and every result records the version that
measured it.

**What each kind of change is:**
- **Major:** a change to what is measured or how, which changes a figure's meaning. That
  covers the corpora and their scale, the rounds, the thread counts, the harness's
  timing, or a stage's method.
- **Minor:** a new figure or table beside the others, which leaves every existing figure
  as it was.
- **Patch:** a change to how results are shown or stored.
- **No version:** a change to the benchmark's files that changes nothing it measures,
  shows or stores, such as a CI or build fix or a test. Its pull request says so, and why.

**Who decides:** whether a change alters a figure's meaning is the author's and the
reviewer's call. CI checks only that `BENCH` and the newest entry here agree; it cannot
tell whether a change needed a new version.

**Comparing results:**
- **A figure compares only within the major version in which its meaning was last set**,
  on one runner CPU model and core count, one set of corpora, and one corpus content (its
  recorded sha256).
- **Each family of figures records the majors that redefined it** (`MEANING` in
  `bench/report.py`). The site's history joins a family's points across later majors that
  left it alone, and splits it where a major redefined it.
- **Every point keeps the version that measured it.**

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the
benchmark follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html). Entries up
to 2.1.0 were written after the fact, from the descriptions in `bench/report.py` and their
pull requests. Until then, these versions were kept by hand, with no check in CI.

## [3.4.0] - 2026-11-09

### Fixed

- **The machine-state recorder's validity gaps** (the Researcher's REVIEW 6738754551, on
  #290's `examples/ab.rs` at 637ad34b). Opt-in (`ab run --machine-state`), recorded and never
  judged, as before. Nothing changes without the flag, and no measured figure changes meaning.
  - **A failed enable or disable was ignored** (`Perf::each` dropped the ioctls' return
    values). Each failure is now recorded (`perf_ctl_errors`: counter, request, errno). That
    counter's value is null, with the failure as its reason.
  - **`eff_ghz`'s multiplexing check is stated exactly.** It is null where cycles or the
    task clock was multiplexed, the two counters it divides. 3.3.0's description ("null where
    a counter was multiplexed") claimed more than its code checked. Instructions' and reference
    cycles' validity is their own: `perf_multiplexed` lists every counter that counted less
    than its enabled time. Any use of instructions with cycles (CPI) needs both.
  - **Each counter's raw time enabled and running** (`perf_times_ns`, ns, never scaled). 3.3.0
    kept only the least running share, rounded to 3 places (`perf_running`, kept, now to 6).
  - **The boundary skew:** the counters are enabled, and disabled, one after another.
    `perf_skew_ns` gives each sequence's duration and the order, so their intervals are
    known to differ by up to that much.
  - **The user-only fallback** (`perf_event_paranoid` 2 or more): `eff_ghz` is null there. The
    task clock's exclusion of kernel time isn't established: on the dev VM's kernel, a
    user-only task clock read 1.000 of a both-mode one over kernel-heavy work
    (`ab telemetry-check`'s `task-clock-modes`).
  - **Cycles over `eff_ghz` is the task clock again.** It is no independent frequency
    correction, and is never presented as one.

### Added

- **Each machine-state observation says when, over what and in which units** (Guardian D064's
  measurement limits).
  - **Beside each sysfs reading,** before and after a run:
    - its time (`t_unix_ns`, the reading's start, CLOCK_REALTIME) and duration (`read_s`);
    - each CPU's own `scaling_cur_freq` in kHz by CPU id (`cpu_khz`, with `cpu_khz_why` for any
      CPU that can't be read);
    - each cpufreq policy's id, CPUs, driver, governor and EPP (`policies`), raw;
    - the ACPI platform profile;
    - the run's own `t_spawn_unix_ns` and `t_exit_unix_ns`, so each reading's distance from the
      run is explicit.
    - The 3.3.0 mean (`freq_mhz`) is kept. It is a mean over the allowed CPUs at one moment, a
      policy's requested or reported frequency there, not the clock during the run.
  - **In the child, with the counters:**
    - reference cycles (`ref_cycles`), raw. A constant-rate count grows with elapsed on-CPU
      time; it does not remove a slowdown;
    - the counters' coverage (`perf_coverage`);
    - the CPUs the child may run on (`cpus_allowed`, from `/proc/self/status`);
    - the process's voluntary and involuntary context switches across the timed region
      (`ctxsw_voluntary`, `ctxsw_involuntary`, the sampler's own included);
    - the CPUs the sampler saw the run's threads on (`cpus_seen`: each thread's last CPU at
      each 10 ms sample, a sample and not an account).
  - **`ab sample [--count N] [--interval S]`:** untimed and read-only. Exactly N readings
    (default 10) S seconds apart (default 1), one JSON line each. Unlike `ab settle` it
    judges nothing: it never stops early, waits for no condition, and never fails, so its
    length is fixed before it starts.
  - **`ab telemetry-check`:** untimed. It exercises the recorder's failure paths and its
    coverage, one JSON line a case, and exits 1 if any case FAILs. The cases:
    - a failed enable and disable;
    - a counter that can't be opened;
    - the multiplexing predicate, and live multiplexing under 24 extra cycle counters;
    - the user-only fallback, forced;
    - the task clock's user-only semantics on this kernel;
    - inheritance: a thread started before the counters open is not counted, one started in
      the region is, and nothing outside it is.

    A case without the events it needs is UNAVAILABLE, never PASS. `just
    bench-telemetry-check` runs it on a workstation, untimed, as one bundle.
  - **`report.py machine`** shows the reference cycles, the counters' least share running,
    the allowed and seen CPUs, and each run's counter faults. Its labels say what each column
    is ("cycles/task-clock GHz", "scaling_cur_freq mean"), not "the effective clock".
- **Overhead, with the flag only:** a fourth counter (reference cycles), cloned for each
  thread as the other three are; two `getrusage` calls and one `/proc/self/status` read in
  the child, outside the wall's region; and, in the sampler, one more field parsed from the
  `stat` it already reads. Inside the timed region the counters' context-switch cost grows
  with the counter count. That cost is untested on real hardware: the dev VM has no PMU.

## [3.3.0] - 2026-11-09

### Added

- **The machine's state beside each run, recorded and never judged** (`examples/ab.rs`'s
  `machine` module, `bench/report.py`'s `machine_state`). Opt-in: only `ab run
  --machine-state` records it; without the flag no record, and no timed path, changes, and
  `ab run` clears `AB_MACHINE_STATE` on its children so an inherited one never turns the
  counters on. No frozen recipe passes the flag. Why: 41 of the 140 runs of
  `run-fed-leader/20261109T080633Z` (#270) took twice the wall with every thread's CPU
  doubled, consistent with a halved clock, and nothing recorded the clock.
  - **In the child, over exactly the region its wall is:** `cycles` and `instructions`
    (perf's hardware events) and `task_clock_s` (the counted threads' CPU time),
    `perf_event_open` on the child, inherited by every thread it starts. They are opened
    disabled before the region, enabled just before its clock starts, disabled just after it
    stops, and read after. Cycles count user and kernel mode where `perf_event_paranoid`
    allows it, else user mode only (`perf_scope`). `eff_ghz` is cycles over the task clock,
    null where a counter was multiplexed (`perf_running` below 1). The parallel arms'
    sampler starts before the counters open and is never counted. A program's arm
    (`lbzip2`, `indexed_bzip2`, `bz2ng`) is not counted.
  - **In the parent, before the run starts and after it ends,** outside the run's other
    readings: `machine_pre` and `machine_post`, each with the mean `scaling_cur_freq` over
    the run's CPUs, the governor and energy-performance preference of the cpufreq policies
    holding them, the turbo state (`intel_pstate/no_turbo` or `cpufreq/boost`), and the
    package temperature (thermal zone `x86_pkg_temp`, else hwmon `coretemp`'s package, else
    `k10temp`'s Tctl).
  - Every field is null with a `<field>_why` reason where it can't be read: never a guess.
  - **Overhead, with the flag only:** a few syscalls in the child outside its region
    (opening, enabling, disabling and reading the counters: in its process CPU, not its
    wall; the counters count the tail of the enables and the head of the disables). Inside
    the region: the kernel clones the three counters for each thread the decode starts,
    and saves and restores them at each context switch of a counted thread. The parent's
    sysfs reads fall outside every reading of the run.
- **`ab env`'s `machine` field:** the cpufreq driver, governor, EPP and turbo state over
  every policy, `perf_event_paranoid`, and whether perf can count cycles here (and in which
  mode), or why not. Always printed; `ab env` times nothing.
- **`ab settle [--seconds N] [--interval S]`:** untimed. It samples the clock (the mean over
  the allowed CPUs) and the package temperature every S seconds (default 1) until both
  hold within the registered tolerance over the last 5 samples: the clock within 3% of the
  window's mean and the temperature within 2 °C (max − min). It stops after N seconds
  (default 120), prints one JSON line, and never fails: `settled` is null with a reason
  where neither can be read. Recipes may call it before round 0; no frozen recipe does.
- **The report's machine-state table** (`summary`'s `machine_state`, and `report.py machine
  RUNS.jsonl`, one row a run, for any recipe's runs file). It shows the measured cycles a
  byte and the effective clock beside the wall, and the clock and temperature before and
  after. It feeds no verdict: the assessment, the figures and the accounting read nothing
  from it (bench/test_report.py checks they're unchanged). The existing "cycles a byte at
  the reported clock" figures are unchanged and still modelled from `/proc/cpuinfo`'s clock.

## [3.2.3] - 2026-11-06

### Added

- **Every thread each run's child ran, busy or not** (`examples/ab.rs`, `observed_threads`).
  The sampler's `thread_cpu` is the child's own report, and lists only the threads that ran
  more than 2% of the busiest's CPU, so it can't show a team's idle helper. Now the parent
  `ab` reads the names of every thread each child runs from `/proc`, every 5 ms
  (`AB_CENSUS_MS`), the same way for every arm, including a base build's own `ab`, which
  records no such list itself. The three-team policy's A/B (`bench/policy_ab.py`) reads each
  run's plan from both. This is a change to what is stored. Every figure keeps its meaning:
  the census runs in the parent, outside the child whose figures are reported.

## [3.2.2] - 2026-11-06

### Fixed

- **A merge whose head was also benchmarked by a dispatch is linked** (`bench/links.py`).
  A dispatch given a base, such as a cross-model check, runs as a pull request and uploads
  the same `bench-pr-<head>` artifact as the pull request's own run. `sync` took the newest
  artifact, found a `workflow_dispatch` run and left the merge unlinked, so #207 broke the
  history's chain. It now takes the newest artifact from a successful `pull_request` run.
  This is a change to what is stored. Every figure keeps its meaning.

## [3.2.1] - 2026-11-05

### Added

- **Each merged pull request's paired benchmark is kept** (`bench/links.py`,
  `.github/workflows/bench-links.yml`). Its last run's head against base, on one runner, is
  stored as a link of the history's chain in the results store: `links.jsonl`,
  `pr/<commit>.json` and `pr-raw/<commit>.jsonl`.
  - A push to master links its own merge. A dispatch links every earlier merge whose
    artifact is still kept (90 days).
  - Nothing is measured again: the figures are the pull request's own runs.
  - This is a change to what is stored. Every figure keeps its meaning.

## [3.2.0] - 2026-10-29

### Added

- **libsais in the kernels' table:** libsais 2.12.1's `libsais_unbwt_cyclic_ctx`, its
  inverse BWT for bzip2's cyclic BWT, beside mibz2's and bz2ngrs's kernels.
  - It is built single-threaded from its pinned, SHA-256-checked source
    (bench/libsais.sh), the same commit and flags as the cooperative pilot's, without
    OpenMP.
  - It takes the counts from MTF, as mibz2's build does, and is timed as one call a
    block. Its build and walk are inside that call, so only its total is shown.
  - It is an inverse BWT, not a bzip2 decoder, so it's in the kernels' table only, never
    in the decoders' figures or the Alternatives chart.

Every existing figure is as it was.

## [3.1.0] - 2026-10-29

### Added

- **Two alternatives, on master and from the Actions tab:**
  - `bz2ngrs`, Softleif's safe-Rust port of bz2-ng (`DecompressReader`), at a pinned
    commit, in bench/alternatives' build of the harness;
  - `bz2ng`, bz2-ng's C, its `bzip2 -dc` built at a pinned commit and read through a pipe.
  - Both are single-threaded, so they run at 1 thread only, as libbzip2 does.
- **The inverse BWT alone, mibz2's against bz2ngrs's** (`kernels` in bench/alternatives;
  `--kernels` in bench/report.py): a table beside the stage table, on master and from the
  Actions tab only.
  - Both kernels run on the same Wikidata blocks on one CPU, with the build, the walk
    and the output's assembly timed apart, after a check that they give every block the
    same bytes.
- **examples/ab.rs's one-shot arms**, `mibz2-oneshot` and `bz2ngrs-oneshot` (the file
  read whole, then decoded into one Vec), for bench/compete.sh. They're not in the
  workflow's arms.

Every existing figure is as it was: the new arms and the table sit beside them.

## [3.0.0] - 2026-10-28

### Added

- **Resources, beside the decoder's figures** (`resources`, `paired_resources` and
  `assessment` in bench/report.py), on the Wikidata blocks:
  - CPU per byte at each thread count;
  - its **amplification**: CPU at N threads over the same build's at 1, paired by round.
    1× means no extra CPU for the same bytes. Rising, it means the work costs more as
    threads are added, from stalls, sharing or synchronisation;
  - parallel efficiency;
  - each build's best thread count;
  - the threads' mean run-queue wait;
  - the CPU the per-thread split leaves unattributed.
- **On a pull request:**
  - head's amplification over base's, at each thread count;
  - best against best;
  - a one-line **assessment** from four comparisons declared in `assessment`: speed (best
    against best), CPU per byte at the most threads, amplification, and peak RSS at the
    most threads. It says whether the run could answer (parity, accounting, throttling),
    and when a change needs a machine with more cores.
  - Nothing in it fails CI, except the outputs and the accounting's own check.
- **2-thread runs, pinned to one CPU a core** (`ab run --spread`). On CI's runners (4 CPUs
  on 2 cores) they are two cores' worth, so their amplification is free of SMT. The
  4-thread runs share each core's two siblings, as before.
- **Recorded per run:**
  - the CPUs a run was given (`cpus`);
  - the threads' run-queue wait (`runq_s`, from schedstat);
  - the sampler's own CPU and CPUs (`sampler_s`, `sampler_cpus`);
  - the cgroup's CPU throttling over the run (`throttled_n`, `throttled_ms`; null where
    unreadable);
  - the system's CPU pressure stall time (`psi_cpu_ms`, PSI; null without it). A CPU
    stall is waiting for a CPU, never a core waiting on memory.
- **`AB_SAMPLE_MS`:** the sampler's interval (default 10; 0: no sampler). The Actions tab's
  `base_sample_ms` input sets it for the base arm, so a run with the base set to head
  measures the sampler's own cost to the decode.
- **Each thread count's placement, as a row label:** "pinned to CPU 1", "CPUs 0,2, one a
  core", or "all 4 CPUs, sharing SMT siblings".
- **`ab env`:** the CPUs allowed, their physical cores, the cgroup's CPU quota, and a check
  that the run-queue counter counts (threads twice the CPUs spin for 0.3 s).
- **The CPU accounting's own check** (`accounting`): on this build's runs, the per-thread
  split never holds more than the process's total, getrusage over every thread. Failing
  it fails the run, as broken instrumentation.

### Fixed

- **The arms' order now alternates (`arm_order` in examples/ab.rs).** Before, a round's order
  was rotated a step and then reversed in every other round. With two arms the reversal
  undid the rotation, so on every pull request since 1.0.0 head ran before base in every
  round, at every thread count.
  - An A/A run on CI (#69: one commit as head and as base, 10 rounds, EPYC 9V74) measured
    the first-run arm at:
    - wall 1.8% faster at 1 thread (−3.2%, −0.4%) and 2.3% faster at 2 (−4.1%, −0.5%);
    - CPU 2.4% lower at 4 threads (−3.5%, −1.3%);
    - 7 of 42 head-over-base rows excluding no change, against about 2 by chance;
    - an assessment of "Better (CPU cost)" for no change at all.
  - Over 2 × arms rounds, each arm now runs in each position equally often, with a test.
  - **Past head-over-base verdicts within a few percent may lean toward head.** Larger
    ones stand.

### Changed

- **The decoder's figures are redefined (`MEANING`: decoder [1, 3]).** The per-thread
  sampler now samples every 10 ms instead of every 50 ms, and it is kept out of the run.
  - **Why the old split was short.** At 50 ms, mibz2's workers, which end when the jobs
    run out before the reader's last block, each lost up to 50 ms. That was 0.1-0.35 s a
    run, never missing from the CPU total but missing from the split and from the
    whole-dump model's workers: about 14% of the workers' CPU on CI's 4-thread Wikidata
    runs. At 10 ms the split's shortfall is 4-9%, now shown as unattributed.
  - **Kept out of the run.** The sampler moves itself off the CPUs a run is pinned to, so
    it never preempts a pinned decode, and its own CPU (RUSAGE_THREAD, exact) is left out
    of the run's CPU.
  - **The decode's clock starts only once the sampler has moved.**
  - **Measured on CI** (`base_sample_ms=0`: the same commit sampled and unsampled):
    - wikidata within noise at 1/2/4 threads (+1.6%, −1.5%, +2.4%, intervals about ±4%);
    - before that handshake, the 12 ms `onebyte` runs were +9.8% at 1 thread.
  - **A 1-thread run of an in-process arm runs no sampler at all.** After the handshake,
    sampling still cost 12 ms single-threaded runs +4.7–17% in wall (`runs`, `onebyte`),
    though wikidata stayed within noise (+0.6%, +0.7%, +1.2% at 1/2/4 threads, its CPU at 4
    threads +0.2%). Its split is its own thread's CPU, and its run-queue wait is its own
    schedstat.
  - **On a 2-core runner a 2-thread run's other CPUs are its own cores' SMT siblings,** so
    there the sampler shares the workers' cores.
  - **What that removes.** Kept in, at 10 ms, it cost the 1-thread runs +1.0% (+0.4%,
    +1.7%) in wall and CPU on #69's first bench. Before 3.0.0 the sampler's smaller cost
    was in every run's CPU. Runs on every CPU (4 threads on CI) still share their CPUs
    with it.
- **A thread is keyed by its id alone, its name read with its CPU from one `stat`.** A name
  read apart could come back empty as a thread ended, which made a second entry for the
  same thread and counted it twice.
- **The CPU split table shows the unattributed CPU.** A base built before 3.0.0 still
  samples every 50 ms, so on such a pull request its split and the model undercount its
  workers against head's.

## [2.1.1] - 2026-10-27

### Changed

- **The site's history joins a family of figures across majors that left it alone**
  (`MEANING`, `docs/scripts/bench_history.py`).
  - The decoder's own figures (throughput, CPU time and peak RSS, by corpus and thread
    count) were set at 1.0.0 and redefined by no later major. So their 1.x and 2.x points
    share a line, on one runner model and core count and one set of corpora.
  - The stage table's figures were redefined at 2.0.0, so they stay split.
  - Every point still records the version that measured it.
- **The history joins a corpus's points only where its content is the same** (its recorded
  sha256, `line_key`).
  - `text` is generated from this crate's own sources (`tests/common/mod.rs`), so its bytes
    change with them: in the seven records, seven contents, 778–982 KB compressed. Its
    points were one line before, and are now each a point of their own.
  - The other six corpora are the same bytes in every record, and keep their lines.
  - A pull request's head and base still decode one generated `text`, so its comparison
    stands.
- **This changelog,** moved from `bench/report.py`'s docstring. CI now checks that
  `BENCH` matches its newest entry (`bench/test_report.py`).

No figure is measured differently. On the seven records, the history keeps its
98 points, each with its values and interval, and shows them as 50 lines in 3 classes,
not 56 lines in 4.

## [2.1.0] - 2026-10-27

### Added

- **The room walk's segment assembly**, timed on its own: a row within the walk's in
  the stage table (#49, for #48).
  - The walk's row still counts the assembly, so every existing figure is as it was.
  - `bench/stages.py` and the report read a stage only where a build has it, both ways
    round, with `bench/test_report.py` as its test.

## [2.0.0] - 2026-10-27

### Changed

- **The stage table's figures change meaning** (#47). `examples/phases.rs` decodes the
  file in decode order, each stage timed as it runs and its fastest pass kept, with the
  other walks in passes of their own.
  - Before, each stage kept its best of repeats on the same block, and the walks ran back
    to back on it: a warm state no decode reaches (`docs/journal/2026-10-27-walk-sweep.md`).
  - **The decoder's own figures are measured as before.**
  - At the time, the history restarted every series at 2.0.0, as it did at any major.
    2.1.1 joins the decoder's series across it again.

## [1.2.0] - 2026-10-27

### Added

- **The stage table** (#35): each stage's cycles a byte on the Wikidata blocks at
  1 thread, from `bench/stages.py`'s runs of `examples/phases.rs`. On a pull request it
  shows head over base per stage. Every existing figure is as it was.

## [1.1.0] - 2026-10-27

### Added

- **Every alternative on master's run** (#26), each arm classed as block-parallel or
  single-threaded (`parallel`). A single-threaded arm runs at 1 thread only, and its
  1-thread runs stand, marked, in every thread count's figures.

## [1.0.0] - 2026-10-27

### Added

- **The benchmark** (#11): the decoder timed on each pull request against its base, and
  on master against pbz2 and lbzip2. Throughput, CPU time and peak RSS, by corpus and
  thread count, paired by round.
