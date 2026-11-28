# 2026-11-17 · MONODROMY 48 · Scoring records 45 and 46 against the -3 run. The contention term transfers: from the cycle and miss slopes, k = 1.9–2.1, near p = 2. But W\* = 7 at both levels for different reasons: at -9 the cache and CPU limits, at -3 the serial scanner. Record 46's form has no serial term and gets -3 wrong. Homogeneity's sharper test frozen for the low sweep

Asked by the Coordinator (10:24), after `workers-level3/run-20261117T101655Z`:
- *the bundle:* on `evidence` at 19ae1b96; its SHA-256 39ee3f8a…1c5c, rechecked here;
- *the scoring LOG:* 6945320222.

Two questions:
- *Does W_CPU explain W\* = 7 at both levels?*
- *Does the contention term scale as b ∝ (F/L)^k?*

**Labels:** **C** counted · **M** modelled · **Conj**.

## 0. Answers

1. **Record 45 §3:** I agree with LOG 6945320222's scores.
   - *Homogeneity's "W\* = 9 or 10":* **refuted.**
   - *"7 against 10 no better than 0.98":* borderline, at 0.978 [0.973, 0.983].
   - *The bandwidth rival:* refuted.
   - *What failed:* the inference, not the contention term. I assumed nothing but contention binds between 7 and 10 workers.
2. **Record 46 at -3, applied as frozen (09:12, before this run): refuted.**
   - *What its form gives:* W_L3 ≈ 22 and W_CPU ≈ 8.8, so W\* = 9, with 7 against 10 about 1.18 (slower).
   - *Measured:* 7 best, at 0.978. Two workers off, and the gain has the wrong sign.
   - *So W_CPU does not explain W\* = 7 at both levels.* At -3, 2.6 of the 10 CPUs are idle at 10 workers, so no CPU budget binds.
3. **What binds at -3: the serial stage (C).** The occupancy traces, both at 10 workers, unpinned:

   | | -9 | -3 |
   |---|--:|--:|
   | workers decoding | 89% | 68% |
   | idle while the scanner reads a block | 5% | **28%** |

   - *The plateau:* throughput sits at 2.47–2.53 GB/s for W = 6–10, and only W = 4 falls below it (1.343).
   - *So W\* = 7 at both levels is a coincidence of two limits:* at -9 the L3 (7.9) and the CPU budget (8.2); at -3 the scanner.
4. **k (C).** Over the same W range at both sizes, with F(-3)/F(-9) = 0.355:

   | measure | W range | ratio, -3 against -9 | k |
   |---|---|--:|--:|
   | cycles a byte, slope | 6–9 | 0.138 | **1.91** |
   | cycles a byte, slope | 7–10 | 0.164 | 1.74 (the -9 W = 10 point sits under its curve) |
   | LLC misses a byte, slope | 6–9 | 0.114 | **2.10** |
   | LLC misses a byte, slope | 7–10 | 0.135 | 1.94 |

   **k ≈ 1.9–2.1, against p ≈ 2.0 at -9 (misses' log-slope).** The contention term transfers.
   - *The walls are no measure of k:* 0.881 and 0.978 give 1.63, but at -3 the scanner caps the wall, and the cap, not contention, sets that gain.
5. **But not as a pure power law (C).** The W-shape at -3 is flatter: LLC misses rise as W^1.1 over 6–9, against W^2.0 at -9.
   - *That rules out:* c = a + γ(WF/L)^p with one p.
   - *What it leaves:* homogeneity in x = WF/L with a convex g(x), whose local exponent rises from about 1 at x ≈ 0.25 to about 2 at x ≈ 0.8, as an LRU miss curve does.
   - *The test,* §3: frozen now for the low sweep, which hasn't run.

## 1. The corrected form (M)

**The rate.** Throughput is the smaller of the workers' rate and the serial stage's:

  R(W) = min( R_w(W), R_ser(W) )

- **R_w** is record 46's:
  - worker CPUs: min(0.93 W, N − ν − ι);
  - divided by c(W) = a + b · W · W_run, with b ∝ F^k and k ≈ 2.
- **R_ser** is the scanner and reader's rate:
  - about 2.5 GB/s of output at -3 on the i9;
  - above about 2.1 GB/s at -9, where it does not bind (idle on it 5%).

**R_ser is itself contended.** The scanner's loads share the L3 and the ring, so fewer workers speed
it a little. That is the 2% from 10 to 7 on the -3 plateau, where worker CPU no longer matters.

**The best W.**

  **W\* ≈ min(C, W_L3, W_CPU, W_ser)**, with W_ser = the least W whose R_w reaches R_ser

- *W_ser at -3:* R_ser · c(W) / (clock · 0.93) ≈ 2.53 GB/s × 13.8 / 4.91 GHz / 0.93, about 7.6, or
  7.1 at a busy share of 1. The W = 6 arm, 1.3% under the plateau, places it at 6.5–7.5.
- *Above W_ser,* each extra worker only adds contention, to its own cycles and to the scanner's.

**The input the plan lacks: the serial stage's cost a byte as a function of block size s.**
- *Why it varies:* there are 3× the blocks a byte at -3, so whatever the scanner and reader pay a
  block weighs 3× more.
- *Its form:* a kernel constant, the serial CPU per block and per compressed byte, measured once
  (the scanner's task-clock per block in the occupancy trace).
- *What the plan needs:* W_ser from s, beside W_L3 from s.

## 2. Caveats (C)

1. **The -3 occupancy run exited non-zero:** "ledger check: VIOLATED", 63 per-worker violations. Its
   timeline is reported, but it comes from a flagged run. The scanner-bound reading also rests
   on the ab arms' plateau and on 2.6 idle CPUs (aggregate task-clock at W = 10: 7.38 of 10).
2. **The occupancy traces are unpinned** (20 CPUs allowed); the ab arms are pinned to 10.
3. **F's ratio is total worker buffers at the end** (5.97 against 16.81 MB), not a measured hot
   set. If the hot set shrank by a different factor, k moves by ln(that ratio ÷ 0.355) ÷ ln 0.355.

## 3. Homogeneity's sharper test, frozen before the low sweep (M)

**What homogeneity in x says.** If contention depends only on x = W·F/L, then -9 at W workers sits at
the same x as -3 at W / 0.355 workers. LLC misses a byte should then match, after scaling by the L3
accesses a byte (L2 misses 0.689 at -9 against 0.586 at -3, a factor of 1.176).

**Predicted -9 LLC load misses a byte** (`LLC-load-misses` ÷ output bytes, as `counters.txt` reports
them), read off -3's curve:

| -9 at W | same x as -3 at W | predicted | interval |
|--:|--:|--:|---|
| 2 | 5.6 | **0.0022** | [0.0017, 0.0028] |
| 3 | 8.5 | **0.0034** | [0.0027, 0.0042] |
| 4 | 11.3 | 0.0045 (extrapolated past -3's 10) | [0.0035, 0.0058] |

- *The local exponent* at -9 between W = 2 and 3 should be 1.0–1.4, not 2.0.
- *Refuted if:* -9 at W = 2–3 lies more than 25% off these, or rises as W^2 there. Then contention
  depends on W and F separately, not only on their product, and k ≈ p at 6–9 was a coincidence of
  ranges.

**The serial cap in the low sweep.** At 4–8 threads, -9's throughput stays under 2.1 GB/s, so the
scanner should not bind:
- *idle on it:* under 8% at every arm with a trace;
- *the walls:* records 44 and 46's predictions stand unchanged.

## 4. For the plan (#522)

**Where trimming matters.**
- *At -3:* it gains 2% at most (the scanner caps), but costs nothing on Wikidata. Text at -3 was
  0.959–0.961 at 6 and 4 workers.
- *At -9:* 7 at 10 threads stays right on both counts.

**The rule needs block size twice:**
1. *W_L3:* through F(s);
2. *W_ser:* through the serial cost a block.

No machine fact beyond C, N, L and s is needed.
