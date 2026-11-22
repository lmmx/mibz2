# Carbon Cycle 41 (`3/20`) — Push-Streaming `StreamDecoder::parscan` over `#739` (`SlabRope`, Seam-Invariant Anchor Finder, and `VmHWM` Audit)

**Date:** 2026-11-22  
**Branch:** `research/carbon-word-transport`  
**Base (`origin/master`):** `77c41fc7` (`src/` at `3296195e`, Wormhole `#739` plateau stack) + `#742` adopted block reader (`7b901b67`)  
**Safety:** `#![forbid(unsafe_code)]` on `src/par_scan.rs`, `src/huff.rs`, `src/block.rs`

---

## 1. Context & Architecture (`C41`)

Following the i9 results on Interim Handoff 0 (`Decision 2, BLOCK-READER` **ADOPTED** at `0.977x` 1T / `0.884x` 8T / `0.932x` 10T, RSS `<= 1.02x`; `Decision 1, PARSCAN` **NOT ADOPTED** due to `1.111x` slower at 6T and whole-slice + `4*threads` memory reaching `1.28x` at 10T and `1.99x` at 20T) and the merge of Wormhole's `#739` plateau stack (`3296195e`, `0.63x` peak RSS), Cycle 41 moves parallel scanning **inside `StreamDecoder`** (`StreamDecoder::parscan`, `BzReader::parscan`, `decode_parscan`):

1. **Zero-Copy Bounded `SlabRope` (`Arc<Vec<u8>>`):**
   - Incoming `4 MiB` chunks (`BzReader`) are wrapped in `Arc<Vec<u8>>` with zero byte copies; small `< 64 KiB` writes coalesce in place via `Arc::get_mut`.
   - As `chain_bit` advances past `front.end()`, slabs are popped and unwrapped via `Arc::try_unwrap` straight back into `State::free_in` (`StreamDecoder::spare`), eliminating `ChannelSource::buf`'s duplicate `8-16 MiB` sliding copy buffer.
2. **Chunk-Invariant Disjoint 4-Byte Anchor Seam Scanner:**
   - Scans global 4-byte anchors `p = 4 * m` using the `8 KiB` `ANCHOR_BITMAP` (`4x` unrolled interior loop + `12 B` cross-slab seam window), proven invariant across `1 B .. 4 MiB` chunkings (`slab_rope_seam_invariance_across_chunk_sizes`).
3. **Next-Magic Horizon + Zero-Level-Guess (`C39a`) Feeding `#739`'s `Jobs` -> `decode_owned` -> `Ready` Ring + `12 MiB` `READY_BYTES` Cap:**
   - Parallel Huffman scanning produces verified `(place, bit, symbols)` into `#739`'s `Jobs` queue with monotonic `batch.blocks_issued` and `pending_publishes` drain fencing, so every BWT inversion uses `#739`'s exact `decode_owned` (`invert_runs` into pooled `out`, `unrle_in_place`, tiled CRC, `Ready` ring, and `READY_BYTES` cap) and `trimmed_here` i9 worker counts.
4. **Tight `syms` Allocation (`read_item_tight`):**
   - Reserves `(sel_cap * 50 + 66).min(MAX_SYMS)` (`~200-350 KiB` on text) instead of `MAX_SYMS = 1.80 MiB` per slot, while preserving `MAX_SYMS` on `Scanner`'s `read_item_with_scratch` for `trace::tests` ledger parity.

Artifact saved at `docs/journal/2026-11-22-carbon-cycle41/c41_par_scan.rs`.

---

## 2. Correctness Verification

- `par_scan::tests::slab_rope_seam_invariance_across_chunk_sizes`: **PASS** (`1, 2, 3, 4, 5, 7, 8, 13, 64, 513, 2048 B` chunkings exact).
- `tests/par_scan.rs` (`6/6` integration suites: `valid_streams_of_every_shape`, `streaming_chunks_and_i9_dispatch`, `slow_reader_backpressure_without_deadlock`, `false_magics_and_headers`, `mutated_files` x600, `a_panic_in_the_callback_reaches_the_caller` with `CancelGuard`): **PASS**.
- `c41_bench_and_verify verify`:
  - `PASS 1`: `wikidata_x3` (`36 MiB` compressed, `9 x 4 MiB` chunks, 3 streams) `BzReader::parscan` (`1..20T`) + `decode_parscan` exact.
  - `PASS 2`: `1 B`, `7 B`, `63 B`, `4 KiB`, `65,537 B`, `1 MiB` multi-level (`1..=9`) + planted-magic chunking exact.
  - `PASS 3`: Slow-reader backpressure & `read_block` recycling deadlock-free (`2, 6, 16T`).
  - `PASS 4`: `cancel()` and `PanickingReader` unwind safety exact.
  - `PASS 5`: **`1,500 / 1,500`** streaming chunked mutation fuzz cases exact on both prefix bytes emitted before error and `Error { kind, block, bit }` (caught and fixed an out-of-order `publish_batch` `sst.issued`/`sst.end` race on `case 18` during development).

---

## 3. Subprocess Peak RSS (`VmHWM`) Audit on `wikidata_x3.bz2` (`36 MiB` in -> `629 MB` out)

Median of 3 interleaved child-process runs (`Master` = `3296195e` `#739` `BzReader::parallel`; `Cand` = `BzReader::parscan`):

| Threads | `i9` (`Machine::service_target()`) Master MiB | `i9` Cand MiB | `i9` Ratio | `native` (2-core untrimmed) Master MiB | `native` Cand MiB | `native` Ratio |
|---:|---:|---:|---:|---:|---:|---:|
| `1` | `28.73` | `28.51` | **`0.9925x`** | `28.73` | `28.46` | **`0.9906x`** |
| `2` | `67.64` | `57.05` | **`0.8434x`** | `63.91` | `61.07` | **`0.9554x`** |
| `4` | `49.08` | `41.71` | **`0.8499x`** | `97.02` | `102.59` | `1.0574x` |
| `6` | `56.64` | `48.95` | **`0.8643x`** | `128.88` | `123.23` | **`0.9562x`** |
| `8` | `59.86` | `56.41` | **`0.9425x`** | `62.74` | `56.63` | **`0.9026x`** |
| `10` | `59.81` | `52.86` | **`0.8838x`** | `64.96` | `66.28` | `1.0203x` |
| `12` | `60.29` | `51.41` | **`0.8527x`** | `67.59` | `71.85` | `1.0630x` |
| `16` | `58.81` | `51.70` | **`0.8792x`** | `75.23` | `87.95` | `1.1690x` |
| `20` | `58.92` | `52.71` | **`0.8947x`** | `84.27` | `104.76` | `1.2431x` |

- Under `Machine::service_target()` (`i9` profile), peak RSS is **`0.843x–0.943x` (`6–11 MiB` BELOW `#739` `master` across all `2–20` threads)**, eliminating Handoff 0's `1.28x–1.99x` memory blowup.
- Under `native` untrimmed `16–20` workers, RSS reached `1.169x / 1.243x` because every assisting worker retained a `2 MiB` `stitch_buf` on `NeedMore` and `allow_spec = slots` (`64–80`) queued up to `80` `Symbols` ahead of `READY_BYTES`.

---

## 4. Same-Binary Push-Streaming A/B + A/A (`2`-Core Xeon Platinum 8481C VM, 5 Interleaved Rounds)

`MasterA` / `MasterB` = `origin/master` `3296195e` (`#739`); `MasterBR` = `7b901b67` (`#739 + #742`); `Cand` = `C41` `BzReader::parscan`.

### `wikidata-l9` (`209.70 MB` out; Serial-stage CPU share: Master `16.92%` (`80.74 ms`) -> Cand `0.71%`; Modeled 10T crit `0.7139x`)

| Threads | MasterA (ms) | A/A (`MasterB/A`) | `Cand / MasterA` | `Cand / MasterBR` | CPU Ratio (`Cand/A`) |
|---:|---:|---:|---:|---:|---:|
| `1` | `775.52` | `0.9843x` | **`0.9734x`** | `1.0047x` | `0.9810x` |
| `2` | `449.33` | `1.0098x` | **`0.9886x`** | `1.0072x` | `0.9737x` |
| `4` | `464.16` | `1.0206x` | **`0.9838x`** | `1.0005x` | `0.9852x` |
| `6` | `488.53` | `0.9809x` | **`0.9814x`** | `1.0058x` | `0.9882x` |
| `8` | `449.98` | `1.0036x` | `1.0595x` | `1.0600x` | `0.9963x` |
| `10` | `475.49` | `0.9759x` | `1.0325x` | `1.0659x` | `0.9885x` |
| `12` | `475.55` | `0.9662x` | `1.0556x` | `1.0568x` | `0.9943x` |
| `16` | `484.52` | `0.9650x` | `1.0478x` | `1.0822x` | `0.9875x` |
| `20` | `474.60` | `1.0008x` | `1.0741x` | `1.0763x` | `1.0046x` |

### `wikidata-l1` (`36.00 MB` out; Serial-stage CPU share: Master `23.92%` (`29.83 ms`) -> Cand `0.65%`; Modeled 10T crit `0.4886x`)

| Threads | MasterA (ms) | A/A | `Cand / MasterA` | `Cand / MasterBR` | CPU Ratio |
|---:|---:|---:|---:|---:|---:|
| `1` | `162.38` | `1.0054x` | **`0.9448x`** | `0.9984x` | `0.9448x` |
| `2` | `96.05` | `1.0155x` | `1.0352x` | `1.0575x` | `0.9687x` |
| `4` | `97.14` | `1.0159x` | `1.0405x` | `1.0940x` | `0.9559x` |
| `6` | `96.11` | `1.0169x` | `1.1007x` | `1.1207x` | `0.9685x` |
| `8` | `99.50` | `1.0158x` | `1.0286x` | `1.0809x` | `0.9666x` |
| `10` | `99.28` | `0.9874x` | `1.0417x` | `1.0768x` | `0.9722x` |
| `16` | `103.37` | `0.9774x` | `1.0267x` | `1.0831x` | `0.9692x` |
| `20` | `102.20` | `1.0336x` | `1.0655x` | `1.0850x` | `0.9849x` |

### `records-l9` (`9.60 MB` out; Serial-stage CPU share: Master `23.77%` (`37.95 ms`) -> Cand `1.58%`; Modeled 10T crit `0.5086x`)

| Threads | MasterA (ms) | A/A | `Cand / MasterA` | `Cand / MasterBR` | CPU Ratio |
|---:|---:|---:|---:|---:|---:|
| `1` | `173.36` | `0.9992x` | **`0.9683x`** | `0.9985x` | `0.9676x` |
| `2` | `105.48` | `0.9837x` | **`0.9922x`** | `1.0343x` | `0.9852x` |
| `4` | `110.35` | `1.0331x` | **`0.9789x`** | `1.0066x` | `1.0046x` |
| `6` | `116.22` | `0.9956x` | **`0.9813x`** | **`0.9938x`** | `0.9747x` |
| `8` | `120.54` | `0.9895x` | **`0.9887x`** | `1.0298x` | `1.0077x` |
| `10` | `120.85` | `1.0113x` | `1.0390x` | `1.0455x` | `1.0225x` |
| `16` | `122.39` | `1.0127x` | `1.0401x` | `1.0490x` | `1.0262x` |
| `20` | `117.53` | `0.9820x` | `1.0740x` | `1.0902x` | `1.0230x` |

---

## 5. Keep / Kill Decisions & Cycle 42 Targets

- **KEEP (`C41` core):** Zero-copy `SlabRope` + seam-invariant anchor scanner + monotonic `CommitBatch` inside `StreamDecoder::parscan` (`0.84x–0.94x` i9 peak RSS; `0.973x–0.989x` `1T–6T` `wikidata-l9` wall; `1,500/1,500` fuzz exact).
- **KILL (unbounded `allow_spec = slots` worker scan-assist stampede + `PARSCAN_INPUT_QUEUE = 1` + retained `2 MiB` `stitch_buf`):** Waking up to `16–20` workers simultaneously onto `hub.mu` caused `10` lock acquisitions/block, preempted `cands[0]` behind out-of-order speculative scans (`+3–7%` idle wall bubble at `8–20W` and on `wikidata-l1` despite `2–5%` *lower* CPU!), and retained `2 MiB` `stitch_buf`s (`1.17x–1.24x` untrimmed `16–20W` native RSS).
- **Cycle 42 (`4/20`) Focus:** Cap concurrent Huffman scanners (`1..3`), restore `INPUT_QUEUE = 2` with proactive `<= 4 MiB` slab pre-staging, drop >`256 KiB` `stitch_buf`s + cap front lookahead to `workers + 3` (`<= 1.00x` native RSS at `16–20W`), and eliminate redundant `shared.lock()` round-trips in `try_assist_one`.
