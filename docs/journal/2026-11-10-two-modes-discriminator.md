# 2026-11-10: The two modes' discriminator: registration (diagnostic only)

**Status: registered, NOT FROZEN, revised after the Researcher's REVISE** (#145 6757560564;
report `state/reports/two-modes-discriminator-6757368708` at `cf7eae3f`).
- **The recipe:** `just bench-two-modes` (`.just/two-modes.just`). It refuses every run but a DRY
  one until `bench/two-modes.frozen` names a commit.
- **Who freezes it:** only the Coordinator freezes and queues it, after the Researcher's review.

**What it is:** §3 of `2026-11-10-two-modes.md` (the Pathfinder's design), built as a C5
workstation recipe for the i9-10900K.

**What it is for:** at 10 threads the same decode takes about 0.15 s or about 0.30 s, and the
mode persists from run to run. This run collects bounded, honest observations of that.

**Its blocks, in order:**
1. **The gap block, primary.** It is the cheap discriminator: the same decode, with no probe,
   after a 0.25 s idle gap or with none (§1).
2. **The sustained block.** One long decode, with no probe.
3. **The probe passes, secondary.** These run the Pathfinder's §3 table.

**What it does not do:** it does not identify a mechanism.
- Its outcomes say which registered signatures the observations are **compatible with**. The
  signatures are predictions made under assumptions, not an exhaustive causal model, so joint
  mechanisms, measurement error or an unmodelled magnitude can break every one of them.
- Its results choose the next controlled intervention. They prescribe no warm-up, placement or
  power policy.

**Diagnostic only.** No merge gate, acceptance rule or adoption decision reads its outcome.
- It times nothing that is compared with another bundle.
- It makes no verdict about the favourable mode.
- Every record is kept. No run is excluded, replaced or retried because of its mode.

Everything below is fixed before any data:
- the class threshold;
- the analysis;
- the gap block's order and test;
- the probe schedule;
- the deadlines;
- the admission contract.

## 1. The run

### The harness and the probe core

- **The harness is the one that produced the two modes, unchanged.**
  - **The executables:** the decoder and harness are the run-fed leader's second attempt's own,
    not rebuilt. They are `ab` and `windows` from its archived bundle
    `run-fed-leader/20261109T201013Z.tar.gz`, on the `evidence` branch at `65ab4b95`.
  - **Their digests:** the bundle is checked by its SHA-256 (`7e36a5a7…`), and the two
    executables by theirs (`ce1eea53…`, `cf5d74a6…`). They are bench 3.6.0, built at
    `1af3888a` and frozen at `2fab12ca`.
  - **The corpus:** the bench's Wikidata sample (`82f65ace…`), built by that `windows`.
- **One arm, the default plan:** `mibz2-default33`, at 10 threads, with `--settle 0.25`,
  `--sysfs-state` and the default busy gate. This is the run-fed leader recipe's configuration,
  with its gaps between runs.
- **Through `ARM@EXE`:** for each run, ab spawns `bench/two_modes_probe.c`'s `child`, not
  itself. The wrapper:
  1. reads the counters;
  2. starts the probes in a probed slot;
  3. forks, sets the decoder's CPU mask in the child, and execs ab's own
     `child mibz2-default33 10 FILE`;
  4. reads the mask back, through `sched_getaffinity` and `Cpus_allowed_list`;
  5. samples until the decode exits, and reaps it with `wait4`;
  6. reads the counters again and appends one JSON line.

  The decode's stdout and stderr are ab's pipe and terminal, untouched.
- **The probe core:** CPU 9 and its sibling 19, the last physical core.
  - **The decoder's mask:** `0-8,10-18`. Both probe CPUs are out of it in every run, probed or
    not. This is an explicit intervention: the decoder has nine physical cores, not ten.
  - **ab's own placement:** ab itself runs on the decoder's mask (`taskset`).

### The preflight, the gap block and the sustained block

- **First, the preflight.** It is read-only, each collector bounded (TERM at 30 s, KILL 5 s
  after), and each collector's exit status is archived in `preflight/exits.txt`. The attempt is
  REFUSED and archived unless every point below holds:
  - every collector exited 0;
  - the host is the i9-10900K with 20 CPUs and 10 cores, and an unlimited CPU quota;
  - its declared settings hold: intel_pstate, `powersave`, `balance_performance`, turbo on;
  - every CPU has an invariant TSC (`constant_tsc` and `nonstop_tsc`);
  - CPUs 9 and 19 are one core's two siblings, and that core is the last;
  - the core and package throttle counts are readable on every CPU;
  - the probe's 0.2 s selftest pinned itself to CPU 9 and read that back.
- **Then the gap block: FIRST, with no probe, and the primary cheap discriminator** (the
  Pathfinder's CORRECTION, #145, at `0503526f5` on #313).
  - **Why it exists:** `2026-10-28-workstation-scaling.md` §4 (`just bench-regimes`, 6 threads, on
    the i9) found 5 of 12 runs slow after a 0.15 s idle gap, against 0 of 12 with no gap; the
    slow runs used about 1.5× the CPU per block. The design's earlier "not the gap" was read from
    a bundle whose gap never varied (0.256 to 0.264 s), so it ruled nothing out.
  - **What runs:** the default plan (`mibz2-default33`) at 10 threads, ab's own
    `child mibz2-default33 10 FILE` unchanged, one fresh process a run, unpinned on all 20 CPUs,
    with no probe. A small driver (`two_modes.py gaps`) spawns each run, captures the child's own
    record, and reaps it with `wait4`.
  - **The two conditions, 30 runs each:**
    - **gap (G):** the run is spawned 0.25 s after the run before exited, idle in between;
    - **no gap (Z):** it is spawned as soon as the run before exits (a few ms).
  - **Why 0.25 s:** it is the settle of the harness that showed the modes at 10 threads, whose
    records put 0.256 to 0.264 s between runs. §4's 0.15 s came from another harness at 6
    threads.
  - **The order:** drawn once from the fixed seed `20261111` (`GAP_SEED`), 30 of each:
    `GZGZZZGZGZGGZZGZZGGZGZZGZZZZGZGZGGZZGZZZGGGGZZGGGGZZZGGGGGZG`, recorded in the identity.
  - **Its deadline:** 600 s, then KILL 10 s later. It is expected to take about 30 s.
  - **Its admission:** 60 runs in the registered order, each succeeded with the registered
    output, each gap run's measured gap at least 0.245 s and each no-gap run's under 0.05 s, the
    block's exit 0.
- **Then the sustained block, with no probe** (the Pathfinder's addition, #145 6756520552).
  - **What runs:** one process, ab's own `child mibz2-default33 10`, unpinned on all 20 CPUs,
    decodes the corpus repeated 50 times end to end: 593 MB in, 10.5 GB out, hashed and dropped.
  - **The input, by content:** `two_modes.py concat` makes the input, or validates a cached
    one, by digest.
    - **The check:** the SHA-256 of the corpus streamed 50 times, computed from the corpus
      alone, must equal the file's own. A cache with any other digest is rebuilt; size is never
      the test.
    - **The record** (`sustained/concat.json`, and the identity): the corpus's digest and size,
      the copy count, the method (the corpus's bytes repeated, nothing between), and the
      expected and actual digests. The file is digested again after the block, to show it was
      unchanged.
    - **Where it lives:** beside the attempt's directory, never in the bundle. The record is
      enough to rebuild it.
  - **What it records:**
    - the input consumption (`/proc/PID/io` `rchar`) every 10 ms;
    - every CPU's `scaling_cur_freq` and the child's `AnonHugePages`, every 100 ms;
    - the counters before and after, and its `wait4` rusage.
  - **Its deadline:** 300 s, then KILL 10 s later. It is expected to take 8 to 16 s.
  - **A fixed 60 s rest follows** before pass 1. It is a fixed rest, not a demonstrated reset
    of the thermal or power state.
  - **No harness change:** the archived `ab child` runs unchanged. A multi-stream input is what
    the corpus already is: twelve streams one after another.

### The passes and their bounds

- **Pass 1, 60 runs:** P-idle on CPU 9 in 40 runs, and no probe in 20, with the same mask and
  the same readings.
  - **The order:** drawn once, before data, from the fixed seed `20261110` (`SCHEDULE_SEED`).
    It is
    `NPPPPPPPNPNPPPPNPPPPPPPPNPNPPPPPPNNNPPPNPNPNPNPNPPNNPNNNPPNP`, recorded in the identity.
  - **Why randomised:** a randomised order cannot alias a periodic nuisance, as the earlier
    fixed PPN could.
  - **What the contrast is:** the probed against unprobed fast/slow mix is a randomised but
    **descriptive** contrast. Persistence between runs widens its uncertainty beyond
    independent draws, and carryover from a probed run to the next is not excluded.
- **Pass 2, 60 runs:** P-idle on CPU 9 beside P-sib on CPU 19, in every run.
  - **What it is:** it follows pass 1 rather than being interleaved with it, so pass 2 against
    pass 1 is a **serial contrast**. It mixes the sibling's load with whatever changed in
    between.
  - **What it is not:** it is not a causal price of a busy sibling.
- **Every stage that can wait is bounded** by `bench/bounded.py` (master's, from #317).
  The command runs in a process group of its own, which gets TERM at the deadline and KILL
  after the grace, its descendants included. The deadlines are `DEADLINES` in `two_modes.py`:

  | stage | deadline + grace |
  |---|--:|
  | fetches | 120 + 10 s |
  | corpus build | 900 + 10 s |
  | the concatenation | 600 + 10 s |
  | the sustained block | 300 + 10 s |
  | each pass, the archived ab's busy-gate waits included | 900 + 10 s |
  | the judge | 300 + 10 s |

  - **A timeout (124)** is recorded, never retried, and makes the attempt INVALID. No pass
    follows a pass that did not complete.
  - **A group that cannot be emptied (125)** stops the attempt there.
  - **The whole attempt:** its bound is the sum, about 60 minutes. It usually takes about 8.
- **One attempt; no retry to pass.**

## 2. The readings

Each probed run has the readings below. All are unprivileged; none is a performance counter.
Each is read only where its coverage holds, and is otherwise **missing**, never imputed.

- **P-idle's progress (and P-sib's in pass 2).**
  - **The loop:** a pinned thread runs one dependent register add an iteration. Every 2²⁰
    iterations it stores `rdtsc` into a preallocated ring of 2¹⁷ samples, touched before the
    start and never dropped. It also records its own CPU time.
  - **The reading: elapsed-time progress.** Take every sampling interval whose two ends lie
    inside the decode (exec to reaping). The reading is their iterations over the TSC span from
    the first such sample to the last, in Giter/s.
  - **Coverage:** that span over the decode window must be at least 0.90, or the run has no
    progress reading.
    - A pause inside the decode lowers the progress.
    - A pause across the decode's start or end lowers the coverage.
  - **Recorded beside it:** the gaps at each boundary, the longest interval, the first and
    last thirds' progress, and the median interval rate. The median interval rate is an
    **active-loop summary only**: one long pause does not move it.
  - **What the TSC cannot say:** why progress stopped (descheduled, halted or slowed). The
    probe's CPU-time share is recorded so that descheduling can be told apart.
- **Placement, as a running proxy.** Every 2 ms, each of the decoder's tasks' state and last CPU
  (fields 3 and 39 of `/proc/PID/task/TID/stat`). The decoder's threads are those named
  `mibz2-*`.
  - **The reading:** the share of observations of a **running** (R) decoder thread in which
    another decoder thread on its sibling CPU was also seen running in the same sample.
  - **Why only running threads:** field 39 is the CPU last run on, so a sleeping thread's last
    CPU is not occupancy.
  - **Its limits:** the task files are read one after another, not as one snapshot. The share
    is an asynchronous proxy for simultaneous occupancy, not proof of contention.
  - **Coverage:** at least 20 running observations a run.
  - **Reported only:** the earlier last-CPU coincidence among all tasks.
- **The clock (supporting).** Every CPU's `scaling_cur_freq`, every 10 ms.
  - **The reading:** at each frequency reading, the median over the CPUs where the placement
    sample nearest it (within 2 ms) saw a running decoder thread. It is not the union of every
    CPU the run ever visited.
  - **Coverage:** at least 3 aligned readings a run.
  - **Supporting, not decisive:** intel_pstate's `scaling_cur_freq` is an average over the
    kernel's recent updates, not an instantaneous clock. It does not establish whether the
    cores share one clock.
- **Before and after each run:**
  - every file under each CPU's `thermal_throttle`: the core and package throttle counts and
    times, and the power-limit counts where the kernel has them;
  - `thp_*`, `pgfault`, `pgmajfault` and `compact_*` from `/proc/vmstat`. These are
    system-wide, reported and never attributed to the child.
- **The child's own figures:**
  - its `wait4` rusage: minflt, majflt, utime, stime, maxrss and the context switches;
  - its `AnonHugePages` from `/proc/PID/smaps_rollup` every 10 ms, the last and largest value.
- **Once** (`preflight/static.json`):
  - the THP settings;
  - every RAPL zone file, with its error where it is not readable;
  - intel_pstate's settings;
  - each CPU's topology, TSC flags and thermal_throttle files;
  - the kernel.
- **ab's own record, as always:** its sysfs snapshots before and after each run.

## 3. The analysis (`bench/two_modes.py judge`)

### The gap block's reading (primary, fixed before data)

- **Per condition:** the fast and slow counts (fast: the child's wall below 0.21 s), the
  child's CPU per block (CPU over the corpus's 431 blocks; all runs, fast runs and slow runs),
  the median wall and the measured gaps.
- **The test:** the two-sided Fisher exact test of slow against condition, at level 0.05.
  - Runs are serially correlated (the modes persist), so the p value's independence assumption
    does not hold. It is reported as registered and read as descriptive.
  - The slow counts by the previous run's condition (G→G, G→Z, Z→G, Z→Z) are reported as a
    carryover check.
- **The outcome:**

  | outcome | rule | what it means |
  |---|---|---|
  | **THE GAP GOES WITH THE SLOW MODE** | p < 0.05 and the gap condition's slow share is higher | the no-gap condition holds the fast mode: the slow mode is a state a fresh-process, idle-gap harness creates, compatible with H1 (a ramp or an idle-state wake). The registered i9 comparisons taken with a gap read a state a continuous real decode may never meet. It does not identify the wake mechanism |
  | **BOTH CONDITIONS SHOW THE MODE** | p ≥ 0.05 and at least 5 slow runs in each condition | the gap is not the trigger |
  | **NO GAP GOES WITH THE SLOW MODE** | p < 0.05 and the no-gap condition's slow share is higher | the reverse association, reported as observed |
  | **NO SLOW MODE IN THE BLOCK** | fewer than 5 slow runs in all | the mode did not appear in this block |
  | **NOT SEPARATED** | otherwise | neither shown |
  | **INSUFFICIENT** | fewer than 30 runs in a condition | the block is incomplete |

- **First in the outcome line:** the gap block's outcome comes first, then the probe table's
  (secondary), then the sustained block's shape.

### The admission contract

**One contract, `admission_problems`, enforced by the judge on every judgement**, the recipe's
own and any replay. The attempt is INVALID, with its reasons, unless every clause holds.
Missing or contradictory evidence is never admitted.

- **Provenance:**
  - a frozen commit, the inputs matching it, and every source's digest at the head equal to
    its digest at the freeze;
  - the harness bundle's commit, path and digest;
  - a clean tree before and after.
- **Gated stages:** the preflight exited 0 with a PASS judgement, the gap block and the
  sustained block exited 0, and each pass's `ab run` exited 0.
- **The gap block:** its order, seed, gap and count are the registered ones.
- **Identities before and after:**
  - `ab`, `windows`, the probe and the corpus are each unchanged across the run;
  - `ab` and `windows` are the archived executables, and the corpus is the registered one;
  - the sustained input is the corpus repeated (expected digest = actual digest = digest
    after);
  - the sustained decode emitted 50 corpora's bytes.
- **The registered configuration:** the arm, threads, settle, sysfs state, rounds, layout,
  passes, schedule and seed.
- **Every run:**
  - 60 runs a pass, each from the archived executables, at 10 threads, ended in success, with
    the registered output (209,700,196 bytes, hash `a8830caee5c511a6`);
  - the runs ordered, each run's spawn after the run before's exit;
  - one probe record a run, inside its run's spawn and exit, its timestamps ordered (start,
    fork, exec, exit, end), and in its slot of the schedule;
  - the decoder's mask read back as `0-8,10-18`;
  - each probe pinned and read back to its CPU, with nothing dropped.

In a DRY check the registered-host and freeze clauses are waived. The consistency clauses are
not.

### The classes

- **The class threshold:** a run is **fast** when the child's own wall is below **0.21 s**, else
  slow. This is the descriptive separator that found the modes, fixed before this data.
- **What is classed:** pass 1's **probed** runs.
- **The floor:** `MIN_CLASS = 5` is a display floor, not a precision guarantee. With fewer probed
  runs than that in either class, the outcome is **INSUFFICIENT CLASS SUPPORT**. The rows are
  described and no signature is assessed. Four fast and 36 slow runs do not show one mode.
- **The table:** each reading's median by class, with each row's number of observations in each
  class.

### Each row's value

Each row compares the slow class with the fast class. The cutoffs are **unvalidated descriptive
thresholds**, fixed before data and reported beside the raw values.

| row | values (the rules) |
|---|---|
| P-idle's elapsed-time progress | **full**: slow ÷ fast ≥ 0.95; **half**: 0.40 to 0.65; **reduced**: 0.65 to 0.95; **below half**: under 0.40; **rising**: below 0.95 while the slow runs' first-third ÷ last-third median is under 0.85 |
| the decoder CPUs' clock (supporting) | **full**: slow ÷ fast ≥ 0.95; else **low, then rising** if the slow runs' first-third ÷ last-third median is under 0.85; else **reduced** |
| running threads with a running sibling (proxy) | **most**: slow share ≥ 0.50 and 0.15 above fast; **same**: within 0.10; else **between** |
| throttle counts rise, power-limit counts rise | **yes**: rose in ≥ 50% of slow runs and 25 points more than in fast runs; **no**: less than 10 points more; else **between** |
| the child's huge pages in fast runs only | **yes**: `AnonHugePages` > 0 in 50 points more of fast runs than slow; **no**: within 25 points; else **between** |
| system time's share grows | **yes**: slow minus fast ≥ 0.05; **no**: under 0.02; else **between** |

- **Unread:** a row with no observations.
- **Insufficient:** a row with fewer than 5 observations in either class.

### The signatures and their decisive rows

Decisive rows are marked **(d)**.

| row | H1 ramp | H2 packing | H3 stop-clock | H4 pages | H5 power |
|---|---|---|---|---|---|
| P-idle's progress in slow runs | full or rising | full | half **(d)** | full | reduced **(d)** |
| the decoder CPUs' clock (supporting) | low, then rising **(d)** | full | full | full | reduced **(d)** |
| running busy sibling (proxy) | same | most **(d)** | same | same | same |
| throttle counts rise | no | no | yes **(d)** | no | no |
| power-limit counts rise | no | no | no | no | yes **(d)** |
| the child's huge pages in fast runs only | no | no | no | yes **(d)** | no |
| system time's share grows | no | no | no | yes **(d)** | no |

- **H1 admits P-idle "full or rising".** P-idle starts cold with each run. The archived ab waits
  before each run while any CPU is more than half busy, so a probe spinning between runs would
  stall it. Prewarming would also be a new intervention.
  - If the cores share one clock, P-idle ramps with the decoder's under H1. This is a
    conditional compatibility prediction.
  - An isolated idle core need not track a decoder core.

**Each signature's status:**
- **contradicted:** a read row's value is one it does not predict;
- **untestable:** none of its decisive rows is read;
- **compatible in part:** every read row is admitted, and some decisive rows are missing (named);
- **compatible:** every read row is admitted, and every decisive row is read.

**Missing evidence never makes a signature compatible.** With the clock unread, H1 is
untestable, not matched.

### The outcome

The probe table's outcome follows the gap block's in the first line of `judgement.txt` and
`outcome.txt`:
- **COMPATIBLE WITH Hk[, Hj]:** with any part-compatibility and the untestable signatures
  named. When H1 is among them, it is said to rest on the supporting clock reading: its only
  decisive row.
  - **Without power-limit counts, H5 can be at most compatible in part.** Low clock with reduced
    idle-core progress describes a low-progress state compatible with power limiting; it does
    not identify it.
  - Equally, a throttle count that did not rise does not by itself prove that no transient
    event limited throughput.
- **NO REGISTERED SIGNATURE MATCHES:** with each row's observed value. A mechanism outside the
  five, a joint one, or measurement error would show here. None of them is inferred from it.
- **INSUFFICIENT MEASUREMENT SUPPORT:** no signature's decisive rows were read.
- **INSUFFICIENT CLASS SUPPORT:** fewer than 5 probed runs in a class.

### The sustained block: input consumption, error-aware

The block is reported as **raw input consumption**: bytes read in each 100 ms window. It is
classified only by a rule whose error bound comes from the chunking.
- **Input only:** no compression ratio converts it, and no burst threshold classifies it.
- **The windows:** 1 s windows from 1 s (after the read-ahead fill) to the last input read, each
  window's rate in input bytes/s. At least 4 windows are needed.
- **The error bound:** `e = (2C + 2 × 0.01 s × rate) / 1 s`, where C is the largest chunk read
  after the fill. A window's arrived bytes differ from the bytes consumed in it by less than one
  chunk at its edges plus one chunk of refill jitter, and the 10 ms sampling adds two samples'
  worth.
- **A change:** a difference between consecutive windows beyond 2e.

| shape | rule |
|---|---|
| **constant within error** | the windows' range ≤ 2e |
| **alternating beyond error** | at least two reversals among the changes |
| **a sustained change, down (up)** | every change in one direction, or no step change but last and first differ beyond 2e |
| **varies beyond error** | otherwise |
| **too short** | fewer than 4 windows (the DRY check's case) |

- **Validated on synthetic trajectories** with 4 MiB chunks and four chunk phases each:
  - constant rates from 0.68 to 1.3 GB/s decoded-equivalent all read **constant within error**,
    including 0.98 and 1.00, which the earlier rule called alternating;
  - a 4 s alternation reads **alternating**;
  - a drop from 1.3 to 0.68 reads **a sustained change, down**.
- **Kept separately:** the end-to-end decoded bytes over the block's span.
- **The shape is reported beside the table and enters no signature.**
- **Follow-up predictions, not conclusions:**
  - a slow mode that holds or recurs under continuous load would challenge a purely
    short-process account;
  - a constant trajectory would not, by itself, validate one.

### Reported beside it, never matched

- **The probe's effect, descriptively:** pass 1's fast count in probed runs (out of 40) and in
  unprobed runs (out of 20), with the unprobed runs' own table.
- **The serial contrast:** P-idle's progress in pass 2 over pass 1, in all runs and by class,
  and P-sib's over P-idle's in pass 2.
- **Pass 2's own table, rows and signatures.**
- **System-wide `thp_fault_alloc` and `thp_fault_fallback`.**
- **The last-CPU sibling coincidence.**
- **Persistence:** the run order, and how often a fast run follows a fast one against the fast
  share.
- **The gaps:** each pass's gap from ab's exit to its next spawn.

### What an outcome would suggest: follow-up predictions, not remedies

- **Compatible with H1 or H4:** a test of back-to-back decodes in one process, against fresh
  processes. H4-compatible observations do not make the effect only a short-process one:
  long-running processes can allocate and fault again, and huge pages' TLB benefit endures
  beyond the first faults.
- **Compatible with H3 or H5:** a test of throughput under sustained load, with the
  sustained block's trajectory as one prediction it must meet.
- **Compatible with H2:** a controlled placement intervention. The proxy cannot prove contention.
- **No registered signature, or insufficient support:** reported as observed. The raw records
  choose what to measure next.

## 4. Limits

- **The probe changes the machine it measures.**
  - **Capacity:** the decoder has 9 physical cores here, not 10.
  - **Power:** a spinning core adds package power.
  - **Progress, not frequency:** P-idle measures a core's progress.
  - **No comparison:** none of this run's walls is comparable with another bundle's.
- **The observers cost something.**
  - **What they read:** the wrapper's sampler wakes every 2 ms on the decoder's CPUs and reads
    about a dozen stat files. Every 10 ms it also reads 20 sysfs files and the child's
    `smaps_rollup`. Each frequency sweep's own duration is recorded.
  - **ab's census:** it now names the wrapper's threads. The 2 ms placement and the decoder's
    `ran_threads` replace it.
- **The sustained block reads input consumption** at about 4 MB resolution, behind a bounded
  read-ahead, unpinned on all ten cores, unlike the passes.
- **One machine, one day.**
- **Narrowed claims from the design's §1** (the Optimiser's REVIEW 6756418444):
  - equal end-of-run clocks do not exclude events inside a run;
  - every arm showing both modes does not exclude effects of arm and state together.

  This run assumes neither.

## Related evidence (context only, not diagnostic)

**`par-scan/20261110T030608Z`** (on `evidence` at `727674e1`; summarised with the Pathfinder's
addendum in LOG 6758018574 on #145). This workstation run ran 180
cells in three long-lived processes, with plain workers at 1 to 10 threads, and no twofold mode
appeared in it.
- **How it bears on the five:** it fits the short-process mechanisms (H1, H2, H4).
- **Why it is not evidence here:** its arm, its process shape and its plan all differ from the
  runs that showed the modes.
- **What it changes:** it is cited for context only. It changes none of the rules above, and
  this run's outcome is judged without it.

## 5. The local checks (DRY, untimed, never evidence)

### Before the revision (head `3588d35d`)

`just bench-two-modes 1 CORPUS` ran on a busy 4-CPU machine without SMT. **Pass 1 did not run
inside the recipe.**
- The preflight was REFUSED there, recorded, and the check went on.
- The sustained block ran three corpora.
- The archived ab's gate then waited about 45 minutes for another agent's job to free a CPU,
  and the pass was stopped by hand.
- Three direct invocations of the wrapper, outside the recipe, exercised the pass's plumbing.

That gap is why every pass now has a deadline.

### After the revision

**The full in-recipe DRY check at `e5b76ecc`, before the gap block was added.** It ran in an
idle window: `just bench-two-modes 1`.
- **Every stage up to pass 1 ran inside the recipe:**
  - the archived bundle fetched and checked by digest;
  - the corpus built by the archived `windows`, SHA-256 checked;
  - the preflight REFUSED, as it must be on that host, recorded;
  - the 3-copy sustained input verified by content before and after;
  - the sustained block exited 0;
  - pass 1 exited 0 under its deadline: three runs in the schedule's first three slots, each
    with the registered output, the mask read back, and P-idle's coverage 0.999.
- **The admission contract** found no problems in DRY mode, and the bundle was made.
- **Its bundle** is the pull request's record (SHA-256 `c49bf2b1…`).

**The gap block's driver, alone** (`two_modes.py gaps`, untimed, on a small bzip2 input with the
archived `ab`):
- six runs in the registered order's first six slots;
- the measured gaps were 0.2501 s for G and 0.0001 s for Z;
- every run exited 0;
- the judge's gap checks found no problems.

**A full in-recipe DRY check with the gap block is still to come.** It waits for an idle window,
which the shared machine did not have for an hour. Like any DRY check, it proves plumbing only,
never discrimination on the target.

## Reproduce

```text
just bench-two-modes 1 [CORPUS]      # the recipe's own check, anywhere: DRY
python3 bench/test_two_modes.py      # the analysis on synthetic attempts and the counterexamples
cc -O2 -Wall -Wextra -pthread -o /tmp/p bench/two_modes_probe.c && /tmp/p selftest 0 200
```
