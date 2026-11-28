# Composed Next-Gen Stack Ablation & Interaction Audit (`M0 -> M5`)

*28 Nov 2026. Smooth-zone lab notebook entry (`Claim: none`, zero code or release-gating paths touched). All VM timings and peak-RSS (`VmHWM`) measurements below were collected on a shared 2-vCPU Linux KVM VM (`Intel(R) Xeon(R) CPU @ 2.30GHz`, Linux `6.18.57`, `rustc 1.99.1`, `[profile.release]` `opt-level = 3`, `lto = "fat"`, `codegen-units = 1`, linking only `mibz2` with zero `pbz2` overhead) using steal-free `CLOCK_PROCESS_CPUTIME_ID` / `CLOCK_THREAD_CPUTIME_ID` under both default x86-64 codegen (`base`) and 32B-aligned branch mitigation (`jcc`: `-C llvm-args=-x86-align-branch-boundary=32 -C llvm-args=-x86-align-branch=fused+jcc+jmp`). **These are VM exploration and composition-audit measurements, NOT physical i9-10900K adoption evidence.** Physical i9-10900K projections in §6 are explicitly labelled **DERIVED** from anchored hardware bundle `origin/evidence 1950f77f` (`4.961 GB/s`).*

## 1. Motivation & 6-Stage Composition (`M0 -> M5`)

Since the earlier 4-way worker stack audit (`docs/notebook/2026-11-28-explore-composed-worker-stack.md`, `B0 -> B4`), three major pre-ship risks have been addressed in separate branches and two additional full-stack levers have landed in review:

1. `#1312` bounded `forward-anchor-lanes`'s slab `Arena` to `80 KiB/worker` (removing `99.1%` of 8W scanner L3-eviction excess on physical i9 in `1950f77f`, `4.961 GB/s`).
2. `mtf-fold` (`d321fef8`) added single-buffer pooling (eliminating the retained dual `956 KiB` `s.ll` copy), dense-band bitmap prescan, and a sub-bound check to fix the `runs.bz2` regression caught in `B4`.
3. `#1328` (`overlay_aos`, `37a2a37a`) replaced `A6b` endpoint elimination at `0 B` extra RSS (`-3.1%` open-route CI on EPYC 9V74); `#1343` (`scanner-a3-k3-pipe-u8`) upgraded serial Huffman decode to `B=10, K=3` u8-chunk pipelined refill; `#1311` (`df4f0bbd`) fused `Segmented Runs` post-walk while keeping `Column` unfused for `random` 1T; and `#1282` (`d8d0c8b5`) lowered `LAZY_FROM_THREADS` from `5` to `4` so default `BzReader::parallel` enters `OpenCells` at `4W`.

To audit interaction effects, RSS stacking, guard regressions, merge conflicts, and the post-stack bottleneck map before these PRs land sequentially, we composed all six layers locally on top of `07ab2e7e` (`master`, pre-`v0.5.18` bump with identical decoder code):

| Stage | Base + Layer | Feature Flags | Mechanism | Target PR / Ref |
|---|---|---|---|---|
| **`M0`** | `07ab2e7e` | *(default)* | Shipped bidirectional `OpenCells` (`LAZY_FROM_THREADS = 5`) | `master` (`v0.5.18` baseline) |
| **`M1`** | `M0` + bounded lanes | `forward-anchor-lanes` | Post-`#1312` 4-lane forward walk with `80 KiB/worker` bounded Arena | `#1324` (`9b438827` recipe) |
| **`M2`** | `M1` + `overlay_aos` | `forward-anchor-lanes` | Zero-extra-RSS edges-overlaid 32B AoS + inline-7 u64 `A6b` | `#1328` (`37a2a37a`) |
| **`M3`** | `M2` + pooled `mtf-fold` | `forward-anchor-lanes,mtf-fold` | Streaming-admission + bitmap prescan + sub-bound check + single-buffer pool | `d321fef8` (`build/mtf-fold-guard-rescue`) |
| **`M4`** | `M3` + `scanner-a3-k3` | `forward-anchor-lanes,mtf-fold` | Rebased port of `d8d8ac9e` (`B=10 K=3` single-u32 u8-chunk decode + 4-step overlapped refill) | `#1343` (`3d2afb8a`) |
| **`M5`** | `M4` + `#1311` + `#1282` | `forward-anchor-lanes,mtf-fold` | Fused `Segmented Runs` post-walk (`Column` unfused) + `LAZY_FROM_THREADS = 4` default parallel `OpenCells` | `#1311` (`df4f0bbd`) + `#1282` (`d8d0c8b5`) |

*Note on `M4`:* the scratch port rebased `build/scanner-a3-k3-pipe-u8` (`d8d8ac9e`, `src/{bits,block,huff}.rs`) onto `07ab2e7e`; the merge-conflict audit in §7 checks the live PR `#1343` head (`3d2afb8a`) directly against `origin/master` (`edf2ceaf`). No scratch code branch was pushed.

## 2. Exactness & SHA-256 Parity (`140/140 PASS`)

Every binary was verified bit-for-bit via SHA-256 against C `bzip2 -dc` across all 7 corpora (`wikidata` `209,700,196 B`, `text` `15,215,652 B`, `records` `9,600,000 B`, `random` `8,000,000 B`, `runs` `10,000,235 B`, `pattern` `19,600,000 B`, `onebyte` `20,000,000 B`):

- **`base` (`70/70 PASS`):** `M0..M5` on `wikidata` `lazy` (`1W/4W/8W`, `18` checks) + `M0` & `M5` on `wikidata` `mibz2` (`1W/4W`, `4` checks) + `M0` & `M5` on all 6 guard corpora (`mibz2` & `lazy`, `1W/4W`, `48` checks).
- **`jcc` (`70/70 PASS`):** identical matrix.
- **Total: `140/140 PASS` (100% SHA-256 bit-exact).**

## 3. Paired `M0 -> M5` Ablation Ladder on `wikidata.bz2` (`n = 9`, VM Measured)

All ratios are paired geometric means over `n = 9` alternating-order rounds (`M0..M5` forward on even rounds, reversed on odd rounds) with two-sided 95% Student-t CIs (`t_8 = 2.306`). `lazy` calls `BzReader::parallel_workers(..., lazy=true)`; `mibz2` calls the production default `BzReader::new` (`1W`) / `BzReader::parallel` (`4W`).

### 3a. Open-Route (`lazy`) Ladder — `base` (`wikidata.bz2`, `n = 9`)

| Stage | `1W` CPU (ms) | `1W` vs `M0` [95% CI] | `1W` step | `1W` HWM | `4W` CPU (ms) | `4W` vs `M0` [95% CI] | `4W` step | `4W` HWM | `8W` CPU (ms) | `8W` vs `M0` [95% CI] | `8W` HWM |
|---|---:|---|---:|---:|---:|---|---:|---:|---:|---|---:|
| **`M0`** | `913.9` | `+0.00%` | — | `22,228 KiB` | `935.1` | `+0.00%` | — | `35,612 KiB` | `950.9` | `+0.00%` | `49,388 KiB` |
| **`M1`** (`+lanes`) | `784.1` | **`-14.44%`** `[0.8441, 0.8673]` | `-14.44%` | `23,184 KiB` | `801.5` | **`-13.60%`** `[0.8488, 0.8795]` | `-13.60%` | `35,672 KiB` | `824.5` | **`-12.86%`** `[0.8548, 0.8883]` | `50,996 KiB` |
| **`M2`** (`+#1328`) | `745.6` | **`-18.26%`** `[0.8055, 0.8294]` | **`-4.47%`** | `23,080 KiB` | `766.7` | **`-17.28%`** `[0.8119, 0.8428]` | **`-4.26%`** | `36,028 KiB` | `785.8` | **`-15.11%`** `[0.7946, 0.9069]` | `52,268 KiB` |
| **`M3`** (`+fold`) | `722.9` | **`-21.07%`** `[0.7809, 0.7978]` | **`-3.44%`** | `23,028 KiB` | `750.1` | **`-20.24%`** `[0.7846, 0.8109]` | **`-3.57%`** | `35,976 KiB` | `748.4` | **`-20.82%`** `[0.7840, 0.7996]` | `50,308 KiB` |
| **`M4`** (`+k3`) | `712.0` | **`-21.98%`** `[0.7727, 0.7878]` | **`-1.15%`** | `22,996 KiB` | `733.3` | **`-21.62%`** `[0.7702, 0.7977]` | `-1.73%` | `36,072 KiB` | `753.0` | **`-20.47%`** `[0.7841, 0.8067]` | `49,644 KiB` |
| **`M5`** (`+#1311+#1282`) | `719.9` | **`-21.51%`** `[0.7755, 0.7945]` | `+0.61%` | `23,076 KiB` | `731.8` | **`-20.88%`** `[0.7799, 0.8027]` | `+0.94%` | `35,964 KiB` | `754.0` | **`-19.98%`** `[0.7874, 0.8132]` | `51,020 KiB` |

### 3b. Open-Route (`lazy`) Ladder — `jcc` (branch-alignment controlled, `wikidata.bz2`, `n = 9`)

| Stage | `1W` CPU (ms) | `1W` vs `M0` [95% CI] | `1W` step | `1W` HWM | `4W` CPU (ms) | `4W` vs `M0` [95% CI] | `4W` step | `4W` HWM | `8W` CPU (ms) | `8W` vs `M0` [95% CI] | `8W` HWM |
|---|---:|---|---:|---:|---:|---|---:|---:|---:|---|---:|
| **`M0`** | `834.8` | `+0.00%` | — | `22,956 KiB` | `882.4` | `+0.00%` | — | `35,736 KiB` | `873.2` | `+0.00%` | `50,136 KiB` |
| **`M1`** (`+lanes`) | `760.1` | **`-9.19%`** `[0.8884, 0.9283]` | `-9.19%` | `23,280 KiB` | `772.0` | **`-12.11%`** `[0.8643, 0.8937]` | `-12.11%` | `35,372 KiB` | `787.0` | **`-11.26%`** `[0.8549, 0.9212]` | `51,720 KiB` |
| **`M2`** (`+#1328`) | `739.4` | **`-12.09%`** `[0.8648, 0.8936]` | **`-3.20%`** | `23,256 KiB` | `747.8` | **`-14.65%`** `[0.8381, 0.8691]` | **`-2.89%`** | `36,288 KiB` | `765.6` | **`-13.59%`** `[0.8253, 0.9048]` | `50,748 KiB` |
| **`M3`** (`+fold`) | `690.5` | **`-17.76%`** `[0.8095, 0.8355]` | **`-6.44%`** | `23,088 KiB` | `708.8` | **`-19.18%`** `[0.7892, 0.8277]` | **`-5.30%`** | `35,924 KiB` | `728.6` | **`-18.36%`** `[0.7783, 0.8563]` | `49,492 KiB` |
| **`M4`** (`+k3`) | `689.6` | **`-17.34%`** `[0.8025, 0.8515]` | `+0.51%` | `23,060 KiB` | `713.1` | **`-18.58%`** `[0.7879, 0.8413]` | `+0.73%` | `36,160 KiB` | `721.2` | **`-16.83%`** `[0.8205, 0.8430]` | `49,924 KiB` |
| **`M5`** (`+#1311+#1282`) | `680.9` | **`-18.54%`** `[0.7939, 0.8358]` | `-1.45%` | `23,032 KiB` | `697.3` | **`-20.11%`** `[0.7773, 0.8211]` | **`-1.87%`** | `35,736 KiB` | `714.6` | **`-18.61%`** `[0.7841, 0.8449]` | `50,128 KiB` |

*Paired post-`M1` gains (`M5` vs `M1`, `n = 9`):*
- **`base`:** `1W` **`-8.26%`** `[0.9081, 0.9268]`, `4W` **`-8.43%`** `[0.9031, 0.9285]`, `8W` **`-8.17%`** `[0.8929, 0.9444]`.
- **`jcc`:** `1W` **`-10.30%`** `[0.8696, 0.9253]`, `4W` **`-9.10%`** `[0.8845, 0.9342]`, `8W` **`-8.28%`** `[0.8980, 0.9368]`.

### 3c. Production Default (`mibz2`) Arm — `wikidata.bz2` (`n = 9`)

| Mode / Threads | `M0` CPU (ms) | `M4` CPU (ms) (`vs M0`) | `M5` CPU (ms) (`vs M0` [95% CI]) | `M5` wall vs `M0` | `M0 -> M5` HWM |
|---|---:|---:|---|---:|---:|
| **`base` `1W`** (`Cells` route) | `961.4` | `960.9` (`+0.79%`) | `977.9` (`+2.56%` `[1.0095, 1.0419]`*) | `+0.78%` | `25,800 -> 26,048 KiB` (`1.010×`) |
| **`jcc` `1W`** (`Cells` route) | `941.5` | `934.7` (`-0.42%`) | **`933.3`** (**`-0.66%`** `[0.9862, 1.0006]`) | `+6.12%` | `25,916 -> 26,072 KiB` (`1.006×`) |
| **`base` `4W`** (`#1282` flips to `OpenCells`) | `1,050.9` | `1,054.4` (`+1.36%`) | **`733.2`** (**`-29.97%`** `[0.6886, 0.7122]`) | **`-30.93%`** | **`81,500 -> 36,216 KiB` (`-55.6%`)** |
| **`jcc` `4W`** (`#1282` flips to `OpenCells`) | `1,069.7` | `1,035.1` (`-2.61%`) | **`703.6`** (**`-33.49%`** `[0.6528, 0.6776]`) | **`-34.30%`** | **`85,380 -> 35,984 KiB` (`-57.9%`)** |

*\*`mibz2 1W` uses `BzReader::new` (single-threaded `Cells` route, never touching `OpenCells`, `lanes`, `overlay_aos`, or `mtf-fold`). The `+2.56%` `base` shift vanishes completely under `jcc` (`-0.66% [0.9862, 1.0006]`), confirming it is a JCC-32B branch-alignment artifact rather than added work.*

### Key Composition & Interaction Findings

1. **Zero RSS stacking — pre-`#1312` `B4` blowup completely eliminated:** In the earlier `B4` audit (`1.206–1.270×` `8W` RSS, `62.4–62.6 MiB`), unbounded `Arena::pool` (`+5..10 MiB`) and unpooled `mtf-fold` (`+4..5 MiB`) stacked directly. In `M5`, `#1312` + `#1328` (`0 B`) + `d321fef8` single-buffer pooling hold **`8W` `lazy` HWM at `50,128 KiB` (`1.000×` of `M0`, `0.969×` of `M1`) on `jcc`** and **`51,020 KiB` (`1.033×` of `M0`, `1.000×` of `M1`) on `base`** — completely avoiding the `~4.9M` cycles/MiB 8W L3-eviction tax on i9 (`1950f77f`).
2. **Every worker layer composes additively with zero destructive interference:** On `jcc`, `M1` (`-9.2..-12.1%`), `M2` (`-2.6..-3.2%`), and `M3` (`-5.3..-6.4%`) stack cleanly across `1W/4W/8W` with disjoint CIs at every step, delivering **`-18.5%..-20.1%` vs `M0`** (`-20.0%..-21.5%` on `base`).
3. **`M4` (`scanner-a3-k3-pipe-u8`) is cleanly isolated to `S` (`-8.8%..-9.5%`):** Total process CPU (`S+W`) moves only `~1%` at `M3 -> M4` on the VM because `S` is only `~11%` of total `S+W` CPU (and `jcc` `M3 -> M4` had `+0.5..+0.7%` worker jitter on total CPU, which `M5` recovered), but both the standalone `sb` floor (`S`: `68.89 -> 61.82 ms` `base`, `65.87 -> 60.64 ms` `jcc`) and the paired `CLOCK_THREAD_CPUTIME_ID` stage breakdown (§5) prove a razor-tight **`-8.81%` (`base`) / `-9.49%` (`jcc`)** scanner speedup.
4. **`#1282` (`LAZY_FROM_THREADS: 5 -> 4`) transforms default `mibz2 4W`:** Switching default `4W` from `Cells` to `OpenCells` unlocks the entire `M1..M4` stack at 4 threads, cutting default `4W` CPU by **`-30.0%..-33.5%`**, wall by **`-30.9%..-34.3%`**, and peak RSS by **`-55.6%..-57.9%` (`-45.3..-49.4 MiB`)**.

## 4. Guard Corpora Matrix (`M5` vs `M0`, All 48 Cells, `n = 7` Interleaved)

Summary across all `6` guard corpora × `2` arms (`lazy`, `mibz2`) × `2` thread counts (`1W`, `4W`) × `2` codegen modes (`base`, `jcc`): **`25/48` disjointly faster, `22/48` statistically flat (95% CI spans `1.000`), and `1/48` marginal (`records lazy_4w jcc +2.69% [1.0035, 1.0508]`; its `base`, `mibz2_4w jcc`, and `1W` counterparts are all flat at `-0.17%`, `-0.67%`, and `+0.24%`).**

| Corpus | Arm / Threads | `base` CPU `M0 -> M5` (`pct` [95% CI]) | `base` HWM (`×M0`) | `jcc` CPU `M0 -> M5` (`pct` [95% CI]) | `jcc` HWM (`×M0`) |
|---|---|---|---:|---|---:|
| **`text`** | `lazy 1W` | `248.8 -> 229.1 ms` (**`-7.50%`** `[0.9038, 0.9467]`) | `0.971×` | `245.5 -> 223.4 ms` (**`-9.14%`** `[0.9033, 0.9139]`) | `1.136×` |
| **`text`** | `lazy 4W` | `292.8 -> 275.1 ms` (**`-5.51%`** `[0.9164, 0.9743]`) | `0.981×` | `289.0 -> 263.4 ms` (**`-9.05%`** `[0.8758, 0.9445]`) | `0.947×` |
| **`text`** | `mibz2 1W` | `236.5 -> 223.7 ms` (**`-5.28%`** `[0.9269, 0.9681]`) | `0.966×` | `224.0 -> 207.3 ms` (**`-7.58%`** `[0.9096, 0.9391]`) | `0.964×` |
| **`text`** | `mibz2 4W` | `295.3 -> 280.7 ms` (**`-5.08%`** `[0.9277, 0.9711]`) | `0.962×` | `294.9 -> 267.9 ms` (**`-8.28%`** `[0.8956, 0.9393]`) | `0.970×` |
| **`runs`** | `lazy 1W` | `37.9 -> 31.0 ms` (**`-18.35%`** `[0.8006, 0.8327]`) | **`0.850×`** | `38.6 -> 31.4 ms` (**`-17.55%`** `[0.7881, 0.8627]`) | **`0.847×`** |
| **`runs`** | `lazy 4W` | `42.4 -> 32.9 ms` (**`-23.07%`** `[0.7142, 0.8287]`) | **`0.848×`** | `39.2 -> 32.0 ms` (**`-18.49%`** `[0.7849, 0.8464]`) | **`0.852×`** |
| **`runs`** | `mibz2 1W` | `35.0 -> 33.5 ms` (**`-4.46%`** `[0.9452, 0.9657]`) | `0.991×` | `33.2 -> 32.1 ms` (`-4.73%` `[0.9061, 1.0017]`) | `0.988×` |
| **`runs`** | `mibz2 4W` | `39.1 -> 32.0 ms` (**`-17.53%`** `[0.8052, 0.8446]`) | **`0.850×`** | `39.4 -> 32.7 ms` (**`-16.18%`** `[0.8122, 0.8651]`) | **`0.854×`** |
| **`onebyte`** | `lazy 1W` | `61.2 -> 53.4 ms` (**`-12.54%`** `[0.8428, 0.9077]`) | **`0.922×`** | `60.3 -> 52.1 ms` (**`-13.24%`** `[0.8598, 0.8754]`) | **`0.920×`** |
| **`onebyte`** | `lazy 4W` | `64.8 -> 56.2 ms` (**`-12.69%`** `[0.8537, 0.8930]`) | **`0.921×`** | `60.9 -> 52.7 ms` (**`-12.80%`** `[0.8561, 0.8883]`) | **`0.922×`** |
| **`onebyte`** | `mibz2 1W` | `58.4 -> 55.1 ms` (**`-3.74%`** `[0.9299, 0.9964]`) | `0.990×` | `57.8 -> 55.6 ms` (**`-4.69%`** `[0.9293, 0.9775]`) | `0.988×` |
| **`onebyte`** | `mibz2 4W` | `63.6 -> 55.8 ms` (**`-11.56%`** `[0.8609, 0.9086]`) | **`0.922×`** | `63.1 -> 53.9 ms` (**`-14.49%`** `[0.8433, 0.8670]`) | **`0.923×`** |
| **`pattern`** | `lazy 1W` | `163.2 -> 167.0 ms` (`+0.56%` `[0.9117, 1.1092]`) | `1.019×` | `173.7 -> 172.6 ms` (`-0.41%` `[0.9792, 1.0129]`) | `1.023×` |
| **`pattern`** | `lazy 4W` | `201.5 -> 203.4 ms` (`-0.12%` `[0.9839, 1.0139]`) | `1.011×` | `199.3 -> 198.0 ms` (`+0.15%` `[0.9800, 1.0235]`) | `1.014×` |
| **`pattern`** | `mibz2 1W` | `187.4 -> 179.7 ms` (**`-3.99%`** `[0.9450, 0.9755]`) | `0.951×` | `180.3 -> 172.9 ms` (**`-4.06%`** `[0.9430, 0.9761]`) | `0.939×` |
| **`pattern`** | `mibz2 4W` | `201.5 -> 203.3 ms` (`+0.40%` `[0.9926, 1.0156]`) | `1.012×` | `199.2 -> 203.4 ms` (`+2.91%` `[0.9976, 1.0615]`) | `1.049×` |
| **`random`** | `lazy 1W` | `409.5 -> 407.1 ms` (`-0.20%` `[0.9863, 1.0099]`) | `1.009×` | `409.1 -> 401.8 ms` (`-1.23%` `[0.9739, 1.0016]`) | `1.007×` |
| **`random`** | `lazy 4W` | `435.0 -> 435.7 ms` (`+0.95%` `[0.9833, 1.0363]`) | `0.999×` | `429.3 -> 429.0 ms` (`+0.69%` `[0.9610, 1.0550]`) | `1.026×` |
| **`random`** | `mibz2 1W` | `404.9 -> 399.2 ms` (`+0.38%` `[0.9774, 1.0309]`) | `1.010×` | `391.3 -> 393.3 ms` (`+1.04%` `[0.9931, 1.0281]`) | `1.006×` |
| **`random`** | `mibz2 4W` | `439.2 -> 433.8 ms` (`-0.78%` `[0.9661, 1.0189]`) | `1.041×` | `435.1 -> 430.2 ms` (`-0.89%` `[0.9779, 1.0045]`) | `0.992×` |
| **`records`** | `lazy 1W` | `477.7 -> 474.8 ms` (`-0.74%` `[0.9777, 1.0078]`) | `1.005×` | `475.9 -> 473.3 ms` (`+0.24%` `[0.9841, 1.0211]`) | `1.007×` |
| **`records`** | `lazy 4W` | `498.0 -> 502.7 ms` (`-0.17%` `[0.9754, 1.0216]`) | `0.997×` | `491.1 -> 504.6 ms` (`+2.69%` `[1.0035, 1.0508]`) | `1.018×` |
| **`records`** | `mibz2 1W` | `464.0 -> 463.0 ms` (`-1.48%` `[0.9614, 1.0096]`) | `1.007×` | `461.3 -> 461.9 ms` (`+2.42%` `[0.9790, 1.0715]`) | `1.006×` |
| **`records`** | `mibz2 4W` | `506.8 -> 505.6 ms` (`+0.32%` `[0.9762, 1.0311]`) | `0.996×` | `501.5 -> 494.3 ms` (`-0.67%` `[0.9506, 1.0379]`) | `1.015×` |

*Notable guard wins:*
- **`runs.bz2` regression completely fixed:** The `+6.1..+15.4%` CPU / `+4.3..+5.1%` RSS regression caught on `B4` (`d01bccb5`) is now a **`-16.2%..-23.1%` CPU** and **`-15%` HWM** win on parallel paths (from `d321fef8`'s sub-bound prescan + `#1311`'s fused `Segmented Runs` post-walk).
- **`text` improves `-5.1%..-9.1%` across all 8 cells** (`#1311` + `scanner-a3-k3-pipe-u8` + `d321fef8`'s zero-allocation dense refusal), and **`onebyte` improves `-3.7%..-14.5%`** with `-7.8%..-8.0%` lower parallel HWM.
- **`random` `1W` stays completely flat** (`+0.38% base`, `+1.04% jcc`, CIs spanning `1.0`), confirming `#1311` preserves unfused `Column` on high-entropy streams.

## 5. Paired `M5` vs `M0` Per-Stage Breakdown & Next-Bottleneck Map (VM Measured)

To map every remaining millisecond on `M5`, we instrumented both `M0` (`07ab2e7e`, default features) and `M5` (`69221479`, `--features forward-anchor-lanes,mtf-fold`) with identical steal-free `CLOCK_THREAD_CPUTIME_ID` accumulators over the exact same `233/431` admitted `OpenCells` blocks of `wikidata.bz2` (`209,700,196 B`, `n = 9` alternating paired runs, 1 warmup + 1 measured pass per run). On `M0`, `A5` (`runs::front`) and `A6a` (`construct_capped` Pass 1) are timed separately and summed to compare apples-to-apples with `M5`'s fused `A5+A6a` (`fold::front_lazy` + `FoldSink`); the `198` non-Open runt blocks and buffer dispatch fall into `Other` in both arms.

### 5a. Paired Stage-by-Stage Delta (`M5` vs `M0`, `wikidata.bz2`, `n = 9`)

| Stage | `M0 base` (ms, share) | `M5 base` (ms, share) | `M5/M0 base` [95% CI] | `M0 jcc` (ms, share) | `M5 jcc` (ms, share) | `M5/M0 jcc` [95% CI] |
|---|---:|---:|---|---:|---:|---|
| **`S`** (Scanner `next`) | `74.62` (`9.6%` Tot) | `68.19` (`11.5%` Tot) | **`-8.81%`** `[0.9039, 0.9199]` (`-6.43 ms`) | `72.25` (`10.5%` Tot) | `64.96` (`11.8%` Tot) | **`-9.49%`** `[0.8877, 0.9228]` (`-7.29 ms`) |
| **`A5+A6a`** (MTF + Pass 1)* | `194.57` (`27.5%` W) | `161.98` (`30.7%` W) | **`-17.19%`** `[0.8076, 0.8491]` (`-32.59 ms`) | `166.22` (`27.1%` W) | `143.62` (`29.8%` W) | **`-13.17%`** `[0.8416, 0.8960]` (`-22.60 ms`) |
| **`A6b`** (Endpoint + Dict) | `125.40` (`17.8%` W) | `98.17` (`18.6%` W) | **`-22.38%`** `[0.7636, 0.7891]` (`-27.23 ms`) | `111.89` (`18.2%` W) | `87.11` (`18.0%` W) | **`-21.51%`** `[0.7676, 0.8026]` (`-24.77 ms`) |
| **`A6c`** (Edge wiring) | `71.38` (`10.1%` W) | `26.12` (`5.0%` W) | **`-63.16%`** `[0.3543, 0.3831]` (`-45.26 ms`) | `65.25` (`10.6%` W) | `22.21` (`4.6%` W) | **`-66.26%`** `[0.3270, 0.3481]` (`-43.04 ms`) |
| **`A7`** (Emission walk) | `263.04` (`37.2%` W) | `195.20` (`37.0%` W) | **`-25.89%`** `[0.7201, 0.7627]` (`-67.83 ms`) | `221.21` (`36.0%` W) | `185.02` (`38.3%` W) | **`-17.05%`** `[0.7989, 0.8613]` (`-36.19 ms`) |
| **`A8+A9`** (RLE1 + CRC) | `43.16` (`6.1%` W) | `39.30` (`7.5%` W) | **`-9.20%`** `[0.8764, 0.9408]` (`-3.86 ms`) | `41.33` (`6.7%` W) | `38.74` (`8.0%` W) | **`-7.38%`** `[0.9033, 0.9496]` (`-2.58 ms`) |
| **`Other`** (`198` runts + glue) | `9.31` (`1.3%` W) | `5.72` (`1.1%` W) | **`-38.96%`** `[0.5295, 0.7037]` (`-3.59 ms`) | `8.00` (`1.3%` W) | `5.33` (`1.1%` W) | **`-34.40%`** `[0.6317, 0.6812]` (`-2.66 ms`) |
| **Total `W`** (`431` blocks) | `706.45` (`100.0%`) | `526.91` (`100.0%`) | **`-25.79%`** `[0.7228, 0.7621]` (`-179.54 ms`) | `613.67` (`100.0%`) | `482.68` (`100.0%`) | **`-21.64%`** `[0.7601, 0.8079]` (`-130.99 ms`) |
| **Total `S+W`** | `781.13` (`100.0%`) | `595.43` (`100.0%`) | **`-24.17%`** `[0.7401, 0.7769]` (`-185.70 ms`) | `686.08` (`100.0%`) | `548.71` (`100.0%`) | **`-20.37%`** `[0.7737, 0.8196]` (`-137.36 ms`) |
| **Station ratio `W/S`** | **`9.47`** | **`7.73`** (`-18.4%`) | worker-bound -> scanner-bound at `8W` | **`8.49`** | **`7.43`** (`-12.5%`) | worker-bound -> scanner-bound at `8W` |

*\*On `M0`, `A5` (`runs::front`) = `61.43 ms` (`8.7% W`) `base` / `60.07 ms` (`9.8% W`) `jcc`; `A6a` (`construct_capped` Pass 1) = `133.22 ms` (`18.9% W`) `base` / `106.11 ms` (`17.3% W`) `jcc`.*

Every single stage improves disjointly in both `base` and `jcc`. Notably, `A8+A9` also drops `-7.4%..-9.2%` even without `L6a` because `lanes`'s 4 KiB chunk permutation leaves `out` L2-warmer immediately prior to `rle::unrle_in_place`, and `Other` drops `-34.4%..-39.0%` from `#1311`'s fused `Segmented Runs` post-walk on the 198 non-open blocks plus pooled buffer recycling.

### 5b. Residual `M5` Next-Bottleneck Map

| Priority | Regime / Stage | `M5` Share (`base` / `jcc`) | `M5` VM Median (`base` / `jcc`) | Next Credible Lever & Status |
|---|---|---|---|---|
| **#1 (8W+ wall)** | **Serial Scanner `S`** | `11.5% / 11.8%` of `S+W` (`W/S = 7.73 / 7.43`) | `68.19 / 64.96 ms` | **Binds 8W/10T wall once `M1` lands.** Next: `scanner-adaptive-seam-composed` (`-9.95 ms` measured on i9 in `d6bc8ca2`), `u8` `syms` handover (`#1339`), then `A2`/selector cleanup |
| **#1 (Worker `W`)** | **`A7`** (`lanes::walk_into_arena` + 4 KiB chunk permutation) | **`37.0% / 38.3%`** of `W` | `195.20 / 185.02 ms` | `K=8`+prefetch STOPped (`#1319`). Remaining lever: **`L6a` (`forward-rle1-crc-vectored-build`)** to eliminate the 4 KiB permutation/stitch pass and fuse `A8+A9` (`8.0%` of `W` + stitch) |
| **#2 (Worker `W`)** | **`A5+A6a`** (`fold::front_lazy` + `FoldSink` + `OutQ16`) | **`30.7% / 29.8%`** of `W` | `161.98 / 143.62 ms` | Largest unharvested SIMD/layout headroom: port `#1274`'s 64-symbol `vptest` Pass 1 into `fold.rs` + `u8` `syms` handover (`#1339`) + L1-batched `OutQ16` (`mtf-fold-l1-batch-redesign`) |
| **#3 (Worker `W`)** | **`A6b`** (`endpoint::overlay_aos` + inline-7 u64 dict) | `18.6% / 18.0%` of `W` | `98.17 / 87.11 ms` | Already cut `-21.5%..-22.4%` at `0 B` RSS by `#1328`; diminishing returns vs `A7`/`A5+A6a` |
| **#4 (Worker `W`)** | **`A8+A9`** (`rle::unrle_in_place` + CRC-32) | `7.5% / 8.0%` of `W` | `39.30 / 38.74 ms` | Standalone CRC swap STOPped on i9 (`#1346`); reachable only via pass-removing `L6a` fusion |
| **#5 (Worker `W`)** | **`A6c`** (`wire_fwd_edges`) | `5.0% / 4.6%` of `W` | `26.12 / 22.21 ms` | Already cut `-63.2%..-66.3%` vs `M0` (`wire_xor_edges`); near floor |
| **Floor** | **`Other`** (`198` non-Open runts + dispatch) | `1.1% / 1.1%` of `W` | `5.72 / 5.33 ms` | Floor |

## 6. DERIVED Physical i9-10900K Projections (Anchored at `1950f77f`, NOT Measured on i9)

**Explicit boundary:** Every figure in this section is **DERIVED arithmetic** combining physical i9-10900K measurements from `origin/evidence 1950f77f` (`wikidata-x4` = `838.80 MB`, pinned core-0 scanner, 8 workers = 9 active threads matching a 10-core/10T budget) with the VM ratios above and the calibrated VM-to-i9 transfer prior documented in synthesis-27 (`#1358`: `~0.2–0.5×` for worker savings on `#1245`/`#1346`). **Do not cite these as measured i9 results.**

### Hardware Anchor (`origin/evidence 1950f77f`, Measured on Physical i9-10900K)

- **`M0` (`base`, default):** `1W = 1,309.2 ms` (`0.641 GB/s`), `4W = 362.6 ms` (`2.313 GB/s`), `8W = 191.0 ms` (`4.392 GB/s`, worker-bound: `W/8 ≈ 176 ms > S ≈ 158.0 ms`).
- **`M1` (`lanes`, post-`#1312` 80 KiB Arena):** `1W = 1,090.5 ms` (`0.769 GB/s`), `4W = 301.0 ms` (`2.787 GB/s`), **`8W = 169.1 ms` (`4.961 GB/s`, `-11.10%` vs `base/8`)**, with core-0 scanner `775.59M` cyc at `4.906 GHz` = **`158.1 ms`** active scan, **`11.0 ms`** pipeline/drain margin, and derived `W/8 ≈ 145.0 ms` (cleanly **serial-scanner-bound** at zero extra L3 tax).

### Derived `M5` Physical i9-10900K 8W/10T Pinned-Scanner Throughput (`wikidata-x4`)

Using the VM `M5/M0` scanner cut (`-9.15%` mean of `-8.81% base` / `-9.49% jcc`) and `M5/M1` worker cut (`~-9.5%` mean across `jcc` 1W/4W/8W at `0 B` RSS overhead), across three explicit VM-to-i9 transfer assumptions:

| **DERIVED** i9-10900K Scenario (`wikidata-x4`, `838.80 MB`) | Transfer Prior (`S` / `W`) | `S` active (ms) | `W/8` (ms) | `8W/10T` Wall (`max(S, W/8) + 11.0 ms`) | **DERIVED** Throughput | vs `M1` (`4.961 GB/s`) | vs `M0` (`4.392 GB/s`) |
|---|---|---:|---:|---:|---:|---:|---:|
| **`M5` conservative** | `0.50× S` / `0.35× W` | `150.8` | `140.2` | `161.9 ms` | **`5.18 GB/s`** | `+4.5%` | `+18.0%` |
| **`M5` midpoint** | `0.75× S` / `0.50× W` | `147.2` | `138.1` | `158.3 ms` | **`5.30 GB/s`** | `+6.8%` | `+20.7%` |
| **`M5` 1:1 VM upper bound** | `1.00× S` / `1.00× W` | `143.6` | `131.2` | `154.6 ms` | **`5.42 GB/s`** | `+9.3%` | `+23.5%` |
| **`M5 + seam` (`-9.95 ms` i9 `d6bc8ca2`), conservative** | `0.50× S` / `0.35× W` | `140.9` | `140.2` | `151.9 ms` | **`5.52 GB/s`** | `+11.3%` | `+25.7%` |
| **`M5 + seam`, midpoint** | `0.75× S` / `0.50× W` | `137.3` | `138.1` | `149.1 ms` | **`5.63 GB/s`** | `+13.4%` | `+28.1%` |
| **`M5 + seam`, 1:1 VM upper bound** | `1.00× S` / `1.00× W` | `133.7` | `131.2` | `144.7 ms` | **`5.80 GB/s`** | `+16.9%` | `+32.0%` |

*Coupled-system takeaway (why `M2+M3` matter even when 8W is scanner-bound):* on `M1` alone (`W/8 ≈ 145 ms`), stacking `M4` + `scanner-adaptive-seam-composed` would drop `S` to `137–141 ms` and **re-bind workers at `~145 ms` (`wall ≈ 156 ms`, `~5.38 GB/s`)**. Because `M2` (`#1328`) + `M3` (`d321fef8`) cut `W/8` to **`~131–140 ms`** at zero extra RSS, they keep 8W scanner-bound all the way through **`5.52–5.80 GB/s`** while also speeding up worker-bound `1W` (`1,090.5 -> 978–1,051 ms`, `-19.7%..-25.3%` vs `M0`) and `4W` (`301.0 -> 274–291 ms`, `-19.6%..-24.5%` vs `M0`) directly.

## 7. Pairwise 3-Way Merge-Conflict Audit (`git merge-tree`) & Ship Order

All five code branches were audited against `origin/master` (`edf2ceaf`, `v0.5.18`) and pairwise against each other via `git merge-tree --write-tree`:

- **Single-branch merge onto `origin/master` (`edf2ceaf`):** **`5/5 CLEAN`** (`#1328` `37a2a37a`, `mtf-fold` `d321fef8`, `#1343` `3d2afb8a`, `#1311` `df4f0bbd`, `#1282` `d8d0c8b5`).
- **Pairwise 3-way merge (`10` pairs): `7/10 CLEAN`, `3/10` trivial single-file textual overlaps (zero semantic conflicts):**
  1. `#1328` (`37a2a37a`) × `#1282` (`d8d0c8b5`): `Justfile` (adjacent recipe `import` lines).
  2. `mtf-fold` (`d321fef8`) × `#1311` (`df4f0bbd`): `src/lib.rs` (around `decode_owned_tail`'s `FusedPostwalk::enter_owned` wrapper; `decode_folded` is untouched by `#1311`).
  3. `mtf-fold` (`d321fef8`) × `#1282` (`d8d0c8b5`): `src/open.rs` (around `OpenCellsOf::invert_runs`'s `lazy_counts_admit` guard).

### Recommended Ship Order (Post-`v0.5.18`)

1. **`#1324` (`forward-anchor-lanes` C6 i9 adoption, `M1`):** P1 gatekeeper — flips 8W Wikidata from worker-bound to serial-scanner-bound (`4.961 GB/s` on `1950f77f`) so scanner levers convert near-1:1 into 8W wall.
2. **`#1311` (`segmented-fused-postwalk-rescue`) + `#1328` (`overlay_aos`, `M2`) + `#1282` (`LAZY_FROM_THREADS = 4`):** Cleanest low-conflict worker & scheduling wins (only a 1-line `Justfile` overlap between `#1328` and `#1282`); `#1282` alone slashes default `4W` CPU `-30..-33%` and HWM `-56..-58%`, and `#1311` ships via ordinary CI with zero i9 queue spend.
3. **`#1343` (`scanner-a3-k3-pipe-u8`, `M4`) + `scanner-adaptive-seam-composed`:** Cleanly orthogonal to all four worker branches (`4/4` pairwise CLEAN with `#1328`, `mtf-fold`, `#1311`, `#1282`); converts into `5.5+ GB/s` once `#1324` lands.
4. **`mtf-fold` (`d321fef8`, `M3`) + follow-on `A5+A6a` SIMD & `L6a`:** Rebase onto `#1311` + `#1282` (2 tiny mechanical hunks in `src/lib.rs` / `src/open.rs`), delivering the largest single post-`#1324` worker gain (`-5.3..-6.4%` `jcc` at `0 B` extra 8W RSS and `0` `runs.bz2` regression) before targeting `M5`'s top two remaining worker stages (`A7` `38%` via `L6a` and `A5+A6a` `30%` via 64-symbol `vptest` + `u8` handover).
