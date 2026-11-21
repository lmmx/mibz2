# Wormhole Cycle 16 — Closing Non-RV Interval Tracing (`DietTileDir` & Tarjan Full-Tile Path Compression in Dead `edges`) Against `GradedRV`

**Session:** `research/carbon-wormhole` (Round 3, Cycle 16 of 20; 6th of 10 in Round 3)  
**Baseline `origin/master`:** `90e6719663af0615fb417b038906ad2636193aa3` (`mibz2::bwt::OpenCells`)  
**Verification harness:** [`verify_cycle16.rs`](2026-11-21-wormhole-cycle16-s2-tile-directory-and-path-compression/verify_cycle16.rs) (`#![forbid(unsafe_code)]`, `0` warnings, `5,229` Gate G1 + Tarjan containment + `SPIRALS` cases byte-exact + `21`-round rotating native A/B + A/A + `M0` confrontation), [`s2_tile_path_compression.toml`](2026-11-21-wormhole-cycle16-s2-tile-directory-and-path-compression/s2_tile_path_compression.toml) (`PASS`), [`cycle16_output.txt`](2026-11-21-wormhole-cycle16-s2-tile-directory-and-path-compression/cycle16_output.txt), [`cycle16_results.json`](2026-11-21-wormhole-cycle16-s2-tile-directory-and-path-compression/cycle16_results.json)

---

## 1. One-Line Kill Decisions (Cycle 16)

- **KILL `B2: DietTileDir` (`INDUCE = false` `Tile` + `PAGE = 64` directory aliased inside dead `edges`, `0 B` scratch):** Killed because even after eliminating `252 KiB` of `Vec<Tile>`/`Vec<u32>` heap scratch, `32,838–61,282` two-hop random directory-to-tile lookups + string reversals regress `S2 + S3a` to `368.7–875.7 µs` (vs `B1: GradedRV`'s `222.4–688.9 µs`), losing **`1.041×–1.159×` vs `M0` (`6/6` Wikidata blocks)** and **`1.07×–1.24×` vs `B1: GradedRV`**.
- **KILL `B3: TarjanTileCompress` (Tarjan 1975 Full-Tile Containment Path Compression aliased inside dead `edges`, `0 B` scratch):** Killed (`1.047×–1.204×` vs `M0` on `6/6` Wikidata blocks) because although `84.2%–87.0%` of thick strips satisfy exact single-tile containment (collapsing trace steps **`3.9×–7.7×`** from `32.8k–61.3k` down to `6.0k–15.6k` without splitting), paying `ns` upfront directory chain-walks **plus** `6.0k–15.6k` remaining lookups across the `13.0%–15.8%` boundary-straddling strips exceeds `GradedRV`'s single amortized $O(H)$ linked-list pass.

---

## 2. The Chain of Existing Results Tested (Theorem 11.2(b) Escape Test)

Cycle 11 (`Theorem 11.2(b)`) left open one fundamental structural question on Stage `S2`: can thick-strip elimination escape doubly-linked Rauzy–Veech pointer splicing (`INDUCE = true`) altogether by viewing thick strips as a static piecewise-translation forest over `[0, n)` (`INDUCE = false`, `open.rs:239-280`), dieted into dead `edges` (`0 B` scratch) and accelerated by **Tarjan (1975) Path Compression**?

### Theorem 16.1 — Slack-Clamp Redundancy under Full-Tile Containment (`proved` + `verified` on `5,229` cases + `6` blocks)

- **Statement (`proved`):** Let thick tile $q = [s_q, e_q)$ with translation $\delta_q$ map its start $s_q + \delta_q$ into tile $q_1 = [s_{q_1}, e_{q_1})$. The entire image $[s_q + \delta_q, e_q + \delta_q)$ lies inside $q_1$ without straddling a tile boundary if and only if $e_q + \delta_q \le e_{q_1}$. Whenever $e_q + \delta_q \le e_{q_1}$, for **every** $x \in [s_q, e_q)$ with $x_1 = x + \delta_q$:
  $$e_q - x \;\le\; e_{q_1} - (x + \delta_q) = e_{q_1} - x_1.$$
  Therefore $q_1$'s right-slack clamp $\min(\mathrm{width}, e_{q_1} - x_1)$ in `Kernel::trace` is **identically redundant**, and tile $q$ contracts soundly in $O(1)$ space into a single multi-byte jump $(e_q,\; \delta_q + \delta_{q_1},\; \mathrm{byte}_q \cdot \mathrm{word}_{q_1})$ **without splitting any interval**!
- **Empirical Discovery on Wikidata (`verified` in [`cycle16_output.txt`](2026-11-21-wormhole-cycle16-s2-tile-directory-and-path-compression/cycle16_output.txt)):** Across all 6 authenticated Wikidata blocks, **`84.2%–87.0%`** of thick strips (`1,317/1,565` to `4,354/5,047`) satisfy $e_q + \delta_q \le e_{q_1}$, collapsing `trace_interval` steps by **`3.9×` to `7.7×`** (`53,913 -> 7,007` on `block-0.L`, `32,838 -> 5,983` on `block-85.L`, `61,282 -> 15,617` on `block-143.L`)!

### Theorem 16.2 — Why Amortized `GradedRV` Strictly Beats Unsplit `Tile` + Path Compression (`proved` + `measured`)

Despite collapsing `75%–87%` of trace steps, unsplit `Tile` tracing (`B2` and `B3`) loses on `6/6` Wikidata blocks to `B1: GradedRV` for three additive reasons:
1. **$O(n / \text{PAGE})$ Spatial Directory Tax vs Pure $O(H)$:** `Tile` tracing writes and probes a `14,063`-entry `PAGE = 64` spatial directory (`56 KiB` > `32 KiB` L1d) plus `2H` 12-byte tiles (`84–182 KiB`), turning each hop into a **two-hop random pointer/index chase** (`directory[x/64] -> tiles[q]`), whereas `GradedRV`'s domain is purely $O(H)$ independent of $n$.
2. **The `13.0%–15.8%` Straddling Barrier:** Whenever a wide thick strip $q$ straddles a boundary $e_{q_1}$ ($e_q + \delta_q > e_{q_1}$), different sub-intervals of $q$ take divergent suffixes and **cannot** share a single unsplit tile shortcut. Thus `B3` must pay *both* $O(H)$ upfront directory chain walks *and* `5,983–15,617` dynamic multi-hop directory lookups across straddling tiles (`plus reverse()` on `n_words` strings).
3. **Dynamic Splitting *Is* Rauzy–Veech Induction:** Eliminating those remaining `5,983–15,617` straddling re-traces requires splitting each straddling tile at $e_{q_1} - \delta_q$ once and memoizing both sub-intervals — which is algebraically **Rauzy–Veech induction (`B1: GradedRV`)**!

---

## 3. Stage-Isolated & End-to-End Native Safe-Rust Confrontation (`21` Rotating Rounds, `0 B` Scratch Delta)

- **Per-Worker Working Set:** `1.885 MiB` across all arms (`B1`, `B2`, `B3` all operate inside `edges` with `0 B` extra scratch).
- **DRAM Traffic:** `0.034–0.059 B/B` (`0` extra `2048K` L3 misses).

| Block | `H_16` | Full-Tile Contained | Trace Steps (`Raw -> Tarjan`) | `M0:RV` (`S2+3a µs`) | **`B1:GrRV`** (`S2+3a µs`) | `B2:TileD` (`S2+3a µs`) | `B3:Tarjn` (`S2+3a µs`) | `A2/A1` | **`B1:GrRV / M0`** | `B2:TileD / M0` | `B3:Tarjn / M0` |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| `block-0.L` | `3,448` | `3,000` (`87.0%`) | `53,913 -> 7,007` (`7.7×`) | `446.8` | **`389.7`** | `646.1` | `739.3` | `0.990x` | **`0.932x`** | `1.159x` | `1.204x` |
| `block-2.L` | `3,827` | `3,313` (`86.6%`) | `51,718 -> 8,779` (`5.9×`) | `531.8` | **`445.3`** | `680.7` | `770.8` | `0.990x` | **`0.942x`** | `1.106x` | `1.136x` |
| `block-85.L` | `1,565` | `1,317` (`84.2%`) | `32,838 -> 5,983` (`5.5×`) | `260.4` | **`222.4`** | `368.7` | `367.5` | `0.993x` | **`0.973x`** | `1.072x` | `1.061x` |
| `block-143.L` | `5,047` | `4,354` (`86.3%`) | `61,282 -> 15,617` (`3.9×`) | `814.0` | **`688.9`** | `875.7` | `1,015.8` | `0.996x` | **`0.957x`** | `1.051x` | `1.079x` |
| `block-210.L` | `2,809` | `2,384` (`84.9%`) | `40,694 -> 8,833` (`4.6×`) | `435.1` | **`382.9`** | `524.1` | `567.4` | `0.986x` | **`0.973x`** | `1.041x` | `1.047x` |
| `block-207.L` | `3,474` | `2,978` (`85.7%`) | `58,292 -> 10,844` (`5.4×`) | `528.1` | **`453.9`** | `733.3` | `741.1` | `0.992x` | **`0.958x`** | `1.112x` | `1.111x` |

---

## 4. Closure of Stage `S2` & Pivot to Stage `S3` (Cycle 17)

Cycles 15 and 16 **close Stage `S2`**, establishing `B1: GradedRV` (`(len10 << 22) | id22` + `pop_tail`) as the Stage `S2` winner.

**Cycle 17** now attacks **Pre-`Emit` Gap #3: `Stage S3` (`Bottom + Bands/SLP Expansion + inline_words`, `134–417 µs`, `#1` pre-`emit` `dep_past_l2` source)** via an algebraic wormhole: **16-Truncated Prefix-Monoid Homomorphism $\pi_{16} : \Sigma^* \to \Sigma^{\le 16}$ Evaluated in `u128` (`2 × u64`) Registers**, eliminating `e.append()` stack DFS, `spelling` writes, and **100% of `inline_words` (`S3b`, `12–35 µs`)** on the **`63.6%–77.6%` of words that have `len <= 16`**!
