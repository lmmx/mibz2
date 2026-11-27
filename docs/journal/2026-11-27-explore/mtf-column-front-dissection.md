# Dissecting `src/mtf.rs` (`Column`-route MTF/RLE2 front) on off-Wikidata corpora

*Smooth-zone probe (`Claim: none`, zero `src/` edits; probe code in [`examples/mtf_column_front_probe.rs`](../../../examples/mtf_column_front_probe.rs)).*

---

## 1. Motivation and scope

[`segmented-fallback-dissection.md`](segmented-fallback-dissection.md) (#1042) exposed a second large, completely unheld bottleneck on off-Wikidata `Column` blocks: **`src/mtf.rs` (`mtf::mtf` / `mtf_avx2`)** takes **52.9% of total worker time** on `elf` (`110.12 ms` vs `build 40.89 ms`, `walk+asm 47.17 ms`, `rle+crc 9.99 ms`) and **43.6%** on `prose` (`39.62 ms`).

This probe:
1. Quantifies the exact symbol and MTF-index (`nn = s - 1`) distribution across deterministic off-Wikidata `Column` corpora (`elf`, `prose`, `pysrc`, `random`, `records`) plus `Runs`-route guard corpora (`json`, `rustsrc`, `text`, `wikidata`).
2. Dissects `src/mtf.rs`'s compiled `mtf_avx2` machine code (`objdump -d`) and isolates the loop-shell (`s <= 1` / RLE2 / `counts` / `out` stores) cost from the MTF list cost using non-decoding skeleton floors.
3. Evaluates 100% byte-exact candidate levers across 7 interleaved repeats on **both** baseline x86-64 and `-C target-cpu=native` (reporting both steal-free `CLOCK_THREAD_CPUTIME_ID` and wall `Instant::now()`).

**Host (C5 diagnostic note):** 2-vCPU Intel Xeon @ 2.30 GHz KVM VM (Haswell/Broadwell, AVX2, BMI2, L1d 32 KiB, L2 256 KiB, L3 45 MiB).

---

## 2. Phase 1 dissection: symbol census, `objdump -d`, and skeleton floor

### 2.1 Symbol and MTF-index distribution across 9 corpora

(`*` denotes corpora with 0 production `Column` blocks, shown over all blocks for reference/guard comparison.)

| corpus | blks | `Column` blks | `col_bytes` | literals | `RUNA/B`% | `<16`% | `16..32`% | `32..64`% | `64..128`% | `128+`% | `flip32`% | `flip64`% | `ship_ms` | `skel_ms` | `ns/lit` |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| `elf` | 20 | 19 | 17,024,563 | 7,526,039 | 27.2% | 61.5% | 9.5% | 8.6% | 9.6% | 10.7% | 27.3% | 22.5% | 159.72 | 82.50 | 21.22 |
| `prose` | 11 | 9 | 8,099,830 | 3,256,397 | 32.5% | 84.0% | 9.3% | 5.6% | 1.0% | 0.0% | 9.9% | 1.9% | 56.23 | 41.46 | 17.27 |
| `pysrc` | 11 | 2 | 1,799,962 | 577,426 | 36.9% | 87.1% | 8.2% | 3.6% | 1.1% | 0.0% | 8.1% | 2.0% | 7.31 | 5.45 | 12.67 |
| `random` | 18 | 18 | 16,000,004 | 15,875,136 | 0.8% | 11.8% | 12.5% | 24.8% | 47.3% | 3.7% | 36.7% | 49.6% | 199.39 | 45.52 | 12.56 |
| `records` | 22 | 22 | 19,200,003 | 19,139,317 | 0.3% | 4.5% | 4.9% | 14.5% | 27.5% | 48.5% | 17.1% | 36.3% | 220.49 | 53.45 | 11.52 |
| `json*` | 12 | 0 | 10,008,903 | 1,168,856 | 38.1% | 92.7% | 5.3% | 1.7% | 0.3% | 0.0% | 3.6% | 0.6% | 13.96 | 10.36 | 11.94 |
| `rustsrc*` | 6 | 0 | 4,650,694 | 1,152,247 | 42.1% | 85.4% | 9.7% | 4.2% | 0.7% | 0.0% | 8.6% | 1.4% | 16.37 | 12.54 | 14.21 |
| `text*` | 24 | 0 | 20,922,262 | 5,155,962 | 42.3% | 85.6% | 9.6% | 4.1% | 0.7% | 0.0% | 8.5% | 1.4% | 73.75 | 56.75 | 14.30 |
| `wikidata` | 431 | 48 (runts) | 465 | 464 | 0.2% | 97.2% | 2.8% | 0.0% | 0.0% | 0.0% | 0.0% | 0.0% | 0.00 | 0.00 | 9.94 |

### 2.2 Skeleton micro-dissection and `objdump -d` findings

Skeleton ablations on `Column` blocks (best of 3 thread-CPU ms, baseline x86-64) decompose `shipped` into two distinct, nearly orthogonal bottlenecks:

| corpus (`Column` blks) | `shipped` | `sk_ship` (no MTF list) | `sk_8Bst` (+8B branch) | `sk_nocnt` (no `counts`) | `sk_nobr` (no `s<=1` / `run!=0` branches) | MTF list share | RLE2 branch share |
|---|---:|---:|---:|---:|---:|---:|---:|
| `elf` (19) | 107.15 | 60.39 | 63.91 | 56.49 | 20.34 | **43.6%** (`46.8 ms`) | **37.4%** (`40.1 ms`) |
| `prose` (9) | 40.41 | 29.66 | 31.37 | 28.14 | 9.65 | **26.6%** (`10.8 ms`) | **49.5%** (`20.0 ms`) |
| `pysrc` (2) | 8.01 | 5.96 | 6.39 | 5.64 | 1.79 | **25.6%** (`2.1 ms`) | **52.1%** (`4.2 ms`) |
| `random` (18) | 212.05 | 58.08 | 58.16 | 48.68 | 31.85 | **72.6%** (`154.0 ms`) | **12.4%** (`26.2 ms`) |
| `records` (22) | 230.22 | 68.43 | 68.41 | 57.45 | 38.30 | **70.3%** (`161.8 ms`) | **13.1%** (`30.1 ms`) |

`objdump -d` of `mibz2::mtf::mtf_avx2` confirms the exact mechanisms:
1. **RLE2 branch mispredictions (`s <= 1` and `run != 0`):** On `prose` (`32.5%` run symbols) and `elf` (`27.2%` run symbols), alternating run/literal transitions cost **49.5%** of `prose` time and **37.4%** of `elf` time purely in branch mispredictions (`sk_ship 29.66 -> sk_nobr 9.65 ms` on `prose`; `60.39 -> 20.34 ms` on `elf`).
2. **Memory-resident `second()` tail + `nn < 32` / `nn < 64` mispredictions:** `mtf_avx2` holds only `0..31` in registers (`xmm9`, `xmm10`) and keeps `32..255` as 7 YMM rows on the stack (`0x40(%rsp)..0x100(%rsp)`). On `nn >= 64` (`20.3%` on `elf`, `51.0%` on `random`, `76.0%` on `records`), `second()` issues 7 stack YMM loads, 7 × 5 vector ops (`vpminub + vpcmpeqb + vperm2i128 + vpalignr + vpblendvb`), 7 stack YMM stores, plus a scalar byte load from `tail[t]` hitting store-forwarding latency, while `nn < 32` and `nn < 64` flip up to `36.7%` and `49.6%` of literals.
3. **`memset` call-clobber spills:** `std::ptr::write_bytes` emits `vzeroupper` + `call memset@GLIBC` on every `run > 64`, forcing `max` (`0x10(%rsp)`) and `counts` (`0x20(%rsp)`) to spill and reload from the stack on every single literal.

---

## 3. Frozen candidate levers and decision rule (frozen BEFORE candidate timing)

### Candidate levers (all 100% exact vs `mtf::mtf_scalar` on `ll`, `counts`, and `Result<usize, ErrorKind>`)

1. **`A_brl_rle_tier32`:** Branchless RLE2 (`STEPS[0]` no-op on `s <= 1`, unconditional 32-byte `vmovdqu` run store + `counts[c0] += emit_run`, merged `s < min(33, eob)` guard, inline AVX2 long-run fill with zero `memset` call) paired with signed `vpcmpgtb` (`^ 0x80`) stack `second()`.
2. **`B_reg8_tier32`:** All 256 MTF bytes pinned in SIMD registers (`f0, f1` XMM + `y1..y7` YMM, zero stack list, zero `memset` call, signed `vpcmpgtb` shift masks, 3-level register mux byte extraction for `nn >= 32`, zero `t < 32` branch), keeping branched RLE2.
3. **`C_reg8_branchless`:** Full 256-byte branchless in-register 8×YMM (`y0..y7`), zero branches on `nn`, signed `vpcmpgtb` masks, inline AVX2 long-run fill, keeping branched RLE2.
4. **`D_brl_rle_reg8_tier64`:** Combined branchless RLE2 + 64-byte L1 in 4 XMM registers (`f0..f3`, covering `98.9%` of `prose`/`pysrc` and `79.6%` of `elf` at 3-cycle latency) + 6 YMM registers (`y2..y7`) for `nn >= 64`, zero stack list, zero `memset` call.
5. **`E_brl_rle_reg8_branchless`:** Combined branchless RLE2 + full 256-byte branchless 8×YMM (`y0..y7`), zero branches on `s <= 1`, `run != 0`, or `nn`.
6. **`F_adaptive_reg8`:** Per-block $O(1)$ 128-symbol sample (~100 ns/block, zero extra memory) dispatching each block to the matching in-register kernel (`D_brl_rle_reg8_tier64`, `C_reg8_branchless`, or `E_brl_rle_reg8_branchless`).

### Frozen decision rule

- **Exactness gate:** 100% byte-identical `ll`, `counts`, and `Result<usize, ErrorKind>` against `mtf::mtf_scalar` on **every block** of all 9 corpora (both `Column` blocks and forced guard blocks on `wikidata`/`text`/`json`/`rustsrc`) plus synthetic error/edge tests (`RunTooLong`, `BlockTooLong`, `s > 256` clamp, trailing run without EOB, empty block, tiny `eob < 33`).
- **Performance PASS gate:** Admit an `idea` build claim (`mtf-column-front-build`, `owned_paths = ["src/mtf.rs"]`) **ONLY IF** a candidate lever cuts `src/mtf.rs` front time (steal-free `CLOCK_THREAD_CPUTIME_ID` median across 7 interleaved repeats) by **`>= 15.0%`** with **non-overlapping `[min..max]` spreads** vs `shipped` on **BOTH** baseline x86-64 and `-C target-cpu=native` across the primary `Column` aggregate (`elf + prose + pysrc + random + records`), with **clear improvement on `elf` and `prose` individually**, and **zero regression (`<= +1.0%` or overlapping)** on `wikidata` / `text` guards.
- **STOP rule:** If no candidate clears all criteria, verdict is **STOP** and we record a scoped row in `frontier/dead-ends.tsv` without admitting a build claim.

---

## 4. Candidate benchmark results (7 interleaved repeats, 100% exact across all arms)

Every arm across all four rounds passed **100% exactness verification** against `mtf::mtf_scalar` on `ll`, `counts`, and `Result<usize, ErrorKind>` across all **555 blocks** of the 9 corpora plus **10 synthetic edge/error cases** (`empty_eob`, `tiny_eob_5`, `long_run_and_tail`, `exact_max_len`, `err_block_too_long_run`, `err_block_too_long_lit`, `err_run_too_long`, `trailing_run_no_eob`, `full_nn_sweep_255`, `clamp_over_256`).

### 4.1 Round 1 — Pre-registered branchless RLE2 and in-register 8×YMM arms (`bench`, default flags)

**Baseline x86-64 (3,955 / 3,955 exact):**

| arm | 5-corp CPU med `[min..max]` | Δ% | disj | `elf` | `prose` | `pysrc` | `random` | `records` | `text` / `wikidata` |
|---|---:|---:|---|---:|---:|---:|---:|---:|---|
| `shipped` | 602.15 `[597.50..617.95]` | +0.00% | — | 108.23 | 40.85 | 7.97 | 211.78 | 232.44 | 79.04 / 188.81 |
| `A_brl_rle_tier32` | 636.45 `[633.63..661.89]` | +5.70% | SLOWER | +12.8% | +10.2% | +10.3% | +6.6% | +0.9% | +11.7% / +13.2% |
| `B_reg8_tier32` | 636.27 `[632.92..660.30]` | +5.67% | SLOWER | +0.3% | −1.0% | −2.5% | +4.6% | +11.2% | −1.8% / −3.3% |
| `C_reg8_branchless` | 748.39 `[743.05..764.95]` | +24.29% | SLOWER | +36.3% | +63.6% | +59.7% | +11.4% | +21.9% | +55.7% / +70.2% |
| `D_brl_rle_reg8_t64` | 707.74 `[701.17..746.54]` | +17.54% | SLOWER | +18.4% | +25.0% | +24.9% | +18.2% | +15.6% | +26.0% / +38.4% |
| `E_brl_rle_reg8_bl` | 883.60 `[877.00..903.55]` | +46.74% | SLOWER | +79.4% | +125.5% | +123.1% | +24.1% | +35.6% | +123.5% / +144.4% |
| `F_adaptive_reg8` | 771.33 `[766.88..774.81]` | +28.10% | SLOWER | +78.1% | +23.7% | +25.0% | +10.7% | +21.2% | +25.8% / +37.4% |

**Native (`-C target-cpu=native`, 3,955 / 3,955 exact):**

| arm | 5-corp CPU med `[min..max]` | Δ% | disj | `elf` | `prose` | `pysrc` | `random` | `records` | `text` / `wikidata` |
|---|---:|---:|---|---:|---:|---:|---:|---:|---|
| `shipped` | 607.09 `[601.83..607.52]` | +0.00% | — | 109.08 | 41.50 | 8.13 | 215.19 | 232.07 | 80.65 / 191.51 |
| `A_brl_rle_tier32` | 614.01 `[612.82..630.59]` | +1.14% | SLOWER | +9.8% | +7.0% | +7.4% | −0.1% | −2.7% | +8.5% / +10.5% |
| `B_reg8_tier32` | 653.24 `[649.38..676.40]` | +7.60% | SLOWER | +1.5% | −1.3% | −1.2% | +4.5% | +15.8% | −0.5% / −4.5% |
| `C_reg8_branchless` | 750.60 `[744.08..794.70]` | +23.64% | SLOWER | +36.8% | +62.5% | +57.1% | +9.0% | +22.0% | +50.8% / +68.3% |
| `D_brl_rle_reg8_t64` | 729.80 `[722.88..749.54]` | +20.21% | SLOWER | +14.8% | +16.5% | +17.2% | +21.0% | +22.6% | +18.8% / +29.5% |
| `E_brl_rle_reg8_bl` | 879.98 `[876.08..900.29]` | +44.95% | SLOWER | +76.2% | +118.2% | +121.6% | +22.7% | +36.4% | +115.5% / +141.6% |
| `F_adaptive_reg8` | 768.69 `[762.66..796.72]` | +26.62% | SLOWER | +76.0% | +16.4% | +17.3% | +9.3% | +22.4% | +18.5% / +28.4% |

### 4.2 Rounds 2–3 — Persistence follow-ups: moving bases, 3-tier tail, variable-`k` loops, and flat signed/unaligned tails (default flags)

Following Round 1's diagnosis (12-cycle YMM mux latency + Port-5 saturation + unconditional RLE2 store overhead), Rounds 2 and 3 tested 11 additional mechanisms (all 100% exact: 3,390/3,390 in Round 2, 3,955/3,955 in Round 3):

- **Moving-base sub-lists (`G_regs_bases16`, `H_regs_bases32`, `J_bases16_full`, `K_b16_uncond_fl`):** regressed **+28.1% to +57.1%** baseline and **+51.0% to +84.6%** native (`records` +59%..+130%, `random` +12%..+67%) due to unpredictable variable-trip chunk shift loops.
- **Variable-`k` loops (`L_unaligned_7`, `M_aligned_xmm14`, `N_aligned_ymm7_k`, `O_unaligned_full`):** regressed **+15.2% to +22.1%** baseline and **+18.8% to +45.2%** native (`records` +31.6%..+84.5%) for the same reason.
- **3-tier tail (`I_tier32_64_128`):** beat `shipped` **−5.63% base / −5.17% native** (`random` **−19.2% / −15.4%**), but added a `t < 96` misprediction on `records` (+5.4% / +5.9%).
- **Flat signed/unaligned tails (`Q_signed_flat7`, `P_flat_unal_8`):** showed large unmitigated swings (`Q_signed_flat7` **−16.78%** baseline vs **−7.38%** native), while `wikidata` (which has **0.0%** `nn >= 32`!) swung **−18.3% to +6.2%** across arms (`M_aligned_xmm14` −18.3% base, `L_unaligned_7` −12.2% native, `Q_signed_flat7` −9.3% base / +6.2% native) — exposing a severe **Intel JCC erratum** alignment confound on this VM (as documented in `examples/mtf_two_level.rs`).

### 4.3 Round 4 — JCC-mitigated showdown (`-C llvm-args=-x86-branches-within-32B-boundaries`, 4,520 / 4,520 exact)

Recompiling with `-C llvm-args=-x86-branches-within-32B-boundaries` eliminated the JCC erratum (`shipped` itself dropped `187.54 -> 150.61 ms` on `wikidata`, `40.45 -> 33.08 ms` on `prose`, and `107.93 -> 93.89 ms` on `elf`) and isolated the true algorithmic deltas across all top mechanisms (`I`, `Q`, `P2` with partial-store elimination, `Q2` aligned-8, `R` tier32 + flat unaligned 7-row, and adaptive `S`/`T`):

**Baseline x86-64 (JCC-mitigated, 7 interleaved repeats):**

| arm | 5-corp CPU med `[min..max]` | Δ% | disj | `elf` (19) | `prose` (9) | `pysrc` (2) | `random` (18) | `records` (22) | `text` / `wikidata` |
|---|---:|---:|---|---:|---:|---:|---:|---:|---|
| `shipped` | 543.53 `[541.05..554.64]` | +0.00% | — | 93.89 | 33.08 | 6.63 | 192.46 | 218.21 | 65.28 / 150.61 |
| `I_tier32_64_128` | 525.33 `[519.71..543.68]` | −3.35% | ovlp | +2.9% | −0.9% | −3.6% | **−16.5%\*** | +4.3% | −1.1%\* / +1.3% |
| `Q_signed_flat7` | 503.83 `[501.87..565.65]` | −7.30% | ovlp | **−3.5%\*** | +2.2% | −1.0% | −10.5% | −8.4% | +0.8% / +0.8% |
| `P2_flat_unal8_np` | 545.31 `[543.80..554.72]` | +0.33% | ovlp | +16.4%! | +51.8%! | +46.0%! | −11.4%\* | −5.9%\* | +43.0%! / +60.9%! |
| `Q2_flat_algn8` | 550.29 `[549.11..583.59]` | +1.24% | ovlp | +18.3%! | +53.6%! | +47.5%! | −10.5%\* | −5.1% | +45.6%! / +63.7%! |
| **`R_t32_flat_unal7`** | **497.63 `[497.07..537.86]`** | **−8.45%** | **FASTER** | **−2.8%\*** | +2.6% | +0.4% | **−10.6%\*** | **−10.4%** | +2.1% / +1.4% |
| `S_adapt_Q_P2` | 509.16 `[506.98..540.48]` | −6.32% | FASTER | −2.6% | +1.9% | −1.3% | −10.5% | −5.7%\* | +1.4% / +1.2% |
| `T_adapt_Q_Q2` | 513.26 `[508.30..528.32]` | −5.57% | FASTER | −2.7% | +2.8% | −0.2% | −10.1%\* | −4.8%\* | +1.4% / +1.3% |

**Native (`-C target-cpu=native`, JCC-mitigated, 7 interleaved repeats):**

| arm | 5-corp CPU med `[min..max]` | Δ% | disj | `elf` (19) | `prose` (9) | `pysrc` (2) | `random` (18) | `records` (22) | `text` / `wikidata` |
|---|---:|---:|---|---:|---:|---:|---:|---:|---|
| `shipped` | 551.25 `[545.22..564.63]` | +0.00% | — | 94.01 | 33.14 | 6.58 | 194.84 | 221.02 | 64.99 / 153.85 |
| `I_tier32_64_128` | 524.58 `[518.29..539.23]` | −4.84% | FASTER | −1.5% | −2.2% | −2.9% | **−16.0%\*** | +3.1% | −0.2% / −2.0%\* |
| `Q_signed_flat7` | 505.36 `[500.15..528.69]` | −8.33% | FASTER | **−3.9%\*** | +1.0% | −0.8% | −10.9%\* | −9.0%\* | +1.5% / −1.2% |
| `P2_flat_unal8_np` | 536.14 `[526.70..554.66]` | −2.74% | ovlp | +16.6%! | +51.7%! | +46.7%! | −15.7%\* | −10.4% | +43.4%! / +58.3%! |
| `Q2_flat_algn8` | 535.59 `[532.41..539.62]` | −2.84% | FASTER | +15.9%! | +51.1%! | +46.8%! | −14.7%\* | −9.8%\* | +43.5%! / +57.2%! |
| **`R_t32_flat_unal7`** | **499.68 `[494.28..521.96]`** | **−9.36%** | **FASTER** | **−3.7%\*** | +1.9% | −0.6% | **−12.5%\*** | **−10.7%** | +1.0% / +0.1% |
| **`S_adapt_Q_P2`** | **494.16 `[488.75..497.85]`** | **−10.36%** | **FASTER** | **−2.9%\*** | +1.8%! | −0.9% | **−15.4%\*** | **−10.6%\*** | +1.1% / −0.1% |
| `T_adapt_Q_Q2` | 500.08 `[495.16..504.93]` | −9.28% | FASTER | −2.9%\* | +1.0% | −0.9% | −14.7%\* | −9.2%\* | +1.1% / −1.1% |

*(\* = disjoint faster vs `shipped`; ! = disjoint slower vs `shipped`.)*

---

## 5. Structural diagnosis: why `src/mtf.rs` hits a tight ceiling

Across **20 exact arms spanning 4 rounds**, four concrete microarchitectural constraints explain why `mtf_avx2` cannot be cut by `>= 15%` on real off-Wikidata `Column` blocks:

1. **Irreducible RLE2 branch cost on `prose` and `elf`:** Skeleton ablations proved **49.5%** of `prose` (`20.0 ms`) and **37.4%** of `elf` (`40.1 ms`) is unpredictable `s <= 1` / `run != 0` branching. Branchless RLE2 (`A`, `D`, `E`, `K`) regressed **+5.7% to +57.1%** because unconditional 32-byte run stores, `STEPS[0]` shuffles, and back-to-back `counts` updates on `RUNA/RUNB` symbols cost more than the mispredictions they remove.
2. **In-register byte-extraction latency and Port-5 saturation:** Moving all 256 bytes into 8 YMM registers (`B`, `C`, `D`, `E`, `F`) replaces `shipped`'s 5-cycle scalar load `*tail.add(t)` with a ~12-cycle 3-level `vpblendvb` mux tree directly on `f0`'s critical path and drives up to 34 Port-5 uops/literal, losing **+5.7% to +46.7%**.
3. **Variable-trip chunk loops mispredict on high-entropy tails:** Moving-base sub-lists (`G`, `H`, `J`, `K`) and variable-`k` loops (`L`, `M`, `N`, `O`) lose **+15.2% to +84.6%** because `nn >> 4` / `nn >> 5` is unpredictable on `elf`, `random`, and `records`.
4. **Genuine tail speedup is real (`−8.45%` base / `−9.36%..−10.36%` native) but sub-threshold and concentrated on synthetic high-entropy data:** Eliminating the `t < 32` branch + `vpminub` mask chain (`R_t32_flat_unal7`, `Q_signed_flat7`, `S_adapt_Q_P2`) cleanly cuts `random` by **−10.6% to −15.4%** and `records` by **−10.4% to −10.7%**, yet cuts real-world `elf` by only **−2.8% to −3.9%** and leaves `prose` flat or slightly slower (`−0.9%` to `+2.6%`). Removing `t < 32` helps `random`/`records` (`flp64 = 36–50%`) at a slight `+1–2%` cost on `prose` (`5.6%` in `32..63`, `1.0%` in `64+`).

---

## 6. Verdict (strict C2): **STOP (Scoped — Sub-Threshold Gain)**

- **Decision under frozen rule:** **STOP.** No arm reaches the frozen `>= 15.0%` 5-corpus `Column` gate on JCC-clean builds (best: `R_t32_flat_unal7` at **−8.45%** baseline / **−9.36%** native; `S_adapt_Q_P2` at **−6.32%** baseline / **−10.36%** native), and real-world `elf` (`−2.8%..−3.9%`) and `prose` (`−0.9%..+2.6%`) move by less than 4%.
- **Action:** Do **not** admit `mtf-column-front-build`. Record this scoped negative result in [`frontier/dead-ends.tsv`](../../../frontier/dead-ends.tsv) and [`frontier/deltas/2026-11-27-mtf-column-front-dissection.md`](../../../frontier/deltas/2026-11-27-mtf-column-front-dissection.md). Future work on `src/mtf.rs` would require either a different ISA primitive (e.g., AVX-512 `vpermb`/`vpermi2b` 64-byte full-register permutes, where 256 bytes fit in 4 ZMM registers with 1-instruction extraction) or fusing MTF directly into Huffman decoding (`A3`) rather than standalone `src/mtf.rs` list layouts on AVX2.

### Reproducibility commands

```bash
# Exactness across all 20 arms (555 corpus blocks + 10 synthetic edge/error cases)
cargo run --release --example mtf_column_front_probe -- exact <corpora/*.bz2>

# JCC-mitigated 7-repeat Round 4 showdown (baseline x86-64 and native)
RUSTFLAGS="-C llvm-args=-x86-branches-within-32B-boundaries" \
  cargo run --release --example mtf_column_front_probe -- bench4 <corpora/*.bz2>
RUSTFLAGS="-C target-cpu=native -C llvm-args=-x86-branches-within-32B-boundaries" \
  cargo run --release --target-dir /tmp/mtf-nat-jcc --example mtf_column_front_probe -- bench4 <corpora/*.bz2>
```
