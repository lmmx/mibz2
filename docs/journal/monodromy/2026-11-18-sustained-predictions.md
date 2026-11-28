# 2026-11-18 · MONODROMY 64 · Frozen before the sustained-load recalibration: 8 workers best in the clamped state at 10 or more threads, pinned and unpinned. This revises record 58's weak conjecture ("at or below 7") the other way, before any data, from the recalibration bundle's own clamped runs and a cycles argument. Clock, power and probe predicted; four design notes

**The request.** The Coordinator's CHECKPOINT 6984235551, for the Pathfinder's design (CHECKPOINT
6984075662). Frozen **before any of its data.**

**Labels:** **C** counted (from `recalibrate/run-20261118T122744Z`, record 58's bundle) · **M** · **Conj** · **I**.

## 1. The basis: the recalibration's own clamped runs (C), and why they reverse record 58's guess (M)

**The clamped runs, taken apart.** Record 58 split each bimodal cell into slow (clamped clock) and fast
(turbo) runs. Their geometric means against 7/28, by state, on Wikidata ×4 (n = 3–9 per state; small):

| cell | 8/32, clamped / turbo | 10/x, clamped / turbo | 6/24, clamped / turbo |
|---|---|---|---|
| unpinned 10 | **0.960** / 0.956 | **0.964** / 1.044 | 1.058 / 1.073 |
| unpinned 12 | **0.977** / 0.990 | 1.000 / 1.066 | — |
| unpinned 16 | **0.976** / 0.988 | 0.977 / 1.028 | — |
| unpinned 20 | **0.964** / 0.977 | 0.971 / 1.020 | — |
| pinned 10 | **0.973** / 0.966 | 1.014 / 1.057 | 1.061 / 1.104 |
| unpinned 8 | 0.971 / 0.968 (8/32 against 7/28) | — | — |

**What they show:**
- *In the clamped state 8 beats 7 everywhere,* by 2–4%.
- *10 workers move from losing* (1.02–1.07 in turbo) *to about level* (0.96–1.01) when clamped.

**Why (M).** Two effects favour more workers under a clamp:
1. **Contention is cheaper in cycles.** A clock lower by about 1.4× makes each DRAM or L3 miss cost
   about 1.4× fewer core cycles, so the contention term b falls relative to a. The L3-limited optimum
   √(a/b) rises by about √1.4, from 7.9 to about 9.3.
2. **A power cap rewards width.** With P ∝ N f³ fixed, f ∝ N^(−1/3) and throughput ∝ N f ∝ N^(2/3),
   which increases in N.

**So record 58's weak conjecture,** that the clamped optimum sits at or below the turbo one, **is
revised before the data:** I now predict the clamped optimum at or above it. Both stand frozen, to be
scored.

## 2. The predictions (wall over the shipped reference, sustained state, Wikidata)

| cell | reference | **predicted best** | arms: predicted [interval] |
|---|---|---|---|
| **Unpinned** (for 10–20 threads) | lazy 7/28 | **8** | 8/32: **0.97 [0.94, 1.00]** · 10/40: 0.985 [0.95, 1.02] · 6/24: 1.06 [1.03, 1.09] |
| **Pinned 10** | lazy 7/28 | **8** | 8/32: **0.975 [0.95, 1.00]** · 10/40: 1.01 [0.98, 1.05] · 6/24: 1.06 [1.03, 1.09] |
| **Pinned 8** | lazy 6/24 | **6** | 7/28: 1.03 [1.00, 1.06] · 8/32: 1.04 [1.01, 1.07] · 5/20: 1.12 [1.10, 1.15] |
| **Pinned 6** | eager 5/20 | **5** | 6/24: 1.07 [1.05, 1.09] · 4/16: 1.03 [1.02, 1.05] |
| **Pinned 4** | eager 3/12 | **3** | 4/16: 1.045 [1.03, 1.06] · 2/8: 1.37 [1.33, 1.40] |

**How these were set.**
- *Pinned 8, 6 and 4:* centred on the recalibration's tight turbo-state ratios (1.040 / 1.050 /
  1.132, 1.073 / 1.033, 1.045 / 1.366).
- *Pinned 8:* the CPU budget (record 49), not contention, sets the trim there. A clamp narrows the
  gaps slightly.
- *Pinned 6 and 4:* these may never clamp (§3).

## 3. Clock, power and the probe (M, Conj)

**The effective clock** (cycles ÷ ref-cycles; the turbo state is 1.326, about 4.9 GHz):

| cells | predicted | GHz |
|---|---|---|
| clamped (unpinned, pinned 10) | **0.93–0.97** | 3.45–3.6, the observed 1.40× slowdown |
| pinned 8 | 1.0–1.33 (Conj) | partial clamp, uncertain |
| pinned 6 and 4 | 1.32–1.38 | turbo, unclamped |

- *Within a clamped cell,* arms with fewer workers run 2–5% faster clocks: 6/24 above 10/40.

**Package power** (`energy-pkg`):
- *Clamped cells:* steady near the board's long-term limit, about equal across arms. **115–135 W if
  PL1 is at Intel's 125 W.**
- *If the board's limits are higher and the clamp is thermal:* 150–200 W, drifting with
  temperature (Conj). The probe decides which.
- *Unclamped cells:* 4 cores about 70–100 W, 6 cores about 90–130 W.

**DRAM:** per output byte as in the admit run (8.05 B at 7 workers, 9.16 at 8). GB/s falls with the
throughput, about 15 GB/s at 7 clamped.

**The probe** (120 s, 10 workers, unpinned):
- *Onset:* **within 0.5–5 s.** In the recalibration, a heavy run became slow after under a second of
  prior heavy load.
- *Steady level:* about **1.40× below turbo** (3.4–3.7 GHz), stable within ±3% after onset if the
  limit is PL1-type.
- *If thermal:* a slow downward drift over the 120 s.

**Refuted if:**
1. at 10 or more threads, unpinned, 7 beats 8 by more than 1%, which supports record 58's
   conjecture instead; or
2. the clamped cells' clock stays at 1.2 or more, meaning the sustained state isn't reached.

## 4. Design notes, all of which can be met within the design

1. **Screen each decode for its clock state.** Record cycles ÷ ref-cycles per decode, which the design
   already collects. In the cells meant to be clamped, flag or VOID any decode at 1.2 or more, so
   that no arm mixes states as the bimodal regime did. Report the clamped fraction per arm.
2. **Pinned 6 and 4 may never clamp.** Report their state, so that "the sustained state decides"
   isn't claimed for cells that stayed in turbo.
3. **Carry-over between arms.** Back to back with no gates, an arm inherits the previous arm's
   thermal or power state. Rotation balances it. If the probe shows a time constant under 2 s, it is
   negligible; if longer, report each decode's predecessor.
4. **Unpinned standing for 10–20 threads** is supported: the recalibration's unpinned 10/12/16/20
   cells behaved alike in both states.
