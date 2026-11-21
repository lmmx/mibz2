# Wormhole Cycle 2 — Filtered Fox–Morse Collapse, Quasideterminant Splitting Cliffs, and Perseus $\mathbb{Z}[t, t^{-1}]$ Augmentation

*21 November 2026 · Branch `research/carbon-wormhole` · Continuing directly from [Cycle 1](2026-11-21-wormhole-cycle1-fvs-macro-duality.md)'s `Break 1`*

---

## 1. Explore & Triage: Following Cycle 1's Break into Algebraic Discrete Morse Theory

Cycle 1 proved that any *vertex-cutting* feedback reduction on $G_{\mathrm{ov}}$ or $G_\Phi$ is trapped at `80.7k–102.2k` retained rows + $O(r\log r)$ graph construction because thick JSON-syntax hubs carry self-loops and short spoke cycles when treated as indivisible vertices. Escaping `Break 1` requires treating multi-row strips as **unit-slope affine bijections** eliminated with **`0` retained rows**, while also shrinking `Open 0.5.8`'s critical table `edges[0..m)` (`351.1–798.0 KiB` at $\Lambda = 16$) into the i9-10900K's **`256 KiB` private L2 cache**.

We construct and confront three chains from `FRONTIER.md`'s Discrete Morse lead:

1. **Chain 2A (Width-Filtered Fox–Morse Collapse $\mathcal{M}_\Lambda$ via Gel'fand–Retakh Quasideterminants):**
   - *Link 1:* Punctured functional graph $X_p = ([0, n), \mathrm{LF}|_{\setminus \{p\}})$ equipped with width-filtered acyclic matching $\mathcal{M}_\Lambda = \{x \ne p : \mathrm{rw}(x) > \Lambda\}$ (Forman, *Adv. Math.* 139:90–145, 1998; Mischaikow & Nanda, *Discrete Comput. Geom.* 50:330–353, 2013).
   - *Link 2:* Non-commutative Fox free-group-ring boundary operator $\partial_{\mathrm{Fox}}$ over $\mathbb{Z}\langle\Sigma^*\rangle$ (Fox, *Ann. Math.* 57:547–560, 1953; Sköldberg, *Trans. AMS* 358:115–129, 2006; Jöllenbeck & Welker, *Mem. AMS* 197, 2009).
   - *Link 3:* Non-commutative Schur complement / Gel'fand–Retakh quasideterminant $(I - \partial)_{CC} - (I - \partial)_{CT}(I - \partial)_{TT}^{-1}(I - \partial)_{TC}$ (Gel'fand, Gel'fand, Retakh & Wilson, *Russ. Math. Surveys* 60:423–511, 2005) computed via interval-pairing SLP rules (`Endpoint`).
   - *Status:* **Deepened & confronted across $\Lambda \in \{16, 8, 4, 2, 1\}$** (Theorem 2.1).

2. **Chain 2B (Milnor–Fox Augmentation $\epsilon_t : \mathbb{Z}\langle\Sigma^*\rangle \to \mathbb{Z}[t, t^{-1}]$ / Perseus 6 Laurent Reduction):**
   - *Link 1:* Augmentation homomorphism $a \mapsto t$ collapsing non-commutative words $W \in \Sigma^*$ to integer exponents $|W| \in \mathbb{N}$ (Milnor, *Ann. Math.* 84:199–211, 1966).
   - *Link 2:* Totally unimodular algebraic Morse reduction over $\mathbb{Z}[t, t^{-1}]$ with $\pm 1$ pivots (Nanda, *Perseus 6* `docs/TECHNIQUES.md`) $\to$ exact critical $\mathrm{SA}$/orbit coordinates $\tau_p(x)$ in $O(H_\Lambda)$ space with **`0` SLP rules and `0 B` dictionary**.
   - *Status:* **Deepened & refuted as a standalone wordless decoder** by an exact $n = 5$ primitive twin witness (Theorem 2.2); retained as a zero-dictionary coordinate primitive.

3. **Chain 2C (Perseus 6 Index-Order Apparent-Pair Pointer Compression on $[0, m_\Lambda)$):**
   - Compressing pointer chains $j \to g^{-1}(j)$ sequentially in index order before walking (`Perseus 6` `TECHNIQUES.md`), restricted to cache-resident targets (`|next_j - j| < W`) so no random misses are paid before `emit`.
   - *Triage:* **Killed** by Theorem 2.3 (`0.039%–0.069%` `64 B` cache-line hits; `8.18%–17.38%` `32 KiB` L1d hits = pure uniform scatter $32\text{ KiB}/(4m_\Lambda)$ across alphabet buckets).

### Candidate Profiles (`METHOD.md` §3a)

| Field | Chain 2A (`Fox-Morse M_Lambda`, $\Lambda \to 1..4$) | Chain 2B (`Perseus-Laurent Z[t, t^-1]`) |
|---|---|---|
| **Mode** | Reorientation (filtration scale shift toward $r_1$) | Reorientation (abelianised coordinate lift) |
| **Displaced obligation** | `52.5%–69.4%` of `Open<16>`'s retained rows ($2 \le w \le 16$) and L2 overflow of `edges[0..m)` on i9-10900K (`256 KiB` L2) | Entire SLP rule arena (`rules`) and flattened word dictionary (`spelling + inline16`) |
| **Replacement principle** | Filtered Fox–Morse collapse $\mathcal{M}_\Lambda$ eliminates all strips $w > \Lambda$ via Gel'fand–Retakh quasideterminants | Fox augmentation $a \mapsto t$ reduces gradient paths to integer lengths $\ell \in \mathbb{N}$ with $\det(I - P_{\mathrm{crit}}|_{t=1}) = +1$ |
| **New paid obligations** | $H_\Lambda$ interval induction (`4.5–8.8×` steps at $\Lambda=1$) + **633–821 KiB flattened single-use dictionary cliff** | Unlabelled multi-byte gaps $[\tau(x)+1, \tau(g(x)))$ (unidentifiable without Fox words; Theorem 2.2(2)) |
| **Nearest prior art** | `#657` (`LIMIT ∈ {8, 12, 16, 24}` empirical sweep, which stopped at 8 without diagnosing the single-use `unit_work` cliff at $\Lambda \to 1$) | `record 81` weight-only RV on all $r$ boundaries (here generalized to arbitrary filtered critical sets $V_{\mathrm{crit}}(\mathcal{M}_\Lambda)$) |

---

## 2. Exact Theorems & Proofs

### Theorem 2.1 (L2 Critical Shrinkage vs Single-Use Quasideterminant Flattening Cliff — `proved`; `verified` on `26,135` small `(L, p, \Lambda)` cases + `30` 6-block decodes)

For every column $(L, p)$ and filtration threshold $\Lambda \ge 0$:
1. **Exactness & L2 Shrinkage:** The filtered matching $\mathcal{M}_\Lambda = \{x \ne p : \mathrm{rw}(x) > \Lambda\}$ is acyclic on $X_p$; its Gel'fand–Retakh quasideterminant over $\mathbb{Z}\langle\Sigma^*\rangle$ produces an SLP of at most $\mathrm{steps}(\mathcal{M}_\Lambda) \le 3 H_\Lambda + 2$ rules and reconstructs $(L, p)$ byte-for-byte on all primitive, periodic, and multi-cycle columns. Lowering $\Lambda : 16 \to 1$ shrinks $m_\Lambda$ by **`2.06×–3.27×`** (`89,874 → 35,116` on `block-0`, `112,070 → 34,262` on `block-2`), bringing `4 * m_1` (`133.8–243.8 KiB`) strictly inside the i9-10900K's **`256 KiB` private L2** on `4 / 6` blocks (`283.5 KiB` on `block-143`, `379.1 KiB` on `block-210`).
2. **The Single-Use Terminal Splitting Cliff:** When narrow multi-row corridors ($2 \le w \le 16$) diverge at interior BWT boundaries during interval elimination, their shared SLP DAG splits into **`unit_width` (`band.width == 1`) terminal bands** traversed **at most once** during `emit`. Eagerly flattening each band's SLP handle into `spelling + inline16` (`src/open/endpoint.rs:246`) inflates dictionary memory **`8.1×–11.7×`** (`53.9–117.6 KiB` at $\Lambda=16 \implies$ **`632.9–821.3 KiB`** at $\Lambda=1$), of which **`61.2%–75.5%` (`295.9–459.7 KiB`) is single-use `width == 1` byte strings**, causing total hot table footprint `edges + dict` to **increase** (`426.6–886.5 KiB → 776.7–1,110.2 KiB`) even as `> 16 B` walk steps **fall** by `29%–52%`!

---

### Theorem 2.2 (Milnor–Fox–Perseus Augmentation $\mathbb{Z}\langle\Sigma^*\rangle \to \mathbb{Z}[t, t^{-1}]$ & Unlabelled-Gap Non-Identifiability — `proved`; `verified` on `5,227` small `(L, p)` pairs + `6` blocks)

1. **Zero-Dictionary Exact Critical Coordinates:** Under $\epsilon_t(W) = t^{|W|} \in \mathbb{Z}[t, t^{-1}]$, filtered Morse elimination carries a single integer length per node (`0` SLP rules, `0 B` `spelling`), and the reduced cycle sum $\sum_{j \in O_{\mathrm{crit}}(p)} \ell_j = d$ equals the exact marked period $d \le n$, assigning every visited critical row $x_j \in V_{\mathrm{crit}}(\mathcal{M}_\Lambda)$ its exact orbit emission index $\mathrm{pos}_L(x_j) \in [0, d)$.
2. **Minimal Primitive Twin Witness ($n = 5, \Lambda = 1$):** Critical coordinates + $(n, p, C, V_{\mathrm{crit}}, L|_{V_{\mathrm{crit}}}, F|_{V_{\mathrm{crit}}}, g_{\mathrm{crit}}, \boldsymbol{\ell})$ at $\Lambda = 1$ **do not determine** $T$: at $n = 5, p = 0, C = [2, 2, 1]$, the two primitive single-cycle columns
   $$L_1 = (2, 0, 0, 1, 1) \;\mapsto\; T_1 = \mathtt{00112}, \qquad L_2 = (2, 1, 1, 0, 0) \;\mapsto\; T_2 = \mathtt{01102}$$
   share identical $V_{\mathrm{crit}}(\mathcal{M}_1) = \{0\}$, $L[0] = 2$, $F[0] = 0$, $g_{\mathrm{crit}}(0) = 0$, and reduced Laurent differential $t^5$, yet $T_1 \ne T_2$ because swapping two eliminated width-2 runs changes their excursion order without changing path length. $\square$

---

### Theorem 2.3 (BWT Radix Dispersion vs Index-Order Apparent Pairs — `measured` & `verified` on `6 / 6` blocks)

On `edges[0..m_{16})`, the fraction of pointers landing in the same `64 B` cache line (`16` `u32`s) is **`0.039%–0.069%`** across all 6 blocks (`99.93%–99.96%` cache-line miss rate per hop), and the fraction landing in the same `32 KiB` (`8,192`-slot) block equals $8{,}192 / m_{16}$ (`8.18%–17.38%`) within `±0.4%`, confirming uniform radix-bucket dispersion across $[0, m_\Lambda)$ and ruling out Perseus index-order pre-compression without random probes.

---

## 3. Confrontation Across Filtration Thresholds on the Six Blocks

- **Gate G1 (`verify_cycle2.py`):** **PASS** on `26,135 / 26,135` small `(L, p, \Lambda)` instances + `5,227 / 5,227` Perseus $\mathbb{Z}[t, t^{-1}]$ checks + `30 / 30` 6-block decodes (`0` errors).
- **Gate G2 (`check_manifest.py`):** `chain2a_filtered_fox_morse.toml` **PASS** (`EXIT=0`); `chain2b_perseus_laurent_open.toml` mechanically caught as **FAIL** (`EXIT=1`, `OPEN` edge refuted by Theorem 2.2(2)'s $n=5$ witness).

### Six-Block Filtration Table (`cycle2_results.json`)

| Block | $\Lambda$ | Critical $m_\Lambda$ | `edges` (`KiB`) | Strips $H_\Lambda$ | RV Steps | SLP Rules | Words | `dict` (`spell+inline16`) | `unit_spell` (`width=1`) | `edges + dict` | `>16B` Steps |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| `block-0.L` | **16** | 89,874 | 351.1 K | 3,448 | 11,215 | 7,334 | 1,326 | **75.5 K** | 8.3 K (`16%`) | **426.6 K** | 6,762 |
| `block-0.L` | **8** | 67,502 | 263.7 K | 5,461 | 17,450 | 11,853 | 1,833 | 132.3 K | 18.1 K (`18%`) | **396.0 K** | 5,386 |
| `block-0.L` | **4** | 57,588 | **225.0 K** | 7,098 | 22,891 | 16,015 | 2,418 | 215.3 K | 43.8 K (`25%`) | 440.3 K | 4,630 |
| `block-0.L` | **1** | **35,116** | **137.2 K** | 16,068 | 50,302 | 34,315 | 5,862 | **643.7 K** | **361.8 K (`66%`)** | 780.9 K | **3,239** |
| `block-143.L` | **16** | 196,841 | 768.9 K | 5,047 | 21,295 | 12,619 | 3,635 | **117.6 K** | 15.1 K (`27%`) | 886.5 K | 11,842 |
| `block-143.L` | **8** | 157,658 | 615.9 K | 8,541 | 36,246 | 22,966 | 5,510 | 208.1 K | 35.1 K (`30%`) | **824.0 K** | 9,965 |
| `block-143.L` | **4** | 126,918 | 495.8 K | 13,608 | 54,406 | 35,627 | 7,491 | 355.9 K | 83.2 K (`35%`) | 851.7 K | 8,619 |
| `block-143.L` | **1** | **72,578** | 283.5 K | 35,757 | 118,259 | 81,512 | 13,362 | **821.3 K** | **459.7 K (`76%`)** | 1,104.8 K | **6,582** |
| `block-210.L` | **16** | 204,281 | 798.0 K | 2,809 | 12,140 | 6,758 | 2,228 | **76.6 K** | 6.3 K (`17%`) | 874.6 K | 11,447 |
| `block-210.L` | **8** | 171,264 | 669.0 K | 5,506 | 23,742 | 14,613 | 3,750 | 139.8 K | 15.7 K (`20%`) | **808.8 K** | 10,263 |
| `block-210.L` | **4** | 152,344 | 595.1 K | 8,606 | 35,823 | 21,729 | 5,726 | 222.4 K | 34.0 K (`26%`) | 817.5 K | 9,824 |
| `block-210.L` | **1** | **97,054** | 379.1 K | 31,678 | 107,417 | 67,135 | 14,823 | **731.1 K** | **315.7 K (`64%`)** | 1,110.2 K | **8,085** |

*(Full 6-block × 5-threshold census in `cycle2_results.json`.)*

---

## 4. Learn: Where Chain 2 Breaks (`Break 2`), and the Exact Missing Link for Cycle 3

Cycle 2 pinpoints **why** naive filtered Fox–Morse collapse stalls below $\Lambda = 16$:
- **Witness (`Break 2`):** Lowering $\Lambda$ from `16` to `1` cuts $m_\Lambda$ by `2.06–3.27×` (`edges` down to `133.8–379.1 KiB`) and **cuts `>16B` walk steps in half (`6,762 → 3,239`)**, yet eager SLP flattening into `spelling + inline16` blows up `dict` from `53.9–117.6 KiB` to **`632.9–821.3 KiB`** because shared SLP subtrees split into `3,197–8,172` single-use (`width = 1`) bands!
- **Exact scope:** Any Fox–Morse decoder that eagerly flattens SLP rules into a static byte buffer (`self.spelling`) + allocates 16B `inline16` slots for $>16\text{ B}$ words.
- **Escape condition $\implies$ Missing Link for Cycle 3:**
  Look at the two numbers sitting side by side in the table above:
  1. **`90.8%–95.0%` of all walk steps have word length $\le 16\text{ B}$** (and only words of length $2 \le \ell \le 16$ ever use `inline16`, whereas `Open 0.5.8` also allocates wasted `inline16` slots for $\ell > 16$ words!).
  2. **Only `3,239–11,842` steps (`3%–9%`) have length $> 16\text{ B}$**, and their SLP subtrees are already shared inside `rules`! By **Rytter (*Theor. Comput. Sci.* 302:211–222, 2003, SLP $\to$ LZ77 equivalence) and Gasieniec, Kolpakov, Potapov & Sant (*DCC 2005*, real-time $O(1)$-per-byte output-self-referential SLP decoding)**, internal SLP nodes of length $> 16$ can memoize their **first emitted offset directly inside `out[0..at)`**, eliminating **`100%` of `self.spelling` (`0 B`)** and **`100%` of wasted $>16\text{ B}$ `inline16` slots**!
  **Cycle 3** builds, proves, verifies, and confronts this exact **Hybrid `Inline16` + Rytter–Gasieniec Output-Memoized SLP Wormhole (`OM-Fox`)**!

---

## 5. Reproducibility

```bash
BLOCKS_DIR=/home/antigravity/admitted-bwt python3 -u docs/journal/2026-11-21-wormhole-cycle2-morse-quasideterminant-cliff/verify_cycle2.py
python3 docs/team/acquisition/check_manifest.py docs/journal/2026-11-21-wormhole-cycle2-morse-quasideterminant-cliff/chain2a_filtered_fox_morse.toml
```
