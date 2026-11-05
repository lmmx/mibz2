# Split condition variables: the wake count, measured first

Astra's REVIEW 6651150258 ranked split condition variables third: one condition variable
for "the next block is ready" and one for "a slot is free", with targeted notifies. Its
falsifier was to count wrong-predicate wakes and runnable delay at window 16, and to reject
the change "if waits are not spurious-wake driven". This entry is that count. Nothing was
built.

## What waits on what

At master 0769b52f there is one `Condvar` in `parallel::Shared`. Two threads wait on it:

- **the reader** (`StreamDecoder::read`), for `ready[taken]` or the end;
- **the scanner**, for a slot: `issued - taken < slots`.

The workers wait on the jobs channel, not on the condition variable. A worker calls
`notify_all` after every block it finishes. That wakes the scanner, although only the reader
frees a slot. It also wakes the reader, although the reader can use only block `taken`. The
reader calls `notify_all` after every block it takes.

## Measured

`2026-11-05-split-condvars/cvcount.patch` adds relaxed atomic counters around the two waits:

- the waits;
- the wasted wakes, where the waiter woke and its predicate was still false;
- the time spent in `wait`;
- the `notify_all` calls.

It also adds `examples/cvcount.rs`, which decodes `wikidata-independent.bz2` (473 blocks,
235.8 MB) through `BzReader::parallel` at window 4 × threads, three rounds each.

The machine was the shared 4-CPU VM (Xeon @ 2.80 GHz, 1 thread a core, no SMT), so this is
local and indicative. It is a count, not a timing.

| threads | reader waits | wasted | scanner waits | wasted | reader / scanner time waiting | wall |
|---|--:|--:|--:|--:|---|--:|
| 2 | 258-296 | 17-78 (7-26%) | 260-306 | 31-96 (12-31%) | 666-737 / 507-578 ms | 0.69-0.76 s |
| 4 | 335-371 | 226-262 (67-73%) | 280-318 | 195-233 (70-73%) | 537-635 / 396-451 ms | 0.54-0.65 s |

There were 473 worker notifies and 473 reader notifies in every run, one each per block.

## Result: rejected by its own falsifier

- **The waits are real, not wake-driven.** At 4 threads the scanner spends about 70% of the
  wall waiting for a slot, because the window is full. The reader waits 98-99% of it,
  because the next block is not done. Both mean the workers bind; neither is a lost or
  late wake.
- **The wasted wakes come to about one a block** at 4 threads: about 460 for 473 blocks.
  - Each costs a futex wake, a lock and a sleep: microseconds.
  - It may also preempt a worker on a busy CPU.
  - Even at 10-20 µs a wake, that is 5-9 ms of CPU over about 2.4 CPU-seconds, 0.2-0.4%.
  - That is far below the 4-thread rows' ±2-5% resolution on CI.
- **So split condition variables are not built.** Reopen only if the decoder's shape
  changes so that waits become wake-bound: for example, many more threads than blocks in
  flight, or a much smaller window.
- **What else it says:** the scanner idles most of the 4-thread wall waiting for slots.
  Byte-budgeted admission and self-scanning workers (Astra's ranks 4 and 5) compete for
  the same slack, so it bounds them too: at window 16 there is little handoff time left to
  recover.
