# Scanner-Contention Bounded-Slab Recheck Verdict (`scanner-contention-bounded-recheck`): **EXPLAINED**

- **Claim:** `frontier/claims/scanner-contention-bounded-recheck.toml` (C1a diagnostic probe)
- **Frozen recipe:** `just submit bench-scanner-contention` (`bench/scanner_contention.py`, unchanged from `a7ac01d8`)
- **Commit measured:** `8346e392a6f6b73660107308812570b1e897df09` (post-`#1312` `0762c7ea` bounded 80 KiB/worker slab pool)
- **Evidence bundle:** `origin/evidence` `1950f77ff56bbed8bde9510276472f53b7aace64` — `scanner-contention/run-20261128T065932Z.tar.gz` (`15,393` bytes), SHA-256 `3857bbc196c499e28c8103736d4eda22a2ad27e55f779c78c1bc14efe2c29c82` (verified against `.tar.gz.sha256`, commit message, and noticeboard `#145`).
- **Host & validity:** physical `Intel(R) Core(TM) i9-10900K CPU @ 3.70GHz`, 10 physical cores / 20 logical CPUs unrestricted, `cpu_quota = unlimited`, `throttling = false`, `perf_event_paranoid = 0`, `strict_i9 = true`, `smoke = false`, `stub_pmu = false`, `rounds = 12`, `settle_s = 2`, `scanner_cpu = 0`, `worker_mask = 1..9,11..19`, `distinct_binaries = true` (`base` `451f96c0...` vs `lanes` `5217cfc7...`).
- **Parity:** 4/4 exact SHA-256 (`02949df2160357245d83e635cb155950ad0d31d2b4b8703d8e2d1be0bd3577bf`, `838,800,784` bytes, `1,724` blocks, `replayed_c = 0`) against C `libbzip2` (`wikidata-x4`).

## Frozen Stop-Test Verdict: **EXPLAINED**

Let `E = (lanes 8W/4W scanner cycles) - (base 8W/4W scanner cycles)`:

| Frozen Criterion | Pre-`#1312` (`a7ac01d8`, unbounded pool) | Post-`#1312` (`1950f77f`, 80 KiB/w pool) | Threshold | Status |
| --- | --- | --- | --- | --- |
| `base` 8W/4W scanner cycles | `1.0230 [1.0196, 1.0265]` (`+17.43M`) | `1.0231 [1.0199, 1.0263]` (`+17.49M`) | control | identical (`+0.0001`) |
| `lanes` 8W/4W scanner cycles | `1.0647 [1.0619, 1.0675]` (`+49.28M`) | `1.0235 [1.0202, 1.0268]` (`+17.77M`) | collapse | **-4.12 pp** |
| Excess `E = lanes - base` | `+0.0417` (`+4.17%`) | **`+0.0004`** (`+0.039%`) | `<= 0.010` | **EXPLAINED** (`99.1%` removed) |
| L3-miss-stall increment (4W -> 8W) | `lanes +37.64M` vs `base +15.40M` (`2.44x`) | `lanes +17.67M` vs `base +15.22M` (**`1.161x`**) | `<= 1.5x` | **EXPLAINED** |
| 8W peak RSS (`lanes` vs `base`) | `+6.53 MiB` (`1.061x`) | **`-1.32 MiB`** (`0.9900x [0.9834, 0.9966]`) | — | **`-7.85 MiB`** on i9 |
| 8W pinned-scanner wall (`wikidata-x4`) | `lanes 0.1787 s` (`4.69 GB/s`) vs `base 0.1926 s` (`4.36 GB/s`) | **`lanes 0.1691 s` (`4.961 GB/s`)** vs `base 0.1910 s` (`4.392 GB/s`) | — | **`-9.6 ms` (`-5.4%`)**, **`-11.10%`** vs `base/8` |

Both frozen `EXPLAINED` clauses pass by wide margins (`E = +0.000386 <= 0.010` and L3-miss-stall increment ratio `1.161x <= 1.5x`), and the twenty-fifth synthesis pass's working-set prediction (`1.025–1.03`) is confirmed on physical silicon (`1.0235`).

## Per-Cell Medians (`n = 12` interleaved rounds, `wikidata-x4` = `838.80 MB`)

| Cell | Scanner cycles (M) | `clk_r` | IPC | `stalls_l3_miss` (M) | L3 stall % | `LLC-load-misses` (M) | Wall (s) | Throughput (GB/s) | Peak RSS (MiB) |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `base/1` | 757.47 | 1.4258 | 1.884 | 18.27 | 2.4% | 0.310 | 1.3092 | 0.641 | 78.93 |
| `base/4` | 758.69 | 1.3275 | 1.881 | 24.06 | 3.2% | 0.376 | 0.3626 | 2.313 | 91.93 |
| `base/8` | 774.99 | 1.3258 | 1.841 | 39.25 | 5.0% | 0.502 | 0.1910 | 4.392 | 104.61 |
| `lanes/1` | 756.93 | 1.4312 | 1.885 | 18.09 | 2.4% | 0.304 | 1.0905 | 0.769 | 78.95 |
| `lanes/4` | 757.46 | 1.3265 | 1.884 | 24.39 | 3.2% | 0.377 | 0.3010 | 2.787 | 91.25 |
| `lanes/8` | 775.59 | 1.3260 | 1.840 | 41.88 | 5.4% | 0.502 | 0.1691 | 4.961 | 103.29 |

## Paired Ratios (`n = 12`, `t_11 = 2.201`)

### Within-arm scanner scaling
- `base 8W/4W`: `1.0231 [1.0199, 1.0263]` (`+17.49M` cyc, `+15.22M` L3-miss stalls, `87.0%` L3 share, LLC misses `1.336x`)
- `lanes 8W/4W`: `1.0235 [1.0202, 1.0268]` (`+17.77M` cyc, `+17.67M` L3-miss stalls, `99.4%` L3 share, LLC misses `1.331x`)
- `base 4W/1W`: `1.0006 [0.9987, 1.0025]` (`+0.45M` cyc); `lanes 4W/1W`: `0.9997 [0.9966, 1.0029]` (`-0.20M` cyc)
- Retired scanner instructions remain invariant across all 6 cells (`1,427.0M`, `0.99995..1.00001x`).

### Cross-arm paired `lanes / base`
- **`k = 1`:** wall `0.8290 [0.8160, 0.8421]` (`-17.10%`), cycles `1.0003 [0.9980, 1.0026]`, peak RSS `1.0003 [0.9992, 1.0014]` (`+0.03%`).
- **`k = 4`:** wall `0.8317 [0.8266, 0.8368]` (`-16.83%`), cycles `0.9994 [0.9974, 1.0015]`, peak RSS `0.9929 [0.9874, 0.9985]` (`-0.71%`).
- **`k = 8`:** wall **`0.8890 [0.8816, 0.8964]` (`-11.10%`, `4.961 GB/s`)**, cycles **`0.9998 [0.9977, 1.0020]` (`-0.02%`)**, LLC misses `1.0022 [0.9922, 1.0123]`, L3 stalls `1.0693 [1.0502, 1.0888]`, peak RSS **`0.9900 [0.9834, 0.9966]` (`-1.00%`, `-1.32 MiB` lower than base)**.

## Implications
1. **Causal proof of L3 working-set coupling:** bounding `Arena::pool` to `80 KiB/worker` (`#1312`) removed `99.1%` of `lanes`'s 8W scanner-cycle excess (`E: +4.17% -> +0.04%`), bringing `lanes/8` scanner cycles (`775.59M`) and IPC (`1.840`) to exact parity with `base/8` (`774.99M`, `1.841`) while lowering 8W peak RSS below `base` (`-1.32 MiB`, `0.990x`).
2. **Real 8W throughput jumps to `4.961 GB/s` (pinned scanner):** eliminating the `~31.5M` extra L3-stall cycles on the bound 8W scanner cuts real pinned-scanner `lanes/8` wall from `0.1787 s` (`4.69 GB/s` in `a7ac01d8`) to `0.1691 s` (`4.961 GB/s` in `1950f77f`, `-11.10%` vs `base/8` vs `-7.2%` before `#1312`) — nearly matching the prior `4.99 GB/s` ideal-scanner number from `e98948b6` while using the real scanner!
3. **Pricing future worker levers:** RSS-neutral or RSS-negative worker levers (`a6b-zero-rss-endpoint-build`, `forward-rle1-crc-vectored-build`) pay zero scanner L3 tax at the knee, whereas any multi-MiB per-worker working-set expansion past the 20 MiB L3 taxes the bound scanner at `~4.9M` cycles/MiB (~`0.63%`/MiB at 8W).
