# External research report 155: the researcher's theoretical review of wake placement (candidate 2): fixed per-worker placement with movable support threads, capacity-aware opportunity accounting, and the scanner/reader notification split (verbatim)

A research input, not a finding of this repository.

- **Relayed:** by the maintainer, 2026-11-02, with reports 153, 154 and 156.
- **Source:** the independent researcher of reports 138–154.
- **In reply to:** the maintainer's instruction to treat each candidate in turn as report 154
  did. That instruction for this candidate was not relayed separately.

**Evidence cutoff:** 2 November 2026. Source review of the decoder at `13e9854`, the dispatch
investigation and its analysis scripts, and Linux documentation and source; no new run.

It is kept exactly as received (rendered text, cards flattened).

## The message, as received

`````text
## Candidate 2: wake placement and pipeline utilization

**I would narrow this to one concrete first intervention: give each decoding worker its own fixed core within the existing four-core allocation, while leaving the scanner, feeder and reader eligible to run across that same allocation.**

That is a **candidate for evaluation, not a recommendation to change the default**. It has a useful theoretical property: it makes the observed worker-on-worker placement pattern impossible, without changing the receiver, decoding algorithms or number of workers. Whether that improves completed throughput and CPU efficiency remains empirical.

The review also exposes an important qualification to the opportunity estimate: **the recorded 5.65% is waiting-worker time, not a direct measurement of recoverable idle-core capacity**. The existing analysis is valid, but we must not turn that percentage straight into a promised speedup.

I examined the production scheduling code at `13e9854`, the dispatch investigation and its analysis scripts, and Linux’s scheduling and affinity interfaces. This is source review and theoretical analysis; I did not change code or run a new experiment.

## 1. What the evidence actually identifies

The dispatch investigation separates two costs that are easy to conflate:

| Observation | Recorded result | What it establishes |
|---|---:|---|
| Direct receiver obstruction | **1.90% of aggregate worker time** | A comparatively small measured lock/handoff obstruction |
| Runnable in receiver-mutex acquisition, with an allocated CPU idle | **5.65% of aggregate worker time** | Ready-to-run workers sometimes coexist with idle allocated capacity |
| Long runnable intervals placed on the waker’s CPU | **87% of long-interval time** | Strong association between the wake relationship and placement |
| Long runnable intervals in mutex-wait phase, woken by another worker | **96% of long-interval time** | The worker-to-worker wake chain is particularly relevant |

The last two percentages concern the investigation’s **long intervals**, not every wake-up or all decoder time. These observations are descriptive findings preserved alongside—not substituted for—the registered negative result on receiver replacement. 

There is a further detail that helps locate the problem. The analysis reports that waits exceeding 100 μs were much more frequent when all four CPUs were busy **at the wake-up** than when a CPU was already idle: approximately **37.6% versus 3.7%**. Some long waits then persisted after a CPU became idle. 

So the hypothesis should not simply be:

> “Linux ignores an obviously idle CPU when a worker wakes.”

A more defensible formulation is:

> **A worker can be queued behind another active thread, then remain delayed as usable capacity appears elsewhere.**

That distinction matters when choosing a remedy.

## 2. The source explains why workers wake other workers

The production worker acquires the shared receiver mutex, receives a job, releases the mutex, and then decodes:

```text
lock shared receiver
    receive one job
unlock receiver
    obtain output buffer
    decode the block
    publish the result
repeat
```

The receiver guard’s scope ends **before decoding begins**. Thus the problem is not that one worker holds the receiver lock throughout a block. 

However, releasing a contended mutex can wake a waiting worker. Rust’s Linux futex-backed mutex implementation uses a one-waiter wake on contended unlock. The releasing thread is then free to continue into its own substantial decoding work. [doc.rust-lang.org](https://doc.rust-lang.org/nightly/src/std/sys/sync/mutex/futex.rs.html?utm_source=chatgpt.com)

That creates the relevant sequence:

```text
Worker A receives a job.
Worker A releases the receiver and wakes worker B.
Worker A begins decoding.
Worker B is placed on a CPU occupied by A or another active thread.
Worker B waits—even if another allocated CPU subsequently becomes idle.
```

The source establishes the wake opportunity; the recorded scheduling analysis supplies evidence that worker-to-worker wakes and long waits occur together. It does not establish that every such wake follows exactly this entire sequence.  

**The opportunity is therefore to change where ready workers can execute, while retaining the existing receiver.**

This is why the small direct-lock-obstruction result does not dispose of the placement hypothesis.

## 3. What Linux research resolves—and what it does not

I checked upstream Linux 6.8, the kernel family named in the dispatch record.

Its wake-up placement path does more than apply `wake_affine()`. The normal fast path subsequently calls `select_idle_sibling()`, which considers idle CPUs in the relevant cache domain. Later load balancing is another mechanism, with migration checks that include affinity and cache-hotness. Consequently, a trace showing “wakee target equals waker CPU” does **not** uniquely identify which internal heuristic determined the outcome. [GitHub](https://raw.githubusercontent.com/torvalds/linux/v6.8/kernel/sched/fair.c)

This was upstream source review, not a reconstruction of every patch and runtime setting in the captured Ubuntu kernel.

The theoretical lesson is important:

**We do not have to prove which scheduler heuristic is responsible before evaluating an intervention that removes the problematic placement possibility.**

Conversely, finding a plausible explanation in scheduler source would not prove that changing it improves this decoder.

The reported approximately 2.36 ms long-wait median should also remain a descriptive observation. It does not establish a fixed delay that every affected worker pays and that we can simply subtract from runtime. 

## 4. The opportunity needs better accounting than “5.65% is available”

This is the main refinement from reading the actual classification code.

For each interval, the analysis checks whether **any** allocated CPU is idle. It then credits that interval separately to each qualifying runnable worker. It also requires queued work, but does not cap the accumulated worker-time by the number of idle CPUs or available jobs. 

That is correct for its purpose: measuring workers’ states. It is not an error in the registered result.

But consider this hypothetical interval:

```text
One CPU is idle for 1 ms.
Two workers are runnable and waiting.
One queued job is available.
```

The recorded category can contain **2 worker-ms**. Only **1 CPU-ms** is available to fill immediately.

There is another qualification: the classifier assigns runnable state before checking receiver ownership. A runnable worker in the mutex-acquisition phase is not necessarily already able to acquire an unowned receiver and start an independent job. 

### A more useful opportunity quantity

For the particular worker-delay hypothesis, define:

- \(I(t)\): idle allocated CPUs;
- \(R(t)\): qualifying runnable workers awaiting a job;
- \(Q(t)\): queued, unclaimed jobs.

With common CPU eligibility and idealised zero-cost lock acquisition, an optimistic instantaneous amount of additional decoding activity is:

\[
H(t)=\min\!\left(I(t),R(t),Q(t)\right).
\]

Then:

\[
E=\frac{1}{T}\int_0^T H(t)\,dt
\]

expresses that opportunity in **average additional active cores**.

With different affinity masks, the calculation needs to match workers to idle CPUs on which they are actually eligible, rather than just taking the minimum of three counts.

This is my proposed analytical refinement, not a quantity already calculated in the archive. It describes the directly fillable capacity of the **recorded schedule under simplifying assumptions**. It is not a simulation of the changed schedule or a guaranteed speedup.

The existing 5.65% corresponds arithmetically to:

\[
4\times0.0565=0.226
\]

average waiting-worker equivalents. **That is an optimistic screening envelope for this category, not 0.226 cores we know how to recover.**

The scanner-delay category must also be kept separate: making the scanner run sooner can change when jobs become available, and the resulting gains cannot simply be added to worker-delay estimates.

### More activity must outweigh any increase in cost per byte

The relevant accounting identity is:

\[
\text{throughput}=
\frac{\text{aggregate CPU activity}}
{\text{CPU cost per decoded byte}}.
\]

For illustration—not a forecast—raising average activity from 3.1 to 3.3 cores gives about **6.5% more throughput if CPU cost per byte is unchanged**. If that additional overlap raises cost per byte by 5%, the throughput improvement becomes only about **1.4%**, with worse CPU efficiency.

This is why the walker-ladder finding matters to candidate 2: reducing scheduling gaps may expose more concurrent traversal, whose cost cannot be assumed unchanged.

**The goal is more completed decoding at acceptable CPU cost, not merely busier cores or shorter wake-up delays.**

## 5. The strongest first intervention: fixed worker placement, movable support threads

Here is the precise proposal I would retain.

Let the allowed allocation contain four logical CPUs on four distinct physical cores:

\[
C=\{c_0,c_1,c_2,c_3\}.
\]

Assign:

| Thread role | Allowed CPUs |
|---|---|
| Decoding worker 0 | \(\{c_0\}\) |
| Decoding worker 1 | \(\{c_1\}\) |
| Decoding worker 2 | \(\{c_2\}\) |
| Decoding worker 3 | \(\{c_3\}\) |
| Scanner, feeder and reader | The original set \(C\) |

**“Its own core” means a fixed eligibility assignment, not an exclusive reservation.** Support threads can still execute there.

Keep the same worker count, receiver, admission window, decoding kernels and output ordering.

### What can be proved about this policy

Suppose distinct workers \(W_i\) and \(W_j\) have distinct singleton masks.

If \(W_i\) wakes \(W_j\), the waking worker runs on \(c_i\), while the awakened worker is only eligible for \(c_j\). Therefore:

\[
i\ne j
\quad\Longrightarrow\quad
\text{worker-waker CPU}\ne\text{worker-wakee target CPU}.
\]

**The observed worker-on-worker co-location pattern is excluded by construction**, provided the assignments are successfully applied and remain in force. Linux exposes CPU affinity as a per-thread eligibility constraint. [man7.org](https://man7.org/linux/man-pages/man2/sched_setaffinity.2.html)

That gives the proposal unusually clear diagnostic value. It changes a specific scheduling possibility without changing the queue implementation.

It also need not introduce per-row work or per-block affinity changes. The assignment can happen once when each owned worker starts.

### Why jobs should remain dynamically assigned

Pinning a **worker thread** does not require statically partitioning the input blocks.

The existing shared receiver can continue handing the next job to whichever worker acquires it. That retains dynamic distribution of blocks, whose decode durations need not be equal. Each worker still owns its existing scratch buffers. This follows directly from keeping the current worker loop unchanged after startup. 

### Why support threads should remain movable

The scanner, reader and feeder still need CPU time. Assigning them a fifth core would change the resource budget. Reserving one of the four cores exclusively for support work would make a different trade-off involving fewer decoding cores.

Leaving support threads eligible across \(C\) is the smaller intervention. It preserves their ability to use capacity released by waiting workers.

The implementation would also need to respect thread-affinity inheritance: a newly created thread inherits its creator’s mask. Narrowing each worker’s mask **inside that worker**, rather than temporarily narrowing the creator’s mask, avoids accidentally restricting the scanner or later-created threads. [man7.org](https://man7.org/linux/man-pages/man3/pthread_setaffinity_np.3.html)

### What cannot be proved

This policy does **not** guarantee that every idle CPU is used.

For example, worker 2 may be ready but its assigned CPU may be occupied by the scanner, while worker 1 is asleep and CPU 1 is idle. Worker 2 cannot migrate there under this policy.

Other risks include support-thread delays, changes in cache locality, and greater overlap between memory-intensive decodes. The kernel’s own workqueue documentation describes a real trade-off between locality and maintaining utilization; that is useful context, not evidence that this particular decoder will benefit. [Kernel.org](https://www.kernel.org/doc/html/v6.8/core-api/workqueue.html)

So the theoretical conclusion is deliberately narrow:

> **Fixed worker placement eliminates one observed class of worker-worker placement, but can replace it with different scheduling delays. Its net value must be measured.**

That is sufficient to justify considering one bounded placement-only comparison. It is not sufficient to ship it.

## 6. A second source-level issue can also be resolved theoretically

The scanner and reader currently share one condition variable, despite waiting for different predicates.

The scanner waits for:

\[
\texttt{issued}-\texttt{taken}<\texttt{slots}
\]

or cancellation.

The reader waits for the next ordered result, end-of-stream or cancellation. Worker publication calls `notify_all()`, and taking a result also calls that same condition variable.  

**Publishing a decoded block does not, by itself, free an admission slot.** The slot count changes when the reader increments `taken`.

Thus an ordinary publication can wake the scanner even though the scanner’s slot predicate remains false.

This can be established from the state transitions without another experiment.

### A correct separation is possible

A two-condition-variable design, sharing the existing state mutex, could route notifications by purpose:

| State transition | Relevant notification |
|---|---|
| The next ordered result becomes ready | Reader |
| Reader takes a block and opens a previously full window | Scanner |
| Scanner finishes or publishes its terminal error | Reader |
| Cancellation | Both |

The reader can avoid waking for out-of-order results: once it consumes an earlier result, it checks for the next one under the mutex before sleeping again.

The usual correctness requirement remains: examine and change predicates under the shared mutex, and wait in a loop. Rust’s condition-variable API atomically releases the mutex when waiting and reacquires it before returning; spurious wakes must still be tolerated. [doc.rust-lang.org](https://doc.rust-lang.org/std/sync/struct.Condvar.html)

### A tempting one-line change would be unsafe

**Do not simply replace the existing shared `notify_all()` with `notify_one()`.**

A counterexample follows from the current predicates:

1. The window is full; the scanner waits for a slot.
2. The reader waits for its next block.
3. The final outstanding worker publishes that block.
4. `notify_one()` wakes the scanner rather than the reader.
5. The scanner finds the window still full and sleeps again.
6. No further publication occurs to wake the reader.

That modified protocol can strand both threads. This is a counterexample to the proposed one-line shortcut, **not a defect in the current `notify_all()` implementation**.

### Why I would not combine this with the first intervention

Separating notifications does not directly alter the worker-to-worker receiver wake chain. Its possible performance benefit is through less unnecessary support-thread activity and different scheduling interactions.

It is a well-defined secondary design within candidate 2, but **I would not bundle it into the placement-only candidate**. Otherwise, a result would be harder to interpret, and a modest investigation would become a multi-change pipeline rewrite.

## 7. What the eventual evaluation must preserve

For the fixed-worker-placement candidate, the narrow comparison is:

> **Same decoder, same four-core allocation, same four workers and receiver; only worker affinity differs.**

The preparation should establish the actual allowed CPU set, choose distinct physical cores without using their SMT siblings, apply the assignments before the worker loop, and verify the resulting masks. Affinity can be constrained further by cpusets; a failed or altered assignment must not silently pass as the intended experiment. [man7.org](https://man7.org/linux/man-pages/man2/sched_setaffinity.2.html)

The existing decoder invariants remain the reference: bounded admission, ordered results and errors, cancellation, channel shutdown and scratch reuse. Placement does not change their state transitions, although an implementation still needs to verify its startup and failure paths.  

A convincing outcome would require two separate answers:

**Did the intended scheduling pattern change?**  
The worker-waker same-CPU pattern should disappear under verified singleton assignments. The analysis should then check whether delays moved elsewhere, especially to scanner or reader execution.

**Did the decoder improve?**  
Whole-decoder throughput and aggregate CPU cost must meet their predeclared conditions. Reduced wake latency alone is insufficient.

A throughput improvement would support the **placement policy’s engineering value**. It would not, by itself, prove that wake affinity was the precise cause: persistent worker placement also changes migration and locality.

Conversely, if the targeted delays fall but throughput or CPU efficiency does not improve, that is a useful stopping result for this policy. It would mean we removed a visible symptom without recovering worthwhile overall performance.

I would not add `yield_now()`, spinning, priority changes or sleeps to this candidate. In particular, Linux does not provide a reliable “run the worker I just woke” guarantee through `sched_yield()` under ordinary scheduling. [man7.org](https://man7.org/linux/man-pages/man2/sched_yield.2.html)

## 8. What is now resolved, and what still needs evidence

| Question | Conclusion |
|---|---|
| Can the receiver’s wake chain matter even when direct lock obstruction is small? | **Yes.** Unlocking can affect another worker’s placement before the unlocking worker starts decoding. |
| Does the evidence identify a specific Linux scheduler heuristic as the cause? | **No.** It identifies a placement/delay pattern. |
| Does 5.65% mean that much CPU capacity is recoverable? | **No.** It is aggregate waiting-worker time and needs capacity- and job-aware accounting. |
| Can one policy exclude worker-waker co-location? | **Yes.** Distinct fixed worker masks do so. |
| Does that policy guarantee higher throughput or lower CPU cost? | **No.** Other delays and contention can offset the gain. |
| Can scanner/reader notifications be separated correctly? | **Yes.** Their predicates and necessary notifications can be specified from the source. |
| Should notification separation be combined with the first placement comparison? | **No.** Keep the first intervention interpretable. |

**My research judgment is that candidate 2 remains credible, but it is now a specific placement hypothesis rather than a broad “pipeline cleanup” opportunity.**

The most useful remaining analysis before implementation is a capacity-aware pass over the existing scheduling records: count runnable workers against simultaneously idle eligible CPUs and genuinely queued jobs, preserving the existing clock-uncertainty guards. I inspected the analysis code but did not independently replay the compressed scheduler traces here, so that refined opportunity has not been calculated in this review.

After that, the first concrete candidate should be **fixed decoding-worker placement with movable support threads, keeping the receiver and admission policy unchanged**. No new workstation command or run is called for by this review.
`````
