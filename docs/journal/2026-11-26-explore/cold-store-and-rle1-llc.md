# Cold-store oracle artifacts, the real delivery curve, and the unmapped RLE1/CRC LLC cliff

*Cold primary-evidence exploration (`explore`, 26 November 2026). Inspected `src/` and raw `runs.jsonl` bundles before reading earlier interpretations.*

## Primary evidence inspected

- **Zero-copy floor (`cc8ba1374f32`):** `zero-copy-floor/run-20261126T112751Z.tar.gz`, SHA-256 `10df0d895755494ec6f06369071d649674d855ab953201a2b2888845ab265e34` (code commit `1e27e46f972f`; 648 decodes, pinned on i9-10900K: 10 cores, 20 MiB L3, scanner pinned to core `[6, 16]`, workers+reader on 9 remaining physical cores, PMU group 5 + uncore IMC live).
- **Pooled scan (`c71c4a56c145`):** `pooled-scan/run-20261126T133034Z.tar.gz`, SHA-256 `97e397ad586a6700c04a874e50dbc5e26ee9fc91912b115111cbe0550ddc8867` (code commit `1f6a578ce3c1`; 384 timed + 96 traced decodes, unpinned on i9-10900K).
- **Ideal-parts matrix (`7dd7b07083b2`):** `ideal-matrix/run-20261125T125334Z.tar.gz`, SHA-256 `f0acf19929c8e2f59284e8271b0e109d51c47cd284e1fc511959334f60970fe5` (code commit `e9d5e778251d`; 2,492 decodes).
- **Source code:** `src/rle.rs`, `src/parallel.rs`, `src/open.rs`, `src/open/outq.rs`, `src/lib.rs`, `src/ideal/`, and `cc8ba137`'s `src-against-pin.diff` (`src/open/supplied.rs`, `src/ideal/rle1.rs`).

## Summary of findings

One physical mechanism — **20 MiB L3 cache capacity vs. multi-hundred-MiB frozen oracle stores and multi-pass block buffers** — exposes three hidden assumption failures on the current frontier map (`D`, `E_consume`, and paired-oracle `K1` kill tests) and one completely unmapped production bottleneck (`R`: RLE1/CRC under LLC pressure):

1. **Delivery (`D`) is not a flat 48 ms floor (`delivery-copyout` premise falsified).** `SFCMR`'s flat 48 ms is the single-core DRAM read bandwidth of the 209.7 MiB cold `R` oracle store (`853.5 MiB` DRAM reads at every `k`). On real blocks (`c71c4a56`), `Reader::copy_out` is **27.6–29.1 ms** at `k = 1..6` and **34.0 ms** at `k = 8`, then degrades to **43.7 ms** (`none`), **67.4 ms** (`PC`), and **77.9 ms** (`SC`) at `k = 10` as `ready_bytes` fills 12–16 MiB of the 20 MiB L3. Moreover, zero-copy block lending already exists at `StreamDecoder::read_block` (`src/parallel.rs:880`).
2. **`E_consume = 298 ms` (`spelling-census`, `rauzy-output`) includes ~55–65 ms of cold-store DRAM reads.** At `k = 1`, `SECMRD` reads **504.3 MiB** from DRAM — **3.34×** more than the entire shipped decoder `none` (**151.1 MiB**), because it streams 425.1 MiB of synthetic pre-recorded `u32` codes plus `C`'s cold 120.6 MiB store into a cold `out` buffer. With warm `C` (`7dd7b070` arm `E`), `consume_ns` drops immediately by 41.5 ms (`300.9 → 259.4 ms`); true in-situ spelling is likely **~230–245 ms**.
3. **Asymmetric cold-store bias in paired-oracle `K1` tests (`c71c4a56`).** In `c71c4a56`, the zero-copy `C` oracle added `+176..+294 ms` (`+20..+33 ms/worker`) of cold-store `A7` bloat to the **worker-bound** `PC` and `SC` arms while adding **0 ms** to the **scanner-bound** `C` arm (`job_wait = 50.6–91.8 ms/worker`). Subtracting that traced `ΔA7/k` bloat shifts `k = 8` capture from `0.524` to `~0.727` (`PC/C ~ 0.713` vs. `√(SC/C) ~ 0.778`). Under C2, `pooled-scan`'s KILL verdict stands (and `P/none` is still `+5.2%` at `k = 8`), but future paired-oracle registrations must not pair a worker-bound candidate against a scanner-bound cold-store `C` baseline.
4. **Off the map: `R` (RLE1/CRC, `A9`) is 3.5× larger than `surface.tsv` reports and drives the 8→10-worker LLC cliff.** `surface.tsv` lists `R` as `1.7% (1w), -0.9% (8w), small` with no claim because `7dd7b070`'s copy-based `R` oracle masked ~55 ms of `R` behind an 839 MiB copy. Primary zero-copy (`cc8ba137` `SECMD − SECMRD`) and traced (`c71c4a56` `A9`) data show `R` is **66.6–74.4 ms at 1W (5.0–5.7%)**, **104–115 ms total CPU at 8W (7.1% of worker CPU)**, and explodes **2.55×–2.9×** under LLC pressure (`133–170 ms` in `none`/`SECMD`, `191 ms` in `PC@10W`, `216 ms` in `SC@10W`). In `src/rle.rs:103–170`, `unrle_in_place` executes **4 full-buffer passes (2 full-buffer writes)** on every block with `shift > 0`.

---

## 1. Delivery (`D`): cold-store DRAM artifact vs. real L3-eviction curve

In `cc8ba137`, the zero-copy `R` oracle stores the 431 distinct decoded blocks in a 209.7 MiB global store (`store.R.bytes = 209,707,092`; total oracle store `520.2 MiB`) and hands the reader `BlockBuf::Shared(Arc<[u8]>)` (`src/ideal/rle1.rs` in `cc8ba137` diff). Because 209.7 MiB exceeds the i9-10900K's 20 MiB L3 by 10.5×, `SFCMR` streams all 839 MiB cold from DRAM at every worker count:

| arm (`cc8ba137`) | `k` | wall (ms) | `LLC-load-misses` | `imc data_reads` (MiB) | `imc data_writes` (MiB) | IMC BW (GB/s) |
|---|---:|---:|---:|---:|---:|---:|
| `none` (shipped, real `D`) | 1 | 1349.01 | 700.5 K | **151.1** | 100.9 | 0.20 |
| `SFCMR` (oracle `SFCMR`, real `D`) | 1 | 54.47 | **5,085.0 K** | **853.5** | 18.0 | 16.78 |
| `SFCMRD` (all oracle) | 1 | 5.79 | 300.6 K | 52.8 | 12.7 | 11.87 |
| `SFCMR` | 8 | 53.24 | **5,108.8 K** | **853.0** | 18.2 | 17.16 |
| `SFCMR` | 10 | 54.41 | **5,061.0 K** | **856.1** | 26.2 | 17.00 |

At `k = 1`, `none` runs the exact same real `StreamDecoder::read` copy-out of 839 MiB, yet its **entire** decode reads only **151.1 MiB** from DRAM — 5.6× less than `SFCMR` alone — because blocks stay hot in the 20 MiB L3 between worker write and reader copy.

In `c71c4a56`'s traced runs (`n = 3` per cell, medians `[min..max]`), real `Reader::copy_out` is neither 48 ms nor flat:

| `k` | `none` `copy_out` (ms) | `none` `ready_bytes` hwm (MiB) | `PC` `copy_out` (ms) | `SC` `copy_out` (ms) | `SC` `ready_bytes` hwm (MiB) |
|---:|---:|---:|---:|---:|---:|
| 1 | **29.1** `[27.1..30.8]` | 0.9 | — | — | — |
| 6 | **27.6** `[27.5..31.3]` | 5.1 | 29.8 `[29.6..31.5]` | 29.0 `[28.4..31.9]` | 6.0 |
| 7 | **30.6** `[29.7..31.8]` | 5.2 | 33.7 `[31.1..33.9]` | 32.5 `[31.0..35.7]` | 7.7 |
| 8 | **34.0** `[33.0..35.3]` | 6.9 | 40.0 `[38.1..40.7]` | 39.5 `[39.2..40.7]` | 8.6 |
| 9 | **43.2** `[35.8..43.9]` | 12.0 | 51.5 `[50.9..55.5]` | 54.9 `[52.4..60.5]` | 10.3 |
| 10 | **43.7** `[42.7..44.1]` | 14.6 | **67.4** `[65.5..68.2]` | **77.9** `[69.8..81.0]` | 16.3 |

Two implications for `delivery-copyout`:
- At `k = 1..8`, serial `copy_out` is **27.6–34.0 ms** (L3-hot, ~25–31 GB/s), not 48 ms.
- At `k = 9..10` and whenever worker throughput approaches reader speed (`PC`, `SC`), `READY_BYTES = 12 MiB` (`src/parallel.rs:74`) and `SLOTS_PER_THREAD = 4` (`src/parallel.rs:47`) let 12–16.3 MiB of unread ready blocks sit in flight out of a 20 MiB L3, evicting ready blocks to DRAM and pushing `copy_out` up to **67.4–77.9 ms** (slower than the single-core 48 ms DRAM floor because the reader now competes with 10 workers for DRAM!).
- Finally, zero-copy block lending is already implemented in `StreamDecoder::read_block` (`src/parallel.rs:880–915`, `#[doc(hidden)]`, used by `BzReader`'s `BufRead` and `examples/bzcat.rs`).

---

## 2. `E_consume` (298 ms on `surface.tsv`) includes cold-store DRAM reads

In `cc8ba137`, `SECMRD` runs `consume_codes` (`src/open/supplied.rs:312` in `cc8ba137` diff) by borrowing `&Built` from `C`'s 120.6 MiB frozen store (`store.C.bytes = 120,613,698`) and `&[u32]` from `supplied_codes`' 106.3 MiB store (`store.code_bytes = 106,270,324` across 233 distinct Open blocks × 4 streams = **425.1 MiB** of `u32` codes streamed per decode).

At `k = 1` in `cc8ba137`:
- `none` (entire shipped decoder): `imc data_reads = 151.1 MiB`, `A7` (walk + spelling) = `545.2 ms` (`zero-copy-floor.tsv:68`).
- `SCMRD` (walk + spelling from cold `C` store): `imc data_reads = 546.6 MiB` (**3.62× `none`**), `LLC-load-misses = 3,401.0 K` (**4.86× `none`**), `A7 = 697.9 ms` (**+152.7 ms / +28.0%**).
- `SECMRD` (spelling only from cold `C` + cold `supplied_codes` store): `imc data_reads = 504.3 MiB` (**3.34× `none`**), `consume_ns = 300.9 ms`, `wall = 304.3 ms`.

Cross-check against `7dd7b070` (`k = 1`): in arm `E`, where `C` is **real** (so `inline16`, `lens`, `spelling`, and `outq`-warmed `out` are in L2/L3, though `codes` are still read from DRAM), median `consume_ns` is **259.4 ms** — already **41.5 ms lower** than `SECMRD`'s `300.9 ms`. Subtracting the DRAM read cost of the 425.1 MiB synthetic `u32` codes stream (absent in production, where `cf = ef >> bits` lives in a register in `src/open.rs:1012` — at ~17–18 GB/s, ~24 ms) puts true in-situ spelling near **~230–245 ms** (~17–18% of 1W wall, ~55–65 ms below the 298 ms figure). `rauzy-output` and `forward-emission` remain high-value targets, and should apply `#951`'s `spelling-census` ratios to this ~230–245 ms warm-cache i9 baseline rather than `SECMRD`'s 298 ms cold-store headline.

---

## 3. Asymmetric cold-store bias in paired-oracle `K1` tests (`c71c4a56`)

In `c71c4a56`, the zero-copy `C` oracle inflated worker `A7` in every `C` arm (`C`, `PC`, `SC`) relative to the corresponding real-`C` arm (`none`, `P`, `S`):

| `k` | `ΔA7 (C − none)` | `ΔA7 (PC − P)` | `ΔA7 (SC − S)` | `C` worker `job_wait/k` | Obs `PC/C` vs `√(SC/C)` (cap) | Adj `PC/C` vs `√(SC/C)` (cap) |
|---:|---:|---:|---:|---:|---:|---:|
| 6 | +175.7 ms (+29.3/w) | +181.8 ms (+30.3/w) | +169.4 ms (+28.2/w) | 20.8 ms/w | 1.061 vs 0.943 (−0.55) | 0.897 vs 0.858 (0.39) |
| 7 | +175.9 ms (+25.1/w) | +193.0 ms (+27.6/w) | +185.6 ms (+26.5/w) | 40.5 ms/w | 0.942 vs 0.892 (0.28) | 0.792 vs 0.807 (0.60) |
| 8 | +234.6 ms (+29.3/w) | +220.4 ms (+27.5/w) | +196.2 ms (+24.5/w) | 50.6 ms/w | 0.863 vs 0.860 (0.52) | **0.713 vs 0.778 (0.73)** |
| 9 | +177.8 ms (+19.8/w) | +293.9 ms (+32.7/w) | +257.7 ms (+28.6/w) | 91.8 ms/w | 0.868 vs 0.872 (0.55) | **0.690 vs 0.778 (0.79)** |
| 10 | +242.9 ms (+24.3/w) | +244.7 ms (+24.5/w) | +366.1 ms (+36.6/w) | 73.7 ms/w | 0.860 vs 0.886 (0.65) | **0.729 vs 0.768 (0.66)** |

*(Obs walls from timed medians, `n = 12`; `ΔA7` from traced medians, `n = 3`; Adj subtracts `ΔA7/k` from worker-bound `PC` and `SC` while leaving scanner-bound `C` unchanged.)*

The bias is structural: `C` alone is **scanner-bound** (`50.6–91.8 ms/worker` of idle `job_wait` at `k = 8, 9`), so `C`'s wall absorbs **0 ms** of the cold-store `A7` bloat, whereas `PC` and `SC` are **worker-bound** (`job_wait = 3.6` and `0.2 ms/worker` at `k = 8`) and pay **100%** of it (`+24.5..+32.7 ms/worker`). Subtracting equal bloat `Δ` from both `(C − PC)` and `(C − SC)` strictly lowers `(C − PC)/(C − SC)` whenever `PC > SC`.

**C2 compliance:** This does not relax or overturn `pooled-scan`'s frozen KILL verdict on `c71c4a56` (`P/none` is still `+5.2%` at `k = 8` and `+10.4%` at `k = 6` on current workers). Rather, it establishes a methodological constraint for `pooled-scan-hybrid` and future paired-oracle tests: any `K1` test comparing a worker-bound candidate against a scanner-bound zero-copy `C` baseline is systematically biased against the candidate unless `C`'s cold-store inflation is either eliminated (e.g., testing against a real faster `C` or a warm-cache replay) or accounted for in the pre-registered estimator.

---

## 4. Off the map: `R` (RLE1/CRC, `A9`) and the 8→10-worker LLC-capacity cliff

`surface.tsv` (row `R`) lists `RLE1/CRC` as `1.7% (1w), -0.9% (8w), small, claims: -`, and `dead-ends.tsv` (row 8) concludes memory is not a third station at 8–10 workers because `SECMRD` wall was flat and peak DRAM bandwidth in `none` was 15 GB/s (`floor/README.md` §4). Both conclusions missed an LLC-capacity mechanism visible in `cc8ba137` and `c71c4a56`:

### 4.1 Why `surface.tsv` underestimated `R` by 3.5×

In `7dd7b070`, the `R` oracle copied 839 MiB from its store (~55 ms at 1W), cancelling most of `R`'s real cost (`none − R = 21.7 ms`, `1.7%`). Both `cc8ba137` (zero-copy `SECMD − SECMRD`) and `c71c4a56` (traced real `A9`) measure `R`'s true cost — and its steep LLC-pressure blow-up:

| `k` | `cc8ba137` `SECMD − SECMRD` wall (ms) | `R` CPU (ms) | `R` cycles (M) | `R` `ΔLLC-st` (K) | `R` `Δimc` rd+wr (MiB) | `SECMD` wall (ms) | `SECMD` BW (GB/s) | `c71c4a56` `none` `A9` (ms) | `c71c4a56` `PC`/`SC` `A9` (ms) |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 1 | 66.59 | 66.8 | 338.9 | +51.4 | +10.0 | 370.89 | 1.55 | **74.4** | — |
| 6 | 12.39 | 73.8 | 366.2 | +195.3 | +73.9 | 65.42 | 10.28 | **89.5** | 95.1 / 94.2 |
| 7 | 11.12 | 75.3 | 373.8 | +285.0 | +128.0 | 57.23 | 12.97 | **95.3** | 103.4 / 102.3 |
| 8 | 14.40 | **110.3** | **557.1** | +1,034.1 | **+286.5** | **55.38** | 16.66 | **104.0** | 118.6 / 119.7 |
| 9 | 15.35 | **145.3** | **719.6** | +2,258.3 | **+583.5** | **55.82** | 22.43 | **126.5** | 149.7 / 152.3 |
| 10 | 16.95 | **170.5** | **863.0** | +3,185.3 | **+766.7** | **56.62** | **25.91** | **133.0** | **191.2 / 215.6** |

Notice three facts from `cc8ba137` (where `eff_GHz = 4.905 GHz` is flat across `k = 6..10` and `k = 6..8` has **zero SMT sharing** on workers):
- Even between `k = 6` and `k = 8` (zero SMT, constant 4.905 GHz), pure `R` cycles jump **+52%** (`366.2M → 557.1M`) and `R` DRAM traffic jumps **3.9×** (`+73.9 → +286.5 MiB`, **28.7×** of `k = 1`).
- At `k = 10`, `R` adds **+766.7 MiB** of DRAM traffic (`+427.4` rd + `+339.3` wr) and **+3.19M** LLC store misses, pushing `SECMD` to **25.91 GB/s** and causing `SECMD` wall to scale **backwards** (`55.38 → 55.82 → 56.62 ms`).
- In `none` across `k = 1 → 6 → 8 → 10`, total DRAM traffic climbs **11.2×** (`252 → 894 → 1,602 → 2,831 MiB`, 3.4× the 839 MiB decoded corpus) and total CPU cycles climb **+19.2%** (`8,233.8M → 9,815.7M`, **+1,582M extra cycles ≈ 322 ms** of wasted CPU at 4.905 GHz; `+255M` cycles already between `k = 6` and `k = 8` with zero SMT).

### 4.2 Root cause in `src/rle.rs:103–170` (`unrle_in_place`)

On the run route (`src/lib.rs:635`), every block calls `rle::unrle_in_place(&mut out, &mut s.rle_runs, &mut s.ll, k)`. Whenever `shift > 0` (even `shift == 1` — a single 6-byte run anywhere in the block!), `src/rle.rs:117–169` executes **four full-buffer passes** over the ~900 KB block:

1. **Pass 1 (`find_runs`, line 117):** full read of `buf[0..n]`.
2. **Pass 2 (lines 135–139):** `buf.reserve_exact(shift)` (which reallocates if `cap < n + shift`; initial pool allocation in `src/parallel.rs:1210` is `n + n/16 + 64`), `buf.resize(n + shift, 0)`, then `buf.copy_within(first..n, first + shift)` — **full read + full write** shifting almost the entire block up by `shift`.
3. **Pass 3 (lines 150–166):** forward expansion loop `buf.copy_within(src..from, dst)` + `fill` — **second full read + second full write** shifting almost the entire block back down.
4. **Pass 4 (lines 142–148, 168):** `crc::update_with` in 64 KiB tiles — **fourth read** over the buffer.

At `k = 1`, the ~900 KB block stays inside the 20 MiB L3 across all four passes (`+10.0 MiB` DRAM). At `k = 8..10`, with 8–10 workers plus 12–16 MiB of unread blocks in `Ready`, those two full-buffer write passes spill dirty cache lines out of the 20 MiB L3 and back from DRAM — multiplying `R`'s cycles by **2.55×** and DRAM traffic by **77×**.

### 4.3 New direction opened (`idea` claim: `single-pass-rle1-crc`)

Opened `frontier/claims/single-pass-rle1-crc.toml` at `status = "idea"`: first characterise `shift`, `min_grown`, `realloc` rate, and per-pass time of `unrle_in_place` on real blocks, then evaluate single-write-pass alternatives (e.g., backward in-place expansion when `min_grown >= 0` or one-pass `unrle + crc` from `buf` into a swapped/recycled buffer `stage`, cutting full-buffer writes in half and reducing LLC writeback pressure at `k = 8..10`).
