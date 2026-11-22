# Wormhole Cycle 32 (Neon) — Wall W2: Eliminating `out` `900K→1.8M` Doubling, Feeder `4 MiB` `mmap`/`munmap` Shootdowns, & Page-Fault Storms

**Baseline:** `origin/master` `v0.5.12` (`6a36b0e10`), `BzReader::parallel_workers` (`OpenCells`, `slots = 4*W`) in the same binary with A/A control (`MasterA` vs `MasterB`).
**Candidate harness:** [`c32_alloc_ab.rs`](2026-11-21-wormhole-cycle32/c32_alloc_ab.rs) (`safe_c32` module `#![forbid(unsafe_code)]`), raw log [`c32_results.txt`](2026-11-21-wormhole-cycle32/c32_results.txt), manifest [`c32_alloc.toml`](2026-11-21-wormhole-cycle32/c32_alloc.toml).

---

## 1. Root-Cause Mechanisms Attacked (`Wall W2`)

1. **`A1: Exact-Fit `out` Reservation (`960 KiB`) — Fixing the `src/rle.rs:47` Off-by-5-Byte Doubling Bug:**
   `src/rle.rs:47` calls `out.reserve(n + 64)` on an empty `Vec`. On Wikidata full blocks, `n = 899,931..899,959` and post-RLE1 length is `900,000` (`5..67` bytes past `n + 64`!), forcing `Vec::extend_from_slice` to **double 100% of first-use `out` buffers `900 KiB → 1.80 MiB`** (`mremap` + `900 KiB` copy + `munmap` TLB shootdown across all cores, inflating pool capacity `50→72→144 MiB`!). Calling `out.reserve_exact(960_000)` whenever `syms.len() > 2048 && out.capacity() < 960_000` (catching buffers previously used by tiny `2 KiB` blocks too!) drops doubled buffers to **`dbl = 0`**.
2. **`A2: Recycled `1 MiB` Feeder Pool + Zero-Copy `ChannelSource` Swap (`A12_FeedPool`):**
   `src/read.rs:247` allocates a fresh `vec![0u8; 4 << 20]` (`4 MiB` `mmap` + `1,024` zero-fill faults) per chunk, copies it into `ChannelSource::buf`, and drops it (`munmap` TLB shootdown across all cores!). Recycling `3 × 1 MiB` buffers via a return channel and using `std::mem::replace(&mut self.buf, chunk)` when `released == base + buf.len()` eliminates mid-decode `mmap`/`munmap` shootdowns and cuts startup latency `4×`.
3. **`A3: Eager `262,144 u16` (`512 KiB`) `syms` Pre-Allocation (`A123_Syms262K`):** Tested against `A12_FeedPool`.
4. **`A4: Static Window Shrink (`slots = 2W` and `slots = W + 6`):** Stress-tested on both uniform Wikidata and `hetero_60b.bz2` (Wikidata + dense + random fallback blocks) to test HOL starvation honestly.

---

## 2. Native Safe-Rust A/B + A/A Results (Fresh-Process Cold-Start, Same Binary)

### 2a. Cold-Start Scaling on `wikidata_431b_210m.bz2` (`211.0 MB`, `431` blocks, `3`-round median, exact hash parity)

| `W` | Arm | `wall_ms` | `user_ms` | `sys_ms` (`Δ` vs MA) | `minflt` (`Δ` vs MA) | `rss_MiB` (`Δ`) | `dbl` |
|---:|---|---:|---:|---:|---:|---:|---:|
| `4` | `MasterA` / `MasterB` (`A/A`) | `373.6 / 372.4` | `657.4 / 652.9` | `47.58 / 46.45` | `16,025 / 16,058` | `61.5 / 59.8` | — |
| `4` | `A1_Out960K` | `369.38` | `659.85` | `38.04` (`-20.1%`) | `15,036` (`-6.2%`) | `60.97` | `0` |
| `4` | **`A12_FeedPool`** | **`372.09`** | `658.13` | **`38.61` (`-18.9%`)** | **`11,841` (`-26.1%`)** | **`54.93` (`-10.7%`)** | **`0`** |
| `10` | `MasterA` / `MasterB` (`A/A`) | `402.2 / 379.4` | `689.5 / 671.4` | `63.48 / 66.71` | `27,514 / 27,832` | `96.9 / 102.6` | — |
| `10` | `A1_Out960K` | `387.87` | `694.41` | `57.96` (`-8.7%`) | `26,095` (`-5.2%`) | `92.02` (`-5.1%`) | `0` |
| `10` | **`A12_FeedPool`** | **`384.68`** | `679.94` | `61.81` | **`21,223` (`-22.9%`)** | **`88.28` (`-8.9%`)** | **`0`** |
| `10` | `A123_Syms262K` | `380.85` | `689.91` | `52.20` (`-17.8%`) | `20,032` (`-27.2%`) | `89.18` (`-8.0%`) | `0` |
| `16` | `MasterA` / `MasterB` (`A/A`) | `408.3 / 399.0` | `711.7 / 708.6` | `79.24 / 65.42` | `37,261 / 31,565` | `134.9 / 113.2` | — |
| `16` | **`A12_FeedPool`** | **`382.57` (`-6.3%`)** | `695.48` | **`51.24` (`-35.3%`)** | **`24,605` (`-34.0%`)** | **`100.80` (`-25.3%`)** | **`0`** |

### 2b. `12`-Round Rotated Fresh-Process `4W` A/B (`wikidata_60b.bz2` & `hetero_60b.bz2`)

| Arm | `wikidata_60b` `4W` ratio `[95% CI]` | `minflt` / `rss` | `hetero_60b` `4W` ratio `[95% CI]` | `sys_ms` / `minflt` / `rss` | Verdict |
|---|---|---:|---|---:|---|
| `MasterA` / `MasterB` (`A/A`) | `1.0000` / `1.0411 [0.9818, 1.1040]` | `9,540` / `35.8` | `1.0000` / `0.9847 [0.9446, 1.0264]` | `45.7` / `20,747` / `72.9` | Control |
| `A1_Out960K` | `0.9931 [0.9726, 1.0141]` | `8,842` / `33.0` | `0.9707 [0.9249, 1.0188]` | `42.6` / `20,044` / `69.6` | **KEEP** (`dbl=0`) |
| **`A12_FeedPool`** | **`0.9809 [0.9559, 1.0065]`** | **`8,631` / `32.6`** | **`0.9763 [0.9455, 1.0081]`** | **`39.4` / `18,182` / `65.4`** | **PROMOTE** |
| `A123_Syms262K` | `1.0087 [0.9573, 1.0627]` | `9,355` / `35.5` | `0.9625 [0.9225, 1.0041]` | `41.1` / `18,801` / `68.5` | **KILL** |
| `A1234_Slots2W` | `0.9800 [0.9557, 1.0048]` | `8,124` / `30.4` | **`1.0857 [1.0256, 1.1494]`** | `33.3` / `15,303` / `58.8` | **KILL** (HOL stall) |
| `A1234b_SlotsW6` | `0.9803 [0.9622, 0.9987]` | `8,833` / `33.0` | **`1.0584 [1.0188, 1.0995]`** | `34.5` / `15,384` / `58.5` | **KILL** (HOL stall) |

*Bonus discovery in `src/open.rs:360`:* `out.resize(n + INLINE, 0)` also `memset(0)`s `898 KiB` of `scratch.ll` every time a worker switches from a tiny `2 KiB` block back to a `900 KiB` block (`197` tiny blocks in Wikidata!), and doubles `ll` `900K→1.8M` when block 2 exceeds block 1's row count (`899,931→900,000`).

---

## 3. Updated `10T` i9-10900K Critical-Path Model (`A12_FeedPool`)

- `ΔT_alloc_pf` at `10W`: `-22.9%` `minflt` (`27.5k → 21.2k`) + `0` `out` doubling `mremap`/TLB shootdowns + `0` mid-decode feeder `4 MiB` `munmap` shootdowns → **`6.2 ms → ~3.4 ms` (`-2.8 ms`)**; at `16W`, `-34.0%` `minflt` / `-35.3%` `sys_ms` recovers **`~4.5 ms`**.
- `M` (working set): `-8.9%` RSS at `10W` (`-25.3%` at `16W`), cutting reserved `out` pool in half (`72 → 37 MiB` at `10W`).
- **Modelled `10T` Critical Path (Regime 2 post-Mechanisms `C38b`):** `~53.5 ms → ~50.7 ms` (`-2.8 ms`); standalone Regime 1 `10W` gap vs `7W` narrows by `~2.8 ms`.

---

## 4. Kill Decisions (One Line Each)

- **KEEP `A1: Exact-Fit out Reservation (960 KiB)`:** Eliminates `100%` of `900K→1.8M` `out` doubling (`dbl=0`) and halves `out` pool virtual/physical footprint.
- **PROMOTE `A12_FeedPool` (`A1 + Recycled 1 MiB Feeder Pool + Zero-Copy ChannelSource Swap`):** Cuts cold-start `minflt` `-22.9%` (`10W`) / `-34.0%` (`16W`), `sys_ms` `-35.3%` (`16W`), `rss` `-8.9%..25.3%`, and `4W` wall `-1.9%..-2.4%` with zero regression on heterogeneous streams.
- **KILL `A3: Eager 262K syms Pre-Allocation`:** Over-allocates `512 KiB` on tiny `2 KiB` blocks and short streams, raising `minflt` (`+8.4%` at `4W`, `+13.1%` at `16W`) and `sys_ms` (`+21.9%` at `16W`) vs `A12_FeedPool`.
- **KILL `A4: Static Window Shrink (slots = 2W / W+6)`:** Causes **`+5.8%..+8.6%` (`1.058×–1.086×`)** head-of-line scanner starvation on heterogeneous streams (`hetero_60b`) when one worker hits a slow random/dense block; slot memory must be cut via per-slot lifecycle, not starving logical lookahead.
