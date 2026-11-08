# The plan policy's bytes-per-L3 premise: the diagnostic's result

8 November 2026. This is registration 2 of `2026-11-07-general-plan-policy.md`. It was frozen at
9f13b4e8 and run at a5dcaebd (#254). It is a diagnostic (RULES C1).

## The run

- **Bundle:** `plan-bytes/20261108T093442Z.tar.gz` on `evidence` at a078ca88.
  - Its SHA-256 is 28c77deb369fce908f6c67d0540f665a8102b9d4b727ce51668d4a9bc69fbadd, which
    matches the notice on #145.
  - It was run with `just submit`, from the workstation queue.
- **VALID:** every check passed.
- **The cells:** one cell at 10 threads, unmasked, on the wikidata corpus recompressed at
  levels 6, 7 and 9. That is 20 rounds for each arm at each level.

**Conventions:**
- Speed is 4 × 2's wall over the arm's, so above 1 the arm is faster.
- In this report, CPU is the arm's over 4 × 2's, so above 1 the arm spends more.

## The family (each against 4 × 2, Bonferroni over 12 tails)

| level | arm | speed | word | median wall s (4×2 → arm) | median CPU s | peak RSS MB |
|---|---|---|---|---|---|---|
| 6 | 3 × 3 | 0.916 [0.890, 0.943] | SLOWER | 0.130 → 0.142 | 0.875 → 0.951 | 59 → 49 |
| 6 | 3 × 3 p | 1.163 [1.136, 1.191] | FASTER | 0.130 → 0.111 | 0.875 → 0.953 | 59 → 56 |
| 7 | 3 × 3 | 0.957 [0.942, 0.972] | SLOWER | 0.138 → 0.145 | 0.922 → 0.955 | 67 → 56 |
| 7 | 3 × 3 p | 1.204 [1.172, 1.236] | FASTER | 0.138 → 0.114 | 0.922 → 0.974 | 67 → 60 |
| 9 | 3 × 3 | 1.087 [1.067, 1.108] | FASTER | 0.163 → 0.150 | 1.083 → 0.952 | 79 → 65 |
| 9 | 3 × 3 p | 1.264 [1.242, 1.287] | FASTER | 0.163 → 0.128 | 1.083 → 1.057 | 79 → 73 |

## The registered reading

**PREMISE SUPPORTED.**
- Plain 3 × 3 against 4 × 2 has an upper bound below 1.03 at level 6.
- Its estimate at level 6 is below level 9's by more than both intervals' half-widths combined.
- So width 3's advantage depends on the bytes per team against the L3, as rule A's T_cap
  assumes.

This fails to refute the premise on one machine. It validates no extrapolation to other L3
sizes.

## Recorded, not judged

- **Pipelining beats 4 × 2 at every level, by 16–26%,** including the two levels where plain
  width 3 loses.
  - At level 6 it costs 9% more CPU.
  - At level 9 it spends 2% less CPU and 6 MB less peak RSS.
- **What T_cap gates is plain width 3, not the pipelined plan.**
  - The pipelined team's gain over plain 3 × 3 is large at every level: 0.142 → 0.111 s at
    level 6, 0.150 → 0.128 s at level 9.
  - Rule A's ⌊L3 / 6 MB⌋ cap may be too conservative for the pipelined plan, which is the
    plan it actually picks.
- **This agrees with plan-boundaries,** where pipelined plans beat 4 × 2 at 8 and 9 threads.
  The revised rule should be registered for pipelined plans throughout, with the bytes premise
  re-tested on pipelined arms.

## What it decides

Nothing ships. Rule A's premise survives this machine. The next registration is the revision
(a pipelined fallback, and T_cap tested on pipelined arms). The levels diagnostic is still to
land.
