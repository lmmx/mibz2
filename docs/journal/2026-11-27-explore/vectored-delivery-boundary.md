# Vectored Delivery Boundary (`Open` -> `parallel.rs` -> `BzReader` / `Read` / Python / `decode_parallel`)

**Date:** 2026-11-27
**Scope:** Smooth-zone architecture and benchmark probe (`Claim: none`, zero `src/` edits, `#![forbid(unsafe_code)]`)
**Reproducer:** [`examples/vectored_delivery_probe.rs`](../../../examples/vectored_delivery_probe.rs)
**Related:** `docs/journal/2026-11-27-explore/forward-rle1-crc-fusion.md` (#1045), `docs/journal/2026-11-27-synthesis/seventh-pass.md` (#1052), `frontier/claims/forward-rle1-crc-vectored-build.toml`, `frontier/claims/forward-4lane-anchor-probe.toml`, `frontier/claims/forward-emission.toml`

---

## 1. Motivation

`forward-rle1-crc-fusion` (#1045) proved that 4-lane chunked forward emission with zero-scratch in-place RLE1 compaction and tiled PCLMULQDQ CRC32 saves `+49.9 ms` (`+3.82%` 1W warm) / `+31.8 ms` (`+2.43%` cold) on `wikidata x4` and cuts logical `E+A9` write traffic `-53.4%` (`1,718 -> 800 MiB`), **only if** the 4 lanes are delivered as vectored slices rather than stitched into a single contiguous `Vec<u8>` (stitched 4-lane failed due to `74.9%` `memmove` = `598.9 MiB`). Simultaneously, `forward-4lane-anchor-probe` explores `K=4..16` sampled-anchor segments.

Before `forward-emission` and `forward-rle1-crc-vectored-build` touch `src/parallel.rs` and `src/read.rs`, four concrete boundary questions needed primary evidence:

1. **Delivery overhead vs stitch penalty:** Does delivering `K=4` or `S=16` disjoint slices inside one recycled `Vec<u8>` slab add measurable overhead to `BufRead::fill_buf`, `Read::read` (64 KiB / 256 KiB), or `decode_parallel`, compared to `contiguous_1slice` and `stitched_4lane`?
2. **Slab sizing & RSS budget (`<= 1.05x`):** What per-segment slab sizing avoids both worst-case RLE1 over-allocation (`4 x 900 KiB` or `span/4`) and stitch copies while staying within `<= 1.05x` shipped RSS (`n + n/16 + 64` in `src/parallel.rs:1209`)?
3. **Python `#1033` `Reader::read(size)` impact:** `#1033` reads Python chunks via a single `BufRead::fill_buf()` call. How much does exposing per-lane/per-segment slices multiply Python `read(size)` crossings for `K=4` vs `S=16`?
4. **Exact safe-Rust API & touchpoints:** What exact `ReadyBlock` representation preserves zero extra heap allocations, existing `free_out` slab recycling, `Ready::bytes` backpressure accounting, zero `unsafe`, and public API compatibility?

---

## 2. Experimental Setup

- **Corpus:** `/tmp/wikidata.bz2` (431 real blocks, `209,700,196` decoded bytes = `199.99 MiB`, SHA-256 verified) plus 4 synthetic RLE1-expansion stress blocks (`text_900k`, `runs_900k`, `onebyte_900k_out`, `onebyte_5.18M_out`).
- **Four delivery layouts** (exact output hash parity verified across all four: `0x80f2395ca32b5fc4`):
  1. `contiguous_1slice` — shipped single contiguous `out[0..len]`.
  2. `vectored_4lane` — `K=4` equal-stride non-empty lanes packed at fixed slab offsets inside one recycled `Vec<u8>`.
  3. `vectored_16seg` — `S=16` sampled-anchor order-statistic segments (deterministic `XorShift64`, RLE1-run-safe cuts) packed inside one recycled `Vec<u8>`.
  4. `stitched_4lane` — `K=4` lanes compacted in place via `copy_within` before delivery (the `#1045` stitched failure mode).
- **Six consumer patterns:**
  - `fill_buf_hash` — zero-copy `BufRead::fill_buf` + `consume` hashing all decoded bytes (8-byte carry across slice boundaries).
  - `read_64k_single` / `read_256k_single` — single-slice `Read::read` (one `fill_buf` slice per call, mirroring `BzReader::read` and Python `#1033` `Reader::read(size)`).
  - `read_64k_scatter` / `read_256k_scatter` — scatter-fill `Read::read` draining across segments inside the current `ReadyBlock` to fill the caller's buffer.
  - `decode_parallel` — `out.extend_from_slice(s)` per segment into a pre-sized output `Vec<u8>`.
- **Protocol:** Both baseline x86-64 (`cargo build --release`) and `-C target-cpu=native`, 7 interleaved repeats (`--repeats 7`), both **warm** (L2/L3-resident slabs) and **cold** (64 MiB LLC flush between blocks) regimes, reporting steal-free Linux thread-CPU ms (`/proc/thread-self/schedstat` field 0) scaled to the full 431-block `199.99 MiB` wikidata stream.

---

## 3. Slab Sizing & RLE1 Expansion Census

### 3.1 Wikidata (431 real blocks)

- Pre-RLE1 total: `199.99 MiB`; decoded total: `199.99 MiB` (`1.0000x`).
- Shipped `src/parallel.rs:1209` capacity formula `n + n/16 + 64` (`+6.25% + 64 B`): sum = `212.52 MiB`, max block capacity = `933.9 KiB`.

**`K = 4` equal-stride lanes (`1,130` non-empty segments across 431 blocks):**
- Segment expansion `post/pre`: `p50 = 1.0000`, `p95 = 1.0001`, `p99 = 1.0003`, **`max = 1.0008`**.
- Segment length: `p10 = 0.0 KiB`, `p50 = 219.7 KiB`, `p90 = 219.7 KiB`.

| Per-segment slab formula | Overflows (segs / blks) | Total slab MiB | vs shipped | Max block KiB | vs shipped max |
|---|---:|---:|---:|---:|---:|
| `+0% + 16 B` | `63` / `46` | `200.01` | `0.9412x` | `879.0` | `0.9412x` |
| `+3.125% + 64 B` (`n/32`) | **`0` / `0`** | `206.31` | `0.9708x` | `906.6` | `0.9708x` |
| `+5.0% + 64 B` (`n/20`) | **`0` / `0`** | `210.06` | `0.9884x` | `923.1` | `0.9884x` |
| **`+6.25% + 64 B` (`n/16`, shipped)** | **`0` / `0`** | **`212.56`** | **`1.0002x`** | **`934.1`** | **`1.0002x`** |
| `+10.0% + 256 B` (`n/10`) | `0` / `0` | `220.27` | `1.0365x` | `967.8` | `1.0363x` |
| `+15.0% + 256 B` | `0` / `0` | `230.27` | `1.0835x` | `1011.8` | `1.0834x` |
| `+25.0% + 256 B` (`span/4`) | `0` / `0` | `250.27` | **`1.1776x`** | `1099.6` | **`1.1774x`** |

**`S = 16` sampled-anchor order-statistic segments (`3,926` non-empty segments across 431 blocks):**
- Segment expansion `post/pre`: `p50 = 1.0000`, `p95 = 1.0000`, `p99 = 1.0004`, **`max = 1.0040`**.
- Segment length: `p10 = 2.9 KiB`, `p50 = 36.1 KiB`, `p90 = 124.1 KiB`.

| Per-segment slab formula | Overflows (segs / blks) | Total slab MiB | vs shipped | Max block KiB | vs shipped max |
|---|---:|---:|---:|---:|---:|
| `+0% + 16 B` | `81` / `47` | `200.06` | `0.9414x` | `879.2` | `0.9414x` |
| `+3.125% + 64 B` (`n/32`) | **`0` / `0`** | `206.48` | `0.9716x` | `907.4` | `0.9716x` |
| **`+6.25% + 64 B` (`n/16`, shipped)** | **`0` / `0`** | **`212.73`** | **`1.0010x`** | **`934.8`** | **`1.0010x`** |
| `+10.0% + 256 B` (`n/10`) | `0` / `0` | `220.95` | `1.0397x` | `970.9` | `1.0396x` |
| `+25.0% + 256 B` (`span/4`) | `0` / `0` | `250.95` | **`1.1808x`** | `1102.6` | **`1.1806x`** |

### 3.2 Synthetic RLE1-Expansion Guards

| Block | Pre-RLE1 | Decoded | Expansion | `K=4` (`+6.25%` / `+25%`) | `S=16` (`+6.25%` / `+25%`) |
|---|---:|---:|---:|---|---|
| `text_900k` | `900,108` | `900,000` | `1.00x` | `0/4` ovf / `0/4` ovf | `0/16` ovf / `0/16` ovf |
| `runs_900k` | `435,278` | `900,000` | `2.07x` | `4/4` ovf / `4/4` ovf | `16/16` ovf / `16/16` ovf |
| `onebyte_900k_out` | `17,375` | `900,000` | `51.80x` | `4/4` ovf / `4/4` ovf | `16/16` ovf / `16/16` ovf |
| `onebyte_5.18M_out` | `100,000` | `5,180,000` | `51.80x` | `4/4` ovf / `4/4` ovf | `16/16` ovf / `16/16` ovf |

### 3.3 Buffer-Sizing Conclusion (Critical for `forward-rle1-crc-vectored-build`)

1. **No fixed slab multiplier avoids adversarial RLE1 expansion (`2.07x` on `runs`, `51.8x` on `onebyte`).** Over-allocating `span/4` (`+25%`) buys **zero** overflow protection on `runs`/`onebyte`, while pushing per-block RSS to **`1.178x` shipped** (`1,100 KiB`), which **violates** the `<= 1.05x RSS` gate in `forward-rle1-crc-vectored-build.toml` and increases per-worker LLC footprint (`k * W_worker`).
2. **Shipped `pre_seg + pre_seg/16 + 64` (`+6.25% + 64 B`) is the exact sweet spot:** `0/431` overflows on `wikidata`, `0/1` on `text_900k`, **`1.0002x` shipped RSS** for `K=4` (`1.0010x` for `S=16`).
3. **Zero-scratch in-place tile headroom:** For `#1045`'s chunked in-place tile compaction (`CHUNK_STEPS = 4096`), each lane only needs `pre_seg + pre_seg/16 + CHUNK_STEPS + 64` (adds `4 KiB * 4 = 16 KiB` = `+1.7%` per 900 KB block -> **`1.017x` shipped**, comfortably under `<= 1.05x`), with **suffix-only scratch spill** (or clean fallback to shipped single-lane `unrle`) on the rare segment that exceeds its slab (`0%` on wikidata/text; `100%` clean fallback on `runs`/`onebyte`).

---

## 4. Python `#1033` & `Read::read` Crossing Census

Call counts to consume all `209,700,196` bytes (`431` wikidata blocks):

| Layout | single-slice 8K | single-slice 64K | single-slice 256K | single-slice 1M | scatter-fill 64K | scatter-fill 256K |
|---|---:|---:|---:|---:|---:|---:|
| `contiguous_1slice` | `25,828` | `3,460` | `1,130` | `431` | `3,200` | `800` |
| `vectored_4lane` | `26,294` (`+1.8%`) | `3,926` (`+13.5%`) | **`1,130` (`+0.0%`)** | `1,130` (`2.62x`) | `3,200` | `800` |
| `vectored_16seg` | `27,715` (`+7.3%`) | **`5,638` (`+62.9%`)** | **`3,947` (`3.49x`)** | **`3,926` (`9.11x`)** | `3,200` | `800` |

**Takeaway for `src/python.rs` (#1033):**
- **At `K=4`:** `#1033`'s single-`fill_buf()` path is essentially unaffected for common Python read sizes: `read(8K)` rises `+1.8%`, `read(64K)` rises `+13.5%`, and `read(256K)` is **literally unchanged (`+0.0%`, 1,130 calls)** because a 4-lane slice (`~219.7 KiB`) fits inside 256 KiB just as a `879 KiB` block splits into 4 `256 KiB` reads. Only `read(1M+)` sees more crossings (`2.62x`).
- **At `S=16`:** Single-slice `fill_buf()` inflates `read(64K)` calls `+62.9%`, `read(256K)` `3.49x`, and `read(1M)` `9.11x`. If `S > 4` is ever shipped, `BzReader::read` and Python `Reader::read(size)` (via `PyBytes::new_with` scatter loop across segments in the current block) keep call counts flat at `3,200` / `800` with zero extra allocation.

---

## 5. Delivery Benchmark Results (7 Interleaved Repeats, Thread-CPU ms / 200 MiB Wikidata)

### 5.1 Native (`-C target-cpu=native`)

| Consumer | `contiguous_1slice` warm / cold | `vectored_4lane` warm / cold | `vectored_16seg` warm / cold | `stitched_4lane` warm / cold |
|---|---:|---:|---:|---:|
| `fill_buf_hash` | `43.68` / `47.94` | `43.69` (`1.000x`) / `44.00` (`0.918x`) | `43.69` (`1.000x`) / `45.01` (`0.939x`) | **`58.28` (`1.334x`) / `63.88` (`1.333x`)** |
| `read_64k_single` | `21.48` / `23.82` | `14.57` / `24.00` (`1.008x`) | `14.52` / `23.32` (`0.979x`) | **`36.36` (`1.693x`) / `42.05` (`1.766x`)** |
| `read_64k_scatter` | `14.57` / `23.98` | `14.51` / `23.77` (`0.991x`) | `14.57` / `22.92` (`0.956x`) | **`28.24` (`1.938x`) / `39.97` (`1.666x`)** |
| `read_256k_single` | `21.77` / `25.46` | `21.65` / `27.95` (`1.098x`) | `21.84` / `24.00` (`0.942x`) | **`43.68` (`2.007x`) / `43.99` (`1.728x`)** |
| `read_256k_scatter` | `14.57` / `27.70` | `14.58` / `23.99` (`0.866x`) | `14.57` / `23.88` (`0.862x`) | **`29.05` (`1.994x`) / `43.97` (`1.588x`)** |
| `decode_parallel` | `29.12` / `47.87` | `36.28` / `47.92` (`1.001x`) | `29.13` / `47.89` (`1.000x`) | **`50.97` (`1.750x`) / `63.94` (`1.336x`)** |

### 5.2 Baseline x86-64

| Consumer | `contiguous_1slice` warm / cold | `vectored_4lane` warm / cold | `vectored_16seg` warm / cold | `stitched_4lane` warm / cold |
|---|---:|---:|---:|---:|
| `fill_buf_hash` | `43.51` / `47.46` | `43.30` (`0.995x`) / `43.99` (`0.927x`) | `43.69` (`1.004x`) / `43.91` (`0.925x`) | **`57.56` (`1.323x`) / `63.79` (`1.344x`)** |
| `read_64k_single` | `14.57` / `25.39` | `14.61` / `26.49` (`1.043x`) | `20.90` / `23.79` (`0.937x`) | **`32.93` (`2.261x`) / `43.91` (`1.729x`)** |
| `read_64k_scatter` | `14.49` / `23.97` | `14.57` / `23.31` (`0.973x`) | `14.57` / `20.00` (`0.834x`) | **`29.14` (`2.011x`) / `39.74` (`1.658x`)** |
| `read_256k_single` | `14.57` / `24.00` | `21.56` / `24.00` (`1.000x`) | `14.57` / `23.87` (`0.995x`) | **`36.21` (`2.486x`) / `43.28` (`1.804x`)** |
| `read_256k_scatter` | `14.63` / `25.79` | `15.74` / `23.97` (`0.929x`) | `14.48` / `23.43` (`0.908x`) | **`36.40` (`2.488x`) / `44.25` (`1.716x`)** |
| `decode_parallel` | `36.35` / `45.74` | `36.44` (`1.003x`) / `47.81` (`1.045x`) | `36.33` (`0.999x`) / `47.51` (`1.039x`) | **`58.13` (`1.599x`) / `63.80` (`1.395x`)** |

**Key measurement conclusion:**
- **Vectored delivery (`K=4` AND `S=16`) is zero-cost:** across all 6 consumers and both builds, `vectored_4lane` and `vectored_16seg` match `contiguous_1slice` within timer quantization/noise (`~0.86x..1.05x` cold; `fill_buf_hash` `0.92x..1.00x`).
- **Stitching costs `+16..+22 ms` per 200 MiB** (`+33%..+80%` slower delivery, `~+64..+88 ms` on `wikidata x4`!), cleanly reproducing `#1045`'s `598.9 MiB` `memmove` penalty.

---

## 6. Proposed Safe-Rust `ReadyBlock` Interface & Exact Touchpoints

### 6.1 `ReadyBlock` (`src/parallel.rs`, zero `unsafe`, zero extra heap allocations)

```rust
pub const MAX_BLOCK_SLICES: usize = 16;

/// A decoded block stored as 0..=MAX_BLOCK_SLICES non-empty `(start, end)` slices inside a single
/// recycled `Vec<u8>` slab. Contiguous blocks use `count = 1, ranges[0] = (0, len)`.
pub struct ReadyBlock {
    buf: Vec<u8>,
    ranges: [(u32, u32); MAX_BLOCK_SLICES],
    count: u8,
    total_len: u32,
}
```

- **Inline size:** `24 (Vec) + 128 ([(u32, u32); 16]) + 1 + 3 (pad) + 4 = 160 bytes` (no `Vec<Range>` heap allocation; fits cleanly inside `VecDeque<Ready>` slot storage).
- **Invariants:** Every active `ranges[i]` satisfies `start < end <= buf.len()` (empty segments omitted at construction); `total_len == sum(end - start)`.
- **Recycling (`free_out`):** `pub fn into_buf(self) -> Vec<u8> { self.buf }` drops directly back into `State::free_out` with zero changes to slab pooling (`decode_owned` / `take_out` / `recycle`).
- **Backpressure (`Ready::size`):** `Ready::size` uses `blk.total_len()` instead of `v.len()` so `Ready::bytes` tracks exact logical bytes (preventing over-accounting for inter-lane slack gaps).
- **Compatibility helper (`into_contiguous`):** For `#[doc(hidden)] pub fn read_block(&self, spent: Vec<u8>) -> Result<Option<Vec<u8>>>` (used by `examples/bzcat.rs` and any legacy single-`Vec` caller), a fallback `into_contiguous()` compacts via `copy_within` only when `count > 1`, while new `read_block_vectored(&self, spent: Vec<u8>) -> Result<Option<ReadyBlock>>` exposes zero-stitch vectored delivery to `decode_parallel`, `BzReader`, and `examples/bzcat.rs`.

### 6.2 Exact File Touchpoints (for `forward-emission` / `forward-rle1-crc-vectored-build`)

1. **`src/parallel.rs`:**
   - Replace `Ready::Block(Vec<u8>)` with `Ready::Block(ReadyBlock)` (`L99-L183`).
   - Update `Ready::size` (`L121`) to `b.total_len()`.
   - Update `Reader` state (`L283-L290`) from `(cur: Vec<u8>, at: usize)` to `(cur: ReadyBlock, seg: usize, at: usize)`.
   - Update `StreamDecoder::read` (`L841-L875`) to scatter-fill `dst` across `cur.slice(seg)` segments within the current `ReadyBlock` before blocking on `advance()` (holds `read(64K)` / `read(256K)` call counts flat even at `S=16`).
   - Add `read_block_vectored(&self, spent: Vec<u8>) -> Result<Option<ReadyBlock>>` (`L880-L915`); keep `read_block` delegating via `into_contiguous()` for backwards compatibility.
   - Update `recycle` / `advance` (`L958-L1009`) to return `cur.into_buf()` to `free_out`.
   - Keep `decode_owned` pool sizing (`L1209`) at `n + n/16 + 64` (plus `+ 4096 * K` only when fused tile compaction is active, `<= 1.017x` shipped).
2. **`src/read.rs`:**
   - Update `Par` (`L57-L64`) from `(buf: Vec<u8>, at: usize)` to `(blk: ReadyBlock, seg: usize, at: usize)`.
   - `BufRead::fill_buf` (`L345-L367`): loop advancing `seg` past exhausted slices, calling `read_block_vectored(blk.into_buf())` only when `seg >= blk.slice_count()`, returning `&blk.slice(seg)[at..]`. Zero copies, exact `BufRead` contract.
   - `Read::read` (`L321-L343`): either delegate to `fill_buf` (already zero overhead at `K=4`) or scatter-copy across remaining slices of `blk` (recommended if `S > 4`).
3. **`src/lib.rs`:**
   - `decode_parallel` (`L879-L915`): switch from `read_block` to `read_block_vectored` and drain `for s in blk.slices() { out.extend_from_slice(s); }`, recycling `blk.into_buf()`. Zero stitch copy.
4. **`src/python.rs`:**
   - At `K=4`, **zero changes required** (`#1033` `fill_buf` has `+0.0%` extra crossings at `256K` and `+1.8%` at `8K`).
   - If `S=16` is ever enabled, switch `Reader::read(size)` from single `fill_buf()` to a scatter loop inside `PyBytes::new_with` across `cur` slices so `read(64K/256K/1M)` stays at 1 Python allocation per call.
5. **`examples/bzcat.rs`:**
   - Switch `dec.read_block(spent)` to `dec.read_block_vectored(spent)` and write `for s in b.slices() { out.write_all(s)?; }` (or `write_vectored` with `[IoSlice; 16]`), recycling `b.into_buf()`.

---

## 7. Caveats

- Timings were collected on a 2-vCPU VM (single-thread steal-free thread CPU via `/proc/thread-self/schedstat`, C5 diagnostic; `~4 ms` scheduling quantization floor visible in single-thread 200 MiB scans). Cold/warm relative ratios (`vectored` = `~0.92x..1.05x` contiguous vs `stitched` = `1.33x..2.49x`) are clear and reproducible; multi-worker 8W/10W L3/DRAM effects will be measured on the i9-10900K in `forward-rle1-crc-vectored-build`.
