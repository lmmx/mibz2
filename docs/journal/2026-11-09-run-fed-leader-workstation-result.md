# The run-fed leader on the i9: the registered workstation run's result

9 November 2026. This is the workstation run registered in `2026-11-09-run-fed-leader.md` §8
(#270, branch `research/run-fed-leader`), frozen at `cccdfdf7`, with the ready tag at `e7885568`.

## The run

- **Bundle:** `run-fed-leader/20261109T080633Z.tar.gz` on `evidence` at `378f0637`.
  - Its SHA-256 is `59ca0db2384dca9b57d6965bbbf61d788758827dd2195ac74e8cbf6dcfa1cea0`, which
    matches the notice on #145 (6735256377) and was checked again for this entry.
  - It ran with `just submit` at `e7885568`, with `frozen cccdfdf7`.
  - The machine was the i9-10900K.
- **Outcome as recorded:** VALID (JUDGED (workstation)): 20 rounds at 10 threads and 140 timed
  records, against `mibz2-default33`. Both occupancy captures were VALID, with no perturbation
  shown.

## The registered reading

| arm (against `default33`) | wall [95% CI] | CPU [95% CI] |
|---|---|---|
| `runfed-dir` (primary) | 0.9681 [0.8535, 1.0981] | 0.9698 [0.8459, 1.1119] |
| `runfed-ctl` (control) | 1.0072 [0.8442, 1.2017] | 1.0144 [0.8398, 1.2254] |
| `runfed-reg` | 1.0424 [0.8936, 1.2161] | 1.0423 [0.8821, 1.2317] |
| `pipe3-both` | 1.0354 [0.8984, 1.1934] | 1.0381 [0.8933, 1.2065] |
| `runfed-ls` (secondary) | 0.9934 [0.8222, 1.2002] | 0.9735 [0.7969, 1.1892] |
| `runfed-hs` (secondary) | 1.0603 [0.8932, 1.2585] | 1.0461 [0.8719, 1.2551] |

**Primary: INCONCLUSIVE. Control fidelity FAILED.** `ctl`/`default33`'s interval is not inside
[0.98, 1.02]. By §8, INCONCLUSIVE goes back for revision. The run supports no claim about the
run-fed leader on the i9, for or against.

## Why the intervals are about twenty times CI's: the machine's state, not the decoder

The intervals are about ±15%, where CI's run on the same design was about ±0.8% (`03e52206`). The
per-round records show why. This is read from `runs.jsonl` after the run, as a diagnosis, not a
reading of the registered comparisons.
- **The walls are bimodal.** Most runs take 0.14–0.17 s. **41 of the 140** take about twice
  that (0.20–0.38 s; the 90th percentile is 0.315).
- **In those runs every thread's CPU time doubles with the wall.** The decoder's total CPU is
  2.17 s against 1.06 s, and each named thread's share doubles alike (for example `mibz2-0`
  0.21 s against 0.10 s).
  - Doing the same work in twice the CPU time is the signature of a halved effective clock
    (a frequency or power state). It is not a scheduling stall: a thread waiting off-CPU does not
    accrue CPU time.
  - Other processes' load moved little (`others_busy` 0.16 against 0.13).
- **It hits every arm, in clustered rounds** (rounds 0, 1, 4–6, 8, 9, 12–16 and 18), so paired
  ratios inherit it whenever only one member of a pair is slowed.
- **The run just before it on the same machine had none.** `service-lookahead/20261109T075939Z`,
  which ended minutes earlier, had 160 runs with a maximum of 0.172 s and none beyond 1.6 times
  its median. So the state changed between or during the runs.
- **What was not recorded:** the core clock per run (`scaling_cur_freq`), the package
  temperature, the power profile and the governor. The cause can't be named from the bundle.

**A post-hoc sensitivity, never a verdict:** excluding every round in which either member of a
pair ran at double CPU leaves 11–13 pairs per arm. `dir`/`default33` is then 0.929
[0.872, 0.990]. That subset is chosen after the run from the outcome's own variable. It is
reported only to show that the machine state, not the arms, drives the width. It carries no
weight.

## What this changes

- **Nothing about the run-fed leader.** CI's attributed LEAD (`03e52206`) stands as CI's
  reading. The i9 has given no reading yet.
- **The workstation harness should record the core clock** (each run's mean
  `scaling_cur_freq` across the CPUs it used, or the APERF/MPERF ratio) and the package
  temperature. A rule could then make a slowed run visible, or invalid, by registration, rather
  than inflating the interval.
  - That is a change for review. It must not be applied after the fact to this run.
- **The next run of this design** needs its registration revised (§8's INCONCLUSIVE rule), the
  harness change above, and the machine state checked before it starts.

## Follow-up (PROTOCOL §11)

- **The Guardian and the Researcher:** is clock logging, and a registered rule for slowed runs,
  admissible for the revised workstation run, and in what form?
- **The Optimiser:** sharpen what an i9 run of the run-fed leader still has to show, beside
  CI's attributed LEAD.
- **Louis:** whether anything about the workstation's power or thermal setting changed between
  the two runs. Only he can say.
