# Carbon Cycle 46 (8/20): Safe Fused Huffman->`Runs` Kills & Stage-Balanced `2..20T` `parscan`

**Date:** 2026-11-22  
**Branch:** `research/carbon-word-transport`  
**Base `origin/master`:** `8c1a73ea67dba7f2b97528f9ae0ad954d30f60e6` (`#739` + `#745`)  
**Constraint:** `#![forbid(unsafe_code)]` on all new/modified candidate code.

---

## 1. Hypotheses Tested & One-Line Kills

First, we built a dedicated `#![forbid(unsafe_code)]` L1d-tiled (`3,200` symbols = `6.5 KiB`) fused Huffman -> `Runs` screen (`$SCRATCH/c46_fused_runs_screen.rs`) verifying `100%` record-exactness (`364/431` blocks fused on `wikidata-l9`, `361/361` on `wikidata-l1`, `0/11` on `records-l9`) against `read_item_with_scratch + runs::front` (`front_avx2`). Second, we unified `StreamDecoder::parscan`'s `S = 1` (`single_slab_scanner`) and `ParScanHub` paths to extend scanner-resident `runs::front` down into the `2..=3`-worker tier (`2T`, and `4T` on the 10-core i9 where `trimmed` sets `n = 3`) without adding `ParScanHub` overhead at `4..=6` workers.

| Candidate | Description | `wikidata-l9` `P1+P2` | `wikidata-l1` `P1+P2` | Verdict | One-Line Kill / Keep Reason |
|---|---|---:|---:|---|---|
| `Mtf16_Direct` (`#![forbid(unsafe_code)]`) | `3.2K`-sym L1d tile + `2x u64` (`16B`) shift MTF + `[u64; 256]` packed `cnt` | `2.7235x` | `2.6163x` | **KILL** | Inlined variable-length `tail.copy_within(0..t, 1)` (`27%` of literals) emits `call memmove@PLT`, spilling register MTF state across every literal, while `%cl` shifts serialize. |
| `Mtf32_Direct` (`#![forbid(unsafe_code)]`) | `3.2K`-sym L1d tile + `4x u64` (`32B`) shift MTF | `2.6497x` | `2.5409x` | **KILL** | 5-way MTF bucket branch (`<8/<16/<24/<32/else`) mispredicts on ~`40%` of literals alongside `memmove` caller-saved register spills. |
| `Mtf32_Bitmap` (`#![forbid(unsafe_code)]`) | `64`-sym `u64` `tzcnt` run-skipper + `4x u64` shift MTF | `2.4768x` | `2.3584x` | **KILL** | `u64` `tzcnt` run-skipping cuts `10%`, but scalar `u64::from(s > 1) << t` bitmap build + branched shift MTF remains `2.36x` slower than `front_avx2`. |
| `Mtf32Mask_Clean` (`#![forbid(unsafe_code)]`) | Branchless `MTF32_MASKS` (`1 KiB` `4x u64` constant-shift blend) + `#[cold]` tail | `2.0762x` | `2.0375x` | **KILL** | Eliminating `memmove` spills and MTF bucket branches saves `60 ms`, but per-symbol scalar loop + `4x u64` blends still execute `~4x` more instructions than `front_avx2` `pshufb`. |
| `Mtf32Mask_Swar` (`#![forbid(unsafe_code)]`) | `8x8` constant-shift SWAR `u64` bitmap + `tzcnt` run-skipper + `MTF32_MASKS` | `1.7404x` | `1.6665x` | **KILL** | Best pure-safe fused consumer (`-36%` vs `Mtf16_Direct`), but safe scalar `8x8` SWAR bitmap + `4x u64` mask blend (`118.5 ms` P2) cannot match `front_avx2`'s `_mm256_movemask_epi8` + `pshufb` (`40.9 ms` P2). |
| `c46_s23_h1` (`helpers = 1` at `n = 4..=6`) | `ParScanHub` with `1` helper + `prepare_runs` at `4..=6` workers | `4T: 1.036x`, `6T: 1.020x` | `4T: 1.035x`, `6T: 1.094x` | **KILL** | Enabling `ParScanHub` anchor scan + `HubState` mutex at `4..=6` workers adds `+1.5–2.8%` CPU and regresses `4–6T` wall vs zero-overhead `single_slab_scanner`. |
| **`c46b` (Stage-Balanced `2..20T` `parscan`)** | Unified `ParJobs` launcher: `single_slab_scanner` at `n <= 6` with `prepare_runs` at `n in 2..=3`; `ParScanHub(helpers = 2, prepare_runs = true)` at `n >= 7` | `4T: 0.984x`, `10T: 0.969x`, `12T: 0.943x`, `20T: 0.941x` | `2T: 0.963x`, `6T: 0.996x`, `20T: 0.996x` | **KEEP** | Matches exact stage inequality ($P_1 + P_2 = 105.4\text{ ms} < \frac{344.8\text{ ms}}{n} \iff n \in \{2, 3\}$ for $S=1$), cutting `2T` & `4T i9` (`n=3`) worker critical path `-10.7%` and RSS to `43.55 MiB` (`0.913x`) while restoring `parallel.rs` `Jobs`/`JobsTx` verbatim to `origin/master` (`-51` lines). |

---

## 2. Subprocess `VmHWM` Peak RSS (`wikidata_x3.bz2`, `629.10 MB` decoded)

| Threads | `native` Master | `native` `c46b` | `native` Ratio | `i9` Master | `i9` `c46b` | `i9` Ratio | Gate (`<= 1.10x` at `6–20T`) |
|---:|---:|---:|---:|---:|---:|---:|---|
| `1` | `28.48 MiB` | `28.31 MiB` | `0.9940x` | `28.52 MiB` | `28.45 MiB` | `0.9974x` | — |
| `2` | `63.71 MiB` | `59.79 MiB` | **`0.9384x`** | `64.15 MiB` | `58.83 MiB` | **`0.9170x`** | — |
| `4` | `94.43 MiB` | `88.36 MiB` | **`0.9356x`** | `47.69 MiB` | `43.55 MiB` | **`0.9133x`** | — |
| `6` | `132.22 MiB` | `133.54 MiB` | `1.0100x` | `55.77 MiB` | `51.52 MiB` | **`0.9239x`** | **PASS** |
| `8` | `62.18 MiB` | `48.44 MiB` | **`0.7791x`** | `61.52 MiB` | `49.96 MiB` | **`0.8122x`** | **PASS** |
| `10` | `67.53 MiB` | `54.21 MiB` | **`0.8027x`** | `59.90 MiB` | `49.50 MiB` | **`0.8264x`** | **PASS** |
| `12` | `71.88 MiB` | `56.71 MiB` | **`0.7891x`** | `59.96 MiB` | `48.75 MiB` | **`0.8130x`** | **PASS** |
| `16` | `76.34 MiB` | `62.54 MiB` | **`0.8193x`** | `59.34 MiB` | `49.07 MiB` | **`0.8269x`** | **PASS** |
| `20` | `83.15 MiB` | `72.93 MiB` | **`0.8770x`** | `59.34 MiB` | `48.80 MiB` | **`0.8225x`** | **PASS** |

---

## 3. Same-Binary Push-Streaming A/B (`7` Interleaved Rounds, `-C opt-level=3 -C codegen-units=1`)

- `cargo fmt --check` & `cargo clippy --all-targets -- -D warnings`: **PASS** (`0` warnings)
- `c41_bench_and_verify verify`: **PASS** (`PASS 1..5`, `1,500 / 1,500` streaming chunked mutation fuzz cases exact)

### `wikidata-l9` (`209.70 MB` out; Serial share: Master `14.20%` (`65.66 ms`) -> Cand `0.74%`; Modeled 10T crit = `0.7163x`)

| Threads | MasterA (ms) | A/A | `Cand / MasterA` | `Cand / M329` | CPU Ratio |
|---:|---:|---:|---:|---:|---:|
| `1` | `760.68` | `0.9923x` | **`0.9984x`** | `0.9819x` | `0.9986x` |
| `2` | `413.52` | `1.0017x` | `1.0260x` | `1.0053x` | `0.9955x` |
| `4` | `442.08` | `0.9926x` | **`0.9840x`** | `0.9763x` | `0.9947x` |
| `6` | `453.60` | `0.9940x` | `1.0153x` | `0.9946x` | `1.0099x` |
| `8` | `440.63` | `1.0239x` | `1.0058x` | `0.9975x` | `1.0064x` |
| `10` | `458.27` | `1.0002x` | **`0.9689x`** | `0.9952x` | `1.0067x` |
| `12` | `460.69` | `1.0048x` | **`0.9432x`** | `0.9402x` | `0.9920x` |
| `16` | `460.43` | `0.9888x` | **`0.9585x`** | `0.9623x` | `0.9860x` |
| `20` | `476.87` | `0.9800x` | **`0.9406x`** | `0.9605x` | `0.9896x` |

### `wikidata-l1` (`36.00 MB` out; Serial share: Master `18.40%` (`21.97 ms`) -> Cand `0.59%`; Modeled 10T crit = `0.6203x`)

| Threads | MasterA (ms) | A/A | `Cand / MasterA` | `Cand / M329` | CPU Ratio |
|---:|---:|---:|---:|---:|---:|
| `1` | `155.80` | `0.9993x` | **`0.9978x`** | `0.9422x` | `0.9978x` |
| `2` | `94.32` | `1.0037x` | **`0.9633x`** | `0.9308x` | `0.9914x` |
| `4` | `90.08` | `1.0073x` | `1.0135x` | `0.9823x` | `0.9938x` |
| `6` | `88.56` | `1.0022x` | **`0.9957x`** | `0.9577x` | `0.9984x` |
| `8` | `92.03` | `1.0023x` | `1.0911x` | `1.0505x` | `1.0255x` |
| `10` | `92.88` | `1.0031x` | `1.0699x` | `1.0409x` | `1.0163x` |
| `12` | `93.34` | `1.0191x` | `1.0831x` | `1.0428x` | `1.0089x` |
| `16` | `95.69` | `0.9802x` | `1.0165x` | `0.9985x` | `1.0090x` |
| `20` | `97.33` | `0.9976x` | **`0.9955x`** | `0.9737x` | `1.0009x` |

### `records-l9` (`9.60 MB` out; Serial share: Master `21.32%` (`33.69 ms`) -> Cand `1.36%`; Modeled 10T crit = `0.4675x`)

| Threads | MasterA (ms) | A/A | `Cand / MasterA` | `Cand / M329` | CPU Ratio |
|---:|---:|---:|---:|---:|---:|
| `1` | `168.67` | `1.0003x` | `1.0068x` | `0.9611x` | `1.0068x` |
| `2` | `91.37` | `0.9986x` | `1.0264x` | `1.0078x` | `1.0267x` |
| `4` | `94.79` | `0.9771x` | `1.0014x` | `0.9884x` | `1.0166x` |
| `6` | `97.81` | `1.0064x` | `1.0397x` | `1.0046x` | `1.0216x` |
| `8` | `102.66` | `1.0170x` | `1.0158x` | `0.9979x` | `1.0228x` |
| `10` | `105.25` | `1.0221x` | `1.0392x` | `1.0063x` | `1.0507x` |
| `12` | `105.31` | `0.9655x` | `1.0625x` | `1.0248x` | `1.0597x` |
| `16` | `107.60` | `0.9933x` | `1.0325x` | `1.0422x` | `1.0338x` |
| `20` | `108.61` | `0.9814x` | `1.0353x` | `1.0261x` | `1.0411x` |

---

## 4. Target for Cycle 47 (`9/20`)

Profile the internal phases of **`src/open.rs`** (`100%` safe Rust — `maximal` / `Pass` first pass, `endpoint` elimination, dictionary/band build, `emit` walk) alongside **`runs::takes_runs`** (`src/runs.rs`) on `wikidata-l9` and `wikidata-l1` (`320.6 ms` = `70.9%` of total decode time!), and attack the largest non-pointer-chasing safe construction overhead before publishing **Interim Handoff 2** in **Cycle 48 (`10/20`)**.
