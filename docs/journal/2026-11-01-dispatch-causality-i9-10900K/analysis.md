# Dispatch causality on the i9-10900K: results, reproduction and interpretation

**The run:**
- the frozen protocol (`2026-11-01-dispatch-causality-protocol.md`), with amendment 1;
- pin `48f8cda`, head `64483e8`;
- authorised in reports 110–112.

**The archive:** `archive/`, exactly as recorded (`i9-10900K.tar.gz`, sha256 `d73d330b…`).

**Two kinds of reading below:**
- **§1–§3 are the registered result,** and its independent reproduction.
- **§4–§6 are descriptive,** post hoc readings of the same evidence. They are not
  decision rules, and they change no threshold.

## 1. The registered decision

| | |
|---|---|
| **Outcome** | **do not proceed to MPMC**: receiver obstruction below the materiality threshold in every run |
| Receiver obstruction (D1: `rx.blocked_by_owner` + `rx.handoff_asleep` + `rx.running_owner_held`) ÷ aggregate worker time | mean **1.90%** over 10 runs (1.39–2.59%), against 5%; **0 of 10** runs reach 5% |
| …with an allocated CPU idle | 1.79% |
| Validity | every check holds on every w = 4 run: runs present, coverage, perturbation, alignment, completeness, TIDs seen by the kernel, scheduler state, trace completeness |
| Expansion | the quality checks passed on the first three w = 4 pairs, so the run expanded to 10 |

**The evidence's quality:**
- **The kernel's stream** was complete on CPUs 1–4 before the capture (`stream-probe.json`:
  idle switches paired 24/24, 12/12, 4/4, 21/21; nothing lost).
- **Tier A:** all 8 tracepoints, futex events included.
- **Perturbation, instrumented ÷ uninstrumented:**
  - w = 4: throughput 1.006, CPU-s/GB 1.013;
  - w = 1: 1.010 and 0.999;
  - all within 3%.
- **Per run:**
  - coverage 98.3–98.6%;
  - scheduler state known for 100% of queued-work nondecoding time;
  - alignment uncertainty 0.19–0.28 µs;
  - 0 events lost, 0 unreadable lines, 0 missed switches, 0 missing wakeups;
  - 6 unresolved-tid `sched_switch` events a run: the threads' exits, kept.
- **Same machine code:** 104 `mibz2` functions, 32,298 instructions, as the base.
- **Correctness:** all three corpora decode to their frozen hashes at w = 1 and 4, and every
  capture's output was 207,000,624 bytes.
- **Throughput,** uninstrumented w = 4: a median of 0.2377 s wall and 0.736 CPU-s for the
  207 MB slice, about 870 MB/s.

## 2. Independent reproduction

The frozen analysis (`bench/dispatch.py` at the pin) was re-run on every raw capture: the
marks, `perf script` text and `perf report --stats` in `archive/capture/`.
- All 13 run analyses (10 at w = 4, 3 at w = 1) match the archived ones field for field.
- `result.json` from the reproduced runs is **identical** to the archived one.

## 3. Where the time goes (registered classes, pooled over the 10 w = 4 runs)

**Aggregate worker time W** = 9.427 worker-s, over a 2.357 s span.

| Worker time | Share of W |
|---|---:|
| Decoding | ≈ 69.6% |
| **No job available** (nondecoding; reported apart) | 18.9%, of which asleep 17.4% |
| **Queued-work nondecoding** (classified) | 11.5% |

The classified 11.5%, broken down:

| Class | % of W | % of queued-work time |
|---|---:|---:|
| `runnable.mutex_wait`, an allocated CPU idle | 5.65 | 49.2 |
| `runnable.mutex_wait`, all four busy | 2.40 | 20.9 |
| `rx.blocked_by_owner` (D1) | 1.25 | 10.8 |
| `runnable.recv` | 1.06 | 9.3 |
| `rx.handoff_asleep` (D1) | 0.65 | 5.7 |
| notify, recv running, release, state lock, publish, buffer | ≈ 0.3 | ≈ 2.8 |
| unresolved (send bracket, clock) | 0.18 | 1.6 |

**The ranked owners (D2):**
- runnable (scheduling) **80.2%**;
- receiver **16.5%**;
- unresolved 1.6%;
- notification 0.7%;
- everything else under 0.5%.

**FIFO backlog** (job-seconds with every worker decoding): 0.675 s over the 10 runs.

**The exploratory share** (`receiver_associated_exploratory`: the registered obstruction plus
runnable time in the receiver mutex's wait phase):
- 9.95% (8.97–11.04%), at or above 5% in all 10 runs;
- **not a criterion,** by decision (a).
- §4 shows what it is made of.

**The one-worker reference** (3 runs):
- receiver obstruction 0;
- queued-work time 1.7–2.6% of worker time;
- no-job time 11.5–12.3%;
- coverage 91–94%;
- every check holds.

## 4. Descriptive: what "runnable" is here (not registered)

`analysis/placement.py` and `analysis/waker.py` examine every worker interval from wake to
run in the receiver's phases. They use the frozen loaders and reconstruction.

**It is placement, not a shortage of cores:**
- **The long intervals dominate.** 405 intervals of more than 0.5 ms carry 782 ms of the
  950 ms runnable in the receiver's phases (82%).
- **A CPU sat idle.** For 70% of that time (549 ms), at least one of the four allocated
  CPUs was idle.
- **The typical wait** of a long interval is a median of **2.36 ms**, about one scheduler
  slice, behind:
  - a decoding worker (416 ms);
  - or the scanner (334 ms).

**The woken worker was placed on its waker's CPU:**
- 87% of the long runnable time (682 of 782 ms) had the wakeup's target CPU equal to the
  CPU that issued `sched_waking`.
- 96% (751 ms) is in the mutex-wait phase, **woken by another worker**: the previous owner
  of the receiver, whose unlock (`futex` wake) placed the next waiter on its own CPU. It
  then went on decoding there for a slice, while another allocated CPU stayed idle.
- Woken by the scanner in `recv`: 30 ms.

**The reading:**
- The registered receiver obstruction is small (1.9%). Workers seldom sleep behind an owner.
- But the mutex's serial handoff decides **who wakes whom**. The kernel's wake-affine
  placement then queues the woken worker behind its waker, a decoding worker.
- Most of the 8% of worker time "runnable in the mutex wait" is that combination: a
  handoff wake plus placement, with idle capacity present. It is not competition for a
  full set of cores.
- **This does not show that an MPMC queue would recover it.** With a multi-consumer channel
  the scanner, not a worker, would wake an idle receiver. The same placement behind the
  scanner would be possible: 334 ms of long waits were already behind the scanner.
- Mutex ownership, as registered, is not the bottleneck. **Wake placement is the largest
  owner of queued-work time,** and it is recoverable only through scheduling choices.
  Those are outside this trial, and nothing is authorised.

**Wake-ups onto an idle CPU** (`analysis/explore.py`):
- They cost little: a median of about 42 µs onto an idle CPU in its deepest state (C-state
  index 3), and about 2 µs from the shallow states.
- The C-states' exit latency is **not** a material part of the delay.

## 5. Descriptive: the no-job time (18.9% of W; not registered)

While a worker was not decoding and no job was available, the scanner was:

| The scanner | Share of the workers' no-job time |
|---|---:|
| scanning, running | 43.7% |
| in its slot wait, **runnable but not running**, the window full | 23.2% |
| in its slot wait, runnable but not running, the window open | 19.2% |
| in its slot wait, asleep, the window full | 7.2% |
| other | 6.7% |

**Over the whole span:**
- the scanner was runnable but not running in its slot wait **36%** of the time, asleep
  there 33%, and running a scan 27%;
- the admission window (8 blocks) was full **60%** of the time.

**The reading:**
- Part of the workers' idle time is the scanner being slow to *get a CPU*, not slow to
  scan.
- The scanner is woken by every `notify_all`: workers' publications and the reader's
  takes share `shared.cv`. With the window full, most of those wakes find nothing to do.
- The window's 60% full points at the ordered reader holding slots (§6).
- These are observations for the next decision. Nothing here was registered, and nothing
  is changed.

## 6. The reader (observational, by decision (c))

**Resolved blocks:** for the 962 blocks whose path resolved, publication to the take totals
140.7 ms. Almost all of it (134.4 ms) is the reader woken but not yet running: a median of
about 40 µs, with a long tail.

**Unresolved blocks:**
- A further 331 blocks (306.6 ms, a median of about 1.06 ms) did not fit the
  publish-wake-run-relock order.
- The reader was mostly already woken, by the condition variable it shares with the
  scanner.
- They are not decomposed by this instrument.

**Futex mapping:**
- the receiver mutex: one address, 100% of its waits;
- the reader's and the scanner's condition-variable waits are on the shared `cv`.

## 7. Limitations

- One machine (i9-10900K, kernel 6.8.0-57, `powersave` governor), one corpus (Wikidata
  20261026), w = 4 on CPUs 1–4, siblings idle.
- §4–§6 are post hoc. Their figures are descriptive, chosen after seeing the data, and
  carry no registered threshold.
- **The placement attribution** reads the wakeup's `target_cpu` against the CPU that issued
  `sched_waking`, and the idle state of the four allocated CPUs from `sched_switch`. It
  does not see the scheduler's internal reasons for a choice.
- **The reader's unresolved blocks** (26% of its next-block waits) are counted, not
  explained.
- **Instrumented runs** add a mark inside the receiver guard. Perturbation stayed within
  1.4%.

## 8. Closure (after the reviews: external reports 113–124)

The investigation is closed. No further dispatch experiment is planned. **Two distinct
conclusions are preserved, and neither is to be read as the other:**

1. **MPMC receiver replacement failed its registered test.**
   - Direct receiver obstruction was 1.90% of aggregate worker time, against 5%, and
     0 of 10 runs reached it.
   - No MPMC implementation is authorised.
   - The exploratory receiver-associated share (9.95%) includes runnable time. It is not
     direct obstruction, and it is not a second criterion.
2. **Wake-affine placement remains an untested hypothesis.**
   - The long runnable waits on the waker's CPU, with another allocated CPU idle (§4),
     are credible evidence of a scheduling interaction.
   - They do not establish recoverable throughput.
   - The negative MPMC result is **not** evidence against every dispatch-related
     improvement.

**For the research register:** *MPMC rejected under the registered direct-obstruction
criterion; wake-affine placement remains an unresolved, independently testable
hypothesis.*

**Parked:** MPMC, the split of the condition variable, and worker or thread affinity. If
affinity is revisited:
- the whole system (scanner, feeder, reader and workers) must fit the same four physical
  cores;
- one worker rigidly pinned per core is not automatically better;
- success is full-decoder throughput and CPU-s/GB, never wake latency alone.

A fifth core for the scanner changes the hardware budget, so it can't show better
four-core efficiency.

### The reviewers' exploratory figures, checked against the raw traces

| Figure (the reviews) | Checked | Notes |
|---|---|---|
| Runnable in the receiver-acquisition phase, 8.05% of W | **8.05%** (5.65 + 2.40) | `result.json` classes |
| …of which with an allocated CPU idle, 5.65 points | **5.65** | `runnable.mutex_wait.cpu_idle` |
| Runnable in the receive phase, 1.06% of W | **1.06%** | |
| Worker wake-ups matched to their next switch-in, 4,523 | **4,529** | `analysis/wake_occupancy.py`; a second `sched_waking` before the same switch-in is counted once |
| Median wake-to-run, a CPU idle at the wake / all busy | **39.9 / 10.5 µs** | |
| Over 100 µs, CPU idle / all busy | **3.7% / 37.6%** (reviews: 37.4%) | |
| p90 wake-to-run, all busy | **2.41 ms** | |
| Reader handoffs reconstructed / unresolved | **962 / 331** | §6 |
| Reader, publication to wake / relock / take | 2.3 / 2.4 / 1.7 ms in all; wake-to-running **134.4 ms** | §6 |
| Scanner runnable with the window open, during workers' no-job time | **19.2%** of no-job time, so **3.6% of W** | §5 |

**How the two views reconcile:**
- The reviews split wake-ups by occupancy *at the wake*.
- §4 measures the long waits that follow: a CPU often goes idle during such a wait, and
  the waiting worker isn't moved to it.
- Both hold. The long tail is the waits that start with all four CPUs busy and outlast the
  moment one of them frees.

**How to read the scanner:**
- Only the 19.2% with the window open (3.6% of W) is plausibly actionable starvation.
- Waking the scanner sooner while the window is full (23.2%) may accomplish nothing until
  a slot is freed.

**What none of these figures shows:**
- **Runnable but not running is not wasted capacity.** It becomes recoverable only if
  another allocated CPU was idle *and* a schedule exists that uses it without costing
  more elsewhere: cache locality, or the scanner and reader's own access to a core.
- **`target_cpu` equal to the waker's CPU shows where the kernel placed the thread, not
  why.**
