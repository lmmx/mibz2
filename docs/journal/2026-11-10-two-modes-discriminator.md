# 2026-11-10: The two modes' discriminator: registration (diagnostic only)

**Status: registered, NOT FROZEN.** The recipe is `just bench-two-modes` (`.just/two-modes.just`).
It refuses every run but a DRY one until `bench/two-modes.frozen` names a commit. Only the
Coordinator freezes and queues it, after the Researcher's review.

- **What it is:** §3 of `2026-11-10-two-modes.md` (the Pathfinder's design) built as a C5
  workstation recipe for the i9-10900K.
- **What it is for:** at 10 threads the same decode takes about 0.15 s or about 0.30 s, and the
  mode persists from run to run. This run is meant to name the mechanism.
- **Diagnostic only.** No merge gate, acceptance rule or adoption decision reads its outcome. It
  times nothing that is compared with another bundle, and it makes no verdict about the
  favourable mode. Every record is kept. No run is excluded, replaced or retried because of its
  mode.

Everything below, the class threshold and the analysis included, is fixed before any data.

## 1. The run

- **The harness that produced the two modes, unchanged.** The decoder and harness executables
  are the run-fed leader's second attempt's own, not rebuilt: `ab` and `windows` from its
  archived bundle `run-fed-leader/20261109T201013Z.tar.gz` on the `evidence` branch at
  `65ab4b95`.
  - The bundle is checked by its SHA-256 (`7e36a5a7…`), and the two executables by theirs
    (`ce1eea53…`, `cf5d74a6…`): bench 3.6.0, built at `1af3888a`, frozen at `2fab12ca`.
  - The corpus is the bench's Wikidata sample (`82f65ace…`), built by that `windows`.
- **One arm, the default plan:** `mibz2-default33`, at 10 threads, with `--settle 0.25`
  `--sysfs-state` and the default busy gate. This is the run-fed leader recipe's configuration,
  and its gaps between runs.
- **Through `ARM@EXE`:** ab spawns `bench/two_modes_probe.c`'s `child`, not itself, for each run.
  The wrapper:
  1. reads the counters;
  2. starts the probes;
  3. forks, sets the decoder's CPU mask in the child, and execs ab's own `child mibz2-default33
     10 FILE`;
  4. reads the mask back (`sched_getaffinity` and `Cpus_allowed_list`);
  5. samples until the decode exits, and reaps it with `wait4`;
  6. reads the counters again and appends one JSON line.

  The decode's stdout and stderr are ab's pipe and terminal, untouched, so ab records the child's
  own line (wall, CPU, output, hash) exactly as before.
- **The probe core:** CPU 9 and its sibling 19, the last physical core. Both are taken out of the
  decoder's mask, which is `0-8,10-18`. ab itself runs on that mask (`taskset`), so nothing of
  ours shares the probe core but the probes.
- **First, the sustained block, with no probe** (the Pathfinder's addition, #145 6756520552).
  - **What runs:** one process, ab's own `child mibz2-default33 10`, unpinned on all 20 CPUs,
    decodes the corpus concatenated 50 times (593 MB in, 10.5 GB out, hashed and dropped) end
    to end. The concatenation is kept beside the attempt's directory, never in the bundle. It
    goes through the probe binary's `sustained` mode, which neither pins nor spins.
  - **What it records:** the input's progress (`/proc/PID/io` `rchar`) every 10 ms, summed
    into 100 ms windows; every CPU's `scaling_cur_freq` and the child's `AnonHugePages` every
    100 ms; the counters before and after, and its `wait4` rusage.
  - **Why the input:** the harness keeps no output file, and its decoder reads its input in
    chunks (about 4 MB each) a bounded distance ahead of what it has decoded, so the input's
    progress tracks throughput at chunk resolution. A 100 ms window holds one or two chunks on
    the i9, so the signature is read on 500 ms blocks.
  - **Its deadline:** 300 s (TERM, then KILL 10 s after). The block is expected to take 8 to
    16 s. A block that times out or fails is recorded and makes the attempt INVALID; it is never
    retried.
  - **A fixed 60 s rest follows** before pass 1, so the probe passes don't start in the block's
    heat.
  - **No harness change:** the block runs the archived `ab child` unchanged; a multi-stream
    input is what the corpus already is (twelve streams one after another).
- **Pass 1, 60 runs:** P-idle on CPU 9, on a fixed schedule, `PPN` repeating from the first run.
  Two runs of every three are probed; the third runs with no probe, with the same mask and the
  same readings. That makes 40 probed runs and 20 unprobed, interleaved.
- **Pass 2, 60 runs:** P-idle on CPU 9 beside P-sib on CPU 19, in every run. This prices what a
  busy sibling costs the first probe, the calibration H2 needs.
- **Before anything runs, a preflight.** It is read-only, each collector bounded (TERM at 30 s,
  KILL 5 s after), and each collector's exit status is archived in `preflight/exits.txt`. The
  attempt is REFUSED and archived unless every point below holds:
  - every collector exited 0;
  - the host is the i9-10900K with 20 CPUs and 10 cores, and an unlimited CPU quota;
  - its declared settings hold: intel_pstate, `powersave`, `balance_performance`, turbo on;
  - every CPU has an invariant TSC (`constant_tsc` and `nonstop_tsc`);
  - CPUs 9 and 19 are one core's two siblings, and that core is the last;
  - the core and package throttle counts are readable on every CPU;
  - the probe's 0.2 s selftest pinned itself to CPU 9 and read that back.
- **One attempt; no retry to pass.** It takes about seven minutes, the corpus build aside.

## 2. The readings

Each run has the following readings. All are unprivileged; none is a performance counter.

- **P-idle's rate (and P-sib's in pass 2).**
  - **The loop:** a pinned thread runs one dependent register add an iteration, and every 2²⁰
    iterations stores `rdtsc` into a preallocated ring of 2¹⁷ samples. The ring is touched
    before the start and never dropped.
  - **The rate:** 2²⁰ over the ticks between samples, times the TSC's rate (calibrated against
    `CLOCK_MONOTONIC` in each run), in Giter/s.
  - **The run's reading:** the median over the samples inside the decode, from the exec to the
    reaping, and the medians of its first and last thirds.
- **Placement.** Every 2 ms, each of the decoder's tasks' state and last CPU (field 39 of
  `/proc/PID/task/TID/stat`).
  - **The decoder's threads:** those named `mibz2-*`.
  - **The busy-sibling share:** the share of (sample, thread) pairs in which the thread's
    sibling CPU also held another decoder thread. The same share among running threads only is
    reported beside it.
- **The clock during the run.** Every CPU's `scaling_cur_freq` every 10 ms. The reading is the
  median over the CPUs the decoder's threads were seen on, by sample, then over the run, and over
  its first and last thirds.
  - **A supporting reading, not a decisive one.** On this driver the value is a ratio of
    counters that both stop when a core halts, and it is only as fresh as its last update.
- **Before and after each run:**
  - every file under each CPU's `thermal_throttle` (the core and package throttle counts and
    times, and the power-limit counts where the kernel has them);
  - `thp_*`, `pgfault`, `pgmajfault` and `compact_*` from `/proc/vmstat`.
- **The child's own figures:** its `wait4` rusage (minflt, majflt, utime, stime, maxrss and the
  context switches), and its `AnonHugePages` from `/proc/PID/smaps_rollup` every 10 ms (the
  last and largest values).
- **Once (`preflight/static.json`):**
  - the transparent huge page settings (`enabled`, `defrag` and the rest);
  - every file of the RAPL zones under `/sys/class/powercap/intel-rapl*`, each with its error
    where it is not readable;
  - intel_pstate's settings;
  - the topology, the TSC flags and the thermal_throttle files of every CPU;
  - the kernel.
- **ab's own record, as always:** its sysfs snapshots before and after each run, from
  `--sysfs-state`.

## 3. The analysis (`bench/two_modes.py judge`)

### The admission

The attempt is INVALID, with its reasons, unless all of these hold:
- each pass has 60 runs and one probe record a run;
- each record comes from the archived executables, at 10 threads, ended in success, with the
  registered output (209,700,196 bytes, hash `a8830caee5c511a6`);
- each probe record falls inside its run's spawn and exit;
- each probe record is its slot of the fixed schedule;
- the decoder's mask was read back as `0-8,10-18` in every run;
- each probe was pinned and read back to its CPU, with nothing dropped;
- each probed run has at least 10 P-idle samples inside its decode.

### The classes

- **The class threshold:** a run is fast when the child's own wall is below **0.21 s**, else
  slow. It is the threshold of the reading that found the modes, fixed before this data.
- **The table:** each reading's median by class, over pass 1's **probed** runs.
- **The share of runs in which a count rose:**
  - a throttle count, core or package;
  - a power-limit count;
  - `thp_fault_alloc`.
- **Fewer than 5 runs in either class:** the outcome is **ONE MODE**, the table is described,
  and no column is matched.

### Each row's value

Each row compares the slow class with the fast class.

| row | values (the rules) |
|---|---|
| P-idle's rate | **full**: slow ÷ fast ≥ 0.95; **half**: 0.40 to 0.65; **reduced**: 0.65 to 0.95; **below half**: under 0.40; **rising**: below 0.95 while the slow runs' first-third ÷ last-third median is under 0.85 |
| the decoder CPUs' clock (supporting) | **full**: slow ÷ fast ≥ 0.95; else **low, then rising** if the slow runs' first-third ÷ last-third median is under 0.85; else **reduced** |
| busy sibling | **most**: slow share ≥ 0.50 and 0.15 above fast; **same**: within 0.10; else **between** |
| throttle counts rise, power-limit counts rise | **yes**: rose in ≥ 50% of slow runs and 25 points more than in fast runs; **no**: less than 10 points more; else **between** |
| `thp_fault_alloc` in fast runs only | **yes**: rose in 50 points more of fast runs than slow; **no**: within 25 points; else **between** |
| system time's share grows | **yes**: slow minus fast ≥ 0.05; **no**: under 0.02; else **between** |

A row whose readings are missing (a kernel without power-limit counts, say) is **unread**. It
admits any column, and the outcome names it.

### The columns

The Pathfinder's table, with one change, explained after it:

| row | H1 ramp | H2 packing | H3 stop-clock | H4 pages | H5 power |
|---|---|---|---|---|---|
| P-idle's rate in slow runs | full **or rising** | full | half | full | reduced |
| the decoder CPUs' clock (supporting) | low, then rising | full | full | full | reduced |
| threads with a busy sibling | same | most | same | same | same |
| throttle counts rise | no | no | yes | no | no |
| power-limit counts rise | no | no | no | no | yes |
| `thp_fault_alloc` in fast runs only | no | no | no | yes | no |
| system time's share grows | no | no | no | yes | no |

**The change: H1 admits P-idle "rising".** The probe cannot be kept spinning between runs. ab's
gate before each run waits while any CPU is more than half busy (`others_busy`, hard-coded), so
a probe spinning in the gaps would stall the run. P-idle therefore starts with each run, a
moment before the decode. If the i9's cores share one clock, as client parts of this generation
are thought to, P-idle ramps with the decoder's cores under H1. The Pathfinder's "full"
prediction assumed a probe that was already warm.

### The outcome

The outcome is the first line of `judgement.txt` and `outcome.txt`:
- **MATCH Hk:** exactly one column admits every readable row. Any unread rows are named.
- **AMBIGUOUS:** more than one column admits them.
- **SIXTH MECHANISM:** no column admits them. **This is reported as such**, with each row's
  observed value as the description of what the mode is. That is still more than the record
  holds.
- **ONE MODE:** fewer than 5 probed runs in a class.
- **Resting on the supporting reading:** a match is marked so when its column would also admit
  a slow mode differing from the fast in nothing but the clock row. That is H1's case: only the
  clock row separates it from no difference at all.

### Reported beside it, never matched

- **The probe's own effect:** pass 1's fast count in probed runs (out of 40) and in unprobed
  runs (out of 20), with the unprobed runs' own table.
- **A busy sibling's price:** P-idle's rate in pass 2 over pass 1, in all runs and by class,
  and P-sib's rate over P-idle's in pass 2.
- **Pass 2's own table and rows.**
- **Persistence:** the run order as F and s, fast-after-fast against the fast share.
- **The gaps:** each pass's gap from ab's exit to its next spawn, against the source bundle's
  0.256 to 0.264 s.

### The sustained block's signature

The block is read on 500 ms blocks from 1 s to its last input read (the first second holds the
decoder's initial read-ahead). Each block's rate is decoded-equivalent: input bytes × 209,700,196
÷ 11,854,933, over 0.5 s. A block is **fast** when that rate reaches the class threshold's, one
corpus in 0.21 s (0.999 GB/s).

| signature | rule | predicted by |
|---|---|---|
| **fast and steady** | at least 80% of blocks fast | H1 ramp, H2 placement, H4 first faults: a long process ramps, spreads and faults once |
| **settles low** | at most 20% fast | H3 stop-clock, H5 power limit: continuous load holds the slow mode |
| **alternates** | between, with at least 2 switches | H3, H5: throttling or the limit cycling |
| **mixed** | between, with fewer switches | none |
| **too short** | fewer than 4 blocks | none (the DRY check's case) |

The signature is reported beside the table and never changes the table's match. The outcome
line appends it, with whether it is what the matched column predicts: for example "sustained
block: fast and steady (consistent with H1)", or "(not what H3 predicts)".

### What each outcome means

- **H1 or H4: a property of short, fresh processes.** The i9's burst figures say nothing about
  long decodes. Timed registrations on this harness need back-to-back decodes in one process,
  or a warm-up.
- **H3 or H5: the machine under load.** Long decodes live in the slow mode, and work per byte
  under a power budget is what sets throughput.
- **H2: placement.** Owning it (one decoder thread a physical core) is the remedy, the design
  property in §5 of the design.
- **A sixth mechanism, AMBIGUOUS or ONE MODE:** stated as observed. ONE MODE would itself
  say the probe's presence, or the day, changed the mix. The unprobed block reads the first.

## 4. Limits

- **The probe changes the machine it measures.**
  - **Capacity:** the decoder has 9 physical cores here, not 10. The busy-sibling row compares
    classes within this run, never with other bundles.
  - **Power:** a spinning core adds package power. Under H5 the probe itself may shift the mix,
    and the unprobed block is there to show it.
  - **Progress, not frequency:** P-idle measures a core's progress, its clock times its duty,
    whatever lowers either.
  - **No comparison:** none of this run's walls is comparable with another bundle's.
- **The probe cannot be warm before a run** (§3, the H1 change).
- **The sustained block reads progress at the input.** It reads the input's progress at about
  4 MB resolution, a bounded read-ahead in front of the output, and nothing about the last
  read-ahead's worth. It runs unpinned on all ten cores, unlike the passes. Its signature is a
  consistency check on the table, not a column.
- **The observers cost something.** The wrapper's sampler wakes every 2 ms on the decoder's
  CPUs, reading about a dozen stat files, and every 10 ms reads 20 sysfs files and the child's
  `smaps_rollup`. Each freq sweep's own duration is recorded.
- **ab's view of the child changes.** Its census now watches the wrapper, not the decoder, so
  `observed_threads` names the wrapper's threads. The decoder's own `ran_threads`, and the 2 ms
  placement, are what to read.
- **What the inputs can't decide:**
  - `scaling_cur_freq` is supporting only;
  - the throttle and power-limit counts are whatever the kernel exposes, and an unread row is
    named;
  - `/proc/vmstat` is system-wide;
  - a probe on an idle core cannot see a mechanism that acts per core.
- **One machine, one day.**
- **Narrowed claims from the design's §1** (the Optimiser's REVIEW 6756418444):
  - equal end-of-run clocks do not exclude events inside a run;
  - every arm showing both modes does not exclude effects of arm and state together.

  This run assumes neither.

## 5. The local check (DRY, untimed, never evidence)

`just bench-two-modes 1 CORPUS` on a 4-CPU machine without SMT:
- **The preflight:** REFUSED, as it must be there, recorded, and the check went on.
- **The sustained block:** three corpora, untimed; its signature is "too short" by
  construction.
- **The passes:** pass 1 ran three runs through the probe on the machine's last CPU, on the
  PPN schedule; pass 2 was skipped, since there is no sibling.
- **The judge:** labelled the attempt DRY, and refuses it as INVALID without `--dry`.
- **The bundle** was made.

The record is the pull request's.

## Reproduce

```text
just bench-two-modes 1 [CORPUS]      # the recipe's own check, anywhere: DRY
python3 bench/test_two_modes.py      # the analysis on synthetic attempts, each column's signature
cc -O2 -Wall -Wextra -pthread -o /tmp/p bench/two_modes_probe.c && /tmp/p selftest 0 200
```
