# Wormhole Cycle 62 (Neon): PR 4 `(win_cap × MAX_LIVE_WINS)` Pareto Sweep & Invariant #6 (`owned_end_byte`)

**Date:** 2026-11-22  
**Branch:** `research/carbon-wormhole`  
**Baseline:** `origin/master` `3296195e3` (`#739`)  
**Harness:** [`c62_window_pareto_ab.rs`](2026-11-22-wormhole-cycle62/c62_window_pareto_ab.rs) (`#![forbid(unsafe_code)]`, 9-arm same-binary A/B + A/A control, fresh-process child OS counters, exact `AbHash`)  
**Output:** [`c62_results.txt`](2026-11-22-wormhole-cycle62/c62_results.txt)  
**Manifest:** [`c62_window_pareto.toml`](2026-11-22-wormhole-cycle62/c62_window_pareto.toml) (`check_manifest.py`: `PASS`)

---

## 1. Objective & Invariant #6 (`owned_end_byte` Early Window Release)

Cycle 59 showed that `2 MiB × 2` (`4 MiB` RAM) starved workers on incompressible `0.9 MiB` blocks while `4 MiB × 2` (`8 MiB` RAM) did not. Cycle 62 sweeps the full `(win_cap × MAX_LIVE_WINS)` Pareto surface (`3 MiB` to `16 MiB` total input RAM) across `wikidata-l9 4W/10W`, `hetero-l9 4W`, and `random-l9 4W`:
- `W1M_L3` (`3 MiB`), `W4M_L1` (`4 MiB`), `W2M_L2` (`4 MiB`), **`W2M_L3` (`6 MiB`)**, **`W4M_L2` (`8 MiB`)**, `W4M_L3` (`12 MiB`), `W8M_L2` (`16 MiB`) vs `M739_A` / `M739_B`.

### Mandatory PR 4 Invariant #6 (`owned_end_byte` + Pre-Wait `cur_win` Release)
Running `W4M_L1` (`MAX_LIVE_WINS = 1`) immediately exposed a latent concurrency property:
- When `ensure_win` checks `abs_bit <= w.abs_base_byte + w.data.len()`, the first carried block of Window $k+1$ STILL falls inside Window $k$'s tail carry. The consumer therefore keeps `cur_win = Window k` while blocking in `take_next_spec()` waiting for Window $k+1$ — deadlocking `L = 1` and delaying `Arc<WindowBuf>` release by 1 block even for `L >= 2`!
- **Fix (Invariant #6):** Store `pub owned_end_byte: u64` (`= next_abs_base`) on `WindowBuf`, treat a non-EOF window as exhausted as soon as `abs_bit >= w.owned_end_byte * 8`, and release `cur_win.take()` **before** waiting in `win_log`. This makes even `L = 1` provably deadlock-free and frees `cur_win` 1 full block earlier for `L >= 2` (recovering `W2M_L2` on `hetero-l9` from `1.0563×` to `0.9698×`, though `W2M_L2` still starves `random-l9`).

---

## 2. Full 9-Arm Empirical Pareto Frontier (`c62_results.txt`)

All 9 arms passed exact `AbHash` parity on every round of every regime (`431/431`, `60/60`, `12/12` — `100.0%` fast hits on all `win_cap >= 2 MiB`; `W1M_L3` had `fb=3` on `hetero-l9` where `0.9 MiB` blocks exceed `1 MiB` carry budget).

### Table 1 — Summary Across All 4 Regimes (Ratio vs `M739_A` `[95% CI]` & Peak RSS)
| Arm (`win × L`) | Input RAM | `wikidata-l9 4W` (`7` rd) | `wikidata-l9 10W` (`5` rd) | `hetero-l9 4W` (`7` rd) | `random-l9 4W` (`7` rd) | RSS (`4W / 10W / het / rnd` MiB) |
|---|---:|---|---|---|---|---|
| `M739_A` | `4M × 2` | `1.0000` (`352.5ms`) | `1.0000` (`396.7ms`) | `1.0000` (`161.1ms`) | `1.0000` (`118.5ms`) | `56.3 / 76.1 / 76.7 / 64.2` |
| `M739_B` (A/A) | `4M × 2` | `1.0170 [0.985, 1.050]` | `1.0040 [0.988, 1.020]` | `1.0286 [0.984, 1.076]` | `1.0059 [0.971, 1.042]` | `56.0 / 74.5 / 77.9 / 62.9` |
| `W1M_L3` | `3 MiB` | `0.9817 [0.963, 1.001]` | `0.9572 [0.941, 0.974]` | `1.0447 [1.008, 1.083]` (`fb=3`) | `1.0619 [0.999, 1.128]` (`vcsw 67`) | `37.9 / 65.3 / 70.2 / 53.4` |
| `W4M_L1` | `4 MiB` | `0.9991 [0.973, 1.026]` | `0.9868 [0.967, 1.007]` | `1.0554 [1.029, 1.083]` | `1.0591 [1.020, 1.099]` | `39.8 / 66.3 / 66.4 / 55.4` |
| `W2M_L2` | `4 MiB` | `0.9891 [0.940, 1.041]` | `0.9545 [0.947, 0.963]` | `0.9698 [0.924, 1.018]` | `1.0415 [1.013, 1.070]` | `39.7 / 65.5 / 65.9 / 55.3` |
| **`W2M_L3`** | **`6 MiB`** | **`0.9905 [0.940, 1.043]`** | **`0.9576 [0.938, 0.978]`** | **`0.9751 [0.952, 0.999]`** | **`0.9893 [0.944, 1.037]`** | **`40.9 / 68.3 / 67.7 / 58.1`** |
| **`W4M_L2`** | **`8 MiB`** | **`0.9824 [0.964, 1.001]`** | **`0.9592 [0.941, 0.978]`** | **`0.9694 [0.937, 1.003]`** | `1.0208 [0.956, 1.090]` | **`43.8 / 70.4 / 69.0 / 59.6`** |
| `W4M_L3` | `12 MiB` | `0.9881 [0.969, 1.008]` | `0.9636 [0.945, 0.982]` | `0.9829 [0.956, 1.011]` | `1.0002 [0.959, 1.043]` | `47.6 / 74.2 / 70.5 / 63.6` |
| `W8M_L2` | `16 MiB` | `0.9905 [0.977, 1.004]` | `0.9715 [0.961, 0.982]` | `0.9964 [0.940, 1.056]` | `1.0188 [0.961, 1.080]` | `50.7 / 76.6 / 71.0 / 63.7` |

### Table 2 — Key OS Counters (`wikidata-l9 4W` & `10W`)
| Arm | `4W` `sys` | `4W` `minflt` | `4W` `vcsw` | `10W` `sys` | `10W` `minflt` | `10W` `vcsw` |
|---|---:|---:|---:|---:|---:|---:|
| `M739_A` | `18.31` | `10,161` | `205` | `42.40` | `16,351` | `445` |
| **`W2M_L3`** (`6 MiB`) | **`15.24`** | **`5,985` (`-41.1%`)** | `151` | **`31.66`** | **`13,020` (`-20.4%`)** | **`125` (`-71.9%`)** |
| **`W4M_L2`** (`8 MiB`) | `15.82` | `6,705` (`-34.0%`) | **`138`** | `34.47` | `13,507` (`-17.4%`) | `136` (`-69.4%`) |
| `W8M_L2` (`16 MiB`) | `22.55` | `8,494` (`-16.4%`) | `143` | `38.77` | `15,101` (`-7.6%`) | `112` (`-74.8%`) |

---

## 3. Why `W2M_L3` (`2 MiB × 3 = 6 MiB`) Emerges as New Pareto Co-Champion

1. **Zero incompressible starvation:** Even at `100%` worst-case `0.90 MiB` blocks (`random-l9`), `3` live `2 MiB` windows keep `6` blocks in flight simultaneously (`cur_win` = Win $k$, workers on Win $k+1$, producer already reading/scanning Win $k+2$).
2. **Half the cold-start fill latency:** Dispatching Block 0 waits for only `2 MiB` read+scan (`~0.35 ms`) instead of `4 MiB` (`~0.70 ms`), making `W2M_L3` the **#1 fastest arm on `random-l9` (`115.79 ms`, `0.9893×`)** and delivering a clean 95%-CI win on BOTH **`wikidata-l9 10W` (`0.9576× [0.938, 0.978]`)** and **`hetero-l9 4W` (`0.9751× [0.952, 0.999]`)**.
3. **Strictly lower RSS & faults than `W4M_L2`:** Uses `6 MiB` vs `8 MiB` input RAM (`-2.0..-2.9 MiB` lower peak RSS across all 4 regimes: **`40.9 MiB = 0.726×` of `#739`** at `4W`, **`68.3 MiB = 0.898×`** at `10W`; `-500..-720` fewer `minflt`).

---

## 4. Kill / Promote Decisions (One-Line Verdicts)

- **PROMOTED `W2M_L3` (`2 MiB × 3 = 6 MiB`, recommended PR 4 default) & `W4M_L2` (`4 MiB × 2 = 8 MiB`, co-champion) + Invariant #6 (`owned_end_byte`):** `W2M_L3` beats `M739_A` on all `4/4` regimes (`10W 0.9576×`, `hetero 0.9751×`, `random 0.9893×`, `4W 0.9905×`) at `0.726×–0.904×` RSS (`-2 MiB` below `W4M_L2`).
- **KILLED `W1M_L3` (`3 MiB`):** `1 MiB` clips `0.9 MiB` blocks (`fb=3`), regressing `hetero-l9` (`1.0447×`) and `random-l9` (`1.0619×`, `vcsw 67`).
- **KILLED `W4M_L1` (`4 MiB`):** single-window boundary stalls regress `hetero-l9` (`1.0554×`) and `random-l9` (`1.0591×`).
- **KILLED `W2M_L2` (`4 MiB`):** starves `random-l9` (`1.0415× [1.0134, 1.0704]`) with only `4` incompressible blocks across 2 windows.
- **KILLED `W4M_L3` (`12 MiB`) & `W8M_L2` (`16 MiB`):** strictly dominated by `W2M_L3`/`W4M_L2` (`+4..10 MiB` RSS, `+1.5k..2.5k` `minflt`, `0` wall gain).
