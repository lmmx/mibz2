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
  - the sampler's own CPU (`sampler_s`);
  - the cgroup's CPU throttling over the run (`throttled_n`, `throttled_ms`; null where
    unreadable);
  - the system's CPU pressure stall time (`psi_cpu_ms`, PSI; null without it). A CPU
    stall is waiting for a CPU, never a core waiting on memory.
- **`ab env`:** the CPUs allowed, their physical cores, the cgroup's CPU quota, and a check
  that the run-queue counter counts (threads twice the CPUs spin for 0.3 s).
- **The CPU accounting's own check** (`accounting`): on this build's runs, the per-thread
  split never holds more than the process's total, getrusage over every thread. Failing
  it fails the run, as broken instrumentation.

### Changed

- **The decoder's figures are redefined (`MEANING`: decoder [1, 3]).** The per-thread
  sampler, a thread inside each run's process whose CPU is the run's, now samples every
  10 ms instead of every 50 ms.
  - At 50 ms, mibz2's workers, which end when the jobs run out before the reader's last
    block, each lost up to 50 ms. That was 0.1-0.35 s a run, never in the CPU total but
    in the split and in the whole-dump model's workers: about 14% of the workers' CPU on
    CI's 4-thread Wikidata runs.
  - At 10 ms the split's shortfall is 4-6% (unattributed, now shown).
  - The sampler's own CPU is recorded: about 0.01 s a second of run.
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
