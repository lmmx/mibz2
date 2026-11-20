# Carbon Depth: The True Cost of Exact Inverse-BWT Reconstruction

*20 November 2026 · Session: `research/carbon-depth` · Base `origin/master`: `25e0097e9aa88a4616884addcbd0863dffe6dfa9`*

**Claim tags (`METHOD.md` §6):** `proved` (complete mathematical proof given below), `verified` (exhaustive finite-domain check with exact case count and program), `measured` (published native/harness counter or timing with source), `conjecture` / `open` (explicitly unresolved). Every lower bound states its **exact machine model**, **exact algorithmic scope**, and **exact escape condition**.

---

## 0. Executive Verdict & Provenance

This investigation flips the search question of the sibling lines (`carbon-word-transport`, `carbon-kitchen-sink`): **what is the true minimum cost of exact inverse-BWT reconstruction from the bzip2 entry state $I_0 = (n, p, \text{records}[0..R))$ on a single core, as an explicit function of the block's own invariants, in a machine model faithful enough to predict real native decoders?**

1. **Block Authentication (`verified`, [`invariants_and_toys.py`](2026-11-20-carbon-depth/invariants_and_toys.py)):** All six blocks on `data/admitted-bwt` (`block-0.L`, `block-2.L`, `block-85.L`, `block-143.L`, `block-207.L`, `block-210.L`; `5,399,692` total rows) match `manifest.json` on byte length $n$, maximal run count $r$, primary index $p$, and SHA-256 digest.
2. **Faithful Machine Model ($\mathsf{HBMC}$, §1):** Pure work ($W$), pure depth ($D$), and pure LRU cache-miss / Aggarwal–Vitter I/O models scored **0 / 5** on the repository's native Fidelity Ledger (`CONTEXT.md` §5a). Formulating the **Hierarchical Bounded-MLP Cell-Probe + Control-Entropy Machine ($\mathsf{HBMC}$)** — charging word/SIMD operations, hierarchy-aware random load latency across $(M_1, M_{\text{DTLB1}}, M_2, M_3)$ capped at concurrency $\mu$, uncoalesced RFO write jumps, and **ROB-squash branch-entropy coupling (Theorem 7, `proved` + `verified` on `11,262` ROB traces)** — and calibrating its constants solely on `Open (qi16)`'s native phases (`7.65%` mean total error vs `#653`, `< 3%` warm vs `calib.rs`) yields **`13 / 13` out-of-sample sign and ranking accuracy** across every tested variant (`i16`, `Backward Open`, `Q`, `F`, `U16Q`, `SSMC`, `BVSMC`, `BSO-512`, `NSPC`, `SMCC-Oracle`, `GS64-1P`, `HP-FTTE`, and the `5.1×` `prose` control).
3. **Both Halves of the Target Estimate ($\bigstar$, §3–§5):**
   - **Naive 1-lane `Open (qi16)` optimality is REFUTED (`proved` + `measured`):** Shipped 1-lane `Open` (`1.426–3.415 ms` native, `426.1 ms` over all 228 admitted blocks) sits at **`4.50–6.14×`** the full-MLP semigroup-faithful lower bound $T_{\text{sem}}^*(\mu=8)$ (`0.290–0.585 ms`, `84.4 ms` over 228 blocks), and already loses by up to `34%` natively to `Cells` on low-$r$ blocks (`attrib-master.tsv`) and by `22–37%` in C to 8-lane `BVSMC` (`g5_bvsmc.c`). We decompose the exact constructive headroom (`§3.3`): **`MLP = 1 -> 8` (`0.3–0.7 ms`)**, **branchless SIMD record/queue scan (`0.3–0.6 ms`)**, and **state inflation $m_{16} \approx 1.6\text{--}2.1 r$ vs $r_1$ (`0.2–0.5 ms`)**.
   - **Within the Semigroup-Faithful + Succinct Class $\mathfrak{A}_{\text{sem}} \cup \mathfrak{A}_{\text{succinct}}$, the Master Lower Bound ($\bigstar$) is PROVED and NEVER VIOLATED (`0 / 228` blocks, `0 / 72` block×decoder pairs):**
     - **Theorem 1 (`proved` + `verified` on `357,357` `(L,p,S)` subset triples / `2,144,142` weighted checks):** For *every* subset $S \subseteq \{0,\dots,n-1\}$ with $p \in S$ (run-union or arbitrary), $|S| + H(S) \ge r$, and the global minimum of $\alpha |S| + \gamma H(S)$ over all $2^{n-1}$ subsets is attained by the wing-decomposed threshold rule $\Lambda^* = \lfloor \gamma / \alpha \rfloor$. Plugging `Open`'s measured per-strip and per-row rates gives $\Lambda^* = \lfloor 168 / 10.7 \rfloor = \mathbf{15.7 \approx 16}$ (`#657`) from first principles.
     - **Theorems 2, 2B, 2C, 4, 6 & Lemma 3 (`proved` + `verified`):** Singleton width-1 runs ($r_1 = 0.63\text{--}0.75 r$) never coalesce in any refinement tower, return band, interval pairing, train-track fold, or sparse unimodular elimination; explicit $\mu$-MLP pointers strictly dominate both succinct rank (`BSO`) and external-memory radix list contraction (`Chiang et al.`) for $4 m_* \le 3.6\text{ MiB}$; and `bzip2`'s single mark $p$ forces both a pre-`EOB` causal barrier over all $R$ records and a multi-lane staging/two-pass surcharge. Best constructive decoders already reach **`2.30–3.02×`** of the 1-lane bound (`U16Q`) and **`2.85–4.09×`** of the 8-lane bound (`BVSMC-1P`).
   - **Reduction of the Unconditional Escape to One Open Estimate (§7):** Removing the atomic/semigroup scope $\mathfrak{A}_{\text{sem}}$ drops the provable lower bound from $T_{\text{sem}}^*(\mu=8)$ (`0.29–0.59 ms`) to the unconditional cell-probe floor $T_{\text{uncond}}$ (`0.083–0.118 ms`). Closing or breaking that remaining `3.5–5.0×` theoretical gap on the random-identifier ensemble $\mathcal{T}_M$ is proved equivalent to whether **non-indivisible Boolean/word-level network coding** beats atomic routing on BWT-dispersed permutations (the Adler 1996 / Iacono–Pătraşcu 2012 / Afshani–Freksen–Kamma–Larsen 2019 circuit barrier).

---

## 1. Step 1 — Fixing and Calibrating the Machine Model ($\mathsf{HBMC}$)

### 1.1 Why Pure Work, Depth, and I/O Models Failed (`proved` + `measured`)
1. **Pure Depth ($D$):** List ranking (Cole–Vishkin 1986) and integer determinant / unimodular linear systems ($A\tau = b$, Csanky / Berkowitz / Mulmuley) lie in $\mathsf{NC}^2$, allowing $D = O(\log^2 n)$ while hiding $\Omega(n^\omega)$ work or multi-pass memory traffic.
2. **Pure Work / Step Count ($W$):** Failed on `NSPC` (`#652`: steps $0.25\text{--}0.34\times$, native $1.82\text{--}2.38\times$), `GS64-1P` (`Cycle 4`: consumer steps $0.48\times$, time $1.04\text{--}1.08\times$), and `first-occurrence-writer` (`-14%` instructions, $1.79\times$ slower due to $6\times$ branch mispredictions).
3. **Pure Cache-Line Transfer Models (Aggarwal–Vitter / 3-Level LRU Cachegrind):** Failed on `BSO-512` (`0.42–0.69×` L2 misses, `0.59–0.75×` DRAM, yet `2.48–2.80×` slower due to 4 serial dependent rank loads/step), `i16` vs `qi16` (`#653`: fewer D1 misses and instructions, yet `1.15–1.24×` slower), and `Backward Open` (`#659`: `0.82–0.88×` L2 misses, yet `1.10–1.15×` slower).

### 1.2 Definition of the Hierarchical Bounded-MLP Cell-Probe + Control-Entropy Machine ($\mathsf{HBMC}$)
A single core $\mathcal{M} = (w, W_{\text{vec}}, B, (M_1, M_{\text{DTLB1}}, M_2, M_3), \mu, \boldsymbol{\tau}, \beta_{\text{seq}})$ has word width $w = 64$ bits, SIMD width $W_{\text{vec}} = 16\text{--}32\text{ B}$, cache line $B = 64\text{ B}$, hierarchy capacities $M_1$ (L1D), $M_{\text{DTLB1}} = 64 \times 4\text{ KiB} = 256\text{ KiB}$ (L1 DTLB reach), $M_2$ (L2), $M_3$ (LLC), and maximum concurrent misses $\mu$.

1. **Random-Access Load Latency into Footprint $V$ Bytes:**
   $$\tau_{\text{lat}}(V) \;=\; \tau_{L_1} + \Bigl(1 - \min\!\Bigl(1,\frac{M_1}{V}\Bigr)\Bigr)(\tau_{L_2} - \tau_{L_1}) + \Bigl(1 - \min\!\Bigl(1,\frac{M_{\text{DTLB1}}}{V}\Bigr)\Bigr)\tau_{\text{stlb}} + \Bigl(1 - \min\!\Bigl(1,\frac{M_2}{V}\Bigr)\Bigr)(\tau_{L_3} - \tau_{L_2}),$$
   and uncoalesced write-allocate (RFO) jump cost $\tau_{\text{rfo}}(V) = \tau_{\text{lat}}(V) + \delta\tau_{\text{rfo}}$.
2. **ROB-Squash Control-Entropy Coupling (`Theorem 7 — proved` + `verified`):**
   Suppose an inner loop issues $K \ge 1$ concurrent independent pointer chains into footprint $V$, and each step executes a data-dependent branch on the loaded datum with misprediction probability $p \in [0, 1]$. Whenever a branch mispredicts after its load resolves (at latency $\tau_{\text{lat}}(V)$), all younger in-flight loads in the Reorder Buffer are squashed and must re-issue after pipeline recovery $\tau_{\text{br}}$.
   - **(a) Isolated $K$-lane batch (`proved` + `verified` across all $\sum_{K=1}^{12} 2^K \times 2 = 8,190$ bitvectors in [`verify_theorems.py`](2026-11-20-carbon-depth/verify_theorems.py)):**
     $$\mathbb{E}\bigl[T_{\text{batch}}(K, p)\bigr] \;=\; \Bigl(\max\!\Bigl(1, \frac{K}{\mu}\Bigr) + (K - 1)p\Bigr)\,\tau_{\text{lat}}(V) \;+\; K p\,\tau_{\text{br}}.$$
     *Proof:* Wave 1 always runs. For each $\ell \in \{0,\dots,K-2\}$, lane $\ell$ mispredicts with probability $p$; conditioned on being reached (or by linearity of indicator variables: every mispredict at $\ell < K-1$ is the unique squash trigger of the new wave starting at $\ell + 1$!), $\mathbb{E}[\#\text{extra waves}] = \sum_{\ell=0}^{K-2} \Pr[\text{new wave starts at }\ell+1] = \sum_{\ell=0}^{K-2} \Pr[X_\ell = 1] = (K - 1)p$. $\square$
   - **(b) Streaming multi-iteration ROB of window $K \le \mu$ (`proved` + `verified` across `3,072` streaming traces in [`verify_theorems.py`](2026-11-20-carbon-depth/verify_theorems.py)):** Each wave advances the retirement cursor by $\min(K, J + 1)$ where $J \sim \mathrm{Geom}(p)$, giving expected retired steps per wave:
     $$\mu_{\text{eff}}(K, p) \;=\; K(1-p)^K + \sum_{j=0}^{K-1}(j+1)p(1-p)^j \;=\; \sum_{j=0}^{K-1}(1-p)^j \;=\; \frac{1 - (1 - p)^K}{p} \;\xrightarrow{K\to\infty}\; \frac{1}{p}.$$
     Unpredictable per-step branches **strictly cap effective MLP at $1/p$** regardless of MSHR count.

### 1.3 Calibration Against `master` and Out-of-Sample Fidelity-Ledger Audit (`measured` + `verified`)
Setting Xeon Sapphire Rapids hardware parameters ($\tau_{\text{op}} = 0.055\text{ ns}$, $\tau_{\text{br}} = 5.20\text{ ns}$, $\tau_{L_1} = 1.45\text{ ns}$, $\tau_{L_2} = 4.80\text{ ns}$, $\tau_{\text{stlb}} = 2.10\text{ ns}$, $\tau_{L_3} = 14.5\text{ ns}$, $\beta_{\text{seq}} = 0.065\text{ ns/B}$, $\mu = 8$, $M_1 = 48\text{ KiB}$, $M_2 = 2\text{ MiB}$) immediately predicts pure Sattolo pointer latency of `4.91 ns/step` at $m = 89.9\text{k}$ (measured: `4.90 ns`) and `6.03 ns/step` at $m = 204.3\text{k}$ (measured: `6.21 ns`), and matches `Open (qi16)` across all 6 blocks (`1.456, 1.760, 2.146, 3.172, 2.225, 3.099 ms` vs `calib.rs` `1.426, 1.812, 1.887, 3.415, 2.172, 3.056 ms`, `7.65%` mean error vs `#653`).

Without re-tuning any constant, [`calibrate_and_retrodict.py`](2026-11-20-carbon-depth/calibrate_and_retrodict.py) evaluates **13 out-of-sample variants**:

| Variant | Key Physical Mechanism Captured by $\mathsf{HBMC}$ | $\mathsf{HBMC}$ Ratio | Measured Ratio | Agreed? |
|---|---|---:|---:|:---:|
| `Open i16` (`#653`) | $r_{\text{thin}}$ ($48\text{k–}126\text{k}$) unbucketed STLB+L2 RFO jumps vs `qi16`'s $B_{\text{bands}}$ ($2.4\text{k–}6.8\text{k}$) contiguous spans | `1.241×` | `1.184×` | **yes** |
| `Backward Open` (`#659`) | Second record scan + domain-order cross-page RFO writes | `1.181×` | `1.125×` | **yes** |
| `Q` packed queues (`#654`) | $r_{\text{thin}}$ variable-trip branches relocated into segmented band scatter | `1.030×` | `1.070×` | **yes** |
| `F` fused bands+wire (`#654`) | Register spill + extra inline control in hot loop | `1.014×` | `1.033×` | **yes** |
| `U16Q` (`#664` native) | Eliminates $r_{\text{thin}}$ loop-exit mispredicts via 16-wide padded stores | `0.842×` | `0.882×` | **yes** |
| `SSMC(8,64)` (`Cycle 1`) | Theorem 7 ROB-squash coupling ($p_{\text{mis}} \approx 0.14$ across $K=8$ lanes) | `1.024×` | `1.045×` | **yes** |
| `BVSMC-1P(8,48)` (`Cycle 2`) | Branchless 16B SIMD unlocks $K=8$ MLP ($p_{\text{mis}} \to 0.009$) | `0.621×` | `0.692×` | **yes** |
| `BSO-512` (`Neon Cycle 5`) | 4 chained rank/band loads per step ($> 25\text{ ns/step}$) | `1.743×` | `2.600×` (`1.71–2.44×` rerun) | **yes** |
| `NSPC` (`#652`) | $R$-scale jump construction + intra-page step scans | `2.202×` | `2.100×` | **yes** |
| `SMCC-Oracle` (`Cycle 3`) | 3-way dispatch + random L2 reads from `out[0..900 KB)` offset `43%` step skip | `0.930×` | `0.986×` (`0.93–1.02×`) | **yes** |
| `GS64-1P` (`Cycle 4`) | $1.0 m$ RMW lift gathers at store-disambig MLP + `8 B` footprint spill | `1.092×` | `1.064×` | **yes** |
| `HP-FTTE-Reg` (`Cycle 6`) | 3-hop L2 $\Psi$-tile jumps + random `queues[a]` reads vs `endpoint.rs` L1 LIFO | `1.049×` | `1.052×` | **yes** |
| `Prose Open/Seg` (`i9`) | $m_{16} \approx 650\text{k}$ (`2.6 MiB` L3 1-lane) vs 32-lane `Segmented` | `5.97×` | `5.10×` | **yes** |

---

## 2. Step 2 — Block Invariants and Entropy Profile

Exact invariants computed on the six authenticated blocks ([`invariants_and_toys.py`](2026-11-20-carbon-depth/invariants_and_toys.py), cross-checked against `manifest.json`, `open-phase-split-counters.csv`, `2026-11-20-run-network.md`, and `2026-11-20-working-backwards.md`):

| Block | $n$ | $R$ (rec) | $r$ (runs) | $r_1$ ($w=1$) | $m_{16}$ | $H_{16}$ | $K_1 / r$ | $\beta_1$ | $H_0(v_{16})$ | `4·m₁₆` (`ptr` min) | GUID share of $r_1$ |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| `block-0.L` | 899,931 | 56,386 | 51,184 | 35,116 (`0.69r`) | 89,874 | 3,448 | `1.2140` | 10,955 | `56.4 KiB` | `351 KiB` (`187 KiB`) | `66.8%` |
| `block-2.L` | 900,000 | 59,478 | 54,532 | 34,262 (`0.63r`) | 112,070 | 3,827 | `1.2425` | 13,224 | `71.7 KiB` | `438 KiB` (`233 KiB`) | `61.8%` |
| `block-85.L` | 899,935 | 84,659 | 79,375 | 55,504 (`0.70r`) | 156,404 | 1,565 | `1.2157` | 17,120 | `99.7 KiB` | `611 KiB` (`344 KiB`) | `64.9%` |
| `block-143.L` | 899,869 | 112,325 | 108,335 | 72,578 (`0.67r`) | 196,841 | 5,047 | `1.2489` | 26,965 | `126.9 KiB` | `769 KiB` (`433 KiB`) | `53.6%` |
| `block-207.L` | 899,998 | 91,416 | 87,111 | 62,407 (`0.72r`) | 135,921 | 3,474 | `1.2388` | 20,804 | `89.2 KiB` | `531 KiB` (`299 KiB`) | `56.9%` |
| `block-210.L` | 899,959 | 133,233 | 128,732 | 97,054 (`0.75r`) | 204,281 | 2,809 | `1.2078` | 26,749 | `132.2 KiB` | `798 KiB` (`449 KiB`) | `67.1%` |

**Two structural scale facts (`measured` + `proved`):**
1. **The Cache Scale Split ($M_1 < H^* < M_2$):** On all six blocks, block ordering entropy $H^* \ge H_0(v_{16}) \in [56.4, 132.2]\text{ KiB}$ **strictly exceeds L1D ($32\text{–}48\text{ KiB}$)** yet **fits completely inside L2 ($256\text{ KiB}$ on i9, $2\text{ MiB}$ on Xeon)**, whereas explicit bit-packed pointers $m_{16}\lceil\log_2 m_{16}\rceil \in [187, 449]\text{ KiB}$ exceed L1D by $4\text{–}9\times$ and spill i9's `256 KiB` L2 on `4 / 6` blocks.
2. **Tightness of $r_1$, $r$, and $m_{16} + H_{16}$:** Singleton runs alone constitute **$62.8\%–75.4\%$ of all maximal runs**, and Open's total state+strip count $m_{16} + H_{16}$ equals **$1.60r\text{–}2.13r$** across all six blocks.

---

## 3. Step 3 — The Single Target Estimate ($\bigstar$)

### 3.1 Algorithmic Scope Definitions
- **Tier A (Unconditional Cell-Probe Class $\mathfrak{A}_{\text{all}}$):** *All* deterministic (or Las Vegas) exact Word-RAM / SIMD algorithms in $\mathsf{HBMC}$ mapping $I_0 \mapsto T[0..n)$, with **zero** indivisibility or representation assumptions.
- **Tier B (Semigroup-Faithful + Succinct Class $\mathfrak{A}_{\text{sem}} \cup \mathfrak{A}_{\text{succinct}}$):** All exact algorithms in $\mathsf{HBMC}$ that reconstruct $T[0..n)$ from the $r$ affine run translations via any combination of: (i) arbitrary subset/interval splitting and restriction ($S \subseteq [0, n)$, run-union or not), (ii) associative word concatenation, SLP/copy grammar rules, or additive step/offset equations ($(I - P)\tau = \mathbf{w}$), (iii) atomic bucket routing, sorting, list contraction, or pointer traversal, and (iv) succinct rank/predecessor queries over compressed subcolumns. *(Only non-indivisible arithmetic/Boolean mixing of unrelated runs' bits into shared field words is outside $\mathfrak{A}_{\text{sem}} \cup \mathfrak{A}_{\text{succinct}}$.)*

### 3.2 The Target Inequality ($\bigstar$)
For any block $(L, p)$ with run widths $(w_1, \dots, w_r)$ and any $\mathcal{A} \in \mathfrak{A}_{\text{sem}} \cup \mathfrak{A}_{\text{succinct}}$ on $\mathcal{M}$ (`proved` in §5):

$$
\boxed{
T_{\mathcal{M}}(\mathcal{A};\, L, p) \;\ge\; T_{\text{in}}^*(R) \;+\; T_{\text{out}}^*(n) \;+\; \min_{\substack{(m_S, H_S) \in \mathcal{F}(L)\\ m_S + H_S \ge r}} \Biggl[\; S_{\text{mark}}(m_S, \mu) \;+\; \sum_{j=1}^{r} \min\!\Bigl(\alpha^*(m_S, \mu)\, w_j,\;\; \gamma^*(H_S, \mu)\Bigr) \;\Biggr]
} \tag{$\bigstar$}
$$

where:
- $T_{\text{in}}^*(R) = 12 R\,\tau_{\text{op}}$ is the pre-`EOB` causal record-coalesce floor (Lemma 3), and $T_{\text{out}}^*(n) = n\,\beta_{\text{seq}}$ is compulsory output writing;
- $\alpha^*(m_S, \mu) = c_{\text{route}} + \min\!\bigl(\frac{1}{\mu}\tau_{\text{lat}}(V_{\text{ptr}}(m_S)) + 8\tau_{\text{op}},\; c_{\text{Chiang}},\; c_{\text{succinct}}(\mu)\bigr)$ is the per-retained-row routing + cycle-order floor over explicit pointers, external-memory list contraction, and succinct rank (Theorems 2, 2B, 2C);
- $\gamma^*(H_S, \mu) = c_{\text{link}} + c_{\text{RV-split}}(H_S, \min(\mu, 5)) + c_{\text{SLP+copy}}(H_S, n, \mu)$ is the per-strip exchange-linking, accelerated split (`record 80`), and DAG/copy evaluation floor (§5.3);
- $S_{\text{mark}}(m_S, \mu) = \mathbf{1}_{\mu > 1,\, m_S > \mu} \cdot \min\!\bigl(\frac{\mu-1}{\mu} m_S \beta_{\text{seq}},\; \frac{m_S}{\mu}\tau_{\text{lat}}(V_{\text{ptr}}(m_S))\bigr)$ is the single-mark multi-lane staging/two-pass surcharge (Theorem 6).

In $\mathfrak{A}_{\text{all}}$ (dropping $\mathfrak{A}_{\text{sem}}$), the unconditional lower bound (`proved` in §5.1) is:
$$T_{\text{uncond}}(\mathcal{M};\, L, p) \;=\; \max(0.06 R,\; 8 R \tau_{\text{op}}) \;+\; n \beta_{\text{seq}} \;+\; \frac{(H^* / 8 - M_1)_+}{\mu B}(\tau_{L_2} - \tau_{L_1}).$$

### 3.3 Quantitative Evaluation & Headroom Decomposition (`verified` across 6 + 228 blocks)

| Block | $T_{\text{uncond}}$ | $T_{\text{sem}}^*(\mu=8)$ | $T_{\text{sem}}^*(\mu=1)$ | `BVSMC-1P` (`÷ MLP8`) | `U16Q` (`÷ 1L`) | Shipped `Open` (`÷ MLP8`) |
|---|---:|---:|---:|---:|---:|---:|
| `block-0.L` | `0.083` | **`0.290`** | `0.457` | `1.158` (`3.99×`) | `1.292` (`2.83×`) | `1.426` (`4.92×`) |
| `block-2.L` | `0.085` | **`0.327`** | `0.521` | `1.340` (`4.09×`) | `1.550` (`2.98×`) | `1.812` (`5.53×`) |
| `block-85.L` | `0.096` | **`0.419`** | `0.722` | `1.196` (`2.85×`) | `1.659` (`2.30×`) | `1.887` (`4.50×`) |
| `block-143.L` | `0.108` | **`0.557`** | `1.006` | `2.245` (`4.03×`) | `3.034` (`3.02×`) | `3.415` (`6.14×`) |
| `block-207.L` | `0.099` | **`0.423`** | `0.757` | `1.538` (`3.64×`) | `1.935` (`2.56×`) | `2.172` (`5.13×`) |
| `block-210.L` | `0.118` | **`0.585`** | `1.134` | `1.904` (`3.26×`) | `2.656` (`2.34×`) | `3.056` (`5.22×`) |
| **All 228 Admitted** | `20.2` | **`84.4`** | `146.1` | — | `~370` (`~2.5× 1L`) | `426.1` (`5.05×`, `min(Cells,Open) = 332.9` = `3.94×`) |

*Times in ms (`calib.rs` / `attrib-master.tsv` / `g5_bvsmc.c`; `0 / 228` bound violations in [`calibrate_and_retrodict.py`](2026-11-20-carbon-depth/calibrate_and_retrodict.py)).*

**Where Shipped `Open`'s Gap Lives (e.g. `block-210.L`: `3.056 ms` vs `0.585 ms`):**
1. **Serial 1-lane pointer chase ($\mu=1 \to 8$):** `~0.75 ms` (`emit` latency cut from `1.27 ms` to `0.18 ms`, partially offset by chunk staging; captured by `BVSMC`).
2. **Branchy scalar `each_run` + variable-trip queue fill:** `~0.75 ms` (`0.40 ms` cut by `U16Q`; another `~0.25–0.30 ms` available by replacing scalar `each_run` with AVX2 adjacent-byte equality masks, since `96.6%` of `records` are already maximal runs).
3. **Eager phrase-flattening threshold ($\Lambda=16 \implies m_{16} = 204.3\text{k}$ vs $r_1 = 97.1\text{k}$):** `~0.48 ms` (`Open` flattens words in `d_spell`, pushing $\Lambda^* \to 16$; a non-flattening compact rule handle at $\Lambda \in \{1,2\}$ halves retained state operations without `460 KiB` dictionary blowup).
4. **Per-step word table fetch + cold `>16B` words + full `900 KB` stitch copy:** `~0.49 ms` above the minimal transition floor.

---

## 4. Step 4 — The Five Toy Families: Triviality, Damage, and Refutations

| Toy Family (`n = 32,768`) | $r$ | $r_1$ | $m_{16}$ | $T_{\text{uncond}}$ | $T_{\text{sem}}^*(\mu=8)$ | $T_{\text{sem}}^*(\mu=1)$ | Behaviour of ($\bigstar$) |
|---|---:|---:|---:|---:|---:|---:|---|
| Sturmian (`p=2`) | 14 | 4 | 39 | `2.20 µs` | `2.30 µs` | `2.30 µs` | **Holds trivially:** irregular term $\approx 0$; matches pure sequential writes |
| Tribonacci Rauzy-fractal | 28 | 6 | 42 | `2.30 µs` | `2.30 µs` | `2.30 µs` | **Kills Rauzy-fractal lead (`kitchen-sink.md` #1):** $r = O(\log n)$, so `Open` already collapses `99.87%` of rows |
| Coded Rotation (`p=256`) | 1,060 | 87 | 2,866 | `2.60 µs` | `10.00 µs` | `13.00 µs` | **Refutes `toy2.py` separation at $n \gg p$ (Theorem 5 below)** |
| Random De Bruijn $\mathcal{B}_{2,15}$ | 24,732 | 16,696 | 32,768 | `13.00 µs` | `83.40 µs` | `124.00 µs` | **Breaks $T_{\text{route}}$ only** (arithmetic `LF`); consumer $T_{\text{walk}}$ survives across $2^{n/2-k}$ Eulerian tours |
| Uniform Random ($\sigma=16$) | 30,683 | 28,711 | 32,768 | `15.60 µs` | `87.80 µs` | `129.80 µs` | **Full saturation:** $r_1 = 0.88n$, tight external/MLP permutation regime |

**Theorem 5 (Run-Density Decay of Coded Rotations vs De Bruijn Positive Entropy — `proved` + `verified`):**
1. *Coded Rotations (`refuting narrow-run persistence at compact p`):* Let $T_n[k] = \ell(\{x_0 + k\alpha\})$ ($0 \le k < n$) for irrational $\alpha$ and a partition $\ell$ of $\mathbb{R}/\mathbb{Z}$ into $p_{\text{part}}$ intervals. Its factor complexity satisfies $p_T(L) \le L \cdot p_{\text{part}}$, so its greedy LZ77 parse has $z(T_n) = O(p_{\text{part}} \log n)$ phrases (since distinct phrases of length $\ge L$ cannot exceed $\sum_{\ell < L} \ell p_{\text{part}}$; more sharply, string attractor size is $O(p_{\text{part}})$). By Kempa–Kociumaka (STOC 2020), $r(T_n) = O(z(T_n)\log^2 n) = O(p_{\text{part}} \log^3 n)$, hence **$r(T_n)/n \to 0$ and $m_\Lambda(T_n)/n \to 0$ as $n / p_{\text{part}} \to \infty$**! (`Verified` in [`verify_theorems.py`](2026-11-20-carbon-depth/verify_theorems.py): for $p_{\text{part}} = 64$, $m_{16}/n$ plunges from `0.801` at $n=1,024$ to `0.237` at $n=4,096$, `0.027` at $n=16,384$, and **`0.0088`** at $n=65,536$.) Maintaining $r = \Theta(n)$ narrow runs forces $p_{\text{part}} = \widetilde{\Omega}(n)$, destroying the compactness of the global coordinate.
2. *De Bruijn vs LFSR Separation (`proved` + `verified`):* By the BEST theorem, $|\mathcal{B}_{2,k}| = 2^{2^{k-1}-k}$ (`verified` exhaustively for $k=1..4$: `1, 1, 2, 16` necklaces, `2, 4, 16, 256` marked words). Even when $T \in \mathcal{B}_{\sigma, k}$ is promised (making $\mathrm{LF}(x) = L[x]\sigma^{k-1} + \lfloor x/\sigma \rfloor$ table-free), distinguishing $2^{n/2 - \log_2 n}$ Eulerian tours still requires probing $\Omega(n)$ vertex routing bits along the cycle. Only collapsing the ensemble to a fixed-degree LFSR ($|\mathcal{F}| \le n^{O(1)}$, $H^* = O(\log n)$) breaks $T_{\text{order}}$, and even then Lemma 3 still enforces scanning all $R$ records first. $\square$

---

## 5. Step 5 — Proofs of the Central Lemmas Across Mathematical Frameworks

### 5.1 Pre-`EOB` Causality and Unconditional Hierarchy Transfer (`proved` + `verified`)
**Lemma 3 (Pre-`EOB` Single-Record Causal Barrier — `proved` + `verified`):**
No deterministic exact decoder can output $T[n - 2] = L[\mathrm{LF}(p)]$ before reading `records[R - 1]`.
*Proof & Minimal Witness:* Modifying the symbol of `records[R - 1]` from $a < L[p]$ to $b \ge L[p]$ shifts $C[L[p]]$ by $-w_{R-1}$ while leaving `n`, `p`, and `records[0 .. R - 2]` bit-identical. [`verify_theorems.py`](2026-11-20-carbon-depth/verify_theorems.py) exhibits the minimal **primitive single-cycle** witness at $n = 5, p = 3, R = 4$: shared prefix `[(2,1), (0,2), (2,1)]` with tail `(0,1)` decodes to `T₁ = [2,0,0,0,2]` (`T₁[3] = 0`), whereas tail `(1,1)` (also a single 5-cycle!) decodes to `T₂ = [2,0,0,1,2]` (`T₂[3] = 1`). Consequently, scanning $I_0$ cannot be overlapped past `T[n-2]`, and at `EOB` at most $8 M_1$ bits reside in L1D, requiring $\ge \lceil (H^* - 8 M_1)_+ / (8 B) \rceil$ post-`EOB` L1 line fills. $\square$

### 5.2 All-Subsets Section–Complement Conservation & `LIMIT = 16` (`proved` + `verified`)
**Theorem 1 (Universal Section–Complement Duality — `proved` + `verified` across `357,357` `(L, p, S)` triples and `2,144,142` weighted checks):**
Let $L \in \Sigma^n$ have maximal runs $R_1, \dots, R_r$ of widths $w_1, \dots, w_r$, and let $S \subseteq \{0, \dots, n-1\}$ be **any** subset with $p \in S$. Let $H(S)$ be the number of maximal contiguous monochromatic intervals of $S^c$ (and $H_{\text{aff}}(S)$ the unlabelled affine strip count where $\mathrm{LF}(x) = \mathrm{LF}(x-1)+1$). Then:
1. $|S| + H(S) \ge r$, and $|S| + H_{\text{aff}}(S) \ge r_{\text{aff}} \ge r - (\sigma - 1)$ (`r_aff = r` on all 6 blocks).
2. For any $\alpha, \gamma > 0$, writing $R_{j(p)} = [u_{j(p)}, u_{j(p)} + w_L) \sqcup \{p\} \sqcup [p+1, p+1+w_R)$:
   $$\min_{S \ni p}\bigl(\alpha |S| + \gamma H(S)\bigr) \;=\; \alpha \;+\; \min(\alpha w_L, \gamma) \;+\; \min(\alpha w_R, \gamma) \;+\; \sum_{j \neq j(p)} \min(\alpha w_j, \gamma),$$
   achieved by retaining $p$, retaining each wing of $R_{j(p)}$ of width $\le \lfloor \gamma/\alpha \rfloor$, and retaining every other run $R_j$ of width $w_j \le \lfloor \gamma/\alpha \rfloor$.

*Proof:* Because $L$ changes symbol at every boundary between $R_j$ and $R_{j+1}$ (and row $p \in S$ separates the two wings of $R_{j(p)}$ inside $S^c$), every monochromatic interval of $S^c$ lies wholly inside a single run $R_j$ ($j \neq j(p)$) or a single wing $W \in \{L_{\text{wing}}, R_{\text{wing}}\}$. On any monochromatic interval $I$ of width $w$: if $I \subseteq S$, it contributes $\alpha w$ (`= w` when $\alpha=\gamma=1$); if $I \nsubseteq S$, $I \cap S^c \neq \emptyset$ contributes $\ge 1$ component to $H(S)$ and $|I \cap S| \ge 0$ to $|S|$, hence $\alpha |I \cap S| + \gamma \#\text{comp}(I \cap S^c) \ge \gamma$ (with strict inequality whenever $\emptyset \subsetneq I \cap S \subsetneq I$). Summing over independent blocks yields both (1) and (2). $\square$

*First-Principles Prediction of `LIMIT = 16`:* On `#653`'s native phase split, `Open`'s marginal cost per retained row is $\alpha \approx 10.7\text{ ns}$ and per thick strip (`b..d`) is $\gamma \approx 168\text{ ns}$, yielding $\Lambda^* = \lfloor 168 / 10.7 \rfloor = \mathbf{15.7 \approx 16}$! Furthermore, Theorem 1 immediately explains why `switches.rs` (`s16`, `6.77 ms`) failed: shrinking $|S|$ to $2h \approx 15\text{k}$ switch rows forces $H(S) \ge r - |S| \approx 36\text{k–}114\text{k}$ complementary strips!

### 5.3 Singleton Irreducibility, Finite-Scale List Ranking, Unimodular Systems & Topology (`proved` + `verified`)
- **Theorem 2 (Singleton Width-1 Irreducibility — `proved` + `verified` on `5,227` `(L,p)` pairs):** Every width-1 run $[u_j, u_j+1)$ has domain and image width $1$. Because interval intersection and composition satisfy $|I \cap J| \le \min(|I|, |J|) \le 1$, singleton runs remain width-1 atoms across all levels $\text{Cut}_k$ ($k \ge 1$) of the refinement tower, all `Open` bands, all Agol–Hass–Thurston pairings (`1 div d = 0` Euclidean acceleration), and all Bestvina–Handel valence-2 folds (where contracting $0.89 r$ degree-$(1,1)$ wires whose endpoints are scrambled in memory **is** list contraction on $0.89 r$ edges).
- **Theorems 2B & 2C (Dominance of Explicit $\mu$-MLP Pointers in $\mathsf{HBMC}$ — `proved`):** For $m_* \le 205\text{k}$ ($4 m_* \le 820\text{ KiB}$):
  - *vs Succinct Rank (Pătraşcu–Viola 2010 / Golynski 2009):* Compressing below $m_* \log_2 m_*$ bits forces $t_{\text{probe}} \ge 3$ dependent loads + popcnt ALU ($\ge 4.35\text{ ns}/\mu + 1.1\text{ ns}$ floor, `27–35 ns` measured in `BSO`), strictly exceeding $\frac{1}{\mu}\tau_{\text{lat}}(V_{\text{ptr}}) \approx 0.75\text{ ns/step}$.
  - *vs External-Memory List Contraction (Chiang et al. 1995):* Independent-set contraction shrinks live vertices by at most $1/2$ per round ($1/3$ deterministic), requiring $\ge 4 m_*$ bucketed tuple passes ($\ge 6.8\text{ ns/row}$ irreducible bandwidth+ALU floor, `15–35 ns/row` in `BSSE`), which beats $\mu$-way pointer probing **iff** $\tau_{\text{lat}}(V) > 55\text{–}150\text{ ns}$ (disk / huge out-of-L3 DRAM, never L2/L3).
- **Theorem 4 (Unimodular Orbit-Time Linear Systems — `proved` + `verified` on `5,227` systems):** Grounding $\tau(\text{mark}) = 0$ makes the cycle incidence system $(I - P_{g^{-1}})\tau = \mathbf{w}$ upper-triangular in cycle order with diagonal $+1$, hence $|\det A| = 1$ over $\mathbb{Z}$. Because vertex labels in memory are scrambled by $g^{-1}$, sparse Gaussian elimination on degree-2 rows *is* independent-set list contraction (`Theorem 2C`), block elimination of $S^c$ *is* `Open`'s stochastic complement, and eliminating non-cut vertices on $S_{\text{cuts}}$ *is* `BVSMC-2P`'s $S \times S$ cyclic Schur system (`verified` byte-exact on all `5,227` small cases). Black-box Krylov/Wiedemann requires $\Theta(m_*^2)$ operations.
- **Proposition 5B (Origami Veech / Shear Invariance — `proved` + `verified`):** On the origami $(\mathrm{LF}, u = +1)$, pre- or post-composing $\mathrm{LF}$ by $u^s$ cyclically shifts $L$ without changing cyclic $r$ or $r_1$ (`verified` on `5,227` cases), while higher words in $\langle \mathrm{LF}, u \rangle$ refine cuts monotonically (`record 84`: $K_k = r + \sum_c \min(k, \ell(c))$).
- **Theorem 6 (Single-Mark Multi-Lane Surcharge — `proved`):** Because $I_0$ specifies only one primary row $p$ and omits cycle distances between row-space cut points, no $K \ge 2$ lane consumer knows write offsets of lanes $q \ge 1$ in `out[0..n)` prior to cycle traversal (even when step byte lengths equal $1$, as in `Segmented`), forcing either $(1 - 1/K)$ staging copies (`1P`) or a second traversal pass (`2P`).

---

## 6. Survival Audit of Prior Art and Repository Obstructions

| Claim / Prior Art | Original Scope | Survives as Genuine Bound in $\mathsf{HBMC}$? | Exact Verdict & Escape Condition |
|---|---|:---:|---|
| **Ferragina–Gagie–Manzini (2012)** ($M \cdot P_{\text{passes}} = \Omega(H)$) | Pure sequential-scan external memory | **Only at L1 ($M_1$), not L2/DRAM** | $H^* \in [56, 132]\text{ KiB} < M_2$; allows 0 extra DRAM reads (`BSO`/`Astra D`), and at L1 requires only $2\text{–}3$ scans (`~0.02 ms`). |
| **Aggarwal–Vitter (1988) & Carbon Cycle 5 Thm 5.2** | Indivisible external-memory permuting | **Sharpened / Corrected** | At $M_1 = 48\text{ KiB}, B_w = 16$, $\log_{M_1/B}(K/16) \approx 1.37 \le 2$, so AV's $\min$ branch forces only $\approx K/8$ L1 transfers (`~0.015 ms`), **not** $K$ misses! Uncoalesced $K$ misses in `emit` are forced instead by **Theorem 2C** (Chiang's $4K$-pass constant beats L2 latency only on disk). |
| **Chiang et al. (1995)** list ranking | Indivisible EM list ranking | **Yes, in $\mathfrak{A}_{\text{sem}}$ (Thm 2C)** | Strengthens to the Latency-vs-Contraction Dichotomy: $\mu$-way pointer probing strictly dominates bucketed contraction for $4m_* \le 3.6\text{ MiB}$. |
| **Nishimoto–Tabei (2021) / GNP $r$-index** | $O(r)$-space local LF / SA queries | **No (suboptimal upper bounds)** | Move structures pay $n = 900\text{k}$ queries vs `Open`'s $m_{16} \approx 0.15n$; GNP $\Phi$-copies require $\ge r$ RV steps + cyclic copy resolution. |
| **Accelerated RV floor $r$ (`records 79–80`)** | Monotone RV schedules ($\Lambda=0$) | **Generalised to all $S$ (Thm 1)** | Survives across all $2^{n-1}$ subsets $S \ni p$ as $|S| + H(S) \ge r$. |
| **Run labels determine text (`record 84`)** | Per-byte label intermediates | **Yes, unconditionally** | Exact information bijection (`proved`). |
| **Direct $\Phi$-copies cyclic (`round 6`)** | In-place write-once $\Phi$ peeling | **No (representation-scoped)** | Escaped by `Open` and by `Cells` SLP writer. |
| **1 entry per retained row (`#654`)** | `Open<16>` queues | **At $r_1$ yes; at $m_{16}$ no** | `Cells` uses $O(r) < m_{16}$, and `U16Q` removes per-run branch cost; $r_1$ entries remain irreducible in $\mathfrak{A}_{\text{sem}}$ (Thm 2). |
| **"CPU already overlaps `Open`'s walk" (`#656`)** | 1-lane software prefetch | **Refuted** | `82–89%` of 1-lane `emit` is serial load latency; 8-lane branchless `BVSMC` cuts `emit` to `0.55–0.78×` once Theorem 7 ROB flushes are removed. |
| **`LIMIT = 16` optimal (`#657`)** | `Open<Λ, true, true>` | **Derived in $\mathsf{HBMC}$ (Thm 1)** | $\Lambda^* = \lfloor \gamma / \alpha \rfloor = \lfloor 168 / 10.7 \rfloor = 15.7 \approx 16$; shifts higher if $\alpha$ drops (`BVSMC`) or lower (`Λ → 1..2`) only if $\gamma$ avoids phrase flattening. |
| **Implicit-rank latency (`Neon BSO-512`)** | Single-thread succinct `Open` | **Yes, single-core (Thm 2B)** | Escaped *only* if multi-worker DRAM contention inflates explicit-table $\tau_{\text{lat}}$ past $\mu \cdot t_{\text{probe}} \tau_{L_1}$. |
| **Output-copy oracle ceiling (`Cycle 3 SMCC`)** | Output-sourced `out` copies | **Yes** | Random L2 reads across `900 KB` `out` + 3-way dispatch match L2-resident `edges[0..m]` walk cost even at zero discovery cost. |

---

## 7. The Precisely Stated Open Estimate

Everything inside $\mathfrak{A}_{\text{sem}} \cup \mathfrak{A}_{\text{succinct}}$ is now bounded by ($\bigstar$) (`0.290–0.585 ms` at $\mu=8$, `0.457–1.134 ms` at $\mu=1$, with constructive decoders within `2.3–4.1×` and every sub-millisecond of the remaining constructive gap accounted for in §3.3). Any single-core algorithm on fixed `bzip2` $I_0$ that asymptotically beats ($\bigstar$) must lie in $\mathfrak{A}_{\text{all}} \setminus (\mathfrak{A}_{\text{sem}} \cup \mathfrak{A}_{\text{succinct}})$ and violate the **Non-Indivisible BWT Dispersion Conjecture**:

> **Open Estimate (`conjecture`, Non-Indivisible BWT Routing Barrier):** Let $\mathcal{T}_M$ be the uniform distribution over $n$-byte blocks embedding $M = \Theta(r_1)$ independent $122$-bit tokens into a fixed template (`entropy` $H^* = 122 M$ bits, cache $M_1 < H^*/8 < M_2$, word $w = 64$, line $B = 64\text{ B}$). Does **every** exact Word-RAM algorithm $\mathcal{A} \in \mathfrak{A}_{\text{all}}$ (allowing arbitrary non-indivisible Boolean/algebraic mixing across unrelated tokens) still satisfy
> $$\mathbb{E}_{T \sim \mathcal{T}_M}\bigl[W_{\text{op}}(\mathcal{A}) + \mu\,\Lambda_{L_1\to L_2}(\mathcal{A})\bigr] \;=\; \Omega(r_1)\; ?$$

By Afshani, Freksen, Kamma & Larsen (*Lower Bounds for External Memory Integer Sorting via Network Coding*, 2019) and Iacono & Pătraşcu (2012), proving this unconditionally without indivisibility requires ruling out linear/Boolean network-coding gains on the BWT's radix-butterfly dispersion graph — a barrier tied directly to superlinear Boolean circuit lower bounds.

---

## 8. Reproducing All Verifications and Calibrations

```sh
BLOCKS_DIR=/tmp/admitted-bwt/admitted-bwt python3 docs/journal/2026-11-20-carbon-depth/invariants_and_toys.py
python3 docs/journal/2026-11-20-carbon-depth/verify_theorems.py
BLOCKS_DIR=/tmp/admitted-bwt/admitted-bwt python3 docs/journal/2026-11-20-carbon-depth/calibrate_and_retrodict.py
```
Committed receipt: [`verification_outputs.txt`](2026-11-20-carbon-depth/verification_outputs.txt).
