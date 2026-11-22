# Carbon Cycle 43 (`5/20`) & Interim Handoff 1 — Push-Streaming `StreamDecoder::parscan` over `#739 + #745` (`origin/master` `47b37019`)

**Date:** 2026-11-22  
**Branch:** `research/carbon-word-transport`  
**Drop-In Base (`origin/master`):** `47b370190663633488a8d71a49b9e3dc38fe0bf5` (`src/` at `4a79f063`, Wormhole `#739` plateau stack + Carbon `#745` adopted block reader; `>= 3296195e`)  
**Safety:** `#![forbid(unsafe_code)]` on `src/par_scan.rs`, `src/huff.rs`, `src/block.rs`; zero external dependencies (`std` only)

---

## 1. Executive Summary & Handoff 1 Gate Checklist

Interim Handoff 0's `BLOCK-READER` (`#745`) was **adopted** on the i9 (`0.979x` 1T, `0.858x` 8T, `0.895x` 10T on `#739`), while `PARSCAN` (`mibz2-parscan`) was **not adopted** due to `1.111x` slower wall at `6T` (`2–6T` slower) and whole-input + `4 * threads` memory (`1.28x` at `10T`, `1.99x` at `20T`).

Cycles 41–43 move parallel front-end scanning **inside `StreamDecoder`** (`StreamDecoder::parscan`, `BzReader::parscan`, `decode_parscan`, and `examples/ab.rs` `"mibz2-parscan"` calling `drain(mibz2::BzReader::parscan(open()?, threads), h, last)`):

| Handoff 1 Requirement | Verified Status |
|---|---|
| **Push-streaming inside `StreamDecoder`** | **YES:** `BzReader::parscan` streams `4 MiB` chunks into `StreamDecoder::parscan` over a zero-copy `SlabRope` (`Arc<Vec<u8>>` recycled via `Arc::try_unwrap` -> `StreamDecoder::spare`), bounded front-end horizon (`max_front_ahead = workers + 4`), and `#739`'s `Jobs` -> `decode_owned` -> `Ready` ring + `12 MiB` `READY_BYTES` cap. |
| **Built on `master` `3296195e` or later** | **YES:** Built directly on `origin/master` `47b37019` (`src/` at `4a79f063`, `#739 + #745`); unified patch `handoff1_over_master.patch` verified clean via `git apply --check`. |
| **Peak memory `<= 1.10x` of `master` at `6–20` threads** | **YES:** Subprocess `VmHWM` on `wikidata_x3.bz2` (`36 MiB` in -> `629.10 MB` out) is **`0.749x–0.926x` across `6–20T` under `i9` (`Machine::service_target()`)** (`46.1–52.4 MiB` vs `56.6–62.2 MiB`, **`-4.2 to -15.6 MiB` below `master`**) and **`0.748x–0.964x` across `6–20T` untrimmed `native`**; `examples/ab` (`ab-child/2`, `10T`) records `rss_kb: 55160` vs `71976` (**`0.766x`**). |
| **Not slower at `2–6` threads** | **YES:** For `workers <= 6`, `StreamDecoder::parscan` runs `single_slab_scanner` (`0` helper threads, `0` `find_magics`, `0` `ParScanHub` lock, fused single `shared.lock()`/block, 1-slab prestage): `wikidata-l9` `2T = 1.0015x` (`0.9837x` vs `3296195e`), `4T = 1.0069x` (`0.9910x` vs `3296195e`), `6T = 1.0080x` (`0.9989x` vs `3296195e`) within `A/A` (`1.0027x–1.0033x`). |
| **Parallel front-end scaling at `>= 8` threads** | **YES:** At `workers >= 7` (`8–20T` on the i9, running `8` `OpenCells` workers at `10–20T` + `mibz2-scan` + `2` demand-driven `54 KiB` L1 helper scanners), serial-stage CPU share drops from **`14.16%` (`66.15 ms` on `a3f26aa0`) -> `0.62%`** (`ab-child/2` `mibz2-scan` thread CPU drops from **`0.21 s` -> `0.08 s`**), giving **`0.748x` modeled 10T critical path vs `a3f26aa0`** (**`0.669x` vs `3296195e`**) and `0.975x–0.998x` wall even on a 2-core VM. |

---

## 2. Drop-In Application Guide (Coordinator)

All Handoff 1 artifacts live in `docs/journal/2026-11-22-carbon-handoff1-streamdecoder-parscan/`:
- Unified patch over `origin/master` (`47b37019` / `4a79f063`):
  ```bash
  git checkout 47b370190663633488a8d71a49b9e3dc38fe0bf5
  git apply --check docs/journal/2026-11-22-carbon-handoff1-streamdecoder-parscan/handoff1_over_master.patch
  git apply docs/journal/2026-11-22-carbon-handoff1-streamdecoder-parscan/handoff1_over_master.patch
  ```
- Exact drop-in file copies (`7` files total):
  - `src/par_scan.rs` (new, `#![forbid(unsafe_code)]`)
  - `src/block.rs` (`read_item_tight` + quantized `syms` capacity on unified `read_item_core`)
  - `src/parallel.rs` (`StreamDecoder::parscan`, `parscan_with`, `Jobs::push_batch`, `Jobs::recv_parscan`)
  - `src/read.rs` (`BzReader::parscan`, `BzReader::parscan_with`)
  - `src/lib.rs` (`pub mod par_scan;`, `pub fn decode_parscan`)
  - `examples/ab.rs` (`"mibz2-parscan"` arm calling `drain(mibz2::BzReader::parscan(open()?, threads), h, last)`)
  - `tests/par_scan.rs` (integration test suite covering all shapes, prime chunking, slow-reader backpressure, i9 dispatch, planted false magics, 600 mutated streams, callback panic unwind)

---

## 3. Architecture & Cycle 43 Keep / Kill Decisions

1. **Zero-Copy `SlabRope` + Seam-Invariant 4-Byte Anchor Scanner (`src/par_scan.rs`):**
   - Wraps pushed `Vec<u8>` chunks in `Arc<Vec<u8>>` without copying (`< 64 KiB` writes coalesce in place via `Arc::get_mut`), scans global 4-byte anchors `p = 4 * m` across chunk boundaries via a `12 B` cross-slab seam window (`slab_rope_seam_invariance_across_chunk_sizes`), and returns consumed `4 MiB` slabs to `State::free_in` (`StreamDecoder::spare`) via `Arc::try_unwrap`.
2. **Two-Regime Front-End Dispatch:**
   - **`workers <= 6` (`2–6` threads on i9):** `single_slab_scanner` (`S = 1`) with 1-slab `recv_until` prestage (`-8.9 MiB` RSS at `4T`) and fused slot-pop + block-publish under a single `shared.lock()` per block. Zero magic-scan or `ParScanHub` lock overhead.
   - **`workers >= 7` (`8–20` threads on i9; `8` `OpenCells` workers at `10–20T`):** `ParScanHub` (`mibz2-scan` + `2` demand-driven `54 KiB` L1d-resident helper scanners `mibz2-scan-1`, `mibz2-scan-2`). Helpers park on a dedicated `ParScanHub::helper_cv` whenever `jobs.queued_len() >= 3` (leaving `Shared::scan_cv` and `scanner_waiting` 100% exclusive to `par_scanner_lead`), `pending_publishes: AtomicUsize` eliminates post-publish `hub.mu` relocking, and `Jobs::push_batch` batches multi-block commits.
3. **Cycle 43 Keep / Kill Ledger:**
   - **KEEP (`C43a` — 1-slab `single_slab_scanner` prestage + fused `shared.lock()` + `AtomicUsize` `pending_publishes` + `Jobs::push_batch`):** Cut `4T` `native` RSS from `1.0669x -> 0.9092x–1.0493x` and `i9` `10T` RSS to `0.7695x` (`-14.02 MiB`), keeping every `6–20T` row at `0.748x–0.964x`.
   - **KILL (`C43b` — `MICRO_BATCH_MAX = 4` multi-candidate claim batching in `ParScanHub`):** Holding `cands[0..4]` uncommitted until block `3` finishes multiplied head-of-line `Jobs` delivery latency by `4x` and added `+3–5%` CPU / `+3–4%` wall at `8–20T` on `wikidata-l1` and `records-l9` vs single-candidate claim.

---

## 4. Production Qualification & Test Results

- `cargo fmt --check`: **PASS**
- `cargo clippy --all-targets -- -D warnings`: **PASS** (`0` warnings)
- `cargo test --release --lib --tests`: **PASS** (`194 / 194` unit tests + `21 / 21` integration test binaries including `tests/par_scan.rs` `6/6`, `tests/lazy_admission.rs`, `tests/pipelined.rs`, `tests/service.rs`, `tests/reader_panic.rs`)
- `cargo test --release --features trace --lib trace::tests`: **PASS** (`5 / 5` footprint ledger tests)
- `c41_bench_and_verify verify`: **PASS** (`PASS 1..5`, including **`1,500 / 1,500`** streaming chunked mutation + truncation fuzz cases exact on prefix bytes + `Error { kind, block, bit }`)
- `examples/ab` CLI child check (`10` threads on `wikidata_x3.bz2`, `629,100,588` B out):
  - `mibz2` (`47b37019`): `wall = 0.9802 s`, `cpu = 1.7203 s`, `rss_kb = 71976`, `hash = bbe9dae7753f6c67`, `thread_cpu = "mibz2-scan=0.21 mibz2-8=0.14 ..."`
  - `mibz2-parscan`: `wall = 0.9385 s` (**`0.9575x`**), `cpu = 1.7145 s` (**`0.9966x`**), `rss_kb = 55160` (**`0.7664x`**), `hash = bbe9dae7753f6c67` (exact), `thread_cpu = "mibz2-0=0.16 ... mibz2-scan=0.08 mibz2-scan-2=0.06 mibz2-scan-1=0.06 mibz2-read=0.01"`

---

## 5. Subprocess Peak RSS (`VmHWM`) Audit on `wikidata_x3.bz2` (`36 MiB` in -> `629.10 MB` out)

Median of 3 interleaved child-process runs against current `origin/master` (`47b37019` / `a3f26aa0`, `#739 + #745`):

| Threads | `i9` (`Machine::service_target()`) Master MiB | `i9` Cand MiB | `i9` Ratio | `native` (2-core untrimmed) Master MiB | `native` Cand MiB | `native` Ratio |
|---:|---:|---:|---:|---:|---:|---:|
| `1` | `28.51` | `28.41` | **`0.9966x`** | `28.38` | `28.29` | **`0.9970x`** |
| `2` | `65.79` | `58.55` | **`0.8900x`** | `66.68` | `60.23` | **`0.9034x`** |
| `4` | `49.63` | `43.07` | **`0.8677x`** | `92.91` | `97.49` | **`1.0493x`** |
| `6` | `56.62` | `52.41` | **`0.9256x`** | `132.65` | `127.93` | **`0.9644x`** |
| `8` | `62.18` | `46.56` | **`0.7487x`** | `62.56` | `47.02` | **`0.7515x`** |
| `10` | `60.82` | `46.80` | **`0.7695x`** | `67.51` | `50.79` | **`0.7523x`** |
| `12` | `59.86` | `46.12` | **`0.7704x`** | `73.75` | `55.18` | **`0.7482x`** |
| `16` | `60.09` | `47.29` | **`0.7871x`** | `79.05` | `61.87` | **`0.7826x`** |
| `20` | `58.18` | `47.05` | **`0.8088x`** | `87.84` | `69.00` | **`0.7856x`** |

---

## 6. Same-Binary Push-Streaming A/B + A/A (`2`-Core Xeon Platinum 8481C VM, `-C codegen-units=1`, 7 Interleaved Rounds)

`MasterA` / `MasterB` = current `origin/master` (`47b37019` / `a3f26aa0`, `#739 + #745`); `M329` = `3296195e` (`#739` prior `master`); `Cand` = Handoff 1 `BzReader::parscan`.

### `wikidata-l9` (`209.70 MB` out; Serial-stage CPU share: Master `a3f26aa0` `14.16%` (`66.15 ms`) -> Cand `0.62%`; Modeled 10T crit vs `a3f26aa0` = `0.7480x`, vs `3296195e` = `0.669x`)

| Threads | MasterA (ms) | A/A (`MasterB/A`) | `Cand / MasterA` (`a3f26aa0`) | `Cand / M329` (`3296195e`) | CPU Ratio (`Cand/A`) |
|---:|---:|---:|---:|---:|---:|
| `1` | `748.93` | `0.9988x` | `1.0016x` | **`0.9836x`** | `1.0014x` |
| `2` | `444.82` | `1.0033x` | `1.0015x` | **`0.9837x`** | `1.0029x` |
| `4` | `449.57` | `1.0029x` | `1.0069x` | **`0.9910x`** | `1.0021x` |
| `6` | `467.65` | `1.0027x` | `1.0080x` | **`0.9989x`** | `1.0002x` |
| `8` | `461.74` | `0.9913x` | **`0.9978x`** | `1.0124x` | `1.0102x` |
| `10` | `464.55` | `1.0173x` | **`0.9871x`** | **`0.9838x`** | `1.0200x` |
| `12` | `470.13` | `0.9980x` | **`0.9906x`** | `1.0087x` | `1.0210x` |
| `16` | `476.65` | `0.9929x` | **`0.9751x`** | **`0.9782x`** | `1.0082x` |
| `20` | `470.30` | `0.9709x` | **`0.9797x`** | **`0.9947x`** | `1.0172x` |

### `wikidata-l1` (`36.00 MB` out; Serial-stage CPU share: Master `a3f26aa0` `18.37%` (`21.64 ms`) -> Cand `0.58%`; Modeled 10T crit vs `a3f26aa0` = `0.5454x`)

| Threads | MasterA (ms) | A/A | `Cand / MasterA` | `Cand / M329` | CPU Ratio |
|---:|---:|---:|---:|---:|---:|
| `1` | `154.49` | `0.9927x` | **`0.9959x`** | **`0.9450x`** | `0.9956x` |
| `2` | `94.03` | `1.0104x` | `1.0127x` | **`0.9748x`** | `0.9922x` |
| `4` | `90.72` | `1.0221x` | `1.0596x` | `1.0201x` | `1.0071x` |
| `6` | `91.14` | `0.9985x` | `1.0207x` | **`0.9746x`** | `1.0079x` |
| `8` | `93.39` | `1.0225x` | `1.1230x` | `1.0724x` | `1.0366x` |
| `10` | `96.00` | `1.0137x` | `1.0535x` | `1.0187x` | `1.0333x` |
| `12` | `95.75` | `0.9886x` | `1.1035x` | `1.0861x` | `1.0549x` |
| `16` | `95.88` | `1.0177x` | `1.0588x` | `1.0198x` | `1.0217x` |
| `20` | `98.27` | `1.0302x` | `1.0576x` | `1.0416x` | `1.0323x` |

### `records-l9` (`9.60 MB` out; Serial-stage CPU share: Master `a3f26aa0` `21.29%` (`32.38 ms`) -> Cand `1.51%`; Modeled 10T crit vs `a3f26aa0` = `0.4798x`)

| Threads | MasterA (ms) | A/A | `Cand / MasterA` | `Cand / M329` | CPU Ratio |
|---:|---:|---:|---:|---:|---:|
| `1` | `167.64` | `0.9924x` | **`0.9980x`** | **`0.9692x`** | `0.9979x` |
| `2` | `96.76` | `0.9630x` | `1.0148x` | `1.0280x` | `1.0264x` |
| `4` | `98.22` | `1.0076x` | `1.0287x` | `1.0162x` | `1.0312x` |
| `6` | `101.19` | `1.0288x` | `1.0453x` | `1.0200x` | `1.0110x` |
| `8` | `102.74` | `1.0415x` | `1.0459x` | `1.0276x` | `1.0385x` |
| `10` | `106.53` | `1.0101x` | `1.0353x` | `1.0051x` | `1.0372x` |
| `12` | `108.92` | `0.9894x` | `1.0539x` | `1.0402x` | `1.0433x` |
| `16` | `113.41` | `0.9761x` | `1.0251x` | `1.0204x` | `1.0373x` |
| `20` | `110.38` | `0.9761x` | `1.0389x` | `1.0080x` | `1.0445x` |

> [!NOTE]
> On this 2-core cloud VM, `8–20T` schedules `9–23` threads onto `2` physical cores (`+2–4%` context-switch floor on `95 ms` short runs). On the 10-physical-core i9, `mibz2-scan` (`0.21 s` on `a3f26aa0`) is split across `mibz2-scan` (`0.08 s`) + `2` `54 KiB` L1 helper scanners (`0.06 s` each) alongside `8` dedicated `OpenCells` workers (`0.748x` modeled 10T critical path vs `a3f26aa0` at `0.770x` RSS).
