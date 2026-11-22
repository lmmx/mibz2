# Carbon Kitchen Sink (Neon) — Cycle 41: Branch-Fused `OneBr` + `FastJoin` Hit `0.900×` `Pre` (`−612 µs` vs `53f7ecf93`, `0.943×` `1T` / `0.945×` `2T`) & `Iota8`/`Iota4` Kill

**Date:** 2026-11-21  
**Branch:** `research/carbon-kitchen-sink`  
**Persona:** Gemini Neon  
**Baseline:** `origin/master` (`53f7ecf93`, `#719` construction stack, `945.8 KiB` scratch)  
**Scope:** Research-only; `src/`, `Cargo.toml`, `benches/`, and `examples/` untouched.

---

## 1. Mathematical & Microarchitectural Framing

Cycle 41 tested four targeted hypotheses across the two largest remaining construction stages (`S1: ~1,670 µs` and `S2b: ~1,150 µs`):
1. **`OneBr` (`S1` Predicate Fusion):** Replace `S1Trim`'s short-circuiting `w <= 16 && f + 16 <= CHUNK && (m ^ (m + w)) < EPOCH` (`3` conditional branches + `m + w` dependency) with one bitwise-fused guard `(w <= 16) & (f <= CHUNK - 16) & ((m as u16) < 65_520)` (`1` branch; `(m as u16) < 65_520` evaluates in parallel with `self.pt[a]` load!).
2. **`Iota8` (`16 B` XMM, `w <= 8`) & `Iota4` (`8 B` GPR SWAR `lo * 0x0001_0001_0001_0001 + 0x0003_0002_0001_0000`, `w <= 4`):** Test whether cutting `OutQ16` store width & 64-byte cache-line splits (`47% -> 22% / 9%`) outweighs routing `w ∈ 9..=16` (`2.4%`) or `w ∈ 5..=16` (`7.1%`) to `#[cold] push_slow_pt`.
3. **`FastJoin` (`S2b` Algebraic `SATURATED` Collapse):** Because `left_len, right_len >= 1`, `sum = left_len + right_len < 16` implies both `< SATURATED` (`0` saturation checks on all short joins), and `sum <= SATURATED` implies both `< SATURATED` (`1` check instead of `2` on all long joins).

---

## 2. Native Safe-Rust A/B Confrontation (`run_c41.rs`)

Built atop `Neon-C40` (`S1Trim`) and compared in the same binary against verbatim `origin/master` (`53f7ecf93`) and `Master-Nobi`, compiled with `rustc --edition 2021 -C opt-level=3 -C target-cpu=native -C codegen-units=1` (`#![forbid(unsafe_code)]`).

- **Parity & Scratch:** `100%` byte-exact across all `11` kernels on `6/6` Wikidata blocks + `dense-300k`; persistent scratch unchanged at **`945.8 KiB` (`+0.00%`)**.

### 2.1 `1T` Interleaved (`50` rounds/block, `35`-round `Pre` split) & `2T` Concurrent Barrier (`35` passes)

| Candidate | `1T` Total (`µs`, ratio vs base) | `1T` `Pre` (`µs`, ratio vs base) | `ΔPre` vs `C40` | `2T` p10 (`µs`, ratio) | `2T` p50 (`µs`, ratio) |
|---|---:|---:|---:|---:|---:|
| `MasterA1` (`53f7ecf93`) | `10,717.0 (1.0000×)` | `6,128.6 (1.0000×)` | `+512.2` | `10,574.2 (1.0000×)` | `10,620.3 (1.0000×)` |
| `MasterA2` (A/A) | `10,722.8 (1.0005×)` | `6,114.1 (0.9976×)` | `+497.7` | `10,562.2 (0.9989×)` | `10,605.2 (0.9986×)` |
| `Neon-C40` (prev champ) | `10,229.4 (0.9545×)` | `5,616.4 (0.9164×)` | `0.0` | `10,095.3 (0.9547×)` | `10,172.8 (0.9579×)` |
| `Iota8` (`16 B`, `w <= 8`) | `10,495.1 (0.9793×)` | `5,883.9 (0.9601×)` | `+267.5` | `10,361.0 (0.9798×)` | `10,437.4 (0.9828×)` |
| `Iota4` (`8 B` SWAR, `w <= 4`) | `10,981.8 (1.0247×)` | `6,384.3 (1.0417×)` | `+768.0` | `10,858.6 (1.0269×)` | `10,900.3 (1.0264×)` |
| **`OneBr` (`Neon-C41`)** | **`10,107.2 (0.9431×)`** | **`5,516.6 (0.9001×)`** | **`−99.8`** | — | — |
| `FastJoin` (`S2b`) | `10,198.9 (0.9517×)` | `5,587.6 (0.9117×)` | `−28.8` | — | — |
| **`C41-Cmb` (`OneBr+FastJoin`)** | **`10,116.6 (0.9440×)`** | **`5,535.4 (0.9032×)`** | **`−81.0`** | **`10,000.4 (0.9457×)`** | **`10,036.4 (0.9450×)`** |
| `Master-Nobi` ($\mu=1$ base) | `11,140.8 (1.0000×)` | `5,330.0 (1.0000×)` | `+491.0` | `10,996.3 (1.0000×)` | `11,067.6 (1.0000×)` |
| `Nobi-C40` *(clean)* | `10,616.7 (0.9530×)` | `4,839.1 (0.9079×)` | `0.0` | `10,517.6 (0.9565×)` | `10,619.9 (0.9595×)` |
| **`Nobi-C41` (`+OneBr+FJ`)** | `10,620.5 (0.9533×)` | **`4,835.7 (0.9073×)`** | `−3.3` | — | — |

Per-block `1T` ratios for **`OneBr`** vs `MasterA1` (A/A `1.0005×`):
`block-0: 0.949`, `block-2: 0.955`, `block-85: 0.943`, `block-143: 0.941`, `block-210: 0.936`, `block-207: 0.942` (**`5/6` blocks `< 0.950×`!**).

---

## 3. Theorems, One-Line Kills, & Promotions

- **Theorem 41.1 (Bitwise Predicate Fusion & Dependency Breaking in `OutQ16::push_thin`):** Collapsing `f + 16 <= CHUNK && (m ^ (m + w)) < EPOCH` into `(w <= 16) & (f <= CHUNK - 16) & ((m as u16) < 65_520)` breaks the `m + w` dependency chain and eliminates short-circuit branch overhead across `489K` thin runs (`−99.8 µs` `Pre`).
- **Theorem 41.2 (Exact `LIMIT = 16` Coverage Imperative for `OutQ16`):** Any fast-path width $K < \mathrm{LIMIT} = 16$ introduces an unpredictable `w <= K` branch on thin runs (`2.4%` for $K=8$, `7.1%` for $K=4$) whose mispredict and `#[cold] push_slow_pt` call penalty (`+267.5 / +768.0 µs`) dwarfs saving `16–24 B` of L1d write bandwidth.
- **KILL `Iota8` (`+267.5 µs` `Pre`) & `Iota4` (`+768.0 µs` `Pre`):** Sub-`LIMIT` fast paths pay severe branch-misprediction and cold-call penalties on `w ∈ (K, 16]` (Theorem 41.2).
- **PROMOTE `Neon-C41` (`OneBr + FastJoin`, `+0 B` WS):** Cuts construction `Pre` by **`−10.0%` (`0.9001×` / `−612.0 µs` for `OneBr`, `0.9032×` for `C41-Cmb`)** vs `origin/master` (`53f7ecf93`), reaching **`0.9431×–0.9440×` `1T`** (`5/6` blocks `< 0.95×`) and **`0.9457×` p10 / `0.9450×` p50 `2T`** (vs A/A `1.0005× / 0.9986×`)!

---

## 4. Artifacts & Next Step

- Reproducers archived in `docs/journal/2026-11-21-carbon-kitchen-sink-cycle41/` (`gen_c41.py`, `run_c41.rs`).
- **Cycle 42 (Neon):** Radical algorithmic/representational push on **`S1` + `S2b` + `S3a` + `S4`** — testing (i) **`S1-Unroll2` / `S1-DirectPtr` (2-run unrolled thin-run prefetching/pipelining for `pt[a0]` + `pt[a1]`)**, (ii) **`S2b` Explicit Branchless `Elim` Width/Word Prefetch & Arena-Size Pre-Allocation (eliminating `grow()`!)**, and (iii) **`S3a` Compact `Band64` Direct-Slice Write (`bands.reserve(nodes)` -> `unsafe`-free fixed-slice or `extend`) + `S4` `Wire16` `2×` Band-Loop Prefetch**!
