# Wormhole Cycle 33 (Neon) — Wall W3 & W5: L1/L2-Tiled `RLE1 + CRC32` Fusion & Zero-Copy `BufRead` Output Assembly (`4W = 0.9625×`, `-2.0 B/B` DRAM)

**Baseline:** `origin/master` `v0.5.12` (`6a36b0e10`), `BzReader::parallel_workers` (`OpenCells`, `slots = 4*W`), with `AbHash` (exact `examples/ab.rs` 4-lane 32-byte consumer hash) and A/A control (`MasterA` vs `MasterB`) in the same binary.
**Candidate harness:** [`c33_rle_crc_ab.rs`](2026-11-21-wormhole-cycle33/c33_rle_crc_ab.rs) (`safe_c33` module `#![forbid(unsafe_code)]`), raw log [`c33_results.txt`](2026-11-21-wormhole-cycle33/c33_results.txt), manifest [`c33_rle_crc_assembly.toml`](2026-11-21-wormhole-cycle33/c33_rle_crc_assembly.toml).

---

## 1. Mechanisms Investigated (`Wall W3` Post-BWT Traffic & `Wall W5` Reader Assembly)

1. **RLE1 Run Census & L1/L2-Tiled `unrle + CRC32` Fusion (`C1/C2`):**
   On the six authenticated Wikidata blocks, `rle::unrle` encounters only **`23–83` 4-byte runs per `900 KB` block** (mean `41.7` runs/block → **mean literal span = `21.6 KiB`**!). Yet `v0.5.12`'s `unrle_avx2` (`src/rle.rs:62`) executes `out.extend_from_slice(&rle[i..i+32])` `~28,125` times/block and writes all `900 KiB` (evicting Comet Lake's `256 KiB` L2!) before `crc::crc32` re-reads `900 KiB` from L3/DRAM (`+1.0 B/B`). Fusing span-bulk `unrle` + streaming `crc32` (noting `src/crc.rs::clmul_update(state, slice)` ALREADY takes a streaming `state: u32`!) in `32 KiB` L1/L2 tiles keeps `100%` of `out` cache-hot during CRC.
2. **`C33a_BatchFill` (Multi-Block Batch-Fill `Read::read`):** Draining multiple already-ready consecutive blocks under a single `Mutex<State>` lock into the caller's `4 MiB` buffer (cutting lock acquisitions `431 → 121` at `4W`, `84` at `10W`), while still copying into caller's buffer.
3. **`C33b_ZeroCopy` (`std::io::BufRead::fill_buf` + `consume`):** Zero-copy standard-library trait on `BzReader` (and `with_block` on `StreamDecoder`) handing `&r.cur[r.at..]` directly to the consumer (`ab`, `bzcat`, `wikidata-pq`), eliminating the entire `211 MB` `r.cur → 4 MiB buf` `memcpy` (**`-2.0 B/B` DRAM traffic**) and the `4 MiB` caller buffer (`-1,024` `minflt`)!

---

## 2. Native Safe-Rust A/B + A/A Results

### 2a. `RLE1 + CRC32` Stage Microbench (`6` auth blocks × `39` = `234` blocks, `210.6 MB`, `15` rotated rounds, exact CRC parity)

| Safe-Rust Mechanism | Median (`ms`) | Ratio vs `Sep_Span+SB8` `[95% CI]` | Cache Fit (`48 KiB` L1d) | Verdict |
|---|---:|---|---|---|
| `Sep_Span+SB8` (unfused) | `383.12` | `1.0000 [1.0000, 1.0000]` | Evicts L1d/L2 (`900 KiB`) | Baseline |
| `Sep_Span+SB16` | `209.97` | `0.5493 [0.5459, 0.5527]` | `16 KiB` tbl + `900 KiB` | Faster unfused |
| **`Fused_32K+SB8`** | **`168.73`** | **`0.4423 [0.4386, 0.4461]` (`2.26×`)** | **`8 KiB` tbl + `32 KiB` tile = `40 KiB <= 48 KiB` L1d** | **KEEP (tiles `unrle+crc`)** |
| `Fused_32K+SB16` | `211.69` | `0.5527 [0.5512, 0.5543]` | `16 + 32 = 48 KiB` L1d thrash | **KILL `SB16` tiled** |
| `Fused_16K+SB16` | `211.99` | `0.5553 [0.5515, 0.5591]` | `16 + 16 = 32 KiB` (more loop ohd) | **KILL `16K+SB16`** |

### 2b. `12`-Round Rotated Fresh-Process `4W` A/B (`wikidata_431b_210m.bz2`, `211.0 MB`, exact `AbHash` parity)

| Arm | `4W` med (`ms`) | Ratio vs `MasterA` `[95% CI]` | `user_ms` | `sys_ms` | `minflt` | `rss_MiB` | `locks` | Verdict |
|---|---:|---|---:|---:|---:|---:|---:|---|
| `MasterA` | `368.54` | `1.0000 [1.0000, 1.0000]` | `654.57` | `37.03` | `15,749` | `59.23` | `431` | Control |
| `MasterB` (`A/A`) | `368.94` | `0.9949 [0.9755, 1.0146]` | `654.46` | `36.22` | `15,806` | `60.06` | `431` | `A/A` clean |
| `C32_OneBlkCopy` | `367.64` | `0.9954 [0.9814, 1.0096]` | `658.40` | `29.78` | `12,007` | `56.45` | `432` | C32 carry |
| `C33a_BatchFill` | `370.10` | `0.9960 [0.9827, 1.0095]` | `662.30` | `32.81` | `12,861` | `59.98` | `121` | **KILL** |
| **`C33b_ZeroCopy`** | **`355.71`** | **`0.9625 [0.9416, 0.9839]`** | **`636.97` (`-17.6`)** | **`30.89`** | **`11,742` (`-25.4%`)** | **`55.94`** | `432` | **PROMOTE** |

### 2c. Cold-Start `W = 10` Check (`wikidata_431b_210m.bz2`, `3`-round median)

| Arm (`W = 10`) | `wall_ms` | `user_ms` | `sys_ms` | `minflt` | `rss_MiB` | `locks` |
|---|---:|---:|---:|---:|---:|---:|
| `MasterA` / `MasterB` | `373.63 / 379.14` | `676.68 / 671.88` | `49.02 / 61.46` | `26,455 / 26,782` | `97.21 / 94.43` | `431` |
| `C32_OneBlkCopy` | `372.59` | `685.05` | `39.75` | `21,093` | `90.33` | `432` |
| `C33a_BatchFill` | `380.01` | `702.17` | `43.48` | `21,971` | `93.93` | `84` |
| **`C33b_ZeroCopy`** | **`357.84` (`-4.23%`)** | **`649.38` (`-27.3 ms`)** | `48.05` | **`19,666` (`-25.7%`)** | **`83.43` (`-14.2%`)** | `432` |

---

## 3. Updated `10T` i9-10900K Critical-Path Model (`C32 + C33b + Tiled RLE/CRC`)

- **Post-BWT memory traffic (`B`):** `C33b_ZeroCopy` cuts **`-2.0 B/B`** (`-422 MB` per `211 MB` decode: removes `r.cur` read + `4 MiB buf` write); `64 KiB` L2-tiled `unrle_avx2 + clmul_update` cuts another **`-1.0 B/B`** (`-211 MB`: eliminates `out` L3/DRAM re-read on Comet Lake's `256 KiB` L2) → total post-BWT traffic falls from **`7.0 B/B → 4.0 B/B` (`-43%`)**!
- **`T_reader_serial`:** drops from `~16.0 ms → ~8.5 ms` (`-7.5 ms` serial memcpy eliminated, cutting reader's concurrent DRAM BW load from `~6 GB/s → ~2.5 GB/s`).
- **Modelled `10T` Critical Path (Regime 2 post-Mechanisms `C38b`):** `~50.7 ms → ~46.5 ms` (`-4.2 ms` from `-3.0 B/B` DRAM/L3 pressure relief + `T_worker` RLE/CRC locality).

---

## 4. Kill Decisions (One Line Each)

- **PROMOTE `C33b_ZeroCopy` (`std::io::BufRead::fill_buf` + `consume`):** Beats `v0.5.12` at `4W` (`0.9625 [0.9416, 0.9839]`) and `10W` (`-4.23%` wall, `-27.3 ms` user CPU, `-25.7%` `minflt`, `-14.2%` RSS) while removing `-2.0 B/B` of DRAM traffic with zero API break.
- **KEEP `C2: L1/L2-Tiled unrle + CRC32 Fusion` (`32–64 KiB` tiles):** `2.26×` faster in safe Rust (`0.4423×`) and directly eliminates `1.0 B/B` L3/DRAM re-read on Comet Lake's `256 KiB` L2 when paired with `src/crc.rs::clmul_update`'s existing streaming `state` argument.
- **KILL `C33a_BatchFill`:** Despite cutting lock acquisitions `431 → 84..121`, holding a batch of blocks while copying into a `4 MiB` buffer delays `free_out` return (`+854..878` `minflt`, `+3.5 MiB` RSS) and fails to remove the `211 MB` `memcpy` (`0.9960×`, neutral/slower at `10W`).
- **KILL `Fused_32K+SB16` / `Fused_16K+SB16`:** `16 KiB` slice-by-16 tables spill L1d alongside the tile (`211.7 ms` vs `168.7 ms` for `Fused_32K+SB8`).
