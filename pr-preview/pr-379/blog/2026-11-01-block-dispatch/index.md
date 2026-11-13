[mibz2 · Research blog](<https://lmmx.github.io/mibz2/pr-preview/pr-379/blog/index.md>)

# Block dispatch in mibz2: mutexes, futexes and the scheduler

How mibz2's parallel decoder hands blocks to its workers, why some take up to a millisecond to be picked up, and how to tell lock contention from scheduling delay.

1 November 2026 · 12 min read · [Investigation in progress](<#8-what-happens-next>)

mibz2 has two very different kinds of parallelism:

1. **Across blocks:** different CPU cores decompress different bzip2 blocks at the same time.
2. **Within a block:** each worker follows 32 independent chains through the inverse BWT's table, to hide memory latency. The [previous post](<https://lmmx.github.io/mibz2/pr-preview/pr-379/blog/2026-10-31-room-walk-overlap/index.md>) is about that one.

This post is about the first kind: how work is handed to the workers, and an apparent delay in that handoff that may or may not be a real bottleneck. Finding out which takes a short tour of mutexes, futexes and the Linux scheduler.

## 1\. The journey of a bzip2 block

The journey of a bzip2 block through mibz2

Seven threads in all at four workers: the feeder, the scanner, four workers and the caller's thread as the reader. At most a fixed number of blocks (the window, `slots`) is between the scanner and the reader at any time.

A bzip2 block goes through several stages. In mibz2 they are split between two kinds of thread:

| Stage | Thread | Why there |
| --- | --- | --- |
| Find the block, read its header, Huffman-decode its symbols | the scanner | A block's end is only known once its symbols are read, so this part is sequential |
| Move-to-front and its run expansion | a worker | Independent per block |
| Inverse BWT: build the table, walk it | a worker | The expensive part, and independent per block |
| RLE1 and the block's CRC | a worker | Independent per block |

This split matters for everything below: a faster inverse BWT can't help if the scanner, or the handoff of jobs to workers, becomes the limit.

All seven threads (the feeder, the scanner, four workers and the reader) share the CPUs the operating system gives them. In the measurements below that is four physical cores, so there are moments when more threads could run than there are cores to run them.

## 2\. Two waits in three lines of Rust

Here is how a worker gets its next job, from [`src/parallel.rs`](<https://github.com/lmmx/mibz2/blob/master/src/parallel.rs>):

```rust
let job = {
    let rx = jobs.lock().unwrap_or_else(|e| e.into_inner());
    rx.recv()
};
```

`jobs` is a `Mutex<Receiver<Job>>`: one standard-library channel receiver, shared by all the workers behind a mutex. Two different waits can happen here:

1. **Waiting for the mutex.** Another worker holds it, so this one can't reach `recv()`.
2. **Waiting for a job.** This worker holds the mutex, but the channel is empty, so `recv()` parks the thread until the scanner sends something.

The second wait happens *while the mutex is held*: the guard `rx` lives until the end of the block. That makes possible a situation that is easy to miss in the code:

Four workers, one shared receiver

The possible *receiver convoy*. A true multi-consumer queue would let A and B wait for jobs independently. Whether this sequence happens often enough to matter is what the diagnostic measures.

None of this is incorrect. Every job is still taken exactly once, in order. The question is only whether it is slow.

### What a futex is

When worker B "waits for the mutex", what actually happens? On Linux, Rust's standard `Mutex` is a single atomic word, with a **futex** ("fast userspace mutex") behind it.

How a futex-backed mutex waits

Only the contended path reaches the kernel, which is why kernel traces see futex calls for contention and never for an uncontended lock. The trace marks in mibz2's own code cover both.

A futex is the kernel's way of letting a thread sleep on an address until another thread says "wake up". Most lock operations never need it. The channel's `recv()` sleeps the same way: it parks the thread on a futex of its own.

## 3\. Woken is not running

This is the distinction the whole investigation turns on. A thread can be in three states:

| State | Using a CPU? | What it means |
| --- | --- | --- |
| Running | Yes | Executing instructions, possibly stalled on memory |
| Runnable | No, not yet | Ready to execute; waiting for the scheduler to give it a CPU |
| Asleep | No | Waiting for an event: a futex wake, a job, a lock |

A worker's wake-up, in three states

Conceptual timing, not measured values. A futex moves a thread from asleep to runnable; only the scheduler moves it from runnable to running.

So when a job takes a long time to be claimed, there are at least three different explanations, each calling for a different response:

| Explanation | What would help |
| --- | --- |
| **Mutex ownership:** the worker is asleep because another worker owns the receiver | A true multi-consumer queue, so idle workers wait independently |
| **Scheduling:** the worker is runnable, but every CPU is busy with another thread | Nothing in the queue, and perhaps nothing at all if those threads are doing useful work |
| **Backlog:** every worker is busy decoding, so the job simply waits its turn | Nothing in the queue: decoding faster |

A simple application-level trace can't tell these apart. All three show up as "the job was enqueued at t₀ and claimed at t₁".

## 4\. What the traces have shown so far

The decoder can record its own timeline: a mark at each step of each thread, with a nanosecond timestamp. On an i9-10900K, decoding a 207 MB slice of Wikidata at four workers, some jobs took nearly a millisecond to be claimed. That looked suspicious, until the delays were sorted by how busy the workers were when the job arrived:

Most slow claims are queueing

The i9-10900K's ten four-worker traces of the Wikidata corpus. With three or four workers busy, the next job usually waits behind decoding already under way: ordinary queueing, not evidence of lock contention by itself.

Most of the slow claims are backlog. But a smaller set is more interesting: jobs that arrived with **three workers decoding, no older job waiting, and one worker free**. For those, the free worker's last recorded step tells us where it was:

| The free worker's last mark when the job arrived | Cases | Median time to claim |
| --- | ---: | ---: |
| `JobRecv`: holding the mutex, inside `recv()` | 198 | 42 µs |
| `JobLock`: about to take the mutex | 149 | 454 µs |

When the free worker is already inside `recv()`, it claims the job in a median of 42 µs. When it is still trying to take the mutex, it takes ten times as long, and another worker claimed the job first in 89 of the 149 cases.

That gap could be mutex contention. But the kernel's own accounting of each thread's run-queue wait, read between decodes, points elsewhere: in the 120 gaps between decodes that contain such a `JobLock` case, **78% of the elapsed time was runnable but not running.** The workers were awake and eligible to run, but weren't on a CPU. Across all the gaps between decodes, the figure is 34%.

That still doesn't make the delay waste. With seven threads on four cores, a runnable worker may simply be waiting while the scanner or another worker does useful work. The question is what the other CPUs were doing at that moment.

## 5\. The reader has its own wait

When a worker finishes a block it doesn't write to the output. It publishes the block into a shared `ready` map keyed by its position, and the reader takes blocks strictly in order.

Blocks finish out of order

Finished blocks wait in `ready` for their turn. The bounded window keeps that wait from turning into unbounded memory.

The scanner is held back by a bounded admission window: at most `slots` blocks may be between the scanner and the reader (eight at four workers). Before reading the next block, the scanner waits:

```rust
while !st.cancelled && st.issued - st.taken >= shared.slots {
    st = shared.cv.wait(st).unwrap_or_else(|e| e.into_inner());
}
```

That keeps memory bounded when one slow block holds up the reader.

The scanner and the reader share one condition variable, `shared.cv`, though they wait for different things: the scanner for a free slot, the reader for the next block in order. Workers call `notify_all()` when they publish a block, and the reader calls it when it takes one. That is correct, but every notification wakes both kinds of waiter, whether or not its own condition changed. Earlier measurements suggested the reader is sometimes late to take a block that is already ready. Whether notification, the lock or scheduling causes that is still open.

## 6\. Measuring it without fooling ourselves

To separate the three explanations, the next experiment joins two views of the same run:

- **The decoder's own marks,** now at every boundary that matters: the attempt to take the receiver mutex, holding it, `recv()` returning, the guard's release, the decode, the publication, and the reader's waits and wake-ups. Each thread records into a fixed, preallocated buffer, with its operating-system thread id.
- **The kernel's scheduler events,** recorded with `perf` on the allocated CPUs: every wake-up, every switch of a CPU from one thread to another, and every futex wait with its address.

With both, every moment a worker spends not decoding while a job is available can be classified. Was it asleep while another worker owned the receiver? Asleep in the handoff, with the mutex free? Runnable but not running, and if so, what was occupying each CPU? Or publishing, or waiting on the shared lock?

Joining them needs care:

- **Two clocks.** The marks use Rust's `Instant`, and the kernel stamps its events with its own clock. The decoder reads `CLOCK_MONOTONIC` and immediately makes a recognisable system call, so the gap between the two clocks can be measured from the trace itself. On the preparation machine it is known to within 0.6 µs.
- **The instrument changes what it measures.** Each instrumented run is paired with an uninstrumented one. If tracing moves throughput or CPU time by more than 3%, the result is reported as provisional.
- **The decision rules were written down before any data.** An isolated multi-consumer queue experiment is justified only if receiver-specific obstruction is at least 5% of the workers' total time, in at least 80% of runs. Runnable time near the mutex is reported beside that, but never counted in it, because it may be ordinary competition for the cores.

### When the instrument lies

The preparation runs turned up a lesson worth sharing. A review of the analysis asked a pointed question: could lost kernel events pass unnoticed? Checking it showed they could. `perf script` prints nothing about events it dropped unless asked with `--show-lost-events`. Events whose thread perf couldn't resolve were printed in a form the parser skipped.

The surprise came from the kernel itself. On the virtual machine used for preparation, the kernel emitted every switch *into* idle on three of its four CPUs, but never a switch *out* of idle. perf's raw dump and ftrace showed the same, so it isn't perf or the parser.

When the instrument lies

On the preparation VM, the kernel recorded switches into idle on three of its four CPUs but never the switches out. Running time read as waiting. The analysis now marks such intervals unknown, and refuses a run whose idle switches don't pair.

The effect was subtle and pointed in a dangerous direction. A worker switched in from idle appeared to be woken but never running, so its running time was classified as waiting for a CPU, which is exactly the "scheduling" explanation. In one run, 19 ms of the 77 ms counted as runnable was this artifact.

Now, the analysis:

- treats any interval the scheduler stream contradicts as unknown;
- requires every CPU's switches into and out of idle to pair up;
- runs a short probe before any capture, and refuses to start on a kernel that fails it.

A wrong answer delivered confidently would have been worse than no answer.

## 7\. Two problems, one CPU

Even with perfect dispatch, four workers each decode less efficiently than one. On the i9, four workers spend about **30% more cycles per decoded byte** than one worker, while executing almost the same instructions (1.3% more). The extra cycles go to stalls on memory: each worker's inverse-BWT table competes for the same 20 MB last-level cache, alongside the input, output and intermediate buffers.

Two problems, one CPU

Better dispatch uses the cores more fully; better decoding makes each core's work cheaper.

These interact. Better dispatch keeps more inverse-BWT walks running at once, which raises the competition for cache and claws back part of its own gain. That is why every experiment is judged on the whole decoder's throughput and CPU-seconds per GB, never on one stage in isolation.

## 8\. What happens next

The measurement comes next. It ends in one of two places:

- **If receiver ownership really does keep free workers from available jobs,** at least 5% of their time in at least 80% of runs, a multi-consumer queue becomes a justified, isolated experiment.
- **If the receiver is usually free and the delay is scheduling or backlog,** that hypothesis is retired, and the effort goes to the larger problem: the inverse BWT's memory behaviour.

This post is a snapshot of an investigation in progress, and its figures come from the traces available so far. A few lines of synchronisation code can decide how well a 32-lane, latency-hiding decoder uses a whole multicore CPU. Whether they do here is a question for the evidence.

---

**Sources:**

- the worker, scanner and reader loops: [`src/parallel.rs`](<https://github.com/lmmx/mibz2/blob/master/src/parallel.rs>);
- the i9 traces and the cycle accounting: [the resource attribution's analysis](<https://github.com/lmmx/mibz2/blob/docs/decoder-attribution-records/docs/journal/2026-11-01-decoder-attribution-i9-10900K/analysis.md>);
- the diagnostic's registered protocol: [`2026-11-01-dispatch-causality-protocol.md`](<https://github.com/lmmx/mibz2/blob/research/dispatch-causality/docs/journal/2026-11-01-dispatch-causality-protocol.md>);
- the trace-completeness checks: [amendment 1](<https://github.com/lmmx/mibz2/blob/research/dispatch-causality/docs/journal/2026-11-01-dispatch-causality/amendment-1.md>).
