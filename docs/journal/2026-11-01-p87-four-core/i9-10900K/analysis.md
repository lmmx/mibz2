# P87 four-core diagnostic: the i9-10900K run

**Run:** 1 November 2026, label `i9-10900K`, pin `6463e44` (HEAD `045808a`), CPUs 1–4, the
siblings idle. The runner's own output is `run.log`; the frozen analysis's output is
`trial.json` and `verdict.json`.

**Reproduced.** The frozen `bench/p87.py` at the pin, run again on `raw.tsv` here, gives a
`trial.json` and `verdict.json` identical to the run's. The binaries match `binaries.sha256`.

**In short.** P87 is a credible optimisation candidate, not yet promotable. The capture is
valid. On the primary Wikidata workload at four workers, the diagnostic binary is measurably
faster and cheaper with B: throughput +1.35% [+0.29%, +2.41%], CPU per GB −1.71%
[−2.37%, −1.06%]. Both registered decisions stay as emitted. The mechanism is INCONCLUSIVE:
the concurrency-specific saving is positive, but its interval crosses the 0.10 materiality
threshold. Engineering is NOT DEMONSTRATED: five no-harm controls are inconclusive, and none
shows harm. A detected primary improvement and a failed promotion gate are not
contradictory. This is a valid experimental result, not a production decision (§5). The
guardian's and the independent researcher's reviews of this run agree with that reading.

**As emitted** (`verdict.json`, unedited):

```json
"decisions": {
 "mechanism": "INCONCLUSIVE",
 "engineering": "NOT DEMONSTRATED",
 "practical": false,
 "matrix": "Neither decision reached: nothing changes; the record says which interval crossed which bound."
}
```

## 1. Validity

Every precondition held:

| Check | Result |
| --- | --- |
| Sources against the frozen hashes; HEAD against the pin | match (25 sources) |
| Tests: arms, harness, trace, analysis | pass |
| E1 one compiled path; E2 shipped kernels; E3 default build is the base's | pass (`code-audit.json`, `same-code.json`: 104 functions, 32,298 instructions) |
| `--check`: 3 corpora, arms A and B and the shipped path, w = 1 and 4, sequential | all frozen hashes |
| S1 correctness: every output's SHA-256, plan `Workers(w)`, thread inventory, affinity | all true (378 rows) |
| Placement balance | 10 / 10 per arm and cell |
| A/A calibration | 1 of 12 rows excludes 1 (k* = 3); D_AA = +0.002 [−0.024, +0.028] contains 0 |
| Instructions B/A over whole decodes (S3) | 0.99995 at w = 1 (limit 0.5%), 0.99998 at w = 4 |
| S2 trace perturbation, median full/off | CPU 1.014–1.024, wall 1.000–1.017 (limit 3%): interpretable |
| S2 marks | complete, in order; 434 blocks, rows and blocks identical in both arms |
| Counters | IMC and core both available; nothing replaced |
| Host during S1 | other CPUs ≤ 0.2% busy, idle siblings ≤ 0.03%, no steal |

S1 took 164 s.

## 2. The registered decisions

| Decision | Outcome | Why |
| --- | --- | --- |
| Mechanism | **INCONCLUSIVE** | D = **+0.078** CPU-s/GB, 95% [+0.042, +0.114]: the lower bound is below 0.10, the upper above it |
| Engineering | **NOT DEMONSTRATED** | The primary at w = 4 meets its condition, but five no-harm rows are INCONCLUSIVE (none NEGATIVE), and IMPROVES needs all of them to PASS |
| Practical flag | false | throughput lower95 1.003 < 1.02 |
| Outcome matrix | (INCONCLUSIVE, NOT DEMONSTRATED) | "Neither decision reached: nothing changes" |

These stand as registered. Nothing below changes them.

### The rows behind them (S1, 20 rounds, ρ₁-corrected 95% intervals)

| Corpus | w | CPU-s/GB A → B | CPU B/A | Throughput B/A | No-harm |
| --- | ---: | --- | --- | --- | --- |
| wikidata-20261026 | 4 | 3.843 → 3.777 | **0.983** [0.976, 0.989] | **1.013** [1.003, 1.024] | (primary) |
| wikidata-20261026 | 1 | 2.842 → 2.854 | 1.004 [0.994, 1.014] | 1.002 [0.994, 1.011] | INCONCLUSIVE |
| records | 4 | 19.47 → 19.35 | 0.994 [0.989, 0.999] | 0.990 [0.962, 1.019] | INCONCLUSIVE |
| records | 1 | 16.99 → 17.01 | 1.001 [0.989, 1.013] | 0.999 [0.986, 1.012] | INCONCLUSIVE |
| random | 4 | 18.28 → 18.27 | 0.999 [0.993, 1.006] | 1.010 [0.971, 1.049] | INCONCLUSIVE |
| random | 1 | 15.99 → 16.03 | 1.003 [0.988, 1.018] | 0.995 [0.979, 1.012] | INCONCLUSIVE |

RSS passes in every cell (max rule ≤ 1.004).

**Why the controls are inconclusive.** Every control's interval contains 1.00, and none
shows harm. The no-harm band, ±1%, is narrower than the precision those corpora reach in 20
rounds: they are 8–10 MB, so each child decodes for tens of milliseconds and the throughput
intervals are ±1.3–4%. That limit was foreseeable from the walk trial's i9 rows (the
`records` CPU-s/GB varies about 1.8% between rounds there), and the protocol didn't size the
controls or the band for it. It is a design limitation of this diagnostic, recorded here
rather than worked around.

## 3. What the run shows beyond the decisions (descriptive)

These figures are not gates. They describe the run, and the protocol's cautions apply:
S2 and S3 come from the phase instrument, not the timed binary, and S3's counters are
machine-wide.

### Where B's saving appears

| | A − B, CPU-s/GB |
| --- | --- |
| Δ1 = C(A,1) − C(B,1) | −0.012 [−0.041, +0.016] |
| Δ4 = C(A,4) − C(B,4) | **+0.066** [+0.040, +0.091] |
| D = Δ4 − Δ1 | **+0.078** [+0.042, +0.114] |

B saves nothing measurable with one worker and about 0.066 CPU-s/GB with four. D's interval
excludes zero, but its size relative to the registered 0.10 is undetermined.

### By step (S2, 6 repetitions, thread CPU per decoded GB)

| Step | w = 1, A − B | w = 4: A → B | w = 4, A − B |
| --- | --- | --- | --- |
| MTF | −0.002 [−0.004, +0.001] | 0.501 → 0.491 | +0.010 [+0.006, +0.013] |
| T-vector build | −0.002 [−0.005, +0.002] | 0.589 → 0.572 | +0.017 [+0.008, +0.026] |
| Traversal | +0.001 [−0.039, +0.041] | 1.710 → 1.674 | +0.036 [−0.004, +0.076] |
| Assembly | +0.001 [−0.001, +0.002] | 0.147 → 0.120 | +0.026 [+0.020, +0.033] |
| RLE1 and CRC | −0.001 [−0.002, +0.000] | 0.195 → 0.186 | +0.009 [+0.005, +0.013] |

- **The new mark splits the old interval.** Traversal is 1.162 → 1.710 ns per segmented row
  from one worker to four (A), and assembly 0.041 → 0.147. So assembly grows 3.5× under
  concurrency, against the traversal's 1.47×. It is still a small share of the step.
- **What is resolved and what isn't.** At four workers, the assembly saving is clearly
  resolved: 0.147 → 0.120, interval +0.020 to +0.033. The traversal estimate, +0.036, is
  positive but its interval, −0.004 to +0.076, includes zero. So P87 measurably improved the
  whole four-worker decoder and its assembly step. It has not independently been shown to
  speed up the dependent traversal, though the difference is in the expected direction.
- **At four workers, B is cheaper in every step, including MTF and the table build.** Those
  run before the assembly they could be affected by, within the same block. A saving there
  can only come indirectly, from other workers or from cache state left by earlier blocks,
  and at one worker there is none. Following the guardian's caution, this is evidence of
  an indirect, concurrency-specific effect. It is not by itself proof of cross-worker cache
  interference: changed scheduling or step overlap could contribute.
- These are separately instrumented measurements. Their sum (0.098, thread CPU) is not a
  decomposition of S1's uninstrumented Δ4 (0.066, process CPU), and isn't presented as one.

### DRAM traffic (S3, 6 repetitions, bytes per decoded byte, corrected; see §4)

| | A | B | B − A | B/A |
| --- | ---: | ---: | --- | ---: |
| w = 1, reads | 0.381 | 0.363 | −0.018 [−0.120, +0.084] | 0.95 |
| w = 1, writes | 0.271 | 0.250 | −0.021 [−0.059, +0.017] | 0.92 |
| w = 4, reads | **9.008** | 8.110 | **−0.898** [−1.139, −0.657] | 0.90 |
| w = 4, writes | **6.686** | 5.815 | **−0.871** [−0.971, −0.770] | 0.87 |

- **The decoder's DRAM traffic per decoded byte is about 24 times higher with four workers
  than with one,** in both arms. This is the largest single number the investigation has
  produced. With one worker, nearly all the block's working data stays on chip. With four,
  most of what a worker writes is evidently written back to DRAM and read again.
- **B removes about 0.9 bytes read and 0.9 bytes written per decoded byte at four workers,
  and nothing measurable at one.** That is about one block-sized buffer's worth of traffic:
  what you'd expect if, with four workers, `rle` is evicted between uses and its lines are
  fetched for ownership and written back every block, and B removes exactly that.
- **An inference, not a measurement:** a worker's working set per full block is roughly
  `ll` 0.9 MB, the T-vector 3.6 MB, the segments about 0.9 MB, `rle` 0.9 MB and its output
  buffer. One such set fits in the i9's 20 MB L3; four don't. The DRAM figures fit that
  picture of capacity crowding. They don't exclude the other explanations in the protocol
  (request congestion in the L3, write effects).

### Frequency

Cycles per reference cycle at w = 4 were 1.341 (A) and 1.321 (B), and at w = 1 1.399 and
1.398. B's 1.8% fewer cycles at four workers were spent at a slightly lower effective clock.
This is reported as an effect of the arm, not a confounder.

## 4. A defect in the frozen analysis's S3 output

`bench/p87.py counters` treated perf's `uncore_imc` values as 64-byte line counts and
multiplied by 64. perf reports them already scaled, in MiB (the unit column in
`counters/*.csv`). So the DRAM figures in `counters.json` and `verdict.json` are
64 / 2²⁰ times the true bytes per decoded byte. Their ratios and signs are right; their
magnitudes are not.

- **Affected:** S3's DRAM figures only, which are descriptive.
- **Not affected:** both decisions, which never read them; the instruction check, which uses
  the unscaled core counters; and every S1 and S2 figure.
- **The parser, corrected (post-capture).** `parse_stat` now keeps perf's unit, and
  `imc_bytes` converts by it: MiB × 2²⁰, a unitless count × 64; any other unit is refused.
- **Two regression tests in `bench/test_p87.py`:** one uses this run's own MiB lines, the
  other covers the unit handling. Both fail on the frozen `p87.py` and pass on the corrected
  one.
- **The replay.** The corrected `counters` over this archive's raw CSVs gives
  `counters-corrected.json`. A verdict over it gives `verdict-corrected-s3.json`: the
  decisions and every non-DRAM figure are identical to `verdict.json`, and the DRAM figures
  are exactly 16,384× the frozen ones. `dram.py`, written separately from the raw CSVs into
  `dram.json`, agrees to the last digit. The §3 table comes from them.
- **The frozen files are left as produced:** `counters.json` and `verdict.json`, and the raw
  perf output in `counters/`.
- **The branch no longer runs.** Its `bench/p87.py` and `bench/test_p87.py` now differ from
  `frozen.json`'s hashes, so the branch's runner will refuse to start. That's intended: the
  code that ran is the pin, `6463e44`, and this label has had its one run.
- **Why the dry run missed it:** the VM's perf had no memory-controller counters, so the
  path never saw a MiB line.

## 5. Where this leaves P87

Under its registered rules, the four-core diagnostic reached neither decision. It doesn't
close P87 negative, and it doesn't license a production change. It is a valid experiment
with a positive primary result and an inconclusive registered decision, not an
unsuccessful one.

The descriptive picture is consistent and one-directional:

- with four workers, reusing `ll`:
  - lowers CPU per GB by 1.7% [1.1, 2.4];
  - raises throughput by 1.3% [0.3, 2.4];
  - cuts DRAM traffic by 10–13%;
- with one worker it does nothing measurable;
- no control shows harm, and memory is unchanged.

The larger finding is the 24-fold rise in DRAM traffic from one worker to four. It bears
on the whole four-worker penalty, not just on P87.

What remains uncertain:
- whether the primary improvement holds up across the project's controls and other
  workloads;
- how much of it comes from interference in the traversal itself, rather than from local or
  persistent effects of the assembly;
- which of the traffic reductions, if any, causes the CPU saving. The counters are
  machine-wide, and a change in how the decoder's steps overlap could also change traffic.

The reviews agree on the order of what comes next. None of it is authorised here.

1. **Preserve and verify this result first.** No i9 rerun, no retuning, and no change to
   the frozen trial. This record, the corrected replay and the regression tests are that
   step.
2. **Any follow-up is a separately registered trial, not a repeat of this one.** If the
   maintainer wants one, the natural candidate is a production-oriented validation: reuse
   `ll` and also drop `rle`, which saves memory too. Its controls should be sized from this
   run's variances so that they can actually meet their no-harm band (larger control
   corpora, or more rounds). Its tolerances must be set before it runs, not loosened after
   seeing these results.
3. **P87's historical bar, at 6–10 workers,** remains separate and unevaluated.
4. **The neighbour experiment isn't the next step.** The guardian's view is that P87's result
   deserves careful interpretation first. It stays relevant, as the independent researcher
   notes: P87's saving is much smaller than the four-worker penalty, and the 24× DRAM figure
   suggests the per-worker footprint as a whole is worth understanding.

## 6. Closure (after the guardian's and the researcher's reviews of this record)

The diagnostic is **closed**. The registered verdict stands as emitted. The corrected traffic
analysis is preserved beside the frozen one. The three readings (the maintainer's from the
i9 output, this replay, and the researcher's audit of the archive) agree on the facts. The
record keeps three layers apart:

**The statistical result (measured).**
- **Primary, four workers:** on Wikidata, B improves throughput by +1.35% [+0.29, +2.41] and
  CPU per GB by −1.71% [−2.37, −1.06].
- **Mechanism:** D = +0.078 CPU-s/GB [+0.042, +0.114]. The interval excludes zero, so a
  positive concurrency interaction is supported, but it crosses the 0.10 materiality
  threshold.
- **The five other no-harm cells:** all inconclusive, and none demonstrates a regression.
  For example, `records` at four workers has throughput 0.990, interval 0.962 to 1.019:
  consistent with a small regression, no effect, or a small improvement.
- **Steps (separately instrumented):** the assembly saving is resolved; the traversal
  saving is not.
- **DRAM at four workers (after the unit correction):** reads −10%, writes −13%.

**The physical interpretation (proposed, not identified).**
- **What the results fit:** changing the assembly destination changes the four-worker
  memory environment. The earlier steps' savings point to cross-worker or persistent
  effects.
- **What they don't show** is that the traffic reduction mediates the CPU saving. Both were
  observed under the intervention, and the counters are machine-wide. "P87 improves
  performance by reducing DRAM traffic" is not a claim this record makes.
- **What remains of the penalty:** the larger inverse-BWT penalty is untouched, and its
  physical cause is not isolated.

**The engineering decision (registered).**
- **No promotion:** engineering is NOT DEMONSTRATED. That is a failure to establish every
  required condition, not evidence of a regression.
- **Diagnostic, not production:** the arms were compared on one diagnostic binary, both
  through the diagnostic path. The +1.35% is a diagnostic A/B improvement, not a measured
  improvement over the shipped decoder.

**Disposition.** P87 is a credible optimisation candidate with a genuine positive primary
result. It is not yet a production-ready optimisation.
- **Next:** a production-shaped validation, under its own protocol registered before it
  runs. Its controls must be powered for their gates, and its tolerances must not be
  loosened because the Wikidata result was encouraging.
- **Still open:** P87's historical bar at 6–10 workers, unevaluated.
- **Preserved:** the calibrated neighbour experiment, as the broader follow-up.

**The design lesson.** This protocol worked out precision only for the primary comparison,
not for every mandatory criterion. The controls' throughput noise per round was 6–8% at
four workers: decodes of about 50 ms are dominated by fixed per-run jitter. A gate needs
its precision checked when it is designed, not after the run.
