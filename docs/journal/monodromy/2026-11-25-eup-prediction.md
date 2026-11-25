# Record 131. Pre-registered prediction: e_up at k 8, λ 10% (stage 2b)

**Written 03:02Z, 25 November, before stage 2b's bundle exists. I have read stage 2a's journal figures (§74) only; no stage 2b data.** Answers QUEUED 7151347431. Status: **conjectured** (a model's forecast), from **measured** inputs.

## 1. The estimand

From the registration (stage 2b, frozen): L = 1 + λ = 1.1; e_up = log(T_all / T_others) / log L; e_down = log(T_scan / T0) / log L; H = log(T_all / T0) / log L. Write e_oth = log(T_others / T0) / log L. Then **e_up = H − e_oth, identically** (proved, by definition). So the prediction is two numbers: how homogeneous the wall is, and how hard the non-scanner side binds at 8.

## 2. The model, fitted on stage 2a's surface only

Wall T = (S^p + W^p)^(1/p), S the scanner's side, W everything else, a = S^p / (S^p + W^p) at the base.

Stage 2a's four surface elasticities at k 8 (scanner 0.87 at +10%, 0.93 at +20%; workers 0.50, 0.64) are four equations in (p, a). They fit **p ≈ 21, a ≈ 0.74**, reproducing all four within 0.03 (argued: 2 parameters, 4 data; not a held-out test). p = 21 means the knee is about 3% wide in S/W: sharp, but wider than a 10% dose is small against.

## 3. What changes in 2b

1. **The scanner is pinned to the fastest core with its sibling idle.** Stage 1 gave the scanner 1.06 inflation at 8 workers; pinning recovers part of it. I take the scanner 3% faster relative to the rest (range 0 to 6%). Because p is large, a is very sensitive: a falls from 0.74 to **0.60** (range 0.74 to 0.45).
2. **"Others" includes the reader and A9, A10,** which 2a's worker surface did not dose: +0.02 on e_oth.
3. **H below 1.** 2a's (20, 20) cell gave a slowdown of 0.154, that is H ≈ 0.79, and less than the scanner alone (0.170). Pacing adds cycles but no memory traffic, so paced workers contend less and the scanner speeds up: the wall is not exactly degree 1. With every activity paced I expect less of this than in 2a's cell: **H = 0.92** (range 0.80 to 1.02).

## 4. The prediction

| quantity at k 8, λ 10% | point | interval I commit to |
|---|---|---|
| e_oth | 0.65 | 0.50 to 0.78 |
| H | 0.92 | 0.80 to 1.02 |
| **e_up** | **0.28** | **0.12 to 0.48** |
| e_down | 0.79 | 0.66 to 0.90 |
| d = e_down − e_up | +0.50 | above +0.25: ASYMMETRIC |
| e_up at λ 20% | lower than at 10% by 0.08 to 0.20 | (e_oth rises to about 0.77) |

In words: **under a third of a 10% scanner speed-up reaches the wall at 8 workers**, about 3% of wall, because the workers are 3% behind and take over. At k 10 I expect e_up ≥ 0.6.

**The model-only check, free of my guesses for a and H:** from 2b's own e_down and e_oth/H at 10%, the implied exponent p (solving L^(p·e_down) + L^(p·e_oth/H) = 1 + L^p) lies in 12 to 35.

## 5. What kills it

- **e_up's point estimate outside 0.12 to 0.48:** the forecast fails. Above 0.6: the pinned scanner still binds alone, and the soft maximum fitted on 2a doesn't transfer across placement. Below 0.1: pinning moved the knee above 8.
- **Implied p outside 12 to 35:** the soft-maximum form itself fails, whatever a is.
- **H's interval outside [0.85, 1.15]** (NOT-HOMOGENEOUS): e_up is uninterpretable by the registration; my H range straddles that edge, so I count this outcome as predicted at about 1 in 3, and score only e_oth and e_down then.
- **d < 0.25:** no asymmetry, so 8 is not at the kink under pinning.

## 6. If it holds, what it prices (argued)

Multi-symbol decoding at −15% on A3 (81% of the scanner) is −12% on S. At 8 workers that buys about 3 to 4% of wall, not 12; the balance count moves from 8 to about 9, where memory and CPU cost rise (§76). The scanner lever and the worker levers (T2 construction, 0.45 of worker time) are complements at the knee: either alone returns its e, both together return their sum, near H.
