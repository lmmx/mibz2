# Gemini 4 Neon: response, verbatim

Received by Louis on 2026-11-19 in reply to [prompt.md](prompt.md); filed unedited. Its claims are
under review ([README.md](README.md)); nothing here is established until that review says so.

---

# Can a Better Induction Schedule Beat Rauzy–Veech BWT Reconstruction?

## 1. Formal Model: Finite Marked Discrete Interval Exchanges and Induction Schedules

Let $\Sigma$ be a finite ordered alphabet ($\vert{}\Sigma\vert{} \le 256$) and let $L \in \Sigma^n$ be a Burrows–Wheeler last column of length $n$ with a designated **primary row** $p_0 \in [n] := \{0, \dots, n-1\}$. Let $C[c] = \vert{}\{i \in [n] : L[i] < c\}\vert{}$ be the cumulative symbol-count array, and let $\psi = \mathrm{LF} : [n] \to [n]$ be the stable-rank LF permutation:

$$\psi(i) = C[L[i]] + \vert{}\{j < i : L[j] = L[i]\}\vert{}.$$

### 1.1 Finite Marked dIETs

A **finite marked discrete interval exchange (dIET)** is a tuple $\mathcal{I} = (X, T, \mathcal{R}, \omega, p, \delta)$ where:

1. $X \subseteq [n]$ is a nonempty finite ordered set of size $m = \vert{}X\vert{}$, written as a disjoint union of $c \ge 1$ maximal contiguous intervals $X = \bigsqcup_{j=1}^c [a_j, b_j)$ in $[n]$.
2. $T : X \to X$ is a bijection.
3. $\mathcal{R} = \{R_1, \dots, R_r\}$ is a partition of $X$ into $r$ **continuity intervals** (each contiguous in $X$) such that on every $R_k$, $T$ is an order-preserving translation in $X$-coordinates: if $x-1, x$ are adjacent in the same component of $X$ and both lie in $R_k$, then $T(x)$ immediately follows $T(x-1)$ in $X$.
4. $\omega : \mathcal{R} \to \Sigma^+$ assigns a **return word** (represented as a byte or an SLP/DAG node) to each interval $R_k$.
5. $(p, \delta) \in X \times \mathbb{Z}_{\ge 0}$ is the **transported primary mark**, where $p$ is the first row of $X$ reached from $p_0$ along the forward/backward orbit and $\delta$ is the byte offset between $p_0$ and $p$.

For the initial BWT, $X_0 = [n]$, $T_0 = \psi$, $\mathcal{R}_0$ is the partition into the $r$ maximal constant-symbol runs of $L$, $\omega_0(R_k) = L[R_k]$ (length $1$), $p = p_0$, and $\delta = 0$.

**Finite / Multi-Cycle Caveat.** Unlike minimal IETs on $[0, 1)$, $T_0 = \psi$ is a finite permutation and need not be a single $n$-cycle:

* If the input block is periodic, $S = u^q$ with $q > 1$ and primitive $u$ of length $d = n/q$, then the $q$ identical rotations of $S$ are tied and stably ordered; $\psi$ decomposes into $q$ disjoint cycles of length $d$ (e.g., for $S = \mathtt{abab}$, $L = \mathtt{bbaa}$ and $\psi = (0\; 2)(1\; 3)$).
* For an arbitrary (e.g., corrupted) $L \in \Sigma^n$, $\psi$ can have multiple cycles of unequal lengths.
* Consequently, a section $X \subset [n]$ is **admissible for $p_0$** only if $X$ intersects the cycle $\mathcal{C}(p_0)$ of $p_0$. Even then, other cycles may miss $X$ completely. All identities below therefore track $n_{\mathrm{cov}}(X) := \sum_{\mathcal{C} \cap X \neq \emptyset} \vert{}\mathcal{C}\vert{} \le n$, with equality $n_{\mathrm{cov}}(X) = n$ whenever every cycle meets $X$ (in particular for any primitive BWT block).

### 1.2 Induced Systems and Schedules

For any nonempty section $X' \subset X$, the **first-return time** on cycles meeting $X'$ is $\tau_{X \to X'}(x) = \min\{t \ge 1 : T^t(x) \in X'\}$, and the **induced map** is $T_{X'}(x) = T^{\tau_{X \to X'}(x)}(x)$ with concatenated return word:

$$\omega_{X'}(x) = \omega(x) \cdot \omega(T(x)) \cdots \omega(T^{\tau_{X \to X'}(x)-1}(x)).$$

If $p \in X \setminus X'$ and $\mathcal{C}(p) \cap X' \neq \emptyset$, the mark transports to $p' = T^s(p) \in X'$ where $s = \min\{t \ge 1 : T^t(p) \in X'\}$, with $\delta' = \delta + \sum_{t=0}^{s-1} \vert{}\omega(T^t(p))\vert{}$.

An **induction schedule** is a chain of nested sections

$$\mathcal{S} = (X_0 \supset X_1 \supset \dots \supset X_k)$$

together with:

* an **algorithmic discovery mode** at each step $j \to j+1$ (**row-level** on $X_j$ vs. **interval-level** on $\mathcal{R}_j$),
* a **spelling representation** (explicit flat strings vs. an SLP/DAG over $\mathcal{R}_j$), and
* a **residual decoder** that walks $\mathcal{I}_k$ on $m_k = \vert{}X_k\vert{}$ rows and expands the return words to emit $n$ bytes.

---

## 2. Key Insight: A Two-Part Structural Resolution

Our analysis yields two complementary theorems: a **sharp obstruction** explaining why non-Rauzy–Veech section schedules (word cylinders, skipped levels, sparse exceptions) repeatedly lose on paid construction cost, and a **constructive parallel schedule** that preserves every algebraic invariant of Rauzy–Veech induction while breaking its serial dependency bottleneck.

### Part A — Obstruction (Why Alternative Sections and Level-Skipping Lose)

1. **Exact Discontinuity Budget (Theorem 1 — *Proved*).** Let $D(T)$ and $C(T)$ be the sets of internal domain and codomain discontinuities of a dIET $(X, T, \mathcal{R})$ with $r = \vert{}\mathcal{R}\vert{}$ intervals. For *any* section $X' \subset X$ with $c$ maximal contiguous components and internal boundary set $\partial_{\mathrm{int}} X'$:

$$r(T_{X'}) \le r + (c - 1) + \big\vert{}\partial_{\mathrm{int}} X' \setminus (D(T) \cup C(T))\big\vert{}.$$

This single bound is **tight** (Section 3.1) and unifies:
* Rauzy–Veech end-cuts ($c = 1$, $\partial_{\mathrm{int}} X' \subset D(T) \cup C(T) \implies r(T_{X'}) \le r$),
* arbitrary single contiguous intervals ($c = 1 \implies r(T_{X'}) \le r + 2$), and
* word cylinders $I(P)$ (where dynamic clopenness under $\mathrm{LF}$ eliminates boundary splits entirely, giving $r(\mathrm{LF}_{I(P)}) \le r$).

2. **The Kac–Discovery & Unfactored-Spelling Obstruction (Theorem 2 — *Proved*).**
* In any **row-level** discovery schedule, inducing $X_0 \to X_1$ alone requires at least $n_{\mathrm{cov}}(X_1)$ evaluations of $T_0$ (for primitive BWTs, **at least $n$ steps**—the entire work of standard BWT inversion—before level 1 even begins).
* In an **interval-level** schedule, skipping intermediate induction levels replaces a factored Straight-Line Program (SLP) of return words with an unfactored trie/expansion whose total spelling volume $W(X) = \sum_{I \in \mathcal{R}_X} \tau(I)$ can be **exponentially larger** than the factored schedule: on Fibonacci/Christoffel BWTs ($r = 2$), direct induction to a constant-size word cylinder costs $\Theta(n)$ interval steps and spelling length, whereas factored Rauzy–Veech induction costs $O(\log n)$.

### Part B — Constructive Opportunity (Multi-Boundary Parallel Rauzy–Veech)

Standard Rauzy–Veech (`Cells`) is optimal in **interval count** ($\Delta r \le 0$) and **SLP factoring** ($O(1)$ rule per step), but has **parallel depth $\Theta(S_{\mathrm{RV}})$**: cutting only at the single right endpoint $m_t$ forces a strictly serial chain of dependent L2/L3 cache misses.

3. **Multi-Boundary Parallel Rauzy–Veech (Theorem 3 — *Proved*).**
Pin $K$ alphabet boundaries $0 = b_0 < b_1 < \dots < b_K = n$ chosen from $\{C[c] : c \in \Sigma\} \subset C(\mathrm{LF})$. Splitting any $L$-run straddling a $b_j$ adds at most $K - 1$ intervals **once** (e.g., $K = 32$ adds $\le 31$ intervals to $r \approx 200{,}000$, i.e., $< 0.02\%$). Thereafter:
* Every pinned boundary $b_j$ remains in $D(T) \cap C(T)$ under right-end cuts on its component $[a_j, b_j)$, with **zero coordinate shifting** (pure doubly-linked interval pointer updates) and **$\Delta r \le 0$ at every step**.
* Any subset $S \subseteq \{1, \dots, K\}$ of fronts whose touched intervals $\{A_j, P_j\}$ are pairwise disjoint is **conflict-free and confluent**: all $\vert{}S\vert{}$ Rauzy–Veech steps execute simultaneously with identical output to any serial order, exposing **$\vert{}S\vert{}$-way Memory-Level Parallelism (MLP)** and allowing **greedy max-width selection** ($\max_j \min(\vert{}A_j\vert{}, \vert{}B_j\vert{})$) to eliminate more rows per step.

---

## 3. Proofs and Counterexamples

### 3.1 Proof of Theorem 1 (Exact Discontinuity Budget) and Tightness Counterexample

**Theorem 1 (*Proved*).** *Let $(X, T, \mathcal{R})$ be a finite dIET with $r = \vert{}\mathcal{R}\vert{}$ intervals, internal domain cuts $D(T)$ ($\vert{}D(T)\vert{} = r - 1$), and internal codomain cuts $C(T)$. Let $X' = \bigsqcup_{i=1}^c [u_i, v_i) \subseteq X$ be a union of $c$ maximal contiguous components in $X$, and let $\partial_{\mathrm{int}} X'$ be the set of endpoints $u_i, v_i$ strictly inside $X$. Then the number of maximal continuity/return-word intervals of $T_{X'}$ satisfies*

$$r(T_{X'}) \le r + (c - 1) + \big\vert{}\partial_{\mathrm{int}} X' \setminus (D(T) \cup C(T))\big\vert{}.$$

*Moreover, if $X = [n]$, $T = \mathrm{LF}$, and $X' = I(P)$ is a nonempty word cylinder for a pattern $P \in \Sigma^+$, then $r(\mathrm{LF}_{I(P)}) \le r$.*

**Proof.** Work in the induced linear order of $X'$. Every internal discontinuity of $T_{X'}$ (where either $T_{X'}$ fails to preserve adjacency in $X'$ or the return word changes) is one of:

1. **Inter-component cuts:** At most $c - 1$ boundaries separating $v_i - 1$ from $u_{i+1}$.
2. **Intra-component splits:** Let $(x-1, x)$ be an adjacent pair inside a single component $[u_i, v_i)$ of $X'$ that splits under $T_{X'}$. Let $t \ge 0$ be the smallest index where $(T^t(x-1), T^t(x))$ fails to remain an adjacent pair that either both stay outside $X'$ or both return to $X'$. For all $0 \le s \le t$, $T^s(x) = T^s(x-1) + 1$ in $X$, and for $0 < s < t$, neither point is in $X'$. There are two cases at step $t$:
* **Type A ($T^t(x) \in D(T)$):**
* If $d := T^t(x) \in D(T) \cap \mathrm{int}(X')$ (meaning both $d-1, d \in X'$), then $t$ **must** be $0$: if $t \ge 1$, both $T^t(x-1) = d-1$ and $T^t(x) = d$ would return adjacently to $X'$ at step $t$, contradicting a split. Thus $d$ accounts for at most one split, namely $(d-1, d)$ at $t = 0$.
* If $d \in D(T) \setminus \mathrm{int}(X')$, then $t \ge 1$ and $x$ is uniquely determined as the first backward hit $x = T^{-\sigma'(d)}(d)$ where $\sigma'(d) = \min\{s \ge 1 : T^{-s}(d) \in X'\}$.
Hence every $d \in D(T)$ generates **at most one** Type A split, for at most $\vert{}D(T)\vert{} = r - 1$ splits total.

* **Type B ($T^t(x) \notin D(T)$):**
Then $t \ge 1$ and exactly one of $\{T^t(x-1), T^t(x)\}$ lies in $X'$, so $b := T^t(x) \in \partial_{\mathrm{int}} X'$. Because $T^t(x) = T^t(x-1) + 1$ *and* $T^{t-1}(x) = T^{t-1}(x-1) + 1$, applying $T^{-1}$ gives $T^{-1}(b) = T^{-1}(b-1) + 1$, so **$b \notin C(T)$**. For each $b \in \partial_{\mathrm{int}} X' \setminus (D(T) \cup C(T))$, $x$ is uniquely determined as $T^{-\sigma'(b)}(b)$.

Summing all cuts and adding $1$ yields

$$r(T_{X'}) \le 1 + (c - 1) + (r - 1) + \big\vert{}\partial_{\mathrm{int}} X' \setminus (D(T) \cup C(T))\big\vert{} = r + (c - 1) + \big\vert{}\partial_{\mathrm{int}} X' \setminus (D(T) \cup C(T))\big\vert{}.$$

**Word-cylinder refinement:** Let $X' = I(P)$ ($c = 1$) under $T = \mathrm{LF}$. If $(x-1, x)$ in $I(P)$ has no Type A split before step $t \ge 1$, then $L[\mathrm{LF}^s(x-1)] = L[\mathrm{LF}^s(x)]$ for all $0 \le s < t$. Since both rotations at $x-1, x$ begin with $P$, prepending the same $t$ bytes implies that the rotations at $\mathrm{LF}^t(x-1)$ and $\mathrm{LF}^t(x)$ share a common prefix of length $t + \vert{}P\vert{} > \vert{}P\vert{}$. Therefore $\mathrm{LF}^t(x-1) \in I(P) \iff \mathrm{LF}^t(x) \in I(P)$, so **no Type B split can occur**. Thus $r(\mathrm{LF}_{I(P)}) \le 1 + 0 + (r - 1) + 0 = r$. $\square$

---

**Tightness Counterexample (Interior Domain-Interval Elimination Increases $r$).**
Consider $n = 7$, $r = 3$ with domain intervals $R_1 = [0, 2)$, $R_2 = [2, 3)$, $R_3 = [3, 7)$ and:

$$T([0, 2)) = [5, 7), \qquad T([2, 3)) = [0, 1), \qquad T([3, 7)) = [1, 5).$$

* $T$ is a **single $7$-cycle**: $0 \to 5 \to 3 \to 1 \to 6 \to 4 \to 2 \to 0$.
* $D(T) = \{2, 3\}$ and $C(T) = \{1, 5\}$.
* Eliminate the single interior domain interval $Y = R_2 = [2, 3)$ (note $T(Y) \cap Y = \emptyset$, so every return time is $\le 2$). Then $X' = [0, 2) \cup [3, 7)$ has $c = 2$ and $\partial_{\mathrm{int}} X' = \{2, 3\} \subset D(T)$.
* In compressed coordinates $[0, 6)$ (mapping $0,1,3,4,5,6 \mapsto 0,1,2,3,4,5$), the first-return map $T_{X'}$ is:

$$[0, 2) \mapsto [4, 6), \quad [2, 3) \mapsto [1, 2), \quad [3, 4) \mapsto [0, 1), \quad [4, 6) \mapsto [2, 4).$$

* All four pieces have distinct shifts ($+4, -1, -3, -2$), so $r(T_{X'}) = 4 = r + 1$, matching Theorem 1's bound $3 + (2-1) + 0 = 4$ with equality.

This refutes the natural heuristic that eliminating an independent domain interval ($T(R_a) \cap R_a = \emptyset$) cannot increase interval count.

---

### 3.2 Proof of Theorem 2 (Kac–Discovery and Unfactored-Spelling Obstruction)

**Theorem 2 (*Proved*).**

1. *(Row-Level Kac Lower Bound)* For any finite dIET $(X_0, T_0)$ and any nonempty section $X_1 \subset X_0$, the sum of first-return times satisfies

$$\sum_{x \in X_1} \tau_{X_0 \to X_1}(x) = n_{\mathrm{cov}}(X_1).$$

Hence any row-level schedule $\mathcal{S} = (X_0, \dots, X_k)$ that discovers $T_{j+1}$ by tracing orbits of $X_{j+1}$ in $T_j$ performs $\sum_{j=0}^{k-1} n_{\mathrm{cov}}^{(j)}(X_{j+1}) \ge n_{\mathrm{cov}}(X_1)$ transitions ($= n$ for any primitive BWT).
2. *(Exponential Separation of Factored vs. Unfactored Interval Induction)* Let $F_q$ be the $q$-th Fibonacci word of length $n = f_q$ ($f_1 = 1, f_2 = 2, f_{q} = f_{q-1} + f_{q-2}$). Its BWT is a single $n$-cycle with $r = 2$ runs. For any $1 \le d \le q - 2$, let $X_d = I(P_d)$ be the word cylinder of the prefix $P_d$ of length $f_d - 2$ (occurring $f_{q-d+1}$ times). Then:
* **Direct (unfactored) interval induction** $X_0 \to X_d$ produces $r(T_{X_d}) = 2$ intervals with return times $f_d$ and $f_{d-1}$, so the explicit spelling volume is $W(X_d) = f_d + f_{d-1} = f_{d+1}$ and forward interval-tracing visits $\Omega(f_{d+1})$ states. Setting $d = q - 2$ ($\vert{}X_d\vert{} = 2$) gives $\Theta(n)$ work and $\Theta(n)$ spelling storage.
* **Factored Rauzy–Veech induction** $X_0 \to X_1 \to \dots \to X_d$ performs $O(d) = O(\log n)$ elementary interval operations and builds an SLP of size $O(\log n)$ representing the exact same return words.

**Proof.**

1. On each cycle $\mathcal{C}$ of $T_0$ intersecting $X_1$, the segments $\{T_0^t(x) : x \in \mathcal{C} \cap X_1,\; 0 \le t < \tau_{X_0 \to X_1}(x)\}$ partition $\mathcal{C}$ into disjoint subsets. Summing over all $x \in \mathcal{C} \cap X_1$ gives $|\mathcal{C}|$; summing over all cycles meeting $X_1$ gives $n_{\mathrm{cov}}(X_1)$.
2. By Mantaci–Restivo–Sciortino (2003), a word has $r = 2$ BWT runs if and only if it is a conjugate of a standard/Christoffel word (which includes every Fibonacci word $F_q$, a single $f_q$-cycle since $\gcd(f_{q-1}, f_q) = 1$). On a $2$-interval exchange with lengths $(f_{q-1}, f_{q-2})$, each Rauzy–Veech step performs one subtraction $(f_{m}, f_{m-1}) \mapsto (f_{m-1}, f_{m-2})$ and records one concatenation rule $w_m = w_{m-1} w_{m-2}$. After $d$ steps, the two return words have lengths $f_d$ and $f_{d-1}$, so $W(X_d) = f_{d+1}$, while the Rauzy–Veech SLP has $d = O(\log n)$ rules. $\square$

**Diagnostic Consequence for Section 4's Empirical Findings:**
This theorem explains *both* observed blowups without appealing to an unproved conservation law:

* **Skipping two token-BWT levels (28.1M $\to$ 89.0M lookups, 2.36$\times$ slower):** Dropping intermediate levels destroys the SLP factorization $\omega_{0 \to 3} = \sigma_1 \circ \sigma_2 \circ \sigma_3$, forcing the decoder to rediscover and re-traverse shared $X_1$-return words inside every distinct $X_3$-return word directly in the length-$n$ array $L$.
* **Sparse exceptions (2.35$\times$ acquisition vs. 0.628$\times$ emission):** Acquiring exceptions from an 8-byte context model operates at **row level** on $n$ positions, paying the full Kac floor $n$ (plus context hashing/verification) *before* emission begins.

---

### 3.3 Proof of Theorem 3 (Multi-Boundary Parallel Rauzy–Veech Induction)

**Construction.** Choose $K$ distinct alphabet boundaries $0 = b_0 < b_1 < \dots < b_K = n$ from $\{C[c] : c \in \Sigma\} \cup \{0, n\}$ (for instance, partitioning $[n]$ into $K \in \{16, 32\}$ roughly equal-mass segments). Since every $C[c]$ is a codomain boundary of $\mathrm{LF}$, we have $b_j \in C(\mathrm{LF})$ for all $j$. For each $b_j \notin D(\mathrm{LF})$, split the unique $L$-run containing $b_j$ at $b_j$ (and its image at $\mathrm{LF}(b_j)$). This yields a dIET on $r' \le r + K - 1$ intervals with $K$ pinned components $W_j = [a_j, b_j)$ ($a_j = b_{j-1}$) whose endpoints all lie in $D(T) \cap C(T)$.

Represent the dIET **without integer coordinates** as an array of $r'$ interval structs:

```text
Interval { len: u32, dom_prev, dom_next, cod_prev, cod_next: u32, word: SlpNode }
```

Maintain $K$ front pointers `(A_j, B_j)` giving the rightmost domain interval $A_j$ and rightmost codomain interval $B_j$ of component $W_j$, plus $K$ left-sentinel markers `dom_sent[j]`, `cod_sent[j]` at $a_j$.

**Single-Front Update at $b_j$ (with $\delta_j = \min(\lambda(A_j), \lambda(B_j))$):**

* **Case 1 ($\lambda(A_j) > \lambda(B_j)$):** Cut $Y_j = B_j$ (the codomain of interval $B_j$, whose domain lies elsewhere).
1. Shrink $A_j$: `A_j.len -= B_j.len`.
2. Unlink $B_j$ from the codomain list (advancing `cod_front[j] = B_j.cod_prev`) and re-insert $B_j$ immediately to the **right** of $A_j$ in the codomain list.
3. Record SLP rule: `B_j.word = concat(B_j.word, A_j.word)`.

* **Case 2 ($\lambda(A_j) < \lambda(B_j)$):** Exact dual (shrink `B_j.len -= A_j.len`, unlink $A_j$ from the domain list, re-insert $A_j$ immediately to the right of $B_j$ in the domain list, set `A_j.word = concat(B_j.word, A_j.word)`).
* **Case 3 ($\lambda(A_j) = \lambda(B_j)$):**
* If $A_j \neq B_j$: cut $Y_j = A_j = B_j$ (domain of $A_j$ equals codomain of $B_j$); merge $B_j$ into $A_j$'s target, unlink $A_j$ from domain and $B_j$ from codomain, and delete one interval ($\Delta r = -1$).
* If $A_j = B_j$: $A_j$ is a **closed self-loop** of $\lambda(A_j)$ parallel cycles of period $1$ in $T_X$ (each emitting `A_j.word`). Emit/record if it contains the primary mark $p$, unlink $A_j$ from both lists, and delete it ($\Delta r = -1$).

* **Zorich Acceleration (Optional):** If $A_j = B_j$ is impossible and $\lambda(A_j) \ge 2 \lambda(B_j)$ with $A_j$'s codomain *not* adjacent to $b_j$, standard Euclidean quotient $q = \lfloor (\lambda(A_j) - 1) / \lambda(B_j) \rfloor$ applies whenever $B_j$ would repeatedly hit $A_j$ (i.e., when $B_j$ maps back into $A_j$).

**Theorem 3 (*Proved*).** *Let $S \subseteq \{1, \dots, K\}$ be any batch of active fronts such that:*

1. *For every $j \in S$, $A_j \neq B_j$ (handled separately as Case 3b), and*
2. *(Conflict-Free Condition) For all distinct $j \neq j'$ in $S$, the touched interval sets $\{A_j, B_j, \mathrm{cod\_next}(A_j), \mathrm{cod\_prev}(B_j), \mathrm{dom\_next}(B_j), \mathrm{dom\_prev}(A_j)\}$ are pairwise disjoint.*

*Then:*

1. *(Component Length Conservation)* *For every component $W_j$, $\sum_{I \in \mathrm{dom}(W_j)} \lambda(I) = \sum_{I \in \mathrm{cod}(W_j)} \lambda(I)$ is invariant. Hence $W_j$'s domain list hits its left sentinel if and only if its codomain list does.*
2. *(Exactness & Confluence)* *Executing all $|S|$ updates simultaneously yields the exact induced dIET on $X' = X \setminus \bigsqcup_{j \in S} Y_j$, identical to executing the cuts in any serial order, with $r(T_{X'}) \le r(T_X)$.*
3. *(Primary Mark Transport)* *If $p \in Y_j$ (with offset $\theta$ from the left of $Y_j$), then in Case 1 ($Y_j = B_j$), $p$ maps under $T_X$ to $T_X(Y_j) \subset T_X(A_j) \subseteq X'$ at offset $\theta$ inside the newly re-inserted codomain of $B_j$, with $\delta' = \delta + |\omega(A_j)|$ (and dually in Case 2).*

**Proof.**

1. Under Case 1 at front $j$: component $W_j$ loses $\delta_j = \lambda(B_j)$ from `A_j` (domain) and $\delta_j$ from `B_j` (codomain). The component $W_{j'}$ containing `A_j`'s codomain loses $\delta_j$ from `A_j`'s codomain and gains $\delta_j$ from `B_j`'s re-inserted codomain (net $0$). The component containing `B_j`'s domain is unchanged. Case 2 is symmetric; Case 3 removes $\delta_j$ from both sides of $W_j$.
2. Because $W_j$ are pairwise disjoint, $Y_j \cap Y_{j'} = \emptyset$. The conflict-free condition ensures $T_X(Y_j) \cap Y_{j'} = \emptyset$ for all $j \neq j' \in S$ (since $T_X(Y_j) \subseteq \mathrm{cod}(A_j)$ and $Y_{j'} \subseteq \mathrm{cod}(B_{j'})$ or $\mathrm{dom}(A_{j'})$, which are disjoint from $\mathrm{cod}(A_j)$). Therefore every $x \in X'$ has $\tau_{X \to X'}(x) \in \{1, 2\}$, and if $T_X(x) \in Y_j$ then $T_X^2(x) \in X'$. Thus the $|S|$ cuts commute as set-valued first-return operators and as doubly-linked-list pointer rewirings. By Theorem 1, since $c$ does not increase and each new endpoint $b_j - \delta_j$ lies in $D(T_X) \cup C(T_X)$, $r(T_{X'}) \le r(T_X)$.
3. Immediate from $T_X(Y_j) \subseteq X'$ under the conflict-free condition. $\square$

---

## 4. Complete Cost Model ($n \le 900{,}000$, 7–10 Concurrent Workers)

Let $r \le n$ be the BWT run count (typically $r \in [1.5 \times 10^5, 3.5 \times 10^5]$ for compressible text at $n = 900{,}000$), $m_{\mathrm{target}}$ be the stop-and-walk residual size (e.g., $m_{\mathrm{target}} \le 32{,}768$ so the residual table fits in L1/L2 cache), and $S$ be the number of induction steps to reach $m_{\mathrm{target}}$.

| Cost Dimension | Dense LF (`Scalar-cap`) | Single-End RV (`Cells`) | Word-Cylinder / Skip-Level | Multi-Boundary Parallel RV ($K$ fronts) |
| --- | --- | --- | --- | --- |
| **Initial setup** | $O(n)$ sequential (build 3.6 MB LF table) | $O(n)$ sequential (build $r$ runs + links) | $O(n)$ sequential + rank/FM index | $O(n)$ sequential + $O(K)$ boundary splits |
| **Dependent memory accesses (induction)** | $0$ (no induction) | $S_{\mathrm{RV}}$ **serialized** L2/L3 misses (MLP $= 1$) | $\ge n$ row-level lookups (Kac floor); $3\times$ higher when skipping levels | $S_{\mathrm{MB}} \le S_{\mathrm{RV}}$ misses, issued in **$B$-way parallel batches** (MLP $= B \approx 8\text{--}16$) |
| **Dependent memory accesses (residual walk)** | $n$ random L2/L3 misses (MLP via couriers) | $m_{\mathrm{target}}$ L1/L2 hits + SLP expansion | $m_{\mathrm{target}}$ L1/L2 hits + dictionary copy | $m_{\mathrm{target}}$ L1/L2 hits + SLP expansion |
| **Arithmetic & branching** | Very low (1 load + 1 store/byte) | Moderate (list unlink/relink + SLP append) | High (FM backward search / context hash) | Moderate + $O(B)$ conflict check (registers/L1) |
| **Parallel depth (within 1 block)** | $O(n / C_{\mathrm{couriers}})$ | $\Theta(S_{\mathrm{RV}})$ (**strictly serial**) | $O(n / C_{\mathrm{couriers}})$ | $\Theta(S_{\mathrm{MB}} / B)$ (**$B\times$ lower depth**) |
| **Peak retained memory per worker** | $\approx 3.6\text{ MB}$ (`u32` LF) + $0.9\text{ MB}$ out | $O(r)$ intervals + SLP ($\approx 16\text{--}24\text{ bytes}/r$) | $O(n)$ FM/rank + explicit dictionary $W(X)$ | Same as `Cells` $+ < 1\text{ KB}$ for $K$ fronts |

### Critical Cache & Multi-Worker Reality Check

At $7\text{--}10$ concurrent workers on a typical desktop/server L3 cache (16–36 MB shared):

* `Scalar-cap`'s $3.6\text{ MB} \times 8 = 28.8\text{ MB}$ working set saturates L3, turning dependent LF loads into L3/DRAM stalls.
* `Cells` shrinks the working set to $O(r)$ during induction and $O(m_{\mathrm{target}})$ during emission, **but** if $r \approx 250{,}000$ and each interval struct is 20–24 bytes, the interval table is **5–6 MB**—larger than `Scalar-cap`'s 3.6 MB dense LF table unless packed tightly (e.g., 16 bytes/interval $\implies 4.0\text{ MB}$ at $r = 250{,}000$, or $2.4\text{ MB}$ at $r = 150{,}000$).
* **Conflict rate of Multi-Boundary RV:** Each front touches $\le 6$ intervals out of $r_{\mathrm{live}}$. By a union bound, the expected number of conflicts in a batch of $B$ fronts is at most $\frac{6 B (B-1)}{2 r_{\mathrm{live}}} = \frac{3 B^2}{r_{\mathrm{live}}}$. For $B = 16$ and $r_{\mathrm{live}} = 100{,}000$, this is **$0.0077$ conflicts per batch** ($> 99\%$ of batches execute all $16$ fronts in parallel). Even at $r_{\mathrm{live}} = 5{,}000$, $> 85\%$ of fronts execute per batch.

---

## 5. Concrete Comparison with `Cells`

1. **Same algebraic engine, different schedule:** `Cells` is the $K = 1$ special case of Multi-Boundary Parallel RV (one pinned boundary at $b_1 = n$). Every algebraic property that makes `Cells` work—$\Delta r \le 0$, coordinate-free pointer updates, $O(1)$ SLP rule creation, zero $O(n)$ discovery passes—is **strictly preserved** for $K > 1$.
2. **Breaking the serial latency wall:** In `Cells`, step $t+1$ cannot even compute the address of its next interval until step $t$'s cache miss returns. With $K = 16\text{--}32$ fronts, the decoder issues $B$ independent `prefetch` / load chains for `(A_j, B_j)` simultaneously, saturating the CPU's Miss Status Holding Registers (MSHRs) just as `mibz2`'s 32 couriers do during the walk phase.
3. **Greedy volume elimination:** In `Cells`, if the single right endpoint $m_t$ lands on a narrow interval ($\min(\lambda(A), \lambda(B)) = 1$), it must pay a full cache miss to eliminate $1$ row. With $K = 32$ (or $K = 64$ candidates, batching the top $B = 16$), the schedule prioritizes fronts with the **largest** $\min(\lambda(A_j), \lambda(B_j))$ (and exact-match $\Delta r = -1$ fronts), reducing the total number of induction steps $S_{\mathrm{MB}}$ needed to reach $m_{\mathrm{target}}$.
4. **What does NOT improve:** If $r \approx n$ (incompressible blocks where every run has length $1$), neither `Cells` nor Multi-Boundary RV can beat a dense LF table's memory footprint; both should detect high $r/n$ during the initial sequential scan and fall back to `Scalar-cap`.

---

## 6. Relevant Prior Art vs. New Deductions

* **Rauzy (1979), Veech (1982), Zorich (1996):** Classical single-end induction and Euclidean acceleration on continuous minimal IETs; proves $\Delta r \le 0$ for right-end cuts.
* **Durand (1998), Lopez–Narbel (1999), Vuillon (2001):** Derived sequences, return substitutions, and the $2$-return-word characterization of Sturmian words.
* **Mantaci–Restivo–Sciortino (2003):** Characterization of $r = 2$ BWTs as conjugates of Christoffel/standard words (used in our Theorem 2.2 separation).
* **Dolce & Perrin (2017):** Branching / two-sided Rauzy induction on IETs (allowing cuts at either end $0$ or $n$) and bifix-code return words.
* **Boucher–Gagie–Kuhnle–Manzini (2019):** Prefix-Free Parsing (PFP), which is word-cylinder induction $I(P)$ applied to forward BWT construction on repetitive pangenomes.
* **Nishimoto & Tabei (2021):** The move structure, which statically splits dIET intervals so each codomain interval holds $O(1)$ domain boundaries for $O(1)$ row-level LF queries—dual to RV induction's dynamic contraction.

**Distinct deductions in this note:**

1. The unified **Exact Discontinuity Budget** (Theorem 1) in terms of $|\partial_{\mathrm{int}} X' \setminus (D(T) \cup C(T))|$ and the minimal 7-cycle counterexample showing interior domain-interval elimination increases $r$.
2. The **Kac–Discovery & Unfactored-Spelling Obstruction** (Theorem 2) formally explaining the empirical blowups of level-skipping and row-level sparse-exception acquisition.
3. **Multi-Boundary Parallel Rauzy–Veech** (Theorem 3): pinning $K$ alphabet boundaries $C[c] \in C(\mathrm{LF})$ to obtain a coordinate-free, confluent, $B$-way MLP induction schedule with $\Delta r \le 0$.

---

## 7. Decisive Experiment ($n \le 900{,}000$)

To determine in under a day whether Multi-Boundary Parallel RV has practical merit over `Cells` and `Scalar-cap`, run a single controlled ablation on the existing 10-block benchmark corpus at **both 1 worker and 8 concurrent workers**:

1. **Baseline:** Current `Cells` (single-end RV, $K = 1$, stop-and-walk at $m_{\mathrm{target}}$) and `Scalar-cap`.
2. **Variant A (Pure MLP, $K = B = 16$):** Pin $16$ alphabet boundaries $C[c_j]$ partitioning $[n]$ into $\approx n/16$ mass each. Run conflict-free batched RV (round-robin over all non-conflicting fronts) with software prefetching on `A_j.cod_next` / `B_j.dom_next`, stopping at the same $m_{\mathrm{target}}$.
3. **Variant B (Greedy Width + MLP, $K = 64$, batch $B = 16$):** Pin $64$ boundaries; at each round, pick the $16$ conflict-free fronts maximizing $\min(\lambda(A_j), \lambda(B_j))$.

**Measurements to record:**

* **Induction steps** $S_{\mathrm{MB}}$ vs. $S_{\mathrm{RV}}$ (tests whether greedy width selection meaningfully cuts step count).
* **Cycles per induction step** at 1 and 8 workers (tests whether $16$-way MLP actually hides L2/L3 latency under multi-worker cache contention).
* **Peak live SLP / induction width** during reverse reconstruction (tests whether dispersing cuts across $K$ components inflates live rebuild memory).
* **End-to-end inverse-stage throughput** relative to `Scalar-cap` (target: $\ge 1.25\times$ at 7–10 workers, peak RSS $\le$ `Scalar-cap`).

**Falsification criteria:**

* If Variant A's induction-stage CPU is **not** at least $1.8\times$ faster than `Cells`'s induction stage at 8 workers, then `Cells` is already arithmetic/branch-bound rather than cache-latency-bound, and parallelizing the induction schedule cannot close the gap to $1.25\times$.
* If Variant B increases peak live induction width by $> 25\%$ without a compensating drop in $S_{\mathrm{MB}}$, then greedy max-width scheduling hurts memory locality during emission.

---

## 8. Research Judgement

**What is settled:**

* **Row-level section discovery is a dead end for beating `Scalar-cap` on general blocks.** By Theorem 2.1, any row-level acquisition (word cylinders via FM-search, or 8-byte sparse-exception discovery) pays at least $n$ steps before emission begins; it can only win if acquisition is strictly sequential and branch-free (cheaper per byte than a single courier-pipelined LF load), which 8-byte context models are not.
* **Skipping intermediate induction levels is mathematically counterproductive.** By Theorem 2.2, intermediate levels are not redundant scaffolding—they are the **SLP factorization** of the return words. Removing them trades a compact DAG for an exponentially larger explicit spelling volume.

**What remains open and likely to fail:**

* **The $r/n$ regime boundary:** On bzip2 blocks where $r/n > 0.45$, an interval-exchange representation (16–24 bytes/interval) approaches or exceeds the 3.6 MB dense `u32` LF table in size, erasing the cache-footprint advantage at 7–10 workers. No induction schedule—parallel or serial—can overcome a working set larger than the dense LF table when runs are short.
* **Induction width under multi-boundary cuts:** Theorem 3 guarantees $\Delta r \le 0$ and confluence, but does **not** bound the peak live width of the reverse rebuild. Dispersing cuts across $K$ components could keep more long return words simultaneously live during emission.

**What justifies further work:**
If the decisive experiment shows that (a) `Cells`'s induction phase is indeed L2/L3-latency-bound at 8 workers, (b) $K = 16$ Multi-Boundary Parallel RV cuts induction wall-time by $\ge 2\times$ without inflating peak rebuild width, and (c) an adaptive $r/n$ gate routes low-$r/n$ blocks to Multi-Boundary RV and high-$r/n$ blocks to `Scalar-cap`, then the $1.25\times$ throughput target at 7–10 workers is realistically within reach without increasing peak memory. Otherwise, the evidence points strongly to a structural ceiling: at $n \le 900{,}000$, once `Cells` already achieves $\Delta r \le 0$ and $O(1)$ SLP factoring, further gains lie in **compact interval packing and emission layout**, not in exotic section geometries.
