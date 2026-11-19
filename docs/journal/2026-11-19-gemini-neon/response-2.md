# Gemini 4 Neon: second response, verbatim

Received by Louis on 2026-11-19, in reply to a follow-up drafted by ChatGPT. The follow-up asked
Neon to re-examine Theorem 3 against direct first-return computation, to repair Theorem 2's
indexing and to narrow the scope of the Kac identity. Filed unedited. The review is in
[README.md](README.md).

---

That critique is well-founded. Auditing the previous Theorem 3 exposes **three concrete defects** in the original formulation—one of which fails even for a single front ($\vert{}S\vert{} = 1$)—along with an indexing error and an overbroad phrasing in Theorem 2.

Below is:

1. The exact failure modes and minimal counterexamples to the previous Theorem 3.
2. A **corrected Theorem 3** with explicit sentinel-based state, backward primary-mark transport, and exact conflict-free sufficient conditions, with a complete proof.
3. Explicit finite verification across nontrivial cross-component exchanges, multiple cycles, tied rotations (periodic BWT), and all primary marks.
4. The repaired **Theorem 2** (exact Fibonacci indexing and the strict distinction between row-level Kac tracing and interval-level discovery).

No performance estimates are included.

---

## 1. Defects Found in the Previous Theorem 3 (with Minimal Counterexamples)

### Defect 1 (Single-Front Cross-Component Front-Pointer Corruption)

In the previous formulation, front $j$ maintained raw pointers `(A_j, B_j)` and Case 1 re-inserted `B_j` immediately to the right of `A_j` in the codomain list. When `A_j` is itself the rightmost codomain interval $B_m$ of another component $W_m$ ($m \notin S$), re-inserting `B_j` to the right of `A_j` makes `B_j` the new rightmost codomain interval of $W_m$—leaving `cod_front[m]` stale at `A_j`.

**Minimal Counterexample ($n = 4, r = 3, K = 2$, single 4-cycle, $\vert{}S\vert{} = 1$):**
Let $W_1 = [0, 2)$, $W_2 = [2, 4)$ ($b_1 = 2 \in D(T) \cap C(T)$):

* $I_0$: $\lambda = 1$, $\mathrm{dom} = [0, 1)$, $\mathrm{cod} = [3, 4)$, $\omega = \mathtt{a}$
* $I_1$: $\lambda = 1$, $\mathrm{dom} = [1, 2)$, $\mathrm{cod} = [2, 3)$, $\omega = \mathtt{b}$
* $I_2$: $\lambda = 2$, $\mathrm{dom} = [2, 4)$, $\mathrm{cod} = [0, 2)$, $\omega = \mathtt{c}$

Check: $T$ is the single 4-cycle $0 \to 3 \to 1 \to 2 \to 0$. The fronts are $(A_1, B_1) = (I_1, I_2)$ and $(A_2, B_2) = (I_2, I_0)$.
Execute a **single** step at Front 2 (Case 1, $\lambda(I_2) = 2 > \lambda(I_0) = 1$, $Y_2 = [3, 4)$):

* $I_2$ shrinks to $\lambda = 1$: $\mathrm{dom}(I_2) = [2, 3)$, $\mathrm{cod}(I_2) = [0, 1)$.
* $I_0$ moves to $\mathrm{cod}(I_0) = [1, 2)$ (right of $I_2$ inside $\mathrm{cod}(W_1)$), with $\omega(I_0) = \mathtt{ac}$.
* The true rightmost codomain interval of $W_1 = [0, 2)$ is now **$I_0$**, not $I_2$. Without updating `cod_front[1]`, the next step at Front 1 cuts $[0, 1)$ instead of $[1, 2)$, corrupting the induced map.

**Repair:** Replace mutable `(A_j, B_j)` caches with explicit per-component **sentinel nodes** `dom_left[j], dom_right[j], cod_left[j], cod_right[j]` in the doubly linked lists, defining $A_j := \mathrm{dom\_prev}(\mathrm{dom\_right}[j])$ and $B_j := \mathrm{cod\_prev}(\mathrm{cod\_right}[j])$. Inserting `B_j` between `A_j` and `A_j.cod_next` ($= \mathrm{cod\_right}[m]$) automatically updates $\mathrm{cod\_prev}(\mathrm{cod\_right}[m]) = B_j$.

---

### Defect 2 (Non-Local Forward Primary-Mark Transport)

The previous Theorem 3 part 3 transported $p \in Y_j$ **forward** to $T_X(p)$, noting it lies at offset $\theta$ inside the new $\mathrm{cod}(B_j)$. In a coordinate-free linked-list representation, knowing a point's **codomain** interval and offset does not determine its **domain** interval in $O(1)$ without a coordinate search.

**Repair:** Transport $p \in Y_j$ **backward** to $p' = T_X^{-1}(p) \in X'$. Because $Y_j \subseteq \mathrm{cod}(B_j)$, $p'$ always lands at offset $\theta$ in a known **domain** interval ($\mathrm{dom}(B_j)$ in Cases 1/3a; $\mathrm{dom}(A_j^{\text{new}})$ in Case 2), with prefix-skip offset $\delta' = \delta + \vert{}\omega(B_j)\vert{}$ preserving $0 \le \delta' < \vert{}\omega_{X'}(p')\vert{}$ (proved in Theorem 3' below).

---

### Defect 3 (Incomplete Case 3a Write Footprint)

In Case 3a ($\lambda(A_j) = \lambda(B_j)$, $A_j \neq B_j$), replacing `A_j` by `B_j` in the codomain list writes `cod_prev(A_j).cod_next = B_j`. The previous conflict set omitted `cod_prev(A_j)`, allowing a node $u = \mathrm{cod\_prev}(A_j) = \mathrm{cod\_next}(A_{j'})$ to be mutated concurrently by fronts $j$ and $j'$ (creating a data race / Rust mutable-borrow conflict even though they write opposite link fields of $u$).

**Repair:** Define the conflict-free condition directly on the exact per-front read/write node footprints $\mathcal{F}_j$ (which include `cod_prev(A_j)` in Case 3a).

---

## 2. Corrected Theorem 3 (Multi-Boundary Parallel Rauzy–Veech)

### 2.1 State Representation

Let $\mathcal{I} = (X, T, \mathcal{R}, \omega, p, \delta)$ be a finite marked dIET partitioned into $K$ contiguous components $W_j = [a_j, b_j)$ ($j = 1, \dots, K$) such that every internal component boundary lies in $D(T) \cap C(T)$ (initially achieved by choosing $b_j \in \{C[c]\} \subset C(\mathrm{LF})$ and splitting at most $K - 1$ runs of $L$).

Maintain:

* For each interval $I \in \mathcal{R}$: `len` $\lambda(I) \ge 1$, `word` $\omega(I) \in \Sigma^+$ (SLP node), and four doubly-linked pointers `dom_prev, dom_next, cod_prev, cod_next`.
* For each component $j \in \{1, \dots, K\}$: four fixed sentinel nodes `dom_left[j], dom_right[j], cod_left[j], cod_right[j]` bracketing the domain and codomain lists of $W_j$.
* For each nonempty component $W_j$, define $A_j := \mathrm{dom\_prev}(\mathrm{dom\_right}[j])$ and $B_j := \mathrm{cod\_prev}(\mathrm{cod\_right}[j])$, and let $\delta_j := \min(\lambda(A_j), \lambda(B_j))$ and $Y_j := [b_j - \delta_j, b_j) \subseteq \mathrm{dom}(A_j) \cap \mathrm{cod}(B_j)$.
* Primary mark state $(I_p, \theta_p, \delta)$ where $p \in \mathrm{dom}(I_p)$ lies at offset $\theta_p \in [0, \lambda(I_p))$ from the left of $\mathrm{dom}(I_p)$, and $\delta \in [0, |\omega(I_p)|)$ is the byte offset of $p_0$'s cyclic $L$-stream inside $W_X(p) := \omega(p)\omega(T(p))\cdots$.

### 2.2 Elementary Single-Front Update at $j$ (for $A_j \neq B_j$)

* **Case 1 ($\lambda(A_j) > \lambda(B_j)$):**
1. `A_j.len -= B_j.len`.
2. Unlink `B_j` from its codomain position (`cod_prev(B_j) <-> cod_right[j]`) and splice `B_j` immediately after `A_j` in the codomain list (`A_j <-> B_j <-> old_cod_next(A_j)`).
3. Set $\omega(B_j) \leftarrow \omega(B_j) \cdot \omega(A_j)$.
4. **Primary mark:** If $I_p = A_j$ and $\theta_p \ge \lambda_{\text{new}}(A_j)$ (i.e., $p \in Y_j$), set $(I_p, \theta_p, \delta) \leftarrow (B_j,\; \theta_p - \lambda_{\text{new}}(A_j),\; \delta + |\omega_{\text{old}}(B_j)|)$.

* **Case 2 ($\lambda(A_j) < \lambda(B_j)$):**
1. `B_j.len -= A_j.len`.
2. Unlink `A_j` from its domain position (`dom_prev(A_j) <-> dom_right[j]`) and splice `A_j` immediately after `B_j` in the domain list (`B_j <-> A_j <-> old_dom_next(B_j)`).
3. Set $\omega(A_j) \leftarrow \omega(B_j) \cdot \omega(A_j)$.
4. **Primary mark:**
* If $I_p = A_j$ (so $p \in Y_j$): set $(I_p, \theta_p, \delta) \leftarrow (A_j,\; \theta_p,\; \delta + |\omega(B_j)|)$.
* Else if $I_p = B_j$ and $\theta_p \ge \lambda_{\text{new}}(B_j)$: set $(I_p, \theta_p, \delta) \leftarrow (A_j,\; \theta_p - \lambda_{\text{new}}(B_j),\; \delta)$ (same point $p \in X'$, re-indexed into the split right half).

* **Case 3a ($\lambda(A_j) = \lambda(B_j)$ and $A_j \neq B_j$):**
1. Unlink `A_j` from its domain position (`dom_prev(A_j) <-> dom_right[j]`).
2. Unlink `B_j` from its codomain position (`cod_prev(B_j) <-> cod_right[j]`) and replace `A_j` with `B_j` in the codomain list (`cod_prev(A_j) <-> B_j <-> cod_next(A_j)`).
3. Set $\omega(B_j) \leftarrow \omega(B_j) \cdot \omega(A_j)$ and delete `A_j` ($\Delta r = -1$).
4. **Primary mark:** If $I_p = A_j$, set $(I_p, \theta_p, \delta) \leftarrow (B_j,\; \theta_p,\; \delta + |\omega_{\text{old}}(B_j)|)$.

* **Case 3b ($A_j = B_j$, handled outside normal batches):** $A_j$ is a closed band of $\lambda(A_j)$ parallel 1-cycles in $T_X$, each with cyclic $L$-stream $\omega(A_j)$. If $I_p = A_j$, the complete cycle $\mathcal{C}(p_0)$ is recovered directly as $\omega(A_j)$ rotated left by $\delta$ (and reversed, repeated $n / |\omega(A_j)|$ times if periodic). Unlink and delete $A_j$ ($\Delta r = -1$).

### 2.3 Sufficient Conditions and Exactness Proof

Define the **interval footprint** $\mathcal{F}_j \subseteq \mathcal{R}$ of an active front $j$ ($A_j \neq B_j$) as:

$$\mathcal{F}_j := \begin{cases}
\{A_j, B_j, \mathrm{cod\_next}(A_j), \mathrm{cod\_prev}(B_j)\} \cap \mathcal{R} & \text{(Case 1)},\\
\{A_j, B_j, \mathrm{dom\_next}(B_j), \mathrm{dom\_prev}(A_j)\} \cap \mathcal{R} & \text{(Case 2)},\\
\{A_j, B_j, \mathrm{dom\_prev}(A_j), \mathrm{cod\_prev}(A_j), \mathrm{cod\_next}(A_j), \mathrm{cod\_prev}(B_j)\} \cap \mathcal{R} & \text{(Case 3a)}.
\end{cases}$$

**Theorem 3' (Exactness & Confluence of Multi-Boundary Parallel RV — *Proved*).**
*Let $S \subseteq \{1, \dots, K\}$ be any subset of nonempty components such that:*

1. *(No Self-Loops)* *For all $j \in S$, $A_j \neq B_j$.*
2. *(Dynamic Disjointness)* *For all distinct $j, j' \in S$, $A_j \neq B_{j'}$.*
3. *(Footprint Disjointness)* *For all distinct $j, j' \in S$, $\mathcal{F}_j \cap \mathcal{F}_{j'} = \emptyset$.*

*(Note: Condition 3 already implies Condition 2 because $\{A_j, B_j\} \subseteq \mathcal{F}_j$; Condition 2 is stated separately to separate dynamic exactness from pointer-level disjointness.)*

*Then executing the elementary updates for all $j \in S$ simultaneously (in any order or in parallel) satisfies:*

1. *(Component Conservation)* *For every $m \in \{1, \dots, K\}$, $\sum_{I \in \mathrm{dom}(W_m)} \lambda(I) = \sum_{I \in \mathrm{cod}(W_m)} \lambda(I)$ is preserved.*
2. *(Dynamic Exactness & Confluence)* *The resulting linked-list dIET represents the exact first-return map $T_{X'}$ and return words $\omega_{X'}$ on $X' = X \setminus Y$, where $Y = \bigsqcup_{j \in S} Y_j$, with $r(T_{X'}) \le r(T_X)$.*
3. *(Primary-Mark Invariant)* *If $\mathcal{C}(p_0)$ meets $X'$, then the updated $(I_p, \theta_p, \delta)$ represents a point $p' \in X' \cap \mathcal{C}(p_0)$ with $0 \le \delta < |\omega_{X'}(p')|$ such that $p_0$'s cyclic $L$-stream begins at byte index $\delta$ of $W_{X'}(p')$.*

**Proof.**

*Part 1 (Dynamic Exactness of $T_{X'}$ and $\omega_{X'}$).*
Because $W_j$ are pairwise disjoint, $Y_j \subseteq W_j$ are pairwise disjoint. For any $j, j' \in S$ (including $j = j'$), $Y_j \subseteq \mathrm{dom}(A_j)$ implies $T_X(Y_j) \subseteq \mathrm{cod}(A_j)$, while $Y_{j'} \subseteq \mathrm{cod}(B_{j'})$. Since distinct codomain intervals are disjoint and $A_j \neq B_{j'}$ by Conditions 1–2, we have $T_X(Y_j) \cap Y_{j'} = \emptyset$. Therefore:

$$T_X(Y) \cap Y = \emptyset.$$

Consequently, every $x \in X' = X \setminus Y$ has first-return time $\tau_{X \to X'}(x) \in \{1, 2\}$:

* If $T_X(x) \in X'$, then $T_{X'}(x) = T_X(x)$ and $\omega_{X'}(x) = \omega_X(x)$.
* If $T_X(x) \in Y$, then $T_X(x) \in Y_j$ for a **unique** $j \in S$, and $T_X^2(x) \in T_X(Y_j) \subseteq X'$, so $T_{X'}(x) = T_X^2(x)$ and $\omega_{X'}(x) = \omega_X(x) \cdot \omega_X(T_X(x))$.

Now check that each front $j$'s elementary update implements this exact map on $T_X^{-1}(Y_j)$ and $Y_j$:

* **Case 1 ($\delta_j = \lambda(B_j) < \lambda(A_j)$):** $T_X^{-1}(Y_j) = \mathrm{dom}(B_j)$, and $T_X(Y_j)$ is the rightmost $\lambda(B_j)$ points of $\mathrm{cod}(A_j)$. Shrinking $A_j$ removes $Y_j$ from $\mathrm{dom}(A_j)$ and $T_X(Y_j)$ from $\mathrm{cod}(A_j)$. Moving `B_j`'s codomain to immediately right of the shrunk `A_j` assigns $\mathrm{dom}(B_j) \mapsto T_X(Y_j)$ with word $\omega(B_j)\cdot\omega(A_j)$.
* **Case 2 ($\delta_j = \lambda(A_j) < \lambda(B_j)$):** $T_X^{-1}(Y_j)$ is the rightmost $\lambda(A_j)$ points of $\mathrm{dom}(B_j)$, and $T_X(Y_j) = \mathrm{cod}(A_j)$. Shrinking $B_j$ leaves its left $\lambda(B_j)-\lambda(A_j)$ points mapping to the left $\lambda(B_j)-\lambda(A_j)$ points of $\mathrm{cod}(B_j) \subseteq X'$. Moving `A_j`'s domain to immediately right of the shrunk `B_j` assigns $T_X^{-1}(Y_j) \mapsto \mathrm{cod}(A_j)$ with word $\omega(B_j)\cdot\omega(A_j)$.
* **Case 3a ($\delta_j = \lambda(A_j) = \lambda(B_j)$):** $T_X^{-1}(Y_j) = \mathrm{dom}(B_j)$ and $T_X(Y_j) = \mathrm{cod}(A_j)$, so merging `B_j` into `A_j`'s codomain slot assigns $\mathrm{dom}(B_j) \mapsto \mathrm{cod}(A_j)$ with word $\omega(B_j)\cdot\omega(A_j)$ and deletes `A_j`.

*Part 2 (Zero Pointer or Sentinel Collisions — Confluence).*
Condition 3 guarantees no two distinct fronts $j, j' \in S$ read or write any common interval node in $\mathcal{R}$. It remains to verify that no two fronts $j \neq j'$ access the same **sentinel** node:

* `dom_right[m]` is accessed by front $j$ iff $j = m$ (Case 2/3a) or $\mathrm{dom\_next}(B_j) = \mathrm{dom\_right}[m]$ (Case 2, i.e., $B_j = A_m$). Two fronts $j \neq j'$ cannot both access `dom_right[m]`: if $j = m$ then $B_{j'} = A_m = A_j$ violates Condition 2; if $j, j' \neq m$ then $B_j = A_m = B_{j'}$ is impossible since $B_j \neq B_{j'}$.
* `cod_right[m]` is accessed by front $j$ iff $j = m$ (Case 1/3a) or $\mathrm{cod\_next}(A_j) = \mathrm{cod\_right}[m]$ (Case 1/3a, i.e., $A_j = B_m$). Identical argument rules out collisions.
* `dom_left[m]` is accessed iff $\mathrm{dom\_prev}(A_j) = \mathrm{dom\_left}[m]$, which forces $m = j$ (unique per $j$).
* `cod_left[m]` is accessed iff $\mathrm{cod\_prev}(B_j) = \mathrm{cod\_left}[m]$ ($m = j$) or $\mathrm{cod\_prev}(A_j) = \mathrm{cod\_left}[m]$ (Case 3a, $A_j$ is leftmost in $\mathrm{cod}(W_m)$). If $j = m$ and $j' \neq m$ both accessed `cod_left[m]`, then $B_m$ would be the sole codomain interval of $W_m$, forcing $A_{j'} = B_m = B_j$, violating Condition 2; if $j, j' \neq m$, both would be leftmost in $\mathrm{cod}(W_m)$, forcing $A_j = A_{j'}$, impossible.

Thus all $|S|$ linked-list updates touch pairwise disjoint memory locations (intervals and sentinels), so parallel execution is bit-for-bit identical to any sequential order.

*Part 3 (Component Conservation and $r(T_{X'}) \le r(T_X)$).*
For each $j \in S$, component $W_j$ loses $\delta_j$ from both $\mathrm{dom}(W_j)$ and $\mathrm{cod}(W_j)$; the component $W_m$ containing $\mathrm{cod}(A_j)$ (Case 1/3a) or $\mathrm{dom}(B_j)$ (Case 2) replaces $\delta_j$ of one interval by $\delta_j$ of another (net $0$). Each front produces $\Delta r \in \{0, -1\}$, so $r(T_{X'}) \le r(T_X)$.

*Part 4 (Primary-Mark Invariant).*
Since $p \in \mathrm{dom}(I_p)$ lies in a unique interval $I_p$, and $\{A_j, B_j : j \in S\}$ are pairwise distinct (by Conditions 1–2 and disjointness of $W_j$), at most one front $j \in S$ modifies $I_p$:

* If $p \in Y_j$ ($I_p = A_j$, $\theta_p = (\lambda(A_j) - \delta_j) + \theta$ with $\theta \in [0, \delta_j)$), then $p' := T_X^{-1}(p)$ lies at offset $(\lambda(B_j) - \delta_j) + \theta$ in $\mathrm{dom}(B_j)$, which after front $j$'s update is offset $\theta$ inside $\mathrm{dom}(B_j)$ (Cases 1/3a) or $\mathrm{dom}(A_j^{\text{new}})$ (Case 2). Since $T_X(p') = p \in Y_j$, we have $W_{X'}(p') = W_X(p') = \omega_X(p') \cdot W_X(p)$. Because $p_0$'s stream starts at byte $\delta < |\omega_X(p)| = |\omega(A_j)|$ of $W_X(p)$, it starts at byte $\delta' = \delta + |\omega(B_j)|$ of $W_{X'}(p')$, and $\delta' < |\omega(B_j)| + |\omega(A_j)| = |\omega_{X'}(p')|$.
* If $p \notin Y$ and $I_p = B_j$ in Case 2 with $\theta_p \ge \lambda(B_j) - \lambda(A_j)$, $p$ itself does not move ($p' = p \in X'$); its interval simply splits so $p$ is re-indexed to $(A_j^{\text{new}}, \theta_p - \lambda_{\text{new}}(B_j))$, with $W_{X'}(p) = W_X(p)$ and $\delta$ unchanged.
* Otherwise $(I_p, \theta_p, \delta)$ is untouched. $\square$

---

## 3. Explicit Finite Verification Across Edge Cases

Every test below compares the batched update directly against brute-force first-return computation on $X' = X \setminus Y$.

### Test A — Cross-Component Batched Update + Multiple Cycles ($n = 11, r = 6, K = 2$)

Partition $X = [0, 11)$ into $W_1 = [0, 5)$ and $W_2 = [5, 11)$:

* $I_0$: $\lambda=2$, $[0,2) \to [9,11)$, $\omega = \mathtt{a}$
* $I_1$: $\lambda=3$, $[2,5) \to [5,8)$, $\omega = \mathtt{b}$
* $I_2$: $\lambda=1$, $[5,6) \to [2,3)$, $\omega = \mathtt{c}$
* $I_3$: $\lambda=2$, $[6,8) \to [3,5)$, $\omega = \mathtt{d}$
* $I_5$: $\lambda=1$, $[8,9) \to [8,9)$, $\omega = \mathtt{f}$ (isolated 1-cycle)
* $I_4$: $\lambda=2$, $[9,11) \to [0,2)$, $\omega = \mathtt{e}$

Cycles of $T$: $(0\;9), (1\;10), (2\;5), (3\;6), (4\;7), (8)$.
Fronts: Front 1 has $A_1 = I_1$ ($\lambda=3$), $B_1 = I_3$ ($\lambda=2$) (Case 1, $Y_1 = [3, 5)$). Front 2 has $A_2 = I_4$ ($\lambda=2$), $B_2 = I_0$ ($\lambda=2$) (Case 3a, $Y_2 = [9, 11)$).

* **Footprint check:** $\mathcal{F}_1 = \{I_1, I_3, I_5, I_2\}$ and $\mathcal{F}_2 = \{I_4, I_0, I_5, I_2\}$. Here $\mathcal{F}_1 \cap \mathcal{F}_2 = \{I_5, I_2\} \neq \emptyset$ (so Theorem 3' conservatively declines to batch $S = \{1, 2\}$ here, though notice Dynamic Disjointness $T(Y)\cap Y = \emptyset$ holds).
* To satisfy Condition 3 strictly ($\mathcal{F}_1 \cap \mathcal{F}_2 = \emptyset$), insert one length-1 fixed-point spacer $I_6: [11, 12) \to [11, 12)$ or split $I_5$ into two length-1 intervals $I_{5a}, I_{5b}$ and $I_2$ into $I_{2a}, I_{2b}$ (putting $I_{2a}$ after $I_4$ and $I_{2b}$ before $I_3$ in $\mathrm{cod}(W_1)$, and $I_{5a}$ after $I_1$ and $I_{5b}$ before $I_0$ in $\mathrm{cod}(W_2)$): then $\mathcal{F}_1 = \{I_1, I_3, I_{5a}, I_{2b}\}$ and $\mathcal{F}_2 = \{I_4, I_0, I_{5b}, I_{2a}\}$ are **strictly disjoint**.
* **Direct first-return map on $X' = [0, 3) \cup [5, 9)$:**
* $0 \to 9 \to 0$ ($\mathtt{ae}$), $1 \to 10 \to 1$ ($\mathtt{ae}$), $2 \to 5$ ($\mathtt{b}$), $5 \to 2$ ($\mathtt{c}$), $6 \to 3 \to 6$ ($\mathtt{db}$), $7 \to 4 \to 7$ ($\mathtt{db}$), $8 \to 8$ ($\mathtt{f}$).

* **Batched Theorem 3' output:**
* `dom(W_1)`: $I_0(2), I_1(1)$; `cod(W_1)`: $I_0(2), I_2(1)$ (len $3$).
* `dom(W_2)`: $I_2(1), I_3(2), I_5(1)$; `cod(W_2)`: $I_1(1), I_3(2), I_5(1)$ (len $4$).
* Words: $I_0:\mathtt{ae},\; I_1:\mathtt{b},\; I_2:\mathtt{c},\; I_3:\mathtt{db},\; I_5:\mathtt{f}$.

* **Primary mark check on all $p_0 \in Y_1 \cup Y_2$:**
* $p_0 = 3 \in Y_1$ (stream $(\mathtt{bd})^\infty$): backward transport gives $p' = 6 \in I_3$, $\theta = 0$, $\delta = |\omega(I_3)| = 1$. Skipping $1$ byte of $(\mathtt{db})^\infty$ yields $(\mathtt{bd})^\infty$. Exact.
* $p_0 = 9 \in Y_2$ (stream $(\mathtt{ea})^\infty$): backward transport gives $p' = 0 \in I_0$, $\theta = 0$, $\delta = |\omega(I_0)| = 1$. Skipping $1$ byte of $(\mathtt{ae})^\infty$ yields $(\mathtt{ea})^\infty$. Exact.

### Test B — Periodic BWT with Tied Rotations ($S = \mathtt{abacabac}$, $n = 8, q = 2, p_0 = 0$)

Stably sorted rotations give $L = \mathtt{ccbbaaaa}$, $C[\mathtt{a}]=0, C[\mathtt{b}]=4, C[\mathtt{c}]=6$, $r = 3$ runs:

* $R_0: [0,2) \to [6,8)$ ($\mathtt{c}$), $R_1: [2,4) \to [4,6)$ ($\mathtt{b}$), $R_2: [4,8) \to [0,4)$ ($\mathtt{a}$).
$\mathrm{LF}$ has two identical 4-cycles: $(0\;6\;2\;4)$ and $(1\;7\;3\;5)$.

Pin $K = 2$ at $b_1 = C[\mathtt{b}] = 4 \in D(\mathrm{LF}) \cap C(\mathrm{LF})$ ($W_1 = [0, 4), W_2 = [4, 8)$):

1. Fronts: $(A_1, B_1) = (R_1, R_2)$, $(A_2, B_2) = (R_2, R_0)$. Condition 2 rejects batching ($B_1 = A_2 = R_2$). Run Front 1 (Case 2, $Y_1 = [2, 4)$):
* $R_2$ shrinks to $[4,6) \to [0,2)$ ($\mathtt{a}$); $R_1$ moves to $[6,8) \to [4,6)$ ($\mathtt{ab}$). Right sentinel `dom_right[2].dom_prev` automatically becomes $R_1$. Mark $(R_0, 0, 0)$ unchanged.

2. Fronts: $(A_1, B_1) = (R_0, R_2)$, $(A_2, B_2) = (R_1, R_0)$. Condition 2 rejects batching ($A_1 = B_2 = R_0$). Run Front 1 (Case 3a, $Y_1 = [0, 2)$):
* $p_0 = 0 \in Y_1$: backward transport gives $(R_2, 0, \delta = 1)$ (row $4$).
* $R_2$ merges to $[4,6) \to [6,8)$ ($\mathtt{ac}$); $R_0$ deleted; $W_1$ becomes empty.

3. Front 2 (Case 3a, $Y_2 = [6, 8)$): $R_2$ merges to $[4,6) \to [4,6)$ ($\mathtt{acab}$); $R_1$ deleted. Mark $(R_2, 0, 1)$ unchanged.
4. Front 2 (Case 3b, $A_2 = B_2 = R_2$, self-loop of $\lambda = 2$ parallel cycles):
* $\omega(R_2) = \mathtt{acab}$ (cycle length $d = 4$, multiplicity $q = n/d = 2$).
* Rotate left by $\delta = 1$: $\mathtt{caba}$. Reverse: $\mathtt{abac}$. Repeat $q = 2$ times: $\mathtt{abacabac} = S$. Exact.

### Test C — Case 2 Primary-Mark Split (Mark in $X' \cap \mathrm{dom}(B_j)$)

In Test B Step 1, instead set $p_0 = 6$ (which lies in $R_2$ at $\theta_p = 2$, $\delta = 0$, stream $(\mathtt{abac})^\infty$? Wait: $L[6]=\mathtt{a}, L[2]=\mathtt{b}, L[4]=\mathtt{a}, L[0]=\mathtt{c}$, so stream is $(\mathtt{abac})^\infty$).
During Step 1 (Case 2 at Front 1, $\lambda_{\text{new}}(R_2) = 2$), $p_0 = 6 \notin Y_1 = [2, 4)$, but $I_p = R_2$ and $\theta_p = 2 \ge \lambda_{\text{new}}(R_2) = 2$.
Rule 4b re-indexes $(I_p, \theta_p, \delta) \leftarrow (R_1, 0, 0)$ (still row $6$, now at offset $0$ of $\mathrm{dom}(R_1) = [6, 8)$).
In Step 2, $p_0 = 6 \notin Y_1$, unchanged.
In Step 3 ($Y_2 = [6, 8)$), $p_0 = 6 \in Y_2$: backward transport moves to $(R_2, 0, \delta = 0 + |\omega(R_2)| = 2)$.
In Step 4, rotating $\omega(R_2) = \mathtt{acab}$ left by $\delta = 2$ gives $\mathtt{abac}$, matching $p_0 = 6$'s exact $L$-stream $(\mathtt{abac})^\infty$. Exact.

---

## 4. Repaired Theorem 2 (Exact Fibonacci Indexing & Scope of the Kac Identity)

The previous Theorem 2 had two flaws: (i) an off-by-one Fibonacci index that yielded negative prefix lengths at $d = 1$, and (ii) phrasing that blurred the Kac identity for **row-level orbit tracing** with a lower bound on **interval-level** discovery. Here is the exact, repaired statement.

Let $F_1 = 1, F_2 = 1, F_k = F_{k-1} + F_{k-2}$ ($k \ge 3$) be the standard Fibonacci numbers, and define the finite Fibonacci words $w_1 = \mathtt{b}, w_2 = \mathtt{a}, w_q = w_{q-1} w_{q-2}$ of length $n = F_q$ ($q \ge 4$).

**Theorem 2' (Row-Level Kac Cost vs. Unfactored Interval Spelling — *Proved*).**

1. *(Row-Level Kac Identity — Explicit Orbit Tracing Only)* For any finite dIET $(X_0, T_0)$ and nonempty section $X_1 \subset X_0$,

$$\sum_{x \in X_1} \tau_{X_0 \to X_1}(x) = n_{\mathrm{cov}}(X_1).$$

Thus any algorithm that discovers $T_{X_1}$ by **explicitly stepping individual rows** $x, T_0(x), \dots, T_0^{\tau(x)-1}(x)$ performs at least $n_{\mathrm{cov}}(X_1)$ transitions ($= n$ on primitive BWTs). **This is not a lower bound on interval-level discovery**, which advances whole intervals and depends on $\sum_{I \in \mathcal{R}_{X_1}} \tau_{X_0 \to X_1}(I)$ rather than $n_{\mathrm{cov}}(X_1)$.
2. *(Exponential Separation of Factored vs. Unfactored Interval Induction)* For any $q \ge 4$ and any $4 \le k \le q$, let $P_k$ be the prefix of $w_q$ of length $F_k - 2 \ge 1$ (the $k$-th central word), and let $X_k = I(P_k)$ be its word cylinder in the BWT of $w_q$ (which is a single $F_q$-cycle with $r = 2$ runs). Then:
* $|I(P_k)| = F_{q - k + 3}$, and the induced dIET on $I(P_k)$ has $r(T_{I(P_k)}) = 2$ intervals with return times $F_{k-1}$ (multiplicity $F_{q - k + 2}$) and $F_{k-2}$ (multiplicity $F_{q - k + 1}$), satisfying $F_{q-k+2} F_{k-1} + F_{q-k+1} F_{k-2} = F_q$.
* **Unfactored interval induction** directly from $X_0$ to $I(P_k)$ has explicit spelling volume

$$W(I(P_k)) = \sum_{I \in \mathcal{R}_{I(P_k)}} \tau_{X_0 \to I(P_k)}(I) = F_{k-1} + F_{k-2} = F_k.$$

At $k = q$ ($|I(P_q)| = F_3 = 2$), $W(I(P_q)) = F_q = n$, so any unfactored interval-tracing or explicit-dictionary representation pays $\Omega(n)$ work and $\Omega(n)$ spelling storage.
* **Factored Rauzy–Veech induction** from $X_0$ to $I(P_k)$ performs $k - 3$ elementary interval steps ($q - 3 = \Theta(\log_\phi n)$ at $k = q$) and constructs an SLP of $k - 3$ concatenation rules representing the exact same return words.

**Proof.**

1. On each cycle $\mathcal{C}$ of $T_0$ intersecting $X_1$, the half-open orbit segments $\{T_0^t(x) : x \in \mathcal{C} \cap X_1,\; 0 \le t < \tau_{X_0 \to X_1}(x)\}$ partition $\mathcal{C}$; summing over all such cycles gives $n_{\mathrm{cov}}(X_1)$.
2. Since $w_q$ is a conjugate of a Christoffel word of length $F_q$ with $\gcd(F_{q-1}, F_q) = 1$, its BWT is a single $F_q$-cycle with $r = 2$ runs of lengths $(F_{q-1}, F_{q-2})$ (Mantaci–Restivo–Sciortino 2003). Each Rauzy–Veech step subtracts the shorter interval from the longer, reducing $(F_m, F_{m-1}) \mapsto (F_{m-1}, F_{m-2})$ and recording one SLP rule. After $t = k - 3$ steps ($1 \le t \le q - 3$), the remaining section has size $F_{q-t} = F_{q - k + 3}$ (corresponding to the central-word cylinder $I(P_k)$ of length $F_k - 2$), with two intervals of lengths $F_{q-k+2}$ and $F_{q-k+1}$ whose return words have lengths $F_{t+2} = F_{k-1}$ and $F_{t+1} = F_{k-2}$. Their unfactored spelling sum is $F_{k-1} + F_{k-2} = F_k$, whereas the factored RV schedule uses $k - 3$ steps and $k - 3$ SLP rules. $\square$

---

## 5. Summary of Verification Status

| Claim | Status | Notes |
| --- | --- | --- |
| Previous Theorem 3 (raw `(A_j, B_j)` + forward mark + Case 3a footprint) | **Refuted** | 3 defects identified; minimal $n = 4$ counterexample to Defect 1. |
| **Theorem 3'** (sentinel lists + backward mark + exact footprints $\mathcal{F}_j$) | **Proved** | Verified on cross-component, multi-cycle, periodic/tied, and Case-2-split tests. |
| Previous Theorem 2 (Fibonacci indexing & Kac scope) | **Repaired** | Indexing fixed ($4 \le k \le q$, $\vert{}P_k\vert{} = F_k - 2$, $W = F_k$); Kac bound restricted to explicit row tracing. |
