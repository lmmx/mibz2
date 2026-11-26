# Source-Checked Literature-to-Representation Note: Permutation Patterns for Inverse BWT

*26 November 2026 — smooth-zone primary-source check under claim [`permutation-patterns`](../../../../frontier/claims/permutation-patterns.toml), complementing the verbatim lead transcript [`chatgpt-response.md`](chatgpt-response.md) (preserved unedited).*

## Verification provenance

All bibliographic details and theorem statements below were verified directly against primary DROPS/LIPIcs landing pages, arXiv abstracts/HTML/extracts, and published paper excerpts on 26 November 2026. Where a full PDF proof was not parsed end-to-end (specifically the internal lemmas of Kozma–Opler and Barbay–Navarro's full PDF beyond verified abstracts/theorem extracts), only directly verified statements are asserted.

---

## 1. Primary sources and exact theorem statements

### 1.1 Kozma & Opler (ESA 2026 / arXiv:2510.20382)

- **Citation:** László Kozma and Michal Opler, *"Compact Representations of Pattern-Avoiding Permutations"*, 34th Annual European Symposium on Algorithms (**ESA 2026**), LIPIcs vol. 388, pp. 92:1–92:18, published 25 August 2026, DOI [`10.4230/LIPIcs.ESA.2026.92`](https://drops.dagstuhl.de/entities/document/10.4230/LIPIcs.ESA.2026.92); full version [`arXiv:2510.20382v1`](https://arxiv.org/abs/2510.20382) (23 October 2025).
- **Verified results:**
  - **Theorem 1 (arXiv Theorem 1.1):** Any permutation $\tau$ of size $n$ avoiding an arbitrary (and a priori unknown) fixed pattern $\pi$ can be represented in $O(n \lg s_\pi)$ bits (where $s_\pi$ is the Stanley–Wilf limit of $\pi$), supporting $\tau(i)$ and $\tau^{-1}(i)$ queries in $O(1)$ time, and constructed in $O(n \lg s_\pi)$ time. (The paper notes an inherent factor-2 space overhead over $n \lg s_\pi$ because both row- and column-permutation structures are stored across a 2-level balanced matrix division $(R_1, C_1)$ coarsening $(R_2, C_2)$.)
  - **Theorem 2 (arXiv Theorem 1.2):** For permutations of bounded treewidth $t \in O(1)$ avoiding $\pi$ (which includes separable permutations and bounded contiguous-run classes), space tightens to succinct $n \lg s_\pi + o(n)$ bits with $O(1)$ $\tau(i)$ and $\tau^{-1}(i)$ queries (independent of $t$ and $|\pi|$) and $O_t(n)$ build time.
  - **Theorem 3:** Rectangle range minimum and rectangle range counting in $O(\lg \lg n)$ time using $O(n \lg s_\pi)$ bits and $O(n \lg s_\pi)$ build time.
  - **Corollary 4 (arXiv Corollary 1.4):** Any permutation of twin-width $d \in O(1)$ is representable in $O_d(n)$ bits with $O(1)$ forward/inverse queries and $O_d(n)$ build time.
- **Explicitly stated open questions (directly verified in arXiv:2510.20382 §Open questions):**
  1. Making Theorem 1 succinct (closing the factor-2 space overhead) remains open.
  2. **Iterated / power queries $\tau^k(i)$ within similar bounds are explicitly left open.**
- **Mapping to `mibz2`:**
  - **Constant-factor trap vs flat `edges[j]`:** Answering a single $\tau(i)$ query in Kozma–Opler's 2-level matrix division calls `colPerm`, `parentCell(j, c)`, `rowRank(j, c)`, variable-length child pointers, and rank/select lookups across levels $C_2 \to C_1 \to \text{root} \to R_1 \to R_2$. By contrast, Open's `edges[j]` (`src/open.rs`) packs **both** `next` (18 bits) and `code` (14 bits) into a single 32-bit word (`4 B/vertex`), yielding both navigation and label index in **one** L2/L3 load.
  - **Stanley–Wilf limit on raw LF vs retained $T$:** Raw LF over alphabet $\sigma$ avoids $D_{\sigma+1}$ (decreasing pattern of length $\sigma+1$), whose Stanley–Wilf limit is $s_{D_{\sigma+1}} = \sigma^2$. On real Wikidata blocks ($\sigma \in [48, 256]$), $\lg s_{D_{\sigma+1}} = 2 \lg \sigma \approx 11\text{--}16$ bits (with Kozma–Opler's $2\times$ factor: $22\text{--}32$ bits), offering little or no asymptotic space advantage over $\lceil \lg m \rceil \approx 17\text{--}18$ bits. Worse, Open's retained first-return map $T$ does **not** inherit $D_{\sigma+1}$-avoidance (verified exact counterexample below).

### 1.2 Chakraborty, Jo, Kim, & Sadakane (ISAAC 2024)

- **Citation:** Sankardeep Chakraborty, Seungbum Jo, Geunho Kim, and Kunihiko Sadakane, *"Succinct Data Structures for Baxter Permutation and Related Families"*, 35th International Symposium on Algorithms and Computation (**ISAAC 2024**), LIPIcs vol. 322, pp. 17:1–17:17, published 4 December 2024, DOI [`10.4230/LIPIcs.ISAAC.2024.17`](https://drops.dagstuhl.de/entities/document/10.4230/LIPIcs.ISAAC.2024.17); full version [`arXiv:2409.16650`](https://arxiv.org/abs/2409.16650).
- **Verified results:**
  - **Baxter permutations** ($\operatorname{Av}(2\text{-}41\text{-}3, 3\text{-}14\text{-}2)$): succinct representation with $O(f_1(n))$ forward and $O(f_2(n))$ inverse queries for any increasing $f_1 = \omega(\log n)$, $f_2 = \omega(\log^2 n)$.
  - **Separable permutations** ($\operatorname{Av}(2413, 3142)$, equivalently permutations whose substitution-decomposition tree contains only linear $\oplus/\ominus$ nodes and zero prime nodes): first succinct representation supporting $\rho(i)$ and $\rho^{-1}(j)$ in $O(1)$ time (circumventing Golynski's SODA 2009 general permutation lower bound).
- **Mapping to `mibz2`:** Separability is testable **exactly** in $O(m)$–$O(m \log m)$ time via the strong-interval tree (`prime_nodes == 0`). If raw LF or Open's $T$ contains `2413` or `3142` (or a large prime core), separable-permutation representations do not apply directly.

### 1.3 Brignall (2008 / 2010) & Bergeron–Chauve–de Montgolfier–Raffinot (2008)

- **Citations:**
  - Robert Brignall, *"A Survey of Simple Permutations"*, [`arXiv:0801.0963v2`](https://arxiv.org/html/0801.0963v2) (18 April 2008; published in *Permutation Patterns*, LMS Lecture Note Series 376, 2010; institutional copy `oro.open.ac.uk/23475/1/simples-survey.pdf`).
  - Anne Bergeron, Cedric Chauve, Fabien de Montgolfier, and Mathieu Raffinot, *"Computing common intervals of $K$ permutations, with applications to modular decomposition of graphs"*, *SIAM J. Discrete Math.* 22(3):1022–1039, 2008 (preceded by Uno & Yagiura 2000 for $O(n)$ single-permutation interval decomposition).
- **Verified definitions & theorems:**
  - An **interval** of $\pi$ is a contiguous index range $I = [a, b]$ whose image $\pi(I)$ is also contiguous ($\max_{i\in I}\pi(i) - \min_{i\in I}\pi(i) = b - a$). A permutation of length $n \ge 4$ with no proper nontrivial intervals is **simple**.
  - **Substitution decomposition (Albert–Atkinson, Prop. 1.1–1.2):** Every permutation uniquely decomposes as an inflation $\sigma[\alpha_1, \dots, \alpha_k]$ of either a simple permutation $\sigma$ ($k \ge 4$) or a linear order $\oplus$ ($12\cdots k$) / $\ominus$ ($k\cdots 21$).
  - A **strong interval** commutes with every other interval ($J \subseteq I$, $I \subseteq J$, or $I \cap J = \emptyset$). Every permutation of length $n$ has at most $2n - 1$ strong intervals, forming its unique **strong-interval tree** (computable in $O(n)$ time, or $O(n \log n)$ via a segment-tree permutation-tree sweep).
  - **Invariance under inversion:** $I = [a, b]$ is an interval of $\pi$ with $\pi(I) = [c, d]$ iff $[c, d]$ is an interval of $\pi^{-1}$ with $\pi^{-1}([c, d]) = [a, b]$. Thus $\pi$ and $\pi^{-1}$ have isomorphic strong-interval trees, identical prime-node arity spectra, and identical separability.
  - **Uniform random baseline (Albert–Atkinson–Klazar, Thm. 2.1):** For a uniform random $\pi \in S_n$, $\mathbb{E}[X_2] \to 2$, $\sum_{k=3}^{n-1} \mathbb{E}[X_k] \to 0$, and the fraction of simple permutations approaches $e^{-2} \approx 13.5\%$; the typical strong-interval tree is nearly flat with a single prime node of size $\approx n - 2$.
- **Mapping to `mibz2`:**
  - In raw LF, every maximal byte run of length $\ell \ge 2$ is a strict $+1$ linear $\oplus$ interval ($\operatorname{LF}(i+d) = \operatorname{LF}(i) + d$).
  - In Open's retained map $T$, thin runs of length $w \in [2, 16]$ that stay within a single band remain strict $+1$ linear $\oplus$ intervals (with identical return-word labels!). The decisive empirical question is whether $T$ has **hierarchical** strong intervals beyond strict $+1$ runs, or whether deflating strict $+1$ runs leaves a single giant simple (prime) core.

### 1.4 Barbay & Navarro (TCS 2013 / arXiv:1108.4408)

- **Citation:** Jérémy Barbay and Gonzalo Navarro, *"On compressing permutations and adaptive sorting"*, *Theoretical Computer Science* **513** (2013), pp. 109–123, DOI [`10.1016/j.tcs.2013.10.019`](https://doi.org/10.1016/j.tcs.2013.10.019) (early version STACS 2009; preprint [`arXiv:1108.4408`](https://arxiv.org/abs/1108.4408)).
- **Verified mechanisms & bounds:**
  - **Contiguous ascending / monotone runs (`nRuns`, length vector `vRuns`):** A Huffman-shaped merge tree (equivalent to a Huffman-shaped wavelet tree over run merges) represents $\pi$ in $(1 + H(\text{vRuns}))(n + o(n)) + O(\text{nRuns} \lg n)$ bits, supporting $\pi(i)$ and $\pi^{-1}(i)$ in $O(1 + \lg \text{nRuns})$ worst-case and $O(1 + H(\text{vRuns}))$ average time (via `rank1` to locate the run leaf followed by upward `select0`/`select1` traversal to the root).
  - **Strict runs (`nSRuns`, length vector `vSRuns`):** Maximal contiguous segments with $\pi(i+1) = \pi(i) + 1$ (or $\pm 1$ for strict monotone runs). Collapsing strict runs reduces $\pi$ to a strict-run-head permutation $\pi'$ of size `nSRuns`, plus a bitvector $B$ of length $n$ with `nSRuns` 1-bits marking run heads (`rank1` + `select` recovers the run head and offset `i - head_src` so $\pi(i) = \text{head\_dst}[r] + (i - \text{head\_src})$).
  - **Shuffled Up Sequences (`nSUS`) and Shuffled Monotone Sequences (`nSMS`):** Decomposing $\pi$ into interleaved (non-contiguous) ascending/monotone subsequences. By Dilworth's theorem, the minimum number of ascending subsequences equals the length of the **Longest Decreasing Subsequence** ($\min \text{nSUS} = \text{LDS}(\pi)$).
- **Direct structural bridge to `src/open.rs` (crucial discovery):**
  - Open's `A6c` wiring (`src/open.rs` lines 1370–1387 of the mirrored harness) writes `edges[dst..dst + span]` band-by-band from `self.bands` and `queues[a]`!
  - Because `queues[a]` lists retained ranks of byte `a` in **strictly increasing order**, **every single Open band `[dst, dst + span)` is a contiguous ascending run of the forward retained map $T = \Psi_R$ WITH A CONSTANT WORD CODE `bcode`!**
  - Therefore, on $T = \Psi_R$ (the exact permutation traversed by the forward consumer!), the number of **labelled contiguous ascending runs** `nLRuns(T)` is **guaranteed $\le$ `#bands`** (typically $\sim 3,500$ runs across $m \approx 102,000$ retained rows — a $\sim 29\times$ run ratio!), whereas `nRuns(T^{-1})` on $\text{LF}_R$ is much larger because consecutive thin runs belong to different bytes!
  - Furthermore, inside each band, every thin run of width $w \in [2, 16]$ produces a **labelled strict $+1$ run** (`T(i+1) = T(i) + 1` and `code(i+1) = code(i)`), directly realizing Barbay–Navarro's `nSRuns` structure *with label preservation*!
  - **Consumer engineering challenge:** Barbay–Navarro's original Huffman-merge wavelet tree performs $O(1 + H(\text{vRuns})) \approx 10\text{--}12$ dependent `select` operations per step (far slower than one 32-bit load from `edges[j]`). To have any chance of beating `ctrl_shipped_fwd`, a run-based representation must achieve **$O(1)$ single-cache-line random access** (e.g., cache-line-aligned monotone-block delta encoding for ascending runs, or 64-bit `popcnt` + `lzcnt` rank/prev-bit indexing for strict $+1$ runs) while shrinking the working set into 256 KiB L2!

### 1.5 Even-Zohar & Leng (SODA 2021 / arXiv:1911.01414)

- **Citation:** Chaim Even-Zohar and Calvin Leng, *"Counting Small Permutation Patterns"*, Proceedings of the 2021 ACM–SIAM Symposium on Discrete Algorithms (**SODA 2021**), pp. 2288–2302, DOI [`10.1137/1.9781611976465.136`](https://doi.org/10.1137/1.9781611976465.136); preprint [`arXiv:1911.01414v3`](https://arxiv.org/abs/1911.01414).
- **Verified results:** Corner-tree formulas count all patterns of order 2 and 3 (all 6 length-3 patterns) and 8 patterns of order 4 (`1234, 1243, 2134, 2143, 3412, 3421, 4312, 4321`) in $\tilde{O}(n)$ time, plus 23 independent linear combinations of the 24 order-4 patterns in $\tilde{O}(n)$ time and all 24 order-4 patterns in $\tilde{O}(n^{3/2})$ time (or $\tilde{O}(n^{5/3})$ time, $\tilde{O}(n)$ space).
- **Mapping to `mibz2`:** Gives an exact $O(m \log m)$ diagnostic (6 exact 3-pattern counts via Fenwick trees) to distinguish true order structure from uniform random permutations without sampling bias. As noted in `chatgpt-response.md`, a non-random 3-pattern profile is diagnostic only — it does not by itself supply an execution shortcut.

---

## 2. Raw LF vs. Open's Labelled First-Return Map $T$

| Property | Raw LF ($\pi \in S_n$) | Open's Forward Retained Map ($T = \Psi_R \in S_m$) | Open's Inverse Retained Map ($T^{-1} = \text{LF}_R \in S_m$) |
|---|---|---|---|
| **Avoids $D_{\sigma+1}$?** | **Proved YES** (stable sort partitions into $\sigma$ ascending subsequences $\implies \text{LDS} \le \sigma$). | **NOT inherited** in general (exact counterexample: $L = \texttt{bbaaa}$, $\sigma = 2$, raw LF $= [4,5,1,2,3] \in \operatorname{Av}(321)$, retaining $\{1,2,3,4\}$ gives $\text{LF}_R = [4,3,1,2]$ and $\Psi_R = [3,4,2,1]$, both containing `321`!). | **NOT inherited** (same `bbaaa` witness: `4,3,1` is `321`). |
| **Contiguous ascending runs (`nRuns`)** | At most $r$ (one per maximal byte run; mean run $\approx 16$). | **At most `#bands`** ($\approx 3,500 \ll m \approx 102,000$; mean band $\approx 29$!) and **label-constant per band**. | Much larger ($\sim$ half of thin runs, since adjacent thin runs belong to distinct bytes). |
| **Strict $+1$ runs (`nSRuns`)** | At most $r$ (every byte run is $+1$ linear with constant byte). | Preserved within each thin run staying inside one band (width $\le 16$, constant word code). | Same strict $+1$ run structure under $\pi \leftrightarrow \pi^{-1}$ bijection! |
| **Strong-interval tree / separability** | Every byte run is a $\oplus$ leaf interval; quotient structure measured in census. | Isomorphic strong-interval tree to $T^{-1}$; measured **exactly** in Part 1 census. | Exact bijection of intervals with $T$: $I$ interval of $T \iff T(I)$ interval of $T^{-1}$. |
| **Label coupling** | 1 byte per row (`L[i]`), implicit in F-bucket for $\Psi$. | Variable-length return word (`code` $\in [0, D)$), packed into same 32-bit `edges[j]` word as `next`. | Same label set, attached to outgoing arcs of $\text{LF}_R$. |
| **What consumer actually needs** | $n$ steps of $\Psi$ (or $\text{LF}$), emitting 1 byte/step. | $m$ steps of $T$ emitting `word[code[j]]` in text order. Compressing `next` alone without keeping `code` cheap adds indirection! | Not traversed directly (forward text order uses $T = \Psi_R$). |

---

## 3. Representation hypotheses carried into pre-registration

From this primary-source audit, four concrete pattern-derived families emerge, each tied to an **exact, verifiable structural condition** on the training blocks:

1. **Family A — Labelled Strict $+1$ Run / $\oplus$-Interval Rank-Indexed Map** *(Barbay–Navarro `nSRuns` + Brignall linear $\oplus$ intervals)*: collapses every maximal labelled strict $+1$ run (`T(j+1) == T(j)+1 && code(j+1) == code(j)`) into a single 32-bit head edge + 64-bit `popcnt`/`lzcnt` rank-and-prev-bit directory (`0.1875 B/row`). Directly tests whether halving the edge table into 256 KiB L2 outweighs the extra bit-rank arithmetic and second cache access.
2. **Family B — Labelled Ascending-Run Monotone-Block Map** *(Barbay–Navarro `nRuns` specialized for $O(1)$ cache-line access)*: exploits the proved structural theorem `nLRuns(T) <= #bands` by packing fixed-width blocks of $B$ consecutive vertices into cache-line-friendly monotone delta blocks `(base, code, relative_offsets)` (with exact escape for boundary/wide blocks), avoiding both wavelet-tree $O(\lg \text{nRuns})$ chains and binary search.
3. **Family C — General Substitution-Decomposition / Separable Tree Map** *(Brignall strong intervals + Chakraborty et al. ISAAC 2024)*: viable only if the exact strong-interval tree avoids a giant prime core (`max_prime_arity / m <= 0.50` or separable).
4. **Family D — Pattern-Avoidance Matrix-Division Map** *(Kozma–Opler ESA 2026)*: viable only if $T$ satisfies exact short-pattern avoidance ($|\pi| \le 4$ or $\text{LDS} \le 8$) with small Stanley–Wilf limit.
