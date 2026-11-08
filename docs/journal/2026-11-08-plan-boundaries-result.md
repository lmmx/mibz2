# The plan policy's boundaries: the diagnostic's result

8 November 2026. This is registration 1 of `2026-11-07-general-plan-policy.md`
(`2026-11-07-plan-boundaries-registration.md`). It was frozen at 9f13b4e8 and run at a5dcaebd
(#254). It is a diagnostic (RULES C1), so it can't qualify a production change.

## The run

- **Bundle:** `plan-boundaries/20261108T092231Z.tar.gz` on `evidence` at ad4f7321.
  - Its SHA-256 is f8a6eeaef4814fb992bf8678ebb600e6b5a6eb097c38926df3249654f82d6e5e, which
    matches the notice on #145.
  - It was run with `just submit`, from the workstation queue.
- **VALID:**
  - 320 runs, none failed, with the same bytes from every run;
  - every cell under its registered mask;
  - every run's observed plan was the forced one;
  - a clean tree, the frozen inputs unchanged, and the commit at the head.
- **The controls** compare today's default with 3 × 2 forced, the same plan. They came out at
  1.001 (unmasked, 6 threads) and 1.005 (masked to 6 cores).

**Two conventions:**
- Speed is 4 × 2's wall over the arm's, so above 1 the arm is faster.
- The report's CPU column is the arm's CPU over 4 × 2's, so **below 1 the arm spends less.**
  That is the reverse of the convention in the pipeline journals.

## The family (each against 4 × 2, Bonferroni over 14 tails)

| cell | arm | speed | word | median wall s (4×2 → arm) | median CPU s | peak RSS MB |
|---|---|---|---|---|---|---|
| U8 (8 threads, unmasked) | 3 × 2 | 0.989 [0.969, 1.009] | UNRESOLVED | 0.165 → 0.167 | 1.097 → 0.850 | 82 → 67 |
| U8 | 2 × 4 p | 1.079 [1.055, 1.102] | FASTER | 0.165 → 0.153 | 1.097 → 0.949 | 82 → 57 |
| U9 (9 threads, unmasked) | 3 × 3 p | 1.141 [1.119, 1.163] | FASTER | 0.165 → 0.145 | 1.094 → 1.039 | 82 → 75 |
| M8 (8 cores, one CPU each) | 3 × 2 | 0.988 [0.975, 1.002] | UNRESOLVED | 0.161 → 0.163 | 1.042 → 0.829 | 82 → 67 |
| M8S (8 cores, both CPUs) | 3 × 2 | 1.011 [0.986, 1.036] | UNRESOLVED | 0.166 → 0.164 | 1.109 → 0.834 | 82 → 67 |
| M9 (9 cores, one CPU each) | 3 × 2 | 1.001 [0.985, 1.018] | UNRESOLVED | 0.164 → 0.165 | 1.088 → 0.835 | 82 → 67 |
| M9 | 3 × 3 p | 1.146 [1.127, 1.166] | FASTER | 0.164 → 0.143 | 1.088 → 0.967 | 82 → 75 |

## The registered readings

- **The shared 9-thread cell supports both rules:** 3 × 3 p is 14% faster than 4 × 2, unmasked.
- **Rule A is FALSIFIED at 8 threads.**
  - At 8 threads rule A keeps the old width-2 plan, 4 × 2, because three pipelined teams of
    three don't fit. Yet 2 × 4 p, two pipelined teams of four, is 7.9% faster.
- **Rule B is neither falsified nor supported at 8 threads.**
  - 3 × 2 is UNRESOLVED at all three 8-thread cells, with point estimates of 0.988 to 1.011.
  - Its upper bounds (1.002 to 1.036) don't all fall below 1.03, so its predicted gain is not
    excluded either.
- **The 9-core mask (information only):** 3 × 3 p is 14.6% faster than 4 × 2 with no core left
  over. Rule A's core-left-over condition (3·T_cap ≤ P − 1) is unnecessary on this machine.
  The 9-core shape becomes a candidate envelope entry, by its own registration.
- **It shows nothing about the L3 extrapolation:** no cell changes the L3.

## What it shows beyond the readings (recorded, not judged)

- **The old width-2 plan is dominated on this machine wherever a pipelined plan fits.**
  - 2 × 4 p beats 4 × 2 at 8 threads on all three counts: wall, CPU (13.5% less) and peak RSS
    (57 against 82 MB).
  - 3 × 3 p beats it at 9 threads on all three too.
- **Rule A's error is its fallback, not its width-3 choice.** When three teams of three don't
  fit, the rule falls back to unpipelined width 2. Pipelined teams of four, fewer of them, are
  the better fallback.
  - This fits #247's unregistered pipe4@10 (2 × 4) cell, 0.159 s against team4@10's 0.181 s.
  - It fits the knee journal's account too: a larger team leaves its leader idle for longer,
    so pipelining recovers more.
- **3 × 2 against 4 × 2 is a wash on wall, and much cheaper.** It spends 19–25% less CPU and
  15 MB less peak RSS, at an unresolved speed.
- **The unmasked cells ran at lower clocks.** The mean sampled frequency was about 2,030 MHz
  unmasked, against about 2,850 MHz masked to one CPU a core. The masked cells were not slower
  for it, so the frequency figure is per-sample and not comparable across masks.

## What it decides, and what it doesn't

- **Rule A as designed is not made the default.** #254 stays an unmerged research base.
  Rule A is falsified at 8 threads, and the envelope already keeps 0.2.6's choice on the ten-core
  topology, so nothing ships changes.
- **The next design step** is a revised rule whose fallback is pipelined teams of four when three
  teams of three don't fit. It needs its own registration, against 4 × 2 and 3 × 2, at 8
  threads and below.
- **The plan-bytes and plan-levels diagnostics** are still to land. They test the L3
  extrapolation and the levels, which this run doesn't touch.
- **Qualifying registrations 3 and 4** still wait on Louis's A/A method decision (C6).
