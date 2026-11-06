# External research report 126: the guardian's progress report and status update: serial work on P87 (verbatim)

A research input, not a finding of this repository.

- **Relayed:** by the maintainer, 2026-11-01 16:40 UTC.
- **Source:** the guardian.
- **In reply to:** the maintainer's question: "alright i built the prompt, is that all then? so to recoup: we are no working serially: just the claude code, no astra, and the dual work of astra and claude code was kept and consolidated into this prompt as work for claude? give me a progress report and status update".

**Evidence cutoff:** 1 November 2026, before any timed P87 experiment: a proposal and its review, not a performance result.

It is kept exactly as received (rendered text, cards flattened).

## The message, as received

`````text
Yes. That's exactly where we are. We've completed the parallel research and review phase, consolidated the findings, and returned to a single active engineering track: Claude Code.

Astra and the researcher aren't being abandoned; their work is now incorporated into the experiment Claude is preparing. There's no reason for another consultation until a concrete technical question arises.

# Progress report — 1 November 2026

Research direction selected

## 1. Our current position

Current phase

## P87 — Buffer-lifetime experiment

Protocol preparation and diagnostic implementation

1. Completed: Identify and measure the four-worker decoder slowdown.
2. Completed: Investigate MTF, walker instructions and dispatch as potential improvements.
3. Completed: Obtain independent proposals from Claude and Astra, with adversarial researcher review.
4. Current: Claude prepares and freezes the P87 diagnostic.
5. Next: Review the frozen protocol, then authorize one i9 capture if it passes.

## 2. What we've learned

Four-worker throughput

# \~869 MB/s

Shipped decoder on i9-10900K

Four-worker CPU cost

# 3.579

CPU-seconds per GB decoded

Inverse-BWT scaling penalty

# 0.68–0.72

CPU-s/GB, including associated work

Walk + assembly scaling penalty

# 0.609

CPU-s/GB, whole corpus

The evidence now points quite strongly toward the expense of concurrent inverse-BWT work, rather than simply too many instructions.

Four workers incur approximately 32% more CPU time per GB than one worker, with only 1.3% more retired instructions per byte.

We're investigating the cost of executing those instructions under shared-cache and memory-system pressure.

## 3. What happened to the other research directions?

| Investigation                           | Current status               | Why                                                                                      |
| --------------------------------------- | ---------------------------- | ---------------------------------------------------------------------------------------- |
| P74-C MTF                               | Closed negative              | Stage gains didn't clear full-decoder gates                                              |
| Unrolled BWT walker                     | Closed negative              | Faster standalone walk, negligible i9 four-worker throughput gain                        |
| MPMC receiver                           | Closed under registered rule | 1.90% direct obstruction against 5% threshold                                            |
| Wake-affine scheduling                  | Open, parked                 | Credible secondary opportunity, not yet experimentally tested                            |
| Claude's neighbour-interference harness | Preserved for later          | Potentially revealing, but synthetic calibration needs stronger controls                 |
| P87 buffer reuse                        | Selected next experiment     | Direct intervention on the actual decoder with relatively limited algorithmic disruption |

Importantly, we haven't concluded that dispatch is perfect or that the walker algorithm cannot improve. We've determined which hypotheses have earned additional engineering work at this point.

## 4. How the independent agents' work was consolidated

Astra

Proposed P87: reuse the now-dead `ll` storage for segment assembly instead of writing to the separate `rle` arena.

Contributed the intervention, difference-in-differences design, and physical mechanism hypotheses.

Claude

Independently reanalysed the archived decoder phases, finding that walk-plus-assembly accounts for much of the concurrency penalty.

Contributed the phase measurements and a broader interference-harness proposal, retained as a future option.

Researcher

Audited both proposals, corrected interpretations of memory counters, and refined the acceptance criteria.

Contributed the whole-corpus 0.609 CPU-s/GB figure, safeguards against false causal claims, and independent decision gates.

Guardian

Reconciled the findings and translated them into a bounded experimental instruction.

Selected P87 first, preserved alternatives, and set the boundary before real benchmarking.

So yes: Claude is now carrying out a consolidated direction informed by everyone's contributions, not simply following Astra's idea uncritically.

One nuance: Claude's full neighbour-harness proposal is preserved, not incorporated into the P87 implementation. We're not merging the two experiments. That would destroy the clean causal contrast we're trying to achieve.

## 5. Exactly what Claude is doing next

Claude's P87 preparation has five main jobs:

1. Implement isolated A/B decoder paths that use either separate `rle` storage or the freed `ll` storage, leaving production defaults unchanged.
2. Verify the T-vector builder and walker have equivalent machine code across arms, and keep allocations and initial page residency comparable.
3. Separate the timing of construction, actual traversal, assembly and RLE1/CRC.
4. Freeze a protocol with two independent decisions: whether the concurrency penalty falls by at least 0.10 CPU-s/GB, and whether four-worker CPU cost and throughput actually improve.
5. Complete correctness checks, analysis tests and untimed verification, then report the frozen commit and exact i9 command.

The historical P87 criterion at 6–10 workers remains untouched. This is a new four-worker diagnostic of its underlying mechanism.

## 6. What you need to do right now

i9 workstation

No action

You don't need to reserve cores, run `perf`, change affinities or prepare a benchmark environment yet.

Claude Code

Active track

Send your consolidated prompt if you haven't already, then let Claude complete the diagnostic preparation.

Astra and researcher

Standby

No new prompt needed. We can consult them again if Claude encounters an important machine-code issue or the frozen protocol reveals a substantive disagreement.

## 7. What success would look like

The immediate objective is not to achieve 1.5× in this single experiment.

We're asking whether buffer reuse can remove a meaningful portion of the memory-system penalty without changing the successful four-byte T-vector algorithm.

- If P87 materially improves concurrency efficiency and absolute four-worker performance, we have a candidate for production validation and a reason to investigate further buffer-lifetime optimizations.
- If it improves only assembly or RLE1, we may have a useful local optimization, but not an explanation of the walk-scaling penalty.
- If memory traffic falls without a CPU improvement, that tells us this particular traffic isn't a major limiting factor.
- If it has little effect, we move toward Claude's refined interference investigation rather than forcing another representation change.

### Overall assessment

We've moved from broad optimization exploration to evidence-led, targeted intervention.

The coordination has converged naturally. The next meaningful evidence will come from Claude's frozen P87 preparation and, after review, its i9 experiment.

So yes: we're working serially for now, with Claude Code as the only active agent. Astra's architectural insights, Claude's independent discoveries, and the researcher's critical corrections have all been accounted for. Nothing further is required of you until Claude returns with the prepared diagnostic.
`````
