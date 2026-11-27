# Dissection of Public Rust Streaming I/O, Whole-Buffer Decode, Multi-Stream Framing, and `src/block.rs`

*Date:* 27 November 2026
*Zone:* Smooth (`docs/journal/2026-11-27-explore/read-io-and-block-framing-dissection.md`, `examples/read_io_framing_dissection.rs`, zero edits to `src/`)
*Hardware & environment:* 2-vCPU Haswell KVM VM, Linux (glibc dynamic `M_MMAP_THRESHOLD`), `rustc 1.99.1`, baseline x86-64 and JCC-32B-aligned (`-C llvm-args=-x86-branches-within-32B-boundaries`).
*Measurement discipline:* Every timing point runs in a **fresh isolated subprocess** (`n = 5` paired interleaved alternating rounds; `taskset -c 0` for `t = 1`, both vCPUs for `t = 4`), recording wall time, exact process CPU (`getrusage(RUSAGE_SELF)` user + sys, immune to host preemption), exact peak RSS (`/proc/self/status` `VmHWM`), and exact minor page faults (`ru_minflt`). Output verification (FNV-1a-64 + byte length against `mibz2::decode`) runs strictly **after** timers and `getrusage` stop. Every run across all axes matched `mibz2::decode` 100% byte-for-byte (0 mismatches).

---

## 1. Frozen Stop Test (C1a / C2 — Frozen Before Measurements)

- **Exactness invariant:** Every prototype matches shipped `mibz2::decode` 100% byte-for-byte on every corpus (`wikidata.bz2`, `text.bz2`, `random.bz2`, `zeros_100m.bz2`, `multistream_wiki_50.bz2`, `multistream_wiki_200.bz2`, `multistream_small_200.bz2`), with zero new `unsafe` code (`#![forbid(unsafe_code)]` for all candidate mechanisms).
- **Independent per-axis PASS / STOP bars (both baseline x86-64 and JCC-32B-aligned builds required):**
  1. **Axis A (`BzReader::read_to_end` & `Read` chunking, `src/read.rs`):** PASS iff a zero-`unsafe` `BzReader::read_to_end` override cuts `wikidata.bz2` CPU by `>= 10.0%` disjoint OR peak RSS (`VmHWM`) by `>= 10.0%` at `t = 1` or `t = 4` across both builds, with zero `> 2%` regression on `text`, `random`, or `zeros_100m`. Otherwise STOP.
  2. **Axis B (`decode()` & `decode_parallel()` output `Vec` growth/copies, `src/lib.rs`):** PASS iff improved capacity sizing / growth cuts `decode()` or `decode_parallel()` CPU by `>= 5.0%` disjoint OR peak RSS (`VmHWM`) by `>= 15.0%` on `wikidata.bz2` (or `>= 25%` RSS on `random`/`zeros_100m`) with zero `> 2%` CPU regression on any corpus. Otherwise STOP.
  3. **Axis C (Multi-stream `.bz2` framing & `syms` reuse across `Item::End`, `src/stream.rs` + `src/block.rs`):** PASS iff preserving `syms` across `Item::End` cuts 200-stream concatenated `.bz2` CPU by `>= 8.0%` disjoint OR minor page faults by `>= 50%` on default Linux across both builds, with zero regression on single-stream equivalents. Otherwise STOP.
  4. **Axis D (`src/block.rs` framing & `src/bits.rs` refill / `objdump -d`):** PASS iff dissection + `objdump -d` finds avoidable framing/refill/panic/spill overhead `>= 5.0%` of serial scan `S` outside already-owned A2 (`#1117`) and A3 (`#1153`), with a byte-exact prototype cutting it `>= 5.0%` disjoint across both builds. Otherwise STOP.

---

## 2. Executive Summary: All Four Axes **`STOP`** on Default Linux (with Two High-Value Nuances)

| Axis | Target | Frozen Verdict (default Linux) | Key measured finding & mechanism |
|---|---|---|---|
| **A** | `BzReader::read_to_end` & `Read` chunking (`src/read.rs`) | **`STOP`** (`< 10%` on `wikidata`) | Rust 1.99 `default_read_to_end` already passes full `Vec` spare capacity (`656` calls, not `25,829`), and `io::copy(&mut r, &mut Vec)` specializes to `read_to_end`. Overriding `Seq::read_to_end` to decode directly into caller's `Vec` (`A6`) eliminates spare-capacity `memset(0)` + `Seq.out` copy: **`-8.1%` base / `-6.1%` JCC disjoint CPU** on `wikidata` `t=1` (`-3.8%` RSS), **`-15%..-26%` RSS** on `text`/`random`, and **`-50.5%` base / `-48.7%` JCC CPU (`-41.3%` RSS)** on expansive `zeros_100m`. `Par::read_to_end` via `fill_buf` (`A5`) cuts `zeros_100m` `-10.8%..-14.3%` CPU / `-13.7%` RSS, is flat on `wikidata` (`-1.6%..-3.3%`), and regresses `random` `+4.1%..+6.1%` from doubling `8 → 13.7 MiB`. True `8 KiB` `io::copy` to a non-`Vec` writer makes `25,829` calls vs `432` `fill_buf` calls but is CPU/RSS-neutral in pure Rust (`859 vs 871 ms` `t=1`, `1000 vs 996 ms` `t=4`) because the `8 KiB` buffer stays L1d-hot. |
| **B** | `decode()` & `decode_parallel()` `Vec` growth (`src/lib.rs`) | **`STOP`** | On Linux glibc, `Vec` buffers `>= 128 KiB` grow via `mmap` + **`mremap(MREMAP_MAYMOVE)`** (O(1) page-table remap, zero physical byte copying) and demand-paging never backs untouched virtual tail capacity. Cutting `decode()` `realloc`s from `9` (`215.4 MiB` logical move) to `1` (`1.7 MiB`) via adaptive ratio estimation left `VmHWM` identical (`227.9 vs 227.9 MiB`), `minflt` identical (`59,853 vs 59,852`), and CPU flat (`0.967×` base, `1.048×` JCC). `decode_parallel` `4×` cap on `random` (`30.7 vs 7.7 MiB` virtual) had zero RSS cost (`59.6 vs 60.4 MiB`). `Vec<Vec<u8>>` + exact concat (`B2`) regressed `wikidata` `t=4` by **`+45.6..+48.4%` wall, `+12.5..+14.6%` CPU, and `+55.5..+56.6%` RSS (`442.7 vs 284.7 MiB`)**. |
| **C** | Multi-stream `.bz2` restart & `Item::End` `syms` drop (`src/stream.rs`, `src/block.rs`) | **`STOP`** *(default glibc)* | (1) `StreamDecoder` and `BzReader::parallel` do **zero** thread respawning or `Scratch` teardown across 50–200 concatenated streams (`multistream_wiki_50/200` run slightly faster and at lower RSS than single-stream twins at `t=4`). (2) `src/stream.rs:161` + `src/block.rs:195` **does drop the `1.80 MiB` `syms` `Vec<u16>` on every `Item::End`** (`free@GLIBC_2.2.5` confirmed in `objdump -d`). On default Linux glibc, stream 0's `munmap` ratchets `M_MMAP_THRESHOLD` above `1.80 MiB`, masking streams `1..199` (`0.979–1.004×` CPU, `-9%..-27%` `minflt`). Under a fixed-threshold allocator (`MALLOC_MMAP_THRESHOLD_=131072`, matching musl/macOS/containerized thresholds), preserving `syms` across `Item::End` cuts multi-stream scan CPU **`-32.3%..-35.6%` disjoint**, cuts scan `minflt` **`-95.1%..-98.6%` (`4,033 → 55`)**, and cuts full `t=1` decode CPU **`-6.7%..-8.6%` disjoint (`-53.5%..-82.2%` `minflt`)**. |
| **D** | `src/block.rs` framing & `src/bits.rs` refill (`objdump -d`) | **`STOP`** | Pure header/framing parsing (`magic`, `crc`, `orig_ptr`, `in_use`, `selectors`, `codelens`) is `0.8%–1.5%` (base) / `1.8%–4.7%` (JCC) of `read_item_with_scratch` (A2 header/table build is already owned by `#1117`, A3 symbol loop by `#1153`). `objdump -d` confirms `vzero = 0` in `read_item_with_scratch`, `decode_runs_bmi2`, and `BzReader::read`, clean register-held `Reg` bit-reader with cold `refill_slow`, and only 1 cold `panic_bounds_check` per function. |

---

## 3. Corpora

All corpora generated deterministically via `libbzip2 -9` (plus repo `wikidata.bz2`):

| Corpus | Compressed (B) | Output (B) | Ratio | Streams | Blocks |
|---|---:|---:|---:|---:|---:|
| `wikidata.bz2` | 11,854,933 | 209,700,196 | 17.69× | 12 | 431 |
| `text.bz2` | 3,379,241 | 14,000,000 | 4.14× | 1 | 15 |
| `random.bz2` | 8,035,648 | 8,000,000 | 1.00× | 1 | 9 |
| `zeros_100m.bz2` | 113 | 104,857,600 | 927,943× | 1 | 3 |
| `multistream_wiki_50.bz2` | 1,547,391 | 26,214,400 | 16.94× | 50 | 50 |
| `singlestream_wiki_50.bz2` | 1,431,924 | 26,214,400 | 18.31× | 1 | 30 |
| `multistream_wiki_200.bz2` | 3,776,542 | 52,428,800 | 13.88× | 200 | 200 |
| `singlestream_wiki_200.bz2` | 3,006,155 | 52,428,800 | 17.44× | 1 | 59 |
| `multistream_small_200.bz2` | 790,452 | 6,553,600 | 8.29× | 200 | 200 |
| `singlestream_small_200.bz2` | 326,771 | 6,553,600 | 20.06× | 1 | 8 |

---

## 4. Axis A: `BzReader::read_to_end` & `Read` Chunking (`src/read.rs`)

### 4.1 `wikidata.bz2` (`209.7 MB`, `n = 5` interleaved)

| Build | Mode | `t` | CPU med (ms) | CPU ratio `[95% CI]` | `VmHWM` (MiB) | `minflt` | Final `cap` (MiB) | `read` calls |
|---|---|--:|--:|---|--:|--:|--:|--:|
| base | `A0_r2e_seq` (shipped) | 1 | `1134.1` | `1.000` | `236.8` | `62,125` | `360.0` | `656` |
| base | `A1_iocopy_seq` (`&mut Vec`) | 1 | `1113.3` | `0.981 [0.963, 0.998]` | `236.7` | `62,125` | `360.0` | `656` |
| base | `A2_read64k_seq` | 1 | `1112.1` | `0.974 [0.938, 1.010]` | `236.8` | `62,137` | `256.1` | `3,461` |
| base | `A3_read1m_seq` | 1 | `1125.2` | `0.988 [0.949, 1.029]` | `237.6` | `62,342` | `219.7` | `432` |
| base | `A4_fillbuf_drain_seq` (ceiling) | 1 | `882.3` | `0.791 [0.739, 0.847]` | `36.8` | `10,925` | `0.0` | `432` |
| base | `A5_proto_r2e_seq` (`fill_buf`) | 1 | `1090.4` | `0.961 [0.930, 0.994]` | `236.8` | `62,122` | `219.7` | `0` |
| base | **`A6_proto_seq_direct`** | 1 | **`1035.2`** | **`0.919 [0.900, 0.939]`** | `227.9` | `59,851` | `219.7` | `0` |
| JCC | `A0_r2e_seq` (shipped) | 1 | `1101.2` | `1.000` | `236.8` | `62,126` | `360.0` | `656` |
| JCC | `A4_fillbuf_drain_seq` (ceiling) | 1 | `877.5` | `0.779 [0.725, 0.835]` | `36.8` | `10,926` | `0.0` | `432` |
| JCC | `A5_proto_r2e_seq` (`fill_buf`) | 1 | `1087.1` | `0.984 [0.950, 1.019]` | `236.8` | `62,122` | `219.7` | `0` |
| JCC | **`A6_proto_seq_direct`** | 1 | **`1029.4`** | **`0.939 [0.892, 0.990]`** | `227.9` | `59,852` | `219.7` | `0` |
| base | `A0_r2e_par` (shipped) | 4 | `1197.0` | `1.000` | `287.8` | `77,824` | `360.0` | `656` |
| base | `A4_fillbuf_drain_par` (ceiling) | 4 | `955.3` | `0.793 [0.771, 0.817]` | `91.5` | `26,108` | `0.0` | `432` |
| base | `A5_proto_r2e_par` (`fill_buf`) | 4 | `1191.8` | `0.984 [0.948, 1.021]` | `293.2` | `79,379` | `219.7` | `0` |
| JCC | `A0_r2e_par` (shipped) | 4 | `1203.5` | `1.000` | `303.5` | `82,119` | `360.0` | `656` |
| JCC | `A4_fillbuf_drain_par` (ceiling) | 4 | `958.0` | `0.795 [0.763, 0.828]` | `92.3` | `26,659` | `0.0` | `432` |
| JCC | `A5_proto_r2e_par` (`fill_buf`) | 4 | `1169.9` | `0.967 [0.928, 1.007]` | `291.1` | `78,509` | `219.7` | `0` |

### 4.2 Guard Corpora (`text.bz2`, `random.bz2`, `zeros_100m.bz2`) & Non-`Vec` `io::copy`

| Corpus | Build | `t` | `A0_r2e` CPU / `VmHWM` | Best proto (`A6` Seq / `A5` Par) CPU / `VmHWM` | CPU ratio `[95% CI]` | RSS delta |
|---|---|--:|---|---|---|---:|
| `text.bz2` | base | 1 | `151.8 ms` / `33.4 MiB` | `138.8 ms` / `28.3 MiB` (`A6`) | **`0.909 [0.831, 0.996]`** | **`-15.3%`** |
| `text.bz2` | JCC | 1 | `146.1 ms` / `33.4 MiB` | `136.9 ms` / `28.4 MiB` (`A6`) | `0.958 [0.862, 1.064]` | **`-15.0%`** |
| `random.bz2` | base | 1 | `210.5 ms` / `35.9 MiB` | `197.0 ms` / `26.7 MiB` (`A6`) | `0.936 [0.853, 1.027]` | **`-25.6%`** |
| `random.bz2` | JCC | 1 | `199.6 ms` / `35.9 MiB` | `186.9 ms` / `26.6 MiB` (`A6`) | `0.956 [0.896, 1.019]` | **`-25.9%`** |
| `random.bz2` | base / JCC | 4 | `236.8` / `238.2 ms` | `246.2` / `241.6 ms` (`A5`) | `1.041` / `1.061` *(8→13.7 MiB overshoot)* | `+4.8 / +2.3 MiB` |
| `zeros_100m.bz2` | base | 1 | `208.6 ms` / `183.6 MiB` | `105.7 ms` / `107.8 MiB` (`A6`) | **`0.495 [0.472, 0.519]`** | **`-41.3%`** |
| `zeros_100m.bz2` | JCC | 1 | `207.3 ms` / `183.6 MiB` | `104.5 ms` / `107.8 MiB` (`A6`) | **`0.513 [0.480, 0.549]`** | **`-41.3%`** |
| `zeros_100m.bz2` | base | 4 | `285.6 ms` / `234.4 MiB` | `244.9 ms` / `202.3 MiB` (`A5`) | **`0.857 [0.837, 0.878]`** | **`-13.7%`** |
| `zeros_100m.bz2` | JCC | 4 | `277.4 ms` / `234.4 MiB` | `251.7 ms` / `202.4 MiB` (`A5`) | **`0.892 [0.846, 0.940]`** | **`-13.7%`** |

- **Non-`Vec` `std::io::copy(&mut BzReader, &mut CountingSink)` on `wikidata.bz2` (`n = 5`, base):**
  - `t = 1`: `A1b_iocopy_sink_seq` (`25,829` calls of `8 KiB`) `859.0 ms` CPU / `36.6 MiB` vs `A4_fillbuf_drain_seq` (`432` calls) `870.8 ms` (`1.026 [0.957, 1.099]`) / `36.6 MiB`.
  - `t = 4`: `A1b_iocopy_sink_par` (`25,829` calls) `1000.0 ms` CPU / `93.9 MiB` vs `A4_fillbuf_drain_par` (`432` calls) `996.3 ms` (`1.005 [0.961, 1.051]`) / `94.8 MiB`.
  - **Takeaway:** Unlike Python's `RawIOBase.readall()` (`#1147`), Rust's `8 KiB` `io::copy` buffer fits in L1d and adds zero CPU overhead in pure Rust; `BufRead::fill_buf` still saves `60×` fewer `write(2)` syscalls when writing to OS files/pipes.
- **Why `A6` beats `A0` by `-49%..-51%` on `zeros_100m` and `-6.1%..-8.1%` on `wikidata` `t=1`:** Rust's `default_read_to_end` must zero-initialize spare capacity before passing `&mut [u8]` to `Read::read`, AND `Seq::read` copies out of an intermediate `Seq.out` (`46.6 MiB` on `zeros_100m`). `A6` decodes straight into the caller's `Vec<u8>`. Because `wikidata` falls below the frozen `>= 10.0%` bar (`-6.1%..-8.1%`), Axis A is scored **`STOP`** strictly under C2; `Seq::read_to_end` remains an attractive low-risk ~25-LOC zero-`unsafe` cleanup if `src/read.rs` is edited later.

---

## 5. Axis B: `decode()` & `decode_parallel()` Output `Vec` Growth (`src/lib.rs`)

| Corpus | Build | Mode | `t` | CPU med (ms) | CPU ratio `[95% CI]` | `VmHWM` (MiB) | `minflt` | `realloc`s | Logical move (MiB) | Final `cap` (MiB) |
|---|---|---|--:|--:|---|--:|--:|--:|--:|--:|
| `wikidata` | base | `B0_decode` (shipped, cap `0`) | 1 | `1001.7` | `1.000` | `228.0` | `59,853` | `9` | `215.4` | `219.7` |
| `wikidata` | base | `B1_decode_1x` | 1 | `950.4` | `0.955 [0.920, 0.992]` | `227.9` | `59,853` | `5` | `348.5` | `361.8` |
| `wikidata` | base | `B1_decode_adapt` | 1 | `967.1` | `0.967 [0.927, 1.008]` | `227.9` | `59,853` | **`1`** | **`1.7`** | `254.8` |
| `wikidata` | JCC | `B0_decode` (shipped, cap `0`) | 1 | `991.9` | `1.000` | `227.9` | `59,853` | `9` | `215.4` | `219.7` |
| `wikidata` | JCC | `B1_decode_1x` | 1 | `1040.9` | `1.039 [0.985, 1.095]` | `228.0` | `59,852` | `5` | `348.5` | `361.8` |
| `wikidata` | JCC | `B1_decode_adapt` | 1 | `1052.4` | `1.048 [0.995, 1.104]` | `228.0` | `59,852` | **`1`** | **`1.7`** | `254.8` |
| `wikidata` | base | `B0_decode_par` (shipped `4×`) | 4 | `1181.1` | `1.000` | `284.7` | `84,757` | `3` | `315.0` | `361.8` |
| `wikidata` | base | `B1_decode_par_1x` | 4 | `1168.0` | `0.989 [0.967, 1.012]` | `287.7` | `83,865` | `5` | `348.5` | `361.8` |
| `wikidata` | base | `B2_decode_par_concat` | 4 | **`1322.7`** | **`1.125 [1.092, 1.159]`** | **`442.7`** | **`131,162`** | `0` | `200.0` | `200.0` |
| `wikidata` | base | `B3_decode_par_512k_1x` | 4 | `1188.3` | `1.003 [0.966, 1.041]` | `285.2` | `84,665` | `5` | `348.5` | `361.8` |
| `wikidata` | JCC | `B0_decode_par` (shipped `4×`) | 4 | `1197.9` | `1.000` | `282.0` | `82,670` | `3` | `315.0` | `361.8` |
| `wikidata` | JCC | `B1_decode_par_1x` | 4 | `1185.3` | `0.988 [0.966, 1.011]` | `286.1` | `83,633` | `5` | `348.5` | `361.8` |
| `wikidata` | JCC | `B2_decode_par_concat` | 4 | **`1371.4`** | **`1.146 [1.123, 1.170]`** | **`441.5`** | **`130,543`** | `0` | `200.0` | `200.0` |
| `random` | base / JCC | `B0_decode_par` (`4×`, `30.7 MiB` cap) | 4 | `237.2` / `237.3` | `1.000` | `59.6` / `59.1` | `14,656` | `0` | `0.0` | `30.7` |
| `random` | base / JCC | `B1_decode_par_1x` (`7.7 MiB` cap) | 4 | `234.3` / `235.3` | `1.000` / `0.971` | `60.4` / `60.2` | `14,636` | `0` | `0.0` | `7.7` |
| `zeros_100m` | base | `B0_decode_par` / `B2_concat` | 4 | `237.8` / `233.9` | `0.998 [0.953, 1.046]` | `206.4` / `150.2` | `55,749` | `3` / `0` | `131.3` / `100.0` | `175.1` / `100.0` |

**Mechanism:** Linux glibc backs large `Vec`s with `mmap` and grows them via `mremap(MREMAP_MAYMOVE)` (zero physical byte copying, zero extra `minflt` or `VmHWM` even when `as_ptr()` moves) while demand-paging leaves unused `4×` virtual capacity unbacked. Holding all decoded chunks to concatenate once (`B2`) explodes `wikidata` `t=4` wall `+45.6..+48.4%`, CPU `+12.5..+14.6%`, and RSS `+55.5..+56.6%`. **Verdict: `STOP`.**

---

## 6. Axis C: Multi-Stream `.bz2` Framing & `Item::End` `syms` Drop (`src/stream.rs`, `src/block.rs`)

### 6.1 Thread/Pool Reuse Audit (`src/parallel.rs` + `src/stream.rs`)

Code audit (`src/stream.rs:150-162`, `src/parallel.rs:1204-1274`) confirms that across concatenated streams, `Scanner::next` transitions `State::Stream → State::Header { first: false } → State::Stream` inline in a single loop. `StreamDecoder` and `BzReader::parallel` **never** tear down or respawn worker threads, `Scratch` buffers, or `free_out` buffers across stream boundaries. On `50 × 512 KiB` and `200 × 256 KiB` Wikipedia-style multistream inputs at `t = 4`, multi-stream decode actually uses **less CPU and lower peak RSS** than single-stream equivalents of the exact same bytes:

| Workload (`t = 4`, shipped `decode_parallel`) | Multi-stream CPU / `VmHWM` | Single-stream twin CPU / `VmHWM` |
|---|---|---|
| `wiki_50` (`50 × 512 KiB` = `25.6 MiB`) | `212.4 ms` / `59.1 MiB` | `226.8 ms` / `66.4 MiB` |
| `wiki_200` (`200 × 256 KiB` = `51.2 MiB`) | `433.7 ms` / `81.4 MiB` | `440.9 ms` / `101.6 MiB` |
| `small_200` (`200 × 32 KiB` = `6.4 MiB`) | `74.5 ms` / `13.1 MiB` | `56.2 ms` / `18.0 MiB` |

### 6.2 The `Item::End` `1.80 MiB` `syms` Drop (`src/block.rs:195`, `src/stream.rs:161`) & Allocator Dependence

`block::read_item_with_scratch` takes `syms: Vec<u16>` (capacity `MAX_SYMS = 900,100` = `1.80 MiB`) by value and returns `Ok(Item::End(crc))` without returning `syms`, invoking `free@GLIBC_2.2.5` (verified in `objdump -d`) on every stream boundary, followed by `syms = Vec::new()` at `src/stream.rs:161` and `syms.reserve(MAX_SYMS)` on the next stream's first block.

| Allocator regime | Corpus (`t = 1`, `n = 5`) | Mode | Shipped `C0` CPU / `minflt` | Keep-`syms` `C1` CPU / `minflt` | CPU ratio `[95% CI]` | `minflt` delta |
|---|---|---|---|---|---|---:|
| **Default Linux glibc** (dynamic `M_MMAP_THRESHOLD`) | `wiki_50` (`50 × 512K`) | scan | `11.4 ms` / `109` | `11.4 ms` / `80` | `1.004 [0.979, 1.031]` | `-26.6%` |
| Default Linux glibc | `wiki_200` (`200 × 256K`) | scan | `28.8 ms` / `71` | `28.7 ms` / `54` | `0.979 [0.913, 1.050]` | `-23.9%` |
| Default Linux glibc | `small_200` (`200 × 32K`) | scan | `9.5 ms` / `32` | `9.3 ms` / `29` | `0.988 [0.949, 1.028]` | `-9.4%` |
| Default Linux glibc | `wiki_200` (`200 × 256K`) | decode `t=1` | `323.1 ms` / `2,232` | `326.2 ms` / `3,174` | `1.014 [1.001, 1.028]` | `+42.2%` |
| **`MALLOC_MMAP_THRESHOLD_=131072`** (fixed threshold) | `wiki_50` (`50 × 512K`) | scan | `16.3 ms` / `1,651` | `11.1 ms` / `81` | **`0.677 [0.664, 0.689]`** | **`-95.1%`** |
| Fixed threshold (`128 KiB`) | `wiki_200` (`200 × 256K`) | scan | `45.8 ms` / `4,033` | `30.2 ms` / `55` | **`0.660 [0.651, 0.670]`** | **`-98.6%`** |
| Fixed threshold (`128 KiB`) | `small_200` (`200 × 32K`) | scan | `14.2 ms` / `922` | `9.2 ms` / `30` | **`0.644 [0.627, 0.661]`** | **`-96.7%`** |
| Fixed threshold (`128 KiB`) | `wiki_200` (`200 × 256K`) | decode `t=1` | `363.9 ms` / `7,435` | `331.7 ms` / `3,456` | **`0.933 [0.900, 0.969]`** | **`-53.5%`** |
| Fixed threshold (`128 KiB`) | `small_200` (`200 × 32K`) | decode `t=1` | `64.1 ms` / `1,087` | `59.1 ms` / `194` | **`0.914 [0.900, 0.929]`** | **`-82.2%`** |

**Verdict:** **`STOP`** on default Linux glibc (glibc ratchets `M_MMAP_THRESHOLD` above `1.80 MiB` after stream 0, masking streams `1..199`). However, returning `syms` inside `Item::End` (or peeking `END_MAGIC` in `Scanner`) is a tiny 3-line cleanup worth applying opportunistically whenever `src/block.rs` / `src/stream.rs` is next touched, because it eliminates **`-32%..-36%` scan CPU / `-95%..-99%` page faults** on musl, macOS, and fixed-threshold containerized allocators.

---

## 7. Axis D: `src/block.rs` Framing & `src/bits.rs` Disassembly Audit

- **Sub-stage share of `read_item_with_scratch` (pinned CPU 0):**
  - Baseline x86-64: pure header/framing (`magic`, `crc`, `orig_ptr`, `in_use`, `selectors`, `codelens`) is **`1.0%`** on `wikidata` (`4.66 / 472.63 ms`), **`0.8%`** on `text` (`1.34 / 171.60 ms`), **`0.9%`** on `random` (`1.89 / 216.94 ms`), **`1.4%`** on `multistream_wiki_200` (`2.19 / 155.96 ms`), and **`1.5%`** on `multistream_small_200` (`1.01 / 66.18 ms`).
  - JCC-32B-aligned: **`1.8%`** on `text`, **`4.7%`** on `random`, **`2.3%`** on `multistream_wiki_200`, **`3.7%`** on `multistream_small_200` (all `< 5.0%` frozen bar; A2 selector/code-length/table-build work is already owned by `#1117`, and A3 symbol decoding by `#1153`).
- **`objdump -d` audit (`/tmp/read-io-probe-base`):**
  - `mibz2::block::read_item_with_scratch`: `1,520` insns, **`vzero = 0`**, `1` cold `panic_bounds_check`, `1` `free@GLIBC_2.2.5` (the `Item::End` `syms` drop), `1` `memset@GLIBC_2.2.5` (`seq_to_unseq`/`lens` stack init), `18` cold `Reg::refill_slow` calls.
  - `mibz2::block::decode_runs_bmi2`: `2,523` insns, **`vzero = 0`** (clean BMI2 `shlx`/`shrx` with zero AVX2 transition penalty), `1` cold `panic_bounds_check`, `8` cold `Reg::refill_slow` calls.
  - `<BzReader<Cursor<Vec<u8>>> as Read>::read`: `112` insns, **`vzero = 0`**, `0` `panic_bounds_check`, clean dispatch to `Seq::read` / `StreamDecoder::read`.
- **Verdict: `STOP`.** No unowned `>= 5%` framing/refill/codegen defect exists in `src/block.rs` or `src/bits.rs`.
