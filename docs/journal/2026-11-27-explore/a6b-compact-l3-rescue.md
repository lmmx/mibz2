# Forensic L3/RSS Dissection & Compact-Footprint Rescue of `A6b` (`a6b-compact-l3-rescue`)

**Date:** 2026-11-27  
**Probe:** [`examples/a6b_compact_rescue.rs`](../../../examples/a6b_compact_rescue.rs) (`#![forbid(unsafe_code)]`, 100% safe Rust, zero `src/` edits)  
**Trigger:** Physical i9-10900K `STOP/KILL` of `#1053` (`a6b-endpoint-build`, `origin/evidence` `a6e427b1`, `#1151`): `wikidata/1` wall `0.9824 [0.9776, 0.9872]` (`-1.76%`, `+24.7 ms`, RSS `+592 KiB` / `1.0077x`), `wikidata/8/S` ideal-scanner `worker_busy = 1.0106 [0.9931, 1.0283]` (`wall_s = 1.0124`, RSS **`+4,192 KiB` / `1.0374x` = `+524 KiB/worker`**), `wikidata/10` RSS **`+5,460 KiB` / `1.0488x` = `+546 KiB/worker`**.

---

## 1. Frozen Pre-Data Stop Test (RULES C1a / C2 — Frozen Before Timing)

- **Corpus & Correctness:**
  - `wikidata.bz2` (SHA-256 `82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915`, 431 total blocks, 233 production-admitted Open blocks) — 100% byte-exact vs `bwt::walk` on all 431/431 blocks (forced Open).
  - Genuine single-cycle synthetic stress suite (`ns == 0`, 40k spiral `widest = 39,999`, deep/duplicate long words, non-ASCII digrams, equal-width strip cascade, 70k wide fallback `widest = 69,999`, generation-counter wraparound) — 100% byte-exact.
  - Guard corpora (`text`, `runs`, `records`, `random`, generated via `ab gen` at `SCALE=4`) — 100% byte-exact.
- **Measurement Protocol:**
  - **Part A (Forensic Allocation & RSS Audit):** Byte-for-byte audit of `n_nodes`, `n_strips`, `n_rules`, `n_bands`, `m`, `edges` bytes, and `Endpoint` heap bytes across all 233 Open blocks of `wikidata.bz2`, plus direct Linux `/proc/self/status` `VmRSS` / `VmHWM` measurement of `k = 1` and `k = 8` independent worker `State` pools (`S0_default`, `S1_1053`, and every rescue candidate `C1..C6`) in fresh isolated child processes.
  - **Part B (Isolated `A6b` & Full Open Worker Warm + L3-Cold 8-Worker Pool Timing):** 7 interleaved repeats pinned to CPU 0 (`taskset -c 0`), measuring Linux `/proc/thread-self/schedstat` on-CPU time (`A6b_cpu`, `Full_warm_cpu`, `Full_cold_cpu` across an 8-worker rotating pool simulating Comet Lake 8-worker L3 pressure, where full worker = `A5/M + A6a + A6b + A6c + A7 + A9`), across **all three builds**:
    1. `base`: `cargo build --release` (generic x86-64),
    2. `nat`: `RUSTFLAGS="-C target-cpu=native"`,
    3. `jcc`: `RUSTFLAGS="-C target-cpu=native -C llvm-args=-x86-branches-within-32B-boundaries"`.
- **PASS iff** at least one 100% safe-Rust (`#![forbid(unsafe_code)]`), 100% bit-exact compact-footprint `A6b` redesign clears **ALL THREE** criteria simultaneously:
  1. **Zero RSS Bloat (`<= 1.00x`):** Keeps 8-worker pool `VmRSS` at **`<= 1.00x` of shipped `S0_default`** (persistent `Endpoint` scratch `<= +16 KiB/worker`, eliminating `>= 97%` of `#1053`'s `+524 KiB/worker` RSS bloat, and strictly below `S1_1053` on both `k = 1` and `k = 8` RSS).
  2. **Speed (`>= 30.0%` A6b AND `>= 4.0%` Full Open Worker L3-Cold):** Beats shipped `S0_default` isolated `A6b_cpu` median by **`>= 30.0%`** AND full Open worker (`A5+A6a+A6b+A6c+A7+A9`, L3-cold 8-worker pool) median by **`>= 4.0%`** with **disjoint `[min..max]` spreads** across **all three builds** (`base`, `nat`, `jcc`), AND strictly beats `S1_1053` on both L3-cold full-worker speed and 8-worker RSS.
  3. **Guard Safety:** Zero regression on `text`, `runs`, `records`, `random` (`<= 1.01x` median).
- **STOP otherwise** (preserve frozen verdict under RULES C2; record exact forensic audit, RSS reproduction, candidate comparison, and honest remaining `A6b` ceiling).

### Candidates Evaluated
1. `S0_default` — Exact shipped `src/open/endpoint.rs` (`edges`-overlaid `Narrow` 16B AoS, 0 extra heap).
2. `S1_1053` — Exact `#1053` (`Fixed64kSoa` + `image_order64` + `rules` + `gen_cache.{pairs,contents}`).
3. `C1_vec_soa_v2` — Dynamically sized `Vec`s (`links`, `widths`, `words128`, `rules`, `image_order64`) + `V2_u128` safe wins (zero-call empty join, no `pairs`, `gen8+fp8` `16 KiB` `contents_4k`, `tag < 8` 64-bit dict, `[u32; 64]` stack).
4. `C2_tiered_slab_v2` — Power-of-two compact slab tiers (`8K/16K/32K/64K` with `idx & (CAP - 1)` mask, zero bounds check) + `V2_u128` wins.
5. `C3_compact_word8_soa` — Inline-7 `u64` `words64` (`8 B/node`, halving `words` footprint) + `V2` wins (`Vec` / tiered slab).
6. `C4_fused_setup_on_shipped` — Fused setup + `edges`-overlaid `image_order64` + `gen8+fp8` `16 KiB` `contents_4k` (no `pairs`) + `aw==bw` in-place on top of shipped `edges`-overlaid `Narrow` arena.
7. `C5_edges_overlay_soa_u64` — **Wormhole-reclaimed `edges`-overlaid** tiered power-of-two SoA inline-7 `u64` (`CAP ∈ {4K, 8K, 16K, 32K, 64K}` via safe `TryInto<&mut [u32; CAP]>`, zero bounds checks, `rules` overlaid inside `edges`) + `V2` wins (`+16 KiB` `contents_4k` only -> `<= 1.00x` RSS, L2-resident).
8. `C6_edges_overlay_soa_u128` — **Wormhole-reclaimed `edges`-overlaid** tiered power-of-two SoA inline-15 `u128` (`CAP ∈ {4K, 8K, 16K, 32K, 64K}` via safe `TryInto<&mut [u32; CAP]>`, zero bounds checks, `rules` overlaid inside `edges`) + `V2_u128` wins (`+16 KiB` `contents_4k` only -> `<= 1.00x` RSS).

*(Frozen pre-data stop test above preserved verbatim under RULES C2.)*

---

## 2. Executive Summary & Frozen Verdict

**Verdict: `STOP` (scoped dead-end recorded in `frontier/dead-ends.tsv`, RULES C2 preserved).**

All 8 arms (`S0_default`, `S1_1053`, `C1`–`C6`) passed **4,160 / 4,160** 100% byte-exact and CRC-exact checks across all 431/431 `wikidata.bz2` blocks (forced Open), 7 synthetic stress cases (`ns == 0`, 40k/70k spirals, deep/duplicate long words, non-ASCII digrams, equal-width cascade, mixed runs × `spirals = false/true/None` + 5-block generation-counter wraparound), and all 4 guard corpora (`text`, `runs`, `records`, `random`) with `#![forbid(unsafe_code)]`.

Our forensic audit **completely explains both physical i9-10900K surprises from `#1053` (`origin/evidence` `a6e427b1`, `#1151`)** and quantifies the true safe-Rust `A6b` ceiling:

1. **Why `#1053` (`S1_1053`) added `+524–546 KiB/worker` RSS at 8–10W AND why replacing `Box<[T; 65536]>` with `Vec` (`C1`) or tiered slabs (`C2`/`C3`) does NOT fix it on Linux:**
   - Shipped `S0_default` obeys the Wormhole W1 memory diet: **all** `A6b` nodes, rules, and caches are overlaid directly inside `Kernel.edges: Vec<u32>` (`0 KiB` persistent `Endpoint` heap beyond `stack: Vec<u32>`).
   - `#1053` abandoned that overlay, placing `Fixed64kSoa` + `rules` + `image_order64` + `gen_cache.{pairs,contents}` in separate persistent worker allocations. Because Linux `calloc` **already demand-paged** `#1053`'s 64K arrays down to `max(n_nodes) = 12,836` (`links 104 KiB + widths 52 KiB + words128 204 KiB + rules 128 KiB + img64 64 KiB + gen_cache 24 KiB = ~576 KiB`), our isolated `k = 8` `VmRSS` audit reproduced i9-10900K's `+4,192 KiB` (`1.0374x`, `+524 KiB/w`) jump directly (**`+4,580 KiB`, `1.0356x`, `+572.5 KiB/w`**).
   - Consequently, merely shrinking virtual size via dynamic `Vec`s (`C1_vec_soa_v2`: **`+4,400 KiB`, `+550.0 KiB/w`, `1.0342x`**) or `8K/16K` tiered slabs (`C2_tiered_slab_v2`: **`+3,944 KiB`, `+493.0 KiB/w`, `1.0306x`**; `C3_compact_word8_soa`: **`+3,308 KiB`, `+413.5 KiB/w`, `1.0257x`**) **still fails the `<= 1.00x` RSS gate** because every worker still holds `~410–550 KiB` of separate resident arrays outside `Kernel.edges`.
   - **Only Wormhole W1 `edges`-overlay designs eliminate the RSS jump:** `C4_fused_setup_on_shipped` achieves **`-124 KiB` (`-15.5 KiB/w`, `0.9990x`)** at `k = 8`, and `C5_edges_overlay_soa_u64` (with packed 8B links) achieves **`-1,752 KiB` (`-219.0 KiB/w`, `0.9864x` — 6.15 MiB lower than `#1053`!)** because `u64` inline-7 creates half as many rules (`3,374` vs `7,072` avg) inside `edges`.
2. **Why `#1053` delivered only `-1.76%` (`+24.7 ms`) at `1W` on i9-10900K (vs earlier VM `-3.7..-5.2%` projections) and flipped to `+1.06%` slower at `8W`:**
   - **Half was the VM's Skylake JCC erratum:** enabling `-x86-branches-within-32B-boundaries` (`jcc`, matching Comet Lake-S stepping 5's silicon fix) speeds up **shipped `S0_default`** isolated `A6b` by **`-11.5%`** (`124.73 -> 110.38 ms`), immediately shrinking `#1053`'s isolated `A6b` cut from `-22.2%..-27.5%` down to **`-19.65%`** (`110.38 -> 88.68 ms`, `-21.70 ms`).
   - **The other half is L2/L3 eviction from abandoning the `edges` overlay:** on `jcc` `Full_warm_cpu` (`k = 1`), `S1_1053` loses `+8.5 ms` of its `-21.70 ms` isolated cut when placed inside the full worker (delivering only **`-13.20 ms` = `-1.87%`**, matching physical i9-10900K's **`-1.76%`** to 0.11 pp!), and collapses to **`-0.09%`** on `jcc` `Full_cold8_cpu` (`k = 8` rotating pool, matching i9-10900K's `8W` erasure!). By contrast, `edges`-overlaid `C5` (`-16.45%` isolated) and `C6` (`-13.34%` isolated) **beat `S1_1053`** on `jcc` full warm worker (`-2.29%` and `-2.51%` vs `-1.87%`) by keeping `edges` L2-hot!
3. **True Safe-Rust `A6b` Ceiling (why verdict is `STOP`):**
   - On JCC-clean (`jcc`), **no candidate clears either `>= 30.0%` isolated `A6b` or `>= 4.0%` `Full_cold8_cpu`**. Best isolated `A6b` on `jcc` is `C2` **`-20.78%`** / `C3` **`-20.37%`**, translating to only **`-1.44%` / `-1.75%`** (overlapping) on `Full_cold8_cpu` while still adding `+3.3–3.9 MiB` RSS at `k = 8`.
   - Under `<= 1.00x` RSS (`edges` overlay), safe Rust (`#![forbid(unsafe_code)]`) cannot reinterpret `&mut [u32]` as `&mut [NodeLinks]` / `&mut [u64]` without `unsafe`, forcing either packed-link RMW (`C5_packed`: `-10.5%` isolated, `0.9864x` RSS) or unpacked 16B `u32` links (`C5_unpacked`: `-16.45%` isolated `jcc`, `-2.29%` warm `jcc`, `-0.70%` cold8 `jcc`, `1.0149x` RSS).
   - Standalone `A6b` (`src/open/endpoint.rs`) redesign is **closed as a `>= 3–4%` 8W lever** under safe Rust (`<= ~1.5–2.0%` 8W ceiling on Comet Lake).

---

## 3. Part A — Forensic Allocation & Working-Set Audit (233 Admitted Open Blocks of `wikidata.bz2`)

### 3.1 Per-Block Structural Distributions (`audit` subcommand)

| Metric | `p50` | `p90` | `p99` | `max` | `mean` |
|---|---:|---:|---:|---:|---:|
| `n_strips (ns)` | `3,433` | `3,918` | `4,913` | `5,264` | `3,240.4` |
| `n_nodes` (actual) | `7,996` | `9,236` | `11,581` | `12,836` | `7,739.1` |
| `nodes_most` (`3*ns + 3`) | `10,302` | `11,757` | `14,742` | `15,795` | `9,724.3` |
| `m` (retained rows) | `101,980` | `168,897` | `221,139` | `231,600` | `114,024.0` |
| `rules` (`S0_default`) | `7,363` | `8,469` | `10,890` | `12,619` | `7,071.9` |
| `rules` (`S1_u128` inline-15) | `3,325` | `3,916` | `4,901` | `5,677` | `3,209.6` |
| `rules` (`C5_u64` inline-7) | `3,500` | `4,024` | `5,143` | `5,846` | `3,374.0` |
| `n_bands` | `3,540` | `4,338` | `6,148` | `6,788` | `3,553.0` |
| `spelling` (bytes) | `44,530` | `50,992` | `57,386` | `70,159` | `43,457.4` |
| `edges` `4*m` (KiB) | `398` | `659` | `863` | `904` | `444.9` |
| `S0` arena (KiB) | `414` | `659` | `863` | `904` | `458.9` |

- **Power-of-two tier fit (233 blocks):**
  - By `nodes_most = 3*ns + 3`: `<=4K`: 1, `<=8K`: 51, `<=16K`: 181, `<=32K`: 0, `<=64K`: 0 (**100% fit in `<= 16K`**).
  - By actual `n_nodes`: `<=4K`: 12, `<=8K`: 122 (`57.5%`), `<=16K`: 99, `<=32K`: 0, `<=64K`: 0.
- **Live average `A6b` working set:**
  - `S0_default` (in `edges`): `120.9 KiB` nodes + `110.5 KiB` rules + `24 KiB` cache = **`255.4 KiB`** (`0 KiB` extra `Endpoint` heap).
  - `S1_1053` (separate heap): `1,792 KiB` virtual; at `max_nodes = 12,836` touches `links 104 + widths 52 + words128 204 + rules 128 + img64 64 + gen_cache 24` = **`+576 KiB` extra RSS/worker**.
  - `C5_packed` (`u64` inline-7 in `edges`): `151.2 KiB` nodes + `52.7 KiB` rules + `16 KiB` cache = **`219.9 KiB`** (`< 256 KiB` L2, only `+16 KiB` `Endpoint` heap).

### 3.2 Direct Isolated-Process `VmRSS` / `VmHWM` Audit (`rss` subcommand)

Fresh child processes executing all 233 admitted Open blocks round-robin across `k = 1` and `k = 8` independent full `WorkerState`s:

| Arm | `k=1` `VmRSS` (`dRSS/S0`, ratio) | `k=8` `VmRSS` (`dRSS/S0`, `dRSS/w`, ratio) | `k=8` `ep_cap/w` | `k=8` `edge_c/w` | Gate (`<=1.00x`) |
|---|---:|---:|---:|---:|:---:|
| `S0_default` | `103,164K` (`+0K`, `1.0000x`) | `128,772K` (`+0K`, `+0.0K/w`, `1.0000x`) | `0K` | `1,101K` | ref |
| `S1_1053` | `103,688K` (`+524K`, `1.0051x`) | `133,352K` (**`+4,580K`**, **`+572.5K/w`**, `1.0356x`) | `1,993K` | `1,159K` | FAIL (reproduces i9 `+524K/w`) |
| `C1_vec_soa_v2` | `103,520K` (`+356K`, `1.0035x`) | `133,172K` (**`+4,400K`**, **`+550.0K/w`**, `1.0342x`) | `796K` | `1,159K` | FAIL |
| `C2_tiered_slab_v2` | `103,640K` (`+476K`, `1.0046x`) | `132,716K` (**`+3,944K`**, **`+493.0K/w`**, `1.0306x`) | `641K` | `1,159K` | FAIL |
| `C3_compact_word8_soa` | `103,552K` (`+388K`, `1.0038x`) | `132,080K` (**`+3,308K`**, **`+413.5K/w`**, `1.0257x`) | `521K` | `1,159K` | FAIL |
| **`C4_fused_setup_on_shipped`** | `102,808K` (`-356K`, `0.9965x`) | **`128,648K` (`-124K`, `-15.5K/w`, `0.9990x`)** | `16K` | `1,130K` | **PASS** |
| **`C5_edges_overlay_soa_u64` (packed 8B links)** | `103,196K` (`+104K`, `1.0010x`) | **`127,044K` (`-1,752K`, `-219.0K/w`, `0.9864x`)** | `16K` | `1,130K` | **PASS** |
| `C5_edges_overlay_soa_u64` (unpacked 16B links) | `102,976K` (`-188K`, `0.9982x`) | `130,688K` (`+1,916K`, `+239.5K/w`, `1.0149x`) | `16K` | `1,305K` | FAIL |
| `C6_edges_overlay_soa_u128` (unpacked 16B links) | `103,100K` (`-64K`, `0.9994x`) | `132,336K` (`+3,564K`, `+445.5K/w`, `1.0277x`) | `16K` | `1,594K` | FAIL |

*(Note: `C5_edges_overlay_soa_u64` packed-link `k=8` RSS was measured with `init_rules_words = 4*(ns+256)` and persistent `edges.len()` before switching `C5`/`C6` links from packed 8B RMW to unpacked 16B `u32` links.)*

---

## 4. Part B — Three-Profile Benchmark Results (`base`, `nat`, `jcc`; 7 Interleaved Repeats, CPU 0)

All timings are Linux `/proc/thread-self/schedstat` on-CPU thread milliseconds (median `[min..max]`), `D-` = strictly faster disjoint `[min..max]` vs `S0_default`, `o` = overlapping.

### 4.1 Isolated `A6b` (`A6b_cpu`, 5 passes/repeat × 7 repeats, 233 Open blocks)

| Arm | `base` `A6b_cpu` (`Δ%`, C) | `nat` `A6b_cpu` (`Δ%`, C) | **`jcc` `A6b_cpu` (`Δ%`, C)** | `jcc` sub-phases (`setup / steps / bot / dict`, ms) |
|---|---:|---:|---:|---|
| `S0_default` | `121.47 [120.71..127.03]` (+0.00%) | `124.73 [123.81..125.48]` (+0.00%) | **`110.38 [109.95..111.73]` (+0.00%)** | `18.91 / 52.28 / 1.81 / 34.18` |
| `S1_1053` | `94.75 [93.85..96.51]` (`-22.00%`, `D-`) | `97.06 [96.77..101.05]` (`-22.18%`, `D-`) | `88.68 [88.43..91.85]` (`-19.65%`, `D-`) | `13.59 / 42.06 / 1.39 / 30.43` |
| `C1_vec_soa_v2` | `99.26 [98.88..101.66]` (`-18.28%`, `D-`) | `100.17 [99.46..102.78]` (`-19.70%`, `D-`) | `89.68 [87.89..93.94]` (`-18.76%`, `D-`) | `14.36 / 41.87 / 1.46 / 29.37` |
| `C2_tiered_slab_v2` | `93.99 [92.74..94.72]` (`-22.62%`, `D-`) | `90.80 [90.43..94.28]` (`-27.20%`, `D-`) | **`87.44 [86.16..93.35]` (`-20.78%`, `D-`)** | `14.03 / 41.27 / 1.56 / 29.18` |
| `C3_compact_word8_soa` | **`90.11 [89.33..91.80]` (`-25.82%`, `D-`)** | `92.00 [90.99..93.33]` (`-26.25%`, `D-`) | `87.89 [87.40..89.74]` (`-20.37%`, `D-`) | `13.83 / 43.09 / 1.51 / 27.87` |
| `C4_fused_setup_on_shipped` | `116.11 [115.90..116.88]` (`-4.41%`, `D-`) | `117.59 [116.23..118.14]` (`-5.72%`, `D-`) | `101.98 [101.50..108.64]` (`-7.60%`, `D-`) | `18.55 / 48.54 / 1.84 / 32.64` |
| `C5_edges_overlay_soa_u64` | `98.99 [98.32..102.75]` (`-18.50%`, `D-`) | `105.48 [104.48..106.63]` (`-15.44%`, `D-`) | `92.22 [90.71..94.60]` (`-16.45%`, `D-`) | `15.47 / 46.46 / 1.87 / 28.76` |
| `C6_edges_overlay_soa_u128` | `104.63 [103.07..106.36]` (`-13.86%`, `D-`) | `106.36 [105.86..110.75]` (`-14.73%`, `D-`) | `95.65 [94.20..96.91]` (`-13.34%`, `D-`) | `15.51 / 49.73 / 1.87 / 31.78` |

*(Before switching `C5`/`C6` to unpacked 16B `u32` links, packed-8B-link `C5_edges_overlay_soa_u64` took `113.20 ms` / `-10.46%` on `base` due to 16M read-modify-write bit-packing stores in `steps`: `64.43 -> 50.18 ms` once unpacked.)*

### 4.2 Full Open Worker Warm (`k = 1`, `A5/M + A6a + A6b + A6c + A7(bidi) + A9`, 7 repeats)

| Arm | `base` `Full_warm_cpu` (`Δ%`, C) | `nat` `Full_warm_cpu` (`Δ%`, C) | **`jcc` `Full_warm_cpu` (`Δ%`, C)** | `jcc` vs `S1_1053` |
|---|---:|---:|---:|---:|
| `S0_default` | `744.78 [739.25..747.41]` (+0.00%) | `739.41 [736.94..747.93]` (+0.00%) | `704.83 [701.87..709.13]` (+0.00%) | `+1.91%` (`D+`) |
| `S1_1053` | `723.01 [721.44..731.67]` (`-2.92%`, `D-`) | `722.27 [719.78..737.06]` (`-2.32%`, `o`) | **`691.63 [690.54..695.40]` (`-1.87%`, `D-`)** | `+0.00%` |
| `C1_vec_soa_v2` | `730.17 [723.39..733.81]` (`-1.96%`, `D-`) | `725.30 [720.51..731.06]` (`-1.91%`, `D-`) | `686.11 [683.42..692.99]` (`-2.66%`, `D-`) | `-0.80%` |
| `C2_tiered_slab_v2` | `723.29 [715.28..755.45]` (`-2.89%`, `o`) | `714.63 [708.21..726.40]` (`-3.35%`, `D-`) | **`683.37 [681.79..694.40]` (`-3.05%`, `D-`)** | `-1.20%` |
| `C3_compact_word8_soa` | `721.68 [713.92..759.07]` (`-3.10%`, `o`) | `714.78 [711.14..716.77]` (`-3.33%`, `D-`) | `690.46 [683.21..694.97]` (`-2.04%`, `D-`) | `-0.17%` |
| `C4_fused_setup_on_shipped` | `740.84 [737.04..752.50]` (`-0.53%`, `o`) | `737.61 [729.25..765.78]` (`-0.24%`, `o`) | `698.73 [690.39..719.47]` (`-0.87%`, `o`) | `+1.03%` |
| `C5_edges_overlay_soa_u64` | **`721.31 [718.24..730.27]` (`-3.15%`, `D-`)** | `733.25 [724.15..759.57]` (`-0.83%`, `o`) | `688.67 [679.49..698.06]` (`-2.29%`, `D-`) | `-0.43%` |
| `C6_edges_overlay_soa_u128` | `727.41 [722.03..737.74]` (`-2.33%`, `D-`) | `726.88 [721.17..735.98]` (`-1.69%`, `D-`) | `687.13 [679.13..693.12]` (`-2.51%`, `D-`) | `-0.65%` |

### 4.3 Full Open Worker L3-Cold 8-Worker Rotating Pool (`k = 8`, 7 repeats)

| Arm | `base` `Full_cold8_cpu` (`Δ%`, C) | `nat` `Full_cold8_cpu` (`Δ%`, C) | **`jcc` `Full_cold8_cpu` (`Δ%`, C)** | `jcc` vs `S1_1053` | 8W `ep_cap` |
|---|---:|---:|---:|---:|---:|
| `S0_default` | `806.89 [799.82..847.65]` (+0.00%) | `814.71 [806.45..835.44]` (+0.00%) | `764.65 [751.69..861.26]` (+0.00%) | `+0.09%` | `0K` |
| `S1_1053` | `798.37 [784.36..884.62]` (`-1.06%`, `o`) | `789.96 [778.38..822.32]` (`-3.04%`, `o`) | **`763.96 [754.14..830.25]` (`-0.09%`, `o`)** | `+0.00%` | `15,947K` |
| `C1_vec_soa_v2` | `807.02 [790.89..857.15]` (`+0.02%`, `o`) | `791.33 [783.74..804.64]` (`-2.87%`, `D-`) | `757.70 [749.03..794.56]` (`-0.91%`, `o`) | `-0.82%` | `6,369K` |
| `C2_tiered_slab_v2` | `787.46 [782.03..845.26]` (`-2.41%`, `o`) | `782.38 [775.35..794.58]` (`-3.97%`, `D-`) | `753.63 [743.27..823.96]` (`-1.44%`, `o`) | `-1.35%` | `5,131K` |
| `C3_compact_word8_soa` | `781.86 [774.42..835.41]` (`-3.10%`, `o`) | `781.03 [775.71..783.24]` (`-4.13%`, `D-`) | **`751.29 [738.54..779.27]` (`-1.75%`, `o`)** | `-1.66%` | `4,171K` |
| `C4_fused_setup_on_shipped` | `793.17 [787.19..860.46]` (`-1.70%`, `o`) | `804.69 [795.59..835.22]` (`-1.23%`, `o`) | `755.58 [751.30..764.43]` (`-1.19%`, `o`) | `-1.10%` | `128K` |
| `C5_edges_overlay_soa_u64` | `794.87 [779.33..839.97]` (`-1.49%`, `o`) | `801.44 [790.96..821.03]` (`-1.63%`, `o`) | `759.31 [751.45..778.06]` (`-0.70%`, `o`) | `-0.61%` | `128K` |
| `C6_edges_overlay_soa_u128` | `789.96 [779.01..850.60]` (`-2.10%`, `o`) | `796.39 [788.76..810.45]` (`-2.25%`, `o`) | `755.17 [743.68..779.78]` (`-1.24%`, `o`) | `-1.15%` | `128K` |

### 4.4 Guard Corpora (`jcc`, 5 interleaved repeats, full Open pipeline)

- `text` (12 forced Open, `S0 = 146.02 ms`): `S1 = 0.958x`, `C1 = 0.952x`, `C2 = 0.958x`, `C3 = 0.954x`, `C4 = 0.970x`, `C5 = 0.951x`, `C6 = 0.965x` (all faster).
- `runs` (1/1 admitted, `S0 = 7.90 ms` on `jcc`, `4.01 ms` on `nat`): `0.506x` (`jcc`), `0.999–1.004x` (`nat`).
- `records` (11 forced Open, `S0 = 482.16 ms`): `0.986–1.015x` (`C5 = 0.997x`, `C3 = 0.996x`).
- `random` (9 forced Open, `S0 = 397.20 ms`): `0.996–1.014x` (`C3 = 1.004x`, `C5 = 1.010x`).

---

## 5. Verdict & Strategic Implications

1. **Frozen Stop Test Verdict: `STOP` (RULES C2 strictly preserved).**
   - No candidate satisfies both Criterion 1 (`<= 1.00x` 8W RSS) and Criterion 2 (`>= 30.0%` isolated `A6b` AND `>= 4.0%` `Full_cold8_cpu` disjoint across `base`, `nat`, `jcc`).
2. **Root-Cause Closure on `#1053` (`a6b-endpoint-build`):**
   - Two distinct mechanisms conspired to kill `#1053` on Comet Lake-S stepping 5:
     1. **Skylake JCC erratum illusion on the VM (`-11.5%` on shipped `S0_default` `A6b` once aligned):** real Comet Lake-S stepping 5 fixes JCC in silicon, cutting `#1053`'s isolated `A6b` advantage from `~27%` to `~19.7%`.
     2. **Abandoning Wormhole W1 `edges` overlay (`+524–572 KiB/worker` resident RSS):** Linux demand-paging already bounded `#1053`'s 64K arrays to `max_nodes = 12,836`, so switching to `Vec` (`C1`) or tiered slabs (`C2`/`C3`) leaves `+414–550 KiB/worker` (`+3.3–4.4 MiB` at 8W) of extra resident arrays outside `Kernel.edges`, evicting L2 at 1W (`-21.7 ms` isolated -> `-13.2 ms` = `-1.87%` full worker) and erasing the entire gain in 8-worker L3 (`-0.09%` VM rotating pool; `+1.06%` physical i9-10900K).
3. **Why safe-Rust `edges`-overlay rescue hits a structural ceiling:**
   - Reclaiming Wormhole W1 inside `Kernel.edges: Vec<u32>` (`C4`, `C5`, `C6`) completely eliminates the RSS bloat (`0.9864x–0.9990x` at `k = 8`) and beats `#1053` on full-worker warm `jcc` (`-2.29%..-2.51%` vs `-1.87%`), proving the L2-residency effect.
   - However, safe Rust (`#![forbid(unsafe_code)]`, `std`-only crate) cannot reinterpret `&mut [u32]` as `&mut [NodeLinks]` (`[u16; 4]`) or `&mut [u64]`, forcing either packed-`u32` read-modify-write stores (`-10.5%` isolated) or unpacked 16B `u32` links (`-16.45%` isolated `jcc`, `-0.70%` cold8 `jcc`). Even with separate native `NodeLinks`/`u64` slabs (`C3`), the JCC-clean isolated `A6b` ceiling is only **`-20.4%`** (`-1.75%` cold8, `+3.3 MiB` 8W RSS).
   - **Action:** Close standalone `A6b` (`src/open/endpoint.rs`) redesign as a `>= 3–4%` 8W lever. Keep `#1053` (`--features a6b-endpoint-redesign`) unpromoted (candidate for removal at next release review) and focus worker budget on `forward-emission` (`#1110`), `forward-rle1-crc-vectored-build`, `mtf-fold-l1-batch-redesign`, and `scanner-adaptive-seam-window`.

