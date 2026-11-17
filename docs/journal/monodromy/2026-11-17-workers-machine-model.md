# 2026-11-17 · MONODROMY 45 · A worker count the machine sets. The L3 term binds where the workers' footprints fill 0.8 of the L3, whatever the exponent; homogeneity makes k the same exponent as W's. A -3 file at 10 threads pins k, and should remove the trim. On CI's 2-core runners the L3 term never binds: W* = 4, and production's eager plan stays best

Asked by the Coordinator (08:59), a standing assignment from Louis. The calibration record is
`docs/journal/2026-11-17-worker-count-calibration.md` (#522). Three deliverables:
1. the model's form and parameters, fitted on the i9;
2. what a second block size must show to pin k;
3. predictions for a GitHub runner, as a formula, frozen before the CI sweep.

**Labels:** **C** counted · **M** modelled · **Conj** conjecture.

## 0. Answers

1. **The form (M on C).** Decoding throughput with W workers, with wall = 1 / R(W) and W\* = argmax_W R(W):

   R(W) = Y(W) · (1 − d(N − W)) / (1 + γ · x(W)^p),  where x(W) = W · F(s) / L

   - **x:** the workers' combined footprint as a share of the L3.
     - *L:* L3 bytes, as `l3_bytes()` reads them.
     - *F(s):* one worker's hot footprint at block size s, from the stream header.
   - **Y:** the cores' yield. Y(W) = min(W, C) + (σ − 1) · max(0, W − C) on C physical cores, where σ is
     the yield of a core running two workers on SMT siblings.
   - **d:** the busy deficit, d(h) = d₀ e^(−h/h₀). Here h = N − W is the number of CPUs the process may
     use (N) left over after the workers.
2. **The L3 term's optimum is robust; its exponent is not (C).**
   - *What the i9 can't pin:* W = 6–9 fit equally well for any p from 1.5 to 2.5 (rms 0.006–0.009).
     The uncontended cost then ranges from 0.30 to 0.49 of production's cycles at 10 workers.
   - *What every such fit gives:* the same optimum, **x\* ≈ 0.8** (0.77 to 0.84). That is
     W_L3 ≈ 0.8 · L / F(s): about 7.9 on the i9 at -9.
   - *The teams rule:* production's teams policy already sizes to "live tables near 0.7 of the L3".
     This is the same law, one level down.
3. **k = p, if contention depends only on the total occupancy x (P, from homogeneity).** A capacity
   effect depends only on x, which makes F/L's exponent the same as W's. So the block-size run tests
   two things: whether contention is a function of x alone, and what p is.
4. **The second block size: a -3 file, not -1 (§3).** At -1 a lazy worker's footprint (about
   0.25 MB) fits in the i9's 256 KiB L2, which changes the uncontended cost and confounds the test.
   - *At -3, homogeneity predicts:* x at 10 workers falls to about 0.33. Then **the trim
     disappears at 10 threads**: W\* is 9 or 10, set by the busy deficit alone.
   - *Pinning k:* LLC misses a byte at -3 with W workers should equal those at -9 with
     W · F(-3)/F(-9) workers, which the low sweep's W = 2–5 provides.
5. **On CI's runner (§4) the L3 term doesn't bind:** W_L3 ≈ 0.8 × 32 MiB / 2.1 MB ≈ 12.8 ≫ 4.
   - *What sets W\*:* the SMT and busy terms, calibrated from production's own 2- and 4-thread
     runs there.
   - *The prediction:* **W\* = 4, with 3 slower by 0–5%, and 2 by 3–15%.** Every lazy arm loses to
     production's eager four.
   - *The plan:* no trim and no lazy kernel on a 2-core runner.
6. **The ship rule can't fail on this runner, which I flag.** "W\* within one worker" passes any
   outcome in 3–5. Since 4 vCPUs allow only W = 2–4, it passes everything but W\* = 2, which the
   data already rule out. §5 proposes a rule that can fail.

## 1. The parameters, fitted on the i9 (C, M)

The data: the 10-thread counters (#515, LOG 6943194948) and the past-the-cores counters (#514),
cycles a byte for W = 6–9. The second series is joined by one free scale (1.161).

| p | rms | a (uncontended) | b | max residual |
|--:|--:|--:|--:|--:|
| 1.5 | 0.0061 | 0.300 | 0.0248 | 0.012 |
| 1.75 | 0.0058 | 0.370 | 0.0129 | 0.010 |
| **2.0** | 0.0061 | 0.422 | 0.0068 | 0.011 |
| 2.5 | 0.0085 | 0.494 | 0.0020 | 0.015 |
| 3.0 | 0.0119 | 0.543 | 0.0006 | 0.019 |

**What fixes p.** LLC misses a byte rise as W^2.0 at 10 threads and W^1.6 past the cores. I take
**p = 2 [1.5, 2.5]**.
- *γ:* in x's units, with F = 2.1 MB a lazy worker at -9 (#522) and L = 20 MiB, γ = (b/a) /
  (F/L)^p. That is **1.60** at p = 2.
- *The optimum without the busy term:* x\* = (γ(p − 1))^(−1/p). That is 0.79 at p = 2, 0.84 at 1.5
  and 0.77 at 2.5.

**The busy deficit.** From the counted busy shares at 10 threads, normalised to their saturation:
- d(0, 1, 2, 3) = 0.18, 0.10, 0.036, 0.018;
- the fit: **d₀ = 0.18, h₀ = 1.3 CPUs**.

On the i9, ab pins a run of T ≤ 10 threads to T CPUs on T cores (`--spread`), so N = T.

**The model overstates the gains at h = 0–2 by about a third.** This is record 44's calibration
(0.65), on its own two points; the low sweep scores it now.

The structural cause (C, Conj):
- *Measured:* c(10) at 10 threads is 1.0.
- *The quadratic gives:* 1.099.
- *So:* at W = N, cycles a byte sit 9% under the curve, as if partly idle workers relieved the
  contention.

Until a run separates the two, the deficit and the relief are calibrated together, by that factor.

**The SMT yield σ (C, on the runner).** Production's own runs on CI give throughput at 4 threads
over 2, each with 4 CPUs allowed (`occupancy`, 20 runs, 16–17 November):

| CPU | 4 threads over 2 |
|---|---|
| EPYC 7763 | 1.09–1.12 |
| EPYC 9V74 | 1.05–1.15 |
| EPYC 9V45 | 1.03–1.10 |
| Xeon 8370C / 8573C | 1.08–1.09 |

The workers' decoding share is 0.96 at 2 workers and 0.86 at 4.
- *σ with no L3 term:* 1.24.
- *σ with the i9's γ carried over:* 1.38.

The i9's σ is not measured for the current kernel.

## 2. What the form says about the plan (M)

**For W ≤ C,** W\* is the smaller of:
- the L3 optimum, about 0.8 · L / F(s);
- the busy optimum, a fixed number of CPUs left over, about T − 3 on the i9.

The trim pays only where the L3 optimum is the smaller.

**The inputs.** All of them are already read at runtime:
- C and N: `physical_cores()` and the affinity;
- L: `l3_bytes()`;
- s: the stream header.

**One kernel constant: F(s).** Its absolute value cancels on the i9, because x\* was calibrated
with it. It matters only through F(s)/F(900 k) at another block size and through L/L_i9 on another
machine:

  **W_L3(machine, s) ≈ 7.9 · (L / 20 MiB) · (F(900 k) / F(s))**.

## 3. What the -3 run must show (pins k)

**The run.** Wikidata ×4 recompressed at -3, on the i9, at 10 threads: lazy W = 10, 9, 8, 7, 6, and 4
as a low anchor. Report:
- cycles and LLC load misses a byte;
- the busy share;
- each worker's peak footprint (the ledger), which gives F(-3)/F(-9) by count, not by
  proportion to s.

**Predictions (M), with F(-3)/F(-9) = ρ (≈ 1/3 if the hot footprint scales with s):**

| | homogeneity, k = p = 2 | if contention does not shrink with F (bandwidth-bound) |
|---|---|---|
| x at 10 workers | 0.33 | (irrelevant) |
| cycles a byte, 6 against 10 | 0.95–0.98 | 0.67, as at -9 |
| **best W at 10 threads** | **9 or 10** (busy term only); 7 against 10 no better than 0.98 | 7, about 0.885 as at -9 |
| LLC misses a byte, 10 against 6 | about (10/6)² × the -9 ratio at W = 2–3.3 | 2.5, as at -9 |

**Pinning k.** From the two sizes, k = ln(b(-3)/b(-9)) / ln ρ.
- *The resolution:* the contention share of cycles at 10 workers falls from about 0.6 at -9 to
  about 0.07 at -3 (k = 2) or 0.2 (k = 1). The difference between W = 6 and 10 then resolves k to
  about ±0.3.
- *The pairing:* misses at -3, W = 6–10, against misses at -9, W = 2–3.3 from the low sweep, test
  whether contention depends on x alone, with no fitted k.

**Why not -1.** A lazy worker's footprint at -1, about 0.25 MB, sits at the i9's L2 size, so the
uncontended cost changes with s and isn't held fixed. If a -1 run is cheaper to stage, its W\*
prediction is the same (no trim), but it cannot pin k.

## 4. Frozen predictions for a CI runner (M)

**The formula,** for any runner: read C, N, L and SMT from `lscpu`, then

  W\* = argmax over W ≤ N of Y(W; C, σ) · (1 − 0.18 · c · e^(−(N−W)/1.3)) / (1 + 1.60 (W · F / L)²)

- *σ:* 1.24 to 1.38.
- *c:* scales the busy deficit to the machine: 1 on the i9 (pinned to T CPUs). The runner's
  decoding shares fit c ≈ 0.55, which gives deficits 0.10 at h = 0 and 0.02 at h = 2, against the
  i9's 0.18 and 0.036.

**For GitHub's ubuntu-latest:** 4 vCPUs on 2 cores (C = 2, N = 4), L = 32 MiB as lscpu reports for
the 7763, 9V74 and 9V45.
- *The L3 term:* x(4) = 0.25 lazy and 0.34 eager. Contention is at most 10–18% of cycles, and it
  never binds below W = 12.
- *What decides W\*:* SMT and the busy share.

| arm (unpinned, 4 CPUs) | against lazy at 4 workers | against production (eager, 4 workers) |
|---|---|---|
| lazy 4 | 1 | **1.07 [1.02, 1.12]** (+25% instructions; little contention to relieve; i9 at 4 threads: 1.086) |
| lazy 3 | **1.02 [0.99, 1.05]** | 1.09 [1.03, 1.16] |
| lazy 2 | **1.09 [1.03, 1.15]** | 1.17 [1.08, 1.27] |
| **W\* (lazy)** | **4**, or 3 within 2% | |
| **plan** | | **production unchanged:** eager, W = T |

**By CPU model.** Where SMT yields least (9V45, 1.03–1.10), W = 3 is likeliest to tie. Where it
yields most (7763), 4 wins more clearly.

**The other tenants.** Other customers' VMs share the runner's L3, on the same CCD. If they took most
of it (L near 8 MiB), the L3 term would bind near W = 3, and 2 workers would scale below 2× over 1.
They measure 1.94–2.09×, so the L3 the decoder can use is large. A run whose 1→2 scaling falls
below 1.85 has a crowded L3, and this prediction does not apply to it.

**Refuted if:**
1. lazy 3 beats lazy 4 by more than 2%; or
2. lazy 2 is within 3% of lazy 4; or
3. any lazy arm beats production by more than 2%.

## 5. A transfer test that can fail

**The proposed criterion.** On a 4-vCPU runner, W\* ∈ {2, 3, 4} and a prediction of 3 or 4 is within
one worker of all of them, so the ship rule as written passes by construction. I propose to judge
the runner on §4's three refuters and its intervals.

**Where a stronger transfer test is possible:**
- *The i9 at -3 (§3).* It moves W_L3 from 7.9 to about 24, so the trim should vanish: a clear,
  falsifiable change on hardware we have.
- *A machine where the L3 binds at a different W.* That needs at least 8 physical cores and a
  different L3 per core, for example GitHub's larger runners. Those are spending, Louis's under
  RULES D.

## Reproduce

Each number above is arithmetic on the counted figures it cites.
- *The p profile:* least squares of (a + bW^p)/y − 1 over both series, with one shared scale.
- *The runner ratios:* the occupancy runs' `off`-mode MB/s at 2 and 4 threads (`occupancy`
  workflow, 16–17 November, 20 runs).
