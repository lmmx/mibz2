# Sense 4 (`independent-set`): Lateral Mathematics #1 — Miller–Reif & Cole–Vishkin Recursive Independent-Set Contraction

## 1. Exact Definition of "Moves Together"

In parallel graph/tree algorithms (Miller–Reif 1985; Cole–Vishkin 1986), a hard serial chain of length `m` becomes `O(m)` total-work contraction by breaking symmetry **without knowing text order** and splicing out an **independent set** of nodes simultaneously:

- Let `(V_k, R_k)` be the active cycle at round `k` (`V_0 = M`, `R_0 = R`, either `Open`'s retained map `m_{16}` or raw `LF` on `n` rows).
- Select a deterministic, text-order-blind **independent set** `I_k \subset V_k \setminus \{\mathrm{mark}\}` (meaning `u \in I_k \implies R_k(u) \notin I_k` and `R_k^{-1}(u) \notin I_k`) using one of two local rules:
  - **4A (Recursive Local-Minima / Valley Rule):** `I_k = \{u \in V_k \setminus \{\mathrm{mark}\} : u < R_k(u) \;\land\; u < R_k^{-1}(u)\}`.
  - **4B (Cole–Vishkin 1986 Bit-Difference Symmetry Breaking):** Define the 1-round proper coloring `cv(u) = 2\cdot \mathrm{lsb}(u \oplus R_k(u)) + \mathrm{bit}(u, \mathrm{lsb}(u \oplus R_k(u))) \in [0, 2\lceil\log_2 m\rceil)`, and set `I_k = \{u \in V_k \setminus \{\mathrm{mark}\} : cv(u) < cv(R_k(u)) \;\land\; cv(u) < cv(R_k^{-1}(u))\}`.
- **What "moves together" means:** Every `u \in I_k` moves together with its immediate predecessor `p_u = R_k^{-1}(u)` as one conflict-free atomic splice `p_u \to u \to R_k(u) \implies p_u \to R_k(u)`.

## 2. Composition Rule, Proofs, and Exactness

- **Composition rule (Phase 1 — Forward Contraction):** Because `I_k` is independent, `p_u` and `s_u = R_k(u)` are guaranteed to survive in `V_{k+1} = V_k \setminus I_k`. All `|I_k|` splices execute simultaneously in ONE conflict-free sort-merge round (`Q_{\mathrm{contract}} = 0`, `D_{\mathrm{contract}} = 0`):
  - `R_{k+1}(p_u) = s_u`;
  - `w_{k+1}(p_u) = w_k(p_u) \cdot w_k(u)` (stored as a 1-node DAG concat; weight `wt(p_u) \leftarrow wt(p_u) + wt(u)`);
  - log the splice triple `(p_u, u, wt_{\mathrm{old}}(p_u))`.
- **Phase 2 — Miller–Reif Reverse Unspooling (`Q_{\text{run,lone}} = 0`):** When `|V_L| = 1` (`V_L = \{\mathrm{mark}\}`), set `\mathrm{pos}[\mathrm{mark}] = 0` and replay the splice logs in reverse (`k = L-1 \dots 0`): for each logged `(p_u, u, wt_{\mathrm{old}})`, set `\mathrm{pos}[u] = \mathrm{pos}[p_u] + wt_{\mathrm{old}}`. One final base-256 radix sort by `\mathrm{pos}[u]` orders all `m` transitions in exact backward text order, and one sequential scan emits `T`.
- **Theorem 4.1 (Exactness & `O(m)` Geometric Total Work — PROVED, both Phase-1 DAG & Phase-2 Reverse-Unspool verified byte-exact on 8/8 blocks):**
  1. *Exactness:* Independence of `I_k` implies no splice modifies a deleted node's predecessor or successor, preserving the single-cycle invariant and the exact cyclic word concatenation. Reverse unspooling assigns each `u` its exact cyclic distance `\mathrm{pos}[u] \in \{0,\dots,m-1\}` from `mark` by induction on `k`.
  2. *Why Cole–Vishkin (4B) avoids monotone stalls (corrected per #24/#26):* On a monotone stretch `x \to x+1 \to x+2 \to \dots`, `u < R_k(u)` has no local minima until the stretch ends, stalling Local-Minima (`88\text{--}270` rounds on Wikidata). Under 1-round Cole–Vishkin, `\mathrm{lsb}(x \oplus (x+1))` breaks monotone runs; however, the observed `2.94\text{--}2.97\,m` total element-touch density reflects random-label local-minima density (`1/3 \implies 3.00\,m`) on these permutations rather than a worst-case CV guarantee (1-round CV with `2 \log m` colours bounds independent-set density only by `\approx 1/(4 \log m)`). `\square`

## 3. Prior Art

- Neon N-20 C4 (single-level unimodal arches on raw `LF` and `m_{16}`, equivalent to Astra investigation 1 / SA-IS LMS cuts).
- Miller & Reif (1985) parallel tree/list contraction; Cole & Vishkin (1986) deterministic coin-tossing; Anderson & Miller (1988); Chiang et al. (1995) external-memory list ranking (CGGTVV).
- **Correction (#24/#26):** Recursive independent-set contraction is classic external-memory list ranking (CGGTVV), not a new sense; the `2.94\text{--}2.97\,m` figure is empirical random-label density on these blocks, not a universal worst-case CV law.

## 4. Measurements on Novel Random Hex and the Six Admitted Blocks

Script: [`s4_independent_set.py`](s4_independent_set.py); raw JSON: [`results_s4_independent_set.json`](results_s4_independent_set.json). Both Phase-1 root-DAG expansion and Phase-2 reverse-unspool rank emission passed `decoded == T_ref` on all 8 blocks (plus raw `LF` on `synth-hex-iid` and `block-0.L`).

| Block | `m_{16}` | 4A Local-Min rounds / `r_1` / touches / `P_{\mathrm{bytes}}` (1ph) | 4B Cole–Vishkin rounds / `r_1` / touches / `P_{\mathrm{bytes}}` (1ph) | 4B Full 2-phase `P_{\mathrm{bytes}}` (vs S3) |
|---|---:|---:|---:|---:|
| `synth-hex-iid-65536` | 65,536 | 66 / 0.676 / `3.56 m` / 341.9 | 38 / 0.643 / `2.95 m` / 283.1 | 596.8 (vs 1,518.6) |
| `synth-tmpl-hex-1000x40` | 47,001 | 42 / 0.691 / `3.78 m` / 158.0 | 38 / 0.642 / `2.94 m` / 122.9 | 259.3 (vs 652.8) |
| `block-0.L` | 88,290 | 88 / 0.674 / `3.75 m` / 35.4 | 36 / 0.643 / `2.94 m` / 27.7 | 59.1 (vs 156.8) |
| `block-2.L` | 109,542 | 79 / 0.670 / `3.76 m` / 43.9 | 37 / 0.643 / `2.94 m` / 34.4 | 73.0 (vs 195.9) |
| `block-85.L` | 155,124 | 142 / 0.680 / `4.02 m` / 66.6 | 39 / 0.644 / `2.94 m` / 48.7 | 103.1 (vs 289.5) |
| `block-143.L` | 193,449 | 154 / 0.675 / `3.77 m` / 77.9 | 40 / 0.643 / `2.94 m` / 60.7 | 128.2 (vs 364.4) |
| `block-210.L` | 202,025 | 137 / 0.681 / `3.96 m` / 85.3 | 46 / 0.644 / `2.95 m` / 63.5 | 134.3 (vs 381.2) |
| `block-207.L` | 134,609 | 270 / 0.673 / `4.07 m` / 58.4 | 36 / 0.645 / `2.95 m` / 42.4 | 89.9 (vs 249.6) |
| *Raw `LF`: `block-0.L`* | *899,931* | *105 / 0.658 / `3.77 n`* | *44 / 0.654 / `2.97 n`* | *`P \approx 600` sweeps* |

## 5. Referee Verdict in `(Q, D, P, S)` under `Π`

- **Contraction on novel random hex:** Contracts the retained graph via classic external-memory list ranking (`2.94\text{--}2.97\,m` empirical touches on these blocks, matching random-label local-minima density `3.00\,m`). On `m_{16}`, total `Q` still includes thick-strip acquisition (`Q_{\mathrm{acq}} \ge m` on hex / `2.4\,m_{16}` on block-0); literal `Q = 0, D = 0` holds only on raw `LF` at `P \approx 600 \gg K = 2` sweeps.
- **Where the cost goes:** Each shrinking sort-merge round counts as `2 p_{\mathrm{radix}} + 2 = 8` sweeps (`P \approx 600 \gg K = 2`). In streamed byte volume, `P_{\mathrm{bytes}}` is `59\text{--}134` (`2.6\text{--}2.8\times` below S3). Peak `S` is `4.9\text{--}9.8\text{ MB}` on `m_{16}`.
- **Verdict:** Classic external-memory list ranking relocating `Q` into `P \approx 600 \gg K = 2` sweeps; not a new sense and not a breakthrough at `K = 2`.
