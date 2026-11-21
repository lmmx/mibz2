# Carbon Kitchen Sink — Cycle 7: Decoding Without an Orbit Walk? Burnside–BEST De Bruijn Condensation, Dyadic Stream-Merge Exponentiation $\mathrm{LF}^{2^j}$, and Fourier Cauchy-Like Resolvents

**Date:** 2026-11-20
**Branch:** `research/carbon-kitchen-sink`
**Inputs:** [`2026-11-20-carbon-kitchen-sink-cycle6.md`](2026-11-20-carbon-kitchen-sink-cycle6.md) (`0dc670f`), `docs/team/research-charter/FRONTIER.md` (Foundational Questions 1 & 3).
**Artefacts:** [`probe_cycle7.py`](2026-11-20-carbon-kitchen-sink-cycle7/probe_cycle7.py), [`probe_cycle7_results.json`](2026-11-20-carbon-kitchen-sink-cycle7/probe_cycle7_results.json).

---

## 0. Motivation: Three Collective Alternatives to Sequential Orbit Traversal

Every known exact BWT decoder (`Cells`, `Open`, `BSO`, `RV+SLP`) reconstructs $T$ by following a sequential orbit (either on $[0, n)$ or on a reduced critical set $V_{\mathrm{crit}}$). Cycle 6 proved that global placement potentials $\tau : [0, n) \to \mathbb{Z}/n\mathbb{Z}$ shatter into $\ge 0.986n$ affine pieces on real blocks. Cycle 7 therefore asks a more radical question: **can we obtain $T$ without walking an orbit at all, by replacing pointer traversal with collective graph condensation, pointer-free interval-exchange squaring, or Fourier/NTT resolvent evaluation?**

We formulate, prove, and confront three exact non-walk mathematical representations:
1. **Spectral Graph Theory & Combinatorics on Words:** Exact Eulerian De Bruijn condensation of $(k+1)$-mer spectra via a new **Burnside–BEST Branching-Unitig Identity**, pinpointing the exact Kotzig phase-transition scale $k^*(T)$ on all 6 Wikidata blocks.
2. **Interval-Exchange Semigroup Doubling:** Computing $\mathrm{LF}, \mathrm{LF}^2, \mathrm{LF}^4, \dots, \mathrm{LF}^{2^j}$ via **pure sequential two-pointer stream merges** (`0` random pointer chases), measuring exact dyadic piece growth $r(\mathrm{LF}^{2^j})$ across $j = 0 \dots 19$.
3. **Harmonic Analysis & Structured Operator Resolvents:** Transforming $P_{\mathrm{LF}}$ into Discrete Fourier / NTT space $\widehat{P}_{\mathrm{LF}} = \mathcal{F} P_{\mathrm{LF}} \mathcal{F}^{-1}$ to evaluate the generating polynomial $F_T(z) = \mathbf{e}_p^\top (I - z N_p)^{-1}\mathbf{L}$ collectively at roots of unity.

---

## 1. The Burnside–BEST De Bruijn Identity & Kotzig Condensation Phase Transition

Because row $x$ of a BWT starts with $F[x] = T_{(\tau(x)+1)\bmod n}$ and ends with $L[x] = T_{\tau(x)}$, a single $O(r)$ scan of $L$ against the $\alpha$ byte buckets of $F$ yields — with **zero** queues, arrays, or walks — the exact forward digram multigraph $G_1(T)$ on $\alpha \le 96$ vertices. More generally, let $G_k(T) = (V_k, E_k)$ be the cyclic order-$k$ De Bruijn multigraph of $T$, with out-degrees $d^+(u) = \mathrm{count}_T(u)$ and parallel edge multiplicities $m_k(u, uc) = \mathrm{count}_T(uc)$, and let $\mathcal{W}(G_k(T))$ be the set of distinct cyclic necklaces $[T']$ sharing $T$'s $(k+1)$-mer spectrum.

### Theorem 7.1 (Burnside–BEST Identity, Non-Branching Unitig Invariance & Kotzig Threshold — *proved*; *verified* on `1,170` small spectrum classes + 6 Wikidata blocks)

1. **Exact Burnside–BEST Multiplicity Formula:** Let $L'_k(G_k)$ be any $(|V_k|-1)\times(|V_k|-1)$ cofactor of the directed Laplacian $L_k = \mathrm{diag}(d^+) - A_k$. Permuting parallel copies of each $(k+1)$-mer under $H = \prod_{(u,v)} S_{m_k(u,v)}$ acts on unrooted labeled Eulerian circuits with stabilizer order $|\mathrm{Stab}_H(C)| = n / \mathrm{per}(T')$, yielding
   $$Z_k(T) \;:=\; \sum_{[T'] \in \mathcal{W}(G_k(T))} \frac{\mathrm{per}(T')}{n} \;=\; \frac{\det L'_k(G_k) \cdot \prod_{u \in V_k} (d^+(u) - 1)!}{\prod_{u \in V_k,\, c \in \Sigma} m_k(u, uc)!}.$$
   When $T$ is primitive ($\mathrm{per}(T) = n$), $Z_k(T) \ge 1$ and **$G_k(T)$ condenses to $T$ uniquely ($\mathcal{W}(G_k(T)) = \{[T]\}$) if and only if $Z_k(T) = 1$.**

2. **Exact Reduction to Right-Branching $k$-Mers ($V_k^{\mathrm{br}}$):** Let $V_k^{\mathrm{br}} = \{u \in V_k : |\{c : m_k(u, uc) > 0\}| \ge 2\}$ be the right-branching $k$-mers (detected in linear time on the cyclic LCP array as `[a, b)` intervals of $\mathrm{LCP} \ge k$ containing at least one $\mathrm{LCP}[y] = k$). Every non-branching vertex $w \notin V_k^{\mathrm{br}}$ has $m_k(w, wc) = d^+(w)$, contributing $(d^+(w)-1)!/d^+(w)! = 1/d^+(w)$ to the factorial product and pivot $+d^+(w)$ to Gaussian elimination of $L'_k$. These two factors cancel identically ($\frac{1}{d^+(w)} \cdot d^+(w) = 1$), reducing $Z_k(T)$ **without approximation** to the condensed multigraph $G_k^{\mathrm{br}}$ on $V_k^{\mathrm{br}}$ alone:
   $$Z_k(T) \;=\; \det L'^{\mathrm{br}}(G_k^{\mathrm{br}}) \cdot \prod_{u \in V_k^{\mathrm{br}}} \frac{(d^+(u) - 1)!}{\prod_{c \in \Sigma} m_k(u, uc)!} \qquad (V_k^{\mathrm{br}} \ne \emptyset),$$
   and $Z_k(T) = |V_k|/n = \mathrm{per}(T)/n$ when $V_k^{\mathrm{br}} = \emptyset$.

3. **Sharp Kotzig Phase Transition ($k^*(T) \le L_{\max}(T) + 1$):** Since $\mathcal{W}(G_{k+1}(T)) \subseteq \mathcal{W}(G_k(T))$, $Z_k(T)$ is monotonically non-increasing in $k$, defining a unique critical integer $k^*(T) = \min\{k \ge 1 : Z_k(T) = 1\}$. Notice that $k^*(T)$ can be **strictly smaller** than the repeat-free threshold $L_{\max}(T) + 1$: a 2-petal figure-8 repeat $u \to A \to u \to B \to u$ ($d^+(u) = 2, m = 1, 1$) has $(2-1)!/(1!1!) = 1$ unique cyclic tour, whereas either a **Kotzig interleaved pair** ($\dots u \dots v \dots u \dots v \dots$) or a **3-petal cloverleaf** ($d^+(u) = 3, m = 1, 1, 1 \implies (3-1)! = 2$) forces $Z_k(T) \ge 2$.

### Exact Eulerian Ambiguity & $k^*(T)$ Census on the 6 Wikidata Blocks (`probe_cycle7.py`)

| Block | $\log_2 Z_1$ (`k=1` free digrams) | $\log_2 Z_{16}$ | $\log_2 Z_{64}$ | $\log_2 Z_{512}$ | Exact Kotzig $k^*(T)$ | $\|V_{k^*}^{\mathrm{br}}\|$ at $k^*$ | $\log_2 Z_{k^*-1}$ | Repeat-free $L_{\max}+1$ | Gap $(L_{\max}+1) - k^*$ |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| `block-0.L` | **2,634,132.7 b** | 66,313.5 b | 20,598.1 b | 2,182.3 b | **4,744** | 1 | 1.0000 b (`Z=2`) | 4,822 | **78 B** |
| `block-2.L` | **2,692,981.3 b** | *74,612 b*\* | 19,993.5 b | 1,664.2 b | **10,526** | 1 | 1.0000 b (`Z=2`) | 10,528 | **2 B** |
| `block-85.L` | **2,594,578.3 b** | *109,595 b*\* | 25,903.0 b | 190.4 b | **925** | 2 | 2.0000 b (`Z=4`) | 926 | **1 B** |
| `block-143.L` | **2,807,938.9 b** | *128,899 b*\* | 35,342.4 b | 3,174.1 b | **9,482** | 3 | 1.0000 b (`Z=2`) | 17,638 | **8,156 B** |
| `block-210.L` | **2,737,478.9 b** | *141,089 b*\* | 34,904.6 b | 1,667.2 b | **4,345** | 2 | 1.0000 b (`Z=2`) | 6,939 | **2,594 B** |
| `block-207.L` | **2,783,474.1 b** | *91,203 b*\* | 22,520.9 b | 1,648.6 b | **4,360** | 2 | 2.0000 b (`Z=4`) | 4,363 | **3 B** |

*(\* Starred $k=16$ entries show exact $H_{\mathrm{loc}}(G_{16})$ where $|V_{16}^{\mathrm{br}}| > 1,600$; across all measured points, the Kirchoff tree deficit $H_{\mathrm{loc}} - \log_2 Z_k$ is only `12–135` bits!)*

**Mathematical & Algorithmic Verdict on De Bruijn Condensation:**
- Theorem 7.1(3) is witnessed cleanly in the wild: on `block-143.L`, unique global Eulerian condensation happens at **$k^* = 9,482$ — `8,156` bytes earlier than $L_{\max} + 1 = 17,638$** (`3` non-interleaved figure-8 repeats of lengths up to `17,637` commute cyclically with $Z = 1$!), and on `block-0.L` at $k = 4,743$ a single 3-petal cloverleaf (`V_br = 1, H_loc = log2(6) = 2.585`, $Z = 2$) splits into a 2-petal figure-8 ($Z = 1$) at $k^* = 4,744$.
- Yet as a non-walk BWT decoder, De Bruijn condensation hits a decisive information-theoretic wall: the $O(r)$-free digram graph $G_1(T)$ leaves **`2.59–2.81 million` bits (`2.88–3.12 bits/byte`) of Eulerian tour ambiguity**, and reaching $Z_k = 1$ requires extracting $(k^*+1)$-mers of length **$k^* \in [925,\, 10{,}526]$** from $(L, p)$, which costs far more than a single $m$-step walk.

---

## 2. Dyadic Interval-Exchange Exponentiation $\mathrm{LF}^{2^j}$ via Pointer-Free Stream Merges

Can we replace sequential pointer chasing (`x = lf[x]`) with **collective binary exponentiation** $\mathrm{LF}, \mathrm{LF}^2, \mathrm{LF}^4, \dots, \mathrm{LF}^{2^{19}}$ using only sequential stream merges?

### Theorem 7.2 (Pointer-Free Stream-Merge Squaring & Time-Reversal Symmetry — *proved*; *verified* on `230` primitive BWTs + Tribonacci + 6 Wikidata blocks)

Let $f, g : [0, n) \to [0, n)$ be piecewise unit-slope bijections with $r(f)$ and $r(g)$ maximal affine intervals $(d_i, w_i, c_i)$.
1. Sorting $f$'s intervals by codomain start $c_i$ yields a partition of $[0, n)$ whose linear two-pointer stream merge against $g$'s domain-sorted intervals computes $g \circ f$ with **zero random array accesses** in $O(r(f) + r(g))$ merge steps plus $O(r(g\circ f))$ radix-sort work, satisfying
   $$r(g \circ f) \le r(f) + r(g) - 1, \qquad r(\mathrm{LF}^{2^{j+1}}) \le 2\,r(\mathrm{LF}^{2^j}) - 1.$$
2. Because $\mathrm{LF}^n = \mathrm{id}_{[0,n)}$ on any primitive BWT and $h^{-1}$ has the exact same number of maximal unit-slope intervals as $h$ (exchanging domain and codomain intervals), piece complexity obeys exact arch symmetry:
   $$r(\mathrm{LF}^N) = r\!\left((\mathrm{LF}^N)^{-1}\right) = r(\mathrm{LF}^{n - N}) \qquad \forall N \in [0, n].$$

### Exact Dyadic Piece-Complexity Curve $r(\mathrm{LF}^{2^j})$ (`probe_cycle7.py`)

| Input | $n$ | $j=0$ ($r$) | $j=1$ ($\mathrm{LF}^2$) | $j=2$ ($\mathrm{LF}^4$) | $j=4$ ($\mathrm{LF}^{16}$) | $j=6$ ($\mathrm{LF}^{64}$) | $j=8$ ($\mathrm{LF}^{256}$) | $j=10$ ($\mathrm{LF}^{1024}$) | $j=16$ ($\mathrm{LF}^{65536}$) | $\sum_{j=0}^{19} r(\mathrm{LF}^{2^j})$ |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| **Sturmian ($r=2$)** | $n$ | **2** | **2** | **2** | **2** | **2** | **2** | **2** | **2** | **$40 = O(\log n)$** |
| **Tribonacci Rauzy** | 1,705 | 17 | 30 | 54 | 171 | 508 | 1,271 | **1,705** (`1.00n`) | — | saturates at $j=9$ |
| `block-0.L` | 899,931 | 51,184 | 62,136 (`1.21×`) | 76,187 (`1.23×`) | 131,417 | 289,617 | 585,885 (`0.65n`) | 850,365 (`0.94n`) | 899,034 (`0.999n`) | **12.87 $n$** |
| `block-2.L` | 900,000 | 54,532 | 67,755 (`1.24×`) | 85,160 (`1.26×`) | 154,557 | 321,134 | 616,690 (`0.69n`) | 832,389 (`0.92n`) | 899,909 (`1.000n`) | **13.02 $n$** |
| `block-85.L` | 899,935 | 79,375 | 96,492 (`1.22×`) | 117,406 (`1.22×`) | 205,406 | 436,053 | 736,207 (`0.82n`) | 886,108 (`0.98n`) | 899,872 (`1.000n`) | **13.86 $n$** |
| `block-143.L` | 899,869 | 108,335 | 135,299 (`1.25×`) | 170,773 (`1.26×`) | 296,102 | 476,278 | 653,364 (`0.73n`) | 802,692 (`0.89n`) | 899,264 (`0.999n`) | **13.91 $n$** |
| `block-210.L` | 899,959 | 128,732 | 155,477 (`1.21×`) | 187,480 (`1.21×`) | 309,480 | 523,604 | 716,830 (`0.80n`) | 864,458 (`0.96n`) | 899,491 (`0.999n`) | **14.56 $n$** |
| `block-207.L` | 899,998 | 87,111 | 107,914 (`1.24×`) | 134,064 (`1.24×`) | 233,759 | 384,380 | 583,710 (`0.65n`) | 805,122 (`0.89n`) | 898,928 (`0.999n`) | **13.35 $n$** |

**Two Structural Lessons:**
1. **Shallow Doubling Coherence ($j = 1, 2$):** In the first two doublings, $r(\mathrm{LF}^2)/r(\mathrm{LF}) = \mathbf{1.21\text{--}1.25\times}$ (far below the `2.00×` worst case!), because `75–79%` of run boundaries flow through pass-through JSON corridors without splitting.
2. **Katok–Zemljakov Linear Growth Kills Full Binary Exponentiation ($j \ge 8$):** Even on the idealized 2D Tribonacci Rauzy fractal ($r_0 = 17, n = 1,705$), a $\ge 3$-interval exchange grows linearly as $r(T^N) = \Theta(N)$, so dyadic powers $N = 2^j$ grow exponentially in $j$ and saturate at $1.00n$ by $j = 9$. On all 6 Wikidata blocks, $r(\mathrm{LF}^{2^j})$ crosses `0.89–0.98n` by $j = 10$ and totals **`12.87n–14.56n` interval records** across $j = 0 \dots 19$ (`~11.6M–13.1M` 12-byte records + radix sorts $\gg$ `Open`'s single $0.10\text{--}0.23n$ walk).

---

## 3. Fourier-Space Cauchy-Like Resolvents $\widehat{P}_{\mathrm{LF}}$ & Zero-Defect Clustering

Representing $T$ by its generating polynomial $F_T(z) = \sum_{s=0}^{n-1} T_{n-1-s} z^s = \mathbf{e}_p^\top (I - z N_p)^{-1}\mathbf{L} \in \mathbb{F}_q[z]$ reduces emission to multipoint evaluation at $n$-th roots of unity $\zeta^k$ plus one `IFFT`/`INTT`. Conjugating $P_{\mathrm{LF}}$ by the Discrete Fourier Transform $(\mathcal{F}_{j,x} = \zeta^{jx})$ reveals its exact operator type:

### Theorem 7.3 (Fourier Cauchy-Like Identity & Mantaci–Restivo–Sciortino Zero-Defect Criterion — *proved*; *verified* via exact DFT on `230` primitive BWTs + 6 Wikidata blocks)

1. **Exact Fourier Cauchy-Like Form:** Let $\Omega = \mathrm{diag}(1, \zeta, \dots, \zeta^{n-1})$ with $\zeta = e^{2\pi i / n}$. Then $\widehat{P}_{\mathrm{LF}} = \mathcal{F} P_{\mathrm{LF}} \mathcal{F}^{-1}$ satisfies the exact Sylvester/Cauchy-like displacement equation
   $$(1 - \zeta^{j - k})\,(\widehat{P}_{\mathrm{LF}})_{j, k} \;=\; \frac{1}{n} \sum_{u=0}^{r-1} \zeta^{j d_u}\!\left(\zeta^{-k c_u} - \zeta^{-k(\mathrm{LF}((d_u - 1)\bmod n) + 1)}\right) \qquad \forall j, k \in [0, n),$$
   expressing $\widehat{P}_{\mathrm{LF}} - \Omega \widehat{P}_{\mathrm{LF}} \Omega^{-1}$ as a sum of at most $r$ rank-1 outer products (rank $\le r - 1$).
2. **Zero-Defect Criterion:** The $u$-th Cauchy generator vanishes ($c_u \equiv \mathrm{LF}((d_u - 1)\bmod n) + 1 \pmod n$) **if and only if** $(d_u - 1, d_u)$ is a **perfectly clustered cyclic-alphabet transition**: $d_u - 1$ is the final occurrence of byte $b_{u-1}$ in $L$, $b_u$ is the cyclically next present byte in $\Sigma$, and $d_u$ is the first occurrence of $b_u$. In particular, $\widehat{P}_{\mathrm{LF}}$ is diagonal in Fourier space (making $(I - z \widehat{P}_{\mathrm{LF}})^{-1}$ invertible in a single FFT!) **if and only if** $\mathrm{LF}$ is a pure cyclic rotation ($r = 1$ or primitive Sturmian $r = 2$ or shifted $F$).
3. **Zero Cancellations on Real Blocks:** Across all **6/6 Wikidata blocks**, `active_cauchy_generators == r` identically (`51,184/51,184`, `54,532/54,532`, `79,375/79,375`, `108,335/108,335`, `128,732/128,732`, `87,111/87,111` — **`0` cancellations out of `509,269` run heads!**). Because superfast Cauchy-like linear solvers require $\tilde{\Omega}(r_{\mathrm{disp}}^{\omega - 1} n) \ge \Omega(r n) \approx 5 \times 10^{10}$ operations at displacement rank $r \ge 51{,}184$, direct Fourier/NTT resolvent inversion cannot compete with $O(n)$ time.

---

## 4. Bridge to Cycle 8: Changing the Problem via Topological Skew-Products

Cycles 6 and 7 reveal a single unifying root cause behind every obstruction: **random hex/digit fibers (`[0-9a-f]`) embedded inside a deterministic JSON base shatter interval continuity across $[0, n)$**, forcing $r = 51\text{K}..129\text{K}$, $r(\Delta\boldsymbol{\tau}) \approx n$, $r(\mathrm{LF}^{2^{10}}) \approx n$, and $r_{\mathrm{Cauchy}} = r$.

In **Cycle 8**, we attack that root cause directly via **Topological Skew-Product / Fiber-Bundle Decomposition ($\pi : \Sigma \twoheadrightarrow \overline{\Sigma}$)** and **Ihara–Bowen–Lanford Spectral Zeta Determinants**:
1. What happens when we factor $T$ and $L$ into a **coarse syntactic base** $\overline{\Sigma}$ (collapsing `[0-9a-f] -> HEX`) plus a **4-bit nibble fiber stream** $\boldsymbol{\xi}$ — comparing **post-BWT column quotienting $\pi(L)$** (4-bit wavelet-radix fiber transport) against **pre-BWT topological factor splitting $\mathrm{BWT}(\pi(T)) \times \boldsymbol{\xi}_{\mathrm{text}}$** (`Foundational Question 4: "Is the inverse BWT the right problem?"`)?
