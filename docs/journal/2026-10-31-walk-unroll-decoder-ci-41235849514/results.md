# The unrolled room walk's full-decoder trial: exploratory CI run 41235849514 (w = 1, 2)

**The run (measured):**
- Head `14f731a` (pin `f7ac01f`, amendment 1), started by
  `research/walk-unroll-decoder-ci-trigger` (`93ef5d7`).
- Three `ubuntu-24.04` jobs, rustc 1.97.0.
- The binary's SHA-256 is `a5268c5e…7885e`, identical in all three. It is kept here as
  `walk_decoder.gz`.
- In every job, the pre-timing checks passed (hashes, correctness, the arm's tests, W1–W4
  (b)). W4 (c)'s signature matched the recorded acceptance.

| Job | CPU | Outcome |
|---|---|---|
| r1 | AMD EPYC 7763 (Zen 3) | **aborted in the timed trial** after 178 rows (the warm-ups and about 1.5 rounds): a harness defect (below). Not a result. The partial rows are kept as `raw-partial-aborted.tsv.gz`, unanalysed. |
| r2 | AMD EPYC 9V74 (Zen 4) | **aborted in the synthetic self-test**, before timing, from the same defect |
| r3 | Intel Xeon 6975P-C | **completed:** 20 rounds × 11 corpora × w = 1, 2 |

## The harness defect (measured; not an affinity escape)

**What failed:** `thread |||| outside the allocation`, from `assert_confined` on a child's
**end** snapshot.

**What the snapshot is:** it is taken after `dec.join()`, and lists `/proc/self/task`.

**Why it can catch a thread mid-release:**
- In r3's completed rows, the end snapshot still lists a joined decoder thread
  (`mibz2-scan`, `mibz2-0` or `mibz2-1`) in 79 of 1,386 children. In r1, the figure is 12 of
  178.
- A joined thread can stay listed until the kernel releases it, like the feeder in the
  P74-C trial's amendment 1.
- When the snapshot reads such a task mid-release, its `status` is already gone, so the
  record is empty (`||||`), and the check reads an empty `Cpus_allowed_list` as outside
  the allocation.

**The snapshot taken at spawn,** with every thread alive, was never empty.

**The same code is in the frozen P74-C harness** (v1.1, `research/p74c-decoder`
`3870c12`). Its CI validation (41198956443) completed without hitting this; the defect is
latent there.

## r3: one complete job (exploratory, Intel Xeon 6975P-C, 2 physical cores)

**Host:** CPUs 0 and 1 allocated, and their siblings 2 and 3 at 0.2% busy. Steal under
0.01%.

**Checks:** correctness, plan and confinement true. A/A: 0 of 44 rows exclude 1.

Throughput (X) B/A, then CPU-s/GB B/A, then the peak-RSS max rule, then the A′/A null on
throughput:

| Corpus | w | X B/A [95%] | CPU B/A [95%] | RSS max rule | X A′/A |
|---|--:|---|---|--:|---|
| wikidata-20261026 (primary) | 1 | 1.002 [0.999, 1.004] | 0.999 [0.997, 1.001] | 1.004 | 0.998 |
| wikidata-20261026 (primary) | 2 | 1.000 [0.988, 1.013] | 0.997 [0.995, 0.999] | 0.997 | 0.994 |
| wikidata-fitted | 1 | 1.006 [1.003, 1.009] | 0.997 [0.995, 0.998] | 0.997 | 1.001 |
| wikidata-fitted | 2 | 1.007 [0.998, 1.017] | 0.997 [0.995, 0.998] | 1.007 | 1.006 |
| wikidata-heldout | 1 | 1.005 [1.003, 1.007] | 0.996 [0.993, 0.999] | 1.001 | 0.999 |
| wikidata-heldout | 2 | 1.004 [0.995, 1.012] | 0.996 [0.995, 0.997] | 0.998 | 1.005 |
| **records** | **1** | **0.928 [0.924, 0.933]** | **1.055 [1.050, 1.060]** | 0.997 | 1.000 [0.996, 1.004] |
| **records** | **2** | **0.963 [0.948, 0.979]** | **1.051 [1.045, 1.057]** | 1.068 | 1.002 |
| random, mixed, text, text-l1, text-l5 | 1, 2 | 0.996 to 1.016 | 0.996 to 1.005 | 0.995 to 1.028 | 0.992 to 1.026 |
| zeros (fallback control) | 1, 2 | 1.007, 0.985 | 0.994, 1.017 | 1.004, 1.005 | 1.017, 0.987 |
| period-3 | 1, 2 | 1.034 [0.947, 1.130], 0.990 [0.888, 1.105] | 0.968, 1.001 | 1.000, 0.955 | 1.018, 0.990 (wide) |

**The registered rules at w = 1 and 2** (descriptive; the overall verdict is "NOT
APPLICABLE": there is no w = 4):
- **Controls: NEGATIVE.** `records` w1 is a demonstrated regression (upper 0.933 < 0.97).
  The other rows not passing (`records` w2, `text-l5` w2, `zeros` w2, `period-3`) are
  INCONCLUSIVE from wide intervals.
- **RSS: INCONCLUSIVE.** The rows are `records` w2, `text` w1 and `text-l5` w1.
- **CPU-s/GB is gated only at w = 4** (none here). It falls 0.1% to 0.4% on Wikidata and
  rises 5% on `records`.

## Reading (deduced; one complete job, on one Intel host)

1. **On this Intel host, the decoder-level gain on Wikidata is about +0.2% to +0.7%**, not
   the +3% to +6% the protocol predicted. CPU-s/GB is 0.1% to 0.4% lower.
   - The stage PASS was measured on AMD: the walk 12–13% faster on Zen 3 and Zen 4.
   - No Intel stage measurement exists. So whether the walk itself gains little on this
     core, or the gain is diluted in the decoder, isn't established.
2. **`records` regresses by 4–7% in throughput, and uses 5% more CPU**, with a tight A/A
   (1.000 [0.996, 1.004]).
   - It is a real, measured cost of the integrated arm on this host.
   - The cause isn't established. Candidates:
     - shorter prefetch lead when rounds run faster;
     - the 32 duplicated cold paths, through I-cache or branch prediction;
     - the W4 (c) layout difference (the accepted confounder).
3. **The AMD hosts, where the stage gain was measured, produced no decoder-level data,
   because of the harness defect.**
4. **None of this decides the trial.** The i9 (Intel, 4 physical cores) is the deciding
   host. This run warns that its primary could be INCONCLUSIVE or NEGATIVE, and that its
   `records` control could fail.
