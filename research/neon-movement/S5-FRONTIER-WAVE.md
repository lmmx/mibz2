# Sense 5 (`frontier-wave`): Lateral Mathematics #2 — Thick-Boundary Synchronous Radix-Wave (`\partial_{\mathrm{thick}}`-Wave)

## 1. Exact Definition of "Moves Together"

In Rauzy–Veech (`Open`), wide runs (`w \ge L_{\min}`) are contracted into thick strips, leaving thin excursions that enter from a thick strip and eventually return to one. Define the **thick-to-thin boundary** `\partial_{\mathrm{thick}} \subset M` as the set of retained rows whose predecessor `\mathrm{FL}(x)` lies in a thick run (`w \ge L_{\min}`), together with the primary index `p`.

- **Automatic, template-free anchor set:** `\partial_{\mathrm{thick}}` is acquired in **one streaming pass** over `IN` (`Q = 0`, `s = 256` cursors) by marking which retained rows are reached from another retained row; the unmarked ones are exactly the exits of thick strips.
- **What "moves together" means:** All `K_{\mathrm{exc}} = |\partial_{\mathrm{thick}}|` thin excursions across the entire block **move together synchronously at the same thin-excursion depth `t = 0, 1, 2, \dots`** as ONE strictly row-sorted frontier `F_t = \{(\mathrm{LF}^t(x), \mathrm{eid}(x)) : x \in \partial_{\mathrm{thick}}, \text{ still thin}\}`!

## 2. Composition Rule, Proofs, and Exactness

- **Composition rule (Run-Record Ordered-Frontier Merge — `Q_{\mathrm{wave}} = 0`, `D_{\mathrm{wave}} = 0`):**
  Maintain `F_t` sorted strictly ascending by `cur_row`. To advance `F_t \to F_{t+1}`:
  1. Stream-merge `F_t` against **one sequential scan of the `r` run records `IN`** (size `r \approx 0.057\text{--}0.143\,n`, NOT `n`-byte `L`!). For each `(cur_row, eid) \in F_t` falling in run `(s_i, w_i, b_i)` with running bucket offset `\mathrm{occ}[b_i]`: emit byte `b_i` to excursion `eid` and compute `nxt_row = \mathrm{occ}[b_i] + (cur_row - s_i)`.
  2. If `nxt_row` enters a thick run or hits `p`, record `cur_row` as excursion `eid`'s exit; otherwise append `(nxt_row, eid)` to bucket `b_i` (`256` sequential write cursors `\le s_{\max}`).
  3. Concatenate buckets `0 \dots 255` to obtain `F_{t+1}`.
- **Theorem 5.1 (Exactness & Sortedness Invariant — PROVED & byte-verified on 8/8 blocks):**
  1. *Sortedness (Ordered-Frontier Lemma, MONODROMY 104 Lemma F):* Because `\mathrm{LF}` is strictly increasing on each symbol class `L^{-1}(b)` and `F_t` is strictly ascending, each bucket `b` is strictly ascending in `nxt_row`, and bucket `b` lies strictly below bucket `b'` for `b < b'`. Hence `F_{t+1}` is strictly ascending with zero sorting. (Asserted at every round on all 8 blocks.)
  2. *Exactness:* Every thin row in `M` has unique in-degree 1 under `\mathrm{LF}` and belongs to a unique excursion starting in `\partial_{\mathrm{thick}}`, so `\sum_{\mathrm{eid}} |\mathrm{exc}(\mathrm{eid})| = m`. Gluing each excursion's single exit row `x_{\mathrm{last}}` through `Open`'s thick-strip table (`K_{\mathrm{exc}}` links instead of `m`) recovers the exact single cycle of length `n`. Verified `decoded == T_ref` byte-for-byte on all 8 blocks. `\square`

## 3. Prior Art

- MONODROMY 104 (Ordered-Frontier Lemma), MONODROMY 113 §3 (Relation C pairing and synchronous wave scanning `n`-byte `L` per round), MONODROMY 114 (Sense E), `neon-bipolar-contraction` Cycle 4 (thin-excursion spelling conservation).
- **What was missed / new here:**
  1. `\partial_{\mathrm{thick}}` supplies the wave front **automatically from Rauzy–Veech** with zero template oracle.
  2. Stream-merging `F_t` against **`IN` (`r` run records)** instead of `n`-byte `L` exploits Wikidata's `r/n = 0.057\text{--}0.143`, slashing streamed byte volume to **`P_{\mathrm{bytes}} = 19.6\text{--}75.0`** (`3\times` less than S4 and `8\times` less than S3!).
  3. Quantifying the exact hybrid `(P_{\mathrm{cap}}, Q_{\mathrm{tail}})` Pareto curve shows `88\text{--}93%` of all thin rows finish in `P_{\mathrm{cap}} = 32` rounds (`wave_P_bytes = 12.7\text{--}31.5`), and `99.9\text{--}100%` finish by `P_{\mathrm{cap}} = 64` rounds!

## 4. Measurements on Novel Random Hex and the Six Admitted Blocks

Script: [`s5_frontier_wave.py`](s5_frontier_wave.py); raw JSON: [`results_s5_frontier_wave.json`](results_s5_frontier_wave.json). All 8 blocks pass byte-exact verification.

| Block | `L_{\min}` | `K_{\mathrm{exc}}` (`\!/ h_1`) | Exc. len mean / p99 / `d_{\max}` | `P=32` done / `Q_{\mathrm{tail}}/h_1` / `P_{\mathrm{bytes}}` | Full stream `Q_{\mathrm{acq,strips}}` + `Q_{\mathrm{run,exc}}` = `Q_{\mathrm{tot}}` (`\!/ h_1`) | Full `P` / `P_{\mathrm{bytes}}` / `S` |
|---|---:|---:|---:|---:|---:|---:|
| `synth-hex-iid-65536` | 2* | 7,071 (`0.123 h_1`) | 8.1 / 35 / 78 | 87.8% (`P=16`) / — / 107.8 | 5,474 + 7,071 = 12,545 (`0.218 h_1`) | 81 / 464.4 / 1.00 MB |
| `synth-tmpl-hex-1000x40` | 16 | 2,001 (**`0.050 h_1`**) | 23.5 / 41 / **41** | 80.8% / 0.226 / 112.3 | 312 + 2,001 = **2,313 (`0.058 h_1`)** | 44 / 146.8 / 0.66 MB |
| `block-0.L` | 16 | 14,855 (`0.423 h_1`) | 5.9 / 41 / **48** | **91.3%** / 0.220 / **12.7** | 122,116 + 14,855 = 136,971 (`3.90 h_1`) | 51 / **19.6** / 2.55 MB |
| `block-2.L` | 16 | 21,739 (`0.634 h_1`) | 5.0 / 41 / **58** | **93.3%** / 0.213 / **13.8** | 123,609 + 21,739 = 145,348 (`4.24 h_1`) | 61 / **24.8** / 2.96 MB |
| `block-85.L` | 16 | 21,815 (`0.393 h_1`) | 7.1 / 43 / **59** | **89.0%** / 0.308 / **19.9** | 47,794 + 21,815 = **69,609 (`1.25 h_1`)** | 62 / **36.0** / 3.00 MB |
| `block-143.L` | 16 | 43,958 (`0.606 h_1`) | 4.4 / 42 / **86** | **91.4%** / 0.229 / **27.0** | 136,376 + 43,958 = 180,334 (`2.48 h_1`) | 89 / **68.0** / 4.74 MB |
| `block-210.L` | 16 | 35,459 (**`0.365 h_1`**) | 5.7 / 43 / **80** | **87.8%** / 0.254 / **31.5** | 70,083 + 35,459 = **105,542 (`1.09 h_1`)** | 83 / **75.0** / 4.14 MB |
| `block-207.L` | 16 | 25,848 (`0.414 h_1`) | 5.2 / 42 / **86** | **89.7%** / 0.223 / **21.3** | 105,582 + 25,848 = 131,430 (`2.11 h_1`) | 89 / **54.5** / 3.21 MB |

*\*`synth-hex-iid-65536` has zero runs of width `\ge 16`, so `L_{\min} = 2` (width `\ge 2` runs as anchors) is reported.*

## 5. Referee Verdict in `(Q, D, P, S)` under `Π`

- **Contraction on novel random hex vs Wikidata:**
  - On **templated novel 40-hex (`synth-tmpl-hex-1000x40`)**, the `∂_thick`-wave collapses `Q_{\mathrm{tot}}` (INCLUDING thick-strip acquisition!) to **`0.058 h_1`** (`17\times` below `h_1`!) in `41` rounds (`d_{\max} = 41` = exact 40-hex SHA1 + delimiter!).
  - On **all 6 admitted Wikidata blocks**, `p99 = 41\text{--}43` and `d_{\max} = 48\text{--}86`, cutting `Q_{\mathrm{run}}` to `K_{\mathrm{exc}} = 0.365\text{--}0.634\,h_1` with **`Q_{\text{thin-wave}} = 0`** and the lowest streamed byte volume of any `Q_{\mathrm{thin}} = 0` method (`P_{\mathrm{bytes}} = 19.6\text{--}75.0`, `S = 2.55\text{--}4.74\text{ MB}`)!
  - On **pure unstructured novel hex (`r/n = 0.936`)**, `L_{\min} = 16` degenerates (`K_{\mathrm{exc}} = 1`) and `L_{\min} = 2` pays `P_{\mathrm{bytes}} = 464.4` (`24\times` higher than `block-0.L`), proving how strongly the `IN`-merge wave leverages scaffold sparsity `r/n \ll 1`.
- **Where the cost goes under `Π`:**
  - In **stream-wave mode**: `Q_{\mathrm{thin}} = 0` and `D_{\mathrm{thin}} = 0`, relocated into `P = 51\text{--}89` sweeps over `IN` (`> K = 2`), plus thick-strip acquisition `Q_{\mathrm{acq,strips}}` and excursion glue `Q_{\mathrm{run,exc}} = K_{\mathrm{exc}}`.
  - In **probe-wave mode** (`P = 3`): `Q = Q_{\mathrm{acq,strips}} + m + K_{\mathrm{exc}}`, but thin-wave dependent depth collapses from `m` (`88\text{--}202\text{k}`) to **`d_{\max} = 48\text{--}86`** (`> 1{,}000\times` reduction!) with strictly ascending-row accesses in every round.
- **Verdict:** Among all `Q_{\mathrm{thin}} = 0` senses, `∂_thick`-Wave achieves the **lowest byte volume (`19.6\text{--}75.0`) and lowest memory (`2.55\text{--}4.74\text{ MB}`)** on real Wikidata blocks, while cleanly confirming `P = d_{\max} > K = 2` under `Π`'s strict pass budget.
