# Dispatch causality on the i9: protocol (diagnostic only)

**Status:**
- Registered with the freeze in `2026-11-01-dispatch-causality/`.
- The maintainer authorised the protocol, the instrumentation, the correctness checks,
  the freeze and an untimed dry run (reports 106–109). The real i9 capture waits for its
  own authorisation.
- **What it is:** a diagnostic of the shipped decoder's dispatch. It has no arm and no
  optimisation.
- **What it changes:** no MPMC queue, no split of the condition variable, no change of
  default. P74-C and the unrolled walk stay closed.

**The question (report 106):** does receiver ownership, or the serial wake handoff through
it, materially obstruct a worker that could otherwise receive work?

## 1. What the archive already shows (reproduced, read only: `bench/dispatch_reproduce.py`)

From the attribution run's ten four-worker full traces
(`docs/decoder-attribution-records`, `2026-11-01-decoder-attribution-i9-10900K/occupancy/wikidata-20261026/marks`):

| Figure | Report | Reproduced |
|---|---|---|
| Run-queue wait over every inter-decode gap, pooled | 106: 0.884 of 2.639 s, 33.5% | 0.884 of 2.639 s, **33.5%** |
| Zero-backlog observations by the fourth worker's last mark | 106: 149 `JobLock`, 198 `JobRecv` | 149, 198 (and 3 `Ready`, 3 `DecodeEnd`) |
| …the job claimed by that worker | 106: 60 / 149, 194 / 198 | 60 / 149, 194 / 198 |
| The 120 distinct gaps holding a zero-backlog `JobLock`, pooled run-queue wait | 106: 0.348 of 0.447 s, 78.0% | 0.348 of 0.447 s, **78.0%** |
| Median claim delay by workers decoding at enqueue (0–4) | 105: 49.5, 48.4, 61.6, 977, 1,221 µs | 49.5, 48.4, 61.6, 977, 1,221 µs |

- **These are encompassing-gap measurements, not fractions of individual claim delays.**
- **The 2.7% run-queue figure of the attribution run is inside `decode_block`.** It says
  nothing about the gaps between blocks.

## 2. Hypotheses, and what would distinguish them (registered)

| | Mechanism | Its signature in this capture |
|---|---|---|
| D1 | **Receiver ownership:** a worker waits for the receiver's mutex while another holds it, with a job available | the worker in `JobLock`→`JobRecv` and **asleep**, while another worker is in `JobRecv`→`JobGot` and a job is available |
| D1′ | **The serial wake handoff:** the mutex is free, but its next waiter isn't woken yet, with a job available | the worker in `JobLock`→`JobRecv` and asleep, with no owner, and a job available |
| D2 | **Scheduling:** a runnable worker isn't running | the worker runnable (woken or preempted) in any non-decoding phase. Split by whether an allocated CPU was idle, and by what occupied the four CPUs |
| D3 | **Channel wake:** the receiver's owner sleeps in `recv` while a job is available | owner in `JobRecv`→`RecvDone`, asleep |
| D4 | **FIFO backlog:** jobs wait because every worker decodes | a job available with no worker outside `decode_block` (job-seconds, apart from worker time) |
| D5 | **Publication, buffers, the shared lock:** delay after `DecodeEnd` or after `JobGot` | the worker in `DecodeEnd`→`Notified` or `JobGot`→`DecodeStart`, asleep on the shared state's lock, or running |

**The reader's path,** reported beside the hypotheses: for each block the waiting reader
takes next, from its `Ready` to its `Taken`. The segments are:
1. to the reader's wake;
2. to its running;
3. to its relocking;
4. to the take.

## 3. The configuration

- **The decoder:** the shipped one, `BzReader::parallel_with(File, w, 2w, Kernels::detect())`,
  through `examples/occupancy --child` (`Plan::Workers(w)` asserted).
- **The corpus:** `wikidata-20261026`, the walk trial's frozen file (`cf13f439…`).
  - `records` and `random` are decoded to their frozen hashes for correctness only.
- **The allocation:** CPUs 1–4, four physical cores, siblings idle. It is the attribution
  run's rule, with the whole process pinned (`taskset`).
- **Unchanged:** the queue, worker count, admission window, kernels, affinity, power
  settings, cancellation, shutdown, ordering and bounded memory.

## 4. The instrumentation (`cfg(feature = "trace")` only; reviewed in §8)

**The trace marks:**

| Where | Mark | Records |
|---|---|---|
| Scanner, the send | `Issued` … `Enqueued` | the send's bracket: the job is available somewhere inside it |
| Worker, the receiver | `JobLock` (attempt), `JobRecv` (acquired), **`RecvDone`** (recv returned, guard held), `JobGot` (guard released) | ownership is [`JobRecv`, `JobGot`) |
| Worker, its buffer | `JobGot`, **`BufLockGot`** (shared lock acquired), `DecodeStart` | |
| Worker, publication | `DecodeEnd`, **`PubLockGot`**, `Ready` (before `notify_all`), **`Notified`** (after) | |
| Reader | **`ReadLock`** (before the lock), `ReadWait`, **`CvWait`** / **`CvWoke`** around each wait, `Taken` | |
| Scanner, slot waits | `SlotWait`, **`CvWait`** / **`CvWoke`** around each wait, `SlotGot` | |
| Clock | **`ClockAnchor`** ×2 at the start and ×2 at the end (the second of each measured) | CLOCK_MONOTONIC read, then `sched_yield` |

**How marks are kept:**
- **Bounded per-thread buffers:** 32,768 marks a thread, allocated once at its first mark
  and never reallocated. A full buffer counts the drops and keeps its end mark's slot.
- **OS thread ids:** one `/proc/thread-self` read per thread.
- **The epoch on CLOCK_MONOTONIC,** read with the `Instant` epoch.
- **No per-event `/proc` reads, allocation or shared lock:** level 1 only (the timeline),
  without level 2's clocks.

**The kernel's events:**
- **The capture:** `perf record -k CLOCK_MONOTONIC -C 1,2,3,4`, on the four allocated CPUs.
  It records every task on them, ours and foreign.
- **The events:**
  - `sched:sched_switch`, `sched_wakeup`, `sched_wakeup_new` and `sched_waking`;
  - `syscalls:sys_enter_sched_yield` (for the anchors);
  - `syscalls:sys_enter_futex` and `sys_exit_futex`;
  - `power:cpu_idle`.
- **Each is probed first.** An unavailable essential event stops the run. An unavailable
  optional one (futex, idle) is recorded and not replaced.

**Scheduler states per thread:**
- **running:** switched in;
- **runnable:** from the first of `sched_waking` and `sched_wakeup`, or switched out with
  `R`;
- **asleep:** switched out otherwise.
- A switch-in from sleep with no wake recorded makes that sleep **unresolved**, not
  asleep.

**What the futex waits are:** each is attributed to the phase it occurs in, giving the
receiver's mutex, the channel, the shared lock, or the condition variable (reader and
scanner). Each phase's dominant address and its share are reported. Std's channel parks
each thread on its own address, so the channel's addresses are per thread.

## 5. Classification and decisions (registered)

**What is classified:** queued-work nondecoding time. That is each worker's time outside
`DecodeStart`–`DecodeEnd` while at least one job is available, from the send's
`Enqueued` to the receiver's `RecvDone`. It is classified by the worker's phase and its
scheduler state, every class split by whether an allocated CPU was idle:
- **receiver:** `rx.blocked_by_owner` (D1), `rx.handoff_asleep` (D1′),
  `rx.running_owner_held`;
- **runnable:** by phase (D2);
- **channel wake:** `recv.asleep` (D3);
- **the shared lock:** `state_lock.blocked` and `state_lock.running` (D5);
- **publication and buffer work:** `notify`, `post`, `publish`, `release`, `buffer` (D5);
- **unresolved:**
  - `unresolved.sched`;
  - `unresolved.clock` (within the alignment uncertainty of a boundary);
  - `unresolved.send_bracket` (no job surely available, one in the send's bracket).

**Reported apart:**
- time with no job available, by state;
- the FIFO backlog, as job-seconds with every worker decoding;
- the CPUs' occupants while a worker was runnable.

**The decision rules:**
- **D1 — promote an MPMC experiment** if the receiver-specific obstruction,
  (`rx.blocked_by_owner` + `rx.handoff_asleep` + `rx.running_owner_held`) ÷ aggregate
  worker time (w × span), meets both conditions:
  - its mean over the w = 4 runs is **≥ 5%**;
  - **at least 80% of the runs** each reach 5%.

  Its share with an allocated CPU idle is reported beside it. Obstruction with all four
  CPUs occupied is not idle-core waste.
- **D2 — otherwise, do not proceed to MPMC.** Report the ranked owners of queued-work
  nondecoding time:
  - runnable time with the CPUs' occupants (scheduling, placement, oversubscription);
  - the channel wake;
  - publication and the shared lock;
  - the FIFO backlog.
- **D5 — if publication, the reader's notification or the shared lock dominates,** name
  the mechanism, and change nothing in this trial.

## 6. Validity, and the stopping rule (registered)

**Perturbation:**
- Every instrumented run is paired with an uninstrumented one: the same binary at trace
  level 0, without perf. The order alternates.
- **Tier A** has all events. If its throughput or CPU-s/GB moves by more than **3%** over
  the first three w = 4 pairs, the capture restarts in **tier B**, with no futex events.
- **Peak RSS** is reported. The instrumented RSS includes the trace's buffers, so it is
  not a production figure.

**Clock alignment:**
- For each anchor, the offset is kernel `sched_yield` entry − (epoch + the mark's time).
- The correction is their mean.
- The uncertainty is their half-difference plus the larger read-to-syscall gap.
- **It must be ≤ 20 µs.** Time within the uncertainty of a mark or availability boundary
  is `unresolved.clock`.

**Completeness:**
- every thread's start and end, and no marks dropped;
- every traced thread's TID seen by the kernel;
- scheduler state known for ≥ 99% of queued-work nondecoding time.

**Coverage:** the classified share of queued-work nondecoding time, **≥ 90%**, is a validity
objective. Unresolved time is reported, never assigned.

**The schedule:**
- **First:** 3 w = 4 pairs at the chosen tier, then 3 w = 1 pairs (the one-worker
  reference).
- **Expansion to 10 w = 4 pairs** happens only if, on the first three, every quality check
  holds: completeness, alignment, TID correlation, scheduler coverage and perturbation.
  **It never depends on what they show.**
- If a check fails, the run stops at three and reports which one.

**The interpretation's gate:** the decision of §5 is interpreted only if these all hold on
every run: coverage, perturbation, alignment and completeness. Otherwise it is reported as
provisional, with the failing checks named.

**Correctness:** every corpus decodes to its frozen hash at w = 1 and 4, and every capture
run's output hash is checked.

**C6:** the default-feature decoder must disassemble identically to the base commit's
(`bench/attribution_same_code.py`).

## 7. The command (after authorisation)

```
just bench-dispatch i9-10900K
```

- **Privileges:** sudo once, for perf's tracepoints and `-C`.
- **Corpora:** the walk trial's work directory on the i9, `target/walk-decoder/work/corpora`,
  hash-checked.
- **Archive:** `target/dispatch-results/i9-10900K.tar.gz`. It holds:
  - the run log, binaries' hashes, C6's report, manifest and environment;
  - the tracepoints' support;
  - `pairs.tsv`;
  - each instrumented run's marks, sidecar and `perf script` text (gzipped), and its
    analysis JSON. perf.data is deleted once scripted;
  - `quality*.json`;
  - `result.json`.

## 8. Instrumentation review (source level)

**The receiver mutex's lifetime is the same:**
- `RecvDone` is marked inside the guard's block, after `recv` returns.
- The guard is dropped at the block's end, as before. That adds one mark's time to the
  hold at trace level ≥ 1, and nothing at level 0.

**The other marks change no control flow:**
- `BufLockGot` and `PubLockGot` are marked inside critical sections that already exist.
- `Notified` is after `notify_all`.
- `CvWait` / `CvWoke` sit around the existing `wait` calls, inside the existing predicate
  loops.
- The notification policy is unchanged.

**Without the feature,** every mark compiles away. C6 checks the result on the whole
`mibz2` code.
