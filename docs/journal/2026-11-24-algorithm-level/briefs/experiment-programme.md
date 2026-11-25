# The experiment programme: from "0.9 and 0.9 make 0.8" to a characterised decoder

*The Coordinator, 25 November 2026. This is the experiment the course mining and the design reviews lead to: what runs, in what order, what each run decides, who owns it, and what follows. It supersedes no registration; each run below has its own, frozen before its data.*

## Where it started, and why the problem had to be posed first

**The starting observation (the four-arm test, journal §44).** Two oracles each make the decoder faster on their own. Supplying the emission codes (arm B) gives 0.976 of the shipped time at 10 workers. Supplying the scanner's output (arm C) gives 0.806. Both together (arm D) give 0.741, so D/C = 0.920. A gain that is invisible alone (B) appears once another bottleneck is removed (C). That is the masked bottleneck: the "0.9 and 0.9 make 0.8" intuition, measured.

**Why that wasn't enough.** One ratio at one worker count says *that* two things interact, not *why*, nor where else they would. Without a model of the decoder, every proposed speedup is a guess about which resource it relieves. So the work went:
1. pose the problem (`../programme.md` steps 1–3);
2. measure the decoder across 1–20 workers (the scanner profile, §57);
3. read the performance-analysis literature in full (`../methodology/`);
4. have four independent readers mine it against the measurements (`../reports/2026-11-25-course-mining.md`);
5. have three independent reviewers check the experimental design (`character-review-decisions.md`).

What it bought:
- a two-station model that predicted the knee from four worker counts (7.49 predicted, 7.67 measured);
- a priced understanding of bottleneck migration;
- two engineering wins found by modification analysis;
- a design that can tell mechanisms apart.

## The runs, in order

Each run names its owner, its registration, what it decides and its rough workstation time. "Diagnostic" means it decides nothing about production on its own.

| # | run | owner | status | what it decides | time on the i9 |
|---|---|---|---|---|---|
| 1 | **stage 1, the character baseline** (`bench-character`, #841) | Pathfinder | queued (#842); ready tag awaits #841's green CI | nothing (C1a diagnostic). It *measures*, at every count 1–20: each thread's time by activity, per-visit demands, buffer occupancy, which limit binds, the reader's work, placement and achieved frequency, and the operational-law identities. It fixes stage 2's anchors and the noise that sizes stage 2 | about 23 min |
| 2 | **the scanner plumbing fix** (input copy at chunk boundaries; symbol buffer re-zeroing) | Optimiser (QUEUED 7147862124) | to build | an ordinary merge under PERFORMANCE-POLICY.md: CI's bench, plus a paired A/B at 8, 9 and 10 workers. Predicted: about 15 ms off the scanner, about 6% wall at 8 workers, 8–9% at 9–10 | about 5 min |
| 3 | **stage 2, the mechanisms** (`character-review-decisions.md` §2) | Pathfinder | drafting; frozen after stage 1 | which mechanism controls at the knee and above it. Screen (+20% per activity at stage-1 anchors); a 3×3 (scanner, workers) surface at k\*; contrasts (window × byte gate; placement × turbo at 4, the scanner's placement at 10 and 16; the 12-window order permutation; memory dose); the oracle arms B and C from the four-arm binary; 20% held out | about 45–60 min |
| 4 | **the worker-count recalibration** (7 against 8, and 9, through the public API) | Pathfinder or Optimiser | after run 2 merges | a production default: an adoption registration under RELEASING 5c's existing design, on the master that carries run 2, since a cheaper scanner moves the knee | about 13 min |
| 5 | **the joint frontier** (scanner and worker oracles together at 6, 8 and 10 workers, with the reader's activity measured) | Pathfinder | after stage 2 | prices combined improvements, with bottleneck migration measured rather than modelled. This is ChatGPT Pro's sequence step 3 | about 20 min |

Runs 1 and 2 can proceed together: run 2 is on the Optimiser's side and touches production code; run 1 is a research feature. Run 3 waits on run 1's data. Run 4 waits on run 2. Run 5 waits on run 3.

## The analysis, and where mathematics comes in

**Who:** MONODROMY owns the analysis of runs 1, 3 and 5. Its analysis plan is registered before the data (channel #39, part 2). The Coordinator merges and decides.

**What the analysis produces:**
1. **A validated model of the decoder.** A trace-driven simulation of the finite pipeline with its real admission predicate and in-order reader, checked against the 20 measured walls with conservation identities. Bounds come by resource, and MVA only on a documented surrogate (design decisions §0).
2. **A response map.** For each activity and each count: does making it cheaper move the wall? By how much, before the bottleneck migrates? This is the systems-biology picture of programme step 5: which "reactions" control the "flux", where, and how that shifts.
3. **The targets, re-ranked.** T1–T6 (`../math-targets/targets.tsv`), each with its measured demand, its control at each count, and the ceiling an improvement there could reach.

**Mathematics against engineering.** Each target that controls somewhere is classified:
- **engineering:** the cost is in how the work is done. Plumbing, buffering, placement, scheduling. Example: run 2;
- **mathematics:** the cost is in what work is done. The number of dependent reads, passes or state an algorithm needs (the referee: Q, D, P, S), where a different construction could need fewer. Examples are the Huffman scan's 1.5 ns a symbol, the workers' inversion, and emission. The composition principles (`../essence.md`: contraction and enrichment along composition) are the generator; the cost rules are the referee.

The hope, which the analysis tests: some controlling step is done the first way that came to mind, and a technique from the kitchen-sink mathematics changes what it costs.

## Revisiting earlier rejected ideas

Many ideas were tested earlier and rejected: the queue issue's Rejected list, and the registrations judged NOT ADOPTED. Several were rejected *on a decoder whose scanner bound everything above about 7 workers*. A worker-side gain could not show while the scanner held the wall, which is exactly the four-arm masking.

**The rule (decided here):** once the scanner's ceiling moves (run 2, then any scanner mathematics), re-qualify the rejected ideas whose rejection is plausibly masked. Not all of them, every time. Only those whose registration showed:
- a gain at counts below the knee but none above it, or
- worker-side demand reduction without wall gain.

Stage 1 and stage 2 data decide which those are. MONODROMY's analysis plan names the criterion (channel #39). The candidates are listed once the response map exists, and re-run in one batch on the new master.

## Timing

- **Tonight, if CI is green:** run 1 (23 min) on the workstation; run 2 built and CI-benched.
- **After run 1's bundle:** stage 2 frozen and run (about an hour on the i9); MONODROMY's analysis starts on stage 1's data.
- **Likely tomorrow:** stage 2's analysis, the recalibration (run 4), the joint frontier (run 5), then the classification of targets and the first mathematical attempts.

## Records

Every run's registration, bundle, report and decision goes where the earlier ones went:
- registrations: `docs/journal/`;
- bundles: the `evidence` branch;
- extracted TSVs and scripts: `../math-targets/data/`;
- decisions: journal sections and LOGs on #145.

The process itself is recorded in `../process/protocol.md`, so it can be reused.
