# 2026-11-05: Workers wait for jobs without holding the jobs' lock: benefit not shown (closed)

Registered before any CI timing of this branch (PR #203). The result is at the end; the
registration above it is as committed on the branch. The decision rule may be amended only in a
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

## Result: benefit not shown, closed unmerged

The benefit the rule required was not shown, so #203 was closed unmerged. The branch
`perf/job-handoff` is kept. Both bench runs are in `2026-11-05-job-handoff/`.

| run | CPU | 4t wall | 4t CPU | 4t amplification, head over base |
|---|---|--:|--:|--:|
| 42057901182, attempt 1 | EPYC 7763 | −0.9% (−3.3, +1.6) | −0.4% (−1.4, +0.6) | −0.5% (−1.8, +0.8) |
| 42057901182, attempt 2 | EPYC 9V45 | +0.9% (−1.3, +3.2) | **+3.1% (+1.5, +4.7) slower** | **+3.5% (+0.6, +6.6) more** |

- **Benefit:** neither run was faster beyond noise. Run 2's wall and CPU point estimates are both
  slower, so a third run could not meet the rule. None was dispatched.
- **No harm (wall rows):** no wall row was flagged slower in either run. The 2-thread wall rows
  were +1.4% and −0.4%, neither significant.
- **Correctness:** the same bytes on every corpus in both runs; CI and Miri green.
- **Mechanism** (OBSERVED, occupancy run 42057901176, 9V45; `occupancy-42057901176-9v45.txt`):
  - 4-thread `w.lock`: 0.0085% of worker time, against the predicted ≤ 0.3%;
  - `i:q`: 1.6%, against the predicted 0.8–1.6%.

  The baselines were measured on the 9V74 and 7763, so the before → after crosses models.
- **Prediction against outcome:** 4-thread wall was predicted −0.2% to −0.8% and was not resolved
  in either run. 4-thread CPU was predicted flat within ±0.3% and rose 3.1% on the 9V45.

**What it says** (INFERRED):
- The change removed the lock time and some of the queued idle, but 4-thread wall did not move.
- On the 9V45, the workers' summed CPU rose, 0.50 → 0.53 s (the bench's CPU attribution), and
  so did amplification.
- At 4 threads on 2 SMT cores, a worker's idle gives its sibling the whole core. Filling that
  idle puts more walks beside walks at the co-run penalty, ×2.1–2.2 on AMD
  (`2026-11-05-binding-census.md`), and gains no throughput.
- This agrees with the census: the 4-thread row is bound by core throughput, not by the
  handoff.

**What it changes:**
- Removing handoff idle is closed where threads exceed physical cores.
- It changes no premise at one thread a physical core, wikidata-pq's configuration. There no
  SMT sibling takes up a worker's idle, so idle removed could be wall gained.
  - The 2-thread row, one thread a core, was flat, but its queued idle is about 0% (occupancy).
  - At higher core counts the case is unmeasured.
- It bears on the pair matrix (`2026-11-05-corun-placement.md`). The idle it filled was worth
  nothing in wall, which is what a walk costing about 2× beside a busy sibling predicts. The
  matrix measures that cost directly.

