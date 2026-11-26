# The full ideal-parts matrix: six parts, real or ideal, at 1, 6, 8 and 10 workers

**The run:** `ideal-matrix/run-20261125T125334Z`, recipe `bench-ideal-matrix` (PR 889 at e9d5e778). It is evidence 7dd7b070, SHA-256 f0acf199…70fe5, checked.
- **Size:** 2,492 decodes in 4 h 15 min.
- **Valid in every check:**
  - every phase passed;
  - the pin held in 352 of 352 traces;
  - the removed parts' busy time was 0.00–0.19%, against the 5% VOID line;
  - every arm was byte-identical;
  - the clock ratio was 1.000 in every arm except the all-ideal at 1 worker (0.958).
- **Files:** the report (`report.txt`) and `ideal-matrix.tsv` are here. The analysis program is `../analysis/ideal_matrix.py` (PR 889). The rule is `../../registration.md`, clarifications 1–8.

**The parts:** S the scan, E emission, C construction, M MTF/RLE2, R RLE1/CRC/output, D delivery.

## 1. The room: each part alone ideal, over all real

Each cell gives the measured ratio and, in brackets, its value net of the oracle's handover, a figure reported only (clarification 6).

| workers | S | E | C | M | R | D |
|---|---|---|---|---|---|---|
| 1 | 1.010 (0.998) | **0.835** ROOM (0.617) | **0.579** ROOM (0.556) | **0.925** ROOM (0.918) | 0.983 (0.943) | 1.011 |
| 6 | 0.974 (0.891) | **0.851** ROOM (0.634) | **0.754** ROOM (0.729) | **0.916** ROOM (0.908) | 0.996 (0.952) | 0.989 NO ROOM |
| 8 | 0.968 (0.841) | 0.965 (0.752) | 0.944 (0.918) | 0.976 (0.966) | 1.009 (0.956) | 0.992 |
| 10 | **0.919** ROOM (0.749) | 0.978 (0.784) | 0.947 (0.924) | 0.979 (0.970) | 1.021 (0.962) | 0.994 NO ROOM |

Unmarked cells are UNRESOLVED. Bold cells are ROOM: the interval's upper bound is ≤ 0.95. The full intervals are in `report.txt`.

## 2. The masking: each part with one other ideal (the strongest pairs)

| workers | C with S ideal | S with C ideal | E with C ideal (1) or S ideal | D with S ideal |
|---|---|---|---|---|
| 1 | 0.569 | 0.992 | E with C: 0.704 | 0.987 |
| 6 | **0.605** (alone 0.754) | **0.781** (alone 0.974) | E with S: 0.845 | 0.994 |
| 8 | **0.661** (alone 0.944) | **0.679** (alone 0.968) | E with S: 0.866 | 0.979 |
| 10 | **0.825** (alone 0.947) | **0.801** (alone 0.919) | E with S: 0.909 | 0.963 |

**The interaction that dominates the matrix is S × C,** the scanner with construction. On log wall:
- q_SC = −0.110 at 6 workers (16.9% of the variation) and −0.071 at 8 (13.1%), against −0.002 at 1 and −0.019 at 10;
- 4q, the difference of differences, is −0.44 at 6 and −0.28 at 8.

From 6 to 8 workers, the scanner and construction co-limit. Making either alone ideal gains 3–6% at 8. Making both ideal gains far more. Next come:
- S × D at 8–10 (4q −0.13 and −0.14);
- C × D (−0.11 and −0.08);
- E × C at 1 worker (−0.19).

**The labels.** Nothing is labelled MASKED under the rule: every part with a masked-looking pattern has more than ±2% room alone, so it is UNRESOLVED rather than NO ROOM. In substance, C at 8 and 10, and S at 6 and 8, are masked behind each other.

## 3. The allocation of variation (log wall, the 32 fraction arms)

| workers | S | E | C | M | R | D | 2fi | 3fi (aliased) | residual |
|---|---|---|---|---|---|---|---|---|---|
| 1 | 0% | 14% | **80%** | 4% | 0% | 0% | 2% | 0% | 0.2% |
| 6 | 22% | 8% | **46%** | 2% | 0% | 0% | 20% | 2% | 0.4% |
| 8 | **46%** | 1% | 19% | 1% | 1% | 4% | 22% | 5% | 2% |
| 10 | **53%** | 0% | 9% | 1% | 3% | 10% | 18% | 6% | 1% |

**Construction dominates below the knee, and the scanner above it.** Two-factor interactions carry about a fifth of the variation at 6–10 workers.

## 4. The floor (every part ideal): what it is and isn't

| workers | all ideal over all real | all-ideal wall | all-real wall |
|---|---|---|---|
| 1 | 0.299 | 0.391 s | 1.306 s |
| 6 | 0.387 | 0.094 s | 0.243 s |
| 8 | 0.578 | 0.112 s | 0.195 s |
| 10 | 0.721 | 0.142 s | 0.196 s |

**The floor is mostly the oracles' own cost, not the decoder's residual.**
- **At 1 worker,** the handovers' times sum to 0.392 s (S 15, E 286, C 30, M 9 and R 52 ms), equal to the all-ideal wall of 0.391 s.
- **From 8 to 10 workers,** the all-ideal wall *rises* (0.112 → 0.142 s), while the per-thread handovers barely change. That points to many threads copying large replayed buffers at once and contending for memory bandwidth.

So the matrix cannot say what the decoder's irreducible residual is. That needs oracles whose handover costs nothing: a zero-copy handover, as Antigravity assumed for R. This is the main limitation of this run, and the case for a follow-up.

**The handovers also hide part of some rooms:**
- **E's oracle copies about 40 ms per thread.** Its measured room above the knee is small (0.965 at 8), but net of the handover it is about 0.75.
- **R's handover costs more than RLE1 and CRC save** (1.009–1.021 at 8–10). Net, R's own work is about 4%.

The net figures assume each handover lies on the critical path. They are a bound, not a measurement.

## 5. What it says, plainly

1. **Below the knee (1 to 6 workers), construction is the largest room.** Building Open's structures, ideal, is 0.58 of the wall at 1 worker and 0.75 at 6. Emission is next (0.83–0.85; net of handover, about 0.62), then MTF/RLE2 (0.92).
2. **At the knee and above (8 to 10), the scanner is the largest room** (0.92 at 10; net of handover, 0.75–0.84). But at 6–8 it co-limits with construction: either part's room appears only when the other is cheap too.
3. **RLE1/CRC/output and delivery have little room of their own.** Delivery's matters only once the scanner is free (S × D).
4. **For the mathematics,** the targets ranked by measured room are:
   1. construction (C, target T2);
   2. the scan (S, T4);
   3. emission (E, T1).

   S and C have to be attacked together to win at the operating points that matter (8 workers and up).
5. **The decoder's true residual is unmeasured** (§4).

## 6. Antigravity's predictions, scored (`scoring-neon-1.tsv`)

**Values:** 14 of 28 hit.
- **Strong:** C at 1 and 6, E at 1, 6 and 8, S at every count, M at 6 and 8, D at 1, 6 and 8.
- **Missed:**
  - C at 8 and 10, predicted almost masked (0.97–0.98), measured with more room (0.94–0.95);
  - R at every count: the prediction assumed a zero-copy handover, and the prediction matches R's net figure at 1 and 6;
  - **the floor at every count:** predicted 0.25–0.33, measured 0.30–0.72. That miss is mostly the oracle-handover artifact in §4.

**Labels:** of the predicted labels, 2 hit (D NO ROOM at 6 and 10). Every MASKED label missed, for the reason in §2.

**Interactions:** S × C was named the largest interaction, and it is.
