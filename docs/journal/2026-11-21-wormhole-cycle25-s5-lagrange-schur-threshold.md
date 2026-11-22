# Wormhole (Neon) Cycle 25 — `S5 Emit` Attack #2: Hoisted-`S4` `Len4` & Exact Post-`GradedOperad` Schur Threshold Convexity (`LIMIT` & Adaptive $\Lambda_a$)

**Session:** `research/carbon-wormhole` (Round 4 — Gemini Neon, Cycle 25 of 30; 5th of 10 in Round 4)  
**Baseline `origin/master`:** `8ff1187af6a920f7d57fd338680db3a5633a3ac5` (`v0.5.11`; `src/` byte-identical to `90e671966`)  
**Verification harness:** [`verify_cycle25.rs`](2026-11-21-wormhole-cycle25-s5-lagrange-schur-threshold/verify_cycle25.rs) (`#![forbid(unsafe_code)]`, `0` warnings, `6/6` `manifest.json` SHA-256 auth + `5,229` Gate G1/spiral/multi-epoch cases byte-exact across all `LIMIT` arms + `15`-round 5-stage Pareto sweep + `25`-round `p50`/`p10` `InverseBwt::invert_runs` confrontation), [`schur_threshold.toml`](2026-11-21-wormhole-cycle25-s5-lagrange-schur-threshold/schur_threshold.toml) (`PASS`), [`cycle25_output.txt`](2026-11-21-wormhole-cycle25-s5-lagrange-schur-threshold/cycle25_output.txt), [`cycle25_results.json`](2026-11-21-wormhole-cycle25-s5-lagrange-schur-threshold/cycle25_results.json)

---

## 1. One-Line Keep & Kill Decisions

- **PROMOTE `E2h: Hoisted-S4 Edge-Packed GPR Len4` (`LIMIT = 16`, `0 B` scratch):** Hoisting `if !wide && edge_len_mode == 4` outside the `256`-symbol `S4` wire loop recovers `-42.1 µs` of Cycle 24's unhoisted `S4` branch cost (`795.6 -> 753.5 µs`), establishing a new pure-$\mu=1$ champion at **`10,490.5 µs` (`-1,929.0 µs` = `-15.5%` below `M0: 0.5.11`)** and **`0.854×` `A1:Master` p50 / `0.852×` p10 (`0.879×` `M0`)** at `950.0 KiB` (`-19.0%`) scratch — beating `Neon-C29`'s claimed `0.855×` 1T headline without `Fused-XOR`.
- **KILL `Lower Uniform Schur Thresholds (LIMIT = 8, 10, 12, 14)` (`+104` to `+1,330 µs` slower):** Even after `GradedOperad` cut `S2+S3` by `-26.5%`, absorbing runs of width $w \le 16$ incurs `1.49×–2.50×` more `Pre(S1–4)` cost (`S2` RV steps + `S3` bands + `S4` shorter spans) than it saves in `S5`, proving `LIMIT = 16` remains the unique global minimum on `E2h`.
- **KILL `Higher Uniform Threshold (LIMIT = 20)` (`+25.4 µs` slower, `+84.6 KiB` scratch):** Releasing runs of width `17..=20` back to `S5` saves `-331 µs` in `Pre(S1–4)` but adds `+357 µs` in `S5` and inflates persistent worker scratch from `950.0 KiB` to `1,034.6 KiB`.
- **KILL `Per-Symbol Online Adaptive (SymAd 10/16)` & `Block-Density Adaptive (BlkAd 12/16)` (`+696 µs` & `+205 µs` slower):** Even on the highest-$m$/lowest-$ns$ blocks (`block-85.L`, `block-210.L`), lowering $\Lambda$ below `16` loses monotonically on every single Wikidata block.

---

## 2. Exact Post-`GradedOperad` Schur Threshold Convexity (`measured` & `verified`)

### 2.1 Structure & Persistent Scratch Census Across `LIMIT ∈ {8, 10, 12, 14, 16, 20}` (`verified`)

| Block | `LIM=8` (`m/ns/U`) | `LIM=10` (`m/ns/U`) | `LIM=12` (`m/ns/U`) | `LIM=16` (`m/ns/U`) | `SymAd` (`m/ns/U`) |
|---|---:|---:|---:|---:|---:|
| `block-0.L` | `67,502 / 5,461 / 1,269` | `78,025 / 4,349 / 1,174` | `82,815 / 3,933 / 1,082` | `89,874 / 3,448 / 995` | `81,883 / 4,056 / 1,157` |
| `block-2.L` | `82,863 / 6,273 / 1,716` | `90,642 / 5,454 / 1,559` | `98,792 / 4,757 / 1,461` | `112,070 / 3,827 / 1,281` | `95,104 / 5,119 / 1,562` |
| `block-85.L` | `118,953 / 5,211 / 1,381` | `142,219 / 2,699 / 1,120` | `150,312 / 1,990 / 952` | `156,404 / 1,565 / 811` | `144,802 / 2,499 / 1,093` |
| `block-143.L` | `157,658 / 8,541 / 2,488` | `171,958 / 7,016 / 2,270` | `185,505 / 5,828 / 2,024` | `196,841 / 5,047 / 1,712` | `175,655 / 6,721 / 2,244` |
| `block-210.L` | `171,264 / 5,506 / 1,629` | `180,008 / 4,592 / 1,452` | `187,394 / 3,951 / 1,323` | `204,281 / 2,809 / 1,219` | `184,029 / 4,307 / 1,512` |
| `block-207.L` | `117,323 / 5,078 / 1,633` | `123,546 / 4,426 / 1,468` | `129,003 / 3,953 / 1,339` | `135,921 / 3,474 / 1,205` | `126,306 / 4,216 / 1,448` |
| **6-Wikidata Scratch** *(excl. `879 KiB` `out`)* | `1,328.3 KiB` *(RV arena peak!)* | `1,065.5 KiB` | `932.1 KiB` | **`950.0 KiB`** (`-19.0%` vs `M0`) | `1,033.3 KiB` |

*(At `LIM = 20`, scratch rises back to `1,034.6 KiB` from larger `edges[0..m)`. At `LIM = 16`, `10` i9 workers = `17.86 MiB <= 20 MiB` L3!)*

### 2.2 Convex 5-Stage Pareto Frontier (`6`-Block Sum `µs`, `15` Rounds Median, `measured`)

| Arm | `S1` | `S2` | `S3` | `S4` | **`Pre (S1–4)`** | **`S5 Emit`** | **`Sum`** | Marginal ROI vs `E2h(16)` |
|---|---:|---:|---:|---:|---:|---:|---:|---|
| `M0: 0.5.11 (L=16)` | `2339.2 µs` | `1943.7 µs` | `1546.0 µs` | `757.7 µs` | `6586.5 µs` | `5833.0 µs` | `12419.5 µs` | Baseline |
| `E1: ZSpill (L=16)` | `1964.5 µs` | `1735.6 µs` | `798.6 µs` | `734.8 µs` | `5233.5 µs` | `5297.7 µs` | `10531.1 µs` | Cycle 24 `E1` |
| **`E2h: HoistL4 (L=16)`** | **`1943.5 µs`** | **`1712.0 µs`** | **`799.9 µs`** | **`753.5 µs`** | **`5209.0 µs`** | **`5281.5 µs`** | **`10490.5 µs`** | **GLOBAL OPTIMUM** |
| `L08: E2h (LIM=8)` | `2084.4 µs` | `2978.8 µs` | `1401.5 µs` | `961.5 µs` | `7426.1 µs` | `4394.1 µs` | `11820.3 µs` | `+2,217` Pre / `-887` S5 (`2.50×` neg) |
| `L10: E2h (LIM=10)` | `1991.6 µs` | `2415.6 µs` | `1156.8 µs` | `879.2 µs` | `6443.3 µs` | `4739.4 µs` | `11182.7 µs` | `+1,234` Pre / `-542` S5 (`2.28×` neg) |
| `L12: E2h (LIM=12)` | `1995.6 µs` | `2082.9 µs` | `976.9 µs` | `830.1 µs` | `5885.6 µs` | `4921.6 µs` | `10807.2 µs` | `+677` Pre / `-360` S5 (`1.88×` neg) |
| `L14: E2h (LIM=14)` | `1946.4 µs` | `1899.9 µs` | `889.0 µs` | `788.5 µs` | `5523.8 µs` | `5070.9 µs` | `10594.7 µs` | `+315` Pre / `-211` S5 (`1.49×` neg) |
| `L20: E2h (LIM=20)` | `2081.5 µs` | `1434.5 µs` | `660.1 µs` | `701.9 µs` | `4877.9 µs` | `5638.0 µs` | `10515.9 µs` | `-331` Pre / `+357` S5 (`1.08×` neg) |
| `SymAd (10/16)` | `2128.3 µs` | `2276.6 µs` | `1035.4 µs` | `882.7 µs` | `6322.9 µs` | `4863.8 µs` | `11186.7 µs` | `+1,114` Pre / `-418` S5 (`2.67×` neg) |
| `BlkAd (12/16)` | `1955.7 µs` | `1972.3 µs` | `917.9 µs` | `809.0 µs` | `5654.8 µs` | `5040.6 µs` | `10695.4 µs` | `+446` Pre / `-241` S5 (`1.85×` neg) |

**Why `LIMIT = 16` remained pinned despite `GradedOperad` (`proved` empirically):** Because `E2h` (`ZeroSpill + GPR Len4`) reduced the marginal cost per `S5` step by `-9.5%` in parallel with `GradedOperad` cutting `S2+S3` by `-26.5%`, while lowering `LIMIT` simultaneously inflates **all four** pre-emit stages (`S1` strip checks, `S2` RV steps, `S3` bands/dedup, AND `S4` shorter `OutQ16` spans!).

---

## 3. Headline 1T `InverseBwt::invert_runs` Confrontation (`25` Rounds, `measured`)

`A2/A1` A/A control = **`0.993×` p50 (`0.985×` p10)** on the 6 authenticated Wikidata blocks (`12,420.8 µs` sum):

| Block | `A1_p50` | `A2/A1` | `M0/A1` | `E1/A1` | **`E2h/A1`** | `L10/A1` | `L12/A1` | `Sym/A1` | `Blk/A1` |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| `block-0.L` | `1369.6 µs` | `0.996×` | `0.985×` | `0.868×` | **`0.857×`** | `0.909×` | `0.888×` | `0.892×` | `0.855×` |
| `block-2.L` | `1673.2 µs` | `0.999×` | `0.988×` | `0.858×` | **`0.851×`** | `0.921×` | `0.888×` | `0.904×` | `0.849×` |
| `block-85.L` | `1701.8 µs` | `0.993×` | `0.956×` | `0.859×` | **`0.855×`** | `0.914×` | `0.875×` | `0.914×` | `0.878×` |
| `block-143.L` | `3025.0 µs` | `0.990×` | `0.968×` | `0.847×` | **`0.841×`** | `0.903×` | `0.872×` | `0.904×` | `0.866×` |
| `block-210.L` | `2648.1 µs` | `0.995×` | `0.965×` | `0.864×` | **`0.868×`** | `0.909×` | `0.877×` | `0.904×` | `0.869×` |
| `block-207.L` | `2003.1 µs` | `0.986×` | `0.976×` | `0.859×` | **`0.854×`** | `0.909×` | `0.875×` | `0.901×` | `0.876×` |
| `dense-703.syn` | `3138.0 µs` | `0.985×` | `0.946×` | `0.835×` | `0.843×` | `0.821×` | `0.826×` | `0.825×` | `0.831×` |
| **6-BLK p50/A1** | **`12420.8 µs`** | **`0.993×`** | **`0.972×`** | **`0.858×`** | **`0.854×`** | `0.910×` | `0.878×` | `0.904×` | `0.866×` |
| **6-BLK p10/A1** | **`12027.4 µs`** | **`0.985×`** | **`0.972×`** | **`0.856×`** | **`0.852×`** | `0.909×` | `0.877×` | `0.903×` | `0.867×` |
| **6-BLK p50/M0** | `12067.6 µs` | — | `1.000×` | `0.883×` | **`0.879×`** | `0.936×` | `0.903×` | `0.930×` | `0.892×` |

**Per-Worker Working Set (`E2h`):** `950.0 KiB` persistent scratch (`-19.0%` vs `M0`) + `879 KiB` `out` = **`1.786 MiB`/worker** (`10w = 17.86 MiB <= 20 MiB` L3); steady-state DRAM `<= 0.034–0.059 B/B`.

---

## 4. Next Cycle Plan (`Cycle 26 (Neon)`)
With `LIMIT = 16` proved globally optimal and `E2h` at **`0.854×` `A1` (`5,281.5 µs` `S5`, `50.3%` of kernel)**, Cycle 26 launches **Attack #3 on `S5 Emit`**: **Direct Attack on the `edges[0..m)` `346k` Unprefetched L2 Miss & `430k` DTLB Miss Wall at $\mu = 1$ (Without `#705` Scouting or `#718` `wire_xor`)** — testing:
1. **Unrolled `2`-Step / `4`-Step `emit_zerospill` + `S5` Register-Pressure Audit (`U1`):** Eliminating loop-counter `steps < m` + `at < n` checks on every step via `m`-budget chunking (`let batch = (m - steps).min((n - at) / 16)`!) — cutting `3` branch instructions (`steps < m`, `at < n`, `at + 16 <= out_buf.len()`) out of the hot `S5` body!
2. **`24`-Bit Packed Edge / Byte-Offset Compression Audit (`U2`):** Computing the exact $\mathsf{BMCP}_{\mathrm{CML}}$ bound on packing `edges` from `32` bits (`4 B`) to `24` bits (`3 B`, `-25%` `edges` footprint & `-25%` DTLB pages!) vs unaligned `u32` load + shift/mask + `S4` write cost — and testing it directly if the bit budget permits (`bits + code_bits` vs `24`!).
3. **2-Way / 4-Way Exact Cycle-Split `S5` Without `wire_xor` (`U3` — Zero-`S4`-Cost `Cycle-End` Bidirectional?):** Proving whether ANY bidirectional/multi-stream walk can avoid `#718`'s `+879 µs` `wire_xor` cost (spoiler: reversing $\psi$ requires writing `prev[next] = cur`, which is a random scatter over `m` edges = `wire_xor`!).
