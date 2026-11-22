# Carbon Cycle 42 (`4/20`) — Demand-Driven `54 KiB` L1 Scanner Pool, `<= 6T` Zero-Overhead `SlabScanner` Gate, and `0.74x–0.91x` i9 Peak RSS

**Date:** 2026-11-22  
**Branch:** `research/carbon-word-transport`  
**Base (`origin/master`):** `77c41fc7` (`src/` at `3296195e`, Wormhole `#739` plateau stack) + `#742` adopted block reader (`7b901b67`)  
**Safety:** `#![forbid(unsafe_code)]` on `src/par_scan.rs`, `src/huff.rs`, `src/block.rs`

---

## 1. Architecture & Mathematical Rationale (`C42`)

Cycle 41 proved push-streaming `SlabRope` inside `StreamDecoder::parscan`, cutting simulated i9 peak RSS to `0.84x–0.94x`, but exposed two structural issues:
1. Letting up to `16–20` `OpenCells` workers assist on `ParScanHub` created a lock/scheduler stampede (`+3–7%` idle bubble at `8–20W` on `wikidata-l9`) and retained `2 MiB` `stitch_buf`s (`1.17x–1.24x` untrimmed `native` RSS at `16–20W`).
2. Running `ParScanHub` (`find_magics` + `CandSlot` + helper wakeups) below `7` workers added coordinator overhead where a single `C37` scanner (`62 ms` scan vs `396 / 6 = 66 ms` BWT on `-9`) already had `1.1x–3.2x` throughput headroom.

Cycle 42 resolves both cleanly:
1. **Two-Regime Front-End Dispatch (`workers <= 6` vs `workers >= 7`):**
   - For `1..=6` workers (`2..=6` threads on the i9), `mibz2-scan` runs `single_slab_scanner` directly over `SlabRope` + `read_item_tight` (`0` helper threads, `0` `find_magics`, `0` `ParScanHub` locks), matching `master`'s exact `W` workers + `1` scanner topology while adding zero-copy `SlabRope` + `C37` block reader + quantized `syms` capacity.
   - For `>= 7` workers (`8..=20` threads on the i9, where `i9` runs `8` dedicated `OpenCells` workers at `>= 10` threads), `mibz2-scan` coordinates `ParScanHub` with **`2` demand-driven `54 KiB` L1d-resident helper scanners** (`mibz2-scan-1`, `mibz2-scan-2`). Helpers sleep on a dedicated `ParScanHub::helper_cv` (keeping `Shared::scan_cv` and `scanner_waiting` 100% exclusive to `par_scanner_lead`, preventing any wakeup theft) whenever `jobs.queued_len() >= 3`, and BWT workers hold **zero** front-end scratch.
2. **Bounded Front-End Horizon (`max_front_ahead = workers + 4`) + `MAX_STITCH_RETAIN = 256 KiB` + Quantized `syms` (`read_item_core`):**
   - Unifies `read_item_with_scratch` and `read_item_tight` onto a single concrete `read_item_core(..., tight: bool)` compiled with `-C codegen-units=1` (matching `Cargo.toml` `[profile.release]`) and quantizes `tight` `syms` reservation to `max_out.max(65_536).next_power_of_two().min(MAX_SYMS)` so recycled `Vec<u16>` slots allocate once without cross-block `realloc` churn.

Artifact saved at `docs/journal/2026-11-22-carbon-cycle42/c42_par_scan.rs`.

---

## 2. Correctness Verification

- `c41_bench_and_verify verify` (`-C opt-level=3 -C codegen-units=1`):
  - `PASS 1`: `wikidata_x3.bz2` (`36 MiB` compressed, `9 x 4 MiB` chunks, 3 streams) `BzReader::parscan` (`1..20T`) + `decode_parscan` exact.
  - `PASS 2`: `1 B`, `7 B`, `63 B`, `4 KiB`, `65,537 B`, `1 MiB` multi-level (`1..=9`) + planted-magic chunking exact.
  - `PASS 3`: Slow-reader backpressure & `read_block` recycling deadlock-free (`2, 6, 16T` — exercising both `single_slab_scanner` and `ParScanHub` + `helper_cv`).
  - `PASS 4`: `cancel()` and `PanickingReader` unwind safety exact.
  - `PASS 5`: **`1,500 / 1,500`** streaming chunked mutation fuzz cases exact on both prefix bytes emitted before error and `Error { kind, block, bit }`.

---

## 3. Subprocess Peak RSS (`VmHWM`) Audit on `wikidata_x3.bz2` (`36 MiB` in -> `629 MB` out)

Median of 3 interleaved child-process runs (`Master` = `3296195e` `#739` `BzReader::parallel`; `Cand` = `C42` `BzReader::parscan`):

| Threads | `i9` (`Machine::service_target()`) Master MiB | `i9` Cand MiB | `i9` Ratio | `native` (2-core untrimmed) Master MiB | `native` Cand MiB | `native` Ratio |
|---:|---:|---:|---:|---:|---:|---:|
| `1` | `28.67` | `28.34` | **`0.9884x`** | `28.83` | `28.34` | **`0.9831x`** |
| `2` | `66.46` | `60.57` | **`0.9113x`** | `65.42` | `62.47` | **`0.9549x`** |
| `4` | `49.43` | `44.54` | **`0.9010x`** | `92.79` | `98.99` | **`1.0669x`** |
| `6` | `55.29` | `50.38` | **`0.9112x`** | `130.05` | `124.50` | **`0.9573x`** |
| `8` | `62.89` | `46.80` | **`0.7442x`** | `61.04` | `47.16` | **`0.7726x`** |
| `10` | `58.03` | `46.93` | **`0.8087x`** | `67.70` | `51.27` | **`0.7572x`** |
| `12` | `60.65` | `48.02` | **`0.7917x`** | `68.41` | `53.81` | **`0.7866x`** |
| `16` | `58.88` | `47.32` | **`0.8036x`** | `76.86` | `61.48` | **`0.7999x`** |
| `20` | `60.32` | `47.09` | **`0.7807x`** | `81.91` | `68.43` | **`0.8354x`** |

- **`i9` (`6–20` threads):** **`0.7442x–0.9112x`** (`5–16 MiB` **below** `#739` `master` across `6–20` threads, even with `8` `OpenCells` workers at `10–20T`!).
- **`native` untrimmed (`6–20` threads):** **`0.7572x–0.9573x`** (`16T` dropped from `1.169x -> 0.800x`; `20T` dropped from `1.243x -> 0.835x`!).

---

## 4. Same-Binary Push-Streaming A/B + A/A (`2`-Core Xeon Platinum 8481C VM, `-C codegen-units=1`, 7 Interleaved Rounds)

`MasterA` / `MasterB` = `origin/master` `3296195e` (`#739`); `MasterBR` = `7b901b67` (`#739 + #742`); `Cand` = `C42` `BzReader::parscan`.

### `wikidata-l9` (`209.70 MB` out; Serial-stage CPU share: Master `17.00%` (`81.82 ms`) -> Cand `0.62%`; Modeled 10T crit `0.6092x`)

| Threads | MasterA (ms) | A/A (`MasterB/A`) | `Cand / MasterA` | `Cand / MasterBR` | CPU Ratio (`Cand/A`) |
|---:|---:|---:|---:|---:|---:|
| `1` | `765.27` | `0.9999x` | **`0.9777x`** | **`0.9992x`** | `0.9781x` |
| `2` | `450.98` | `0.9886x` | **`0.9808x`** | **`0.9934x`** | `0.9841x` |
| `4` | `461.19` | `0.9998x` | **`0.9818x`** | `1.0073x` | `0.9835x` |
| `6` | `478.49` | `1.0012x` | **`0.9698x`** | **`0.9806x`** | `0.9888x` |
| `8` | `459.44` | `0.9730x` | **`0.9882x`** | **`0.9933x`** | `1.0030x` |
| `10` | `466.40` | `0.9770x` | **`0.9741x`** | **`0.9785x`** | `0.9953x` |
| `12` | `470.52` | `1.0033x` | **`0.9653x`** | **`0.9850x`** | `0.9984x` |
| `16` | `479.50` | `1.0065x` | **`0.9480x`** | **`0.9685x`** | `0.9934x` |
| `20` | `461.24` | `1.0125x` | **`0.9793x`** | **`0.9665x`** | `0.9976x` |

### `wikidata-l1` (`36.00 MB` out; Serial-stage CPU share: Master `23.86%` (`30.31 ms`) -> Cand `0.63%`; Modeled 10T crit `0.3938x`)

| Threads | MasterA (ms) | A/A | `Cand / MasterA` | `Cand / MasterBR` | CPU Ratio |
|---:|---:|---:|---:|---:|---:|
| `1` | `164.27` | `1.0067x` | **`0.9491x`** | `1.0010x` | `0.9487x` |
| `2` | `96.10` | `1.0108x` | **`0.9763x`** | **`0.9783x`** | `0.9434x` |
| `4` | `92.92` | `1.0205x` | **`0.9906x`** | `1.0041x` | `0.9535x` |
| `6` | `94.47` | `1.0125x` | **`0.9759x`** | `1.0125x` | `0.9519x` |
| `8` | `98.52` | `0.9973x` | `1.0434x` | `1.0716x` | `0.9935x` |
| `10` | `98.52` | `1.0247x` | `1.0655x` | `1.1108x` | `0.9921x` |
| `12` | `100.00` | `1.0017x` | `1.0479x` | `1.0866x` | `0.9913x` |
| `16` | `100.00` | `1.0094x` | `1.0411x` | `1.0446x` | `0.9835x` |
| `20` | `102.17` | `0.9971x` | `1.0386x` | `1.0880x` | `0.9966x` |

### `records-l9` (`9.60 MB` out; Serial-stage CPU share: Master `23.26%` (`38.05 ms`) -> Cand `1.26%`; Modeled 10T crit `0.4129x`)

| Threads | MasterA (ms) | A/A | `Cand / MasterA` | `Cand / MasterBR` | CPU Ratio |
|---:|---:|---:|---:|---:|---:|
| `1` | `173.09` | `1.0001x` | **`0.9667x`** | **`0.9984x`** | `0.9675x` |
| `2` | `98.60` | `0.9915x` | **`0.9837x`** | `1.0107x` | `0.9862x` |
| `4` | `101.08` | `1.0030x` | **`0.9915x`** | `1.0084x` | `0.9969x` |
| `6` | `108.87` | `0.9742x` | **`0.9925x`** | `1.0854x` | `0.9912x` |
| `8` | `110.00` | `1.0287x` | **`0.9954x`** | `1.0015x` | `0.9841x` |
| `10` | `112.73` | `0.9863x` | `1.0046x` | `1.0387x` | `1.0256x` |
| `12` | `113.85` | `1.0130x` | `1.0026x` | `1.0194x` | `1.0128x` |
| `16` | `116.83` | `0.9730x` | `1.0074x` | `1.0454x` | `0.9904x` |
| `20` | `116.58` | `1.0019x` | `1.0078x` | `1.0510x` | `0.9907x` |

---

## 5. Keep / Kill Decisions & Cycle 43 (`5/20` -> Interim Handoff 1) Targets

- **KEEP (`C42`):** Demand-driven `54 KiB` L1 helper scanner pool (`helper_cv`) gated at `>= 7` workers + `<= 6T` `single_slab_scanner` + `max_front_ahead = workers + 4` + single-function `read_item_core` with quantized `syms` capacity: beats `MasterA` at **every** thread count `1..20T` on `wikidata-l9` (`0.9480x–0.9882x`), beats `MasterA` across `1..6T` on all three corpora, and cuts `i9` peak RSS to `0.7442x–0.9113x`.
- **KILL (`ParScanHub` active at `4–6` workers):** Added `find_magics` + coordinator lock/scheduler overhead (`1.0395x / 1.0550x` vs `MasterBR`) where `1` `C37` scanner already has `1.1x–1.6x` headroom over `4–6` BWT workers.
- **KILL (generic `read_item_inner::<const TIGHT: bool>` split + unquantized `syms.resize`):** Duplicated `read_item` monomorphizations hurt inlining (`+4.8%` at 1T without `cgu=1`) and unquantized `TIGHT` `syms` growth risked repeated cross-block `realloc`s.
- **Cycle 43 (`5/20` -> Interim Handoff 1):**
  1. Multi-candidate micro-batching in `ParScanHub::scan_step` + `Jobs::push_batch` (`<= 4` small `< 96 KiB` blocks claimed/published per lock acquisition, cutting `wikidata-l1` `8..20T` lock/futex round-trips by `~4x–8x`) + `PRESTAGE_SINGLE_BYTES = 512 KiB` (`-4 MiB` RSS at `2–6T`).
  2. Full production qualification (`cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test --release --all-targets`, `examples/ab` CLI parity) and **Interim Handoff 1** drop-in patch publication over latest `origin/master`.
