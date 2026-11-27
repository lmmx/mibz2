# Cold dissection & redesign of Serial Scanner `A2` (`A2a..d`: selectors, code lengths, and `Huff::build_into`)

**Date:** 27 November 2026
**Branch / zone:** `explore/scanner-a2-dissection` (`Claim: none`, smooth zone only, zero `src/` edits, `#![forbid(unsafe_code)]`)
**Harness:** [`examples/scanner_a2_dissection.rs`](../../../examples/scanner_a2_dissection.rs)
**Hardware:** 2-vCPU Haswell Xeon @ 2.30 GHz KVM VM (32 KiB L1d / 256 KiB L2 / 45 MiB L3, no vPMU), pinned with `taskset -c 0`, 7 interleaved repeats with rotated arm order per repeat, reporting steal-free thread on-CPU time (`/proc/thread-self/schedstat` field 0) and wall-clock time across three compiler builds:
1. **Baseline x86-64** (`cargo build --release --example scanner_a2_dissection`, matching shipped wheels / workstation bench builds)
2. **Native** (`RUSTFLAGS="-C target-cpu=native"`, Haswell AVX2 + BMI2)
3. **Native + JCC-32B control** (`RUSTFLAGS="-C target-cpu=native -C llvm-args=-x86-align-branch-boundary=32 -C llvm-args=-x86-align-branch=fused+jcc+jmp"`, removing Intel JCC-erratum 32-byte branch-alignment noise)

**Verdict: PASS** (all frozen stop-test gates cleared non-overlapping across all three builds, zero `unsafe`, 100% bit-exact LUTs and symbol/bit-position parity across all 6 corpora + 1,002 synthetic edge-case tables). Queued build claim [`scanner-a2-fast-build`](../../../frontier/claims/scanner-a2-fast-build.toml) at `status = "idea"` behind `#1034` (`scanner-coalesced-u8-decode`).

---

## Frozen Stop Test (frozen in `examples/scanner_a2_dissection.rs` before any benchmark data)

- **PASS** if a 100% symbol-for-symbol and bit-position-exact safe-Rust (`#![forbid(unsafe_code)]`) `A2a..d` redesign:
  1. Cuts standalone `A2a..d` vs post-`#1034` baseline (`A2a+b+c + A2d_u8_gated64`) by `>= 25.0%` (non-overlapping 7-repeat thread-CPU spread), AND
  2. Cuts full post-`#1034`-style `A2a..d + A3` serial scan `S'` (coalesced `K=2` + `u8` LUT + no-zero `syms`) by `>= 3.0%` non-overlapping across BOTH baseline x86-64 and `-C target-cpu=native`, confirmed under Intel JCC-erratum 32B branch alignment control, with zero `A3` regression and no `> 2%` regression on off-wikidata guard corpora (`text`, `records`, `random`, `runs`, `onebyte`).
- **STOP** otherwise (record scoped dead-end in `frontier/dead-ends.tsv`).

### Verdict Summary (`wikidata.bz2`, 431 blocks, 7 interleaved repeats, steal-free thread CPU)

| Build | Standalone `A2a..d` (`A2_cand_tot_g64` vs `A2_post1034_tot`) | Gate (`>= 25%` disjoint) | Full `S'` (`S_all_g64` vs `S_post1034_ref`) | Full `S'` (`S_all_t16` vs `S_post1034_ref`) | Gate (`>= 3.0%` disjoint) |
|---|---:|:---:|---:|---:|:---:|
| **Baseline x86-64** | `6.057` vs `8.946 ms` (**−32.3%**, DISJOINT) | **PASS** | `75.27` vs `79.83 ms` (**−5.71%**, DISJOINT) | `74.72` vs `79.83 ms` (**−6.40%**, DISJOINT) | **PASS** |
| **Native (`target-cpu=native`)** | `5.560` vs `8.917 ms` (**−37.7%**, DISJOINT) | **PASS** | `70.80` vs `76.44 ms` (**−7.38%**, DISJOINT) | `72.03` vs `76.44 ms` (**−5.77%**, DISJOINT) | **PASS** |
| **Native + JCC-32B** | `5.082` vs `7.434 ms` (**−31.6%**, DISJOINT) | **PASS** | `69.35` vs `72.57 ms` (**−4.44%**, DISJOINT) | `68.96` vs `72.57 ms` (**−4.98%**, DISJOINT) | **PASS** |

Off-wikidata guard corpora (`S_all_g64` vs `S_post1034_ref` across all 3 builds): **`text` −2.47%..−3.96%**, **`records` −0.20%..−1.96%**, **`random` −0.37%..−0.58%**, **`runs` −14.55%..+0.70%**, **`onebyte` exact (`0.00 ms`)** — zero regressions anywhere.

---

## 1. Exact Corpus Census (6 corpora)

Pinned `wikidata.bz2` (SHA-256 `82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915`) plus 5 off-wikidata guard corpora (`text`, `records`, `random`, `runs`, `onebyte`):

| Corpus | Blocks (full / runt) | Selectors | Tables (used / unused) | `group_uses` (`0 / 1..4 / 5..15 / 16..63 / >=64`) | Used `max_l <= 10` / `> 10` | Mean short (`<= 9`) | Mean tail (`1024 - filled`) | `A2c` lens (`1b / 3b / 5b / 7b / slow`) | Multi-LUT lookups (lens/lookup) |
|---|---:|---:|---:|---|---:|---:|---:|---|---:|
| **`wikidata`** | 431 (233 / 198) | 501,448 | 1,796 (1,606 / 190) | `190 / 85 / 123 / 0 / 1398` | 208 / 1,398 | 25.5 | **9.1** | 133,280 (`68.5% / 25.7% / 4.8% / 0.8% / 0.08%`) | 38,573 (**3.46**) |
| **`text`** | 26 (26 / 0) | 194,528 | 156 (156 / 0) | `0 / 0 / 0 / 0 / 156` | 0 / 156 | 32.7 | 12.0 | 21,066 (`57.5% / 31.3% / 8.6% / 2.3% / 0.31%`) | 6,914 (**3.05**) |
| **`records`** | 22 (22 / 0) | 384,009 | 132 (132 / 0) | `0 / 0 / 0 / 0 / 132` | 0 / 132 | 255.0 | **1.0** | 34,056 (`84.0% / 14.8% / 0.3% / 0.6% / 0.40%`) | 6,489 (**5.25**) |
| **`random`** | 18 (18 / 0) | 319,989 | 108 (108 / 0) | `0 / 0 / 0 / 0 / 108` | 0 / 108 | 146.2 | 5.9 | 27,864 (`82.9% / 13.4% / 1.5% / 0.4% / 1.75%`) | 5,839 (**4.77**) |
| **`runs`** | 1 (1 / 0) | 2,484 | 6 (6 / 0) | `0 / 0 / 0 / 0 / 6` | 0 / 6 | 144.2 | 14.8 | 1,524 (`64.6% / 29.8% / 4.7% / 0.8% / 0.07%`) | 408 (**3.74**) |
| **`onebyte`** | 1 (0 / 1) | 1 | 2 (1 / 1) | `1 / 1 / 0 / 0 / 0` | 1 / 0 | 5.0 | 0.0 | 10 (`50.0% / 40.0% / 0.0% / 10.0% / 0.00%`) | 10 (**1.00**) |

### Structural findings from the census

1. **Clean bimodal table-use distribution on `wikidata`:** Every runt table has `<= 15` uses (`1..4`: 85; `5..15`: 123), **zero** tables sit in `16..63`, and every full-block used table has `>= 64` uses (`1,398`). Thus `g64` (block-level `n_sel >= 64`) and `t16` (per-table `group_uses[g] >= 16`) build the exact same `1,398` `K=2` tables on `wikidata`, while `t16` also protects future mixed blocks that contain a rarely used table (`< 16` groups).
2. **`A2d` wastes 99.1% of its `d_arr` zeroing and double-writes the entire 1,024-entry `lut`:** `src/huff.rs::build_into` (`#1034` post-u8) first zeroes `lut[0..1024]` and `d_arr[0..1024]`, fills `lut[0..filled_to]` with `K=1` single-symbol entries, and then runs Pass 2 to overwrite the **same** `1,015` entries (`1024 - 9.1` mean tail!) with `K=2` pairs. Worse, `filled_to` averages `1,014.9 / 1,024` (`tail = 9.1` entries on `wikidata`, `1.0` on `records`, `5.9` on `random`), and every `K=2` fill calls `self.lut[..].fill(e)` inside an inner loop over `s2`, paying redundant `K=1` stores and unvectorized short fills.
3. **`A2b` selector MTF decodes 501,448 individual bytes AND `#1034` re-scans them in `A3`:** Shipped `A2b` (`src/block.rs`) reads one unary selector at a time via `leading_ones()` into `sel_buf[0..n_sel]` (`501,448` byte stores + MTF shift-register updates), and `#1034`'s coalesced `A3` loop then re-scans `while sel_buf[sel + rem] == g` across those same `501,448` bytes to compute run lengths! Because `62.4%` of selectors on `wikidata` are `j = 0` (`0`-bit repeats of the active MTF front), consecutive `0`-bits directly encode the run length of the active table `pos & 0xff`.
4. **`A2c` delta code lengths spend 94.2% of symbols on 1-bit (`68.5%`) or 3-bit (`25.7%`) deltas:** Shipped `A2c` decodes one code length at a time with an inner bit-by-bit `while r.bit()` loop (`133,280` iterations on `wikidata`). An 8-bit `const`-generated multi-symbol lookup table (`DELTA_LUT_MULTI: [MultiDelta; 256]`, `4 KiB`, fitting effortlessly in L1d) decodes up to **8 consecutive unchanged `0`-bits or 1–6 mixed `0`/`10`/`11` delta lengths per table lookup**, cutting `133,280` iterations to **`38,573` lookups (`3.46 lens/lookup` on `wikidata`, `5.25` on `records`, `4.77` on `random`)**.

---

## 2. Three Winning Safe-Rust Mechanisms (`#![forbid(unsafe_code)]`)

### 2.1 `A2b_direct_runs` (`parse_a2b_direct_runs`): branchless 1-iteration-per-run `leading_zeros` + direct `(group, run_len)` emission

Instead of decoding one selector per iteration into `sel_buf[..n_sel]` and re-scanning `sel_buf` in `A3`:
- Pack the 6-entry MTF state in a single `u64` (`pos = 0x05_04_03_02_01_00`, front group in byte 0).
- While `rem > 0`, refill `r.buf` (`live >= 56`) and count consecutive leading `0`-bits (`z = r.buf.leading_zeros().min(rem).min(r.live)`) in **one instruction (`lzcnt`)**:
  - Every leading `0`-bit is a `j == 0` selector repeating the current front group `g = (pos & 0xff) as u8`.
  - Accumulate directly into the active run length (`run_len += z`) with no per-selector loop and no `sel_buf` byte store.
  - When a `1`-bit stops the run (`r.buf`'s MSB is `1`), read the unary MTF index `j = (!r.buf).leading_zeros()` (checking `j < n_groups` and `r.live > j`), emit `(g, run_len)` directly into `runs: &mut [(u8, u16)]` (splitting only if `run_len > u16::MAX`, impossible since `MAX_SELECTORS = 18002`), accumulate `group_uses[g] += run_len`, and rotate `pos` branchlessly via `LOW_MASK[j]`:
    ```rust
    let m = LOW_MASK[j];
    let v = (pos >> (j * 8)) & 0xff;
    pos = (pos & !m) | ((pos & (m >> 8)) << 8) | v;
    ```
- **Double win:** Cuts standalone `A2b` by **−33.1% to −38.5% DISJOINT** on `wikidata` (and **−7.7% to −11.2% DISJOINT** on `text`, **−5.6% to −13.3% DISJOINT** on `random`), AND eliminates `#1034`'s `501,448`-byte `sel_buf` re-scan inside `A3` — which is why joint `S_all_g64` saves **more milliseconds** than standalone `A2` (`−4.56 ms` base / `−5.64 ms` native / `−3.22 ms` JCC-32B vs `−2.89` / `−3.36` / `−2.35 ms` standalone `A2`!). Even on `records` (where short runs cause standalone `A2b_direct_runs` to run `+16..+23%` slower), eliminating the `A3` re-scan nets **−0.20% to −1.96% faster** joint `S'` across all 3 builds!

### 2.2 `A2c_lut_multi` (`parse_a2c_lut_multi`): `4 KiB` compile-time 8-bit multi-symbol delta LUT

- Precomputed `const fn build_delta_lut_multi() -> [MultiDelta; 256]` (`#[repr(C)]`, 16 B/entry = `4 KiB` static RO table) maps every 8-bit prefix `peek8 = (r.buf >> 56) as usize` to `(count: u8, bits: u8, deltas: [i8; 8], min_off: i8, max_off: i8)`:
  - Up to 8 consecutive code-length deltas decoded in a single lookup (`0` = `+0` in 1 bit, `10` = `+1` in 3 bits, `11` = `-1` in 3 bits, chained as long as the complete code fits inside 8 bits).
  - Range validation (`1..=20`) is checked in **O(1) per lookup** (`curr + min_off >= 1 && curr + max_off <= 20`) instead of per symbol, falling back to `parse_a2c_one_slow` only when `count == 0` (`>= 9`-bit multi-step delta, `0.08%` of symbols on `wikidata`) or when `curr` nears the `1`/`20` boundary or `< 8` symbols remain in the table.
- Cuts standalone `A2c` by **−63.1% to −66.7% DISJOINT** on `wikidata` (`0.954 -> 0.352 ms` base, `0.961 -> 0.320 ms` native, `0.956 -> 0.319 ms` JCC-32B), **−75.3% to −80.0% DISJOINT** on `text`, **−47.2% to −75.0% DISJOINT** on `records`, and **−48.2% to −66.7% DISJOINT** on `random`.

### 2.3 `A2d_direct_g64` / `A2d_direct_t16` (`Huff::build_direct_single_write`): single-write fused `K=2` LUT + `split_at_mut` + tail-only fill

Instead of zeroing `lut[0..1024]` + `d_arr[0..1024]`, filling `K=1` into `lut`, and then overwriting `lut` with `K=2` pairs:
1. Compute canonical counts, `first_code`, `base`, `limit`, and sorted `perm` in one pass, recording only the short codes (`len <= LUT_BITS`) in compact stack arrays (`short_sym`, `short_len`, `short_end` — mean `25.5` symbols on `wikidata`, max `258`).
2. **Single-write fused `K=2` root fill (when `build_pairs`):** For each first short symbol `s1` (`l1 <= 9`, `s1 < 256` and `s1 != eob`), directly emit its `K=2` paired children `(s1, s2)` for all `s2` with `l1 + l2 <= 10` straight into `lut[slot..slot + span]` using `split_at_mut` (eliminating bounds checks and enabling auto-vectorized slice fills), and fill only the trailing unpaired remainder slots (`l1 + l2 > 10`) with the single-symbol `K=1` entry. Every entry in `lut[0..filled_to]` is written **exactly once** (zero double-writes, zero `d_arr[0..1024]` array at all!).
3. **Tail-only long-code / sub-LUT fill (`filled_to..1024`):** Only the remaining `1024 - filled_to` entries (mean **`9.1` entries** on `wikidata`, `1.0` on `records`, `5.9` on `random`) ever touch `build_tail_sublut` or `BAD` fill.
4. **Bit-exactness verified:** `1,002/1,002` synthetic edge-case tables (`alpha = 258`, `sym = 256`, `eob = 257`, single-symbol, max-length 20, incomplete, oversubscribed, 500 randomized valid/incomplete tables) AND all `2,009` real used tables across all 6 corpora produce **bit-for-bit identical** `lut[0..1024]`, `sub_lut`, `limit`, `base[..=12]`, `perm`, `min_len`, and `max_len` vs `build_post1034`!
- Cuts standalone `A2d` vs `A2d_post1034_g64` by **−24.0% DISJOINT** baseline (`4.479 -> 3.405 ms`), **−36.7% DISJOINT** native (`5.020 -> 3.179 ms`), and **−24.8% DISJOINT** native+JCC-32B (`3.762 -> 2.831 ms`; `A2d_direct_t16` **−25.1% DISJOINT**).

---

## 3. Full Benchmark Tables (7 Interleaved Repeats, Steal-Free Thread CPU)

### 3.1 Primary Target: `wikidata.bz2` (431 blocks: 233 full, 198 runts; 24,979,299 symbols)

#### Standalone `A2` sub-phases and combined `A2a..d` (`ms/pass`, 7 repeats × 25 iters)

| Arm | Baseline x86-64 CPU (`min..max`) | vs ref | Native CPU (`min..max`) | vs ref | Native + JCC-32B CPU (`min..max`) | vs ref |
|---|---:|---:|---:|---:|---:|---:|
| `A2a_header` | `0.000 [0.000..0.159]` | — | `0.011 [0.000..0.160]` | — | `0.000 [0.000..0.160]` | — |
| `A2b_shipped` *(ref)* | `3.217 [3.148..3.461]` | ref | `2.649 [2.558..2.751]` | ref | `2.592 [2.545..2.824]` | ref |
| `A2b_fast_mtf` *(ruled out)* | `3.859 [3.705..3.977]` | `+20.0%` | `3.672 [3.522..3.838]` | `+38.6%` | `3.245 [3.190..3.307]` | `+25.2%` |
| **`A2b_direct_runs`** | **`1.982 [1.899..2.068]`** | **`−38.4%` DISJ** | **`1.773 [1.600..1.912]`** | **`−33.1%` DISJ** | **`1.595 [1.588..1.785]`** | **`−38.5%` DISJ** |
| `A2c_shipped` *(ref)* | `0.954 [0.884..1.035]` | ref | `0.961 [0.954..1.153]` | ref | `0.956 [0.752..0.960]` | ref |
| `A2c_lut1` *(1 KiB, 1 sym/lkup)* | `0.797 [0.672..0.864]` | `−16.5%` DISJ | `0.793 [0.628..0.800]` | `−17.5%` DISJ | `0.797 [0.792..0.852]` | `−16.6%` |
| **`A2c_lut_multi`** *(4 KiB, 3.46/lkup)* | **`0.352 [0.317..0.478]`** | **`−63.1%` DISJ** | **`0.320 [0.317..0.479]`** | **`−66.7%` DISJ** | **`0.319 [0.286..0.403]`** | **`−66.6%` DISJ** |
| `A2d_shipped_all` *(pre-#1034)* | `4.762 [4.631..4.895]` | — | `5.270 [5.111..5.496]` | — | `3.939 [3.826..4.581]` | — |
| `A2d_post1034_g64` *(ref)* | `4.479 [4.291..4.619]` | ref | `5.020 [4.928..5.212]` | ref | `3.762 [3.656..3.825]` | ref |
| `A2d_split_vec_g64` | `4.192 [4.148..4.337]` | `−6.4%` | `4.108 [3.993..4.745]` | `−18.2%` DISJ | `3.671 [3.611..3.819]` | `−2.4%` |
| **`A2d_direct_g64`** | **`3.405 [3.355..3.511]`** | **`−24.0%` DISJ** | **`3.179 [3.035..3.325]`** | **`−36.7%` DISJ** | **`2.831 [2.768..2.990]`** | **`−24.8%` DISJ** |
| **`A2d_direct_t16`** | **`3.487 [3.398..3.568]`** | **`−22.2%` DISJ** | **`3.188 [3.083..4.309]`** | **`−36.5%` DISJ** | **`2.820 [2.787..3.017]`** | **`−25.1%` DISJ** |
| `A2d_direct_t16_sub4` | `3.367 [3.339..3.498]` | `−24.8%` DISJ | `3.188 [3.085..3.263]` | `−36.5%` DISJ | `2.871 [2.711..3.020]` | `−23.7%` DISJ |
| `A2_post1034_tot` *(ref)* | `8.946 [8.779..9.564]` | ref | `8.917 [8.784..9.088]` | ref | `7.434 [7.283..8.224]` | ref |
| **`A2_cand_tot_g64`** | **`6.057 [5.905..6.157]`** | **`−32.3%` DISJ** | **`5.560 [5.422..5.622]`** | **`−37.7%` DISJ** | **`5.082 [5.011..5.140]`** | **`−31.6%` DISJ** |
| **`A2_cand_tot_t16`** | **`5.987 [5.773..6.157]`** | **`−33.1%` DISJ** | **`5.585 [5.412..5.803]`** | **`−37.4%` DISJ** | **`5.146 [4.981..5.315]`** | **`−30.8%` DISJ** |

#### Joint End-to-End Post-`#1034` Serial Scan `S'` (`A2a..d + A3`, `ms/pass`, 7 repeats × 5 iters)

| Arm | Baseline x86-64 CPU (`min..max`) | vs ref | Native CPU (`min..max`) | vs ref | Native + JCC-32B CPU (`min..max`) | vs ref |
|---|---:|---:|---:|---:|---:|---:|
| `S_post1034_ref` *(ref)* | `79.83 [78.63..82.53]` | ref | `76.44 [75.35..76.99]` | ref | `72.57 [72.26..76.36]` | ref |
| `S_fast_a2b_only` | `76.71 [76.37..77.43]` | `−3.91%` DISJ | `72.64 [72.26..73.59]` | `−4.98%` DISJ | `71.41 [70.14..73.89]` | `−1.61%` |
| `S_fast_a2c_only` | `78.83 [78.03..79.65]` | `−1.26%` | `75.89 [74.34..78.92]` | `−0.73%` | `72.47 [71.81..76.78]` | `−0.15%` |
| `S_fast_a2d_only` | `78.27 [77.86..80.26]` | `−1.95%` | `74.22 [73.24..75.65]` | `−2.90%` | `72.10 [71.35..74.75]` | `−0.65%` |
| **`S_all_g64`** | **`75.27 [74.20..76.27]`** | **`−5.71%` DISJ** | **`70.80 [70.05..71.48]`** | **`−7.38%` DISJ** | **`69.35 [68.22..70.11]`** | **`−4.44%` DISJ** |
| **`S_all_t16`** | **`74.72 [73.93..75.10]`** | **`−6.40%` DISJ** | **`72.03 [69.62..73.10]`** | **`−5.77%` DISJ** | **`68.96 [68.56..71.52]`** | **`−4.98%` DISJ** |
| `S_all_t16_sub4` | `75.38 [74.61..78.25]` | `−5.57%` DISJ | `70.84 [70.33..72.05]` | `−7.32%` DISJ | `70.17 [68.87..70.63]` | `−3.31%` DISJ |
| `S_all_fastmtf_g64` *(ruled out)* | `78.52 [78.19..80.68]` | `−1.63%` | `74.60 [74.43..75.61]` | `−2.41%` | `72.26 [71.84..74.03]` | `−0.43%` |

Note on JCC-32B alignment control: unmitigated native `A2d_post1034_g64` (`5.020 ms`) carried a `~1.26 ms` 32B branch-alignment penalty that vanished under `-x86-align-branch-boundary=32` (`3.762 ms`), and `S_post1034_ref` dropped from `76.44 ms` to `72.57 ms`. Crucially, **even after removing all JCC alignment artifacts**, `A2_cand_tot_g64` still cuts `A2a..d` by **−31.6% DISJOINT** (`7.434 -> 5.082 ms`) and `S_all_g64`/`S_all_t16` still cut full `S'` by **−4.44% / −4.98% DISJOINT** (`72.57 -> 69.35 / 68.96 ms`).

---

### 3.2 Off-Wikidata Guard Corpora (Joint End-to-End `S'`, 7 repeats, steal-free thread CPU)

| Corpus | Build | `S_post1034_ref` (ms) | `S_all_g64` (ms, vs ref) | `S_all_t16` (ms, vs ref) | Standalone `A2_cand_tot_g64` vs `A2_post1034_tot` |
|---|---|---:|---:|---:|---:|
| **`text`** (26 blks) | Base | `32.36 [32.24..33.89]` | `31.56 [31.30..33.33]` (**−2.47%**) | `31.73 [31.48..31.97]` (**−1.94%** DISJ) | `1.705` vs `2.072 ms` (**−17.7%** DISJ) |
| **`text`** | Native | `31.43 [31.05..32.84]` | `30.19 [29.72..30.41]` (**−3.96%** DISJ) | `30.23 [29.77..30.71]` (**−3.84%** DISJ) | `1.526` vs `1.921 ms` (**−20.6%** DISJ) |
| **`text`** | Native+JCC | `30.55 [30.28..32.55]` | `29.71 [29.48..30.31]` (**−2.73%**) | `29.70 [28.96..31.00]` (**−2.78%**) | `1.387` vs `1.645 ms` (**−15.7%** DISJ) |
| **`records`** (22 blks) | Base | `94.58 [93.12..98.20]` | `93.57 [92.55..94.42]` (**−1.06%**) | `93.48 [92.31..96.40]` (**−1.16%**) | `3.452` vs `3.143 ms` (`+9.8%` ovl) |
| **`records`** | Native | `89.42 [88.51..91.88]` | `87.67 [87.28..90.52]` (**−1.96%**) | `87.95 [86.77..92.12]` (**−1.65%**) | `3.065` vs `2.674 ms` (`+14.6%` ovl) |
| **`records`** | Native+JCC | `87.04 [86.13..88.10]` | `86.86 [86.33..88.28]` (**−0.20%**) | `87.71 [86.37..89.29]` (`+0.77%` ovl) | `2.761` vs `2.432 ms` (`+13.5%` ovl) |
| **`random`** (18 blks) | Base | `73.72 [72.17..74.54]` | `73.29 [71.37..73.80]` (**−0.58%**) | `73.22 [72.26..76.19]` (**−0.68%**) | `2.222` vs `2.568 ms` (**−13.5%** DISJ) |
| **`random`** | Native | `70.14 [69.80..71.43]` | `69.73 [69.05..70.64]` (**−0.58%**) | `69.36 [68.68..70.61]` (**−1.11%**) | `2.026` vs `2.214 ms` (**−8.5%** DISJ) |
| **`random`** | Native+JCC | `67.95 [67.14..69.41]` | `67.70 [66.60..70.29]` (**−0.37%**) | `67.31 [66.75..67.53]` (**−0.95%**) | `1.791` vs `1.995 ms` (**−10.2%** DISJ) |
| **`runs`** (1 blk) | Base / Nat / JCC | `0.40 / 0.47 / 0.40` | `0.40 / 0.40 / 0.40` (`+0.70% / −14.55% / −0.06%`) | `0.50 / 0.40 / 0.40` | `0.040` vs `0.040 ms` |
| **`onebyte`** (1 runt) | Base / Nat / JCC | `0.00 / 0.00 / 0.00` | `0.00 / 0.00 / 0.00` (exact, `< 10 us`) | `0.00 / 0.00 / 0.00` | `0.000` vs `0.000 ms` |

---

## 4. Ruled-Out Variants (Negative Findings)

1. **`A2b_fast_mtf` (per-selector `if (r.buf as i64) >= 0` branch writing `sel_buf`):** Regresses **+20.0% to +63.5%** across `wikidata` (`+20.0% / +38.6% / +25.2%`), `text` (`+34.3%..+52.3%`), `records` (`+18.6%..+38.3%`), and `random` (`+39.2%..+63.5%`), turning joint `S_all_fastmtf_g64` into a `+1.3%..+2.1%` regression on `text`/`random`. Cause: `j == 0` occurs on `62.4%` of selectors on `wikidata`, making the branch near-worst-case unpredictable (`~38%` misprediction rate) vs shipped's branchless `leading_ones()` + mask rotation, whereas `A2b_direct_runs` counts the whole `0`-bit run branchlessly in one `lzcnt` and skips `sel_buf` completely.
2. **`A2c_lut1` (`1 KiB` single-symbol delta LUT):** Cuts `wikidata` (`−16.5%..−17.5%`) and `text` (`−25.8%..−40.0%`), but **regresses `+24.6%..+66.4%` on `records` and `+1.0%..+33.3%` on `random`** because `83–84%` of code lengths on uniform tables are `1b` (`0`-bit unchanged), where shipped's predicted `if r.bit() == 0` branch beats a single-symbol table lookup. `A2c_lut_multi` solves this completely by consuming up to 8 `0`-bits per lookup (`5.25` lens/lookup on `records`, **−47.2%..−75.0% DISJOINT**).
3. **`A2d_split_vec_g64` (`split_at_mut` + tail-only `d_arr` without single-write fusion):** Saves `−6.4%` base / `−18.2%` native / `−2.4%` JCC-32B on `wikidata` — decisively superseded by single-write fused `K=2` `A2d_direct_g64` (`−24.0% / −36.7% / −24.8%` DISJOINT).
4. **`sub_bits = 4` (`A2d_direct_t16_sub4`):** Virtually indistinguishable from shipped `sub_bits = 6` (`3.367` vs `3.405 ms` base, `3.188` vs `3.179 ms` native, `2.871` vs `2.831 ms` JCC-32B), and slightly noisier in joint `S'` (`70.17` vs `69.35 ms` JCC-32B); keep shipped `sub_bits = 6`.

---

## 5. Exactness Audit & Modeled i9-10900K Impact

- **Synthetic edge-case LUT parity (`verify_synthetic_edge_cases`):** `1,002 / 1,002` tables bit-identical between `build_post1034` and `build_direct_single_write` (covering `alpha = 258`, `sym = 256`, `eob = 257`, single-symbol tables, `max_len = 20`, incomplete trees, oversubscribed trees, and 500 seeded random tables).
- **Real corpus parity:** `2,009 / 2,009` used tables bit-identical across all 6 corpora (`1,606` wikidata + `156` text + `132` records + `108` random + `6` runs + `1` onebyte), AND 100% symbol-for-symbol and ending-bit-position parity against `mibz2::stream::Scanner` across all 8 full `A2+A3` configurations on all 6 corpora (`> 65M` symbols verified per run).
- **Modeled i9-10900K impact:** Post-`#1034` serial scan `S'` is modeled at `~145 ms` on `wikidata x4` (`frontier/surface.tsv` row `S`). Applying the verified `−4.44%` (JCC-32B conservative) to `−7.38%` (native) joint `S'` reduction (`−5.71%` baseline x86-64) removes **`~6.4–10.7 ms` directly from serial `S'`** (`~145 ms -> ~134–139 ms`), lowering the pure scan-bound ceiling at `8–10W` directly and relaxing the `140 ms` slack-hybrid 8W worker budget (`(8*W'/8 + S')/9 + 4.85 ms`, `#1056`) by `~0.8–1.2 ms/w` — all in 100% safe Rust (`#![forbid(unsafe_code)]`) with zero new memory allocations (`runs: Vec<(u8, u16)>` replaces `sel_buf: Vec<u8>`).

---

## 6. Follow-Up Action

Opened striated build claim [`frontier/claims/scanner-a2-fast-build.toml`](../../../frontier/claims/scanner-a2-fast-build.toml) at `status = "idea"` (`owned_paths = ["src/block.rs", "src/huff.rs"]`, `mechanisms = ["scanner-a2-build"]`, queued cleanly behind `#1034`'s `scanner-coalesced-u8-decode`) with a frozen workstation stop test.
