# 2026-11-05: Workers wait for jobs without holding the jobs' lock: registration

Registered before any CI timing of this branch. The decision rule may be amended only in a
commit made before the first bench comment on this PR, and each amendment says what changed
and why.

## The change

The census (`2026-11-05-binding-census.md`) found an idle worker holding the jobs'
`Mutex<Receiver>` across `recv`. At 4 threads that is 3.0–3.5% of worker time in the lock,
and 1.7–2.4% idle while a job sits queued (`i:q`, OBSERVED in occupancy runs 42009406264 and
41976148952).

- **The queue.** `channel::<Job>()` behind `Arc<Mutex<Receiver>>` is replaced by a private
  `JobQueue`: a `Mutex<{VecDeque, waiting, receivers, closed}>` and a `Condvar`, std only.
- **Its ends,** `JobTx` and `JobRx`, behave like a channel's:
  - `send` pushes under the lock and notifies one waiter after unlocking, only if one waits;
  - `send` fails once every receiver is dropped (unwinding included);
  - `recv` pops FIFO or waits on the condvar, which gives the lock up, and returns `None`
    only once the queue is closed and empty;
  - the sender's drop closes the queue and wakes every waiter.
- **What does not change:** ordering, the window, the cancel and drain paths, the error paths,
  the ordered output and the public API. The trace marks `JobLock` and `JobRecv` keep their
  points.
- **New tests:**
  - 5 `miri_queue_*` unit tests;
  - `tests/handoff.rs`: thousands of tiny streams across 8 thread and window configurations,
    repeated decodes, cancel and drop mid-stream at 1–16 threads, and errors in order among
    small blocks, each under a 300 s watchdog.
- **`miri.yml`** now also triggers on `src/parallel.rs`.

## Measured, untimed

All OBSERVED, with files in `2026-11-05-job-handoff/`.

- **Code size.**
  - The job receive is `JobRx::recv`: 151 instructions, against std mpmc's 0x1219-byte `recv`.
  - The Segmented worker body shrinks from 734 instructions to 678.
  - The 7 mpmc instantiations for the job type are gone.
- **Instructions.** Callgrind on a synthetic of both handoffs: about 900 → 333 Ir a job.
  Against a block's work this is negligible: the lever is wake-ups and latency.
- **Lock-caused waits** in the synthetic: a returning worker that finds a job queued but the
  lock held. Per job, old → new:
  - 4 threads, window 16: 0.010–0.013 → 0.000–0.003;
  - 4 threads, 80% short blocks: 0.021–0.045 → 0.001–0.007;
  - 8 threads, window 32: 0.030–0.071 → 0.000–0.004.
- **The real decoder at 4 threads**, on a 583-block Wikidata slice, 6 interleaved pairs: context
  switches per block fall, voluntary −8% and involuntary −9%.
- **The real decoder at 2 threads:** involuntary switches rise, 0.136 → 0.255. The old `recv`
  spun and yielded before parking, which absorbed gaps of a few microseconds. This is the row
  to watch.
- **Tests:** `cargo test` passes in full. `miri_queue_*` passes 5/5 under Miri, twice. Clippy
  and fmt are clean. Negative controls: a queue that never notifies, and one whose close wakes
  no one, each fail the queue tests.

## The prediction, recorded before any run

- **4-thread wall:** −0.2% to −0.8%. The ceiling is the `i:q` share × the 0.59–0.72
  idle-to-wall conversion (#180), about ≤ 1.0–1.7%.
- **4-thread CPU:** flat within ±0.3%.
- **1-thread and 2-thread wall:** flat.
- **The mechanism, in an occupancy run** on the master baseline's CPU model:
  - `t.w.lock`: 3.0–3.5% → ≤ 0.3%;
  - `i:q`: 1.7–2.4% → about 0.8–1.6%.

The wall effect is probably inside the PR bench's 4-thread noise (±2–5%).

## The decision rule

- **Correctness (required):** CI green, Miri included, and the same bytes on every corpus in
  every bench run.
- **Benefit (required):** `wikidata` 4-thread wall or CPU faster beyond noise in at least one
  run, with the same figure's point estimate not slower in the other. Without that it is
  reported as not shown and does not merge, whatever the mechanism figures say.
- **Mechanism (reported, not a substitute for benefit):** one occupancy dispatch on a runner of
  a model with a master baseline. `t.w.lock` above 1% at 4 threads refutes the change.
- **No harm (required):** a wall row flagged slower is a harm signal, settled by one more run
  on the same head. Flagged again, it is a harm and the rule fails. The 2-thread rows are read
  first, for the short-gap risk. At most three runs.
