# How the compact GRH proof closes its squeeze — and whether it maps onto mibz2

Thread 7 of [`../plan.md`](../plan.md), answering part 3 of [`../rauzy-census/prompt.md`](../rauzy-census/prompt.md). Reading note only: no code.

**Primary source read:** `https://github.com/lsernau/grh-lean`, compact edition `grh-compact/` (~76k LOC Lean 4, 219 files in `src/New/`, 573 files in `src/Lib/`) alongside the readable edition's `grh/GUIDE.md` (~300k LOC). As noted in `grh-compact/README.md`, this is a machine-checked Lean 4 development depending only on `[propext, Classical.choice, Quot.sound]` (`grh-compact/src/Statements.lean`, `src/Main.lean`); no mathematician has read it through.
**Prior note compared:** [`../../2026-11-21-the-shape-of-the-rh-proof.md`](../../2026-11-21-the-shape-of-the-rh-proof.md), which worked from Chris Birkbeck's second-hand summary of the original ~17M-line development and recorded that Birkbeck had *"not yet understood how these combine to exclude zeros"*.

---

## Verdict (frozen stop test)

**STOP — THREAD 7 ENDS AS THIS NOTE.**

Reading the actual Lean spine (`grh-compact/src/New/Main.lean`) resolves Birkbeck's open question completely, and simultaneously settles the frozen stop test: **no move of the GRH squeeze has an exact, load-bearing counterpart in mibz2's three bounds** (journal §70's lower bounds, `ideal-parts/results/full/README.md`'s measured rooms, and Part 2's network calculus upper bounds). The GRH proof does not work by intersecting three unrelated bounds on a performance target; it works by (i) bounding **one exact algebraic covolume** from above and below with an **identical leading term `h_main`** that cancels to zero via the global product formula, and (ii) feeding the resulting **one-sided** upper bound on a non-negative prime sum into **Landau's analytic-continuation lemma** after twisting away zero cancellations. None of those mechanisms maps onto quantities mibz2 measures or can compute.

---

## 1. How the squeeze closes (for a programmer, with exact Lean names)

### The setup: one covolume bounded from two sides

Fix a modulus $N_0$ (a multiple of `18480`) and a prime $d \equiv N_0/2 - 1 \pmod{N_0}$ (so $d \equiv 7 \pmod 8$, `Parameters` in `grh-compact/src/New/Data/`). Let $K = \mathbb{Q}(\sqrt{-d})$ with quadratic character $\chi_K$, and $E = K(\zeta_{N_1})$ be a large cyclotomic CM field with $f = \varphi(N_0)\cdot\#\mathrm{tops}$ infinite places (`Data.tops`, `Data.thick`, scales in `grh-compact/src/New/Main.lean:88-103`).

The proof works with two finite-dimensional spaces of automorphic forms and a chain of $m \in [\sqrt{x}, 2\sqrt{x}]$ theta lifts:
- **Columns (`D.Column`)**: algebraic modular forms on a *definite* unitary group $U(k)$ (finite-set functions, easy to bound in norm and integrality).
- **Rows (`D.rows`, dimension $D = \mathrm{rowDim} = \mathrm{finrank}_{\mathbb{C}}\, D.\mathrm{rows}$)**: cusp forms ("good" level-$g$ forms, projected by `D.e₀`) on an *indefinite* unitary group $U(r)$, where $r + 1 = k + m$.
- **The chain of $m$ theta lifts (`D.Θ ϑ`)**: for each tame anticyclotomic character $\vartheta$ (`D.MFoldTwist`), `D.Θ ϑ` maps `D.rows` into forms on a definite unitary group $U(Y_m)$, with squared Gram determinant (`Data.𝔷 ϑ = ((D.Θ ϑ).normDet : ℂ)^2` in `New/Rows/RowVolume.lean:33`).

For any $n$ linearly independent rows $y : \mathrm{Fin}\, n \to D.\mathrm{rows}$, two numbers describe their untwisted lifts `D.Θ 1 ∘ y`:
1. **`V` (archimedean covolume bound):** $\mathrm{covolumeSq}(D.\Theta\, 1 \circ y, \sigma) \le e^V$ at every automorphism $\sigma \in \mathrm{Aut}(\mathbb{C})$;
2. **`δ` (p-adic denominator/index bound, `Data.IndexLe y δ` in `New/Endgame/ShortAndEnough.lean:130`):** at every prime $\ell$ and $p$-adic embedding $\iota$ above $\ell$, for every twist $\vartheta$ coprime to $\ell$, an $\mathcal{O}_\iota$-matrix $u$ with $|\det u|_\iota \ge e^{-e_\ell}$ ($\sum_\ell e_\ell \le \delta$) makes `D.Θ ϑ (u · y)` integral (`integralForms ι`).

Set the error unit `budget = f · a · (log b)^3` (`Data.budget`), `L = log b · budget = f · a · (log b)^4`, the main height **`h_main = m · k · 𝖴`** (`Data.mainHeight`, `New/Endgame/ShortAndEnough.lean:32`), and the prime-sum excess **`rem = Data.remainder`** (`New/Endgame/ShortAndEnough.lean:42`), which measures how far a weighted sum over non-split primes $\equiv 1 \pmod{N_0}$ exceeds its main term.

### What Theorems A, B, and C each bound

| Theorem | Lean name & file | Direction & inequality | Theory / mechanism |
|---|---|---|---|
| **A (short forms)** | `Data.short_forms` (`New/Endgame/ShortAndEnough.lean:138`) | **Upper bound** on norms & $p$-adic index: $\exists q \in \mathbb{Q}_{>0}$ and $B_s = q \cdot D.\mathrm{projectedPoincare}\, s$ such that (a) integral ramified columns $\varphi$ with $\|\Theta_1(B_s \varphi)\|^2_\sigma \le \exp(h_{\mathrm{main}} - m\cdot\mathrm{rem} + L)$ span `D.ramifiedColumns`, and (b) any finite family in $\bigcup_{s\in t} B_s(\mathrm{integralRamified})$ has `IndexLe` $\le \#t \cdot \dim(\mathrm{Column}) \cdot \mathrm{budget}$. | **Iterated theta seesaw** (`Data.seesaw`, `New/Seesaw/SeesawTheorem.lean`) transfers indefinite Poincaré lifts to finite coset sums of *definite* theta lifts; **Hecke/jet divisibility** at non-split primes (`New/Columns/LocalStatement.lean`, `weylNormSq_Bmu_le`, `exists_indexDvd_Bmu_divisible`) extracts a rational factor $q$ dividing out the prime-sum gains (`exists_rat_log_eq_and_padic_norm_eq`); the **level prime $N$** is chosen in a short window (`Data.heightPerRow_le`, `ShortAndEnough.lean:48`) so $\log N$ cancels the Stirling/Chowla–Selberg jet excess `m(w̄ + G)`, leaving `h_main - m·rem + budget`. |
| **B (enough forms)** | `Data.enough_forms` (`New/Endgame/ShortAndEnough.lean:173`) | **Spanning / dimension lower bound:** $\exists t \subset D.G$ with $\#t \cdot \dim(\mathrm{Column}) \le 5 D \log(1 + b^2)$ whose projected Poincaré images span dimension $n \ge (1 - 4/b^2) D$. | **Petersson / Bessel second-moment filling** (`Data.ramifiedColumns_fill_rows`, `New/Filling/FillingTheorem.lean:22`, invoking `Fits.exists_translate_finrank_comap_le` and `exists_few_translates`): averages over the finite level group $G = \prod_v \mathrm{GL}_r(\mathbb{F}_v)$ to show translates of ramified columns spread uniformly across `D.rows`, so $O(\log b)$ translates miss at most a $4/b^2$ fraction of dimensions. |
| **A + B combined** | `Data.exists_family_covolumeSq_le` (`New/Endgame/LongForms.lean:78`) | **Upper bound on covolume + index of an almost-basis:** $\exists n \ge (1 - 4/b^2)D$ independent rows $y$ with `IndexLe y δ`, $\delta \le 5 D \log(1+b^2)\,\mathrm{budget}$, and $\mathrm{covolumeSq}(\Theta_1 \circ y, \sigma) \le \exp(n(h_{\mathrm{main}} - m\cdot\mathrm{rem} + L))$. | Pick a basis $y$ from $\bigcup_{s\in t} B_s(S)$ (where $S$ is Theorem A's short spanning set and $t$ is Theorem B's small translate set) and apply Hadamard's inequality (`covolumeSq_le`). |
| **C (integral forms are long)** | `Data.integral_forms_long` (`New/Endgame/LongForms.lean:63`) | **Lower bound on the exact same covolume + index:** $D \cdot (h_{\mathrm{main}} - 3L) \le V + 2\delta$. | **Global product formula** (`HeightLe.nonneg`, `New/Heights/Heights.lean`) applied to the algebraic identity $(\mathfrak{z}(\theta_\iota)/\mathfrak{z}(1))^2 \cdot (\mathfrak{z}(1)^2 / \mathfrak{z}(\theta_\iota)^2) = 1$, where: (i) `exists_isFitFamily` (`New/Selmer/FitTwistsExist.lean:65`) picks anticyclotomic twists $\theta_\iota$ with tiny Selmer/congruence denominators (via Taylor–Wiles patching, Kisin modules, and Galois cohomology); (ii) `Data.volume_iota_eq_prod_liftVolume` (`New/Rows/RowVolume.lean`) factors $\mathfrak{z}(\vartheta)$ into $m$ single-lift volumes `liftVolume ϑ j`; (iii) `heightLe_liftVolume_ratio` (`New/Rows/LiftVolumeHeight.lean`) proves each of the $m$ lifts gains $D \cdot k \cdot \mathfrak{U} = D \cdot h_{\mathrm{main}} / m$ (`heightLe_volume_ratio_sq`, `LongForms.lean:19`); (iv) `heightLe_volume_rows` (`LongForms.lean:39`) completes the $(1 - 4/b^2)D$ rows $y$ to a full basis using `Data.crude_rows` (`New/Exchange/CrudeRows.lean`) and `one_determinant` (`New/Heights/OneDeterminant.lean`), bounding the height of $\mathfrak{z}(\theta_\iota)/\mathfrak{z}(1)$ by $V + 2\delta + D b$. |

### How `star_inequality` combines A, B, and C (`New/Main.lean:121-145`)

Let $h = h_{\mathrm{main}} - m\cdot\mathrm{rem} + L$ be Theorem A's per-row log-norm bound, so $V = n \cdot h$ for the $n \ge (1 - 4/b^2)D$ independent rows $y$ produced by `exists_family_covolumeSq_le` (A + B). Theorem C (`integral_forms_long`) gives:
$$D(h_{\mathrm{main}} - 3L) \;\le\; n h + 2\delta \;=\; D h + (D - n)(-h) + 2\delta.$$
Now substitute the two quantitative bounds from B and A:
- The missing-dimension share is $(D - n) \le (4/b^2) D$, and a crude bound (`abs_mainHeight_add_remainder_le`, `ShortAndEnough.lean:74`) gives $-h \le b^2 x^{99}$, so $(D - n)(-h) \le 4 D x^{99}$ (the $b^2$ cancels!).
- The $p$-adic index is $2\delta \le 10 D \log(1 + b^2)\,\mathrm{budget}$.

Therefore (`New/Main.lean:132-145`):
$$D(h_{\mathrm{main}} - 3L) \;\le\; D\bigl(h_{\mathrm{main}} - m\cdot\mathrm{rem} + L + 4x^{99} + 10\log(1+b^2)\,\mathrm{budget}\bigr).$$
Divide both sides by $D > 0$ (`finrank_rows_ne_zero`). **`h_main` cancels identically on both sides**, leaving:
$$m \cdot \mathrm{rem} \;\le\; 4L + 4x^{99} + 10\log(1+b^2)\,\mathrm{budget} \;\le\; (\log b)^2 \cdot \mathrm{budget}.$$
That is **`star_inequality` (`New/Main.lean:121`)**.

### Step-by-step: how a hypothetical zero with $\operatorname{Re}(\rho) > 1/2$ contradicts `star_inequality`

1. **Rewrite `rem` as a difference of prime-sum errors (`Data.remainder_eq`, `New/Endgame/ShortAndEnough.lean:86`):**
   Define the weighted non-split prime remainder at modulus $M$ and scale $y$ (`primeError`, `New/Endgame/PrimeError.lean`):
   $$R_M(y) \;=\; \frac{1}{y}\sum_{\substack{1 \le \nu \le y \\ \nu \equiv 1 \bmod M}} \Lambda(\nu)(1 - \chi_K(\nu))\!\left(\frac{y}{\nu} - 1\right) \;-\; \frac{\log y}{\varphi(M)} \;-\; c_M.$$
   Note the weights $\Lambda(\nu)(1 - \chi_K(\nu)) \ge 0$ (`lambdaClass_nonneg_and_abs_le`). `Data.remainder_eq` proves:
   $$\mathrm{rem} \;=\; a\Bigl(\#\mathrm{thick}\cdot\bigl(R_{N_0}(x) - R_{N_0}(x_1)\bigr) + \#\mathrm{tops}\cdot R_1(x_1)\Bigr).$$
   Rearranging for $R_{N_0}(x)$ and substituting `star_inequality` gives:
   $$R_{N_0}(x) \;\le\; \frac{(\log b)^2 \cdot \mathrm{budget}}{m \cdot a \cdot \#\mathrm{thick}} \;+\; \Bigl(R_{N_0}(x_1) - \frac{\#\mathrm{tops}}{\#\mathrm{thick}} R_1(x_1)\Bigr).$$
2. **Kill the second-scale term at $x_1$ (`eventually_primeError_le_rpow`, `New/Main.lean:149-167`):**
   Both $v \mapsto R_{N_0}(e^v)$ and $v \mapsto R_1(e^v)$ are `IsPolar` (`isPolar_primeError`, `New/Endgame/PrimeError.lean`): their Laplace transforms continue meromorphically with no pole in $\operatorname{Re}(s) \ge 1$. Lemma `IsPolar.exists_window_sub_mul_le` (`New/Endgame/OneSidedCriterion.lean:73`) proves by a positive-kernel moment argument (integrating against $v^k e^{-3v}$ with $k = \lceil t^2/9\rceil$) that in every long window $u = \log x_1 \in [t, t^2]$ ($t = 10^6 \log x$), there is a point $u$ where
   $$R_{N_0}(e^u) - \frac{\#\mathrm{tops}}{\#\mathrm{thick}} R_1(e^u) \;\le\; e^{-t} \;=\; x^{-10^6}.$$
   Meanwhile, with $\log b = x^\varepsilon$, $\#\mathrm{thick} \ge \#\mathrm{tops}/\log b$, $f = \varphi(N_0)\#\mathrm{tops}$, and $m \ge \sqrt{x}$, the first term is
   $$\frac{(\log b)^2 \cdot f a (\log b)^3}{m \cdot a \cdot \#\mathrm{thick}} \;\le\; \frac{\varphi(N_0)(\log b)^6}{\sqrt{x}} \;=\; \varphi(N_0)\,x^{6\varepsilon - 1/2}.$$
   Hence for every $0 < \varepsilon < 1/100$ and all sufficiently large $x$:
   $$R_{N_0}(x) \;\le\; x^{8\varepsilon - 1/2} \;=\; x^{\beta - 1}, \qquad \beta = \tfrac{1}{2} + 8\varepsilon.$$
   **Crucially, this bound is one-sided** (upper bound only, no lower bound on $R_{N_0}(x)$).
3. **Assume a zero $\rho$ of $L(s, \chi)$ with $\operatorname{Re}(\rho) > 1/2$ (`New.grh`, `New/Main.lean:174-190`):**
   Choose $\varepsilon = (\min(\operatorname{Re}\rho, 1) - 1/2)/51$, so $\varepsilon < 1/100$ and $\beta = 1/2 + 8\varepsilon < \operatorname{Re}\rho$. Pass to the induced character modulo $N_0 = 18480\,q$, so `entireL χ' ρ = 0`, and call `entireL_ne_zero_of_primeError_le` (`New/Endgame/OneSidedCriterion.lean:28`).
4. **Pick a quadratic twist $d$ that does NOT cancel the zero (`exists_entireL_quadTwist_ne_zero`, `Lib/Analytic/QuadraticTwist/ZeroFreeTwist.lean`):**
   In the Laplace transform of $R_{N_0}(e^v)$, the residue at $s = \rho$ is proportional to
   $$\frac{1}{\varphi(N_0)} \sum_{\psi \bmod N_0} \bigl(\mathrm{ord}_\rho L(s, \psi) - \mathrm{ord}_\rho L(s, \psi\cdot\chi_K)\bigr)$$
   (divided by $\rho(\rho-1)$). If a quadratic twist $L(s, \psi\cdot\chi_K)$ also vanished at $\rho$, its negative contribution could cancel $\mathrm{ord}_\rho L(s, \chi') > 0$. `exists_entireL_quadTwist_ne_zero` (proved via a zero-density estimate `zero_density_quadTwist` using Pólya–Vinogradov and the three-lines theorem `three_lines_bound_of_LFunction_eq_zero`) finds a prime $d \equiv N_0/2 - 1 \pmod{N_0}$ such that **no** twisted $L(s, \psi\cdot\chi_K)$ vanishes at $\rho$ (and none vanishes at any real $\sigma \in (\beta, 1]$). Thus the pole of the Laplace transform of $R_{N_0}$ at $\rho$ has **strictly nonzero residue**.
5. **Landau's lemma closes the contradiction (`IsPolar.landau`, `Lib/Analytic/Landau/PolarLandau.lean`):**
   Because $E(v) = R_{N_0}(e^v)$ satisfies the one-sided bound $E(v) \le e^{(\beta - 1)v}$ for large $v$, the non-negative function $e^{(\beta-1)v} - E(v) \ge 0$ has a Laplace transform that converges up to its rightmost real singularity (which lies at $\operatorname{Re}(s) \le \beta$ because real zeros in $(\beta, 1]$ were ruled out). By Landau's lemma (`IsPolar.landau`), no non-real pole can exist in $\operatorname{Re}(s) > \beta$ either — forcing the residue at $\rho$ to be $0$, contradicting step 4!
6. **Functional equation gives all zeros on $\operatorname{Re}(s) = 1/2$:** `New.zeros_on_line` and `New.zeta_zeros_on_line` (`New/Main.lean:194-205`), exported as the five theorems in `src/Main.lean`.

---

## 2. What the 200-fold rewrite dropped (vs the 21 November note)

Comparing `grh-compact/` (~76k LOC) against the 6 moves (A–F) and 12 levers recorded in [`../../2026-11-21-the-shape-of-the-rh-proof.md`](../../2026-11-21-the-shape-of-the-rh-proof.md) (from Chris Birkbeck's overview of the original ~17M-line development):

### 2.1 Heavy machinery from Birkbeck's item 5 — almost entirely dropped

| Infrastructure reported in the 21 Nov note (17M lines) | Status in `grh-compact/` (76k lines) | Evidence in `grh-compact/src/` |
|---|---|---|
| Resolution of singularities in char 0 | **Dropped** | 0 files (`rg -i "resolution of singularities"` = 0). |
| Arithmetic Hilbert–Samuel | **Dropped** | 0 files (`rg -i "hilbert.*samuel"` = 0). |
| Moriwaki's bigness theorem | **Dropped** | 0 files (`rg -i moriwaki` = 0). |
| Toroidal compactifications of Siegel varieties | **Dropped** | 0 files (`rg -i toroidal` = 0). |
| Kawamata–Viehweg vanishing & Nadel vanishing | **Dropped** | 0 files (`rg -i kawamata` = 0; `nadel` only occurs as a substring of `inAdelic`/`unAdelic`). |
| Grothendieck–Messing & prismatic Dieudonné theory | **Dropped / replaced** | 0 files for Dieudonné; only 1–2 bibliographic comments cite Messing/prismatic. Replaced by direct Kisin-module frames (`Lib/PadicHodge/WachKisin/FrameTheorem.lean`) and crystalline Frobenius comparison (`Lib/PadicComparison/CrystallineFrobenius/CrystallineCohomology.lean`, `ellAdicCohom.isCrystalline_and_linearFrob_rel`). |
| **Local Langlands for $\mathrm{GL}_n$** | **Explicitly dropped** | `Lib/GaloisRep/LocalGlobal/LabelCompatibility.lean:7`: *"It is stated as polynomials killing Weil or inertia elements, without local Langlands, and is reduced to the spherical case by cyclic base change."* `grh/GUIDE.md:30`: *"local polynomials only kill and need not be characteristic, and no Weil–Deligne representation is built."* |
| Pointwise PNT in progressions at $x_1$ (present in 300k `grh/`) | **Dropped in `grh-compact/`** | `New/Endgame/OneSidedCriterion.lean:5-7`: `IsPolar.exists_window_sub_mul_le` replaces a pointwise PNT in progressions when choosing the second scale $x_1$. |

### 2.2 The 21 November note's six moves (A–F) and twelve levers

| 21 Nov Move / Lever | Survived the 200× shortening? | How it appears (or changed) in `grh-compact/` |
|---|---|---|
| **Move A / Lever 1** (*Strengthen to GRH over cyclotomic families*) | **Survived intact** | Essential: `New.grh` needs both the cyclotomic CM field $E = K(\zeta_{N_1})$ (creating $f = \varphi(N_0)\#\mathrm{tops}$ infinite places) and the family of quadratic twists $K = \mathbb{Q}(\sqrt{-d})$ (`ZeroFreeTwist.lean`). |
| **Move B / Levers 2, 3, 4, 5, 6** (*Three-way squeeze, growth rates, inequalities only, two-faced invariant, amplify, positivity*) | **Survived, with crucial clarification** | **Survived:** amplification by $m \approx \sqrt{x}$ theta lifts (`New/Main.lean:94`); positivity of squared norms/Gram determinants (`covolumeSq`, `normDet^2`) and non-negative prime weights `lambdaClass ≥ 0`; pure inequalities (`HasLogBound`). **Clarified:** the invariant is NOT a Selmer group itself — it is the **log covolume + $p$-adic index `(V + 2δ)/D` of a single chain of theta lifts**, with Selmer/congruence bounds (`New/Selmer/FitTwistsExist.lean`) serving as an *ingredient inside Theorem C* to choose twists with small denominators, and **exact cancellation of `h_main`** (not merely comparing growth exponents) closing `star_inequality`. |
| **Move C / Lever 8** (*Local Langlands for $\mathrm{GL}_n$ / complete local classification*) | **Dropped / weakened** | Full local Langlands was **not** needed! Only split-place tame depth-zero labels via cyclic base change to spherical compatibility (`Lib/GaloisRep/LocalGlobal/LabelCompatibility.lean`, `Lib/GaloisRep/Main/SphericalAllRanks.lean:16` `sphericalCompat_std`), with Galois reps matching Hecke polynomials for `mostFrobenii` only (`grh/GUIDE.md:30`). |
| **Move D / Lever 7** (*Build Galois reps geometrically via Shimura varieties + theta lifts*) | **Survived, heavily streamlined** | Still built with zero axioms inside `Lib/Shimura/` and `Lib/Cohomology/PointClass/GaloisRep.lean:225` (`Model.exists_galois_rep`, using Woods Hole `woods_hole` + simple trace formula `Lib/Adelic/TraceFormula`), without toroidal compactifications or Siegel-variety Arakelov theory. |
| **Move E / Lever 9** (*No axioms; re-optimise route when formalisation cost changes*) | **Strongly confirmed** | `src/Lib/` is still **573 of 792 files** (72% of the compact proof). Re-optimising the route for minimal proof cost is literally how 17M lines collapsed 200× to 76k lines — every general textbook theorem was replaced by the narrowest special case the endgame actually calls. |
| **Move F / Levers 10, 11, 12** (*Cheap kernel check; toy dropped; trust decoupled from understanding*) | **Survived intact** | Still true: verified by Lean + Comparator (`grh-compact/README.md:30`), still unread by humans, toy remark absent. |

*Note on what we cannot verify from `grh-lean` alone:* `grh-lean` does not contain the original 17M-line repository, so for items absent from `grh-compact/` (Moriwaki, Hilbert–Samuel, toroidal compactifications, etc.) we can verify they are absent here (`0` files) and confirm what `grh-compact/` uses instead (`HeightLe.nonneg`, `Data.seesaw`, `Model.exists_galois_rep`, `LabelCompatibility.lean`), but we cannot audit line-by-line which parts of the original 17M-line development were genuinely load-bearing vs unused scaffolding.

---

## 3. Mapping onto mibz2's three bounds

We hold three bounds/measurements on mibz2's decoder wall $T_{\mathrm{wall}}(k)$:
1. **§70 lower bounds:** $L(k) = \max(S, \bar W, R) \le T_{\mathrm{wall}}(k)$ from work conservation (`character-stage1/analysis/lower_bound.tsv`: $T_{\mathrm{wall}}/L = 1.002\text{–}1.014$ for $k \le 6$, $1.041\text{–}1.048$ at $k = 8\text{–}10$).
2. **Ideal-parts measured rooms:** counterfactual walls when subsets of $\{S, M, C, E, R, D\}$ are replaced by oracles (`ideal-parts/results/full/README.md`: at 8w, `195 ms` baseline, `C` ideal `184 ms`, `S` ideal `189 ms`, `S+C` ideal `125 ms`; plus zero-copy floor `~5 ms` plumbing).
3. **Part 2's Network Calculus upper bounds:** `leads/pre-port-tests/README.md` (`M1` strict worst-case `U_NC / T_wall = 3.21–3.47×`, `M3a` Open-only strict `1.74–1.88×`, `M2` empirical window-closed `3.09–11.56×`, `M3b`/Panco non-strict corpus-mean open-loop `1.03–1.20×`).

### Move-by-move test for an exact counterpart

| Move in `grh-compact` (§1) | Proposed mibz2 role | Does an exact counterpart exist in our three bounds? | Tag |
|---|---|---|---|
| **Theorem A (`short_forms`)**: upper bound on a global covolume containing a *negative defect term* (`h_main − m·rem + L`) | Network Calculus upper bound $T_{\mathrm{wall}}(k) \le U_{\mathrm{NC}}(k)$ (Part 2) | **No.** $U_{\mathrm{NC}}(k)$ bounds wall time from above by `bottleneck_work + latency_slack` (positive slack, never `main − m·defect`). Moreover, Part 2 showed that strict worst-case $U_{\mathrm{NC}}$ is **3.21–3.47×** $T_{\mathrm{wall}}$ (and blind to $k = 8$), while tight open-loop $U_{\mathrm{fwd}}$ (`1.06–1.26×`) is non-strict and just re-adds the same measured resource demands as $L(k)$. | `[PROVED]` |
| **Theorem B (`enough_forms`)**: Petersson/Bessel spanning bridge turning per-vector norms into a $D$-dimensional covolume bound | Ideal-parts matrix (`ideal-parts/results/full/README.md`) | **No.** Ideal-parts measures counterfactual walls of one concrete decoder implementation under 32–64 arm configurations. There is no vector space of candidate traces or algorithms of dimension $D$ being spanned to assemble a determinant; measuring 32 arms gives a factorial decomposition of *current* code, not a spanning theorem over a design class. | `[PROVED]` |
| **Theorem C (`integral_forms_long`)**: product-formula lower bound (`D(h_main − 3L) ≤ V + 2δ`) matching Theorem A's exact leading term `h_main` | §70 work-conservation lower bound $L(k) = \max(S, \bar W, R) \le T_{\mathrm{wall}}(k)$ | **No (partial analogy only).** Both are proved lower bounds from a conservation law (product formula vs work conservation). But Theorem C's entire power is that its leading term `h_main` matches an *independent* upper bound's leading term `h_main` so their difference isolates a tiny defect `m·rem`. In mibz2, $L(k) \le T_{\mathrm{wall}}(k) \le U_{\mathrm{NC}}(k)$ merely brackets $T_{\mathrm{wall}}(k)$ between two functions of the *same* stage times; subtracting gives $0 \le U_{\mathrm{NC}} - L$ (scheduling/heterogeneity slack $\ge 0$), which says nothing about whether a faster algorithm exists. | `[ARGUED]` |
| **`star_inequality` + amplification ($m \approx \sqrt{x}$ lifts)**: `h_main` cancels identically, dividing the error by $m \approx \sqrt{x}$ | Composing §70 + ideal-parts + NC to squeeze out an algorithm defect | **No.** Neither our work-conservation lower bound nor NC has an amplification functor analogous to $m$-fold theta lifting (concatenating `wikidata ×4` scales work, latency, and wall linearly — it does not amplify a sub-leading structural defect by $\sqrt{x}$ relative to a fixed budget), and there is no common two-faced invariant whose main term cancels between an algebraic lower bound and an analytic upper bound. | `[ARGUED]` |
| **Zero exclusion (`exists_window_sub_mul_le` + `exists_entireL_quadTwist_ne_zero` + `IsPolar.landau`)**: one-sided bound + analytic continuation + positivity forbids poles in $\operatorname{Re}(s) > \beta$ | Excluding infeasible decoder designs (e.g. ruling out `T_wall < T_target`) | **No exact counterpart.** The only exclusion mibz2 actually uses is **trivial one-sided pruning** from §70 (`L(X) > T_target ⇒ design X cannot beat T_target`, e.g. any serial scanner with $S = 185\text{ ms}$ cannot reach $125\text{ ms}$ regardless of $k$ or worker speed). That is a single inequality, not a Landau-type pole exclusion from a one-sided bound. | `[PROVED]` |
| **What *would* be needed for a genuine squeeze in mibz2** *(outside our current three bounds)* | A **two-faced complexity invariant** (e.g. cell-probe / cache-miss lower bound in terms of BWT run count $r$ or attractor size $\gamma$, matching a constructive upper bound with the same leading constant) | `[CONJECTURED]` — this is the 21 November note's Lever 4 (and MONODROMY's referee-class SAT lower bound in §72). We do **not** currently hold either a non-trivial cache-miss lower bound matching Open's leading term or a computable upper bound over a design family; our current three bounds operate on fixed-stage durations of one pipeline, not on algorithm-independent complexity invariants. | `[CONJECTURED]` |

---

## 4. Application of the frozen stop test

The frozen stop test (`rauzy-census/prompt.md` part 3, item 4) states:

> *If no move in (1) maps onto a quantity we measure or can compute, say so plainly. The thread then ends as this note. Otherwise, name the one experiment or derivation it suggests and what it would cost.*

**Result: STOP — thread 7 ends as this note.**

- None of the five load-bearing moves of `grh-compact`'s squeeze (`short_forms`, `enough_forms`, `integral_forms_long`, exact `h_main` cancellation in `star_inequality`, and Landau's lemma in `entireL_ne_zero_of_primeError_le`) maps onto a quantity that §70, `ideal-parts`, or Network Calculus measures or computes.
- No further reading or derivation on the GRH proof is warranted for mibz2. Combined with Parts 1 and 2 (**STOP** on thread 2 / Rauzy orders; **KILL** on thread 6 / porting NC, uiCA, and `surface_dynamics`), all three offline characterisation and reading threads close cleanly, leaving engineering focus squarely on the measured rooms in `plan.md` §2 threads **3** (pooling the scan, `S + C` at 8w), **4** (`A5 + A6a` fold and `A6c` wiring), and **5** (forward emission lanes `E`).
