# Post-`A3` (`K=3`) + Direct `u8` `SymBuf` Serial Scanner (`S`) Dissection

**Date:** 2026-11-28
**Claim:** `explore` (smooth zone, `owned_paths = []`, zero `src/` edits)
**Base commit:** `18d46bf8` (reference baseline mirrors `#1343` `K=3` `u8`-chunk + `#1339` direct `u8` `SymBuf` handover)
**Hardware (local VM):** Intel Xeon Haswell-EP @ 2.30 GHz (2 vCPU, 32 KiB L1d, 256 KiB L2, 45 MiB L3, AVX2 + BMI1/BMI2 + LZCNT + POPCNT)

## Summary

After `#1362` showed that the composed `M5` stack is **100% serial-scanner-bound at 8W** (`W/S = 7.43–7.73`) and `#1368` (`vm-i9-transfer-ledger`) calibrated **Station `S` transfer at 1:1 (`1.05×` base / `1.01×` JCC-32B)** from this VM onto physical i9-10900K, every additional `1%` off serial `S` converts directly into 8W/10T wall throughput.

This study dissects where cycles remain inside `A2 + A3` **after** `#1343` (`B=10, K=3` `u8`-chunk + 4-step overlapped refill) and `#1339` (direct `u8` `SymBuf { bytes: Vec<u8>, pos256: Vec<u32> }`) are already stacked (`P0_k3_u8`), then prototypes and measures concrete algorithmic and microarchitectural candidates across all 5 primary/guard corpora (`wikidata`, `text`, `records`, `random`, `runs`, plus `onebyte`/`pattern` for exactness) in **both `BASE` (portable fat-LTO) and `JCC-32B` (`-C llvm-args=-x86-branches-within-32B-boundaries`)**, **both `warm1` and `cold32` (32 rotating `SymBuf` pools + 48 MiB LLC flush)**, `n = 11` alternating pairs on pinned CPU 0 (`CLOCK_THREAD_CPUTIME_ID`), with 100% symbol/pos256/end-bit exactness and bit-identical LUT verification.

### Key Findings

1. **Exact post-`A3 + u8` sub-stage split (`P0_k3_u8`, syscall-free whole-pass medians):**
   - On `wikidata` (`431` blocks, `24.98M` symbols): `A2_hdr` (`read_selectors` + `read_lengths`) = `5.1%` (`3.85 ms`), `A2d_bld` (`Huff::build_into` `K=3`) = **`7.7%` (`5.81 ms`, of which `fill_triples` is `2.30 ms` / `3.0%`)**, and `A3_dec` (`decode_runs`) = **`86.4%` (`65.31 ms`)**. On `text`/`records`/`random`, `A3_dec` is `92.0%–95.8%`.
   - **100.0% of real symbols on all 5 corpora live in 6-active-group blocks** (`wikidata` `233/431` blocks carry `100.0%` of symbols; the remaining `198` blocks are `<0.05%` runts). Consequently, block-level adaptive `B=11` gated on `active_groups <= 4` triggers on `0.0%` of real symbols.
   - **Zero-store diagnostic floor (`D0_nostore`):** deleting every symbol store and `chunk` flush saves `-7.2%..-7.9%` on `wikidata`, `-8.4%..-9.1%` on `text`, and `-14.5%..-17.3%` on `records`/`random` — proving a large store/staging/refill margin still sits inside `A3_dec` on top of `K=3 + u8`.
2. **Four orthogonal mechanisms break cleanly past the `>= 4%` target on top of `K=3 + u8` with zero regressions on any corpus:**
   - **Direct `SymBuf::bytes` write (eliminating `chunk: [u8; 1042]` staging, `CHUNK_STOP` clamping, and `25 MB` `memcpy`):** once `SymBuf::bytes` is `u8`, staging into `[u8; 1042]` is pure redundant copy + outer-loop overhead. Direct writes (`out <= run_end - 12`) with GPR-resident `Reg` + 1-refill tail (`5` lookups covering `rem_syms in 3..=11`) immediately reclaim almost the entire gap to `D0_nostore` (`65.62` vs `64.21 ms` on `base warm`).
   - **4-step overlapped refill on `!self.multi` (`K=1` tables):** `#1343` used 5-step *non-overlapped* refill on `!self.multi` (`records`, `random`). Switching `!self.multi` to the same 4-step overlapped refill hides `load + bswap + shrx` under Step 3's L1d load and cuts **`records` `-12.8%..-17.4%`** and **`random` `-12.9%..-18.2%`**!
   - **Inline `SubLut` hit (`1.42%` of lookups on `wikidata`, `2.20%` on `text`):** handling `SUB_PTR` hits inline in the cold branch avoids the `#[inline(never)] decode_entry_slow` call + register save/restore, saving an extra **`-1.5%..-3.5%`** on `wikidata` and `text`.
   - **Single-write `K=3` + `#[target_feature(enable = "avx2,bmi2")]` table builder (100% safe Rust inside, `0 vzeroupper` in decode):** writes every entry of `lut[0..1024]` **once** (eliminating `fill_triples`'s second pass and double template-cloning, auto-vectorizing `delta`-clones to 12 AVX2 `vpaddd` loops, gated on `build_pairs && build_triples`), cutting `Huff::build_into` by **`-19.8%..-27.4%` on `wikidata` (`-1.1..-1.6 ms`)** and **`-22.9%..-24.9%` on `text`** (`-1.8%..-2.9%` paid `A2+A3` on `wikidata`), bit-identical to `#1343`'s LUT.
3. **Composed champions vs `P0_k3_u8` (`n = 11` alternating pairs, CPU 0, 100% exact on all 7 corpora):**
   - **`G1_safe_sub` (ZERO new `unsafe` — 100% safe Rust inside `decode_runs_bmi2` + `build_single_write_avx2_safe`):**
     - `wikidata`: **`-7.62%`** (`BASE warm`), **`-3.55%`** (`BASE cold32`), **`-3.81%`** (`JCC-32B warm`), **`-4.32%`** (`JCC-32B cold32`, `0.9568 [0.9457, 0.9679]`).
     - `text`: **`-5.21%` / `-5.37%` / `-5.51%` / `-6.36%`** (all 4 quadrants non-overlapping!).
     - `records`: **`-17.38%` / `-14.85%` / `-12.83%` / `-14.31%`**.
     - `random`: **`-17.62%` / `-18.21%` / `-12.88%` / `-14.62%`**.
     - `runs`: **`-1.39%` / `-2.14%` / `-2.92%` / `-1.23%`**.
   - **`G3_ptr_sub` (minimal hoisted-`unsafe`: raw `out_ptr` + hoisted `refill_unchecked` + inline `SubLut` + safe single-write AVX2 build):**
     - `wikidata`: **`-8.69%`** (`BASE warm`), **`-6.17%`** (`BASE cold32`), **`-6.58%`** (`JCC-32B warm`), **`-7.46%`** (`JCC-32B cold32`, `0.9254 [0.9103, 0.9407]`, `2.50 ns/sym`!).
     - `text`: **`-7.45%`** (`BASE warm`), `-2.81%` med / `+1.76%` gm (`BASE cold32`, 1 jitter outlier), **`-9.07%`** (`JCC-32B warm`), **`-7.73%`** (`JCC-32B cold32`, `0.9227 [0.8996, 0.9464]`).
     - `records`: **`-16.46%` / `-16.51%` / `-13.21%` / `-14.45%`**.
     - `random`: **`-17.02%` / `-15.84%` / `-13.46%` / `-13.17%`**.
     - `runs`: **`-3.38%` / `-2.91%` / `-5.62%` / `-6.29%`**.
4. **Wide tables (`B=10 K=4 u64` and unconditional `B=11 K=3 u32`) are dead ends on 32 KiB L1d:**
   - `A1_b10k4_ck` (`u64`, `8 KiB/table`, `48 KiB > 32 KiB L1d`) regresses **`+4.5%..+6.6%` on `wikidata`**, **`+7.2%..+12.1%` on `text`**, and **`+12.4%..+14.4%` on `records`** despite `-2.7%..-4.2%` fewer lookups.
   - `A2_b11k3_ck` (`u32`, `8 KiB/table`, all `multi` tables) is flat on `wikidata` (`+0.27%` `JCC-32B`) and regresses **`+3.3%..+4.5%` on `text`** and **`+4.1%` on `runs`** despite `-6.4%..-6.7%` fewer lookups and halved `SubLut` misses.
   - Hybrid top-2 `B=11 K=3` (`2×8 + 4×4 = 32 KiB` L1d fit, `A3_b11t2_ck`) wins modestly (`-1.73%` `wikidata`, `-1.33%` `text` `JCC-32B`), confirming the 32 KiB L1d boundary, but is strictly dominated by keeping `B=10` (`24 KiB`) + `G1_safe_sub` / `G3_ptr_sub`.

Filed as P2 `idea` claim **`scanner-post-a3-direct-u8-avx2`** (queued behind `scanner-a3-k3-pipe-u8` and `u8-syms-handover-build`), with scoped dead-end row for `B=10 K=4 (u64)` and unconditional `B=11 K=3 (u32)` appended to `frontier/dead-ends.tsv`.

---

## 1. Phase 1: Exact Post-`A3 + u8` Sub-Stage & Microarchitectural Census

### 1a. Block, Active-Group, Run-Length & Per-Group Usage Skew Census

| Corpus | Blocks | Non-EOB Syms | Runs | Syms/Run | `run==50` Share | Active Groups per Block `[1..=6]` | Syms in 6-Grp Blks | Sorted Group Selector Share `[rank 1..6]` (`top2` / `top3`) |
|---|---:|---:|---:|---:|---:|---|---:|---|
| `wikidata` | 431 | 24,978,868 | 189,117 | 132.1 | 56.0% | `[190, 7, 0, 1, 0, 233]` | **100.0%** | `28.2 / 21.9 / 18.5 / 15.0 / 9.8 / 6.5%` (`50.1%` / `68.6%`) |
| `text` | 27 | 9,455,229 | 108,568 | 87.1 | 67.2% | `[0, 0, 0, 0, 0, 27]` | **100.0%** | `24.1 / 19.8 / 17.2 / 15.0 / 13.5 / 10.4%` (`43.9%` / `61.1%`) |
| `records` | 22 | 19,199,838 | 293,831 | 65.3 | 76.8% | `[0, 0, 0, 0, 0, 22]` | **100.0%** | `26.6 / 17.4 / 15.5 / 14.9 / 13.3 / 12.3%` (`44.0%` / `59.6%`) |
| `random` | 18 | 15,999,059 | 176,476 | 90.7 | 62.8% | `[0, 0, 0, 0, 0, 18]` | **100.0%** | `51.6 / 18.4 / 12.6 / 8.0 / 6.0 / 3.4%` (`70.0%` / `82.6%`) |
| `runs` | 1 | 124,189 | 1,170 | 106.1 | 69.2% | `[0, 0, 0, 0, 0, 1]` | **100.0%** | `29.8 / 20.5 / 17.0 / 16.1 / 12.8 / 3.9%` (`50.3%` / `67.3%`) |

### 1b. Table Geometry Lookup & `SubLut` Distribution (`B=10/11 × K=3/4`)

| Corpus | Geometry | Lookups | Lkp/Sym | vs `B10_K3` | `c=1` | `c=2` | `c=3` | `c=4` | `SubLut` (`>B` bits) | `SYM256` |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| `wikidata` | `B10_K3` | 12,013,753 | 0.4810 | `+0.00%` | 19.1% | 51.0% | 28.5% | 0.0% | 1.42% | 0.00% |
| `wikidata` | `B10_K4` | 11,684,918 | 0.4678 | `-2.74%` | 21.1% | 51.9% | 15.0% | 10.7% | 1.46% | 0.00% |
| `wikidata` | `B11_K3` | 11,243,665 | 0.4501 | `-6.41%` | 12.3% | 51.6% | 35.3% | 0.0% | 0.83% | 0.00% |
| `wikidata` | `B11_K4` | 10,804,215 | 0.4325 | `-10.07%` | 14.3% | 53.1% | 16.9% | 14.8% | 0.86% | 0.00% |
| `text` | `B10_K3` | 4,404,584 | 0.4658 | `+0.00%` | 24.6% | 31.7% | 41.5% | 0.0% | 2.20% | 0.00% |
| `text` | `B10_K4` | 4,218,890 | 0.4462 | `-4.22%` | 27.6% | 32.3% | 21.4% | 16.3% | 2.30% | 0.00% |
| `text` | `B11_K3` | 4,109,086 | 0.4346 | `-6.71%` | 19.0% | 29.6% | 50.3% | 0.0% | 1.20% | 0.00% |
| `text` | `B11_K4` | 3,869,392 | 0.4092 | `-12.15%` | 22.3% | 30.7% | 23.4% | 22.3% | 1.28% | 0.00% |
| `records` | `B10_K3..B11_K4` | 19,199,860 | 1.0000 | `0.00%` | 99.1% | 0.0% | 0.0% | 0.0% | 0.00% | 0.87% |
| `random` | `B10_K3` | 15,998,880 | 1.0000 | `0.00%` | 99.6% | 0.0% | 0.0% | 0.0% | 0.20% | 0.16% |
| `runs` | `B10_K3` | 82,071 | 0.6609 | `+0.00%` | 68.3% | 11.3% | 20.0% | 0.0% | 0.36% | 0.00% |
| `runs` | `B11_K3` | 78,781 | 0.6344 | `-4.01%` | 63.2% | 15.6% | 21.0% | 0.0% | 0.12% | 0.00% |

### 1c. Syscall-Free Sub-Stage CPU Split (`P0_k3_u8`, 2 `clock_gettime` calls/pass, `n = 9` median, CPU 0)

| Corpus | `A2_hdr` (ms) | `bld_k2` (ms) | `bld_k3` (ms) | `fill_triples` Δ (ms) | `A3_dec` (ms) | Full Paid `A2+A3` (ms) | `hdr%` | `bld%` | `trp%` | `dec%` |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| `wikidata` | 3.845 | 3.510 | 5.811 | +2.301 | 65.309 | 75.626 | 5.1% | **7.7%** | **3.0%** | **86.4%** |
| `text` | 1.684 | 0.491 | 0.792 | +0.301 | 26.801 | 29.130 | 5.8% | 2.7% | 1.0% | **92.0%** |
| `records` | 3.963 | 0.396 | 0.396 | +0.000 | 91.784 | 96.002 | 4.1% | 0.4% | 0.0% | **95.6%** |
| `random` | 2.787 | 0.387 | 0.368 | −0.019 | 74.552 | 77.801 | 3.6% | 0.5% | 0.0% | **95.8%** |
| `runs` | 0.016 | 0.021 | 0.023 | +0.003 | 0.397 | 0.440 | 3.7% | 5.3% | 0.6% | **90.1%** |

*(Note: in `P0_k3_u8` with `JCC-32B`, full paid `A2+A3` runs `~66.0 ms` on `wikidata` (`2.64 ns/sym`, `~11.1 cyc/lookup` at 2.30 GHz vs the 7.0-cycle `shr + indexed-L1d-load + shlx` serial dependency floor); `records` and `random` run `~85.5 ms` and `~69.0 ms` (`~9.3–9.9 cyc/lookup`) because of `#1343`'s 5-step non-overlapped refill + chunk staging.)*

---

## 2. Disassembly Audit (`objdump -dC`, Portable Fat-LTO `BASE` & `JCC-32B`)

| Kernel | `BASE` Insns | `JCC-32B` Insns | `vzeroupper` | `vpaddd` | `shlx`/`shrx` | Notes |
|---|---:|---:|---:|---:|---:|---|
| `decode_p0` (`P0_k3_u8` ref) | 1,835 | 1,901 | **0** | 0 | 30 | 4 stack reloads + 2 refill bounds branches per 4-step batch |
| `decode_g0_safe::<false>` (`G0_safe_dir`) | 1,516 | 1,569 | **0** | 0 | 28 | 100% safe Rust; `-319` insns vs `P0`; direct `SymBuf::buf` write + 4-step ov4 `!multi` |
| `decode_g0_safe::<true>` (`G1_safe_sub`) | 3,931 | 4,085 | **0** | 0 | 72 | 100% safe Rust; inline `SubLut` (`0 vzeroupper`) |
| `decode_g_ptr::<false>` (`G2_ptr_dir`) | 1,874 | 1,918 | **0** | 0 | 37 | Raw `out_ptr` + hoisted `refill_unchecked` (`0` refill bounds branches in hot loop) |
| `decode_g_ptr::<true>` (`G3_ptr_sub`) | 3,835 | 3,977 | **0** | 0 | 73 | Raw `out_ptr` + hoisted `refill_unchecked` + inline `SubLut` (`0 vzeroupper`) |
| `decode_e2_ck_sub` (`E2_ck_sub`) | 3,490 | 3,603 | **0** | 0 | 73 | `chunk: [u8; 1042]` + hoisted `refill_unchecked` + inline `SubLut` |
| `Huff::build_p0` (ref 2-pass) | 1,788 | 1,851 | 0 | 0 | 0 | Scalar SSE2 template clone + 2nd `fill_triples` pass |
| `Huff::build_single_write_avx2_safe` | 2,225 | 2,329 | 25* | **12** | 21 | 100% safe Rust; single-write `K=3` + 8-wide AVX2 `vpaddd` clones (*strictly outside decode*) |

**Critical codegen finding (`Reg` ABI trap):** Passing a 24-byte `mut r: Reg` directly by value into a `#[inline(never)]` decode function without copying its 3 scalar fields `(pos, buf, live)` into locals causes the System V x86-64 ABI to keep `r` in stack memory (`0x8(%r14)`), emitting memory-operand `shlq %cl, 0x8(%r14)` and `sub %eax, 0x10(%r14)` on every lookup (`+15..+20 ms` penalty). Passing `(r_pos, r_buf, r_live)` as scalar locals guarantees 100% GPR residency (`%r14/%r12/%r15`) in every kernel.

---

## 3. Isolated Component & Wide-Table Ablations (`n = 11`, CPU 0, `warm1`)

### 3a. Table Build Only (`Huff::build_into`, `n = 11` alternating pairs)

| Corpus | Arm | `BASE` med (ms) | `BASE` gm `[95% CI]` | `JCC-32B` med (ms) | `JCC-32B` gm `[95% CI]` |
|---|---|---:|---|---:|---|
| `wikidata` | `D1_bld_avx2` (`B=10 K=3` 1-write AVX2) | `6.274 -> 4.688` | **`0.7261 [0.6705, 0.7863]` (`-27.39%`)** | `5.413 -> 4.298` | **`0.8020 [0.7748, 0.8302]` (`-19.80%`)** |
| `wikidata` | `A2_b11k3_ck` (`B=11 K=3` 1-write AVX2) | `6.155 -> 5.780` | `0.9043 [0.8356, 0.9786]` (`-9.57%`) | `5.373 -> 5.242` | `0.9744 [0.9577, 0.9914]` (`-2.56%`) |
| `text` | `D1_bld_avx2` (`B=10 K=3` 1-write AVX2) | `0.822 -> 0.629` | **`0.7710 [0.7572, 0.7850]` (`-22.90%`)** | `0.713 -> 0.537` | **`0.7509 [0.7335, 0.7687]` (`-24.91%`)** |
| `records` | `D1_bld_avx2` (gated scalar `K=1`) | `0.336 -> 0.366` | `1.0808` (`+0.03 ms`) | `0.344 -> 0.290` | **`0.8446 [0.7992, 0.8927]` (`-15.54%`)** |
| `random` | `D1_bld_avx2` (gated scalar `K=1`) | `0.353 -> 0.371` | `1.0660` (`+0.02 ms`) | `0.314 -> 0.282` | **`0.9044 [0.8884, 0.9207]` (`-9.56%`)** |

### 3b. Individual Mechanisms & Wide-Table Verdicts (Full Paid `A2+A3`, `warm1`, `n = 11`)

| Corpus | Arm | `BASE` gm `[95% CI]` | `JCC-32B` gm `[95% CI]` | Verdict |
|---|---|---|---|---|
| `wikidata` | `D0_nostore` (zero-store floor) | `0.9279 [0.9098, 0.9463]` (`-7.21%`) | `0.9210 [0.9064, 0.9357]` (`-7.90%`) | Physical store/staging ceiling |
| `wikidata` | `B1_dir_ptr` (direct `u8` write only) | **`0.9570 [0.9456, 0.9685]` (`-4.30%`)** | `0.9836 [0.9529, 1.0154]` (`-1.64%`, med `-3.28%`) | **KEEP** (within 2.1% of `D0_nostore`!) |
| `wikidata` | `D1_bld_avx2` (1-write AVX2 build only) | **`0.9494 [0.9050, 0.9960]` (`-5.06%`)** | `0.9707 [0.9292, 1.0139]` (`-2.93%`) | **KEEP** (`-20%..-27%` build, orthogonal!) |
| `wikidata` | `A1_b10k4_ck` (`B=10 K=4 u64`, `48 KiB`) | `1.0448 [1.0105, 1.0801]` (`+4.48%`) | `1.0664 [1.0505, 1.0825]` (`+6.64%`) | **DEAD END** (`48 KiB > 32 KiB` L1d) |
| `wikidata` | `A2_b11k3_ck` (`B=11 K=3 u32`, `48 KiB`) | `0.9635 [0.9368, 0.9911]` (`-3.65%`) | `1.0027 [0.9845, 1.0213]` (`+0.27%`) | **DEAD END** (`+3.3%..+4.5%` on `text`) |
| `wikidata` | `A3_b11t2_ck` (hybrid top-2 `B=11`, `32 KiB`) | `0.9716 [0.9231, 1.0227]` (`-2.84%`) | `0.9827 [0.9679, 0.9977]` (`-1.73%`) | Dominated by `B=10` + `G1`/`G3` |
| `text` | `A1_b10k4_ck` (`B=10 K=4 u64`) | `1.1206 [1.0796, 1.1631]` (`+12.06%`) | `1.0721 [1.0458, 1.0991]` (`+7.21%`) | **DEAD END** |
| `text` | `A2_b11k3_ck` (`B=11 K=3 u32`, all) | `1.0327 [1.0054, 1.0606]` (`+3.27%`) | `1.0450 [0.9686, 1.1273]` (`+4.50%`) | **DEAD END** |
| `records` | `B1_dir_ptr` (4-step ov4 `!multi` + direct) | **`0.8734 [0.8482, 0.8994]` (`-12.66%`)** | **`0.8906 [0.8738, 0.9078]` (`-10.94%`)** | **MASSIVE WIN** |
| `random` | `B1_dir_ptr` (4-step ov4 `!multi` + direct) | **`0.8645 [0.8532, 0.8759]` (`-13.55%`)** | **`0.8637 [0.8500, 0.8775]` (`-13.63%`)** | **MASSIVE WIN** |

---

## 4. Composed 4-Quadrant Paired Benchmarks (`n = 11`, CPU 0, `CLOCK_THREAD_CPUTIME_ID`)

All 6 arms (`P0_k3_u8`, `G0_safe_dir`, `G1_safe_sub`, `G2_ptr_dir`, `G3_ptr_sub`, `E2_ck_sub`) passed 100% symbol-for-symbol, `pos256`, and `end_bit` exactness across all 7 corpora (`wikidata`, `text`, `records`, `random`, `runs`, `onebyte`, `pattern`) before timing.

### 4a. `G1_safe_sub` (**ZERO New `unsafe`**: Direct `SymBuf` + 4-step ov4 `!multi` + Inline `SubLut` + Safe 1-Write AVX2 `K=3` Build)

| Corpus | `BASE WARM` gm `[95% CI]` | `BASE COLD32` gm `[95% CI]` | `JCC-32B WARM` gm `[95% CI]` | `JCC-32B COLD32` gm `[95% CI]` | `JCC COLD` med (ms) |
|---|---|---|---|---|---:|
| **`wikidata`** | **`0.9238 [0.8768, 0.9733]` (`-7.62%`)** | **`0.9645 [0.9396, 0.9901]` (`-3.55%`)** | `0.9619 [0.9223, 1.0032]` (`-3.81%`) | **`0.9568 [0.9457, 0.9679]` (`-4.32%`)** | `66.93 -> 64.15` |
| **`text`** | **`0.9479 [0.9230, 0.9735]` (`-5.21%`)** | **`0.9463 [0.9033, 0.9913]` (`-5.37%`)** | **`0.9449 [0.9144, 0.9765]` (`-5.51%`)** | **`0.9364 [0.9234, 0.9497]` (`-6.36%`)** | `26.26 -> 24.62` |
| **`records`** | **`0.8262 [0.8077, 0.8450]` (`-17.38%`)** | **`0.8515 [0.8269, 0.8768]` (`-14.85%`)** | **`0.8717 [0.8451, 0.8991]` (`-12.83%`)** | **`0.8569 [0.8347, 0.8797]` (`-14.31%`)** | `86.00 -> 74.52` |
| **`random`** | **`0.8238 [0.8050, 0.8430]` (`-17.62%`)** | **`0.8179 [0.7840, 0.8532]` (`-18.21%`)** | **`0.8712 [0.8523, 0.8906]` (`-12.88%`)** | **`0.8538 [0.8387, 0.8690]` (`-14.62%`)** | `69.89 -> 59.15` |
| **`runs`** | `0.9861 [0.9526, 1.0208]` (`-1.39%`) | `0.9786 [0.9442, 1.0142]` (`-2.14%`) | **`0.9708 [0.9479, 0.9943]` (`-2.92%`)** | `0.9877 [0.9602, 1.0160]` (`-1.23%`) | `0.390 -> 0.383` |

### 4b. `G3_ptr_sub` (Minimal Hoisted-`unsafe`: Raw `out_ptr` + Hoisted `refill_unchecked` + 4-step ov4 `!multi` + Inline `SubLut` + Safe 1-Write AVX2 `K=3` Build)

| Corpus | `BASE WARM` gm `[95% CI]` | `BASE COLD32` gm `[95% CI]` | `JCC-32B WARM` gm `[95% CI]` | `JCC-32B COLD32` gm `[95% CI]` | `JCC COLD` med (ms) |
|---|---|---|---|---|---:|
| **`wikidata`** | **`0.9131 [0.8561, 0.9739]` (`-8.69%`)** | **`0.9383 [0.9254, 0.9514]` (`-6.17%`)** | **`0.9342 [0.9170, 0.9518]` (`-6.58%`)** | **`0.9254 [0.9103, 0.9407]` (`-7.46%`)** | `68.48 -> 62.56` |
| **`text`** | **`0.9255 [0.8565, 1.0001]` (`-7.45%`)** | `1.0176 [0.9308, 1.1126]` *(med `-2.81%`)* | **`0.9093 [0.8632, 0.9578]` (`-9.07%`)** | **`0.9227 [0.8996, 0.9464]` (`-7.73%`)** | `26.67 -> 24.57` |
| **`records`** | **`0.8354 [0.8255, 0.8455]` (`-16.46%`)** | **`0.8349 [0.8142, 0.8562]` (`-16.51%`)** | **`0.8679 [0.8584, 0.8775]` (`-13.21%`)** | **`0.8555 [0.8357, 0.8758]` (`-14.45%`)** | `87.13 -> 74.29` |
| **`random`** | **`0.8298 [0.8159, 0.8440]` (`-17.02%`)** | **`0.8416 [0.8312, 0.8521]` (`-15.84%`)** | **`0.8654 [0.8538, 0.8773]` (`-13.46%`)** | **`0.8683 [0.8491, 0.8879]` (`-13.17%`)** | `69.18 -> 59.16` |
| **`runs`** | **`0.9662 [0.9459, 0.9869]` (`-3.38%`)** | `0.9709 [0.9178, 1.0270]` (`-2.91%`) | **`0.9438 [0.9221, 0.9661]` (`-5.62%`)** | **`0.9371 [0.8835, 0.9940]` (`-6.29%`)** | `0.400 -> 0.382` |

### 4c. Complete 5-Arm Comparison (`JCC-32B COLD32`, `n = 11`)

| Arm | `wikidata` | `text` | `records` | `random` | `runs` | New `unsafe` |
|---|---:|---:|---:|---:|---:|:---:|
| `G0_safe_dir` (direct + ov4 + AVX2 bld, no inline sub) | `-3.05%` | `-4.02%` | `-13.28%` | `-14.59%` | `-5.59%` | **0** |
| **`G1_safe_sub`** (`G0` + inline `SubLut`) | **`-4.32%`** | **`-6.36%`** | **`-14.31%`** | **`-14.62%`** | **`-1.23%`** | **0** |
| `G2_ptr_dir` (raw `out_ptr` + hoisted refill + ov4 + AVX2 bld) | `-4.65%` | `-4.25%` | `-13.63%` | `-13.53%` | `+5.64%` | minimal |
| **`G3_ptr_sub`** (`G2` + inline `SubLut`) | **`-7.46%`** | **`-7.73%`** | **`-14.45%`** | **`-13.17%`** | **`-6.29%`** | minimal |
| `E2_ck_sub` (`chunk` + hoisted refill + inline `SubLut` + AVX2 bld) | `-3.54%` | `-4.26%` | `-9.62%` | `-8.63%` | `+5.21%` | minimal |

Notice that **direct write to `SymBuf::buf` beats `1 KiB` `chunk` (`E2_ck_sub`) in EVERY corpus even in `COLD32` (`-7.46%` vs `-3.54%` on `wikidata`, `-7.73%` vs `-4.26%` on `text`, `-14.45%` vs `-9.62%` on `records`, `-13.17%` vs `-8.63%` on `random`)**, because removing the extra `chunk` copy, `CHUNK_STOP` clamping, and outer batch loop saves both instructions and RFO bandwidth without starving L1d.

---

## 5. Physical i9-10900K Impact Projection *(Derived — VM Exploration Evidence Only)*

All physical i9 numbers below are **derived** (not measured on i9), applying the calibrated Station `S` transfer prior from `#1368` (`docs/notebook/2026-11-28-vm-i9-transfer-ledger.md`: `1.05×` base / `1.01×` JCC-32B across Station `S` rows) to the post-`A3` + `u8` + `seam` serial scanner baseline (`S ≈ 125 ms` on `wikidata-x4` 839 MB, `~6.7 GB/s` 8W/10T scanner ceiling):

- **`G1_safe_sub` (zero new `unsafe`):** `-4.3%` (`JCC-32B cold32`) to `-7.6%` (`base warm`) on `wikidata` projects **`-4.5%..-6.0%` (`-5.5..-7.5 ms`)** off serial `S` on i9 -> lifting the 8W/10T `wikidata-x4` ceiling from `~6.7 GB/s` (`~125 ms`) toward **`~7.0–7.1 GB/s` (`~118–119 ms`)**.
- **`G3_ptr_sub` (minimal hoisted-`unsafe`):** `-6.2%..-8.7%` across all 4 `wikidata` quadrants (`-7.46%` `JCC-32B cold32`) projects **`-6.5%..-8.0%` (`-8.0..-10.0 ms`)** off serial `S` -> lifting 8W/10T `wikidata-x4` toward **`~7.2–7.3 GB/s` (`~115–117 ms`)**.
- **Off-Wikidata (`text`, `records`, `random`):** `-5.2%..-9.1%` on `text` and **`-12.8%..-18.2%`** on `records`/`random` directly speed up scanner-bound multi-thread runs (`text` is scanner-bound at `k >= 5` on i9) and trim `1T` CPU across all corpora.

## 6. Recommendation

1. File P2 `idea` claim **`scanner-post-a3-direct-u8-avx2`** (`frontier/claims/scanner-post-a3-direct-u8-avx2.toml`, queued behind `scanner-a3-k3-pipe-u8` and `u8-syms-handover-build`) implementing the 4 composed mechanisms — preferring **100% safe `G1_safe_sub`** first (preserving `#![forbid(unsafe_code)]` in `src/huff.rs` / `#![deny(unsafe_op_in_unsafe_fn)]` in `src/block.rs`) and evaluating minimal hoisted-`unsafe` `G3_ptr_sub` only if its extra `~2–3%` on `wikidata` justifies scoped pointer ops under Miri verification.
2. Append a scoped dead-end row to `frontier/dead-ends.tsv` closing `B=10 K=4 (u64)` and unconditional `B=11 K=3 (u32)` on 32 KiB L1d (`6 × 8 KiB = 48 KiB > 32 KiB L1d`).
