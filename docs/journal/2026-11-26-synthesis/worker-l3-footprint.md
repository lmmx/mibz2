# Worker L3 footprint census and 8–10-worker L3 cost attribution (`worker-l3-footprint`)

*Smooth-zone probe (`frontier/claims/worker-l3-footprint.toml`, 26 November 2026). Reproducible zero-`unsafe` census in `examples/worker_l3_footprint_census.rs`; zero `src/` edits; no workstation run. Fits use only existing evidence bundles on `origin/evidence`: pooled-scan `c71c4a56c145` (tarball SHA-256 `97e397ad586a6700c04a874e50dbc5e26ee9fc91912b115111cbe0550ddc8867`) and zero-copy-floor `cc8ba1374f32` (tarball SHA-256 `10df0d895755494ec6f06369071d649674d855ab953201a2b2888845ab265e34`). Cold-store oracle arms (`C`, `PC`, `SC`, `SCMRD`, `SECMD`, `SECMRD`, `SFCMR`) are excluded from every fit.*

---

## Frozen `stop_test` verdict (strict C2)

**Verdict: `(a)` passes `R² >= 0.5` — leave `delivery-l3-working-set` as is (`status = "idea"`), no `dead-ends.tsv` row.**

On `c71c4a56` (`none` and `S` combined, `k = 6..10`, steady-phase `Δ(A9 + copy_out)` over each arm's `k = 6` median):
- **(a) `ready_bytes` high-water mark:** **`R² = 0.9320`** (cell medians, `n = 10`; `0.8722` across all `n = 30` traced runs; `0.9553` `none` alone; `0.9245` `S` alone; `0.9802` on `cc8ba137` pinned `none`).
- **(b) `ready_bytes` steady time-average:** **`R² = 0.9680`** (`n = 10`; `0.9384` across `n = 30` runs).
- **(c) `k · W_worker + ready_avg`:** **`R² = 0.9767`** at measured time-weighted active `W_worker = 0.845 MiB`, **`0.9750`** at resident `W_worker = 0.897 MiB`, **`0.9619`** at A7 peak `W_worker = 1.352 MiB` (`n = 10`; `0.9713` / `0.9691` / `0.9527` across `n = 30` runs).

Because `(a)` achieves `R² = 0.9320 >= 0.5` (and `(c) − (a) = +0.045 < 0.15`), the frozen `stop_test` keeps `delivery-l3-working-set` intact rather than dropping it.

**Physical caveat revealed by sub-regime and within-cell diagnostics (§3):** `(a)`, `(b)`, and `(c)` are strongly collinear across `k = 6..10` (`r(k, ready_hwm) = 0.858`, `r(k, ready_avg) = 0.841`), so full-range linear `R²` cannot isolate `READY_BYTES` from `k · W_worker`. Two natural sub-experiments separate them:
1. **Zero-SMT `k = 6..8` on pinned `cc8ba137` (`none`, `n = 12` runs):** `ready_hwm` barely moves (`5.15 → 5.15 → 6.01 MiB`) and `ready_avg` is `0.31 → 0.59 MiB`, yet `IMC` DRAM traffic jumps `+672 MiB` (`930 → 1,602 MiB`), `LLC` misses jump `+2.98M`, and `A9` rises `+13.5 ms`. Here **(a) collapses to `R² = 0.344` (`IMC`: `0.407`) while (c) holds `R² = 0.947` (`A9`: `0.981`, `IMC`: `0.991`)**, proving the **per-worker live set** (`5.4 → 7.2 MiB` avg / `8.1 → 10.8 MiB` peak) drives the initial `k = 6..8` L3 spillover. Across `k = 6..10`, `(c)` predicts `cc8ba137` `ΔIMC` with **`R² = 0.9988`** (`n = 20`: `0.9959`).
2. **Fixed-`k` within-cell bursts (`c71c4a56` `n = 30`; `cc8ba137` `n = 20`):** Holding `(arm, k)` fixed, within-cell `ready_avg` correlates strongly with `A9 + copy_out` (`r = +0.955` on `c71c4a56`, `+0.826` on pinned `cc8ba137`), `A7` (`r = +0.896 / +0.909`), `LLC` (`r = +0.926`), and `IMC` (`r = +0.871`), while `ready_hwm` is weak within-cell (`R² = 0.072–0.165`). At `k = 9..10`, SMT/scheduling jitter and ready-queue surges couple into a positive feedback loop; observational traces cannot prove how much of that `k = 9..10` jump a `READY_BYTES` cap can break (and `third-pass.md` §1's wall-budget ceiling still caps `ΔA9/k` at `≤ 0.9%` at 8W, `≤ 2.2%` at 10W in `none`, `≤ 2.5%` in `S`).

---

## 1. Per-block and per-stage worker live-set census (`corpora/wikidata.bz2` and `×4`)

Command:
```bash
cargo run --release --example worker_l3_footprint_census -- corpora/wikidata.bz2
cargo run --release --example worker_l3_footprint_census -- corpora/wikidata.bz2 --x4
```
- **Exactness:** Every block's output (`431` blocks / `209,700,196 B` on `wikidata`; `1,724` blocks / `838,800,784 B` on `wikidata ×4`) matched `mibz2::decode` byte-for-byte.
- **Cross-check against `c71c4a56` (`wikidata ×4`):**
  - Routes: `open = 912`, `open_dense = 20`, `skipped = 600`, `column = 192` — **exact match** to `c71c4a56` `routes`.
  - `symbols` sum (`x4`): `199,834,392 B` — **exact match** to `c71c4a56` `queue_syms_bytes`.
  - `Open Built` sum (`431` distinct): `120,589,466 B` vs `c71c4a56` `oracle_store.C.bytes = 120,613,698 B` (exact `233 × 104 B` struct `Entry` overhead = `24,232 B`, **0.000% data discrepancy**).
  - `MTF` output sum (`431` distinct): `33,438,757 B` vs `oracle_store.M.bytes = 33,508,597 B` (diff = `431 × Entry` + column `counts`).
- **Bimodal corpus structure:** All `233` full blocks (`99.996%` of decoded bytes) run `Open` (`228` standard + `5` dense; `232/233` bidirectional, `0/233` wide edges, `0/431` RLE1 staged). All `198` non-Open blocks (`150` skipped + `48` column) are tiny runts (`max 1.2 KiB` decoded, total `8.9 KiB` = `0.004%` of bytes). Per-block distributions on `wikidata ×4` are identical to `wikidata` (exact 4× repeat).

### 1.1 Per-buffer sizes (real blocks)

| Buffer | Open blocks (`n = 233`, 99.996% B) mean / p50 / p90 / max | All blocks (`n = 431`) mean / max | Lifetime / aliasing in `src/` |
|---|---:|---:|---|
| `out` (decoded len) | `878.9 / 878.9 / 878.9 / 878.9 KiB` | `475.1 / 878.9 KiB` | Popped from `free_out` after A5 (`cap = 933.9 KiB`); published to `Ready` after A9 |
| `stage` (`s.ll`) | **`0.0 / 0.0 / 0.0 / 0.0 KiB`** | `0.0 / 0.0 KiB` | Untouched on run-route (`rle_staged = 0/431`); ≤ 0.05 KiB on column runts |
| `symbols` (`b.syms`) | `209.3 / 188.3 / 297.5 / 390.2 KiB` | `113.2 / 390.2 KiB` | Read in A5, then **returned to `free_syms` in `take()` before `out` is popped** |
| `MTF output` (`runs.runs` + `counts`/`per`) | `138.8 / 121.4 / 204.8 / 363.8 KiB` | `75.8 / 363.8 KiB` | Written in A5, read in A6a, **dead after A6a** |
| `MTF AVX2 bitmaps` (`lit` + `one`) | `26.2 / 23.5 / 37.2 / 48.8 KiB` | `14.2 / 48.8 KiB` | Transient in A5 (`runs::front`) only |
| `OutQ16` (queues) | `279.1 / 254.0 / 389.0 / 516.0 KiB` | `150.9 / 516.0 KiB` | **Aliased inside `out[..n+16]`** (written A6a, dormant A6b, read A6c, overwritten A7) |
| `OutQ16 Spill` (`next`, `ovf`, `cuts`) | `2.3 / 2.0 / 3.5 / 5.0 KiB` | `1.3 / 5.0 KiB` | Separate small scratch (`ovf = 0` on full blocks) |
| `Open A6b arena` (strips + nodes + rules) | `282.1 / 295.3 / 335.1 / 476.6 KiB` | `152.5 / 476.6 KiB` | **Aliased inside `edges`** (with 24 KiB dedup cache); overwritten in A6c |
| `Open A6b bands` | `27.8 / 27.7 / 33.9 / 53.0 KiB` | `15.0 / 53.0 KiB` | Written A6b, read A6c, dead after A6c |
| `Open Built` (`edges[..m]` + `inline16` + `lens` + `spelling`) | `505.4 / 461.8 / 711.3 / 954.2 KiB` | `273.2 / 954.2 KiB` | `edges[..m]` `445.4 KiB` mean + dict `60.0 KiB` (`extra = 0` on all blocks) |
| `RLE1 rle_runs` | `0.4 / 0.2 / 0.9 / 9.6 KiB` | `0.2 / 9.6 KiB` | Written & read in A9 only (`8 B/run`) |
| `Segmented tt` (`4·n`) | `0.0 KiB` | `0.1 / 4.6 KiB` | Runts only (max `4.6 KiB` on skipped blocks) |

### 1.2 Simultaneously live bytes per worker by stage (strictly respecting aliasing & lifetimes)

Because `OutQ16` aliases `out`, the A6b arena + dedup cache alias `edges`, `stage` is unused, and `symbols` is freed before `out` is taken, instantaneous live bytes per worker are substantially smaller than the naive sum of buffers:

| Stage | Buffers live simultaneously | Full-block mean / p50 / p90 / max (KiB) | Byte-weighted mean (MiB) | i9 time share (`none`/`S` @ 8W) |
|---|---|---:|---:|---:|
| **A5** (MTF/RLE2) | `symbols` + `MTF output` + `bitmaps` | `374.3 / 332.1 / 540.7 / 802.1` | `0.366` | `9.9% / 10.5%` |
| **A6a** (Open pass 1) | `runs` + `OutQ16(in out)` + `Spill` + `strips(in edges)` + stack | `462.2 / 421.2 / 641.7 / 824.9` | `0.451` | `15.1% / 15.0%` |
| **A6b** (Endpoint + dict) | active: `arena(in edges)` + `bands` + `dict` + `stack` | `369.9 / 383.8 / 430.6 / 599.6` | `0.361` | `16.5% / 16.4%` |
| **A6b** (+ dormant `OutQ16`) | above + `OutQ16(in out)` + `Spill` waiting for A6c | `651.3 / 643.8 / 770.5 / 1051.3` | `0.636` | — |
| **A6c** (Open wiring) | active: `edges[..m]` + `bands` + `OutQ16(in out)` + `Spill` | `754.6 / 682.0 / 1081.1 / 1461.2` | `0.737` | `12.2% / 12.0%` |
| **A6c** (+ dormant dict) | above + `inline16` + `lens` + `spelling` waiting for A7 | `814.6 / 745.1 / 1124.7 / 1510.8` | `0.795` | — |
| **A7** (Open emit — **peak**) | `Built` (`edges` + `dict`) + `out[..n+16]` | **`1384.3 / 1340.7 / 1590.2 / 1833.2`** | **`1.352`** | **`39.3% / 39.6%`** |
| **A9** (RLE1 + CRC) | `out[..n+shift]` + `rle_runs` | `879.3 / 879.1 / 879.8 / 888.5` | `0.859` | `7.0% / 6.6%` |

- **Time-weighted per-worker live set** (constant to `±0.005 MiB` across `k = 6, 8, 10` in both `none` and `S`):
  - **`W_active = 0.845 MiB/worker`** (`865 KiB`, actively touched bytes weighted by i9 stage time)
  - **`W_resident = 0.897 MiB/worker`** (`919 KiB`, including cross-stage dormant `OutQ16` in A6b and `dict` in A6c)
  - **`W_peak(A7) = 1.352 MiB/worker`** (`1,384 KiB`, peak stage active set per block)
- **Persistent allocated `Scratch` capacity HWM per worker:** `1.492 MiB` (`open.edges` `904.7` + `runs.runs` `361.8` + `open.spell` `96.0` + `open.bands` `64.0` + `runs bitmaps` `48.8` + `open.dict` `29.8` + `rle_runs` `9.6` + `open.spill` `8.0` + `tt` `4.6 KiB`, `ll = 0`), or `4.120 MiB` virtual/RSS including one pooled `out` (`933.9 KiB`) and one HWM pooled `syms` (`1,758.0 KiB`).
- **Scale vs `Ready` on the 20 MiB L3 i9-10900K:** At `k = 6, 8, 10`, `k · W_resident` = **`5.38, 7.18, 8.97 MiB`** (`k · W_peak(A7)` = **`8.11, 10.82, 13.52 MiB`**), against `ready_avg` = **`0.31–2.30 MiB`** (4–17× smaller) and `ready_hwm` = **`5.15–16.31 MiB`**.

---

## 2. Regression fits on existing traces (`c71c4a56` and `cc8ba137`, real-`C`/`E` arms only)

### 2.1 Primary data table (`c71c4a56` `none` & `S`, `cc8ba137` pinned `none`; cell medians, steady phase)

| Bundle / arm | `k` | `A9` (ms) | `copy_out` (ms) | `Δ(A9+CO)` over `k=6` | `(a)` `ready_hwm` (MiB) | `(b)` `ready_avg` steady (MiB) | `(c-res)` `k·0.897 + avg` (MiB) | `cc8` `LLC` (M) | `cc8` `IMC` (MiB) |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| `c71c` `none` | 6 | 88.42 | 27.29 | 0.00 | 5.15 | 0.307 | 5.69 | — | — |
| `c71c` `none` | 7 | 94.23 | 30.16 | +8.29 | 5.15 | 0.416 | 6.70 | — | — |
| `c71c` `none` | 8 | 102.76 | 33.53 | +21.02 | 6.87 | 0.604 | 7.78 | — | — |
| `c71c` `none` | 9 | 125.64 | 42.72 | +53.26 | 12.02 | 1.855 | 9.93 | — | — |
| `c71c` `none` | 10 | 131.28 | 43.06 | +57.78 | 14.59 | 1.918 | 10.89 | — | — |
| `c71c` `S` | 6 | 84.80 | 27.06 | 0.00 | 6.01 | 0.333 | 5.72 | — | — |
| `c71c` `S` | 7 | 87.68 | 29.43 | +6.12 | 6.01 | 0.447 | 6.73 | — | — |
| `c71c` `S` | 8 | 95.40 | 33.47 | +17.01 | 6.01 | 0.641 | 7.82 | — | — |
| `c71c` `S` | 9 | 105.02 | 35.87 | +29.03 | 8.58 | 0.884 | 8.96 | — | — |
| `c71c` `S` | 10 | 127.72 | 47.78 | +63.64 | 16.31 | 2.298 | 11.27 | — | — |
| `cc8` `none` (pin) | 6 | 88.34 | 28.27 | 0.00 | 5.15 | 0.311 | 5.69 | 5.81 | 930.1 |
| `cc8` `none` (pin) | 7 | 94.75 | 28.58 | +6.72 | 5.15 | 0.416 | 6.70 | 7.11 | 1211.3 |
| `cc8` `none` (pin) | 8 | 101.79 | 32.33 | +17.51 | 6.01 | 0.592 | 7.77 | 8.79 | 1602.1 |
| `cc8` `none` (pin) | 9 | 111.48 | 35.51 | +30.38 | 8.58 | 0.769 | 8.84 | 10.47 | 1937.1 |
| `cc8` `none` (pin) | 10 | 138.16 | 45.43 | +66.98 | 13.73 | 2.303 | 11.27 | 17.74 | 2824.2 |

*(Steady-phase `ready_avg` = `occupancy.ready_bytes.integral / (t2_ns - t1_ns)`, matching `third-pass.md` §1; full-run `integral / end_ns` gives `R²` within `0.004` across every fit.)*

### 2.2 OLS `R²` comparison across predictors (a), (b), and (c)

| Dataset & target (`Δ` over `k = 6`) | `(a)` `ready_hwm` | `(b)` `ready_avg` | `(c-act)` `k·0.845+avg` | `(c-res)` `k·0.897+avg` | `(c-pk)` `k·1.352+avg` | `k` alone |
|---|---:|---:|---:|---:|---:|---:|
| **`c71c` `none+S` cell medians (`n=10`) `Δ(A9+CO)`** | **`0.9320`** | `0.9680` | **`0.9767`** | `0.9750` | `0.9619` | `0.9033` |
| `c71c` `none+S` cell medians (`n=10`) `ΔA9` | `0.9304` | `0.9597` | `0.9693` | `0.9676` | `0.9547` | `0.8970` |
| `c71c` `none+S` cell medians (`n=10`) `Δcopy_out` | `0.9203` | `0.9630` | `0.9733` | `0.9716` | `0.9588` | `0.9011` |
| `c71c` `none+S` all-phase medians (`n=10`) `Δ(A9+CO)` | `0.9334` | `0.9667` | `0.9784` | `0.9765` | `0.9660` | `0.9092` |
| **`c71c` `none+S` individual runs (`n=30`) `Δ(A9+CO)`** | **`0.8722`** | `0.9384` | **`0.9713`** | `0.9691` | `0.9527` | `0.8738` |
| `c71c` `none+S` individual runs (`n=30`) `ΔA9` | `0.8628` | `0.9338` | `0.9681` | `0.9660` | `0.9498` | `0.8718` |
| `c71c` `none+S` individual runs (`n=30`) `Δcopy_out` | `0.8658` | `0.9175` | `0.9454` | `0.9432` | `0.9265` | `0.8480` |
| `c71c` `none` only (`n=5` cells / `n=15` runs) `Δ(A9+CO)` | `0.9553 / 0.8888` | `0.9744 / 0.9507` | `0.987 / 0.982` | `0.9863 / 0.9807` | `0.9780 / 0.9698` | `0.9385 / 0.9190` |
| `c71c` `S` only (`n=5` cells / `n=15` runs) `Δ(A9+CO)` | `0.9245 / 0.8687` | `0.9643 / 0.9486` | `0.978 / 0.970` | `0.9762 / 0.9680` | `0.9603 / 0.9464` | `0.8889 / 0.8412` |
| `cc8` pinned `none` (`n=5` cells) `Δ(A9+CO)` | `0.9802` | `0.9510` | `0.9777` | `0.9754` | `0.9578` | `0.8768` |
| `cc8` pinned `none` (`n=5` cells) `ΔLLC` (ld+st) | `0.9733` | `0.9716` | `0.9653` | `0.9622` | `0.9401` | `0.8468` |
| **`cc8` pinned `none` (`n=5` cells) `ΔIMC` (rd+wr)** | `0.9375` | `0.8939` | **`0.9988`** | **`0.9983`** | `0.9922` | `0.9428` |
| `cc8` pinned `none` (`n=20` runs) `Δ(A9+CO)` / `ΔIMC` | `0.8871 / 0.8543` | `0.9542 / 0.9068` | `0.975 / 0.996` | `0.9725 / 0.9959` | `0.9565 / 0.9898` | `0.8822 / 0.9436` |

*(Using `Δready_hwm` over `k=6` instead of raw `ready_hwm` gives `R² = 0.9389` on `n=10` cells and `0.8748` on `n=30` runs; excluding `k=6` gives `R² = 0.9563` on `n=8` cells and `0.8644` on `n=24` runs. Every specification places `(a)` well above `0.5` and `(c) − (a) < 0.15`.)*

---

## 3. Separating the two mechanisms: zero-SMT `k = 6..8` vs fixed-`k` bursts at `k = 9..10`

Because `k`, `ready_hwm`, and `ready_avg` are strongly collinear over `k = 6..10` (`r(k, hwm) = 0.858`, `r(k, avg) = 0.841`, `r(hwm, avg) = 0.924`), full-range univariate `R²` is high for all three predictors. Disaggregating by regime exposes **two distinct contributions**:

1. **Zero-SMT `k = 6..8` (`cc8ba137` pinned `none`, `n = 12` runs, scanner on `[6, 16]`):**
   Workers and reader run on 9 dedicated physical cores with zero SMT sharing. From `k = 6` to `k = 8`, `ready_hwm` barely changes (`5.15 → 5.15 → 6.01 MiB`) and `ready_avg` stays `0.31 → 0.59 MiB`, yet `IMC` DRAM traffic rises **`+672 MiB`** (`930 → 1,602 MiB`), `LLC` misses rise **`+2.98M`** (`5.81M → 8.79M`), and `A9` rises **`+13.5 ms`** (`88.3 → 101.8 ms`):
   - `(a)` `ready_hwm` `R²`: `0.344` (`Δ(A9+CO)`), `0.355` (`A9`), `0.420` (`LLC`), `0.407` (`IMC`).
   - `(c)` `k · 0.897 + ready_avg` `R²`: **`0.947`** (`Δ(A9+CO)`), **`0.981`** (`A9`), **`0.989`** (`LLC`), **`0.991`** (`IMC`).
   - **Conclusion:** Below SMT oversubscription (`k = 6..8`), the per-worker live set (`5.4 → 7.2 MiB` avg, `8.1 → 10.8 MiB` A7 peak — `40–54%` of the 20 MiB L3) is the driver of L3 eviction, not `ready_hwm`.

2. **Multiple OLS & within-cell (`(arm, k)` demeaned) variation at `k = 9..10`:**
   - Multiple OLS on `c71c4a56` (`n = 30`): `Δ(A9 + copy_out) = -46.59 + 6.70·k + 19.39·ready_avg` (**`R² = 0.9876`**; both `k` and `ready_avg` enter strongly, for both `ΔA9` (`+4.91 ms/w`, `+14.00 ms/MiB`, `R² = 0.9838`) and `Δcopy_out` (`+1.78 ms/w`, `+5.39 ms/MiB`, `R² = 0.9629`)).
   - Strictly holding `(arm, k)` constant (demeaning within each cell, so `k · W_worker` has zero variance):
     - `c71c4a56` (`n = 30`): within-cell `ready_avg` tracks `A9 + copy_out` with `r = +0.955` (`R² = 0.912`), `A9` with `r = +0.971` (`R² = 0.944`), `copy_out` with `r = +0.742` (`R² = 0.551`), and `A7` with `r = +0.896` (`R² = 0.803`). Example: `none@9` run `blk=1` (`hwm = 6.87, avg = 0.726`) shows `A9 = 111.9, CO = 35.3, A7 = 583.3 ms` vs `blk=0,2` (`hwm = 12.0–13.7, avg = 1.86–1.88`) at `A9 = 125.6–126.1, CO = 42.7–43.6, A7 = 625.9–628.1 ms`.
     - `cc8ba137` pinned `none` (`n = 20`): within-cell `ready_avg` tracks `A9 + copy_out` (`r = +0.826`, `R² = 0.683`), `A7` (`r = +0.909`, `R² = 0.826`), `LLC` (`r = +0.926`, `R² = 0.858`), and `IMC` (`r = +0.871`, `R² = 0.758`), whereas within-cell `ready_hwm` is weak (`R² = 0.072–0.199`).
   - **Why `A7` moves alongside `A9`, `copy_out`, and `ready_avg`:** At `k = 9..10` (11–12 runnable threads on 10 physical cores in `none`, 11 in `S@10`), OS scheduling collisions on SMT siblings slow the reader or workers, allowing unread blocks to accumulate (`ready_avg` jumps `0.6 → 2–3 MiB`, `ready_hwm` `6 → 16 MiB`), which increases L3 eviction across `A7`, `A9`, and `copy_out` together. Observational traces cannot separate how much of this `k = 9..10` jump is broken by a tighter `READY_BYTES` cap vs caused directly by SMT oversubscription.

---

## 4. Implications for the frontier

1. **`delivery-l3-working-set` stays at `status = "idea"`** per the frozen `stop_test` (`(a)` `R² = 0.932 >= 0.5`). If it is later admitted and tested on the i9, it should log `ready_avg` alongside `ready_hwm` and recognise that `k = 6..8` L3 growth (`+13.5 ms` `A9`, `+672 MiB` DRAM) comes from the per-worker live set (`k · W_worker`), leaving only the `k = 9..10` surge inside `READY_BYTES`'s reach (`third-pass.md` §1 ceiling: `≤ 2.2%` of 10W wall in `none`, `≤ 2.5%` in `S`).
2. **Worker-side footprint levers (`mtf-fold-build`, `forward-emission`, `bwt-digram-return-map`) attack the larger steady-state L3 term:** `A7` alone (`1.352 MiB/worker` = `Built 505 KiB + out 879 KiB`, `39.3%` of worker time) accounts for the peak of every worker's L3 footprint (`10.8 MiB` at 8W, `13.5 MiB` at 10W). Any reduction in `Built` (`bwt-digram-return-map`) or `A5+A6a` (`mtf-fold-build`) shrinks both CPU time and the `k = 6..8` L3 pressure directly.
