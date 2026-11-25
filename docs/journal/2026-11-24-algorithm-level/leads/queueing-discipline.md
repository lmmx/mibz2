# Lead: change the decoder's queueing discipline or topology, not only its stage demands

*Louis, 24 November 2026: "do you think it might be possible that we have got software with a somehow conventional form of queueing that we could entirely replace with one of the other forms and this might allow some far better perf?"*

## What the decoder's queueing is today

It is read from `src/parallel.rs` on master.
- **A closed network.** A fixed population of at most `slots` blocks (W = 4k on the workers path) is in flight.
- **Admission.** It is `issued − taken < slots`, plus the ready-output cap `READY_BYTES` = 12 MiB with a floor.
- **Dispatch.** A single serial scanner issues blocks *in stream order*; workers take jobs first-come first-served.
- **Completion.** A **resequencing** stage: the ordered reader takes block `taken` and nothing later.
- **Teams.** The pipelined plans put several threads on one block (leader and helpers).

This is the textbook conventional form: FCFS dispatch, a fixed-population window, and in-order resequencing at the exit.

## What queueing theory says can and cannot be gained

- **The ceiling does not move with the discipline (proved, operational laws).** For a work-conserving system in steady state, throughput is bounded by X ≤ min(N/(D + Z), 1/D_max) and by k/b: the demands set the ceiling. Re-ordering work cannot raise it. Only reducing demands (mathematics, engineering) or adding capacity at the bottleneck (a parallel scanner) does.
- **What the discipline can change is how close the decoder gets to that ceiling (argued).** This system departs from the steady-state, work-conserving ideal in exactly the ways that leave room:
  - **Head-of-line blocking at the resequencer.** One slow block holds the window, because `taken` cannot advance past it. The scanner stalls, and workers idle while later blocks sit finished in `ready`. This is the *resequencing-delay* problem, with its own literature.
  - **A finite stream.** Start-up and drain matter. The last blocks form a straggler tail: a makespan problem, not a steady-state one.
  - **Variable job sizes.** By M/G/1, waiting grows with E[S²], not only E[S]. Long-tail blocks are the mechanism.

## Concrete alternatives

Each is a known form, and each is testable.

| alternative | what it changes | known from | what makes it possible here |
|---|---|---|---|
| **Size-aware dispatch** (LPT / longest-expected-first within the window) | It starts big blocks early, so they don't become the straggler or the head of the line. | Makespan scheduling (Graham's LPT); Harchol-Balter, task assignment by size | **Job sizes are nearly free.** Record 122 shows the Huffman scan yields n, r, h₁ and the run-width histogram as counters, so a block's expected inversion cost is known before dispatch. Scheduling theory's strongest results assume known sizes. |
| **Priority to the head of the line** (oldest unfinished block gets helpers) | It shortens the block that blocks the resequencer. | Least-attained / priority scheduling; work stealing | The team and helper machinery exists (`Plan::Teams`). The question is the trigger: give helpers to the block at `taken` when it lags. |
| **Admission by work, not by count** | The window holds a bounded amount of *work*, so a few huge blocks can't fill it. | Closed networks with load-dependent population; byte-based flow control | `READY_BYTES` already caps output bytes. Capping admitted *expected work* is the analogue on the input side. |
| **More than one doorman** | It turns the serial scanner into a station with parallel servers. | Cross-block parallel parse (lbzip2; record 122) | `briefs/scanner.md`, part 2. This is a topology change, and the only row that raises the ceiling. |
| **Relaxed resequencing** | Completed out-of-order blocks no longer hold admission credit while they wait. For example, spill or stream them, or write positionally once their offsets are known. | Resequencing buffers | It is limited by the output contract: a byte stream must be written in order, and offsets need prefix sums of lengths. A positional file sink could relax it. |

## How to tell, before building any of it

**The key quantity is efficiency against the bound:** at each k, the measured throughput divided by min(N/(D + Z), 1/D_max, k/b).
- **If efficiency is near 1:** discipline cannot help. Only lower demands (the mathematics) or more capacity at the bottleneck will.
- **If efficiency is well below 1:** the gap is what the queueing discipline loses. Stage 1's records show where it goes: workers idle while the scanner is blocked on admission; finished blocks waiting in `ready` behind the head; the drain tail. A discrete-event simulation with the *measured* per-block demands can then replay each alternative above (size-aware dispatch, head-of-line helpers, work-based admission) and predict its gain, before any code changes. That is modification analysis applied to the discipline rather than to a demand.

Stage 1 already logs what this needs: per-block issued, enqueued, got, ready and taken timestamps; per-activity demands; and route class (Pathfinder, PR #841). This analysis is added to the design review (`briefs/character-design-review.md`, question 6).
