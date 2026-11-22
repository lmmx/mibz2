# Carbon Cycle 34 — Safe-Rust SWAR-Bitmap + GP `u128` 2-Level MTF `runs::front` vs Shipped `front_avx2`

**Baseline:** `origin/master` (`53f7ecf9`) `mibz2::runs::front` (`unsafe` AVX2+BMI2 `front_avx2`) and `mibz2::runs::front_scalar`.
**Artifact:** `docs/journal/2026-11-21-carbon-cycle34/c34_safe_runs_front.rs` (`#![forbid(unsafe_code)]`).

## 1. Experiment

Stage #2 of the front end (`runs::front`, MTF + RLE2 $\to$ `u16` run records) takes `41.4–43.0 ms` (`8.6%` of `1T` decode, `8.8%` of `10T` CPU) on `wikidata.bz2`. Prior journal `#25` rejected scalar GP MTF head registers due to register spills across inline `copy_within` (`memmove`), while `#324` established two-pass `lit`/`one` bitmaps + 2-level XMM `pshufb` MTF (`front_avx2`).

To test the exact boundary of `#![forbid(unsafe_code)]` on Stage #2, we constructed `front_c34`:
1. **Carry-Free 4-Lane SWAR `u64` Bitmap Pass:** Computes `lit` (`s > 1`) and `one` (`s != 0`) 4 `u16`s at a time via carry-free mask-add + `0x0001_0002_0004_0008` bit-gather multiply.
2. **By-Value Pinning of `(lo, hi)` GP `u128` MTF Head + Out-of-Line Seward `SlidingTail`:** Keeps MTF bytes `0..15` in two `u64` registers (`lo`, `hi`) and isolates `nn >= 16` into `#[cold] #[inline(never)] SlidingTail::move_ge16` (`[u8; 4096]` + `base: [usize; 16]`) so `lo` and `hi` never spill across `memmove`.

## 2. Native Safe-Rust A/B + A/A Results (`wikidata.bz2`, `383` Run-Path Blocks, 15 Rotated Rounds)

| Arm | Median (ms) | vs `front_avx2` (`MasterA`) | vs `front_scalar` | Verdict |
|---|---:|---:|---:|---|
| `MasterA` (`front_avx2`) | **42.99** | `1.0000×` | `0.1581×` (`-84.19%`) | **KEEP SHIPPED** |
| `MasterB` (`A/A` ctrl) | 42.85 | `0.9967×` | `0.1576×` (`-84.24%`) | Control |
| `Master` (`front_scalar`) | 271.93 | `6.3251×` | `1.0000×` | Scalar ref |
| `C34 SafeRuns` (`SWAR + GP`) | 198.25 | `4.6113×` (`+361.1%`) | **`0.7290×` (`-27.10%`)** | **KILL vs AVX2** |

**Parity:** `431/431` `wikidata.bz2` blocks run-record-exact and row-count-exact.

## 3. Kill Decision

- **`C34 SafeRuns` (`4.6113×` vs `front_avx2`, despite `0.7290×` vs `front_scalar`):** Killed immediately as a production `runs::front` replacement because `#![forbid(unsafe_code)]` cannot emit branchless 32-byte XMM `_mm_shuffle_epi8` (`pshufb`) shuffles, leaving a 3-way branch (`nn < 8`, `nn < 16`, `nn >= 16`) + variable GP shift chain across `15.15M` literals (`4.6×` slower than `front_avx2`).
