# The unrolled room walk's full-decoder trial: second exploratory CI run 41243689201 (w = 1, 2)

**The run (measured):**
- Head `f5c492d` (pin `df90ba8`: amendments 1 and 2), started by
  `research/walk-unroll-decoder-ci-trigger` (`39fb07b`).
- The second exploratory validation. It replaces **none** of run 41235849514's records:
  that run's aborted AMD jobs, r1's partial rows, and r3's completed Xeon 6975P-C result
  (the `records` regression included) stand as recorded.

**All three jobs completed:**
- The binary's SHA-256 is `cb7711c9…`, identical in all three. It is kept here as
  `walk_decoder.gz`.
- Pre-timing checks, W1–W4 (b), and the recorded W4 (c) acceptance all passed.
- Correctness, plan and confinement true. A/A passes: 3, 0 and 0 of 44 rows exclude 1.
- Steal 0, and the idle siblings at 0.3–0.4% busy.

**The lifecycle fix in practice:** end snapshots held 7 (r1), 1 (r2) and 0 (r3) exited
records, every one classified `EXITED` and none `UNRESOLVED`. Under the earlier harness,
each would have aborted the job.

## B/A, the geometric mean of the paired ratios with its 95% interval

Throughput (X) is B/A; CPU is CPU-s/GB B/A.

| Corpus, w | r1 EPYC 7763 (Zen 3) X / CPU | r2 EPYC 9V45 (Zen 5) X / CPU | r3 EPYC 9V74 (Zen 4) X / CPU |
|---|---|---|---|
| **wikidata-20261026 (primary), w1** | **1.081 [1.072, 1.090]** / 0.932 | **1.179 [1.151, 1.207]** / 0.868 | **1.017 [1.001, 1.035]** / 0.983 |
| **wikidata-20261026 (primary), w2** | **1.059 [1.028, 1.092]** / 0.934 | **1.184 [1.141, 1.229]** / 0.829 | **1.020 [0.999, 1.043]** / 0.974 |
| wikidata-fitted, w1 / w2 | 1.079 / 1.041 | 1.207 / 1.180 | 1.035 / 1.037 |
| wikidata-heldout, w1 / w2 | 1.094 / 1.066 | 1.238 / 1.191 | 1.013 / 1.015 |
| **records, w1** | **0.948 [0.942, 0.953]** / 1.041 | 1.003 [0.983, 1.023] / 1.000 | **0.917 [0.912, 0.923]** / 1.067 |
| **records, w2** | **0.966 [0.950, 0.982]** / 1.037 | 1.007 [0.982, 1.033] / 0.997 | **0.951 [0.930, 0.972]** / 1.063 |
| text, w1 / w2 | 1.022 / 1.010 | 1.111 / 1.060 | 1.024 / 1.001 |
| text-l1, w1 / w2 | 1.003 / 1.011 | 1.072 / 1.036 | 1.006 / 1.006 |
| text-l5, w1 / w2 | 1.015 / 1.007 | 1.065 / 1.056 | 1.003 / 1.014 |
| period-3, w1 | 1.050 | 1.161 | 1.032 |
| random, mixed-independent, zeros | 0.972 to 1.014 | 0.994 to 1.056 | 0.985 to 1.005 |

**Peak RSS:** the max rule is 0.97 to 1.04 on every corpus. The rows not passing are
small-file rows, with wide intervals (mixed-independent w2, period-3 w2).

**The registered rules at w = 1, 2** (descriptive; the overall verdict is "NOT
APPLICABLE": there is no w = 4):

| Job | Controls | RSS |
|---|---|---|
| r1 | NEGATIVE (`records` w1 and w2) | INCONCLUSIVE |
| r2 | INCONCLUSIVE (wide small-file rows) | INCONCLUSIVE |
| r3 | NEGATIVE (`records` w1 and w2, and `random` w1 and w2 on wide intervals) | INCONCLUSIVE |

## Reading (deduced; exploratory, one job a host)

1. **The decoder-level effect depends strongly on the core.** On the primary at w1:

   | Host | Throughput | CPU-s/GB |
   |---|--:|--:|
   | Zen 5 (9V45) | +18% | −13% |
   | Zen 3 (7763) | +8% | −7% |
   | Zen 4 (9V74) | +2% | −2% |
   | Intel Xeon 6975P-C (run 41235849514 r3) | +0.2% | −0.1% |

   Zen 3 and Zen 4 match the stage trial's order of magnitude. Zen 5 had no stage
   measurement.
2. **`records` regresses on three of the four hosts:** −5% (Zen 3), −8% (Zen 4) and −7%
   (Intel) at w1, with 4–7% more CPU and tight A/A intervals.
   - It is neutral on Zen 5.
   - It is a real cost of the integrated arm on that input class. Its cause isn't
     established (candidates are listed in run 41235849514's `results.md`).
3. **The other controls** are within noise or gain, apart from the wide small-file rows.
4. **None of this decides the trial.**
   - The deciding host is the i9 (Intel, Comet Lake, 4 physical cores), and the only
     Intel decoder evidence so far is neutral on Wikidata with a `records` regression.
   - An architecture-specific dispatch would be a separate, future hypothesis.
