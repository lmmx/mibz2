# The pipelined team: the registered run's result

7 November 2026. The run registered in `2026-11-07-team-pipeline-registration.md`, frozen at
f8aa9dc2. #247, head 7c59de46.

## The run

- **Bundle:** `team-pipeline/20261107T042622Z.tar.gz` on `evidence`, at 679d0fc6.
  - Its SHA-256 is f2f912f6c5db8d0962b087c9868875012bdbd64b490fd0fbce853e9fe7b0ac1b, checked
    against the notice on #145.
  - Louis ran it with `just submit` (RULES C5).
- **Machine:** Intel Core i9-10900K (10 cores, 20 MiB L3).
- **Validity:** VALID.
  - 20 rounds per cell, a clean tree, the frozen inputs unchanged, and `commit.txt` at the head.
  - The same output bytes from every run.
  - The noise control N1 (team3@10 against mibz2@10, the same plan) is 1.008 [0.992, 1.026],
    within 1 ± 0.03.
- **An earlier attempt** (`team-pipeline/20261107T042539Z`) was refused, correctly. The checkout
  held modified `bench/release-scaling/2026-11-06-…` files left over from the 6 November
  overwrite.

## The registered comparisons

Speed is the reference's wall over the arm's, so above 1 the arm is faster. CPU is the
reference's CPU over the arm's, so below 1 the arm spends more.

| | arm | against | speed [95% CI] | CPU [95% CI] | registered outcome |
|---|---|---|---|---|---|
| C1 (causal) | pipe3@10 (3 × 3) | team3@10 | 1.042 [1.016, 1.068] | 0.918 [0.895, 0.942] | SMALL GAIN |
| C2 (causal) | pipe4@12 (3 × 4) | team4@12 | 1.032 [1.020, 1.044] | 0.897 [0.884, 0.910] | GAIN |
| P1 (product) | pipe4@12 (3 × 4) | mibz2@10 (3 × 3) | 1.054 [1.039, 1.069] | 0.769 [0.758, 0.780] | GAIN |

**Medians:**
- pipe3@10: 0.146 s wall, 1.060 s CPU, 75 MB peak RSS;
- team3@10: 0.151 s, 0.959 s, 67 MB;
- pipe4@12: 0.146 s, 1.264 s, 75 MB;
- team4@12: 0.149 s, 1.137 s, 67 MB;
- mibz2@10: 0.154 s, 0.976 s, 67 MB.

**The traces** (occupancy, 431 blocks, medians over 6 complete rounds). These counts use the
corrected measure (#247's review), which credits a leader only with its own team's helpers:

| trace | MTF inside another block's run (cross thread) | same thread | RLE1/CRC overlapped |
|---|---|---|---|
| pipe3@10 | 147 | 225.5 | 153 |
| pipe4@12 | 140.5 | 217.5 | 145 |

## What it shows

- **Established, on this machine and corpus:**
  - Overlapping the leader's serial work with the helpers' build and walk makes a team faster
    at a matched plan: +4.2% at 3 × 3 on 10 threads, and +3.2% at 3 × 4 on 12. Both intervals
    exclude zero.
  - At 3 × 3, the gain falls below the registered GAIN threshold (a lower bound above 1.02). Its
    lower bound is 1.016, so it is recorded as SMALL GAIN, as registered.
- **It costs CPU:**
  - 8–11% more at a matched plan.
  - The product arm costs 30% more CPU than today's default, for 5.4% less wall. It also runs
    12 decoding threads against the default's 9, as the registration disclosed.
  - Part of the extra CPU is the spin-and-yield waits, which never park (#247).
- **It costs memory:** peak RSS rises 67 → 75 MB, about 3 MB per team, in line with the
  prototype's second front set and assembled buffer.
- **The overlap mechanism is present:**
  - About a third of blocks (147 of 431 at 3 × 3) had the leader's move-to-front run during its
    own team's build or walk of another block.
  - Small blocks (198 of the 431 are below `MIN_COOP`) give up an overlap each, as the
    registration records.
  - So the gain comes from a minority of blocks. More overlap per block is the obvious next
    lever.
- **What it doesn't separate:** the gain from the overlap, and the gain from the leader joining
  the walk as one more walking thread (#247, "Gain attribution"). The registration says so.
- **Unregistered cells, which decide nothing:** pipe4@10 (2 × 4) ran at 0.159 s against
  team4@10's 0.181 s. Two teams of four gained most from pipelining, which fits a leader that is
  idle for longer when its team is larger. Worth a registration if 2 × 4 becomes a candidate
  plan.

## Against the model

The knee journal's model is C1 = max(S, αP0) + H. The 3 × 3 result implies α ≈ 1.55 when
H = 0, which is near the top of the sensitivity range: the parallel stage slowed by about half
when the leader left it. That is near the break-even point the journal derived (about 1.6), and
it fits the small gain.

Reducing α (fewer spins, less contention for the build and walk) is a second lever, alongside
more overlap per block.

**α is not identified.** A stand-in Researcher review (an independent ChatGPT read, relayed by
Louis; not the Researcher task's REVIEW) refitted the 1.042 gain under different assumed handoff
costs:

| assumed handoff | fitted α |
|---|---|
| 0 ms | 1.545 |
| 0.05 ms | 1.468 |
| 0.10 ms | 1.392 |

These are conditional calculations with the model's approximate stage times, not a measurement
of the parallel stage. The 29–33% outside-inversion shares were measured at another team layout,
so they don't pin α down either. Cutting spin or parking overhead is a candidate improvement to
the implementation, and its result won't establish the inferred mechanism.

## What it decides, and what it doesn't

- **It is evidence for H1's mechanism question** under its registration, on this machine.
- **It does not change the default.** Adopting the pipeline into master still needs CI's paired
  bench (RULES C3, C5).
- **The CPU and memory cost needs weighing.** A default that spends 9% more CPU for 4% less wall
  on a 10-core machine is a trade-off to put to Louis, not to adopt by inference.

**Next:**
1. CI's paired bench of #247 must show that the default path is unchanged (it is opt-in).
2. Two levers on the prototype, each registered before it runs:
   - overlap on more blocks: stage small blocks too, so that `l.next` isn't left empty;
   - parking instead of yielding in the helpers' waits, to cut the CPU cost.
3. H2 (placement) stays deferred until per-thread affinity records exist.
4. The Guardian's own review of #247 is queued (QUEUED 6699704291).
