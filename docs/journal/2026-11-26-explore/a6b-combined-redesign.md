# Whole-A6b combined redesign: fused setup, safe 64K u16-link inline15 SoA steps, and generation-tagged dictionary

Claim [`a6b-combined-redesign`](../../../frontier/claims/a6b-combined-redesign.toml) — smooth-zone characterisation targeting the unheld `C-A6b` phase (`216.0 ms` at 1W = `16.54%` of the `1,306 ms` wall; `30.4 ms/w` at 8W; `src/open/endpoint.rs`), combining all three `A6b` sub-phases (`setup`, `steps`, `dict`) and pricing them end-to-end across full `C + E` (`A6a + A6b + A6c + A7`) under both shipped bidirectional emission and matched forward emission. Zero edits to `src/`, zero `unsafe`.

**Verdict: PASS.** The winning design (`comb_nobc_soa_gen`: fused single-pass setup with packed `u64` image order + safe zero-bounds-check `Box<[T; 65536]>` `u16`-link `inline15` SoA induction with direct `aw == bw` in-place replace + generation-tagged L1d-hot dictionary caches with `contents` dedup and dead `stats` gated) clears every criterion of the frozen `stop_test`:
- **100% byte-exact on all 431/431 blocks** of `corpora/wikidata.bz2` against `bwt::walk` across all 9 arms (both shipped bidi and matched forward emit; zero `wideA`, zero `wideE`).
- **Cuts `A6b` by −27.4% to −28.8%** (`0.7119×–0.7263×`, completely disjoint `[min..max]` spreads on all 4 build × emit cells).
- **Cuts full `C + E` by −5.4% to −7.6%** (`0.9245×–0.9457×`, completely disjoint `[min..max]` spreads on both wall-clock and on-CPU thread clock across all 4 build × emit cells).
- **Projected i9-10900K saving:** **`+48.7 to +67.7 ms`** i9-anchored (**`+3.73% to +5.18%`** of the `1,306 ms` 1W wall; sub-phase anchored `+59.2 to +64.3 ms`; proportional `+58.0 to +80.6 ms`; modeled 8W worker CPU **`+8.2 to +9.1 ms/worker`**) — **1.24× to 1.73× the frozen 3.0% (`39.18 ms`) gate**.
- **Zero `A7` regression:** `A7_Δ = −0.44% to +0.32%` across all 4 cells (`<= 1.0%` guard), with practically identical dictionary compactness (`246,683` vs `246,680` entries, `64,314` long entries, `9,888.3 KiB` spelling).
- **Follow-on admitted at `status = "idea"`:** striated build claim [`a6b-endpoint-build`](../../../frontier/claims/a6b-endpoint-build.toml).

---

## 1. Harness, hardware, method & frozen stop test

- **Harness:** [`examples/a6b_combined_probe.rs`](../../../examples/a6b_combined_probe.rs), faithfully mirroring shipped `src/open.rs`, `src/open/endpoint.rs`, and `src/open/outq.rs` (`LIMIT = 16`, `INLINE = 16`, `CERTIFIED_WIDTH = 32_768`, `OutQ16`, `emit_bidirectional`, and matched forward `emit`), including shipped `endpoint.rs:811-824`'s live `stats` updates in `shipped_baseline`.
- **Corpus:** `corpora/wikidata.bz2` (SHA-256 `82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915`; `431` total blocks / `209,706,557` rows; `233` production-admitted Open blocks / `209,697,662` rows matching `results-2/README.md`).
- **Hardware & builds:** 2-vCPU Intel Xeon @ 2.30 GHz KVM VM (Haswell/Broadwell, AVX2, BMI2, 32 KiB L1d, 256 KiB L2, 45 MiB L3), pinned to CPU 0 (`taskset -c 0`), single-threaded, warm reusable `ProbeKernel` and `out` buffer across blocks.
  - **Baseline x86-64:** `cargo build --release --example a6b_combined_probe`
  - **Native:** `RUSTFLAGS="-C target-cpu=native" cargo build --release --example a6b_combined_probe`
- **Method:** Full warm-up + exactness check over all `431/431` blocks (forced Open on every block vs `bwt::walk`), followed by `7` interleaved repeats over the `233` production-admitted Open blocks. Sub-phases (`A6a`, `A6b-setup`, `A6b-steps`, `A6b-dict`, `A6c`, `A7`) are timed with nanosecond `Instant::now()`; each repeat's full `C + E` is additionally cross-checked with Linux `/proc/thread-self/schedstat` (`se.sum_exec_runtime`, 100% safe `read_at(0)`, zero `unsafe`) to verify absence of scheduler preemption bias.
- **i9-10900K calibration & frozen gate (unchanged from claim):** `A6a = 200.3 ms`, `A6b = 216.0 ms` (`setup 41.7`, `steps 110.4`, `dict 63.9`), `A6c = 161.4 ms`, `A7 = 490.3 ms` (`C + E = 1,068.0 ms`), `1W wall = 1,306 ms`, `8W A6b = 30.4 ms/w`, `8W A7 = 69.5 ms/w`. PASS iff combined redesign cuts full `C + E` by `>= 3.0%` of the 1W wall (`>= 39.18 ms` i9-anchored), with non-overlapping `[min..max]` spreads vs baseline on BOTH baseline x86-64 and `-C target-cpu=native`, and `<= 1.0%` `A7` regression.

---

## 2. Mechanisms tested (incremental ablation ladder)

All redesigned arms preserve `Wide` (`most > 65_536`) and `SPIRALS = true` (`widest > CERTIFIED_WIDTH`) paths and use zero `unsafe`.

1. **`shipped_baseline`:** Exact mirror of `src/open.rs:424-450` + `src/open/endpoint.rs`, including the 6 live `stats` updates inside the band loop (`endpoint.rs:811-824`).
2. **`baseline_no_stats`:** Same as `shipped_baseline` with dead `stats` updates gated behind `census`.
3. **`setup_u16_fused`:** Fuses `src/open.rs:424-450` (`to += first[a]` + `image_order` scatter) with `endpoint.rs:675-700` (strip node creation + domain gaps) into **one single forward pass** over `strips_raw`, scattering a packed `u64` `(to: 20b | width: 20b | sid: 16b)` into `image_order64` so the image-gap pass reads sequentially with **zero random reads of `strips_raw`**. Replaces bit-packed `u32` RMW links with direct `u16` fields in SoA (`links: Vec<NodeLinks>`, `widths: Vec<u32>`, `words32: Vec<u32>`) and uses branchless sentinel-0 list appends for `order[0]`/`order[1]`.
4. **`inline15_soa_zcache`:** Replaces `u32` word handles with `words128: Vec<u128>` (`tag = byte 15`: `0` empty, `1..=15` inline bytes in `0..tag` with zero tail, `0xFF` long rule handle carrying full 32-bit `rule_id` and 32-bit exact `len` — eliminating `SATURATED`/`ID_MASK`/`len()` checks). Because `left != 0` implies `8 * ll >= 8`, inline concatenation (`ll + rl < 16`, `76.1%` of non-zero joins) compiles to `(left + (u128::from(rl) << 120)) | (right << (8 * ll))` in registers, cutting rule allocations **−54.6%** (`1,647,746 → 747,845`). Adds direct in-place replace on `aw == bw` (`36.98%` of steps: `pop_tail(x, 0) + replace(y, x, 0) + pop_tail(y, 1)` instead of `insert(y, x, 0); cut(y, 0)`). Keeps per-block zero-filled dict cache to isolate step gains.
5. **`combined_soa_gen`:** Replaces the per-block `24 KiB` `cache[..6144].fill(0)` with persistent generation-tagged L1d-hot caches (`contents[4096]` = 16 KiB `gen16|code16`, `pairs[1024]` = 8 KiB `gen18|code16|key30` for long words only), skips `pairs` and `short_payload` for `len <= 15` words (constructing `entry_u128` directly in registers), caches node metadata (`width`, `word`, `bottom`, `next0`) in the outer band loop, and reuses dead `(prev1, next1)` `u16` fields for the 32-bit `bottom` coordinate.
6. **`comb_nobc_soa_gen` (winning design):** Identical algorithm to `combined_soa_gen`, storing SoA arrays in reusable `Box<[T; 65536]>` indexed by `usize::from(u16)`. LLVM proves every index `< 65536` statically, eliminating **100% of bounds checks** in `setup`, `eliminate`, `assign_bottom`, and `dict` with **zero `unsafe`**. Only indices `0..nodes` (`avg ~9,720` nodes ≈ `273 KiB`) are ever touched.
7. **`comb_nobc_soa_nod`:** Same as `comb_nobc_soa_gen`, dropping `contents` dedup (keeping generation-tagged `pairs` for long words only) to price the `A6b-dict` vs `A7` trade-off directly.
8. **`comb_nobc_aos32_gen`:** Safe 64K AoS32 comparison (`Node32` = 32B aligned, `links + width + word: u128` in one cache line).
9. **`comb_nobc_i7_gen`:** Safe 64K AoS24 `inline7` comparison (`Node24` = 24B, `word: u64` inline `len <= 7`, `786,142` rules, `~233 KiB` live working set).

---

## 3. Baseline x86-64 results (7 interleaved repeats, 233 admitted blocks)

### 3.1 Shipped bidirectional emission (`C + E`)

| arm | `A6a` | `setup` | `steps` | `dict` | `A6b` med `[min..max]` | `A6b` rat | `A6c` | `A7` | `A7_Δ%` | `C+E` wall `[min..max]` | `C+E` CPU `[min..max]` | `i9_anc_Δ` (`%1w`) | `8w_wkr_Δ` | `A6b`/`CE` |
|---|---:|---:|---:|---:|---|---:|---:|---:|---:|---|---|---:|---:|---|
| `shipped_baseline` | 119.20 | 26.60 | 64.52 | 40.52 | 131.66 `[130.08..135.22]` | 1.0000× | 78.57 | 236.10 | +0.00% | 566.31 `[552.60..569.84]` | 563.44 `[551.32..570.07]` | +0.00 ms (+0.00%) | +0.00 ms | — |
| `baseline_no_stats` | 119.72 | 26.62 | 64.75 | 39.55 | 131.34 `[129.94..132.11]` | 0.9976× | 79.66 | 236.72 | +0.26% | 567.55 `[552.14..573.77]` | 562.10 `[551.44..565.64]` | −3.89 ms (−0.30%) | −0.11 ms | ovlp/ovlp |
| `setup_u16_fused` | 119.09 | 15.30 | 58.35 | 35.73 | 109.55 `[108.65..111.26]` | 0.8321× | 80.76 | 233.63 | −1.05% | 542.55 `[534.82..548.71]` | 542.75 `[535.08..545.83]` | +37.10 ms (+2.84%) | +5.83 ms | DISJ/DISJ |
| `inline15_soa_zcache` | 120.08 | 18.55 | 52.27 | 35.56 | 106.41 `[105.77..107.73]` | 0.8082× | 81.50 | 234.88 | −0.52% | 543.24 `[531.00..546.04]` | 535.09 `[531.35..543.30]` | +36.47 ms (+2.79%) | +6.19 ms | DISJ/DISJ |
| `combined_soa_gen` | 119.81 | 18.92 | 52.42 | 35.57 | 107.76 `[105.40..111.60]` | 0.8185× | 81.78 | 235.76 | −0.14% | 545.51 `[532.12..568.32]` | 539.13 `[531.02..555.33]` | +32.31 ms (+2.47%) | +5.62 ms | DISJ/ovlp |
| **`comb_nobc_soa_gen`** | **120.54** | **15.50** | **44.47** | **34.80** | **95.08 `[93.10..99.32]`** | **0.7222×** | **82.23** | **236.85** | **+0.32%** | **533.05 `[518.57..549.50]`** | **528.94 `[515.56..542.59]`** | **+48.67 ms (+3.73%)** | **+8.22 ms** | **DISJ/DISJ** |
| `comb_nobc_soa_nod` | 119.32 | 15.53 | 44.69 | 29.26 | 89.56 `[87.94..91.96]` | 0.6802× | 81.54 | 241.07 | +2.10% | 533.11 `[519.99..540.37]` | 526.97 `[516.08..535.52]` | +52.45 ms (+4.02%) | +8.26 ms | DISJ/DISJ |
| `comb_nobc_aos32_gen` | 120.40 | 16.00 | 47.71 | 37.26 | 101.04 `[99.50..102.68]` | 0.7674× | 82.31 | 234.32 | −0.75% | 539.25 `[527.29..555.71]` | 538.25 `[524.78..553.44]` | +44.23 ms (+3.39%) | +7.59 ms | DISJ/ovlp |
| `comb_nobc_i7_gen` | 119.57 | 16.27 | 49.57 | 36.37 | 102.01 `[100.95..103.99]` | 0.7748× | 80.44 | 235.02 | −0.46% | 538.89 `[527.98..550.18]` | 535.38 `[526.30..550.04]` | +46.41 ms (+3.55%) | +7.16 ms | DISJ/DISJ |

*(Independent earlier 7-repeat baseline bidi run corroborated closely: `comb_nobc_soa_gen` `A6b = 94.43 [93.26..98.40]` vs `132.15 [131.35..138.55]` (`0.7146×`), `C+E = 531.01 [521.39..553.62]` vs `568.37 [562.70..595.63]` (`0.9343×`, DISJ), `A7_Δ = −0.39%`, `i9_anc_Δ = +61.39 ms` (`+4.70%` 1W wall).)*

### 3.2 Matched forward emission (`C + E`, `force_forward = true`)

| arm | `A6a` | `setup` | `steps` | `dict` | `A6b` med `[min..max]` | `A6b` rat | `A6c` | `A7` | `A7_Δ%` | `C+E` wall `[min..max]` | `C+E` CPU `[min..max]` | `i9_anc_Δ` (`%1w`) | `8w_wkr_Δ` | `A6b`/`CE` |
|---|---:|---:|---:|---:|---|---:|---:|---:|---:|---|---|---:|---:|---|
| `shipped_baseline` | 116.71 | 26.26 | 63.61 | 40.02 | 130.41 `[128.77..133.88]` | 1.0000× | 32.26 | 228.73 | +0.00% | 508.05 `[500.96..531.38]` | 507.25 `[503.53..520.75]` | +0.00 ms (+0.00%) | +0.00 ms | — |
| `combined_soa_gen` | 117.27 | 18.55 | 52.17 | 35.49 | 106.32 `[105.31..110.15]` | 0.8153× | 32.51 | 229.33 | +0.26% | 485.63 `[482.53..517.08]` | 486.89 `[482.91..504.31]` | +36.43 ms (+2.79%) | +5.43 ms | DISJ/ovlp |
| **`comb_nobc_soa_gen`** | **117.42** | **15.33** | **44.33** | **34.62** | **94.71 `[93.04..98.47]`** | **0.7263×** | **32.74** | **228.76** | **+0.01%** | **473.78 `[468.83..495.51]`** | **469.91 `[467.64..492.04]`** | **+55.45 ms (+4.25%)** | **+8.31 ms** | **DISJ/DISJ** |
| `comb_nobc_soa_nod` | 116.79 | 15.09 | 44.26 | 29.07 | 88.38 `[87.64..100.21]` | 0.6778× | 32.55 | 232.11 | +1.47% | 469.09 `[467.35..504.25]` | 471.04 `[467.15..479.40]` | +60.80 ms (+4.66%) | +8.77 ms | DISJ/ovlp |
| `comb_nobc_aos32_gen` | 118.83 | 15.72 | 47.18 | 37.25 | 99.93 `[98.68..103.33]` | 0.7663× | 33.08 | 235.32 | +2.88% | 488.39 `[474.58..515.00]` | 481.32 `[475.39..492.67]` | +28.65 ms (+2.19%) | +5.10 ms | DISJ/ovlp |
| `comb_nobc_i7_gen` | 117.47 | 16.34 | 49.20 | 35.81 | 101.20 `[100.24..108.43]` | 0.7761× | 32.70 | 228.58 | −0.07% | 480.52 `[476.16..561.46]` | 479.26 `[474.02..490.06]` | +45.22 ms (+3.46%) | +6.85 ms | DISJ/ovlp |

---

## 4. Native (`-C target-cpu=native`) results (7 interleaved repeats, 233 admitted blocks)

### 4.1 Shipped bidirectional emission (`C + E`)

| arm | `A6a` | `setup` | `steps` | `dict` | `A6b` med `[min..max]` | `A6b` rat | `A6c` | `A7` | `A7_Δ%` | `C+E` wall `[min..max]` | `C+E` CPU `[min..max]` | `i9_anc_Δ` (`%1w`) | `8w_wkr_Δ` | `A6b`/`CE` |
|---|---:|---:|---:|---:|---|---:|---:|---:|---:|---|---|---:|---:|---|
| `shipped_baseline` | 133.81 | 26.43 | 63.91 | 41.24 | 131.58 `[128.95..137.32]` | 1.0000× | 86.40 | 241.93 | +0.00% | 593.72 `[582.10..617.32]` | 594.89 `[583.45..612.43]` | +0.00 ms (+0.00%) | +0.00 ms | — |
| `baseline_no_stats` | 133.76 | 26.34 | 64.11 | 40.27 | 130.48 `[128.02..132.35]` | 0.9917× | 86.14 | 239.15 | −1.15% | 588.87 `[577.91..609.55]` | 587.24 `[575.51..598.48]` | +7.99 ms (+0.61%) | +1.05 ms | ovlp/ovlp |
| `setup_u16_fused` | 132.04 | 16.57 | 60.54 | 36.48 | 113.57 `[112.91..114.85]` | 0.8631× | 88.65 | 237.75 | −1.73% | 573.24 `[567.77..581.71]` | 569.13 `[563.41..575.41]` | +36.51 ms (+2.80%) | +5.36 ms | DISJ/DISJ |
| `inline15_soa_zcache` | 133.72 | 18.35 | 51.63 | 35.13 | 105.39 `[104.32..107.44]` | 0.8010× | 87.99 | 239.60 | −0.97% | 567.48 `[559.80..573.10]` | 567.29 `[557.26..571.40]` | +44.87 ms (+3.44%) | +6.72 ms | DISJ/DISJ |
| `combined_soa_gen` | 133.82 | 18.82 | 52.29 | 36.06 | 107.08 `[103.68..109.12]` | 0.8138× | 89.33 | 240.81 | −0.46% | 572.52 `[552.78..582.67]` | 569.34 `[555.49..575.10]` | +36.98 ms (+2.83%) | +5.98 ms | DISJ/ovlp |
| **`comb_nobc_soa_gen`** | **134.91** | **15.83** | **43.72** | **35.48** | **94.89 `[93.95..96.46]`** | **0.7212×** | **89.70** | **242.29** | **+0.15%** | **561.47 `[556.40..571.57]`** | **557.32 `[555.35..566.41]`** | **+51.68 ms (+3.96%)** | **+8.37 ms** | **DISJ/DISJ** |
| `comb_nobc_soa_nod` | 134.34 | 15.98 | 44.19 | 29.36 | 90.29 `[86.40..91.43]` | 0.6862× | 90.07 | 243.84 | +0.79% | 557.31 `[541.17..587.55]` | 556.14 `[540.87..573.22]` | +56.26 ms (+4.31%) | +8.99 ms | DISJ/ovlp |
| `comb_nobc_aos32_gen` | 134.94 | 16.50 | 47.65 | 35.73 | 99.61 `[97.19..101.21]` | 0.7570× | 89.06 | 243.02 | +0.45% | 566.72 `[549.24..577.25]` | 567.20 `[547.62..570.20]` | +43.62 ms (+3.34%) | +7.07 ms | DISJ/DISJ |
| `comb_nobc_i7_gen` | 133.99 | 15.79 | 47.16 | 35.92 | 98.26 `[97.93..105.25]` | 0.7468× | 89.75 | 241.68 | −0.10% | 563.23 `[550.55..593.17]` | 563.48 `[551.28..582.82]` | +48.69 ms (+3.73%) | +7.77 ms | DISJ/ovlp |

### 4.2 Matched forward emission (`C + E`, `force_forward = true`)

| arm | `A6a` | `setup` | `steps` | `dict` | `A6b` med `[min..max]` | `A6b` rat | `A6c` | `A7` | `A7_Δ%` | `C+E` wall `[min..max]` | `C+E` CPU `[min..max]` | `i9_anc_Δ` (`%1w`) | `8w_wkr_Δ` | `A6b`/`CE` |
|---|---:|---:|---:|---:|---|---:|---:|---:|---:|---|---|---:|---:|---|
| `shipped_baseline` | 132.80 | 26.52 | 63.56 | 40.70 | 131.00 `[129.18..132.34]` | 1.0000× | 33.26 | 225.62 | +0.00% | 522.68 `[516.21..532.09]` | 519.19 `[515.06..525.22]` | +0.00 ms (+0.00%) | +0.00 ms | — |
| `combined_soa_gen` | 132.26 | 18.36 | 51.28 | 35.40 | 104.75 `[103.90..108.07]` | 0.7996× | 33.44 | 224.68 | −0.42% | 494.19 `[493.50..520.22]` | 494.20 `[490.82..504.84]` | +45.24 ms (+3.46%) | +6.38 ms | DISJ/ovlp |
| **`comb_nobc_soa_gen`** | **131.09** | **15.30** | **43.23** | **34.81** | **93.26 `[92.63..98.29]`** | **0.7119×** | **33.10** | **224.63** | **−0.44%** | **483.24 `[479.99..499.89]`** | **482.74 `[478.73..489.92]`** | **+67.70 ms (+5.18%)** | **+9.06 ms** | **DISJ/DISJ** |
| `comb_nobc_soa_nod` | 131.62 | 15.33 | 43.00 | 28.96 | 87.71 `[86.02..88.30]` | 0.6695× | 33.46 | 229.96 | +1.92% | 482.36 `[478.58..498.13]` | 483.25 `[477.80..490.43]` | +62.73 ms (+4.80%) | +8.71 ms | DISJ/DISJ |
| `comb_nobc_aos32_gen` | 132.24 | 16.27 | 46.84 | 35.49 | 98.46 `[97.09..100.10]` | 0.7516× | 33.67 | 226.74 | +0.50% | 489.98 `[486.56..502.02]` | 490.49 `[486.48..502.49]` | +50.05 ms (+3.83%) | +7.21 ms | DISJ/DISJ |
| `comb_nobc_i7_gen` | 131.95 | 15.33 | 46.65 | 35.59 | 97.67 `[96.47..99.54]` | 0.7456× | 33.66 | 224.39 | −0.55% | 488.03 `[485.67..494.84]` | 487.55 `[483.32..490.44]` | +56.96 ms (+4.36%) | +8.12 ms | DISJ/DISJ |

---

## 5. Structural metrics & design findings

| arm | steps | rules | bands | `dict_ent` | `long_ent` | `spelling` KiB | `wideE` | `wideA` |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| `shipped_baseline` / `setup_u16_fused` | 2,668,970 | 1,647,746 | 827,858 | 246,680 | 64,314 | 9,888.3 | 0 | 0 |
| `comb_nobc_soa_gen` / `comb_nobc_aos32_gen` | 2,668,970 | **747,845 (−54.6%)** | 827,858 | **246,683 (+0.001%)** | **64,314** | **9,888.3** | 0 | 0 |
| `comb_nobc_i7_gen` | 2,668,970 | 786,142 (−52.3%) | 827,858 | 246,683 | 64,314 | 9,888.3 | 0 | 0 |
| `comb_nobc_soa_nod` | 2,668,970 | 747,845 | 827,858 | **427,381 (+73.3%)** | 105,732 | 9,969.9 | 0 | 0 |

**Key mechanisms explaining the PASS:**
1. **Why combining all 3 sub-phases clears the gate with headroom (~1.3–1.7×):**
   - `A6b-setup` (`26.5 → 15.4 ms`, **−41.7%**, `+17.4 ms` on i9): single-pass strip/gap construction + packed `u64` `image_order64` eliminating random gathers from `strips_raw` + branchless sentinel-0 `order` appends + direct `u16` links.
   - `A6b-steps` (`64.2 → 43.8 ms`, **−31.8%**, `+35.1 ms` on i9; `24.1 → 16.4 ns/step`): `inline15` `u128` SoA cutting rules by −54.6%, direct `aw == bw` in-place replace on `36.98%` of steps, and safe zero-bounds-check `Box<[T; 65536]>` indexed by `usize::from(u16)` (which alone removes ~8 ms vs dynamic-`Vec` `combined_soa_gen`!).
   - `A6b-dict` (`40.7 → 34.8 ms`, **−14.5%**, `+9.3 ms` on i9): direct `u128` short-word extraction (no `pairs` lookup or `short_payload` load for `len <= 15`), generation-tagged L1d-hot caches (no per-block 24 KiB zero-fill), node-cached outer band loop, and dead `stats` gated behind `census`.
2. **Why `SoA` (`comb_nobc_soa_gen`) beats `AoS32` and `AoS24-inline7`:**
   - SoA keeps `links` (`8 B/node`, `~78 KiB` live, 8 nodes per 64B cache line) + `widths` (`4 B/node`, `~39 KiB` live) compact inside L2 so neighbor link updates (`xp`, `yp`, `yn`, `xn`) never pull 24–32 byte nodes. `comb_nobc_soa_gen` beats `comb_nobc_aos32_gen` by `~5–6 ms` and `comb_nobc_i7_gen` by `~4–7 ms` on `A6b` across all 4 cells.
3. **Why `GenDedup` (`comb_nobc_soa_gen`) beats `NoContentsDedup` (`comb_nobc_soa_nod`) end-to-end:**
   - Dropping `contents` dedup shaves another `~5.5 ms` in `A6b-dict` (`34.8 → 29.3 ms`), but inflates `dict_entries` by `+73.3%` (`246,680 → 427,381`, growing `inline16` from `16.9 → 29.4 KiB/blk`), which regresses `A7` by **`+1.47%..+2.10%`** (breaching the `<= 1.0%` `A7` guard and giving back `~3–5 ms` in `A7`). Keeping `GenDedup` achieves virtually identical net `C + E` with **zero `A7` regression** (`−0.44%..+0.32%`).
4. **Memory footprint caveat for `a6b-endpoint-build` (k=8–10 L3 check):**
   - `Box<[NodeLinks; 65536]>` (512 KiB) + `Box<[u32; 65536]>` (256 KiB) + `Box<[u128; 65536]>` (1 MiB) reserves `1.75 MiB` of virtual memory per worker, though only `0..nodes` (`avg ~273 KiB`, `max ~420 KiB` on wikidata) is ever faulted or touched in cache, and it is dead before `A6c`/`A7`. Notice a slight `+1–3 ms` rise in shipped `A6c` (`wire_xor`) on some runs (0 ms on forward `A6c`), likely from `self.edges` no longer being warmed during `A6b`. `a6b-endpoint-build`'s i9 workstation registration must verify RSS and k=8–10 worker CPU + L3 impact directly on the i9-10900K.
5. **Reproducibility:**
   ```bash
   cargo build --release --example a6b_combined_probe
   taskset -c 0 ./target/release/examples/a6b_combined_probe corpora/wikidata.bz2 --repeats 7
   RUSTFLAGS="-C target-cpu=native" cargo build --release --example a6b_combined_probe
   taskset -c 0 ./target/release/examples/a6b_combined_probe corpora/wikidata.bz2 --repeats 7
   ```
