# Preview: Composed `mtf-fold` (`d321fef8`) + `F8` (`4fe4f865`) + `bitmap_prescan_bmi2` + Lazy `thin[a]`

**Scope & honesty notice:** All timings in this note were measured on a shared 2-vCPU Linux KVM VM (`rustc 1.99.1`, `[profile.release]` `opt-level = 3`, `lto = "fat"`, `codegen-units = 1`, `others_busy` ~ `1.0–1.7` from concurrent VM workloads). Both process CPU time (`cpu`, immune to scheduler preemption) and wall time (`wall`) are reported side by side. **These are local VM preview measurements, NOT physical i9-10900K C6 adoption evidence.** The composed implementation and frozen C6 harness (`bench-mtf-fold-i9`) live on local branch `build/mtf-fold-full-f8` (`16f37aa2`, unpushed pending post-release integration).

**Headline VM preview (`n = 11` alternating AB/BA pairs, 100% SHA-256 exact vs C `libbzip2`):**
- **Default shipped decoder vs `--features mtf-fold`:** `wikidata/1W` **`-4.87%` CPU** `[0.9271, 0.9762]` (`base`) / **`-7.84%` CPU** `[0.8979, 0.9459]` (`JCC-32B`); `wikidata/8W` **`-5.06%` CPU** `[0.9365, 0.9625]` (`base`, RSS **`0.9391x`**, `-3.2 MiB`) / **`-7.25%` CPU** `[0.9169, 0.9383]` (`JCC-32B`, RSS **`0.9489x`**).
- **Stacked on `#1312` (`--features forward-anchor-lanes` vs `--features forward-anchor-lanes,mtf-fold`):** `wikidata/1W` **`-7.48%` CPU** `[0.9132, 0.9374]` (`base`, `732.9 -> 679.4 ms`) / **`-8.09%` CPU** `[0.9070, 0.9313]` (`JCC-32B`, `682.1 -> 623.6 ms`); `wikidata/8W` **`-7.45%` CPU** (`base`, RSS **`0.9777x`**) / **`-7.50%` CPU** (`JCC-32B`, RSS **`0.9628x`**, `-2.1 MiB`). Combined `lanes + fold` cuts `wikidata/1W` CPU by **`-17.9%`** (`827.9 -> 679.4 ms`, `base`) and **`-19.0%`** (`769.5 -> 623.6 ms`, `JCC-32B`).
- **Guards & RSS:** The earlier `+16%` `runs.bz2` regression is completely eliminated (`-0.55%` / `-0.02%` `base`, `-2.08%` / `+1.61%` `JCC-32B`), and the `+4.2..+5.0 MiB` 8W dual-buffer RSS stacking is replaced by a net RSS reduction on `wikidata/8W` (`0.939x–0.978x`). One watchpoint under `#1312`: `random/1W` CPU reads `+2.87%` (`base`) / `+1.87%` (`JCC-32B`), to be adjudicated directly on physical i9 under the frozen `<= 1.020` C6 guard clause.

---

## 1. Composed Mechanism Stack (`--features mtf-fold`)

The candidate composes four mechanisms behind `--features mtf-fold` (default build unchanged):

1. **Streaming admission & single-buffer output pooling (`d321fef8`):**
   - Aligns `runs::fold::refused()` to `OpenCells`'s `0.21` dense ceiling (`ceil(21 * max_len / 100)`).
   - Pre-screens dense-band (`lits >= ceil(max_len / 10)`) and sparse/sub-bound (`lits * 64 < max_len || end * 14 < max_len`) injective blocks over Pass 1's `(lit, one)` bitmaps *before* acquiring an output buffer or entering Pass 2, reusing Pass 1 bitmaps on `Outcome::NotOpen` (`runs::front_with_bitmaps` + `invert_runs_not_open`).
   - Lazily borrows a recycled output buffer from `free_out` (`decode_owned_pooled`) and returns `b.syms` on admission, eliminating dual-buffer 8W RSS stacking.
2. **64-symbol `u64 vptest` Pass 1 (`4fe4f865`, `#1274` port):**
   - Builds `(lit, one)` 64 symbols/iter using `_mm256_subs_epu16(x, eob - 1)` so a single `vptest` proves both no-EOB and `s < b.eob` with only 2 trailing zero words (`1` `vzeroupper`, `0` YMM spills).
   - Default-target Miri caught a missing `if b.eob <= 1 { return Err(Outcome::Bailed); }` guard in `front_scalar_with_syms`, now fixed to match `build_bitmaps_avx2`.
3. **Hoisted `Slot8` `OutQ16` + `NO_MARK` batching + lazy `thin[a]` (`src/open/outq.rs` [`#![forbid(unsafe_code)]`], `src/open.rs`):**
   - Returns `(cnt, batch_rows)` from `fill_batch_avx2`, split-borrows `out: &mut [u8]` and packed 8-byte `Slot8 { cur, end }` from cold `FoldMeta`, hoists both `NO_EPOCH` and `NO_MARK` per 512-run batch, and keeps `FoldMeta::push_slow` 4-rank unrolled and `grow_edges_vec` cold.
   - Derives `thin[a]` lazily via `FoldMeta::thin_of(slots[a], a)` from `cap_thin[a]` and `Slot8` only on thick runs, epoch cuts, and block end—removing all `thin[a]` loads/adds/stores from `flush_batch_fast`'s 28-instruction thin-run hot loop.
4. **Runtime-gated BMI1/BMI2/LZCNT bitmap prescan (`bitmap_prescan_bmi2`):**
   - `#[target_feature(enable = "bmi1,bmi2,lzcnt")]` prescan with inner `while w != 0` and `_bzhi_u64` (`0` `vzeroupper`, `0` `ymm`), matching the scalar reference on dense-band, sparse, sub-bound (`runs.bz2`), and bail edge cases.

---

## 2. Correctness, Safety & Harness Verification

- **Lint & tests:** `cargo fmt --check`, 5/5 feature `clippy --all-targets -- -D warnings`, all unit and integration tests, `bench/test_mtf_fold_i9.py` (`13/13`), and `ruff check` all pass.
- **Miri (`0 UB`):**
  - Default target (`--features mtf-fold`): **8/8 pass, `0 UB`** (`open::b5::miri_open_kernel_small_columns`, `open::outq::tests::miri_fold_meta_slot8_parity_and_boundaries`, `open::outq::tests::miri_outq_wire_and_wire_xor`, `runs::fold::tests::miri_bitmap_prescan_bmi2_matches_scalar`, `runs::fold::tests::miri_fold_above_eob_and_eob_le_1`, `runs::tests::fold_tests::{miri_fold_front_edges, miri_fold_front_is_the_records_maximal_runs, miri_fold_open_kernel_is_the_shipped_path}`).
  - AVX2+BMI1+BMI2+LZCNT+POPCNT (`RUSTFLAGS="-C target-feature=+avx2,+bmi1,+bmi2,+lzcnt,+popcnt"`, `--features mtf-fold`): **8/8 pass, `0 UB`** (same fold/outq/prescan suite plus `runs::tests::miri_runs_avx2_kernels_selected`, with `miri_fold_open_kernel_is_the_shipped_path` exercising both scalar and AVX2+BMI2 kernels: `forced [6, 0, 6], production [2, 5, 5]`).
- **Exactness vs C `libbzip2` (`bz2.open`):**
  - **112 / 112** untimed `AB_SHA256=1` `ab child` checks across all 8 distinct release binaries × 7 corpora (`wikidata` `209,700,196 B`, `text` `15,109,558 B`, `records` `9,600,000 B`, `random` `8,000,000 B`, `runs` `10,000,235 B`, `pattern` `19,600,000 B`, `onebyte` `20,000,000 B`) × `{1W, 8W}`: **100% byte- and SHA-256-exact**.
  - **1,232 / 1,232** timed repeats across all 4 paired comparisons: zero errors, 100% exact `out` and 64-bit `hash`.
- **Live C6 harness check (`bench-mtf-fold-i9`):**
  - `python3 bench/mtf_fold_i9.py run <out> --smoke` exited `0` (`smoke_ok: True`, `VOID`), `python3 bench/mtf_fold_i9.py score <out>` exited `1` (`VOID`), and `bash bench/bundle.sh <out>` produced a `11 KiB` tarball containing all 18 expected artifacts.

---

## 3. Disassembly Audit (`objdump -dC`, `base` & `JCC-32B`)

| Symbol | `base` insns (`vzu / ymm / xmm`) | `JCC-32B` insns (`vzu / ymm / xmm`) | Notes |
|---|---:|---:|---|
| `mibz2::runs::fold::fill_batch_avx2` | `285` (`0 / 0 / 21`) | `321` (`0 / 0 / 21`) | Pure 128-bit XMM; `second_cold` out-of-line |
| `<mibz2::open::outq::FoldMeta>::push_slow` | `393` (`0 / 0 / 39`) | `428` (`0 / 0 / 39`) | 4-rank unrolled cold path, `#![forbid(unsafe_code)]` |
| `mibz2::open::grow_edges_vec` | `123` (`0 / 0 / 0`) | `142` (`0 / 0 / 0`) | Cold thick-strip growth helper |
| `mibz2::runs::fold::bitmap_prescan_bmi2` | `256` (`0 / 0 / 0`) | `276` (`0 / 0 / 0`) | Pure GPR + `_bzhi_u64` (`bmi1,bmi2,lzcnt`) |
| `<mibz2::open::FoldSink>::flush_batch_fast` | `204` (`0 / 0 / 14`) | `199` (`0 / 0 / 14`) | **28-insn** thin-run hot loop; **`0` `thin[a]` memory ops** |
| `<mibz2::open::FoldSink>::flush_batch_epoch` | `207` (`0 / 0 / 14`) | `224` (`0 / 0 / 14`) | Epoch-checking batch path |
| `<mibz2::open::FoldSink>::flush_batch_marked` | `237` (`0 / 0 / 16`) | `269` (`0 / 0 / 16`) | Cold marked-row batch path |
| `mibz2::runs::fold::build_bitmaps_avx2` | `298` (`1 / 38 / 3`) | `322` (`1 / 38 / 3`) | Single `vzeroupper` at function exit |

---

## 4. Powered Paired Benchmarks (`n = 11` Alternating Pairs, All 7 Corpora, 1W & 8W)

All 1W runs use `mibz2-workers-lazy` pinned to CPU 0 (exercising `OpenCells` directly); all 8W runs use production `mibz2`. Ratios are paired geometric means (`cand / base`) with two-sided 95% Student-t CIs (`df = 10`); RSS ratios are paired medians (`cand / base`).

### 4.1 Baseline x86-64 (`base`): `default` vs `--features mtf-fold`

| Cell | Base CPU (ms) | Cand CPU (ms) | CPU Delta [95% CI] | Faster | Base Wall (ms) | Cand Wall (ms) | Wall Delta [95% CI] | RSS Base / Cand (KiB) | RSS Ratio |
|---|---:|---:|---|---:|---:|---:|---|---:|---:|
| **`wikidata/1W`** | `827.9` | `780.5` | **`-4.87%`** `[0.9271, 0.9762]` | `10/11` | `1312.8` | `1237.6` | **`-9.93%`** `[0.8157, 0.9944]` | `25028 / 24740` | `0.9920x` |
| **`wikidata/8W`** | `929.1` | `876.9` | **`-5.06%`** `[0.9365, 0.9625]` | `11/11` | `835.2` | `770.4` | **`-6.14%`** `[0.8911, 0.9887]` | `53844 / 50588` | **`0.9391x`** |
| `text/1W` | `120.3` | `118.1` | `-1.45%` `[0.9675, 1.0038]` | `7/11` | `218.0` | `164.6` | `-10.36%` `[0.7326, 1.0968]` | `20412 / 20544` | `1.0076x` |
| `text/8W` | `201.0` | `196.2` | `-1.69%` `[0.9352, 1.0335]` | `5/11` | `219.3` | `209.3` | `+0.08%` `[0.8918, 1.1232]` | `80968 / 81264` | `1.0143x` |
| `records/1W` | `235.7` | `237.0` | `+0.75%` `[0.9884, 1.0269]` | `3/11` | `335.9` | `399.9` | `+8.75%` `[0.9275, 1.2750]` | `32588 / 32712` | `1.0038x` |
| `records/8W` | `284.6` | `285.6` | `+1.23%` `[0.9806, 1.0450]` | `6/11` | `387.6` | `384.5` | `+3.30%` `[0.9557, 1.1166]` | `73076 / 75432` | `1.0515x` |
| `random/1W` | `202.0` | `201.1` | `+0.26%` `[0.9926, 1.0126]` | `6/11` | `267.2` | `285.9` | `+6.05%` `[0.9110, 1.2346]` | `29244 / 29228` | `0.9997x` |
| `random/8W` | `248.4` | `251.7` | `+0.89%` `[0.9932, 1.0249]` | `4/11` | `325.2` | `349.4` | `+5.49%` `[0.9745, 1.1419]` | `71236 / 72108` | `1.0252x` |
| `runs/1W` | `29.7` | `30.0` | `-0.55%` `[0.9738, 1.0157]` | `5/11` | `46.7` | `48.1` | `-5.51%` `[0.7026, 1.2708]` | `23072 / 22428` | `0.9914x` |
| `runs/8W` | `30.0` | `29.9` | `-0.02%` `[0.9707, 1.0297]` | `6/11` | `92.9` | `78.4` | `-11.25%` `[0.7138, 1.1033]` | `19556 / 19552` | `0.9904x` |
| `pattern/1W` | `59.5` | `59.7` | `+0.56%` `[0.9709, 1.0415]` | `6/11` | `84.7` | `96.7` | `-1.45%` `[0.7860, 1.2356]` | `12812 / 12736` | `0.9838x` |
| `pattern/8W` | `139.0` | `139.9` | `-0.11%` `[0.9666, 1.0322]` | `5/11` | `149.0` | `161.2` | `+3.89%` `[0.9174, 1.1767]` | `68356 / 67552` | `0.9882x` |
| `onebyte/1W` | `40.9` | `41.6` | `+1.96%` `[0.9967, 1.0430]` | `3/11` | `56.5` | `70.1` | `+7.27%` `[0.7451, 1.5445]` | `33000 / 32160` | `0.9946x` |
| `onebyte/8W` | `41.6` | `42.1` | `+1.53%` `[1.0017, 1.0291]` | `2/11` | `116.0` | `135.0` | `+1.03%` `[0.7791, 1.3100]` | `29508 / 29452` | `1.0011x` |

### 4.2 JCC-32B-Aligned (`-C llvm-args=-x86-branches-within-32B-boundaries`): `default` vs `--features mtf-fold`

| Cell | Base CPU (ms) | Cand CPU (ms) | CPU Delta [95% CI] | Faster | Base Wall (ms) | Cand Wall (ms) | Wall Delta [95% CI] | RSS Base / Cand (KiB) | RSS Ratio |
|---|---:|---:|---|---:|---:|---:|---|---:|---:|
| **`wikidata/1W`** | `769.5` | `714.1` | **`-7.84%`** `[0.8979, 0.9459]` | `11/11` | `2137.3` | `1894.7` | `-9.48%` `[0.8070, 1.0153]` | `24840 / 24684` | `0.9899x` |
| **`wikidata/8W`** | `858.5` | `790.1` | **`-7.25%`** `[0.9169, 0.9383]` | `11/11` | `806.0` | `755.1` | **`-7.53%`** `[0.8940, 0.9564]` | `52508 / 51700` | **`0.9489x`** |
| `text/1W` | `126.1` | `124.3` | `+0.36%` `[0.9606, 1.0485]` | `5/11` | `356.9` | `328.8` | `+2.03%` `[0.8528, 1.2208]` | `20484 / 20208` | `0.9938x` |
| `text/8W` | `199.3` | `201.1` | `+0.84%` `[0.9603, 1.0589]` | `4/11` | `244.1` | `242.6` | `+1.42%` `[0.9375, 1.0972]` | `80852 / 81416` | `0.9995x` |
| `records/1W` | `239.7` | `237.6` | `-0.50%` `[0.9742, 1.0162]` | `6/11` | `578.0` | `635.1` | `+4.83%` `[0.9749, 1.1273]` | `32580 / 32684` | `1.0032x` |
| `records/8W` | `280.5` | `284.1` | `-0.16%` `[0.9833, 1.0136]` | `6/11` | `431.3` | `421.2` | `-0.88%` `[0.9322, 1.0540]` | `72352 / 72772` | `1.0096x` |
| `random/1W` | `203.0` | `202.9` | `-0.31%` `[0.9781, 1.0161]` | `5/11` | `537.1` | `501.5` | `-4.77%` `[0.8024, 1.1302]` | `29312 / 29228` | `0.9971x` |
| `random/8W` | `249.5` | `249.4` | `-1.33%` `[0.9651, 1.0088]` | `6/11` | `354.1` | `368.9` | `+1.20%` `[0.9192, 1.1141]` | `72440 / 72452` | `0.9853x` |
| `runs/1W` | `30.8` | `30.3` | `-2.08%` `[0.9418, 1.0180]` | `5/11` | `79.8` | `75.7` | `-4.91%` `[0.8287, 1.0911]` | `23268 / 22720` | `0.9935x` |
| `runs/8W` | `30.3` | `30.7` | `+1.61%` `[0.9878, 1.0453]` | `6/11` | `85.4` | `87.6` | `+5.54%` `[0.8766, 1.2707]` | `21452 / 21296` | `0.9976x` |
| `pattern/1W` | `66.6` | `67.2` | `+1.43%` `[0.9366, 1.0985]` | `5/11` | `163.6` | `183.4` | `+6.39%` `[0.8718, 1.2985]` | `12804 / 12716` | `0.9909x` |
| `pattern/8W` | `139.1` | `142.2` | `-0.06%` `[0.9781, 1.0211]` | `3/11` | `169.1` | `156.4` | `-6.93%` `[0.8580, 1.0096]` | `66852 / 68540` | `1.0201x` |
| `onebyte/1W` | `41.7` | `43.0` | `+1.02%` `[0.9989, 1.0215]` | `3/11` | `111.9` | `111.6` | `-3.71%` `[0.8100, 1.1447]` | `32384 / 32420` | `1.0011x` |
| `onebyte/8W` | `42.6` | `42.6` | `+0.27%` `[0.9725, 1.0338]` | `4/11` | `142.1` | `133.7` | `+1.64%` `[0.8374, 1.2335]` | `31368 / 29488` | `0.9667x` |

### 4.3 Baseline x86-64 (`base`) on `#1312`: `forward-anchor-lanes` vs `forward-anchor-lanes,mtf-fold`

| Cell | Base CPU (ms) | Cand CPU (ms) | CPU Delta [95% CI] | Faster | Base Wall (ms) | Cand Wall (ms) | Wall Delta [95% CI] | RSS Base / Cand (KiB) | RSS Ratio |
|---|---:|---:|---|---:|---:|---:|---|---:|---:|
| **`wikidata/1W`** | `732.9` | `679.4` | **`-7.48%`** `[0.9132, 0.9374]` | `11/11` | `1473.2` | `1302.3` | **`-10.31%`** `[0.8360, 0.9623]` | `25112 / 24892` | `0.9916x` |
| **`wikidata/8W`** | `828.9` | `750.8` | **`-7.45%`** `[0.9117, 0.9396]` | `11/11` | `695.1` | `628.8` | **`-5.25%`** `[0.9191, 0.9767]` | `52624 / 52072` | `0.9777x` |
| `text/1W` | `122.9` | `120.2` | `-0.52%` `[0.9680, 1.0224]` | `8/11` | `234.7` | `232.8` | `-3.56%` `[0.8144, 1.1420]` | `20500 / 20488` | `0.9955x` |
| `text/8W` | `197.2` | `197.1` | `-2.87%` `[0.9342, 1.0098]` | `7/11` | `196.4` | `195.1` | `-1.89%` `[0.9060, 1.0625]` | `80348 / 79288` | `0.9869x` |
| `records/1W` | `235.1` | `240.8` | `+0.98%` `[0.9827, 1.0376]` | `3/11` | `447.7` | `482.1` | `+2.52%` `[0.9080, 1.1576]` | `32616 / 32788` | `1.0047x` |
| `records/8W` | `285.2` | `288.7` | `+1.79%` `[1.0007, 1.0355]` | `3/11` | `314.7` | `309.2` | `+0.49%` `[0.9770, 1.0335]` | `77972 / 74136` | `0.9528x` |
| `random/1W` | `203.9` | `211.0` | `+2.87%` `[1.0132, 1.0444]` | `1/11` | `381.2` | `423.4` | `+5.30%` `[0.8395, 1.3209]` | `29292 / 29336` | `1.0029x` |
| `random/8W` | `252.0` | `254.2` | `+2.72%` `[1.0029, 1.0521]` | `2/11` | `269.3` | `284.3` | `+4.34%` `[0.9412, 1.1568]` | `72132 / 71980` | `1.0016x` |
| `runs/1W` | `30.4` | `30.3` | `+0.50%` `[0.9927, 1.0174]` | `6/11` | `53.2` | `51.9` | `+3.39%` `[0.9472, 1.1285]` | `22940 / 23288` | `1.0066x` |
| `runs/8W` | `30.1` | `30.2` | `+2.57%` `[0.9967, 1.0556]` | `3/11` | `74.4` | `75.8` | `+7.18%` `[0.9751, 1.1780]` | `21400 / 19740` | `0.9998x` |
| `pattern/1W` | `63.1` | `60.9` | `-3.74%` `[0.9196, 1.0077]` | `8/11` | `130.2` | `123.5` | `-11.04%` `[0.7636, 1.0365]` | `12824 / 12836` | `1.0034x` |
| `pattern/8W` | `137.0` | `139.4` | `+0.92%` `[0.9755, 1.0440]` | `4/11` | `131.1` | `143.7` | `+6.21%` `[0.9484, 1.1894]` | `67328 / 67976` | `1.0131x` |
| `onebyte/1W` | `41.9` | `42.2` | `-0.64%` `[0.9686, 1.0193]` | `7/11` | `79.3` | `76.9` | `-7.07%` `[0.8289, 1.0419]` | `31888 / 31052` | `1.0016x` |
| `onebyte/8W` | `41.8` | `41.8` | `+0.43%` `[0.9799, 1.0293]` | `6/11` | `101.8` | `95.0` | `-3.86%` `[0.7289, 1.2681]` | `30932 / 33012` | `1.0175x` |

### 4.4 JCC-32B-Aligned on `#1312`: `forward-anchor-lanes` vs `forward-anchor-lanes,mtf-fold`

| Cell | Base CPU (ms) | Cand CPU (ms) | CPU Delta [95% CI] | Faster | Base Wall (ms) | Cand Wall (ms) | Wall Delta [95% CI] | RSS Base / Cand (KiB) | RSS Ratio |
|---|---:|---:|---|---:|---:|---:|---|---:|---:|
| **`wikidata/1W`** | `682.1` | `623.6` | **`-8.09%`** `[0.9070, 0.9313]` | `11/11` | `1405.8` | `1238.3` | `-7.93%` `[0.8221, 1.0311]` | `24992 / 24884` | `0.9976x` |
| **`wikidata/8W`** | `762.3` | `695.9` | **`-7.50%`** `[0.8968, 0.9541]` | `10/11` | `558.0` | `525.2` | **`-6.40%`** `[0.8952, 0.9787]` | `55196 / 53100` | **`0.9628x`** |
| `text/1W` | `124.8` | `125.2` | `+0.92%` `[0.9719, 1.0480]` | `4/11` | `268.5` | `242.4` | `-6.95%` `[0.8178, 1.0586]` | `20496 / 20720` | `1.0086x` |
| `text/8W` | `198.3` | `199.4` | `+0.90%` `[0.9792, 1.0397]` | `4/11` | `172.3` | `181.7` | `-0.23%` `[0.9281, 1.0724]` | `80420 / 82012` | `1.0078x` |
| `records/1W` | `238.6` | `237.1` | `+0.45%` `[0.9883, 1.0208]` | `6/11` | `478.6` | `515.5` | `+3.16%` `[0.9718, 1.0952]` | `32540 / 32868` | `1.0097x` |
| `records/8W` | `282.0` | `286.3` | `+1.93%` `[0.9756, 1.0649]` | `5/11` | `259.4` | `250.6` | `-0.66%` `[0.8856, 1.1144]` | `72664 / 74932` | `0.9879x` |
| `random/1W` | `200.7` | `204.8` | `+1.87%` `[1.0073, 1.0302]` | `2/11` | `446.6` | `420.4` | `+0.12%` `[0.9098, 1.1018]` | `29216 / 29420` | `1.0064x` |
| `random/8W` | `246.9` | `244.0` | `-0.38%` `[0.9774, 1.0154]` | `5/11` | `231.1` | `235.1` | `+2.79%` `[0.9430, 1.1204]` | `70384 / 71420` | `1.0444x` |
| `runs/1W` | `30.4` | `30.6` | `+0.68%` `[0.9921, 1.0218]` | `4/11` | `58.0` | `69.2` | `+4.95%` `[0.8955, 1.2301]` | `21632 / 22520` | `1.0119x` |
| `runs/8W` | `31.4` | `31.2` | `+0.91%` `[0.9817, 1.0372]` | `4/11` | `55.3` | `53.5` | `-2.00%` `[0.8925, 1.0760]` | `19548 / 19812` | `1.0108x` |
| `pattern/1W` | `63.5` | `67.9` | `+2.05%` `[0.9709, 1.0727]` | `5/11` | `131.0` | `131.7` | `+11.26%` `[0.9211, 1.3439]` | `12708 / 12944` | `1.0179x` |
| `pattern/8W` | `138.0` | `140.9` | `+0.03%` `[0.9617, 1.0404]` | `5/11` | `112.9` | `118.3` | `+2.14%` `[0.8851, 1.1786]` | `66432 / 67268` | `1.0085x` |
| `onebyte/1W` | `43.5` | `42.8` | `-0.76%` `[0.9708, 1.0145]` | `5/11` | `111.0` | `93.1` | `-1.97%` `[0.8406, 1.1431]` | `32844 / 33108` | `1.0036x` |
| `onebyte/8W` | `43.7` | `42.4` | `-1.66%` `[0.9660, 1.0011]` | `8/11` | `81.6` | `75.3` | `-6.03%` `[0.8807, 1.0025]` | `29264 / 30876` | `1.0521x` |

---

## 5. Analysis & Next Steps

1. **Full additivity with `#1312` (`forward-anchor-lanes`):** Because `#1312` accelerates `A7` (emission), `A4 + A5` (MTF + construction Pass 1/2) becomes a larger share of total decode time; the composed `mtf-fold` win correspondingly grows from `-4.87%` / `-7.84%` on `default` to **`-7.48%` / `-8.09%`** on `#1312`.
2. **Guard & RSS recovery:** `bitmap_prescan_bmi2` + lazy `thin[a]` + `decode_owned_pooled` eliminate both previously observed blockers (`+16%` `runs.bz2` regression and `+4.2..+5.0 MiB` 8W dual-buffer RSS stacking).
3. **Physical i9 adjudication:** Once `v0.5.18` settles, open the code PR from `build/mtf-fold-full-f8`, complete one independent C6 registration review of `docs/journal/2026-11-28-mtf-fold-f8-i9/registration.md`, and queue `bench-mtf-fold-i9` on the physical i9-10900K (watching `random/1W` and `onebyte/8W` closely against the frozen `<= 1.020` wall / `<= 1.030x` RSS guard thresholds).
