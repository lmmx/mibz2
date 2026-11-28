# Explore: 4-Way Composed Post-#1217 Worker Stack & Residual B4 Stage Dissection

**Scope & honesty notice:** All timings and RSS figures in this note were measured on a shared Linux KVM VM (`rustc 1.99.1`, steal-free `CLOCK_PROCESS_CPUTIME_ID` / `CLOCK_THREAD_CPUTIME_ID`, both default baseline x86-64 `base` and `-C llvm-args=-x86-branches-within-32B-boundaries` `jcc`, `[profile.release]` `opt-level = 3`, `lto = "fat"`, `codegen-units = 1`). **These are VM exploration measurements, NOT physical i9-10900K adoption evidence.** Physical i9-10900K 8-worker projections in §6 are explicitly labelled **derived** from anchored i9 bundles (`origin/evidence e98948b6`, `ccfa473f`, `#1119`).

## 1. Motivation & 4-Layer Composition

Four independent worker advances have been built across `src/open.rs`, `src/open/outq.rs`, `src/open/lanes.rs`, `src/rle.rs`, `src/runs/fold.rs`, and `src/lib.rs`, none of which had been composed end-to-end on the same `OpenCells` hot path:

| Layer | Scratch SHA | Mechanism | Status on `origin/master` (`1e4d2be7`) |
|---|---|---|---|
| **`B0`** | `4bb65910` | Shipped default bidirectional `OpenCells` walk | Base before `#1294` |
| **`B1`** | `4bb65910` | `+ --features forward-anchor-lanes` (`#1110` + `#1217`, `#![forbid(unsafe_code)]` disjoint-slab 4-lane forward walk) | Opt-in feature; C6 i9 `DO NOT ADOPT` on `Arena::pool` RSS (`#1296`), P1 `forward-emission-bounded-slab-rescue` filed |
| **`B2`** | `1d0b6885` | `B1 + a6c-wire-redesign` (`624a915e`, `#![forbid(unsafe_code)]` `Cell` 4× `wire_xor` + register-cursor overlapping `wire_fwd`) | Merged as `#1294` (`0e9b59ff`) |
| **`B3`** | `995fc2b8` | `B2 + L6a` (`1cd3129d`, zero-stitch slab AVX2 RLE1 + L2-hot 64 KiB CRC-32 in `src/open/lanes.rs` + `src/rle.rs`) | Local prototype (`forward-rle1-crc-vectored-build`) |
| **`B4`** | `5c0ae451` | `B3 + mtf-fold-guard-rescue` (`d01bccb5`, `--features forward-anchor-lanes,mtf-fold`, dense-band bitmap prescan + bitmap reuse, **wired directly into `L6a`'s `lanes::walk_fused_same`**) | Local prototype (`mtf-fold-streaming-admission`) |

### Critical `B4` Composition Wiring Note
Out of the box, `mtf-fold`'s `OpenCellsOf::fold` called `self.open.walk(...)` followed by `rle::unrle_in_place` in `decode_folded`, which would silently bypass `L6a` (`lanes::walk_fused_same`). In scratch commit `5c0ae451`, we resolved the `src/lib.rs` `decode_owned` interaction (`if fold_not_open { invert_runs_not_open } else { invert_runs_with_rle }`) and added `InverseBwt::fold_runs_with_rle`, wiring `lanes::walk_fused_same` directly into `OpenCellsOf::fold_inner` so `B4` genuinely executes all four advances together on every admitted Wikidata block (`233/233` Open blocks folded, `0` bailed, `0` refused). Because `#1294` (`a6c-wire-redesign`) already parameterized `XorReaderOf<C, STRIDE>` and `ReaderOf<C, STRIDE>` over `<C, STRIDE>`, `finish_with::<true, true, { outq::FOLD_CHUNK }, { outq::FOLD_STRIDE }>` compiled with zero changes.

## 2. Exactness & SHA-256 Parity

Every binary ran through the production `BzReader::parallel_workers(Cursor::new(data), w, SLOTS_PER_THREAD * w, Kernels::detect(), true)` path (matching `ab`'s `mibz2-workers-lazy`) against C `bzip2 -dc` across all 7 corpora (`wikidata`, `text`, `records`, `random`, `runs`, `pattern`, `onebyte`):

- **`base`:** `19/19 PASS` (`B0..B4` on `wikidata` `1W` + `B4` on all 7 corpora at `1W` and `4W`).
- **`jcc`:** `19/19 PASS` (same matrix).
- **Total:** **`38/38 PASS` (100% SHA-256 bit-exact)**.

## 3. Step-by-Step Ablation on `wikidata.bz2` (`n = 9` Interleaved Repeats)

Pure scanner CPU (`Scanner::next` on CPU 0, `n = 7` medians): `S_base = 74.13 ms [72.53..76.87]`, `S_jcc = 70.72 ms [70.21..74.47]`. Worker CPU is reported as both raw process CPU (`CLOCK_PROCESS_CPUTIME_ID`) and scanner-subtracted `W ≈ cpu - S_med` (exact `decode_owned` `CLOCK_THREAD_CPUTIME_ID` is cross-checked in §5). Paired ratios are geometric means with 95% Student-t CIs over `n = 9` alternating-order rounds.

### Baseline x86-64 (`base`)

| Arm | `1W` CPU (ms) | `1W` vs `B0` [95% CI] | `1W` `W` (ms) | `W` vs `B0` | `W` step | `1W` RSS | `4W` CPU (ms) | `4W` `W` vs `B0` | `4W` `W` step | `4W` RSS |
|---|---:|---|---:|---:|---:|---:|---:|---:|---:|---:|
| **`B0`** | `974.1` | `+0.00%` | `899.9` | `+0.00%` | — | `27,140 KiB` | `1,031.9` | `+0.00%` | — | `35,352 KiB` |
| **`B1`** (`+#1217`) | `821.8` | **`-16.46%`** `[0.8196, 0.8515]` | `747.7` | **`-17.79%`** | `-17.79%` | `27,740 KiB` | `843.9` | **`-19.66%`** | `-19.66%` | `40,132 KiB` |
| **`B2`** (`+#1294`) | `836.4` | `-14.77%` `[0.8309, 0.8741]` | `762.2` | `-15.97%` | **`+2.21%`** | `28,192 KiB` | `849.0` | `-18.94%` | **`+0.89%`** | `39,732 KiB` |
| **`B3`** (`+L6a`) | `845.8` | `-14.62%` `[0.8244, 0.8844]` | `771.7` | `-15.80%` | `+0.21%` | `27,988 KiB` | `859.1` | `-18.70%` | `+0.30%` | `39,908 KiB` |
| **`B4`** (`+fold`) | **`756.6`** | **`-23.58%`** `[0.7412, 0.7879]` | **`682.4`** | **`-25.49%`** | **`-11.51%`** | `28,840 KiB` | **`777.7`** | **`-26.76%`** | **`-9.91%`** | `42,072 KiB` |

### JCC-32B-Aligned (`jcc`, branch-alignment noise controlled)

| Arm | `1W` CPU (ms) | `1W` vs `B0` [95% CI] | `1W` `W` (ms) | `W` vs `B0` | `W` vs `B1` | `W` step [95% CI] | `4W` CPU (ms) | `4W` `W` vs `B0` | `4W` `W` vs `B1` | `4W` `W` step [95% CI] |
|---|---:|---|---:|---:|---:|---|---:|---:|---:|---|
| **`B0`** | `882.6` | `+0.00%` | `811.8` | `+0.00%` | `+12.21%` | — | `882.2` | `+0.00%` | `+12.82%` | — |
| **`B1`** (`+#1217`) | `769.7` | **`-9.99%`** `[0.8767, 0.9240]` | `699.0` | **`-10.88%`** | `+0.00%` | **`-10.88%`** `[0.8658, 0.9172]` | `788.9` | **`-11.36%`** | `+0.00%` | **`-11.36%`** `[0.8741, 0.8988]` |
| **`B2`** (`+#1294`) | `751.2` | `-12.01%` `[0.8613, 0.8989]` | `680.5` | `-13.07%` | `-2.45%` | `-2.45%` `[0.9475, 1.0043]` | `789.5` | `-11.54%` | **`-0.20%`** | **`-0.20%`** `[0.9898, 1.0064]` |
| **`B3`** (`+L6a`) | `745.1` | `-12.84%` `[0.8509, 0.8929]` | `674.4` | `-13.97%` | `-3.47%` | `-1.04%` `[0.9589, 1.0212]` | `773.2` | `-13.20%` | `-2.07%` | **`-1.88%`** `[0.9652, 0.9974]` |
| **`B4`** (`+fold`) | **`707.4`** | **`-19.00%`** `[0.7931, 0.8272]` | **`636.7`** | **`-20.68%`** | **`-11.00%`** | **`-7.80%`** `[0.8969, 0.9479]` | **`730.7`** | **`-18.27%`** | **`-7.80%`** | **`-5.85%`** `[0.9243, 0.9591]` |

### 8-Worker `OpenCells` RSS & CPU Scaling (`wikidata.bz2`, `n = 5`)

| Arm | `8W base` RSS (`×B0` / `×B1`) | `8W base` CPU | `8W jcc` RSS (`×B0` / `×B1`) | `8W jcc` CPU |
|---|---:|---:|---:|---:|
| **`B0`** | `49,092 KiB` (`1.000×` / `0.823×`) | `1,041.1 ms` | `51,928 KiB` (`1.000×` / `0.914×`) | `923.5 ms` |
| **`B1`** (`+#1217`) | `59,664 KiB` (**`1.215×`** / `1.000×`) | `854.2 ms` | `56,836 KiB` (**`1.095×`** / `1.000×`) | `829.0 ms` |
| **`B2`** (`+#1294`) | `57,740 KiB` (`1.176×` / `0.968×`) | `870.4 ms` | `59,340 KiB` (`1.143×` / `1.044×`) | `809.2 ms` |
| **`B3`** (`+L6a`) | `57,368 KiB` (`1.169×` / `0.962×`) | `853.2 ms` | `58,456 KiB` (`1.126×` / `1.029×`) | `785.9 ms` |
| **`B4`** (`+fold`) | **`62,364 KiB`** (**`1.270×`** / `1.045×`) | **`814.1 ms`** | **`62,620 KiB`** (**`1.206×`** / `1.102×`) | **`745.7 ms`** |

### Key Ablation Findings
1. **`B4` (`mtf-fold` wired into `L6a`) is the dominant post-`#1217` worker lever:** On top of `B3`, `mtf-fold` cuts worker CPU by **`-7.80%` (`1W jcc`) / `-5.85%` (`4W jcc`)** and **`-11.51%` (`1W base`) / `-9.91%` (`4W base`)** — all disjoint faster — bringing cumulative `B4/B0` to **`-20.68%` (`1W jcc`) / `-18.27%` (`4W jcc`)** and **`-25.49%` / `-26.76%` (`base`)**, and `B4/B1` (post-`#1217`) to **`-11.00%` (`1W jcc`) / `-7.80%` (`4W jcc`) / `-10.0%` (`8W jcc` CPU)**.
2. **`#1294` (`a6c-wire-redesign`, `B1 -> B2`) is near-zero once `forward-anchor-lanes` ships:** On `B0` (bidirectional XOR walk), `finish_with` calls `wire_xor_edges`, where `#1294`'s `Cell` 4× `XorReaderOf` delivers its speedup. Once `forward-anchor-lanes` ships, `wire_xor_edges` executes on **0/233** Open blocks; only `wire_fwd_edges` runs, and §5 shows `A6c` (`wire_fwd_edges`) is already just **`5.2%`** (`31.0 ms`) of `B4` worker CPU. Thus `B1 -> B2` yields **`-0.20%` (`4W jcc`, flat)** / `-2.45%` (`1W jcc`, overlapping 1.0) and `+0.89%..+2.21%` on `base`.
3. **`L6a` (`B2 -> B3`) scales up under multi-worker L3 pressure at zero extra RSS:** Under `jcc`, `L6a` improves from `-1.04%` (`1W`) to **`-1.88%` `[0.9652, 0.9974]` disjoint faster (`4W`)** and **`-2.9%` (`8W` CPU, `809.2 -> 785.9 ms`)** while reducing `8W` RSS (`-884 KiB` `jcc`, `-372 KiB` `base`). Under `base`, Haswell JCC-32B shift masks the small `~15 ms` `A8+A9` cut inside `A7`/`A5+A6a`.
4. **CRITICAL PRE-SHIP RSS CATCH (stacks directly with `#1296`):** `B1` (`#1217` unbounded `Arena::pool`) adds `+4.9..+10.3 MiB` at `8W` (the exact root cause of `#1296`'s i9 C6 `DO NOT ADOPT` at `+6.71 MiB` / `1.108×`), and `B4` (`mtf-fold`) stacks **another `+4.2..+5.0 MiB` (`~525–625 KiB/worker`)** on top of `B3` (`57.4 -> 62.4 MiB` `base`, `58.5 -> 62.6 MiB` `jcc`, **`1.206–1.270× B0`**) because `fold_inner` keeps a second `~956 KiB` `s.ll` buffer sized to `bound + INLINE` alongside `out`. Therefore `forward-emission-bounded-slab-rescue` (`P1`, eliminating the retained `Arena::pool`) MUST land first, and `mtf-fold` must avoid retaining a dual `956 KiB` `s.ll` buffer across blocks before queueing on the i9.

## 4. Guard Corpora (`B4` vs `B0` & `B1`, `n = 5`) & Pre-Ship `runs.bz2` Catch

| Corpus | Mode | `1W` `B4/B0` | `1W` `B4/B1` | `1W` RSS (`×B1`) | `4W` `B4/B0` | `4W` `B4/B1` | `4W` RSS (`×B1`) | `fold` (`f/b/no`) |
|---|---|---:|---:|---:|---:|---:|---:|---|
| **`text`** | `base` / `jcc` | `-2.15%` / `+1.53%` | **`-0.49%`** / **`+0.40%`** | `1.020×` / `0.979×` | `+1.43%` / `-0.48%` | `+2.90%` / **`-0.60%`** | `1.024×` / `0.930×` | `0/0/14` |
| **`records`** | `base` / `jcc` | `-0.64%` / `-7.25%` | `+0.23%` / `-0.54%` | `1.002×` / `1.003×` | `+0.98%` / `+0.03%` | `-0.99%` / `+0.69%` | `0.974×` / `1.004×` | `0/0/0` |
| **`random`** | `base` / `jcc` | `-6.97%` / `-0.25%` | `-0.61%` / `+0.49%` | `1.003×` / `1.003×` | `-5.50%` / `+1.32%` | `+0.02%` / `+0.60%` | `0.994×` / `1.007×` | `0/0/0` |
| **`pattern`** | `base` / `jcc` | `-7.51%` / `+0.14%` | `-9.15%` / `-1.37%` | `1.000×` / `1.000×` | `-3.41%` / `-1.77%` | `+1.87%` / `-3.57%` | `1.044×` / `0.987×` | `0/0/0` |
| **`onebyte`** | `base` / `jcc` | `-23.85%` / `-0.58%` | `+0.60%` / `+1.92%` | `1.005×` / `1.006×` | `-6.04%` / `-1.91%` | `-3.89%` / `+1.02%` | `1.003×` / `1.002×` | `0/0/0` |
| **`runs`** | `base` / `jcc` | **`+6.14%`** / **`+7.37%`** | **`+7.74%`** / **`+8.36%`** | **`1.042×`** / **`1.043×`** | **`+15.40%`** / **`+5.54%`** | **`+12.72%`** / **`+6.69%`** | **`1.040×`** / **`1.048×`** | **`0/0/1`** |

- **`text`, `records`, `random`, `pattern`, `onebyte` (5/6 guards) are 100% clean:** `mtf-fold-guard-rescue`'s dense-band bitmap prescan refuses all `14/14` `text` blocks before `buf` allocation, turning the prior `+36..+47%` `text` regression into `-0.49%..+0.40%` (`1W`) and `-0.60%` (`4W jcc`) with `0.93–0.98×` `jcc` RSS.
- **PRE-SHIP CATCH ON `runs.bz2` (`B3 -> B4` isolation verified):** Across `B0..B3` on `runs.bz2` (`ab gen ... 4`, `10 MiB` decoded, `38 KiB` compressed), `1W` CPU is flat at `20.37–21.28 ms` and RSS is flat at `17,852–18,008 KiB`; at `B4` alone, RSS jumps `+672–880 KiB` (`1.043–1.051×`) and CPU regresses **`+7.74%..+8.36%`** (`1W`, disjoint `[1.0455, 1.1231]` `jcc`) / **`+6.69%..+12.72%`** (`4W`) with `not_open = 1`.
  - **Root cause:** `runs.bz2`'s single block with `syms >= MIN_SYMS (4096)` has `n << max_len (900,000)`, so its literal count falls below `lits_min = bound / 10` (`in_dense = false`). It therefore **skips** `d01bccb5`'s `bitmap_within_dense_cap` prescan, invokes `make_sink` to allocate and zero the `956 KiB` `buf` (driving the `+4.3–5.1%` RSS spike), runs Pass 2 + `OutQ16`, and only then fails `Self::cap(records, n, p)` (`Outcome::NotOpen`) and replays `runs::front_with_bitmaps` + `LazyCells`.
  - **Fix before shipping `mtf-fold`:** Extend the pre-allocation bitmap prescan (or check `n` / `records` from Pass 1's `(lit, one)` bitmaps whenever `lits < bound / 10` on sub-bound blocks) so `not_open` blocks with `n << bound` never allocate `buf` or run Pass 2.

## 5. Disassembly Hygiene (`objdump -d`) & `B4` Per-Stage Worker Dissection

### `objdump -d` Hygiene on `B4` (`base` & `jcc`)

| Function | `b4-base` insns / span / frame | `b4-jcc` insns / span / frame | `vzeroupper` | `ymm` | `rsp_ld / rsp_st` |
|---|---|---|---:|---:|---:|
| `QueuesOf<1008, 1024>::push_slow` | `299` / `1,263 B` / `0x458` | `310` / `1,311 B` / `0x458` | **`0`** | **`0`** | `28 / 13` |
| `QueuesOf<512, 512>::push_slow` | `289` / `1,231 B` / `0x458` | `298` / `1,279 B` / `0x458` | **`0`** | **`0`** | `30 / 13` |
| `runs::fold::fill_batch_avx2` | `256` / `1,183 B` / `0x78` | `270` / `1,215 B` / `0x78` | **`0`** | **`0`** | `22 / 17` |
| `wire_xor_edges::<512,512>` & `<1008,1024>` | `417` / `1,855 B` / `0x148` | `440` / `1,887 B` / `0x148` | **`0`** | **`0`** | `74 / 46` |
| `wire_fwd_edges::<512,512>` & `<1008,1024>` | `342` / `1,487 B` / `0xe8` | `367` / `1,535 B` / `0xe8` | **`0`** | **`0`** | `43 / 26` |
| `lanes::walk_into_arena` | `1,074` / `5,135 B` / `0x2e8` | `1,106` / `5,215 B` / `0x2e8` | **`0`** | **`0`** | `156 / 124` |
| `rle::unrle_crc_slabs_avx2::<32768>` | `937` / `3,999 B` / `0xa8` | `976` / `4,095 B` / `0xa8` | `21`* | `36` | `99 / 53` |

*\*All 21 `vzeroupper` instructions in `unrle_crc_slabs_avx2` sit strictly before external calls (`grow_out_slabs`, 64 KiB `clmul_update` / `crc::update`, `memcpy`, and cold panic paths); zero appear in the 32B AVX2 scan loop. Both `<512,512>` and `<1008,1024>` wire functions emit identical instruction counts and stack spill counts.*

### Residual Per-Stage Worker Breakdown at `B4` (`wikidata.bz2`, `233/233` Open Blocks, `431` Total Blocks)

Instrumented `B4` (`5c0ae451` with steal-free `CLOCK_THREAD_CPUTIME_ID` accumulators around the five non-overlapping stages inside `fold_inner` / `finish_with` / `walk_fused_same`, plus outer `decode_owned`, pinned CPU 0, `n = 7` medians):

| Stage | Description | `base` median (ms) `[min..max]` | `base` share | `jcc` median (ms) `[min..max]` | `jcc` share | Frontier Status |
|---|---|---:|---:|---:|---:|---|
| **`A7`** | `lanes::walk_into_arena` (`K = 4` forward lanes into 32 KiB slabs) | **`208.21`** `[203.07..213.86]` | **`31.9%`** | **`196.27`** `[188.92..201.88]` | **`33.1%`** | **#1 remaining stage; `K = 6/8` MLP + `edges[next]` prefetch UNTESTED** -> filed `forward-lanes-mlp-and-prefetch` (`idea`) |
| **`A5+A6a`** | `fold::front_lazy` + `FoldSink` (`OutQ16` + thick strips) | **`203.03`** `[194.67..205.33]` | **`31.1%`** | **`177.81`** `[173.70..183.93]` | **`30.0%`** | **#2 remaining stage** (`fold.rs` still uses pre-`#1238` 32-sym `vpermq` Pass 1; candidate for zero-`vpermq` 64-sym u64 `vptest` Pass 1 + 2-cycle XMM `STEPS` port after `runs-front-i9-rescue`) |
| **`A6b`** | `endpoint.rs` strip setup + induction + dict | **`139.22`** `[135.40..142.43]` | **`21.3%`** | **`120.49`** `[119.53..127.14]` | **`20.3%`** | `#3`; `a6b-compact-footprint-rescue` stopped in `dead-ends.tsv` (`+413..+550 KiB/w` RSS); `a6b-v2-fold` at `idea` |
| **`A8+A9`** | `rle::unrle_crc_slabs` (zero-stitch AVX2 RLE1 + 64 KiB CRC-32) | **`61.09`** `[58.56..62.90]` | **`9.3%`** | **`60.74`** `[57.58..62.64]` | **`10.3%`** | `#4`; CRC half targeted by `crc32-avx2-ring-rescue` (`#1298` queued on i9) |
| **`A6c`** | `wire_fwd_edges` (`ReaderOf<1008,1024>`) | **`34.42`** `[31.95..35.10]` | **`5.3%`** | **`31.02`** `[29.53..31.97]` | **`5.2%`** | `#5`; shrunk from ~15% (`wire_xor`) to 5.2% (`wire_fwd`) |
| **Other** | 198 non-Open runts + buffer swap/dispatch | `7.52` | `1.2%` | `6.27` | `1.1%` | Floor |
| **Total `W`** | Full `decode_owned` (`431` blocks) | **`653.49`** `[630.06..662.65]` | `100.0%` | **`592.60`** `[577.91..611.93]` | `100.0%` | |

## 6. Derived Physical i9-10900K Projections & Ship-Order Recommendation

Anchoring to physical i9-10900K `wikidata x4` (`838.9 MB` decoded) measurements (`origin/evidence e98948b6` & `ccfa473f`: `B0` `1W = 1,353.4 ms`, `8W real wall = 195.9 ms` / `4.28 GB/s`, `8W wkr_busy/w = 196.45 ms` none / `188.84 ms` ideal-S; `B1` `#1217` `1W = 1,113.4 ms`, `8W real wall = 184.4 ms` / `4.55 GB/s`, `8W wkr_busy/w = 169.61 ms` none / `166.37 ms` ideal-S) and applying the VM `jcc` `B4/B1` worker scaling (`-7.8%` conservative `4W` to `-11.0%` `1W`, midpoint `-9.4%`):

| Metric (`wikidata x4`, **DERIVED** i9-10900K) | Conservative (`-7.8%` vs `B1`) | Midpoint (`-9.4%` vs `B1`) | `1W`-scaled (`-11.0%` vs `B1`) |
|---|---:|---:|---:|
| **`1W` forced-open wall** (vs `B0` `1,353.4 ms`) | `1,026.6 ms` (`-24.1%`) | `1,008.7 ms` (`-25.5%`) | `990.9 ms` (`-26.8%`) |
| **`8W` `wkr_busy/w`** (`none` / `ideal-S`) | `156.4 / 153.4 ms` (`-20.4%`) | `153.7 / 150.7 ms` (`-21.8%`) | `151.0 / 148.1 ms` (`-23.2%`) |
| **`k*` vs shipped scanner (`S ≈ 184.4 ms`)** | `6.78` (deeply scanner-bound) | `6.67` (deeply scanner-bound) | `6.55` (deeply scanner-bound) |
| **`k*` vs composed scanner (`#1144 + #1117 + 6cfb56f1`, `S ≈ 152.7 ms`)** | **`8.19`** (`156.4 ms`, **`5.36 GB/s`**) | **`8.05`** (`153.7 ms`, **`5.46 GB/s`**) | **`7.91`** (`152.7 ms`, **`5.49 GB/s`**) |

**Derived takeaway:** Stacking the composed worker stack (`B4` with bounded slabs) with the composed scanner stack (`#1144` seam + `#1117` A2 + `6cfb56f1` A3, `-10.3 ms` `in_maint` and `-12.3%` `S` -> `S ≈ 152.7 ms`) places the scanner and 8 workers in **near-exact 8-worker station balance (`k* ≈ 7.91–8.19`, `~151–156 ms`, `~5.36–5.49 GB/s` derived, `+25..+28%` over shipped `4.28 GB/s`)**. Without the composed scanner stack, `B4` is deeply scanner-bound at `k = 8` (`k* ≈ 6.6–6.8`), paying off primarily at `k = 1..6` and CPU efficiency.

### Recommended Ship Order
1. **`forward-emission-bounded-slab-rescue` (`P1`) FIRST:** Eliminates `#1217`'s `+6.71 MiB` `Arena::pool` double-buffer RSS tax (`<= 128 KiB/worker` bounded slabs) and unlocks default forward-lane adoption in `v0.5.18`. (Also retires `wire_xor_edges` on all Open blocks.)
2. **`L6a` (`forward-rle1-crc-vectored-build`) SECOND (or integrated directly into bounded-slab streaming emission):** Zero extra RSS (`-0.3..-0.9 MiB` at `8W`), `-1.9%..-2.9%` at `4W–8W jcc`, and naturally pairs with bounded slabs by streaming RLE1+CRC32 out of the active slabs into `out`.
3. **`mtf-fold-streaming-admission` (`mtf-fold-guard-rescue`) THIRD, after fixing two catches exposed here:** (a) guard sub-bound `n << max_len` blocks (fixing `runs.bz2` `+6..+15%` CPU and `+4.3..+5.1%` RSS), and (b) stop retaining a dual `956 KiB/worker` `s.ll` buffer across blocks (`+4.2..+5.0 MiB` at `8W`). Delivering `-7.8%..-11.0%` worker CPU on top of `B1..B3` makes this the single highest-leverage post-`#1217` worker advance.
4. **Next worker frontier probes (`>= 10%` of `B4`):** Attack **`A7` (`33.1%` of `B4`)** via `forward-lanes-mlp-and-prefetch` (`K = 6/8` bounded `16 KiB` slabs + `edges[next]` prefetch) and **`A5+A6a` (`30.0%` of `B4`)** by porting `runs-front-i9-rescue`'s zero-`vpermq` 64-sym `vptest` Pass 1 + 2-cycle XMM `STEPS` into `src/runs/fold.rs` once both land.
