# Carbon Kitchen Sink — Cycle 8: Topological Skew-Products, Wreath Factorisation of $\mathrm{LF}$, Functorial Non-Commutativity $\pi(\mathrm{BWT}(T)) \ne \mathrm{BWT}(\pi(T))$, and $G_{\mathrm{ov}}$ Spectral Zeta Entropy

**Date:** 2026-11-20
**Branch:** `research/carbon-kitchen-sink`
**Inputs:** [`2026-11-20-carbon-kitchen-sink-cycle7.md`](2026-11-20-carbon-kitchen-sink-cycle7.md) (`c9262be`), `docs/team/research-charter/FRONTIER.md` (Foundational Question 4 & `working-backwards.md` STARLIT provocation).
**Artefacts:** [`probe_cycle8.py`](2026-11-20-carbon-kitchen-sink-cycle8/probe_cycle8.py), [`fast_cyclic_bwt.c`](2026-11-20-carbon-kitchen-sink-cycle8/fast_cyclic_bwt.c), [`probe_cycle8_results.json`](2026-11-20-carbon-kitchen-sink-cycle8/probe_cycle8_results.json).

---

## 0. Motivation: Real Texts as Topological Skew-Products

Cycles 5–7 established that structured JSON/markup corpora with embedded identifiers (`$UUID`s, `SHA1` hashes, numeric IDs) behave dynamically like a **skew-product (fiber bundle)** $\overline{T} \ltimes \boldsymbol{\xi}$: a low-entropy syntactic base $\overline{T}$ coupled to a 4-bit cryptographic-noise fiber $\boldsymbol{\xi}$.

Cycle 8 confronts this structure mathematically from three directions:
1. **Post-BWT Wreath Factorisation (Unmodified `.bz2` Streams):** Given $(L, p)$ directly from a standard `.bz2` block, how does $\mathrm{LF}$ factor under a surjective alphabet quotient $\pi : \Sigma \twoheadrightarrow \overline{\Sigma}$, how much does $r(\pi(L))$ collapse, and what exact fiber obligation remains?
2. **Functorial Non-Commutativity & Pre-BWT Skew-Product Splitting (`Foundational Question 4` / `STARLIT`):** Because $\pi(\mathrm{BWT}(T)) \ne \mathrm{BWT}(\pi(T))$, what happens if a lossless syntactic bijection $T \leftrightarrow (\overline{T}, \boldsymbol{\xi}_{\mathrm{text}})$ separates hex-token fibers *before* the BWT?
3. **Bowen–Lanford–Ruelle Spectral Zeta Determinant on $G_{\mathrm{ov}}$:** Quantifying the exact exponential pseudo-orbit inflation $h_{\mathrm{top}}(G_{\mathrm{ov}}) = \log_2 \lambda_{\max}(A_{\mathrm{ov}})$ when passing from $([0, n), \mathrm{LF})$ ($h_{\mathrm{top}} = 0$) to the 1-step run-overlap network $G_{\mathrm{ov}}$.

---

## 1. Post-BWT Wreath Factorisation of $\mathrm{LF}$ & the Fiber-Scattering Law

Let $\pi : \Sigma \twoheadrightarrow \overline{\Sigma}$ be any alphabet surjection with local fiber coordinates $\iota_{\overline{b}} : \pi^{-1}(\overline{b}) \xrightarrow{\sim} \{0, \dots, d_{\overline{b}} - 1\}$. Decompose $L \in \Sigma^n$ into its coarse quotient column $\overline{L} = \pi(L) \in \overline{\Sigma}^n$ and, for each non-singleton class $\overline{b}$ ($d_{\overline{b}} > 1$), its dense fiber stream $\boldsymbol{\xi}^{(\overline{b})} \in \{0, \dots, d_{\overline{b}} - 1\}^{N_{\overline{b}}}$.

### Theorem 8.1 (Wreath Factorisation of $\mathrm{LF}$ & Multi-Queue Fiber Scattering — *proved*; *verified* on `5,456` 4-ary words + 6 Wikidata blocks)

1. **Exact Wreath Decomposition:** For every $x \in [0, n)$, let $\overline{b} = \overline{L}[x]$ and $k = \mathrm{rank}_{\overline{b}}^{\overline{L}}(x)$. Note $k = k_u + (x - d_u)$ is unit-slope affine on every maximal run $[d_u, d_u + w_u)$ of $\overline{L}$. Then:
   - if $d_{\overline{b}} = 1$ (`uncollapsed base byte`), $\mathrm{LF}(x) = C[\overline{b}] + k$ is rigidly unit-slope affine on the entire coarse run;
   - if $d_{\overline{b}} > 1$ (`fiber-carrying class`),
     $$\mathrm{LF}(x) = C\!\left[\iota_{\overline{b}}^{-1}\!\left(\xi^{(\overline{b})}_k\right)\right] + \mathrm{rank}_{\xi^{(\overline{b})}_k}^{\boldsymbol{\xi}^{(\overline{b})}}(k).$$
2. **Post-BWT Fiber-Scattering Obstruction:** Under $\pi_{\mathrm{hex}} : \mathtt{[0\text{-}9a\text{-}f]} \mapsto \mathtt{HEX}$ on the 6 Wikidata blocks, $r(\pi_{\mathrm{hex}}(L))$ drops by **`42.7%–55.0%`** (`51,184 → 28,197` on `block-0`; `108,335 → 48,798` on `block-143`) and coarse thin rows $m(\pi_{\mathrm{hex}}(L))$ drop by `26%–42%`. However, a coarse `HEX` run $[d_u, d_u + W)$ of $\pi_{\mathrm{hex}}(L)$ is **not** a free `Open` strip: $\mathrm{LF}$ scatters $[d_u, d_u + W)$ into $d_{\mathtt{HEX}} = 16$ monotone sub-streams across $C[\mathtt{'0'}..\mathtt{'f'}]$ via the `296,480–325,544`-nibble fiber $\boldsymbol{\xi}^{(\mathtt{HEX})}$. Resolving $\mathrm{rank}_{\xi_k}^{\boldsymbol{\xi}}(k)$ along the orbit requires either storing explicit target edges on `HEX` rows (recovering `Open`) or 4-level wavelet-rank queries (recovering `BSO`'s serial rank latency).

---

## 2. Functorial Non-Commutativity & Reversible Pre-BWT Factor Splitting (`Pre-HexTok`)

### Theorem 8.2 (Functorial Non-Commutativity & Quote-Free Syntactic Hex-Span Bijection — *proved*; *verified* on `5,456` small words + byte-exact round-trip on all `5,399,692` bytes of the 6 Wikidata blocks)

1. **Minimal Non-Commutativity Witness ($n = 4$):** For $\pi : \{0, 1\} \mapsto 0,\, 2 \mapsto 2$ and $T = (0, 0, 2, 1)$,
   $$\pi(\mathrm{BWT}(0,0,2,1)) = (0, 0, 2, 0) \;\ne\; (2, 0, 0, 0) = \mathrm{BWT}(\pi(0,0,2,1)).$$
   In $\pi(\mathrm{BWT}(T))$, suffixes are sorted by the fine alphabet *before* $\pi$ is applied, so random fiber differences (`0 < 1`) tear apart identical downstream contexts via Cycle 5's Bayer–Diaconis riffle shuffle. Applying $\pi$ *before* $\mathrm{BWT}$ keeps identical templates contiguous in suffix order (`r = 2` vs `r = 3`).

2. **Exact Syntactic Bijection `Pre-HexTok` ($T \leftrightarrow (\overline{T}, \boldsymbol{\xi}_{\mathrm{text}})$):** Partition any byte string $T \in \Sigma^*$ into maximal contiguous spans $s = T[a..b)$ of characters in $\Sigma_{\mathrm{span}} = \mathtt{[0\text{-}9a\text{-}fA\text{-}F\text{-}]}$ (completely independent of JSON quote parity across block boundaries!). Whenever $|s| \ge 16$ (capturing 36-byte `$UPPERCASE-UUID` and `lowercase-uuid` tokens plus 40-byte `SHA1` hashes, while leaving short numeric IDs, `\uXXXX` escapes, and all English JSON keys/prose untouched):
   - if $s \subseteq \mathtt{[0\text{-}9a\text{-}f\text{-}]}$, replace every non-`'-'` byte with `'0'` and append its 4-bit nibble to $\boldsymbol{\xi}_{\mathrm{text}}$;
   - else if $s \subseteq \mathtt{[0\text{-}9A\text{-}F\text{-}]}$ (so $s$ contains at least one `'A'..'F'`), replace every non-`'-'` byte with `'A'` and append its 4-bit nibble to $\boldsymbol{\xi}_{\mathrm{text}}$.
   Because boundary membership $b \notin \Sigma_{\mathrm{span}}$ and both case predicates are invariant under $T \mapsto \overline{T}$, `Pre-HexTok` is an **exact, zero-escape lossless bijection on $\{0..255\}^*$** whose inverse splices nibbles sequentially into `out[0..n)` with **zero random memory accesses**.

### Empirical Confrontation on All 6 Authenticated Wikidata Blocks (`probe_cycle8.py`)

Here `bz(T)` and `bz(T_bar)` denote exact `MTF + RLE2 + Order-0 Shannon` byte sizes of the BWT columns, and `raw4(xi)` is **uncompressed 4.000 bits/nibble bit-packing** (`len(xi) / 2` bytes; Order-0 entropy of `xi` is `3.994–3.997 bits/nibble`, confirming `>99.8%` pure white noise!).

| Block | Orig $r$ | `Pre-HexTok` $r$ | Orig `Open<16>` $m$ | `Pre-HexTok` $m$ | Orig `bz(T)` | `bz(T_bar)` (`Huffman` work!) | `raw4(xi)` | Total `bz(T_bar)+raw4(xi)` | Size vs Orig |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| `block-0.L` | 51,184 | **18,379** (`0.359×`) | 89,874 | **38,653** (`0.430×`) | 44,202 B | **18,442 B** (`0.417×`) | 29,043 B | 47,485 B | `1.074×` |
| `block-2.L` | 54,532 | **23,038** (`0.422×`) | 112,070 | **58,463** (`0.522×`) | 48,955 B | **23,606 B** (`0.482×`) | 28,375 B | 51,981 B | `1.062×` |
| `block-85.L` | 79,375 | **29,338** (`0.370×`) | 156,404 | **70,168** (`0.449×`) | 65,055 B | **25,672 B** (`0.395×`) | 45,484 B | 71,156 B | `1.094×` |
| `block-143.L` | 108,335 | **54,431** (`0.502×`) | 196,841 | **128,084** (`0.651×`) | 91,092 B | **50,155 B** (`0.551×`) | 30,780 B | **80,935 B** | **`0.889×` (−11.1%)** |
| `block-210.L` | 128,732 | **48,221** (`0.375×`) | 204,281 | **108,765** (`0.532×`) | 98,567 B | **41,811 B** (`0.424×`) | 41,183 B | **82,993 B** | **`0.842×` (−15.8%)** |
| `block-207.L` | 87,111 | **39,835** (`0.457×`) | 135,921 | **79,896** (`0.588×`) | 71,005 B | **36,397 B** (`0.513×`) | 26,416 B | **62,813 B** | **`0.885×` (−11.5%)** |
| **6-Block Sum** | **509,269** | **213,242 (`0.419×`)** | **895,391** | **484,029 (`0.541×`)** | **418,876 B** | **196,083 B (`0.468×`)** | **201,280 B** | **397,363 B** | **`0.949×` (−5.1%)** |

**Why this answers Foundational Question 4 ("Is the inverse BWT the right problem?"):**
1. **Why BWT Struggles on Modern Structured Data:** Feeding 4-bit random UUID nibbles into a BWT suffix sort inflicts a **double penalty**: it creates `296,027` artificial BWT run breaks across the 6 blocks (`509,269` vs `213,242`), simultaneously inflating `MTF + Huffman` compressed size on UUID-heavy blocks (`+11%` to `+16%` larger than raw 4-bit packing because `80,511` extra MTF switches on `block-210` cost `>5.5` bits/nibble!) AND doubling `Open<16>`'s random-pointer walk (`895K` vs `484K` hops) and Huffman bitstream (`419 KB` vs `196 KB`)! *(On `block-0, 2, 85`, repeated 40-char `SHA1` reference hashes give standard BWT a `6–9%` deduplication edge; deduplicating repeated tokens before nibble extraction captures both.)*
2. **Scope Boundary:** `Pre-HexTok` requires control of the **encoder/format** (a STARLIT-style BWT preprocessor, not compatible with legacy `.bz2` bitstreams). For unmodified `.bz2` inputs, Theorem 8.1(2) proves the random nibbles are already mixed into $F$'s sort order and cannot be bypassed without `296K–326K` fiber rank/pointer operations.

---

## 3. Bowen–Lanford–Ruelle Spectral Zeta Determinant & $G_{\mathrm{ov}}$ Pseudo-Orbit Entropy

### Theorem 8.3 (Zeta Determinant Collapse vs $G_{\mathrm{ov}}$ Topological Entropy — *proved*; *verified* on `230` binary BWTs + 6 Wikidata blocks)

On any primitive BWT $(L, p)$, $P_{\mathrm{LF}}$ is a single $n$-cycle, so its Artin–Mazur dynamical zeta reciprocal is $\det(I_n - z P_{\mathrm{LF}}) = 1 - z^n$ with topological entropy $h_{\mathrm{top}}(\mathrm{LF}) = 0$. Passing to the unweighted $r$-vertex run-overlap graph $G_{\mathrm{ov}}$ forgets intra-run sub-interval alignment, embedding the true orbit inside a subshift of finite type $\Sigma_{G_{\mathrm{ov}}}$ whose Perron–Frobenius eigenvalue $\lambda_{\max}(A_{\mathrm{ov}})$ measures the exponential growth rate of spurious pseudo-orbits:
- `block-0.L`: $\lambda_{\max} = 3.50287 \implies h_{\mathrm{top}}(G_{\mathrm{ov}}) = \mathbf{1.8085\text{ bits/step}}$;
- `block-2.L`: $\lambda_{\max} = 3.37779 \implies h_{\mathrm{top}}(G_{\mathrm{ov}}) = \mathbf{1.7561\text{ bits/step}}$;
- `block-85.L`: $\lambda_{\max} = 5.18630 \implies h_{\mathrm{top}}(G_{\mathrm{ov}}) = \mathbf{2.3747\text{ bits/step}}$;
- `block-143.L`: $\lambda_{\max} = 5.35430 \implies h_{\mathrm{top}}(G_{\mathrm{ov}}) = \mathbf{2.4207\text{ bits/step}}$;
- `block-210.L`: $\lambda_{\max} = 4.56016 \implies h_{\mathrm{top}}(G_{\mathrm{ov}}) = \mathbf{2.1891\text{ bits/step}}$;
- `block-207.L`: $\lambda_{\max} = 3.85915 \implies h_{\mathrm{top}}(G_{\mathrm{ov}}) = \mathbf{1.9483\text{ bits/step}}$.

Every step on $G_{\mathrm{ov}}$ without exact metric width tracking multiplies trajectory ambiguity by **`3.38–5.35×`**, giving the exact spectral counterpart to Cycle 4's Ribbon Metric-Blindness theorem.

---

## 4. Bridge to Cycle 9

In **Cycle 9**, we move from 1-step spectral properties of $G_{\mathrm{ov}}$ to **Cellular Sheaf Cohomology $H^0(\mathcal{G}; \mathcal{F})$**, **Non-Abelian $\mathrm{SL}_2(\mathbb{F}_P)$ Sanov–Cayley Monodromy**, and **Directed Feedback Vertex Sets vs Undirected Forests** (`Foundational Question 3` — quantifying the exact duality between directed feedback cuts and diamond-path explosion on $G_{\mathrm{ov}}$).
