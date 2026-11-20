# Carbon Kitchen Sink — Cycle 4: Ribbon-Graph / Veech Surface Duality, Stallings Folding, Schur-Complement Optimality, and Post-BWT Semigroups

*20 November 2026 — Branch `research/carbon-kitchen-sink` (autonomous cycle 4)*

## 1. Overview of Cycle 4

Following the mathematical-maximalism charter with economical, theorem-first falsifiers, Cycle 4 investigates four previously open structural doorways in `FRONTIER.md`:

1. **Grothendieck Ribbon Graphs / Veech Zippered-Rectangle Surfaces & Bestvina–Handel Folding (`FRONTIER.md` Leads 2 & 3):** constructs the canonical oriented ribbon graph $\Sigma_{\mathrm{rib}} = (D, \sigma, \alpha)$ on the $2|E_{\mathrm{ov}}|$ darts of the overlap multigraph $G_{\mathrm{ov}}$, proves an **Exact Wall-Boundary Face Bijection** ($\mathrm{faces}(\sigma \circ \alpha) \cong \mathrm{cycles}(\Pi_{\mathrm{wall}})$ on $2r$ run walls, verified across `1,508` small primitive BWTs + all `6` Wikidata blocks), explains why $\Sigma_{\mathrm{rib}}$ has **`5×` lower genus** (`5,448–13,336`) than the row-level origami lift (`25,590–64,364`) with only **`60–484` faces** (including one giant face of `53%–98.7%` of all darts), and proves **Ribbon Metric-Blindness** (explicit equal-$(n, C, p)$ primitive witness at $n = 10, r = 4$) and **Stallings Fold Collapse** (`99.87%–99.99%` same-byte incoming switch adjacencies).
2. **Meyer Non-Commutative Schur Complements & Universal Knapsack Optimality of `Open` (`FRONTIER.md:134`):** proves that every word-homogeneous affine strip of $(x \mapsto (\mathrm{LF}(x), L[x]))$ on $[0, n) \setminus \{p\}$ lies inside a single maximal sub-run, so across **all** $2^{n-1}$ subsets $S \subseteq [0, n) \setminus \{p\}$ with $\le H$ strips, retained size $|J| = n - |S|$ is bounded below by the exact greedy run-width knapsack $m^*(H)$ (`verified` on `72,816` subset configurations; `Open<16>` achieves **`0` rows of slack** on all `6 / 6` blocks, whereas alphabet-subset Schur complements leave `3.3×–4.9×` more retained rows).
3. **Tropical Min-Plus Duality of Irreducible $\mathrm{LCP}_F$ vs Separation Depth (`FRONTIER.md:131-132`):** proves and verifies across all **`4,889,423` reducible rows** on the six blocks that cyclic rotation $\mathrm{LCP}_F[y] = \min(n, 1 + \mathrm{LCP}_F[\Psi(y)])$ outside $\overline{C}$ (dual to $\ell(x) = 1 + \ell(\mathrm{LF}(x))$ outside $\overline{D}$), with $\sum_{y \in \overline{C}} \mathrm{LCP}_F[y] = 0.964n\text{--}1.393n$ (`3.1%–4.1%` of Kärkkäinen–Kempa–Piątkowski's $2n \log_2 r$ bound) and **median irreducible LCP = `3` bytes on all 6/6 blocks** ($16^3 = 4,096$ hex-GUID collision scale).
4. **Post-BWT Krohn–Rhodes / Affine-Linear $\mathbb{F}_2[x]/(P_{32})$ Transducer Lift (`FRONTIER.md` Foundational Question 4):** proves that bzip2's post-BWT pipeline $\Sigma^* \xrightarrow{\mathcal{T}_{\mathrm{RLE1}}} \Sigma^* \xrightarrow{(\mathrm{len}, \mathrm{CRC}_{32})} \mathbb{N} \ltimes \mathbb{F}_2^{32}$ lifts any Straight-Line Program of size $G$ to a 5-state finite transducer over $\mathbb{N} \ltimes \mathbb{F}_2[x]/(P_{32}(x))$ (`verified` bit-exact across all `6 / 6` blocks), while measuring only **`23–83` `RLE1` triggers per 900 KB block** (`99.71%–99.93%` of 32B windows pure identity).

---

## 2. Ribbon Graphs, Veech Surfaces, and Stallings Folding (`FRONTIER.md` Leads 2 & 3)

### 2.1 Construction of the BWT Ribbon Surface $\Sigma_{\mathrm{rib}}$

Let $G_{\mathrm{ov}} = (V, E_{\mathrm{ov}})$ be the overlap multigraph on the $r$ maximal runs $V = \{0, \dots, r-1\}$, where each run $u$ has incoming overlap edges $\mathrm{in}(u) = (e^{\mathrm{in}}_{u,0}, \dots, e^{\mathrm{in}}_{u, a_u - 1})$ ordered left-to-right across $[d_u, d_u + w_u)$ and outgoing overlap edges $\mathrm{out}(u) = (e^{\mathrm{out}}_{u,0}, \dots, e^{\mathrm{out}}_{u, b_u - 1})$ ordered left-to-right across $[c_u, c_u + w_u)$.

Equip the $2|E_{\mathrm{ov}}|$ darts $D = \{2e, 2e+1 : e \in E_{\mathrm{ov}}\}$ ($2e$ = tail at bottom of $\mathcal{R}_u$, $2e+1$ = head at top of $\mathcal{R}_v$, $\alpha(d) = d \oplus 1$) with the counter-clockwise rectangle-boundary rotation $\sigma \in S_D$:
$$\sigma_u = \big(2 e^{\mathrm{in}}_{u, a_u - 1} + 1,\; \dots,\; 2 e^{\mathrm{in}}_{u, 0} + 1,\; 2 e^{\mathrm{out}}_{u, 0},\; \dots,\; 2 e^{\mathrm{out}}_{u, b_u - 1}\big).$$

### 2.2 Theorem 1A — Exact Wall-Boundary Face Bijection & Veech Genus Formula (**proved & verified**)

Define the **wall-return permutation** $\Pi_{\mathrm{wall}}$ on the $2r$ vertical run walls $\mathcal{W} = \{L_0, \dots, L_{r-1}, R_0, \dots, R_{r-1}\}$ ($L_u$ descending from $d_u^+$ to $c_u^+$; $R_v$ ascending from $(c_v + w_v)^-$ to $(d_v + w_v)^-$):
$$\Pi_{\mathrm{wall}}(L_u) = \begin{cases} L_v & \text{if } c_u = d_v \in \overline{D}, \\ R_{u'} & \text{if } c_u \notin \overline{D},\; c_{u'} + w_{u'} = c_u, \end{cases} \qquad \Pi_{\mathrm{wall}}(R_v) = \begin{cases} R_{u'} & \text{if } d_v + w_v = c_{u'} + w_{u'} \in \overline{C} \cup \{n\}, \\ L_{v+1} & \text{if } d_v + w_v \notin \overline{C} \cup \{n\}. \end{cases}$$

**Claim (proved).** $\Pi_{\mathrm{wall}}$ is a bijection on $\mathcal{W}$ whose cycles are in canonical 1-to-1 correspondence with the face cycles of $\varphi = \sigma \circ \alpha$ on $D$. For every primitive BWT ($b_0(G_{\mathrm{ov}}) = 1$), the oriented ribbon genus of $\Sigma_{\mathrm{rib}}$ is
$$g_{\mathrm{rib}} = \frac{|E_{\mathrm{ov}}| - r + 2 - |\mathrm{cycles}(\Pi_{\mathrm{wall}})|}{2} = \frac{k + 2 - F_{\mathrm{rib}}}{2},$$
where $k = r - |\overline{D} \cap \overline{C}|$ is the NOP-7 copy count.

*Proof.* Under $\sigma_u$, the two transitions $2e^{\mathrm{in}}_{u,0} + 1 \mapsto 2e^{\mathrm{out}}_{u,0}$ and $2e^{\mathrm{out}}_{u,b_u-1} \mapsto 2e^{\mathrm{in}}_{u,a_u-1} + 1$ traverse $L_u$ and $R_u$ respectively. Every remaining transition $2e^{\mathrm{in}}_{v,j} + 1 \mapsto 2e^{\mathrm{in}}_{v,j-1} + 1$ ($j > 0$, sitting at a pure codomain cut $c_u \in \overline{C} \setminus \overline{D}$) takes a dart arriving via $\alpha$ from $2e^{\mathrm{out}}_{u,0}$ (exiting $L_u$) directly to $2e^{\mathrm{in}}_{v,j-1}+1$, whose $\alpha$-image $2e^{\mathrm{out}}_{u', b_{u'}-1}$ immediately enters $R_{u'}$ ($c_{u'} + w_{u'} = c_u$). Symmetrically, every bottom transition $2e^{\mathrm{out}}_{u,j} \mapsto 2e^{\mathrm{out}}_{u,j+1}$ ($j < b_u - 1$, sitting at $d_{v+1} \in \overline{D} \setminus \overline{C}$) maps the exit of $R_v$ via $\alpha$ directly into $L_{v+1}$. At coincident endpoints $y \in (\overline{D} \cap \overline{C}) \cup \{0, n\}$, $e^{\mathrm{out}}_{u,0} = e^{\mathrm{in}}_{v,0}$ (giving $L_u \mapsto L_v$) and $e^{\mathrm{in}}_{v,a_v-1} = e^{\mathrm{out}}_{u',b_{u'}-1}$ (giving $R_v \mapsto R_{u'}$). Thus $\Pi_{\mathrm{wall}}$ is the exact first-return map of $\varphi = \sigma \circ \alpha$ to $\mathcal{W}$, and every dart orbit meets $\mathcal{W}$. Euler's formula $r - |E_{\mathrm{ov}}| + F_{\mathrm{rib}} = 2 - 2g_{\mathrm{rib}}$ with $|E_{\mathrm{ov}}| - r = k$ gives $g_{\mathrm{rib}}$. $\square$

- **Verified (`1,514 / 1,514` checks, 0 errors):** `verify_cycle4.py` verified `faces(sigma o alpha) == cycles(Pi_wall)` on all `1,508` primitive binary ($n \le 8$) and ternary ($n \le 6$) BWTs plus all `6` Wikidata blocks.
- **Six-block comparison (`probe_cycle4.py`, measured):**

| Block | $r$ | $|E_{\mathrm{ov}}|$ | Ribbon faces $F_{\mathrm{rib}}$ | Max face share of $2|E_{\mathrm{ov}}|$ | Ribbon genus $g_{\mathrm{rib}}$ | Origami genus $g_{\mathrm{orig}}$ | Genus ratio $g_{\mathrm{rib}} / g_{\mathrm{orig}}$ | Same-byte switch folds |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| `block-0.L` | 51,184 | 62,138 | **60** | `122,709 / 124,276` (**98.7%**) | **5,448** | 25,590 | **0.213×** | `10,940 / 10,954` (**99.87%**) |
| `block-2.L` | 54,532 | 67,755 | **75** | `71,774 / 135,510` (**53.0%**) | **6,575** | 27,263 | **0.241×** | `13,219 / 13,223` (**99.97%**) |
| `block-85.L` | 79,375 | 96,494 | **295** | `112,104 / 192,988` (**58.1%**) | **8,413** | 39,686 | **0.212×** | `17,112 / 17,119` (**99.96%**) |
| `block-143.L` | 108,335 | 135,299 | **294** | `216,908 / 270,598` (**80.2%**) | **13,336** | 54,167 | **0.246×** | `26,959 / 26,964` (**99.98%**) |
| `block-207.L` | 87,111 | 107,914 | **257** | `136,530 / 215,828` (**63.3%**) | **10,274** | 43,556 | **0.236×** | `20,800 / 20,803` (**99.99%**) |
| `block-210.L` | 128,732 | 155,480 | **484** | `188,365 / 310,960` (**60.6%**) | **13,133** | 64,364 | **0.204×** | `26,744 / 26,748` (**99.99%**) |

### 2.3 Theorem 1B — Ribbon Metric-Blindness & Stallings-Fold Collapse Obstructions (**proved & verified**)

1. **Ribbon Metric-Blindness (`FRONTIER.md` Lead 3 resolved):** Plain face traversal $\varphi = \sigma \circ \alpha$ is *not* $\mathrm{LF}$ in disguise (it walks $2|E_{\mathrm{ov}}| \approx 2.4r$ boundary darts in `60–484` cycles, independent of $n$). However, $(G_{\mathrm{ov}}, \sigma, \alpha, \{b_u\})$ is constant on the entire $\beta_1$-dimensional Veech chamber $\mathcal{C}(G_{\mathrm{ov}}, \sigma) = \{w \in \mathbb{Z}_{>0}^{E_{\mathrm{ov}}} : \partial w = 0\} \subset \mathbb{Z}^{E_{\mathrm{ov}}}$ (`\beta_1 = 10,955–26,965` degrees of freedom).
   - **Minimal equal-$(n, C, p)$ non-pumping witness (`verified` in `verify_cycle4.py`):** At $n = 10, r = 4, p = 0, C = [5, 5]$:
     $$W_0 = \mathtt{0000110111} \; (\mathrm{BWT} = \mathtt{1000111100}) \quad\text{vs}\quad W_1 = \mathtt{0001011011} \; (\mathrm{BWT} = \mathtt{1001111000})$$
     possess **100% identical byte-labelled ribbon graphs $(G_{\mathrm{ov}}, \sigma, \alpha, b_u)$, equal length $n = 10$, equal counts $C = [5, 5]$, and equal primary $p = 0$**, yet decode to different cyclic run orders (`0101` with 4 runs vs `010101` with 6 runs) because shifting 1 unit of width around $H_1(G_{\mathrm{ov}}; \mathbb{Z})$ reverses a Rauzy–Veech metric inequality!
2. **Stallings-Fold Collapse (`FRONTIER.md` Lead 2 resolved):** Under edge byte-labelling, `100%` of outgoing switch adjacencies at $u$ share byte $b_u$, and `99.87%–99.99%` of incoming switch adjacencies at $v$ share the same incoming byte (different bytes occur ONLY at the $\le \alpha - 1 \le 95$ bucket boundaries $C[a]$, in fact `3–14` times per block!). Consequently, Stallings folding collapses $G_{\mathrm{ov}}$ toward the $\alpha$-vertex alphabet quotient, erasing branch order; preserving order requires keeping the planar train-track weights and resolving switches via metric width subtraction (**which IS Rauzy–Veech splitting**).

---

## 3. Theorem 2 — Universal Schur-Complement Knapsack Optimality of `Open` (**proved & verified**, closing `FRONTIER.md:134`)

`FRONTIER.md:134` asked: *"Stochastic complementation (Meyer 1989), the matrix form of Open's elimination: are there other complements, with other retained sets, with cheaper structure?"*

**Theorem 2 (proved & verified on `72,816` subset configurations + `6 / 6` blocks).** Let $L \in \Sigma^n$, $p \in [0, n)$, and let $w'_{(1)} \ge \dots \ge w'_{(r')}$ be the sorted widths of the maximal constant-byte runs of $[0, n) \setminus \{p\}$. For **any** eliminated subset $S \subseteq [0, n) \setminus \{p\}$, let $k_{\mathrm{wh}}(S)$ be the number of maximal contiguous intervals $I \subseteq S$ on which $x \mapsto (\mathrm{LF}(x) - x, L[x])$ is constant (the minimum number of strips in any word-emitting auxiliary interval exchange eliminating $S$). Then:
$$|J| = n - |S| \;\ge\; m^*\big(k_{\mathrm{wh}}(S)\big) := n - \sum_{j=1}^{\min(k_{\mathrm{wh}}(S),\, r')} w'_{(j)}.$$

*Proof.* Every contiguous interval $I \subseteq S \subseteq [0, n) \setminus \{p\}$ on which $L[x]$ is constant is a sub-interval of one of the $r'$ maximal constant-byte runs $R'_1, \dots, R'_{r'}$ of $[0, n) \setminus \{p\}$ (adjacent runs carry distinct bytes $b_u \ne b_{u+1}$, and moreover across all `509,263` adjacent run pairs on the six blocks, `0` share the same shift $\delta_u = \delta_{u+1}$). If $S$ places $c_i \ge 0$ disjoint intervals inside $R'_i$ ($\sum_i c_i = k_{\mathrm{wh}}(S)$), the mass captured in $R'_i$ is at most $\mathbf{1}[c_i \ge 1] \cdot |R'_i|$, which is maximised for fixed $\sum c_i = H$ by setting $c_i = 1$ on the $H$ widest sub-runs. $\square$

- **Zero-gap optimality on all 6 blocks:** `Open<16>` achieves $|J| = m^*(H_{\mathrm{Open}})$ with **`0` rows of slack** on `6 / 6` blocks, whereas the best **alphabet-subset Schur complement** ($S = \{x : L[x] \in \Sigma_{\mathrm{elim}}\}$ at budget $\le H_{\mathrm{Open}}$ runs) leaves **`444,433–744,844` retained rows (`3.3×–4.9×` worse)**! Eliminating non-thick runs via bounded-depth DAG Schur complements requires $\Theta(r)$ run-graph construction, hitting Cycle 1's Ephemeral-Run Obstruction.

---

## 4. Theorem 3 & 4 — Tropical LCP Duality (`FRONTIER.md:131`) & Post-BWT `RLE1 ∘ CRC32` Transducer (`FRONTIER.md` Q4)

1. **Tropical Forward/Backward Reducibility Duality (`proved`, `verified` on `4,889,423 / 4,889,423` reducible rows):**
   - Forward separation depth $\ell(x) = \min\{t \ge 0 : \mathrm{LF}^t(x) \in \overline{D}\}$ obeys $\ell(x) = 1 + \ell(\mathrm{LF}(x))$ on $[0, n) \setminus \overline{D}$ ($\ell|_{\overline{D}} = 0$).
   - Cyclic prefix match $\mathrm{LCP}_F[y]$ obeys the dual backward recurrence $\mathrm{LCP}_F[y] = \min(n, 1 + \mathrm{LCP}_F[\Psi(y)])$ on $[1, n) \setminus \overline{C}$, reducing all $n$ values to the $r - 1$ **irreducible codomain heads** $\overline{C} \setminus \{0\}$.
   - On the 6 blocks, $\sum_{y \in \overline{C}} \mathrm{LCP}_F[y] = 0.964n\text{--}1.393n$ (`3.1%–4.1%` of KKP's $2n \log_2 r$ bound), with **median `3` bytes on all 6/6 blocks** (matching the $16^3 = 4,096$ collision depth of random hex GUIDs).
2. **Post-BWT Krohn–Rhodes / $\mathbb{F}_2[x]/(P_{32})$ Semigroup Homomorphism (`proved`, `verified` bit-exact on `6 / 6` blocks):**
   - Because `CRC32` acts on words $W \in \Sigma^\ell$ by the affine-linear semigroup $(v \mapsto v \cdot x^{8\ell} \oplus \mathrm{crc}_0(W) \bmod P_{32})$ and `RLE1` is a 5-state Mealy machine, their composition lifts to any SLP / `Open` dictionary in $O(G)$ operations (`verify_cycle4.py` passes bit-exact on all 6 blocks).
   - Empirically, `RLE1` triggers only **`23–83` times per 900 KB block** (`99.71%–99.93%` of 32B windows are pure identity). Thus post-BWT `unrle + crc32` (`20.6 + 11.0 = 31.6 ms` across Wikidata in `2026-11-20-open-stage-attribution.md`) is essentially a pure memory-bandwidth tax caused by writing and re-reading the intermediate `900 KB` `s.ll` buffer — fusible directly into `Open::emit`'s sequential output loop if needed.

---

## 5. Artefacts in This Directory

- `docs/journal/2026-11-20-carbon-kitchen-sink-cycle4/probe_cycle4.py` & `probe_cycle4_results.json` — exact invariants across all 6 blocks for Worlds 1–4.
- `docs/journal/2026-11-20-carbon-kitchen-sink-cycle4/verify_cycle4.py` — fast exact verifier (`1,514` ribbon-wall checks, explicit $n=10$ equal-$(n,C,p)$ ribbon-twin witness, `72,816` Schur subset checks, and `6/6` block `RLE1 ∘ CRC32` transducer checks).
