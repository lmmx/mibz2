# Delivery copy-out characterisation and design options (`delivery-copyout`)

*Smooth-zone probe (`frontier/claims/delivery-copyout.toml`, 26 November 2026). Reproducible zero-`unsafe` census in `examples/delivery_copyout_census.rs`; zero `src/` edits.*

---

## 1. Premise check: the "48 ms flat" floor was a cold-store artifact

Before spending on API changes, re-reading primary evidence from `#952` (`docs/journal/2026-11-26-explore/cold-store-and-rle1-llc.md`) and `#954` (`docs/journal/2026-11-26-synthesis/README.md`) against `origin/evidence` (`cc8ba1374f32`, SHA-256 `10df0d89…5e34`; `c71c4a56c145`, SHA-256 `97e397ad…8867`) falsifies two premises in the original `delivery-copyout` claim:

1. **Real delivery copy-out on the i9-10900K is NOT flat at 48 ms.**
   In `cc8ba1374f32`, `SFCMR − SFCMRD = 47.3–49.3 ms` at every worker count because the `R` oracle held 431 decoded blocks in a 209.7 MiB global store (10.5× the i9's 20 MiB L3), forcing `SFCMR` to read **853.5 MiB** cold from DRAM at ~17 GB/s (vs **151.1 MiB** total DRAM reads for the entire shipped `none` decode at `k = 1`).
   Direct inspection of `c71c4a56c145`'s traced `runs.jsonl` (`n = 3` per cell, `wikidata ×4` = 838.80 MB, 1,724 blocks, `examples/ab.rs` 4 MiB caller buffer + 4-lane `Hash`) shows the real i9 delivery curve:

   | arm | `k` | wall (ms) | `copy_out` median `[min..max]` (ms) | `A10` (ms) | `read_wait` (ms) | `ready_bytes` hwm | `ready_bytes` avg | `ready` hwm | `A9` CPU (ms) | eff. BW |
   |---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
   | `none` | 1 | 1299.2 | **29.1** `[27.1..30.8]` | 2.9 | 1265.2 | 0.9 MiB | 0.0 MiB | 3 | 74.4 | 28.8 GB/s |
   | `none` | 6 | 243.6 | **27.6** `[27.5..31.3]` | 1.2 | 212.0 | 5.1 MiB | 0.3 MiB | 16 | 89.5 | 30.4 GB/s |
   | `none` | 7 | 212.0 | **30.6** `[29.7..31.8]` | 1.2 | 178.9 | 5.2 MiB | 0.4 MiB | 17 | 95.3 | 27.5 GB/s |
   | `none` | 8 | 193.7 | **34.0** `[33.0..35.3]` | 1.4 | 157.2 | 6.9 MiB | 0.6 MiB | 23 | 104.0 | 24.7 GB/s |
   | `none` | 9 | 198.8 | **43.2** `[35.8..43.9]` | 1.3 | 153.9 | 12.0 MiB | 1.8 MiB | 32 | 126.5 | 19.4 GB/s |
   | `none` | 10 | 214.3 | **43.7** `[42.7..44.1]` | 1.2 | 168.5 | 14.6 MiB | 1.9 MiB | 36 | 133.0 | 19.2 GB/s |
   | `PC` | 8 | 160.7 | **40.0** `[38.1..40.7]` | 1.9 | 118.6 | 7.7 MiB | 0.9 MiB | 20 | 118.6 | 21.0 GB/s |
   | `PC` | 10 | 149.8 | **67.4** `[65.5..68.2]` | 1.8 | 80.1 | 12.0 MiB | 2.4 MiB | 34 | 191.2 | 12.5 GB/s |
   | `SC` | 8 | 133.3 | **39.5** `[39.2..40.7]` | 1.8 | 91.6 | 8.6 MiB | 1.0 MiB | 26 | 119.7 | 21.2 GB/s |
   | `SC` | 9 | 131.2 | **54.9** `[52.4..60.5]` | 1.9 | 75.6 | 10.3 MiB | 1.9 MiB | 31 | 152.3 | 15.3 GB/s |
   | `SC` | 10 | 148.2 | **77.9** `[69.8..81.0]` | 1.8 | 65.0 | 16.3 MiB | 4.3 MiB | 37 | 215.6 | 10.8 GB/s |

   Two immediate consequences follow from this table:
   - **L3-hot serial delivery is 27.6–34.0 ms (`25–30 GB/s`), not 48 ms.** At `k = 9..10`, `READY_BYTES = 12 MiB` (`src/parallel.rs:74`) and `SLOTS_PER_THREAD = 4` (`src/parallel.rs:47`) let 12.0–16.3 MiB of unread blocks occupy 60–82% of the 20 MiB L3, evicting ready blocks and worker RLE1 buffers to DRAM simultaneously (`copy_out` 27.6 → 77.9 ms; `A9` 89.5 → 215.6 ms).
   - **Delivery is NOT yet on the serial critical path today** (`read_wait = 65.0–168.5 ms` even in `SC@10W`). Today's wall-time cost of delivery at `k = 9..10` is **indirect**: L3 capacity eviction coupled with `R` (`A9`) plus reader CPU contention on SMT siblings (`77.9 ms` of reader core time). Delivery becomes the direct serial wall floor only once scan and workers drop below ~30–40 ms.

2. **Zero-copy block lending ALREADY exists in the crate — publicly on `BzReader` and `#[doc(hidden)] pub` on `StreamDecoder`.**
   - `BzReader<R>` already implements public **`std::io::BufRead`** (`src/read.rs:339–367`), lending the decoded block `&[u8]` directly from `fill_buf()` with zero copy.
   - `StreamDecoder::read_block(&self, spent: Vec<u8>) -> Result<Option<Vec<u8>>, Error>` (`src/parallel.rs:880–915`, `#[doc(hidden)] pub`) already implements **both** zero-copy block lending AND caller-buffer pool recycling (`spent` returns to `State::free_out` when `cap >= POOLED_FROM = 64 KiB`), used internally by `BzReader::fill_buf`, `decode_parallel` (`src/lib.rs:813`), and `examples/bzcat.rs:49`.
   - `mibz2::scan` + `mibz2::decode_blocks` (`src/read.rs:513`, `src/lib.rs:876`) already expose caller-dispatched parallel decoding into caller-owned `Vec<u8>` buffers.

---

## 2. Measurement environment, corpus, and exactness

- **VM hardware:** 2-vCPU Intel Xeon @ 2.30 GHz (AVX2 + BMI1 + PCLMULQDQ live), 32 KiB L1d, 256 KiB L2, **45 MiB L3** (2.25× the i9-10900K's 20 MiB L3; caveat noted explicitly in every projection below).
- **Corpus:** `corpora/wikidata.bz2` (11,854,933 B compressed; 431 blocks; **209,700,196 B** = 199.99 MiB decoded; `wikidata ×4` = 1,724 blocks, **838,800,784 B** = 838.80 MB = 799.94 MiB).
- **Exactness:** Every isolated kernel and every end-to-end arm was verified byte-for-byte against `mibz2::decode` (`SHA-256 = 3f075e30f4f8c73414150655afbf57caa2647c7927e743fc0635b4912d65cdaf`, `CRC32`, and `ab.rs`-compatible 4-lane `FastHash`).
- **Protocol:** `cargo run --release --example delivery_copyout_census -- corpora/wikidata.bz2 7 5` — 1 warmup + **7 interleaved repeats** for all micro-characterisations (scaled to 838.80 MB = `wikidata ×4`) and 1 warmup + **5 interleaved repeats** for all real end-to-end parallel decodes. All medians and `[min..max]` ranges reported below.

### Block geometry & RLE1 expansion (Section 0)

| metric (`corpora/wikidata.bz2`, 431 blocks) | measured value |
|---|---|
| Decoded block size (min / p10 / median / mean / p90 / p99 / max) | `0.0 / 0.0 / 878.8 / 475.1 / 878.9 / 878.9 / 878.9 KiB` |
| Blocks exceeding `64 KiB` / `256 KiB` / `512 KiB` | **233 / 431 (54.1%)** (carrying ~99% of bytes) |
| Blocks exceeding `900 KB` / `1 MiB` | **0 / 431 (0.0%)** |
| Blocks with RLE1 `shift > 0` / `len > n` | **59 / 431 (13.7%)** / **45 / 431 (10.4%)** |
| `shift` p50 / p95 / max | `0 B / 48 B / 193 B` |
| Blocks overflowing `prod_pool_cap = n + n/16 + 64` (`src/parallel.rs:1210`) | **0 / 431 (0.0%)** (`max (n+shift)/n = 1.000×`) |

Because the corpus is strongly bimodal (233 full ~879 KiB blocks + 198 small/trailing blocks), standard Rust `io::Read` buffers (`8–256 KiB`) cannot hold a single full block, while `src/parallel.rs:1210`'s pooled `Vec` capacity causes zero reallocations on real `wikidata` blocks (though arbitrary bzip2 streams can expand up to ~51.8× / ~46 MiB in RLE1, prohibiting sound fixed-size `&mut [u8]` worker targets without a resize/fallback path).

---

## 3. Measurements across all four delivery options

### Option 1: Baseline `io::Read` copy-out across caller buffer sizes (838.80 MB, 7 repeats)

Warm working set = `4.29 MiB` (8 real blocks, `0.10×` VM L3); Cold working set = `199.99 MiB` (all 431 blocks ×4, `4.44×` VM L3).

| caller buffer | warm pure copy ms `[min..max]` | warm GB/s | cold pure copy ms `[min..max]` | cold GB/s | warm copy+Hash ms `[min..max]` | cold copy+Hash ms `[min..max]` |
|---|---:|---:|---:|---:|---:|---:|
| `8 KiB` (L1d-fit) | **35.82** `[33.47..43.91]` | 23.42 | **85.56** `[83.70..96.69]` | 9.80 | **93.96** `[90.71..109.17]` | 150.04 `[149.04..154.05]` |
| `64 KiB` (L2-fit) | **59.35** `[53.91..68.74]` | 14.13 | **92.49** `[88.46..103.76]` | 9.07 | **99.26** `[95.31..102.54]` | **143.10** `[141.02..165.87]` |
| `256 KiB` (L2-edge) | **69.49** `[68.92..75.60]` | 12.07 | **94.34** `[91.99..117.21]` | 8.89 | **101.62** `[96.42..122.90]` | **141.80** `[139.78..154.63]` |
| `1024 KiB` (L3) | **70.87** `[62.56..74.30]` | 11.84 | **107.18** `[102.91..115.68]` | 7.83 | **122.53** `[114.92..132.71]` | 156.85 `[152.26..162.19]` |
| `4096 KiB` (`ab.rs`) | **71.48** `[62.85..75.37]` | 11.74 | **104.95** `[101.86..110.55]` | 7.99 | **122.46** `[116.62..133.27]` | 156.50 `[154.45..161.57]` |
| `full Vec` (839 MB, pre-faulted) | **151.28** `[147.21..156.26]` | 5.54 | **189.43** `[185.17..195.41]` | 4.43 | — | — |

**Key finding (L2 destination spill in `examples/ab.rs`):** A `64 KiB` caller buffer beats `ab.rs`'s `4 MiB` buffer by **23.2 ms (−19%)** warm and **13.4 ms (−8.6%)** cold in `copy+Hash`. Because `4 MiB` allows `read()` to copy an entire ~879 KiB block before returning, the copy evicts the 256 KiB L2 before `Hash::update` reads byte 0; with a `64 KiB` buffer, `Hash::update` consumes every chunk hot from L2. Writing into a contiguous 839 MB `Vec` (even pre-faulted) drops throughput to **4.43–5.54 GB/s** due to L3 write-allocate (RFO) + dirty DRAM writeback.

---

### Option 2: Zero-copy block lending (`BufRead` / `read_block`) vs `io::Read` (838.80 MB)

#### 2A. Isolated consumer comparison (7 interleaved repeats, 838.80 MB)

| consumer workload | warm lend (ms) | warm `read(64K)` (ms) | warm `read(4M)` (ms) | warm `lend/4M` | cold lend (ms) | cold `read(64K)` (ms) | cold `read(4M)` (ms) | cold `lend/4M` |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| discard / no-op | **0.00** `[0.00..0.00]` | 58.50 `[51.04..61.60]` | 71.93 `[64.74..73.65]` | **0.00×** | **0.00** `[0.00..0.00]` | 94.89 `[89.98..99.44]` | 105.35 `[102.62..110.57]` | **0.00×** |
| 4-lane `Hash` (`ab.rs`) | **58.76** `[57.65..73.78]` | 97.17 `[95.38..97.79]` | 122.55 `[120.05..123.74]` | **0.48×** | **100.56** `[97.95..108.31]` | 140.80 `[139.50..144.49]` | 157.73 `[154.20..165.69]` | **0.64×** |
| newline scan (`wc -l`) | **672.61** `[660.21..689.72]` | 705.61 `[703.44..723.85]` | 731.08 `[726.25..734.14]` | **0.92×** | **667.37** `[661.17..679.82]` | 753.71 `[742.01..768.37]` | 760.73 `[755.61..779.16]` | **0.88×** |
| `CRC32` (PCLMULQDQ) | **64.97** `[63.71..65.57]` | 103.69 `[98.15..104.99]` | 128.50 `[122.84..132.95]` | **0.51×** | **101.33** `[99.83..102.71]` | 147.88 `[146.53..150.51]` | 162.51 `[158.81..166.06]` | **0.62×** |

Notice how **zero-copy lending** and **L3 residency** compose nearly additively on lightweight consumers (`Hash` / `CRC32`):
- Cold `read(4M) + Hash` = **157.73 ms**
- Lending alone (still cold) = **100.56 ms** (**−57.17 ms / −36%**)
- L3-hot alone (still `read(4M)`) = **122.55 ms** (**−35.18 ms / −22%**)
- **Both together (warm lend + Hash)** = **58.76 ms** (**−98.97 ms / −63%**, **2.68× faster**)

#### 2B. Real end-to-end parallel decodes on `wikidata` (209.7 MB) and `wikidata ×4` (838.8 MB)

2 vCPUs (`threads = 2`, `slots = 8`), `Cursor<Arc<[u8]>>` zero-clone input, 5 interleaved repeats, SHA-256 + `FastHash` verified:

| end-to-end arm (`threads=2, slots=8`) | `wikidata` (209.7 MB) ms `[min..max]` | vs `Bz4M` | `wikidata ×4` (838.8 MB) ms `[min..max]` | MB/s | vs `Bz4M` |
|---|---:|---:|---:|---:|---:|
| `BzReader::read(8K)` | 427.9 `[414.5..437.7]` | +0.03% | — | — | — |
| `BzReader::read(64K)` | 429.2 `[406.7..432.4]` | +0.33% | 1681.8 `[1637.8..1759.3]` | 498.8 | **−1.84%** (−31.6 ms) |
| `BzReader::read(256K)` | 416.1 `[414.0..432.8]` | −2.74% | — | — | — |
| `BzReader::read(1024K)` | 427.2 `[417.1..429.2]` | −0.15% | — | — | — |
| `BzReader::read(4096K)` (baseline) | 427.8 `[412.4..438.4]` | +0.00% | 1713.4 `[1659.7..1934.7]` | 489.6 | +0.00% |
| **`BzReader::BufRead(fill_buf)` (public)** | 425.9 `[415.6..490.0]` | −0.44% | **1615.3** `[1601.2..1727.2]` | **519.3** | **−5.73%** (−98.1 ms) |
| `StreamDecoder::read(64K)` | 445.0 `[415.6..486.1]` | +4.02% | 1688.0 `[1630.2..1746.2]` | 496.9 | −1.48% (−25.4 ms) |
| `StreamDecoder::read(4096K)` | 415.8 `[412.7..450.9]` | −2.81% | 1701.9 `[1632.5..1708.9]` | 492.9 | −0.67% (−11.5 ms) |
| **`StreamDecoder::read_block(recycled)`** | **413.1** `[407.0..422.0]` | **−3.43%** | **1639.5** `[1609.2..1665.4]` | **511.6** | **−4.31%** (−73.9 ms) |
| `StreamDecoder::read_block(unrecycled)` | 420.8 `[404.5..424.3]` | −1.64% | — | — | — |
| `decode_parallel(contiguous Vec)` | 554.2 `[551.8..568.6]` | +29.55% | 2434.6 `[2215.3..2507.6]` | 344.5 | +42.09% (+721.2 ms) |
| `read_block -> pre-sized Vec` | 552.7 `[541.2..556.3]` | +29.21% | 2486.3 `[2285.6..2793.3]` | 337.4 | +45.11% (+772.8 ms) |
| `read_block -> Vec<Vec<u8>>` (0-copy, unrecycled) | 530.6 `[522.2..615.6]` | +24.03% | 3024.7 `[2933.8..3352.4]` | 277.3 | **+76.53%** (+1311.3 ms) |

**Three lessons from 2B:**
1. On `wikidata ×4` (even with only 2 vCPUs!), **`BzReader`'s existing public `BufRead::fill_buf`** cuts total end-to-end wall by **−5.73% (−98.1 ms)** and `StreamDecoder::read_block(recycled)` cuts it by **−4.31% (−73.9 ms)** because on 2 vCPUs the reader thread competes directly with the 2 worker threads for CPU cycles, just as it competes with SMT siblings at `k = 9..10` on the 10-core i9!
2. **Whole-file materialisation (`decode_parallel`) cannot avoid DRAM write + page-fault cost (+721–773 ms on `x4`)**, and attempting "zero-copy whole-file materialisation" by keeping `Vec<Vec<u8>>` without recycling (`read_block(Vec::new())`) is **even slower (+1,311 ms / +76.5%)** because workers must allocate and kernel-zero-fault 1,724 fresh ~935 KiB allocations instead of reusing 8 hot pooled buffers! This proves **buffer recycling is mandatory** for any block-lending API.
3. **Internal zero-copy win found (`src/python.rs:121–139`):** Python's `Reader::read(size)` currently allocates a freshly zeroed `Vec<u8>`, copies via `reader.read(&mut buf)`, and then copies a *second* time into `PyBytes::new(py, &buf[..n])`. Switching `python.rs` to `BufRead::fill_buf` (or `PyBytes::new_with`) removes an entire copy + allocation with zero Rust public API change.

---

### Option 3: Caller-provided buffers & `read()` straight into user's slice

- **3A. Caller-provided buffer pool:** Already implemented for `Vec<u8>` by `StreamDecoder::read_block(spent)` (`src/parallel.rs:880, 958–965, 1207–1211`): passing `spent` back inserts it into `State::free_out`, workers invert and run `unrle_in_place` directly into it, and `read_block` hands ownership back with zero copy and `0/431` reallocations on `wikidata.bz2`. Supporting *non-`Vec`* fixed-size caller pools (e.g., arenas, `mmap` slabs) is unsound without an overflow/fallback path because bzip2 RLE1 can expand a 900 KB block up to ~46 MiB (~51.8×).
- **3B. `read()` straight into the user's `&mut [u8]` slice:**
  - Structurally blocked for parallel workers: workers finish out of order while `io::Read::read(&mut [u8])` supplies a single mutable slice synchronously for only the next in-order block, whose uncompressed length is unknown before RLE1, and **54.1%** of `wikidata` blocks (233/431, ~99% of bytes) exceed 64–512 KiB caller buffers.
  - The only way to write straight into `buf` on the reader thread is **deferred serial RLE1 + CRC** (workers stop after BWT inversion, reader runs `rle::unrle` + `crc::crc32` into `buf`). Measured on real inverted `wikidata` blocks (scaled to 838.80 MB, 7 interleaved repeats):

    | regime | deferred serial `unrle + CRC + Hash` (ms) | baseline `copy_out(4M) + Hash` (ms) | ratio |
    |---|---:|---:|---:|
    | Warm (4.29 MiB L3) | **223.64** `[218.81..232.07]` | **121.20** `[118.37..133.88]` | **1.85× slower** |
    | Cold (199.99 MiB DRAM) | **254.93** `[248.51..279.97]` | **157.15** `[152.55..177.50]` | **1.62× slower** |

  - On the i9-10900K (`c71c4a56`), 1W `R` (`A9`) is **74.4 ms** vs 1W `copy_out` **29.1 ms** (**2.56× slower**). Deferring RLE1 onto the serial reader thread makes the serial delivery floor **1.6–2.6× worse** AND still fails on any caller buffer `< 879 KiB` (plus delays block CRC verification until after partial bytes are exposed if streamed). **Option 3B is strictly dominated.**

---

### Option 4: Multi-thread copy vs L3-bounded working set (`READY_BYTES` / `slots`)

#### 4A. Multi-threaded `memcpy` (1T vs 2T on 2-vCPU Xeon, 838.80 MB, 7 repeats)

| copy pattern | 1-thread ms `[min..max]` (GB/s) | 2-thread ms `[min..max]` (GB/s) | speedup |
|---|---:|---:|---:|
| `4 MiB` buf, warm L3 (per-block 2T split) | **72.68** `[67.47..77.07]` (11.54) | 109.67 `[103.88..115.28]` (7.65) | **0.66× (slower)** |
| `4 MiB` buf, cold DRAM (per-block 2T split) | **107.80** `[103.34..112.85]` (7.78) | 143.67 `[140.99..148.19]` (5.84) | **0.75× (slower)** |
| `839 MB Vec`, warm L3 (batch-8 2T) | 149.92 `[146.13..155.24]` (5.59) | **100.81** `[99.40..104.37]` (8.32) | **1.49×** |
| `839 MB Vec`, cold DRAM (batch-8 2T) | 187.71 `[184.86..193.68]` (4.47) | **122.77** `[118.75..130.26]` (6.83) | **1.53×** |

Per-block multi-thread copy into a streaming buffer **hurts by 25–34%** (`3,448` barrier wakeups + cache-line bouncing over 475 KiB mean blocks). Batched 2-thread copy into a pre-sized contiguous `Vec` achieves `1.49–1.53×` *only* on idle cores — useless during saturated 8–10W decoding where stealing a worker core slows workers and adds DRAM contention.

#### 4B. Working-set footprint sweep across the VM's 45 MiB L3 cliff (838.80 MB, 7 repeats)

| `W` (MiB) | blocks | `W / L3_VM (45 MiB)` | `copy_out(4M)` ms (GB/s) | `copy(4M)+Hash` ms (GB/s) | `lend+Hash` ms (GB/s) |
|---:|---:|---:|---:|---:|---:|
| 1.72 | 3 | 0.04× | 69.44 `[56.48..71.78]` (12.08) | 115.81 `[112.14..117.66]` (7.24) | **57.48** `[56.60..58.50]` (14.59) |
| 2.57 | 4 | 0.06× | **59.32** `[55.04..71.18]` (14.14) | **115.34** `[112.42..120.39]` (7.27) | **57.47** `[56.54..66.90]` (14.59) |
| 4.29 | 8 | 0.10× | 63.19 `[60.41..73.15]` (13.28) | 119.65 `[112.82..122.37]` (7.01) | 59.04 `[57.23..62.00]` (14.21) |
| 8.58 | 16 | 0.19× | 68.61 `[64.87..74.60]` (12.22) | 123.88 `[116.75..129.38]` (6.77) | 64.68 `[59.77..72.25]` (12.97) |
| 12.02 | 23 | 0.27× | 70.62 `[64.02..74.26]` (11.88) | 131.85 `[119.95..135.81]` (6.36) | 68.05 `[60.20..90.84]` (12.33) |
| **16.31** | 31 | **0.36×** | **80.68** `[68.92..114.42]` (10.40) | **136.32** `[128.80..157.44]` (6.15) | **83.35** `[67.41..101.73]` (10.06) |
| 24.03 | 48 | 0.53× | 91.48 `[76.47..105.49]` (9.17) | 143.52 `[134.65..162.31]` (5.84) | 104.58 `[83.24..111.53]` (8.02) |
| **32.62** | 64 | **0.72×** | **94.46** `[88.87..109.38]` (8.88) | **151.69** `[144.49..174.34]` (5.53) | **99.79** `[96.01..110.79]` (8.41) |
| 48.07 | 100 | 1.07× | 102.91 `[97.55..130.36]` (8.15) | 157.27 `[153.39..163.35]` (5.33) | 102.25 `[97.82..134.23]` (8.20) |
| 64.37 | 138 | 1.43× | 105.00 `[98.59..115.05]` (7.99) | 159.93 `[152.25..170.26]` (5.24) | 105.38 `[100.05..108.54]` (7.96) |
| 199.99 | 431 | 4.44× | 105.68 `[104.12..106.51]` (7.94) | 156.47 `[155.55..161.40]` (5.36) | 101.86 `[99.82..106.27]` (8.23) |

**Exact hardware scaling to the i9-10900K (20 MiB L3):**
On this VM, degradation begins at **`W / L3 ≈ 0.36×`** (not `1.0×`!) and reaches **97%** of the cold-DRAM plateau by **`W / L3 ≈ 0.72×`**, because the 4 MiB caller buffer, set-associativity conflicts, and concurrent threads share L3. Scaling `0.36×` and `0.72×` to the i9's **20 MiB L3** predicts onset at **`7.2 MiB`** and full cold-DRAM saturation by **`14.4 MiB`** — matching `c71c4a56` directly (`none@8W` onset at `rb_hwm = 6.9 MiB` → `34.0 ms`; `none@9–10W` at `12.0–14.6 MiB` → `43.2–43.7 ms`; `SC@10W` at `16.3 MiB` → `77.9 ms` under concurrent worker DRAM traffic).

#### 4C. `StreamDecoder` `slots` sweep caveat (wikidata.bz2, 2T, 5 repeats)

| `slots` | `StreamDecoder::read(4M)` ms `[min..max]` | `StreamDecoder::read_block(recycled)` ms `[min..max]` | vs `slots = 8` |
|---:|---:|---:|---:|
| `2` (`1×T`) | 716.6 `[697.4..757.9]` | 713.4 `[697.1..722.8]` | **+71.0% (severe stall)** |
| `4` (`2×T`) | 458.1 `[450.9..491.7]` | 476.5 `[446.4..533.0]` | **+9.4% to +14.6% stall** |
| `8` (`4×T`) | **418.9** `[411.8..459.0]` | **415.9** `[405.4..431.4]` | baseline |

**Crucial input to `delivery-l3-working-set`:** Because `wikidata` blocks are bimodal, shrinking `slots` to `1×T–2×T` causes severe head-of-line worker starvation when a small block finishes behind a large block. The existing byte-based `Cap` guard (`src/parallel.rs:402–408`: throttle only when `ready.bytes >= c.bytes && ahead >= c.floor`) avoids starving small blocks while bounding unread bytes in L3. Tightening `READY_BYTES` (e.g. `12 MiB → 2–4 MiB`, `0.10–0.20×` of 20 MiB L3) — while keeping `slots = 4×T` — is the clean mechanism to test on the i9.

---

## 4. i9-10900K grounded projections & design matrix

Using `c71c4a56` (`copy_out` = `29.1 ms` 1W, `27.6 ms` 6W, `34.0 ms` 8W, `43.7–77.9 ms` 10W) and `cc8ba137` (`SFCMR − SFCMRD` = `48 ms` cold single-core; `SFCMRD` = `5.3 ms`) anchored by the VM's dimensionless `lend/copy` ratios (`0.48×` warm, `0.64×` cold) and `W/L3` cliff (`0.36–0.72×`):

| Option | Already exists? | Public API change? | Projected i9 `copy_out` (`838.8 MB`, `Hash`) | Impact on `8–10W` wall today | Costs & failure modes |
|---|---|---|---|---|---|
| **1a. Baseline `io::Read` (4 MiB)** | Yes (`BzReader`, `StreamDecoder`) | No | `27.6–34.0 ms` (1–8W), `43.7–77.9 ms` (10W) | Baseline | 879 KiB blocks spill 256 KiB L2 before consumer reads; 12–16.3 MiB `Ready` spills 20 MiB L3 |
| **1b. `io::Read` with 64 KiB buf** | Yes (caller choice) | No | `~23–28 ms` (1–8W, −15..19% vs 4M) | Minor (−0..1% on 8–10W) | More `read()` mutex calls (13.4K vs 1.7K); still copies every byte once |
| **2a. `BzReader` `BufRead::fill_buf`** | **Yes (already public!)** (`src/read.rs:339`) | **No** | **`~13–17 ms`** (L3-hot) / **`~31 ms`** (cold) | Removes reader L2/L3 dst writes & halves reader CPU at 9–10W; future serial floor `~13–15 ms` | Streaming consumers only; cannot avoid final copy for contiguous `Vec<u8>` |
| **2b. Expose `StreamDecoder::read_block` (or `BlockGuard`)** | `#[doc(hidden)] pub` (`src/parallel.rs:880`) | **Yes (needs Louis)** | **`~13–17 ms`** (L3-hot) / **`~5.3 ms`** (discard) | Same as 2a for push-API callers; zero feeder thread | Raw `read_block(spent)` has a footgun: forgetting to recycle `spent` causes **+76.5%** page-fault slowdown (`BlockGuard` with `Drop`-recycling fixes this) |
| **3a. Caller-provided buffer pool** | `Vec<u8>` via `read_block(spent)` | Yes if custom allocator | Same as 2b | Same as 2b | Fixed-size non-`Vec` slabs unsound under bzip2's 51.8× worst-case RLE1 expansion without fallback |
| **3b. `read()` straight into `&mut [u8]`** | No | No (or pull callback) | **`~74–120+ ms`** (deferred serial `R`) | **Severe regression** (1.6–2.6× slower serial floor) | Workers finish out-of-order; 54.1% of blocks exceed 64–512 KiB slices; breaks pre-delivery CRC if streamed |
| **4a. Multi-thread `memcpy`** | No | No (internal) | Per-block: **slower (`0.66–0.75×`)**; Batched Vec: `~1.5×` on idle cores only | **Regresses 8–10W** (steals worker cores + DRAM BW) | Unusable for streaming `io::Read`; only helps idle-core whole-file `decode_parallel` |
| **4b. L3-bounded `READY_BYTES` (`delivery-l3-working-set`)** | `READY_BYTES = 12 MiB` (`src/parallel.rs:74`) | **No** | Keeps `copy_out` at **`~28–34 ms`** at 9–10W (vs `43.7–77.9 ms`) | **High at 9–10W**: jointly rescues `D` (`−10..44 ms` reader CPU) AND `R` (`A9`: `215.6 → ~100–120 ms`) | Must throttle via byte `Cap`, NOT low `slots` (Section 4C: `slots = 1–2×T` stalls +10..71%) |

---

## 5. Design recommendation

1. **Do NOT pursue Option 3b (`read()` straight into `&mut [u8]`) or Option 4a (multi-thread `memcpy`).** Both are strictly dominated by measurements: deferred serial RLE1+CRC is 1.62–1.85× slower on VM (~2.5× on i9) and fails `< 900 KiB` caller buffers; per-block MT `memcpy` is 0.66–0.75× slower and steals worker cores/DRAM at 8–10W.
2. **Highest-leverage no-API-change delivery work is already scoped: `delivery-l3-working-set` (paired with `single-pass-rle1-crc`).**
   Bounding `READY_BYTES` to **2–4 MiB** (`0.10–0.20×` of the i9's 20 MiB L3, keeping `slots = 4×threads` so bimodal blocks do not stall) attacks the real 9–10W i9 bottleneck where `D` (`43.7–77.9 ms`) and `R` (`133–216 ms`) blow up together under L3 eviction.
3. **Internal zero-copy adoption (no public API change):**
   - `src/python.rs:121–139` should adopt `BzReader`'s existing `BufRead::fill_buf` (or `PyBytes::new_with`) to eliminate its double copy on every Python `read()`.
   - Callers of `BzReader` already have public zero-copy lending via `BufRead::fill_buf` / `consume` (worth documenting prominently alongside `read()`).
4. **Public API proposal (`stabilise-read-block`, opened at `status = "idea"`, P4, `needs_louis`):**
   - Exposing zero-copy lending on `StreamDecoder` directly is **deferred** until scan + workers approach the ~28–34 ms L3-hot delivery floor (or downstream push-API users request it).
   - When ready, prefer a **`Drop`-recycling lent-block guard** (e.g. `StreamDecoder::read_block(&self) -> Result<Option<DecodedBlock<'_>>, Error>` implementing `Deref<Target = [u8]>`, returning its `Vec<u8>` to `State::free_out` on `Drop`) over un-hiding raw `read_block(spent: Vec<u8>)`, so callers cannot accidentally trigger the **+76.5%** unrecycled allocation penalty measured in Section 2B. Any public API change requires Louis's sign-off per `delivery-copyout`'s `stop_test` and RULES D.
