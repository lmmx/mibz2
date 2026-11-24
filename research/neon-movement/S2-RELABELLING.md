# Sense 2 (`relabelling`): Moving Together Up to a Consistent Relabelling

## 1. Exact Definition of "Moves Together"

Rauzy–Veech requires exact byte equality `L[x] = L[x+1]` on adjacent rows. Avoiding the already-killed post-BWT alphabet quotient (`π(L)`, Carbon C-3 / kitchen-sink Cycles 7–8), we test three genuinely distinct mathematical senses of moving together up to a relabelling:

- **2A (Period-`p` Strided / Arithmetic-Progression Rauzy–Veech, `p \in \{2,3,4\}`):** Rows in a maximal non-constant periodic segment `L[s + j] = L[s + (j \bmod p)]` (`0 \le j < m`, minimal period `p \ge 2`, `m \ge 2p`) are **all width-1 lone rows** in Rauzy–Veech (`p = 1`). Partition `[s, s+m)` into its `p` residue progressions `C_r = \{s + r + t\cdot p : 0 \le t < q_r\}` (`0 \le r < p`). Rows in `C_r` **move together with stride `p`** under cyclic byte relabelling `j \mapsto j + 1 \pmod p`.
- **2B (Monotone Ramp Relabelling):** Adjacent rows satisfying `L[s + j] = c_0 + j\cdot \delta` (`\delta \in \{+1, -1\}`, length `\ge 3`), e.g. sequential counter digits `'0','1','2',...`. Here the emitted byte is a linear function of the parameter `w(j) = c_0 + j\cdot \delta`.
- **2C (Baker `S_\Sigma`-Equivariant / Parameterized `k`-Step Trajectories):** Two lone rows `x, y \in J_1` **move together at height `k` up to consistent alphabet relabelling** (`x \sim_{k, S_\Sigma} y`) iff there exists a bijection `\phi \in S_\Sigma` such that `L[\mathrm{LF}^j(y)] = \phi(L[\mathrm{LF}^j(x)])` for all `0 \le j < k`.

## 2. Composition Rules, Proofs, and Obstruction Theorem

- **Theorem 2A (Exact Affine Transport on Periodic Lone-Row Progressions — PROVED & byte-verified 8/8):**
  Let `[s, s+m)` have period `p` and let `\mu_r = \#\{0 \le j < p : L[s+j] = L[s+r]\}` be the multiplicity of byte `c_r = L[s+r]` in one period. Then for every `0 \le t < q_r`:
  $$\mathrm{LF}(s + r + t\cdot p) \;=\; \mathrm{LF}(s + r) + t \cdot \mu_r.$$
  *Proof.* Every half-open period `[s+r+t\cdot p,\; s+r+(t+1)\cdot p)` is a cyclic shift of `L[s..s+p-1]`, hence contains exactly `\mu_r` occurrences of `c_r`. Since `\mathrm{LF}(x) = C[c_r] + \mathrm{rank}_{c_r}(x)`, `\mathrm{rank}_{c_r}` increments by exactly `\mu_r` per step `t \to t+1`. When `\mu_r = 1` (e.g. `a,b,a,b,\dots`), `\mathrm{LF}` maps the **scattered** progression `C_r` onto a **contiguous** interval `[\mathrm{LF}(s+r),\; \mathrm{LF}(s+r)+q_r)`. Affine maps `t \mapsto a + b\cdot t` close under composition (`(\mathbb{Z}, +) \rtimes \mathbb{Z}_{>0}`), strictly generalizing Rauzy–Veech (`b = 1`). `\square`
  - Verified exact on every single periodic segment across all 8 blocks, and byte-exact decode (`decoded == T_ref`) using ONLY the affine rule inside periodic segments.
- **Theorem 2C (Bell-Number Shape Collapse vs. `S_\Sigma`-Equivariance Obstruction — PROVED):**
  1. *Shape collapse:* The number of `S_\Sigma`-equivalence classes of `k`-step backward words is at most the `k`-th Bell number `B_k` (`B_4 = 15`, `B_6 = 203`, `B_8 = 4{,}140`), independent of `|\Sigma|` and regardless of randomness.
  2. *Equivariance obstruction:* Knowing `\phi(L[x]) = L[y]` and `\mathrm{LF}(x)` determines `\mathrm{LF}(y)` iff the bucket-rank discrepancy
     $$\delta_\phi(x, y) \;=\; \bigl(\mathrm{LF}(y) - C[\phi(c)]\bigr) - \bigl(\mathrm{LF}(x) - C[c]\bigr)$$
     is known. Under i.i.d. uniform symbols on `n` rows, `\mathrm{rank}_{\phi(c)}(y) - \mathrm{rank}_c(x)` fluctuates as `\Theta(\sqrt{n/|\Sigma|})` even at `x \approx y`, and on inhomogeneous texts (Wikidata) as `\Theta(n)`. Thus `S_\Sigma` contracts word *templates* (saving only repeated-symbol slots inside a `k`-mer) but **cannot transport row addresses** because `\mathrm{LF}` is not `S_\Sigma`-equivariant. `\square`

## 3. Prior Art

- Carbon C-3 (`research/carbon-new-basis`, Theorem QS-1) and kitchen-sink Cycle 7 line 106 / Cycle 8 (post-BWT alphabet quotienting `\pi(L)` and functorial non-commutativity `\pi(\mathrm{BWT}(T)) \ne \mathrm{BWT}(\pi(T))`).
- MONODROMY 114 Sense G (alphabet reordering) and Sense J (linear address prediction); Baker (1993) parameterized string matching.
- **New here:** Period-`p` strided affine Rauzy–Veech (2A) on lone rows, monotone ramp census (2B), and exact Bell-number collapse + rank-discrepancy obstruction measurements (2C).

## 4. Measurements on Novel Random Hex and the Six Admitted Blocks

Script: [`s2_relabelling.py`](s2_relabelling.py); raw JSON: [`results_s2_relabelling.json`](results_s2_relabelling.json). All 8 blocks pass byte-exact verification.

| Block | `h_1` | 2A Periodic (`p=2..4`) `w_1` in / saved UB | 2B Ramps (`\ge 3`) `w_1` | 2C `k=8` Exact `\to` Param classes (collapse) | 2C `k=8` Exact / Param shared `h_1` | 2C Median rank disc. `\delta_\phi` |
|---|---:|---:|---:|---:|---:|---:|
| `synth-hex-iid-65536` | 57,458 | 960 (1.67%) / 469 (**0.82%**) | 986 (1.72%) | 57,457 `\to` **2,197** (`26.2\times`) | 0.0% / **98.9%** | 33 |
| `synth-tmpl-hex-1000x40` | 39,909 | 641 (1.61%) / 296 (**0.74%**) | 709 (1.78%) | 34,565 `\to` **1,802** (`19.2\times`) | 14.5% / **98.7%** | 256 |
| `block-0.L` | 35,116 | 2,223 (6.33%) / 936 (**2.67%**) | 450 (1.28%) | 29,570 `\to` **1,485** (`19.9\times`) | 19.0% / **98.7%** | 3,213 |
| `block-2.L` | 34,262 | 2,230 (6.51%) / 919 (**2.68%**) | 481 (1.40%) | 28,150 `\to` **1,440** (`19.6\times`) | 21.4% / **98.8%** | 3,124 |
| `block-85.L` | 55,504 | 3,507 (6.32%) / 1,496 (**2.70%**) | 871 (1.57%) | 44,228 `\to` **1,672** (`26.5\times`) | 24.2% / **99.2%** | 2,554 |
| `block-143.L` | 72,578 | 4,024 (5.54%) / 1,385 (**1.91%**) | 1,026 (1.41%) | 59,343 `\to` **1,799** (`33.0\times`) | 22.4% / **99.4%** | 1,664 |
| `block-210.L` | 97,054 | 4,159 (4.29%) / 1,636 (**1.69%**) | 1,454 (1.50%) | 80,438 `\to` **1,981** (`40.6\times`) | 20.4% / **99.5%** | 1,377 |
| `block-207.L` | 62,407 | 3,911 (6.27%) / 1,722 (**2.76%**) | 851 (1.36%) | 50,443 `\to` **1,768** (`28.5\times`) | 23.4% / **99.2%** | 1,863 |

## 5. Referee Verdict in `(Q, D, P, S)` under `Π`

- **2A (Period-`p` Strided Rauzy–Veech):** Genuine, exact generalization of Rauzy–Veech (`Q_{\mathrm{acq}} = 0`, `P_{\mathrm{acq}} = 1`, `s = 256`) that **really does contract width-1 lone rows** into affine progressions without any hidden walk. However, on novel random hex only `0.82%` of `h_1` is saved (and `1.69\text{--}2.76%` on Wikidata), because random hex does not repeat periodically across adjacent suffix rows.
- **2B (Monotone Ramps):** Only `1.28\text{--}1.78%` of `h_1` (max length `4\text{--}6`), and target rows still jump across distinct `F`-buckets.
- **2C (`S_\Sigma`-Equivariance):** Produces a spectacular **`19.6\text{--}40.6\times` collapse** in word-shape classes (`98.7\text{--}99.5%` of `h_1` covered even on 100% novel random hex!), **but hits a proved Equivariance Obstruction**: median target-rank discrepancy `\delta_\phi` is `33` on 65k i.i.d. hex and `1{,}377\text{--}3{,}213` on Wikidata, so `\phi` cannot transport target addresses without an addressed lookup `Q = 1` per step.
- **Verdict:** Relabelling yields one genuine exact lone-row contraction (2A, `\le 2.8%` of `h_1`) and one sharp structural negative (2C, shape contracts `99%` while `LF`-transport fails equivariance). Neither lowers `Q` by a constant fraction on novel hex.
