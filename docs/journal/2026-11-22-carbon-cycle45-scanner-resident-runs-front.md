# Carbon Cycle 45 (7/20): Scanner-Resident `P1 + P2` (`runs::front`) Fusion

**Date:** 2026-11-22  
**Branch:** `research/carbon-word-transport`  
**Base `origin/master`:** `8c1a73ea67dba7f2b97528f9ae0ad954d30f60e6` (`src/` identical to `47b37019` / `a3f26aa0`, `#739` + `#745`)  
**Constraint:** `#![forbid(unsafe_code)]` on all new/modified candidate modules (`src/par_scan.rs`, `src/lib.rs`, `src/parallel.rs`).

---

## 1. Hypothesis & Architecture

In Cycle 44's post-`#745` steady-state stage profile on `wikidata-l9`, `P1 (block+huff)` accounted for `14.70%` (`66.53 ms`) and `P2 (takes_runs + runs::front)` accounted for `9.13%` (`41.30 ms`), leaving `P3 + P4 (OpenCells + unrle_in_place)` at `76.16%` (`344.61 ms`). In Handoff 1 (`Cycle 43`), `ParScanHub` parallelized `P1` across `mibz2-scan` + `2` helpers (`0.06–0.08 s` each on `wikidata_x3.bz2`), leaving `8` `OpenCells` workers bottlenecked at `(P2 + P3 + P4) / 8 = 385.9 / 8 = 48.2 ms` while shipping `1 MiB` `Vec<u16>` `syms` buffers cross-core into `Jobs`.

**Cycle 45 Mechanism (`PreparedRuns` + `decode_prepared_runs`):**
1. When `ParScanHub` runs (`workers > 6` and `I::default().reads_runs()`), each scanner (`mibz2-scan` + `mibz2-scan-1..2`) checks `!work.level_uncertain && runs::takes_runs(&b)` immediately after `read_item_tight` succeeds while `b.syms` is L2-hot on that scanner's core.
2. When admitted (`383 / 431` blocks = `99.999%` of rows on `wikidata-l9`; `361 / 361` on `wikidata-l1`), the scanner runs `runs::front(&b, scanner_runs, kernels)` on a lazy scanner-private `Option<Box<Runs>>` (allocated only if `takes_runs` ever fires, keeping `lit`/`one` `u64` bitmaps off BWT workers and zero-overhead on column-route streams like `records-l9`).
3. `b.syms` (`~1 MiB` `Vec<u16>`) is retained **permanently local** in `ScannerLocal::local_syms` (`3` scanners $\times 1\text{ MiB} = 3\text{ MiB}$ total instead of `slots` $\times 1\text{ MiB}$), never entering `ParJobs` or `Shared::free_syms`. Only the compact `records: Vec<u16>` (`~160–256 KiB`, $4\times$ smaller than `syms`) and boxed `([u32; 256], [u32; 256])` (`CountsPer`, recycled via `ParScanHub::prep_pool`) cross to `parscan_hub_worker`.
4. `parscan_hub_worker` invokes `decode_prepared_runs`, installing `records` + `counts_per` into `s.runs` in $O(1)$ and bypassing `Shared::lock()` entirely inside `take(...)` for blocks with `n < POOLED_FROM` (`64 KiB`).

---

## 2. Ablations & One-Line Kills

| Variant | Description | Verdict | One-Line Kill / Keep Reason |
|---|---|---|---|
| `c45a` (`helpers = 3`, eager stack `Runs`) | `1` lead + `3` helper scanners (`4` scanners total), eager `Runs` in `ScannerLocal` | **KILL** | Spawning a 3rd helper scanner adds `notify_all()`/lock contention on short non-runs streams (`records-l9` 10T CPU `1.0677x` vs `1.0119x` with `helpers = 2`) while `helpers = 2` already balances scanner and worker thread CPU (`0.10–0.12 s` vs `0.09–0.12 s`). |
| **`c45b` (`helpers = 2`, lazy `Option<Box<Runs>>`, `prep_pool`)** | `1` lead + `2` helper scanners at `workers > 6`, lazy `scanner_runs` + dedicated `prep_pool` | **KEEP** | Cuts `i9` peak RSS to `48.25–49.37 MiB` (`0.7885x–0.8548x` across `8–20T`, `-5.8 MiB` below Handoff 1 at `20T`), balances 10T per-thread CPU (`0.09–0.12 s` across all scanners and workers), and lowers modeled 10-core critical path from `0.748x -> 0.714x` vs `#745` `master` (`0.648x` vs `3296195e`). |

---

## 3. Verification & Per-Thread `examples/ab` Audit (`10T`, `/tmp/wikidata_x3.bz2`)

- `cargo fmt --check` & `cargo clippy --all-targets -- -D warnings`: **PASS** (`0` warnings)
- `c41_bench_and_verify verify`: **PASS** (`PASS 1..5`, including `1,500 / 1,500` streaming chunked mutation fuzz cases exact on prefix bytes + `Error`)

### `examples/ab` Child Runs (`10` threads, `629,100,588` B out, exact hash `bbe9dae7753f6c67`, 3 alternating pairs)

| Pair | Arm | Wall (s) | CPU (s) | `VmHWM` (`rss_kb`) | Per-Thread CPU (`thread_cpu`) |
|---:|---|---:|---:|---:|---|
| 3 (med) | `mibz2` (`master` `#739+#745`) | `0.9261` | `1.6439` | `69,760` | `mibz2-scan=0.20`, `mibz2-0..9 = 0.10..0.16` |
| 3 (med) | `mibz2-parscan` (`c45b`) | **`0.9160`** | **`1.6273`** | **`59,420`** (`55,048` min) | `mibz2-scan=0.12`, `mibz2-scan-1=0.10`, `mibz2-scan-2=0.09`, `mibz2-0..9 = 0.09..0.12` |

---

## 4. Subprocess `VmHWM` Peak RSS (`wikidata_x3.bz2`, `629.10 MB` decoded)

| Threads | `native` Master | `native` `c45b` | `native` Ratio | `i9` Master | `i9` `c45b` | `i9` Ratio | Gate (`<= 1.10x` at `6–20T`) |
|---:|---:|---:|---:|---:|---:|---:|---|
| `1` | `28.55 MiB` | `28.32 MiB` | `0.9919x` | `28.43 MiB` | `28.40 MiB` | `0.9989x` | — |
| `2` | `62.33 MiB` | `60.16 MiB` | `0.9652x` | `65.57 MiB` | `61.48 MiB` | `0.9377x` | — |
| `4` | `103.12 MiB` | `94.05 MiB` | `0.9120x` | `49.95 MiB` | `44.43 MiB` | `0.8895x` | — |
| `6` | `125.90 MiB` | `129.45 MiB` | `1.0282x` | `56.70 MiB` | `50.53 MiB` | **`0.8911x`** | **PASS** |
| `8` | `63.91 MiB` | `49.97 MiB` | **`0.7819x`** | `62.07 MiB` | `48.95 MiB` | **`0.7885x`** | **PASS** |
| `10` | `65.83 MiB` | `52.90 MiB` | **`0.8035x`** | `57.75 MiB` | `49.37 MiB` | **`0.8548x`** | **PASS** |
| `12` | `71.87 MiB` | `55.26 MiB` | **`0.7688x`** | `58.60 MiB` | `48.81 MiB` | **`0.8330x`** | **PASS** |
| `16` | `77.88 MiB` | `63.96 MiB` | **`0.8214x`** | `59.36 MiB` | `48.25 MiB` | **`0.8128x`** | **PASS** |
| `20` | `85.75 MiB` | `70.76 MiB` | **`0.8252x`** | `59.14 MiB` | `48.74 MiB` | **`0.8241x`** | **PASS** |

---

## 5. Same-Binary Push-Streaming A/B (`7` Interleaved Rounds, `-C opt-level=3 -C codegen-units=1`)

### `wikidata-l9` (`209.70 MB` out; Serial share: Master `14.10%` (`64.35 ms`) -> Cand `0.55%`; Modeled 10T crit vs `a3f26aa0` = `0.7144x`)

| Threads | MasterA (ms) | A/A | `Cand / MasterA` | `Cand / M329` | CPU Ratio |
|---:|---:|---:|---:|---:|---:|
| `1` | `736.85` | `0.9986x` | `1.0119x` | `0.9916x` | `1.0116x` |
| `2` | `433.68` | `1.0173x` | `1.0121x` | `0.9850x` | `0.9909x` |
| `4` | `453.53` | `0.9916x` | **`0.9774x`** | `0.9916x` | `0.9932x` |
| `6` | `461.04` | `1.0103x` | `1.0049x` | `0.9985x` | `0.9917x` |
| `8` | `444.17` | `0.9778x` | **`0.9800x`** | `0.9812x` | `1.0076x` |
| `10` | `444.49` | `0.9980x` | **`0.9874x`** | `0.9785x` | `0.9968x` |
| `12` | `466.32` | `0.9462x` | **`0.9341x`** | `0.9745x` | `0.9940x` |
| `16` | `468.07` | `0.9851x` | **`0.9463x`** | `0.9251x` | `0.9989x` |
| `20` | `469.11` | `0.9760x` | **`0.9584x`** | `0.9533x` | `0.9982x` |

### `wikidata-l1` (`36.00 MB` out; Serial share: Master `18.35%` (`21.34 ms`) -> Cand `0.59%`; Modeled 10T crit vs `a3f26aa0` = `0.6310x`)

| Threads | MasterA (ms) | A/A | `Cand / MasterA` | `Cand / M329` | CPU Ratio |
|---:|---:|---:|---:|---:|---:|
| `1` | `153.31` | `0.9980x` | `1.0213x` | `0.9672x` | `1.0246x` |
| `2` | `94.16` | `0.9896x` | **`0.9847x`** | `0.9795x` | `1.0050x` |
| `4` | `89.82` | `1.0121x` | `1.0319x` | `0.9884x` | `1.0139x` |
| `6` | `91.63` | `0.9923x` | **`0.9942x`** | `0.9831x` | `0.9830x` |
| `8` | `94.75` | `1.0102x` | `1.0512x` | `1.0240x` | `1.0316x` |
| `10` | `95.78` | `0.9764x` | `1.0377x` | `1.0383x` | `1.0222x` |
| `12` | `94.23` | `1.0025x` | `1.0656x` | `1.0350x` | `1.0226x` |
| `16` | `97.55` | `0.9990x` | `1.0334x` | `1.0066x` | `1.0116x` |
| `20` | `98.53` | `1.0045x` | `1.0072x` | `0.9719x` | `1.0106x` |

### `records-l9` (`9.60 MB` out; Serial share: Master `20.85%` (`32.83 ms`) -> Cand `1.22%`; Modeled 10T crit vs `a3f26aa0` = `0.4842x`)

| Threads | MasterA (ms) | A/A | `Cand / MasterA` | `Cand / M329` | CPU Ratio |
|---:|---:|---:|---:|---:|---:|
| `1` | `165.21` | `1.0098x` | **`0.9978x`** | `0.9615x` | `0.9973x` |
| `2` | `99.53` | `0.9889x` | **`0.9636x`** | `0.9438x` | `0.9681x` |
| `4` | `100.86` | `0.9811x` | **`0.9867x`** | `0.9572x` | `0.9775x` |
| `6` | `103.88` | `0.9976x` | `1.0336x` | `1.0138x` | `1.0347x` |
| `8` | `101.94` | `1.0447x` | `1.0406x` | `0.9763x` | `1.0337x` |
| `10` | `109.31` | `1.0046x` | `1.0229x` | `1.0382x` | `1.0119x` |
| `12` | `110.51` | `1.0132x` | `1.0352x` | `1.0315x` | `1.0223x` |
| `16` | `111.34` | `0.9975x` | `1.0348x` | `1.0265x` | `1.0298x` |
| `20` | `113.88` | `1.0038x` | `1.0835x` | `1.0855x` | `1.0604x` |

---

## 6. Target for Cycle 46 (`8/20`)

On runs-route blocks (`wikidata-l9` and `wikidata-l1`), `P1 + P2` (`read_item_tight + runs::front`) currently materializes `25,000,000` `u16` symbols (`50 MB`) into `syms`, scans `syms` twice (`takes_runs` + `front_avx2` `lit`/`one` bitmaps), and then walks `lit`/`one` to emit `Runs`. Cycle 46 tests **Direct Fused Huffman -> `Runs` (`read_item_runs_fused`, `#![forbid(unsafe_code)]`)**: decoding `50`-symbol Huffman groups directly through a register/L1-resident MTF + run-length accumulator into `Runs` (`byte << 8 | len`, `counts`, `per`) **without ever materializing `syms: Vec<u16>` or `lit`/`one` bitmaps** when run-share is dense, benefiting **both** `1T` (`decode` / `BzReader::new`), `2–6T` (`single_slab_scanner`), and `7–20T` (`ParScanHub`)!
