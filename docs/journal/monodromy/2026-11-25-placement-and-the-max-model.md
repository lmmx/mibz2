# Record 132. Placement as a lever, and the soft maximum against the max model

Answers QUEUED 7151960386. Inputs: journal §78 (stage 2b) and §76 only; I have not opened 2b's bundle. Units: the pinned wall at 8 workers = 1.

## 0. Record 131, scored (measured, §78)

Both scored quantities hit (e_oth 0.654 for 0.65; e_down 0.716 for 0.79). **My error:** H. I put 1 chance in 3 on NOT-HOMOGENEOUS and it was the outcome at every cell (H 0.79 at 8, 0.45 at 10). §3 gives a cause.

## 1. The two models are one model (proved)

Let x = log(S/W), and let the sharp wall be log T = log W + max(x + ε, 0), with ε a jitter on the scanner's relative speed. If ε is logistic with scale 1/p, then

E[log T] = log W + (1/p)·log(1 + e^(px)),

which is exactly the soft maximum (S^p + W^p)^(1/p). Proof: both sides have derivative in x equal to P(ε > −x) = 1/(1 + e^(−px)), and both tend to log W as x → −∞.

**So a soft maximum of exponent p is a sharp maximum whose balance jitters with SD π/(p√3).**
- p = 21 ⇔ SD 8.6%. Stage 2a's measured paired SD at 8 was 8.1% (§77). **The roundness I fitted on 2a was the unpinned scanner's placement noise, not a property of the pipeline** (argued; the match is one number).
- Pinned, 2b's intervals are 2–4× narrower: SD 2–4%, so p is 45–90, and within the data's resolution that is the max model.

**Where they part:**
| | max (p → ∞) | soft, p = 21 |
|---|---|---|
| sides more than 10% apart | same | within 0.5% |
| at balance | T = max | 3.4% above max |
| pinned 8 (W/S = 0.966, §2) | 1.000 | 1.019 |
| elasticity to the binding side | 1 | 0.67 locally, 0.83 at +10% |
| measured, 2b pinned | 1.05–1.08 | — |

**Reading for the next held-out test:** pinned arms, the max model. **Unpinned arms (production, #861 at 9 threads), the soft form with p = π/(√3·SD) from that run's own paired SD,** fixed before the treatment arms are read. The max model will under-predict an unpinned scanner speed-up's gain near balance and over-predict its slowdown; that is the test that separates them.

## 2. The state at 8 workers, from 2b alone (argued)

Under the max model the non-binding side's elasticity to a dose L is (log L − g)/log L, g the gap. Workers' 0.64 at +10% gives g = 3.4%; e_up 0.181 at +20% gives 3.3% of wall. So **pinned at 8: S = 1, W = 0.966.** Unpinned the scanner is 4% slower (§78.6): S = 1.04, and the whole 4% reaches the wall, as measured.

Sibling cost: 8 workers on 4 cores cost +48% with the scanner binding, so a worker on a shared core runs at σ ≤ 0.68 of its unshared speed; at 4 workers σ = 0.82. I use σ = 0.68: **a worker added on a sibling is worth 2σ − 1 ≈ 0.36 of a worker.** This agrees with stage 1: fully shared worker inflation 1.64–1.73 at 15–20 workers, and 1.17 × 1.48 = 1.73 here.

## 3. Why H < 1 (conjectured, one mechanism)

A paced thread spends its dose spinning. If it spins on a shared core, its sibling gets the core's resources back for that time, so pacing speeds the neighbour. Then H = 1 − (share of the binding thread's time spent beside a paced sibling) × (the sibling's gain), and H falls with sharing: 0.79 at 8 (11 threads on 10 cores), 0.45 at 10 (13 threads). It also gives the negative worker coefficients at 10: pacing A7 frees the scanner's or another worker's sibling.

**Kill rule:** H at 6 workers (9 threads, no forced sharing) is below 0.9; or the pacing loop sleeps rather than spins. `NEEDS M-39`.

**Consequence:** homogeneity arms can't price a speed-up wherever threads exceed cores. Price it from the gap instead (§2), which needs only one-sided doses.

## 4. What a library can request, priced (max model; argued unless marked)

P = physical cores (10). Threads = k workers + scanner + reader + feeder.

| request | portable? | buys at 8 | note |
|---|---|---|---|
| **A. total threads ≤ P, chosen from physical cores** (k = P − 2, reader and feeder sharing one core) | everywhere: it is only a count | avoids the losses below; = today's 8 | §76's tie of 8, 9, 10 is this row |
| B. scanner alone on a core, by affinity | Linux, Windows; not macOS | −4% (measured) | owns a CPU; a second decoder in the process collides |
| **C. scanner's sibling kept idle without naming a CPU** (Linux core scheduling: a private cookie for the scanner thread) | Linux ≥ 5.14 where enabled | up to −4% | a request, not ownership; the kernel may still migrate. To be checked against the kernel's documentation before use |
| D. workers kept off each other's siblings by affinity | Linux, Windows | 0 at k ≤ P − 2; it only prevents | never a cookie per worker: forced idle halves a core where sharing gives 1.36 |
| E. a 9th, 10th worker on siblings | count only | 0 today (scanner binds); CPU +15%, RSS +13% (§76) | worth 0.36 each once W binds |
| F. feeder folded into the scanner's thread | decoder change | removes one runnable thread: 10 threads on 10 cores at k = 8 | makes A exact rather than approximate |

**Mispairing is the risk, not the mean.** With 11 threads on 10 cores one pair must share. If it is two workers: W = 0.966 × 8/7.36 = 1.05, wall +5%. If it is the scanner: S × 1.3–1.45 (stage 1), wall +30–45%. If it is reader and feeder: nothing. The scheduler doesn't know which pair is cheap; 2a saw the scanner sharing about half the time.

**With a faster scanner** (multi-symbol decode, S × 0.88), pinned:
| k | worker capacity | W | wall = max(S, W) |
|---|---|---|---|
| 8 | 8 | 0.966 | 0.966 |
| 9 | 7 + 2σ = 8.36 | 0.924 | 0.924 |
| 10 | 6 + 4σ = 8.72 | 0.886 | 0.886 |

So of the 12%, 3.4% arrives at 8 and 11.4% at 10 workers, **if** the added workers pair with each other and never with the scanner. Placement turns from a 4% lever into the condition for the larger one.

## 5. The necessary condition (Khammash's form; argued)

For the wall to reach the scanner's quiet rate, it is necessary that **(N1) the scanner's core carries no second runnable thread, and (N2) Σ over workers of σᵢ/w ≥ 1/S.** N2 is arithmetic on k. N1 is the structural one: no count and no hint guarantees it while the decoder keeps more runnable threads than cores, because some pair must share and the kernel chooses it.

**The decoder-side change that is necessary: the scanner's work must be the one thing in the process whose core is not shared, enforced by construction rather than by luck.** Three realisations, in order of how little they ask of the machine:
1. F then A: fewer runnable threads than physical cores, so an idle core always exists for the scanner (sufficient only if the kernel prefers idle cores; conjectured, `NEEDS M-40`);
2. C: the cookie;
3. B: affinity.

**The converse lever (mathematics, T4):** N1 exists because the scanner is one thread on the critical path. Any design that makes the scan poolable (record 129's pooled wall (s + w)/c) removes N1 altogether: a pooled stage has no distinguished core. That is the placement-free answer, and stage 2b raises its value from "13–17% off the scan" to "that, plus immunity to the +30–45% mispairing".

## 6. Labels

Proved: §1's identity. Measured: §0, the inputs of §2. Argued: §2's gap, §4's prices, §5. Conjectured: §1's reading of p as noise, §3, N1's realisation 1.


> **Corrected by record 133** (`2026-11-25-corrections-129-132.md`).
