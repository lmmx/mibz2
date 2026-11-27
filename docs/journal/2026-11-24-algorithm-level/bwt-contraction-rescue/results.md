# Results: Contracted Return-Map Constructor Rescue (`bwt-contraction-constructor-rescue`)

*Completed on 27 November 2026 under claim [`bwt-contraction-constructor-rescue`](../../../../frontier/claims/bwt-contraction-constructor-rescue.toml), following frozen [`protocol.md`](protocol.md) and owner rescue direction ([`2026-11-27-contraction-rescue.md`](../../../2026-11-27-contraction-rescue.md)). Preserves `#1013` ([`bwt-digram-return-map`](../bwt-digram-return-map/results.md)) STOP verdict intact. Zero `src/` edits, `#![forbid(unsafe_code)]`, zero workstation spending.*

---

## Verdict (strict C2): **STOP — Scoped Obstruction (Feasible Budget Exhausted for Post-A6c `F`-Bucket Contraction)**

Cold code dissection of `#1013` identified and eliminated real contingent waste (`72.4%` / `27.3 MiB` redundant `c0 == LONG` `spelling` copies, `u128` bit-shift assembly, 1024-slot `dm_cache`, unguarded `runs.bz2`). The best redesign (**`r1_clean1_2part`**) cuts contraction cost **`-9.0%` baseline / `-13.9%` native** (`108.69 -> 98.86 ms` base, `111.44 -> 95.94 ms` native), recovers **`19%` base / `38%` native** of `#1013`'s deficit vs `ctrl_fwd_1lane`, matches `ctrl_shipped_bidi`, and eliminates the `runs.bz2` regression (`1.015x` base / `1.018x` native vs `1.586x` / `1.375x` in `#1013`).

However, **neither pre-registered redesign comes close to the frozen `+39.18 ms` 1-worker gate**, and the full cost ledger demonstrates an **exhausted feasible budget / hardware obstruction** for post-`A6c` `F`-bucket contraction over Open's `edges[0..m]` graph:
- **Held-out Wikidata (116 blocks, 7 interleaved repeats):**
  - `r1_clean1_2part` loses **`1.142x` (`-135.6 ms` i9) baseline / `1.079x` (`-78.9 ms` i9) native** vs `ctrl_fwd_1lane`, **`1.141x` / `1.083x`** vs paid `ctrl_fwd_4lane_onl`, and **`1.299x` (`-246.3 ms` i9) / `1.261x` (`-208.0 ms` i9)** vs `ctrl_fwd_4lane_ora` (all non-overlapping 7-repeat spreads).
  - `r2_mlp4_buf_bump` (4-lane full-chain L1d-buffer MLP chase + zero-hash bump dict) loses **`1.177x` base / `1.140x` native** vs `ctrl_fwd_1lane` and **`1.346x` / `1.317x`** vs `ctrl_fwd_4lane_ora`, directly proving why combining 4-lane MLP with byte collection and commit cannot approach the bare `pure_chase4` floor.
- **Guard `text.bz2` (forced, 24 blocks):** Still regresses **`1.327x` base / `1.257x` native** (`r1`) and **`1.497x` / `1.431x`** (`r2`) vs `ctrl_fwd_1lane` (`1.972x / 1.857x` vs `ctrl_fwd_4lane_ora`).
- **Scoped obstruction:** Fusing elimination into `A6c` (`wire_dual_space`, `wire_forward_fused_elim`) destroys `Reader::wire`'s SIMD loop and costs more than `prep_tag`; post-`A6c` contraction faces a `30.6 ms` bare 4-lane chase floor (`prep_tag 3.4 + pure_chase4 27.2 ms`, zero bytes/dict/writes) that already exceeds the entire `~24–25 ms` VM budget required to beat `ctrl_fwd_4lane_ora` by `+39.18 ms` i9, while real 4-lane byte-collecting chase is slower than clean 1-lane chase (`51.5 ms` floor) due to branch mispredictions and commit stalls across 4 concurrent variable-length chains (`67.7%` of 4-lane iterations have a terminating lane).

---

## 1. Exactness Verification (100% Byte-Identical on Both Builds)

Every arm (all 5 controls/references + `r1_clean1_2part` + `r2_mlp4_buf_bump` + forced-small `min_m = 4, min_ef = 0.05` variants of R1 and R2 = 9 arms) passed byte-identical verification against `bwt::walk` on both builds (**8,406 / 8,406** total checks, zero mismatches):

| Suite | Blocks | Arms | Baseline x86-64 | `-C target-cpu=native` |
|---|---:|---:|---|---|
| `synthetic_stress_11` (dense/periodic/multicycle/run-heavy/edge) | 11 | 9 | `99 / 99` PASS | `99 / 99` PASS |
| `wikidata_admitted_233` (train 117 + held-out 116) | 233 | 9 | `2,097 / 2,097` PASS | `2,097 / 2,097` PASS |
| `wikidata_forced_nonadm_198` (`Cap::NONE`) | 198 | 9 | `1,782 / 1,782` PASS | `1,782 / 1,782` PASS |
| `guard_runs_admitted` (`ab gen`, `runs.bz2`) | 1 | 9 | `9 / 9` PASS | `9 / 9` PASS |
| `guard_text_forced` (`ab gen`, `text.bz2`) | 24 | 9 | `216 / 216` PASS | `216 / 216` PASS |
| **Total** | **467** | **9** | **4,203 / 4,203 PASS** | **4,203 / 4,203 PASS** |

---

## 2. Cold Dissection & Exploratory Train-Set Ledger (`train[0..117]`, `105,298,817` Rows)

### 2.1 Contingent vs algorithmic costs in `#1013` (`fb_elim_75_L16`)

From `protocol.md` §2 (measured strictly on `train[0..117]` before any held-out run):
- **Graph split:** $m = 13.04\text{M}$ (`435.4 KiB/blk`), $m_K = 3.21\text{M}$ (`24.6%`, `107.2 KiB/blk`), $m_E = 9.83\text{M}$ (`75.4%`). Outgoing from $K$: `56.4%` $K \to K$ direct edges (`1.81M`), `43.6%` $K \to E^+ \to K$ chains (`1.40M`, mean `7.02` hops).
- **Chain length skew:** `84.8%` of chains are $\le 8$ hops (~31% of $E$ hops); the `15.2%` of chains $>8$ hops (max ~219) carry **~69% of all $E$ hops**. `96.89%` of $E$ hops are 1-byte literals.
- **Spelling redundancy:** `#1013` wrote `37.6 MiB` (`370.0 KiB/blk`) to `spelling`, of which **`27.3 MiB` (`72.4%`)** was redundant copying of already-stored `c0 == LONG` prefix words (`16.9%` of chains).
- **Uncontended train sub-phases** (`ctrl_fwd_1lane = 249.82 ms`, `ctrl_fwd_4lane_ora ~ 219.5 ms`):
  - `prep_score`: `3.39 ms` (fusable into `A6b` `induced_with` -> `~0.5 ms`).
  - `prep_tag`: `3.44 ms` (unavoidable without slowing SIMD `wire_forward`).
  - `pure_chase1`: `51.51 ms`; `pure_chase4`: `27.16 ms` (zero byte extraction/dict/writes).
  - Non-chase assembly/dict/spelling/writes in `#1013`: `44.92 ms` (above `chase1`), `69.27 ms` (above `chase4`). Total `#1013` `contr = 103.26 ms` (`A7 = 42.03 ms`).

### 2.2 All exploratory train-set attempts retained

| Exploratory mechanism (train `0..117`) | Measured effect vs `#1013` / `fwd` | Outcome & reason |
|---|---|---|
| Dual-space `A6c` (`wire_dual_space`, monotonic `v` translation) | `A6c = 32.52 ms` vs `wire_fwd 16.38 + prep_tag 3.44 = 19.82 ms` (`2.20x` `fwd_A6c`); `chase4_E` only `-3.5 ms` | **Rejected:** per-band `u_seg`/`v_seg` splits break `Reader::wire` SIMD `pmovzxwd + por` loop |
| Fused `A6c` bucket tagging (`wire_forward_fused_elim`) | `A6c = 20.32 ms` vs `16.66 ms` (`+3.66 ms` vs `3.44 ms` `prep_tag` saved) | **Rejected:** splitting bands at `r_first` inside `A6c` costs more than sequential `tag_and_extract` |
| Wavefront lockstep chase (`wf8`/`wf16`/`wf32`) | `41–46 ms` vs `pure_chase4 = 27.16 ms` | **Rejected:** straggler convoy drain (`11.9%` chains `>16` hops up to `219`) |
| 4-lane 6-state machine / 2-pass `hop_pool` / 2-lane `u64` 8B-evict | 6-state `184 vs 111 ms`; `hop_pool` `357 vs 295 ms`; 2-lane `u64` `contr = 96.45 ms` | **Rejected / Upgraded:** state mispredicts & spills; `u64` 8B cap left ~69% of $E$ hops serial -> upgraded to full-chain 4-lane `r2_mlp4_buf_bump` |
| Co-located `Edge8` (zero dict growth, 8B edges) | Cut `contr` `-21%`, but regressed `A7` `+22..+39%` (`61 vs 44 ms` uncontended) | **Rejected:** 2x wider `edges8` (`214 KiB/blk`) + 4-way tag branch in `A7` |
| Raised `LIMIT = 24 / 32` with fast `w <= 32` `Queues::push` | Cut `A6a+A6b` `-15..-23 ms`, grew $m$ `+17..+27%`, increased `contr + A7` more than saved | **Rejected:** net `C+E` loss |
| **Pre-registered R1 (`r1_clean1_2part`)** | Train `contr = 94.60 ms` vs `#1013 106.89 ms` (`-11.5%`); `C+E = 266.73` vs `bidi 267.26 ms` | **Advanced to held-out** |
| **Pre-registered R2 (`r2_mlp4_buf_bump`)** | Train `contr = 97.93 ms` vs `#1013 106.89 ms` (`-8.4%`); `A7 = 40.94 vs 37.37 ms` | **Advanced to held-out** |

---

## 3. Held-Out Wikidata Results (`admitted[117..233]`, 116 Blocks, `104,398,845` Rows, 7 Interleaved Repeats)

All `i9_CE` projections anchor `ctrl_shipped_bidi` at `1,068.0 ms` via paired per-repeat medians (`I9_CE = 1068.0 * median(arm[r] / bidi[r])`); `dF1`, `dF4o`, `dF4r` are positive when faster than the respective forward control (gate: `>= +39.18 ms`).

### 3.1 Baseline x86-64 (`cargo run --release --example bwt_contraction_rescue`)

| Arm | `A6a` | `A6b` | `A6c` | `contr` | `A7` | `C+E` med (min..max) ms | `vsF1` (min..max) | `vsF4o` (min..max) | `vsF4r` (min..max) | `i9_CE` | `dF1` | `dF4o` | `dF4r` |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| `ctrl_shipped_bidi` | 54.14 | 63.51 | 40.16 | 0.00 | 117.13 | 275.09 (271.28..287.41) | 1.145 (1.123..1.153) | 1.132 (1.099..1.144) | 1.299 (1.274..1.320) | 1068.0 | -135.1 | -124.4 | -245.7 |
| `ctrl_fwd_1lane` | 53.82 | 63.33 | 16.33 | 0.00 | 108.46 | 242.33 (235.66..255.82) | 1.000 (1.000..1.000) | 0.990 (0.978..1.005) | 1.138 (1.110..1.154) | 932.9 | +0.0 | +10.7 | -110.7 |
| `ctrl_fwd_4lane_onl` | 54.37 | 63.22 | 16.29 | 0.00 | 108.66 | 241.66 (237.64..261.58) | 1.010 (0.995..1.023) | 1.000 (1.000..1.000) | 1.150 (1.126..1.165) | 943.6 | -10.7 | +0.0 | -121.3 |
| `ctrl_fwd_4lane_ora` | 54.61 | 63.17 | 16.08 | 0.00 | 79.06 | 212.44 (207.02..224.94) | 0.878 (0.866..0.901) | 0.869 (0.858..0.888) | 1.000 (1.000..1.000) | 822.3 | +110.7 | +121.3 | +0.0 |
| `ref1013_fb75_L16` | 53.93 | 65.51 | 16.15 | 108.69 | 38.31 | 283.84 (276.47..299.15) | 1.171 (1.164..1.197) | 1.163 (1.144..1.177) | 1.336 (1.313..1.354) | 1100.7 | -167.7 | -157.0 | -278.4 |
| **`r1_clean1_2part`** | 53.41 | 65.84 | 16.26 | **98.86** | 41.61 | **275.18** (271.03..292.14) | **1.142** (1.132..1.158) | **1.141** (1.117..1.144) | **1.299** (1.285..1.329) | 1068.5 | **-135.6** | **-124.9** | **-246.3** |
| **`r2_mlp4_buf_bump`** | 53.83 | 66.10 | 16.36 | **104.62** | 44.01 | **285.33** (279.40..302.67) | **1.177** (1.177..1.198) | **1.166** (1.157..1.183) | **1.346** (1.330..1.359) | 1107.7 | **-174.8** | **-164.1** | **-285.5** |

### 3.2 Native (`RUSTFLAGS="-C target-cpu=native"`)

| Arm | `A6a` | `A6b` | `A6c` | `contr` | `A7` | `C+E` med (min..max) ms | `vsF1` (min..max) | `vsF4o` (min..max) | `vsF4r` (min..max) | `i9_CE` | `dF1` | `dF4o` | `dF4r` |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| `ctrl_shipped_bidi` | 71.13 | 66.15 | 41.54 | 0.00 | 115.14 | 296.67 (289.56..304.16) | 1.069 (1.047..1.084) | 1.068 (1.058..1.075) | 1.228 (1.218..1.250) | 1068.0 | -68.9 | -67.6 | -198.0 |
| `ctrl_fwd_1lane` | 73.11 | 65.39 | 17.22 | 0.00 | 121.42 | 278.04 (267.06..287.09) | 1.000 (1.000..1.000) | 1.006 (0.989..1.019) | 1.163 (1.131..1.177) | 999.1 | +0.0 | +1.4 | -129.0 |
| `ctrl_fwd_4lane_onl` | 71.01 | 65.36 | 17.45 | 0.00 | 122.52 | 277.09 (269.44..287.52) | 0.994 (0.982..1.011) | 1.000 (1.000..1.000) | 1.159 (1.139..1.170) | 1000.4 | -1.4 | +0.0 | -130.4 |
| `ctrl_fwd_4lane_ora` | 71.15 | 65.52 | 17.10 | 0.00 | 83.87 | 237.50 (233.53..248.00) | 0.860 (0.849..0.884) | 0.863 (0.855..0.878) | 1.000 (1.000..1.000) | 870.0 | +129.0 | +130.4 | +0.0 |
| `ref1013_fb75_L16` | 71.48 | 68.55 | 17.12 | 111.44 | 42.83 | 311.94 (303.14..324.88) | 1.127 (1.097..1.143) | 1.128 (1.117..1.134) | 1.310 (1.276..1.317) | 1125.8 | -126.7 | -125.4 | -255.8 |
| **`r1_clean1_2part`** | 72.18 | 68.42 | 17.36 | **95.94** | 44.55 | **299.20** (291.59..313.67) | **1.079** (1.042..1.116) | **1.083** (1.062..1.109) | **1.261** (1.212..1.267) | 1078.0 | **-78.9** | **-77.5** | **-208.0** |
| **`r2_mlp4_buf_bump`** | 71.74 | 68.63 | 16.94 | **111.18** | 47.74 | **315.50** (307.37..326.56) | **1.140** (1.125..1.151) | **1.141** (1.136..1.146) | **1.317** (1.293..1.341) | 1146.4 | **-147.3** | **-146.0** | **-276.4** |

---

## 4. Guard Corpora Results (`ab gen`, 7 Interleaved Repeats)

### 4.1 `runs.bz2` (1 admitted block, `508,269` rows)

| Arm | Baseline `contr` / `C+E` (ms) | Baseline `vsF1` (min..max) | Native `contr` / `C+E` (ms) | Native `vsF1` (min..max) |
|---|---:|---:|---:|---:|
| `ctrl_fwd_1lane` | 0.00 / 1.86 | 1.000 (1.000..1.000) | 0.00 / 2.07 | 1.000 (1.000..1.000) |
| `ctrl_fwd_4lane_onl` | 0.00 / 1.75 | 0.929 (0.846..1.061) | 0.00 / 1.93 | 0.929 (0.763..1.059) |
| `ref1013_fb75_L16` | 1.33 / 2.99 | **1.586** (1.348..1.815) | 1.08 / 2.94 | **1.375** (1.187..1.466) |
| **`r1_clean1_2part`** | **0.00** / 1.90 | **1.015** (0.869..1.164) | **0.00** / 2.10 | **1.018** (0.832..1.085) |
| **`r2_mlp4_buf_bump`** | **0.00** / 1.96 | **1.060** (0.935..1.447) | **0.00** / 2.14 | **1.047** (0.849..1.168) |

The $O(1)$ pre-construction guard (`m >= 32,768`, `elim_frac >= 0.55`) completely eliminates `#1013`'s `runs.bz2` regression at zero contraction cost.

### 4.2 `text.bz2` (24 forced blocks, `21,411,796` rows)

| Arm | Baseline `contr` / `A7` / `C+E` (ms) | Baseline `vsF1` (`vsF4r`) | Native `contr` / `A7` / `C+E` (ms) | Native `vsF1` (`vsF4r`) |
|---|---:|---:|---:|---:|
| `ctrl_fwd_1lane` | 0.00 / 156.39 / 254.04 | 1.000 (1.492) | 0.00 / 163.53 / 273.39 | 1.000 (1.478) |
| `ctrl_fwd_4lane_onl` | 0.00 / 106.75 / 205.63 | 0.810 (1.200) | 0.00 / 113.88 / 224.38 | 0.819 (1.222) |
| `ctrl_fwd_4lane_ora` | 0.00 / 73.36 / 171.48 | 0.670 (1.000) | 0.00 / 72.72 / 184.12 | 0.677 (1.000) |
| `ref1013_fb75_L16` | 187.79 / 81.80 / 368.71 | 1.448 (2.144) | 184.28 / 79.72 / 373.02 | 1.384 (2.052) |
| **`r1_clean1_2part`** | **159.96** / 77.10 / **336.84** | **1.327** (1.972) | **153.77** / 76.56 / **341.88** | **1.257** (1.857) |
| **`r2_mlp4_buf_bump`** | **186.39** / 92.96 / **380.69** | **1.497** (2.225) | **187.18** / 97.09 / **400.21** | **1.431** (2.142) |

On forced English text (`text.bz2`), ASCII letter buckets satisfy both `m >= 32,768` and `elim_frac >= 0.55`, so contraction triggers and pays `~2x` more in `contr` (`154–160 ms`) than it saves in `A7` (`79–87 ms`).

---

## 5. Diagnosis, Selection Exposure, and Shelving Decision

1. **Why no further micro-optimization of post-`A6c` `F`-bucket contraction is warranted:**
   - **Pre-`A6c` fusion fails** because `wire_forward` (`16.3 ms`) is a pure SIMD `pmovzxwd + por` pass over `OutQ16`; any per-bucket rank split (`wire_dual_space`, `wire_forward_fused_elim`) destroys SIMD vectorization and adds `+3.7..+16.1 ms`, exceeding `prep_tag` (`3.4 ms`).
   - **Post-`A6c` 4-lane MLP cannot hide byte-collecting contraction** (`r2_mlp4_buf_bump` is slower than 1-lane `r1_clean1_2part` on both builds: `104.6` vs `98.9 ms` base, `111.2` vs `95.9 ms` native) because `56.4%` $K \to K$ edges plus variable $K \to E$ chain lengths (`1..219`) cause a lane termination on `67.7%` of 4-lane iterations, triggering branch mispredictions and commit stalls that flush OoO MLP. Removing `dm_cache` deduplication simultaneously inflates `inline16` (`~107 -> ~200 KiB/blk`), costing `+2.4..+3.2 ms` in `A7`.
   - **Budget exhaustion is structural:** Even if all non-chase assembly/dict/write work were magically zero, `A6a + A6b + A6c + prep_tag + pure_chase4 + A7(r1)` (`~208 ms` base / `~233 ms` native) cannot clear `+39.18 ms` i9 (`~10 ms` VM) over `ctrl_fwd_4lane_ora` (`212.4 ms` base / `237.5 ms` native). Against `ctrl_fwd_1lane`, `r1_clean1_2part` would still need an additional `-43 ms` base / `-32 ms` native (`contr <= 55–64 ms`, at or below bare `prep_tag + pure_chase1 = 55 ms` with zero byte assembly).
2. **Selection exposure disclosure:** All design and tuning occurred solely on `train[0..117]`; `held-out[117..233]` was evaluated once under frozen `protocol.md`. Because `#1013` previously measured `admitted[117..233]` (informing the rescue brief), `admitted[117..233]` carries prior mechanism-level exposure; had any arm passed, confirmatory validation would have required an independently registered fresh corpus. Because both redesigns decisively fail C2 on held-out and guard corpora, no confirmatory run is warranted.
3. **Scoped shelving decision:** Close `bwt-contraction-constructor-rescue` as **STOP (scoped obstruction)** and shelve post-`A6c` `F`-bucket contraction over Open's `edges[0..m]` graph. This does **not** close `forward-emission`, `mtf-fold-build`, or non-edge-chasing mathematical operators under the separate P1 claim `computational-permutation-structure`.

---

## 6. Reproducibility Commands

```bash
# Baseline x86-64 exactness + 7-repeat held-out & guard benchmark
cargo run --release --example bwt_contraction_rescue -- exact corpora/wikidata.bz2 --runs /path/to/runs.bz2 --text /path/to/text.bz2
cargo run --release --example bwt_contraction_rescue -- bench corpora/wikidata.bz2 --repeats 7 --runs /path/to/runs.bz2 --text /path/to/text.bz2

# Native (-C target-cpu=native)
RUSTFLAGS="-C target-cpu=native" cargo run --release --example bwt_contraction_rescue -- exact corpora/wikidata.bz2 --runs /path/to/runs.bz2 --text /path/to/text.bz2
RUSTFLAGS="-C target-cpu=native" cargo run --release --example bwt_contraction_rescue -- bench corpora/wikidata.bz2 --repeats 7 --runs /path/to/runs.bz2 --text /path/to/text.bz2
```
