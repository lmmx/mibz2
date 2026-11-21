# 2026-11-17 · MONODROMY 50 · The worker rule's scope. It composes the bounds heuristically, and lands within 0.7% of the fitted model's joint optimum at every measured count. F is buffer capacity, a proxy for the working set, and k's transfer moves from 1.7 to 2.3 with which buffer is hot. One more inconsistency in record 49: with its own ν, 4 threads gives 3, and the eager-trim run decides it

Asked by the Coordinator (10:48), after an outside review of the blog post's terminology that
Louis obtained. Two points of substance, for record 49's spec:
1. the min-of-bounds rule is a heuristic composition, not a joint optimum;
2. F is a proxy.

**Terminology from here on:**
- *contention-associated overhead,* for the b-term;
- *cache-derived concurrency bound* (W_L3);
- *per-worker CPU utilisation* (the busy share);
- *LLC-sharing domains*.

**Labels:** **C** counted · **M** modelled · **Conj**.

## 1. The rule composes bounds; it does not jointly optimise (M)

**What record 49's rule does.** It estimates three bounds separately and takes the smallest:
- the cache-derived concurrency bound W_L3, about 0.8 L / F;
- the CPU bound, 0.80 N;
- the serial bound, W_ser.

**Why that is a heuristic.** The bounds interact: removing a worker both frees CPU and lowers LLC
pressure, so near two bounds the true optimum can sit below both. The model they come from is
one function:

  wall(W) ∝ c(W) / min(0.93 W, N − ν − ι), with c(W) = a + b · W · W_run and W_run = min(0.93 W, N − ν − ι) / 0.93

**The check.** Integers W, with ν = 0.16 W, ι = 0.125 N, and W_L3 = 7.9 at -9 on the i9:

| N | joint optimum of the fitted model | min of bounds, m | the rule's pick (smaller within 1%) | rule against joint | measured best |
|---|---|---|---|---|---|
| 4 | 3 (4 at +2.1%) | 3.2 | 3 | 1.000 | **4** (lazy 3 at 1.014 of lazy 4; eager 4 best) |
| 6 | 5 | 4.8 | 5 | 1.000 | 5 |
| 8 | 6 | 6.4 | 6 | 1.000 | 6 |
| 10 | 8 (7 at +0.7%) | 7.9 | 7 | 1.007 | 7 |

**What the check shows:**
- *Composing the bounds costs at most 0.7%* against the fitted model's own optimum, at the measured
  counts.
- *The model is the larger error.* It misses the measured best at N = 4 by 3–4% on the 3-against-4
  comparison: it says 3 wins by 2.3%; measured, 4 wins by 1.4%. At N = 10 it misses by 1.9%.

**Another inconsistency in record 49, corrected here.** Its table gives 4 at N = 4, from record
46's earlier non-worker load (ν = 0.5, fixed). With the spec's own ν ≈ 0.16 W (or the measured
0.646 and 0.617), the rule gives **3**.
- *What decides it:* the eager-trim run's eager 3 at 4 threads, predicted 0.99 [0.95, 1.03]
  against production's eager 4.
- *Until then:* the spec's safe default at N = 4 stays **eager 4**, as measured.
- *A guard for the plan:* no trim at N ≤ 4 until a run supports one.

## 2. F is a proxy (C, M)

**What F is.** The 2.1 MB per worker, and the -3 / -9 ratio of 0.355 (5.97 / 16.81 MB), are
end-of-block **buffer capacities** (`cells.buf` + `tt` + `seg.bufs` + `ll`). They are neither a
temporal working set nor a measured LLC residency.
- *Where it is harmless:* on the i9 at -9, W_L3's constant was calibrated with the same F, so F's
  absolute value cancels.
- *Where it bites:* only through ratios, across block sizes and across machines.

**Does k's transfer (record 48) depend on F being capacity? Yes, through the ratio.** k =
ln(b₋₃ / b₋₉) / ln(ρ), with ρ the hot set's -3 / -9 ratio. The buffers shrank by different
factors:

| buffer | -9 (MB) | -3 (MB) | ratio | k from the cycle slope (0.138) |
|---|--:|--:|--:|--:|
| `cells.buf` | 6.08 | 2.10 | 0.345 | 1.86 |
| `tt` | 5.40 | 1.68 | 0.311 | 1.69 |
| `seg.bufs` | 3.32 | 1.41 | 0.425 | 2.31 |
| `ll` | 1.48 | 0.53 | 0.358 | 1.94 |
| the total (used) | 16.81 | 5.97 | 0.355 | 1.91 |

So **k is 1.7 to 2.3**, depending on which buffer carries the reuse. It is near 2 in every case,
but not pinned. Record 49's W_L3 scaling, F(900 k) / F(s), inherits that spread.

**The stronger input, before the plan ships off the i9.** A working-set or miss-ratio-curve
measurement per block size: the LLC miss ratio against the cache a worker effectively holds.
- *Here:* the i9-10900K has no cache partitioning (CAT), so the curve would come from software:
  sampled load addresses (PEBS through `perf mem`) and their reuse distances, block by block.
- *Its use:* this replaces F by the cache size at which a worker's miss ratio reaches a set level.
  That is the quantity W_L3 needs.
- *How much it would cost:* noted as the input to measure, not a new line.

## 3. Record 49's spec, restated with its scope

W\* = the smaller of ⌊m⌋ and ⌈m⌉ within 1%, where

  m = min(N, Σ_d min(N_d, W_L3,d), 0.80 N, W_ser)

- **A heuristic composition** of separately estimated bounds. On the i9 it is within 0.7% of the
  fitted model's joint optimum.
- **F is buffer capacity,** a proxy for the hot set.
- **The kernel at trimmed counts, and N ≤ 4, wait on the eager-trim run.**
- **Calibrated on one machine.** It ships off the i9 only after a transfer test, with a
  working-set measurement preferred to F.

## Amendment (17 November, 11:00): what F is (C)

The Coordinator corrected its 10:48 message: the 2.1 MB is the calibration journal's hot (reused)
footprint per worker, not the buffer sum. §2 above inherited the wrong description.

**1. What F is in records 45–50.**
- *Records 45, 46 and 49:* F = 2.1 MB a lazy worker at -9, taken from #522.
- *It matches record 39's census:* the lazy store is 29.5 B a letter (words, slots, entries,
  bytes), and Wikidata at -9 has r = 69,048 letters a block (every 24th block). That is
  **2.04 MB**. So F is the lazy kernel's per-letter store, an estimate of the reused working set,
  not LLC residency.
- *On the i9 at -9:* its absolute value cancels in W_L3, as before.

**2. k's transfer used the buffer ratio as a stand-in. Yes.**
- *Where:* records 48 and 50 used 0.355, the occupancy instrument's summed buffers (5.97 / 16.81
  MB), as the hot-set ratio.
- *The hot set counted directly:* -3 has r = 26,046 letters a block (every 72nd block, the same
  offsets as -9's every 24th). The store is 0.77 MB, so the hot-set ratio is **0.377** (r/n:
  0.0868 against 0.0767; letters are denser in smaller blocks).

| slope ratio, -3 against -9 | k on the buffer ratio, 0.355 | **k on the hot-set ratio, 0.377** |
|---|--:|--:|
| cycles, W = 6–9 (0.138) | 1.91 | **2.03** |
| LLC misses, W = 6–9 (0.114) | 2.10 | **2.23** |
| cycles, W = 7–10 (0.164) | 1.74 | **1.85** |
| LLC misses, W = 7–10 (0.135) | 1.94 | **2.05** |

**What changes:**
- *k:* with the hot set, k is **1.85–2.23, centred near 2.0–2.1**, about 0.1 higher than on the
  buffers. §2's 1.7–2.3 by-buffer range is superseded; it compared buffers.
- *W_L3 at -3:* 0.8 × 20 MiB / 0.77 MB is about 22, against record 45's "about 24". That changes
  nothing at 10 threads, where the scanner binds (record 48).
- *Still open:* the census counts what the kernel stores, not what it reuses within the L3's
  window. A miss-ratio curve (§2) stays the stronger input.
