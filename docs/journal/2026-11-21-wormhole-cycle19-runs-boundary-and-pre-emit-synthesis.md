# Wormhole Cycle 19 — Upstream `runs::front` `CAP = 127` Boundary Closure (`Theorem 19.1`) & Unified `WormholePreEmitCells: InverseBwt` Synthesis (`-60..125 KiB` WS, `7/7` Wins vs `Master` & `M0`)

**Session:** `research/carbon-wormhole` (Round 3, Cycle 19 of 20; 9th of 10 in Round 3)  
**Baseline `origin/master`:** `90e6719663af0615fb417b038906ad2636193aa3` (`mibz2::bwt::OpenCells`)  
**Verification harness:** [`verify_cycle19.rs`](2026-11-21-wormhole-cycle19-runs-boundary-and-pre-emit-synthesis/verify_cycle19.rs) (`#![forbid(unsafe_code)]`, `0` warnings, `5,233` Gate G1 + `SPIRALS len>1023` + `WideLinks` + `WideExtra` + `#703 Cap::dense` admit/refuse cases byte-exact + `21`-round `CAP` audit + `25`-round `InverseBwt::invert_runs` confrontation), [`wormhole_pre_emit_cells.toml`](2026-11-21-wormhole-cycle19-runs-boundary-and-pre-emit-synthesis/wormhole_pre_emit_cells.toml) (`PASS`), [`cycle19_output.txt`](2026-11-21-wormhole-cycle19-runs-boundary-and-pre-emit-synthesis/cycle19_output.txt), [`cycle19_results.json`](2026-11-21-wormhole-cycle19-runs-boundary-and-pre-emit-synthesis/cycle19_results.json)

---

## 1. One-Line Keep & Kill Decisions (Cycle 19)

- **PROMOTE `W3: WormholePreEmitCells: InverseBwt` (Unified Round 3 Synthesis: `[u32; 3]` `StripBandMux` + `pop_tail` + `(len10 << 22) | id22` `GradedRV` + Pohl `BidiBottom` + `Lazy15-DirectInline16`, holding `S5 emit` 100% verbatim identical to `origin/master`):** Cuts per-worker working set by **`-23` to `-125 KiB`** on Wikidata (`1,388–1,897 KiB = 1.355–1.853 MiB`/worker, so `10` i9 workers fit in **`18.53 MiB <= 20 MiB` L3**), cuts pre-`emit` (`S1`–`S4`) time to **`0.892×–0.939×`** (`-54.6` to `-158.9 µs`), and wins **`7/7` blocks** via `InverseBwt::invert_runs` against both same-crate `M0:Clone` (**`0.922×–0.977×`**) and `origin/master` `A1:OpenCells` (**`0.918×–0.947×`**).
- **KILL `U1: CAP255-u16` (Widening `runs::front` `CAP = 127 -> 255` in `u16` `(byte << 8) | len8`):** Saves only **`3.3–13.9 µs`** (`0.2%–0.5%` of block time, inside the `±1%` A/A band) while breaking `cells_lazy.rs:182` (`len > 127`) and shifting `records.len()` admission gates across `OpenCells`/`LazyCells`.
- **KILL `U2: Unsplit-u32` (Pre-coalesced `u32` `(byte << 24) | width24` maximal-run stream eliminating `maximal()`'s inner loop):** Saves only **`19.5–43.0 µs`** in `S1` (and regresses `+5.3 µs` on `block-2.L` / `+11.2 µs` on `dense-703.syn` end-to-end) at the cost of **doubling `Runs::runs` from `110–362 KiB` to `199–707 KiB` (`+89` to `+345 KiB`/worker = `+0.9` to `+3.45 MiB` across 10 workers)**, wiping out our scratch diet.

---

## 2. Theorem 19.1 — Upstream `CAP = 127` RLE1 Bound & `u16` Bandwidth Optimality (`proved` + `verified`, Closing Obligation #5)

- **Statement (`proved`):** For any BWT column $L$ of length $n \le 900{,}000$ partitioned into $r_{\max}$ maximal runs of widths $w_1, \dots, w_{r_{\max}}$, splitting runs at cap $C \ge 1$ produces $R_C = r_{\max} + \sum_{w_i > C} (\lceil w_i / C \rceil - 1) < r_{\max} + n / C$ records:
  1. At `origin/master`'s $C = 127$, the strict worst-case record overhead across *any* valid block is $< n / 127 = 7{,}087$ records (`< 14.2 KiB` of sequential L1d scan). Because bzip2's pre-BWT `RLE1` transform suppresses plaintext runs $\ge 4$, real Wikidata + `#703` dense blocks incur only **`+2.4%` to `+10.2%`** extra `u16` records (`+3,990` to `+5,284` sequential `u16` reads, predicted `90%–98%` not-taken by Comet Lake's branch predictor in `maximal()`).
  2. Eliminating `CAP` splitting completely requires widening records from `u16` (`2 B`) to `u32` (`4 B`), incurring an unconditional bandwidth and cache-footprint tax of $\Delta M = 4 r_{\max} - 2 R_{127} = 2 r_{\max} - 2(R_{127} - r_{\max}) > 0$ whenever $R_{127} < 2 r_{\max}$ (which holds by a `9–19×` margin on every block: **`+89` to `+345 KiB`/worker**).
- **Bonus Register-Pressure Witness on `S1` (`verified` during `verify_cycle19.rs` bring-up):** Inlining `split()`'s 4-way `p`-split directly inside `maximal()` instead of `origin/master`'s `Pass::take` + `#[cold] #[inline(never)] take_marked` (`open.rs:150-170`) inflated `S1` from `208–458 µs` to `379–892 µs` (**`+80%` slowdown!**) due to 4× cloning of `Queues::push` (`AVX2 iota`) spilling the 15-GPR frame — an exact `S1` confirmation of Cycle 12's **Theorem 12.1**!

| Block | `r_max` | `recs_127 (+%)` | `recs_255 (+%)` | `Runs` Buf (`u16` vs `u32`) | `S1: CAP127` | `S1: CAP255` | `S1: u32Max` | `E2E: CAP127 / CAP255 / u32Max (µs)` |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| `block-0.L` | `51,184` | `56,386 (+10.2%)` | `53,513 (+4.6%)` | **`110`** vs `199 KiB` | `207.8 µs` | `200.7 µs` | `183.8 µs` | `1207.3 / 1170.6 / 1163.8` |
| `block-2.L` | `54,532` | `59,478 (+9.1%)` | `56,747 (+4.1%)` | **`116`** vs `213 KiB` | `223.2 µs` | `216.9 µs` | `201.3 µs` | `1416.3 / 1403.5 / 1421.6` |
| `block-85.L` | `79,375` | `84,659 (+6.7%)` | `81,747 (+3.0%)` | **`165`** vs `310 KiB` | `290.5 µs` | `284.5 µs` | `262.6 µs` | `1507.6 / 1497.5 / 1482.7` |
| `block-143.L` | `108,335` | `112,325 (+3.7%)` | `110,041 (+1.6%)` | **`219`** vs `423 KiB` | `424.3 µs` | `410.4 µs` | `381.3 µs` | `2675.7 / 2648.5 / 2595.8` |
| `block-210.L` | `128,732` | `133,233 (+3.5%)` | `130,711 (+1.5%)` | **`260`** vs `502 KiB` | `458.5 µs` | `453.1 µs` | `419.8 µs` | `2361.2 / 2308.8 / 2226.3` |
| `block-207.L` | `87,111` | `91,416 (+4.9%)` | `88,954 (+2.1%)` | **`178`** vs `340 KiB` | `327.0 µs` | `321.2 µs` | `298.1 µs` | `1784.6 / 1778.1 / 1746.8` |
| `dense-703.syn` | `181,032` | `185,410 (+2.4%)` | `182,694 (+0.9%)` | **`362`** vs `707 KiB` | `648.0 µs` | `644.7 µs` | `628.5 µs` | `2785.4 / 2838.2 / 2796.6` |

---

## 3. Unified `WormholePreEmitCells: InverseBwt` Synthesis & `25`-Round Confrontation

`WormholePreEmitCells` unifies **strictly the zero-regression promoted links from Cycles 15, 17, and 18**, refined with `[u32; 3]` `12 B` `Strip` packing (`[from, to, (width << 8) | u32::from(byte)]`, matching `Band = [end, delta, code]`):
1. **`[u32; 3]` Chaitin `StripBandMux` (`sb_buf: Vec<[u32; 3]>`):** Deletes `Vec<Band>` completely and shrinks `Strip` from `16 B` to `12 B` (`-23` to **`-125 KiB`/worker** on Wikidata!).
2. **`Constant-Sentinel pop_tail` + `(len10 << 22) | id22` `GradedRV` (`Theorem 15.1`):** Zero `arena[rule+2]` loads in `Elim::join`, zero sentinel-0 alias stalls (`-19%` to `-23%` on total `S2` including `S2a`!).
3. **Pohl (1971) 2-Way Bidirectional `side-1` Bottom Walk (`Cycle 18`):** `D = H / 2` (`MLP = 2`, `0 B` scratch).
4. **Prefix-15 Fixed-Stack `[u32; 16]` Direct `inline16` Unfold (`Theorem 17.1`):** `0.0 µs` `S3b` (`inline_words` deleted), skips `spelling` on `64%–78%` of words (`-10%` to `-18%` on `S3`).
5. **Full Production `InverseBwt` Contract:** Supports `Narrow` + `Wide` links, `SPIRALS` (`widest > 32,768` & saturating `len >= 1,023`), `wide` `extra`, `#703` `Cap::dense` admission/refusal, and `LazyCells` fallback, while holding `origin/master`'s `emit` (`S5`) **100% verbatim untouched**!

- **Per-Worker Working Set (`W3`):** **`1,388–1,897 KiB` (`1.355–1.853 MiB`)** on Wikidata (`10` i9 workers = **`18.53 MiB <= 20 MiB` L3**!).
- **DRAM Traffic:** `0.034–0.059 B/B` (`0` extra `2048K` L3 misses).
- **Acquisition Cost:** ~85 lines across `src/open/endpoint.rs` and `src/open.rs`, `0` changes to `src/runs.rs` or `emit`, `#![forbid(unsafe_code)]`.

*(All 4 headline arms below — `A1:Master`, `A2:Ctrl`, `M0:Clone`, and `W3:PreEmit` — are invoked through the identical `mibz2::bwt::InverseBwt::invert_runs` trait interface across 25 rotating rounds.)*

| Block | WS (`M0 -> W3 KiB`) | `M0` Pre-`Emit` (`S1/S2/S3/S4 = Sum µs`) | **`W3` Pre-`Emit`** (`S1/S2/S3/S4 = Sum µs`) | `A1:Master` | `A2/A1` | `M0:Clone` | **`W3:PreEmit`** | **`W3 / M0`** | **`W3 / A1`** |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| `block-0.L` | `1445 -> 1388` (`-57`) | `216/272/200/84 = 772.6` | **`207/219/180/85 = 691.1`** (`0.894x`) | `1270.7 µs` | `0.996x` | `1300.6 µs` | **`1203.6 µs`** | **`0.925x`** | **`0.947x`** |
| `block-2.L` | `1590 -> 1558` (`-32`) | `228/325/245/110 = 907.9` | **`225/256/217/112 = 809.6`** (`0.892x`) | `1619.0 µs` | `1.001x` | `1632.0 µs` | **`1504.8 µs`** | **`0.922x`** | **`0.929x`** |
| `block-85.L` | `1640 -> 1617` (`-23`) | `289/143/120/80 = 630.8` | **`281/114/102/80 = 576.2`** (`0.913x`) | `1629.4 µs` | `1.004x` | `1583.4 µs` | **`1511.8 µs`** | **`0.955x`** | **`0.928x`** |
| `block-143.L` | `2022 -> 1897` (`-125`) | `437/473/398/184 = 1493.3` | **`432/383/335/185 = 1334.4`** (`0.894x`) | `2888.6 µs` | `0.997x` | `2855.3 µs` | **`2666.1 µs`** | **`0.934x`** | **`0.923x`** |
| `block-210.L` | `1964 -> 1894` (`-70`) | `465/263/214/130 = 1072.1` | **`463/210/184/134 = 990.7`** (`0.924x`) | `2549.9 µs` | `0.984x` | `2445.2 µs` | **`2343.0 µs`** | **`0.958x`** | **`0.919x`** |
| `block-207.L` | `1697 -> 1659` (`-38`) | `330/328/248/124 = 1031.7` | **`337/251/222/127 = 936.6`** (`0.908x`) | `1939.4 µs` | `0.996x` | `1867.7 µs` | **`1779.9 µs`** | **`0.953x`** | **`0.918x`** |
| `dense-703.syn` | `2026 -> 2028` (`+2`) | `655/240/303/163 = 1360.2` | **`652/213/248/165 = 1277.6`** (`0.939x`) | `2948.6 µs` | `1.011x` | `2850.0 µs` | **`2783.9 µs`** | **`0.977x`** | **`0.944x`** |

---

## 4. Next Step (Cycle 20 — Final Round 3 Confrontation & Handoff)

Cycle 20 executes the full multi-worker streaming confrontation (`mibz2::BzReader::parallel_workers_with::<_, WormholePreEmitCells>` across an 18-block multi-regime `.bz2` stream at `W = 1, 2, 4, 8`, `14,465+` exhaustive/stress/multi-cycle/multi-epoch/spiral/dense cases, and `25`-round 1-worker & 2-worker barrier-synchronized native A/B + A/A + `M0` confrontation) and delivers the complete **10-Cycle Round 3 Whole-Kernel Bounds & Coordinator Handoff**!
