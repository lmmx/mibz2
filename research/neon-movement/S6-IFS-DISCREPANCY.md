# Sense 6 (`ifs-discrepancy`): Lateral Mathematics #3 — Iterated Function System (IFS) Contraction-Mapping Transport & Exact Discrepancy

## 1. Exact Definition of "Moves Together"

In fractal geometry and metric dynamical systems (Barnsley IFS; learned/PGM indexes), a map with constant average slope `p_c < 1` contracts distances geometrically. Viewing `\mathrm{LF}` on `[0, n)` through this lens:

- Define for each symbol `c \in \Sigma` (global `B = 1`) or each pair `(b, c)` on `B` equal macro-blocks of width `W = \lceil n / B \rceil` the **affine contraction**:
  $$f_{b,c}(x) \;=\; C[c] + \mathrm{chk}[b][c] + \frac{\mathrm{chk}[b+1][c] - \mathrm{chk}[b][c]}{W_b}\,(x - bW),$$
  where `\mathrm{chk}[b][c] = \mathrm{rank}_c(bW)` is acquired in one streaming pass over `IN` (`Q_{\mathrm{acq}} = 0`, `s = 256` cursors).
- **What "moves together" means:** All rows `x` inside macro-block `b` holding byte `L[x] = c` **move together under the single affine contraction `f_{b,c}`**, up to the exact **Brownian-bridge rank discrepancy**:
  $$\varepsilon_B(x) \;=\; \mathrm{LF}(x) - \operatorname{round}\!\bigl(f_{b, L[x]}(x)\bigr).$$

## 2. Composition Rule, Theorem, and Exactness

- **Composition rule:** Iterating `\mathrm{LF}(x) = f_{c_j}(x) + \varepsilon(x)` along `k` steps with emitted word `(c_0, \dots, c_{k-1})` yields:
  $$\mathrm{LF}^k(x) \;=\; (f_{c_{k-1}} \circ \dots \circ f_{c_0})(x) \;+\; \sum_{j=0}^{k-1} \left(\prod_{i=j+1}^{k-1} p_{c_i}\right) \varepsilon\!\bigl(\mathrm{LF}^j(x)\bigr).$$
- **Theorem 6.1 (IFS Contraction vs. Boundary-Error Wall — PROVED & verified on 8/8 blocks):**
  1. *(a) Initial-error annihilation:* Because `p_c < 1` (`p_c = 1/16` on uniform hex, `p_c \le 0.15` on Wikidata), any initial row-address perturbation `\Delta_0` (even `\Delta_0 = n/2`) decays geometrically as `\prod_{i=0}^{k-1} p_{c_i}`, dropping strictly below `0.05` rows by `k = 5` steps (`< 0.001` rows on all 6 Wikidata blocks!). Thus the affine word map `(f_{c_{k-1}} \circ \dots \circ f_{c_0})` forgets the starting row `x` completely after 5 steps.
  2. *(b) Boundary-error & unsmoothed-`L` wall:* The final term `j = k - 1` has contraction factor `1`, so `|\mathrm{LF}^k(x) - (f_{c_{k-1}} \circ \dots \circ f_{c_0})(x)|` never decays below the 1-step discrepancy `|\varepsilon(\mathrm{LF}^{k-1}(x))|` (`\Theta(\sqrt{n/|\Sigma|})` on i.i.d. hex, `\Theta(n)` globally on inhomogeneous Wikidata, and `\Theta(\sqrt{W \cdot p_c})` on `B`-block piecewise-linear IFS). Because `L` is uncorrelated across adjacent lone rows in novel hex, even a `1`-row prediction error reads the wrong byte `L[y \pm 1]`, forcing an exact correction at every step.
- **Exactness — PROVED (byte-exact verified on 8/8 blocks at `B = 256`):** Storing `(\mathrm{chk}, \varepsilon_B)` reproduces `\mathrm{LF}(x) = \operatorname{round}(f_{b, L[x]}(x)) + \varepsilon_B(x)` identically without ever reading `\mathrm{LF}[]`.

## 3. Prior Art

- MONODROMY 114 Sense J ("together up to a predicted address" — 1-line note on `\sqrt{w}` error); Ferragina–Manzini (2000) FM-index sampled rank checkpoints; Ferragina–Vinciguerra (2020) PGM-index; Neon N-10 (`SA[x]` smoothness audit).
- **New here:** Formalizing `LF` as a `1/|\Sigma|`-contracting IFS, proving and empirically separating **initial-error annihilation** (`\Delta_0 = 450{,}000 \to 0.0000` by `k = 5`) from the **uncontracted final-step boundary discrepancy** (`1{,}355\text{--}1{,}788` rows), and measuring exact target-address bit compression (`20` bits `\to 2.1\text{--}3.4` bits) with byte-exact IFS+residual decode on all 8 blocks.

## 4. Measurements on Novel Random Hex and the Six Admitted Blocks

Script: [`s6_ifs_discrepancy.py`](s6_ifs_discrepancy.py); raw JSON: [`results_s6_ifs_discrepancy.json`](results_s6_ifs_discrepancy.json). All 8 blocks pass byte-exact decode at `B = 256`.

| Block | `B=1` `w_1` `|\varepsilon|` med / max / bits | `B=256` `w_1` `|\varepsilon|` med / max / zero% / bits | `B=1024` `w_1` `|\varepsilon|` med / zero% / bits | `\Delta_0 = n/2` init error (`k=1 \to k=5`) | Boundary error (`k=1 \to k=5`) |
|---|---:|---:|---:|---:|---:|
| `synth-hex-iid-65536` | 16 / 93 / 5.39 | **1** / 9 / 26.6% / **2.11** | **1** / 41.4% / **1.69** | `2044.5 \to \mathbf{0.0312}` | `16.6 \to 16.3` |
| `synth-tmpl-hex-1000x40` | 274 / 4,289 / 9.22 | **1** / 72 / 23.8% / **2.22** | **1** / 39.0% / **1.74** | `1402.0 \to \mathbf{0.0038}` | `270.4 \to 300.5` |
| `block-0.L` | 1,093 / 31,653 / 11.39 | **10** / 847 / 5.4% / **4.83** | **3** / 12.8% / **3.39** | `6061.0 \to \mathbf{0.0000}` | `1040.9 \to 1355.2` |
| `block-2.L` | 1,147 / 32,333 / 11.35 | **10** / 818 / 5.0% / **4.91** | **3** / 12.7% / **3.42** | `5857.0 \to \mathbf{0.0000}` | `1132.2 \to 1382.6` |
| `block-85.L` | 1,367 / 34,852 / 11.59 | **8** / 861 / 5.5% / **4.70** | **3** / 12.3% / **3.40** | `6408.5 \to \mathbf{0.0001}` | `1315.9 \to 1505.9` |
| `block-143.L` | 1,386 / 34,407 / 11.71 | **10** / 845 / 5.4% / **4.85** | **3** / 12.5% / **3.41** | `6573.5 \to \mathbf{0.0006}` | `1425.8 \to 1787.6` |
| `block-210.L` | 1,440 / 35,969 / 11.74 | **9** / 861 / 5.0% / **4.73** | **3** / 13.3% / **3.24** | `5947.5 \to \mathbf{0.0001}` | `1496.3 \to 1650.1` |
| `block-207.L` | 1,083 / 34,185 / 11.46 | **9** / 858 / 5.5% / **4.86** | **3** / 13.9% / **3.35** | `5451.0 \to \mathbf{0.0002}` | `1066.1 \to 1485.1` |

## 5. Referee Verdict in `(Q, D, P, S)` under `Π`

- **What it contracts:** Piecewise-linear IFS (`B = 256\text{--}1024`) compresses the **bit-width of every lone row's `LF` target** from `20` bits down to **`1.7\text{--}2.1` bits** on novel random hex and **`3.2\text{--}4.9` bits** on Wikidata (reducing working memory `S` for storing `LF` by `4\text{--}6\times` at `Q_{\mathrm{acq}} = 0, P_{\mathrm{acq}} = 1`).
- **Where the cost goes:** Under `Π`, exactness still requires either (i) loading `\varepsilon_B[x]` (`Q = n, D = n, P = 2`), or (ii) seeking to checkpoint block `b` and scanning `\le W/2` bytes (`Q = n, D = n`, plus `W/2` local scan).
- **Verdict:** Compresses target **space `S`** (`4\text{--}6\times`) and proves sharply why metric contraction (`16^{-k}`) cannot eliminate `Q` on unsmoothed novel hex: initial error vanishes in 5 steps, but the final step's uncontracted Brownian-bridge residual stays `O(\sqrt{W \cdot p_c})`.
