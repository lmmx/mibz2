# Inbox 6926191785 — i9 plan headroom from archived evidence

Brief: [#160 comment 6926191785](https://github.com/lmmx/mibz2/pull/160#issuecomment-6926191785). Analysis only; no new run, build, timing, plan change or merge.

## Basis and semantics

All 29 archives used below were read from [`evidence` at 14b969f4](https://github.com/lmmx/mibz2/tree/14b969f4ef2d9c8da4c7ac77643babbddc28b6c6). Each compressed bundle matched its adjacent `.sha256`. A table label such as `lazy-place/run-20261116T101100Z` means `evidence/<label>.tar.gz`; each row names the measured arm. The paired wall ratio is arm / that bundle's `mibz2` production arm, with the bundle's reported 95% interval. Lower is faster. **That production arm is contemporaneous, not always the current 0.5.1 plan.** A dash means no paired cell. The hint/ring and forced stop-walk tables compare against forced lazy, marked **L**, because those measurement rosters contain no production arm; cross-bundle division would not be a paired interval. The 10 November and earlier release-scaling baselines are also historical. These are 95% per-cell intervals, not multiplicity-adjusted admission tests.

The latest release-scaling bundle at 09:24 and the placement bundle at 10:11 are after 0.5.1. The latter's source [`examples/ab.rs` at f40eb4cc](https://github.com/lmmx/mibz2/blob/f40eb4cc1d74d24c13f57736db5d443cda8b20c0/examples/ab.rs) states the exact contrast: shipped plan past cores uses `min(requested, cores)` lazy workers but `SLOTS_PER_THREAD × requested` in-flight window; `-cores` keeps ten workers and sizes the window by workers; `-cores1` uses nine workers and a worker-sized window. The 09:24 release-scaling pair gives v0.5.0 / 0.5.1 = 1.079 [1.068,1.090] at 10, 1.066 [1.047,1.085] at 12, 1.084 [1.064,1.104] at 16, 1.084 [1.039,1.132] at 20. Earlier cap gains were incorporated into #468; do not count them again.

## Paired headroom grid

### Forced counts and crossover (Wikidata, contemporaneous production)

| bundle | arm | 8 | 9 | 10 | 12 | 16 | 20 |
|---|---|---|---|---|---|---|---|
| crossover/run-20261115T150151Z | workers | 0.938 [0.919, 0.958] | 0.956 [0.947, 0.967] | 1.123 [1.109, 1.136] | 1.200 [1.176, 1.224] | 1.286 [1.251, 1.323] | 1.406 [1.384, 1.430] |
| crossover/run-20261115T151452Z | workers-w | 0.923 [0.915, 0.930] | 0.964 [0.956, 0.972] | 1.153 [1.131, 1.174] | 1.284 [1.263, 1.306] | 1.402 [1.378, 1.427] | 1.488 [1.444, 1.533] |
| crossover/run-20261115T191448Z | workers-w | 1.001 [0.982, 1.021] | 0.994 [0.978, 1.011] | 1.040 [1.019, 1.061] | 1.187 [1.156, 1.218] | 1.356 [1.323, 1.389] | 1.451 [1.403, 1.500] |
| crossover/run-20261115T191448Z | team2 | 1.188 [1.165, 1.211] | 1.137 [1.119, 1.156] | 1.261 [1.244, 1.278] | 1.431 [1.394, 1.469] | 1.762 [1.726, 1.798] | 2.046 [2.017, 2.076] |
| crossover/run-20261115T191448Z | team3 | 1.361 [1.335, 1.387] | 1.040 [1.020, 1.060] | 1.025 [1.011, 1.039] | 1.111 [1.081, 1.141] | 1.319 [1.279, 1.361] | 1.447 [1.429, 1.466] |

### Lazy crossover, cap, and occupancy (Wikidata, contemporaneous production)

| bundle | arm | 8 | 9 | 10 | 12 | 16 | 20 |
|---|---|---|---|---|---|---|---|
| lazy-crossover/run-20261116T014443Z | workers-eager | 1.035 [1.014, 1.056] | 1.100 [1.073, 1.127] | 1.037 [1.019, 1.056] | 1.184 [1.157, 1.213] | 1.345 [1.313, 1.377] | 1.426 [1.397, 1.455] |
| lazy-crossover/run-20261116T014443Z | workers-lazy | 1.007 [0.982, 1.034] | 0.998 [0.979, 1.018] | 0.931 [0.905, 0.958] | 1.001 [0.974, 1.029] | 1.123 [1.087, 1.160] | 1.194 [1.175, 1.214] |
| lazy-crossover/run-20261116T014443Z | team2 | 1.235 [1.212, 1.259] | 1.261 [1.229, 1.294] | 1.261 [1.245, 1.277] | 1.418 [1.375, 1.463] | 1.759 [1.703, 1.818] | 2.061 [2.011, 2.113] |
| lazy-crossover/run-20261116T014443Z | team3 | 1.425 [1.393, 1.459] | 1.147 [1.120, 1.174] | 1.028 [1.002, 1.056] | 1.109 [1.069, 1.150] | 1.290 [1.264, 1.316] | 1.463 [1.437, 1.490] |
| lazy-crossover/run-20261116T021858Z | workers-eager | 1.029 [1.006, 1.052] | 1.103 [1.079, 1.128] | 1.028 [1.017, 1.038] | 1.177 [1.150, 1.204] | 1.347 [1.312, 1.383] | 1.453 [1.433, 1.474] |
| lazy-crossover/run-20261116T021858Z | workers-lazy | 0.993 [0.978, 1.008] | 1.008 [0.989, 1.027] | 0.917 [0.898, 0.936] | 1.021 [1.001, 1.042] | 1.131 [1.109, 1.154] | 1.214 [1.192, 1.235] |
| lazy-crossover/run-20261116T021858Z | team2 | 1.253 [1.195, 1.314] | 1.263 [1.250, 1.276] | 1.263 [1.246, 1.280] | 1.432 [1.416, 1.448] | 1.766 [1.728, 1.804] | 2.074 [2.041, 2.108] |
| lazy-crossover/run-20261116T021858Z | team3 | 1.437 [1.400, 1.476] | 1.147 [1.131, 1.163] | 1.022 [1.003, 1.041] | 1.129 [1.104, 1.156] | 1.323 [1.304, 1.342] | 1.475 [1.451, 1.500] |
| lazy-crossover/run-20261116T093528Z | workers-eager | 1.041 [1.018, 1.066] | 1.115 [1.097, 1.133] | 1.111 [1.084, 1.138] | 1.274 [1.244, 1.305] | 1.451 [1.426, 1.477] | 1.532 [1.507, 1.557] |
| lazy-crossover/run-20261116T093528Z | workers-lazy | 1.003 [0.981, 1.026] | 1.010 [0.992, 1.029] | 1.005 [0.981, 1.030] | 1.089 [1.064, 1.115] | 1.194 [1.167, 1.222] | 1.265 [1.245, 1.287] |
| lazy-crossover/run-20261116T093528Z | team2 | 1.233 [1.210, 1.256] | 1.274 [1.262, 1.287] | 1.356 [1.322, 1.392] | 1.527 [1.498, 1.557] | 1.859 [1.798, 1.922] | 2.204 [2.150, 2.258] |
| lazy-crossover/run-20261116T093528Z | team3 | 1.440 [1.405, 1.476] | 1.159 [1.137, 1.182] | 1.108 [1.076, 1.142] | 1.214 [1.183, 1.247] | 1.389 [1.360, 1.418] | 1.568 [1.540, 1.597] |
| lazy-cap/run-20261116T024424Z | workers-lazy-cap | — | — | 0.934 [0.914, 0.955] | 0.926 [0.900, 0.953] | 0.945 [0.924, 0.966] | 0.950 [0.929, 0.972] |
| lazy-cap/run-20261116T024424Z | workers-lazy | — | — | 0.931 [0.909, 0.954] | 1.011 [0.971, 1.052] | 1.147 [1.106, 1.189] | 1.226 [1.205, 1.247] |
| lazy-occupancy/run-20261116T032432Z | workers-lazy-cap | 0.989 [0.969, 1.008] | 1.018 [0.993, 1.044] | 1.014 [0.991, 1.038] | — | — | 0.942 [0.917, 0.968] |
| lazy-occupancy/run-20261116T032432Z | workers-lazy | 0.989 [0.971, 1.007] | 1.009 [0.990, 1.028] | 1.009 [0.990, 1.028] | — | — | 1.220 [1.189, 1.251] |

### Occupancy and placement (Wikidata ×8, contemporaneous production)

| bundle | arm | 8 | 9 | 10 | 12 | 16 | 20 |
|---|---|---|---|---|---|---|---|
| lazy-occupancy/run-20261116T032432Z | workers-lazy-cap | 1.000 [0.994, 1.006] | 1.001 [0.995, 1.007] | 1.002 [0.994, 1.011] | — | — | 0.935 [0.923, 0.948] |
| lazy-occupancy/run-20261116T032432Z | workers-lazy | 1.000 [0.993, 1.006] | 1.001 [0.996, 1.007] | 0.999 [0.989, 1.008] | — | — | 1.206 [1.189, 1.224] |
| lazy-place/run-20261116T101100Z | workers-lazy-cores | — | — | 1.001 [0.994, 1.008] | 0.997 [0.989, 1.005] | — | 0.994 [0.988, 1.000] |
| lazy-place/run-20261116T101100Z | workers-lazy-cores1 | — | — | 0.984 [0.979, 0.989] | 0.954 [0.945, 0.964] | — | 0.951 [0.945, 0.957] |
| lazy-place/run-20261116T101100Z | workers-lazy-cores-pin | — | — | 1.005 [1.001, 1.010] | 0.981 [0.975, 0.986] | — | 0.980 [0.971, 0.988] |
| lazy-place/run-20261116T101100Z | workers-lazy-cores1-pin | — | — | 0.987 [0.981, 0.993] | 0.959 [0.953, 0.965] | — | 0.959 [0.950, 0.968] |

### Placement (single Wikidata copy, contemporaneous production)

| bundle | arm | 8 | 9 | 10 | 12 | 16 | 20 |
|---|---|---|---|---|---|---|---|
| lazy-place/run-20261116T101100Z | workers-lazy-cores | — | — | 0.990 [0.966, 1.014] | — | — | 0.936 [0.919, 0.953] |
| lazy-place/run-20261116T101100Z | workers-lazy-cores1 | — | — | 0.971 [0.949, 0.994] | — | — | 0.888 [0.869, 0.906] |
| lazy-place/run-20261116T101100Z | workers-lazy-cores-pin | — | — | 1.005 [0.976, 1.036] | — | — | 0.943 [0.916, 0.971] |
| lazy-place/run-20261116T101100Z | workers-lazy-cores1-pin | — | — | 0.972 [0.945, 1.000] | — | — | 0.914 [0.900, 0.929] |

### Hint and ring (Wikidata, paired against forced lazy L; production absent)

| bundle | arm | 8 | 9 | 10 | 12 | 16 | 20 |
|---|---|---|---|---|---|---|---|
| lazy-hint/run-20261116T021013Z | workers-lazy-hint | 1.006 [0.993, 1.020] | 1.003 [0.987, 1.021] | 0.989 [0.965, 1.013] | — | — | — |
| lazy-ring/run-20261116T025027Z | workers-lazy-ring | 1.030 [1.011, 1.049] | 1.013 [0.989, 1.037] | 1.016 [1.000, 1.033] | — | — | — |

### Stop-walk (Wikidata ×4; judged pair against production, forced pair against lazy L)

| bundle | arm | 8 | 9 | 10 | 12 | 16 | 20 |
|---|---|---|---|---|---|---|---|
| stop-walk/run-20261116T162328Z | stop15 | 1.012 [1.004, 1.019] | 1.010 [1.004, 1.016] | 0.991 [0.980, 1.003] | 0.997 [0.985, 1.010] | — | 1.015 [1.003, 1.027] |
| stop-walk/run-20261116T162328Z | workers-stop14 | 1.030 [1.020, 1.040] | 1.018 [1.012, 1.024] | 1.018 [1.003, 1.033] | 0.982 [0.970, 0.994] | — | 0.951 [0.938, 0.964] |
| stop-walk/run-20261116T162328Z | workers-stop15 | 1.018 [1.011, 1.025] | 1.007 [1.000, 1.014] | 1.004 [0.992, 1.016] | 0.986 [0.978, 0.994] | — | 0.965 [0.957, 0.973] |
| stop-walk/run-20261116T162328Z | workers-stop16 | 1.017 [1.005, 1.029] | 1.008 [1.003, 1.013] | 1.007 [0.996, 1.017] | 0.994 [0.986, 1.001] | — | 0.979 [0.969, 0.989] |

### Release scaling (Wikidata; each release paired to production at that run date)

| bundle | arm | 8 | 9 | 10 | 12 | 16 | 20 |
|---|---|---|---|---|---|---|---|
| release-scaling/run-20261106T233611Z | v0.2.4 | 1.008 [1.000, 1.016] | — | 1.119 [1.101, 1.138] | 1.087 [1.060, 1.114] | 1.081 [1.057, 1.105] | 1.082 [1.058, 1.107] |
| release-scaling/run-20261107T232252Z | v0.2.5 | 0.993 [0.983, 1.004] | — | 1.037 [0.984, 1.093] | 1.046 [1.022, 1.070] | 1.061 [1.029, 1.095] | 1.065 [1.031, 1.101] |
| release-scaling/run-20261110T091259Z | v0.2.6 | 0.988 [0.973, 1.003] | — | 0.984 [0.955, 1.014] | 1.015 [0.971, 1.060] | 0.978 [0.957, 0.999] | 1.003 [0.978, 1.030] |
| release-scaling/run-20261110T095909Z | 8f540ef1190689f59a279f081d8405f46669fba0 | 1.032 [1.009, 1.056] | — | 1.053 [0.987, 1.124] | 1.035 [0.999, 1.072] | 1.055 [0.985, 1.130] | 1.014 [0.953, 1.078] |
| release-scaling/run-20261110T101103Z | 5e7b995471591eacdd5ae32cebfb8bbb578cbff5 | 0.996 [0.984, 1.009] | — | 0.971 [0.886, 1.064] | 0.988 [0.905, 1.080] | 1.017 [0.963, 1.074] | 0.938 [0.881, 0.998] |
| release-scaling/run-20261110T122748Z | v0.2.7 | 0.997 [0.986, 1.007] | — | 0.993 [0.985, 1.001] | 1.003 [0.982, 1.024] | 0.985 [0.954, 1.018] | 0.968 [0.931, 1.007] |
| release-scaling/run-20261110T160612Z | ba52e4d6f05729b6317ac8b7b37e576d79b193d3 | 1.151 [1.139, 1.163] | — | 1.138 [1.123, 1.154] | 1.117 [1.061, 1.176] | 1.108 [1.047, 1.172] | 1.150 [1.099, 1.204] |
| release-scaling/run-20261110T170952Z | v0.2.8 | 1.147 [1.134, 1.160] | — | 1.145 [1.127, 1.164] | 1.111 [1.094, 1.129] | 1.109 [1.086, 1.133] | 1.102 [1.075, 1.130] |
| release-scaling/run-20261114T143213Z | v0.2.9 | 0.998 [0.988, 1.009] | — | 0.996 [0.979, 1.014] | 0.983 [0.966, 1.000] | 0.998 [0.974, 1.022] | 0.984 [0.958, 1.011] |
| release-scaling/run-20261114T231702Z | v0.3.0 | 1.006 [0.997, 1.016] | — | 1.000 [0.986, 1.013] | 1.024 [0.996, 1.053] | 1.017 [0.995, 1.039] | 0.987 [0.945, 1.031] |
| release-scaling/run-20261115T123513Z | 55a58b18b1a40d7ee0ba01ea47a48bf60f8ce2ca | 1.005 [0.994, 1.017] | — | — | — | — | — |
| release-scaling/run-20261115T134325Z | v0.3.1 | 0.998 [0.988, 1.007] | — | 0.995 [0.981, 1.010] | 0.994 [0.977, 1.012] | 0.960 [0.929, 0.992] | 0.958 [0.933, 0.984] |
| release-scaling/run-20261115T143048Z | v0.3.2 | 1.000 [0.990, 1.009] | — | 0.997 [0.989, 1.004] | 1.005 [0.965, 1.046] | 0.996 [0.977, 1.015] | 1.000 [0.970, 1.030] |
| release-scaling/run-20261115T193005Z | v0.3.3 | 1.078 [1.057, 1.100] | — | 0.993 [0.976, 1.011] | 1.010 [0.986, 1.035] | 1.004 [0.980, 1.028] | 0.993 [0.964, 1.023] |
| release-scaling/run-20261116T013343Z | v0.4.0 | 1.040 [1.025, 1.055] | — | 0.994 [0.980, 1.007] | 0.979 [0.947, 1.012] | 1.001 [0.983, 1.019] | 0.999 [0.976, 1.024] |
| release-scaling/run-20261116T092431Z | v0.5.0 | 1.006 [0.991, 1.022] | — | 1.079 [1.068, 1.090] | 1.066 [1.047, 1.085] | 1.084 [1.064, 1.104] | 1.084 [1.039, 1.132] |



### Split-051, ratio of the older configuration to 0.5.1 / PR468

The [`split-051/run-20261116T145423Z`](https://github.com/lmmx/mibz2/tree/14b969f4ef2d9c8da4c7ac77643babbddc28b6c6/split-051) bundle paired v0.5.0, PR462 and PR468 in ten rounds. The stored report prints PR468 / earlier; these reciprocal ratios and intervals are earlier / PR468, so the denominator is current production. At 8–10 threads a held spinner ran (free core at 8–9, sharing a decoder core at 10); none at 12–20.

| arm / PR468 | 8 | 9 | 10 | 12 | 16 | 20 |
|---|---|---|---|---|---|---|
| PR462 | 0.9802 [0.9712,0.9894] | 0.9833 [0.9650,1.0019] | 1.0036 [0.9723,1.0361] | 1.0820 [1.0591,1.1055] | 1.0717 [1.0550,1.0889] | 1.0620 [1.0453,1.0790] |
| v0.5.0 | 1.0008 [0.9902,1.0116] | 0.9884 [0.9706,1.0066] | 1.0786 [1.0523,1.1057] | 1.0588 [1.0419,1.0758] | 1.0533 [1.0284,1.0787] | 1.0600 [1.0283,1.0927] |

### Every interval wholly below one, and its disposition

- **Still-open, post-0.5.1 plan:** `lazy-place` Wikidata ×8: `-cores1` at 10/12/20 (0.984/0.954/0.951); `-cores-pin` at 12/20 (0.981/0.980); `-cores1-pin` at 10/12/20 (0.987/0.959/0.959). Single-copy Wikidata: `-cores` at 20 (0.936), `-cores1` at 10/20 (0.971/0.888), `-cores-pin` at 20 (0.943), `-cores1-pin` at 20 (0.914). All exact intervals and arm/bundle identities are in the grid. The unpinned nine-worker arm dominates the pin variants on ×8. Its same-run ratio to the ten-worker `-cores` arm is 0.983 [0.975,0.991] at 10, 0.957 [0.949,0.965] at 12, and 0.956 [0.952,0.960] at 20, isolating the benefit of one fewer worker **under the same worker-sized window**. The result against production combines that count with the smaller window; it does not isolate the window's effect at nine workers.
- **Current-plan split anomaly:** `split-051` PR462 / PR468 at 8 is 0.9802 [0.9712,0.9894]. #468 changes the plan only *past* the cores, so there is no 8-thread plan change to take. The two builds differ, but this cell alone cannot assign the difference to the plan. Confirm before any action.
- **Already incorporated into 0.5.1:** older `crossover` forced workers at 8/9 (four cells: 0.938/0.956, `-w` 0.923/0.964); `lazy-crossover` forced lazy at 10 in its first two runs (0.931/0.917); `lazy-cap` capped lazy at 10/12/16/20 (0.934/0.926/0.945/0.950) and uncapped lazy at 10 (0.931); `lazy-occupancy` cap at 20 on one and eight copies (0.942/0.935). These prompted #462/#468, so no remaining plan gain follows from them.
- **Historical release comparator wins only:** `v0.2.6` at 16 (0.978 [0.957,0.999]), build `5e7b9954` at 20 (0.938 [0.881,0.998]), `v0.3.1` at 16/20 (0.960 [0.929,0.992] / 0.958 [0.933,0.984]). Each denominator is its then-master, not 0.5.1; none licenses a current-plan change. All other direct cells have intervals touching or exceeding one.

## Plan ranking and adoption decision

1. **Nine lazy workers plus worker-sized in-flight window at 10–20 requested threads on this ten-core i9.** The direct post-0.5.1 x8 gains are 1.6%, 4.6%, 4.9% at 10/12/20; single-copy gains 2.9%/11.2% at 10/20. It leaves a physical core for scanner/consumer and lowers work in flight. The key mechanism is count/occupancy, not proven LLC relief. This is the highest-value plan candidate. It has not been tested at 8, 9 or 16, nor across the production corpus guard. **Current-master confirmation is needed before adoption**: the run is on f40eb4cc's branch, though it descends from post-0.5.1 master. Register the new acceptance question first, pair nine-worker/window against current production on the i9 grid and CI's relevant guards. No such run is performed here.
2. **Worker-sized window at ten workers.** `-cores` beats current production only on one-copy Wikidata at 20 (0.936 [0.919,0.953]); ×8 is at parity at 10/12/20. The plan patch would size its queue by actual workers. A confirming current-master run is needed for general adoption; it is a separable fallback if nine workers regress a guard.
3. **Affinity pinning.** `-cores-pin` at 12/20 is ≈2% faster on ×8, but `-cores1-pin` is slower than unpinned nine workers. It is dominated and less portable. Confirm only if the unpinned route fails.
4. **Historical thresholds, uncapped SMT, stop-walk, ring/hint, prior releases:** no remaining demonstrated gain against 0.5.1. The uncapped lazy arm at 20 is 1.206–1.265 of contemporaneous production on ×8/one-copy in occupancy and later crossover; forced stop-walk's apparent 12/20 gain is relative to uncapped lazy and does not establish a production win. No plan change.

## Layout ranking — measured efficiency and binding cost

[#503's replay](https://github.com/lmmx/mibz2/pull/503) used 196 Wikidata blocks, 11,090,867 letters and rustc 1.99.1. [MONODROMY record 37](https://github.com/lmmx/mibz2/pull/145#issuecomment-6925942307) defines W10 but provides no writer count; [record 38](https://github.com/lmmx/mibz2/pull/145#issuecomment-6926156107) is a forecast, not a measurement.

| layout over W16 | bytes saved / letter | extra writer instructions / letter | saved bytes / extra instruction | writer wall / W16 | status |
|---|---:|---:|---:|---:|---|
| W8 | 4.57 | 9.3 (46.0−36.7) | **0.491** | 1.16–1.19 | measured; dependent length loads hurt |
| W12 | 2.29 | 12.9 (49.6−36.7) | **0.178** | 1.01–1.03 | measured; writer near parity |
| W10 | 3.43 | unknown | unknown | unknown | 40-bit tagged children; writer replay pending |
| common-child grouping | 2.53 with 8-byte header over W16; 2.37 with 16-byte header | unknown | unknown | unknown | census bound, not writer replay; savings overlap other layouts |

**Efficiency order from measured counts:** W8 > W12. W10 and grouping are unranked by the requested ratio until their writer deltas are counted. W10 beats W12's 0.178 only if its added writer cost is under 19.3 instructions/letter; grouping's 2.53 B beats it only if under 14.3. Those thresholds are deductions, not measurements. The numerical ratio alone is misleading: W8 has the better ratio but the worse binding writer time.

**Build priority:** W12 induction-side store/count first, then **W10 writer replay** in #503. W10 retains lengths and could save 3.43 B/letter without W8's dependent length rebuild; build its induction path only if its writer stays within W12's. Common-child grouping follows if its group-header and seek costs can be counted against the writer. W8 is last because the measured writer is 16–19% slower. None is an adopted production change. The counters show unchanged LLC misses after a 5 B/letter cut; whether W12/W10 relieve L2 contention remains unmeasured. No byte saving is assumed to become wall gain.
