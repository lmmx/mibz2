# The pipelined team's levers: the registered run's result

7 November 2026. This is the run registered in `2026-11-07-pipeline-levers-registration.md`
(on research/team-pipeline-levers, #249), frozen at 1c853cfc, at head 7a5be722.

## The run

- **Bundle:** `pipeline-levers/20261107T091926Z.tar.gz` on `evidence` at 808dc913.
  - Its SHA-256 is ba6e2c98848f6b96b2b31d7a3cc15a500b32fcf7599b70ca524cac40ef465a4f, which
    matches the notice on #145.
  - Louis ran it with `just submit` (RULES C5).
- **Machine:** Intel Core i9-10900K.
- **VALID:**
  - 20 rounds in every cell, with no unregistered cells;
  - a clean tree, the frozen inputs unchanged, and `commit.txt` at the head;
  - the same bytes from every run;
  - the noise control N1 at 1.007 [0.987, 1.027].

Ratios are the reference over the arm: above 1, the arm is faster (speed) or spends less (CPU).

## The registered comparisons (each lever against the pipe)

| | lever | plan | speed | outcome | CPU | outcome |
|---|---|---|---|---|---|---|
| A1 | park | 3×3@10 | 0.999 [0.981, 1.017] | NULL | 1.055 [1.039, 1.071] | CPU SAVING |
| A2 | park | 3×4@12 | 0.995 [0.981, 1.010] | NULL | 1.051 [1.037, 1.065] | CPU SAVING |
| S1 | spin2k | 3×3@10 | 1.004 [0.984, 1.024] | NULL | 1.006 [0.986, 1.027] | NULL |
| S2 | spin2k | 3×4@12 | 1.005 [0.994, 1.016] | NULL | 1.003 [0.991, 1.015] | NULL |
| B1 | small | 3×3@10 | 1.043 [1.020, 1.066] | GAIN | 1.007 [0.987, 1.028] | NULL |
| B2 | small | 3×4@12 | 1.034 [1.023, 1.045] | GAIN | 0.994 [0.982, 1.006] | NULL |
| AB1 | both | 3×3@10 | 1.038 [1.021, 1.055] | GAIN | 1.032 [1.018, 1.046] | SMALL CPU SAVING |
| AB2 | both | 3×4@12 | 1.033 [1.023, 1.043] | GAIN | 1.031 [1.018, 1.044] | SMALL CPU SAVING |

**Lever B's traces** (medians over 6 complete rounds, out of 431 blocks):

| plan | arm | same-team overlap | join delay (summed) | late joins / runs |
|---|---|---|---|---|
| 3×3@10 | pipe | 144 | 14.3 ms | 62 / 233 |
| 3×3@10 | small | 236.5 | 26.0 ms | 113 / 233 |
| 3×3@10 | both | 229.5 | 22.4 ms | 100.5 / 233 |
| 3×4@12 | pipe | 147 | 28.1 ms | 91.5 / 233 |
| 3×4@12 | small | 219 | 40.9 ms | 131.5 / 233 |
| 3×4@12 | both | 229 | 37.6 ms | 124.5 / 233 |

The mechanism is SHOWN at every plan: overlap rises by 72 to 92 blocks, well above the 10%
threshold of 43 blocks. So is the counterexample, a LATER JOIN, at every plan.

## What it shows

- **A: parking cuts CPU by about 5% while the wall holds.** That is lever A's registered
  question, answered YES at both plans.
- **S: the spin budget is not a lever.** Going from 4,096 to 2,000 spins moved neither figure.
  Park's CPU saving therefore comes from parking, not from spinning less. This is the
  stand-in Researcher's confound, now resolved by measurement.
- **B: priming the pipeline after small blocks makes it about 3–4% faster at no CPU cost.**
  - Same-team overlap rises from about a third of blocks to about half.
  - The leader does join the walk later, as the stand-in Researcher predicted, but the speed
    gain survives it.
  - The outcome map has no row for this combination (SHOWN, LATER JOIN, GAIN); its later-join
    row expects NULL or REGRESSION. **Recorded reading, not a registered verdict:** the later
    join is real and is paid for, but more overlap outweighs it on this machine and corpus.
  - A cap on the jobs a leader takes in one run, the map's next step for the counterexample,
    stays a candidate refinement: it might keep the overlap and recover some of the join
    delay.
- **AB: the levers combine, with some interaction.**
  - The combined speed is about B's.
  - The CPU saving (3.2%) is smaller than park's alone (5.5%), as the report's interaction
    check flags. The likely reason is that a busier leader leaves less waiting to park, but
    that is not established.
- **Gain attribution is as before.** The run can't separate the overlap's gain from the
  leader's extra walking.

## Against today's teams (reported, never judged)

These comparisons were reported, not registered as decisions:

| arm | against | speed | CPU |
|---|---|---|---|
| pipe3@10 | team3@10 | 1.033 [1.017, 1.049] | 0.922 |
| pipe3-small@10 | team3@10 | 1.077 [1.053, 1.102] | 0.929 |
| pipe3-both@10 | team3@10 | 1.072 [1.053, 1.091] | 0.951 |
| pipe4-both@12 | team4@12 | 1.074 [1.058, 1.090] | 0.924 |

- **The pipeline's cost has shrunk.** The plain pipeline cost about 8% more CPU for 3% less
  wall. With both levers, at the same 10 threads and the same 3×3 plan, it is about 7% faster
  for about 5% more CPU.
- **Against the shipped default (mibz2@10),** the medians are 0.142 s against 0.154 s of wall
  and 1.030 s against 0.973 s of CPU. Peak RSS is 75 against 67 MB.
- **The pipe3@10 against team3@10 figure reproduces.** It was 1.042 in the first run and is
  1.033 here.

## What it decides, and what it doesn't

- **This is evidence for each lever's registered question, on this machine.** It is not
  adoption. Adopting a pipelined default still needs:
  - CI's paired bench (RULES C3);
  - the Guardian's review, which is queued;
  - Louis's call on the CPU and memory trade-off.
- **The candidate is pipe3 with both levers at 10 threads:** about 7% faster than today's
  default plan, for about 5% more CPU and 8 MB more peak RSS.
- **Next:**
  - the service-core experiment (research/service-core) tests whether taking the serial work
    out of the teams beats all of this at an equal core budget;
  - a job cap for lever B is a candidate refinement;
  - α remains unidentified, and no outcome here establishes it.
