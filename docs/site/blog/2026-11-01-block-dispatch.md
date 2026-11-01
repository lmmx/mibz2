---
title: "Block dispatch in mibz2: mutexes, futexes and the scheduler"
image: assets/social/blog-2026-11-01-block-dispatch.png
description: "How mibz2's parallel decoder hands blocks to its workers, why some take up to a millisecond to be picked up, and how to tell lock contention from scheduling delay."
---

<div class="post-hero" markdown>

<div class="post-hero-text" markdown>

[mibz2 · Research blog](index.md)
{: .post-kicker }

# Block dispatch in mibz2: mutexes, futexes and the scheduler

How mibz2's parallel decoder hands blocks to its workers, why some take up to a millisecond
to be picked up, and how to tell lock contention from scheduling delay.
{: .post-dek }

1 November 2026 · 12 min read · [Investigation in progress](#8-what-happens-next)
{: .post-meta }

</div>

<div class="post-hero-art"><img src="../assets/images/hero.svg" alt=""></div>

</div>

mibz2 has two very different kinds of parallelism:

1. **Across blocks:** different CPU cores decompress different bzip2 blocks at the same
   time.
2. **Within a block:** each worker follows 32 independent chains through the inverse BWT's
   table, to hide memory latency. The [previous post](2026-10-31-room-walk-overlap.md) is
   about that one.

This post is about the first kind: how work is handed to the workers, and an apparent delay
in that handoff that may or may not be a real bottleneck. Finding out which takes a short
tour of mutexes, futexes and the Linux scheduler.

## 1. The journey of a bzip2 block

<figure class="bf-fig">
<p class="bf-title">The journey of a bzip2 block through mibz2</p>
<svg viewBox="0 0 720 488" role="img" aria-label="A compressed stream feeds the scanner, which sends jobs through one shared channel to four workers; they publish finished blocks to a ready map, from which the reader takes them in order."><rect x="110" y="0" width="500" height="58" rx="8" class="bf-card"/><text x="126" y="26" class="bf-ts">Compressed stream</text><text x="126" y="47" class="bf-tm">A long sequence of bzip2 blocks, read in chunks by the feeder thread (mibz2-read)</text><path d="M360 62V84m-5-5 5 5 5-5" class="bf-arrow"/><rect x="110" y="88" width="500" height="58" rx="8" class="bf-card"/><text x="126" y="114" class="bf-ts">Scanner · mibz2-scan</text><text x="126" y="135" class="bf-tm">Finds each block in order, reads its header and Huffman-decodes its symbols</text><path d="M360 150V172m-5-5 5 5 5-5" class="bf-arrow"/><rect x="40" y="176" width="640" height="132" rx="12" class="bf-panel"/><text x="360" y="202" class="bf-ts" text-anchor="middle">Shared job channel · Mutex&lt;Receiver&lt;Job&gt;&gt;</text><rect x="58" y="218" width="142" height="54" rx="8" class="bf-cell-hi"/><text x="129" y="241" class="bf-t-strong" text-anchor="middle">Worker 0 · mibz2-0</text><text x="129" y="260" class="bf-tm" text-anchor="middle">one block at a time</text><rect x="212" y="218" width="142" height="54" rx="8" class="bf-cell-hi"/><text x="283" y="241" class="bf-t-strong" text-anchor="middle">Worker 1 · mibz2-1</text><text x="283" y="260" class="bf-tm" text-anchor="middle">one block at a time</text><rect x="366" y="218" width="142" height="54" rx="8" class="bf-cell-hi"/><text x="437" y="241" class="bf-t-strong" text-anchor="middle">Worker 2 · mibz2-2</text><text x="437" y="260" class="bf-tm" text-anchor="middle">one block at a time</text><rect x="520" y="218" width="142" height="54" rx="8" class="bf-cell-hi"/><text x="591" y="241" class="bf-t-strong" text-anchor="middle">Worker 3 · mibz2-3</text><text x="591" y="260" class="bf-tm" text-anchor="middle">one block at a time</text><text x="360" y="294" class="bf-tm" text-anchor="middle">Move-to-front, the inverse BWT, RLE1 and the block's CRC</text><path d="M360 312V334m-5-5 5 5 5-5" class="bf-arrow"/><rect x="110" y="338" width="500" height="58" rx="8" class="bf-card"/><text x="126" y="364" class="bf-ts">ready · a map keyed by block position</text><text x="126" y="385" class="bf-tm">Each worker publishes its finished block here, in whatever order it finishes</text><path d="M360 400V422m-5-5 5 5 5-5" class="bf-arrow"/><rect x="110" y="426" width="500" height="58" rx="8" class="bf-card-hi"/><text x="126" y="452" class="bf-ts">Reader · the caller's own thread</text><text x="126" y="473" class="bf-tm">Takes the blocks in stream order and hands back their bytes</text></svg>
<figcaption>Seven threads in all at four workers: the feeder, the scanner, four workers and the caller's thread as the reader. At most a fixed number of blocks (the window, <code>slots</code>) is between the scanner and the reader at any time.</figcaption>
</figure>

A bzip2 block goes through several stages. In mibz2 they are split between two kinds of
thread:

| Stage | Thread | Why there |
| --- | --- | --- |
| Find the block, read its header, Huffman-decode its symbols | the scanner | A block's end is only known once its symbols are read, so this part is sequential |
| Move-to-front and its run expansion | a worker | Independent per block |
| Inverse BWT: build the table, walk it | a worker | The expensive part, and independent per block |
| RLE1 and the block's CRC | a worker | Independent per block |

This split matters for everything below: a faster inverse BWT can't help if the scanner, or
the handoff of jobs to workers, becomes the limit.

All seven threads (the feeder, the scanner, four workers and the reader) share the CPUs the
operating system gives them. In the measurements below that is four physical cores, so
there are moments when more threads could run than there are cores to run them.

## 2. Two waits in three lines of Rust

Here is how a worker gets its next job, from
[`src/parallel.rs`](https://github.com/lmmx/mibz2/blob/master/src/parallel.rs):

```rust
let job = {
    let rx = jobs.lock().unwrap_or_else(|e| e.into_inner());
    rx.recv()
};
```

`jobs` is a `Mutex<Receiver<Job>>`: one standard-library channel receiver, shared by all
the workers behind a mutex. Two different waits can happen here:

1. **Waiting for the mutex.** Another worker holds it, so this one can't reach `recv()`.
2. **Waiting for a job.** This worker holds the mutex, but the channel is empty, so `recv()`
   parks the thread until the scanner sends something.

The second wait happens *while the mutex is held*: the guard `rx` lives until the end of the
block. That makes possible a situation that is easy to miss in the code:

<figure class="bf-fig">
<p class="bf-title">Four workers, one shared receiver</p>
<svg viewBox="0 0 720 280" role="img" aria-label="Worker A sleeps inside recv while holding the receiver mutex; worker B waits for the mutex; workers C and D are decoding. When two jobs arrive, B cannot take the second until A has woken, taken the first and released the lock."><rect x="0" y="0" width="352" height="92" rx="10" class="bf-cell-hi"/><text x="18" y="28" class="bf-ts">Worker A</text><text x="18" y="52" class="bf-tm">Asleep inside recv(): no job yet</text><text x="18" y="72" class="bf-pill-pass-t">Holds the receiver mutex</text><rect x="368" y="0" width="352" height="92" rx="10" class="bf-pill-ready"/><text x="386" y="28" class="bf-ts">Worker B</text><text x="386" y="52" class="bf-tm">Waiting for the receiver mutex</text><text x="386" y="72" class="bf-pill-ready-t">Cannot reach recv() at all</text><rect x="0" y="104" width="352" height="70" rx="10" class="bf-card"/><text x="18" y="132" class="bf-ts">Worker C</text><text x="18" y="154" class="bf-tm">Decoding a block</text><rect x="368" y="104" width="352" height="70" rx="10" class="bf-card"/><text x="386" y="132" class="bf-ts">Worker D</text><text x="386" y="154" class="bf-tm">Decoding another block</text><path d="M360 182V206m-5-5 5 5 5-5" class="bf-arrow"/><rect x="140" y="210" width="440" height="66" rx="10" class="bf-card"/><text x="360" y="236" class="bf-ts" text-anchor="middle">The scanner enqueues two jobs</text><text x="360" y="258" class="bf-tm" text-anchor="middle">A must wake, return from recv() and drop the lock before B can even start waiting for one</text></svg>
<figcaption>The possible <em>receiver convoy</em>. A true multi-consumer queue would let A and B wait for jobs independently. Whether this sequence happens often enough to matter is what the diagnostic measures.</figcaption>
</figure>

None of this is incorrect. Every job is still taken exactly once, in order. The question is
only whether it is slow.

### What a futex is

When worker B "waits for the mutex", what actually happens? On Linux, Rust's standard
`Mutex` is a single atomic word, with a **futex** ("fast userspace mutex") behind it.

<figure class="bf-fig">
<p class="bf-title">How a futex-backed mutex waits</p>
<svg viewBox="0 0 720 298" role="img" aria-label="If the lock is free, one atomic compare-and-swap takes it in userspace. If it is held, the thread spins briefly and then sleeps in the kernel through a futex; the holder's unlock wakes one waiter, which becomes runnable but not yet running."><rect x="160" y="0" width="400" height="58" rx="8" class="bf-card"/><text x="176" y="26" class="bf-ts">Worker B wants the receiver mutex</text><text x="176" y="47" class="bf-tm">Rust's std Mutex on Linux: one atomic word, and a futex behind it</text><path d="M175 62V92m-5-5 5 5 5-5" class="bf-arrow"/><path d="M545 62V92m-5-5 5 5 5-5" class="bf-arrow"/><rect x="0" y="96" width="350" height="92" rx="10" class="bf-cell-hi"/><text x="16" y="122" class="bf-ts">The lock is free</text><text x="16" y="144" class="bf-tm">One atomic compare-and-swap, entirely in userspace.</text><text x="16" y="162" class="bf-tm">No system call, no kernel involvement.</text><text x="16" y="180" class="bf-pill-pass-t">Fast path</text><rect x="370" y="96" width="350" height="92" rx="10" class="bf-pill-ready"/><text x="386" y="122" class="bf-ts">The lock is held</text><text x="386" y="144" class="bf-tm">Spin briefly, then futex(FUTEX_WAIT): the kernel</text><text x="386" y="162" class="bf-tm">takes the thread off the CPU until it is woken.</text><text x="386" y="180" class="bf-pill-ready-t">Contended path</text><path d="M545 192V214m-5-5 5 5 5-5" class="bf-arrow"/><rect x="370" y="218" width="350" height="76" rx="8" class="bf-card"/><text x="386" y="244" class="bf-ts">Unlock: futex(FUTEX_WAKE)</text><text x="386" y="265" class="bf-tm">The kernel makes one waiter runnable.</text><text x="386" y="283" class="bf-tm">Runnable is not running: it still needs a CPU.</text></svg>
<figcaption>Only the contended path reaches the kernel, which is why kernel traces see futex calls for contention and never for an uncontended lock. The trace marks in mibz2's own code cover both.</figcaption>
</figure>

A futex is the kernel's way of letting a thread sleep on an address until another thread
says "wake up". Most lock operations never need it. The channel's `recv()` sleeps the same
way: it parks the thread on a futex of its own.

## 3. Woken is not running

This is the distinction the whole investigation turns on. A thread can be in three states:

| State | Using a CPU? | What it means |
| --- | --- | --- |
| Running | Yes | Executing instructions, possibly stalled on memory |
| Runnable | No, not yet | Ready to execute; waiting for the scheduler to give it a CPU |
| Asleep | No | Waiting for an event: a futex wake, a job, a lock |

<figure class="bf-fig">
<p class="bf-title">A worker's wake-up, in three states</p>
<svg viewBox="0 0 720 166" role="img" aria-label="A timeline from a job's arrival: the worker is asleep until woken, then runnable but not running until the scheduler gives it a CPU, then running until it claims the job."><path d="M40 70H680" class="bf-rule"/><rect x="250" y="96" width="270" height="14" rx="7" class="bf-pill-ready"/><text x="385" y="134" class="bf-ts" text-anchor="middle">Runnable, not running</text><text x="385" y="154" class="bf-tm" text-anchor="middle">Ready to execute; waiting for the scheduler to give it a CPU</text><rect x="40" y="96" width="206" height="14" rx="7" class="bf-pill-done"/><text x="143" y="134" class="bf-ts" text-anchor="middle">Asleep</text><text x="143" y="154" class="bf-tm" text-anchor="middle">Parked in recv() or on the mutex</text><rect x="524" y="96" width="156" height="14" rx="7" class="bf-cell-hi"/><text x="602" y="134" class="bf-ts" text-anchor="middle">Running</text><text x="602" y="154" class="bf-tm" text-anchor="middle">Takes the lock, receives</text><path d="M40 52V88" class="bf-axis"/><circle cx="40" cy="70" r="6" class="bf-dot-hi"/><text x="40" y="36" class="bf-ts" text-anchor="start">Job arrives</text><path d="M250 52V88" class="bf-axis"/><circle cx="250" cy="70" r="6" class="bf-dot-hi"/><text x="250" y="36" class="bf-ts" text-anchor="middle">Worker woken</text><path d="M520 52V88" class="bf-axis"/><circle cx="520" cy="70" r="6" class="bf-dot-hi"/><text x="520" y="36" class="bf-ts" text-anchor="middle">Worker running</text><path d="M680 52V88" class="bf-axis"/><circle cx="680" cy="70" r="6" class="bf-dot-hi"/><text x="680" y="36" class="bf-ts" text-anchor="end">Job claimed</text></svg>
<figcaption>Conceptual timing, not measured values. A futex moves a thread from asleep to runnable; only the scheduler moves it from runnable to running.</figcaption>
</figure>

So when a job takes a long time to be claimed, there are at least three different
explanations, each calling for a different response:

| Explanation | What would help |
| --- | --- |
| **Mutex ownership:** the worker is asleep because another worker owns the receiver | A true multi-consumer queue, so idle workers wait independently |
| **Scheduling:** the worker is runnable, but every CPU is busy with another thread | Nothing in the queue, and perhaps nothing at all if those threads are doing useful work |
| **Backlog:** every worker is busy decoding, so the job simply waits its turn | Nothing in the queue: decoding faster |

A simple application-level trace can't tell these apart. All three show up as "the job was
enqueued at t₀ and claimed at t₁".

## 4. What the traces have shown so far

The decoder can record its own timeline: a mark at each step of each thread, with a
nanosecond timestamp. On an i9-10900K, decoding a 207 MB slice of Wikidata at four workers,
some jobs took nearly a millisecond to be claimed. That looked suspicious, until the delays
were sorted by how busy the workers were when the job arrived:

<figure class="bf-fig">
<p class="bf-title">Most slow claims are queueing</p>
<svg viewBox="0 0 720 228" role="img" aria-label="Median claim delay by how many workers were decoding when the job was enqueued: about 50 microseconds with 0, 1 or 2 decoding, 977 with 3, and 1,221 with all 4."><text x="0" y="14" class="bf-tm">Workers decoding when the job arrived</text><text x="210" y="14" class="bf-tm">Median time from enqueue to claim</text><text x="0" y="49" class="bf-ts">0 of 4</text><text x="64" y="49" class="bf-tm">9% of jobs</text><rect x="210" y="34" width="430" height="22" rx="4" class="bf-track"/><rect x="210" y="34" width="14.2" height="22" rx="4" class="bf-bar"/><text x="232.19" y="50" class="bf-t-strong">49.5 µs</text><text x="0" y="89" class="bf-ts">1 of 4</text><text x="64" y="89" class="bf-tm">17% of jobs</text><rect x="210" y="74" width="430" height="22" rx="4" class="bf-track"/><rect x="210" y="74" width="13.9" height="22" rx="4" class="bf-bar"/><text x="231.87466666666666" y="90" class="bf-t-strong">48.4 µs</text><text x="0" y="129" class="bf-ts">2 of 4</text><text x="64" y="129" class="bf-tm">30% of jobs</text><rect x="210" y="114" width="430" height="22" rx="4" class="bf-track"/><rect x="210" y="114" width="17.7" height="22" rx="4" class="bf-bar"/><text x="235.65866666666668" y="130" class="bf-t-strong">61.6 µs</text><text x="0" y="169" class="bf-ts">3 of 4</text><text x="64" y="169" class="bf-tm">41% of jobs</text><rect x="210" y="154" width="430" height="22" rx="4" class="bf-track"/><rect x="210" y="154" width="280.2" height="22" rx="4" class="bf-steel"/><text x="498.15933333333334" y="170" class="bf-t-strong">977 µs</text><text x="0" y="209" class="bf-ts">4 of 4</text><text x="64" y="209" class="bf-tm">3% of jobs</text><rect x="210" y="194" width="430" height="22" rx="4" class="bf-track"/><rect x="210" y="194" width="350.0" height="22" rx="4" class="bf-steel"/><text x="567.9626666666667" y="210" class="bf-t-strong">1,221 µs</text></svg>
<figcaption>The i9-10900K's ten four-worker traces of the Wikidata corpus. With three or four workers busy, the next job usually waits behind decoding already under way: ordinary queueing, not evidence of lock contention by itself.</figcaption>
</figure>

Most of the slow claims are backlog. But a smaller set is more interesting: jobs that
arrived with **three workers decoding, no older job waiting, and one worker free**. For those, the free worker's
last recorded step tells us where it was:

| The free worker's last mark when the job arrived | Cases | Median time to claim |
| --- | ---: | ---: |
| `JobRecv`: holding the mutex, inside `recv()` | 198 | 42 µs |
| `JobLock`: about to take the mutex | 149 | 454 µs |

When the free worker is already inside `recv()`, it claims the job in a median of 42 µs.
When it is still trying to take the mutex, it takes ten times as long, and another worker
claimed the job first in 89 of the 149 cases.

That gap could be mutex contention. But the kernel's own accounting of each thread's
run-queue wait, read between decodes, points elsewhere: in the 120 gaps between decodes that
contain such a `JobLock` case, **78% of the elapsed time was runnable but not running.** The
workers were awake and eligible to run, but weren't on a CPU. Across all the gaps between
decodes, the figure is 34%.

That still doesn't make the delay waste. With seven threads on four cores, a runnable worker
may simply be waiting while the scanner or another worker does useful work. The question is
what the other CPUs were doing at that moment.

## 5. The reader has its own wait

When a worker finishes a block it doesn't write to the output. It publishes the block into a
shared `ready` map keyed by its position, and the reader takes blocks strictly in order.

<figure class="bf-fig">
<p class="bf-title">Blocks finish out of order</p>
<svg viewBox="0 0 720 122" role="img" aria-label="Four block cells: block 0 still decoding, blocks 1 to 3 ready. The reader must wait for block 0."><rect x="0" y="0" width="172" height="62" rx="10" class="bf-pill-ready"/><text x="86" y="26" class="bf-ts" text-anchor="middle">#0</text><text x="86" y="47" class="bf-tm" text-anchor="middle">Still decoding</text><rect x="182" y="0" width="172" height="62" rx="10" class="bf-cell-hi"/><text x="268" y="26" class="bf-ts" text-anchor="middle">#1</text><text x="268" y="47" class="bf-tm" text-anchor="middle">Ready</text><rect x="364" y="0" width="172" height="62" rx="10" class="bf-cell-hi"/><text x="450" y="26" class="bf-ts" text-anchor="middle">#2</text><text x="450" y="47" class="bf-tm" text-anchor="middle">Ready</text><rect x="546" y="0" width="172" height="62" rx="10" class="bf-cell-hi"/><text x="632" y="26" class="bf-ts" text-anchor="middle">#3</text><text x="632" y="47" class="bf-tm" text-anchor="middle">Ready</text><text x="0" y="92" class="bf-t">The reader cannot hand back block 1 before block 0, though blocks 1–3 are finished.</text><text x="0" y="114" class="bf-tm">Four slots are held until #0 is done; with a window of eight, the scanner can admit only four more.</text></svg>
<figcaption>Finished blocks wait in <code>ready</code> for their turn. The bounded window keeps that wait from turning into unbounded memory.</figcaption>
</figure>

The scanner is held back by a bounded admission window: at most `slots` blocks may be
between the scanner and the reader (eight at four workers). Before reading the next block,
the scanner waits:

```rust
while !st.cancelled && st.issued - st.taken >= shared.slots {
    st = shared.cv.wait(st).unwrap_or_else(|e| e.into_inner());
}
```

That keeps memory bounded when one slow block holds up the reader.

The scanner and the reader share one condition variable, `shared.cv`, though they wait for
different things: the scanner for a free slot, the reader for the next block in order.
Workers call `notify_all()` when they publish a block, and the reader calls it when it takes
one. That is correct, but every notification wakes both kinds of waiter, whether or not its
own condition changed. Earlier measurements suggested the reader is sometimes late to take a
block that is already ready. Whether notification, the lock or scheduling causes that is
still open.

## 6. Measuring it without fooling ourselves

To separate the three explanations, the next experiment joins two views of the same run:

- **The decoder's own marks,** now at every boundary that matters: the attempt to take the
  receiver mutex, holding it, `recv()` returning, the guard's release, the decode, the
  publication, and the reader's waits and wake-ups. Each thread records into a fixed,
  preallocated buffer, with its operating-system thread id.
- **The kernel's scheduler events,** recorded with `perf` on the allocated CPUs: every
  wake-up, every switch of a CPU from one thread to another, and every futex wait with its
  address.

With both, every moment a worker spends not decoding while a job is available can be
classified. Was it asleep while another worker owned the receiver? Asleep in the handoff,
with the mutex free? Runnable but not running, and if so, what was occupying each CPU? Or
publishing, or waiting on the shared lock?

Joining them needs care:

- **Two clocks.** The marks use Rust's `Instant`, and the kernel stamps its events with its
  own clock. The decoder reads `CLOCK_MONOTONIC` and immediately makes a recognisable system
  call, so the gap between the two clocks can be measured from the trace itself. On the
  preparation machine it is known to within 0.6 µs.
- **The instrument changes what it measures.** Each instrumented run is paired with an
  uninstrumented one. If tracing moves throughput or CPU time by more than 3%, the result
  is reported as provisional.
- **The decision rules were written down before any data.** An isolated multi-consumer
  queue experiment is justified only if receiver-specific obstruction is at least 5% of the
  workers' total time, in at least 80% of runs. Runnable time near the mutex is reported
  beside that, but never counted in it, because it may be ordinary competition for the
  cores.

### When the instrument lies

The preparation runs turned up a lesson worth sharing. A review of the analysis asked a
pointed question: could lost kernel events pass unnoticed? Checking it showed they could.
`perf script` prints nothing about events it dropped unless asked with
`--show-lost-events`. Events whose thread perf couldn't resolve were printed in a form the
parser skipped.

The surprise came from the kernel itself. On the virtual machine used for preparation, the
kernel emitted every switch *into* idle on three of its four CPUs, but never a switch *out*
of idle. perf's raw dump and ftrace showed the same, so it isn't perf or the parser.

<figure class="bf-fig">
<p class="bf-title">When the instrument lies</p>
<svg viewBox="0 0 720 154" role="img" aria-label="Two rows. What happened: asleep, runnable, running, asleep. What the trace said: asleep, then runnable all the way to the switch-out, because the kernel never reported the switch from idle to the worker."><text x="0" y="28" class="bf-ts">What happened</text><rect x="170.0" y="10" width="131.0" height="28" rx="6" class="bf-pill-done"/><text x="237.5" y="29" class="bf-tm" text-anchor="middle">asleep</text><rect x="305.0" y="10" width="86.0" height="28" rx="6" class="bf-pill-ready"/><text x="350.0" y="29" class="bf-tm" text-anchor="middle">runnable</text><rect x="395.0" y="10" width="239.0" height="28" rx="6" class="bf-cell-hi"/><text x="516.5" y="29" class="bf-tm" text-anchor="middle">running</text><rect x="638.0" y="10" width="68.0" height="28" rx="6" class="bf-pill-done"/><text x="674.0" y="29" class="bf-tm" text-anchor="middle">asleep</text><text x="0" y="84" class="bf-ts">What the trace said</text><rect x="170.0" y="66" width="131.0" height="28" rx="6" class="bf-pill-done"/><text x="237.5" y="85" class="bf-tm" text-anchor="middle">asleep</text><rect x="305.0" y="66" width="329.0" height="28" rx="6" class="bf-pill-ready"/><text x="471.5" y="85" class="bf-tm" text-anchor="middle">runnable … still “runnable”</text><rect x="638.0" y="66" width="68.0" height="28" rx="6" class="bf-pill-done"/><text x="674.0" y="85" class="bf-tm" text-anchor="middle">asleep</text><path d="M393.0 4V102" class="bf-cut"/><text x="170" y="124" class="bf-pill-pass-t">At the green line, the switch from idle into the worker was never emitted (CPUs 1–3).</text><text x="170" y="144" class="bf-tm">The worker later switched out while “runnable”: the contradiction that gave it away.</text></svg>
<figcaption>On the preparation VM, the kernel recorded switches into idle on three of its four CPUs but never the switches out. Running time read as waiting. The analysis now marks such intervals unknown, and refuses a run whose idle switches don't pair.</figcaption>
</figure>

The effect was subtle and pointed in a dangerous direction. A worker switched in from idle
appeared to be woken but never running, so its running time was classified as waiting for a
CPU, which is exactly the "scheduling" explanation. In one run, 19 ms of the 77 ms counted as
runnable was this artifact.

Now, the analysis:

- treats any interval the scheduler stream contradicts as unknown;
- requires every CPU's switches into and out of idle to pair up;
- runs a short probe before any capture, and refuses to start on a kernel that fails it.

A wrong answer delivered confidently would have been worse than no answer.

## 7. Two problems, one CPU

Even with perfect dispatch, four workers each decode less efficiently than one. On the i9,
four workers spend about **30% more cycles per decoded byte** than one worker, while
executing almost the same instructions (1.3% more). The extra cycles go to stalls on memory:
each worker's inverse-BWT table competes for the same 20 MB last-level cache, alongside the
input, output and intermediate buffers.

<figure class="bf-fig">
<p class="bf-title">Two problems, one CPU</p>
<svg viewBox="0 0 720 124" role="img" aria-label="Two cards: dispatch efficiency, keeping workers supplied; decoding efficiency, making each block cheaper."><rect x="0" y="0" width="352" height="120" rx="12" class="bf-card"/><text x="20" y="32" class="bf-ts">Dispatch efficiency</text><text x="20" y="58" class="bf-t">Keep the workers supplied with jobs and</text><text x="20" y="78" class="bf-t">waste no time waiting.</text><text x="20" y="104" class="bf-tm">Futexes, the mutex, the scheduler, the channel, notifications</text><rect x="368" y="0" width="352" height="120" rx="12" class="bf-card"/><text x="388" y="32" class="bf-ts">Decoding efficiency</text><text x="388" y="58" class="bf-t">Spend fewer cycles and less memory</text><text x="388" y="78" class="bf-t">bandwidth on each block.</text><text x="388" y="104" class="bf-tm">BWT tables, prefetch, cache residency, buffers</text></svg>
<figcaption>Better dispatch uses the cores more fully; better decoding makes each core's work cheaper.</figcaption>
</figure>

These interact. Better dispatch keeps more inverse-BWT walks running at once, which raises
the competition for cache and claws back part of its own gain. That is why every experiment
is judged on the whole decoder's throughput and CPU-seconds per GB, never on one stage in
isolation.

## 8. What happens next

The diagnostic is built, frozen and checked, and the capture on the i9 comes next. It ends
in one of two places:

- **If receiver ownership really does keep free workers from available jobs,** at least 5%
  of their time in at least 80% of runs, a multi-consumer queue becomes a justified,
  isolated experiment.
- **If the receiver is usually free and the delay is scheduling or backlog,** that
  hypothesis is retired, and the effort goes to the larger problem: the inverse BWT's
  memory behaviour.

This post is a snapshot of an investigation in progress, and its figures come from the
traces available so far. A few lines of synchronisation code can decide how well a 32-lane,
latency-hiding decoder uses a whole multicore CPU. Whether they do here is a question for
the evidence.

---

**Sources:**

- the worker, scanner and reader loops: [`src/parallel.rs`](https://github.com/lmmx/mibz2/blob/master/src/parallel.rs);
- the i9 traces and the cycle accounting: [the resource attribution's analysis](https://github.com/lmmx/mibz2/blob/docs/decoder-attribution-records/docs/journal/2026-11-01-decoder-attribution-i9-10900K/analysis.md);
- the diagnostic's registered protocol: [`2026-11-01-dispatch-causality-protocol.md`](https://github.com/lmmx/mibz2/blob/research/dispatch-causality/docs/journal/2026-11-01-dispatch-causality-protocol.md);
- the trace-completeness checks: [amendment 1](https://github.com/lmmx/mibz2/blob/research/dispatch-causality/docs/journal/2026-11-01-dispatch-causality/amendment-1.md).
