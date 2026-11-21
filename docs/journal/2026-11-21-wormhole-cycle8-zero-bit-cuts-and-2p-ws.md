# Wormhole Cycle 8 — Zero-Bit Xor-Mask Cuts, the `256 KiB` 1-Pass Stall Law, and Two-Pass Zero-Copy MLP (`2P-WS8`)

**Date:** 2026-11-21
**Branch:** `research/carbon-wormhole`
**Baseline `origin/master` SHA:** `d425d772c0a0e6b82e9626b36bbfa4b28a6c9613` (`mibz2 0.5.9`)
**Artefacts:**
- Safe-Rust confrontation & $\mathsf{BMCP}$ simulator (`#![forbid(unsafe_code)]`): [`verify_cycle8.rs`](2026-11-21-wormhole-cycle8-zero-bit-cuts-and-2p-ws/verify_cycle8.rs)
- Gate G2 acquisition manifest (`PASS` via `check_manifest.py`): [`ws_2p_mlp.toml`](2026-11-21-wormhole-cycle8-zero-bit-cuts-and-2p-ws/ws_2p_mlp.toml)
- Raw JSON & confrontation output: [`cycle8_results.json`](2026-11-21-wormhole-cycle8-zero-bit-cuts-and-2p-ws/cycle8_results.json), [`cycle8_output.txt`](2026-11-21-wormhole-cycle8-zero-bit-cuts-and-2p-ws/cycle8_output.txt)

---

## 1. Where Cycle 7's Obstructions Forced the Chain

Cycle 7 closed all three Move-D global-order bypasses ($\Phi$/toehold RV bootstrap at `2.20–5.53 MiB` scratch and `2.6–3.8×` Dieted `emit`'s L2 misses; Duval Lyndon factorisation at `34.6–93.8%` single giant factor; BWT-domain SA-IS at `305k–317k` LMS rows = `1.51–3.42×` $m_{16}$). Any route below Dieted `Open`'s serial L2-miss floor (`25,533–94,739` dependent L3 hits) must therefore:
1. Operate **directly on `Open<16>`'s already-wired `edges[0..m)` table** (`351–798 KiB`), which is strictly smaller than every uncollapsed BWT/RV/SA-IS structure **and already 100% resident inside each worker's `2,048 KiB` 10-worker L3 slice**;
2. Fit inside the **`~270 KiB`/worker L3 headroom** left after `out` (`879 KiB`) + `edges` (`351–798 KiB`) + dictionary (`~272 KiB`), ruling out `#675` (`8-lane BP4K`)'s `900 KiB` staging buffer that overflowed 8–10 workers into DDR4 DRAM;
3. Steal **zero bits** from `edges[j]: u32` (preserving full `(code << bits) | next_j` capacity up to $m \le 900{,}000$) and remain **100% exact on multi-cycle and periodic permutations**.

This cycle constructs **Zero-Bit Xor-Mask Cuts** (`Theorem 8.1`), proves and measures the **`256 KiB` 1-Pass Staging Stall Law** vs **Two-Pass Work-Stealing Zero-Copy MLP (`2P-WS8`)** (`Theorem 8.2`), and isolates the exact **Rényi (1953) Maximum-Spacing Straggler & Pre-Scan Obstruction** (`Theorem 8.3` / Break 8) that sets up Cycle 9.

---

## 2. Mathematical Links & Theorems

### Theorem 8.1 (`proved`; `verified` on `14,409` small, multi-cycle, and multi-epoch cases + `6/6` authenticated blocks) — Zero-Bit Xor-Mask Cut Decomposition & Multi-Cycle Filtering

Let $m \ge 2$, $b = \lceil \log_2 m \rceil$, $k \ge 1$, $s = \max(0, b - k)$, and $\mathrm{cm} = 2^s - 1$. Define the cut predicate and cut set:
$$P_{\mathrm{cut}}(j) \;\Longleftrightarrow\; ((j \oplus \mathrm{mark}) \mathbin{\&} \mathrm{cm}) = 0, \qquad S_{\mathrm{cut}} = \{j \in [0, m) : P_{\mathrm{cut}}(j)\}.$$

1. **Zero-bit anchor & cardinality (`proved`):** Because $(\mathrm{mark} \oplus \mathrm{mark}) \mathbin{\&} \mathrm{cm} = 0$, $\mathrm{mark} \in S_{\mathrm{cut}}$ unconditionally with **zero tag bits** in `edges[j]`. Every cut row has the exact arithmetic progression form $c_q = (q \ll s) \mathbin{|} (\mathrm{mark} \mathbin{\&} \mathrm{cm})$ with cut index $q = j \gg s \in [0, S)$, where $S = |S_{\mathrm{cut}}| \in [2^{k-1}+1,\, 2^k]$ whenever $b \ge k$.
2. **Induced first-return bijection & exact multi-cycle filtering (`proved`):** Because the critical successor map $\pi(j) = \mathrm{edges}[j] \mathbin{\&} (2^b - 1)$ is a **bijection** on $[0, m)$ (a disjoint union of finite cycles), every orbit starting at $c_q \in S_{\mathrm{cut}}$ lies on a finite cycle containing $c_q$ itself and therefore returns to $S_{\mathrm{cut}}$ in $t_q \ge 1$ steps, inducing a well-defined first-return bijection $\widehat{\pi} : S_{\mathrm{cut}} \to S_{\mathrm{cut}}$ with $\sum_{q=0}^{S-1} t_q \le m$ (equality iff every cycle of $\pi$ meets $S_{\mathrm{cut}}$). Walking $\widehat{\pi}$ from $q_0 = \mathrm{mark} \gg s$ until returning to $q_0$ (or reaching byte prefix $\ge n$ on periodic texts) visits **exclusively** the cuts on $\mathrm{mark}$'s cycle in exact text order and discards all phantom cycles of multi-cycle BWT columns in $O(S)$ stack operations and `0 B` heap.

*Verification:* Verified byte-exact across **`14,409`** exhaustive binary ($n \le 9$), ternary ($n \le 6$), and 200 multi-cycle/multi-epoch stress cases ($n \in [2{,}500,\, 178{,}000]$, including random non-primitive multi-cycle columns `L` with $8\text{–}15$ cycles) plus **`6/6` authenticated Wikidata blocks** against `origin/master` `OpenCells`.

---

### Theorem 8.2 (`proved` + `measured` via [`verify_cycle8.rs`](2026-11-21-wormhole-cycle8-zero-bit-cuts-and-2p-ws/verify_cycle8.rs)) — The L3-Safe 1-Pass Staging Stall Law vs Two-Pass Zero-Copy MLP (`2P-WS8`)

Fix per-worker L3 capacity $M_3/P = 2{,}048\text{ KiB}$ ($P = 10$ workers on `20 MiB` shared L3), leaving staging budget $B_{\mathrm{stage}} \le 256\text{ KiB}$ once `out` (`879 KiB`), `edges` (`351–798 KiB`), and dictionary (`~250 KiB`) are allocated.

1. **1-Pass Bounded-Staging Stall Law (`proved` + `measured`):** Any single-pass $\mu$-lane scheme (`1P-Stage256K`) that emits Lane 0 directly at `out[0..]` while $\mu - 1$ auxiliary lanes buffer non-initial cuts into $B_{\mathrm{stage}} = 256\text{ KiB}$ (`per_slot` $\approx B_{\mathrm{stage}}/S$) stalls every auxiliary cut $c_q$ whose output segment exceeds `per_slot` until Lane 0 reaches $c_q$ in text order. Because the induced cycle order $\widehat{\pi}$ on $S_{\mathrm{cut}}$ is a pseudorandom permutation of $[0, S)$ and mean segment byte length is $n/S \approx 56\text{ KiB} \gg 16\text{ KiB}$, auxiliary lanes fill $B_{\mathrm{stage}}$ within the first $\approx B_{\mathrm{stage}}/n \approx 28.4\%$ of bytes (**measured active auxiliary step coverage: `6.03%–13.04%` of $m$** across the 6 blocks; **`86.96%–93.97%` of the walk stalls back to $\mu = 1$**).
2. **Two-Pass Zero-Copy L3-Resident MLP (`proved` + `measured`):** `2P-WS8` (`0 B` heap staging, `512 B` stack) executes:
   - **Pass 4a (Length-Only Cut Scout):** $\mu = 8$ work-stealing register lanes walk `edges[0..m)` + L1d-resident `word_len[code]` (`u16`, `2.6–7.0 KiB`), recording `(seg_len[q], seg_next[q])` on stack;
   - **Prefix Sum ($O(S)$ stack):** Accumulates exact `[cyc_start[s], cyc_end[s])` along $\mathrm{mark}$'s cycle;
   - **Pass 4b (Zero-Copy In-Place Emit):** $\mu = 8$ work-stealing lanes emit directly into disjoint slices of `out[0..n)` with **zero staging copies**.
   Because `edges[0..m)` (`351–798 KiB`) fits **100% inside the worker's `2,048 KiB` L3 slice**, Pass 4a + Pass 4b incur **zero additional `2,048 KiB` L3 misses** (`479–831` misses/block, identical to $\mu=1$ `DietedOpen`!) while cutting critical-lane `dep_past_l2` by **`40.6%–66.2%` (`0.34–0.59×`)**.

---

### Theorem 8.3 (`proved` + `measured`) — Rényi Maximum-Spacing Straggler Law & Break 8

Under $S$ uniform cut points on a cycle of length $m$, let $\Delta_{(S)} = \max_{1 \le i \le S} \Delta_i$ be the longest cut-to-cut segment. By **Rényi's (1953) Order-Statistic Representation** for uniform spacings on the circle,
$$\mathbb{E}[\Delta_{(S)}] \;=\; \frac{m}{S}\,H_S \;=\; \frac{m}{S}\sum_{j=1}^S \frac{1}{j}.$$
Even under **zero-overhead work-stealing** across $\mu$ lanes, serial round depth satisfies the dual lower bound:
$$D_{\mathrm{serial}}(\mu, S) \;\ge\; \max\!\left(\left\lceil \frac{m}{\mu} \right\rceil,\; \Delta_{(S)}\right).$$
- At $k = 5$ ($S \in [17, 27]$ on the 6 blocks), $\frac{H_S}{S} \in [0.144,\, 0.202] > \frac{1}{8} = 0.125$: **a single straggler cut segment exceeds $m/8$**, forcing `rounds_4a` and `rounds_4b` up to **`1.59–2.27×` $m/8$** (e.g., on `block-210.L` at $S = 25$, `rounds_4a = 57,913` vs $m/8 = 25{,}535$!).
- Eliminating straggler dominance at $\mu = 8$ requires $\frac{H_S}{S} \ll \frac{1}{8}$, i.e., **$k = 7$ ($S \in [65, 128]$, where $\frac{H_{96}}{96} = 0.0535 < 0.43 \cdot \frac{1}{8}$)**.

---

## 3. Six-Dimensional $\mathsf{BMCP}$ Cost Ledger & Safe-Rust Confrontation (`measured`)

All measurements come from [`verify_cycle8.rs`](2026-11-21-wormhole-cycle8-zero-bit-cuts-and-2p-ws/verify_cycle8.rs) (`#![forbid(unsafe_code)]`, 21 paired interleaved rounds per block against compiled `origin/master` `mibz2::bwt::OpenCells`, plus exact `FastLru` `256 KiB` L2 / `2,048 KiB` L3 simulation from the identical end-of-Phase-3 cache state as Cycle 6).

### 3.1 Separated Cost Table (`DietedOpen` $\mu=1$ UPPER vs `2P-WS8` $\mu=8, S \le 32$)

| Block | $m_{16}$ | $S$ (`n_cyc`) | `1P-256K` Aux Cov | $Q_{\mathrm{rand}}$ (`Diet` $\to$ `WS8`) | $D_{\mathrm{serial}}$ (`Diet` $\to$ `WS8` `4a+4b`) | $D_{\mathrm{past-}M_2}^{\mathrm{crit}}$ (`Diet` $\to$ `WS8`, ratio) | $C_{\mathrm{miss}}(256\text{K})$ (`Diet` $\to$ `WS8`) | $C_{\mathrm{miss}}(2048\text{K})$ (`Diet` $\to$ `WS8`) |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| `block-0.L` | 89,874 | 22 (22) | `6.89%` | `89,874` $\to$ `179,748` | `89,874` $\to$ `35,914` (`0.40×`) | `25,533` $\to$ **`8,932` (`0.35×`)** | `41,196` $\to$ `64,327` | `809` $\to$ **`809` (`1.00×`)** |
| `block-2.L` | 112,070 | 27 (27) | `6.03%` | `112,070` $\to$ `224,140` | `112,070` $\to$ `42,047` (`0.38×`) | `37,778` $\to$ **`12,781` (`0.34×`)** | `53,619` $\to$ `93,440` | `746` $\to$ **`746` (`1.00×`)** |
| `block-85.L` | 156,404 | 19 (19) | `10.41%` | `156,404` $\to$ `312,808` | `156,404` $\to$ `71,676` (`0.46×`) | `47,553` $\to$ **`22,458` (`0.47×`)** | `62,849` $\to$ `120,227` | `479` $\to$ **`479` (`1.00×`)** |
| `block-143.L` | 196,841 | 24 (24) | `10.28%` | `196,841` $\to$ `393,682` | `196,841` $\to$ `88,572` (`0.45×`) | `94,739` $\to$ **`40,685` (`0.43×`)** | `111,648` $\to$ `235,736` | `831` $\to$ **`831` (`1.00×`)** |
| `block-210.L` | 204,281 | 25 (25) | `6.34%` | `204,281` $\to$ `408,562` | `204,281` $\to$ `114,575` (`0.56×`) | `89,106` $\to$ **`52,920` (`0.59×`)** | `105,368` $\to$ `234,925` | `563` $\to$ **`563` (`1.00×`)** |
| `block-207.L` | 135,921 | 17 (17) | `13.04%` | `135,921` $\to$ `271,842` | `135,921` $\to$ `59,542` (`0.44×`) | `51,240` $\to$ **`21,042` (`0.41×`)** | `67,671` $\to$ `135,527` | `815` $\to$ **`815` (`1.00×`)** |

*Preprocessing & Parallel Work (separated):*
- **Preprocessing ($W_{\mathrm{prep}}, D_{\mathrm{prep}}$):** Identical to `DietedOpen` except populating `word_len[code]: u16` alongside `inline16[code]` in Phase 2 (`+1` `u16` push per distinct word = `1,064–3,224` pushes, `< 5 µs`) and $S \le 32$ stack prefix sums (`< 0.1 µs`).
- **Parallel work ($W_{\mathrm{par}}$):** $W_{\mathrm{4a}} = m$ integer additions + $W_{\mathrm{4b}} = n$ emitted bytes (`0` staging `memcpy` bytes).

### 3.2 Paired Safe-Rust Timing Breakdown (`21` Interleaved Rounds vs `origin/master` `OpenCells`)

*(Note: `Master` `OpenCells` uses `unsafe` unchecked pointer/slice indexing in `src/open.rs`, whereas both `Diet_mu1` and `2P-WS8` compile under `#![forbid(unsafe_code)]` with safe bounds checks.)*

| Block | `Master` (`unsafe` 0.5.9) | `Diet_mu1` (safe, $\mu=1$) | `2P-WS8` (safe, $\mu=8$) | Shared Build (Phases 1–3) | `Diet_mu1` `emit` (implied) | `2P-WS8` Pass 4b (`emit` only) | Pass 4b vs `Diet` `emit` | `2P-WS8` Pass 4a (`scout` tax) | Net `2P-WS8` vs `Diet_mu1` |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| `block-0.L` | `1,288.6 µs` | `1,467.1 µs` | `1,597.5 µs` | `932.5 µs` | `534.6 µs` | **`487.5 µs`** | **`-8.8%` (`-47 µs`)** | `+177.5 µs` | `+130.4 µs` (`1.089×`) |
| `block-2.L` | `1,551.9 µs` | `1,743.2 µs` | `1,842.6 µs` | `1,074.7 µs` | `668.5 µs` | **`555.2 µs`** | **`-16.9%` (`-113 µs`)** | `+212.7 µs` | `+99.4 µs` (`1.057×`) |
| `block-85.L` | `1,593.2 µs` | `1,697.9 µs` | `1,815.0 µs` | `792.0 µs` | `905.9 µs` | **`702.6 µs`** | **`-22.4%` (`-203 µs`)** | `+320.4 µs` | `+117.1 µs` (`1.069×`) |
| `block-143.L` | `2,797.8 µs` | `3,066.2 µs` | **`3,057.6 µs`** | `1,737.9 µs` | `1,328.3 µs` | **`892.7 µs`** | **`-32.8%` (`-436 µs`)** | `+427.0 µs` | **`-8.6 µs` (`0.997×`)** |
| `block-210.L` | `2,462.7 µs` | `2,636.6 µs` | `2,810.3 µs` | `1,306.7 µs` | `1,329.9 µs` | **`954.8 µs`** | **`-28.2%` (`-375 µs`)** | `+548.8 µs` | `+173.7 µs` (`1.066×`) |
| `block-207.L` | `1,859.0 µs` | `2,064.3 µs` | `2,143.1 µs` | `1,208.9 µs` | `855.4 µs` | **`619.6 µs`** | **`-27.6%` (`-236 µs`)** | `+314.6 µs` | `+78.8 µs` (`1.038×`) |

### 3.3 Mandatory Production Metrics for `2P-WS8`
1. **Per-worker working set ($S_{\mathrm{mem}}$):** **`1,051.1 KiB`** persistent scratch (`+7.0 KiB` `word_len` over `Diet_mu1`, **`0 B` staging heap**) + **`878.9 KiB`** `out` = **`1,930.0 KiB` (`1.885 MiB`)/worker**. Across `7–10` workers on the i9-10900K's `20 MiB` shared L3: `7w = 13.2 MiB`, **`10w = 18.85 MiB <= 20.0 MiB` L3 (`100%` L3-resident!)**.
2. **DRAM bytes per output byte:** **`0.034–0.059 B/B`** (`479–831` misses past `2,048 KiB` L3 slice $\times$ `64 B` / `900,000 B`, **zero increase** over `DietedOpen` $\mu=1$, plus compulsory cold input/output writeback).
3. **Acquisition cost:** Validated via `check_manifest.py` on [`ws_2p_mlp.toml`](2026-11-21-wormhole-cycle8-zero-bit-cuts-and-2p-ws/ws_2p_mlp.toml) (`PASS`): zero extra heap structures (`M = 1` stack cut table), $D = m/8$ scout + $D = m/8$ in-place emit.

---

## 4. Break 8 Diagnosis $\longrightarrow$ Exact Target for Cycle 9

Look at what Table 3.2 proves:
- **Zero-copy 8-lane in-place emission (`Pass 4b`) ALREADY crushes 1-lane `emit_serial` on `6/6` blocks by `-8.8%` to `-32.8%` (`-47 µs` to `-436 µs`)** and **already beats `Diet_mu1` end-to-end on `block-143.L`** (`m = 196,841`, the highest `dep_past_l2` block)!
- Three exact, removable overheads ate `+79..174 µs` on the remaining 5 blocks:
  1. **Rényi Straggler Inflation ($S = 17..27$):** At `log2_cuts = 5`, $\mathbb{E}[\Delta_{(S)}] = \frac{m}{S}H_S \approx 0.18\,m > m/8$, inflating `rounds_4a/4b` up to `2.27×` $m/8$ (`57,913` on `block-210.L`). Raising $k$ from `5` to **`7` ($S \in [65, 128]$, `1.5 KiB` stack, still `0 B` heap!)** cuts $\frac{H_S}{S}$ to `0.053`, collapsing stragglers toward $m/8$.
  2. **Unoverlapped `Pass 4a` Walk (`177–549 µs`):** `2P-WS8` walks 100% of $m$ edges in `Pass 4a` before writing a single output byte — even though **`mark`'s cut (`cyc_cuts[0]`) and every contiguous discovered prefix `mark -> c_{q_1} -> c_{q_2} ...` ALREADY know their exact `out` offset during scouting**!
  3. **Per-Step Boundary Branching in `Pass 4b`:** `Pass 4b` checked `at + INLINE <= end_at` on every single step (`8` branches/round) to avoid overwriting the first `1..15` bytes of the next cut segment `out[end_at..end_at+15]`. Writing full-speed sloppy `16 B` stores `out[at..at+16]` unconditionally on every inline step (`0` boundary branches!) and restoring the `16 B` head of each of the $S \le 128$ segments at the end of `Pass 4b` (`< 0.3 µs` total!) removes all inner-loop boundary checks.

Cycle 9 combines all three into **`Relay-MLP` (`Emit-While-Scouting + S=65..128 Rényi De-Straggling + Post-Slop Boundary Restoration + Packed Length-in-Edge / L1d Dictionary`)**.
