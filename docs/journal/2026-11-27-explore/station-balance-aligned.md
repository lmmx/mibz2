# Does JCC 32B alignment tip the scanner:worker balance?

Probe for [`station-balance-aligned`](../../../frontier/claims/station-balance-aligned.toml), 27 November 2026. The tenth synthesis pass ([`tenth-pass.md`](../2026-11-27-synthesis/tenth-pass.md), `#1090`) derived from separate measurements (`#1076`, `#1077`) that Intel JCC 32-byte branch alignment (`-C llvm-args=-x86-branches-within-32B-boundaries`) cuts worker demand ~14% while cutting scanner demand only 1–3%, which would flip aligned `wikidata` at `k = 8` from worker-bound/tie to scanner-bound. This probe tests that station-uneven shift directly on a single machine, single commit, and unchanged `examples/station_balance.rs`.

The decision rule was frozen before data in `frontier/claims/station-balance-aligned.toml` (commit `98f6fdf7`):

> On `wikidata`, if the aligned/default `k*` ratio (`r_aligned = k*_aligned / k*_default`) is `<= 0.92` on both native and baseline with non-overlapping round ranges, **CONFIRMED** (the alignment shift is station-uneven; `DECODER.md`'s knee bullet keeps the codegen condition). If it is `>= 0.97` on either build, **REFUTED** (alignment moves both stations alike; the codegen leg of the tenth pass is withdrawn). Otherwise **INCONCLUSIVE** with the ratios reported. Text corpora are reported, not gated.

## Method

- **Code & kernel:** `examples/station_balance.rs` unchanged at commit `f3055a45` (post-`#1034` scanner and post-`#1080` Segmented fallback; zero `src/` edits). For each block it times `Scanner::next` (`S`) and `decode_block::<OpenCells>` (`W`, production's 4W–10W kernel) separately on one thread, keeping the fastest of 5 passes per block and station, and reports `k* = W / S`.
- **Four release builds** (`lto = "fat"`, `codegen-units = 1`, separate `--target-dir`s):
  - `base_default` (`RUSTFLAGS=""`, `.text` `508,550` B)
  - `base_jcc32b` (`RUSTFLAGS="-C llvm-args=-x86-branches-within-32B-boundaries"`, `.text` `523,174` B, `+2.88%`)
  - `native_default` (`RUSTFLAGS="-C target-cpu=native"`, `.text` `499,402` B)
  - `native_jcc32b` (`RUSTFLAGS="-C target-cpu=native -C llvm-args=-x86-branches-within-32B-boundaries"`, `.text` `513,662` B, `+2.86%`)
- **Eight corpora** matching `#1075` (`bzip2 -9`, SHA-256 prefixes):
  - `wikidata` (`82f65ace`, 431 blocks / 233 full, 209.70 MB raw, 17.69× — byte-identical to `#1075`)
  - `prose` (`77aff65e`, 11/11 blocks, `/usr/share/doc` text + `/var/lib/dpkg/status`, 10.00 MB, 8.87×)
  - `pysrc` (`d36bd89e`, 13/13 blocks, Python 3.12 stdlib `.py`, 12.00 MB, 5.53×)
  - `rustsrc` (`522eb473`, 16/16 blocks, `/opt/cargo/registry/src` `.rs`, 15.00 MB, 8.81×)
  - `json` (`98c81cc1`, 15/15 blocks, repository `.json`/`.jsonl`, 13.20 MB, 8.11× — byte-identical to `#1075`)
  - `mdjournal` (`3225dde9`, 13/13 blocks, repository `docs/**/*.md`, 11.37 MB, 3.90×)
  - `logs` (`19c57336`, 18/18 blocks, `/var/log` + `/var/lib/dpkg` state/md5sums/lists, 16.00 MB, 7.22×)
  - `elf` (`78777db4`, 33/33 blocks, `/usr/bin` ELF binaries, 30.00 MB, 2.96×)
- **Execution:** 2-vCPU Xeon @ 2.30 GHz VM (256 KiB L2, 45 MiB L3, Haswell CPUID with `umip`/`md_clear`, VM diagnostic only under C5), pinned to CPU 0 (`taskset -c 0`), 5 passes per invocation, 3 interleaved rounds with per-round alternating build order (`[BD, BJ, ND, NJ]`, `[NJ, ND, BJ, BD]`, `[BJ, BD, NJ, ND]`). All 96 raw rows are preserved unmodified in [`station-balance-aligned/rounds.tsv`](station-balance-aligned/rounds.tsv) and summarised in [`station-balance-aligned/summary.tsv`](station-balance-aligned/summary.tsv).

## Result

### 1. Gated test (`wikidata`, 431 blocks / 233 full)

| Profile | Build | `S` med (ms) | `W` med (ms) | `k* = W/S` med `[min..max]` | `S` delta | `W` delta | `r_aligned` | Disjoint? | Derived i9 `k*` |
|---|---|---:|---:|---:|---:|---:|---:|:---:|---:|
| Baseline | `base_default` | 72.88 | 739.75 | 10.15 `[9.98..10.47]` | — | — | — | — | 8.00 (ref) |
| Baseline | `base_jcc32b` | 69.84 | 618.38 | **8.84** `[8.83..8.86]` | -4.17% | **-16.41%** | **0.8709** | **Yes** | **6.97** |
| Native | `native_default` | 76.94 | 707.93 | 9.20 `[9.18..9.24]` | — | — | — | — | 8.00 (ref) |
| Native | `native_jcc32b` | 74.25 | 622.93 | **8.38** `[8.37..8.58]` | -3.50% | **-12.01%** | **0.9109** | **Yes** | **7.29** |

Per-round `wikidata` ratios:
- **Baseline:** Round 1 `10.47 -> 8.86` (`0.8462`), Round 2 `10.15 -> 8.83` (`0.8700`), Round 3 `9.98 -> 8.84` (`0.8858`). Ranges `[9.98..10.47]` and `[8.83..8.86]` are separated by `1.12` workers.
- **Native:** Round 1 `9.18 -> 8.37` (`0.9118`), Round 2 `9.24 -> 8.38` (`0.9069`), Round 3 `9.20 -> 8.58` (`0.9326`). Ranges `[9.18..9.24]` and `[8.37..8.58]` are separated by `0.60` workers.

**Frozen rule verdict: CONFIRMED.** Both baseline (`r_aligned = 0.8709`) and native (`r_aligned = 0.9109`) are `<= 0.92` with completely non-overlapping round ranges.

### 2. Off-wikidata corpora (7 corpora, reported not gated)

| Corpus (ratio) | Profile | `S` def -> jcc (ms, %) | `W` def -> jcc (ms, %) | `k*` def `[min..max]` -> jcc `[min..max]` | `r_aligned` (med) | Disjoint? | `r` (min-pass) |
|---|---|---:|---:|---:|---:|:---:|---:|
| `prose` (8.87×) | base | 8.81 -> 8.56 (-2.84%) | 77.82 -> 70.05 (-9.98%) | 8.83 `[8.71..10.11]` -> 8.18 `[8.16..11.06]` | 0.9264 | No\* | 0.9360 |
| `prose` (8.87×) | native | 9.56 -> 9.22 (-3.56%) | 76.87 -> 76.11 (-0.99%) | 8.06 `[8.02..9.39]` -> 8.26 `[7.68..8.79]` | 1.0248\* | No\* | 0.9524 |
| `pysrc` (5.53×) | base | 15.56 -> 15.49 (-0.45%) | 71.67 -> 75.85 (+5.83%\*) | 4.61 `[4.49..5.08]` -> 4.90 `[4.31..4.92]` | 1.0629\* | No\* | 0.9593 |
| `pysrc` (5.53×) | native | 16.91 -> 16.24 (-3.96%) | 70.06 -> 68.63 (-2.04%) | 4.22 `[4.13..4.23]` -> 4.23 `[4.20..5.08]` | 1.0024\* | No\* | 0.9970 |
| `rustsrc` (8.81×) | base | 12.36 -> 12.17 (-1.54%) | 85.44 -> 78.85 (-7.71%) | 6.91 `[6.74..7.85]` -> 6.48 `[6.16..6.93]` | 0.9378 | No\* | 0.9137 |
| `rustsrc` (8.81×) | native | 13.33 -> 12.99 (-2.55%) | 84.63 -> 81.44 (-3.77%) | 6.35 `[6.24..6.64]` -> 6.27 `[5.89..6.61]` | 0.9874 | No\* | 0.9430 |
| `json` (8.11×) | base | 10.58 -> 10.39 (-1.80%) | 62.01 -> 56.45 (-8.97%) | 5.86 `[5.81..6.12]` -> 5.50 `[5.38..5.51]` | **0.9386** | **Yes** | 0.9380 |
| `json` (8.11×) | native | 11.35 -> 11.15 (-1.76%) | 61.60 -> 57.81 (-6.15%) | 5.46 `[5.39..5.51]` -> 5.15 `[5.13..5.22]` | **0.9432** | **Yes** | 0.9540 |
| `mdjournal` (3.90×) | base | 19.39 -> 19.00 (-2.01%) | 90.56 -> 81.38 (-10.14%) | 4.67 `[4.65..5.08]` -> 4.31 `[4.28..4.50]` | **0.9229** | **Yes** | 0.9249 |
| `mdjournal` (3.90×) | native | 20.86 -> 20.54 (-1.53%) | 91.95 -> 85.74 (-6.75%) | 4.41 `[4.34..4.85]` -> 4.20 `[4.09..4.20]` | **0.9524** | **Yes** | 0.9484 |
| `logs` (7.22×) | base | 13.33 -> 13.00 (-2.48%) | 86.51 -> 78.01 (-9.83%) | 6.52 `[6.49..6.71]` -> 6.01 `[5.94..6.02]` | **0.9218** | **Yes** | 0.9138 |
| `logs` (7.22×) | native | 14.51 -> 13.95 (-3.86%) | 85.06 -> 78.95 (-7.18%) | 5.89 `[5.79..6.02]` -> 5.70 `[5.61..5.79]` | 0.9677 | No | 0.9730 |
| `elf` (2.96×) | base | 69.98 -> 68.82 (-1.66%) | 290.99 -> 267.48 (-8.08%) | 4.16 `[4.10..4.33]` -> 3.89 `[3.85..4.04]` | **0.9351** | **Yes** | 0.9407 |
| `elf` (2.96×) | native | 75.33 -> 72.65 (-3.56%) | 288.39 -> 269.64 (-6.50%) | 3.84 `[3.83..3.90]` -> 3.71 `[3.70..3.92]` | 0.9661 | No | 0.9629 |

*(\*In Rounds 2–3, brief host-steal spikes inflated `W` by 15–35 ms on the small 11–16 block `prose`, `pysrc`, and `rustsrc` files — visible in `rounds.tsv`. All 96 rows are retained unmodified; the rightmost column `r (min-pass)` reports `(min W / min S)_jcc / (min W / min S)_def` across the three rounds, which is `< 1.00` on 16/16 corpus×profile cells.)*

## What it means

1. **The tenth pass's codegen leg is confirmed on VM silicon.** Alignment moves `wikidata`'s workers 3.4–3.9× more than the scanner (`-16.41%` vs `-4.17%` baseline; `-12.01%` vs `-3.50%` native), pulling `k*` down by `0.871×` (baseline) and `0.911×` (native) with clean separation across rounds. Scaling from the shipped i9 knee (`k* ≈ 8.0`) puts aligned `wikidata` at **`k* ≈ 6.97–7.29` (derived)** — scanner-bound at `k = 8` by ~9–13% of `S'`, right inside the tenth pass's derived range (`~6–22 ms`).
2. **The shift is strongest on `OpenCells` (`wikidata`) and milder on Segmented-dominated text.** Across the 7 off-wikidata corpora, median `r_aligned` is **`0.9351`** baseline (steal-free `0.9360`, 4/7 disjoint: `json`, `mdjournal`, `logs`, `elf`) and **`0.9677`** native (steal-free `0.9540`, 2/7 disjoint: `json`, `mdjournal`). That matches `#1077`'s station attribution (`OpenCells` carries 397–418 JCC violations vs far milder impact on `Segmented`, which now also runs `#1080`'s AVX2 path on `master`). Alignment nudges already scanner-bound text corpora slightly further toward scanner-bound without reversing `#1075`'s corpus ordering.
3. **Why absolute VM `k*` differs from `#1075` (`10.15` vs `6.33` on `wikidata`).** Two factors: this VM has 256 KiB L2 per core vs `#1075`'s 2 MiB L2 (inflating `W` from `2.03` to `3.53 ns/B`, closer to the i9's 256 KiB L2), and `master` now includes `#1034` (faster scanner) and `#1080` (faster Segmented workers). The paired within-machine ratio `r_aligned` cancels both.
4. **Factual build note (`base` vs `native` on `S`).** On all 8 corpora on this Haswell-CPUID VM, `native_default` ran `S` 5–8% slower than `base_default` (`76.94` vs `72.88 ms` on `wikidata`), so `base_jcc32b` (`S = 69.84 ms`, `W = 618.38 ms`) was the fastest of the four builds on both stations. Whether that is a VM-specific Haswell-vs-BMI2 codegen artifact or holds on Comet Lake-S belongs to `jcc-aligned-release-build`.

## Caveats

- **VM diagnostic only (C5).** This confirms the *station-uneven mechanism* on JCC-mitigated Xeon silicon. Whether Louis's i9-10900K (Comet Lake-S stepping 5) incurs the JCC-erratum penalty in hardware remains settled only by the paired i9 run (`jcc-aligned-release-build`).
- Only `wikidata` and `json` are byte-identical to `#1075`; the other six off-wikidata corpora were rebuilt from matching container/repo sources on this VM (SHA-256 prefixes above).
- Delivery (`D`) and multi-worker L3 contention at `k = 8` are not measured by one-thread `station_balance`.
