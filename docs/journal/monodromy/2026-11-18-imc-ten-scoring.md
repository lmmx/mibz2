# 2026-11-18 · MONODROMY 53 · Record 52's frozen IMC test scored: INCONCLUSIVE, on the supported side, but its "four fifths queueing" reading is refuted. The 10 → 7 gain is mostly the contention term record 46 already carries. Queueing adds about 2% of cycles at 10 workers

**Asked by the Coordinator (09:10).** Score record 52 §4 against `imc-ten/run-20261118T090842Z`
(evidence b589a47e; SHA-256 4aec1052…8b81, rechecked here; LOG 6974087817). No new run.

**Labels:** **C** counted · **M** modelled · **Conj**.

## 1. The frozen predictions, scored (C)

Wikidata ×4, lazy 10/40 against 7/28, unpinned on all 20 CPUs:

| record 52 §4 | measured | |
|---|---|---|
| DRAM 15.5 [14.5, 16.5] bytes a byte | 14.11 | ✗, low |
| DRAM 30.7 [29.0, 32.5] GB/s, at least 88% of the ceiling | 28.6 GB/s, 85.7% | ✗, just low |
| L3-miss latency +20% or more against 7 | +18.3% by medians; +18.0% [+7.4, +20.7] paired | ✗, just short |
| the refuters: DRAM under 80%, or latency under +10% | neither | not refuted |

**Verdict by the frozen rule: INCONCLUSIVE,** as the Coordinator read it. The ×1 check (not judged)
is DRAM 79.7%, latency +26.5%, wall 0.895.

**Traffic grows sublinearly past the fitted range.**
- *The prediction:* 6.66 + 0.088·W·W_run gives 15.3 B at the measured W_run (9.78).
- *Measured:* 14.1.
- *Where it was fitted:* x = 49–82 only.

## 2. Does queueing explain most of the 10 → 7 gain? No: part of it (C, M)

**Record 52's "four fifths" reading came from a wall model that was wrong.** It took wall ∝ c(W)/W
and so predicted 7 against 10 at 0.978 from contention alone. The bundle measures every factor
directly:

| factor, 7 against 10 | measured | the contention model alone |
|---|--:|--:|
| total cycles a byte | 0.660 (16.95 against 25.69) | 0.685, from a + b·W·W_run |
| CPUs busy (total CPU ÷ wall) | 7.96 against 10.72, a ratio of 1.346 | 1.429, if every worker were busy |
| wall | 0.876 | |

Splitting the gain over naive scaling in log terms (ln(1.429 / 0.876) = 0.489):

| share | source |
|--:|---|
| **77%** | the contention-associated overhead the W·W_run term already carries |
| **8%** | cycles at 10 workers beyond that term, +3.8% |
| **12%** | fewer busy CPUs at 10 workers: per-worker utilisation 0.910 against 0.941, plus the non-worker share |
| 3% | residual |

**Queueing's part (M).** The +18% L3-miss latency costs 0.0135 misses a byte × 85 cycles ÷ about
2.2 misses in flight. That is 0.52 cycles a byte, **2.0% of the cycles at 10 workers**: about half the 8% term
above, and about one sixth of the gain beyond the contention model.

**So queueing near saturation is a part of the mechanism, not most of it.** I withdraw record 52's
"about one fifth contention, four fifths queueing" (Conj, now refuted). The W·W_run term carries
most of the gain.

**Where the latency is (Conj).** The rise is larger on ×1 (+26.5%) at *lower* utilisation (79.7%).
So it isn't only the memory controller's queue: the L3-miss path through the ring and the L3 slices
also slows with the miss traffic, which more than doubles (2.17×) from 7 to 10. That is record 51's
mechanism, applied to misses as well as hits.

## 3. Does the falling ceiling belong in the model? Yes, as the denominator of utilisation (C, M)

**What the stream test shows.** The memory controller sustains 33.4 GB/s with 2–4 streams, 31.9 with
10 and 31.3 with 20. That is about −0.25 GB/s a stream past 4: more streams interleave and conflict
in the DRAM banks.

**For the model:** use **B_max(W) ≈ 33.4 − 0.25 · (W − 4)** GB/s for W ≥ 4.

| workers | utilisation against B_max(W) | against the best ceiling |
|---|--:|--:|
| 10 | **90%** | 86% |
| 7 | **76%** | 75% |

**What it changes:**
- *It moves no verdict:* the registered rule used the best ceiling, and C2 is not relaxed after the
  data.
- *Where it matters:* only near saturation. It makes the bandwidth bound bind about half a worker
  earlier than the best ceiling suggests.
- *Its scale:* too small to move W\* on the i9. W_L3 ≈ 7.9 still binds first.

## 4. What stands

- **The knee at 7 workers is contention,** carried by c = a + b·W·W_run.
- **Past 8:** DRAM utilisation (76% → 90% against B_max(W)) and L3-miss latency (+18%) rise. They
  add about 2–4% of cycles, and the busy CPUs fall short of W by 6%.
- **Record 52's lever ranking is unchanged:** the fused finish first. Its saved write bytes count a
  little more past 8 workers, not decisively.
