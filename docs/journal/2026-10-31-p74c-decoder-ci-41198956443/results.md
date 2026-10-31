# P74-C decoder trial, harness v1.1 validation: CI run 41198956443 (exploratory; w = 1, 2)

**The run (measured):**
- Head `3870c12`, pin `e3f0940`: harness v1.1, with `text` built from the v1.0 tree.
- Three `ubuntu-24.04` jobs, each with 2 physical cores and SMT siblings idle (busy
  ≤ 0.4%), and steal 0.
- The binary SHA-256 is `c72c09fd…`, the same in all three jobs.

**Validation of harness v1.1** (all three jobs):
- They completed `--check`, the structural audit and the timing.
- `all_sha_ok`, `all_workers_plan`, `all_threads_within_allocation` and
  `selections_match_arm` are true.
- A/A PASS: 0, 0 and 3 of 28 rows exclude 1, against a threshold of 4.
- A preceding local dry run up to timing on the 4-core preparation VM passed every gate. In
  it, `--check` counted exactly w + 3 threads in all 272 decodes (34 corpora × 2 arms ×
  w = 1–4).

## Figures (measured): B/A, the geometric mean of the paired ratios with its 95% interval

**Primary corpus** (different-dump Wikidata):

| Job | CPU | w | X B/A | CPU-s/GB B/A | RSS max rule |
|---|---|--:|---|---|--:|
| r1 | EPYC 7763 (Zen 3) | 1 | 1.047 [1.041, 1.054] | 0.959 [0.955, 0.964] | 0.998 |
| r1 | EPYC 7763 (Zen 3) | 2 | 1.036 [1.025, 1.047] | 0.965 [0.958, 0.972] | 1.005 |
| r2 | Xeon Platinum 8573C | 1 | 1.053 [1.044, 1.063] | 0.955 [0.947, 0.964] | 0.996 |
| r2 | Xeon Platinum 8573C | 2 | 1.040 [1.026, 1.053] | 0.956 [0.946, 0.967] | 0.999 |
| r3 | EPYC 9V74 (Zen 4) | 1 | 1.057 [1.039, 1.076] | 0.953 [0.937, 0.969] | 0.999 |
| r3 | EPYC 9V74 (Zen 4) | 2 | 1.023 [0.999, 1.048] | 0.967 [0.946, 0.990] | 1.003 |

**The other Wikidata corpora** (fitted and same-dump held-out): +2.4% to +6.3% on every
host and w, with CPU-s/GB −3% to −5.5%.

**Controls** (X B/A):
- random and records are within [0.990, 1.011], and every interval includes or nearly
  includes 1.
- text gains +2% to +9%.
- mixed-independent ranges from 0.950 to 1.009, and its A′/A null moves by as much
  (0.948–1.006).

**Registered rules at w = 1, 2** (descriptive; this host has no w = 4):
- controls INCONCLUSIVE in every job, from mixed-independent;
- RSS NEGATIVE in every job, on:
  - mixed-independent (all jobs);
  - text w2 (r1) and text w1 (r2).

## Reading (deduced)

1. **The decoder-level gain replicates on three CPU families**, including the first Intel
   server core in these runs:
   - **throughput:** +4.7% to +5.7% at w = 1 and +2.3% to +4.0% at w = 2;
   - **CPU-s/GB:** falls 3.3% to 4.7%.
   - **Projection:** on 2 physical cores it shrinks with w, so the projection for the i9
     at w = 4 stays about +1.5% to +3% (model). That straddles the registered 1.02 bar.
2. **The RSS rule is the likeliest registered failure on the i9,** and it reflects a real
   effect, not noise.
   - On the small controls (mixed-independent, 2 MB in 3 blocks; text, 12 MB in 13
     blocks; processes of about 20–40 MB), B's peak RSS is 2–8% higher in the paired mean.
   - **Likely mechanism:** a faster MTF means more decoded blocks sit in the decoder's
     output queue (at most 2w blocks) before the reader drains them.
   - The increase is a few MB, bounded by the queue depth, not by input size. On the
     Wikidata corpora (≈ 250 MB processes) RSS B/A is 0.990–1.005.
   - The P74-C kernel allocates nothing.
   - The rule stays as registered. A NEGATIVE there would need this reading beside it.
