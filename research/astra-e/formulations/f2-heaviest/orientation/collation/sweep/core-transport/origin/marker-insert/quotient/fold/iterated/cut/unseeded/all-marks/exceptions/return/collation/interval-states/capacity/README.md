# How many contiguous prediction states are intrinsically necessary?

For h backward steps, one deterministic state predicts a single length-h word. The rows for which all h transitions are correct must match that word. Maximizing over arbitrary constant labels on C contiguous intervals and dropping transition consistency can only increase the number of correct starts. Write Dₕ(λ) for the maximum matched rows less λ per interior cut, with unrestricted segment count. Then every C-state model satisfies

    n − hE ≤ Aₕ ≤ Dₕ(λ) + λ(C−1).

The first inequality counts disruptions by a union bound. Each exceptional row x can disrupt starts at x, Ψx, …, Ψ^(h−1)x, with repetitions allowed on short cycles. Aₕ counts *overlapping* starts on the entire physical population. The inequality holds for a permutation with any number of cycles. Applying the constant-segment recurrence with every seam allowed computes Dₕ exactly. In a run of identical true h-words, changing a segment label twice is unnecessary. At an optimum, boundaries can be moved to run ends without losing reward or adding cuts. Updating only the run label from the shared restart score computes the exact optimum in one pass over those runs per penalty. The structural oracle uses dense LF and rowwise equality of integer pairs to obtain true words by doubling. It supplies no faster decoder by itself. [Source](capacity.cpp) · [Complete penalized optima](dual.csv)

Summing the inequalities across ten blocks with a shared charge even allows redistribution of a total state budget across all ten. With C=122,189 across the panel, every deterministic physical interval model must make at least **501,759 exceptions**, whatever seams or targets it chooses. The charge-one adaptive solution actually makes 1,522,329. Thus the restricted construction loses substantial accuracy beyond the lower bound, yet matching the measured k=8 total of 409,432 exceptions requires **at least 181,947 states** across these blocks, even without paying discovery or target consistency. At the 44,859-state budget the floor rises to 769,047. These certificates concern contiguous states, deterministic transitions and this input. They allow substantially better models than the measured adaptive solution. A noncontiguous logical state may have many physical components; the bound counts components where its prediction changes.

| State policy | Total states | Actual exceptions | Floor with observed per-block budgets | Floor allowing budget redistribution |
|---|---:|---:|---:|---:|
| Adaptive charge 1 | 122,189 | 1,522,329 | 501,759 | 501,759 |
| Adaptive charge 4 | 44,859 | 1,731,602 | 769,047 | 769,047 |
| Adaptive charge 16 | 17,558 | 2,033,503 | 1,115,880 | 1,109,192 |
| Adaptive charge 64 | 6,937 | 2,430,121 | 1,587,777 | 1,586,737 |
| Fixed prefix k=8 | 815,844 | 409,432 | 54,079 | 53,744 |

At the k=8 budget a one-byte certificate is vacuous. The 16-step relaxed words give the strongest common-charge floor there. At the smaller adaptive budgets the single-byte certificate already dominates. The frozen finite gate checks 2,520 exact penalized optima, 54,600 partition-level inequalities and 2,568 independently iterated LF powers, including short repeat cycles. All 720 real model/horizon combinations passed. These are mathematical bounds on mistakes and overlapping starts, with no wall-time ranking. [Per-model bounds](bounds.csv) · [Shared charges and totals](summary.csv) · [Exact finite counts](small.txt)

```sh
d=$(dirname "$(rg --files | rg '/interval-states/capacity/PLAN.md$')")
g++ -std=c++17 -O3 -Wall -Wextra -pedantic "$d/capacity.cpp" -o /tmp/astra-predictor-bound
/tmp/astra-predictor-bound | diff - "$d/small.txt"
python3 -B "$d/panel.py" /tmp/astra-predictor-bound /tmp/astra-clock-wires "$d/../../../../contexts/blocks.csv" /tmp/astra-predictor-bounds
diff /tmp/astra-predictor-bounds/dual.csv "$d/dual.csv"
diff /tmp/astra-predictor-bounds/bounds.csv "$d/bounds.csv"
python3 -B "$d/report.py" "$d" | diff - "$d/summary.csv"
```
