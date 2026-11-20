# Carbon Kitchen Sink — Cycle 1: Ten Mathematical Worlds, Train-Track Coalescence, and the Ephemeral-Run Streaming Obstruction

*20 November 2026 · Branch `research/carbon-kitchen-sink` · Starting `origin/master` at launch: `3e4279962c440894ce0732c5b81b5bf379f7dcb6` (read-only boundary check after G5: `9948e336ac494573e574765e15ded62357eda4e8`).*

## 1. Environment and Constraints Summary

- **Workspace:** isolated git worktree `/home/antigravity/carbon-kitchen-sink` on branch `research/carbon-kitchen-sink` from `3e427996`, completely separate from `research/carbon-word-transport`.
- **Authenticated inputs:** all six blocks (`block-0.L`, `block-2.L`, `block-85.L`, `block-143.L`, `block-207.L`, `block-210.L`) fetched from `origin/data/admitted-bwt` and verified against `admitted-bwt/manifest.json` by SHA-256 and byte length (**verified**, 6/6 blocks).
- **Scorer & tools:** `docs/team/acquisition/check_manifest.py` operational (Python 3.11); C11 (`gcc -O3 -march=native`), Rust, and NumPy available.
- **Core constraints internalised from `CONTEXT.md` and `FRONTIER.md`:**
  1. Entry state $I_0$ holds only `(records, n, p, C, Σ)` where `records` are capped run records (`length <= 127`, $R \ge r$). Every derived structure pays its full $(W, D, B, Q, M)$ cost; sequential $O(n)$ and dependent $O(n)$ never pool.
  2. Production `Open<16, true, true>` (`qi16`) finishes in `1.35–3.03 ms` in calibrated C (`1.43–3.42 ms` native Rust per `9948e33`), split across run-pass/queue-fill (`28–36%`), thick-strip RV + spelling (`12.5–27.5%`), wiring (`3–5%`), and retained-row emission (`36–48%`).
  3. Structural step reductions (`NSPC`, `RV-Copy`, `BSO`, `Forest`) repeatedly lost natively whenever construction paid extra passes, unpacked compact structures on the critical path, or emitted text-determining run labels (`record 84`).

---

## 2. Explore & Triage: Ten Mathematical Worlds Brought into Contact with Inverse BWT

Following `METHOD.md §1–2` ("Mathematical maximalism: unrestricted exploration on the way in; rigorous confrontation on the way out"), Cycle 1 seeded and executed exact probes across ten distinct mathematical frameworks (`probe_worlds.py`, `probe_deep_math.py`; exact outputs in `probe_worlds_results.json` and `probe_deep_math_results.json`).

| # | Mathematical framework | Bridge to inverse BWT | Empirical / theoretical finding on toys & 6 blocks | Triage disposition |
|---|---|---|---|---|
| **W1** | **Koopman–Bohr spectral theory & 2D Rauzy fractals** (Rauzy 1982; Arnoux–Ito 2001) | Eigenfunctions $\psi_m(x) = e^{2\pi i m \tau(x)/n}$ of $U_{\mathrm{LF}}$; 2D Tribonacci Pisot torus ($x^3 - x^2 - x - 1 = 0$) damaged into $p = 1{,}024$ fractal cells | **Proved** Koopman–Row-Variation Identity; **verified** on damaged 1D ($p=1024, \bar{w}=3.99$) and 2D Rauzy fractal ($p=1024, \bar{w}=2.42$) that $\|\{\Delta\tau\}\|/r \in [0.005, 0.041]$ and low Koopman TV (`0.018`, `0.113`) survive width destruction. On the 6 Wikidata blocks, $\|\{\Delta\tau\}\|/r = 0.881\text{--}0.927$: thick runs have spatial resonance (`tv_thick_m1 = 0.18–0.54`; `block-0` spikes at $\Delta\tau = \pm 1049, -2098, +3147, +5245$ from 1,049-B timestamp stanzas), while width-1 GUID runs hit white noise (`min_tv_w1 = 0.83–0.90`). | **Retained as theorem + diagnostic; killed as standalone global coordinate** on GUIDs |
| **W2** | **Origami puncture monodromy & $\mathrm{SL}(2,\mathbb{Z})$ Veech shears** (`origami-lift.md §6.1–6.2`) | Commutator $k = r u r^{-1} u^{-1}$ and Nielsen shears $r u^{-s}$ ($s \in \{\pm 1, \pm 2\}$) on $(\mathrm{LF}, +1)$ | **Proved** Cone-Cycle Length-Blindness Theorem: $k$'s non-trivial cycle partition equals that of the width-1 reduced run-word $w_{\mathrm{runs}} \in \Sigma^r$ (**verified** 6/6 blocks), carrying 0 bits of width information beyond $w_{\mathrm{runs}}$. Shears $r u^{\pm 1, \pm 2}$ have `13–71` cycles and `0–3` fixed points (**measured**, 6 blocks). | **Closes `origami-lift.md §6.1`**; simple global shears killed |
| **W3** | **Quantum-torus few-nomials, unimodular coboundaries & $p$-adic Hensel / LFSR** | Bivariate kernel $G_f(z,w) = \sum z^x w^{f(x)}$; punctured operator $(I - T_0)\tau = \mathbf{1}$; Berlekamp–Massey over $\mathbb{F}_{257}$ | **Proved** $(zw-1)G_f(z,w) = \sum_{j=1}^r (z^{b_j}w^{d_j} - z^{a_j}w^{c_j})$ (`2r` exact terms on 6/6 blocks) and $\det(I - T_0) = 1$ over $\mathbb{Z}$, with nilpotency index $n$ over every $\mathbb{Z}/p^k\mathbb{Z}$ (killing Hensel bit-lifting without squaring). Berlekamp–Massey degree on 2,048 run shifts mod 257 is **exact $1024 = N/2$** on 6/6 blocks (**measured**). | **Killed** (algebraic irreducibility + nilpotency $n$) |
| **W4** | **Free left-regular band (Tsetlin / Solomon descent algebra) & pre-column MTF+RLE2** (`FRONTIER.md` Obligation 5) | Exact reconstruction of bzip2's Huffman/MTF symbol stream prior to column/record materialisation | **Proved** Exact MTF–Run Bijection; reconstructed `89,730` exact symbols on `block-0` (matching `astra-d-memory.md` to the symbol; `89,730–185,183` across 6 blocks). Active $\alpha = 86\text{--}96$; `98.0–98.6%` of MTF moves sit at index $\le 32$; `nn=1` (`aba`) width-1 pinholes occur `4,339–11,131` times, turning two thin runs thick only `108–498` times (`< 0.4%` of $r$). | **Settles naive pinhole fusion**; pre-column fusion deferred to Cycle 2 |
| **W5** | **Symmetric-group RSK / Schensted–Greene Young diagrams** | Patience-sorting shape $\lambda(\mathrm{LF})$ and $\lambda(g_{\mathrm{Open}})$ | **Proved** via Greene's theorem that $\ell(\lambda(\mathrm{LF})) \le \alpha$. **Measured** on 6 blocks: $\ell(\lambda(\mathrm{LF})) = 59\text{--}74$, $\lambda_1(\mathrm{LF}) = 157{,}821\text{--}176{,}716$; for `Open`'s $g \in S_m$, $\ell(\lambda(g)) = 100\text{--}141$ (vs $2\sqrt{m} \approx 600\text{--}904$ for random $S_m$). | **Retained as structural invariant** (monotone-cover bound) |
| **W6** | **Wheeler $k$-gram synchronising tower** ($C_1 \dots C_8$) | Monotone refinement of context buckets without row scans | **Measured** (`block-0`, `block-210`): $N_1 = 89\text{--}96$, $N_2 = 1{,}628\text{--}2{,}219$, $N_4 = 37{,}490\text{--}78{,}358$, but only `5.05–12.89%` of rows are uniquely synchronised by $k=8$. | **Killed** ($N_k$ approaches $r$ before synchronising GUIDs) |
| **W7** | **Excursion & Single-Use Amortization Theory on `Open`** | Anatomy of `Open<16>`'s $m$ retained steps (`height = 0` vs `height > 0`) | **Measured** (6 blocks): `77.3–86.1%` of `Open`'s $m$ steps are `height == 0` (`thin -> thin`), leaving only $h = 15{,}278\text{--}44{,}622$ (`1.7–5.0%` of $n$) thick excursions; `52.5–69.4%` of $m$ sits in runs of width $2 \le w \le 16$. **Proved** Single-Use Amortization Barrier: on an explicit 1-cycle table, contracting width-1 edges after materialisation has $\le 1:1$ payoff. | **Feeds W8/W9 synthesis** |
| **W8** | **Thurston measured train-track series reduction + left-pinned RV (`TTSR-RV`)** | Collapse all $(d_{\mathrm{in}}, d_{\mathrm{out}}) = (1, 1)$ runs (`87.8–89.3%` of $r$) before RV | **Proved** Universal Width-1 Pass-Through Theorem (100% of width-1 runs have $d_{\mathrm{in}} = d_{\mathrm{out}} = 1$). Cuts accelerated RV to **`0.407r–0.497r`** (`2.0–2.5×` below raw $r$ floor), `m = 0` row walks! | **Promoted to Deepen & Confront (G1–G5)** |
| **W9** | **Non-commutative functorial ordering (`TTO` vs `DECO`: Decoupled Exact-Chain Open)** | Commutator of $\mathcal{E}_{\mathrm{thick}}$ (thick RV) and $\mathcal{S}_{\mathrm{pass}}$ (pass-through contraction) | Discovered non-commutative blow-up in `TTO` (`spelling_bytes` $50\text{ KB} \to 718\text{ KB}$ because width-1 GUID arcs subdivide thick strips). Inverted via **Void-Deleted Exact-Chain Open (`DECO`)**: **zero** thick-strip splitting, RV steps $\le$ `Open`, $m' = 0.42\text{--}0.52\,m$ (`w1`) or $0.68\text{--}0.84\,m$ (`w2`). | **Promoted to Deepen & Confront (G1–G5)** |
| **W10** | **Factorisation monodromy of $g = P \circ \mathrm{LF}_v$** | Separate cycle decompositions of $\mathrm{LF}_v$ and $P$ on $[0, m)$ | **Measured** (6 blocks): $\mathrm{LF}_v$ splits into `8–22` cycles and $P$ into `25–212` cycles (`15–201` fixed points), while $P \circ \mathrm{LF}_v$ stitches into `1` cycle. | **Structural context** |

---

## 3. Auxiliary Mathematical Theorems Established in Explore

### Theorem 1 (Koopman–Row-Variation Identity and Torus Separation — **Proved**; **Verified** on 4 toys + 6 blocks)
Let $f = \mathrm{LF}$ act as a single $n$-cycle on $[0, n)$ with orbit time $\tau(f^t(p)) = t$, and let $\Delta\tau(x) := (\tau((x+1)\bmod n) - \tau(x)) \bmod n$.
1. **Gap-Chain Constancy:** Whenever $L[x] = L[x+1]$ ($x$ is not a run tail), $f(x+1) = f(x) + 1$, hence $\Delta\tau(f(x)) = \Delta\tau(x)$. Consequently $|\{\Delta\tau(x) : 0 \le x < n\}| \le r$.
2. **Exact Koopman Row-Variation:** Every Koopman eigenfunction of $U_f$ on $\mathbb{C}^n$ with unit modulus has the form $\psi_m(x) = c \cdot e^{2\pi i m \tau(x)/n}$ ($m \in \mathbb{Z}/n\mathbb{Z}$), with cyclic row-space total variation
   $$\mathrm{TV}_{\mathrm{row}}(\psi_m) := \sum_{x=0}^{n-1} |\psi_m(x+1) - \psi_m(x)| = 2 \sum_{x=0}^{n-1} \left|\sin\left(\frac{\pi m \Delta\tau(x)}{n}\right)\right|.$$
3. **Separation of Damaged Torus Toys from Unique-ID Blocks:** For a coding of an irrational translation $z \mapsto z + \beta$ on $\mathbb{T}^d$ by $p$ bounded-perimeter cells ($p \ll n$, e.g. $p = 1024$ on $d=1$ circle or $d=2$ Tribonacci Rauzy fractal), run width collapses ($n/r \approx 2.4\text{--}4.0$), yet intra-cell circle/torus monotonicity preserves $|\{\Delta\tau(x)\}| \ll r$ and $\min_{m \neq 0} \mathrm{TV}_{\mathrm{row}}(\psi_m)/n \ll 4/\pi$. Conversely, $M$ independent random strings (GUIDs) induce a random permutation of their $M$ text offsets under lexicographic sorting, forcing $\min_{1 \le m \le 32} \mathrm{TV}_{\mathrm{row}}|_{w=1} / ((4/\pi)M) \in [0.829, 0.903]$ across all six Wikidata blocks (`probe_deep_math.py`). $\square$

### Theorem 2 (Origami Cone-Cycle Length-Blindness — **Proved**; **Verified** on 6/6 blocks)
Let $L \in \Sigma^n$ have maximal runs $(c_0, w_0), \dots, (c_{r-1}, w_{r-1})$ with $r \ge 1$, and let $w_{\mathrm{runs}} := c_0 c_1 \cdots c_{r-1} \in \Sigma^r$ be the width-1 reduced column. Suppose no two cyclically adjacent distinct-symbol runs share an accidental boundary continuity $\mathrm{LF}(s_j - 1) + 1 \equiv \mathrm{LF}(s_j) \pmod n$ (i.e. MONODROMY's $X = \emptyset$, which holds iff the last occurrence of $c_{j-1}$ does not immediately precede the first occurrence of $c_j$ in $F$, and holds on all 6 admitted blocks). Then the non-trivial cycle structure of the origami commutator $k = \mathrm{LF} \circ u \circ \mathrm{LF}^{-1} \circ u^{-1}$ on $[0, n)$ is **canonically isomorphic** to that of $k_{w_{\mathrm{runs}}}$ on $[0, r)$.

*Proof.* Define the run-tail bijection $\beta: \{0, \dots, r-1\} \to \{e_0, \dots, e_{r-1}\} \subseteq [0, n)$ where $e_j = s_j + w_j - 1$ is the last row of run $j$. Inside any run ($x \notin \{e_j\}$), $u^{-1}(y) = y - 1$: $k(y) = y$ whenever $\mathrm{LF}^{-1}(y - 1)$ is not a run tail, i.e., whenever $y \notin \{\mathrm{LF}(e_j) + 1 \bmod n\}$. For $y_j := (\mathrm{LF}(e_j) + 1) \bmod n$, $u^{-1}(y_j) = \mathrm{LF}(e_j)$, so $\mathrm{LF}^{-1}(u^{-1}(y_j)) = e_j$, $u(e_j) = s_{(j+1)\bmod r}$, and $k(y_j) = \mathrm{LF}(s_{(j+1)\bmod r})$. Which run-end image $y_{j'}$ equals $\mathrm{LF}(s_{(j+1)\bmod r})$ depends **solely** on the relative stable-rank ordering of run tails and run heads among runs of each symbol — which is identical in $L$ and in $w_{\mathrm{runs}}$ because collapsing each run $[s_j, e_j]$ to a single point preserves the order of runs within every symbol bucket $c \in \Sigma$ and preserves cyclic adjacency $j \mapsto j + 1 \bmod r$. Thus $k$ restricted to $\{y_0, \dots, y_{r-1}\}$ conjugates directly to $k_{w_{\mathrm{runs}}}$, independent of $(w_0, \dots, w_{r-1})$. $\square$

*Consequence:* Resolving `origami-lift.md §6.1`, walking the cone-point cycles extracts zero information about $(w_0, \dots, w_{r-1})$.

### Theorem 3 (Exact Pre-Column MTF–Run Bijection — **Proved**; **Verified** on 6/6 blocks)
For any bzip2 block with active byte map `seq_to_unseq[0..α-1]` (distinct bytes in ascending order) and initial MTF state `[0, 1, ..., α-1]`, every maximal BWT run $j \in \{0, \dots, r-1\}$ (except $j = 0$ when $L[0] = \text{seq\_to\_unseq}[0]$) corresponds bijectively to one literal symbol $s \in [2, \alpha]$ followed by the bijective base-2 (`RUNA/RUNB`) encoding of $w_j - 1$ (or $w_0$ when $j=0, \text{idx}=0$). Reconstructed total bzip2 symbol count on `block-0.L` is `89,730`, matching the compressed-stream trace in `astra-d-memory.md` line 9 (`89,730`). $\square$

### Proposition 4 (RSK Young-Diagram Height Bound — **Proved**; **Measured** on 6 blocks)
Because $\Psi = \mathrm{LF}^{-1}$ is strictly increasing on each of the $\alpha$ occupied symbol intervals $[C[a], C[a+1))$, $[0, n)$ partitions into $\alpha$ increasing subsequences of $\Psi$. By Greene's Theorem (1974) and RSK symmetry $\lambda(\pi^{-1}) = \lambda(\pi)$, the Young diagram $\lambda(\mathrm{LF})$ has at most $\alpha$ rows. Measured on the 6 blocks: $\alpha \in [86, 96]$, $\ell(\lambda(\mathrm{LF})) \in [59, 74]$, $\lambda_1(\mathrm{LF}) \in [157{,}821, 176{,}716]$, and $\ell(\lambda(g_{\mathrm{Open}})) \in [100, 141]$. $\square$

---

## 4. Deepen: Two Serious Candidates (`TTSR-RV` and `DECO`)

### Candidate A: Train-Track Series-Reduced Accelerated RV (`TTSR-RV`)
- **Mode:** Reorientation.
- **Displaced obligation:** Both `Open`'s $m$-step dependent retained-row walk (`emit`, `36–48%`) and `Cells`'s $\ge r$ monotone RV induction steps across pass-through runs.
- **Replacement principle:** **Universal Width-1 Pass-Through & Left-Pinned Series Reduction Theorem (Proved; Verified on 5,227 small cases + 6 blocks).**
  1. In the run network $G_{\mathrm{run}}$ (vertices = maximal runs after splitting $[s, e)$ at $p$ so $p_{\mathrm{run}} = [p, p+1)$ has width $1$), every run $u$ of width $w_u = 1$ has $|I_u| = |\mathrm{LF}(I_u)| = 1$, which cannot straddle a domain or image boundary; hence $d_{\mathrm{in}}(u) = d_{\mathrm{out}}(u) = 1$ universally.
  2. Deleting all pass-through runs $U_{\mathrm{pass}} = \{u \neq p_{\mathrm{run}} : d_{\mathrm{in}}(u) = d_{\mathrm{out}}(u) = 1\}$ (`87.79–89.25%` of $r$ on the 6 blocks, including 100% of non-primary width-1 runs!) leaves a core $X_{\mathrm{core}}$ of `0.107–0.122r` runs and $E_{\mathrm{core}} = 0.317\text{--}0.371r$ arcs (`16,726–40,818` pieces), with total leaf dictionary size **identically equal** to $E = 2r - 1 - C$ bytes (`62,138–155,480` B). Any closed cycle inside $U_{\mathrm{pass}}$ has $d_{\mathrm{in}} = 1$ at every vertex, so no path from $X_{\mathrm{core}}$ can enter it, and $p_{\mathrm{run}} \in X_{\mathrm{core}}$ guarantees it is disjoint from the marked orbit.
  3. Placing $p_{\mathrm{run}}$'s width-1 piece at index $0$ (`node 1`) of $[0, N_{\mathrm{core}})$ and keeping $y$ in `top_order` on equal-width RV merges ($w_x = w_y, x \neq y$) keeps `node 1` invariant at the leftmost domain slot until `x == y == 1` (`width == 1`), yielding the exact period starting at $p$ with **zero** mark-offset branches and **zero** post-hoc rotation.
  4. Accelerated winner-run steps on $T_{\mathrm{core}}$ drop to **`0.407r – 0.497r`** (`22,009` on `block-0`, `52,349` on `block-210`) — **2.0×–2.5× below** the raw $r$-piece monotone RV floor (`records 79–80`).
- **New paid obligations:** Materialising $r$ runs + $O(r)$ two-pointer overlap merge (`n_arcs = 1.21–1.25r`), tracing $0.89r$ pass-through hops, core incoming-offset bucket insertion, and one-scan SLP memoized expansion into `out[0..n)`.
- **Nearest prior art:** `2026-11-20-run-network.md` (measured the `0.11r` core and rejected it as a *row-walk* section of `0.52n` rows, leaving non-walk core operations open); Dunfield–Thurston (2006) / Agol–Hass–Thurston (2006) band splitting; `Cells` (`src/cells.rs`).

### Candidate B: Decoupled Exact-Chain Open (`DECO`)
- **Mode:** Reorientation / Architectural Hybrid (inverting `TTO`'s non-commutative thick-strip shattering).
- **Displaced obligation:** `48–58%` (`DECO-w1`) or `16–32%` (`DECO-w2`) of `Open`'s retained walk $m$, `queues` volume, and `edges` scatter/footprint, **without** increasing `Open`'s thick-strip RV work.
- **Replacement principle:** **Decoupled Void-Deletion & Tail-Ordered Queue Theorem (Proved; Verified on 28,080 small configs + 6 blocks).**
  1. *Why naive `TTO` shattered thick strips:* Series-reducing width-1 chains *before* thick elimination (`verify_tto.py`, 19,267 small cases + 6 blocks byte-exact) cut $m$ to `0.179–0.240× Open` (`16,098` rows on `block-0`), but subdivided thick strips at outgoing GUID boundaries, inflating `rv_steps` from `11,215` to `33,851` and `spelling_bytes` from `50.7 KB` to `717.6 KB`.
  2. *Inversion (`DECO`):* Restrict exact whole-run coalescence $f(I_u) = I_v$ (`img_start[u] == start[v]` and `width[u] == width[v] >= min_w`, $v \neq p_{\mathrm{run}}$) strictly to thin runs (`width <= 16`). Let $I_{\mathrm{abs}} = \bigsqcup_{v\text{ absorbed}} I_v$. Because $I_{\mathrm{abs}} \subseteq S_{\mathrm{thin}}$ and $I_{\mathrm{abs}} = f(S_{\mathrm{thin}} \setminus \{\text{tails}\}) \subseteq f(S_{\mathrm{thin}})$, we have $I_{\mathrm{abs}} \cap (C_{\mathrm{thick}} \cup f(C_{\mathrm{thick}})) = \emptyset$.
  3. Under monotone compression $[0, n) \setminus I_{\mathrm{abs}} \xrightarrow{\sim} [0, n - |I_{\mathrm{abs}}|)$, **every thick strip stays 100% undivided** with identical width and rigid translation, while `endpoint.rs` omits `has_pred[u]` from domain gaps and `succ_exact[z] != -1` from codomain gaps — strictly **reducing** RV nodes (`6,600` vs `Open`'s `~10,300` on `block-0`) and RV steps (`10,487` vs `11,215`).
  4. Because `img_start[z]` increases monotonically with $z$ inside each symbol bucket, scanning chain tails $z = 0 \dots r-1$ (`succ_exact[z] == -1`) populates per-symbol entrance queues in **exact sorted $E'$ order with zero sorting**.
- **New paid obligations:** Merging capped records into $r$ run arrays + `cod_order[0..r]`, one `2r` two-pointer exact-match scan, 16-lane lockstep chain tracing (`5.7k–11.0k` hops at `w2`; `31.3k–82.7k` hops at `w1`), and run-granularity `DecoQueueItem` band unpacking.
- **Nearest prior art:** `Open` (`src/open.rs`, `proof.md`), `astra-investigation-3.md` line 41 (18-round two-list exact-match contraction across the whole block at `8.49 ms`), `#654` (`Q`, packed thin queues).

### Gate G1 & G2 Verification Summary
- **`verify_ttsr.py`:** **5,227 / 5,227** exhaustive small cases (`3,586` binary $n \le 8$ + `1,641` ternary $n \le 5$, all primary marks, periodic & multi-cycle) + **6 / 6** authenticated blocks byte-exact (**verified**).
- **`verify_tto.py`:** **19,267 / 19,267** exhaustive small-case/limit configurations + **6 / 6** blocks byte-exact (**verified**).
- **`verify_deco.py`:** **28,080 / 28,080** exhaustive small-case configurations (`21,516` binary + `6,564` ternary) + **12 / 12** block runs (`min_w = 1, 2`) byte-exact (**verified**).
- **Gate G2 (`check_manifest.py`):** Both `deco.toml` and `ttsr_rv.toml` **PASS** `docs/team/acquisition/check_manifest.py` with zero open/assumed or double-paid text-determining edges (**verified**).

---

## 5. Confront: Gate G5 Pre-Registered C Falsifier (`g5_kitchen_bench.c`)

- **Protocol (fixed prior to execution):** Start every arm from identical capped `u16` run records (`CAP = 127`), primary `p`, and length `n` using pre-allocated reusable workspaces (matching `Open::Kernel` steady state). Verify byte-exactness against dense LF and 64-byte output canaries on all 6 blocks. Run **21 paired, interleaved rounds** (after 3 warm-up rounds), rotating arm order each round. Pass criterion: median whole-kernel time $\le 0.90\times$ `Open-qi16` on $\ge 4/6$ blocks with none $> 1.00\times$.
- **Baseline fidelity check:** `Open-qi16` medians (`1.35, 1.70, 1.81, 3.03, 2.08, 2.76 ms`) match native Rust `Open` (`calib.rs` in `9948e33`: `1.43, 1.81, 1.89, 3.42, 2.17, 3.06 ms`) and PR `#653`'s phase shares (`pass1: 31%`, `rv+spell: 14–28%`, `wire: 3.7–5.0%`, `emit: 36.8–49.3%`).

### Measured Results (21 Paired Interleaved Rounds, `gcc -O3 -march=native`, µs)

| Block | Arm | Median (µs) | IQR p25–p75 (µs) | vs `Open-qi16` | `pass1` | `chains` | `rv` | `wire` | `emit` | `m_rows` | `rv_steps` |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| **0** | `Open-qi16` | **1,347.3** | 1,321.6–1,391.8 | **1.000×** | 414.7 | 0.0 | 378.0 | 67.8 | **496.5** | 89,874 | 11,215 |
| **0** | `DECO-w2` | 2,566.2 | 2,464.6–2,605.9 | 1.905× | 716.4 | 420.7 | 482.5 | 489.3 | 435.4 | 63,357 | 10,968 |
| **0** | `DECO-w1-fast16` | 2,793.6 | 2,655.1–2,825.3 | 2.073× | 648.9 | 697.7 | 566.4 | 506.5 | **344.6** | 37,740 | 10,487 |
| **0** | `TTSR-RV` | 3,652.9 | 3,625.7–3,795.8 | 2.711× | 656.9 | 1,238.5 | 415.9 | 0.0 | 1,379.4 | 0 | 50,252 |
| **2** | `Open-qi16` | **1,697.1** | 1,640.0–1,737.5 | **1.000×** | 485.2 | 0.0 | 453.8 | 87.6 | **667.2** | 112,070 | 13,671 |
| **2** | `DECO-w2` | 3,015.6 | 2,934.7–3,102.0 | 1.777× | 785.1 | 494.9 | 580.7 | 588.6 | 558.9 | 75,859 | 13,337 |
| **2** | `DECO-w1-fast16` | 3,154.0 | 3,073.4–3,231.0 | 1.858× | 700.1 | 757.6 | 650.8 | 612.9 | **431.7** | 51,412 | 12,880 |
| **2** | `TTSR-RV` | 4,329.1 | 4,224.2–4,495.2 | 2.551× | 726.9 | 1,404.6 | 515.4 | 0.0 | 1,719.7 | 0 | 59,054 |
| **85** | `Open-qi16` | **1,807.9** | 1,731.0–1,833.7 | **1.000×** | 636.3 | 0.0 | 224.7 | 63.0 | **872.0** | 156,404 | 7,130 |
| **85** | `DECO-w2` | 3,547.9 | 3,468.1–3,675.3 | 1.962× | 1,091.5 | 635.1 | 519.2 | 611.7 | 719.7 | 106,384 | 7,070 |
| **85** | `DECO-w1-fast16` | 3,910.1 | 3,833.3–3,936.8 | 2.163× | 1,001.9 | 1,074.9 | 646.2 | 632.6 | **553.9** | 65,815 | 6,945 |
| **85** | `TTSR-RV` | 5,885.2 | 5,683.6–6,011.7 | 3.255× | 1,039.1 | 1,999.8 | 626.9 | 0.0 | 2,200.2 | 0 | 73,968 |
| **143** | `Open-qi16` | **3,030.0** | 3,002.5–3,170.9 | **1.000×** | 866.0 | 0.0 | 676.6 | 139.9 | **1,389.5** | 196,841 | 21,295 |
| **143** | `DECO-w2` | 6,199.1 | 6,110.1–6,445.2 | 2.046× | 1,479.9 | 876.7 | 927.7 | 1,252.5 | 1,700.6 | 154,755 | 21,038 |
| **143** | `DECO-w1-fast16` | 5,963.1 | 5,854.0–6,259.3 | 1.968× | 1,351.2 | 1,503.8 | 1,122.2 | 1,148.8 | **912.3** | 102,108 | 20,544 |
| **143** | `TTSR-RV` | 9,465.7 | 9,157.8–9,637.2 | 3.124× | 1,502.6 | 2,997.9 | 991.9 | 0.0 | 3,926.1 | 0 | 107,889 |
| **207** | `Open-qi16` | **2,082.6** | 1,981.1–2,099.2 | **1.000×** | 646.1 | 0.0 | 448.7 | 96.2 | **859.3** | 135,921 | 13,898 |
| **207** | `DECO-w2` | 4,167.0 | 3,991.5–4,247.2 | 2.001× | 1,190.1 | 649.9 | 656.6 | 789.5 | 838.0 | 114,059 | 13,793 |
| **207** | `DECO-w1-fast16` | 4,635.9 | 4,479.2–4,705.6 | 2.226× | 1,088.1 | 1,208.1 | 859.1 | 851.6 | **601.6** | 69,019 | 13,398 |
| **207** | `TTSR-RV` | 6,997.2 | 6,781.7–7,065.6 | 3.360× | 1,157.6 | 2,307.0 | 755.0 | 0.0 | 2,704.6 | 0 | 85,404 |
| **210** | `Open-qi16` | **2,760.6** | 2,665.8–2,789.0 | **1.000×** | 879.3 | 0.0 | 387.3 | 102.0 | **1,361.8** | 204,281 | 12,140 |
| **210** | `DECO-w2` | 5,686.4 | 5,473.2–5,781.4 | 2.060× | 1,680.2 | 891.3 | 742.8 | 1,002.4 | 1,315.4 | 171,040 | 12,006 |
| **210** | `DECO-w1-fast16` | 6,235.3 | 6,059.2–6,435.3 | 2.259× | 1,564.4 | 1,741.3 | 1,051.9 | 1,022.8 | **858.6** | 97,743 | 11,716 |
| **210** | `TTSR-RV` | 9,918.2 | 9,694.2–10,044.6 | 3.593× | 1,738.6 | 3,408.3 | 1,001.1 | 0.0 | 3,745.0 | 0 | 111,727 |

*(Full 6-arm table including `DECO-w1` and `Open-qi16-fast16` is saved in `g5_kitchen_bench_output.txt`.)*

### Verdict: Gate G5 FAILS for standalone single-thread speedup (`DECO` `1.78–2.26× Open`; `TTSR-RV` `2.55–3.59× Open`), exposing a decisive new structural law.

---

## 6. Learn: The Ephemeral-Run Streaming Obstruction & What Survives

1. **Positive emission validation (`DECO-w1-fast16`):** Cutting retained rows $m$ to `0.42–0.52×` genuinely cuts `emit` time on every block (`496.5 → 344.6 µs` on `block-0`, `872.0 → 553.9 µs` on `block-85`, `1361.8 → 858.6 µs` on `block-210`: **`0.630–0.694× Open`'s `emit`**), saving `152–503 µs`. Likewise, `TTSR-RV`'s core RV induction (`0.41–0.50r` winner-run turns) executes in just `416–1,001 µs`.
2. **Why both lost end-to-end — The Ephemeral-Run Materialisation Obstruction:**
   - **Witness:** `g5_kitchen_bench.c` phase split: `pass1` alone in `DECO`/`TTSR` (`649–1,739 µs` to materialise `run_start[r], run_width[r], img_start[r], cod_order[r]` and run one $2r$ two-pointer merge) already exceeds `Open-qi16`'s `pass1` (`415–879 µs`) by `+234 to +860 µs` — eating 100% of the maximum possible `emit` savings (`152–503 µs`) *before* chain tracing (`421–1,741 µs`) even begins!
   - **Root structural ratio ($m / r \approx 1.56\text{--}1.97$):** On admitted Wikidata blocks, $r = 51{,}184\text{--}128{,}732$ is already **`51%–64%` as large as $m = 89{,}874\text{--}204{,}281$**. `Open` (`src/open.rs:85`) **never materialises $O(r)$ run arrays at all**: maximal runs exist only ephemerally in registers `(c, pos, width)`, writing only $4m$ bytes to sequential `queues[c]` tails and $H \le 5{,}047$ thick strips (`< 80 KB`). Any construction that materialises explicit $O(r)$ run/partition tables (`16–32 B/run` = `0.8–4 MB`) and scans $r$ runs 2–3 times performs $3r \approx 1.8m$ memory operations, exceeding `Open`'s single-use $m$-row budget.
   - **Scope:** Binds any column-in/records-in decoder on $1/64 \le R/n < 0.15$ blocks ($m/r < 2$) that materialises explicit $O(r)$ per-run domain/codomain arrays and multi-pass $O(r)$ merges to contract thin runs.
   - **Escape condition (guiding Cycle 2):** Any winning transformation must **either**:
     1. operate **ephemerally in registers during the single `records` (or pre-record MTF/Huffman) stream** without materialising $O(r)$ arrays, **or**
     2. eliminate `Open`'s `queues + wiring` / `emit` cost directly on `[0, m)` in $\le 1$ pass, **or**
     3. act at $O(H)$ scale ($H \approx 1{,}565\text{--}5{,}047 \ll r$) rather than $O(r)$ scale.
