# The service core's admission threshold: the registered run's result

9 November 2026. This is the run registered in `2026-11-09-service-lookahead-registration.md`
(#271, branch `research/service-lookahead`), frozen at `4ac25fcd`, with the ready tag at `8a0fc8e0`.

## The run

- **Bundle:** `service-lookahead/20261109T075939Z.tar.gz` on `evidence` at `705319fd`.
  - Its SHA-256 is `4df66705a99a1ff7dda4fcea76de655f306a81ddae8261dca077f9f39b84a6c0`, which
    matches the notice on #145 (6735221732) and was checked again for this entry.
  - It ran with `just submit`, from Louis's `just workstation-queue`. `commit.txt` is
    `8a0fc8e0`, and `frozen.txt` gives `frozen 4ac25fcd`.
  - The machine was the i9-10900K (1 socket, 10 cores, a 20 MiB L3), so `mibz2` is the
    pipelined default.
- **VALID:** every check passed.
  - 20 rounds in every cell, each cell's plan as registered (thread counts read from names), and
    no other cell.
  - Every run gave the same bytes as `mibz2@10` (`a8830caee5c511a6`), the registered 209,700,196
    bytes, from the registered corpus (`82f65ace…`).
  - All five traces were complete and as registered: 6 rounds and 431 blocks each, with P inside
    its ceiling in every complete round, and none saturated.
  - The noise control N1 (`team3-twin@10` against `team3@10`) came out at 0.994 [0.975, 1.014].

Speed is the reference's wall over the arm's; CPU is the reference's CPU over the arm's. Above 1,
the arm is faster or spends less.

## The registered comparisons (judged)

| | arm | against | speed [95% CI] | outcome | CPU [95% CI] | outcome |
|---|---|---|---|---|---|---|
| LA3 | svc2-a3@10 (a3, w8) | svc2@10 (a2, w8) | 0.992 [0.968, 1.016] | INCONCLUSIVE | 1.006 [0.987, 1.025] | NULL |
| LA4 | svc2-a4@10 (a4, w8) | svc2@10 | 0.979 [0.951, 1.008] | INCONCLUSIVE | 0.983 [0.958, 1.009] | INCONCLUSIVE |

**The reading, by the registered order: INCONCLUSIVE (step 3).** Neither threshold is a gain,
and both intervals span gain and harm.
- **A higher admission threshold at window 8 is not shown to help.** LA4's point estimate leans
  slower (0.979). Its interval reaches 0.951, so a harm of up to about 5% is not excluded either.
- **NULL is bounded uncertainty,** not an effect shown to be zero.

## Against the model (registered before the run)

The model expected LA3 and LA4 to come out a SMALL GAIN or NULL, P's median to rise with L, and
prep starvation to fall but not below 0.10. It also expected W12 alone to be NULL.

| | expected | seen |
|---|---|---|
| LA3, LA4 | SMALL GAIN or NULL | INCONCLUSIVE, both leaning to no gain |
| P, median of round medians | rising with L | 1.0 (a2) → 1.5 (a3) → 1.5 (a4), at w8 |
| teams idle waiting for prep | falling, not below 0.10 | 0.158 (a2) → 0.133 (a3) → 0.138 (a4), at w8 |
| W12 alone | NULL | 1.000 [0.972, 1.029], NULL |

- **The model held on direction.** P rose and starvation fell a little, but too little to show
  in the wall time.
- **Every w8 trace still shows PREP STARVATION.** Only `a4 w12` relieved it (teams' prep-idle
  share 0.057, P's median of medians 3.0). It then showed a **FINISH BACKLOG** instead (backlog
  prep / finish 2.42 / 0.35).
- **So the binding constraint moves:** at w8, raising the threshold barely raises how much is
  prepared. With both threshold and window raised, preparation catches up and finishing becomes
  the wait.
  - This is read from six traced rounds per arm, as a description. No trace is a judged
    comparison.

## Reported, never judged

| | arm | against | speed [95% CI] | outcome | CPU [95% CI] |
|---|---|---|---|---|---|
| W12 | svc2-w12@10 | svc2@10 | 1.000 [0.972, 1.029] | NULL | 0.978 [0.957, 0.999] |
| J | svc2-a4-w12@10 | svc2@10 | 1.027 [0.993, 1.063] | INCONCLUSIVE | 0.976 [0.952, 1.000] |
| J4 | svc2-a4-w12@10 | svc2-a4@10 | 1.049 [1.017, 1.082] | SMALL GAIN | 0.992 [0.971, 1.014] |
| D-a2w8 | svc2@10 | mibz2@10 (default) | 1.073 [1.041, 1.106] | GAIN | 1.169 [1.135, 1.204] |
| D-a3w8 | svc2-a3@10 | mibz2@10 | 1.064 [1.042, 1.086] | GAIN | 1.175 [1.145, 1.207] |
| D-a4w8 | svc2-a4@10 | mibz2@10 | 1.050 [1.022, 1.080] | GAIN | 1.149 [1.117, 1.182] |
| D-a2w12 | svc2-w12@10 | mibz2@10 | 1.073 [1.052, 1.094] | GAIN | 1.143 [1.122, 1.164] |
| D-a4w12 | svc2-a4-w12@10 | mibz2@10 | 1.102 [1.070, 1.135] | GAIN | 1.140 [1.112, 1.169] |
| R1 | mibz2@10 | team3@10 | 1.047 [1.024, 1.071] | GAIN | 0.935 [0.912, 0.959] |

- **J4 is a SMALL GAIN** for widening the window once the threshold is 4. It is consistent with
  the trace's shift from prep starvation to finish backlog. It is chosen after the run and
  unjudged, so it is a lead for a registration, not a result.
- **D-a4w12, the best of five against the default,** was chosen after the run, so its interval
  is optimistic.
- **The service core against the default, again.** D-a2w8 is the same comparison as #258's
  judged P1 (svc2@10 against mibz2@10), on the same machine and the same corpus, but in another
  run, at another commit, beside other cells:
  - #258 P1: speed 1.045 [1.017, 1.073], CPU 1.146 [1.123, 1.170];
  - here: speed 1.073 [1.041, 1.106], CPU 1.169 [1.135, 1.204].

  Here it is reported and unjudged, so it does not replace P1. But **the direction has now
  replicated on two runs: the service core at 2 × 3 + 2 is faster than the default at 10 threads
  on the i9, and spends about 13–15% less CPU.**
- **R1:** the pipelined default (3 × 3, window 12) is faster than three plain teams by 1.047,
  at a CPU cost of 0.935.

## What this changes

- **The admission threshold alone is not the lever** at window 8. No change to svc2's admission
  policy follows from this run.
- **The finding that stands:** the service core beats the default on the i9, across two runs.
  The next value is in qualifying it, not tuning its admission.
- **Open, as hypotheses for the follow-up:**
  - where the prep starvation that survives every w8 arm comes from (preparation's own cost,
    the service workers' 70% busy time, or the hand-off);
  - whether the joint threshold-and-window change (J4) holds as a registered comparison;
  - whether finishing is the next bound once preparation keeps up.

## Follow-up (PROTOCOL §11)

- **The Optimiser:** sharpen the reading. Price a registered J4-style comparison and the
  service core's path to the default (what it still needs, on which machines).
- **The Researcher:** adjacent angles on prep starvation: what limits the service workers'
  preparation rate, and whether a third service thread or a split preparation would help.
- **Then build the top construction** they name.
