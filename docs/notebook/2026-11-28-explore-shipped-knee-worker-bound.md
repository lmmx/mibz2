# On the shipped build, Wikidata is worker-bound at every i9 plan: there is no 8-worker tie

In one line: the v0.5.18 recalibration bundle's per-thread CPU (current candidate 7be50513, default features) puts Wikidata x4's worker demand per worker 19-78 ms above the scanner at every plan the i9 ships. The scanner reaches the workers only at 10 workers, because worker CPU there rises 17-20%, not because the workers got faster. The "scanner-bound at 8 workers" readings in DECODER.md are about the `forward-anchor-lanes` feature build, which is not the default.

- **Claim:** `explore` (recurring), second clause: try to falsify the premise behind the current top performance work ("scanner and workers co-limit at the eight-worker knee", DECODER.md). Cold readback; nothing was re-run.
- **Evidence:** `origin/evidence` `f8b21e4` recalibrate-0518/run-20261128T100510Z.tar.gz, SHA-256 `51fb951b…97f2` (matches its archived `.sha256`). `commit.txt` = b4d25f64, the recipe commit on top of 7be50513 (only `.just/bench.just`, `examples/ab.rs` arms, a test and the registration change). Both families' frozen verdicts are VOID (the harness called `two_arm.py` without `--ref`; the unpinned family's corpus loop also passed `--unpinned` as a corpus: 384 empty rows, which are excluded here). The raw `pinned.jsonl` / `unpinned.jsonl` rows are intact: 12 interleaved rounds per cell, 2 s settle.
- **Method:** each row's `thread_cpu` gives the CPU time of `mibz2-scan` (S), of each `mibz2-N` worker (their sum is W) and of `main` (the drain: copy-out plus hash). Means over 12 rounds (the field is in 10 ms ticks, so a single run's S is quantised; the mean dithers it). Walls are medians. Paired ratios are per-round medians, with a 4000-resample bootstrap 95% interval. `occ-N-S` = N lazy workers, S blocks in flight.

## Wikidata x4 (839 MB out): which station binds

| family, CPUs | plan (arm) | shipped? | wall ms | S ms | W/k ms | W/k − S | wall − max |
|---|---|---|---:|---:|---:|---:|---:|
| pinned 8 | 6/24 | yes (`TRIMMED_AT_EIGHT`) | 237.4 | 145.8 | 224.3 | **+78.5** | +13.1 |
| pinned 10 | 7/28 | yes (`TRIMMED_WORKERS`) | 212.8 | 150.8 | 190.5 | **+39.6** | +22.3 |
| pinned 10 | 8/32 | no | 193.2 | 150.8 | 170.4 | +19.6 | +22.8 |
| pinned 10 | 9/36 | no | 203.8 | 155.8 | 158.7 | +2.9 | +45.1 |
| unpinned 10 | 8/32 | yes (`UNPINNED_FROM_TEN`) | 200.1 | 155.8 | 175.1 | **+19.3** | +25.0 |
| unpinned 12 | 8/32 | yes | 201.8 | 151.7 | 180.5 | +28.9 | +21.3 |
| unpinned 20 | 8/32 | yes | 199.2 | 151.7 | 178.8 | +27.1 | +20.4 |
| unpinned 12 | 10/40 | no | 207.0 | 168.3 | 165.7 | −2.7 | +38.7 |
| unpinned 20 | 10/40 | no | 201.2 | 167.5 | 166.7 | −0.8 | +33.7 |

- **The scanner is flat:** S = 146-152 ms for 3 to 8 workers in both families (the turbo step is already in: every cell has ≥ 4 cores busy). It rises to 155-168 ms only with 9-10 workers.
- **Worker CPU is flat to 8 workers, then inflates:** W = 1,322-1,363 ms (pinned) and 1,325-1,444 ms (unpinned) for 3-8 workers; 1,428-1,493 ms at 9-10 pinned and 1,521-1,694 ms at 9-10 unpinned. So the 10-worker "tie" (W/k ≈ S) is reached by every worker running ~17-20% slower, and the wall does not improve: 10/40 ÷ 8/32 = 0.999 [0.967, 1.033] at 10T, 1.023 [0.981, 1.032] at 12T, 1.036 [0.966, 1.077] at 20T.
- **The shipped plans are worker-bound, and so are scanner levers masked there.** A scanner cut on the default build buys nothing on i9 Wikidata at 8T or 10T until W/k falls 19-40 ms (11-21%). This agrees with the fifteenth pass (trimmed plans worker-bound by 12-20 ms a copy) and the twenty-first pass's "workers are the long pole", and it contradicts the reading that the knee is a tie.
- **Where the "scanner-bound at 8 workers" statements come from:** the scanner-contention bundles (`a7ac01d8`, `1950f77f`) and the nineteenth pass's trace cells measure the `lanes` arm, i.e. `--features forward-anchor-lanes` (`Cargo.toml` `default = []`). With lanes, worker busy falls from 196.5 to 169.6 ms/w (`e98948b6`) and the scanner binds. That swap is conditional on adopting the lanes (`forward-emission-adopt`, `forward-emission-bounded-slab-rescue`). On today's shipped decoder it has not happened.
- **A third term neither station explains:** wall − max(S, W/k) = 20-25 ms at 8/32 (11-12% of wall), and 34-45 ms with 9-10 workers. This is the known per-call lifecycle cost (surface `P` row: T_fixed = 16.1 ms at 8W, mostly 4 KiB minor faults, thread spawn, first chunk and teardown) plus straggler and window coupling. At 9 workers a 3-a-worker window costs more than a 4-a-worker one (9/27 ÷ 8/32 = 1.061 [1.038, 1.100] vs 9/36 0.994 [0.958, 1.011], unpinned 10T). A deeper window at 10 workers is the only cell family under 8/32 on Wikidata: 10/48 = 0.967 [0.951, 0.999] at 12T, 10/64 = 0.967 [0.940, 0.998] at 16T, 10/80 = 1.000 at 20T. But text pays 1.10-1.13 for the same arms, so no plan follows from that.

## Off Wikidata

- **Random (32 MB out):** scanner-bound everywhere from 4 workers (S 63-70 ms vs W/k ≤ 50), wall flat at 90-100 ms; the third term is 20-36 ms.
- **Text (52.8 MB out):** S is only 20-36 ms, yet the wall is 50-78 ms: neither station sets it. Worker CPU grows with the worker count while the wall does not fall (W = 132 ms at 3 workers, 238 at 7, 308 at 8 and 430 at 10, unpinned). Those cycles are waiting, not service, so W/k is no measure of worker demand here. More workers make text slower: 6/24 ÷ 8/32 = 0.873 [0.838, 0.886] and 7/28 ÷ 8/32 = 0.931 [0.918, 0.955] at unpinned 10T. The lifecycle term binds text (consistent with the surface's +19-35% text slowdown from k = 6 to 8-10, owned by `parallel-decoder-pool-and-sysfs-cache`).

## What this means

- **For research priority on the shipped decoder:** on i9 Wikidata, worker levers pay one-for-one at every shipped plan (up to the 19 ms gap at 8/32), and scanner levers pay nothing there. Scanner levers pay on the text and random panels, and on Wikidata only once the lanes or another ≥ 11% worker cut is adopted. Price scanner work against the lanes-adopted system, and say so.
- **For the release's own question (read-only remark; the verdict is the registration's, not this note's):** the shipped pinned 10T plan, 7/28, takes 1.1016 [1.0991, 1.1032] of 8/32's Wikidata wall, while on text 7/28 is 0.934 [0.898, 0.977] of 8/32 pinned and 0.931 [0.918, 0.955] unpinned. That is a real corpus trade-off for whoever reruns the VOID recalibration under its frozen rule.
- **Caveats:** `thread_cpu` is CPU time in 10 ms ticks, not traced busy time. It includes waiting that spins and syscalls, as text shows. On Wikidata, W is flat from 3 to 8 workers, so the waiting there is small; the 9-10 worker inflation could be waiting or contention, and this data cannot tell them apart. One bundle, one machine, one corpus generator. No trace: the third term's split is borrowed from earlier dissections, not measured here.
