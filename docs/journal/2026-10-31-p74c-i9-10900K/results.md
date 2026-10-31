# P74-C confirmatory run on the maintainer's i9-10900K

**The run** (run by the maintainer, `just bench-p74c i9-10900K`):
- Protocol `2026-10-31-p74-c-confirmatory-protocol.md`, at the pinned commit `8b6caf1`
  (freeze `3b666e6`), built with rustc 1.90.0 on the host.
- Binary SHA-256 `19b8aff4…`. It differs from CI's `311b0775…` because it was built on a
  different host and path; the structural audit was run on this exact binary.
- **Structural audit:** pass, with every gate true. Plain: Free 37 instructions, 6 jcc;
  combined 38, 6. Checkpoints: 38, 6 against 39, 6.
- **CPU:** Intel i9-10900K (Comet Lake). Timing CPU 1, with its sibling CPU 11 idle (0.9%
  busy over the run; CPU 1 99.9%). Steal 0, irq 0.
- **Governor:** `powersave` (intel_pstate); turbo state not recorded.
- **Wall time:** 169 s for the timed run.
- 20 paired rounds, 5 arms.

## Registered verdict (measured, mechanical)

| Gate | Result |
|---|---|
| A/A calibration | pass (1 of 7 allowed rows exclude 1) |
| Primary: fitted Wikidata checkpoint | **pass**: T2/T0 0.791, upper 0.798; noise 0.059 |
| Held-out Wikidata | **pass**: T2/T0 0.800, upper 0.811 |
| Negative controls, joint, ≤ +3% | **fail**: `mixed-independent`, checkpoints, T2/T0 1.017 [1.001, 1.033] |
| Adversarial family | **fail**: 7 of 54 rows (below) |

**Outcome under the registered stop rule: negative.**

**Failing adversarial rows** (T2/T0, upper 95%, limit). They fail because Holm-adjusted
tests couldn't reject "≥ limit"; some upper bounds sit below the limit but are too wide.

| File (variant) | T2/T0 | Upper 95% | Limit |
|---|--:|--:|--:|
| `split-q05` (plain) | 1.010 | 1.024 | 1.03 |
| `edge-l9-s16383` (plain) | 1.000 | 1.008 | 1.01 |
| `edge-l9-s16383` (checkpoints) | 0.981 | 1.020 | 1.01 |
| `edge-l1-s49999` (plain) | 0.893 | 0.991 | 1.03 |
| `edge-l1-s50000` (plain) | 0.993 | 1.013 | 1.01 |
| `edge-l9-s450000` (plain) | 0.992 | 1.007 | 1.01 |
| `edge-l9-s450000` (checkpoints) | 0.999 | 1.011 | 1.01 |

## Figures (measured)

| Corpus, variant | T2/T0 | T2/T1 | T1/T0 | Free/T0 | A/A |
|---|--:|--:|--:|--:|--:|
| fitted Wikidata, plain | 0.819 | 0.777 | 1.054 | 0.814 | 1.004 |
| fitted Wikidata, checkpoints | **0.791** | 0.834 | 0.948 | 0.795 | 1.002 |
| held-out Wikidata, plain | 0.825 | 0.784 | 1.051 | 0.830 | 1.004 |
| held-out Wikidata, checkpoints | **0.800** | 0.843 | 0.950 | 0.805 | 1.001 |
| random, plain | 1.005 | 0.980 | 1.025 | **1.212** | 1.000 |
| random, checkpoints | 1.000 | 1.046 | 0.956 | **1.227** | 1.008 |
| records, plain | 0.997 | 0.983 | 1.014 | **1.437** | 0.998 |
| records, checkpoints | 1.000 | 1.075 | 0.930 | **1.388** | 0.999 |
| mixed-independent, plain | 1.003 | 0.993 | 1.011 | 1.005 | 1.005 |
| mixed-independent, checkpoints | 1.017 | 1.035 | 0.983 | 1.046 | 1.019 [0.983, 1.056] |
| text, plain | 0.826 | 0.772 | 1.070 | 0.822 | 1.001 |
| text, checkpoints | 0.767 | 0.830 | 0.923 | 0.785 | 1.003 |

## Reading (deduced, set against the CI run 41190706676)

1. **The Wikidata stage gain replicates on bare-metal Intel:** −18% to −21%, in and out of
   sample, against −25% to −35% on Zen 3 and Zen 4.
2. **P74 alone loses heavily on Comet Lake controls:** +21% to +23% on random, +39% to +44%
   on records, worse than Zen 4's +1% to +18%. The σ bypass removes all of it: T2/T0 on
   random and records is 0.997–1.005.
3. **The guard's protection is clearly measured on the split files:**

   | File | Free/T0 | T2/T0 |
   |---|--:|--:|
   | `split-q05` | 1.42–1.45 | 0.95–1.01 |
   | `split-q25` | 1.30–1.32 | 0.91–0.97 |
   | `split-q50` | 1.16–1.17 | 0.88–0.90 |

   On run-heavy hug and rank files it costs: for example, `hug-over-p31` gives T2/T0 0.66
   against Free/T0 0.55.
4. **The failures:**
   - **Small-block dispatch, the same pattern as CI:** the bypassed edge files.
   - **Statistical power on wide intervals:** several failing rows have upper bounds
     under their limit.
   - **One control:** mixed checkpoints, T2/T0 1.017 [1.001, 1.033]. Its A/A row is
     unusually wide (1.019 [0.983, 1.056]), so noise on this host is part of the
     explanation, but the row fails as registered.
5. **The layout tax (T1/T0) is large and sign-flipping on this host:** +5% plain and −5%
   checkpoints. So T2/T0 here carries a ±5% code-layout component that P74 doesn't cause.

**Artifacts in this folder:** `summary.json`, `verdict.json`, `environment.json`,
`asm-audit.json`, `run.log`, `stat-*.txt`, `raw.tsv.gz` and `binary.sha256`.
