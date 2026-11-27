# Composed Post-`v0.5.16` Stack Verification: Throughput, Station Balance (`k*`), RSS Law, and Pre-Ship Interference Audit

*Date:* 27 November 2026
*Zone:* Smooth (`docs/journal/2026-11-27-explore/composed-post-0516-stack.md`, zero `src/` edits on this branch)
*Base commit (`L0`):* `origin/master` `d5634cae` (`v0.5.16`)
*Hardware & environment:* 2-vCPU Haswell KVM VM, Linux, `rustc 1.99.1`, baseline x86-64 (`base`) and JCC-32B-aligned (`jcc`, `-C llvm-args=-x86-branches-within-32B-boundaries`, Comet Lake-S stepping 5 silicon-fix control).
*Protocol:* All 7 candidate post-`v0.5.16` builds were composed cumulatively in an isolated scratch worktree (removed after measurement) with **zero merge conflicts** across all 7 commits. Every layer was verified **100% SHA-256 bit-exact against `libbzip2 1.0.8`** (`210/210` checks: 5 layers × 2 profiles × 7 corpora × 3 modes, 0 mismatches), timed with `examples/station_balance` (`taskset -c 0`, 5 passes/call, `n = 3` interleaved alternating rounds) and `examples/ab child` (`n = 5` interleaved alternating rounds across all 7 standard CI corpora: `wikidata`, `text`, `records`, `random`, `runs`, `pattern`, `onebyte`), plus a dedicated 8-worker `OpenCells` (`mibz2-workers-lazy t=8`) L3/RSS audit and 4-way bisect.

---

## 1. Layers Composed (Zero Conflicts Across All 7 Builds)

| Layer | Added commits / PRs | Files touched (disjoint / auto-merged) | `.text` (`ab` base / JCC) |
|---|---|---|---:|
| **`L0`** | `origin/master` `d5634cae` (`v0.5.16`) | baseline | `1,656,662 B` (`+0.00%`) / `1,705,558 B` |
| **`L1`** | `#1179` (`beec702c`, AVX2-ring+Barrett CRC) + `#1183` (`e12e0aa1`, Segmented fused postwalk) + `build/rle1-2load-filter` (`e82a7785`, 2-load m1 RLE1) | `src/crc.rs`, `src/bwt.rs`, `src/lib.rs`, `src/rle.rs` (auto-merged cleanly) | `1,696,038 B` (`+2.38%`) / `1,746,182 B` |
| **`L2`** | `+ build/a6c-wire-redesign` (`a8c57d08`, zero-bc Cell `wire_xor` + SIMD wire) + `build/post-1080-open-admission` (`de3dd0b5`, two-tier admission) | `src/open.rs` (auto-merged cleanly), `src/open/outq.rs`, `src/open/b5.rs`, `src/open/dense.rs` | `1,697,590 B` (`+2.47%`) / `1,747,558 B` |
| **`L3`** | `+ build/plan-schedule-and-lazy-threshold` (`d320c9d6`, `LAZY_FROM_THREADS=4`, odd `trim_cell`, capped `window()`) + `build/scanner-a3-k3-pipe-u8` (`d8d8ac9e`, `B=10 K=3` u8-chunk + 4-step refill) | `src/parallel.rs`, `src/bits.rs`, `src/block.rs`, `src/huff.rs` | `1,700,486 B` (`+2.65%`) / `1,750,630 B` |
| **`L3fwd`** | `L3` compiled with `--features forward-anchor-lanes` (`#1110` + `a6c-wire` forward wire + `#1179` + `#1178`) | research feature enabled | `1,716,294 B` (`+3.60%`) / `1,767,174 B` |

---

## 2. Executive Summary: Two Major Pre-Ship Findings + Full Composed Gains

1. **CRITICAL PRE-SHIP CATCH — `post-1080-open-admission` (`de3dd0b5`) violates the `<= 1.01×` 8-worker L3-footprint law on `wikidata.bz2` (`+34.7..+41.7 MiB` = `+69%..+84%` 8W `OpenCells` RSS!):**
   - At `t = 4`, `L3`'s `LAZY_FROM_THREADS = 4` (`EagerCells → OpenCells`, `82.1 → 63.5 MiB`) masked a massive per-worker `OpenCells` RSS regression introduced in `L2`.
   - Direct `mibz2-workers-lazy` testing at `t = 1` and `t = 8` (matching physical i9-10900K's 8-worker `OpenCells` default) immediately exposed it: `1W` RSS jumped from `24.2 → 33.1 MiB` (`+8.9 MiB/w`, `1.360×`) and `8W` RSS jumped from **`50.5 → 90.7 MiB` (`+40.2 MiB`, `1.797×`)** — **10× larger than `#1053`'s `+4.19 MiB` RSS increase that killed `#1053` on Comet Lake 20 MiB L3**!
   - **4-way bisect pinned `de3dd0b5` as 100% of the jump:**
     - `L0`: `23.5 MiB` (1W) / `50.2 MiB` (8W)
     - `L1` (`#1179` + `#1183` + `rle1-2load`): **`24.2 MiB` (1W) / `50.7 MiB` (8W, `1.010×`)** — CLEAN!
     - `L2a` (`L1 + a6c-wire-redesign` ONLY): **`24.4 MiB` (1W) / `51.6 MiB` (8W, `1.028×`)** — CLEAN!
     - `L2b` (`L1 + post-1080-open-admission` ONLY): **`33.3 MiB` (1W, `+9.8 MiB`) / `84.9 MiB` (8W, `+34.7 MiB` = `1.691×`, and `+11.6%` SLOWER 8W CPU `1090.4 vs 946.4 ms`!!)**!
     - `L3clean` (`L1 + a6c-wire + plan-schedule + scanner-a3`, **without `de3dd0b5`**): **`24.3 MiB` (1W, `843.1 ms` `-3.6%`) / `51.4 MiB` (8W, `1.024×`, `952.7 ms` `-2.5%`)**!
   - **Root cause & exact pre-ship fix:** On `origin/master`, `OpenCellsOf::cap` admits all `r/n < 0.15` blocks outright (`Cap::NONE`), so `0/233` full `wikidata.bz2` blocks fall back to `Segmented`. `de3dd0b5` lowered `OPEN_OUTRIGHT` from `0.15 → 0.10`, subjecting `[0.10, 0.15)` to `Cap::dense` (`m <= 0.22 * n`, `strips <= 16_384`). Borderline full `wikidata` blocks refused by `Cap::dense` fall back to `Segmented::invert_runs`, **permanently allocating `~8.4 MiB/worker` of `Segmented::lf` (`Vec<u32>`, `3.6 MiB`) + `ll` + `rle` scratch on every worker**!
   - **Exact win-win fix before shipping `post-1080-open-admission`:** Keep `OPEN_OUTRIGHT = 0.15` (so `0/233` full `wikidata` blocks spill into `Segmented` and 8W `OpenCells` RSS stays `~51 MiB`), while still **dropping the dead `[0.15, 0.28)` dense band** (`r/n >= 0.15 → Segmented` directly, which delivers the full `-13%..-15%` `text` win and prevents `L3fwdclean`'s `+15.6%` `text` regression under `forward-anchor-lanes`!).

2. **Full Composed Stack Gains (`L3` / `L3clean` / `L3fwd` vs `L0` `v0.5.16`):**
   - **Exactness:** **`210 / 210`** cryptographic SHA-256 parity checks vs `libbzip2 1.0.8` passed (`0` mismatches).
   - **Pinned `station_balance` (`k* = W / S`, JCC-32B Comet Lake stepping 5 control):**
     - `wikidata`: `S` **`-5.2%`** (`69.45 → 65.83 ms`), `W` **`-4.8%`** (`L3`: `622.84 → 592.65 ms`) / **`-9.0%`** (`L3fwd`: `566.68 ms`, `2.970 → 2.702 ns/B`), `k*`: `8.97 → 8.99` (`L3`) / `8.59` (`L3fwd`). On baseline x86-64: `S -4.8%`, `W -3.4%` (`L3`) / **`-11.9%`** (`L3fwd`, `k* 9.73 → 8.98`).
     - `text`: `S` **`-10.4%..-10.9%`** (`29.14 → 26.11/25.95 ms`), `W` **`-13.3%..-14.3%`** (`136.55 → 118.41/116.98 ms`), `k*`: `4.69 → 4.55/4.50`.
     - `records` & `random`: `0` regressions under JCC-32B (`records`: `S 1.009×`, `W 0.995×`; `random`: `S 0.984×`, `W 0.986×`).
   - **4-thread parallel production (`t = 4`, JCC-32B, `L3` / `L3fwd` vs `L0`):**
     - `wikidata`: wall **`-8.0%`** (`L3`, `0.920 [0.881, 0.961]`) / **`-10.6%`** (`L3fwd`, `0.894 [0.863, 0.927]`), CPU **`-9.4%`** (`L3`, `0.906 [0.889, 0.923]`) / **`-11.3%`** (`L3fwd`, `0.887 [0.863, 0.911]`), peak RSS **`-22.7%`** (`82.1 → 63.5 MiB`) / **`-17.5%`** (`67.7 MiB`). (Baseline x86-64 `t=4`: wall **`-9.9%` / `-12.7%`**, CPU **`-6.6%` / `-9.9%`**, RSS **`-25.5%` / `-21.9%`**!)
     - `text`: wall **`-9.3%`** (`L3`) / **`-16.6%`** (`L3fwd`), CPU **`-11.9%`** (`L3`, `0.881 [0.811, 0.957]`) / **`-15.0%`** (`L3fwd`, `0.850 [0.809, 0.893]`), RSS `0.974×` (`L3`).
     - `runs`: wall **`-16.2%` / `-17.3%`**, CPU **`-15.9%` / `-16.5%`** (`0.841 / 0.835`, disjoint), RSS `0.978×`.
     - `onebyte`: wall **`-16.5%` / `-11.4%`**, CPU **`-13.6%` / `-10.4%`** (`0.864 / 0.896`, disjoint), RSS `0.994×`.
     - `records`, `random`, `pattern`: zero disjoint regressions across all 3 (`records` `0.982×`, `random` `1.015–1.034×` overlapping `1.0`, `pattern` `0.989–0.995×`).
   - **Note on `L3fwdclean` 8W RSS (`+7.9 MiB` / `1.157×`, `50.2 → 58.1 MiB`):** `#1110`'s 16-anchor four-lane slabs add `~+0.84 MiB/worker` over `L3clean` (`51.4 → 58.1 MiB`) while cutting 1W `OpenCells` CPU `-10.6%` (`874.2 → 781.8 ms`) and 8W CPU `-4.7%` (`976.9 → 930.6 ms`). The queued `#1168` i9 kill test will directly judge that `+0.84 MiB/w` vs `-9%..-11%` worker CPU trade-off on Comet Lake 20 MiB L3 (and `forward-rle1-crc-vectored-build`'s 32 KiB slabs trim `-4.0%` of `#1110`'s RSS).

---

## 3. Pinned `station_balance` (`S`, `W`, `k* = W / S`, CPU 0, 5 passes × 3 rounds)

### 3.1 JCC-32B-Aligned (`jcc`, Comet Lake stepping 5 control)

| Corpus | Layer | `S` (ms) | `S / L0` | `W` (ms) | `W / L0` | `k* = W/S` | `ns/sym` | `ns/B` |
|---|---|---:|---:|---:|---:|---:|---:|---:|
| `wikidata` | `L0` | `69.45` | `1.000` | `622.84` | `1.000` | `8.97` | `2.780` | `2.970` |
| `wikidata` | `L1` | `69.52` | `1.001` | `609.53` | `0.979` | `8.77` | `2.783` | `2.907` |
| `wikidata` | `L2` | `69.70` | `1.004` | `598.24` | `0.961` | `8.55` | `2.790` | `2.853` |
| `wikidata` | `L3` | **`65.83`** | **`0.948`** | **`592.65`** | **`0.952`** | `8.99` | `2.635` | `2.826` |
| `wikidata` | `L3fwd` | **`66.12`** | **`0.952`** | **`566.68`** | **`0.910`** | `8.59` | `2.647` | `2.702` |
| `text` | `L0` | `29.14` | `1.000` | `136.55` | `1.000` | `4.69` | `3.101` | `4.779` |
| `text` | `L1` | `28.97` | `0.994` | `123.17` | `0.902` | `4.25` | `3.083` | `4.311` |
| `text` | `L2` | `29.05` | `0.997` | `117.12` | `0.858` | `4.03` | `3.092` | `4.099` |
| `text` | `L3` | **`26.11`** | **`0.896`** | **`118.41`** | **`0.867`** | `4.55` | `2.779` | `4.144` |
| `text` | `L3fwd` | **`25.95`** | **`0.891`** | **`116.98`** | **`0.857`** | `4.50` | `2.762` | `4.094` |
| `records` | `L0` → `L3` / `L3fwd` | `84.35 → 85.13 / 85.12` | `1.009` | `303.14 → 302.62 / 301.57` | `0.998 / 0.995` | `3.60 → 3.54 / 3.53` | `4.393 → 4.433` | `15.79 → 15.71` |
| `random` | `L0` → `L3` / `L3fwd` | `67.85 → 66.79 / 66.79` | `0.984` | `265.63 → 262.28 / 261.85` | `0.987 / 0.986` | `3.92 → 3.91 / 3.92` | `4.241 → 4.175` | `16.60 → 16.37` |

### 3.2 Baseline x86-64 (`base`)

| Corpus | Layer | `S` (ms) | `S / L0` | `W` (ms) | `W / L0` | `k* = W/S` |
|---|---|---:|---:|---:|---:|---:|
| `wikidata` | `L0` → `L1` → `L2` → `L3` → `L3fwd` | `71.44 → 70.64 → 71.56 → 67.99 → 68.35` | `0.952 / 0.957` | `695.93 → 682.45 → 688.91 → 672.52 → 613.18` | `0.966 / 0.881` | `9.73 → 9.88 / 8.98` |
| `text` | `L0` → `L1` → `L2` → `L3` → `L3fwd` | `29.79 → 29.55 → 29.68 → 26.83 → 26.92` | `0.901 / 0.904` | `140.67 → 124.72 → 120.90 → 117.52 → 120.73` | `0.835 / 0.858` | `4.77 → 4.39 / 4.44` |
| `records` | `L0` → `L3` / `L3fwd` | `86.04 → 86.65 / 87.03` | `1.007 / 1.012` | `321.56 → 306.03 / 315.64` | `0.952 / 0.982` | `3.74 → 3.52 / 3.62` |
| `random` | `L0` → `L3` / `L3fwd` | `68.73 → 67.70 / 68.34` | `0.985 / 0.994` | `288.61 → 270.68 / 281.61` | `0.938 / 0.976` | `4.21 → 4.00 / 4.12` |

*(Note: on `base`, `records`/`random` `W` shifted from `0.948/0.942` at `L2` to `0.982/0.976` at `L3fwd`, whereas under `jcc` `L2`/`L3`/`L3fwd` are identical within `0.5%`, confirming pure Haswell JCC layout sensitivity and zero algorithmic interference.)*

---

## 4. Whole-Decoder `ab child` Across All 7 Corpora (`n = 5` Interleaved)

### 4.1 Production `t = 4` (`mibz2`, `L0..L2` use `EagerCells`, `L3`/`L3fwd` use `OpenCells`)

| Corpus | Build | `L0` CPU / RSS | `L1` CPU/L0 | `L2` CPU/L0 | `L3` CPU/L0 `[95% CI]` & RSS/L0 | `L3fwd` CPU/L0 `[95% CI]` & RSS/L0 |
|---|---|---|---:|---:|---|---|
| `wikidata` | `base` | `964.9 ms` / `82.0 MiB` | `1.007` | `0.963` | **`0.934 [0.915, 0.953]`** / **`0.745×`** (`61.2 MiB`) | **`0.901 [0.864, 0.940]`** / **`0.781×`** (`64.0 MiB`) |
| `wikidata` | `jcc` | `938.3 ms` / `82.1 MiB` | `0.999` | `0.984` | **`0.906 [0.889, 0.923]`** / **`0.773×`** (`63.5 MiB`) | **`0.887 [0.863, 0.911]`** / **`0.825×`** (`67.7 MiB`) |
| `text` | `base` | `289.5 ms` / `66.1 MiB` | `0.866` | `0.863` | `0.862 [0.712, 1.044]` / `0.954×` | `0.889 [0.690, 1.147]` / `0.964×` |
| `text` | `jcc` | `316.2 ms` / `66.6 MiB` | **`0.900`** | **`0.894`** | **`0.881 [0.811, 0.957]`** / `0.974×` | **`0.850 [0.809, 0.893]`** / `1.018×` |
| `records` | `base` / `jcc` | `487.5` / `493.3 ms` | `1.019` / `0.981` | `0.993` / `0.985` | `0.999` / `0.982` (`0.980 / 0.976×` RSS) | `0.992` / `1.004` (`0.982 / 0.975×` RSS) |
| `random` | `base` / `jcc` | `426.1` / `411.2 ms` | `1.022` / `1.006` | `1.022` / `0.994` | `0.995` / `1.034` (`0.973 / 1.007×` RSS) | `0.999` / `1.015` (`0.995 / 1.025×` RSS) |
| `runs` | `base` / `jcc` | `48.9` / `45.8 ms` | **`0.841` / `0.834`** | **`0.825` / `0.856`** | **`0.799` / `0.841`** (`0.979 / 0.978×` RSS) | **`0.823` / `0.835`** (`0.983 / 0.979×` RSS) |
| `pattern` | `base` / `jcc` | `166.0` / `161.2 ms` | `1.002` / `0.942` | `0.969` / `0.958` | `0.986` / `0.995` (`1.026 / 0.918×` RSS) | `0.955` / `0.989` (`1.117 / 0.992×` RSS) |
| `onebyte` | `base` / `jcc` | `66.5` / `69.4 ms` | **`0.897` / `0.899`** | **`0.897` / `0.887`** | **`0.895` / `0.864`** (`1.001 / 0.994×` RSS) | **`0.903` / `0.896`** (`1.000 / 0.997×` RSS) |

### 4.2 Sequential `t = 1` (`Seq(Cells)`, pinned CPU 0, `n = 5`)

- `wikidata`: `L3` `-4.0%` base (`0.960 [0.945, 0.976]`, `1.004×` RSS) / `-3.0%` JCC (`0.970 [0.940, 1.002]`, `1.000×` RSS).
- `text`: `L3` **`-8.2%`** base / **`-8.8%`** JCC (`0.912 [0.877, 0.949]`, **`0.975×` RSS**); `L3fwd` **`-11.1%`** base / **`-9.8%`** JCC.
- `records`: `L3` `-4.1%` base / `-3.4%` JCC (`1.000×` RSS).
- `random`: `L3` `-0.3%` base / `-0.4%` JCC (`1.000×` RSS).
- `runs`: `L3` **`-4.0%`** base / **`-3.3%`** JCC (`0.985×` RSS).
- `pattern`: `L3` `-2.7%` base / **`-3.2%`** JCC (`1.006×` RSS).
- `onebyte`: `L3` **`-3.6%`** base / `-1.8%` JCC (`0.986×` RSS).

---

## 5. 8-Worker `OpenCells` RSS Audit & 4-Way Bisect (`mibz2-workers-lazy`, `n = 3`)

| Stack | `wikidata` 1W RSS / CPU | `wikidata` 8W RSS (`vs L0`) / CPU | `text` 1W RSS / CPU | `text` 8W RSS / CPU |
|---|---:|---:|---:|---:|
| **`L0`** (`v0.5.16`) | `23.5 MiB` / `874.2 ms` | `50.2 MiB` (`1.000×`) / `976.9 ms` | `26.1 MiB` / `225.0 ms` | `99.3 MiB` / `339.9 ms` |
| **`L1`** (`#1179` + `#1183` + `rle1-2load`) | `24.2 MiB` / `853.5 ms` | **`50.7 MiB` (`1.010×`)** / **`946.4 ms`** | `25.2 MiB` / `201.6 ms` | `101.3 MiB` / `315.7 ms` |
| **`L2a`** (`L1 + a6c-wire-redesign` ONLY) | `24.4 MiB` / `846.2 ms` | `51.6 MiB` (`1.028×`) / `959.7 ms` | `26.1 MiB` / `200.2 ms` | `103.4 MiB` / `311.4 ms` |
| **`L2b`** (`L1 + post-1080-open-admission` ONLY) | **`33.3 MiB`** / `866.4 ms` | **`84.9 MiB` (`1.691×`, `+34.7 MiB`!)** / **`1090.4 ms`** | `25.3 MiB` / `188.5 ms` | `97.4 MiB` / `295.0 ms` |
| **`L3clean`** (`L1 + a6c-wire + plan-schedule + scanner-a3`) | **`24.3 MiB`** / **`843.1 ms`** | **`51.4 MiB` (`1.024×`)** / **`952.7 ms`** | `26.0 MiB` / `198.7 ms` | `98.7 MiB` / **`295.2 ms`** |
| **`L3fwdclean`** (`L3clean + forward-anchor-lanes`) | `25.4 MiB` / **`781.8 ms`** | `58.1 MiB` (`1.157×`, `+7.9 MiB`) / **`930.6 ms`** | `25.4 MiB` / `203.8 ms` | `106.4 MiB` / `341.4 ms` *(needs dense-band drop)* |
| `L3` (`L3clean + de3dd0b5`) | `32.4 MiB` / `850.9 ms` | `89.8 MiB` (`1.789×`, `+39.6 MiB`) / `969.4 ms` | `25.0 MiB` / `196.7 ms` | `101.3 MiB` / `304.4 ms` |
| `L3fwd` (`L3fwdclean + de3dd0b5`) | `33.4 MiB` / `798.3 ms` | `95.4 MiB` (`1.900×`, `+45.2 MiB`) / `966.9 ms` | `25.3 MiB` / `184.1 ms` | `103.3 MiB` / **`294.8 ms`** |

*(JCC-32B 8W `OpenCells` `wikidata` check confirmed the exact same jump: `L0 50.5 MiB`, `L1 53.8 MiB`, `L2 90.7 MiB` (`+40.2 MiB` / `1.797×`), `L3 79.9 MiB`, `L3fwd 98.0 MiB`.)*

---

## 6. Recommended Ship Order & Actionable Pre-Ship Fixes

1. **Ship immediately in order (zero conflicts, zero RSS issues):**
   - `#1179` (`crc32-avx2-ring-barrett-build`, B5 review) → `#1183` (`segmented-fused-postwalk`, B5 review) → `rle1-2load-filter-build` → `a6c-wire-redesign` → `plan-schedule-and-lazy-threshold` → `scanner-a3-k3-pipe-u8`.
   - Combined (`L3clean`): `0` merge conflicts, `0` parity issues, `+2.65%` `.text`, `wikidata` `S -5.2%` / `W -4.8%` / `t=4` wall `-8.0..-9.9%` / RSS `-23..-26%`, `text` `W -13.3..-16.5%` / `S -10.4%`, `runs` `-16..-20%`, `onebyte` `-11..-14%`, and `8W` `OpenCells` RSS flat (`51.4 vs 50.2 MiB`).
2. **Fix `post-1080-open-admission` (`de3dd0b5`) BEFORE opening its PR:**
   - **Do NOT lower `OPEN_OUTRIGHT` from `0.15` to `0.10`**, which caused borderline `wikidata` blocks to allocate `~8.4 MiB/worker` of `Segmented` buffers (`+35..+42 MiB` at 8W!).
   - Keep `OPEN_OUTRIGHT = 0.15` (0 full `wikidata` blocks spilled to `Segmented`, keeping 8W RSS `~51 MiB`) while **still dropping the dead `[0.15, 0.28)` dense band** (`r/n >= 0.15 → Segmented` directly), which preserves the entire `text` win and eliminates `L3fwdclean`'s `+15.6%` `text` regression under `forward-anchor-lanes`!
3. **Hold `forward-anchor-lanes` promotion behind `#1168` i9 kill test** (plus the `post-1080-open-admission` dense-band drop fix and `forward-rle1-crc-vectored-build`'s `-4.0%` slab RSS trim).
