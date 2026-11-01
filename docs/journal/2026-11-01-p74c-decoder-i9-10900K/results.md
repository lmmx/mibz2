# P74-C full-decoder trial: the registered run on the i9-10900K (harness v1.2)

**The run (measured):**
- `just bench-p74c-decoder i9-10900K`, run by the maintainer at head
  `33e0618b1e70190f4e8390eb69ebe2778142fc77` (pin `b406a3b`; amendments 1 and 2).
- rustc 1.90.0. The binary's SHA-256 is `b0a5affe…cdbd14` (kept as `p74c_decoder.gz`).
- The timed trial took 903 s.
- **CPUs:** 1–4 (four physical cores) allocated; their siblings 11–14 were 0.5–0.9% busy
  over the trial; steal 0.
- **Clock:** governor `powersave` (intel_pstate), turbo enabled.
- **Before timing, every check passed:**
  - the 25 sources and 34 corpus hashes;
  - `bzip2 -t` and `-dc`;
  - `--check` at w = 1–4;
  - the selector audit;
  - the P74-C tests;
  - the structural audit;
  - the synthetic self-test.
- **During the trial:** `stderr.txt` was empty, and end snapshots held 0 exited records.
  The lifecycle race didn't occur on this run.
- **The environment snapshot,** taken before the self-test and the trial, lists a browser
  process. The `/proc/stat` figures above, taken over the trial itself, show the allocated
  cores' siblings idle. The other cores' load isn't recorded per CPU here. The committed
  `environment.json` keeps PID, PSR and %CPU only.

## Registered verdict (§8, mechanical): **NEGATIVE**

| Gate | Outcome | Deciding rows |
|---|---|---|
| 0 Correctness | pass | every output hash, every plan `Workers(w)`, every thread within the allocation, the selections matching each arm |
| 1 Primary: wikidata-20261026, w = 4 | **INCONCLUSIVE** | B/A **1.036 [1.012, 1.060]**: lower < 1.02 and upper ≥ 1.02. Not harmful (lower > 1.00). |
| 2 Secondary flags | none raised | fitted 1.013 [1.004, 1.022]; held-out 1.019 [1.006, 1.032] at w = 4 |
| 3 Controls, every w | **INCONCLUSIVE** | `random` w4 0.987 [0.969, 1.005] and `records` w4 0.972 [0.934, 1.012]: lower < 0.97, upper ≥ 0.97. Every other row passes. |
| 4 CPU-s/GB, w = 4 | **INCONCLUSIVE** | `random` 1.001 and `records` 1.006: the point is above 1.00. Wikidata 0.977–0.984 and text 0.966 pass. |
| 5 Peak RSS | **NEGATIVE** | `mixed-independent` w2: the paired lower bound is above 1.01 (1.030 [1.021, 1.039]), while the max rule reads 0.970. INCONCLUSIVE: `random` w4 (max rule 1.035) and `text` w3 (1.028). |
| 6 A/A | pass | 4 of 56 rows exclude 1, against a threshold of 7. One of them is the primary's own A′/A at w4: 1.025 [1.005, 1.044]. |

**Under the stop rule:** P74-C is archived at decoder level. There is no rerun, no tuning
and no threshold change.

## Figures (measured): B/A throughput and CPU-s/GB, by w

| Corpus | w1 | w2 | w3 | w4 |
|---|---|---|---|---|
| wikidata-20261026 (primary) X | 1.043 [1.034, 1.051] | 1.017 [0.994, 1.040] | 1.022 [1.010, 1.034] | **1.036 [1.012, 1.060]** |
| wikidata-20261026 CPU-s/GB | 0.964 | 0.988 | 0.973 | 0.984 [0.967, 1.001] |
| wikidata-fitted X | 1.032 | 1.021 | 1.018 | 1.013 |
| wikidata-heldout X | 1.031 | 1.011 | 1.018 | 1.019 |
| text X | 1.057 | 1.056 | 1.031 | 1.012 |
| random X | 0.995 | 1.003 | 0.999 | 0.987 |
| records X | 1.007 | 1.000 | 1.000 | 0.972 |
| mixed-independent X | 0.983 | 0.995 | 1.005 | 1.010 |

**Adversarial suite** (108 rows, descriptive): B/A from 0.972 (`split-q05` w3) to 1.158.

## Reading (deduced; not a registered result)

1. **The economic outcome would have been INCONCLUSIVE regardless of RSS.**
   - The primary gained +3.6% at w = 4 (lower bound +1.2%), within the pre-run
     model's +1.5% to +3% band.
   - It didn't clear the registered +2% lower bound.
   - That row's own A/A null moved +2.5%, so noise at that cell is comparable to the
     effect.
2. **What made it NEGATIVE:** the RSS rule on `mixed-independent` at w = 2, by the
   paired test (B 3% higher), even though B's maximum was below A's.
   - The mechanism was recorded before this run in
     `2026-10-31-p74c-decoder-rss-note.md`: a 32-byte `ll` reservation mismatch between
     the shipped and P74-C kernels, which re-grows a worker's buffer.
   - The verdict stands as registered. Any production fix (matching reservations) would
     be a separate, pre-registered change.
3. **CPU efficiency:** CPU-s/GB is 1.6–2.3% lower on Wikidata and 3.4% lower on text at
   w = 4. The gate fails only on the `random` and `records` points (+0.05% and +0.6%,
   within their intervals).
4. **Consistent with the exploratory CI runs,** which gave +2.3% to +5.7% at w ≤ 2 on AMD
   and Intel, and with the stage measurement (an i9 plain-MTF stage ratio of 0.819).

**Postmortem (read-only, descriptive):** `postmortem.md`. The engineering lesson from the RSS NEGATIVE: `../2026-11-01-buffer-capacity-contract.md`.
