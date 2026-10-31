# P74-C full-decoder trial: exploratory CI run 41196164480 (w = 1, 2; not confirmatory)

**The run (measured):**
- Pin `535296c` (head `4acf46f`), started by a push to `research/p74c-decoder-ci-trigger`.
- Three `ubuntu-24.04` jobs. Each has 4 vCPUs on 2 physical cores; allocation CPUs 0 and 2,
  with siblings 1 and 3 idle (0.3–0.4% busy).
- Binary SHA-256 `e2fc3aaa…c958`, the same in every job. The structural audit passed.
- 20 paired rounds of B, A and A′.
- The protocol's verdict for this host is "NOT APPLICABLE": there is no w = 4 here, so
  nothing below is a registered result.

**The jobs:**
- **r1 and r2** stopped in the untimed `--check`, a harness defect described in
  `../2026-10-31-p74c-decoder/amendment-1.md`. Their `check.tsv` rows up to the stop pass.
- **r3** (AMD EPYC 7763, Zen 3, 745 s of timing) ran in full:
  - every output's SHA-256 matched;
  - every run planned `Workers(w)`;
  - every thread stayed within the allocation;
  - the selections matched each arm;
  - steal was 0.

## r3: B/A, the geometric mean of the paired ratios with its 95% interval (measured)

Columns: throughput X B/A, CPU-s/GB B/A, peak-RSS max rule, the A′/A null on X, and A's
throughput.

| Corpus | w | X B/A | CPU B/A | RSS max rule | X A′/A | A MB/s |
|---|--:|---|---|--:|---|--:|
| **wikidata-20261026 (primary)** | 1 | **1.053** [1.047, 1.059] | 0.953 [0.948, 0.958] | 0.997 | 1.008 [0.999, 1.018] | 223 |
| **wikidata-20261026 (primary)** | 2 | **1.036** [1.019, 1.054] | 0.966 [0.955, 0.978] | 0.999 | 1.006 [0.999, 1.013] | 355 |
| wikidata-fitted | 1 | 1.046 [1.038, 1.054] | 0.961 [0.954, 0.967] | 1.001 | 0.995 [0.988, 1.001] | 228 |
| wikidata-fitted | 2 | 1.033 [1.021, 1.046] | 0.965 [0.954, 0.976] | 1.002 | 0.997 [0.982, 1.013] | 361 |
| wikidata-heldout | 1 | 1.041 [1.035, 1.047] | 0.965 [0.960, 0.970] | 1.002 | 0.998 [0.991, 1.004] | 237 |
| wikidata-heldout | 2 | 1.031 [1.018, 1.044] | 0.970 [0.963, 0.977] | 1.002 | 1.004 [0.995, 1.014] | 371 |
| random | 1 | 1.005 [0.997, 1.013] | 0.997 | 1.000 | 1.003 | 62 |
| random | 2 | 1.013 [0.995, 1.032] | 0.996 | 1.000 | 1.006 | 82 |
| records | 1 | 0.999 [0.992, 1.006] | 0.999 | 1.000 | 1.001 | 56 |
| records | 2 | 0.989 [0.970, 1.009] | 1.005 | 1.002 | 1.000 | 79 |
| mixed-independent | 1 | 0.987 [0.972, 1.002] | 1.011 | 0.968 | 0.999 [0.978, 1.020] | 55 |
| mixed-independent | 2 | 0.954 [0.910, 1.001] | 1.020 | **1.067** | **0.959** [0.901, 1.021] | 72 |
| text | 1 | 1.079 [1.066, 1.092] | 0.940 | 1.000 | 1.007 | 140 |
| text | 2 | 1.061 [1.043, 1.080] | 0.944 | 0.999 | 1.006 | 190 |

**Adversarial rows (54):**
- X B/A ranges from 0.996 to 1.131.
- The lowest are `edge-l1-s50000` w2 at 0.996 [0.980, 1.013], `split-q05` w2 at 0.999 and
  `edge-l9-s16383` w2 at 1.000.

**The registered rules at w = 1 and 2** (from `verdict.json`; descriptive only):
- **Controls:** INCONCLUSIVE, from `records` w2 and `mixed-independent` w2.
- **RSS:** NEGATIVE on `mixed-independent` w2 (the paired mean 1.040 [1.026, 1.055]).
- **A/A:** PASS (1 of 28 rows excludes 1).

## Reading (deduced; not a registered result)

1. **The decoder-level gain is real on Zen 3, and it shrinks with w.**
   - Primary +5.3% at w = 1 and +3.6% at w = 2. Every Wikidata row's lower bound is above
     1.018. CPU-s/GB falls 3–5%.
   - At w = 2 the throughput gain equals the process CPU saving (1/0.966 = 1.035 against
     1.036).
   - With w workers plus the scanner, feeder and main on 2 physical cores, the decoder
     is CPU-bound as a whole. So the gain is the MTF share of all CPU, not of a worker's
     time.
2. **The implied MTF share** is s = (1 − 1/R)/(1 − r_plain), with Zen 3's
   r_plain = 0.719 (the model in `summary.json`). It is 0.179 at w = 1 and 0.124 at w = 2
   on the primary corpus.
3. **What this projects for the i9 at w = 4 (model, two bounds):**
   - (a) If the trend in s with w continues (×0.69 per doubling, s ≈ 0.085): R ≈
     1/(1 − 0.085·0.181) ≈ **1.016**.
   - (b) If the i9 at w = 4 is CPU-bound like this host at w = 2, with Zen 3's MTF CPU
     share (0.167) scaled by the i9's stage gain (0.181 against 0.281): R ≈ **1.03**.
   - So the prediction is about **+1.5% to +3%**, which straddles the registered 1.02 bar.
     **INCONCLUSIVE is a realistic outcome**, and NEGATIVE (upper < 1.02) is possible.
     That agrees with the protocol's own +1.6% figure.
4. **mixed-independent w2 is the noisy row, not a measured cost.**
   - Its A′/A null moves as much as B/A (X 0.959 against 0.954), and its RSS max rule is
     1.033 for A′/A against 1.067 for B/A.
   - The file is 2 MB in 3 blocks, so peak RSS (≈ 27 MB) depends on whether two blocks'
     buffers overlap in time at w = 2. The P74-C kernel allocates nothing.
   - On the i9 the same corpus at w = 4 can fail the RSS rule the same way. The rule is
     registered and stays as it is. A failure there would need this reading next to it.
5. **The bypass protects as at stage level:**
   - random and records are within ±1.3%;
   - text, with every block in P74 to its end, gains 6–8%;
   - the edge files are at 0.996–1.000.
