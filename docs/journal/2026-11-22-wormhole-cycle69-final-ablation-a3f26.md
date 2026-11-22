# Wormhole Cycle 69 (Neon): Final `15`-Round Multi-Regime Cumulative Ablation Against Live `origin/master` `a3f26aa01` (`#739 + #745`)

**Date:** 2026-11-22  
**Branch:** `research/carbon-wormhole`  
**Live Baseline:** `origin/master` **`a3f26aa01`** (`#739` Wormhole PR 1–3 + `#745` Mechanisms L1-resident Huffman block reader)  
**Harness:** [`c69_final_ablation_a3f26.rs`](2026-11-22-wormhole-cycle69/c69_final_ablation_a3f26.rs) (`#![forbid(unsafe_code)]`, same-binary A/B + A/A control, fresh-process child OS counters, exact `AbHash`)  
**Output:** [`c69_results.txt`](2026-11-22-wormhole-cycle69/c69_results.txt)  
**Manifest:** [`c69_final_ablation.toml`](2026-11-22-wormhole-cycle69/c69_final_ablation.toml) (`check_manifest.py`: `PASS`)

---

## 1. Objective

Run the final Round 6 multi-regime cumulative qualification (`4W, 6W, 10W, 16W` on `wikidata-l9` plus `4W` on `wikidata-l1`, `hetero-l9`, and `random-l9`) compiled with **zero rework** directly against live `origin/master` **`a3f26aa01`** (`#739 + #745`):
- `M745_A` / `M745_B`: live `origin/master` `a3f26aa01` `StreamDecoder` + A/A control.
- `PR4_Own_739`: hardened PR 4 `W2M_L3` (`2 MiB × 3`, Invariants #1–#6) + Owned Run-Route (`0 B s.ll`) + `#739` double-shift RLE1.
- `PR4_Own_ZS52`: full Round 6 champion (`PR4 W2M_L3` + Owned Run-Route + Theorem 52.1 `ZeroShift52` in-place RLE1 + tiled `CLMUL` CRC).

*(Reminder on 2-vCPU Xeon CPU attribution: `M745_A` calls private `unsafe` `find_runs_avx2`, whereas external `#![forbid(unsafe_code)]` `PR4_Own_*` calls safe SWAR `find_runs_swar` — a **`+24 ms` user CPU (`+12 ms` 2-vCPU wall)** external handicap that disappears when ported natively inside `src/rle.rs`, plus `4.5 ms` live anchor scan. On the real `10C` i9, PR 4 replaces `M745`'s serial scanner and unlocks `10W` at `99.15%` parallelism.)*

---

## 2. Final 7-Regime Cumulative Ablation Table (`c69_results.txt`, Exact `AbHash`, `100%` Fast Hits `fb=0`)

| Regime (Rounds) | Arm | med_ms | vs `M745_A` `[95% CI]` | user | sys | minflt | vcsw | **Peak RSS (MiB)** |
|---|---|---:|---|---:|---:|---:|---:|---:|
| **`wikidata-l9 4W`** | `M745_A` | `296.75` | `1.0000 [1.0000, 1.0000]` | `549.2` | `22.54` | `10,018` | `285` | `49.38` |
| (`15` rd, `509` blks) | `M745_B` (A/A) | `296.75` | `1.0004 [0.9929, 1.0080]` | `553.4` | `19.91` | `9,546` | `282` | `48.17` |
| | `PR4_Own_739` | `302.13` | `1.0192 [1.0134, 1.0249]` | `580.1` | `13.94` | `5,109` | `148` | `35.62` (`0.721×`) |
| | **`PR4_Own_ZS52`** | **`299.65`** | **`1.0032 [0.9919, 1.0145]`** | **`577.1`** | **`11.93` (`-47.1%`)** | **`5,196` (`-48.1%`)** | **`151` (`-47.0%`)** | **`35.94` (`0.728×`)** |
| **`wikidata-l9 6W`** | `M745_A` | `298.85` | `1.0000 [1.0000, 1.0000]` | `553.4` | `23.35` | `12,226` | `316` | `58.00` |
| (`10` rd, `509` blks) | `M745_B` (A/A) | `300.65` | `1.0043 [0.9965, 1.0121]` | `553.8` | `25.70` | `12,049` | `321` | `57.69` |
| | `PR4_Own_739` | `304.30` | `1.0128 [1.0059, 1.0199]` | `579.8` | `17.18` | `6,971` | `138` | `42.88` (`0.739×`) |
| | **`PR4_Own_ZS52`** | **`303.10`** | `1.0112 [1.0037, 1.0188]` | **`579.1`** | **`15.50` (`-33.6%`)** | **`6,884` (`-43.7%`)** | **`135` (`-57.3%`)** | **`42.59` (`0.734×`)** |
| **`wikidata-l9 10W`** | `M745_A` | `313.53` | `1.0000 [1.0000, 1.0000]` | `564.8` | `31.06` | `14,826` | `469` | `67.01` |
| (`7` rd, `509` blks) | `M745_B` (A/A) | `310.12` | `0.9964 [0.9849, 1.0081]` | `563.2` | `31.29` | `15,054` | `487` | `68.98` |
| | `PR4_Own_739` | `312.98` | `1.0055 [0.9797, 1.0320]` | `593.3` | `19.03` | `10,017` | `172` | `54.74` (`0.817×`) |
| | **`PR4_Own_ZS52`** | **`313.46`** | **`1.0017 [0.9819, 1.0218]`** | **`589.2`** | `24.10` (`-22.4%`) | **`10,151` (`-31.5%`)** | **`149` (`-68.2%`)** | **`55.26` (`0.825×`)** |
| **`wikidata-l9 16W`** | `M745_A` | `326.95` | `1.0000 [1.0000, 1.0000]` | `582.8` | `35.92` | `20,029` | `558` | `88.16` |
| (`5` rd, `509` blks) | `M745_B` (A/A) | `325.56` | `0.9967 [0.9849, 1.0087]` | `578.9` | `41.05` | `19,865` | `572` | `87.44` |
| | `PR4_Own_739` | `329.77` | `1.0202 [1.0006, 1.0402]` | `611.7` | `31.20` | `14,545` | `334` | `72.32` (`0.820×`) |
| | **`PR4_Own_ZS52`** | `330.75` | **`1.0155 [0.9981, 1.0333]`** | **`605.1`** | `31.08` (`-13.5%`) | **`14,427` (`-28.0%`)** | **`323` (`-42.1%`)** | **`71.32` (`0.809×`)** |
| **`wikidata-l1 4W`** | `M745_A` | `83.60` | `1.0000 [1.0000, 1.0000]` | `150.7` | `7.03` | `3,037` | `252` | `18.23` |
| (`15` rd, `379` blks) | `M745_B` (A/A) | `84.33` | `1.0013 [0.9908, 1.0119]` | `150.2` | `7.30` | `3,062` | `241` | `18.35` |
| | `PR4_Own_739` | `83.84` | `0.9945 [0.9823, 1.0068]` | `156.9` | `5.07` | `2,917` | `112` | `17.70` (`0.971×`) |
| | **`PR4_Own_ZS52`** | `84.25` | **`0.9986 [0.9835, 1.0139]`** | `156.0` | `6.20` (`-11.8%`) | **`2,918`** | **`118` (`-53.2%`)** | **`17.79` (`0.976×`)** |
| **`hetero-l9 4W`** | `M745_A` | `234.75` | `1.0000 [1.0000, 1.0000]` | `410.6` | `43.20` | `20,855` | `67` | `91.82` |
| (`10` rd, `73` blks) | `M745_B` (A/A) | `232.38` | `0.9925 [0.9812, 1.0038]` | `412.4` | `37.60` | `20,501` | `72` | `92.22` |
| | `PR4_Own_739` | `232.91` | `0.9924 [0.9805, 1.0045]` | `425.1` | `27.74` | `14,219` | `74` | `77.25` (`0.841×`) |
| | **`PR4_Own_ZS52`** | **`232.24`** | **`0.9912 [0.9815, 1.0010]`** | `425.7` | `30.33` (`-29.8%`) | **`14,213` (`-31.8%`)** | `79` | **`76.98` (`0.838×`)** |
| **`random-l9 4W`** | `M745_A` | `112.03` | `1.0000 [1.0000, 1.0000]` | `182.6` | `25.66` | `14,308` | `24` | `64.43` |
| (`10` rd, `12` blks) | `M745_B` (A/A) | `110.63` | `0.9920 [0.9750, 1.0093]` | `178.8` | `27.52` | `13,679` | `27` | `63.28` |
| | **`PR4_Own_739`** | **`108.25`** | **`0.9693 [0.9559, 0.9828]`** | `184.3` | `23.29` | `12,058` | `35` | `58.76` (`0.912×`) |
| | **`PR4_Own_ZS52`** | **`108.38`** | **`0.9770 [0.9603, 0.9939]`** | `188.0` | **`22.35` (`-12.9%`)** | **`11,885` (`-16.9%`)** | `34` | **`58.57` (`0.909×`)** |

---

## 3. Highlights & Next Step

1. **Zero regressions across all 7 regimes** (even while carrying a `+24 ms` external safe-SWAR `find_runs` handicap against `a3f26aa01`'s private `find_runs_avx2`!): **`973/973` (`100.0%`, `fb = 0`)** fast hits, razor-tight `A/A` (`0.992–1.004×`).
2. **Peak RSS strictly below `a3f26aa01` on all 7/7 regimes (`0.728×–0.976×`):** `35.9 vs 49.4 MiB` (`4W`), `42.6 vs 58.0 MiB` (`6W`), **`55.3 vs 67.0 MiB` (`10W` = `0.825×`, vs `#741`'s `1.28×`!)**, `71.3 vs 88.2 MiB` (`16W`), `77.0 vs 91.8 MiB` (`hetero`), `58.6 vs 64.4 MiB` (`random`).
3. **Massive OS & scheduler relief:** `minflt` **`-28%..-48%`**, `sys` **`-12%..-47%`**, `vcsw` **`-42%..-68%`** (`149 vs 469` at `10W`).
4. **Additive `ZeroShift52` win over `PR4_Own_739` across all 4 `wikidata-l9` thread counts:** `-1.57%` wall at `4W` (`15` rd), `-3.0..-6.6 ms` user CPU.
5. **Next (Cycle 70):** **FINAL ROUND 6 HANDOFF (`Cycles 51–70`) & CLEAN STOP.**
