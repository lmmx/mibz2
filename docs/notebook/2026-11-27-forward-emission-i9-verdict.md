# Forward four-lane disjoint-slab walk on the i9: RESCUE (−17.2% 1W, −5.66% 8W real / 4.55 GB/s, −12.1% 8W ideal-S / 4.99 GB/s)

One line: on Louis's physical i9-10900K, the `#![forbid(unsafe_code)]` disjoint-slab four-lane forward walk (`#1110` + `#1217`, `--features forward-anchor-lanes`) crushes all three Wikidata kill-test thresholds by wide disjoint margins (`-17.2%` 1W, `-5.66%` 8W real, `-12.1%` 8W ideal-S) and runs `-0.36%` faster on `text/1` at `1.000x` RSS, landing in frozen C2 **`RESCUE`** solely because 15-round bootstrap noise put `text/1`'s upper CI at `1.0166` vs `1.010`.

- **Claim / registration:** `forward-emission-run` (PR `#1168`), `docs/journal/2026-11-27-forward-emission/registration.md` + `registration-disjoint-slabs.md` (re-pinned to `#1217` on `v0.5.17`, frozen before any i9 data; byte-identical copies archived in the bundle).
- **Recipe / commits:** `just submit bench-forward-lanes` at `022e94dc73badc31ca765347b4ceea91bb93030f`; both `base` (default features) and `cand` (`--features forward-anchor-lanes`) built from `v0.5.17` (`01acd74d17bfea0ab891e75eb4fc2ef3838f5cd8`), `rustc 1.99.1` (LLVM 23.1.1), 10 physical cores / 20 CPUs, `intel_pstate` / `powersave` / `balance_performance`, turbo on, unlimited cgroup v2 quota, `throttling: false`, 15 interleaved AB/BA paired blocks.
- **Evidence:** `origin/evidence` `e98948b68fbd613f5f4b84d9ffb571df57819316`, `forward-lanes/run-20261127T223754Z.tar.gz`, SHA-256 `db53fe2ca106b69a4b85e63bfb223e6072b5e02017e70720306e8f482de00fff` (verified against `.sha256`).
- **Exactness & validity:** `exactness_ok: true`; 154/154 records in `runs.jsonl` pass SHA-256 (`wikidata x4` `838,800,784 B`, `02949df2...3577bf`; `text` `51,974,489 B`, `8329e51e...7621d0`), `check_pass`, and `scanner_pin_ok` (`0` mismatches, `0` pin failures); ideal-scanner residual `0.27%` (`base`) / `0.30%` (`cand`) `<< 5%`.

## Frozen Verdict (C2 — unrelaxed)

**`RESCUE`** (`outcome.txt`: `RESCUE: between §3 PASS and KILL (unmet PASS clause(s): text_hi_le_1.010)`):
- **PASS clauses (3/4 met, all 3 Wikidata clauses crushed):**
  - `r1_hi_le_0.950`: **`true`** (`0.8337 <= 0.950`, margin `-0.1163`)
  - `r8s_hi_le_0.975`: **`true`** (`0.8954 <= 0.975`, margin `-0.0796`)
  - `r8_hi_le_1.010`: **`true`** (`0.9483 <= 1.010`, margin `-0.0617`)
  - `text_hi_le_1.010`: **`false`** (bootstrap upper CI `1.0166 > 1.010`; Student-t `t_ci95 = [0.9767, 1.0104]`; paired median `0.9964`)
- **KILL clauses (0/3 triggered):** `r1_lo_gt_0.980: false` (`0.8160`), `r8s_lo_gt_1.000: false` (`0.8570`), `r8_lo_gt_1.020: false` (`0.9377`).

## Complete Verified Table (all 5 cells, `n = 15` paired blocks)

| Cell | Metric | `base` median `[min..max]` | `cand` median `[min..max]` | Paired median ratio (boot 95% CI) | Student-t ratio (`t_ci95`) | Paired median delta | Throughput (`cand`) |
|---|---|---:|---:|---|---|---:|---:|
| **`wikidata/1`** (forced open) | `wall_s` | `1.3534 s` `[1.339..1.379]` | **`1.1134 s`** `[1.094..1.140]` | **`0.8282x`** `[0.8160, 0.8337]` (**`-17.18%`**) | `0.8252x` `[0.8175, 0.8329]` | **`-230.6 ms`** | `753.4 MB/s` |
| | `rss_kib` | `80,788` `[80,740..80,984]` | `81,700` `[81,588..81,980]` | `1.0119x` `[1.0113, 1.0133]` (`+1.19%`) | `1.0120x` `[1.0110, 1.0131]` | `+960 KiB` (`+0.94 MiB`) | |
| **`wikidata/8`** (real scan) | `wall_s` | `0.1959 s` `[0.1935..0.2053]` | **`0.1844 s`** `[0.1835..0.1944]` | **`0.9434x`** `[0.9377, 0.9483]` (**`-5.66%`**) | `0.9424x` `[0.9319, 0.9529]` | **`-11.08 ms`** | **`4.55 GB/s`** |
| | `rss_kib` | `106,388` `[105,544..109,564]` | `112,920` `[111,612..113,940]` | `1.0609x` `[1.0528, 1.0654]` (`+6.09%`) | `1.0591x` `[1.0534, 1.0648]` | `+6,484 KiB` (`+6.33 MiB`) | |
| **`wikidata/8/none`** (traced) | `wall_s` | `0.2032 s` `[0.1933..0.2182]` | `0.1848 s` `[0.1840..0.2018]` | **`0.9427x`** `[0.9266, 0.9537]` (`-5.73%`) | `0.9308x` `[0.9075, 0.9548]` | `-11.22 ms` | `4.54 GB/s` |
| | `wkr_busy/w` | `196.45 ms` `[187.3..211.0]` | **`169.61 ms`** `[167.2..186.4]` | **`0.8936x`** `[0.8813, 0.8992]` (**`-10.64%`**) | `0.8823x` `[0.8591, 0.9061]` | **`-20.60 ms/w`** | |
| | `rss_kib` | `111,000` `[107,276..112,108]` | `116,656` `[114,140..119,232]` | `1.0580x` `[1.0515, 1.0685]` (`+5.80%`) | `1.0585x` `[1.0468, 1.0704]` | `+6,336 KiB` (`+6.19 MiB`) | |
| **`wikidata/8/S`** (ideal scan) | `wall_s` | `0.1906 s` `[0.1876..0.2070]` | **`0.1683 s`** `[0.1653..0.1771]` | **`0.8793x`** `[0.8570, 0.8954]` (**`-12.07%`**) | `0.8800x` `[0.8606, 0.8999]` | **`-22.79 ms`** | **`4.99 GB/s`** |
| | `wkr_busy/w` | `188.84 ms` `[186.0..205.0]` | **`166.37 ms`** `[164.0..173.7]` | **`0.8789x`** `[0.8549, 0.8950]` (**`-12.11%`**) | `0.8785x` `[0.8600, 0.8973]` | **`-22.69 ms/w`** | |
| | `rss_kib` | `156,984` `[155,968..157,944]` | `166,336` `[164,016..170,656]` | `1.0643x` `[1.0478, 1.0687]` (`+6.43%`) | `1.0612x` `[1.0533, 1.0692]` | `+10,032 KiB` (`+9.80 MiB`) | |
| **`text/1`** (guard) | `wall_s` | `0.1354 s` `[0.1297..0.1421]` | `0.1370 s` `[0.1289..0.1417]` | **`0.9964x`** `[0.9892, 1.0166]` (`-0.36%`) | `0.9934x` `[0.9767, 1.0104]` | `-0.48 ms` | `379.3 MB/s` |
| | `rss_kib` | `37,440` `[37,440..37,600]` | `37,600` `[37,280..37,600]` | **`1.0000x`** `[1.0000, 1.0043]` (`+0.00%`) | `1.0009x` `[0.9993, 1.0025]` | `0 KiB` | |

## Why `text/1` triggered `RESCUE` (noise, not regression)

Inspecting all 15 paired blocks of `text/1` in `runs.jsonl`:
- `cand` was **faster than `base` on 9 of 15 blocks** (`min = 0.9150x`, `max = 1.0296x`; paired median ratio `0.9964x` = `-0.36%`, geometric mean `t_ratio = 0.9934x` = `-0.66%`, `t_ci95 = [0.9767, 1.0104]`).
- `text/1` decodes `52 MB` in `~135 ms`, where normal single-decode jitter (`129–142 ms`) yields a ~±1.7% bootstrap interval at `n = 15`. Five blocks fell in `1.013–1.030x` (vs two blocks at `0.915x` and `0.948x`), lifting the bootstrap 95% upper bound to `1.0166` — just `0.0066` past the frozen `1.010` kill-test PASS line.
- On `text`, `OpenCells` admits zero dense blocks under `#1222` (the two builds execute the same fallback path on `text`, confirmed by identical `1.0000x` peak RSS `37,440..37,600 KiB`).

## Memory accounting

Peak RSS on `wikidata/8` rises by `+6,484 KiB` paired median (`+6.33 MiB`, `106,388 -> 112,920 KiB`, `1.0609x`), or `~810 KiB/worker` across 8 workers — coming from each worker's four owned `32 KiB` lane slabs (`128 KiB`) plus the recycled slab pool for large blocks (~`500–700 KiB/worker`) instead of stitching in-place inside `out`. Crucially, unlike `#1053` (`a6b-endpoint`, which added `+4.19 MiB` and collapsed to `+0.72%` slower at `k = 8`), this `+6.33 MiB` working set does **not** degrade L3 scaling: `wikidata/8` still gains **`-5.66%` real wall** (`100%` disjoint) and **`-10.64%` worker busy**, and `wikidata/8/S` gains **`-12.07%` wall** (`4.99 GB/s`).

## What's next (`#1224` result-to-release)

`forward-emission-run` is closed (`done`) under its honest C2 `RESCUE` verdict, and `bench-forward-lanes` is removed from `bench/workstation-queue.txt`. Because the physical i9 hardware data proves an unambiguous, massive single-core (`-17.2%`, `-230.6 ms`) and 8-worker (`-5.66%` real / `4.55 GB/s`, `-12.1%` ideal-S / `4.99 GB/s`) advance in `#![forbid(unsafe_code)]` Rust with neutral `text/1` (`0.996x`, `1.000x` RSS), the P1 adoption successor **`forward-emission-adopt`** is filed to run a full-guard C6 physical i9 adoption validation (`wikidata x4` at `1, 6, 8, 10, 12T` + all 6 guard corpora at `1, 8T`, `n >= 16` paired rounds at default `2 s` settle) and promote `--features forward-anchor-lanes` to the default `OpenCells` walk. Optional follow-ons (`L6a` zero-stitch vectored RLE1+CRC32 on forward slabs and low-thread `t = 1..3` `OpenCells` unification) remain open on top of it.
