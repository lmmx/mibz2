# Composed Serial Scanner Stack Verification: `#1144` (Adaptive Seam) + `#1117` (A2 Fast Build) + `d8d8ac9e` (A3 `K = 3` `u8`-Chunk Pipelined Refill)

*Date:* 27 November 2026
*Zone:* Smooth (`docs/journal/2026-11-27-explore/composed-scanner-stack.md`, zero `src/` edits on this branch)
*Base commit (`S0`):* `926ff31b` (identical to current `origin/master` across all scanner and default worker source files: `src/bits.rs`, `src/block.rs`, `src/huff.rs`, `src/parallel.rs`, `src/stream.rs`, `src/trace.rs`)
*Hardware & environment:* 2-vCPU Intel Xeon Haswell KVM VM (`2.30 GHz`, AVX2 + BMI2), Linux, `rustc 1.99.1`, tested under **both** baseline x86-64 (`base`) and Intel JCC-32B-aligned (`jcc`, `-C llvm-args=-x86-branches-within-32B-boundaries`, Comet Lake-S stepping 5 silicon-fix control).
*Protocol:* All three scanner candidates — `#1144` (`0fda2381`, adaptive per-block lookahead + one-shot seam window), `#1117` (`eb1ec40f`, direct coalesced selector runs + multi-symbol delta code-length LUT + single-write `K = 2` table build), and `d8d8ac9e` (`scanner-a3-k3-pipe-u8`, `B = 10, K = 3` single-`u32` `u8`-chunk decode + 4-step pipelined overlapped refill) — were composed in an isolated scratch worktree (deleted after measurement) across five layers: `S0` (base), `S1` (`+ #1144`), `S2` (`+ #1144 + #1117`), `S13` (`+ #1144 + d8d8ac9e`, omitting `#1117`), and `S3` (`+ #1144 + #1117 + d8d8ac9e`).
- **Exactness:** `150 / 150` cryptographic SHA-256 parity checks against `libbzip2 1.0.8` across all 5 layers × 2 profiles × 7 standard CI corpora (`wikidata`, `text`, `records`, `random`, `runs`, `pattern`, `onebyte`) × (`t = 1`, `t = 4`) plus 8-worker `OpenCells` (`mibz2-workers-lazy t = 8`) on `wikidata`.
- **Station & phase timing:** Pinned CPU 0 (`taskset -c 0`), `n = 7` counterbalanced interleaved rounds (alternating `S0..S3` / `S3..S0`), using `LD_PRELOAD` `CLOCK_MONOTONIC → CLOCK_THREAD_CPUTIME_ID` interposition (zero binary modification, 100% identical `.text`) so `Instant::now()` inside unmodified `examples/station_balance` (`wikidata`, `text`, `records`, `random`, 2 passes/call) and `examples/phases` (`text`, 1 pass/call) measures steal-free per-thread CPU time immune to concurrent VM load.
- **Whole-decoder & multi-seam `in_maint`:** Paired counterbalanced `examples/ab child` (`n = 5` across all 7 corpora at `t = 1` pinned CPU 0 and `t = 4`, plus `n = 3` 8-worker `OpenCells` RSS audit on `wikidata`; `n = 7` on 47.4 MiB / 11-seam `wikidata-x4` at `t = 4` measuring `mibz2-scan` thread CPU = `S + in_maint`), plus direct `--features trace` `examples/occupancy` verification of `t.in_maint_ms`, `t.in_maint_spans`, `t.f.window_cap_max_mb`, and `t.f.w.ledger_violations` on `S3`.

---

## 1. Layers Composed, File Overlap, and `.text` Footprint

| Layer | Commits / PRs | Files touched & conflict audit | `.text` (`ab` base / JCC) |
|---|---|---|---:|
| **`S0`** | `926ff31b` (default scanner baseline) | baseline | `1,656,662 B` (`+0.00%`) / `1,705,558 B` |
| **`S1`** | `+ #1144` (`0fda2381`, adaptive seam window) | `src/parallel.rs`, `src/stream.rs`, `src/trace.rs` (**100% file-disjoint** from `#1117` and `d8d8ac9e`) | `1,660,262 B` (`+0.22%`) / `1,709,270 B` |
| **`S2`** | `S1 + #1117` (`eb1ec40f`, A2 fast build) | `src/block.rs` (`read_selectors` direct runs, `read_lengths` 4 KiB `DELTA_LUT`), `src/huff.rs` (`fill_lut` single-write `K = 2`) | `1,662,950 B` (`+0.38%`) / `1,712,182 B` |
| **`S13`** | `S1 + d8d8ac9e` (`scanner-a3-k3-pipe-u8`, **no `#1117`**) | `src/bits.rs` (`skip_short_shl_low4`), `src/block.rs` (`K3_GATE_SEL = 1024`, `u8` chunk), `src/huff.rs` (`B = 10, K = 3` + 4-step refill + `PatchBuf`) | `1,662,902 B` (`+0.38%`) / `1,712,054 B` |
| **`S3`** | `S1 + #1117 + d8d8ac9e` (full 3-layer stack) | **`src/huff.rs` conflict:** `d8d8ac9e`'s `K = 3` LUT layout overwrites `#1117`'s `K = 2` single-write `fill_lut`; `S3` retains `#1117`'s `A2b + A2c` (`src/block.rs`) + `d8d8ac9e`'s `src/huff.rs` & `src/bits.rs` | `1,664,694 B` (`+0.48%`) / `1,713,910 B` |

**Key structural finding on `#1117` vs `d8d8ac9e`:**
- `#1144` is completely file- and mechanism-disjoint from both `#1117` and `d8d8ac9e` (zero merge conflicts in any order).
- `#1117` and `d8d8ac9e` **conflict in `src/huff.rs` (`Huff::build_into`)** and touch adjacent lines in `src/block.rs`: `#1117`'s `A2d` single-write `fill_lut` targets the old `K = 2` (`LEN1 | SYM2 | SYM1 | bits`) entry layout, whereas `d8d8ac9e` switches to the `K = 3` (`CNT | SYM3 | SYM2 | SYM1 | 00 | bits`) layout using an in-place multi-pass `K = 1 → K = 2 → K = 3` builder (`d_arr` + `SubLut`). Even with `#1117`'s `A2d` superseded in `S3`, `#1117`'s `A2b + A2c` (`src/block.rs`) still stacks nearly multiplicatively with `d8d8ac9e`, and porting `#1117`'s single-write `fill_lut` + tail-only `SubLut` into `d8d8ac9e`'s `K = 3` builder leaves an additional `~0.2–0.9%` of `S` recoverable.

---

## 2. Executive Summary

1. **100% bit-exact parity across all 5 layers and 7 corpora:** **`150 / 150`** cryptographic SHA-256 checks against `libbzip2 1.0.8` passed (`0` mismatches, `0` errors), and traced `S3` passes the footprint ledger with `0` violations across all 5 traced corpora (`wikidata-x4`, `wikidata`, `text`, `records`, `random`).
2. **Near-exact multiplicative composability across all three serial scanner levers (`-12.7%` to `-14.8%` total `S + in_maint` on `wikidata-x4`):**
   - **Pure `in_maint` (`#1144`, `S1 / S0` on 47.4 MiB / 11-seam `wikidata-x4`, `t = 4`, `n = 7`):** cuts `mibz2-scan` thread CPU by **`-5.02%` (`base`, `[0.9210, 0.9796]`) / `-5.08%` (`jcc`, `[0.9365, 0.9621]`)** disjoint while cutting median `t = 4` RSS **`-1.7%` to `-5.8%`**. Direct `--features trace` on composed `S3` confirms exact `in_maint` preservation: **`t.in_maint_spans = 34`** (matching `#1144` standalone; `1.126 ms` steal-free thread CPU vs `~20 ms` VM / `10.69 ms` i9 on `S0`), **`window_cap_max = 4.60 MB`** (`0.27×` vs `S0`'s `16.78 MB`), **`0` ledger violations**.
   - **Pure `A2 + A3` (`station_balance` `S`, `SliceSource`, CPU 0, `n = 7`):**
     - `wikidata`: `#1117` (`S2/S1`) cuts **`-4.55%` base / `-4.39%` JCC**; `d8d8ac9e` (`S13/S1`) cuts **`-6.39%` base / `-6.29%` JCC**; stacked **`S3 / S0`** cuts **`-9.78%` base (`[0.8929, 0.9116]`) / `-10.25%` JCC (`[0.8906, 0.9045]`, `75.62 → 67.73 ms`, `3.027 → 2.711 ns/sym`)** — capturing **98.4% (`jcc`) / 99.0% (`base`)** of the independent product `S2 × S13` (`0.8958` JCC) even before porting `A2d`! This pushes `wikidata`'s operational balance point **`k* = W / S` from `8.85 → 9.78` (`+0.93` workers, JCC; `9.53 → 10.65`, base)**.
     - `text`: `#1117` (`S2/S1`) cuts **`-1.28%` base / `-3.30%` JCC**; `d8d8ac9e` (`S13/S1`) cuts **`-9.70%` base / `-10.27%` JCC**; stacked **`S3 / S0`** cuts **`-11.58%` base (`[0.8795, 0.8888]`) / `-12.43%` JCC (`[0.8661, 0.8854]`, `16.27 → 14.21 ms`, `3.322 → 2.900 ns/sym`, `k* 5.46 → 6.23`)**.
   - **Combined `in_maint + A2 + A3` (`wikidata-x4` `t = 4` `mibz2-scan`, `n = 7`):** **`S3 / S0` = `0.8541 [0.8341, 0.8747]` (`-14.59%` base, `340 → 290 ms`) / `0.8728 [0.8535, 0.8926]` (`-12.72%` JCC, limited by `ab`'s `10 ms` `thread_cpu` quantization; product of exact `S1/S0 = 0.9492` × `station_balance` `S3/S1 = 0.8975` gives `0.8519` = `-14.81%`)**!
3. **Zero `OpenCells` 8-worker L3/RSS regression (passing `<= 1.01×` law cleanly):** On `wikidata` (`mibz2-workers-lazy t = 8`), `S3` median peak RSS is **`48.01 MiB` (`0.929×` vs `S0` `51.70 MiB`, max `49.29 MiB` = `0.949×`, JCC)** and `51.35 MiB` (`1.045×` max `52.44 MiB`, base — within single-chunk feeder variance; `t = 4` `wikidata-x4` median RSS is **`0.942×` base / `0.975×` JCC**).
4. **Two concrete pre-ship findings caught by this audit:**
   - **Finding A (REAL, cross-profile) — `records` pure-scanner `S` crosses `+2.0%` when `#1117` and `d8d8ac9e` stack (`+2.48%` base / `+2.23%` JCC):** Individually both `#1117` (`S2/S1`: `+1.17%` base / `+0.29%` JCC) and `d8d8ac9e` (`S13/S1`: `+1.61%` base / `+1.30%` JCC) stay under the `2.0%` guard ceiling on high-entropy `alpha = 258` `records`, where `K = 3` triples rarely fire and `sym == 256` exercises `PatchBuf`. When composed in `S3`, `objdump -d` reveals `decode_runs_bmi2`'s inner 4-step hot loop picks up **one extra read-only stack reload (`mov 0x1f8(%rsp),%rax`, `rsp_loads: 1 → 2`)** from `#1117`'s `&[(u8, u16)]` slice loop variable, pushing stacked `S3 / S0` on `records` to **`+2.48%` base (`[1.0213, 1.0284]`) / `+2.23%` JCC (`[1.0190, 1.0257]`)**. (Whole-decoder `records` stays unaffected because `S` is only `21%` of `S + W`: `ab t = 4` `jcc` `S3 = 0.966 [0.907, 1.028]`.) Keeping the inner 4-step loop at `1` `(%rsp)` load when rebasing `d8d8ac9e` onto `#1117` (and porting `#1117`'s `A2d` single-write `fill_lut`) resolves this cleanly.
   - **Finding B (Haswell JCC-only artifact, NOT algorithmic) — `base` `t = 1` `random`/`records` `+2–4%` and `text` `MTF` `-8.7%`:** On `base`, `d8d8ac9e`'s `.text` shift moved `mtf::mtf_avx2` from `2` to `5` 32B JCC boundary crossings (and `#1144`'s `+3.6 KiB` shifted `OpenCells` `W` `+5.5%` on `base/wikidata`). Under `jcc` (Comet Lake-S stepping 5 control), `mtf_avx2` crossings drop to **`0`** across all 5 layers, `W` is **`0.9975–1.0051×`** invariant across all 4 corpora × 5 layers, `random` `S` is **`0.9979×`** (`t = 1` `1.0135 [0.956, 1.075]`), and `phases.rs` `MTF + RLE2` on `text` flattens from `0.913×` (`base`) to **`0.985×`** (`jcc`).

---

## 3. Pinned `station_balance` (`S`, `W`, `k* = W / S`, CPU 0, `n = 7` Interleaved, `CLOCK_THREAD_CPUTIME_ID`)

### 3.1 JCC-32B-Aligned (`jcc`, Comet Lake-S stepping 5 control)

| Corpus | Layer | `S` med (ms) | `S / S0` `[95% CI]` | `W` med (ms) | `W / S0` | `k* = W/S` | `ns/sym` |
|---|---|---:|---|---:|---:|---:|---:|
| `wikidata` | `S0` | `75.62` | `1.0000` | `666.80` | `1.0000` | `8.85` | `3.027` |
| `wikidata` | `S1` (`+#1144`) | `75.74` | `0.9999 [0.9934, 1.0065]` | `667.79` | `1.0007` | `8.82` | `3.032` |
| `wikidata` | `S2` (`+#1117`) | `72.45` | **`0.9560 [0.9498, 0.9622]`** (`-4.40%`) | `667.85` | `0.9986` | `9.22` | `2.901` |
| `wikidata` | `S13` (`+A3`, no `#1117`) | `70.62` | **`0.9370 [0.9292, 0.9449]`** (`-6.30%`) | `667.09` | `1.0037` | `9.46` | `2.827` |
| `wikidata` | **`S3` (full stack)** | **`67.73`** | **`0.8975 [0.8906, 0.9045]`** (**`-10.25%`**) | `662.41` | `0.9975` | **`9.78`** | **`2.711`** |
| `text` | `S0` | `16.27` | `1.0000` | `89.32` | `1.0000` | `5.46` | `3.322` |
| `text` | `S1` (`+#1144`) | `16.26` | `1.0008 [0.9901, 1.0116]` | `90.24` | `1.0056` | `5.45` | `3.319` |
| `text` | `S2` (`+#1117`) | `15.75` | **`0.9678 [0.9619, 0.9738]`** (`-3.22%`) | `89.19` | `0.9946` | `5.64` | `3.215` |
| `text` | `S13` (`+A3`, no `#1117`) | `14.56` | **`0.8980 [0.8844, 0.9119]`** (`-10.20%`) | `89.87` | `1.0032` | `6.12` | `2.972` |
| `text` | **`S3` (full stack)** | **`14.21`** | **`0.8757 [0.8661, 0.8854]`** (**`-12.43%`**) | `88.52` | `1.0082` | **`6.23`** | **`2.900`** |
| `records` | `S0 → S1 → S2 → S13 → S3` | `44.50 → 44.41 → 44.64 → 45.09 → 45.41` | `S2 1.0048`, `S13 1.0149`, **`S3 1.0223 [1.0190, 1.0257]`** | `163.14 → 163.58` | `1.0049` | `3.67 → 3.59` | `4.636 → 4.730` |
| `random` | `S0 → S1 → S2 → S13 → S3` | `35.50 → 35.90 → 35.49 → 35.46 → 35.45` | `S2 0.9952`, `S13 0.9975`, **`S3 0.9979 [0.9888, 1.0072]`** | `141.47 → 141.35` | `1.0051` | `3.98 → 3.99` | `4.437 → 4.432` |

### 3.2 Baseline x86-64 (`base`)

| Corpus | Layer | `S` med (ms) | `S / S0` `[95% CI]` | `W` med (ms) | `W / S0` | `k* = W/S` | `ns/sym` |
|---|---|---:|---|---:|---:|---:|---:|
| `wikidata` | `S0` | `78.29` | `1.0000` | `743.15` | `1.0000` | `9.53` | `3.134` |
| `wikidata` | `S1` (`+#1144`) | `78.28` | `0.9999 [0.9905, 1.0094]` | `791.11` | `1.0550` *(JCC)* | `10.02` | `3.134` |
| `wikidata` | `S2` (`+#1117`) | `74.80` | **`0.9544 [0.9458, 0.9631]`** (`-4.56%`) | `757.36` | `1.0165` | `10.13` | `2.994` |
| `wikidata` | `S13` (`+A3`, no `#1117`) | `73.49` | **`0.9360 [0.9249, 0.9473]`** (`-6.40%`) | `738.88` | `0.9906` | `10.08` | `2.942` |
| `wikidata` | **`S3` (full stack)** | **`70.75`** | **`0.9022 [0.8929, 0.9116]`** (**`-9.78%`**) | `755.29` | `1.0093` | **`10.65`** | **`2.832`** |
| `text` | `S0 → S2 → S13 → S3` | `16.57 → 16.22 → 14.93 → 14.62` | `S2 0.9849`, `S13 0.9009`, **`S3 0.8842 [0.8795, 0.8888]`** | `87.59 → 85.35` | `0.9766` | `5.29 → 5.81` | `3.383 → 2.984` |
| `records` | `S0 → S2 → S13 → S3` | `45.26 → 45.62 → 45.90 → 46.33` | `S2 1.0086`, `S13 1.0130`, **`S3 1.0248 [1.0213, 1.0284]`** | `170.66 → 168.69` | `0.9898` | `3.77 → 3.64` | `4.715 → 4.826` |
| `random` | `S0 → S2 → S13 → S3` | `35.86 → 36.09 → 35.95 → 35.62` | `S2 1.0063`, `S13 1.0011`, **`S3 0.9935 [0.9904, 0.9967]`** | `152.11 → 151.83` | `0.9958` | `4.24 → 4.26` | `4.483 → 4.453` |

### 3.3 Pairwise Incremental Layer Transitions (`S`, `station_balance`, `n = 7`)

| Transition (isolated effect) | `wikidata` base / JCC | `text` base / JCC | `records` base / JCC | `random` base / JCC |
|---|---|---|---|---|
| `S1 / S0` (`#1144` on `SliceSource`) | `-0.01%` / `-0.01%` | `-0.23%` / `+0.08%` | `-0.31%` / `+0.19%` | `+0.43%` / `+0.42%` |
| `S2 / S1` (`#1117` full `A2b+A2c+A2d`) | **`-4.55%`** / **`-4.39%`** | **`-1.28%`** / **`-3.30%`** | `+1.17%` / `+0.29%` | `+0.20%` / `-0.90%` |
| `S13 / S1` (`d8d8ac9e` `A3 K=3`, no `#1117`) | **`-6.39%`** / **`-6.29%`** | **`-9.70%`** / **`-10.27%`** | `+1.61%` / `+1.30%` | `-0.32%` / `-0.66%` |
| `S3 / S2` (`+ d8d8ac9e` on `#1117`, drops `A2d`) | **`-5.47%`** / **`-6.11%`** | **`-10.23%`** / **`-9.52%`** | `+1.61%` / `+1.75%` | `-1.27%` / `+0.28%` |
| `S3 / S13` (`+ #1117 A2b+A2c` on `d8d8ac9e`) | **`-3.62%`** / **`-4.21%`** | **`-1.86%`** / **`-2.49%`** | `+1.17%` / `+0.73%` | `-0.76%` / `+0.04%` |
| **`S3 / S0` (combined `A2b+A2c + A3 K=3`)** | **`-9.78%`** / **`-10.25%`** | **`-11.58%`** / **`-12.43%`** | **`+2.48%`** / **`+2.23%`** | **`-0.65%`** / **`-0.21%`** |

---

## 4. `phases.rs` Cross-Check (`text`, CPU 0, `n = 7`) and Direct `in_maint` Trace (`wikidata-x4`)

### 4.1 `examples/phases` (`text`, `c/B` @ 2.3 GHz, `n = 7`, `CLOCK_THREAD_CPUTIME_ID`)

| Phase | `base` `S0 → S1 → S2 → S13 → S3` (`S3/S0`) | `jcc` `S0 → S1 → S2 → S13 → S3` (`S3/S0`) |
|---|---|---|
| **`scanner (headers, Huffman)`** | `2.62 → 2.61 → 2.58 → 2.33 →` **`2.29 c/B`** (**`0.8685 [0.8471, 0.8905]`**) | `2.60 → 2.58 → 2.58 → 2.30 →` **`2.32 c/B`** (**`0.8810 [0.8428, 0.9208]`**) |
| `MTF + RLE2` | `7.05 → 7.01 → 6.80 → 6.46 → 6.40 c/B` (`0.9129`, JCC shift) | `5.68 → 5.66 → 5.68 → 5.61 → 5.62 c/B` (**`0.9850`**, invariant) |
| `T-vector build` | `5.25 → 5.17 → 5.18 → 5.14 → 5.11 c/B` (`0.9771`) | `5.10 → 5.11 → 5.19 → 5.09 → 5.09 c/B` (`0.9956`) |
| `walk, room (used)` | `7.77 → 7.67 → 7.58 → 7.60 → 7.49 c/B` (`0.9707`) | `7.84 → 7.64 → 7.84 → 7.53 → 7.64 c/B` (`0.9791`) |
| `RLE1` / `CRC` | `2.18 → 2.17` (`0.9974`) / `0.20 → 0.20` (`0.9934`) | `2.11 → 2.11` (`0.9731`) / `0.22 → 0.21` (`0.9681`) |

### 4.2 Multi-Seam `wikidata-x4` (`47.4 MiB`, 11 Seams, `ab t = 4`, `n = 7`) & Traced `in_maint` (`S3`)

| Layer | `base` `mibz2-scan` (`S + in_maint`) & `vs S0` | `jcc` `mibz2-scan` (`S + in_maint`) & `vs S0` | `base` / `jcc` med RSS (`vs S0`) |
|---|---|---|---|
| `S0` | `340 ms` (`1.0000`) | `340 ms` (`1.0000`) | `100.91` / `97.93 MiB` (`1.000×`) |
| `S1` (`+#1144`) | **`320 ms`** (**`0.9498 [0.9210, 0.9796]`**, `-5.02%`) | **`320 ms`** (**`0.9492 [0.9365, 0.9621]`**, `-5.08%`) | `99.21` (`0.983×`) / `92.22 MiB` (**`0.942×`**) |
| `S2` (`+#1117`) | **`310 ms`** (**`0.9125 [0.8978, 0.9274]`**, `-8.75%`) | **`310 ms`** (**`0.9110 [0.8959, 0.9264]`**, `-8.90%`) | `93.43` (**`0.926×`**) / `93.40 MiB` (`0.954×`) |
| `S13` (`+A3`, no `#1117`) | **`300 ms`** (**`0.8875 [0.8700, 0.9053]`**, `-11.25%`) | **`300 ms`** (**`0.8771 [0.8593, 0.8953]`**, `-12.29%`) | `95.41` (`0.946×`) / `95.52 MiB` (`0.975×`) |
| **`S3` (full stack)** | **`290 ms`** (**`0.8541 [0.8341, 0.8747]`**, **`-14.59%`**) | **`290 ms`** (**`0.8728 [0.8535, 0.8926]`**, **`-12.72%`**) | **`95.08` (`0.942×`)** / **`95.44 MiB` (`0.975×`)** |

**Direct `--features trace` `examples/occupancy` verification on composed `S3` (`t = 4, slots = 16`, steal-free `CLOCK_THREAD_CPUTIME_ID`):**
- `wikidata-x4`: **`t.in_maint_spans = 34`**, **`t.in_maint_ms = 1.126 ms`** (`0.959 ms` unpreloaded), **`t.f.window_cap_max_mb = 4.596 MB`** (`0.274×` of `S0`'s `16.78 MB`), **`t.f.w.ledger_violations = 0`**.
- `wikidata`: `7` spans, `0.248 ms`, `window_cap_max = 4.395 MB`, `0` violations.
- `text`: `1` span, `0.000 ms`, `window_cap_max = 4.194 MB`, `0` violations.
- `records`: `7` spans, `1.352 ms`, `window_cap_max = 5.374 MB`, `0` violations.
- `random`: `5` spans, `2.799 ms`, `window_cap_max = 6.461 MB`, `0` violations.

---

## 5. Whole-Decoder `ab child` Across All 7 Corpora (`n = 5` Interleaved) & 8W `OpenCells` RSS

### 5.1 Production `t = 4` (`mibz2`, all 7 CI corpora)

| Corpus | Build | `S0` CPU / max RSS | `S1` CPU/S0 | `S2` CPU/S0 | `S13` CPU/S0 | `S3` CPU/S0 `[95% CI]` & RSS/S0 |
|---|---|---|---:|---:|---:|---|
| `wikidata` | `base` | `1064.7 ms` / `90.48 MiB` | **`0.9763`** | `0.9786` | **`0.9648`** | **`0.9597 [0.9381, 0.9819]`** / `0.993×` (`89.84 MiB`) |
| `wikidata` | `jcc` | `1042.1 ms` / `92.29 MiB` | `0.9938` | `0.9229` | `0.9327` | `0.9146 [0.8019, 1.0432]` / **`0.969×`** (`89.43 MiB`) |
| `text` | `base` / `jcc` | `168.4` / `165.6 ms` | `1.014` / `1.020` | `1.006` / `1.017` | `1.012` / `1.029` | `0.973 [0.943, 1.004]` / `1.007 [0.991, 1.023]` (`0.940 / 0.991×` RSS) |
| `records` | `base` / `jcc` | `273.0` / `270.5 ms` | `1.002` / `1.018` | `1.041` / `1.010` | `1.018` / `0.978` | `1.055 [0.973, 1.143]` / `0.966 [0.907, 1.028]` (`0.994 / 0.977×` RSS) |
| `random` | `base` / `jcc` | `238.6` / `230.4 ms` | `0.977` / `0.994` | `0.976` / `1.000` | `1.012` / `1.060` | `1.011 [0.961, 1.064]` / `1.112 [0.962, 1.285]` (`1.000 / 1.000×` RSS) |
| `runs` | `base` / `jcc` | `29.6` / `35.6 ms` | `0.999` / `1.020` | `1.014` / `1.044` | `1.008` / `1.048` | `1.014 [0.979, 1.051]` / `1.040 [0.928, 1.167]` (`0.999 / 1.109×` RSS) |
| `pattern` | `base` / `jcc` | `111.4` / `115.0 ms` | `0.996` / `0.973` | `1.048` / `0.956` | `1.021` / `0.968` | `1.029 [0.996, 1.062]` / `0.957 [0.825, 1.110]` (`1.047 / 0.970×` RSS) |
| `onebyte` | `base` / `jcc` | `42.4` / `42.2 ms` | `1.001` / `1.010` | `1.003` / `1.020` | `1.016` / `1.025` | `1.082 [0.940, 1.246]` / `1.020 [0.982, 1.060]` (`1.007 / 0.999×` RSS) |

### 5.2 Sequential `t = 1` (`BzReader::new`, pinned CPU 0, `n = 5`)

- `wikidata`: `S3` CPU `0.989 [0.966, 1.012]` (`1.003×` RSS) base / `0.963 [0.903, 1.028]` (`1.001×` RSS) JCC.
- `text`: `S3` CPU `0.982 [0.963, 1.001]` (`1.004×` RSS) base / `0.984 [0.907, 1.068]` (`1.008×` RSS) JCC.
- `records`: `S3` CPU `1.021 [1.008, 1.034]` (`1.007×` RSS) base / `1.028 [0.967, 1.094]` (`1.006×` RSS) JCC.
- `random`: `S3` CPU `1.044 [1.014, 1.075]` (base JCC shift in `mtf_avx2`) → **`1.014 [0.956, 1.075]`** (`1.009×` RSS) JCC.
- `runs`, `pattern`, `onebyte`: zero disjoint regressions on `S3` across both `base` and `jcc`; all `t = 1` peak RSS within `0.997–1.018×` (`<= 1.009×` under JCC across all 7 corpora!).

### 5.3 8-Worker `OpenCells` RSS Audit (`mibz2-workers-lazy t = 8`, `wikidata`, `n = 3`)

| Layer | `base` med / max RSS (`vs S0` med) | `base` med CPU | `jcc` med / max RSS (`vs S0` med) | `jcc` med CPU |
|---|---:|---:|---:|---:|
| `S0` | `49.14` / `50.76 MiB` (`1.000×`) | `1015.7 ms` | `51.70` / `51.94 MiB` (`1.000×`) | `897.6 ms` |
| `S1` | `53.50` / `55.81 MiB` (`1.089×`, runs `48.7–55.8`) | `1018.3 ms` | `50.57` / `53.33 MiB` (`0.978×`) | `887.3 ms` |
| `S2` | `48.39` / `51.23 MiB` (`0.985×`) | `1006.5 ms` | `49.58` / `51.04 MiB` (`0.959×`) | `901.1 ms` |
| `S13` | `47.75` / `48.85 MiB` (`0.972×`) | `988.5 ms` | `49.25` / `53.37 MiB` (`0.953×`) | `885.6 ms` |
| **`S3`** | `51.35` / `52.44 MiB` (`1.045×`, runs `48.6–52.4`) | **`958.9 ms`** (`-5.6%`) | **`48.01` / `49.29 MiB` (`0.929×`)** | `888.6 ms` |

*(Peak RSS across all `t = 1` corpora is `<= 1.009×` JCC, `t = 4` `wikidata-x4` median RSS is `0.942×` base / `0.975×` JCC, and 8W `OpenCells` `wikidata` median RSS is `0.929×` JCC. Note: `#1144` raises `free_in` spare-chunk retention from `1` to `INPUT_QUEUE + 2 = 6` chunks, which explains the `±4 MiB` single-run RSS spread on `S1`/`S3` depending on feeder timing — well below `#1144`'s `1.05×` RSS bound and nowhere near `post-1080-open-admission`'s `+35..40 MiB` `Segmented` leak.)*

---

## 6. `objdump -d` Hot-Loop & Codegen Audit (`base` and `jcc`)

| Layer | `decode_runs_bmi2` main inner loop (`jcc`: bytes / insns) | `shlx` / `shrx` / `and $0x3f` | `(%rsp)` loads / stores | `vzeroupper` / `calls` | `jcc_32b_cross` (`base` / `jcc`) | `read_item_with_scratch` bytes (`base` / `jcc`) | `mtf_avx2` `jcc_32b_cross` (`base` / `jcc`) |
|---|---|---|---|---|---|---|---|
| `S0` | `398 B` / `102` (5-step `K = 2`) | `5` / `1` / `0` | `1` / `0` | `0` / `0` | `2` / `0` | `5,945` / `6,037` | `2` / `0` |
| `S1` | `398 B` / `102` (identical to `S0`) | `5` / `1` / `0` | `1` / `0` | `0` / `0` | `2` / `0` | `5,945` / `6,037` | `2` / `0` |
| `S2` | `414 B` / `104` (5-step `K = 2`) | `5` / `1` / `0` | `1` / `0` | `0` / `0` | `2` / `0` | `13,562` / `13,898` | `2` / `0` |
| `S13` | **`263 B` / `73`** (4-step `K = 3` `u8`) | **`4` / `1` / `0`** | **`1` / `0`** | **`0` / `0`** | `1` / `0` | `5,786` / `5,919` | `5` / `0` |
| **`S3`** | **`263 B` / `73`** (4-step `K = 3` `u8`) | **`4` / `1` / `0`** | **`2` / `0`** | **`0` / `0`** | `3` / `0` | `14,468` / `14,810` | `5` / `0` |

- **All BMI2 invariants survive full composition intact:** `S3` preserves exact `4 × shlx` + `1 × shrx` (overlapped refill), **`0` `and $0x3f`**, **`0` `vzeroupper`** (`mtf_avx2` keeps its `2` `vzeroupper`s), **`0` `(%rsp)` stores**, **`0` calls**, and **`0` JCC-32B crossings** under `jcc`, while shrinking the inner hot loop `-33.9%` (`398 → 263 B`).
- **Exact source of `S3`'s `+1 (%rsp)` reload:** At `0x11ee30` (`S3-jcc`), LLVM inserts one read-only `mov 0x1f8(%rsp),%rax` at the top of the 4-step inner loop (compared to `S13-jcc`'s `0x11d4b0..0x11d5b7`, which has only `cmp 0x88(%rsp),%r15`). This arises from `#1117` passing `runs: &[(u8, u16)]` to `decode_runs_bmi2` (adding one live outer-loop register) and directly explains the extra `+0.7%..+1.1%` cost on `records` in `S3` (`+2.23%`) vs `S13` (`+1.30%`).
- **`read_item_with_scratch` growth (`5.9 KiB → 14.5 KiB` in `S2`/`S3`):** LLVM inlines `#1117`'s `read_selectors`, `read_lengths`, and `Huff::build_into` into `read_item_with_scratch`; total `.text` growth across all 3 layers is still tiny (`+8.0 KiB` = `+0.48%`).

---

## 7. Concrete Ship-Order Recommendation (`#1144`, `#1117`, `scanner-a3-k3-pipe-u8`)

1. **Ship `#1144` (`scanner-adaptive-seam-window`) FIRST (already queued at `0fda2381` on `bench/workstation-queue.txt`):**
   - **100% file-disjoint** (`src/parallel.rs`, `src/stream.rs`, `src/trace.rs`) from both `#1117` and `scanner-a3-k3-pipe-u8`.
   - Delivers an orthogonal **`-5.0%`** cut to `wikidata-x4` `mibz2-scan` (`S + in_maint`), `-95%+` `in_maint` (`34` spans, `1.13 ms`), `0.27×` window capacity, and `-2.5%..-5.8%` `t = 4` RSS with zero regressions across all 7 corpora.
   - Wait for its queued i9 bundle and merge immediately on PASS (zero rebase cost for `#1117` or `scanner-a3-k3-pipe-u8`).
2. **Ship `#1117` (`scanner-a2-fast-build`) SECOND (already open on `src/block.rs` + `src/huff.rs`):**
   - Cuts `wikidata` `S` by **`-4.4%` to `-4.6%`** (`text` `-1.3%` to `-3.3%`) with zero guard regressions (`records` `+0.29%` JCC / `+1.17%` base, `random` `-0.90%` JCC), zero `unsafe`, and cleanly establishes `read_selectors` direct runs (`&[(u8, u16)]`) and `DELTA_LUT` (`read_lengths`) on `master`.
   - Its `A2b + A2c` changes (`src/block.rs`) remain worth **`-3.6%` to `-4.2%`** even after `d8d8ac9e` lands on top of it (`S3 / S13`).
3. **Rebase and ship `scanner-a3-k3-pipe-u8` (`d8d8ac9e`) THIRD on top of `#1117` — with TWO targeted pre-ship refinements identified by this audit:**
   - Delivers an additional **`-5.5%` to `-6.1%`** on `wikidata` `S` and **`-9.5%` to `-10.2%`** on `text` `S` on top of `#1117` (bringing total composed `S + in_maint` to **`-14.6%` to `-14.8%`** on `wikidata-x4` and `k*` from `8.85 → 9.78` on JCC / `9.53 → 10.65` on base).
   - **Refinement 1 (recover `~0.9%` `A2d` headroom):** Port `#1117`'s single-write `fill_lut` + tail-only `SubLut` (eliminating `d_arr[0..1024]` and redundant `K = 1` overwrites) into `d8d8ac9e`'s `K = 3` `build_into` rather than overwriting `src/huff.rs` with the older multi-pass builder.
   - **Refinement 2 (keep `records` `S` `<= +1.5%`, below the `2.0%` guard):** Keep `decode_runs_bmi2`'s inner 4-step loop at `1` `(%rsp)` load (eliminating `S3`'s `mov 0x1f8(%rsp),%rax` reload caused by outer `&[(u8, u16)]` register pressure — e.g. by outlining or isolating the inner single-run `decode_run` helper with `#[inline(never)]` / explicit register arguments) before queueing `scanner-a3-k3-pipe-u8`'s i9 kill test.
