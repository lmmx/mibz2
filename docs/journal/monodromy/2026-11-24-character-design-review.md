# Record 128: review of the experimental design in briefs/character.md (channel #38)

*MONODROMY, 24 November 2026. One round, short. Everything is argued unless marked. Changes are named by section of `briefs/character.md`.*

## 1. Identifiability: which mechanisms the design cannot yet separate

| pair | why the planned data confounds them | what separates them | change |
|---|---|---|---|
| scanner-bound against admission-bound | both leave workers idle | scanner on-CPU against scanner blocked on admission (logged); and the response to the window W, which only admission-bound has | §6: make the W test unconditional at the knee, W = 4k against 8k |
| admission-bound against long-tail-block-bound | a long block is one way admission binds | tag every admission-blocked interval with the block holding the oldest credit; and rerun with the same blocks in shuffled order | §3: log the head-of-line block; §6: one shuffled-order run, held out for the simulation to predict |
| bandwidth against L3 capacity | both rise with k and both raise LLC misses | two antagonists that load one and not the other: non-temporal streaming writes (bandwidth, no cache), and a resident 4–8 MB read loop (cache, no bandwidth once warm). Also a corpus of smaller blocks: the table shrinks, bytes per output byte do not, so a capacity cliff moves and a bandwidth limit stays | §6: add both antagonists at k = 10 and 20, and one small-block corpus |
| SMT against turbo | moving k threads onto sibling pairs also changes the number of active cores, so the clock changes | cross them: placement (distinct cores, sibling pairs) × turbo (free, locked), four cells at fixed k | §6: run these two as a 2×2, not separately |
| SMT against everything, in the unpinned primary | the scheduler decides how many cores are in use, and it is not recorded | log the CPU each thread ran on at every event | §3: add it |
| construction against emission | none: separate timers and separate pacing | already separated | none |
| consumer control | A10 is paced only if its demand is material, but it gates admission whatever its demand | covered by the shuffled-order run and the head-of-line tag | none |

**One structural point (proved, for J = min):** one-sided slowdowns cannot locate a bottleneck switch, since the response is flat on one side. The only planned speed-ups are the oracles. Keep arms B and C in stage 2 at the knee as the other half of the response.

## 2. The most informative design for the budget

- **Grid:** 1–20 at 12 randomised rounds is right. Keep it.
- **Anchors:** do not fix them now. k = 1 teaches nothing under pacing: with one worker every coefficient is that activity's time share, which the timers already give. Drop pacing at 1, and at 6 keep only A3 and A7 as a check. Choose the rest from stage 1: the knee k*, k* ± 2, and 20.
- **The 2^(9−4) factorial: cut it.** In a system close to a minimum, the main effects are time shares or zero, and the one interaction that matters is scanner against workers. Resolution IV aliases two-factor interactions with each other and spreads 32 runs over effects that are zero by structure.
- **Do instead, in sequence:**
  1. *Screen:* one coordinate at a time, +20%, at k* and 20.
  2. *Surface:* a 3×3 on (scanner dose, all-worker dose) in {0, 10, 20%}² at k*. Nine conditions. It measures the roundness of the knee, which is the parameter the simulation and the fluid model most need.
- **Dose:** +20% is right. Use +10% only inside the surface.
- **What stage 1 decides:**
  - the paired spread σ, hence rounds: enough for a half-width of 0.1, capped at 12;
  - k*, as the count where scanner on-CPU share passes 0.9;
  - which coordinates to drop: any activity under 2% of its thread's time, since its coefficient cannot exceed its share while workers bind and is zero while the scanner binds;
  - stopping: stop a coordinate once its interval excludes 0.1.

## 3. Measurement validity

1. **One binary for every arm.** Pacing and oracles must be runtime switches. Separate builds change code layout, which is the Mytkowicz effect, and it would be read as a treatment.
2. **Environment:** run under a fixed, minimal environment. Leave address randomisation on, so twelve rounds average over layouts.
3. **Heat and clock:** a 2–3 hour session drifts. Rounds already block it. Add: record mean frequency and package temperature per observation; discard the first observation after any idle gap; carry the previous run's k as a covariate, since a run after k = 20 starts hot.
4. **Warmth:** hold the corpus resident in memory; discard each process's first decode.
5. **Placement:** covered in §1; without the CPU log the unpinned primary cannot be interpreted past 8 workers.
6. **Aggregation:** fit per distinct block, then pool. Pooling first can reverse a trend when block mix and k interact.

## 4. What each model needs that is not yet logged

| model | missing input |
|---|---|
| discrete-event simulation | the queue's service order and which worker takes which block; the head-of-line block for each admission wait; serial correlation of service times in corpus order (computed from the log, no new logging) |
| bounds on the closed network | nothing. Use bounds only: service is not exponential and buffers block, so exact mean-value analysis does not apply. Population is the window, not k |
| fluid | nothing |
| roofline per activity | dependent reads per block (links followed, one increment each) and streamed bytes per activity (computed from n, r, m, not measured). These also give record 124's unit price |
| modification analysis | the same unit prices |
| all | CPU id per event; frequency per observation |

## 5. What to cut

- Pacing at k = 1, and all but two coordinates at 6.
- The fractional factorial.
- A6a, A6b, A6c as three coordinates: pace A6 as one unless a part exceeds 5% of worker time.
- A8 pacing where the fallback route is rare.
- +10% follow-ups outside the surface.
- Mean and peak RSS as per-observation outcomes: record peak once per count.
- Uncore counters except at 10 and 20.
- From §6: the second in-process decode, the real output sink, and the dense and high-expansion corpus classes. They change the workload, not the mechanism. The small-block corpus stays, as §1's discriminator.
- The public-API sweep: one round.

**Net:** roughly 40% fewer paced observations, spent on the knee, the two antagonists and the 2×2.
