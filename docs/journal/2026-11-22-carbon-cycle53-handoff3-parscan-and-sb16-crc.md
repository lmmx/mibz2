# Carbon Cycle 53 (15 / 20) & Interim Handoff 3 — Balanced-XOR-Tree Sliced-by-16 CRC, Low-Thread RSS Diet, and Safe Scalar Stack

- **Date:** 2026-11-22
- **Branch:** `research/carbon-word-transport`
- **Base `origin/master`:** `dd6450e6ab7dc80064f86ce2a737eb4b998cedf1` (includes `#739` Wormhole plateau stack + `#745` Cycle 36 fast bitreader)
- **Drop-In Directory:** `docs/journal/2026-11-22-carbon-handoff3-parscan-rle-huff-crc/`
- **Unified Patch:** `docs/journal/2026-11-22-carbon-handoff3-parscan-rle-huff-crc/handoff3_over_master.patch` (`3,955` lines; `git apply --check` clean on `dd6450e6`)
- **Safety Gate:** `#![forbid(unsafe_code)]` on `src/par_scan.rs`, `src/block.rs`, and `src/huff.rs`; **zero new `unsafe` blocks** across all 12 drop-in files (`100%` safe Rust in `src/crc.rs`, `src/mtf.rs`, `src/runs.rs`, `src/rle.rs`, `src/parallel.rs`, `src/read.rs`, `src/lib.rs`); `std` only.

---

## 1. Summary of Cycles 49–53 Cumulative Delta Over Interim Handoff 2

Interim Handoff 3 packages all winning safe-Rust improvements from **Cycles 49–53** on top of Handoff 2 (`b2baa9c4`, rebased cleanly onto `origin/master` `dd6450e6`):

1. **Cycle 53 (`src/crc.rs`) — Balanced-XOR-Tree Sliced-by-16 `crc::update` (`0.6784x`, `1.474x` faster):**
   - Replaces `master`'s linear 8-byte serial XOR chain (`crc = t7[...] ^ t6[...] ^ ... ^ t0[...]`) in `crc::update` with a `16`-byte (`u128` / `2 x u64`) **Sliced-by-16** lookup table (`const TABLE16: [[u32; 256]; 16]`, `16 KiB` static `.rodata`, computed at compile time by `const fn make_tables()`) evaluated via a **depth-4 balanced binary XOR tree** (`((p0 ^ p1) ^ (p2 ^ p3)) ^ ((p4 ^ p5) ^ (p6 ^ p7))`), with an 8-byte tail using `TABLE16[0..8]` and byte tail using `TABLE16[0]`.
   - Same-binary 25-round A/B (`c53_crc_screen.rs` on `32` `wikidata-l9` blocks = `28.8 MB`):
     - `Master` (`sb8_linear`): `20.64 ms` (`A/A = 1.0015x`, `1.55 GB/s`)
     - `Cand A` (`sb8_tree`, same `8 KiB` table): `18.30 ms` (`0.8866x`, `1.75 GB/s`)
     - **`Cand B` (`sb16_tree`, `16 KiB` table): `14.00 ms` (`0.6784x`, `1.474x` faster, `2.29 GB/s`) — KEPT.**
   - Directly accelerates scalar fallback CRC, multi-stretch `unrle_in_place` negative-zone / positive-zone runs when AVX-512 `VPCLMULQDQ` is absent, and tail/non-PCLMUL targets, contributing to **`0.9778x–0.9925x` total CPU** vs `dd6450e6` across `4..20T` on `wikidata-l9`.

2. **Cycle 49 (`src/huff.rs`, `src/block.rs`) — Auto-Vectorized `vpaddd` Huffman Pair Table Build + `16 KiB` L1d-Tiled `syms.resize`:**
   - `huff::Tables::build` uses `split_at_mut(off_p)` to clone pre-built first-symbol templates across second-symbol offsets via auto-vectorized `vpaddd` + tail-only `d_tail[..LUT_LEN - filled_to]`, and `block::read_item_core` resizes `syms` in `8,192`-`u16` (`16 KiB`) L1d tiles rather than pre-faulting `1.8 MiB` up front (`P1` `0.9790x`, `1T` `0.9637x`, `native` `6T` RSS `-6.5%`).
   - Killed `c49_sel_lens` (`1.0051x` / `1.0307x` — short `0..2` zero runs mispredict) and `c49_quad_lut10` (`1.1062x–1.2754x` — `c <= 4` shrinks unrolled group prefix from `80% -> 60%` and doubles L1d footprint).

3. **Cycle 50 (`src/par_scan.rs`, `src/mtf.rs`) — Lock-Free `claimable: AtomicUsize` Helper Wake Discipline + Lazy `ScanScratch` + Safe `16 KiB`-Tiled `mtf_scalar`:**
   - `ParScanHub` tracks `claimable: AtomicUsize` (`ready_slabs.len() + scan_queue.len()`) so workers check `claimable.load(Acquire) > 0` before locking `hub.mu`, wake helpers strictly 1-for-1 via `wake_helpers(jobs, 1)`, and allocate `54 KiB` `ScanScratch` lazily (`Option<Box<ScanScratch>>`), cutting `records-l9` CPU overhead from `1.04–1.08x` to `1.01–1.03x`.
   - `mtf_scalar` & `front_scalar` copy `b.seq_to_unseq` directly and tile `ll` in `16 KiB` chunks (`0.9323x` `wikidata-l9`, `0.9349x` `records-l9`).

4. **Cycle 51 (`src/par_scan.rs`, `src/runs.rs`) — Little-Endian `8`-Anchor `u64x4` Scanner + Zero-Slack `8`-Wide Safe `runs::fill_scalar`:**
   - `ANCHOR_BITMAP` switched to native `u16::from_le_bytes` (`bswap` removed) with `8`-anchor `u64x4` loop unrolling (`0.8631x` standalone scan, **`0.52%`** serial share on `wikidata-l9`).
   - `runs::fill_scalar` uses `8`-wide fixed-slice writes `&mut tt[w..w + 8]` with `4,096`-`u32` L1d-tiled `tt.resize` and exact `n` capacity (`<= 7`-row scalar tail, `0` slack so `trace::tests` pass `5/5`): **`0.7295x`** (`1.37x` faster).

5. **Cycle 52 (`src/par_scan.rs`) — `single_slab_scanner` `max_front_ahead` Horizon Cap:**
   - Caps single-chunk / tail stream issuance at `(st.issued - st.taken) < hub.max_front_ahead as u64` (`workers + 4`), dropping `4T` `i9` RSS to **`39.31 MiB` (`0.7966x`, `-10.04 MiB`)** and `6T` `i9` RSS to **`43.08 MiB` (`0.7558x`, `-13.92 MiB`)**.
   - Killed `c52b` (`held_out` thread-local `free_out` hoarding: `+4.2..11.2 MiB` RSS at `8–20T` because parked workers strand `900 KiB` buffers).

---

## 2. Applying & Testing Interim Handoff 3 Over `origin/master`

From a clean checkout of `origin/master` (`dd6450e6` or compatible):

```bash
git apply --check docs/journal/2026-11-22-carbon-handoff3-parscan-rle-huff-crc/handoff3_over_master.patch
git apply docs/journal/2026-11-22-carbon-handoff3-parscan-rle-huff-crc/handoff3_over_master.patch
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test --release --lib --tests
cargo test --release trace::tests -- --nocapture
cargo build --release --example ab
```

Drop-in files included in `docs/journal/2026-11-22-carbon-handoff3-parscan-rle-huff-crc/`:
- `src/par_scan.rs` (`#![forbid(unsafe_code)]` push-streaming `ParScanHub`, `ANCHOR_BITMAP` LE `u64x4` scanner, `claimable: AtomicUsize`, `single_slab_scanner` horizon cap)
- `src/block.rs` (`#![forbid(unsafe_code)]` `16 KiB` L1d-tiled `syms.resize` + `read_item_core`)
- `src/huff.rs` (`#![forbid(unsafe_code)]` `10`-bit pair LUT `vpaddd` build + `5`-step unrolled `decode_group_win`)
- `src/crc.rs` (`100%` safe Balanced-XOR-Tree Sliced-by-16 `TABLE16` `crc::update`)
- `src/rle.rs` (Bipolar `unrle_in_place` + `5`-pos SWAR `unrle_scalar` / `find_runs_scalar`)
- `src/mtf.rs` (Safe `16 KiB`-tiled `mtf_scalar` & `front_scalar`)
- `src/runs.rs` (Zero-slack `8`-wide safe `fill_scalar` + `16 KiB` `tt.resize` tiling)
- `src/parallel.rs`, `src/read.rs`, `src/lib.rs` (`StreamDecoder::parscan`, `BzReader::parscan`, `decode_parscan`)
- `tests/par_scan.rs`, `examples/ab.rs` (`mibz2-parscan` A/B target)

---

## 3. Qualification Results

### 3.1 Unit, Integration, Trace Ledger, and `1,500`-Case Streaming Fuzz Gates
- `cargo fmt --check` & `cargo clippy --all-targets -- -D warnings`: **0 warnings, 0 errors**.
- `cargo test --release --lib`: **`194 / 194` passed** (including `crc::tests` across all alignments/lengths and `trace::tests` `5/5`).
- `cargo test --release --tests`: **all `21` integration test suites passed** (`par_scan` `13/13`, `differential` `31/31`, `malformed` `29/29`, `concat_and_limits` `35/35`, `streaming_differential` `7/7`, `thread_scaling` `6/6`, etc.).
- `c41_bench verify`: **`PASS 1..5`** (`1B / 7B / 63B / 4KiB / 65537B / 1MiB` chunking, slow-reader backpressure, `cancel()` + `PanickingReader` unwind safety, and **`1,500 / 1,500` streaming chunked mutation fuzz cases** exact on valid prefix bytes + `Error` vs `master`).

### 3.2 `examples/ab` 10T Child Subprocess Verification (`wikidata_x3.bz2`, `629,100,588` B out)

| Pair | `mibz2` (`dd6450e6`) Wall | `mibz2-parscan` Wall | `mibz2` CPU | `mibz2-parscan` CPU | `mibz2` RSS | `mibz2-parscan` RSS | Hash (`bbe9dae7753f6c67`) |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | :---: |
| 1 | `0.9148 s` | `0.9199 s` | `1.6139 s` | **`1.5953 s`** | `69,048 KiB` | **`58,372 KiB` (`0.845x`)** | Exact |
| 2 | `0.9873 s` | **`0.8826 s`** | `1.6379 s` | **`1.5896 s`** | `70,556 KiB` | **`58,156 KiB` (`0.824x`)** | Exact |
| 3 | `0.8816 s` | `0.8964 s` | `1.6129 s` | **`1.5901 s`** | `70,348 KiB` | **`56,664 KiB` (`0.805x`)** | Exact |
| **Median** | `0.9148 s` | **`0.8964 s` (`0.980x`)** | `1.6139 s` | **`1.5901 s` (`0.985x`)** | `70,348 KiB` | **`58,156 KiB` (`0.827x`)** | **Exact** |

Notice `thread_cpu` inside `mibz2-parscan`: `mibz2-scan=0.11..0.12 s` vs `mibz2-scan=0.21 s` on `master`, while total process CPU drops from `1.6139 s -> 1.5901 s` (`0.985x`) and peak RSS drops by **`-12.19 MiB` (`0.827x`)**!

### 3.3 Subprocess Peak RSS (`VmHWM`) Audit (`wikidata_x3.bz2`, `629 MB` decompressed)

| Threads | `i9` Master MiB | `i9` Cand MiB | `i9` Ratio | `native` Master MiB | `native` Cand MiB | `native` Ratio |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `1` | `28.54` | `28.34` | **`0.9933x`** | `28.29` | `28.36` | **`1.0026x`** |
| `2` | `64.91` | `61.96` | **`0.9546x`** | `68.85` | `59.15` | **`0.8591x`** |
| `4` | `49.35` | `39.31` | **`0.7966x`** | `98.86` | `86.56` | **`0.8756x`** |
| `6` | `57.00` | `43.08` | **`0.7558x`** | `133.68` | `107.66` | **`0.8054x`** |
| `8` | `61.82` | `49.19` | **`0.7958x`** | `60.82` | `48.84` | **`0.8031x`** |
| `10` | `60.64` | `49.89` | **`0.8226x`** | `66.09` | `53.23` | **`0.8053x`** |
| `12` | `59.34` | `48.88` | **`0.8236x`** | `72.23` | `56.51` | **`0.7823x`** |
| `16` | `59.99` | `49.39` | **`0.8233x`** | `79.62` | `64.04` | **`0.8043x`** |
| `20` | `58.79` | `49.08` | **`0.8349x`** | `83.14` | `69.79` | **`0.8394x`** |

Every single `2–20T` point in both `i9` (`0.7558x–0.9546x`) and `native` (`0.7823x–0.8756x`) modes is **`4.5%–24.4%` below `origin/master`** (well inside the `<= 1.10x` gate).

### 3.4 Same-Binary Push-Streaming A/B (`1..20T`, 7 Interleaved Rounds on 2-Core / 4-vCPU Host)

- **`wikidata-l9` (`209.70 MB` out):**
  - Serial-stage CPU share: `14.15%` (`67.19 ms`) -> **`0.52%`** | Modeled `10T` critical-path ratio vs `dd6450e6` = **`0.7164x`**.

| Threads | `MasterA` ms (`dd6450e6`) | `A/A` | `Cand / MasterA` | `Cand / M329` | `CPU Ratio` |
| ---: | ---: | ---: | ---: | ---: | ---: |
| `1` | `768.89` | `0.9947x` | `1.0027x` | **`0.9802x`** | `1.0059x` |
| `2` | `416.89` | `1.0218x` | `1.0401x` | `1.0167x` | **`0.9961x`** |
| `4` | `450.12` | `0.9800x` | `1.0095x` | `1.0079x` | **`0.9778x`** |
| `6` | `466.67` | `1.0048x` | **`0.9920x`** | **`0.9916x`** | **`0.9819x`** |
| `8` | `447.95` | `1.0016x` | **`0.9797x`** | **`0.9629x`** | **`0.9841x`** |
| `10` | `455.37` | `1.0034x` | **`0.9605x`** | **`0.9558x`** | **`0.9827x`** |
| `12` | `448.86` | `1.0053x` | **`0.9896x`** | **`0.9788x`** | **`0.9854x`** |
| `16` | `490.90` | `0.9504x` | **`0.9237x`** | **`0.9575x`** | **`0.9802x`** |
| `20` | `482.42` | `0.9668x` | **`0.9357x`** | **`0.9350x`** | **`0.9925x`** |

- **`wikidata-l1` (`36.00 MB` out):**
  - Serial-stage CPU share: `18.41%` (`22.03 ms`) -> **`0.54%`** | Modeled `10T` critical-path ratio = **`0.6167x`**.
  - `1T`: `0.9967x` vs `dd6450e6` (`0.9448x` vs `M329`), `2T`: `1.0045x` (`A/A = 1.0063x`, `0.9900x` vs `M329`), `CPU Ratio`: `0.9921x–0.9987x` across `1..10T`.
- **`records-l9` (`9.60 MB` out):**
  - Serial-stage CPU share: `20.82%` (`33.63 ms`) -> **`1.21%`** | Modeled `10T` critical-path ratio = **`0.4833x`**.
  - `1T`: `0.9977x` (`0.9689x` vs `M329`), `2T`: `1.0190x` (`A/A = 1.0176x`, `0.9702x` vs `M329`), `10T`: `1.0130x` wall on 2-core host (`1.0294x` CPU).

### 3.5 Safe Scalar Kernel Suite (`Kernels::scalar()`, `#![forbid(unsafe_code)]` / `100%` Safe)
- `rle::unrle` (`64` `wikidata-l9` blocks): `47.94 ms -> 11.40 ms` (**`0.2378x`**, `4.21x` faster, `A/A = 0.9887x`)
- `crc::update` (`32` `wikidata-l9` blocks): `20.64 ms -> 14.00 ms` (**`0.6784x`**, `1.47x` faster, `A/A = 1.0015x`)
- `runs::fill_scalar` (`32` `wikidata-l9` blocks): `24.02 ms -> 17.52 ms` (**`0.7295x`**, `1.37x` faster)
- `mtf::mtf_scalar` (`32` `wikidata-l9` blocks): **`0.9323x`**
- Full `2T` scalar decode (`8` `wikidata-l9` streams): `785.11 ms -> 614.35 ms` (**`0.7825x`**, `A/A = 1.0011x`)

---

## 4. Remaining 5 Cycles (`Cycles 54–58 -> Final Handoff`)
- **Cycle 54 (`16/20`):** Screen deferred `r.live` / MSB sentinel bit-buffer in `huff::decode_group_win` (`90%` of `P1`).
- **Cycle 55 (`17/20`):** Screen progressive reader slab ramp (`512 KiB -> 1 MiB -> 4 MiB`) in `BzReader::parscan` to eliminate first-chunk cold-start starvation on `records-l9` (`3.3 MB`) and `wikidata-l1` (`12 MB`).
- **Cycle 56 (`18/20`):** Screen fused single-pass `mtf_checkpoints_scalar` + precomputed `runs::cuts` division-free threshold table.
- **Cycle 57 (`19/20`):** Screen streaming negative-zone tile CRC in `rle::unrle_in_place` + `u128` 13-position SWAR `find_runs_scalar`.
- **Cycle 58 (`20/20`) + Final Handoff:** Rebase onto latest `origin/master`, run full qualification, publish Final Handoff, push & verify remote SHA, then stop.
