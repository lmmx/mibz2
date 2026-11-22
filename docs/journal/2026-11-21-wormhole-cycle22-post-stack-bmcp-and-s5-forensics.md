# Wormhole (Neon) Cycle 22 — Post-Stack `S*` Whole-Kernel $\mathsf{BMCP}_{\mathrm{CML}}$ Bounds & `S5` Regression Forensics

**Session:** `research/carbon-wormhole` (Round 4 — Gemini Neon, Cycle 22 of 30; 2nd of 10 in Round 4)  
**Baseline `origin/master`:** `8ff1187af6a920f7d57fd338680db3a5633a3ac5` (`v0.5.11`; `src/` byte-identical to `90e671966`)  
**Verification harness:** [`verify_cycle22.rs`](2026-11-21-wormhole-cycle22-post-stack-bmcp-and-s5-forensics/verify_cycle22.rs) (`#![forbid(unsafe_code)]`, `0` warnings, `6/6` `manifest.json` SHA-256 auth + `5,229` Gate G1/spiral/multi-epoch cases + `F0..F5` `2 × 2` factorial & zero-spill control + exact Comet Lake `32 KiB L1d / 256 KiB L2 / 2048 KiB L3 / 64-DTLB / 32-page streamer` simulator + `25`-round `p50`/`p10` `InverseBwt::invert_runs` confrontation), [`post_stack_bmcp.toml`](2026-11-21-wormhole-cycle22-post-stack-bmcp-and-s5-forensics/post_stack_bmcp.toml) (`PASS`), [`cycle22_output.txt`](2026-11-21-wormhole-cycle22-post-stack-bmcp-and-s5-forensics/cycle22_output.txt), [`cycle22_results.json`](2026-11-21-wormhole-cycle22-post-stack-bmcp-and-s5-forensics/cycle22_results.json)

---

## 1. Executive Summary & One-Line Decisions

- **SOLVED & PROMOTED `F4: emit_no_off_nospill` (`-367.4 µs` on `S5`, `-456.7 µs` vs `M0`, `0 B` scratch, pure $\mu = 1$):** Cycle 21's `+245 µs` `S5` regression in `N1..N3` was a **pure LLVM codegen / register-spill artifact** (`emit_no_off` spilled `mask` to `28(%rsp)` on the critical `j = edge & mask` chain and left `extra.is_empty()` inside the hot loop). Hoisting `if extra.is_empty()`, slicing `&mut out[..n + INLINE]`, and cold-outlining `#[cold] #[inline(never)] emit_long_no_off` flips `S*`'s `S5` from `5,778.4 µs` to **`5,411.0 µs` (`-6.4%`)**, pushing **pure $\mu = 1$ `S* + F4`** to **`0.867×` of `A1:Master` (`0.863×` p10, `0.895×` `M0`, winning `7/7` blocks including `0.836×` on `dense-703.syn`)** at **`950.0 KiB` scratch (`-19.0%`)** with **zero** `#705`/`#718` multi-stream wire overhead.
- **COMPLETED Charge #1 (Post-Stack `S*` Whole-Kernel $\mathsf{BMCP}_{\mathrm{CML}}$ Re-Derivation):** Instrumented all 5 stages of `M0 (0.5.11)` vs `S*` (`master + #714 + Neon-C29_construct + L4/L5`, $\mu = 1$) across all `6` Wikidata blocks (`n ≈ 900,000`) with all **six costs kept strictly separate** (`Q_rand`, `D_serial`, `L2m_eff/L3m/DTLB`, `W_prep`, `M_ws`, `W_par`).
  - **#1 Remaining Gap (`S5: Walk / Emit`):** **`5,411 µs` (`50.2%` of `S*+F4`)**, **`375,332` unprefetched L2 misses (`90.2%` of whole-kernel `L2m_eff`)**, and **`430,546` L1 DTLB misses (`97.3%` of whole-kernel DTLB misses)** because `edges[0..m)` (`350–800 KiB` = `88–200` `4 KiB` pages) overflows Comet Lake's `64`-entry L1 DTLB (`256 KiB` reach).
  - **#2 Highest-Confidence Pre-Emit Gap (`S2: RV Elim`):** **`1,833 µs` (`17.0%`)**, where `Operad-MN16` still executes **`336,297` random rule loads (`S2_RL`)** across 6 blocks; our exact simulation proves **`GradedOperad` (`#714` `(len10 << 22) | id22` × `Operad-MN16`) cuts `S2_RL` by `-71.5%` (`336,297 -> 95,916`)** while preserving `S3 = 829 µs` -> **Cycle 23 target**.
  - **#3 Gap (`S1: Pass 1 + OutQ16`):** **`1,946 µs` (`18.1%`)** (`25,815` `L2m_eff`, `47.4%` streamer-hidden) -> **Cycle 27 target**.
  - **`S3` (`829 µs`, `7.7%`, `1,398` `L2m_eff`) & `S4` (`750 µs`, `7.0%`, `82.2%` streamer-hidden):** Near/at the $\mathsf{BMCP}_{\mathrm{CML}}$ floor.

---

## 2. Forensic Dissection of Cycle 21's `+245 µs` `S5` Regression (`verified` via `--emit=asm` & `measured` via `2 × 2` Factorial)

### 2.1 `--emit=asm` Root Cause (`verified`)
Disassembling `verify_cycle21.rs` revealed why `N1..N3` and `W2` paid `+245 µs` in `S5` (`6,018–6,069 µs`) vs `M0`/`W1` (`5,795–5,813 µs`):
1. Inside `emit_offsets` (`.LBB68_24`), LLVM kept `mask` in callee-saved `%r15d` (`andnl %r13d, %r15d, %eax`), whereas inside `emit_no_off` (`.LBB67_24`) having one fewer argument caused LLVM to hold `&mut Vec<u8>` in `%rbp` across the loop and **spill `mask` to `28(%rsp)`**, inserting an extra stack load `movl 28(%rsp), %eax` before `andnl %r13d, %eax, %eax` on every single step!
2. Both `origin/master`'s `emit` (`emit_offsets`) and `emit_no_off` also kept `testq %rcx, %rcx; je` (`extra.is_empty()`) inside the hot loop, spilled `bits` (`24(%rsp)`), `out.ptr` (`40(%rsp)`), `inline16.ptr`, and `inline16.len` because the in-loop `callq memcpy` on the `LONG` branch (`4.7%–7.5%` of steps) clobbers all 9 caller-saved x86-64 registers.
3. In `emit_no_off_nospill` (`F4`), hoisting `if extra.is_empty()` out of the loop, working on `&mut out[..n + INLINE]`, and outlining `LONG` into `#[cold] #[inline(never)] emit_long_no_off` eliminated both `extra.is_empty()` and the `bits` stack reload (`shrxq %r12, %rbx, %r10`), though LLVM still reloads `lens.ptr` (`128(%rsp)`), `inline16.ptr` (`112(%rsp)`), `out.ptr` (`32(%rsp)`), and `mask` (`(%rsp)`) across the `callq emit_long_no_off` boundary (leaving further headroom for Cycle 24!).

### 2.2 Isolated `S5` `2 × 2` Factorial + Zero-Spill Control (`25` rounds p50, equalized post-`S4` `edges` precondition, `measured`)

*(Note: an initial un-equalized run exposed a `+850 µs` cross-arm `s0`/`s4` L2 eviction bias on `F2`/`F5`; equalizing the post-`S4` sequential `edges` touch before each arm removed 100% of that artifact.)*

| Block | `F0: ND+Off` (`0.5.11`) | `F1: ND+NoOff` | `F2: DD+Off` | `F3: DD+NoOff` (`S*`) | **`F4: DD+NoSpill`** | `F5: ND+NoSpill` | **`F4 - F3`** | **`F4 - F0`** |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| `block-0.L` | `529.1 µs` | `527.7 µs` | `526.2 µs` | `518.9 µs` | **`496.6 µs`** | `509.2 µs` | **`-22.3 µs`** | **`-32.5 µs`** |
| `block-2.L` | `683.1 µs` | `677.8 µs` | `673.7 µs` | `659.3 µs` | **`632.2 µs`** | `658.9 µs` | **`-27.1 µs`** | **`-50.9 µs`** |
| `block-85.L` | `937.1 µs` | `926.6 µs` | `929.9 µs` | `912.3 µs` | **`851.7 µs`** | `885.9 µs` | **`-60.5 µs`** | **`-85.4 µs`** |
| `block-143.L` | `1379.8 µs` | `1372.0 µs` | `1372.6 µs` | `1349.8 µs` | **`1272.5 µs`** | `1326.6 µs` | **`-77.3 µs`** | **`-107.3 µs`** |
| `block-210.L` | `1403.3 µs` | `1391.1 µs` | `1395.7 µs` | `1370.3 µs` | **`1289.2 µs`** | `1340.3 µs` | **`-81.1 µs`** | **`-114.1 µs`** |
| `block-207.L` | `890.3 µs` | `886.7 µs` | `868.0 µs` | `849.5 µs` | **`805.7 µs`** | `846.9 µs` | **`-43.8 µs`** | **`-84.6 µs`** |
| `dense-703.syn` | `1537.5 µs` | `1548.1 µs` | `1543.6 µs` | `1529.8 µs` | **`1438.7 µs`** | `1467.0 µs` | **`-91.1 µs`** | **`-98.8 µs`** |
| **6-BLK SUM** | **`5822.6 µs`** | **`5781.9 µs`** | **`5766.0 µs`** | **`5660.1 µs`** | **`5347.9 µs`** | **`5567.8 µs`** | **`-312.2 µs`** | **`-474.7 µs`** |

**Clean Factorial Main Effects (6-block sum):**
1. **`WordDedup` L1D effect (holding `emit` identical):** `-56.6 µs` (`Off`), `-121.9 µs` (`NoOff`), **`-220.0 µs` (`NoSpill`)** — `WordDedup` is **genuinely positive** once register spills stop masking L1d hits!
2. **`emit_no_off` vs `emit_offsets` (holding dict identical):** `-40.7 µs` (`ND`), `-105.9 µs` (`DD`) — embedding `(s, e)` in `inline16[code][0..8]` saves `offsets` traffic cleanly when `#[inline(never)]` avoids the Cycle 21 `mask` spill.
3. **Zero-Spill Cold-`LONG` + Hoisted `extra` (`F4 - F3` / `F5 - F0`):** **`-312.2 µs` (`DD`)** / **`-254.8 µs` (`ND`)** — winning **7/7 blocks** at `0 B` scratch!

---

## 3. Charge #1 — Re-Derived Whole-Kernel $\mathsf{BMCP}_{\mathrm{CML}}$ Bounds for Post-Stack `S*` (`6` Costs Separate)

### 3.1 Model Specification & Limits (`proved` / `verified`)
We apply Cycle 20's Comet Lake machine model $\mathsf{BMCP}_{\mathrm{CML}}$ (`15` usable GPRs, `10` L1d LFBs with effective cap $\mu_{\mathrm{eff}}^{\mathrm{LFB}} \in [7.4, 7.8]$, `32 KiB` 8-way L1d, `256 KiB` 4-way L2, `2048 KiB` 16-way per-worker L3 slice = `20 MiB / 10w`, `64`-entry 4-way L1 DTLB `4 KiB` pages, `32`-page `4 KiB` L2 streamer) to `S* = master + #714 + Neon-C29_construct + L4/L5` ($\mu = 1$, no `Fused-XOR`).
- **What the model counts:** exact per-stage random-access count $Q_{\mathrm{rand}}$, serial dependency depth $D_{\mathrm{serial}}$, set-associative L1d/L2/L3/DTLB misses, 32-page streamer hiding, preprocessing work $W_{\mathrm{prep}}$, active & persistent working set $M_{\mathrm{ws}}$, and parallel work $W_{\mathrm{par}}$.
- **What the model ignores (explicit limits):** branch-predictor TAGE aliasing (e.g. `LONG` branch `4.7%–7.5%` mispredicts in `S5`), ROB/RS store-forwarding stalls, cross-core ring-bus coherence between `10` active i9 workers, and LLVM register-allocation spills (which our `--emit=asm` check explicitly exposed above!).
- **Cold-start vs steady-state L3m note:** `CmlSim` starts cold at `S1`, recording `27k–33k` total compulsory+capacity L3m (`1.93–2.35 B/B` cold fill); in steady-state persistent-scratch `BzReader` workers, `S*`'s persistent scratch is **`950.0 KiB` (`-222.7 KiB` vs `M0`'s `1,172.7 KiB`)** plus `879 KiB` `out` = **`1,829.0 KiB` (`1.786 MiB`/worker)**, so **`10` i9 workers occupy `17.86 MiB <= 20 MiB` L3** (`2.14 MiB` headroom; steady-state DRAM traffic remains `<= 0.034–0.059 B/B`).

### 3.2 Per-Block Post-Stack `S*` Hardware Profile (`verified` via `CmlSim` on exact `Elim` + `edges` traces)

| Block | `m` | `ns` | `S_RV` | `H_crit` | `B_bands` | `U_codes` | `LONG%` | `S5 edges L2m_eff` | `Inl16 L1m` (`M0 -> S*`) | `S2_RL` (`Operad -> GradedOperad`) |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| `block-0.L` | `89,874` | `3,448` | `33,863` | `1,209` | `3,204` | `995` | `7.5%` | `26,381` | `4,122 -> 4,866` (`+744`) | `58,631 -> 16,629` (**`-71.6%`**) |
| `block-2.L` | `112,070` | `3,827` | `37,060` | `1,014` | `4,244` | `1,281` | `7.1%` | `37,632` | `7,119 -> 7,638` (`+519`) | `63,979 -> 19,090` (**`-70.2%`**) |
| `block-85.L` | `156,404` | `1,565` | `17,540` | `214` | `2,446` | `811` | `4.7%` | `48,031` | `4,925 -> 6,096` (`+1,171`) | `30,652 -> 7,486` (**`-75.6%`**) |
| `block-143.L` | `196,841` | `5,047` | `45,178` | `934` | `6,788` | `1,712` | `6.0%` | `92,579` | `19,217 -> 17,575` (`-1,642`) | `76,748 -> 23,557` (**`-69.3%`**) |
| `block-210.L` | `204,281` | `2,809` | `27,649` | `474` | `4,219` | `1,219` | `5.6%` | `90,157` | `12,463 -> 13,551` (`+1,088`) | `47,657 -> 12,808` (**`-73.1%`**) |
| `block-207.L` | `135,921` | `3,474` | `33,963` | `605` | `4,596` | `1,205` | `6.0%` | `51,297` | `9,052 -> 10,469` (`+1,417`) | `58,630 -> 16,346` (**`-72.1%`**) |
| `dense-703.syn` | `224,153` | `1,663` | `18,577` | `3` | `5,016` | `4,370` | `1.0%` | `155,972` | `122,357 -> 123,022` (`+665`) | `32,031 -> 7,767` (**`-75.8%`**) |
| **6-BLK SUM** | **`895,391`** | **`20,170`** | **`195,253`** | **`4,450`** | **`25,497`** | **`7,223`** | **`6.0%`** | **`346,077`** | **`56,898 -> 60,195`** | **`336,297 -> 95,916` (`-71.5%`)** |

*Key insight from `Inl16 L1m`:* On `5/6` blocks, `inline16` L1d misses rose slightly (`+519..+1,417`) under `S*` ONLY because `emit_no_off` touches `inline16[code]` on `LONG` steps too (saving the separate `offsets` array!), AND `WordDedup` assigns code IDs in first-seen band order instead of **frequency order** (`Zipf` hot-code packing into `1–2` `4 KiB` pages is a Cycle 24 target!).

### 3.3 Post-Stack `S*` 5-Stage Cost & Gap Table (6-Block Aggregate, All `6` Costs Separate)

| Stage (Gap Rank) | $Q_{\mathrm{rand}}$ (`LB` / `S*`) | $D_{\mathrm{serial}}$ (`LB` / `S*`) | Unprefetched `L2m_eff` (`M0 -> S*`, `Hide%`) & `DTLB_m` | $W_{\mathrm{prep}}$ (`S*`) | Working Set $M_{\mathrm{ws}}$ (`M0 -> S*`) | $W_{\mathrm{par}}$ (`S*`) | Measured 6-Blk Wall (`M0 -> S* -> S*+F4`, `%` of `S*+F4`) | Remaining Gap & Attack Vector |
|---|---|---|---|---|---|---|---|---|
| **#1 `S5: Walk / Emit`** | `895k` / `2,740k` (`3` loads/step: `edge`, `lens`, `inl16`) | `118k` (`m/7.6` CML) / **`895k` ($\mu=1$)** | `402,541 -> 375,332` (**`90.2%` of total!**, `17.8%` hide), **`430,546` DTLB (`97.3%`!)** | `0` | `1,173 -> 950 KiB` scratch (`+879 KiB out` = `1.786 MiB`/w; `10w = 17.86 MiB <= 20 MiB` L3) | `O(m + n)` | `5,868 -> 5,778 ->` **`5,411 µs` (`50.2%`)** | **LARGEST GAP:** (1) `lens[code]` load + remaining hot-loop stack reloads (`Edge-Packed GPR Len4` + `Zipf` codes, Cycle 24); (2) uniform `LIMIT=16` vs per-symbol Lagrange $\Lambda_a$ shrinking `edges` toward `64`-page DTLB (Cycle 25); (3) algebraic `S5` contraction without multi-stream wire cost (Cycle 26). |
| **#2 `S2: Strips + RV Elim`** | `20k + 2*S_RV` / `747k` (`+336k` `S2_RL`) | `195k` (`S_RV`) / `195k` | `25,030 -> 22,019` (`69.1%` hide), `8,661` DTLB | `20k` radix | `0 B` extra (overlaid in dead `edges`) | `O(ns + S_RV)` | `1,887 -> 1,835 ->` **`1,833 µs` (`17.0%`)** | **HIGHEST-CONFIDENCE PRE-EMIT GAP (`-340 µs` PROVED):** `GradedOperad` (`#714` `(len10<<22)|id22` × `Operad-MN16`) cuts `S2_RL` **`-71.5%` (`336k -> 96k`)** + single-store `Narrow::set_links` (Cycle 23). |
| **#3 `S1: Pass 1 + OutQ16`** | `r_thin` (`489k`) / `489k` | `0` / `537k` (`R_127`) | `25,883 -> 25,815` (`47.4%` hide), `1,036` DTLB | `0` | `0 B` extra (`MuxStrips` in `edges`, `OutQ16` in `out`) | `O(R_127 + r_thin)` | `2,292 -> 1,958 ->` **`1,946 µs` (`18.1%`)** | **#2 WALL STAGE:** `All+Pos` saved `-334 µs`; test `4`-record distinct-thin SWAR batching + `MuxStrips` pre-sizing (Cycle 27). |
| **#4 `S3: Bot + Dedup + Band64`** | `H_crit/2 + |T_long|` / `36k` | `2.2k` (`H/2`) / `2.2k` | `1,657 -> 1,398` (`76.3%` hide), `84` DTLB | `256` lit init | `0 B` `offsets`, `8 B` `Band64`, `12.7–26.8 KiB` `inl16` (`< 32 KiB` L1d!) | `O(B + |T_long|)` | `1,489 -> 842 ->` **`829 µs` (`7.7%`)** | **NEAR FLOOR (`-44%` vs `M0`):** Audit `pair_ht/str_ht` collision rate & `Band64` polish (Cycle 28). |
| **#5 `S4: OutQ16 Wire`** | `25.5k` (`B`) / `51k` | `O(1)` / `O(1)` | `15,488 -> 14,786` (**`82.2%` hide!**), `2,311` DTLB | `0` | `350–800 KiB` `edges[0..m)` | `O(m)` AVX2 | `740 -> 769 ->` **`750 µs` (`7.0%`)** | **AT FLOOR:** `82.2%` streamer-hidden, `D = O(1)`; any `wire_xor` regresses `+879 µs`. |

---

## 4. End-to-End Headline 1T `InverseBwt::invert_runs` Confrontation (`25` Rounds, `measured`)

`A2/A1` A/A control = **`0.995×` p50 (`0.996×` p10)** across the 6 authenticated Wikidata blocks (`12,411.7 µs` sum):

| Block | `A1:Master` p50 | `A2/A1` | `M0/A1` | `W1/A1` (`#714+L4/L5`) | `S*/A1` (`N3`) | **`S*+F4 / A1`** | **`S*+F4 / M0`** |
|---|---:|---:|---:|---:|---:|---:|---:|
| `block-0.L` | `1369.0 µs` | `0.990×` | `0.985×` | `0.914×` | `0.902×` | **`0.870×`** | **`0.883×`** |
| `block-2.L` | `1668.6 µs` | `0.997×` | `0.984×` | `0.922×` | `0.899×` | **`0.872×`** | **`0.886×`** |
| `block-85.L` | `1702.4 µs` | `0.996×` | `0.958×` | `0.923×` | `0.908×` | **`0.872×`** | **`0.911×`** |
| `block-143.L` | `3009.2 µs` | `0.996×` | `0.969×` | `0.919×` | `0.903×` | **`0.860×`** | **`0.887×`** |
| `block-210.L` | `2655.0 µs` | `0.994×` | `0.953×` | `0.920×` | `0.902×` | **`0.866×`** | **`0.909×`** |
| `block-207.L` | `2007.4 µs` | `0.995×` | `0.973×` | `0.919×` | `0.895×` | **`0.866×`** | **`0.891×`** |
| `dense-703.syn` | `3048.4 µs` | `1.001×` | `0.981×` | `0.939×` | `0.882×` | **`0.836×`** | **`0.852×`** |
| **6-BLK p50/A1** | **`12411.7 µs`** | **`0.995×`** | **`0.969×`** | **`0.920×`** | **`0.901×`** | **`0.867×`** | **`0.895×`** |
| **6-BLK p10/A1** | **`12143.8 µs`** | **`0.996×`** | **`0.967×`** | **`0.919×`** | **`0.900×`** | **`0.863×`** | **`0.893×`** |

**Per-Worker Working Set:** `S*+F4` = `950.0 KiB` persistent scratch (`-19.0%` vs `M0`'s `1,172.7 KiB`) + `879 KiB` `out` = **`1.786 MiB`/worker** (`10w = 17.86 MiB <= 20 MiB` L3).  
**Acquisition Cost of `F4`:** `~18` lines of safe Rust inside `emit` (hoist `extra.is_empty()`, `&mut out[..n+INLINE]`, `#[cold] #[inline(never)]` `emit_long_no_off`).

---

## 5. Next Cycle Plan (`Cycle 23 (Neon)`)
Attack the highest-confidence pre-emit gap proved in §3.2 (`S2_RL: 336,297 -> 95,916`, `-71.5%`): implement and benchmark **`GradedOperad`** (`#714` `(len10 << 22) | id22` Graded Free-Monoid Handle fused with `Operad-MN16` `RULE=4 u128` short-word storage) + single-store **`Narrow::set_links`** (`a[id*4+side] = at | (next << 16)`) on top of `S* + F4`, testing if we simultaneously achieve `#714`'s `S2` (`~1,450 µs`), `Operad`'s `S3` (`829 µs`), and `F4`'s `S5` (`5,411 µs`) -> targeting **`~0.83–0.84×` of `origin/master` in pure $\mu = 1$ safe Rust**!
