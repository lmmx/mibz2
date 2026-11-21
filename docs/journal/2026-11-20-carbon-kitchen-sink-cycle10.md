# Carbon Kitchen Sink — Cycle 10: Model-Independent Cut-Communication Complexity, Coset Seed-Pool Orbit Decomposition (`SPOD-Coset512`), and the 10-Cycle Mathematical Atlas

**Date:** 2026-11-20
**Branch:** `research/carbon-kitchen-sink`
**Inputs:** Cycles 1–9 (`d04880a`), `origin/master` Carbon Depth audit (`4ab98cd`), `docs/team/research-charter/FRONTIER.md`.
**Artefacts:** [`probe_cycle10.py`](2026-11-20-carbon-kitchen-sink-cycle10/probe_cycle10.py), [`probe_cycle10_results.json`](2026-11-20-carbon-kitchen-sink-cycle10/probe_cycle10_results.json).

---

## 0. Capstone Mandate

Our final cycle synthesises Cycles 1–10 to answer three overarching questions:
1. **What lower bound is genuinely model-independent** — binding *arbitrary* Boolean circuits, Word-RAM programs, SIMD networks, polynomial/NTT solvers, and streaming sorters without Carbon Depth's semigroup-faithfulness or fitted-constant assumptions?
2. **Which model assumptions in conventional BWT decoders are artefacts that unconventional mathematics *can* break**, and what concrete new primitives emerge?
3. **What is the complete 10-cycle mathematical map** of exact inverse Burrows–Wheeler reconstruction?

---

## 1. Yao Two-Party Spatial Cut-Communication & Aggarwal–Vitter Tile-Transport Census

 Carbon Depth's Tier B `HBMC` bound (`0.29–0.59 ms`) assumed a semigroup-faithful decoder executing one state transition per retained element. To establish what holds **with zero algorithmic structure assumptions**, we apply Yao's two-party communication complexity and Aggarwal–Vitter external-memory permuting directly to the bijection $(L, p) \leftrightarrow T$.

### Theorem 10.1 (Model-Independent Cut-Communication Floor & All-to-All Tile Scatter — *proved*; *verified* on $K \in \{3, 4\}$ permutation gadgets + 6 Wikidata blocks)

1. **Semigroup-Free Yao Cut Floor:** Embed an arbitrary bijection $\sigma \in S_K$ into $K$ structured records $T_\sigma = \mathtt{\$}\,\prod_{i=0}^{K-1} (\kappa_i\,\nu_{\sigma(i)})$ over disjoint key/value sub-alphabets (`n = 2K + 1`, primitive for every $\sigma \in S_K$). Because $T_\sigma \mapsto (L_\sigma, p_\sigma)$ is injective (`K! = 6, 24` verified primitive BWTs), any deterministic circuit, algebraic/NTT transform, or RAM algorithm partitioned across a fast memory boundary separating the key buckets from the value/output region must transfer at least
   $$I_{\mathrm{cut}} \;\ge\; \log_2(K!) \;=\; K \log_2(K / e) + \Theta(\log K) \text{ bits}$$
   across the cut — yielding $\ge \frac{\log_2(K!)}{8 B_{\mathrm{line}}}$ unconditional cache-line transfers (and Aggarwal–Vitter $\Omega\!\left(\frac{K}{B_{\mathrm{line}}} \log_{M / B_{\mathrm{line}}} \frac{K}{B_{\mathrm{line}}}\right)$ I/Os whenever $K$ exceeds fast cache $M$).
2. **Empirical 32 KiB Tile-Transport & `Open<16>` Locality on the 6 Wikidata Blocks (`probe_cycle10.py`):**
   Partitioning $[0, n)$ into $Q = 28$ L1-sized `32 KiB` tiles ($A = \lfloor x / 32768\rfloor$, $B = \lfloor \tau(x) / 32768\rfloor$) and tracking `Open<16>`'s critical walk `edges[0..m)` (`16 B` per `Slot16`):

| Block | `32 KiB` Tile Scatter $H(B\mid A)/\log_2 Q$ | `Open<16>` $m$ | `edges` Footprint (`KiB`) | Consecutive `64 B` Cache-Line Jump Rate | Consecutive `32 KiB` L1-Page Jump Rate |
|---|---:|---:|---:|---:|---:|
| `block-0.L` | **0.9969** (`4.79 / 4.81 b`) | 89,874 | 1,404.3 KiB (L2) | **1.00000** (`100.00%`) | **0.9464** |
| `block-2.L` | **0.9964** (`4.79 / 4.81 b`) | 112,070 | 1,751.1 KiB (L2) | **1.00000** (`100.00%`) | **0.9562** |
| `block-85.L` | **0.9977** (`4.80 / 4.81 b`) | 156,404 | **2,443.8 KiB** (>2M L2) | **1.00000** (`100.00%`) | **0.9691** |
| `block-143.L` | **0.9955** (`4.79 / 4.81 b`) | 196,841 | **3,075.6 KiB** (>2M L2) | **1.00000** (`100.00%`) | **0.9647** |
| `block-210.L` | **0.9940** (`4.78 / 4.81 b`) | 204,281 | **3,191.9 KiB** (>2M L2) | **1.00000** (`100.00%`) | **0.9722** |
| `block-207.L` | **0.9959** (`4.79 / 4.81 b`) | 135,921 | **2,123.8 KiB** (>2M L2) | **1.00000** (`100.00%`) | **0.9622** |

Every `32 KiB` slice of $L$ scatters across all 28 output `32 KiB` slices at **`99.40%–99.77%` of maximum possible uniform entropy**, and **`100.00%` of consecutive critical hops in `Open<16>` jump to a new 64B cache line** (`94.6%–97.2%` jump out of a `32 KiB` L1d cache!).

---

## 2. Single-Puncture Offset Indistinguishability vs Coset Seed-Pool Orbit Decomposition (`SPOD-Coset512`)

A longstanding assumption in `.bz2` decompression is that because a block carries only **one** `primary_index` $p$, intra-block `emit` is inherently sequential (`mibz2` can only parallelise across distinct blocks, leaving $P - B$ cores idle on files with $B < P$ blocks or on tail stragglers).

### Theorem 10.2 (Pre-Walk Offset Indistinguishability & Post-Walk Zero-Bitvector Coset Splicing — *proved*; *verified* on `3,016` small checks + byte-exact on all 6 Wikidata blocks)

1. **Pre-Walk Offset Indistinguishability Witness ($n = 7$):** The two primitive binary BWTs $L_1 = \mathtt{1001000}$ and $L_2 = \mathtt{1100000}$ share identical $(n=7, c_0=5, p=0)$ and identical 2-step local $\mathrm{LF}$ neighbourhoods at both $p = 0$ and $q = 6$, yet $\tau_1(6) = 3 \ne 2 = \tau_2(6)$. No local inspection can place an unvisited seed $q$'s output into `out[0..n)` *before* walking.
2. **Escape via 2-Adic Coset Seed-Pool Orbit Decomposition (`SPOD-Coset512`):** You do **not** need $\tau(q)$ *before* walking! Choose a power-of-two stride $S = 2^b$ (`S = 512`) in `Open<16>`'s dense critical ID space $[0, m)$, defining the equidistributed coset seed pool
   $$\mathcal{S}_S := \{c \in [0, m) : (c \mathbin{\&} (S - 1)) = 0\} \cup \{c_p\}, \qquad K = |\mathcal{S}_S| = \lceil m / S\rceil + \mathbf{1}[512 \nmid c_p].$$
   - **Zero-Bitvector Parallel Walk:** $P$ worker threads dynamically claim seeds $s \in \mathcal{S}_S$, walking read-only `edges[0..m)` into thread-local scratch chunks and stopping each segment on the **single register-immediate test** `(cur & 511) == 0 || cur == cid_p` (**zero** visited bitvectors, **zero** atomic read-modify-writes on `edges`, and `99.8%` branch-not-taken predictability!).
   - **Exact $O(K)$ Post-Walk Cycle Stitching:** Because $(V_{\mathrm{crit}}, \mathrm{next})$ is a single $m$-cycle, the $K \approx 177\text{--}400$ segments partition $[0, m)$ cleanly (`sum(seg_hops) == m`) and induce a single $K$-cycle $\pi : \mathcal{S}_S \to \mathcal{S}_S$ starting at $c_p$. Walking $\pi$ in `< 1 µs` computes the exact output offset of every segment, followed by parallel `memcpy` (`~15–25 µs` across $P$ cores) into `out[0..n)`.

### Exact Work-Stealing Intra-Block Parallel Speedup of `SPOD-Coset512` (`probe_cycle10.py`)

| Block | $m$ (`Open<16>`) | Seeds $K$ (`S=512`) | Max Segment (`% of m`) | **`P = 4` Speedup** | **`P = 8` Speedup** | **`P = 16` Speedup** |
|---|---:|---:|---:|---:|---:|---:|
| `block-0.L` | 89,874 | 177 | `5.86%` | **3.89×** (`97.3%` eff) | **7.12×** (`89.0%` eff) | **12.15×** |
| `block-2.L` | 112,070 | 220 | `3.99%` | **3.99×** (`99.8%` eff) | **7.64×** (`95.5%` eff) | **14.23×** |
| `block-85.L` | 156,404 | 307 | `3.21%` | **3.97×** (`99.3%` eff) | **7.86×** (`98.3%` eff) | **15.28×** |
| `block-143.L` | 196,841 | 386 | `3.79%` | **3.98×** (`99.5%` eff) | **7.76×** (`97.0%` eff) | **14.78×** |
| `block-210.L` | 204,281 | 400 | `3.42%` | **3.95×** (`98.8%` eff) | **7.77×** (`97.1%` eff) | **15.11×** |
| `block-207.L` | 135,921 | 267 | `3.24%` | **3.99×** (`99.8%` eff) | **7.84×** (`98.0%` eff) | **15.26×** |

*(Note on scope: when `mibz2` already saturates all $P$ cores with $\ge P$ independent blocks, block-level parallelism has zero stitching overhead and remains optimal; `SPOD-Coset512` specifically unlocks **`3.9×–15.3×` intra-block scaling on short streams ($B < P$ blocks) and tail-block stragglers** without touching the `.bz2` format!)*

---

## 3. Complete 10-Cycle Mathematical Atlas & Escape Classification

Across Cycles 1–10, we confronted **over 25 mathematical frameworks** with exact theorems, small exhaustive verifiers (`>140,000` verified instances), C benchmarks, and full-corpus censuses on all 6 authenticated Wikidata blocks:

| Regime / Goal | Winning Mathematical Primitive (Escape) | Proved Structural Obstructions Ruling Out Rivals |
|---|---|---|
| **1. Single-Core Legacy `.bz2` (`mibz2` today)** | **Non-Commutative Fox Morse Reduction (`Open<16>`) + Coprime-Stride $\mathbb{Z}^{16}$ Queue Iota + Free-Monoid `Slot16` Emit (`Vec16`, Cycle 3 — shipped in `#668`, `0.922–0.947×` native wall)** | • **Universal Schur Optimality (Cycle 4):** `Open<16>` has `0` rows slack among all $H$-strip affine complements.<br>• **Ephemeral-Run Obstruction (Cycle 1):** explicit $O(r)$ run graphs (`TTSR-RV`, `DECO`, 2nd forest cut Cycle 3, FVS DAG pruning Cycle 9) pay `649–1,739 µs` > walk savings.<br>• **L2 Sheet & Morse Bounds (Cycles 2, 6):** `OIBS` loses `1.13–1.27×` in L2; Mrozek–Batko `Coreduce<16>` is 1-step $\mathrm{LF}$-conjugate to `Open<16>` (`identical |V_crit|`); Perseus apparent pairs leave `0.49–0.55n` cells.<br>• **Non-Walk Formulations (Cycles 6, 7, 9):** $\Delta\boldsymbol{\tau}$ shatters into `0.986–1.000n` runs; De Bruijn uniqueness needs $k^* \in [925, 10526]$; dyadic $\mathrm{LF}^{2^j}$ totals `12.9–14.6n` pieces; Fourier Cauchy rank $= r$; $\mathrm{SL}_2(\mathbb{F}_P)$ ratios hit `1.000n` values. |
| **2. Multi-Core Short-Stream / Tail-Straggler Legacy `.bz2` ($B < P$ blocks)** | **2-Adic Coset Seed-Pool Orbit Decomposition (`SPOD-Coset512`, Cycle 10 Theorem 10.2B):** `(cur & 511) == 0` register guard breaks a single `.bz2` block's `Open<16>` walk across 4–16 cores at **`3.89–3.99×` (`4t`), `7.12–7.86×` (`8t`), `12.15–15.28×` (`16t`)** with 0 bitvectors & 0 format changes | • **Single-Puncture Indistinguishability (Cycle 10):** pre-walk placement $\tau(q)$ is unidentifiable ($n=7$ witness).<br>• **Strict Origami Primitivity (Cycle 5):** `0` intermediate Galois covers (`LF(x+m) ≡ LF(x) mod m` fails for all $m \mid n$ on 6/6 blocks). |
| **3. Format-Changeable BWT Pipelines (STARLIT / Next-Gen `bz3`/Archivers)** | **Quote-Free Syntactic Skew-Product Splitting (`Pre-HexTok`, Cycle 8 Theorem 8.2):** losslessly splits $\ge 16$-byte `[0-9a-fA-F-]` spans into $(\overline{T}, \boldsymbol{\xi}_{\mathrm{text}})$ *before* `BWT`, cutting $r$ to **`0.419×`**, `Open<16>` $m$ to **`0.541×`** (pulling all 6 blocks inside 2 MiB L2!), Huffman stream to **`0.468×`**, and aggregate compressed size to **`0.949×`** (`0.842–0.889×` on UUID-heavy blocks) | • **Functorial Non-Commutativity (Cycle 8):** $\pi(\mathrm{BWT}(T)) \ne \mathrm{BWT}(\pi(T))$; post-BWT $\pi_{\mathrm{hex}}(L)$ leaves `296K–326K` 4-bit fiber rank/pointer scatters.<br>• **Bayer–Diaconis UUID Saturation (Cycle 5):** unsorted UUIDs shatter at $(15/16)^2 = 87.89\%$ width-1 and hit white noise in 3 $\mathrm{LF}$ steps. |
