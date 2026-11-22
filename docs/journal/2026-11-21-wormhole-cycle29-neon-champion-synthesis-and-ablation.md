# Wormhole (Neon) Cycle 29 — Unified Round 4 Champion Synthesis (`WormholeNeonCells`) & 8-Step Cumulative Ablation

**Session:** `research/carbon-wormhole` (Round 4 — Gemini Neon, Cycle 29 of 30; 9th of 10 in Round 4)  
**Baseline `origin/master`:** `8ff1187af6a920f7d57fd338680db3a5633a3ac5` (`v0.5.11`; `src/` byte-identical to `90e671966`)  
**Verification harness:** [`verify_cycle29.rs`](2026-11-21-wormhole-cycle29-neon-champion-synthesis-and-ablation/verify_cycle29.rs) (`#![forbid(unsafe_code)]`, `0` warnings, `6/6` `manifest.json` SHA-256 auth + `5,237` Gate G1/boundary/spiral/multi-epoch cases byte-exact + `8`-step `15`-round 5-stage cumulative ablation + `35`-round paired `95%` CI `InverseBwt::invert_runs` confrontation), [`wormhole_neon_synthesis.toml`](2026-11-21-wormhole-cycle29-neon-champion-synthesis-and-ablation/wormhole_neon_synthesis.toml) (`PASS`), [`cycle29_output.txt`](2026-11-21-wormhole-cycle29-neon-champion-synthesis-and-ablation/cycle29_output.txt), [`cycle29_results.json`](2026-11-21-wormhole-cycle29-neon-champion-synthesis-and-ablation/cycle29_results.json)

---

## 1. Synthesis Verdict & Final `P2c` Keep Decision

- **PROMOTE `R4: WormholeNeonCells` (`S* + F4 NoSpill + G2 GradedOperad + E1 Early-j ZeroSpill + E2h Hoisted-S4 GPR Len4 + P2c CleanPt64`, pure $\mu = 1$, `0 B` extra scratch):**
  - Achieves **`0.8699 [0.8621, 0.8826]` vs `A1:Master`** (`p50 = 0.868×`, `p10 = 0.867×`; **`0.8609 [0.8560, 0.8801]` vs `M0:Clone`**) across the `6` authenticated Wikidata blocks (`A2/A1 = 0.9985 [0.9909, 1.0014]`) and **`0.861×` on `dense-703.syn`** at **`950.0 KiB` (`-19.0%`) persistent scratch** (`10` i9 workers = **`17.86 MiB <= 20 MiB` L3**).
  - **Outperforms `N4: Full Neon-C29` (with `Fused-XOR`) even at 1 thread (`0.868×` vs `0.885×`; `10,189.4 µs` vs `10,300.4 µs` 6-block sum)** while staying **100% $\mu = 1$ single-stream (`S4 = 745 µs` vs `1,594 µs`)**, completely avoiding `#718`'s native 4W `1.021×` `Fused-XOR` regression!
  - **PROMOTE `P2c: CleanPt64` (`R4` over `R3`):** In the `35`-round paired head-to-head audit, `R4 / R3` = **`0.9850 [0.9794, 0.9998]`** (95% CI strictly `< 1.000`), winning **`6/6` Wikidata blocks** (`0.881× -> 0.868×` p50), cutting `S1` (`1,904.3 -> 1,889.1 µs`), `dense-703.syn` (`2,487.4 -> 2,440.0 µs`), and stack frame (`-1 KiB`).

---

## 2. Complete 8-Step Cumulative 5-Stage Ablation (`6`-Block Sum `µs`, `15` Rounds Median, `measured`)

| Step / Arm | `S1` | `S2` | `S3` | `S4` | **`Pre (S1–4)`** | **`S5 Emit`** | **`6-Blk Sum`** | `Dense-703` | Persistent Scratch *(excl. `out`)* |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| `M0: 0.5.11 Clone` | `2204.7 µs` | `1843.9 µs` | `1453.4 µs` | `760.0 µs` | `6262.0 µs` | `5667.5 µs` | `11929.5 µs` | `2944.1 µs` | `1,172.7 KiB` |
| `W1: #714 + L4/L5` *(R3)* | `1974.3 µs` | `1436.0 µs` | `1282.0 µs` | `710.7 µs` | `5403.1 µs` | `5671.2 µs` | `11074.3 µs` | `2788.1 µs` | `1,048.1 KiB` (`-10.6%`) |
| `S*: N3 (Neon-C29 mu=1)` | `1890.0 µs` | `1789.4 µs` | `797.8 µs` | `736.8 µs` | `5214.0 µs` | `5461.2 µs` | `10675.3 µs` | `2590.2 µs` | **`950.0 KiB` (`-19.0%`)** |
| `R1: +F4 NoSpill` *(C22)* | `1874.6 µs` | `1786.2 µs` | `804.2 µs` | `722.4 µs` | `5187.5 µs` | **`5226.7 µs`** | `10414.2 µs` (`-261 µs`) | `2499.5 µs` | `950.0 KiB` |
| `R2: +G2 GradedOperad` *(C23)* | `1872.4 µs` | **`1659.9 µs`** | **`770.3 µs`** | `741.7 µs` | **`5044.2 µs`** | `5204.9 µs` | `10249.1 µs` (`-165 µs`) | `2511.7 µs` | `950.0 KiB` |
| `R3: +E1+E2h (Len4)` *(C24–25)* | `1904.3 µs` | `1666.1 µs` | `766.8 µs` | `731.6 µs` | `5068.9 µs` | **`5147.2 µs`** | `10216.1 µs` (`-33 µs`) | `2487.4 µs` | `950.0 KiB` |
| **`R4: +P2c CleanPt64` *(Champ)*** | **`1889.1 µs`** | **`1655.3 µs`** | **`761.4 µs`** | **`745.2 µs`** | **`5051.1 µs`** | **`5138.3 µs`** | **`10189.4 µs` (`-27 µs`)** | **`2440.0 µs`** | **`950.0 KiB` (`-19.0%`)** |
| `N4: Full Neon-C29 (XOR)` *(ref)* | `1865.4 µs` | `1778.4 µs` | `801.0 µs` | `1593.9 µs` | `6038.7 µs` | `4261.7 µs` | `10300.4 µs` | `2271.7 µs` | `950.0 KiB` *(rejected 4W `#718`)* |

**Net Round 4 Contribution on top of Post-Stack `S*` (`N3`):**
- `Pre(S1–4)`: `5,214.0 -> 5,051.1 µs` (**`-162.9 µs`**, driven by `G2 GradedOperad` `-134 µs` `S2` & `-36 µs` `S3`, plus `P2c` `-1 KiB` stack).
- `S5 Emit` ($\mu = 1$): `5,461.2 -> 5,138.3 µs` (**`-322.9 µs`**, `-529.2 µs` = **`-9.3%` below `M0: 0.5.11`**, driven by `F4 NoSpill` + `E1 Early-j ZeroSpill` + `E2h Hoisted-S4 GPR Len4`).
- **Total on top of `S*`:** **`-485.9 µs` (`-4.6%`)** at **`0 B` extra scratch**!

---

## 3. 35-Round Headline `InverseBwt::invert_runs` & Paired 95% CIs (`measured`)

| Block | `A1_p50` | `A2/A1` | `M0/A1` | `W1/A1` | `S*/A1` | `R2/A1` | `R3/A1` | **`R4/A1`** | `N4/A1` |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| `block-0.L` | `1320.8 µs` | `0.984×` | `0.998×` | `0.931×` | `0.906×` | `0.878×` | `0.872×` | **`0.855×`** | `0.907×` |
| `block-2.L` | `1608.3 µs` | `0.988×` | `1.004×` | `0.939×` | `0.908×` | `0.875×` | `0.868×` | **`0.852×`** | `0.899×` |
| `block-85.L` | `1604.5 µs` | `0.992×` | `0.997×` | `0.958×` | `0.937×` | `0.898×` | `0.895×` | **`0.876×`** | `0.921×` |
| `block-143.L` | `2849.2 µs` | `0.998×` | `1.011×` | `0.947×` | `0.911×` | `0.890×` | `0.874×` | **`0.866×`** | `0.862×` |
| `block-210.L` | `2507.4 µs` | `0.993×` | `0.992×` | `0.965×` | `0.933×` | `0.914×` | `0.894×` | **`0.886×`** | `0.868×` |
| `block-207.L` | `1906.2 µs` | `0.988×` | `1.003×` | `0.953×` | `0.917×` | `0.884×` | `0.877×` | **`0.862×`** | `0.883×` |
| `dense-703.syn` | `2929.0 µs` | `0.998×` | `0.993×` | `0.958×` | `0.895×` | `0.868×` | **`0.859×`** | `0.861×` | `0.784×` |
| **6-BLK p50/A1** | **`11796.4 µs`** | **`0.992×`** | **`1.001×`** | `0.950×` | `0.919×` | `0.892×` | `0.881×` | **`0.868×`** | `0.885×` |
| **6-BLK p10/A1** | **`11128.3 µs`** | **`0.992×`** | **`1.005×`** | `0.946×` | `0.914×` | `0.885×` | `0.875×` | **`0.867×`** | `0.884×` |

**35-Round Paired Non-Parametric 95% Confidence Intervals (6-Block Sum Ratios):**
- **`A2 / A1` (A/A control):** `0.9985×` **`[0.9909, 1.0014]`**
- **`R3 / A1` (`E2h` vs `Master`):** `0.8794×` **`[0.8701, 0.8885]`**
- **`R4 / A1` (`WormholeNeonCells` vs `Master`):** **`0.8699× [0.8621, 0.8826]`**
- **`R4 / M0` (`WormholeNeonCells` vs `Clone`):** **`0.8609× [0.8560, 0.8801]`**
- **`R3 / S*` (Round 4 links on top of `S*`):** `0.9575×` **`[0.9444, 0.9652]`**
- **`R4 / R3` (`CleanPt64` over `E2h`):** **`0.9850× [0.9794, 0.9998]`** *(strictly `< 1.000` -> promoted!)*

**Per-Worker Working Set (`R4`):** `950.0 KiB` persistent scratch (`-19.0%` vs `M0`) + `879 KiB` `out` = **`1.786 MiB`/worker** (`10w = 17.86 MiB <= 20 MiB` L3); steady-state DRAM `<= 0.034–0.059 B/B`.  
**Acquisition Cost of Round 4 Delta over `S*`:** `~130` lines of `#![forbid(unsafe_code)]` safe Rust (`GradedOperad` `~28` lines, `emit_zerospill` + `emit_zerospill_lenp::<28>` + hoisted `S4` `Len4` `~80` lines, `QueuesPt64` `~22` net lines).

---

## 4. Next Cycle Plan (`Cycle 30 (Neon)` — Final Cycle of Round 4)
Run the full Round 4 qualification & multi-worker confrontation on **`WormholeNeonCells` (`R4`)**:
1. `14,466+` exhaustive Gate G1 (`n <= 9` binary, `n <= 6` ternary) + deep spiral + boundary `7..32` + wide links + `#703` dense admit/refuse + column `invert`/`invert_in_place` exactness suite.
2. `21`-block multi-regime `.bz2` stream through `BzReader::parallel_workers_with::<WormholeNeonCells>` at **`W = 1, 2, 4, 8`** (`CRC32` verified against `OpenCells`).
3. `25`-round **`1w` and `2w` barrier-synchronized** `InverseBwt::invert_runs` confrontation against `A1:Master`, `A2:Master`, `M0:Clone`, `S*:N3`, and `N4:Neon-C29 (Fused-XOR)`.
4. Deliver the complete Round 4 (`Cycles 21–30`) whole-kernel $\mathsf{BMCP}_{\mathrm{CML}}$ table, full keep/kill ledger, and clean Coordinator port specification!
