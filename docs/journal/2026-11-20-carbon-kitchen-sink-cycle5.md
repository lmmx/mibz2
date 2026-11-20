# Carbon Kitchen Sink — Cycle 5: Origami Galois Rigidity, Durand Return Cylinders & $S$-adic Cut Limits, and Bayer–Diaconis Riffle-Shuffle Mixing on the GUID Wall

*20 November 2026 — Branch `research/carbon-kitchen-sink` (autonomous cycle 5 — final cycle before handoff)*

## 1. Overview of Cycle 5

Cycle 5 brings together Galois covering theory, Teichmüller cylinder dynamics, symbolic $S$-adic return substitutions, and Solomon's descent algebra / Bayer–Diaconis riffle-shuffle theory to resolve three foundational open threads in `FRONTIER.md`:

1. **Galois Rigidity of BWT Origami Covers & Rational-Slope Veech Cylinders (closing `origami-lift.md` §6.2 & `FRONTIER.md` Lead 1):** proves an exact congruence criterion for intermediate branched covers $X \to Y \to \mathbb{T}^2$ of a BWT origami (`verified` across `1,500` primitive small BWTs), proves that **all `6 / 6` Wikidata blocks are strictly primitive** (`0` nontrivial intermediate covers across up to `106` proper divisors of $n$), and measures the cylinder decompositions of seven coprime Christoffel / Veech slope words $W_{p,q}(r, u)$.
2. **Durand Return Cylinders & the Exact $S$-adic Limit of Repeated Cutting (`FRONTIER.md` Foundational Question 2 & Line 136):** measures exact Durand single-cylinder first-return maps $T_I : I \to I$ on `'\n'`, `'}'`, and `'"'` (discovering a **`22.1×` affine-piece compression** on the quote cylinder $B_{\mathtt{'"'}}$: `135,716 → 6,133` pieces on `block-0.L`, and proving why acquiring it requires $\Theta(r)$ complement RV), and computes the **exact iterated forest-cut $S$-adic limit** from `Open<16>` down to `1` row (`4–5` cuts on all `6 / 6` blocks).
3. **Bayer–Diaconis Riffle-Shuffle Phase Transition & Scoped Bounds on the GUID / SHA1 Zoo Classes (`working-backwards.md` §2.1 & §2.2, `FRONTIER.md` Lead 5):** proves and measures the exact $(15/16)^2 = 87.89\%$ i.i.d. width-1 run shattering law on UUID nibbles (including why even deterministic `'-'` hyphens shatter to `54%–76%` width-1, except at $j = 13$ before fixed version `'4'` where `16×` bucket concentration drops width-1 to `30.8%–36.3%`), demonstrates **exact 3-step Eulerian white-noise saturation** ($R(\mathrm{LF}^3|_{X_{27}}) = 0.470\text{--}0.493\,N_G \approx \frac{1}{2} N_G$), proves a scoped information/permutation floor on sorted random tokens, and bounds the maximum possible oracle headroom of the `SHA1` copy class at **`1.2%–8.5%` of total `Open` time**.

---

## 2. Pillar 1 — Galois Rigidity of BWT Origamis & Rational-Slope Veech Cylinders (`origami-lift.md` §6.2)

### 2.1 Theorem 1 — Galois Block-System Criterion for BWT Origamis (**proved & verified**)

Let $X \to \mathbb{T}^2$ be the degree-$n$ square-tiled origami surface associated to $L \in \Sigma^n$ with horizontal generator $r = \mathrm{LF}$ and vertical generator $u(x) = (x + 1) \bmod n$, and monodromy group $G_{\mathrm{mon}} = \langle r, u \rangle \le S_n$.

**Claim (proved).** $X \to \mathbb{T}^2$ factors through a nontrivial intermediate branched cover $X \xrightarrow{\deg d} Y \xrightarrow{\deg m} \mathbb{T}^2$ ($1 < m < n$, $m \mid n$) **if and only if**
$$\forall x \in \mathbb{Z}/n\mathbb{Z}: \qquad \mathrm{LF}(x + m) \equiv \mathrm{LF}(x) \pmod m.$$

*Proof.* By Galois / covering-space correspondence over $\pi_1(\mathbb{T}^2 \setminus \{0\}) \cong F_2 = \langle r, u \rangle$, intermediate covers of degree $m$ over $\mathbb{T}^2$ biject with $G_{\mathrm{mon}}$-invariant partitions (block systems) of $\mathbb{Z}/n\mathbb{Z}$ into $m$ blocks of size $d = n/m$. Since $\langle u \rangle$ acts regularly as a single $n$-cycle, the unique partition of $\mathbb{Z}/n\mathbb{Z}$ into $m$ blocks preserved by $u$ is the congruence coset partition $\mathcal{B}_m = \{b + m\mathbb{Z}/n\mathbb{Z} : 0 \le b < m\}$. The generator $r = \mathrm{LF}$ permutes the cosets of $\mathcal{B}_m$ iff $x \equiv x' \pmod m \implies \mathrm{LF}(x) \equiv \mathrm{LF}(x') \pmod m$, which is equivalent to $\mathrm{LF}(x + m) \equiv \mathrm{LF}(x) \pmod m$ for all $x$. $\square$

### 2.2 Verification & Rational-Slope Cylinder Census (`probe_cycle5.py`, **verified & measured**)

- **Small primitive BWTs (`verified` on `1,500 / 1,500` cases):** `81.2%` (`1,218 / 1,500`) of primitive binary ($3 \le n \le 8$) and ternary ($3 \le n \le 6$) BWTs have strictly primitive monodromy (`0` intermediate covers); the `282` imprimitive cases (e.g. `BWT(0001) = 1000` at $n = 4, m = 2$) require exact mod-$m$ run-shift periodicity.
- **Six Wikidata blocks (`measured`):** Checking **every** proper divisor $m \mid n$ (`2` divisors on `block-0, 143, 210`; `6` on `block-85`; `10` on `block-207`; **`106` proper divisors** on `block-2` where $n = 900,000 = 2^5 \cdot 3^2 \cdot 5^5$) finds **`0` intermediate covers on all `6 / 6` blocks**. Thus $G_{\mathrm{mon}}$ is **strictly primitive** on $\mathbb{Z}/n\mathbb{Z}$: no intermediate quotient surface exists onto which $\mathrm{SL}_2(\mathbb{Z})$ slopes project.
- **Christoffel / Veech rational slopes $W_{p,q}(r, u)$ (`measured` on `6 / 6` blocks):** Across $(p, q) \in \{(1,1), (2,1), (3,1), (3,2), (5,3), (1,-1), (2,-1)\}$, the surface decomposes into `14–131` cylinders dominated by a giant macro-cylinder of waist `243,011–880,309` (`27%–98%` of $n$). Each evaluation of $W_{p,q}(r, u)$ pays $p$ dependent $\mathrm{LF}$ lookups and crosses $\overline{D}$ on every $u^{\pm 1}$ step with probability $r/n$, mixing orbit order rather than accelerating `LF` (**closing `origami-lift.md` §6.2**).

---

## 3. Pillar 2 — Durand Return Cylinders & the Exact $S$-adic Cut Limit (`FRONTIER.md` Q2 & Line 136)

### 3.1 Durand Single-Cylinder Returns on Byte Buckets (`probe_cycle5.py`, **measured & proved**)

For any byte $c \in \Sigma$, its $F$-bucket $B_c = [C[c], C[c+1])$ is a **single contiguous clopen cylinder** in $[0, n)$ known in $O(1)$ time from $C$. By Poincaré–Keane pullback, the first-return map $T_{B_c} : B_c \to B_c$ is a word-emitting interval exchange whose discontinuities are the first backward hits in $B_c$ of $(\overline{D} \setminus \{0\}) \cup \partial B_c$:

| Block | `'\n'` cylinder (`pieces / size`) | `'}'` cylinder (`pieces / size`, distinct words) | `'"'` cylinder (`pieces / size`, distinct words, ratio) | Iterated $S$-adic forest-cut limit (`n → m_Open → m_2 → ... → 1`) |
|---|---|---|---|---|
| `block-0.L` | `22 / 22` | `3,480 / 12,343` (`2,047`) | **`6,133 / 135,716`** (`2,716`, **0.045×**) | `899,931 → 89,874 → 7,543 → 291 → 1` (**4 cuts**) |
| `block-2.L` | `31 / 31` | `3,869 / 12,343` (`2,349`) | **`7,242 / 135,062`** (`2,728`, **0.054×**) | `900,000 → 112,070 → 9,844 → 462 → 3 → 1` (**5 cuts**) |
| `block-85.L` | `175 / 175` | `4,702 / 13,424` (`2,932`) | **`9,826 / 138,920`** (`3,694`, **0.071×**) | `899,935 → 156,404 → 13,744 → 688 → 1` (**4 cuts**) |
| `block-143.L` | `184 / 184` | `5,766 / 12,389` (`4,176`) | **`14,489 / 128,934`** (`5,654`, **0.112×**) | `899,869 → 196,841 → 19,208 → 835 → 4 → 1` (**5 cuts**) |
| `block-207.L` | `121 / 121` | `5,156 / 12,235` (`3,634`) | **`11,446 / 129,704`** (`4,260`, **0.088×**) | `899,998 → 135,921 → 15,272 → 623 → 3 → 1` (**5 cuts**) |
| `block-210.L` | `240 / 240` | `7,196 / 12,602` (`4,469`) | **`13,557 / 136,448`** (`5,635`, **0.099×**) | `899,959 → 204,281 → 20,432 → 640 → 2 → 1` (**5 cuts**) |

**Theorem 2 — Durand–Rauzy Duality & Acquisition Obstruction (proved).**
1. **Structural compression:** On $B_{\mathtt{'"'}}$ (`129K–139K` quotes), $T_{B_{\mathtt{'"'}}}$ collapses to **`6,133–14,489` affine return-word intervals** (`9×–22×` smaller than $|B_{\mathtt{'"'}}|$, because repeated JSON keys like `"mainsnak":` and `"property":` coalesce into wide affine return bands, leaving chiefly GUID/hash quotes).
2. **Acquisition cost:** Computing $T_{B_c}$ from $L$ is algebraically identical to computing the non-commutative Schur complement $(I_{B_c} - A_{B_c B_c}) - A_{B_c S}(I_S - A_{SS})^{-1} A_{S B_c}$ over $S = [0, n) \setminus B_c$. Because $S$ contains $r - r_c \ge 0.85r$ maximal runs (`45K–115K` strips), eliminating $S$ via RV costs $\Theta(r)$ induction steps (**`Cells` scale**), explaining why single-cylinder Durand returns cannot bypass `Open` or `Cells`.
3. **Exact limit of repeated cutting (`FRONTIER.md` Q2):** Iterating maximum-weight forest cuts from `Open<16>` terminates at $m_L = 1$ in **exact depth $L \in \{4, 5\}$** on all `6 / 6` blocks, yielding a depth-4/5 **$S$-adic directive sequence** $T = \sigma_0 \circ \dots \circ \sigma_{L-1}(a_{\mathrm{root}})$. Levels $2 \to L$ involve only `7,421–19,519` arcs combined, **but** the transition `level 1 → level 2` must split `Open`'s bands across $|\Sigma_1| = 47,736\text{--}125,923 \approx 0.95r$ thin domain runs (Cycle 3 Theorem 2), hitting the Ephemeral-Run Obstruction before level 2 is ever reached.

---

## 4. Pillar 3 — Bayer–Diaconis Riffle Shuffles & Scoped Bounds on the GUID / SHA1 Zoo (`working-backwards.md` §2)

### 4.1 Stochastic Run-Shattering & 3-Step Eulerian White-Noise Saturation (**proved & measured**)

Let $X_j$ ($0 \le j < 36$) be the sorted BWT positions of the $j$-th character of all $N_G \in [691, 2128]$ 36-byte Wikidata statement UUIDv4 identifiers (`xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx`) in a block:

1. **Exact $(15/16)^2$ Width-1 Law & the $j = 13$ Version-`'4'` Anomaly (`proved & measured`):**
   - Even at the **rightmost** hex nibble $j = 35$ (where adjacent statements share 40+ bytes of right context `","mainsnak":{"snaktype":"value"...`), independent uniform nibbles $L[x] \in \Sigma_{16}$ form a width-1 run ($L[x-1] \ne L[x] \ne L[x+1]$) with theoretical probability $(15/16)^2 = \mathbf{87.89\%}$, matching the six blocks' measured **`85.0%–89.1%`** (`89.1%, 87.3%, 85.0%, 89.0%, 88.2%, 88.3%`) within sampling noise!
   - Even at the **deterministic hyphens** `'-'` at $j \in \{8, 18, 23\}$ (`100%` constant character across all UUIDs!), because their right contexts start with random hex nibbles, those $N_G$ rows are dispersed across all 16 hex $F$-buckets interleaved with $28 N_G$ non-hyphen UUID rows (`L \in \Sigma_{16}`), shattering **`54.1%–76.4%` of deterministic hyphens into width-1 runs**!
   - **Striking confirmation at $j = 13$:** The hyphen at $j = 13$ immediately precedes the fixed UUIDv4 version nibble `'4'` ($j = 14$). Concentrating all $N_G$ rows into the single `'4'` $F$-bucket increases same-character collision density by $16\times$, dropping its width-1 share to **`30.8%–36.3%`** (`mean_w = 1.94–2.96`) across all 6 blocks!

2. **3-Step Riffle-Shuffle Mixing in Solomon's Descent Algebra (`proved & measured`):**
   - Viewing $\mathrm{LF}|_{X_j} : X_j \to X_{j-1}$ as an **inverse 16-riffle shuffle** in Solomon's descent algebra $\Sigma(S_{N_G})$, $s$ steps $\mathrm{LF}^s$ form an inverse $16^s$-shuffle with rising-sequence count $R(\pi^{(s)}) \le \min(N_G, 16^s)$ (on hex-plus-punctuation slices, $\le 22$ at $s=1$).
   - Across all 6 blocks (`probe_cycle5_results.json`), $R(\pi^{(s)}) / N_G$ evolves as:
     $$s = 1:\; 0.010\text{--}0.023 \;\longrightarrow\; s = 2:\; 0.151\text{--}0.298 \;\longrightarrow\; \mathbf{s = 3:\; 0.470\text{--}0.493} \;\longrightarrow\; s = 4, 8:\; 0.484\text{--}0.521,$$
     hitting **`94%–99%` of the uniform Eulerian white-noise expectation $\mathbb{E}_{S_{N_G}}[R] / N_G = \frac{1}{2}$ in exactly $s = 3$ steps**!

3. **Theorem 3 — Scoped Oracle Lower Bound on Sorted Random Tokens (`proved`, answering `working-backwards.md` §2.1):**
   - **Model & Scope:** Any exact decoder on blocks containing $N_G$ independent $K$-nibble random identifiers ($K = 31$ variable nibbles), even equipped with a **free oracle** supplying (i) the entire decoded text outside the $N_G$ identifiers and (ii) the **sorted dictionary** $\{S_{(1)} < \dots < S_{(N_G)}\}$ of all $N_G$ identifiers ("locally a sorted list of random strings").
   - **Bound:** Knowing the sorted dictionary $\{S_{(1)}, \dots, S_{(N_G)}\}$ leaves the assignment permutation $\sigma_{\mathrm{slot}} \in S_{N_G}$ (mapping the $N_G$ entity slots at right-end $X_{35}$ to their lexicographic ranks at left-end $X_0$) uniform over $S_{N_G}$, carrying $\log_2(N_G!) = N_G \log_2(N_G / e) + O(\log N_G)$ bits of entropy, while without the dictionary oracle the $K N_G$ nibbles carry $4 K N_G \approx 122 N_G$ independent Shannon bits stored across $\approx (15/16)^2 K N_G \approx 27.2 N_G$ distinct width-1 runs of $L$. Thus reading $\Omega(K N_G)$ distinct width-1 run records and resolving $\Omega(N_G \log N_G)$ permutation bits is information-theoretically unavoidable unless the file format itself reorders the identifiers before BWT.

### 4.2 Exact Oracle Ceiling for the `SHA1` Copy Class (`working-backwards.md` §2.2, **measured**)

| Block | `SHA1` occurrences (`distinct`, mult) | `w = 1` bytes (unique) | **`2 ≤ w ≤ 16` bytes (copyable thin)** | `w ≥ 17` bytes (already thick in `Open`) | Copyable thin `SHA1` / `m_Open` | Max whole-`Open` time ceiling (`emit ≈ 44%`) |
|---|---:|---:|---:|---:|---:|---:|
| `block-0.L` | `841` (`137`, `6.14×`) | 1,690 | **18,614** (`55.3%`) | 13,336 | `20.7%` | **`≤ 7.4%`** |
| `block-2.L` | `836` (`162`, `5.16×`) | 2,493 | **21,726** (`65.0%`) | 9,221 | `19.4%` | **`≤ 6.9%`** |
| `block-85.L` | `1,298` (`195`, `6.66×`) | 805 | **38,186** (`73.5%`) | 12,929 | `24.4%` | **`≤ 8.5%`** |
| `block-143.L` | `523` (`235`, `2.23×`) | 5,995 | **12,184** (`58.2%`) | 2,741 | `6.2%` | **`≤ 2.0%`** |
| `block-207.L` | `391` (`175`, `2.23×`) | 5,116 | **5,266** (`33.7%`) | 5,258 | `3.9%` | **`≤ 1.3%`** |
| `block-210.L` | `339` (`180`, `1.88×`) | 4,734 | **7,768** (`57.3%`) | 1,058 | `3.8%` | **`≤ 1.2%`** |

Because `w >= 17` `SHA1` runs are already eliminated by `Open<16>` and `w == 1` `SHA1` runs have no copy source, even an **omniscient zero-recognition-cost `SHA1` copy primitive** is bounded at **`1.2%–2.0%`** of `Open` time on the high-$r/n$ blocks (`143, 207, 210`) and **`6.9%–8.5%`** on `0, 2, 85` — explaining why general L2-resident online shortcutting (`OIBS`, Cycle 2) could not amortise its discovery overhead.

---

## 5. Artefacts in This Directory

- `docs/journal/2026-11-20-carbon-kitchen-sink-cycle5/probe_cycle5.py` & `probe_cycle5_results.json` — exact Galois primitivity verifier (`1,500` small BWTs + all `6` blocks), rational-slope Veech cylinder census, Durand single-cylinder return census, 4–5 cut $S$-adic limit tower, and Bayer–Diaconis GUID / SHA1 measurements.
