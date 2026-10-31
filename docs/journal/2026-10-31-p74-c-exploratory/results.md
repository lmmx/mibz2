# P74-C: one exploratory Codex Cloud run

**Observed fitted Wikidata T2 time is 23.62% lower in plain mode and 26.54% lower with checkpoints. However, CPU 0 steal time was 6.71%, exceeding the predeclared 1% host-interference threshold: this exploratory run is INCONCLUSIVE as performance evidence. Random checkpoint also does not establish the registered 3% regression bound.**

Run code commit: `4cd54371b5efffcd45fa1b7bd8672f244f203b5c`; working tree was clean. The exception was committed before timing; the strict qualification protocol remains unchanged. Existing production and P74 kernels and policy were not modified. Authorizations and exact command are in [authorization.md](authorization.md).

CPU: AMD EPYC 9V74, Zen 4, KVM, pinned to exposed CPU 0; three exposed cores/one thread per core, two-CPU quota. rustc 1.90.0 / LLVM 20.1.8, existing release fat LTO/one codegen unit, no RUSTFLAGS. Binary SHA-256: `e770d1581fc80b404add689a430e44ee66f6c0349f7935481dade0c239dbe313`. Corpus paths/hashes and full host information: [environment.json](environment.json); exact frozen decoded hashes and coverage: [clean manifest](../2026-10-31-p74-c-validation/clean-preparation-manifest.json).

Exactly one measured invocation; two warm-ups/arm, 12 balanced randomized paired rounds, two corpus passes/observation, both variants, all five frozen corpora. All 360 rows retained, no selective reruns. Final output bytes, counts, checkpoint arrays and libbzip2/independent decode checks passed. [Raw TSV](raw.tsv), [frozen-analysis summary](summary.json), [stderr](stderr.txt), [capture script](capture-once.py). No row was discarded.

All MB/s values below are mean per-round decimal throughput using MTF-stage L bytes. Time ratios are mean within-round ratios; their nominal two-sided 95% Student t intervals use 11 df. Percentages describe time changes, not throughput changes. T1 is forced shipped combined mode, T0 standalone shipped, T2 frozen adaptive mode.

| Corpus | Variant | T0 MB/s | T1 MB/s | T2 MB/s | T2/T0 [95% CI] | T1/T0 [95% CI] |
|---|---|---:|---:|---:|---|---|
| wikidata-fitted | plain | 1679.18 | 1659.00 | 2199.61 | 0.7638 [0.7510, 0.7767] | 1.0123 [0.9920, 1.0325] |
| wikidata-fitted | checkpoint | 1580.90 | 1634.57 | 2152.01 | 0.7346 [0.7211, 0.7480] | 0.9673 [0.9484, 0.9862] |
| random | plain | 109.77 | 112.49 | 112.67 | 0.9754 [0.9426, 1.0082] | 0.9778 [0.9446, 1.0110] |
| random | checkpoint | 112.64 | 108.98 | 110.74 | 1.0178 [0.9887, 1.0468] | 1.0344 [0.9962, 1.0726] |
| records | plain | 95.62 | 95.07 | 97.92 | 0.9767 [0.9543, 0.9990] | 1.0075 [0.9774, 1.0377] |
| records | checkpoint | 102.51 | 100.85 | 103.22 | 0.9940 [0.9693, 1.0187] | 1.0173 [0.9919, 1.0427] |
| mixed-independent | plain | 126.27 | 135.71 | 127.25 | 1.0012 [0.9302, 1.0722] | 0.9339 [0.8649, 1.0028] |
| mixed-independent | checkpoint | 134.46 | 135.47 | 135.61 | 0.9922 [0.9559, 1.0284] | 0.9935 [0.9577, 1.0293] |
| text | plain | 552.99 | 554.94 | 670.33 | 0.8287 [0.7822, 0.8751] | 0.9989 [0.9412, 1.0566] |
| text | checkpoint | 556.01 | 570.24 | 665.01 | 0.8364 [0.8159, 0.8568] | 0.9755 [0.9494, 1.0015] |

## Interpretation

- **Primary checkpoint Wikidata:** T2/T0=0.73458 [0.72113,0.74803], or 26.54% less stage time [25.20%,27.89%]. All 12 round ratios favor T2. Its upper endpoint is below 0.95. The lower saving bound 25.20% exceeds twice the largest T1/T0 endpoint departure from 1 (2×5.16%=10.32%): the inherited primary economic conditions are descriptively met. This remains exploratory, not qualification.
- **Plain Wikidata:** T2/T0=0.76382 [0.75096,0.77668], or 23.62% less stage time [22.33%,24.90%]. All 12 round ratios favor T2.
- **Refactor/layout control:** checkpoint T1 is 3.27% faster than T0 [1.38%,5.16%]; plain T1 is 1.23% slower with an interval spanning no change. Thus no evidence here of a large combined-kernel tax on Wikidata. T2/T1 is 0.75987 checkpoint (24.01% less stage time) and 0.75530 plain (24.47% less). T1 does not account for the whole observed benefit.
- **Random:** plain mean is 2.46% faster with an interval spanning 1; its upper ratio endpoint 1.00820 meets the 1.03 criterion descriptively. Checkpoint mean is 1.78% slower, but its interval [0.98870,1.04683] spans no change and permits >3% regression. That row fails acceptance because the bound is not established; it does not prove a real 4.68% or even 1.78% regression.
- **Records:** plain mean is 2.33% faster, with upper ratio 0.99903; checkpoint mean is 0.60% faster, interval spanning 1, upper ratio 1.01870. Both meet the 1.03 criterion descriptively. All 20 random/records blocks use the untouched shipped kernel under T2, as the pre-run audit established. Their T0/T2 instruction-level kernel work is the same apart from dispatch context; the measurements do not establish zero overhead or exclude layout/noise effects.
- **Mixed control:** plain is inconclusive and does not establish the 3% bound (upper ratio 1.07224); checkpoint meets it (1.02843). Text means are 17.13% less plain time and 16.36% less checkpoint time; descriptive secondary observations only. Neither is independent Wikidata validation.
- **Overall:** promising observed fitted-Wikidata signal, but the host-interference gate fails and control acceptance is incomplete. No shipping or out-of-sample claim, and no numerical whole-decoder speedup. A controlled host must validate the stage/control bounds before proposing a separately authorized 1/2/4-physical-core decoder experiment with wall throughput, CPU-seconds/GB and RSS. No such run was performed.

## Noise and diagnostics

Whole invocation wall time 62.757443 s includes decoding/reference preparation, warm-ups and correctness comparisons, not only timed MTF. cgroup usage increased 63.261126 CPU seconds; nr_periods +627, nr_throttled +0, throttled_usec +0. Pinned CPU 0 exposed steal share: 6.707026%. The predeclared steal-time flag TRIGGERED (6.707026% > 1%): mark the run INCONCLUSIVE regardless of its nominal intervals. The throttling flags did not trigger. [Before](environment-before.json), [after and deltas](quality.json) preserve the evidence.

Pressure metrics are unavailable. Zero cgroup throttling is not proof of isolation or stable frequency; exposed steal positively identifies interference. The VM has shared-host/KVM scheduling and no dedicated physical-core guarantee. Several control intervals are wide; their uncertainty is retained. Multiple nominal comparisons and temporal correlation limit confidence interpretation. No additional counters, assembly-led investigation, outlier exclusion or second run was attempted.

A diagnostics preflight first found /proc/pressure/cpu unavailable and exited before launching the benchmark or creating raw.tsv. It was changed to record unavailable for optional metrics. This was not a timing attempt or selective rerun. Required cpu.stat/quota collection succeeded.

## Checks and stopping boundary

Frozen compressed and decoded SHA-256 hashes were verified for all five existing files; no corpus was regenerated. Rust formatting, release build and harness Clippy passed before execution. The full/focused kernel suites were already passed and kernel sources are unchanged; no unrelated suite was rerun. Raw completeness and paired analysis use unchanged bench/p74_experiment.py. Kernel/selector/assembly were not adjusted after results. No CI dispatch, i9 or multicore experiment occurred. Stop after publishing these records.
