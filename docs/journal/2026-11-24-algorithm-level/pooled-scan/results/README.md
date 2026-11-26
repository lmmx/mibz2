# The pooled scan's kill test: KILL, narrowly, on K1

**The run:** `pooled-scan/run-20261126T133034Z`, recipe `bench-pooled-scan` (#932 at 1f6a578c). It is evidence c71c4a56, SHA-256 97e397ad…8867, checked.
- **Size:** 480 decodes plus the finder pass, 15 minutes on the i9-10900K.
- **Valid:**
  - every phase passed, the exactness phase included (10 tests);
  - every pooled decode was wholly pooled: 0 dropped, 0 fallbacks, 0 degraded.
- **The rule:** `../registration.md` §§4, 5 and 7, frozen before the data. The report is in this folder.

Arms: P is the pooled scan; C, construction made ideal (the zero-copy oracle); S, the scan made ideal.

## The verdict, by the frozen rule

**KILL**, on K1 at both judged counts:

| workers | PC/C [95%] | √(SC/C), the bar | capture of the scan's room behind C | P/none [95%] (K2: kill if the lower bound > 1.03) |
|---|---|---|---|---|
| 8 | 0.862 [0.848, **0.876**] | 0.867 | 0.52 | 1.052 [1.023, 1.081] |
| 9 | 0.869 [0.833, **0.906**] | 0.864 | 0.48 | 1.000 [0.983, 1.017] |
| 10 (reported) | 0.848 [0.825, 0.873] | 0.877 | 0.63 | **0.936** [0.909, 0.964] |
| 6 (reported) | 1.065 | 0.940 | −0.51 | 1.105 |

- **K1:** the pooled scan's point estimate captures about half the scan's room behind construction (0.52 at 8, 0.48 at 9). But the rule needs the upper bound below the bar, and it misses by 0.009 and 0.042.
- **The other gates pass:**
  - **K2:** at 8, P alone is 5% slower, but its lower bound (1.023) is under 1.03;
  - **K3:** peak RSS within 1.4%;
  - **K4:** the finder takes 4.82 ms (9.8 GB/s, AVX2), half its 10 ms limit.

## What the run shows about why

- **Pooling redistributes the scan; it doesn't remove it.** The workers' summed scanning work in P (A2 + A3, 173–193 ms) equals the serial scanner's in none (171–190 ms).
  - The scan's CPU moves onto the workers. Below about 9 workers they are already the binding station, so P is slower there: 10% at 6 and 7, 5% at 8.
  - Above the knee there is spare worker capacity, and P wins: −6.4% at 10.
- **It works where construction is cheap.** With construction ideal, the pooled scan takes 158 ms at 8 against C's 183 and SC's 135. That is about half of what an ideal scan would give.
- **The plumbing is as predicted.** The chain copies 45 MiB for 5–7 ms (estimated about 5), finds magics in 5–7 ms, and removes the 190.6 MiB symbol queue. The workers' idle share falls from 6.5–20% to 1–2.5%.
- **Against the forecast:** P's wall at 8 and 9 is 207.5 and 195.2 ms, against the design analysis's unscored forecast of 172–175 ms.

## What it means, and what is not concluded

- **The prototype stops here as built.** This is the frozen verdict, and it isn't relaxed after the data (C2).
- **The pooled scan isn't dead in principle.** The lesson is quantitative: it's a net win only where workers have slack, which means construction made cheaper, or 10+ workers. That couples it to threads 4–5, as the matrix's S × C result said it would.
- **Two follow-ups the data suggests** (not adopted, not registered):
  1. **A hybrid:** the serial scanner stays, and idle workers scan ahead only when they would otherwise wait. The scan's CPU is then spent only from slack.
  2. **Re-test after construction or emission work lands,** since the same design captured more as C got cheaper. That would be a registered re-run, not a re-reading of this one.
