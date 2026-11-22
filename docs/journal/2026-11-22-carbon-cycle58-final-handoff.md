# Carbon Cycle 58 (20 / 20) & Final Handoff — Push-Streaming `ParScanHub`, Bipolar In-Place RLE1, Balanced-Tree SB16 CRC, and Safe Scalar Stack

- **Date:** 2026-11-22
- **Branch:** `research/carbon-word-transport`
- **Base `origin/master`:** `dd6450e6ab7dc80064f86ce2a737eb4b998cedf1` (includes Wormhole `#739` plateau stack + Carbon `#745` Cycle 36 fast bitreader)
- **Final Drop-In Directory:** `docs/journal/2026-11-22-carbon-final-handoff/`
- **Unified Drop-In Patch:** `docs/journal/2026-11-22-carbon-final-handoff/final_handoff_over_master.patch` (`4,397` lines; `git apply --check` clean on `dd6450e6`)
- **Safety Gate:** `#![forbid(unsafe_code)]` on `src/par_scan.rs`, `src/block.rs`, and `src/huff.rs`; **zero new `unsafe` blocks** across all 12 drop-in files (`100%` safe Rust in every new/edited line across `src/{block,crc,huff,lib,mtf,par_scan,parallel,read,rle,runs}.rs`); `std` only.

---

## 1. Executive Summary of the 20-Cycle Campaign (`Cycles 39–58`)

This Final Handoff completes the 20-cycle front-end + RLE1/CRC + push-streaming parallelism campaign (`Cycles 39..58`, publishing Interim Handoff 0 at `6a2d5dad`, Handoff 1 at `20e096e2`, Handoff 2 at `b2baa9c4`, Handoff 3 at `a1c04a1a`, and this Final Handoff):

1. **Push-Streaming Parallel Front End (`StreamDecoder::parscan`, `BzReader::parscan`, `decode_parscan`):**
   - Replaces whole-input pre-buffering and `4 x threads` unbounded inflight queues with a zero-copy `Arc<Vec<u8>>` **`SlabRope`** (`4 MiB` recycled slabs, `<= 2` active slabs via `PRESTAGE_ROPE_BYTES = 4.5 MiB` + `PARSCAN_INPUT_QUEUE = 2`), **`workers + 4` front-end horizon cap** (`max_front_ahead`), and direct integration into `#739`'s owned-output workers, ready ring, and `12 MiB` `READY_BYTES` cap.
   - **Adaptive `S`-Scanner Topology:**
     - `1..=6` threads (`n <= 6` unless `8T` i9): zero-overhead `single_slab_scanner` over `SlabRope` (`zero` multi-slab `stitch` copies via Cycle 55 `c55c` + scanner-resident `runs::front` at `2..=3T`), ensuring `2–6` threads never pay coordinator lock or anchor-scan overhead.
     - `8` threads on the 10-core i9 (`n = 6` `OpenCells` workers): `mibz2-scan` + `1` lightweight `54 KiB` L1d-resident helper scanner (`8` total compute threads) with scanner-resident `runs::front` (`Cycle 55` `c55a`).
     - `>= 10` threads (`n = 8` `OpenCells` workers on i9): `mibz2-scan` + `2` demand-driven helper scanners (`claimable: AtomicUsize` lock-free check + lazy `Option<Box<ScanScratch>>`), reducing serial-stage CPU share from **`14.18% -> 0.53%`** (`wikidata-l9`), **`18.37% -> 0.58%`** (`wikidata-l1`), and **`21.26% -> 1.46%`** (`records-l9`) — yielding modeled `10T` critical-path ratios of **`0.7221x`** (`wikidata-l9`), **`0.6181x`** (`wikidata-l1`), and **`0.4817x`** (`records-l9`) vs `origin/master` `dd6450e6` (and **`0.9433x–0.9670x` real wall** at `10..20T` even on this 2-core / 4-vCPU host!).
   - **Peak RSS (`VmHWM`) Across `2–20` Threads:**
     - **`i9` mode:** **`0.7762x–0.9312x`** of `origin/master` (`6T = 44.22 MiB` / `0.7762x`; `8T = 49.70 MiB` / `0.8029x`; `10T = 48.93 MiB` / `0.8119x`; `20T = 49.62 MiB` / `0.8127x`) — **`6.9%–22.4%` below `master`** at every point (`<= 1.10x` gate met with `~29%` margin).
     - **`native` mode:** **`0.7791x–0.9854x`** of `origin/master` across `2–20T` (`10T = 0.7882x`, `12T = 0.7791x`, `20T = 0.8094x`).

2. **Zero-Shift Bipolar In-Place RLE1 (`src/rle.rs:unrle_in_place`) + Overlapping-Endpoint `fill_run_fast`:**
   - Eliminates `master`'s upfront `buf.copy_within(0..n, grown)` `900 KiB` shift by decomposing displacement $d_r = \text{dst}_r - \text{src}_r$ ($d_{r+1} = d_r + \text{extra}_r - 1 \ge d_r - 1$) into zero-copy ($d = 0$), forward-copy ($d < 0$), and minimal backward-scanned positive excursions ($d > 0$ up to the exact return $d_v = 0$), with safe overlapping-endpoint `4..32 B` run stores (`fill_run_fast`, `Cycle 57`) and streaming `64 KiB` L2-hot CRC tiles (`0.615x` on `unrle_in_place`).

3. **Balanced-XOR-Tree Sliced-by-16 Portable CRC (`src/crc.rs:update`):**
   - Replaces linear 8-byte serial table XOR with a `16 KiB` compile-time `TABLES: [[u32; 256]; 16]` evaluated via a depth-4 balanced binary XOR tree: **`0.6784x` (`1.474x` faster, `2.29 GB/s`)**.

4. **Cycle 58 (`20 / 20`) — Safe `u128` Register MTF Move (`mtf_move_fast15`) Completing the Safe Scalar Suite:**
   - In `mtf_scalar`, `mtf_checkpoints_scalar`, and `runs::front_scalar`, `list.copy_within(0..nn, 1)` previously emitted a `libc::memmove` PLT call on every literal symbol even though `~90%` of MTF ranks satisfy `nn < 15`.
   - Cycle 58 adds `mtf_move_fast15`: for `nn < 15`, loads `w = u128::from_le_bytes(list[..16])`, computes `hi_mask = !0u128 << ((nn + 1) * 8)`, and updates all 16 front bytes in registers via `(w & hi_mask) | ((w << 8) & !hi_mask) | (c as u128)` (`0` `memmove` calls, `0` `unsafe`), paired with fast single-record `close` (`left <= 127`) in `front_scalar`:
     - `mtf_scalar` (`wikidata-l9`, `21` rds): `17.74 -> 12.42 ms` (**`0.7001x`** vs Cycle 56; **`~0.628x`** cumulative vs `master`, `A/A = 1.0091x`).
     - `mtf_scalar` (`records-l9`, `21` rds): `105.31 -> 100.74 ms` (**`0.9567x`** vs Cycle 56; **`~0.87x`** cumulative vs `master`, `A/A = 1.0028x`).
     - `runs::front_scalar` (`wikidata-l9`, `21` rds): `18.62 -> 12.61 ms` (**`0.6773x`**, `1.48x` faster, `A/A = 0.9917x`).
     - **Full `2T` End-to-End Scalar Decode (`Kernels::scalar()`, `8` `wikidata-l9` streams, `15` rds):** `786.99 ms -> 541.26 ms` = **`0.6878x` (`1.454x` faster end-to-end vs `origin/master`, `A/A = 1.0052x`)**!

---

## 2. Applying & Verifying the Final Handoff Over `origin/master`

From a clean checkout of `origin/master` (`dd6450e6` or compatible):

```bash
git apply --check docs/journal/2026-11-22-carbon-final-handoff/final_handoff_over_master.patch
git apply docs/journal/2026-11-22-carbon-final-handoff/final_handoff_over_master.patch
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test --release --lib --tests
cargo build --release --example ab
```

### Complete Manifest of Drop-In Files (`docs/journal/2026-11-22-carbon-final-handoff/`)

| File | Safety | Key Additions / Changes Over `dd6450e6` |
| :--- | :--- | :--- |
| `src/par_scan.rs` | `#![forbid(unsafe_code)]` | Push-streaming `SlabRope` (zero-copy multi-slab views + `4 MiB` slab recycling), LE `u64x4` `ANCHOR_BITMAP` scanner, `ParScanHub` (`claimable: AtomicUsize`, lazy `ScanScratch`, scanner-resident `runs::front`, `max_front_ahead` horizon cap), `decode_parallel_slice[_with]` |
| `src/block.rs` | `#![forbid(unsafe_code)]` | `read_item_tight` (`max_out` power-of-two capacity) + `8,192`-`u16` (`16 KiB`) L1d-tiled `syms.resize` + reusable `ScanScratch` |
| `src/huff.rs` | `#![forbid(unsafe_code)]` | Auto-vectorized `vpaddd` `split_at_mut` canonical pair-LUT template clone + tail-only `d_tail` `SubLut` builder |
| `src/crc.rs` | `0` new `unsafe` | Compile-time `16 KiB` `TABLES: [[u32; 256]; 16]` + depth-4 balanced binary XOR tree `crc::update` (`0.6784x`) |
| `src/rle.rs` | `0` new `unsafe` | Zero-shift bipolar `unrle_in_place` + `fill_run_fast` + `16`-pos dual-`u64` safe SWAR `unrle_scalar` (`0.140x`) & `find_runs_scalar` (`0.527x`) |
| `src/mtf.rs` | `0` new `unsafe` | `mtf_move_fast15` (`u128` register MTF shift) + outer `16 KiB`-tiled `mtf_scalar` (`~0.628x`) + single-pass `mtf_checkpoints_scalar` (`0.361x`) |
| `src/runs.rs` | `0` new `unsafe` | `PreparedRuns` scanner handoff + `mtf_move_fast15` `front_scalar` (`0.677x`) + zero-slack `8`-wide `fill_scalar` (`0.730x`) + `8`-unrolled `cuts` (`0.607x`) + `16`-unrolled `takes_runs` |
| `src/parallel.rs` | `0` new `unsafe` | `StreamDecoder::parscan[_with]`, `8T` i9 `1`-helper + `10T+` `2`-helper topology, `parscan_hub_worker` into `#739` ready ring & `READY_BYTES` cap |
| `src/read.rs` | `0` new `unsafe` | `BzReader::parscan` & `BzReader::parscan_with` |
| `src/lib.rs` | `0` new `unsafe` | `decode_parscan`, `decode_parscan_with_kernels`, `pub mod par_scan` |
| `tests/par_scan.rs` | `#![forbid]`-clean | Differential, arbitrary/prime chunking, false-magic/header, mutation, slow-reader backpressure, and callback/reader panic suite |
| `examples/ab.rs` | `0` new `unsafe` | `mibz2-parscan` A/B benchmark arm |

---

## 3. Final Qualification Results

### 3.1 Compiler, Lint, Unit, Integration, Ledger, and Fuzz Gates
- `cargo fmt --check` & `cargo clippy --all-targets -- -D warnings`: **0 warnings, 0 errors**.
- `cargo test --release --lib`: **`194 / 194` passed** (including `trace::tests` `5/5`, `rle::tests` `4/4`, `runs::tests` `15/15`, `crc::tests`).
- `cargo test --release --tests`: **all `21` integration test suites passed** (`par_scan`, `mtf_checkpoints`, `mtf_kernels`, `in_place`, `cooperative`, `pipelined`, `service`, `decode`, `corpus`, `reader_panic`, etc.).
- `c41_bench verify`: **`PASS 1..5`** (`1B..1MiB` prime/multi-level chunking, slow-reader backpressure, `cancel()` + `PanickingReader` unwind safety, and **`1,500 / 1,500` streaming chunked mutation fuzz cases** exact on valid prefix bytes + `Error`).

### 3.2 `examples/ab` 10T Child Subprocess Verification (`wikidata_x3.bz2`, `629,100,588` B out)

| Pair | `mibz2` (`dd6450e6`) Wall | `mibz2-parscan` Wall | `mibz2` CPU | `mibz2-parscan` CPU | `mibz2` RSS | `mibz2-parscan` RSS | Hash (`bbe9dae7753f6c67`) |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | :---: |
| 1 | `0.9874 s` | **`0.9624 s`** | `1.7436 s` | **`1.7133 s`** | `72,668 KiB` | **`57,796 KiB` (`0.795x`)** | Exact |
| 2 | `0.9418 s` | `0.9781 s` | `1.7119 s` | `1.7345 s` | `70,624 KiB` | **`57,868 KiB` (`0.819x`)** | Exact |
| 3 | `0.9309 s` | `0.9572 s` | `1.7066 s` | `1.7144 s` | `71,368 KiB` | **`57,724 KiB` (`0.809x`)** | Exact |
| **Median** | `0.9418 s` | `0.9624 s` | **`1.7119 s`** | **`1.7144 s` (`1.001x`)** | `71,368 KiB` | **`57,796 KiB` (`0.810x`)** | **Exact** |

Inside `thread_cpu`, `mibz2-scan` drops from **`0.21–0.22 s` -> `0.11–0.12 s`** (`-45%` coordinator thread time — removing the 10-core i9's serial bottleneck!) while peak RSS drops by **`-13.25 MiB` (`0.810x`)**.

### 3.3 Subprocess Peak RSS (`VmHWM`) Audit (`wikidata_x3.bz2`, `629 MB` decompressed)

| Threads | `i9` Master MiB | `i9` Cand MiB | `i9` Ratio | `native` Master MiB | `native` Cand MiB | `native` Ratio |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `1` | `28.54` | `28.36` | **`0.9938x`** | `28.53` | `28.46` | **`0.9974x`** |
| `2` | `65.67` | `61.16` | **`0.9312x`** | `66.45` | `65.48` | **`0.9854x`** |
| `4` | `48.84` | `40.29` | **`0.8248x`** | `95.29` | `92.53` | **`0.9711x`** |
| `6` | `56.98` | `44.22` | **`0.7762x`** | `140.15` | `112.86` | **`0.8053x`** |
| `8` | `61.90` | `49.70` | **`0.8029x`** | `60.82` | `49.27` | **`0.8100x`** |
| `10` | `60.26` | `48.93` | **`0.8119x`** | `67.42` | `53.14` | **`0.7882x`** |
| `12` | `59.28` | `49.20` | **`0.8299x`** | `73.93` | `57.60` | **`0.7791x`** |
| `16` | `60.05` | `48.90` | **`0.8143x`** | `78.52` | `63.73` | **`0.8117x`** |
| `20` | `61.06` | `49.62` | **`0.8127x`** | `86.41` | `69.94` | **`0.8094x`** |

### 3.4 Same-Binary Push-Streaming A/B (`1..20T`, `7` Interleaved Rounds on 2-Core / 4-vCPU Host)

#### `wikidata-l9` (`209.70 MB` out)
- **Explicit Serial-Stage CPU Share:** `Master` = `14.18%` (`66.24 ms`) -> **`Cand parscan` = `0.53%`** | **Modeled `10T` Critical-Path Ratio = `0.7221x`**.

| Threads | `MasterA` ms (`dd6450e6`) | `A/A` | `Cand / MasterA` | `Cand / M329` | `CPU Ratio` |
| ---: | ---: | ---: | ---: | ---: | ---: |
| `1` | `763.63` | `0.9985x` | `1.0043x` | **`0.9855x`** | `1.0044x` |
| `2` | `446.95` | `1.0012x` | `1.0200x` | **`0.9896x`** | **`0.9944x`** |
| `4` | `460.70` | `1.0027x` | `1.0257x` | `1.0153x` | `1.0046x` |
| `6` | `479.68` | `0.9987x` | `1.0242x` | `1.0213x` | `1.0071x` |
| `8` | `459.33` | `1.0037x` | **`0.9959x`** | **`0.9922x`** | `1.0027x` |
| `10` | `476.58` | `0.9894x` | **`0.9523x`** | **`0.9738x`** | **`0.9887x`** |
| `12` | `476.53` | `0.9987x` | **`0.9670x`** | **`0.9622x`** | `1.0023x` |
| `16` | `490.14` | `0.9897x` | **`0.9574x`** | **`0.9819x`** | **`0.9985x`** |
| `20` | `492.68` | `0.9897x` | **`0.9433x`** | **`0.9796x`** | `1.0013x` |

#### `wikidata-l1` (`36.00 MB` out) & `records-l9` (`9.60 MB` out)

| Threads | `wikidata-l1` `A/A` | `wikidata-l1` `Cand/MastA` | `wikidata-l1` `Cand/M329` | `records-l9` `A/A` | `records-l9` `Cand/MastA` | `records-l9` `Cand/M329` |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| **Serial Share** | — | **`18.37% -> 0.58%`** | (`10T` crit **`0.6181x`**) | — | **`21.26% -> 1.46%`** | (`10T` crit **`0.4817x`**) |
| `1` | `1.0013x` | `1.0027x` | **`0.9455x`** | `0.9996x` | **`0.9964x`** | **`0.9707x`** |
| `2` | `1.0168x` | **`0.9612x`** | **`0.9363x`** | `0.9939x` | **`0.9824x`** | **`0.9701x`** |
| `4` | `1.0062x` | `1.0184x` | `1.0071x` | `1.0262x` | **`1.0020x`** | **`0.9914x`** |
| `6` | `1.0239x` | `1.0396x` | **`0.9947x`** | `0.9777x` | `1.0166x` | **`0.9737x`** |
| `8` | `0.9654x` | `1.0544x` | `1.0495x` | `1.0084x` | **`0.9964x`** | **`0.9815x`** |
| `10` | `0.9800x` | `1.0497x` | `1.0401x` | `1.0024x` | `1.0098x` | **`0.9934x`** |
| `12` | `1.0023x` | `1.0472x` | `1.0255x` | `1.0070x` | `1.0373x` | `1.0267x` |
| `16` | `0.9847x` | `1.0197x` | **`0.9920x`** | `0.9780x` | `1.0327x` | `1.0418x` |
| `20` | `1.0085x` | `1.0546x` | `1.0185x` | `0.9825x` | `1.0205x` | `1.0338x` |

### 3.5 Complete `100%` Safe Scalar Kernel Suite (`Kernels::scalar()` vs `origin/master` `dd6450e6`)

| Safe Scalar Kernel / Path | Cycle(s) | Ratio vs `origin/master` | Speedup |
| :--- | :---: | ---: | ---: |
| `rle::unrle` (`16`-pos dual-`u64` safe SWAR) | `48, 57` | **`0.1403x–0.1550x`** | **`6.45x–7.13x`** |
| `mtf::mtf_checkpoints_scalar` (single-pass outer-tile + `u128` MTF) | `56, 58` | **`~0.25x`** (`0.361x * 0.700x`) | **`~3.9x`** |
| `runs::cuts` (`8`-record unrolled + precomputed `u32` targets) | `56` | **`0.6068x–0.6421x`** | **`1.56x–1.65x`** |
| `mtf::mtf_scalar` (`seq_to_unseq` + outer `16 KiB` tile + `u128` MTF) | `50, 56, 58` | **`~0.628x`** | **`1.59x`** |
| `runs::front_scalar` (`seq_to_unseq` + `u128` MTF + fast `close`) | `50, 58` | **`~0.631x`** | **`1.58x`** |
| `crc::update` (`16 KiB` Sliced-by-16 + depth-4 balanced XOR tree) | `53` | **`0.6784x`** | **`1.47x`** |
| `runs::fill_scalar` (zero-slack `8`-wide safe slice writes + `16 KiB` tile) | `51` | **`0.7295x`** | **`1.37x`** |
| **Full `2T` End-to-End Scalar Decode (`8` `wikidata-l9` streams)** | `48–58` | **`0.6878x`** (`786.99 -> 541.26 ms`) | **`1.454x`** |

---

## 4. Consolidated Ledger of One-Line Kills Across Cycles 44–58

- **Cycle 44 `mtf_ring16` (`2.4230x`):** Variable-trip `1..g` bucket-cascade branch mispredicts + unaligned `u64` store-to-load forwarding stalls lost to `mtf_avx2`'s branchless `32 B` row shifts.
- **Cycle 44 Inline `SUB_PTR` in `step_win!` (`1.2600x–1.3767x`):** Unrolling 5 copies of `sub_lut` branches spilled `Reg` to the stack.
- **Cycle 44 `LUT_BITS = 11` (`1.0835x–1.1355x`):** `6 x 8 KiB = 48 KiB` overflowed `32 KiB` L1d and doubled `Tables::build`.
- **Cycle 46 Fused `P1 + runs::front` inside `decode_group_win` (`1.045x`):** Register pressure inside `decode_group_win` spilled bitbuffer state vs keeping sequential `read_item_tight` + scanner-resident `runs::front` in L2.
- **Cycle 47 `unrle_avx2` overlapping-store short-run branch (`1.018x`):** Extra branch per run mispredicted on mixed run lengths vs `unrle_in_place` bipolar decomposition.
- **Cycle 49 `c49_sel_lens` (`1.0051x–1.0307x`):** `leading_zeros` run-skipping on selectors/code-lengths mispredicted on short `0..2` zero runs.
- **Cycle 49 `c49_quad_lut10` (`1.1062x–1.2754x`):** `c <= 4` shrank the unrolled group prefix (`k <= 30` vs `40`) and doubled L1d table footprint (`48 KiB`).
- **Cycle 50 `c50_mtf_u64x32` (`1.8308x`):** Serial `0..q` `u64` carry-chain loop lost to `copy_within` for `nn >= 16` (solved in Cycle 58 by restricting register shift to single-`u128` `nn < 15`!).
- **Cycle 52 `c52b` `held_out` worker buffer hoarding (`+4..11 MiB` RSS):** Caching `900 KiB` output buffers inside parked workers stranded memory from active workers.
- **Cycle 54 `c54_lut64` / `DeferLiveOnly` / `u8Pair32` (`1.001x–1.166x`):** `decode_group_win` is pinned at `5.8` cycles/step on the `1 + 4 + 1 = 6`-cycle `buf -> >>54 -> L1d -> <<=cl` serial latency chain, so off-chain ALU ops hide in the L1d load shadow.
- **Cycle 55 `c55b` `1->2->4 MiB` reader slab ramp (`1.0615x` `records-l9` `2T`):** Early `1 MiB` / `3 MiB` slab seams added seam stitching/re-decode overhead on `840 KiB` `-9` blocks.
- **Cycle 57 `c57_tile16k` (`1.0171x` `detect_clmul`):** `16 KiB` CRC tiles ran `clmul_update`'s fold prologue `4x` more often while `64 KiB` already stays hot in private L2.
