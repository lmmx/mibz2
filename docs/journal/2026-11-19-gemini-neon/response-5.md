# Gemini 4 Neon: fifth response, verbatim

Received by Louis on 2026-11-19 in reply to the round-5 packet. The packet gave the corrected regime, record 80, the project cost findings and real-block statistics; Neon could not fetch the binary blocks. Filed unedited, except that the embedded verifier is filed separately as [verify_rv_copy.py.txt](verify_rv_copy.py.txt). The review is in [README.md](README.md).

---

# Weight-Only Rauzy–Veech Boundary Sampling and BWT-Native Copy Reconstruction

**Provenance boundary:** All block statistics and performance figures below are **supplied experimental context** from the Coordinator; I have not accessed the `.L` files or executed code. Every mathematical statement is tagged as **Proved**, **Conjecture / Analytical Model**, or **Awaiting Execution** (tested by the self-contained verifier in Section 5).

---

## 1. Direction Chosen and Main Result

With monotone induction scheduling closed (`Cells` is within $\sim 2\%$ of the accelerated winner-run floor), the remaining paid work lives in **SLP/program construction, live rebuild state, and $n$-byte emission**.

In the supplied findings, two observations stand out:

1. **BWT-native copy replay** (Navarro–Ochoa–Prezza Theorem 7: $r$ run-head literals + $k = r - \vert{}\overline{D} \cap \overline{C}\vert{}$ copies; on block 0, $51{,}184$ literals and $10{,}954$ copies) has estimated replay time $0.1\text{--}0.3\text{ ms}$ against $\approx 2.4\text{ ms}$ for an inverse BWT—**but was abandoned** because obtaining the seed bytes in text order appeared to require walking $\mathrm{LF}$ for $n$ steps.
2. **Open question:** *"Can a gap's separation depth — the number of bytes its two rows read together — be found without walking its path?"*

We prove that **both** obstacles fall to the same structural theorem:

* **Theorem 1 (Weight-Only RV Boundary Sampling — *Proved*):** Running accelerated Rauzy–Veech induction with **integer weights only** (no return words, no SLP, no slot recycling, no induction-width buffer) computes, in the exact same $S_{\mathrm{acc}} \approx 1.015\,r$ accelerated steps and $O(r)$ integer memory:
1. the exact cycle ID and LF-orbit position $t(d_k)$ of **all $r$ run-heads** $d_k \in \overline{D}$,
2. the exact cycle ID and LF-orbit position $t(e_k)$ of **all $r$ run-tails** $e_k = d_{k+1} - 1 \in \overline{D} - 1$,
3. the exact LF-orbit position $t(p_0)$ of the primary mark, and
4. the exact separation depth $\ell(c) = \min\{j \ge 0 : \mathrm{LF}^j(c) \in \overline{D}\}$ of **all $r$ boundary gaps** $c \in \overline{C}$.
This answers the project's open question in the affirmative: all $r$ boundary gap separation depths are determined in $O(S_{\mathrm{acc}}) \approx 1.015\,r$ steps without walking $n$ rows and without $O(n \log n)$ pointer jumping.


* **Theorem 2 (Zero-Walk Extraction of the NOP-7 Copy Parse & Depth Hazard — *Proved*):** From $(t(\overline{D}), t(\overline{D}-1))$ alone, one $O(r)$ sequential pass + one $O(r)$ radix sort extracts the **exact** Navarro–Ochoa–Prezza Theorem 7 macro scheme ($r$ literals and $k = r - \vert{}\overline{D} \cap \overline{C}\vert{}$ copies) with zero $O(n)$ LF walking. However, the parse is **bidirectional**, and the byte-level copy depth of row $x$ is **exactly** its run offset $x - d(x)$ (up to $26{,}406$ on block 0).

Whether **`RV-Copy`** beats `Cells` therefore hinges on a single, sharply defined empirical question on the six real blocks: **does multi-pass interval-peeling resolve the $k$ bidirectional NOP-7 copies in $O(k)$ large `memcpy` fragments, or do bidirectional dependencies across long runs shatter them into $\Theta(n)$ tiny fragments?**

---

## 2. Theorem 1: Weight-Only RV Boundary Sampling (*Proved*)

Let $([n], T, \mathcal{R}_0)$ be the finite dIET of a BWT last column $L \in \Sigma^n$ ($T = \mathrm{LF}$, $\vert{}\mathcal{R}_0\vert{} = r$). For each initial interval $I \in \mathcal{R}_0$, let $d_I := \min \mathrm{dom}_0(I) \in \overline{D}$ be its run-head and $e_I := \max \mathrm{dom}_0(I) \in \overline{D} - 1$ be its run-tail. For any row $x$ in a cycle $\mathcal{C}$ of length $d_{\mathcal{C}}$, write $\mathrm{pos}(x) \in \mathbb{Z} / d_{\mathcal{C}}\mathbb{Z}$ for its LF-orbit coordinate ($\mathrm{pos}(T(x)) \equiv \mathrm{pos}(x) + 1 \pmod{d_{\mathcal{C}}}$).

### 2.1 Algorithm (Integer-Only State)

During accelerated single-end Rauzy–Veech induction on $\mathcal{R}$, each live interval $I$ stores only:

* `len[I]` ($\lambda(I) \ge 1$) and `wlen[I]` ($w(I) \ge 1$, initially $1$),
* `h_shift[I]` (initially $0$),
* `t_tok[I]` (initially $I$, or $\bot$ once its right endpoint is no longer an original run-tail) and `t_shift[I]` (initially $0$).

Maintain two $O(r)$ merge-forest arrays `(h_parent, h_delta)` and `(t_parent, t_delta)` (plus a deletion stack for bottom-up/top-down resolution) and the primary-mark state `(ip, theta, p_delta)`.

At each accelerated step on rightmost domain interval $A$ and rightmost codomain interval $B$:

1. **Case 1 ($\lambda(A) > \lambda(B)$, quotient $q \ge 1$, where $q = \lfloor(\lambda(A)-1)/\lambda(B)\rfloor$ if $A = \mathrm{cod\_prev}(B)$ else $1$):**
* *Head tracking:* unchanged (`h(A)` and `h(B)` do not move).
* *Tail tracking (step 1 only):* if `t_tok[A] != ⊥`:
* if `t_tok[B] != ⊥`: record `t_parent[t_tok[A]] = t_tok[B]`, `t_delta[t_tok[A]] = w(B) + t_shift[A] - t_shift[B]`.
* else: set `t_tok[B] = t_tok[A]`, `t_shift[B] = t_shift[A] + w(B)`.
* set `t_tok[A] = ⊥`. (Steps $2 \dots q$ do nothing to tails because `t_tok[A] == ⊥`.)


* *Primary mark:* if `ip == A` and `theta >= la - q * lb`: let $k = (\text{la} - 1 - \theta) // \text{lb} \in \{0, \dots, q-1\}$; set `ip = B`, `theta = theta - (la - (k + 1) * lb)`, `p_delta += w(B) + k * w(A)`.
* *Update:* `len[A] -= q * lb`; `wlen[B] += q * w(A)`; relink $B$ after $A$ in codomain (if $q = 1$ and $A \neq \mathrm{cod\_prev}(B)$).


2. **Case 2 ($\lambda(A) < \lambda(B)$, quotient $q \ge 1$, where $q = \lfloor(\lambda(B)-1)/\lambda(A)\rfloor$ if $B = \mathrm{dom\_prev}(A)$ else $1$):**
* *Head tracking:* `h_shift[A] += q * w(B)`.
* *Tail tracking:*
* Step 1: if `t_tok[A] != ⊥` and `t_tok[B] != ⊥`: record `t_parent[t_tok[A]] = t_tok[B]`, `t_delta[t_tok[A]] = w(B) + t_shift[A] - t_shift[B]`; set `t_tok[A] = t_tok[B]`, `t_shift[A] = t_shift[B]`, `t_tok[B] = ⊥`.
* Else if `t_tok[A] != ⊥`: `t_shift[A] += w(B)`.
* Else if `t_tok[B] != ⊥`: `t_tok[A] = t_tok[B]`, `t_shift[A] = t_shift[B]`, `t_tok[B] = ⊥`.
* Steps $2 \dots q$: if `t_tok[A] != ⊥`: `t_shift[A] += (q - 1) * w(B)`.


* *Primary mark:* if `ip == A`: `p_delta += q * w(B)`. Else if `ip == B` and `theta >= lb - q * la`: let $k = (\text{lb} - 1 - \theta) // \text{la}$; set `ip = A`, `theta = theta - (lb - (k + 1) * la)`, `p_delta += (q - 1 - k) * w(B)`.
* *Update:* `len[B] -= q * la`; `wlen[A] += q * w(B)`; relink $A$ after $B$ in domain (if $q = 1$ and $B \neq \mathrm{dom\_prev}(A)$).


3. **Case 3a ($\lambda(A) = \lambda(B)$, $A \neq B$):**
* *Head tracking:* record `h_parent[A] = B`, `h_delta[A] = w(B) + h_shift[A] - h_shift[B]`.
* *Tail tracking:* identical to Case 1 (step 1).
* *Primary mark:* if `ip == A`: `ip = B`, `p_delta += w(B)`.
* *Update:* `wlen[B] += w(A)`; replace $A$ by $B$ in codomain; unlink and delete $A$.


4. **Case 3b ($A = B$, closed band of $\lambda(A)$ parallel cycles of length $d = w(A)$):**
* Record $A$ as head-root with `cycle_len = w(A)` and `pos[A] = h_shift[A] mod w(A)`.
* If `t_tok[A] != ⊥`: record `t_tok[A]` as tail-root on the same band (and **identical cycle** whenever $\lambda(A) = 1$, i.e., for every primitive BWT!) with `pos[t_tok[A]] = t_shift[A] mod w(A)`.
* If `ip == A`: `pos_primary = p_delta mod w(A)` (on offset `theta` of the band).
* Unlink and delete $A$.



### 2.2 Proof of Theorem 1

We verify the invariants for every live interval $I$ with current domain left endpoint $h(I) = \min \mathrm{dom}(I)$ and right endpoint $r(I) = \max \mathrm{dom}(I)$:


$$(\text{H})\quad \mathrm{pos}(d_I) \equiv \mathrm{pos}(h(I)) + \text{h\_shift}[I], \qquad (\text{T})\quad \text{if } u = \text{t\_tok}[I] \neq \bot,\; \mathrm{pos}(e_u) \equiv \mathrm{pos}(r(I)) + \text{t\_shift}[I].$$

* *Base case:* $h(I) = d_I$, $r(I) = e_I$, `h_shift = t_shift = 0`, `t_tok[I] = I`. Both hold.
* *Case 1:* $\lambda_{\text{new}}(A) = \lambda(A) - q\lambda(B) \ge 1$, so $h(A)$ and $h(B)$ are unchanged, preserving (H). On step 1, $r_{\text{old}}(A) \in Y_1 = \mathrm{cod}(B)$ satisfies $r_{\text{old}}(A) = T_X(r(B)) = T^{w(B)}(r(B))$, so $\mathrm{pos}(r_{\text{old}}(A)) \equiv \mathrm{pos}(r(B)) + w(B)$. Thus:
* if $v = \text{t\_tok}[B] \neq \bot$, $\mathrm{pos}(e_u) - \mathrm{pos}(e_v) \equiv w(B) + \text{t\_shift}[A] - \text{t\_shift}[B]$, an invariant constant independent of future updates on $B$;
* if $\text{t\_tok}[B] = \bot$, moving $u$ onto $B$ with $\text{t\_shift}[B] = \text{t\_shift}[A] + w(B)$ preserves (T).
Setting `t_tok[A] = ⊥` reflects that $r_{\text{new}}(A)$ is an interior point; subsequent accelerated steps $2 \dots q$ therefore leave all tail tokens untouched.


* *Case 2:* $\lambda_{\text{new}}(B) = \lambda(B) - q\lambda(A) \ge 1$, so $h(B)$ is unchanged. In each of the $q$ steps, $h(A)$ moves backward by $T_X^{-1}$ inside $\mathrm{dom}(B)$ (whose return time $w(B)$ stays constant across all $q$ steps), so $\mathrm{pos}(h_{\text{old}}(A)) \equiv \mathrm{pos}(h_{\text{new}}(A)) + q \cdot w(B)$, preserving (H). For tails: on step 1, $r_{\text{old}}(A)$ moves backward by $w(B)$ to $r_{\text{old}}(B) = r_{\text{new}}(A)$, while $r_{\text{old}}(B)$ itself becomes $r_{\text{new}}(A)$ with $0$ shift; hence `t_tok[A]` and `t_tok[B]` meet at $r_{\text{new}}(A)$ with difference $w(B) + \text{t\_shift}[A] - \text{t\_shift}[B]$, and `t_tok[B]` becomes $\bot$. Across steps $2 \dots q$, the surviving `t_tok[A]` moves backward by $w(B)$ per step, adding $(q-1)w(B)$.
* *Case 3a:* $h(A) = T^{w(B)}(h(B))$ and $r(A) = T^{w(B)}(r(B))$, so both merges record exact invariant differences $\mathrm{pos}(d_A) - \mathrm{pos}(d_B)$ and $\mathrm{pos}(e_u) - \mathrm{pos}(e_v)$.
* *Case 3b:* Setting $\mathrm{pos}(h(A)) \equiv 0 \pmod{w(A)}$ fixes $\mathrm{pos}(d_A) \equiv \text{h\_shift}[A] \pmod{w(A)}$. When $\lambda(A) = 1$, $r(A) = h(A)$, so $\mathrm{pos}(e_u) \equiv \text{t\_shift}[A] \pmod{w(A)}$ is anchored in the **exact same** coordinate frame. (When $\lambda(A) > 1$ on a periodic BWT $S = u^q$ with $q = \lambda(A)$, all $q$ cycles of the band spell the identical cyclic word $u$ of length $d = w(A)$ in phase, so $\mathrm{pos}(r(A)) \equiv \mathrm{pos}(h(A)) \equiv 0 \pmod{d}$ remains exact mod $d$.)
* *Gap separation depths:* Once `pos_head[0..r-1]` are resolved in one $O(r)$ reverse-deletion pass and sorted (per cycle) into $\tau_0 < \tau_1 < \dots < \tau_{r_c - 1}$, the gap $c = \mathrm{LF}(d_k)$ at $\tau_i = \mathrm{pos}(d_k)$ has next run-head at $\tau_{i+1}$ (cyclically mod $d_c$), so its exact persistence length is

$$\ell(c) = (\tau_{i+1} - \tau_i - 1) \bmod d_c.$$



Every run-boundary gap separation depth is thus obtained in $O(S_{\mathrm{acc}} + r)$ total work. $\square$

---

## 3. Theorem 2: Zero-Walk Extraction of NOP-7 Copies and the Depth Hazard (*Proved*)

Let $B[t] := L[\mathrm{LF}^t(p_0)]$ for $t \in [0, d)$ (for a primitive BWT, $d = n$ and the text is $S[j] = B[n - 1 - j]$; for periodic $S = u^q$, $d = n/q$ and $S_{\text{full}} = u^q$). Let $o_0, o_1, \dots, o_{r-1}$ be the $r$ runs ordered by codomain start $c_{o_j} = \mathrm{LF}(d_{o_j})$ (obtained in $O(n)$ sequential bucket-append during the initial scan of $L$, with **no** sort).

**Theorem 2 (*Proved*).**
*Assume $\mathrm{LF}$ is a single $n$-cycle (or restrict to a periodic block's primitive cycle of length $d = n/q$). Rotate coordinates so $\tau_0 = 0$ (the first run-head in sorted $\mathrm{pos\_head}$ order). Then:*

1. *Every position $\tau_i = \mathrm{pos\_head}[k]$ ($0 \le i < r$) is a literal $B[\tau_i] = L[d_k]$.*
2. *For every $i \in \{0, \dots, r-1\}$ with $m_i := \tau_{i+1} - \tau_i - 1 > 0$ (taking $\tau_r = d$), the codomain run-head $c = \mathrm{LF}(d_k)$ satisfies $c > 0$ (since $0 \in \overline{D} \cap \overline{C}$ has $m_i = 0$). Letting $o_{j-1}$ be the run immediately preceding $k = o_j$ in codomain order and $s_i := (\mathrm{pos\_tail}[o_{j-1}] + 1) \bmod d$, we have*

$$B[\tau_i + 1 + u] = B[(s_i + u) \bmod d] \qquad \text{for all } 0 \le u < m_i.$$



*The number of nonempty copies is **exactly** $k = r - |\overline{D} \cap \overline{C}|$.*
3. *(Exact Byte-Depth Identity)* *In the resulting bidirectional macro scheme, the copy-parent of $t(x)$ for every $x \notin \overline{D}$ is $t(x - 1)$. Consequently, the macro scheme is acyclic and the exact byte-level dependency depth of position $t(x)$ is its offset $x - d(x)$ within its BWT run, with maximum depth $\max_k (\lambda_k - 1)$.*

**Proof.**

1. Immediate from $B[\mathrm{pos}(d_k)] = L[d_k]$.
2. By definition of consecutive sorted run-heads $\tau_i, \tau_{i+1}$, the rows $c, \mathrm{LF}(c), \dots, \mathrm{LF}^{m_i - 1}(c)$ all lie outside $\overline{D}$, while $\mathrm{LF}^{m_i}(c) \in \overline{D}$. Since $0 \in \overline{D} \cap \overline{C}$, $m_i > 0$ implies $c \notin \overline{D}$, hence $c > 0$. The row $c - 1$ is the codomain right endpoint of run $o_{j-1}$, so $c - 1 = \mathrm{LF}(e_{o_{j-1}})$ and $t(c - 1) = (\mathrm{pos\_tail}[o_{j-1}] + 1) \bmod d = s_i$. By induction on $u = 0, \dots, m_i - 1$, $\mathrm{LF}^u(c) \notin \overline{D}$ implies $\mathrm{LF}^u(c - 1) = \mathrm{LF}^u(c) - 1$ lies in the same run of $L$, so $B[\tau_i + 1 + u] = L[\mathrm{LF}^u(c)] = L[\mathrm{LF}^u(c - 1)] = B[(s_i + u) \bmod d]$. Finally, $m_i > 0 \iff c \notin \overline{D}$, so the count of nonempty copies is $|\overline{C} \setminus \overline{D}| = r - |\overline{D} \cap \overline{C}|$.
3. For any $x \notin \overline{D}$, $t(x) = \tau_i + 1 + u$ with $x = \mathrm{LF}^u(c)$, and its copy-source is $(s_i + u) \bmod d = t(\mathrm{LF}^u(c - 1)) = t(x - 1)$. Iterating $t(x) \to t(x - 1) \to \dots \to t(d(x))$ terminates in exactly $x - d(x)$ steps at the literal $t(d(x))$. $\square$

---

## 4. Complete Cost Model (`RV-Copy` vs. `Cells`, $n \le 900{,}000$, $r/n \in [0.057, 0.143]$)

Let $k = r - |\overline{D} \cap \overline{C}|$ ($\approx 0.21\,r\text{--}0.25\,r$ on the supplied Wikidata blocks), and let $F$ be the total number of contiguous `memcpy` fragments executed during multi-pass interval-peeling of the $k$ NOP-7 copies.

| Cost Component | `Cells` / `LazyCells` | `RV-Copy` (Weight-Only RV + NOP-7 Replay) |
| --- | --- | --- |
| **Initial scan (sequential)** | $O(n)$ build $r$ runs + codomain buckets | Identical $O(n)$ scan + codomain order $o_0 \dots o_{r-1}$ |
| **Induction unpredictable accesses** | $\approx 1.015\,r$ accelerated RV steps (prefetched) | **Exact same** $\approx 1.015\,r$ accelerated RV steps (no SLP/word writes) |
| **Intermediate construction** | Build SLP / return-word program + slot recycling | **Zero SLP.** $O(r)$ reverse-scan of 2 integer trees + $O(r)$ radix sort of $r$ pairs |
| **Emission / output of $n$ bytes** | Traverse SLP / live-width rebuild $\to n$ writes | $r$ literal writes + $F$ contiguous `memcpy`s totaling $n - r$ bytes |
| **Peak retained state (per worker)** | $O(r)$ intervals + live SLP/rebuild width + $0.9\text{ MB}$ out | $O(r)$ flat integer arrays ($\approx 36\text{ B}/r \approx 1.8\text{--}4.6\text{ MB}$) + $0.9\text{ MB}$ out (plus $\le 0.11\text{ MB}$ bitset for `filled`) |

**The decisive trade-off:** `RV-Copy` pays the **exact same** $1.015\,r$ induction steps as `Cells` (lighter per step, with zero SLP), plus a tiny $O(r)$ radix sort, and replaces `Cells`'s entire SLP rebuild/emission with $F$ contiguous `memcpy`s.

* **If $F \le 3k$** (say $30{,}000\text{--}80{,}000$ `memcpy`s of mean length $10\text{--}30$ bytes), `RV-Copy` eliminates SLP construction and rebuild width completely.
* **If $F \gg r$** (e.g., $F > 200{,}000$, mean `memcpy` length $< 4$ bytes) due to bidirectional ping-ponging across the $\sim 26{,}000$-length runs, then Theorem 2.3 acts as a **sharp obstruction** to BWT-native copy replay without a unidirectional (text-order) re-parse.

---

## 5. Self-Contained Python Verifier (`verify_rv_copy.py`)

Standard library only, $O(n)$ memory (flat `array('I')` / `bytearray`), practical for $n \le 900{,}000$ (uses C-speed `bytearray.find` for interval peeling).

Run on each authenticated block:

```bash
python3 verify_rv_copy.py path/to/block-0.L 264045

```

```python
#!/usr/bin/env python3
"""
Verifier for Theorem 1 (Weight-Only RV Boundary Sampling) and Theorem 2
(Zero-Walk NOP-7 Copy Extraction + Multi-Pass Interval-Peeling Replay).

Usage: python3 verify_rv_copy.py <path-to-.L> <primary_index>
Standard library only; O(n) flat memory; practical for n <= 900,000.
"""
from __future__ import annotations
import sys
import time
from array import array
from typing import List, Tuple


def build_runs_and_links(L: bytes):
    """O(n) sequential scan: extract r maximal runs, codomain order, and RV linked lists."""
    n = len(L)
    counts = [0] * 256
    for b in L:
        counts[b] += 1
    C = [0] * 256
    acc = 0
    for ch in range(256):
        C[ch] = acc
        acc += counts[ch]

    # Maximal runs of L
    run_start = array("I")
    run_len = array("I")
    run_sym = bytearray()
    buckets: List[List[int]] = [[] for _ in range(256)]

    i = 0
    rid = 0
    while i < n:
        ch = L[i]
        j = i + 1
        while j < n and L[j] == ch:
            j += 1
        run_start.append(i)
        run_len.append(j - i)
        run_sym.append(ch)
        buckets[ch].append(rid)
        rid += 1
        i = j

    r = rid
    # Codomain order of runs (no sort needed: concatenate character buckets)
    cod_order = array("I")
    for ch in range(256):
        if buckets[ch]:
            cod_order.extend(buckets[ch])

    # Build doubly-linked lists with sentinels:
    # Node IDs 0..r-1 are runs; r=dom_left, r+1=dom_right, r+2=cod_left, r+3=cod_right
    DL, DR, CL, CR = r, r + 1, r + 2, r + 3
    dom_prev = array("I", [0]) * (r + 4)
    dom_next = array("I", [0]) * (r + 4)
    cod_prev = array("I", [0]) * (r + 4)
    cod_next = array("I", [0]) * (r + 4)

    dom_next[DL] = 0
    dom_prev[0] = DL
    for k in range(r - 1):
        dom_next[k] = k + 1
        dom_prev[k + 1] = k
    dom_next[r - 1] = DR
    dom_prev[DR] = r - 1

    first_c = cod_order[0]
    cod_next[CL] = first_c
    cod_prev[first_c] = CL
    for idx in range(r - 1):
        u, v = cod_order[idx], cod_order[idx + 1]
        cod_next[u] = v
        cod_prev[v] = u
    last_c = cod_order[r - 1]
    cod_next[last_c] = CR
    cod_prev[CR] = last_c

    return r, C, run_start, run_len, run_sym, cod_order, dom_prev, dom_next, cod_prev, cod_next


def run_weight_only_rv(
    r: int,
    run_start: array,
    run_len: array,
    dom_prev: array,
    dom_next: array,
    cod_prev: array,
    cod_next: array,
    primary: int,
):
    """
    Accelerated single-end Weight-Only RV (Theorem 1).
    Computes pos_head[0..r-1], pos_tail[0..r-1], and pos_primary with zero SLP/words.
    """
    DL, DR, CL, CR = r, r + 1, r + 2, r + 3
    NONE = 0xFFFFFFFF

    # Locate primary in initial runs (binary search on run_start)
    lo, hi = 0, r - 1
    while lo < hi:
        mid = (lo + hi + 1) >> 1
        if run_start[mid] <= primary:
            lo = mid
        else:
            hi = mid - 1
    ip = lo
    theta = primary - run_start[ip]
    p_delta = 0

    iv_len = array("I", run_len)
    wlen = array("I", [1]) * r
    h_shift = array("q", [0]) * r
    h_parent = array("I", range(r))
    h_delta = array("q", [0]) * r
    h_del_order = array("I")

    t_tok = array("I", range(r))
    t_shift = array("q", [0]) * r
    t_parent = array("I", range(r))
    t_delta = array("q", [0]) * r
    t_del_order = array("I")

    cycles: List[Tuple[int, int, int]] = []  # (root_head, band_width, cycle_len)
    tail_roots: List[Tuple[int, int, int]] = []  # (tok, shift, cycle_len)
    pos_primary = -1
    primary_cycle_len = -1
    acc_steps = 0
    unacc_steps = 0

    while dom_prev[DR] != DL:
        a = dom_prev[DR]
        b = cod_prev[CR]
        la = iv_len[a]
        lb = iv_len[b]
        acc_steps += 1

        if a == b:
            # Case 3b: closed band of `la` parallel cycles of length wlen[a]
            unacc_steps += 1
            cyc_len = wlen[a]
            cycles.append((a, la, cyc_len))
            if t_tok[a] != NONE:
                tail_roots.append((t_tok[a], t_shift[a], cyc_len))
            if ip == a:
                pos_primary = p_delta % cyc_len
                primary_cycle_len = cyc_len
            # Unlink a from both lists
            ap, an = dom_prev[a], dom_next[a]
            dom_next[ap] = an
            dom_prev[an] = ap
            cp, cn = cod_prev[a], cod_next[a]
            cod_next[cp] = cn
            cod_prev[cn] = cp

        elif la > lb:
            # Case 1 (Zorich-accelerated iff cod_prev[b] == a)
            can_accel = cod_prev[b] == a
            q = (la - 1) // lb if can_accel else 1
            unacc_steps += q
            wa = wlen[a]
            wb = wlen[b]

            # Tail tracking (step 1 only)
            ta = t_tok[a]
            if ta != NONE:
                tb = t_tok[b]
                if tb != NONE:
                    t_parent[ta] = tb
                    t_delta[ta] = wb + t_shift[a] - t_shift[b]
                    t_del_order.append(ta)
                else:
                    t_tok[b] = ta
                    t_shift[b] = t_shift[a] + wb
                t_tok[a] = NONE

            # Primary mark
            new_la = la - q * lb
            if ip == a and theta >= new_la:
                k = (la - 1 - theta) // lb
                ip = b
                theta = theta - (la - (k + 1) * lb)
                p_delta += wb + k * wa

            iv_len[a] = new_la
            wlen[b] = wb + q * wa
            if not can_accel:
                # Unlink b from codomain, splice immediately after a
                bp, bn = cod_prev[b], cod_next[b]
                cod_next[bp] = bn
                cod_prev[bn] = bp
                an = cod_next[a]
                cod_next[a] = b
                cod_prev[b] = a
                cod_next[b] = an
                cod_prev[an] = b

        elif la < lb:
            # Case 2 (Zorich-accelerated iff dom_prev[a] == b)
            can_accel = dom_prev[a] == b
            q = (lb - 1) // la if can_accel else 1
            unacc_steps += q
            wb = wlen[b]

            # Head tracking
            h_shift[a] += q * wb

            # Tail tracking (step 1 + remaining q-1 steps)
            ta = t_tok[a]
            tb = t_tok[b]
            if ta != NONE and tb != NONE:
                t_parent[ta] = tb
                t_delta[ta] = wb + t_shift[a] - t_shift[b]
                t_del_order.append(ta)
                t_tok[a] = tb
                t_shift[a] = t_shift[b]
                t_tok[b] = NONE
            elif ta != NONE:
                t_shift[a] += wb
            elif tb != NONE:
                t_tok[a] = tb
                t_shift[a] = t_shift[b]
                t_tok[b] = NONE
            if q > 1 and t_tok[a] != NONE:
                t_shift[a] += (q - 1) * wb

            # Primary mark
            new_lb = lb - q * la
            if ip == a:
                p_delta += q * wb
            elif ip == b and theta >= new_lb:
                k = (lb - 1 - theta) // la
                ip = a
                theta = theta - (lb - (k + 1) * la)
                p_delta += (q - 1 - k) * wb

            iv_len[b] = new_lb
            wlen[a] += q * wb
            if not can_accel:
                # Unlink a from domain, splice immediately after b
                ap, an = dom_prev[a], dom_next[a]
                dom_next[ap] = an
                dom_prev[an] = ap
                bn = dom_next[b]
                dom_next[b] = a
                dom_prev[a] = b
                dom_next[a] = bn
                dom_prev[bn] = a

        else:
            # Case 3a: la == lb, a != b
            unacc_steps += 1
            wb = wlen[b]
            h_parent[a] = b
            h_delta[a] = wb + h_shift[a] - h_shift[b]
            h_del_order.append(a)

            ta = t_tok[a]
            if ta != NONE:
                tb = t_tok[b]
                if tb != NONE:
                    t_parent[ta] = tb
                    t_delta[ta] = wb + t_shift[a] - t_shift[b]
                    t_del_order.append(ta)
                else:
                    t_tok[b] = ta
                    t_shift[b] = t_shift[a] + wb
                t_tok[a] = NONE

            if ip == a:
                ip = b
                p_delta += wb

            wlen[b] = wb + wlen[a]
            # Unlink a from domain
            ap, an = dom_prev[a], dom_next[a]
            dom_next[ap] = an
            dom_prev[an] = ap
            # Unlink b from codomain FIRST (handles a == cod_prev[b])
            bp, bn = cod_prev[b], cod_next[b]
            cod_next[bp] = bn
            cod_prev[bn] = bp
            # Replace a by b in codomain
            acp, acn = cod_prev[a], cod_next[a]
            cod_next[acp] = b
            cod_prev[b] = acp
            cod_next[b] = acn
            cod_prev[acn] = b

    # Resolve head and tail positions in reverse deletion order (pure O(r) sequential)
    d = primary_cycle_len
    pos_head = array("I", [0]) * r
    for root, _, cyc_len in cycles:
        pos_head[root] = h_shift[root] % cyc_len
    for node in reversed(h_del_order):
        p = h_parent[node]
        pos_head[node] = (pos_head[p] + h_delta[node]) % d

    pos_tail = array("I", [0]) * r
    for tok, sh, cyc_len in tail_roots:
        pos_tail[tok] = sh % cyc_len
    for node in reversed(t_del_order):
        p = t_parent[node]
        pos_tail[node] = (pos_tail[p] + t_delta[node]) % d

    return acc_steps, unacc_steps, cycles, pos_head, pos_tail, pos_primary


def replay_nop7_copies(
    d: int,
    r: int,
    run_sym: bytearray,
    cod_order: array,
    pos_head: array,
    pos_tail: array,
):
    """
    Extract NOP-7 parse purely from Weight-Only RV output (Theorem 2) and
    decode B[0..d-1] via multi-pass C-speed interval peeling.
    """
    # Map run_id -> predecessor run_id in codomain order
    cod_pred = array("I", [0]) * r
    for idx in range(1, r):
        cod_pred[cod_order[idx]] = cod_order[idx - 1]

    # Sort the r run-heads by pos_head
    head_order = sorted(range(r), key=pos_head.__getitem__)
    tau0 = pos_head[head_order[0]]

    # Rotate so tau0 == 0 (no destination interval ever wraps mod d)
    B = bytearray(d)
    filled = bytearray(d)
    copies: List[Tuple[int, int, int]] = []  # (dst, src, length)
    unidir_copies = 0
    unidir_bytes = 0

    for i in range(r):
        rid = head_order[i]
        tau = pos_head[rid] - tau0
        B[tau] = run_sym[rid]
        filled[tau] = 1
        nxt_tau = (pos_head[head_order[i + 1]] - tau0) if i + 1 < r else d
        m = nxt_tau - tau - 1
        if m > 0:
            pred_rid = cod_pred[rid]
            src = (pos_tail[pred_rid] + 1 - tau0) % d
            dst = tau + 1
            if src + m <= d:
                copies.append((dst, src, m))
                if src < dst:
                    unidir_copies += 1
                    unidir_bytes += m
            else:
                # Split the at-most-1 copy whose source wraps around d
                first = d - src
                copies.append((dst, src, first))
                copies.append((dst + first, 0, m - first))
                if src < dst:
                    unidir_copies += 1
                    unidir_bytes += first
                unidir_copies += 1
                unidir_bytes += m - first

    # Multi-pass interval peeling (alternating sweep direction for bidirectional cascades)
    active = copies
    passes = 0
    memcpy_frags = 0
    bytes_copied = 0
    self_overlap_frags = 0

    while active:
        passes += 1
        next_active: List[Tuple[int, int, int]] = []
        filled_before = bytes_copied
        it = active if (passes & 1) else reversed(active)

        for dst, src, length in it:
            # Fast path 1: entire source already filled
            first_zero = filled.find(0, src, src + length)
            if first_zero == -1:
                B[dst : dst + length] = B[src : src + length]
                filled[dst : dst + length] = b"\x01" * length
                memcpy_frags += 1
                bytes_copied += length
                continue

            # Fast path 2: left-to-right self-overlap with filled seed [src, dst)
            if src < dst < src + length and first_zero >= dst:
                period = dst - src
                # Periodic doubling copy
                cur = dst
                end = dst + length
                while cur < end:
                    chunk = min(cur - src, end - cur)
                    B[cur : cur + chunk] = B[src : src + chunk]
                    filled[cur : cur + chunk] = b"\x01" * chunk
                    cur += chunk
                memcpy_frags += 1
                self_overlap_frags += 1
                bytes_copied += length
                continue

            # General case: split on filled runs inside [src, src + length)
            pos = src
            end = src + length
            while pos < end:
                one = filled.find(1, pos, end)
                if one == -1:
                    next_active.append((dst + (pos - src), pos, end - pos))
                    break
                if one > pos:
                    next_active.append((dst + (pos - src), pos, one - pos))
                zero = filled.find(0, one, end)
                if zero == -1:
                    zero = end
                seg = zero - one
                d_off = dst + (one - src)
                B[d_off : d_off + seg] = B[one:zero]
                filled[d_off : d_off + seg] = b"\x01" * seg
                memcpy_frags += 1
                bytes_copied += seg
                pos = zero

        assert bytes_copied > filled_before, f"Deadlock on pass {passes}!"
        active = next_active

    return B, tau0, len(copies), unidir_copies, unidir_bytes, passes, memcpy_frags, self_overlap_frags


def ground_truth_check(L: bytes, C: List[int], primary: int, r: int, run_start: array, run_len: array,
                       pos_head: array, pos_tail: array, pos_primary: int, B_replay: bytearray, tau0: int):
    """Independent O(n) LF walk to verify Theorem 1 and Theorem 2 bit-for-bit."""
    n = len(L)
    # Build dense LF once ONLY for independent ground-truth verification
    occ = list(C)
    LF = array("I", [0]) * n
    for i, ch in enumerate(L):
        LF[i] = occ[ch]
        occ[ch] += 1

    # Walk LF from primary
    B_true = bytearray(n)
    row_to_pos = array("I", [0xFFFFFFFF]) * n
    cur = primary
    for t in range(n):
        row_to_pos[cur] = t
        B_true[t] = L[cur]
        cur = LF[cur]
        if cur == primary:
            d = t + 1
            break
    else:
        raise AssertionError("Primary did not return within n steps")

    # Verify every run-head and run-tail position mod d (for single-cycle d == n)
    if d == n:
        shift = (row_to_pos[run_start[0]] - pos_head[0]) % d
        assert (0 - pos_primary) % d == shift, "Primary position mismatch!"
        for k in range(r):
            h_row = run_start[k]
            t_row = h_row + run_len[k] - 1
            assert (pos_head[k] + shift) % d == row_to_pos[h_row], f"Head mismatch at run {k}"
            assert (pos_tail[k] + shift) % d == row_to_pos[t_row], f"Tail mismatch at run {k}"

        # Verify replayed B matches B_true rotated by (tau0 - pos_primary) mod d
        rot = (pos_primary - tau0) % d
        B_rot = B_replay[rot:] + B_replay[:rot]
        assert B_rot == B_true, "Replayed bytes do NOT match true LF orbit!"
    return d


def main() -> None:
    if len(sys.argv) != 3:
        print(f"Usage: {sys.argv[0]} <block.L> <primary_index>")
        sys.exit(1)
    path = sys.argv[1]
    primary = int(sys.argv[2])
    with open(path, "rb") as f:
        L = f.read()
    n = len(L)

    t0 = time.perf_counter()
    r, C, run_start, run_len, run_sym, cod_order, dp, dn, cp, cn = build_runs_and_links(L)
    t1 = time.perf_counter()
    acc, unacc, cycles, pos_head, pos_tail, pos_primary = run_weight_only_rv(
        r, run_start, run_len, dp, dn, cp, cn, primary
    )
    t2 = time.perf_counter()

    print(f"File: {path} (n={n}, r={r}, r/n={r/n:.4f}, primary={primary})")
    print(f"  Cycles found: {len(cycles)} -> {cycles[:5]}")
    print(f"  Weight-Only RV: acc_steps={acc} ({acc/r:.3f}r), unacc_steps={unacc} ({unacc/r:.3f}r) "
          f"[scan={t1-t0:.3f}s, rv={t2-t1:.3f}s]")

    if len(cycles) == 1 and cycles[0][1] == 1:
        d = cycles[0][2]
        t3 = time.perf_counter()
        B_rep, tau0, k_copies, u_copies, u_bytes, passes, frags, ov_frags = replay_nop7_copies(
            d, r, run_sym, cod_order, pos_head, pos_tail
        )
        t4 = time.perf_counter()
        d_true = ground_truth_check(
            L, C, primary, r, run_start, run_len, pos_head, pos_tail, pos_primary, B_rep, tau0
        )
        mean_frag = (n - r) / frags if frags else 0.0
        print(f"  NOP-7 Parse: copies={k_copies} (unidir={u_copies}, {100*u_bytes/(n-r):.1f}% of copy bytes)")
        print(f"  Interval Peeling: passes={passes}, memcpy_frags={frags} ({frags/k_copies:.2f}x copies, "
              f"{frags/r:.2f}r), mean_frag={mean_frag:.1f} B, self_overlap={ov_frags} [replay={t4-t3:.3f}s]")
        print(f"  GROUND-TRUTH VERIFICATION: PASS (d={d_true}, all {r} heads, {r} tails, primary, and {n} bytes exact)")
    else:
        print("  Multi-cycle / periodic block detected; skipping single-cycle replay.")


if __name__ == "__main__":
    main()

```

### Exact Confirmation and Refutation Criteria

1. **Confirms Theorems 1 & 2:** `GROUND-TRUTH VERIFICATION: PASS` prints on all six authenticated blocks (every run-head, every run-tail, primary mark, and all $n$ reconstructed bytes match the independent LF walk bit-for-bit, with `acc_steps` $\approx 1.013\text{--}1.019\,r$). Any assertion failure refutes Theorem 1 or Theorem 2 immediately.
2. **Evaluates `RV-Copy` vs. `Cells` on emission work:**
* **Strong positive result (viable new reconstruction path):** `memcpy_frags / k_copies <= 3.0` and `mean_frag >= 15.0 B` across the six blocks (e.g., $\le 35{,}000$ contiguous `memcpy`s on block 0, zero SLP, zero rebuild width).
* **Sharp negative result (bidirectional fragmentation obstruction):** `memcpy_frags / r >= 3.0` or `mean_frag < 5.0 B` (or `passes > 50`), proving that although Weight-Only RV extracts the NOP-7 parse for free, Theorem 2.3's run-offset depth shatters bidirectional interval replay into byte-scale fragments on real Wikidata blocks.



---

## 6. Research Judgement

1. **What is proved unconditionally:**
* The project's open question on gap separation depth has a **positive** answer: Weight-Only RV computes all $r$ boundary gap separation depths $\ell(c)$, all $r$ run-head positions $\mathrm{pos}(\overline{D})$, and all $r$ run-tail positions $\mathrm{pos}(\overline{D}-1)$ in $S_{\mathrm{acc}} \approx 1.015\,r$ accelerated steps without walking $n$ rows and without $O(n \log n)$ preprocessing.
* This removes the previously assumed barrier to BWT-native copy reconstruction (obtaining seed bytes in text order without walking $\mathrm{LF}$).


2. **What awaits the Coordinator's run on the 6 blocks:**
* Whether `RV-Copy` is a practical breakthrough or hits a bidirectional fragmentation wall depends entirely on `(passes, memcpy_frags, mean_frag)` in `verify_rv_copy.py`.
* If fragmentation is low, `RV-Copy` eliminates SLP construction, slot recycling, and induction-width memory entirely, leaving only $1.015\,r$ integer RV steps + $O(r)$ radix sort + $O(k)$ `memcpy`s.
* If fragmentation is high, the remaining path is a **hybrid**: use Weight-Only RV's $r$ sorted run-heads to emit a *unidirectional* (or depth-bounded) copy parse, or accept that `Cells`'s SLP emission is already near-optimal because the SLP naturally factors the exact long-run self-dependencies that fragment bidirectional copies.
