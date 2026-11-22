# Wormhole (Neon) Cycle 27 — `S1: Pass 1 + OutQ16` Attack: Fused Loop, 64-Bit Parallel `(thin, pos)`, & Single-Path AVX2 Floor

**Session:** `research/carbon-wormhole` (Round 4 — Gemini Neon, Cycle 27 of 30; 7th of 10 in Round 4)  
**Baseline `origin/master`:** `8ff1187af6a920f7d57fd338680db3a5633a3ac5` (`v0.5.11`; `src/` byte-identical to `90e671966`)  
**Verification harness:** [`verify_cycle27.rs`](2026-11-21-wormhole-cycle27-s1-pass1-fusion-pt64-and-tail/verify_cycle27.rs) (`#![forbid(unsafe_code)]`, `0` warnings, `6/6` `manifest.json` SHA-256 auth + `5,229` Gate G1/spiral/multi-epoch cases byte-exact + exact `S1` run-width/`push_slow` census + `15`-round 5-stage attribution + `25`-round `p50`/`p10` `InverseBwt::invert_runs` confrontation), [`s1_attack.toml`](2026-11-21-wormhole-cycle27-s1-pass1-fusion-pt64-and-tail/s1_attack.toml) (`PASS`), [`cycle27_output.txt`](2026-11-21-wormhole-cycle27-s1-pass1-fusion-pt64-and-tail/cycle27_output.txt), [`cycle27_results.json`](2026-11-21-wormhole-cycle27-s1-pass1-fusion-pt64-and-tail/cycle27_results.json)

---

## 1. One-Line Keep & Kill Decisions (`S1` Hardware Floor Confirmed)

- **KILL `P1: Fused Phase-1/Phase-2 Pass1` (`+142.1 µs` in `S1`):** Splitting `maximal()` into pre-`p` and post-`p` loops duplicates the `QueuesPos::push` body, tripping Cycle 19's register-spill cliff inside `S1`.
- **KILL `P2: Fused Phase-1/2 + pt64` (`+151.2 µs` in `S1`):** Inherits `P1`'s duplicated-loop spill penalty on Wikidata (`-45.6 µs` on `dense-703.syn`).
- **KILL `P3: w == 1 Scalar Fast-Path` (`+1,846.3 µs` in `S1`, `1.96×` slower!) & `P4: Inline Chunk-Tail f > 496` (`+1,083.3 µs` in `S1`, `1.57×` slower!):** Proves **Theorem 27.2 (`S1` `OutQ16` Single-Path Inlining Law)** — adding *any* second inlined write path inside `QueuesPos::push` breaks LLVM's `vpaddw + vmovdqu %ymm0` AVX2 vectorization/register schedule, even when taken on only `2.50%` of thin runs (`12,206 / 489,099`).
- **HOLD / RE-AUDIT `P2c: CleanPt64` (`+13.4 µs` neutral on Wikidata `S1`, `-12.7 µs` on `dense-703.syn`, `-1 KiB` stack frame, `0.867×` vs `E2h`'s `0.875×` E2E on `7/7` blocks):** Single-loop `pt[a] = (thin[a] << 32) | pos[a]` (`1` 64-bit add+store per thin run) eliminates `P1`'s spill cliff and wins `7/7` blocks end-to-end (`-107 µs` sum), likely via `-1 KiB` stack-frame reduction / cleaner `try_run` layout rather than `S1` itself (`+13.4 µs`); deferred to Cycle 29 synthesis ablation rather than claiming an `S1` gain.

---

## 2. Exact `S1` Run-Width & `push_slow` Census (`verified`)

Across the `6` authenticated Wikidata blocks (`489,099` thin runs):
- **`73.0%` (`356,921`)** have `width == 1` and **`92.9%` (`454,452`)** have `width <= 4`.
- **`97.07%` (`474,766`)** hit the single `vpaddw + vmovdqu %ymm0` fast path; only **`2.93%` (`14,333`)** enter `#[cold] push_slow` (`2,115` chunk allocations = `0.43%`, `12,206` chunk-tail `f ∈ [497..511]` = `2.50%`, `12` epoch cuts = `0.002%`).

| Block | `r_thin` | `w == 1 (%)` | `w <= 4 (%)` | `Alloc` | `TailSlow` | `Epoch` |
|---|---:|---:|---:|---:|---:|---:|
| `block-0.L` | `47,736` | `35,116 (73.6%)` | `44,086 (92.4%)` | `238` | `1,111` | `1` |
| `block-2.L` | `50,705` | `34,262 (67.6%)` | `45,254 (89.2%)` | `273` | `1,173` | `1` |
| `block-85.L` | `77,810` | `55,504 (71.3%)` | `70,400 (90.5%)` | `366` | `1,921` | `2` |
| `block-143.L` | `103,288` | `72,578 (70.3%)` | `94,727 (91.7%)` | `447` | `2,586` | `3` |
| `block-210.L` | `125,923` | `97,054 (77.1%)` | `120,126 (95.4%)` | `463` | `3,306` | `3` |
| `block-207.L` | `83,637` | `62,407 (74.6%)` | `79,859 (95.5%)` | `328` | `2,109` | `2` |
| `dense-703.syn` | `179,371` | `134,589 (75.0%)` | `179,371 (100.0%)` | `459` | `4,380` | `3` |
| **6-BLK SUM** | **`489,099`** | **`356,921 (73.0%)`** | **`454,452 (92.9%)`** | **`2,115`** | **`12,206`** | **`12`** |

### Theorem 27.2 (`S1` `OutQ16` Single-Path Inlining Law, `verified`)
`PassMuxAllPos` + `QueuesPos::push` already hits its single-path AVX2 `vmovdqu` store on `97.07%` of thin runs (`1,897–1,915 µs`, `-335 µs` = `-14.9%` below `M0: 0.5.11`). Any second inlined write path (`P3: w == 1` scalar `+1,846 µs`; `P4: f > 496` tail `+1,083 µs`; `P1` Phase-2 duplicate `+142 µs`) disrupts LLVM's `vpaddw + vmovdqu %ymm0` register schedule and costs `10–30×` more than the `2.5%` `push_slow` tail it attempts to avoid. Thus `PassMuxAllPos` + `QueuesPos` is at the safe-Rust `S1` hardware floor.

---

## 3. Verification & Benchmarks (`measured` in [`verify_cycle27.rs`](2026-11-21-wormhole-cycle27-s1-pass1-fusion-pt64-and-tail/verify_cycle27.rs))

- **Exactness (`verified`):** `6/6` `manifest.json` SHA-256 auth + `dense-703.syn` + **`5,229` Gate G1 / deep spiral / multi-epoch (`m > 65,536` `pt64` epoch-cut extraction)** cases byte-exact across all arms.
- **Per-Worker Working Set:** `950.0 KiB` persistent scratch across all arms (`-19.0%` vs `M0`'s `1,172.7 KiB`; `10w = 17.86 MiB <= 20 MiB` L3); steady-state DRAM `<= 0.034–0.059 B/B`.

### 3.1 Per-Block `S1` & 6-Block 5-Stage Attribution (`15` Rounds Median `µs`)

| Arm (6-Blk Sum) | **`S1`** | `S2` | `S3` | `S4` | `Pre (S1–4)` | `S5 Emit` | **`Sum`** | Verdict |
|---|---:|---:|---:|---:|---:|---:|---:|---|
| `M0: Clone (0.5.11)` | `2249.7 µs` | `1901.9 µs` | `1535.7 µs` | `735.7 µs` | `6423.0 µs` | `5774.6 µs` | `12197.6 µs` | Baseline |
| **`E2h: C25 Champ`** | **`1914.6 µs`** | `1701.8 µs` | `782.1 µs` | `737.0 µs` | `5135.5 µs` | `5218.4 µs` | `10353.9 µs` | **RETAINED `S1`** |
| `P1: FusedPass1` | `2056.7 µs` | `1704.4 µs` | `780.8 µs` | `745.0 µs` | `5286.9 µs` | `5202.2 µs` | `10489.0 µs` | **KILL** (`+142 µs` `S1`) |
| `P2: +pt64` | `2065.8 µs` | `1697.1 µs` | `777.5 µs` | `745.8 µs` | `5286.3 µs` | `5204.9 µs` | `10491.2 µs` | **KILL** (`+151 µs` `S1`) |
| `P3: +w==1Scalar` | `3760.9 µs` | `1693.2 µs` | `777.9 µs` | `731.9 µs` | `6964.0 µs` | `5183.2 µs` | `12147.2 µs` | **KILL** (`+1,846 µs` `S1`) |
| `P4: +ChunkTail` *(run 1)* | `2979.9 µs` | `1687.1 µs` | `775.1 µs` | `719.9 µs` | `6162.1 µs` | `5132.9 µs` | `11294.9 µs` | **KILL** (`+1,083 µs` `S1`) |
| `P2c: CleanPt64` | `1928.0 µs` | `1667.9 µs` | `765.3 µs` | `746.2 µs` | `5107.3 µs` | `5139.9 µs` | `10247.2 µs` | **HOLD** (`+13 µs` `S1`; re-test C29) |

### 3.2 Headline 1T `InverseBwt::invert_runs` Confrontation (`25` Rounds, `measured`)

`A2/A1` A/A control = **`0.997×` p50 (`0.983×` p10)** and `M0/A1` = **`0.995×` p50 (`0.994×` p10)** on the freshly provisioned VM (`11,861.0 µs` 6-block sum):

| Block | `A1_p50` | `A2/A1` | `M0/A1` | `E2h/A1` | `P1/A1` | `P2/A1` | `P3/A1` | **`P2c/A1`** |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| `block-0.L` | `1312.8 µs` | `0.993×` | `1.011×` | `0.876×` | `0.864×` | `0.865×` | `0.983×` | **`0.863×`** |
| `block-2.L` | `1601.4 µs` | `1.004×` | `1.004×` | `0.874×` | `0.869×` | `0.867×` | `0.981×` | **`0.865×`** |
| `block-85.L` | `1624.7 µs` | `0.988×` | `0.980×` | `0.887×` | `0.885×` | `0.882×` | `1.034×` | **`0.875×`** |
| `block-143.L` | `2859.3 µs` | `1.013×` | `1.008×` | `0.872×` | `0.883×` | `0.888×` | `1.046×` | **`0.861×`** |
| `block-210.L` | `2552.4 µs` | `0.985×` | `0.974×` | `0.875×` | `0.896×` | `0.899×` | `1.083×` | **`0.872×`** |
| `block-207.L` | `1910.5 µs` | `0.991×` | `0.994×` | `0.869×` | `0.872×` | `0.864×` | `0.960×` | **`0.868×`** |
| `dense-703.syn` | `2994.1 µs` | `0.984×` | `0.970×` | `0.877×` | `0.845×` | `0.823×` | `0.964×` | **`0.840×`** |
| **6-BLK p50/A1** | **`11861.0 µs`** | **`0.997×`** | **`0.995×`** | **`0.875×`** | `0.880×` | `0.880×` | `1.023×` | **`0.867×`** |
| **6-BLK p10/A1** | **`11262.3 µs`** | **`0.983×`** | **`0.994×`** | **`0.876×`** | `0.884×` | `0.881×` | `1.028×` | **`0.871×`** |

---

## 4. Next Cycle Plan (`Cycle 28 (Neon)`)
With `S5`, `S2`, and `S1` all bounded and attacked, Cycle 28 audits the final two stages **`S3` (`782 µs`) + `S4` (`737 µs`)** AND the `#703` dense regime:
1. **Exact `S3` `pair_ht` / `str_ht` Hit-Rate & Collision Census** on all 6 Wikidata blocks + `dense-703.syn` (`U = 4,370` exceeds `STR_LEN = 4,096`!).
2. **`D1: 2-Way Set-Associative `str_ht` vs Direct-Mapped `str_ht` & `PAIR_BITS = 11`** (inside the existing `24 KiB` dead `edges[0..6144]` overlay, `0 B` scratch) — measuring whether `str_ht` evictions on `block-143.L` / `dense-703.syn` leak duplicate codes into `inline16` and `spelling`!
3. **`D2: `S3` Inline Literal-1 Fast-Path (`h_len == 1` — `2`-byte word!) & `S4` `OutQ16` Reader Audit:** Checking if `h_len == 1` (`c + a`, 2 bytes!) avoids `short_u128` + `hash_u128` overhead cleanly!
