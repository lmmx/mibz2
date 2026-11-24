# E2 — In-Stage $\mathbb{N}^\Sigma$ Semiring Enrichment: Free 2-Gram Interval Lifting $M_2[a, b]$

**Principle:** Enrichment along composition (Principle 2, answering MONODROMY 115 §4's open question on whether the free `P = 1, Q = 0` prefix-histogram snapshot $M_2[a, b]$ has any non-macro-edge use).  
**Script & Data:** [`e2_prefix_histogram_m2.py`](e2_prefix_histogram_m2.py), [`results_e2_prefix_histogram_m2.json`](results_e2_prefix_histogram_m2.json).

## 1. Exact Mathematical Definition

In a single streaming pass over `IN` (`P = 1, Q = 0, s = |\Sigma| + 1`), maintaining the prefix-histogram vector $\mathrm{occ}[0..|\Sigma|-1] \in \mathbb{N}^\Sigma$ and snapshotting it at the $|\Sigma|$ F-bucket boundaries $C[a]$ yields the exact 2-gram count matrix $M_2[a, b] = |\{x \in I_a : L[x] = b\}|$ and exact 2-gram image intervals:
$$I_{ba} = \mathrm{LF}\big(I_a \cap L^{-1}(b)\big) = \big[C_2[b, a],\; C_2[b, a] + M_2[a, b]\big), \qquad C_2[b, a] = C[b] + \sum_{a' < a} M_2[a', b].$$

We test three exact algebraic riders derived solely from $(M_2, C_2, \mathrm{IN})$ with `Q_acq = 0`:
1. **Hapax-bigram resolution ($M_2[a, b] = 1$):** $\mathrm{LF}(x) = C_2[b, a]$ directly from $(F[x], L[x])$.
2. **Monochromatic-image lone-row bundling:** Whenever $I_{ba}$ lies wholly inside a single maximal run $R_d = [r_s, r_s + r_w)$ of $L$ (byte $d$), *all* $M_2[a, b]$ rows in $I_a \cap L^{-1}(b)$ — even scattered width-1 lone rows — share the same 2nd byte $d$ and compose directly via
   $$\mathrm{LF}^2(x) = C[d] + \mathrm{rank}_d(r_s) + \big(\mathrm{LF}(x) - r_s\big),$$
   emitting $(b, d)$ in one 2-step edge during Pass 2 (`Q_acq = 0`, never touching $I_{ba}$).
3. **Free 2-gram piecewise-linear IFS predictor:** $\widehat{\mathrm{LF}}_2(x) = C_2[b, a] + \lfloor (x - C[a]) M_2[a, b] / |I_a| \rfloor$, sharpening S6's 1-gram predictor using $M_2$'s exact per-F-bucket slopes.

## 2. Correctness (`[proved]`, 8/8 byte-exact verified)

**Theorem E2.1 (Soundness of $M_2$ Monochromatic-Image 2-Step Bypass — proved).**
If $x \in I_a$, $L[x] = b$, $y = \mathrm{LF}(x) \in I_{ba} \subseteq [r_s, r_s + r_w)$ with $L|_{[r_s, r_s + r_w)} \equiv d$, and $y \neq p$, then $\mathrm{LF}$ translates $[r_s, r_s + r_w)$ rigidly onto $[C[d] + \mathrm{rank}_d(r_s),\, C[d] + \mathrm{rank}_d(r_s) + r_w)$, so $\mathrm{LF}^2(x) = C[d] + \mathrm{rank}_d(r_s) + (y - r_s)$ and $(L[x], L[\mathrm{LF}(x)]) = (b, d)$. Guarding $y \neq p$ guarantees the walk never skips `primary_index`, so replacing $x \to y$ by the 2-byte edge $x \to \mathrm{LF}^2(x)$ tiles the single cycle identically. $\square$

*Verification:* Asserted `decoded == T_ref` byte-for-byte on all **8/8 blocks** (`2` synthetic novel hex + `6` admitted Wikidata blocks).

## 3. Prior Art

- **MONODROMY record 115 §1–2 & §4:** Noted $M_2$ comes free in the `P = 1` build sweep and asked whether it has any use beyond macro-edges.
- **Cycle 1 S6 (`ifs-discrepancy`):** Measured 1-gram $B$-bucket IFS target prediction.
- **What is new here:** First exact construction, byte-exact 2-step bypass decoder, quantitative measurement of genuine retained-to-retained bypasses on $M_{16}$, and 1-gram vs 2-gram IFS residual entropy comparison across all 8 blocks.

## 4. Measurements on Novel Random Hex & Six Admitted Blocks (8/8 Byte-Exact)

| Block | $M_2$ size | Raw `Q_run / n` (`M_2` bypass) | `h_1` in mono $I_{ba}$ | Genuine $M_{16}$ saved (`% h_1`) | 1-gram -> 2-gram IFS med `$|\varepsilon|$` | 1-gram -> 2-gram IFS $H(\varepsilon)$ (bits) |
|---|---:|---:|---:|---:|---:|---:|
| `synth-hex-iid-65536` | 1.0 KB | `1.000` (`0.0%`) | `0.00%` | `0` (`0.00%`) | `16 -> 4` (`4.0×`) | `6.77 -> 4.71` (`-2.06 b`) |
| `synth-tmpl-hex-1000x40` | 3.4 KB | `0.827` (`17.3%`) | `2.40%` | `0` (`0.00%`) | `349 -> 8` (`43.6×`) | `11.48 -> 7.41` (`-4.07 b`) |
| `block-0.L` | 31.7 KB | `0.903` (`9.7%`) | `2.94%` | `850` (`2.42%`) | `3,678 -> 414` (`8.9×`) | `14.87 -> 12.18` (`-2.69 b`) |
| `block-2.L` | 29.6 KB | `0.919` (`8.1%`) | `2.68%` | `1,109` (`3.24%`) | `3,725 -> 383` (`9.7×`) | `14.84 -> 12.12` (`-2.72 b`) |
| `block-85.L` | 35.3 KB | `0.925` (`7.5%`) | `1.97%` | `1,429` (`2.57%`) | `3,892 -> 436` (`8.9×`) | `14.89 -> 12.15` (`-2.74 b`) |
| `block-143.L` | 36.9 KB | `0.926` (`7.4%`) | `2.38%` | `1,457` (`2.01%`) | `3,977 -> 303` (`13.1×`) | `14.87 -> 11.94` (`-2.93 b`) |
| `block-210.L` | 36.9 KB | `0.934` (`6.6%`) | `1.47%` | `1,282` (`1.32%`) | `3,872 -> 325` (`11.9×`) | `14.88 -> 11.97` (`-2.91 b`) |
| `block-207.L` | 35.3 KB | `0.912` (`8.8%`) | `2.90%` | `1,259` (`2.02%`) | `4,076 -> 306` (`13.3×`) | `14.92 -> 12.03` (`-2.89 b`) |

## 5. Referee Under $\Pi$

- **`Q`:** `Q_acq = 0` (`P = 2`, `s = |\Sigma| + 2`, $S_{M_2} \le 37\text{ KB}$). On raw LF, `Q_run = 0.903–0.934 n`; on Open's $M_{16}$, genuine retained-to-retained savings are `1.32–3.24%` of $h_1$ on Wikidata and **`0.00%` on novel random hex** (where $M_2[a, b] \approx n/256 \gg 1$, so $I_{ba}$ is never monochromatic).
- **`D`:** Unchanged order (`D = Q_run`).
- **`P`, `S`:** `P = 2` (within `K = 2`), $M_2$ costs only `30–37 KB` (L1/L2-resident) and cuts linear IFS target error by `9–13×` (`2.7–2.9` bits/row) for free.
- **Verdict:** Settles MONODROMY 115's $M_2$ question cleanly: $M_2$ gives a genuine `1.3–3.2% h_1` free bypass on structured Wikidata and a `9–13×` free IFS sharpening, but **`0.00%` contraction on novel random hex** — not a `Q` breakthrough.
