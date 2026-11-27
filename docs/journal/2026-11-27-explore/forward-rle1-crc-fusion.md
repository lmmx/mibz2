# Upstream RLE1 Expansion & Tiled CRC32 Fusion into Forward BWT Emission (`forward-rle1-crc-fusion`)

**Claim:** `forward-rle1-crc-fusion` (smooth-zone probe, zero `src/` edits, `#![forbid(unsafe_code)]`)  
**Prototype:** `examples/forward_rle1_crc_fusion.rs`  
**Verdict:** **PASS** for **4-lane vectored zero-scratch in-place tile fusion** (`fused_4l_tile4096_vec`, admitted at `idea` as `forward-rle1-crc-vectored-build`, queued behind `forward-emission`); **scoped STOP** for **1-lane fused emit**, **inline per-step RLE1 FSM + pre-flagged `WordMeta`**, and **4-lane contiguous stitched fusion** (recorded in `frontier/dead-ends.tsv`).

---

## 1. Environment & Methodology

- **Host:** 2-vCPU Haswell Xeon @ 2.30 GHz (`32 KiB` L1d / `256 KiB` L2 / `45 MiB` L3, AVX2 + PCLMULQDQ, no vPMU).
- **Corpora:**
  - `corpora/wikidata.bz2`: 431 blocks (209,700,196 bytes, SHA-256 `82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915`), split identically to `ideal_permutation_structures.rs`: 233 Open-admitted blocks (`117` train, `116` held-out = `104,398,845` pre-RLE1 bytes = `99.56 MiB`) and 198 non-admitted blocks.
  - All six `ab gen` corpora at scale 4 (`text`, `random`, `onebyte`, `pattern`, `runs`, `records`).
  - 56 synthetic exactness stress blocks including `miri_edge_runs`, `RunUnfinished` at EOF, periodic short cycles, and 4-byte runs placed at every byte offset `-4..=+4` across all three 4-lane boundaries (`256`, `512`, `768`).
- **Builds & Commands:**
  - Baseline x86-64: `cargo build --release --example forward_rle1_crc_fusion`
  - Native (matched AVX2): `RUSTFLAGS="-C target-cpu=native" cargo build --release --example forward_rle1_crc_fusion`
  - Run: `forward_rle1_crc_fusion corpora/wikidata.bz2 --ab-dir <ab-dir> --repeats 7`
- **Timing discipline:** Because other sessions share the 2-vCPU VM, timings use Linux thread-CPU nanoseconds (`/proc/thread-self/schedstat`, 100% `#![forbid(unsafe_code)]`) over single full-pass spans (eliminating wall-clock preemption noise while avoiding per-block tick quantization):
  - **Warm:** single-span differential `(T_{1+k} - T_1) / k` (`k = 3` for 116-block wikidata, up to `k = 40` for 1-block corpora), measuring in-situ L2/L3-warm execution per block.
  - **Cold:** 64 MiB cache flush before each full 116-block pass, timed across the entire pass.
  - 7 interleaved repeats with alternating forward/reverse arm ordering on held-out wikidata and `F1b_repair_unb` (5 interleaved repeats on guard corpora), reporting `median [min..max]` and strict non-overlap against the faster unfused forward control (`ctrl_4lane_unfused`).
- **i9-10900K 1W Projection Anchor:** Consistent with `ideal-permutation-structures` and `rle1-crc-census`, `ctrl_bidi_unfused` (`E + A9`) is anchored to `560.8 ms` (`490.3 ms` bidi `E` + `70.5 ms` midpoint `A9`) out of the `1,306 ms` 1W reference wall (`3.0%` gate = `39.18 ms`).

---

## 2. Exactness Report

Zero byte, length, CRC32, or `ErrorKind::RunUnfinished` mismatches across **13,360 checks per build** (verified identically on both baseline x86-64 and `-C target-cpu=native`):

| Suite | Blocks | Arms | Checks | Result |
|---|---:|---:|---:|---|
| `synthetic+straddles` (incl. `-4..=+4` lane straddles & `RunUnfinished`) | 56 | 22 | 1,232 | **OK** |
| `wikidata_admitted` | 233 | 22 | 5,126 | **OK** |
| `wikidata_non_admitted` | 198 | 22 | 4,356 | **OK** |
| `ab gen` (`text`, `random`, `onebyte`, `pattern`, `runs`, `records`) | 57 | 22 | 1,254 | **OK** |
| `held_out_F1b_repair_unb` (`tau=2` contracted return maps) | 116 | 12 | 1,392 | **OK** |
| **Total** | **660** | — | **13,360** | **100% EXACT** |

---

## 3. Structural Census & Logical Write-Traffic Accounting

### 3.1 Dictionary Pre-flagging (`WordMeta`) & 4-Lane Boundary Census

- **Uncontracted Open (`LIMIT=16`, 116 held-out blocks):**
  - `98,321` multi-byte dictionary words; only **`239` (`0.24%`)** contain an internal 4-byte run.
  - `WordMeta` build cost: `5.25 [4.99..5.25] ms` native / `5.25 [5.22..5.46] ms` baseline.
- **Contracted `F1b_repair_unb` (`tau=2`, 116 held-out blocks):**
  - `546,507` multi-byte dictionary words; only **`1,463` (`0.27%`)** contain an internal 4-byte run.
  - `WordMeta` build cost: `37.71 [37.47..37.98] ms` native / `37.73 [37.15..38.16] ms` baseline.
- **4-lane boundary census (held-out 116 blocks & all 431 wikidata blocks):**
  - Boundary straddles: **`0`** across all 431 wikidata blocks (`0` overflow fallbacks, `0` no-lanes fallbacks on held-out 116).
  - Lanes with `grown == 0`: only **`63 / 464` (`13.6%`)**; **`86.4%`** of lanes change length after RLE1 expansion (mostly shrinking because `89.4%` of wikidata runs have `extra == 0`).
  - Contiguous stitching cost: moves **`74.53 / 99.56 MiB` (`74.9%` of output)** on held-out blocks (`598.9 MiB` on `wikidata x4`).

### 3.2 Full `wikidata x4` Logical Write Traffic (`E + A9`, 1,724 blocks)

| Design | Logical Writes (`E + A9`) | Delta vs Unfused | Meets `>= 50%` Write Cut? |
|---|---:|---:|---|
| Unfused forward `E` + shipped `unrle_in_place` (`A9`) | `800.0 + 918.2 = 1,718.2 MiB` | `0.0%` | baseline |
| Fused 1-lane (in-place single output buffer) | `799.9 MiB` | **`-53.4%`** | YES |
| Fused 4-lane **stitched** (contiguous output `Vec<u8>`) | `1,398.8 MiB` | **`-18.6%`** | **NO** (`74.9%` stitch `memmove`) |
| Fused 4-lane **vectored** (4 lane slices + GF(2) `crc32_combine`) | `799.9 MiB` | **`-53.4%`** | **YES** |

---

## 4. Held-Out Admitted Wikidata (`116` blocks, Uncontracted Open `LIMIT=16`): Full `E + A9`

### 4.1 Native (`-C target-cpu=native`, matched AVX2 run scanner on both control and fused arms)

| Arm | Warm `med [min..max]` ms | `vs4l` | Spread vs 4l | Warm i9 (`d_vs_4l`, `%wall`) | Cold `med [min..max]` ms | `vs4l` | Spread vs 4l | Cold i9 (`d_vs_4l`, `%wall`) |
|---|---:|---:|---|---:|---:|---:|---|---:|
| `ctrl_bidi_unfused` | `143.99 [137.26..151.12]` | `1.282x` | SLOWER | `560.8m (-123.2m, -9.44%)` | `195.62 [191.70..211.76]` | `1.296x` | SLOWER | `560.8m (-128.0m, -9.80%)` |
| `ctrl_1lane_unfused` | `152.01 [146.60..155.18]` | `1.353x` | SLOWER | `592.0m (-154.5m, -11.83%)` | `211.93 [207.57..239.87]` | `1.404x` | SLOWER | `607.6m (-174.7m, -13.38%)` |
| `ctrl_4lane_unfused` | `112.35 [112.01..114.74]` | `1.000x` | ctrl | `437.6m (+0.0m, +0.00%)` | `150.98 [143.98..163.70]` | `1.000x` | ctrl | `432.8m (+0.0m, +0.00%)` |
| `floor_1lane_plus_crc` | `141.81 [138.57..142.54]` | `1.262x` | SLOWER | `552.3m (-114.8m, -8.79%)` | `210.70 [202.95..239.33]` | `1.396x` | SLOWER | `604.0m (-171.2m, -13.11%)` |
| `floor_4lane_plus_crc` | `101.20 [98.85..104.88]` | `0.901x` | FASTER | `394.1m (+43.4m, +3.33%)` | `139.89 [131.87..151.44]` | `0.927x` | overlap | `401.0m (+31.8m, +2.43%)` |
| `fused_1l_fsm_lit_64k` | `156.70 [151.87..182.55]` | `1.395x` | SLOWER | `610.3m (-172.7m, -13.23%)` | `238.21 [231.79..267.75]` | `1.578x` | SLOWER | `682.9m (-250.1m, -19.15%)` |
| `fused_1l_fsm_uni_32k` | `161.02 [158.79..170.26]` | `1.433x` | SLOWER | `627.1m (-189.6m, -14.52%)` | `239.76 [239.03..258.91]` | `1.588x` | SLOWER | `687.3m (-254.5m, -19.49%)` |
| `fused_1l_tile_16k` | `135.98 [134.25..140.37]` | `1.210x` | SLOWER | `529.6m (-92.1m, -7.05%)` | `215.84 [211.08..230.43]` | `1.430x` | SLOWER | `618.8m (-185.9m, -14.24%)` |
| `fused_1l_tile_32k` | `136.98 [134.53..141.23]` | `1.219x` | SLOWER | `533.5m (-95.9m, -7.35%)` | `215.78 [211.75..232.17]` | `1.429x` | SLOWER | `618.6m (-185.8m, -14.22%)` |
| `fused_4l_tile1024_stitch` | `109.47 [107.57..115.59]` | `0.974x` | overlap | `426.4m (+11.2m, +0.86%)` | `147.97 [143.71..161.49]` | `0.980x` | overlap | `424.2m (+8.6m, +0.66%)` |
| `fused_4l_tile1024_vec` | `102.60 [101.13..106.58]` | `0.913x` | **FASTER** | `399.6m (+37.9m, +2.91%)` | `147.68 [143.83..151.99]` | `0.978x` | overlap | `423.4m (+9.5m, +0.72%)` |
| `fused_4l_tile2048_stitch` | `112.97 [110.53..121.17]` | `1.006x` | overlap | `440.0m (-2.4m, -0.19%)` | `147.87 [143.77..155.63]` | `0.979x` | overlap | `423.9m (+8.9m, +0.68%)` |
| `fused_4l_tile2048_vec` | `103.94 [101.22..147.78]` | `0.925x` | overlap | `404.8m (+32.8m, +2.51%)` | `143.80 [139.70..151.85]` | `0.952x` | overlap | `412.2m (+20.6m, +1.57%)` |
| **`fused_4l_tile4096_vec`** | **`99.54 [94.24..102.91]`** | **`0.886x`** | **FASTER** | **`387.7m (+49.9m, +3.82%)`** | **`139.89 [139.74..143.89]`** | **`0.927x`** | **FASTER** | **`401.0m (+31.8m, +2.43%)`** |
| `fused_4l_t1024_defcrc_vec` | `106.54 [103.79..111.04]` | `0.948x` | **FASTER** | `414.9m (+22.6m, +1.73%)` | `143.81 [139.93..147.93]` | `0.953x` | overlap | `412.3m (+20.5m, +1.57%)` |
| `fused_4l_t4096_defcrc_vec` | `102.69 [99.85..110.52]` | `0.914x` | **FASTER** | `399.9m (+37.6m, +2.88%)` | `139.91 [135.85..147.93]` | `0.927x` | overlap | `401.1m (+31.7m, +2.43%)` |
| `fused_4l_fsm32k_stitch` | `164.88 [159.88..169.76]` | `1.468x` | SLOWER | `642.1m (-204.6m, -15.67%)` | `206.82 [199.78..210.31]` | `1.370x` | SLOWER | `592.9m (-160.1m, -12.26%)` |
| `fused_4l_fsm32k_vec` | `147.91 [144.57..158.30]` | `1.317x` | SLOWER | `576.1m (-138.5m, -10.61%)` | `194.81 [191.38..195.82]` | `1.290x` | SLOWER | `558.5m (-125.7m, -9.62%)` |

### 4.2 Baseline x86-64 (control uses runtime-dispatched AVX2 `src/rle.rs`; `#![forbid(unsafe_code)]` fused scanner falls back to SSE2)

| Arm | Warm `med [min..max]` ms | `vs4l` | Spread vs 4l | Warm i9 (`d_vs_4l`, `%wall`) | Cold `med [min..max]` ms | `vs4l` | Spread vs 4l | Cold i9 (`d_vs_4l`, `%wall`) |
|---|---:|---:|---|---:|---:|---:|---|---:|
| `ctrl_bidi_unfused` | `159.23 [156.74..165.51]` | `1.304x` | SLOWER | `560.8m (-130.8m, -10.01%)` | `210.34 [207.82..214.71]` | `1.348x` | SLOWER | `560.8m (-144.9m, -11.10%)` |
| `ctrl_1lane_unfused` | `154.24 [147.82..155.73]` | `1.263x` | SLOWER | `543.2m (-113.2m, -8.67%)` | `219.37 [207.81..223.66]` | `1.406x` | SLOWER | `584.9m (-169.0m, -12.94%)` |
| `ctrl_4lane_unfused` | `122.10 [117.80..122.66]` | `1.000x` | ctrl | `430.0m (+0.0m, +0.00%)` | `155.98 [155.25..159.65]` | `1.000x` | ctrl | `415.9m (+0.0m, +0.00%)` |
| `floor_1lane_plus_crc` | `132.99 [131.29..136.21]` | `1.089x` | SLOWER | `468.4m (-38.4m, -2.94%)` | `199.88 [195.86..210.20]` | `1.281x` | SLOWER | `532.9m (-117.0m, -8.96%)` |
| `floor_4lane_plus_crc` | `106.73 [105.00..109.16]` | `0.874x` | FASTER | `375.9m (+54.1m, +4.15%)` | `139.98 [139.69..147.05]` | `0.897x` | FASTER | `373.2m (+42.7m, +3.27%)` |
| `fused_1l_fsm_lit_64k` | `151.45 [146.47..156.97]` | `1.240x` | SLOWER | `533.4m (-103.4m, -7.91%)` | `231.84 [227.78..235.79]` | `1.486x` | SLOWER | `618.1m (-202.3m, -15.49%)` |
| `fused_1l_fsm_uni_32k` | `161.98 [153.95..171.24]` | `1.327x` | SLOWER | `570.5m (-140.5m, -10.76%)` | `239.90 [231.75..247.56]` | `1.538x` | SLOWER | `639.6m (-223.7m, -17.13%)` |
| `fused_1l_tile_16k` | `142.90 [136.07..143.64]` | `1.170x` | SLOWER | `503.3m (-73.3m, -5.61%)` | `223.85 [215.88..247.92]` | `1.435x` | SLOWER | `596.8m (-181.0m, -13.86%)` |
| `fused_1l_tile_32k` | `141.11 [135.93..151.25]` | `1.156x` | SLOWER | `497.0m (-67.0m, -5.13%)` | `223.35 [219.78..241.06]` | `1.432x` | SLOWER | `595.5m (-179.6m, -13.75%)` |
| `fused_4l_tile1024_stitch` | `123.93 [121.10..143.49]` | `1.015x` | overlap | `436.5m (-6.4m, -0.49%)` | `163.64 [159.87..166.48]` | `1.049x` | SLOWER | `436.3m (-20.4m, -1.56%)` |
| `fused_4l_tile1024_vec` | `111.63 [106.84..126.77]` | `0.914x` | overlap | `393.1m (+36.9m, +2.82%)` | `157.13 [151.92..161.43]` | `1.007x` | overlap | `418.9m (-3.1m, -0.23%)` |
| `fused_4l_tile2048_stitch` | `128.68 [119.75..128.87]` | `1.054x` | overlap | `453.2m (-23.2m, -1.78%)` | `163.93 [163.25..169.97]` | `1.051x` | SLOWER | `437.1m (-21.2m, -1.62%)` |
| **`fused_4l_tile2048_vec`** | **`108.92 [108.46..114.18]`** | **`0.892x`** | **FASTER** | **`383.6m (+46.4m, +3.55%)`** | `151.89 [147.77..155.81]` | `0.974x` | overlap | `405.0m (+10.9m, +0.84%)` |
| `fused_4l_tile4096_vec` | `113.69 [107.38..115.07]` | `0.931x` | **FASTER** | `400.4m (+29.6m, +2.27%)` | `155.87 [151.99..159.33]` | `0.999x` | overlap | `415.6m (+0.3m, +0.02%)` |
| `fused_4l_t1024_defcrc_vec` | `114.59 [112.09..117.09]` | `0.939x` | **FASTER** | `403.6m (+26.4m, +2.02%)` | `151.94 [147.94..153.93]` | `0.974x` | **FASTER** | `405.1m (+10.8m, +0.83%)` |
| `fused_4l_t4096_defcrc_vec` | `115.87 [115.12..121.00]` | `0.949x` | overlap | `408.1m (+21.9m, +1.68%)` | `155.85 [151.78..165.16]` | `0.999x` | overlap | `415.5m (+0.3m, +0.03%)` |
| `fused_4l_fsm32k_stitch` | `166.25 [158.70..171.06]` | `1.362x` | SLOWER | `585.5m (-155.5m, -11.91%)` | `207.89 [203.90..222.59]` | `1.333x` | SLOWER | `554.3m (-138.4m, -10.60%)` |
| `fused_4l_fsm32k_vec` | `162.36 [154.66..166.67]` | `1.330x` | SLOWER | `571.8m (-141.8m, -10.86%)` | `207.55 [203.70..211.81]` | `1.331x` | SLOWER | `553.4m (-137.5m, -10.53%)` |

### 4.3 Mechanism Dissection (Why Zero-Scratch In-Place + Chunk Size `4096` + Vectored Delivery Wins)

1. **Zero-scratch single-pass in-place tile compaction was decisive.** The initial implementation copied every tile containing a run (~80% of 8–32 KiB tiles on wikidata) to `scratch` and re-scanned it from byte 0. On native cold, that double-copy + re-scan turned `fused_4l_tile4096_vec` into `143.91 ms` vs `ctrl_4lane_unfused` `139.72 ms` (`-12.3 ms` i9 slower). Exploiting that `89.4%` of runs have `extra == 0` (`dst <= src`) to compact directly inside the lane buffer from the first run—spilling only the remaining suffix to `scratch` if net growth ever overtakes `src`—flipped native cold from **`-12.3 ms` slower to `+31.8 ms` non-overlapping faster** (`139.89 vs 150.98 ms`), matching `floor_4lane_plus_crc` (`139.89 ms`) to `0.00 ms`.
2. **Warm beats even `floor_4lane_plus_crc` (`99.54 vs 101.20 ms` native).** Running `crc::update_with` on 64 KiB lane tiles keeps the CRC read inside L2 (`256 KiB`), whereas `floor_4lane_plus_crc` runs `crc::crc32` over the full `900 KiB` block in L3 after emission finishes.
3. **Why baseline cold trails native cold (`+10.9 ms` vs `+31.8 ms`).** Under `#![forbid(unsafe_code)]`, `find_next_run_autovec32` cannot use `#[target_feature(enable = "avx2")]` runtime dispatch, so on a baseline x86-64 binary the fused tile scanner compiles to SSE2 (16 B/step) while `ctrl_4lane_unfused` calls shipped `src/rle.rs` runtime-dispatched AVX2 (32 B/step). Calling `src/rle.rs`'s existing AVX2 scanner inside a striated production build eliminates this ~6–7 ms (`~18–20 ms` i9) handicap so baseline matches native.
4. **Why inline FSM + `WordMeta`, 1-lane fused, and 4-lane stitched all fail:**
   - **1-lane fused** (`135.98 ms` warm / `215.78 ms` cold native) loses `-92..-186 ms` i9 to `ctrl_4lane_unfused` because 4-lane MLP saves `+155..+175 ms` on `E` while all removable `A9` is only `32–43 ms`.
   - **Inline FSM + `WordMeta`** loses even to 1-lane unfused (`156.70–161.02 ms` warm / `238.21–239.76 ms` cold native) and regresses 4-lane by `-126..-205 ms` i9: per-visit FSM state + `WordMeta` checks + 4-lane register spills cost far more than bulk AVX2 tile scanning.
   - **4-lane stitched** pays `74.9%` `memmove` (`598.9 MiB` on `wikidata x4`), slashing the write cut to `-18.6%` (`< 50%` gate) and leaving timing flat/slower (`-2.4..+11.2 ms` warm / `+8.6..+8.9 ms` cold native; `-6.4..-23.2 ms` warm / `-20.4..-21.2 ms` cold baseline).

---

## 5. Interaction with Contracted Return Maps (`F1b_repair_unb` / `F4a`, 116 Held-Out Blocks)

| Arm | Native Warm ms (`d_vs_F4a`) | Native Cold ms (`d_vs_F4a`) | Baseline Warm ms (`d_vs_F4a`) | Baseline Cold ms (`d_vs_F4a`) |
|---|---:|---:|---:|---:|
| `ctrl_1lane_unfused` (`F1b`) | `49.78 (-9.9m)` | `65.47 (-15.7m)` | `52.86 (-1.2m)` | `67.96 (-10.7m)` |
| `ctrl_4lane_unfused` (`F4a`) | `47.23 (+0.0m)` | `59.98 (+0.0m)` | `52.52 (+0.0m)` | `63.94 (+0.0m)` |
| `floor_4lane_plus_crc` | `36.62 (+41.3m, FASTER)` | `47.96 (+34.5m, FASTER)` | `37.80 (+51.9m, FASTER)` | `51.89 (+32.1m, FASTER)` |
| `fused_1l_tile_16k` | `45.95 (+5.0m)` | `63.98 (-11.4m, SLOWER)` | `51.06 (+5.1m)` | `70.98 (-18.8m)` |
| `fused_1l_tile_32k` | `45.91 (+5.1m)` | `67.88 (-22.6m, SLOWER)` | `50.00 (+8.9m, FASTER)` | `71.57 (-20.3m)` |
| `fused_4l_tile512_stitch` | `49.90 (-10.4m)` | `63.95 (-11.4m, SLOWER)` | `55.26 (-9.6m)` | `71.92 (-21.3m)` |
| **`fused_4l_tile512_vec`** | **`42.88 (+16.9m, FASTER)`** | **`56.00 (+11.4m)`** | **`48.59 (+13.8m, FASTER)`** | `63.99 (-0.1m)` |
| `fused_4l_tile1024_vec` | `43.29 (+15.4m, FASTER)` | `59.93 (+0.1m)` | `48.51 (+14.1m, FASTER)` | `63.93 (+0.0m)` |
| `fused_4l_fsm32k_vec` | `59.97 (-49.6m, SLOWER)` | `79.96 (-57.3m, SLOWER)` | `62.61 (-35.5m, SLOWER)` | `83.64 (-52.5m, SLOWER)` |

**Finding:** Even when `F1b` shrinks `edges` to L2-resident `93.6 KiB/blk`, 1-lane fused (`fused_1l_tile_16k/32k`) only edges past unfused `F4a` by `+5.0..+8.9 ms` in warm mode and loses `-11.4..-22.6 ms` in cold mode, while **4-lane vectored tile fusion (`fused_4l_tile512_vec`) beats both 1-lane fused and unfused `F4a`** (`+16.9 ms` warm FASTER / `+11.4 ms` cold native).

---

## 6. Guard Corpora (`ab gen`, 5 Interleaved Repeats)

Comparing `fused_4l_tile2048_vec` (same `fused_4lane_tile` kernel as `tile4096_vec`) against `ctrl_4lane_unfused`:

| Corpus (blocks, fallback stats) | Native Warm (`vs4l`) | Native Cold (`vs4l`) | Baseline Warm (`vs4l`) | Baseline Cold (`vs4l`) | Status |
|---|---:|---:|---:|---:|---|
| **`text.bz2`** (`13` blks, `0` straddles, `0` overflow) | `58.60 vs 61.24` (`0.957x`) | `79.92 vs 83.91` (`0.952x`) | `60.90 vs 62.15` (`0.980x`) | `83.93 vs 82.70` (`1.015x`, ovl) | **PASS** (4–5% faster native, neutral base) |
| **`onebyte.bz2`** (`1` blk, `no_lanes=1`) | `3.69 vs 5.33` (`0.693x`, **FASTER**) | `4.63 vs 5.56` (`0.833x`, **FASTER**) | `3.86 vs 5.43` (`0.711x`, **FASTER**) | `4.72 vs 6.32` (`0.745x`, ovl) | **PASS** (17–31% faster) |
| **`runs.bz2`** (`1` blk, `overflow=1` -> 1-lane tile) | `3.14 vs 2.85` (`1.101x`, ovl) | `3.47 vs 3.39` (`1.024x`, ovl) | `3.22 vs 2.97` (`1.088x`, ovl) | `3.64 vs 3.59` (`1.015x`, ovl) | **PASS** (neutral/overlap; `t1024_defcrc_vec` `1.002x`/`0.995x` cold) |
| `random.bz2` (`9` blks) | `56.81 vs 59.94` (`0.948x`, ovl) | `83.96 vs 79.97` (`1.050x`, ovl) | `59.90 vs 60.21` (`0.995x`, ovl) | `80.69 vs 83.96` (`0.961x`, ovl) | neutral |
| `records.bz2` (`11` blks) | `72.87 vs 69.33` (`1.051x`, ovl) | `95.96 vs 95.95` (`1.000x`, ovl) | `77.03 vs 73.22` (`1.052x`, ovl) | `98.78 vs 95.56` (`1.034x`, ovl) | neutral |
| `pattern.bz2` (`22` tiny blks, `no_lanes=22`) | `4.52 vs 3.99` (`1.133x`, SLOWER) | `5.25 vs 4.88` (`1.074x`, ovl) | `5.20 vs 4.82` (`1.077x`, ovl) | `6.20 vs 5.49` (`1.130x`, SLOWER) | tiny-block tile setup (~0.5 ms total; bypass on `no_lanes` in build) |

*(Note: shrinking per-lane pre-allocation slack from `span/4` to `span/32` triggered `12/13` overflow fallbacks on `text.bz2` and regressed it `2.47x`; keeping `span/4` slack yields `0` overflows and `0.952–1.015x` across `text.bz2`.)*

---

## 7. Strict C2 Stop-Test Verdict & Next Step

Against the frozen `stop_test`:

1. **PASS (`forward-rle1-crc-vectored-build` admitted at `idea`):**
   - `fused_4l_tile4096_vec` (4-lane L1d/L2-chunked forward emit + zero-scratch single-pass in-place tile compaction + 64 KiB tiled PCLMULQDQ CRC32 + GF(2) `crc32_combine` + zero-stitch vectored 4-lane delivery):
     - **Matched-AVX2 (`-C target-cpu=native`):** beats `ctrl_4lane_unfused` by **`+49.9 ms` (`+3.82%` of 1,306 ms wall, non-overlapping `[94.24..102.91]` vs `[112.01..114.74]`) warm** — clearing **both** Branch 1 (`>= 39.18 ms` non-overlapping) and Branch 2 (`>= 25 ms` + `>= 50%` write cut) — and by **`+31.8 ms` (`+2.43%` of wall, non-overlapping `[139.74..143.89]` vs `[143.98..163.70]`) cold** — clearing Branch 2 (`>= 25 ms` + **`-53.4%` write traffic** + zero guard regressions on `runs`, `text`, `onebyte`).
     - **Baseline x86-64:** even handicapped by `#![forbid(unsafe_code)]` SSE2 vs control AVX2, `fused_4l_tile2048_vec` clears **both** Branch 1 (`+46.4 ms`, `+3.55%`, non-overlapping) and Branch 2 in warm mode and stays faster in cold mode (`+10.9 ms`, with `t1024_defcrc_vec` non-overlapping `+10.8 ms`); calling `src/rle.rs`'s existing runtime-dispatched AVX2 scanner in a striated build restores native parity.
   - **Prerequisites & architectural caveat:** Queued strictly behind `forward-emission` (which owns 4-lane forward anchors on `src/open.rs`). Requires **vectored 4-slice delivery** in `src/parallel.rs` (e.g., `Block` delivering `[&[u8]; 4]` or scatter-copying 4 slices directly to the caller's reader buffer without an intermediate stitch `memmove`) and bypassing tile fusion on small `no_lanes` blocks (`pattern.bz2`).
2. **Scoped STOP (recorded in `frontier/dead-ends.tsv`):**
   - 1-lane fused forward emit + RLE1/CRC (loses `-73..-186 ms` to 4-lane unfused on Open; loses `-11..-23 ms` cold even on `F1b/F4a`).
   - Inline per-step RLE1 FSM + pre-flagged `WordMeta` (regresses `-18..-255 ms` on 1-lane and 4-lane despite `0.24%` run-bearing words).
   - 4-lane contiguous stitched fusion (`74.9%` `memmove` leaves `-18.6%` write cut `< 50%` and flat/negative timing).
