# Gemini 4 Neon: fourth response, verbatim

Received by Louis on 2026-11-19 in reply to the round-4 prompt, which asked about reducing total induction work rather than dependency depth. Filed unedited, except that the embedded verifier is filed separately as [verify_boundary_records.py.txt](verify_boundary_records.py.txt). The review is in [README.md](README.md).

---

# Boundary-Persistence Arcs: An Exact Characterization of Total Induction Work in Run-Length BWT Inversion

**Execution note:** I cannot run Python in this environment. Every claim below is tagged as **Proved** (complete mathematical proof), **Conjecture / Analytical Model**, or **Awaiting Verification** (accompanied by a self-contained standard-library Python verifier and explicit pass/fail criteria).

---

## 1. Central Thesis

Can an induced representation invert a run-length BWT ($n \le 900{,}000$, admitted regime $r/n \in [0.10, 0.15]$) with **substantially less total paid work** than `Cells`, rather than merely better memory-level parallelism?

We establish an **exact combinatorial identity**—the **Boundary-Record Theorem**—that governs the total step count of *every* monotone component-suffix induction schedule (including single-end Rauzy–Veech `Cells`, two-sided branching RV, and Multi-Boundary RV with arbitrary adaptive front selection):

1. **Exact Work Identity (Theorem 1 — *Proved*):** The $r$ run boundaries of any finite dIET partition all boundary-hitting cycles into $r$ pairwise disjoint **persistence arcs** $A(c) = (c, T(c), \dots, T^{\ell(c)}(c))$. Under *any* monotone induction schedule $\pi$, the exact number of elementary induction steps is

$$S_\pi(T) = r' + \sum_{c \in \overline{C}'} \Big( p_\pi\big(A'(c)\big) + s_\pi\big(A'(c)\big) \Big),$$



where $p_\pi$ and $s_\pi$ count the non-global prefix and suffix maxima of $\pi$ on each arc $A'(c)$.
2. **Hard Floor (Corollary 1 — *Proved*):** Even an **omniscient** offline unaccelerated schedule must pay at least $2r - \vert{}\overline{D} \cap \overline{C}\vert{}$ steps (roughly $1.85\,r\text{--}1.90\,r$ at $r/n \in [0.10, 0.15]$), and even with Zorich acceleration at least $r$ steps. Meanwhile, an oblivious schedule like `Cells` pays $\approx r\big(2\ln(n/r) + 2\gamma - 1\big) \approx 3.8\,r\text{--}4.8\,r$ full steps (or fewer with stop-and-walk). Thus, within monotone induction, the **maximum theoretically available work reduction** over `Cells` is bounded by a factor of roughly $2\times\text{--}2.5\times$ on full induction, and less once stop-and-walk is already tuned.
3. **The Unique Escape Valve Within Monotone Induction (Max-Cut over Pinned Alphabet Boundaries):** Because $p_\pi + s_\pi$ shrinks whenever *interior* points ($x \notin \overline{D} \cup \overline{C}$) are eliminated before *boundary* points, an adaptive **Max-Cut** policy ($\arg\max_j \min(\lambda(A_j), \lambda(B_j))$) over all $\vert{}\Sigma\vert{} \le 256$ pinned byte boundaries $C[c]$ is the natural test case for whether total induction steps can actually be pushed from $\approx 4\,r$ toward the $2\,r$ floor on heavy-tailed BWTs.

---

## 2. Formal Model and the Exact Arc Decomposition

Let $([n], T, \mathcal{R})$ be a finite discrete interval exchange (dIET): $T : [n] \to [n]$ is a bijection (possibly multi-cycle or arising from a periodic BWT with tied rotations) and $\mathcal{R} = \{R_0, \dots, R_{r-1}\}$ is a partition of $[n]$ into $r$ contiguous intervals on each of which $T$ acts as a $+1$-translation ($T(x) = T(x-1) + 1$ for adjacent $x-1, x \in R_k$). We do **not** require $\mathcal{R}$ to be maximally merged (so it covers initial BWT runs as well as refinements obtained by pinning $K - 1$ alphabet boundaries).

Define the domain and codomain left-endpoint sets:


$$\overline{D}(\mathcal{R}) := \{\min R : R \in \mathcal{R}\}, \qquad \overline{C}(\mathcal{R}) := \{\min T(R) : R \in \mathcal{R}\}.$$

**Lemma 1 (Persistence Arc Decomposition — *Proved*).**
*For any finite dIET $([n], T, \mathcal{R})$:*

1. *$T(\overline{D}(\mathcal{R})) = \overline{C}(\mathcal{R})$, and $\vert{}\overline{D}(\mathcal{R})\vert{} = \vert{}\overline{C}(\mathcal{R})\vert{} = r$.*
2. *For every $c \in \overline{C}(\mathcal{R})$, the persistence length $\ell(c) := \min\{k \ge 0 : T^k(c) \in \overline{D}(\mathcal{R})\}$ is finite, and the **persistence arcs***

$$A(c) := \big(c,\; T(c),\; \dots,\; T^{\ell(c)}(c)\big) \qquad \big(c \in \overline{C}(\mathcal{R})\big)$$



*are pairwise disjoint subsets of $[n]$.*
3. *Their union is exactly the union of all cycles of $T$ that intersect $\overline{C}(\mathcal{R})$, so*

$$\sum_{c \in \overline{C}(\mathcal{R})} \big(\ell(c) + 1\big) = n_{\mathrm{cov}}\big(\overline{C}(\mathcal{R})\big) \le n,$$



*with equality $n_{\mathrm{cov}} = n$ whenever every cycle meets $\overline{C}(\mathcal{R})$ (in particular for every primitive BWT).*

**Proof.**

1. For each $R = [a, b) \in \mathcal{R}$, $T$ is a $+1$-translation on $R$, so $\min T(R) = T(a) = T(\min R)$. Since $T$ is a bijection, $T(\overline{D}) = \overline{C}$ and $\vert{}\overline{C}\vert{} = \vert{}\overline{D}\vert{} = r$.
2. For $c \in \overline{C}$, $T^{-1}(c) \in \overline{D}$ lies in the same finite cycle $\mathcal{C}$ of $T$, so $\mathcal{C} \cap \overline{D} \neq \emptyset$ and $\ell(c) < \vert{}\mathcal{C}\vert{}$ is finite. For disjointness, suppose $T^i(c) = T^j(c')$ with $0 \le i \le \ell(c)$ and $0 \le j \le \ell(c')$. Without loss of generality $i \le j$, so $c = T^{j-i}(c')$. If $j > i$, then $T^{j-i-1}(c') = T^{-1}(c) \in \overline{D}$ with $0 \le j - i - 1 < \ell(c')$, contradicting minimality of $\ell(c')$. Hence $i = j$ and $c = c'$.
3. Every $x$ in a cycle $\mathcal{C}$ meeting $\overline{C}$ belongs to $A(c)$ where $c$ is the closest preceding element of $\overline{C}$ along $\mathcal{C}$. Summing $\vert{}A(c)\vert{} = \ell(c) + 1$ over $c \in \overline{C}$ counts each element of $\bigcup_{\mathcal{C} \cap \overline{C} \neq \emptyset} \mathcal{C}$ exactly once. $\square$

*(Remark on periodic / multi-cycle BWTs: any cycle $\mathcal{C}$ disjoint from $\overline{C}$ is also disjoint from $\overline{D}$; its rows ride passively inside intervals of length $\ge 2$ whose left endpoints lie in $\overline{D}$, contributing $0$ to the arc count and $0$ induction steps.)*

---

## 3. Main Result: The Exact Boundary-Record Theorem

Consider any **monotone component-suffix induction schedule** $\mathcal{S}$:

* Start by pinning $K - 1 \ge 0$ internal boundaries in $\overline{C}(\mathcal{R})$ to partition $[n]$ into $K$ components $W_1, \dots, W_K$ (splitting at most $K - 1$ domain runs, yielding refined partition $\mathcal{R}'$ of size $r' \le r + K - 1$ and refined arcs $A'(c)$). Note that $K = 1$ is standard single-end Rauzy–Veech (`Cells`).
* At each step $t = 1, 2, \dots, S$, choose an active component $j_t \in \{1, \dots, K\}$ and execute one elementary Rauzy–Veech step (Case 1, Case 2, Case 3a, or Case 3b), removing the suffix $Y_t = [m_{j_t} - \delta_t, m_{j_t})$ of $W_{j_t}$, until reaching a stopping section $X \subseteq [n]$ (with $X = \emptyset$ for full induction).
* Define the **elimination order** $\pi : [n] \to \mathbb{Z} \times \mathbb{Z}$ lexicographically:
* For every $x \in [n] \setminus X$ removed at step $t(x)$, set $\pi(x) := (t(x), -x)$.
* For every surviving $x \in X$, set $\pi(x) := (S + 1, -x)$.
(Larger $\pi$ means eliminated *later*; within a single cut $Y_t$, the left endpoint $m_{j_t} - \delta_t$ has the largest $\pi$ in $Y_t$.)



For any arc $A'(c) = (x_0, \dots, x_\ell)$ and stopping section $X$, let $i^* := \arg\max_{0 \le k \le \ell} \pi(x_k)$ (so $x_{i^*} \in X$ iff $A'(c) \cap X \neq \emptyset$). Define:

* **Prefix records outside $X$:** $p_{\pi, X}(A'(c)) := \big\vert{}\big\{ i \in \{0, \dots, i^* - 1\} : x_i \notin X \text{ and } \pi(x_i) = \max_{0 \le k \le i} \pi(x_k) \big\}\big\vert{}$.
* **Suffix records outside $X$:** $s_{\pi, X}(A'(c)) := \big\vert{}\big\{ i \in \{i^* + 1, \dots, \ell\} : x_i \notin X \text{ and } \pi(x_i) = \max_{i \le k \le \ell} \pi(x_k) \big\}\big\vert{}$.

*(For single-end RV $K = 1$ with $X = [0, m)$, $\pi(x)$ is strictly decreasing in $x$, so prefix/suffix maxima of $\pi$ are simply **prefix/suffix minima of the row coordinates** $x_k \ge m$.)*

**Theorem 1 (Exact Boundary-Record Theorem — *Proved*).**
*For any finite dIET $([n], T, \mathcal{R})$ and any monotone component-suffix induction schedule $\mathcal{S}$ stopping at section $X \subseteq [n]$:*

1. *(Exact Surviving Interval Count)* *The number of live intervals in the induced dIET on $X$ is*

$$r_\pi(X) = \big\vert{}\big\{ c \in \overline{C}(\mathcal{R}') : A'(c) \cap X \neq \emptyset \big\}\big\vert{}.$$


2. *(Exact Step Count)* *The exact number of elementary Rauzy–Veech steps executed to reach $X$ is*

$$S_\pi(T; X) = \sum_{c \in \overline{C}(\mathcal{R}')} \Big( p_{\pi, X}\big(A'(c)\big) + s_{\pi, X}\big(A'(c)\big) + \mathbf{1}\big[A'(c) \cap X = \emptyset\big] \Big).$$



*In particular, for full induction ($X = \emptyset$),*

$$S_\pi(T) = r' + \sum_{c \in \overline{C}(\mathcal{R}')} \Big( p_\pi\big(A'(c)\big) + s_\pi\big(A'(c)\big) \Big).$$



**Proof.**
Let $X_t \subseteq [n]$ be the surviving section before step $t$ ($X_1 = [n]$, $X_{S+1} = X$). We prove by induction on $t$ that the live intervals $\mathcal{R}_t$ have domain left-endpoints $\overline{D}_t = \{d_t(c) : A'(c) \cap X_t \neq \emptyset\}$ and codomain left-endpoints $\overline{C}_t = \{c_t(c) : A'(c) \cap X_t \neq \emptyset\}$, where for each surviving arc $A'(c) = (x_0, \dots, x_\ell)$:

* $c_t(c) = x_{i_t}$ is the **first** element of $A'(c)$ in $X_t$, and
* $d_t(c) = x_{j_t}$ is the **last** element of $A'(c)$ in $X_t$.

*Base case ($t = 1$, $X_1 = [n]$):* $c_1(c) = x_0 = c$ and $d_1(c) = x_\ell = d$.

*Inductive step (step $t$, cutting $Y_t = [m_j - \delta_t, m_j)$ from component $W_j$):*
Let $a_j = \max(\overline{D}_t \cap W_j)$ and $b_j = \max(\overline{C}_t \cap W_j)$, so $m_j - \delta_t = \max(a_j, b_j)$. Because no element of $\overline{D}_t \cup \overline{C}_t$ lies strictly inside $(m_j - \delta_t, m_j)$, the only boundary in $Y_t$ is its left endpoint $m_j - \delta_t$.

* **Case 1 ($a_j < b_j$):** Then $b_j = c_t(c^*) = x_{i_t}$ for a unique surviving arc $A'(c^*)$ (by pairwise disjointness of arcs, Lemma 1.2). Since $d_t(c^*) \in \overline{D}_t$ and $a_j < b_j$, we have $d_t(c^*) \neq b_j$, so $d_t(c^*) \notin Y_t$, hence $d_t(c^*) \in X_{t+1}$ and $i_t < j_t$.
* Every other surviving arc $A'(c) \neq A'(c^*)$ has $c_t(c), d_t(c) \notin Y_t$, so both endpoints remain unchanged in $X_{t+1}$.
* For $A'(c^*)$: $x_0, \dots, x_{i_t - 1}$ were already eliminated before step $t$ (so $\pi(x_k) < \pi(x_{i_t})$ for all $k < i_t$), while $x_{j_t} \in X_{t+1}$ has $\pi(x_{j_t}) > \pi(x_{i_t})$. Thus $i_t < i^*$ and $x_{i_t}$ is a **strict prefix maximum** of $\pi(A'(c^*))$ outside $X$.
* In RV Case 1, $A_j \neq B_j$, so $\mathrm{cod}(A_j) \cap Y_t = \emptyset$. The codomain cut $b_j = x_{i_t}$ is replaced by $T_{X_t}(b_j) \in \mathrm{cod}(A_j) \subseteq X_{t+1}$, which is the **first** point after $x_{i_t}$ in the forward $T$-orbit of $x_{i_t}$ lying in $X_{t+1}$. Since $x_{j_t} \in X_{t+1}$ lies in $\{x_{i_t+1}, \dots, x_\ell\}$, this first hit is the first element of $A'(c^*)$ in $X_{t+1}$, maintaining the invariant and charging exactly $+1$ to $p_{\pi, X}(A'(c^*))$.


* **Case 2 ($b_j < a_j$):** Exact dual; $a_j = d_t(c^*) = x_{j_t}$ is a **strict suffix maximum** of $\pi(A'(c^*))$ outside $X$, replaced by $T_{X_t}^{-1}(a_j) \in X_{t+1}$, maintaining the invariant and charging $+1$ to $s_{\pi, X}(A'(c^*))$.
* **Case 3a/3b ($a_j = b_j$):** By pairwise disjointness of arcs, $a_j = b_j$ forces $c_t(c^*) = d_t(c^*) = a_j$ for a unique arc $A'(c^*)$. Hence $A'(c^*) \cap X_t = \{a_j\}$, so $A'(c^*) \cap X_{t+1} = \emptyset$ and $a_j = x_{i^*}$ is the global maximum of $\pi(A'(c^*))$. Both endpoints are deleted ($\Delta r = -1$), charging $+1$ to $\mathbf{1}[A'(c^*) \cap X = \emptyset]$.

Since every step $t \in \{1, \dots, S\}$ is in bijection with one term of the sum, both identities follow. $\square$

---

## 4. Consequences: Obstruction vs. Constructive Escape Valve

### 4.1 Hard Work Lower Bound (Obstruction)

**Corollary 1 (Universal Lower Bound for Unaccelerated Monotone Schedules — *Proved*).**
*For any finite dIET $([n], T, \mathcal{R})$ and any monotone component-suffix schedule $\mathcal{S}$ (even an omniscient offline schedule choosing the optimal elimination order $\pi$):*


$$S_\pi(T) \ge r + \big\vert{}\{c \in \overline{C}(\mathcal{R}) : \ell(c) \ge 1\}\big\vert{} = 2r - \vert{}\overline{D}(\mathcal{R}) \cap \overline{C}(\mathcal{R})\vert{}.$$


*(With Zorich Euclidean acceleration collapsing consecutive Case 1/2 steps on the same pair, $S_{\mathrm{Zorich}}(T) \ge r$.)*

**Proof.** For any refined schedule ($r' \ge r$), if $A(c)$ has $\ell(c) \ge 1$ ($x_0 \neq x_\ell$), either an internal split increases $r'$ by at least $1$ (adding $\ge 1$ annihilation step), or $A(c)$ remains unsplit and its unique $\pi$-maximum $x_{i^*}$ cannot equal both $x_0$ and $x_\ell$, forcing $p_\pi(A(c)) + s_\pi(A(c)) \ge 1$. Summing over $c \in \overline{C}(\mathcal{R})$ yields $S_\pi(T) \ge r + \vert{}\{c : \ell(c) \ge 1\}\vert{} = 2r - \vert{}\overline{D} \cap \overline{C}\vert{}$. $\square$

**Implication for $r/n \in [0.10, 0.15]$:**
In a typical BWT with $r/n = 0.12$, $\vert{}\overline{D} \cap \overline{C}\vert{} \approx r^2/n = 0.12\,r$, so **no** unaccelerated monotone schedule can finish in fewer than $\approx 1.88\,r$ steps (and no Zorich-accelerated schedule in fewer than $r$ steps).

### 4.2 Why Oblivious Schedules (`Cells`) Pay $\Theta(r \log(n/r))$

**Analytical Model (Conjecture for Real BWTs, Exact for Uniformly Ordered Arcs):**
If an arc $A(c)$ of length $\ell + 1$ has its coordinates in uniformly random relative order with respect to an oblivious elimination order $\pi$ (such as `Cells`'s $\pi(x) = -x$), then the expected number of prefix maxima and suffix maxima are each $H_{\ell + 1} - 1$ (Knuth's left-to-right maxima). Summing over $r$ arcs of mean length $\bar{\ell} + 1 = n/r$:


$$\mathbb{E}\big[S_{\mathrm{RV}}(T)\big] \approx r \cdot \big(2 \ln(n/r) + 2\gamma - 1\big) \approx \begin{cases} 4.76\,r & (r/n = 0.10), \\ 3.95\,r & (r/n = 0.15). \end{cases}$$


With **stop-and-walk** stopping at section $[0, m)$, differentiating total cost $F(m) = S_{\mathrm{RV}}(T; m) + \beta m$ (where $\beta$ is the cost ratio of one residual walk step to one induction step) gives the optimal knee $\frac{r_{\mathrm{RV}}(m)}{m} \approx \frac{\beta}{2}$. Since $r/n$ is *already* $0.10\text{--}0.15$ at the start, stop-and-walk in `Cells` hits this knee early—leaving only a modest gap ($2\times\text{--}2.5\times$ at best on full induction, smaller with stop-and-walk) between `Cells` and Corollary 1's hard floor.

### 4.3 The Only Constructive Escape Valve: Adaptive Max-Cut Over Pinned Alphabet Boundaries ($K \le 256$)

Theorem 1 identifies *exactly* what an adaptive schedule would have to do to reduce **total steps** (not just MLP):

* To minimize $p_\pi(A'(c)) + s_\pi(A'(c))$, interior points $x_1, \dots, x_{\ell-1} \notin \overline{D} \cup \overline{C}$ must receive **small $\pi$** (eliminated early) while boundary points $x_0, x_\ell$ receive **large $\pi$** (eliminated late).
* Pinning all $|\Sigma| \le 256$ byte boundaries $C[c] \in \overline{C}$ adds at most $255$ intervals to $r \approx 100{,}000$ ($< 0.25\%$).
* An adaptive **Max-Cut** policy chooses at each step $j^* = \arg\max_j \min(\lambda(A_j), \lambda(B_j))$ (maintaining the $\le 256$ fronts in a 1 KB L1-resident tournament tree), eliminating $\delta_{j^*} - 1$ interior points per boundary step.

**Two concrete risks that could defeat Max-Cut:**

1. *Short-codomain blocking:* A long domain run $A_j$ paired with a length-$1$ codomain run $B_j$ has $\delta_j = 1$, hiding the long run from Max-Cut until short neighbors are peeled away.
2. *Narrow headroom:* On heavy-tailed Wikidata blocks at $r/n \in [0.10, 0.15]$, if `Cells` with stop-and-walk already operates near $2\,r$ steps (e.g., if real BWT arcs have non-random coordinate correlations that already give fewer than $2\ln(n/r)$ records), Max-Cut cannot beat the Corollary 1 floor.

---

## 5. Complete Cost Model ($n \le 900{,}000$, Admitted Regime $r/n \in [0.10, 0.15]$)

Here $r \in [90{,}000, 135{,}000]$; $S_{\mathrm{RV}}(m)$ and $S_{\mathrm{MC}}(m)$ denote the exact induction step counts of `Cells` and Max-Cut ($K \le 256$) down to residual size $m$.

| Cost Component | `Scalar-cap` (Run-Record Kernel) | `Cells` (Single-End RV + Stop-and-Walk) | Max-Cut Multi-Boundary RV ($K \le 256$) |
| --- | --- | --- | --- |
| **Construction (sequential)** | $O(r)$ build run-record table | $O(r)$ build $r$ intervals + links | $O(r)$ build $r' \le r + 255$ intervals + 1 KB front tree |
| **Induction unpredictable accesses** | $0$ | $S_{\mathrm{RV}}(m)$ interval pointer updates (prefetched) | $S_{\mathrm{MC}}(m)$ updates (also batchable for MLP) + L1 tree update |
| **Residual walk unpredictable accesses** | $n$ run-record steps (courier-pipelined) | $m$ residual steps | $m$ residual steps |
| **Arithmetic & branching** | Very low (run-offset step) | Moderate (unlink/relink + SLP append) | Moderate + $O(\log K) \le 8$ L1 tournament comparisons/step |
| **Peak retained state (per worker)** | $O(r)$ run records ($\approx 1.0\text{--}1.5$ MB) + $0.9$ MB out | $O(r)$ intervals + live SLP width + $0.9$ MB out | Same as `Cells` $+ \le 2$ KB sentinels/tree (width TBD) |
| **Final output emission** | $n$ byte writes (inline) | $n$ byte writes (SLP/rebuild expansion) | $n$ byte writes (SLP/rebuild expansion) |

**Key structural takeaway:** Because `Scalar-cap` is already a run-record kernel with $O(r)$ working set (fitting comfortably in L2/L3 even at 7–10 workers), `Cells` wins only when $(S_{\mathrm{RV}}(m) + m)$ plus SLP expansion beats $n$ courier-pipelined run-record steps. Max-Cut can only deliver a *fundamental* total-work win if $S_{\mathrm{MC}}(m) \ll S_{\mathrm{RV}}(m)$ without inflating SLP rebuild width.

---

## 6. Self-Contained Python Verifier (`verify_boundary_records.py`)

Run with `python3 verify_boundary_records.py` (Python 3.9+, standard library only). It performs two distinct jobs:

1. **Part 1 (Exhaustive Theorem 1 & Corollary 1 Verification):** Tests Lemma 1, Theorem 1 (both full induction $X = \emptyset$ and intermediate stopped sections $X$, for both $K = 1$ single-end RV and $K > 1$ Multi-Boundary RV under Max-Cut), and Corollary 1 on:
* All binary BWTs of length $n = 2 \dots 8$ ($508$ strings, including all periodic/tied rotations),
* All ternary BWTs of length $n = 2 \dots 5$ ($360$ strings),
* All general dIET permutations of $r = 4, 5$ intervals with varied lengths (multi-cycle and cross-component).


2. **Part 2 (Work-Reduction Test on Heavy-Tailed Admitted-Regime BWTs):** Constructs synthetic repetitive/structured BWTs at $n = 40{,}000$ with $r/n \in [0.10, 0.15]$ (mimicking Wikidata's admitted regime) and measures exact full and stopped step counts for `Cells` ($K = 1$), Max-Cut ($K = \vert{}\Sigma\vert{}$), and the Corollary 1 lower bound $2r - |\overline{D} \cap \overline{C}|$.

```python
#!/usr/bin/env python3
"""
Self-contained verifier for Lemma 1, Theorem 1 (Exact Boundary-Record Theorem),
Corollary 1 (Universal Lower Bound), and Max-Cut vs Single-End RV step counts.
Standard library only. Run with: python3 verify_boundary_records.py
"""
from __future__ import annotations
import random
from dataclasses import dataclass
from itertools import combinations, permutations, product
from typing import Dict, List, Optional, Set, Tuple


@dataclass
class Interval:
    nid: int
    dom_start: int  # original coordinate of left endpoint (for tracking)
    cod_start: int  # original coordinate of codomain left endpoint
    length: int
    dom_prev: int = -1
    dom_next: int = -1
    cod_prev: int = -1
    cod_next: int = -1


def build_bwt_lf(text: str) -> Tuple[List[int], List[int], List[int]]:
    """Return (LF permutation, run_starts D_bar, alphabet C-boundaries in (0, n))."""
    n = len(text)
    rots = sorted((text[i:] + text[:i], i) for i in range(n))
    L = [text[(start - 1) % n] for _, start in rots]
    chars = sorted(set(L))
    C: Dict[str, int] = {}
    acc = 0
    c_bounds: List[int] = []
    for ch in chars:
        if 0 < acc < n:
            c_bounds.append(acc)
        C[ch] = acc
        acc += L.count(ch)
    seen = {ch: 0 for ch in chars}
    LF = [0] * n
    for i, ch in enumerate(L):
        LF[i] = C[ch] + seen[ch]
        seen[ch] += 1
    d_bar = [0] + [i for i in range(1, n) if L[i] != L[i - 1]]
    return LF, d_bar, c_bounds


def compute_arcs(T: List[int], d_bar: List[int]) -> List[List[int]]:
    """Compute persistence arcs A(c) = [c, T(c), ..., T^l(c)] from Lemma 1."""
    d_set = set(d_bar)
    c_bar = sorted(T[d] for d in d_bar)
    assert len(c_bar) == len(d_bar)
    arcs: List[List[int]] = []
    seen: Set[int] = set()
    for c in c_bar:
        arc = [c]
        cur = c
        while cur not in d_set:
            cur = T[cur]
            arc.append(cur)
        for x in arc:
            assert x not in seen, "Arcs not pairwise disjoint!"
            seen.add(x)
        arcs.append(arc)
    return arcs


def predict_from_arcs(
    arcs: List[List[int]],
    elim_step: List[int],  # 1..S for eliminated rows, S+1 for surviving rows in X
    S_plus_1: int,
) -> Tuple[int, int]:
    """Return (predicted_surviving_intervals, predicted_steps) from Theorem 1."""
    surviving = 0
    steps = 0
    for arc in arcs:
        pis = [(elim_step[x], -x) for x in arc]
        max_pi = max(pis)
        i_star = pis.index(max_pi)
        in_X = elim_step[arc[i_star]] == S_plus_1
        if in_X:
            surviving += 1
        else:
            steps += 1  # annihilation step (Case 3a/3b)

        # Prefix maxima strictly before i_star and outside X
        cur_max = (-1, 0)
        for i in range(i_star):
            if pis[i] > cur_max:
                cur_max = pis[i]
                if elim_step[arc[i]] < S_plus_1:
                    steps += 1

        # Suffix maxima strictly after i_star and outside X
        cur_max = (-1, 0)
        for i in range(len(arc) - 1, i_star, -1):
            if pis[i] > cur_max:
                cur_max = pis[i]
                if elim_step[arc[i]] < S_plus_1:
                    steps += 1

    return surviving, steps


def run_mb_rv(
    T: List[int],
    d_bar: List[int],
    cuts: Tuple[int, ...],
    policy: str = "single",   # "single" (rightmost active comp) or "maxcut"
    stop_m: int = 0,
) -> Tuple[int, int, List[int], List[int]]:
    """
    Execute monotone component-suffix RV down to total surviving rows <= stop_m.
    Returns (steps_taken, live_intervals, refined_d_bar, elim_step_per_row).
    """
    n = len(T)
    refined_d = sorted(set(d_bar) | set(cuts))
    all_cuts = (0,) + cuts + (n,)
    K = len(all_cuts) - 1

    # Build sentinel-bracketed linked lists
    nodes: Dict[int, Interval] = {}
    dom_left, dom_right, cod_left, cod_right = [], [], [], []
    sid = -1
    for _ in range(K):
        for arr in (dom_left, dom_right, cod_left, cod_right):
            nodes[sid] = Interval(sid, -1, -1, 0)
            arr.append(sid)
            sid -= 1

    # Intervals
    raw_iv: List[Tuple[int, int, int, int, int]] = []
    for nid, (a, b) in enumerate(zip(refined_d, refined_d[1:] + [n])):
        ca = T[a]
        raw_iv.append((nid, a, b, ca, ca + (b - a)))
        nodes[nid] = Interval(nid, a, ca, b - a)

    live: Set[int] = set(range(len(raw_iv)))
    comp_right_coord: List[int] = []
    for j, (u, v) in enumerate(zip(all_cuts, all_cuts[1:])):
        comp_right_coord.append(v)
        d_ids = [nid for nid, a, b, _, _ in raw_iv if u <= a and b <= v]
        c_ids = [nid for _, nid in sorted((ca, nid) for nid, _, _, ca, cb in raw_iv if u <= ca and cb <= v)]
        for x, y in zip([dom_left[j]] + d_ids, d_ids + [dom_right[j]]):
            nodes[x].dom_next = y
            nodes[y].dom_prev = x
        for x, y in zip([cod_left[j]] + c_ids, c_ids + [cod_right[j]]):
            nodes[x].cod_next = y
            nodes[y].cod_prev = x

    elim_step = [0] * n
    alive_rows = n
    step = 0

    while live and alive_rows > stop_m:
        # Choose active front j
        chosen = -1
        best_delta = -1
        for j in range(K - 1, -1, -1):
            a = nodes[dom_right[j]].dom_prev
            if a == dom_left[j]:
                continue
            b = nodes[cod_right[j]].cod_prev
            delta = min(nodes[a].length, nodes[b].length)
            if policy == "single":
                chosen = j
                break
            elif policy == "maxcut":
                if delta > best_delta:
                    best_delta = delta
                    chosen = j
        assert chosen >= 0
        j = chosen
        a = nodes[dom_right[j]].dom_prev
        b = nodes[cod_right[j]].cod_prev
        na, nb = nodes[a], nodes[b]
        la, lb = na.length, nb.length
        delta = min(la, lb)
        step += 1

        # Record exact eliminated original coordinates in component j
        cut_end = comp_right_coord[j]
        cut_start = cut_end - delta
        for row in range(cut_start, cut_end):
            assert elim_step[row] == 0
            elim_step[row] = step
        comp_right_coord[j] = cut_start
        alive_rows -= delta

        if a == b:
            # Case 3b self-loop
            nodes[na.dom_prev].dom_next = na.dom_next
            nodes[na.dom_next].dom_prev = na.dom_prev
            nodes[na.cod_prev].cod_next = na.cod_next
            nodes[na.cod_next].cod_prev = na.cod_prev
            live.remove(a)
        elif la > lb:
            na.length = la - lb
            bp, bn = nb.cod_prev, nb.cod_next
            nodes[bp].cod_next = bn
            nodes[bn].cod_prev = bp
            an = na.cod_next
            na.cod_next = b
            nb.cod_prev = a
            nb.cod_next = an
            nodes[an].cod_prev = b
        elif la < lb:
            nb.length = lb - la
            ap, an = na.dom_prev, na.dom_next
            nodes[ap].dom_next = an
            nodes[an].dom_prev = ap
            bn = nb.dom_next
            nb.dom_next = a
            na.dom_prev = b
            na.dom_next = bn
            nodes[bn].dom_prev = a
        else:
            ap, an = na.dom_prev, na.dom_next
            nodes[ap].dom_next = an
            nodes[an].dom_prev = ap
            bp, bn = nb.cod_prev, nb.cod_next
            nodes[bp].cod_next = bn
            nodes[bn].cod_prev = bp
            acp, acn = na.cod_prev, na.cod_next
            nodes[acp].cod_next = b
            nb.cod_prev = acp
            nb.cod_next = acn
            nodes[acn].cod_prev = b
            live.remove(a)

    for row in range(n):
        if elim_step[row] == 0:
            elim_step[row] = step + 1
    return step, len(live), refined_d, elim_step


def part1_exhaustive() -> None:
    checked = 0
    # 1a. All binary BWTs n=2..8 and ternary BWTs n=2..5
    for alphabet, lengths in (("ab", range(2, 9)), ("abc", range(2, 6))):
        for n in lengths:
            for tup in product(alphabet, repeat=n):
                text = "".join(tup)
                T, d_bar, c_bounds = build_bwt_lf(text)
                orig_arcs = compute_arcs(T, d_bar)
                lb = 2 * len(d_bar) - len(set(d_bar) & {T[d] for d in d_bar})

                for cuts, policy in [((), "single"), (tuple(c_bounds), "maxcut")]:
                    for stop_m in (0, n // 3, n // 2):
                        steps, live_r, ref_d, elim = run_mb_rv(T, d_bar, cuts, policy, stop_m)
                        arcs = compute_arcs(T, ref_d)
                        pred_r, pred_s = predict_from_arcs(arcs, elim, steps + 1)
                        assert live_r == pred_r, f"live_r mismatch on {text!r}: {live_r} != {pred_r}"
                        assert steps == pred_s, f"steps mismatch on {text!r}: {steps} != {pred_s}"
                        if stop_m == 0:
                            assert steps >= lb, f"Lower bound violated on {text!r}: {steps} < {lb}"
                        checked += 1

    # 1b. General multi-cycle dIETs r=4,5 with varied lengths
    for r in (4, 5):
        for perm in permutations(range(r)):
            for lengths in ([1] * r, [1 + (i % 3) for i in range(r)]):
                n = sum(lengths)
                d_pos, acc = [], 0
                for L in lengths:
                    d_pos.append(acc)
                    acc += L
                c_pos, acc = [0] * r, 0
                for idx in perm:
                    c_pos[idx] = acc
                    acc += lengths[idx]
                T = [0] * n
                for i in range(r):
                    for off in range(lengths[i]):
                        T[d_pos[i] + off] = c_pos[i] + off
                c_internal = tuple(sorted(c_pos[i] for i in range(r) if 0 < c_pos[i] < n))
                lb = 2 * r - len(set(d_pos) & set(c_pos))
                for cuts, policy in [((), "single"), (c_internal[:2], "maxcut")]:
                    for stop_m in (0, n // 2):
                        steps, live_r, ref_d, elim = run_mb_rv(T, d_pos, cuts, policy, stop_m)
                        arcs = compute_arcs(T, ref_d)
                        pred_r, pred_s = predict_from_arcs(arcs, elim, steps + 1)
                        assert live_r == pred_r and steps == pred_s
                        if stop_m == 0:
                            assert steps >= lb
                        checked += 1

    print(f"PART 1 EXHAUSTIVE PASS: {checked} induction runs verified with 0 mismatches.")


def make_structured_block(n: int, seed: int) -> str:
    """Generate a Wikidata-like JSON/template string with r/n in [0.10, 0.15]."""
    rng = random.Random(seed)
    keys = ['"id":"Q', '"type":"item","labels":{"en":"', '"claims":{"P31":[{"id":"Q', '"lang":"en","value":"']
    out: List[str] = []
    tot = 0
    while tot < n:
        k = rng.choice(keys)
        num = str(rng.randint(1000, 999999))
        tail = '"},'
        chunk = k + num + tail
        out.append(chunk)
        tot += len(chunk)
    return "".join(out)[:n]


def part2_admitted_regime() -> None:
    print("\nPART 2: Admitted-regime work comparison (n=20,000, heavy-tailed structured text)")
    for seed in (1, 2, 3):
        text = make_structured_block(20000, seed)
        n = len(text)
        T, d_bar, c_bounds = build_bwt_lf(text)
        r = len(d_bar)
        lb = 2 * r - len(set(d_bar) & {T[d] for d in d_bar})
        s_rv_full, _, ref_d1, elim1 = run_mb_rv(T, d_bar, (), "single", 0)
        s_mc_full, _, ref_d2, elim2 = run_mb_rv(T, d_bar, tuple(c_bounds), "maxcut", 0)
        # Verify Theorem 1 on these large blocks too
        assert predict_from_arcs(compute_arcs(T, ref_d1), elim1, s_rv_full + 1) == (0, s_rv_full)
        assert predict_from_arcs(compute_arcs(T, ref_d2), elim2, s_mc_full + 1) == (0, s_mc_full)

        # Also compare stop-and-walk at m = 0.15 * n
        m_stop = int(0.15 * n)
        s_rv_stop, _, _, _ = run_mb_rv(T, d_bar, (), "single", m_stop)
        s_mc_stop, _, _, _ = run_mb_rv(T, d_bar, tuple(c_bounds), "maxcut", m_stop)
        print(
            f"  seed={seed}: r/n={r/n:.3f} (r={r}) | LB={lb} ({lb/r:.2f}r) | "
            f"full: RV={s_rv_full} ({s_rv_full/r:.2f}r), MaxCut={s_mc_full} ({s_mc_full/r:.2f}r), "
            f"ratio={s_mc_full/s_rv_full:.3f} | "
            f"stop@15%n: RV={s_rv_stop} ({s_rv_stop/r:.2f}r), MaxCut={s_mc_stop} ({s_mc_stop/r:.2f}r), "
            f"ratio={s_mc_stop/s_rv_stop:.3f}"
        )


if __name__ == "__main__":
    part1_exhaustive()
    part2_admitted_regime()

```

### Exact Confirmation and Refutation Criteria

* **Confirms Lemma 1, Theorem 1, and Corollary 1:** `PART 1 EXHAUSTIVE PASS` prints with `0 mismatches` (no assertion failures across all $508$ binary BWTs, $360$ ternary BWTs, all $r \in \{4, 5\}$ dIET permutations, both $K = 1$ and $K > 1$, both full and stopped induction, and the $n = 20{,}000$ blocks). Any assertion failure in Part 1 refutes Theorem 1 and immediately identifies the minimal counterexample.
* **Evaluates the Max-Cut Escape Valve (Part 2):**
* **Refutes Max-Cut as a total-work breakthrough** if `ratio = s_mc_stop / s_rv_stop >= 0.80` (a $< 20\%$ reduction in induction steps cannot overcome the extra front-selection overhead and still leaves total work within the same constant factor of `Cells`).
* **Supports further investigation of Max-Cut** if `ratio <= 0.65` at `stop@15%n` (a $\ge 35\%$ reduction in actual induction steps toward the Corollary 1 floor), in which case the next decisive measurement is whether Max-Cut inflates live SLP induction width on real Wikidata blocks.



---

## 7. Research Judgement

1. **What is established:** The **Exact Boundary-Record Theorem** (Theorem 1) resolves *why* `Cells` pays what it pays: every elementary induction step is a prefix or suffix record along one of $r$ disjoint persistence arcs $A(c)$, and **no** unaccelerated monotone schedule can beat $2r - |\overline{D} \cap \overline{C}|$ steps (Corollary 1).
2. **Where the real bottleneck lives:** In the admitted regime $r/n \in [0.10, 0.15]$, Corollary 1's hard floor ($\approx 1.88\,r$) is already only a factor of $\sim 2\times$ below the oblivious harmonic expectation ($\approx 4\,r$) on full induction—and even closer once stop-and-walk is enabled.
3. **What would change the picture fundamentally:** If Part 2 shows Max-Cut achieves $< 0.65\times$ the induction steps of `Cells` at `stop@15%n`, then adaptive front selection genuinely reduces *total paid work* (not just latency). If instead Max-Cut lands near $0.85\text{--}1.00\times$ due to short-codomain blocking, then **no monotone component-suffix schedule can substantially reduce total induction work over `Cells` in the $r/n \in [0.10, 0.15]$ regime**, and any major breakthrough would have to leave monotone interval induction altogether—specifically by exploiting **multi-step Zorich/cycle collapsing** or **LZ77-style back-reference copying during emission** (cutting the $n$-byte emission cost rather than the $O(r \log(n/r))$ induction cost).
