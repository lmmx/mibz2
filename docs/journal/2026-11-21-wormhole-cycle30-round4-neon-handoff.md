# Wormhole (Neon) Cycle 30 — Full Multi-Worker Qualification, Whole-Kernel $\mathsf{BMCP}_{\mathrm{CML}}$ Closure & 10-Cycle Round 4 Handoff (`WormholeNeonCells`)

**Session:** `research/carbon-wormhole` (Round 4 — Gemini Neon, Cycle 30 of 30; 10th and final cycle of Round 4)  
**Baseline `origin/master`:** `8ff1187af6a920f7d57fd338680db3a5633a3ac5` (`v0.5.11`; `src/` byte-identical to `90e671966`)  
**Verification harness:** [`verify_cycle30.rs`](2026-11-21-wormhole-cycle30-round4-neon-handoff/verify_cycle30.rs) (`#![forbid(unsafe_code)]`, `0` warnings, `6/6` `manifest.json` SHA-256 auth + `14,473` exhaustive/stress/spiral/boundary/wide/dense/column cases + `21`-block multi-regime `BzReader::parallel_workers_with::<WormholeNeonCells>` at `W = 1, 2, 4, 8` + `25`-round `1w` & `2w` barrier-synchronized confrontation with paired `95%` CIs), [`wormhole_neon_final.toml`](2026-11-21-wormhole-cycle30-round4-neon-handoff/wormhole_neon_final.toml) (`PASS`), [`cycle30_output.txt`](2026-11-21-wormhole-cycle30-round4-neon-handoff/cycle30_output.txt), [`cycle30_results.json`](2026-11-21-wormhole-cycle30-round4-neon-handoff/cycle30_results.json)

---

## 1. Executive Summary of All Three Round 4 Charges (`Cycles 21–30`)

1. **Charge #3 — Independent Audit of `Neon-C29` (`Cycle 21`):**
   - Reproduced `Neon-C29`'s 1-thread claim (`0.850×` `M0`, `0.829×` `A1` on Cycle 21's host; `950.0 KiB` = `-19.0%` persistent scratch excl. `879 KiB` `out`).
   - Proved **`60.6%` (`-1,113 µs` of `-1,843 µs`)** of `Neon-C29`'s claimed 1T win came from **`Fused-XOR`** ($\mu = 2$ walk + `+879 µs` random-scatter `wire_xor` in `S4`, already rejected natively at 4W `1.021×` in `#718`).
   - The genuinely transferable $\mu = 1$ construction stack `S*` (`Operad-MN16 + L1D Dedup + MuxStrips/Band64 + L4 BidiBottom + All+Pos`) delivers **`0.922–0.925×` of `A1:Master` at `-19.0%` scratch (`950.0 KiB`)**, cutting `S3` `-45%` and `S1` `-15%`.
2. **Charge #1 — Post-Stack `S*` Whole-Kernel $\mathsf{BMCP}_{\mathrm{CML}}$ Bounds (`Cycle 22`, extended through `Cycles 23–28`):**
   - Instrumented all 5 stages of `M0` vs `S*` in our exact Comet Lake simulator (`32 KiB` L1d, `256 KiB` L2, `2048 KiB` per-worker L3 slice, `64`-entry L1 DTLB, `32`-page `4 KiB` L2 streamer) keeping all **6 costs separate**.
   - Ranked the post-`S*` gaps: **`#1 S5 Emit`** (`51.7%` of `S*`, `90.2%` of unprefetched `L2m_eff`, `97.3%` of L1 DTLB misses `430.5k`), **`#2 S2 RV Elim`** (`336,297` random rule loads `S2_RL`, `-71.5%` reducible via `GradedOperad`), **`#3 S1 Pass 1`** (`17.5%`), **`#4 S3 Dedup+Band64`** (`7.5%`), **`#5 S4 Wire`** (`6.9%`, `82.2%` streamer-hidden).
3. **Charge #2 — Attacking Every Remaining Stage Gap (`Cycles 22–29`):**
   - Synthesized **`WormholeNeonCells` (`R4`)** — a pure **$\mu = 1$ single-stream** `#![forbid(unsafe_code)]` kernel adding **`5` promoted zero-scratch links** on top of `S*`, cutting another **`-5.2%` (`1w: 0.9476 [0.9417, 0.9566]`, `2w: 0.9496 [0.9434, 0.9561]`)** beyond `S*` to reach **`0.8761 [0.8718, 0.8829]` at `1w` and `0.8784 [0.8691, 0.8869]` at `2w` vs `origin/master` (`8ff1187af`)** (`0.8668× / 0.8724×` vs `M0`), **outperforming `Full Neon-C29 (Fused-XOR)` at BOTH `1w` (`0.876×` vs `0.888×`) AND `2w` (`0.878×` vs `0.883×`)** with **zero** `#718` 4W risk and **`950.0 KiB` (`-19.0%`)** persistent scratch (`10` i9 workers = **`17.86 MiB <= 20 MiB` L3**)!

---

## 2. Complete Round 4 One-Line Keep & Kill Ledger (`Cycles 21–30`)

### Promoted into `WormholeNeonCells` (`R4`, on top of Post-Stack `S*`, `0 B` Extra Scratch)
- **PROMOTE `F4: emit_no_off_nospill` (`Cycle 22`, `-235` to `-367 µs` `S5`):** Hoisting `extra.is_empty()`, slicing `&mut out[..n + INLINE]`, and outlining `#[cold] #[inline(never)] emit_long_no_off` cures Cycle 21's LLVM `28(%rsp)` `mask` spill and cuts `S5` by `-6.4%` at `0 B` scratch.
- **PROMOTE `G2: GradedOperad` (`Cycle 23`, `-115` to `-126 µs` `S2` + `-34` to `-49 µs` `S3`):** Fusing `#714`'s `(len10 << 22) | id22` handle onto `Operad-MN16` eliminates **`71.5%` (`240,381` of `336,297`)** of random `S2` rule loads on `tot >= 16` joins and removes `len_operad()` from `S3` (`S2+S3 = -26.5%` vs `M0`).
- **PROMOTE `E1: Loop-Inversion Early-j emit_zerospill` (`Cycle 24`, `-58` to `-118 µs` `S5`, `7/7` blocks):** Moving `emit_long_no_off` out of the hot inner loop and unifying `dict_len = lens.len().min(inline16.len())` deletes the second dictionary bounds check and schedules `j = edge & mask` *before* `lens[code]` and `inline16[code]`.
- **PROMOTE `E2h: Hoisted-S4 Edge-Packed GPR Len4` (`Cycles 24–25`, `-36 µs` `S5` on `3/6` eligible blocks, `0 µs` `S4`):** Packing `len4 = if l <= 15 { l } else { 0 }` into `edge >> 28` whenever `bits + code_bits <= 28` deletes `lens[code]` from the hot loop in `100%` GPR arithmetic (escaping Kitchen Sink Cycle 27's XMM/GPR stall).
- **PROMOTE `P2c: CleanPt64` (`Cycles 27 & 29`, `R4/R3 = 0.9850 [0.9794, 0.9998]` over 35 rounds, `-1 KiB` stack):** Packing `pt[a] = (thin[a] << 32) | pos[a]` updates both `pos[a]` and `thin[a]` in one 64-bit add+store inside the single `maximal()` loop, winning `6/6` Wikidata blocks + `-47 µs` on `dense-703.syn`.

### Killed Routes (`15` Honest Kills Across `Cycles 21–28`, One Line Each)
- **KILL `Fused-XOR` (`Cycle 21` & `#718`):** Pays `+879 µs` random-scatter RMWs in `S4`, regresses `1.021×` natively at 4W (`#718`), and is now beaten even at `1w`/`2w` by pure-$\mu=1$ `WormholeNeonCells`.
- **KILL `G1: Explicit Narrow::set_links` (`Cycle 23`):** `+0.8 µs` in `S2`; LLVM already coalesces adjacent `set_prev`/`set_next` into one 32-bit store.
- **KILL `G3: Tagless Tiered GradedOperad u64/u128` (`Cycle 23`):** `+60.4 µs` slower than `G2` in `S2` due to 3 unpredictable `<= 8` length-boundary branches.
- **KILL `E2b: Edge-Packed GPR Len3` (`Cycle 24`):** `+82.7 µs` on `block-143/210/207` from unpredictable inner-loop exits on `len 8..=16`.
- **KILL `E3: Zipf Frequency-Ordered inline16 Remap` (`Cycle 24`):** `-34.2 µs` in `S5` cannot pay for `+187.6 µs` `S3` sorting & permutation (`5.5×` negative ROI).
- **KILL `Uniform Lower LIMIT = 8, 10, 12, 14` (`Cycle 25`):** `+104` to `+1,330 µs` slower (`1.49–2.50×` negative ROI across `S1–S4`); `LIMIT = 16` remains globally optimal post-`GradedOperad`.
- **KILL `Uniform Higher LIMIT = 20` (`Cycle 25`):** `+25.4 µs` slower and `+84.6 KiB` scratch (`1,034.6 KiB`).
- **KILL `SymAd (10/16)` & `BlkAd (12/16)` Adaptive Schur Thresholds (`Cycle 25`):** `+696 µs` and `+205 µs` slower; `LIMIT = 16` wins on every Wikidata block.
- **KILL `L1/L3: Out-of-Line Pure-Leaf emit_short_run` (`Cycle 26`):** `+314` / `+204 µs` in `S5` from `53,723` `LONG`-exit function calls (`1` every `16.7` steps).
- **KILL `L2: Safe Power-of-2 Edges/Dict BCEL` (`Cycle 26`):** `+677 µs` in `S5` from zero-filling up to `240 KiB` of `edges` tail right before `S5` (L2 eviction).
- **KILL `L4: Inline Chunked Safe-Budget` (`Cycle 26`):** `+5.4 µs` neutral in `S5`, proving `E2h`'s early-`j` scheduling already hides `100%` of non-critical ALU/bounds ops behind `edges[j]` L2/L3/STLB misses (closing `S5` at $\mu=1$ along with Theorems 26.1 & 26.2).
- **KILL `P1/P2: Fused Phase-1/Phase-2 Pass1` (`Cycle 27`):** `+142` / `+151 µs` in `S1`; duplicating `QueuesPos::push` trips Cycle 19's register-spill cliff.
- **KILL `P3: w == 1 Scalar Fast-Path` & `P4: Inline Chunk-Tail f > 496` (`Cycle 27`):** `+1,846 µs` (`2×` slower) and `+1,083 µs` (`+57%` slower) in `S1`, proving Theorem 27.2 (Single-Path AVX2 Inlining Law: any second write path breaks LLVM's `vpaddw + vmovdqu %ymm0` schedule).
- **KILL `D1/D2: 2-Way Set-Associative 4K/8K str_ht` (`Cycle 28`):** `+30.8` / `+22.1 µs` in `S3` (`+62` / `+83 µs` on `dense-703.syn`); Theorem 28.1 proved 1-way `4K` `str_ht` already achieves `99.983%` of exact `HashSet` dedup (`2 / 7,221` dups on Wikidata, `0 / 4,370` on `dense-703.syn`).
- **KILL `D3: S3 h_len == 1 Literal-Pair Fast-Path` (`Cycle 28`):** `+27.4 µs` in `S3` (`+74 µs` dense).

---

## 3. Final Whole-Kernel $\mathsf{BMCP}_{\mathrm{CML}}$ Table (`6` Costs Kept Separate, `6`-Block Wikidata Aggregate)

| Stage | $Q_{\mathrm{rand}}$ (`M0 -> R4`) | $D_{\mathrm{serial}}$ (`LB` / `R4`) | Unprefetched `L2m_eff` (`M0 -> R4`, `Hide%`) & `DTLB_m` | $W_{\mathrm{prep}}$ (`R4`) | Working Set $M_{\mathrm{ws}}$ (`M0 -> R4`) | $W_{\mathrm{par}}$ (`R4`) | Measured 6-Blk Wall (`M0 -> S* -> R4`, `%` of `R4`) | Floor Status |
|---|---|---|---|---|---|---|---|---|
| **`S1: Pass 1 + OutQ16`** | `489k -> 489k` | `0` / `537k` (`R_127`) | `25,883 -> 25,815` (`47.4%` hide), `1,036` DTLB | `0` | `0 B` extra (`MuxStrips` in `edges`, `-1 KiB` `pt64` stack) | `O(R_127 + r_thin)` (`97.1%` AVX2 `vmovdqu`) | `2,205 -> 1,890 ->` **`1,889 µs` (`18.5%`)** | **AT FLOOR** (Thm 27.2) |
| **`S2: Strips + RV Elim`** | `747k -> 507k` (**`-240k` `S2_RL`!**) | `195k` (`S_RV`) / `195k` | `25,030 -> ~17.5k` (`~72%` hide), `8.7k` DTLB | `20k` radix | `0 B` extra (overlaid in dead `edges`) | `O(ns + S_RV)` (`GradedOperad`) | `1,844 -> 1,789 ->` **`1,655 µs` (`16.2%`)** | **AT FLOOR** (C23 `G2`) |
| **`S3: Bot + Dedup + Band64`** | `61k -> 36k` | `2.2k` (`H/2`) / `2.2k` | `1,657 -> 1,398` (`76.3%` hide), `84` DTLB | `256` lit init | `0 B` `offsets`, `8 B` `Band64`, `12.7–26.8 KiB` `inl16` (`< 32 KiB` L1d!) | `O(B + |T_long|)` (`99.983%` exact dedup) | `1,453 -> 798 ->` **`761 µs` (`7.5%`)** | **AT FLOOR** (Thm 28.1) |
| **`S4: OutQ16 Wire`** | `51k -> 51k` | `O(1)` / `O(1)` | `15,488 -> 14,786` (`82.2%` hide), `2,311` DTLB | `0` | `350–800 KiB` `edges[0..m)` | `O(m)` AVX2 + hoisted `Len4` pack | `760 -> 737 ->` **`745 µs` (`7.3%`)** | **AT FLOOR** (Thm 26.2) |
| **`S5: Walk / Emit`** | `2,740k -> ~2,380k` (`0` `lens` on `3/6` blks!) | `118k` (`m/7.6` CML) / **`895k` ($\mu=1$)** | `402,541 -> 375,332` (`90.2%` of total, `17.8%` hide), **`430.5k` DTLB** | `0` | **`950.0 KiB`** scratch (`-19.0%`) + `879 KiB out` = **`1.786 MiB`/w** (`10w = 17.86 MiB <= 20 MiB` L3) | `O(m + n)` (`Early-j ZeroSpill + GPR Len4`) | `5,668 -> 5,461 ->` **`5,138 µs` (`50.4%`)** | **AT $\mu=1$ FLOOR** (Thms 26.1–26.2) |

---

## 4. Cycle 30 Multi-Worker Qualification & `1w`/`2w` Barrier Confrontation (`measured` in [`verify_cycle30.rs`](2026-11-21-wormhole-cycle30-round4-neon-handoff/verify_cycle30.rs))

- **Authentication (`manifest.json`):** `6/6` Wikidata blocks verified (`n`, `primary_index`, `SHA-256`).
- **Exhaustive & Stress Exactness Suite:** **`14,473` cases 100% byte-exact** (`14,204` binary $n \le 9$ & ternary $n \le 6$ at every $p$ + `250` random multi-cycle/periodic/multi-epoch $m > 65{,}536$ + deep `SPIRALS` `widest = 65,000` & `len > 1,023` + length boundaries `7..32` + forced `Wide` links & `wide` `extra` + `#703` `Cap::dense` admit/refuse + column `invert`/`invert_in_place`).
- **Multi-Worker `BzReader::parallel_workers_with::<WormholeNeonCells>` (`W = 1, 2, 4, 8`):** **`21`-block multi-regime `.bz2` stream (`15.49 MiB` unrled, `CRC32 = 0x26ecd15a`) 100% byte-exact at every worker count**.
- **Per-Worker Working Set (`R4`):** **`950.0 KiB` persistent scratch (`-19.0%` vs `M0`'s `1,172.7 KiB`)** + `879 KiB` `out` = **`1.786 MiB`/worker** (`10` i9 workers = **`17.86 MiB <= 20 MiB` L3**, `2.14 MiB` headroom); steady-state DRAM `<= 0.034–0.059 B/B`.

| Block | `1w A1` | `1w A2/A1` | `1w S*/A1` | **`1w R4/A1`** | `1w N4/A1` | **`1w R4/M0`** | `2w A1` | `2w A2/A1` | `2w S*/A1` | **`2w R4/A1`** | `2w N4/A1` |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| `block-0.L` | `1300.7 µs` | `0.992×` | `0.920×` | **`0.873×`** | `0.914×` | **`0.859×`** | `1305.9 µs` | `0.992×` | `0.919×` | **`0.874×`** | `0.912×` |
| `block-2.L` | `1587.0 µs` | `0.992×` | `0.917×` | **`0.874×`** | `0.905×` | **`0.864×`** | `1589.4 µs` | `0.994×` | `0.916×` | **`0.868×`** | `0.880×` |
| `block-85.L` | `1590.8 µs` | `0.992×` | `0.944×` | **`0.884×`** | `0.925×` | **`0.884×`** | `1602.3 µs` | `0.985×` | `0.935×` | **`0.886×`** | `0.922×` |
| `block-143.L` | `2842.0 µs` | `1.002×` | `0.915×` | **`0.869×`** | `0.859×` | **`0.865×`** | `2845.5 µs` | `0.996×` | `0.910×` | **`0.867×`** | `0.855×` |
| `block-210.L` | `2490.2 µs` | `0.997×` | `0.937×` | **`0.882×`** | `0.871×` | **`0.877×`** | `2493.6 µs` | `0.989×` | `0.935×` | **`0.895×`** | `0.878×` |
| `block-207.L` | `1893.0 µs` | `0.993×` | `0.921×` | **`0.874×`** | `0.888×` | **`0.866×`** | `1900.3 µs` | `0.993×` | `0.917×` | **`0.878×`** | `0.884×` |
| `dense-703.syn` | `2916.5 µs` | `1.006×` | `0.890×` | **`0.856×`** | `0.783×` | **`0.869×`** | `2945.0 µs` | `0.991×` | `0.889×` | **`0.854×`** | `0.787×` |
| **6-BLK SUM p50** | **`11703.6 µs`** | **`0.996×`** | `0.925×` | **`0.876×`** | `0.888×` | **`0.869×`** | **`11737.0 µs`** | **`0.992×`** | `0.922×` | **`0.878×`** | `0.883×` |

**25-Round Paired Non-Parametric 95% Confidence Intervals (6-Block Wikidata Sum):**
- `A2 / A1` (A/A control): **`1w = 0.9929× [0.9880, 1.0091]`** | **`2w = 0.9912× [0.9850, 1.0042]`**
- **`R4 / A1` (`WormholeNeonCells` vs `origin/master`):** **`1w = 0.8761× [0.8718, 0.8829]`** | **`2w = 0.8784× [0.8691, 0.8869]`**
- **`R4 / M0` (`WormholeNeonCells` vs `Clone`):** **`1w = 0.8668× [0.8628, 0.8773]`** | **`2w = 0.8724× [0.8666, 0.8762]`**
- **`R4 / S*` (Net Round 4 gain over Codex's `S*` construction stack):** **`1w = 0.9476× [0.9417, 0.9566]`** | **`2w = 0.9496× [0.9434, 0.9561]`**

---

## 5. Clean Coordinator Port Specification (Delta on Top of Codex's `#714 + Neon-C29_construct + L4/L5` Stack)

Every Round 4 link ports cleanly in **`~130` lines of `#![forbid(unsafe_code)]` safe Rust** with **`0 B` extra scratch** and **zero** multi-stream `S4` scatter risk:
1. **`src/open/endpoint.rs` (`G2: GradedOperad`, `~28` lines):** Keep `#714`'s `(len10 << 22) | id22` handle on top of `Operad-MN16` (`RULE = 4`): in `join`, extract `ll = lh >> 22`, `rl = rh >> 22`, run eager `u128` byte merge only when `ll + rl <= 15`, and write `[lid, rid, exact_len, u32::MAX]` with **zero rule loads** when `ll + rl >= 16` (`ll < 1023 && rl < 1023`, saturating `len_operad` fallback for deep spirals); in `S3`, replace `e.len_operad(node_id)` with `raw_word >> 22`.
2. **`src/open.rs` (`F4 + E1 + E2h: Loop-Inversion Early-j ZeroSpill + Hoisted-S4 GPR Len4`, `~80` lines):**
   - Hoist `if !wide && bits + code_bits <= 28` **outside** `S4`'s `for a in 0..256` loop, packing `(len4 << 28) | (code << bits)` (`len4 = if l <= 15 { l } else { 0 }`) into `edges`.
   - In `emit`, hoist `if !extra.is_empty()` outside the loop, unify `dict_len = lens.len().min(inline16.len())`, slice `&mut out[..n + INLINE]`, invert the hot short-word loop around a `#[cold] #[inline(never)] emit_long_no_off` call in the outer loop, and dispatch `emit_zerospill_lenp::<28>` when `len4` is active (zero `lens[code]` load, `100%` GPR shift).
3. **`src/open/outq.rs` + `src/open.rs` (`P2c: CleanPt64`, `~22` net lines):** Replace `pos: [u32; 256]` + `thin: [u32; 256]` during `S1` with `pt: [u64; 256]` (`(thin[a] << 32) | pos[a]`, initialized to `u32::MAX as u64`), updating both via `self.pt[a] = v + ((wu << 32) | wu)` inside the single `maximal()` loop (preserving the single-path AVX2 `vmovdqu` store per Theorem 27.2!) and extracting `thin[a] = (pt[a] >> 32) as u32` after `S1`.
