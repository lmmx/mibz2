# Carbon Cycle 38 — Word-Accelerated Header/Selector/Delta Scan, Bounded-Window Streaming `ParScan`, & Column `mtf` Kill (`0.717×` `L1 ScanOnly`, `0.574×` `10T` Streaming)

**Baseline:** `origin/master` (`53f7ecf9`), Cycle 36 `1T` scan champion (`C36a`), and Cycle 37 `10T` slice champion (`C37`).
**Candidate:** `docs/journal/2026-11-21-carbon-cycle38/c38_hdr_batch_and_stream_parscan.rs` (`#![forbid(unsafe_code)]`).

## 1. Constructions Tested (`C38a`, `C38b`, `C38c`)

1. **`C38a` (`ScannerC38a` / `decode_parscan_c38a` — Word-Accelerated Block Header, `8×` Batched Selector MTF, & `lzcnt` Zero-Delta Code-Length Run-Skipping):**
   - **`in_use` `trailing_zeros` Unpack:** Reverses the 16-bit top-level `in_use16` mask and steps directly over active 16-byte symbol buckets via `trailing_zeros()`.
   - **`8×` Batched Unary Selector MTF:** Since `n_groups <= 6`, each selector consumes `1..=6` bits, so a single `Reg::refill` (`live >= 56`) guarantees `8` complete selectors (`48 <= 56` bits) with zero per-selector refill, `MAX_SELECTORS`, or `overran` branches whenever `r.pos + 16 <= data.len()`.
   - **`lzcnt` Zero-Delta Run-Skipping:** In bzip2's unary delta code-length table, consecutive `0` bits mean consecutive alphabet symbols share the current code length `curr`. When `1 <= curr <= 20`, `z = r.buf.leading_zeros()` fills `run0 = z.min(alpha - sym_idx).min(32)` symbols in one `slice::fill(curr as u8)` + `skip_short(run0)`, then steps through `10` (`+1`) / `11` (`-1`) pairs directly in `r.buf`.
2. **`C38b` (`decode_stream_parscan_c38b` — Bounded `4 MiB` Window Streaming Parallel Front End over `R: std::io::Read`):**
   Extends Cycle 37's zero-serial-thread Disjoint 4-Byte Anchor parallel front end from `&[u8]` slices to arbitrary `R: std::io::Read` streams using a single bounded `4 MiB` buffer shared zero-copy via `Arc<Vec<u8>>` across persistent workers and reclaimed across windows with zero allocations via `Arc::try_unwrap`.
3. **`C38c` (`mtf_safe_c38c` — `#![forbid(unsafe_code)]` `(lo, hi)` + Seward `SlidingTail240` Column `mtf`):**
   Tested whether safe-Rust GP-register top-16 + `SlidingTail240` can beat `master`'s `unsafe` AVX2 `mtf_avx2` (`pshufb` + 7 YMM rows) on dense non-run corpora (`records-l9`, `random-l9`).

## 2. Native Safe-Rust A/B + A/A Results (Same Binary vs `origin/master` `53f7ecf9`)

### 2a. Gate 1 — `ScanOnly` (`15` Rounds) & `1T` `Cells` (`9` Rounds): `C38a` vs `C36a`/`C37` & `Master`

| Corpus & Mode | `MasterA` (ms) | `MasterB` (ms) (`A/A`) | `C36a`/`C37` (ms) | **`C38a` (ms)** | **`A/B` vs `MasterA`** | vs `C36a`/`C37` | Verdict |
|---|---:|---:|---:|---:|---:|---:|---|
| `wikidata-l9` `ScanOnly` | 74.79 | 74.89 (`1.0013×`) | 60.58 (`0.8101×`) | **60.40** | **`0.8076×` (`-19.24%`)** | `0.9969×` | **KEEP** |
| `wikidata-l9` `1T Cells` | 471.65 | 474.60 (`1.0062×`) | 463.41 (`0.9825×`) | **457.73** | **`0.9705×` (`-2.95%`)** | `0.9878×` | **KEEP** |
| **`wikidata-l1` `ScanOnly`** | **28.50** | **28.54 (`1.0013×`)** | **20.84 (`0.7311×`)** | **20.44** | **`0.7171×` (`-28.29%`)** | **`0.9809×`** | **NEW 1T CHAMPION** |
| **`wikidata-l1` `1T Cells`** | **113.28** | **113.65 (`1.0033×`)** | **105.36 (`0.9301×`)** | **104.11** | **`0.9191×` (`-8.09%`)** | **`0.9882×`** | **NEW 1T CHAMPION** |

### 2b. Gate 2 — `10`-Thread Streaming (`io::Read`, `4 MiB` Window `C38b`) & Whole-Slice (`C38a`) on `wikidata-l9` (`11` Rounds)

| Metric (`W = 10`) | `StreamMaster A/B` (ms) (`A/A`) | `Slice C37` (ms) | **`Slice C38a` (ms)** | **`Stream C38b` (`4 MiB` `io::Read`)** | Verdict |
|---|---:|---:|---:|---:|---|
| Serial Front/IO CPU | `88.96 / 88.81` (`0.9982×`) | `0.01` (`0.0001×`) | **`0.01` (`0.0001×`)** | **`3.29` (`0.0370×`, `-96.30%`)** | **KEEP** |
| Per-Worker Avg CPU (`W/10`) | `43.66 / 43.75` (`1.0019×`) | `50.75` (`1.1622×`) | **`50.46` (`1.1556×`)** | **`51.03` (`1.1687×`)** | **KEEP** |
| **`10T CritPath max(S, W/10)`** | **`88.96 / 88.81` (`0.9982×`)** | `50.75` (`0.5704×`) | **`50.46` (`0.5672×`, `-43.28%`)** | **`51.03` (`0.5736×`, `-42.64%`)** | **KEEP BOTH** |
| **`10T TotalCPU` (all threads)** | **`527.53 / 527.28` (`0.9995×`)** | `509.13` (`0.9651×`) | **`506.38` (`0.9599×`, `-4.01%`)** | **`515.66` (`0.9775×`, `-2.25%`)** | **KEEP BOTH** |
| **`10T VM Wall` (2-vCPU)** | **`272.67 / 274.78` (`1.0077×`)** | `257.53` (`0.9445×`) | **`255.53` (`0.9372×`, `-6.28%`)** | **`263.10` (`0.9649×`, `-3.51%`)** | **KEEP BOTH** |

### 2c. Gate 3 — Dense Column `mtf::mtf` Stage Confrontation (`15` Rounds)

| Corpus | `mtf_avx2 A/B` (ms) (`A/A`) | `mtf_scalar` (ms) | `C38c` `mtf_safe` (ms) | `C38c / avx2` | Verdict |
|---|---:|---:|---:|---:|---|
| `records-l9` (`11` blks, `9.60 MB`) | `77.26 / 77.13` (`0.9984×`) | `111.85` (`1.4477×`) | `249.01` | `3.2232×` | **KILL `C38c`** |
| `random-l9` (`9` blks, `8.00 MB`) | `64.23 / 64.21` (`0.9997×`) | `93.83` (`1.4608×`) | `209.17` | `3.2564×` | **KILL `C38c`** |

**Parity:** `826/826` blocks symbol-exact across all 5 corpora; `StreamParScanC38b` byte-exact at `256 KiB`, `512 KiB` multi-stream concat, and `4 MiB` windows + bit-exact truncated `Error` `(kind, block, bit)` parity.

## 3. Kill Decisions

- **`C38c` `mtf_safe_c38c` (`3.22×–3.26×` slower than `mtf_avx2`, `2.23×` slower than `mtf_scalar` on `records-l9`/`random-l9`):** Killed because on dense high-rank alphabets (`nn` uniform across `0..256`), Seward's `SlidingTail` `while u > 1` bucket cascade suffers data-dependent loop-trip branch mispredictions on every symbol whereas `mtf_avx2` updates `f0`/`f1` with `pshufb` and blends 7 fixed YMM rows branchlessly.
