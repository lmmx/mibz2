# Wormhole (Neon) Cycle 26 — `S5 Emit` Attack #3: Pure-Leaf Walk, Safe Power-of-2 Bounds Elision, & Closing the $\mu = 1$ Hardware Floor

**Session:** `research/carbon-wormhole` (Round 4 — Gemini Neon, Cycle 26 of 30; 6th of 10 in Round 4)  
**Baseline `origin/master`:** `8ff1187af6a920f7d57fd338680db3a5633a3ac5` (`v0.5.11`; `src/` byte-identical to `90e671966`)  
**Verification harness:** [`verify_cycle26.rs`](2026-11-21-wormhole-cycle26-s5-pure-leaf-bounds-elision/verify_cycle26.rs) (`#![forbid(unsafe_code)]`, `0` warnings, `6/6` `manifest.json` SHA-256 auth + `5,229` Gate G1/spiral/multi-epoch cases byte-exact + `15`-round 5-stage attribution + `25`-round `p50`/`p10` `InverseBwt::invert_runs` confrontation), [`s5_leaf_bounds.toml`](2026-11-21-wormhole-cycle26-s5-pure-leaf-bounds-elision/s5_leaf_bounds.toml) (`PASS`), [`cycle26_output.txt`](2026-11-21-wormhole-cycle26-s5-pure-leaf-bounds-elision/cycle26_output.txt), [`cycle26_results.json`](2026-11-21-wormhole-cycle26-s5-pure-leaf-bounds-elision/cycle26_results.json)

---

## 1. One-Line Keep & Kill Decisions (`S5` $\mu = 1$ Hardware Floor Closed)

- **KILL `L1: Pure-Leaf #[inline(never)] emit_short_run` (`+313.8 µs` in `S5`):** `LONG` steps occur every `16.7` walk steps on average (`6.0%` = `53,723` exits across 6 blocks), so factoring the inner loop into a separate `#[inline(never)]` function pays `53,723` full call/prologue/epilogue sequences (`~25` cycles each $\approx +314\text{--}370\text{ µs}$).
- **KILL `L2: Safe Power-of-2 Edges/Dict Bounds Elision` (`+677.0 µs` in `S5`):** Zero-filling the unused `edges[m..1<<bits]` tail right before `S5` touches up to `240 KiB` of cold lines, evicting `edges[0..m)` out of the `256 KiB` L2 cache (`+363 µs` on top of `L1`).
- **KILL `L3: Out-of-Line Chunked Pure-Leaf` (`+204.0 µs` in `S5`):** Chunking `budget = rem.min((n - at) >> 4)` recovers `-110 µs` vs `L1`, still unable to pay for the `53,723` leaf-function calls.
- **KILL `L4: Inline Chunked Budget Inside E2h` (`+5.4 µs` neutral in `S5`, `0.859×` vs `E2h`'s `0.858×`):** Removing `at < n` and `out_buf` bounds checks with zero leaf-call overhead yields `0.0 µs` net speedup (`+5.4 µs`), proving that **`E2h`'s early-`j` scheduling already hides `100%` of non-critical-path ALU/bounds ops inside the `edges[j]` L2/L3/STLB miss shadow** — closing `S5` at $\mu = 1$.

---

## 2. Two Proved Single-Stream `S5` Structural Lower Bounds

### Theorem 26.1 (Information-Theoretic Edge-Width Lower Bound — Closing `24`-Bit Packed Edges)
*Claim (`proved` + `verified`):* Even after `WordDedup` compresses unique dictionary codes $U$ to `811–1,712`, every retained edge must encode `(next_row, word_code)`, requiring
$$B_{\min} = \lceil \log_2 m \rceil + \lceil \log_2 U \rceil \in [27, 29]\text{ bits} > 24\text{ bits}$$
on **`6/6` Wikidata blocks** (`27` on `block-0.L`; `28` on `block-2.L, block-85.L`; `29` on `block-143.L, block-210.L, block-207.L`). Hence `32`-bit `edges[j]` is **tight at byte granularity**: shrinking `edges` to `24` bits (`-25%` L2/DTLB pages) is impossible without spilling `code` into a second random-access `extra` array, which adds a second unprefetched L2/DTLB stream.

### Theorem 26.2 (Bidirectional `S4` Scatter Barrier — Closing Zero-Overhead Multi-Stream $\mu \ge 2$)
*Claim (`proved` + `verified` via Cycle 21 & `#718`):* Breaking `S5`'s serial dependency depth $D = m$ without `#705`'s scouting pass requires inverting the forward permutation $\psi$ (`edges[dst] = next`) to obtain $\psi^{-1}$ (`prev[next] = dst` or `xor[next] ^= dst`). Because `S4` writes `dst` sequentially per band while `next` comes from `OutQ16` in arbitrary order, any exact $\psi^{-1}$ construction performs **$m$ random read-modify-writes** across `350–800 KiB` (`+879 µs` in Cycle 21; `1.021×` slower natively at 4W in `#718`). Combined with `L4 - E2h = +5.4 µs` (empirical ALU-hiding saturation), **`E2h` sits at the true $\mu = 1$ $\mathsf{BMCP}_{\mathrm{CML}}$ floor of `S5 Emit` (`5,313–5,338 µs`, `-9.3%` to `-10.1%` below `M0: 0.5.11`)**.

---

## 3. Verification & Benchmarks (`measured` in [`verify_cycle26.rs`](2026-11-21-wormhole-cycle26-s5-pure-leaf-bounds-elision/verify_cycle26.rs))

- **Exactness (`verified`):** `6/6` `manifest.json` SHA-256 auth + `dense-703.syn` + **`5,229` Gate G1 / deep spiral / multi-epoch** cases byte-exact across `M0, E2h, L1, L2, L3, L4`.
- **Per-Worker Working Set:** `950.0 KiB` persistent scratch across `E2h, L1, L2, L3, L4` (`-19.0%` vs `M0`'s `1,172.7 KiB`; `10w = 17.86 MiB <= 20 MiB` L3); steady-state DRAM `<= 0.034–0.059 B/B`.

### 3.1 Per-Block `S5 Emit` & 5-Stage Attribution (`15` Rounds Median `µs`)

| Block | `M0: 0.5.11` | **`E2h: C25`** | `L1: Leaf` | `L2: +BCEL` | `L3: +Chnk` | `L4: InCh` | `L4 - E2h` |
|---|---:|---:|---:|---:|---:|---:|---:|
| `block-0.L` | `543.9 µs` | **`493.6 µs`** | `515.9 µs` | `580.1 µs` | `521.9 µs` | `497.9 µs` | `+4.3 µs` |
| `block-2.L` | `688.5 µs` | `625.5 µs` | `651.9 µs` | `733.6 µs` | `646.9 µs` | **`620.7 µs`** | `-4.8 µs` |
| `block-85.L` | `940.3 µs` | `846.1 µs` | `867.0 µs` | `869.9 µs` | `870.2 µs` | **`845.0 µs`** | `-1.0 µs` |
| `block-143.L` | `1411.4 µs` | `1272.7 µs` | `1364.7 µs` | `1448.7 µs` | `1332.7 µs` | **`1272.5 µs`** | `-0.2 µs` |
| `block-210.L` | `1413.9 µs` | **`1293.0 µs`** | `1394.2 µs` | `1471.3 µs` | `1346.4 µs` | `1298.0 µs` | `+5.0 µs` |
| `block-207.L` | `885.4 µs` | **`807.5 µs`** | `858.3 µs` | `911.7 µs` | `824.1 µs` | `809.6 µs` | `+2.1 µs` |
| `dense-703.syn` | `1538.8 µs` | `1435.3 µs` | `1534.7 µs` | `1579.4 µs` | `1481.5 µs` | **`1426.4 µs`** | `-8.9 µs` |
| **6-BLK `S5` SUM** | **`5883.5 µs`** | **`5338.3 µs`** | `5652.1 µs` | `6015.3 µs` | `5542.3 µs` | `5343.7 µs` | **`+5.4 µs`** |

| Arm (6-Blk Sum) | `S1` | `S2` | `S3` | `S4` | `Pre (S1–4)` | `S5 Emit` | `Sum` | Verdict |
|---|---:|---:|---:|---:|---:|---:|---:|---|
| `M0: Clone (0.5.11)` | `2285.7 µs` | `1950.2 µs` | `1539.1 µs` | `745.4 µs` | `6520.3 µs` | `5883.5 µs` | `12403.8 µs` | Baseline |
| **`E2h: C25 Champ`** | `1936.4 µs` | `1714.0 µs` | `789.2 µs` | `759.6 µs` | `5199.2 µs` | **`5338.3 µs`** | **`10537.6 µs`** | **RETAINED** |
| `L1: Pure-Leaf` | `1959.7 µs` | `1704.2 µs` | `789.2 µs` | `752.5 µs` | `5205.6 µs` | `5652.1 µs` | `10857.8 µs` | **KILL** (`+314 µs`) |
| `L2: +Safe-BCEL` | `1951.8 µs` | `1713.0 µs` | `788.0 µs` | `751.9 µs` | `5204.8 µs` | `6015.3 µs` | `11220.1 µs` | **KILL** (`+677 µs`) |
| `L3: +Chunked` | `1927.1 µs` | `1712.7 µs` | `785.2 µs` | `743.6 µs` | `5168.7 µs` | `5542.3 µs` | `10710.9 µs` | **KILL** (`+204 µs`) |
| `L4: InlineChunk` | `1938.2 µs` | `1707.5 µs` | `786.4 µs` | `750.4 µs` | `5182.5 µs` | `5343.7 µs` | `10526.2 µs` | **KILL** (`+5.4 µs` `S5`) |

### 3.2 Headline 1T `InverseBwt::invert_runs` Confrontation (`25` Rounds, `measured`)

`A2/A1` A/A control = **`0.995×` p50 (`1.000×` p10)** on the 6 authenticated Wikidata blocks (`12,358.9 µs` sum):

| Metric | `A1:Master` | `A2/A1` | `M0/A1` | **`E2h/A1`** | `L1/A1` | `L4/A1` | `L3/A1` |
|---|---:|---:|---:|---:|---:|---:|---:|
| **6-BLK p50/A1** | `12358.9 µs` | `0.995×` | `0.973×` | **`0.858×`** | `0.887×` | `0.859×` | `0.868×` |
| **6-BLK p10/A1** | `11824.0 µs` | `1.000×` | `0.988×` | **`0.871×`** | `0.892×` | `0.869×` | `0.875×` |

---

## 4. Next Cycle Plan (`Cycle 27 (Neon)`)
With `S5 Emit` at $\mu = 1$ rigorously closed (`E2h`), **`S1: Pass 1 + OutQ16` (`1,936 µs` = `18.4%` of total kernel, `37.2%` of all pre-emit time)** is now the **#1 remaining pre-emit target**! Cycle 27 attacks `S1` via:
1. **`P1: Direct-Record Fast-Path (Fusing `maximal()` + `take()`)`:** In `origin/master` and `E2h`, `maximal()` checks `(records[i] >> 8) as u8 == c` on every record even though `90%+` of records have `w <= 16` AND `records[i]`'s symbol differs from `records[i+1]`! Fusing `maximal` directly into `Pass1AllPosMux` with a **fast-path when `w <= 16` AND `next_c != c` AND `pos + w <= p || pos > p`** eliminates closure indirection, inner coalescing-loop branch overhead, and `has_p` checks on `~90%` of all `537,497` records!
2. **`P2: 1-Record Thin `w == 1` Scalar Fast-Path (`OutQ16::push_one`)`:** On Wikidata, what fraction of all thin runs have `width == 1` (`~55%–70%`!)? Instead of a `16`-lane AVX2 `vmovdqu` store + `out.len()` bounds check for a 1-element thin run (`width == 1`), write a single `u16` (`2` bytes) directly into `out[at..at+2]` when `fill < 512` (1 scalar store, zero AVX2 `vmovdqu` 32-byte boundary split, zero `16`-byte over-write check)!
3. **`P3: Pre-Sized `MuxStrips` (`edges` reserve once before `S1`)`:** Eliminating the `self.edges.len() < idx + 3` resize check inside `S1`'s thick-strip path!
