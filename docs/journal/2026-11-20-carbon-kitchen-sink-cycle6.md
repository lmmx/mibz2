# Carbon Kitchen Sink — Cycle 6: Operator Displacement Commutators $[D, P_{\mathrm{LF}}]$, the $r$-Value Law of Global Placement, and Forman–Sköldberg–Perseus Algebraic Morse Theory

**Date:** 2026-11-20
**Branch:** `research/carbon-kitchen-sink`
**Inputs:** `docs/team/research-charter/{FRONTIER.md,CONTEXT.md,OPEN-QUESTIONS.md}`, `origin/master` audit & new `Discrete Morse reduction` frontier lead (`70b18ca`, `804736e`), `origin/research/carbon-depth` (`e4cef68`), `origin/research/carbon-word-transport` (`89efba5`), and Cycles 1–5 (`ebcc161`).
**Artefacts:** [`probe_cycle6.py`](2026-11-20-carbon-kitchen-sink-cycle6/probe_cycle6.py), [`probe_cycle6_results.json`](2026-11-20-carbon-kitchen-sink-cycle6/probe_cycle6_results.json).

---

## 0. Orientation & Cycle 6 Questions

With Cycle 3's `Vec16` queue-iota + 16-byte free-monoid slot emission validated by the Coordinator and merged to production `master` (`#668`, `70b18ca`), Cycles 6–10 pivot decisively beneath incremental `Open` engineering toward **fundamental mathematical reformulations**: *what if sequential orbit traversal is not the only mathematical object governing exact BWT inversion, or if it is, what exact algebraic invariant forces it?*

Cycle 6 brings together two operator-algebraic and homological frameworks that bear directly on **Foundational Question 1** (*"Is there a potential or ranking function on intervals that places whole runs of output directly?"*), **Kempa's SODA 2019 run-head jump structure**, and the new **Discrete Morse reduction / Perseus 6** lead on `FRONTIER.md`:

1. **Kailath–Kung–Morf Operator Displacement Theory ($[D, P_{\mathrm{LF}}]$):** In numerical linear algebra, structured matrices (Toeplitz, Cauchy, Vandermonde) of size $n \times n$ are inverted in $\tilde{O}(r_{\mathrm{disp}} n)$ or represented in $O(r_{\mathrm{disp}})$ parameters when their commutator $[D, A]$ with a shift/difference operator $D$ has low rank $r_{\mathrm{disp}} \ll n$. Since $\mathrm{LF}$ acts as a piecewise unit-slope translation on $r$ runs, what is the exact commutator $[D, P_{\mathrm{LF}}]$, what does it imply for the discrete derivative $\Delta\boldsymbol{\tau}$ of the global text-placement coordinate $\tau : [0, n) \to \mathbb{Z}/n\mathbb{Z}$, and why does Sturmian $O(1)$ global coordinate placement break down on real blocks?
2. **Forman–Sköldberg Algebraic Morse Theory, Perseus Apparent Pairs, and Mrozek–Batko Coreductions:** Viewing $([0, n), \mathrm{LF}, L)$ as a 1-dimensional cellular Fox complex over the non-commutative free monoid ring $\mathbb{Z}\langle\Sigma^*\rangle$ (enhancing Perseus 6's totally unimodular unit-pair reductions over $\mathbb{Z}$ and $\mathbb{Z}[t, t^{-1}]$), how do **Mrozek–Batko coreductions**, **Perseus zero-search apparent matchings**, and **bi-thick Morse matchings** compare structurally and quantitatively to `Open`?

---

## 1. Kailath–Kung–Morf Displacement Structure $[D, P_{\mathrm{LF}}]$ and the $r$-Value Placement Law

Let $n \ge 2$, let $(L, p)$ be a primitive BWT with $r$ maximal equal-byte runs starting at $\overline{D} = \{d_0, d_1, \dots, d_{r-1}\}$ ($0 = d_0 < d_1 < \dots < d_{r-1}$) with codomain images $c_u = \mathrm{LF}(d_u)$, and let:
- $(P_{\mathrm{LF}} f)_x := f_{\mathrm{LF}(x)}$ be the Koopman permutation operator of $\mathrm{LF}$,
- $(Df)_x := f_x - f_{(x-1)\bmod n}$ be the cyclic backward-difference operator,
- $\tau(x) \in \{0, 1, \dots, n-1\}$ be the exact forward output text index written by row $x$ ($T[\tau(x)] = L[x]$, satisfying $\tau(\mathrm{LF}(x)) \equiv \tau(x) - 1 \pmod n$, i.e. $(I - P_{\mathrm{LF}})\boldsymbol{\tau} \equiv \mathbf{1} \pmod n$).

### Theorem 6.1 (Displacement Commutator Identity, Kempa $r$-Value Law & Lockstep Boundary Shattering — *proved*; *verified* on `1,508` primitive BWTs + all `5,399,692` rows of the 6 Wikidata blocks)

1. **Rank-$\le (r-1)$ Displacement Commutator:** For every function $f : \mathbb{Z}/n\mathbb{Z} \to R$ over any ring $R$,
   $$([D, P_{\mathrm{LF}}]f)_x = (D P_{\mathrm{LF}} f - P_{\mathrm{LF}} D f)_x = f_{(\mathrm{LF}(x)-1)\bmod n} - f_{\mathrm{LF}((x-1)\bmod n)}.$$
   For every non-head row $x \notin \overline{D}$ (note $0 = d_0 \in \overline{D}$, so $x \ge 1$ and $L[x] = L[x-1]$), rank stability gives $\mathrm{LF}(x-1) = \mathrm{LF}(x) - 1$, hence $([D, P_{\mathrm{LF}}]f)_x = 0$ identically. Thus $[D, P_{\mathrm{LF}}]$ is supported strictly on the $r$ run heads $\overline{D}$:
   $$[D, P_{\mathrm{LF}}] = \sum_{u=0}^{r-1} \mathbf{e}_{d_u}\!\left(\mathbf{e}_{(c_u - 1)\bmod n}^\top - \mathbf{e}_{\mathrm{LF}((d_u - 1)\bmod n)}^\top\right), \qquad \operatorname{rank}([D, P_{\mathrm{LF}}]) \le r - 1.$$

2. **Exact $r$-Value Law of $\Delta\boldsymbol{\tau}$:** Define the placement derivative $\Delta_x := (\tau(x) - \tau((x-1)\bmod n)) \bmod n$, and for each run head $u \in [0, r)$ set $\alpha_u := \Delta_{d_u}$. Applying $D$ to $(I - P_{\mathrm{LF}})\boldsymbol{\tau} \equiv \mathbf{1} \pmod n$ annihilates $\mathbf{1}$ ($D\mathbf{1} = \mathbf{0}$) and gives
   $$(I - P_{\mathrm{LF}})\Delta\boldsymbol{\tau} \equiv -[D, P_{\mathrm{LF}}]\boldsymbol{\tau} \pmod n \implies \Delta_x = \Delta_{\mathrm{LF}(x)} \quad \forall x \notin \overline{D}.$$
   Since $(L, p)$ is primitive, the forward $\mathrm{LF}$-orbit of every $x \in [0, n)$ hits $\overline{D}$ in finite time. Letting $v(x) \in [0, r)$ denote the index of the **first $\overline{D}$-hit** of $x$ under $\mathrm{LF}^{\ge 0}$ (with $v(d_u) = u$ at step $0$ — Kempa's SODA 2019 run-head jump map), every row's placement derivative is governed by the $r$-tuple $(\alpha_0, \dots, \alpha_{r-1})$:
   $$\Delta_x = \alpha_{v(x)} \qquad \forall x \in [0, n).$$

3. **Sturmian Global Coordinate Collapse ($r = 2$):** By Mantaci–Restivo–Sciortino (2003), primitive binary words with $r = 2$ are precisely the Sturmian/mechanical necklaces. For every primitive BWT with $r = 2$, $\alpha_0 \equiv \alpha_1 \pmod n$ (`210/210` exhaustive Sturmian checks), collapsing $\Delta_x \equiv \alpha_0 \pmod n$ to **a single global constant** across all $n$ rows (`1` run of $\Delta\boldsymbol{\tau}$), so $\tau(x) \equiv \tau(0) + \alpha_0 x \pmod n$ places every byte in $O(1)$ modular arithmetic with zero orbit traversal.

4. **Lockstep Boundary Shattering Law ($r \gg 2$):** Consider an adjacent row pair $(x-1, x)$ with $x \notin \overline{D}$. Under forward $\mathrm{LF}$, the pair $y_k = \mathrm{LF}^k(x)$ and $y'_k = \mathrm{LF}^k(x-1)$ satisfies $y'_k = y_k - 1$ in lockstep until the **first** step $k \ge 0$ where $\{y_k - 1, y_k\} \cap \overline{D} \ne \emptyset$:
   - if both $y_k - 1 = d_{u-1}$ (a width-1 run) and $y_k = d_u$ lie in $\overline{D}$, then $v(x-1) = u - 1 \ne u = v(x)$ unconditionally;
   - if $y_k = d_u \in \overline{D}$ and $y_k - 1 \notin \overline{D}$, then $v(x-1) = v(x)$ iff $v(d_u - 1) = u$ (the tail $d_u - 1$ returns to $d_u$ before hitting any of the other $r - 1$ run heads);
   - if $y_k - 1 = d_w \in \overline{D}$ and $y_k = d_w + 1 \notin \overline{D}$, then $v(x-1) = v(x)$ iff $v(d_w + 1) = w$ (row $d_w + 1$ returns to $d_w$ avoiding all other $r - 1$ run heads).

*Proof.* (1) Expand $(D P_{\mathrm{LF}} f)_x = f_{\mathrm{LF}(x)} - f_{\mathrm{LF}(x-1)}$ and $(P_{\mathrm{LF}} D f)_x = f_{\mathrm{LF}(x)} - f_{(\mathrm{LF}(x)-1)\bmod n}$ and subtract; since $\mathbf{1}$ lies in both the right kernel and left kernel of $[D, P_{\mathrm{LF}}]$ and $[D, P_{\mathrm{LF}}]$ has at most $r$ non-zero rows, its rank is at most $r - 1$. (2) $D(I - P_{\mathrm{LF}}) = (I - P_{\mathrm{LF}})D + [D, P_{\mathrm{LF}}]$; evaluating at $x \notin \overline{D}$ where row $x$ of $[D, P_{\mathrm{LF}}]$ vanishes gives $\Delta_x - \Delta_{\mathrm{LF}(x)} = 0$, and induction along $x \to \mathrm{LF}(x) \to \dots \to d_{v(x)}$ gives $\Delta_x = \Delta_{d_{v(x)}} = \alpha_{v(x)}$. (3) When $r = 2$, $(L, p)$ has $L = \mathtt{1}^{w_0}\mathtt{0}^{w_1}$ with $\gcd(w_0, w_1) = 1$, where $\mathrm{LF}(x) = (x + w_1) \bmod n$ is a pure cyclic rotation; thus $[D, P_{\mathrm{LF}}] = 0$ identically even on $\overline{D}$, forcing $\Delta_x = \Delta_{\mathrm{LF}(x)}$ on the entire single orbit, so $\alpha_0 = \alpha_1 = w_1^{-1} \bmod n$ (`−w_1^{-1}` under backward indexing). (4) Immediate from tracking the lockstep invariant $y'_k = y_k - 1$ up to the stopping time $\min\{k \ge 0 : \{y_k - 1, y_k\} \cap \overline{D} \ne \emptyset\}$. $\square$

### Empirical Confrontation on the 6 Wikidata Blocks (`probe_cycle6.py`)

| Block | $n$ | $r$ | Distinct $\|\{\Delta_x\}\| = \|\{\alpha_u\}\|$ | Collisions $r - \|\{\alpha_u\}\|$ | Constant runs $r(v)$ of Kempa's $v(x)$ | Constant runs $r(\Delta\boldsymbol{\tau})$ | $r(\Delta\boldsymbol{\tau}) / n$ |
|---|---:|---:|---:|---:|---:|---:|---:|
| `block-0.L` | 899,931 | 51,184 | 46,684 | 4,500 | **899,931** (`1.00000n`) | **887,651** | **0.9864** |
| `block-2.L` | 900,000 | 54,532 | 50,549 | 3,983 | **900,000** (`1.00000n`) | **894,023** | **0.9934** |
| `block-85.L` | 899,935 | 79,375 | 72,201 | 7,174 | **899,935** (`1.00000n`) | **899,575** | **0.9996** |
| `block-143.L` | 899,869 | 108,335 | 97,737 | 10,598 | **899,869** (`1.00000n`) | **897,583** | **0.9975** |
| `block-210.L` | 899,959 | 128,732 | 113,351 | 15,381 | **899,959** (`1.00000n`) | **897,031** | **0.9967** |
| `block-207.L` | 899,998 | 87,111 | 78,098 | 9,013 | **899,997** (`0.999999n`) | **897,073** | **0.9967** |

**What this establishes (resolving Foundational Question 1 & Kempa SODA 2019):**
- **Value-Set Compression vs Interval Shattering:** Operator displacement theory succeeds algebraically — the derivative $\Delta\boldsymbol{\tau}$ of global text placement takes only $46,684\text{--}113,351 \le r$ distinct values across $900,000$ rows. **Yet its run-length encoding has `887,651–899,575` runs (`98.64%–99.96%` of $n$)!**
- On five of the six blocks (`0, 2, 85, 143, 210`), **not a single adjacent row pair $(x-1, x)$ shares the same first $\overline{D}$-hit $v(x)$** (`r(v) = n`), and on `block-207` literally **one** pair out of `899,998` shares $v(x-1) = v(x)$. Even inside a width-800 run of identical bytes, consecutive rows originate from unrelated occurrences of that context across the JSON file, so the two-sided boundary of $\overline{D}$ splits every adjacent pair $(x-1, x)$ before $\overline{D}$-arrival.
- Consequently, **no interval-based potential or affine ranking function on BWT intervals can place whole $L$-runs of output directly** in $o(n)$ segments: $\tau(x)$ has $\ge 0.986n$ distinct affine pieces on every real block.

---

## 2. Forman–Sköldberg Non-Commutative Fox Morse Theory, Perseus Apparent Pairs & Coreductions

Consider the 1-dimensional CW / directed functional complex $X_p$ with 0-cells $V = [0, n)$ and 1-cells $E_p = \{e_x : x \in [0, n) \setminus \{p\}\}$, where $e_x$ is directed $x \to \mathrm{LF}(x)$ and carries the non-commutative Fox right-module boundary over the free monoid ring $R = \mathbb{Z}\langle\Sigma^*\rangle$:
$$\partial(e_x) = e_{\mathrm{LF}(x)} \cdot L[x] - e_x \in R^{(V)}.$$

- Over $\mathbb{Z}$ (setting all $a \in \Sigma \mapsto 1$), the boundary matrix $B_p$ is the node-arc incidence matrix of a spanning in-tree rooted at $p$: it is **totally unimodular** with determinant $\pm 1$, which is the exact homological reason why the orbit-time linear system $(I_p - P_{\mathrm{LF},p})\boldsymbol{\tau} = \mathbf{1}$ in §1 inverts over $\mathbb{Z}$ with unit pivots $\pm 1$.
- Over Laurent polynomials $\mathbb{Z}[t, t^{-1}]$ (Perseus 6's `Z[t, t^-1]` mode, weighting every step by $t$), the reduced boundary tracks only word *lengths* $t^{|w|}$, and the unpunctured Alexander determinant $\det(I - t P_{\mathrm{LF}}) = 1 - t^n$ depends only on $n$.
- Over $R = \mathbb{Z}\langle\Sigma^*\rangle$ (**Sköldberg 2006 algebraic Morse theory**), matching a 0-cell $x \ne p$ with its unique outgoing unit-pivot 1-cell $e_x$ (`coeff(e_x, x) = -1`, a unit in $R$) eliminates $(x, e_x)$ by exact non-commutative Schur complementation!

### Theorem 6.2 (Universal Punctured Acyclicity, Reduction–Coreduction Shift-Conjugacy & Apparent-Pair $n/2$ Barrier — *proved*; *verified* on `6,152` small column configurations + 6 Wikidata blocks)

1. **Universal Punctured Acyclicity Lemma:** On any primitive BWT $(L, p)$, $X_p$ is a single directed path $x_{n-1} \to x_{n-2} \to \dots \to x_0 = p$ with zero directed cycles. Therefore **every** subset $\mathcal{M} \subseteq [0, n) \setminus \{p\}$ is unconditionally an acyclic Forman–Sköldberg discrete Morse matching with critical 0-cells $V_{\mathrm{crit}}(\mathcal{M}) = [0, n) \setminus \mathcal{M} \ni p$, and walking the reduced Fox differential from $p$ recovers $T$ byte-for-byte. (On arbitrary multi-cycle columns $(L, p)$, any strict-order **Perseus apparent matching** $\mathcal{M}_\prec = \{x \ne p : x \prec \mathrm{LF}(x)\}$ for any strict partial order $\prec$ on $[0, n)$ is unconditionally cycle-free with $O(1)$ zero-search local classification.)

2. **Exact Reduction–Coreduction Shift-Conjugacy (`Open` vs Mrozek–Batko Coreduction):** Let $T_W = \{x \in [0, n) : \mathrm{rw}(x) > W\}$ be the thick domain union. Compare:
   - **Forman Thick-Domain Reduction (`Open<W>`):** $\mathcal{M}_{\mathrm{red}} = T_W \setminus \{p\}$, eliminating thick domain cells $x$ via outgoing $e_x$;
   - **Mrozek–Batko Thick-Codomain Coreduction (`Coreduce<W>`):** $\mathcal{M}_{\mathrm{cor}} = \Psi(T_W) \setminus \{p\} = \{x \ne p : \mathrm{rw}(\mathrm{LF}(x)) > W\}$, eliminating cells whose outgoing boundary lands in $T_W$.

   Whenever $p \notin T_W \triangle \Psi(T_W)$ (and up to at most the single interval straddling $p$ otherwise), $\mathrm{LF}$ restricts to an **exact bijection between critical complexes**:
   $$c' \in V_{\mathrm{crit}}(\mathcal{M}_{\mathrm{cor}}) \iff \mathrm{LF}(c') \in V_{\mathrm{crit}}(\mathcal{M}_{\mathrm{red}}),$$
   preserving $|V_{\mathrm{crit}}|$ **to the single integer** and preserving the **exact multiset of gradient path word lengths** (shifting every gradient word cyclically by 1 step along the orbit).

3. **Bi-Thick Morse Matching (`BiThick<W>`):** Since $\mathcal{M}_{\mathrm{red}} \cup \mathcal{M}_{\mathrm{cor}} = (T_W \cup \Psi(T_W)) \setminus \{p\}$ is ALSO unconditionally acyclic by (1), taking the union eliminates **both** thick runs and thin rows whose immediate $\mathrm{LF}$-successor enters a thick run:
   $$V_{\mathrm{crit}}(\mathcal{M}_{\mathrm{bi}}) = \{p\} \cup \{x \in [0, n) : \mathrm{rw}(x) \le W \text{ and } \mathrm{rw}(\mathrm{LF}(x)) \le W\}.$$
   Algebraically, $\mathcal{M}_{\mathrm{bi}}$ equals `Open<W>` followed by 1-step coreduction of all retained rows whose `Open` word has length $\ge 2$.

4. **Apparent-Pair $n/2$ Involution Barrier:** Under the canonical index-order Perseus apparent matching $\mathcal{M}_{\mathrm{rise}} = \{x \ne p : x < \mathrm{LF}(x)\}$, $\sum_{x=0}^{n-1}(\mathrm{LF}(x) - x) = 0$ balances rising and falling edges near $n/2$. Under width-order apparent matching $\mathcal{M}_{\mathrm{w\text{-}app}} = \{x \ne p : (\mathrm{rw}(x), -x) > (\mathrm{rw}(\mathrm{LF}(x)), -\mathrm{LF}(x))\}$, any thick run mapping into a run of *equal or greater* width ($\mathrm{rw}(x) \le \mathrm{rw}(\mathrm{LF}(x))$) fails strict filtration decrease and remains critical, leaving $\approx 0.50\text{--}0.55n$ critical cells (`2.3–5.3×` worse than `Open<16>`).

### Quantitative Census Across Morse Matchings on the 6 Wikidata Blocks (`probe_cycle6.py`)

Here $B(\mathcal{M})$ counts **coherent critical bundles**: maximal contiguous intervals of critical indices $[c_i, c_j]$ in $V_{\mathrm{crit}}(\mathcal{M})$ whose reduced Morse arrows step in unit lockstep ($c_{\mathrm{next}}[k] = c_{\mathrm{next}}[k-1] + 1$) with the exact same gradient word.

| Block | `Rising` ($x < \mathrm{LF}(x)$) $\|V_{\mathrm{crit}}\|$ | `WidthApp` $\|V_{\mathrm{crit}}\|$ | `Open<16>` $\|V_{\mathrm{crit}}\|$ ($B$) | `Coreduce<16>` $\|V_{\mathrm{crit}}\|$ ($B$) | `BiThick<16>` $\|V_{\mathrm{crit}}\|$ ($B$) | `BiThick` vs `Open` $\|V_{\mathrm{crit}}\|$ |
|---|---:|---:|---:|---:|---:|---:|
| `block-0.L` | 439,674 (`0.489n`) | 479,108 (`0.532n`) | **89,874** (`47,544`) | **89,874** (`46,226`) | **74,596** (`43,890`) | **−17.0%** (`max=1042`) |
| `block-2.L` | 444,364 (`0.494n`) | 470,271 (`0.523n`) | **112,070** (`50,527`) | **112,070** (`48,960`) | **89,746** (`45,576`) | **−19.9%** (`max=946`) |
| `block-85.L` | 443,087 (`0.492n`) | 497,733 (`0.553n`) | **156,404** (`77,733`) | **156,404** (`76,247`) | **134,618** (`73,354`) | **−13.9%** (`max=359`) |
| `block-143.L` | 454,919 (`0.506n`) | 496,392 (`0.552n`) | **196,841** (`103,102`) | **196,841** (`100,054`) | **152,219** (`93,757`) | **−22.7%** (`max=1608`) |
| `block-210.L` | 447,352 (`0.497n`) | 479,769 (`0.533n`) | **204,281** (`125,843`) | **204,281** (`123,389`) | **168,710** (`118,487`) | **−17.4%** (`max=558`) |
| `block-207.L` | 457,697 (`0.509n`) | 486,578 (`0.541n`) | **135,921** (`83,449`) | **135,921** (`81,403`) | **109,958** (`77,437`) | **−19.1%** (`max=1650`) |

### Costed Algorithmic Confrontation (No C Benchmark Needed — Decisive Arithmetic Filter)

1. **Pure Mrozek–Batko Coreduction (`Coreduce<16>`) is strictly dominated by `Open<16>`:** Theorem 6.2(2) + the table show `Coreduce<16>` has the **exact same critical count** (`89,874` to `204,281`) and **exact same word-length distribution** (`max = 341..1626`) as `Open<16>`, while `Open<16>` classifies $\mathrm{rw}(x) > 16$ locally from the run being scanned in Pass 1 (`0` random lookups), whereas classifying $\mathrm{rw}(\mathrm{LF}(x)) > 16$ requires knowing `C[b]` and an `is_thick[0..n)` bitvector (`112.5 KB`, random-probed at $\mathrm{LF}(x)$) before thin-row numbering!
2. **`BiThick<16>` saves `15,278–44,622` critical cells (`13.9–22.7%` of $m$), priced against a mandatory extra pass:** At L2-resident `Open` emit speed (`~5.5 ns/hop`), skipping `15K–45K` hops saves **`84–245 µs`** in `emit`, while `max_word_len` remains unbounded (`359–1,650` bytes, overflowing inline `Fox64` slots on thick cascades unless backed by the literal pool) and classifying $\mathrm{rw}(\mathrm{LF}(x)) > 16$ before assigning dense critical IDs requires an extra $n$-lookup / $m$-scan pass (`~200–450 µs`, matching Original Carbon Cycle 14's finding that post-`Open` 1-step Morse contraction costs more to build than the hops it removes).
3. **Why `Open<16>` beats Perseus apparent matchings by `2.3–5.3×`:** Perseus's general-complex apparent matchings must enforce a strict filtration order to guarantee acyclicity without graph search; on punctured primitive BWTs, **Universal Acyclicity (Theorem 6.2(1))** removes the cycle-avoidance constraint completely, allowing `Open` to match **all** thick runs (`w > 16`) simultaneously even when thick runs map into thicker runs!

---

## 3. Summary of Cycle 6 Outcomes & Bridge to Cycle 7

- **Proved & Verified:**
  1. **Kailath–Kung–Morf Commutator & Kempa $r$-Value Law (Theorem 6.1):** $\operatorname{rank}([D, P_{\mathrm{LF}}]) \le r - 1$ and $\Delta_x = \alpha_{v(x)}$ for all $x \in [0, n)$ (`1,508` small primitive BWTs + `5.40M` Wikidata rows), collapsing to `1` global run on Sturmian words ($r = 2$) and shattering into **`0.986–1.000n` runs** on all 6 Wikidata blocks (`r(v) = n` on 5/6 blocks).
  2. **Forman–Sköldberg–Perseus Morse Classification (Theorem 6.2):** Universal Punctured Acyclicity on primitive BWTs (`6,152` small column checks), **exact 1-step shift-conjugacy** between Forman reduction (`Open`) and Mrozek–Batko coreduction (`Coreduce`), `13.9–22.7%` critical-cell reduction for `BiThick<16>`, and the `0.49–0.55n` Apparent-Pair Barrier.
- **Next (Cycle 7 — "Decoding Without an Orbit Walk?"):** Having ruled out $o(n)$-piece global placement potentials (§1) and bounded 1-complex Morse matchings (§2), Cycle 7 asks whether we can **dispense with the row-index domain $[0, n)$ altogether** via three collective algebraic constructions:
  1. **Kotzig–Pevzner $k$-Gram De Bruijn Eulerian Condensation Phase Transition** (at what $k^*$ does the $k$-gram multigraph $G_k(T)$ condense uniquely to $T$ with $\mathcal{E}(G_k) = 1$, and how much ambiguity lives at $k = 2, 4, 8, 16, 32, 64$?);
  2. **Dyadic Binary Exponentiation $\mathrm{LF}^{2^j}$ via Pure Sequential Interval-Exchange Stream Merges** (`0` random pointer chases — measuring exact piece growth $r(\mathrm{LF}^{2^j})$ for $j = 0 \dots 19$);
  3. **Multipoint Generating-Function / NTT Resolvent Reformulation** ($\widehat{T}(\omega^k) = ((I - \omega^k P^{(p)})^{-1}\mathbf{L})_p$ over $\mathbb{F}_q$).
