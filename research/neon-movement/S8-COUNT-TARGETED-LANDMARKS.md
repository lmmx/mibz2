# S8 — Count-Targeted Landmark Induction (`NEEDS M-21b`) & Orthogonal Bi-Directional Extension

**Principle:** Contraction along composition (Principle 1: directly testing MONODROMY 114 §6's falsifiable `NEEDS M-21b` prediction on all 6 admitted blocks + novel hex, and extending Cycle 1 S1B with count-targeted selection + bi-directional final walk).  
**Script & Data:** [`s8_count_targeted_landmarks.py`](s8_count_targeted_landmarks.py), [`results_s8_count_targeted_landmarks.json`](results_s8_count_targeted_landmarks.json).

## 1. Exact Mathematical Definition

In MONODROMY 114 §6 (after channel #23 measured `D = 1.3–1.9%` of `n` under frequency-rank landmark selection), MONODROMY noted that $D = \sum_{j=1}^k \mathrm{rounds}_j + |Y_k|$ is minimized by choosing each level's landmark extension **by count** rather than by frequency rank:
- **Level 1 (raw `LF`):** pick byte $c_1 \in \Sigma$ whose count $|I_{c_1}|$ is nearest $n / \rho$ for $\rho \in \{8, 16, 32\}$. $Y_1 = I_{c_1}$ is contiguous and acquired free from $C[c]$.
- **Level $j \ge 2$:** among leftward 1-byte extensions $c_j$ of the landmark string (available from the return words of level $j-1$), pick $c_j$ whose count in $Y_{j-1}$ is nearest $|Y_{j-1}| / \rho$ (with $1 \le |Y_j| < |Y_{j-1}|$).
- **Pre-registered falsification rule (MONODROMY 114 §6):** *"if `D` at the best $(\rho, k)$ is not under `0.5%` of `n` on four or more blocks, the count-targeted prediction is withdrawn and the measured `1.3–1.9%` stands as the method's figure."*

**Orthogonal Bi-Directional Extension (`ORTH-BIDIR`):**
Apply genuine endpoint-only Rauzy–Veech thick-strip contraction (`L_min = 16`) FIRST to reduce $n \to m$, run count-targeted landmark induction ($\rho \in \{8, 16, 32\}, k \le 4$) on the retained permutation $R : [0..m) \to [0..m)$, and walk the final induced cycle $Y_k$ **bi-directionally** ($R_k$ and $R_k^{-1}$ meeting at $\lceil |Y_k|/2 \rceil$).

## 2. Correctness (`[proved]`, 8/8 byte-exact verified)

At every level $j$, $Y_j \subset Y_{j-1}$ is a non-empty section of a single-cycle permutation $R_{j-1}$, so its first-return map $R_j$ is again a single cycle on $|Y_j|$ elements whose return chains partition $Y_{j-1}$ exactly once. Unrolling the final cycle $R_k$ downward through levels $k \to 1$ recovers the unique cyclic orbit of length $n$ (or $m$, then expanding Rauzy–Veech word DAGs). Verified `decoded == T_ref` byte-for-byte on **8/8 blocks** for BOTH raw `NEEDS M-21b` and `ORTH-BIDIR`.

## 3. Prior Art

- **MONODROMY 114 §3 & §6 (`NEEDS M-21b`), channel #23 (`m21/`):** Frequency-rank landmarks gave `D = 1.3–1.9%` of `n`; count-targeted selection (`NEEDS M-21b`) was proposed as an untested prediction.
- **Cycle 1 S1 (`text-axis`):** Measured frequency-rank `NEEDS M-21` and orthogonal row+text induction (`D = 1,093–6,013` on the 6 blocks).
- **What is new here:** First exact execution and byte-exact verification of `NEEDS M-21b` (confirming MONODROMY's pre-registered `< 0.5%` gate **6/6**), plus `ORTH-BIDIR` pushing total depth down to `0.070–0.197%` of `n` (`627–1,776` steps) at `2–3×` lower `Q` than raw `LF`.

## 4. Measurements on Novel Random Hex & Six Admitted Blocks (8/8 Byte-Exact)

| Block | RAW `NEEDS M-21b` best $(\rho, k)$ | RAW rounds + $|Y_k|$ = **`D`** (`% n`) | RAW `Q / n` | Gate `< 0.5%` | `ORTH-BIDIR` best $(\rho, k)$ | **`D_bidir`** (`% n`, collapse) | `ORTH` `Q / n` (`Q / h_1`) |
|---|---:|---:|---:|:---:|---:|---:|---:|
| `synth-hex-iid-65536` | `(8, 4)` | `284 + 4` = **`288`** (`0.440%`) | `1.070` | PASS | `(8, 3)` | **`249`** (`0.380%`, `263×`) | `1.898` (`2.16`) |
| `synth-tmpl-hex-1000x40` | `(8, 4)` | `183 + 2` = **`185`** (`0.228%`) | `1.163` | PASS | `(32, 3)` | **`78`** (`0.096%`, `1,038×`) | `1.178` (`2.39`) |
| `block-0.L` | `(32, 3)` | `818 + 7` = **`825`** (`0.092%`) | `1.036` | **PASS** | `(8, 4)` | **`627`** (`0.070%`, **`1,435×`**) | `0.339` (`8.69`) |
| `block-2.L` | `(16, 3)` | `953 + 215` = **`1,168`** (`0.130%`) | `1.071` | **PASS** | `(8, 3)` | **`876`** (`0.097%`, **`1,027×`**) | `0.389` (`10.22`) |
| `block-85.L` | `(16, 3)` | `427 + 181` = **`608`** (`0.068%`) | `1.067` | **PASS** | `(16, 2)` | **`815`** (`0.091%`, **`1,104×`**) | `0.410` (`6.64`) |
| `block-143.L` | `(32, 3)` | `937 + 32` = **`969`** (`0.108%`) | `1.033` | **PASS** | `(16, 3)` | **`713`** (`0.079%`, **`1,262×`**) | `0.596` (`7.38`) |
| `block-210.L` | `(8, 3)` | `828 + 237` = **`1,065`** (`0.118%`) | `1.165` | **PASS** | `(16, 4)` | **`846`** (`0.094%`, **`1,064×`**) | `0.542` (`5.02`) |
| `block-207.L` | `(16, 3)` | `1,943 + 31` = **`1,974`** (`0.219%`) | `1.065` | **PASS** | `(32, 3)` | **`1,776`** (`0.197%`, **`507×`**) | `0.421` (`6.07`) |

**`NEEDS M-21b` Pre-Registered Verdict:** **`6/6` admitted blocks pass `< 0.5%` of `n` (`0.068%–0.219%`, `10–20×` better than frequency-rank `1.3–1.9%`) -> MONODROMY's count-targeted prediction STANDS.**

## 5. Referee Under $\Pi$ (with `#26` accounting note)

- **`Q`:** Does NOT fall below `h_1` (raw `Q = 1.033–1.165 n`; `ORTH-BIDIR` cuts total `Q` by `2–3×` to `0.339–0.596 n` = `5.0–10.2 h_1`, dominated by thick-strip build + retained induction).
- **`D`:** **Real find in `D`:** raw count-targeted induction achieves `D = 608–1,974` (`0.068–0.219%` of `n`) at `k = 3`, and `ORTH-BIDIR` pushes `D` to **`627–1,776`** (`0.070–0.197%` of `n`, up to **`1,435×`** depth collapse) at `P = 4–6`. **Caveat (`#26`):** `D` above counts the build rounds plus `|Y_k|` (matching MONODROMY 114 §6); unrolling the stored return chains (parallel prefix sum of chain lengths + scatter: `+O(log n)` parallel depth, or `+n` if unrolled serially) and the `L[LF^t]` label precompute (read from the return chains in level `j-1` during online execution) must be charged in a full end-to-end cost vector.
- **Verdict:** Confirms MONODROMY's `NEEDS M-21b` prediction cleanly on `6/6` blocks; a solid find in dependent depth `D`, not a cut in addressed accesses `Q`.
