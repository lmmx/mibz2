# Review brief: the experimental design for the decoder's performance character, before it is frozen

*Louis, 24 November 2026: "shall we go back to antigravity neon/monodromy and review the plan for our experimental analysis of the toy version of the program before we then set out to model it from its results? so that we get the most informative results?"*

**One round, three independent reviewers:** MONODROMY, Neon and ChatGPT Pro. The Coordinator merges the answers and decides, then the Pathfinder freezes stage 2.

**Stage 1 (the baseline at k = 1–20) proceeds meanwhile.** It is not in question, and its data will size stage 2.

## What to review

The plan is `briefs/character.md`, with amendments §7a and §7b, at this commit. The inputs:
- `programme.md`, steps 4–7;
- journal §§44, 47–54;
- `methodology/performance-courses.md`;
- `external/chatgpt-decomposition.md`;
- once it lands: the scanner-profile results (`bench-scan-profile`, run 24 Nov from 22:37 UTC; its bundle will be on #145).

## The questions

1. **Which competing hypotheses must the design tell apart?** List the mechanisms that could explain the 1–20 curve and the four-arm result. For example:
   - scanner-bound;
   - consumer- or admission-bound (issued − taken, the 12 MiB ready cap);
   - long-tail-block-bound;
   - memory-bandwidth-bound;
   - L3-capacity cliff;
   - SMT or core sharing (k + 3 threads);
   - turbo frequency droop;
   - Open construction-bound or emission-bound.

   For each pair that the current design cannot separate, name the observation or intervention that would. This is the identifiability question: are the model's parameters determined by the planned data?
2. **The most informative design for a fixed budget.** The workstation budget is about 2–3 hours of runs in total.
   - **Grid:** is the 1–20 baseline with 12 randomised rounds right?
   - **Anchors:** are {1, 6, 8, 10, 12, 20} right?
   - **Dose and factorial:** is +20% with +10% follow-ups the right dose? Is a 2^(9−4) resolution-IV factorial at k = 8 and 10 the right size, or would a sequential design do better (screen, then a response surface around the knee)?
   - **Stage 1 data:** what should stage 1's data decide about stage 2 (stopping rules, re-allocating rounds)?
3. **Measurement validity.** Watch for "producing wrong data without doing anything obviously wrong" (Mytkowicz et al., 2009): environment size, link order, ASLR, frequency state, cache warmth, process start-up. Which of these could bias a 1–20 sweep on the i9, and how should they be randomised or controlled?
4. **What the model needs from the data.** For each model:
   - the discrete-event simulation;
   - MVA and bounds on the closed network;
   - a fluid approximation;
   - a roofline per activity;
   - modification analysis.

   Is every input logged? The inputs are device demands, visit counts, C² of per-block service times, class (route) membership, and admission and ready-byte occupancy. What is missing?
5. **What should be cut?** Which planned measurements will not change any decision, and so are not worth the workstation time?

6. **Queueing discipline and topology** (Louis's question, `leads/queueing-discipline.md`). The decoder is a closed FCFS network with an in-order resequencer.
   - Does the design measure **efficiency against the bound** at each k, and does it record the losses from head-of-line blocking, resequencing and drain?
   - Which alternative disciplines are worth replaying in the simulation with measured per-block demands: size-aware dispatch using record 122's free counters; helpers for the head-of-line block; work-based admission?
   - What would each need logged?

**The answer format:** short, concrete, and marked proved, argued or conjectured. Name the exact changes to `briefs/character.md`. Don't restate the plan.
