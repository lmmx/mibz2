# E3 — Dynamic Monotone Rank-Envelope (`1/3` Pigeonhole-Pin Law) & Algebraic Enrichment Dichotomy

**Principle:** Enrichment along composition (Principle 2: what one `LF` step reveals for free about other rows, L-BFGS-style dynamic history, and the exact boundary separating commutative vs non-commutative monoid riders).  
**Script & Data:** [`e3_monotone_envelope_dichotomy.py`](e3_monotone_envelope_dichotomy.py), [`results_e3_monotone_envelope_dichotomy.json`](results_e3_monotone_envelope_dichotomy.json).

## 1. Exact Mathematical Definitions

### Part A — Dynamic 1-Lipschitz Monotone Rank-Envelope
Because $\mathrm{LF}$ restricted to each symbol fibre $S_c = \{x_0 < x_1 < \dots < x_{n_c - 1}\}$ satisfies $\mathrm{LF}(x_k) = C[c] + k$ (strictly monotone and 1-Lipschitz), every visited pair $(x_L, y_L)$ and $(x_R, y_R)$ with $L[x_L] = L[x_R] = c$ imposes the exact sandwich on every unvisited $c$-row $z \in (x_L, x_R)$:
$$y_{\mathrm{lo}}(z) = \max\big(y_L + 1,\; y_R - (x_R - z)\big) \;\le\; \mathrm{LF}(z) \;\le\; \min\big(y_R - 1,\; y_L + (z - x_L)\big) = y_{\mathrm{hi}}(z).$$
Whenever $y_{\mathrm{lo}}(z) = y_{\mathrm{hi}}(z)$, $\mathrm{LF}(z)$ is **uniquely deduced** from previously visited rows without reading `tt[z]`. Exhaustive case analysis shows $y_{\mathrm{lo}}(z) = y_{\mathrm{hi}}(z)$ holds iff:
1. **Pigeonhole pin:** $y_R - y_L = 2$ (both immediate $c$-neighbours $k-1$ and $k+1$ in $S_c$ were already visited, forcing $\mathrm{LF}(z) = y_L + 1$);
2. **1-Lipschitz run pin:** $x_R - x_L = y_R - y_L$ (all rows between $x_L$ and $x_R$ share byte $c$, i.e., a multi-row run);
3. **Boundary pin:** $k \in \{0, n_c - 1\}$ with its sole neighbour visited (or $n_c = 1$).

### Part B — The Algebraic Enrichment Dichotomy
Let $r : X \to (M, \otimes, 1_M)$ be any rider acquired from $L$ in $P \le K$ streaming passes, composing along $\mathrm{LF}$-orbits via $R_k(x) = r(x) \otimes r(\mathrm{LF}(x)) \otimes \dots \otimes r(\mathrm{LF}^{k-1}(x))$.

## 2. Correctness & Theorems (`[proved]`, 8/8 byte-exact verified)

**Theorem E3.1 (Exact `1/3` Pigeonhole-Pin Law — proved & verified).**
For any symbol fibre $S_c$ whose visit order along the inverse-BWT walk is uniformly random, each interior lone row $x_k \in S_c$ is visited *after* both $x_{k-1}$ and $x_{k+1}$ with probability exactly $1/3$. Consequently, exactly $1/3$ of lone rows are information-theoretically forced by the dynamic monotone rank-envelope at the instant they are reached. $\square$

**Theorem E3.2 (Algebraic Enrichment Dichotomy — proved with explicit 8/8 witnesses).**
Let $(x, x+1, x+2) \subset I_f$ be any three consecutive lone rows inside a single F-bucket holding three distinct bytes $(a, b, c)$. Among the two non-trivial cyclic 3-rotations $(a,b,c) \mapsto (b,c,a)$ and $(c,a,b)$:
1. Because $a, b, c$ are distinct and $(x, x+1, x+2)$ are contiguous, $\mathrm{LF}'(y) = \mathrm{LF}(y)$ for **all $n - 3$ rows $y \notin \{x, x+1, x+2\}$**, and the global histogram, the 2-gram matrix $M_2[f, \cdot]$, and **every commutative-monoid aggregate** $\bigotimes_{y \in B} \phi(L[y], F[y])$ over any block/bucket containing $[x, x+2]$ are **strictly invariant**.
2. Exactly one of the two 3-rotations (the one matching the cyclic visit order of $\{x, x+1, x+2\}$ along $\mathrm{LF}$) preserves a **single $n$-cycle** while permuting three macroscopic segments of the decoded text $T' \neq T$.
3. Therefore: (i) no commutative monoid/semiring rider can skip lone-row resolution; (ii) any non-commutative rider carrying $k$-step transport faces the exact $(Q, P, s)$ composition trade-off ($P \ge k$ or $s \ge |\Sigma|^k$); (iii) genuine algebraic enrichments that cut total decoder work must therefore **absorb non-`LF` stages via homomorphisms** (upstream: E1's $\mathcal{M}_{\mathrm{MTF}}$; downstream: `carbon-kitchen-sink` Cycle 4's $\mathrm{RLE1} \circ \mathrm{CRC32}$ transducer) or trade $(D, P, S)$. $\square$

## 3. Prior Art

- **MONODROMY record 115 §2:** Argued informally (`[A]`) that no cheap rider cuts `Q` because $\mathrm{LF}^k(x)$ carries $\log_2 n$ new bits; did not analyze the dynamic 1-Lipschitz monotone envelope (`1/3` pigeonhole pin) or give a formal commutative-vs-non-commutative swap witness.
- **Neon N-18 / N-21:** Proved scalar moments $(H, A, A_2)$ fail to fix lone rows via multi-flip witnesses; Theorem E3.2 sharpens this to **any commutative monoid** using a 3-row contiguous witness changing only `3` entries of `LF`.
- **`carbon-kitchen-sink` Cycle 4 (`ba9e533`):** Already proved and verified the downstream $\mathrm{RLE1} \circ \mathrm{CRC32}$ 5-state affine transducer over $\mathbb{N} \ltimes \mathbb{F}_2[x]/(P_{32})$ (Escape Hatch I).

## 4. Measurements on Novel Random Hex & Six Admitted Blocks (8/8 Byte-Exact)

| Block | Lone pigeonhole pins (`% h_1`, theory `33.33%`) | Total envelope pins (`% n`, incl. run pins) | 3-rotation witness `x` (`dLF = 3`, `M_2` & hist inv.) | Decoded text Hamming diff (`% n`) | Byte-exact (pinned steps skip `tt`) |
|---|---:|---:|---:|---:|:---:|
| `synth-hex-iid-65536` | **`33.31%`** | `35.61%` | `x = 0` | `75.2%` | EXACT |
| `synth-tmpl-hex-1000x40` | **`31.93%`** | `62.83%` | `x = 0` | `20.1%` | EXACT |
| `block-0.L` | **`32.07%`** | `95.57%` | `x = 11` | `58.4%` | EXACT |
| `block-2.L` | **`32.02%`** | `95.16%` | `x = 3` | `28.7%` | EXACT |
| `block-85.L` | **`29.86%`** | `93.03%` | `x = 0` | `10.4%` | EXACT |
| `block-143.L` | **`31.75%`** | `90.53%` | `x = 0` | `38.5%` | EXACT |
| `block-210.L` | **`31.29%`** | `89.08%` | `x = 0` | `78.8%` | EXACT |
| `block-207.L` | **`32.16%`** | `92.56%` | `x = 10` | `61.6%` | EXACT |

## 5. Referee Under $\Pi$

- **`Q`:** Although `29.9–33.3%` of lone rows are **information-theoretically forced** by the dynamic monotone rank-envelope without reading `tt[x]`, testing whether $k-1$ and $k+1$ in $S_c$ are already visited requires an addressed lookup into the dynamic envelope state (`Q >= 1` per lone row). Thus `Q` is **not lowered** under $\Pi$ (`Q = n` raw, or `Q = m` on $M_{16}$).
- **`D`, `P`, `S`:** `D = n`, `P = 1`, `S = 5n` (tracking visited bits + ranks).
- **Verdict:** Sharp theoretical boundary — proves both a **positive information-theoretic phenomenon** (`1/3` of all lone rows are dynamically forced by monotonicity alone) and the **exact algebraic obstruction** (Theorem E3.2) explaining why only cross-stage/downstream homomorphisms (E1, `carbon-kitchen-sink` C4) or $(D, P, S)$ trades escape `Q >= h_1` at `K = 2`.
