# Rows-based `runt-exempt-window`: dissecting Wikimedia runt dilution, `#1282` interaction, and expansive-run RSS safety

*28 November 2026 — smooth-zone exploration notebook (zero `src/` edits in this PR; verified prototype committed locally on `explore/runt-window` and not pushed while `#1282` holds `src/parallel.rs`). All timings below were measured on a 2-vCPU `Intel(R) Xeon(R) CPU @ 2.30GHz` Linux 6.18.57 VM (`rustc 1.99.1`, `lto = "fat"`, `codegen-units = 1`) under shared swarm load (load avg ~8–9); they are VM exploration diagnostics, not physical i9-10900K adoption evidence (C2).*

---

## Summary & verdict

1. **Exact rows-based classification (`RUNT_ROWS = 1_200`) completely fixes the `runt-exempt-window` premise defect.**
   - Computing exact BWT row count `n` from `symbols.syms` after `Scanner::next` (O(1) fast-reject via `syms.len() > RUNT_ROWS + 1` on full blocks; ~15-symbol early exit on expansive `RUNA`/`RUNB` run-only blocks) captures **198 / 198 (100.0%)** of Wikimedia `wikidata.bz2` runts (`n = 1..1,185`, decoded `1..1,183 B`) while classifying **0 / 68 (0.0%)** of `onebyte`, `pattern`, `zeros-20blk`, and `runs` blocks as runts (whereas the original `n_sel * 50` heuristic misclassified **100%** of `onebyte`, `pattern`, and `zeros-20blk` blocks!).
   - Under bzip2's worst-case RLE1 expansion (`259 / 5 = 51.8×`), `1,200` rows decode to at most **`62,160 B`**. Detaching a tiny exact-sized `Vec<u16>` copy (`<= 2,402 B`) for exempt runts and retaining the recycled ~1.8 MiB `syms` buffer on the scanner guarantees **`<= 64,562 B < 64 KiB` (`POOLED_FROM`)** per exempt place for **symbols + decoded output combined**, even on adversarial inputs.
   - Peak `VmHWM` on a 22-block, `977 MB` decoded all-zeros stress file (`zeros-20blk.bz2`, `45.9 MiB/block`, `721 B` compressed) is **`534.7 → 534.8 MiB (1.000×)`** at `t = 4, s = 16` and **`844.3 → 861.8 MiB (1.021×)`** at `t = 8, s = 32`.
2. **`#1282` (`plan-schedule-fix`) does NOT eliminate Wikimedia runt dilution.**
   - Auditing `#1282` (`5e98b825`) shows `plan_with` lowers `LAZY_FROM_THREADS` from `8` to `4` (switching `t = 4` default on `< 6` cores from `EagerCells` to faster `OpenCells`, keeping `slots = 16`) and scales `window()` **down** from `4 * t` to `4 * phys` on oversubscribed 6C/8C/10C machines. Faster workers and tighter oversubscribed windows preserve (or sharpen) the ~46% runt slot dilution.
3. **Paired VM A/B results (`n = 11` alternating rounds on `1×`, `n = 9` on `4×` and guards; 72/72 SHA-256 parity across 9 corpora; 78/78 unit + integration tests pass):**
   - `wikidata.bz2` (`1×`, 431 blocks, 198 runts):
     - `t = 4, s = 8` lazy (`2×t` `bzcat` window): **`0.8813 [0.8331, 0.9324]` (−11.87% wall)**, CPU `1.0002×`.
     - `t = 4, s = 16` default eager (`origin/master`): **`0.9510 [0.9051, 0.9992]` (−4.90% wall)**, CPU `0.9911×`, RSS `0.991×`.
     - `t = 4, s = 16` lazy (`#1282` default): **`0.9674 [0.9410, 0.9946]` (−3.26% wall)**, CPU `0.9855×`, RSS **`1.004×`** (`36.1 → 36.3 MiB`).
     - `t = 8, s = 32` lazy (`origin/master` & `#1282` default): **`0.9771 [0.9424, 1.0130]` (−2.29% GM / −4.57% median)**, RSS `1.031×`; with `--features forward-anchor-lanes`: **`0.9634 [0.9059, 1.0245]` (−3.66% GM)**.
   - `wikidata-x4.bz2` (`4×`, 1,724 blocks, 839 MB): `t = 8, s = 32` lazy **`0.9910 [0.9467, 1.0373]` (−0.90% GM)** (`base`, matching `eight-worker-coupling.md`'s physical i9 counterfactual `−0.76%..−1.08%`) and **`0.9502 [0.8704, 1.0372]` (−4.98% GM)** (`forward-anchor-lanes`); `t = 4, s = 16` lazy **`0.9947 [0.9698, 1.0204]` (−0.53% GM)** on this 2×-overcommitted 2-vCPU VM.
   - Runt-free control `wikidata-bzip2-9.bz2` (bit-identical decoded bytes `3f075e30…`, 234 blocks, 0 runts): **`0.9959 [0.9629, 1.0299]`** (`t4 s16`) and **`0.9920 [0.9290, 1.0592]`** (`t8 s32`) — flat within `±1%`.
   - **Honest caveats to verify before adoption:** (a) `wikidata` `t = 2, s = 8` eager showed **`+3.46%..+4.50%`** wall on this loaded 2-vCPU VM (`1.0450 [0.9995, 1.0926]` in Part A; `1.0346 [1.0042, 1.0659]` in Part B, despite `0.999×` RSS and `1.009×` CPU — likely scanner preemption of the 2 eager workers on 2 vCPUs when allowed to run 16 places ahead); (b) `4×` vs `1×` at `t = 4, s = 16` needs quiet 4-vCPU and physical i9 verification.
4. **Disposition:** Re-register as clean P3 `idea` claim `frontier/claims/runt-exempt-window-rows.toml` (queued strictly behind `#1282` on `src/parallel.rs`, with explicit `zeros-20blk`/`onebyte`/`runs`/`pattern` RSS guard and `t = 2` non-regression guard — gating to `threads >= 4` / `slots >= 12` if `t = 2` regresses on quiet hardware), and mark the premise-defective `frontier/claims/runt-exempt-window.toml` as `status = "dropped"` without modifying its frozen `stop_test` (C2).

---

## 1. Exact 6-corpus block census (`syms.len()` vs exact BWT rows `n`)

Re-encoding `corpora/wikidata.bz2` (`82f65ace…`, `11,854,933 B`) with Python's bundled `libbzip2 -9` produced `wikidata-bzip2-9.bz2` (`817e6d89…`, `11,863,758 B` — bit-identical to `encoder-dependence.md`), decoding to identical `209,700,196` bytes (`SHA-256 = 3f075e30f4f8c73414150655afbf57caa2647c7927e743fc0635b4912d65cdaf`). We also generated `zeros-20blk.bz2` (`721 B` compressed, `977,272,832 B` decoded across 22 all-zero blocks).

| Corpus | Blocks | Old heuristic `syms < 1,000` | Rows `<= 1,024` | Rows `<= 1,200` (`RUNT_ROWS`) | Full (`rows >= 65,536`) | Max decoded bytes of `syms < 1,000` | Max decoded bytes of `rows <= 1,200` |
|---|---:|---:|---:|---:|---:|---:|---:|
| `wikidata.bz2` | 431 | 198 (45.9%) | 197 (45.7%) | **198 (45.9%)** | 233 (54.1%) | 1,183 B | **1,183 B** |
| `wikidata-bzip2-9.bz2` | 234 | 0 (0.0%) | 0 (0.0%) | **0 (0.0%)** | 233 (99.6%)* | — | — |
| `onebyte.bz2` | 1 | **1 (100% FALSE)** | 0 (0.0%) | **0 (0.0%)** | 1 (100.0%) | **40,000,000 B** | **0 B** |
| `pattern.bz2` | 44 | **44 (100% FALSE)** | 0 (0.0%) | **0 (0.0%)** | 44 (100.0%) | **899,981 B** | **0 B** |
| `zeros-20blk.bz2` | 22 | **22 (100% FALSE)** | 0 (0.0%) | **0 (0.0%)** | 22 (100.0%) | **45,899,235 B** | **0 B** |
| `runs.bz2` | 1 | 0 (0.0%) | 0 (0.0%) | **0 (0.0%)** | 1 (100.0%) | — | — |

*\*`wikidata-bzip2-9.bz2` has 1 trailing block of `10,979` rows (`5,521` syms).*

On `wikidata.bz2`, the 198 Wikimedia runts have `syms = [2, p50=22, max=845]`, `rows = [1, p50=22, p95=146, max=1,185]`, and `decoded = [1, p50=22, max=1,183 B]`. Setting `RUNT_ROWS = 1_200` captures **100.0%** of Wikimedia runts while proving:
- Max RLE1 decoded output per exempt runt: `1,200 * 259 / 5 = 62,160 B`.
- Max detached `syms` buffer per exempt runt: `1,201 * 2 = 2,402 B`.
- Combined peak per exempt place: **`<= 64,562 B < 65,536 B` (`POOLED_FROM`)**.

---

## 2. Baseline Wikimedia vs `bzip2 -9` runt-dilution gap & `#1282` audit

On `origin/master` (`3c7212bb`), `n = 9` alternating rounds comparing `wikidata.bz2` vs `wikidata-bzip2-9.bz2` (identical decoded bytes):

| Configuration (`mode`, `t`, `slots`) | `wikidata` med (min) ms | `bzip2-9` med (min) ms | Ratio `wiki/bz2` [95% CI] | CPU `w/b` | Peak `VmHWM` `w/b` MiB |
|---|---:|---:|---:|---:|---:|
| `default` `t=2, s=8` (eager, `4×t`) | 2254.2 (1866.2) | 2258.1 (2193.1) | `0.9800 [0.9280, 1.0350]` | `0.9879` | 50.2 / 48.5 |
| `default` `t=2, s=4` (eager, `2×t`) | 2243.6 (1697.5) | 2048.3 (1940.1) | `1.0411 [0.9422, 1.1504]` | `0.9757` | 47.2 / 47.0 |
| `default` `t=4, s=16` (eager, `origin/master`) | 1615.6 (1477.1) | 1683.7 (1550.9) | `0.9665 [0.9156, 1.0202]` | `0.9912` | 79.1 / 76.0 |
| `workers-lazy` `t=4, s=8` (`2×t` `bzcat`) | 1601.0 (1503.2) | 1456.2 (1424.1) | **`1.0914 [1.0510, 1.1333]`** | `0.9882` | 31.1 / 33.0 |
| `workers-lazy` `t=4, s=16` (`#1282` default) | 1503.4 (1400.2) | 1514.9 (1337.6) | `1.0274 [0.9724, 1.0856]` | `1.0190` | 35.8 / 36.9 |
| `workers-lazy` `t=4, s=32` (`8×t` oracle) | 1419.0 (1368.8) | 1474.2 (1364.9) | **`0.9603 [0.9273, 0.9944]`** | `0.9873` | 40.0 / 40.5 |
| `workers-lazy` `t=8, s=32` (prod default) | 1106.4 (1049.6) | 1061.5 (1009.4) | **`1.0402 [1.0107, 1.0705]`** | `0.9990` | 49.0 / 51.3 |
| `workers-lazy` `t=8, s=64` (`8×t` oracle) | 1086.9 (1036.9) | 1085.9 (1027.9) | `1.0087 [0.9613, 1.0586]` | `1.0049` | 55.6 / 53.3 |

**Key takeaway:** Doubling `slots` unconditionally (`16 → 32` at `t = 4`, `32 → 64` at `t = 8`) closes the Wikimedia dilution gap completely, confirming pure slot starvation (identical CPU `0.988–0.999×`), but increases baseline RSS by `+11.7%..+13.5%` on `wikidata` (and inflates `zeros-20blk` proportionally to `slots` if uncapped). Inspecting `#1282` (`5e98b825`, `src/parallel.rs:3509–3517, 3768–3793`) confirms it does **not** widen `slots`: on `< 6` cores it switches `t = 4` from `EagerCells` to faster `OpenCells` (`workers-lazy 4 16`), and on oversubscribed `6C/8C/10C` machines (`n < threads`) it scales `window()` **down** to `4 * phys`.

---

## 3. Prototype design (`explore/runt-window`, local-only, zero `unsafe`)

Implemented locally in `/home/antigravity/wt-explore-runt-window/src/parallel.rs` (not pushed):

1. **`exact_rows_le(&symbols, RUNT_ROWS)` (`RUNT_ROWS = 1_200`):**
   - Fast-path rejects full blocks in O(1) (`if symbols.syms.len() > max_rows + 1 { return false; }`) with zero `syms` cache-line reads on runt-free files.
   - Walks RLE2 (`RUNA = 0`, `RUNB = 1`) with saturating/checked arithmetic, returning `false` as soon as `rows > max_rows`, on `weight` shift overflow, or if `eob` is absent.
2. **Scanner-side `spare_syms` detachment (preventing `1.8 MiB` buffer hoarding by exempt runts):**
   - When `is_runt` is true and `symbols.syms.capacity() > RUNT_ROWS + 1`, the scanner copies the `<= 1,201` `u16`s (`<= 2.4 KiB`) into a fresh exact-sized `Vec<u16>` and retains the recycled large buffer locally in `spare_syms` for the next block.
   - `worker` and `pipelined_worker` only push `syms` back into `free_syms` when `syms.capacity() > RUNT_ROWS + 1`, preventing tiny runt vectors from polluting `free_syms`.
3. **Exact `full_in_flight` ring accounting in `State` and `Shared::full`:**
   - `Ready::new(slots.saturating_mul(2))` and `in_flight_full: Vec<bool>` (power-of-two ring of length `(2 * slots).next_power_of_two()`) guarantee zero `BTreeMap` spillover.
   - `State::issue_place(full: bool)` and `State::take_place()` maintain `full_in_flight` across normal blocks, error slots in `finish(Some(e))`, and `replay_scanner`.
   - `Shared::full` blocks when `ahead >= 2 * slots || full_in_flight >= slots`, or under `READY_BYTES` when `ready.bytes >= cap.bytes && (full_in_flight >= cap.floor || ahead >= 2 * cap.floor)`. On expansive run-only inputs (`0` exempt blocks), `full_in_flight == ahead` identically, leaving `ready-cap-worker-gate` behavior 100% unchanged.

**Verification:**
- **72 / 72 SHA-256 byte-exact parity checks PASS** across all 9 corpora (`wikidata`, `wikidata-bzip2-9`, `runs`, `onebyte`, `pattern`, `text`, `records`, `random`, `zeros-20blk`) on both `base` and `--features forward-anchor-lanes` at `t = 2, 4 (s=8, 16), 8`.
- **78 / 78 Rust tests PASS** (`55/55` `parallel::` including 2 new Miri-safe unit tests `exact_rows_le_classifies_runts_and_rejects_expansive_runs` and `runt_exempt_window_bounds_full_and_total_places`, `12/12` `pipelined`, `10/10` `service`, `1/1` `reader_panic`), `cargo fmt --check` clean.

---

## 4. Paired A/B results: prototype vs `origin/master` (`3c7212bb`)

### Part A — `wikidata.bz2` (`1×`, `base` & `forward-anchor-lanes`) and `wikidata-bzip2-9.bz2` control (`n = 11` alternating rounds)

| Case | Baseline med (min) ms | Proto med (min) ms | `proto/base` GM [95% CI] | CPU `p/b` | Peak `VmHWM` `b → p` MiB (`×`) |
|---|---:|---:|---:|---:|---:|
| `wiki base t2 s8 def` | 2937.6 (2629.6) | 3056.2 (2696.4) | `1.0450 [0.9995, 1.0926]` | `1.0325` | 50.1 → 51.1 (`1.019×`) |
| `wiki base t4 s16 def` | 1890.2 (1790.2) | 1820.7 (1623.2) | **`0.9510 [0.9051, 0.9992]`** | `0.9911` | 81.5 → 80.7 (`0.991×`) |
| `wiki base t4 s8 lazy` (`2×t`) | 1755.2 (1515.5) | 1545.7 (1463.9) | **`0.8813 [0.8331, 0.9324]`** | `1.0002` | 31.1 → 32.6 (`1.048×`) |
| `wiki base t4 s16 lazy` (`#1282` def) | 1601.0 (1464.7) | 1573.7 (1462.2) | **`0.9674 [0.9410, 0.9946]`** | `0.9855` | 36.1 → 36.3 (**`1.004×`**) |
| `wiki base t8 s32 lazy` (prod def) | 1176.4 (1070.2) | 1122.6 (1054.8) | `0.9771 [0.9424, 1.0130]` | `1.0026` | 49.7 → 51.2 (`1.031×`) |
| `wiki lanes t4 s16 lazy` | 1445.8 (1306.1) | 1368.3 (1262.6) | `0.9864 [0.9253, 1.0516]` | `0.9922` | 35.9 → 37.8 (`1.052×`) |
| `wiki lanes t8 s32 lazy` | 1054.7 ( 973.1) | 1041.9 ( 934.8) | `0.9634 [0.9059, 1.0245]` | `0.9889` | 49.5 → 53.2 (`1.075×`) |
| `bzip2-9 base t4 s16 lazy` (control) | 1599.1 (1490.0) | 1602.5 (1413.5) | **`0.9959 [0.9629, 1.0299]`** | `1.0103` | 36.4 → 37.0 (`1.016×`) |
| `bzip2-9 base t8 s32 lazy` (control) | 1037.9 ( 954.1) |  984.7 ( 930.5) | **`0.9920 [0.9290, 1.0592]`** | `0.9743` | 53.0 → 52.4 (`0.988×`) |

### Part B — `wikidata-x4.bz2` (`4×`, 839 MB), `t = 2` recheck, and expansive run-only RSS guards (`n = 9` alternating rounds)

| Case | Baseline med (min) ms | Proto med (min) ms | `proto/base` GM [95% CI] | CPU `p/b` | Peak `VmHWM` `b → p` MiB (`×`) |
|---|---:|---:|---:|---:|---:|
| `wiki-x4 base t4 s16 lazy` | 6362.9 (5806.4) | 6391.6 (5742.4) | `0.9947 [0.9698, 1.0204]` | `1.0010` | 47.8 → 49.9 (`1.044×`) |
| `wiki-x4 base t8 s32 lazy` | 4427.0 (4068.7) | 4365.7 (4046.7) | `0.9910 [0.9467, 1.0373]` | `1.0138` | 65.0 → 66.8 (`1.028×`) |
| `wiki-x4 lanes t8 s32 lazy` | 4918.4 (3952.7) | 4772.6 (4059.3) | `0.9502 [0.8704, 1.0372]` | `0.9709` | 64.9 → 66.9 (`1.030×`) |
| `wiki-1x base t2 s8 def` (recheck) | 3162.6 (2981.3) | 3307.7 (3156.2) | `1.0346 [1.0042, 1.0659]` | `1.0090` | 50.4 → 50.4 (`0.999×`) |
| `zeros-20blk t4 s16 lazy` (977 MB) | 2804.9 (2287.4) | 2842.6 (2591.7) | `0.9983 [0.9197, 1.0836]` | `0.9922` | **534.7 → 534.8 (`1.000×`)** |
| `zeros-20blk t8 s32 lazy` (977 MB) | 3172.1 (2479.0) | 2989.3 (2425.9) | `1.0085 [0.9494, 1.0712]` | `1.0029` | **844.3 → 861.8 (`1.021×`)** |
| `onebyte t4 s16 lazy` (40 MB) |  535.7 ( 505.2) |  503.5 ( 471.0) | `0.9766 [0.9147, 1.0426]` | `0.9855` | 49.8 → 51.0 (`1.024×`) |
| `onebyte t8 s32 lazy` (40 MB) |  489.7 ( 403.8) |  517.9 ( 388.6) | `0.9841 [0.8540, 1.1341]` | `1.0325` | 50.0 → 53.4 (`1.068×`) |
| `runs t4 s16 lazy` (20 MB) |  284.5 ( 233.3) |  323.0 ( 268.5) | `1.0450 [0.9079, 1.2028]` | `1.0292` | 33.3 → 29.8 (`0.895×`) |
| `pattern t4 s16 lazy` (44 blks) |  489.7 ( 452.0) |  491.9 ( 449.4) | `1.0018 [0.9387, 1.0692]` | `1.0220` | 34.2 → 34.9 (`1.018×`) |
| `pattern t8 s32 lazy` (44 blks) |  389.2 ( 331.2) |  371.2 ( 318.7) | `0.9381 [0.8403, 1.0472]` | `1.0081` | 66.6 → 66.5 (`0.999×`) |

---

## 5. Open caveats & next step

1. **`t = 2` on 2-vCPU VM (`+3.46%..+4.50%` wall despite `0.999×` RSS and `1.009×` CPU):** Reproduced across both Part A and Part B. On a 2-vCPU machine running 2 eager `Cells` workers, letting the scanner run 16 places ahead (instead of 8) keeps the scanner awake longer competing for the same 2 vCPUs while the 2 workers invert blocks 0..1. The re-registered claim `runt-exempt-window-rows` explicitly includes `t = 2` as a non-regression guard on quiet 4-vCPU and physical i9 hardware (if `t = 2` still regresses there, gate runt exemption to `slots >= 12` / `threads >= 4`).
2. **`wikidata-x4` at `t = 4, s = 16` (`−0.53%`) vs `wikidata-1x` (`−3.26%`):** On a 2×-overcommitted loaded 2-vCPU VM, 4 workers are severely CPU-throttled (`6.36 s` vs `1.57 s`), so the scanner runs far ahead regardless. Quiet 4-vCPU CI and physical i9 (`k = 4, 6, 8, 10`) measurement after `#1282` lands will give the clean adoption verdict.
3. **Sequencing:** `runt-exempt-window-rows` (`priority = 3`, `status = "idea"`, `owned_paths = ["src/parallel.rs"]`) waits strictly until `#1282` (`plan-schedule-fix`) releases `src/parallel.rs`.
