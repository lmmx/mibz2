# Extraction notes: what has been checked, and how

The précis were written from text extracted with `pdftotext -layout`. Some symbols, overbars, matrices and line breaks were lost. Each place below is in one of four states:
- **A. Author's erratum applied:** from the authors' published errata (https://hybrid.soe.ucsc.edu/publications/errata-book-hybrid-dynamical-systems-modeling-stability-and-robustness, read 25 November 2026).
- **B. Verified:** against the book's own stated solutions or surrounding text.
- **C. Book-level point:** the précis reports the book faithfully, but the printed statement looks wrong or inconsistent (raised in ChatGPT's review, `../../../external/chatgpt-hybrid-systems-review.md`). Check the page before relying on it.
- **D. Reconstruction still unverified:** check against the rendered page.

| state | chapter | place | the issue, and what the précis now says |
|---|---|---|---|
| A | 2 | Definition 2.3, p. 27 | hybrid time domain is the union of a nondecreasing sequence of compact hybrid time domains (the printed truncation test is replaced) |
| A | 2 | Example 2.8, p. 32 | $\phi_d(1,j) = ((3/4)^j, \tfrac14(3/4)^{j-1})$ |
| A | 2 | p. 34, line −8 | the claim that $\phi_b$ jumps out of $D \cup \overline C$ is withdrawn; the twice-jumping solution of Example 2.9 stops at $-1 \in C$ where flow cannot continue |
| A | 2 | p. 37, p. 38 | weak dwell-time subscript ($t_{i_k}$); "is a solution"; Fig. 2.8(a) caption "smaller than or equal to"; $f_q(z)$ for $f_q(\phi)$ |
| A | 5 | Example 5.13, p. 104 | for $S = [a, b]$, $T_S(a) = [0,\infty)$ and $T_S(b) = (-\infty, 0]$. The sine-region wedge remains **D** |
| A | 5 | Definition 5.9, p. 102 | the mapping is from $\mathbb R^m$ to $\mathbb R^n$ |
| A | 6 | Example 6.20, p. 129 | $S_1$ is (strongly) forward invariant and *not* weakly backward invariant |
| A | 6 | Prop 6.26 proof, p. 131; Prop 6.35 title, p. 136 | "backward" for "forward"; title "non-Zenoness and non-continuousness" |
| A | 7 | Lemma 7.17, p. 148; Example 7.16; Example 7.35, p. 161 | $\mathcal A \subset U$; a corrected sentence on $\rho$; $\nabla$ placement |
| B | 2 | Example 2.8 data | $f = (1,1)$, $g(x) = (\tfrac34x_1, \tfrac14x_1)$, checked against the stated solutions and now against the erratum's $\phi_d$ |
| B | 6 | Example 6.4, p. 119 | the flow map is $\dot x_1 = x_3 x_1^2$ (not $x_1^2$), verified against both stated solution pieces, $(1-t)^{-1}$ with $x_3 = 1$ and $(2\varepsilon - 1 + t)^{-1}$ with $x_3 = -1$ |
| C | 3 | Example 3.1, p. 44 | the printed target $\{0\} \times \mathbb R^{n_c} \times [0, T]$ leaves the held control free at $z = 0$; a state with $f_p(0, u) \ne 0$ leaves it before the next sample |
| C | 3 | Example 3.23, pp. 58–59 | the printed $V = \sigma_q(W_q(z))$ vanishes at $(2, 0)$, outside the target $\{q = 1, z = 0\}$ |
| C | 5 | Theorem 5.29(b), p. 113 | printed "not uniformly bounded"; $z_i(t) = t$ is globally unbounded, locally eventually bounded, and has no finite-time blow-up. The coherent reading is "not locally eventually bounded" |
| D | 3 | Examples 3.7, 3.13, 3.21, 3.22, 3.35; Theorem 3.37; Lemma 3.41 | garbled displays rebuilt from context (see the précis) |
| D | 4 | Examples 4.4, 4.15, 4.19; Lemma 4.20, Theorem 4.21 proofs; Definitions 4.13, 6.27; Example 4.14 | staircase map described qualitatively; "jump set" read as the displayed flow set; hats and bars rebuilt |
| D | 5 | (⋆⋆), p. 112; Example 5.13's sine region | index slip corrected; possible lost $w_2 \ge 0$ |
| D | 6 | Lemma 6.9 proof; Example 6.11; Theorem 6.30 proof; Prop 6.33 | lost closure bars; viability condition misprint; "Lemma 5.29" for Theorem 5.29; "pre-forward complete" |
| D | 7 | Examples 7.2, 7.6; Lemmas 7.14, 7.20 proofs; Cor 7.33 title; §7.5 intro; Lemma 7.37 | displays scrambled or symbols renamed |
| D | 8 | (8.1), (8.3); Theorem 8.11 proof; Proposition 8.16 | overbars lost; "T" for τ; ∪ possibly ∩, proof stops short |
| D | 9 | Definition 9.5, Prop 9.6; Example 9.15 | subscript rendering; broken sentence |
