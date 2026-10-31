# P74-C confirmatory run on GitHub Actions: CI run 41190706676

**The run:**
- Protocol `2026-10-31-p74-c-confirmatory-protocol.md`, at the pinned commit `8b6caf1`
  (freeze `3b666e6`).
- Three independent `ubuntu-24.04` jobs, started by a push to `research/p74c-ci-trigger`.
  Each job checked out the pinned commit.
- The binary SHA-256 is `311b0775…577be190`, identical in all three jobs: a reproducible
  build.
- The structural audit passed on the timed binary in every job.
- Each job pinned CPU 2 (core 1) with its sibling, CPU 3, idle.
- Steal on the pinned CPU was 0.00% in every job (`/proc/stat` before and after).
- 20 paired rounds, 5 arms.
- Ratios are geometric means of per-round time ratios, with 95% intervals as the
  summariser computes them (log ratios, the ρ₁ correction, Holm within families).

| Job | CPU | Ticks on the pinned CPU |
|---|---|--:|
| r1 | AMD EPYC 7763 (Zen 3) | 26,098 |
| r2 | AMD EPYC 9V74 (Zen 4) | 20,833 |
| r3 | AMD EPYC 9V74 (Zen 4) | 20,737 |

## Registered verdict (measured, mechanical)

| Gate | r1 7763 | r2 9V74 | r3 9V74 |
|---|---|---|---|
| A/A calibration | pass (0 of 7 allowed rows exclude 1) | pass (2 of 7) | pass (2 of 7) |
| Primary: fitted Wikidata checkpoint, T2/T0 upper 95% ≤ 0.95 and gain > 2× noise | **pass** (0.656) | **pass** (0.748) | **pass** (0.746) |
| Held-out Wikidata (same dump) | **pass** (0.667) | **pass** (0.761) | **pass** (0.756) |
| Negative controls, joint (Holm, ≤ +3%) | **pass** | **pass** | **pass** |
| Adversarial family (Holm; ≤ +3%, or ≤ +1% for bypassed files) | **fail** | **fail** | **fail** |

**Outcome under the registered stop rule (§7.6, "any failing row is a negative"): negative
for option C, on all three hosts.**

**The failing rows** are only bypassed edge files, which run the standalone shipped kernel
through the bypass. They are tested against the +1% limit:

| File (variant) | T2/T0 upper 95%: r1 | r2 | r3 |
|---|--:|--:|--:|
| `edge-l9-s16383` (plain) | 1.022 | 1.016 | 1.010 |
| `edge-l9-s16383` (checkpoints) | 1.029 | 1.010 | 1.016 |
| `edge-l1-s50000` (plain) | 1.013 | 1.021 | 1.012 |
| `edge-l1-s50000` (checkpoints) | pass | 1.010 | pass |

- **Point estimates** range from 0.994 to 1.011.
- **Absolute size:** about 1 µs on blocks of 100–480 µs.
- **The files' A/A rows** range from 0.993 to 1.007.
- **Every non-bypassed adversarial row passes,** at T2/T0 from 0.34 to 0.99.

## Figures (measured), T2/T0 as the ratio

| Corpus, variant | r1 7763 | r2 9V74 | r3 9V74 |
|---|--:|--:|--:|
| fitted Wikidata, plain | 0.719 | 0.724 | 0.722 |
| fitted Wikidata, checkpoints | **0.654** | **0.747** | **0.743** |
| held-out Wikidata, plain | 0.725 | 0.735 | 0.732 |
| held-out Wikidata, checkpoints | **0.666** | **0.760** | **0.754** |
| random, plain / checkpoints | 1.001 / 1.001 | 0.998 / 1.000 | 1.000 / 1.000 |
| records, plain / checkpoints | 1.002 / 1.000 | 1.000 / 1.001 | 0.999 / 0.997 |
| mixed-independent, plain / checkpoints | 0.999 / 1.000 | 1.000 / 1.007 | 0.998 / 1.000 |
| text, plain / checkpoints | 0.693 / 0.753 | 0.832 / 0.850 | 0.831 / 0.842 |

**The other arms on Wikidata, checkpoints** (fitted, then held-out):

| Arm | r1 | r2 | r3 |
|---|--:|--:|--:|
| T1/T0 (the layout tax) | 1.003, 1.005 | 1.027, 1.030 | 1.025, 1.024 |
| T2/T1 (P74's own effect) | 0.652, 0.663 | 0.728, 0.738 | 0.725, 0.736 |
| Free/T0 | 0.768, 0.779 | 0.751, 0.764 | 0.750, 0.760 |

## What the switches show (measured)

- **Fitted and held-out Wikidata:** no handoffs (198 of 431 and 211 of 473 blocks
  bypassed).
- **Random and records:** every block bypassed.
- **Where P74 loses on the controls, it is never run:**

  | Free/T0 | r1 7763 | r2, r3 9V74 |
  |---|---|---|
  | random | 0.90–0.91 | 1.01–1.07 |
  | records | 0.93–0.97 | 1.13–1.18 |

  On Zen 4, then, the protection comes from the σ bypass, not the online guard.
- **Where the guard fires** (22 suite files, mixed, text), T2 is slower than Free. For
  `rank-255` checkpoints on 9V74, T2/T0 is 0.446 against Free/T0 0.430.
  - On every switching block measured, staying in P74 would have been faster.
  - The run has no block that reaches the guard and on which P74 is slower than the
    shipped kernel. The guard's protective value is therefore unobserved, and its cost
    where it fires is a measured loss against Free.
- **The suite's "rank" and "hug" files** are run-heavy. P74 wins on them by 2–3× (Free/T0
  0.34–0.50 on 9V74, 0.62–0.74 on 7763), so they don't stress P74.

## Reading (deduced; not a new registered result)

1. **The Wikidata stage gain replicates on clean hosts, with zero steal:** −25% to −35% on
   checkpoints and −27% to −28% on plain, out of sample (held-out) as well as in sample.
2. **The registered negative** comes from about 1% dispatch overhead on small bypassed
   blocks, not from P74's kernel or the guard.
3. **Within the tested inputs,** the σ bypass alone provides all the observed protection,
   and the guard only costs.

   A policy change, such as dropping the guard or changing what the bypass does on small
   blocks, would be a new hypothesis. It would need its own pre-registration and new
   held-out data, since these results can't be reused to tune it.

**Artifacts:** each job's `summary.json`, `environment.json`, `asm-audit.json`, `run.log`,
`stat-*.txt` and `raw.tsv.gz` are in this folder. The full tarballs are the run's artifacts,
retained until 2026-11-30.
