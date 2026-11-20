# Carbon Cycle 4 · Toy-to-Hard Mathematical Transfer: Deconstructing Coded Rotations, Origami Abel–Jacobi Cohomology, Veech Groups, the De Bruijn High-Entropy Toy, and Iterated Monoid Lifting (`GS-IMD` & `BJ-IMD`)

*20 November 2026 · Carbon (`research/carbon-word-transport`, Cycle 4).*

## 0. Cycle-Boundary Check (`origin/master` Fetch)

At the Cycle 3 → Cycle 4 boundary, fetching `origin/master` (`3e427996`) surfaced:
1. `METHOD.md` §1 ("Mathematical maximalism", `da0482b`) and [`2026-11-20-kitchen-sink.md`](https://github.com/lmmx/mibz2/commit/3e42799): authorising unrestricted mathematical breadth on the way in paired with strict obligation-register and native-cost confrontation on the way out, highlighting **Rauzy fractals** (Priority #1), **Veech groups** on origamis (Priority #2), **Krohn–Rhodes semigroup decomposition**, **numeration systems**, and **random permutation structure**.
2. Louis's foundational provocation on **Toy-to-Hard Mathematical Transfer** (motivated by the discovery architecture reported in Levent Alpöge's unverified RH/GRH note, [`2026-11-20-research-by-obligation.md`](https://github.com/lmmx/mibz2/commit/555958b), and [`2026-11-20-toy-transfer.md`](https://github.com/lmmx/mibz2/commit/ad48e6a)): isolate what makes an unusually strong reconstruction capability work in a tractable family, deliberately break the assumptions until it fails, challenge the choice of toy problem itself if needed, and test whether a native surrogate — particularly **amplification by iterated composition** (an analogue of iterated lifts standing in for multiplicity) — makes exact inverse-BWT reconstruction cheaper.

*(Note: `3e427996` also records the separate `research/carbon-kitchen-sink` launch; per its boundary rule we remain strictly on `research/carbon-word-transport`.)*

---

## 1. Explore: Five Mathematical Worlds for Toy-to-Hard Transfer

Mode: **Reorientation** (`METHOD.md` §1 & §3a). Rather than taking mechanical words (`x + kα mod 1`) uncritically as the right toy, we investigate five distinct mathematical frameworks:

| # | World / Candidate | Mathematical machinery | Proposed reconstruction capability | Likely mathematical or economic trap |
|---|---|---|---|---|
| 1 | **`Coded-Rotation & Rauzy-Fractal Coordinates`** (`toy.py`; `kitchen-sink.md` #1) | Compact abelian / Kronecker dynamics ($S^1$, Pisot torus $\mathbb{T}^{d-1}$) & Ostrowski numeration | Direct $O(1)$ / $O(\text{depth})$ global coordinate lookup `text[k] = Π(x_0 + kβ)` without orbit walking | `toy.py` inflated alphabet $\sigma = p$; torus codings have zero topological entropy ($h_{\text{top}} = 0$), violating the $122 M$-bit entropy bound of unique GUIDs |
| 2 | **`Origami Abel–Jacobi Cohomology`** (`H^1(O, Z/nZ)`) | Discrete exterior calculus & period integration on the square-tiled surface $O = (\mathbb{Z}/n\mathbb{Z}, r, u)$ | Compute $\theta(x) = \mathrm{SA}[x]$ by sequential vertical prefix sum ($Q = 0$) of the vertical 1-form $\Delta = d_u \theta$ | Away from genus $1$, painting $\Delta$ from the $r$ singular cone boundaries along horizontal saddle connections may dualise back to $Q = n$ arc walking |
| 3 | **`Veech-Group Affine Transport`** (`kitchen-sink.md` #2; Schmithüsen 2004) | Stabiliser $\Gamma(O) \le \mathrm{SL}(2, \mathbb{Z})$ & sheet conjugacies $\sigma (r, u) \sigma^{-1} = \Phi_M(r, u)$ | Conjugate $(r, u)$ into an affine frame on the same origami with simpler horizontal cylinders | Generic BWT origamis may have trivial parabolic Veech elements; even when $\sigma \in S_n$ exists, acquiring $\sigma$ from $I_0$ may cost a full $r$-walk |
| 4 | **`De Bruijn Shift-Registers & Krohn–Rhodes`** (`kitchen-sink.md` #3) | Order-$k$ de Bruijn graphs ($n = \sigma^k$, positive entropy, width-1 runs) & finite semigroup wreath products | Replace zero-entropy Sturmian toys with a high-entropy singleton-run toy having $O(1)$ rank-free `LF` and $k$-symbol coordinate readout | Sparse $k$-mer occupancy ($M \ll \sigma^k$) destroys uniform radix division $\lfloor x/\sigma \rfloor$; bijections have zero Krohn–Rhodes reset elements |
| 5 | **`IMD` (`GS-IMD` & `BJ-IMD`)** | Alpöge-style **iterated operator lifting** $(g^{-1}, W) \mapsto (\tilde{g}_L, \tilde{W}_L)$ on non-commutative word-carrying permutations | Replace multi-cursor lane arenas with 1-lane zero-arena operator squaring (`D = 1` hardware-MLP lift sweeps packing $\le 5\text{B}$ or $\le 12\text{B}$ inline words into `u64`/`u128` entries) | Extensional operator lifting issues $1.0m$ gather reads per lift and widens entries from `4 B` to `8–16 B`, risking cache-footprint regression |

---

## 2. Triage & Foundational Theorems (`verify_cycle4.py`)

Every theorem and counterexample below is machine-verified by [`verify_cycle4.py`](2026-11-20-carbon-cycle4/verify_cycle4.py).

### Result 2.1 · Deconstruction of `toy.py` (`94b4d7d`), Covering-Map Folding, and the Zero-Entropy Coordinate Barrier (`proved` + `verified`)

Inspecting `toy.py` on `origin/master` reveals a critical experimental confound in the "damaged toy" table (`p = 1,024 -> r = 1,036, mean width 4.8`), alongside two distinct mathematical phenomena when repaired to a fixed alphabet:

**Theorem 1.1 (Three Regimes of $p$-Piece Coded Rotations & Entropy Obstruction — `proved` + `verified`).**
Let $x_k = (x_0 + k\alpha) \bmod 1$ on $S^1 = \mathbb{R}/\mathbb{Z}$ be coded by $p$ equal intervals $I_j = [(j + c)/p, (j + 1 + c)/p)$:
1. *Alphabet-inflation artifact (`toy.py`, $\sigma = p$):* Assigning each interval a distinct symbol (`chr(0x100 + j)`) forces $r \ge \sigma = p$ trivially, while cyclic monotonicity of the coding ensures that infinite right-context lexicographic order coincides with circle order up to the wrap-around cut at $0$. At $n = 5,000, p = 1,024$, `toy.py`'s $r = 1,036$ has **excess run count $r - \sigma = 12$** (`98.8%` of symbols still have a single run!).
2. *Periodic binary folding (`alt`, $\sigma = 2$):* Coding interval $I_j$ by $j \bmod 2$ factors through the $(p/2)$-sheeted covering endomorphism $\pi(z) = (p/2)z \bmod 1$ onto a **2-piece Sturmian word** of slope $(p/2)\alpha \bmod 1$, keeping $r = O(1)$ (`r = 14` at $p = 1,024, n = 5,000$!).
3. *Aperiodic binary damage (`aperiodic`, $\sigma = 2$):* Coding $I_0, \dots, I_{p-1}$ by an aperiodic binary mask genuinely destroys run width on a 2-letter alphabet (`r = 2,014`, mean width `2.5` at $p = 1,024, n = 5,000$). Likewise, Pisot Rauzy-fractal substitutions (`kitchen-sink.md` Priority #1, e.g. Tribonacci $0 \mapsto 01, 1 \mapsto 02, 2 \mapsto 0$ on $\mathbb{T}^2$) have $r = O(\log n)$ (`r = 18` at $n = 3,136$).
4. *Zero-entropy coordinate barrier:* Every deterministic coding of a translation on $\mathbb{T}^d$ by $p$ semialgebraic/interval pieces has topological entropy $h_{\text{top}} = 0$ and block description length $O((p + d)\log n)$ bits. On any family $\mathcal{F}$ of length-$n$ texts with $\log_2 |\mathcal{F}| = H$ bits (such as $H = 122 M$ bits for $M$ v4 UUIDs), **no** data structure $\Theta$ of size $< H$ bits derived from $I_0$ can evaluate `text[0..n)`. $\square$

| Pieces $p$ ($n = 5,000$) | `toy.py` ($\sigma = p$): $r$ | Excess $r - \sigma$ | Alternating binary ($\sigma = 2$): $r$ | **Aperiodic binary ($\sigma = 2$): $r$ (`mean_w`)** |
|---:|---:|---:|---:|---:|
| 2 | 26 | 24 | 26 | **26 (`192.3`)** |
| 8 | 46 | 38 | 16 | **28 (`178.6`)** |
| 64 | 101 | 37 | 22 | **240 (`20.8`)** |
| 256 | 276 | 20 | 22 | **896 (`5.6`)** |
| 1,024 | 1,036 | **12** | **14** | **2,014 (`2.5`)** |

### Result 2.2 · Discrete Abel–Jacobi Cohomology on the Origami (`proved` + `verified` on `146` primitive small columns + `4` Christoffel slopes)

**Theorem 2.1 (Origami Abel–Jacobi Duality — `proved` + `verified`).**
On the origami $O = (\mathbb{Z}/n\mathbb{Z}, r = \mathrm{LF}, u = +1)$ of any single-cycle column $L$, text position $\theta(x) = \mathrm{SA}[x] \in \mathbb{Z}/n\mathbb{Z}$ ($\theta(p) = n - 1$) is the unique potential of the discrete 1-form $(\omega_r, \omega_u)$ with $\omega_r(x) = \theta(r(x)) - \theta(x) \equiv -1 \pmod n$ and $\omega_u(x) = \theta(u(x)) - \theta(x) = \Delta(x)$:
1. *Exact holonomy jump:* For every $x \in \mathbb{Z}/n\mathbb{Z}$, $\Delta(r(x)) - \Delta(x) \equiv \theta(u(r(x))) - \theta(r(u(x))) \pmod n$. In particular, at all $n - r$ flat squares ($u(r(x)) = r(u(x))$, i.e. $x + 1 \notin D$), $r^*\Delta(x) = \Delta(r(x)) = \Delta(x)$.
2. *Genus-1 Christoffel formula:* When $[r, u] = \mathrm{id}$ ($r(x) = (x + d) \bmod n$ with $\gcd(d, n) = 1$), $r^*\Delta = \Delta$ everywhere on the transitive $r$-cycle, so $\Delta(x) \equiv \Delta_0$ is constant; $d \cdot \Delta_0 \equiv \theta(x + d) - \theta(x) \equiv -1 \pmod n$ yields $\Delta_0 \equiv -d^{-1} \pmod n$ and the walk-free global coordinate $\theta(x) \equiv (n - 1) - d^{-1}(x - p) \pmod n$.
3. *Genus-$g$ saddle-connection duality:* For $r \ge 2$ singular squares, $\Delta$ is constant along each of the $r$ maximal horizontal persistence saddle connections from $\bar{C}$ to $\bar{D}$ (total length $n$). Although the $r$ singular periods $\{\Delta(d - 1)\}_{d \in \bar{D}}$ are computable in $O(r)$ steps (Weight-Only Rauzy–Veech, Record 81), reconstructing $\theta(x) = \theta(0) + \sum_{y < x} \Delta(y) \pmod n$ vertically ($Q = 0$) requires painting each saddle connection's period across its interior squares — which is strictly dual to horizontal LF walking (`arc_synchronous_walk`, Record 84: $W = n, Q = n$). $\square$

### Result 2.3 · Veech-Group Affine Transport (`kitchen-sink.md` Priority #2 — `proved` + `verified` on `6/6` blocks)

**Theorem 3.1 (Veech-Group Conjugacy Acquisition Trap & Parabolic Falsification — `proved` + `verified`).**
An element $M \in \mathrm{SL}(2, \mathbb{Z})$ belongs to the Veech group $\Gamma(O)$ iff $\Phi_M(r, u) = (g_1, g_2)$ is conjugate in $S_n$ to $(r, u)$ via some sheet bijection $\sigma \in S_n$ ($\sigma r = g_1 \sigma, \sigma u = g_2 \sigma$):
1. *Cycle-type falsification on Wikidata (`verified`, `verify_cycle4.py`):* On all 6 authenticated blocks, $r$ and $u$ are single $n$-cycles (`1` cycle), whereas $r u^{-2}, r u^{-1}, r u, r u^2$ and $u r^2$ split into **`13` to `71` disjoint cycles** (`(35,23,27,17), 29` on `block-0`; `(21,22,44,27), 65` on `block-2`; `(13,15,23,21), 19` on `block-85`; `(35,53,53,17), 43` on `block-143`; `(17,28,14,19), 25` on `block-207`; `(21,71,25,25), 47` on `block-210`). Hence $T^b, S T^2 S^{-1} \notin \Gamma(O)$ on `6/6` blocks.
2. *Acquisition trap:* Even when $M \in \Gamma(O)$ exists, if $g_2 = u^{\pm 1}$ then $\sigma(x) = (\pm x + c) \bmod n$ preserves $r_{\text{aff}}$ (Cycle 1 Theorem 1.1); if $\deg_r(g_2) \neq 0$, then determining $\sigma(x) = g_2^x(\sigma(0))$ from $I_0$ requires $n$ dependent evaluations of $g_2$ (hence $\ge n$ evaluations of $r$). $\square$

### Result 2.4 · Challenging the Toy: Order-$k$ De Bruijn Words & Absence of Krohn–Rhodes Resets (`proved` + `verified`)

Unlike Sturmian words ($h_{\text{top}} = 0$, wide runs), **order-$k$ de Bruijn cycles** ($n = \sigma^k$, $|\mathcal{B}_{\sigma, k}| = (\sigma!)^{\sigma^{k-1}} / \sigma^k$) have **positive entropy** $\Theta(n)$ bits and narrow runs ($w \le \sigma$, predominantly width $1$):

**Theorem 4.1 (De Bruijn Rank-Free LF, $k$-Symbol Coordinate Readout, and Damage Law — `proved` + `verified` across `19/19` binary de Bruijn cycles $k = 2, 3, 4$).**
1. *Exact toy capability:* On every $T \in \mathcal{B}_{\sigma, k}$ ($n = \sigma^k$), sorted rotations equal $\Sigma^k$ in exact radix order, so (a) $\mathrm{LF}(x) = c(L[x]) \cdot \sigma^{k-1} + \lfloor x / \sigma \rfloor$ holds with **zero** `C` or `rank` tables, and (b) the base-$\sigma$ digits of row index $x$ directly output the $k$-byte forward substring $T[\mathrm{SA}[x] .. \mathrm{SA}[x] + k)$ in $O(1)$ with zero loads.
2. *Damage to sparse random identifiers:* Restricting $\Sigma^k$ to a sparse subset of $M \ll \sigma^k$ active $k$-mers (e.g., $M \sim 10^3$ UUIDs in $16^{32}$) breaks uniform bucket occupancy from $1$ to $\{0, 1\}$, forcing $\lfloor x / \sigma \rfloor$ back to prefix-sum rank $\operatorname{rank}_c(L, x)$ and shrinking free forward digit readout from $k$ symbols to $\lfloor \log_\sigma M \rfloor \approx 1\text{--}2$ symbols.
3. *Krohn–Rhodes reset obstruction (`kitchen-sink.md` #3):* Because $\mathrm{LF} \in S_n$ and Open's $g^{-1} \in S_m$ are bijections, every partial transition is injective (`defect = 0`); their transformation monoids contain **zero** constant/reset maps, ruling out $O(1)$-lookahead state synchronisation without explicit cycle cuts or $O(m)$ operator composition. $\square$

---

## 3. Deepen: Inline-Payload Iterated Monoid Lifting (`GS-IMD` & `BJ-IMD`)

This brings us directly to the computational form of **Alpöge's multiplicity move**: if real blocks only give 1-step state lookahead (`multiplicity 1`), can **iterated composition sweeps** $(g^{-1}, W) \mapsto (\tilde{g}_L, \tilde{W}_L)$ at `D = 1` per sweep act as a native surrogate for $k$-step De Bruijn readout in a pure 1-lane, zero-arena decoder?

### Mandatory `METHOD.md` §3a Candidate Declaration
- **Mode:** Reorientation (replacing serial 1-step pointer traversal — and avoiding `BVSMC`'s multi-cursor chunk arena — by lifting the operator $g^{-1}$ itself in data-independent sweeps with inline multi-byte word packing; classification deferred to the Coordinator and MONODROMY).
- **Displaced obligation:** Obligation #1 (`m` serial dependent pointer jumps in 1-lane `Open::emit`, cut to `0.23–0.51m` without any chunk pool or segment stitch).
- **Replacement principle (`Theorem 5.1 — Guarded Variable-Stride Monoid Lifting`, `proved`):**
  Pack `next_j` (`20` bits), inline byte length `tag` (`4` bits: `1..B_max`, or `0` for `> B_max` `spelling` fallback), and up to $B_{\max}$ raw ASCII bytes (`5 B` in `uint64_t`; `12 B` in `16 B`-aligned `Slot16`) directly inside each edge entry during `Wire`. In each lift sweep $j = 0 \dots m-1$, compose `edge[j]` with `edge[j1]` (`j1 = next(j)`) whenever `1 <= tag(j) < B_max`, `1 <= tag(j1) <= B_max - tag(j)`, `j1 != mark`, and `j1 != j`:
  - **Branchless Jacobi (`BJ64-1P`):** Unconditional load + `cmovne` into a ping-pong buffer (`D = 1`, full hardware OoO `MLP`).
  - **In-Place Gauss–Seidel (`GS64-1P`, `GS128-2P`):** Mutates `edge[0..m]` in place (`0 B` ping-pong buffer); whenever $j_1 < j$ (`~50%` of edges), `edge[j1]` already holds a lifted multi-byte word, compounding strides within a single pass while preserving the exact non-overshoot invariant (`mark` is never interior to any lifted edge, so short-period and multi-cycle inputs remain byte-exact).
- **New paid obligations:** Widening operator entries from `4 B` (`u32`) to `8 B` (`u64`) or `16 B` (`Slot16`), plus $m$ gather reads per lift sweep.
- **Nearest prior art:** Wyllie (1979) pointer jumping; Cycle 2 `BVSMC` (which parallelises *along the orbit* on `4 B` `u32` entries rather than squaring the operator); `#652` `NSPC` (`per-record` jumps, `1.82–2.38×` Open).

### Gate G1 & Gate G2 — Verification & Manifest (`verified`)
- **[`verify_cycle4.py`](2026-11-20-carbon-cycle4/verify_cycle4.py) (`verified`):** **`10,454 / 10,454` small cases + `6 / 6` adversarial + `5,399,692 / 5,399,692` authenticated bytes exact**; `GS64-1P` cuts 1-lane consumer steps to **`0.450–0.514×` $m$**, `GS64-2P` to **`0.348–0.426×` $m$**, and `GS128-2P` to **`0.226–0.275×` $m$**.
- **[`imd.toml`](2026-11-20-carbon-cycle4/imd.toml) (`verified`):** `check_manifest.py` outputs **`PASS`**.

---

## 4. Confront: Gate G5 Falsifier (`g5_imd.c` — **FAIL**)

Pre-registered in [`g5_imd.c`](2026-11-20-carbon-cycle4/g5_imd.c) (25 paired interleaved rounds on `Xeon Platinum 8481C`, `gcc -O3 -march=native`, **charging both `Lift` and `Walk` inside `LiftEm` and `Tot`**, with identical scalar queue fill and `endpoint.rs` across all arms — `measured`):

| Block | $m$ | `OpEm` (ms) | `BJ64_1` (`Lift` + `Walk`) | **`GS64_1` (ms)** | `G128_2` (ms) | `BJ/Op` `emit` | **`GS/Op` `emit`** | `128/Op` `emit` | `OpTot` (ms) | `TBJ64` (ms) | **`TGS64` (ms)** | **`GS/Op` `tot`** |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| `block-0.L` | 89,874 | 0.532 | `0.669` (`0.239 + 0.430`) | **0.605** | 1.246 | `1.257×` | **`1.136×`** | `2.341×` | 1.465 | 1.625 | **1.563** | **`1.067×`** |
| `block-2.L` | 112,070 | 0.706 | `0.912` (`0.302 + 0.610`) | **0.774** | 1.566 | `1.292×` | **`1.097×`** | `2.220×` | 1.820 | 2.047 | **1.918** | **`1.054×`** |
| `block-85.L` | 156,404 | 0.917 | `1.486` (`0.452 + 1.034`) | **1.035** | 2.324 | `1.621×` | **`1.129×`** | `2.535×` | 1.893 | 2.481 | **2.044** | **`1.080×`** |
| `block-143.L` | 196,841 | 1.502 | `2.417` (`0.619 + 1.798`) | **1.693** | 3.621 | `1.609×` | **`1.127×`** | `2.410×` | 3.358 | 4.305 | **3.598** | **`1.072×`** |
| `block-207.L` | 135,921 | 0.913 | `1.304` (`0.393 + 0.911`) | **0.970** | 2.252 | `1.429×` | **`1.063×`** | `2.467×` | 2.203 | 2.617 | **2.288** | **`1.038×`** |
| `block-210.L` | 204,281 | 1.448 | `2.334` (`0.645 + 1.689`) | **1.605** | 3.647 | `1.611×` | **`1.108×`** | `2.518×` | 2.887 | 3.830 | **3.103** | **`1.074×`** |

**Pre-registered Verdict:** **FAIL** (`0/6` blocks cleared `LiftEm <= 0.85×` or `Tot <= 0.90×`, despite exact byte-for-byte reconstruction, `0 B` arena, and `2.0–4.4×` fewer consumer pointer steps).

---

## 5. Learn: Why Extensional Operator Lifting Loses Where Orbit Splicing Wins (`The Spectral-vs-Extensional Multiplicity Law`)

`g5_imd.c`'s `BJLft` + `BJWlk` split exposes the exact mathematical and physical distinction between Alpöge's setting and positive-entropy BWT inversion:

1. **Compact Spectral Parameter vs Extensional Permutation (`Why the Analogy Breaks Informative Ground`):**
   An automorphic theta lift acts on a compact representation rather than iterating over every term of an exponential sum. By Theorems 1.1, 2.1, and 4.1, positive-entropy BWT blocks ($122 M$ random identifier bits) admit **no** compact torus/spectral coordinate, so $g^{-1} \in S_m$ must be represented extensionally on $m$ states.
2. **The Extensional Operator-Squaring Penalty (`1.5m` Reads $\times$ `2–4×` State Width):**
   - On `block-0.L` (`8m = 719 KiB`, fitting inside L2), `BJ64-1P` **did** speed up the 1-lane walk (`0.430 ms` vs `0.532 ms`), and hardware OoO `MLP` executed all `89,874` lift gathers in `0.239 ms` (`2.66 ns/gather`), yet `0.239 + 0.430 = 0.669 ms > 0.532 ms` because **$1.0m$ producer gathers + $0.55m$ consumer reads $= 1.55m$ random reads** exceed $1.00m$ reads!
   - On `block-210.L` (`m = 204,281`), widening entries from `4 B` (`817 KiB`, L2-resident) to `8 B` (`1.63 MiB`, `src + dst = 3.27 MiB` > `2 MiB` L2) caused `BJWlk` **alone** (`1.689 ms`, `0.55m` steps!) to run slower than `Open::emit` (`1.448 ms`, `1.00m` steps!). In-place `GS64-1P` (`1.63 MiB`, `0.478m` steps) beat `BJ64-1P` on `6/6` blocks (`1.038–1.080×` Open total), but still lost to `4 B` `u32` entries.
3. **Why Domain-Cut Orbit Splicing (`BVSMC`, Cycle 2) Is the Exact 1-Manifold Dual:**
   Because $g^{-1}$ has in-degree $1$ and out-degree $1$ everywhere, cutting the **orbit** at $S \ll m$ points achieves `MLP = 8` on the **unwidened `4 B` `u32` operator** at **`1.00m` total random reads (`0` producer gathers)** — strictly dominating operator squaring on both read count (`1.0m` vs `1.5m`) and cache footprint (`4m` vs `8m–16m`).
