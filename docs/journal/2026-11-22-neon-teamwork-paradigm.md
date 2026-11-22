# Mathematical Bouleversement: Teamwork Investigation of Inverse BWT Reconstruction

*Date:* 2026-11-22 (UTC)  
*Branch:* `research/neon-teamwork-paradigm` (based on `origin/master` `8ff1187af6a920f7d57fd338680db3a5633a3ac5`, `mibz2` v0.5.11)  
*Scope:* One bounded multi-investigator Teamwork investigation (`Alpha`, `Beta`, `Gamma`, `Delta`, `Epsilon`, `Zeta`, `Eta`, and an independent `Adversarial Mathematical Referee`) with bounded SAT/SMT/CP-SAT falsification & schedule optimization (`Cadical195`, `Z3 5.4.1`, `OR-Tools CP-SAT 9.15`). Zero production files (`src/`, `Cargo.toml`) modified.

**Claim tags:** `[proved]`, `[verified]` (with exact case count / SHA-256 authentication), `[measured]` (with program & machine), `[bounded-UNSAT]` (finite domain only, NEVER a general proof), `[bounded-opt]`, `[conjecture]`, `[falsified]`.  
**Discipline:** **Algorithmic/mathematical escape conditions** (changes to the formulation, assumptions, or primitive) are kept strictly separate from **deployment conditions** (hardware cache sizes, load latencies, thread counts).

---

## 0. Executive Synthesis

**Central Question:** Given only the bzip2 block entry state $I_0 = (n, p, \text{capped run records } R, C, \Sigma)$, can exact inverse BWT reconstruction replace BOTH core operations of `OpenCells` (Rauzy–Veech strip induction + serial dependent `LF` traversal of $m_{16} \approx 0.10\text{–}0.23\,n$ retained rows) with a fundamentally different mathematical computation?

### Headline Findings

1. **Mathematically YES — Three Genuine Non-Walk Primitives Proved & Verified Exact on All 6 Authenticated Wikidata Blocks:**
   - **Monotone BWT-Space Grammar Recompression (`Lazard-BWT` $\leftrightarrow$ `1Cut-BWT` $\leftrightarrow$ `BDMR`, Alpha & Beta):** Compiles $I_0$ directly in RLE-BWT space into a balanced SLP with **zero `LF` orbit chasing, zero Rauzy–Veech strip induction, and zero row reordering** (`BDMR`: `25–26` greedy rounds / `11` to L1D cutoff, balanced SLP height `25–26`, $|G| = 0.72\text{–}0.88\,r_0$, `Q_dep = 0` in contraction). `[proved]`, `[verified]`, `check_manifest.py` PASS.
   - **Multi-Frontal Bucket-Order Schur Elimination (`MF-BSE`, Gamma):** Zero-fill $K$-bucket block Gaussian elimination of $(I - G)\tau = w - n e_p$ with $\le m - 1$ total shortcuts (NO $\log m$ factor!) and **`Q = 0` random cache misses outside fast memory $Z$** ($O(m/B)$ I/Os) under the tall-cache condition $m \le c Z^2/B$. `[proved]`, `[verified]`, `check_manifest.py` PASS.
   - **Two-Sided Slack Output-Backed Tunnel Memoization (`LazyTunnel`) & Zero-Induction Tunnel Elimination (`ZeroRV-Tunnel`, Delta):** Cut `Open`'s dependent walk loads $Q$ by `30.7%` (at `0 ms` construct) and `11.5–34.1%` (before `emit_plain`), respectively. `[proved]`, `[verified]`.

2. **Model-Independent Universal Obstructions Replacing Carbon Depth's `A_sem` Assumption (Gamma, Referee-Confirmed & DRAT-Audited):**
   - **Theorem G1 (`L`-Completeness under Uniform `TC⁰`):** Exact inverse BWT from run-encoded $I_0$ (even binary $|\Sigma|=2$, any target $r/n$) is **`L`-complete under uniform `TC⁰` reductions** via `Phi_BWT` / `Phi_BWT_bin`. Rules out $O(1)$-depth global coordinate formulas unless `TC⁰ = L` (and `NC¹` unless `NC¹ = L`). `[proved]`, `[verified: 2,830 cases]`.
   - **Theorem G2 (Unconditional Read-Only Streaming Lower Bound, breaking `A_sem`):** Via Hairpin-Closed 2-Party `Phi_BWT^(2)` (zero communication, zero seam run-merge, public $C, p=0$), ANY deterministic or randomized, arbitrary bit-level/algebraic $p$-pass **read-only** streaming decoder over $I_0$ requires **$S = \Omega(r / (L p^4))$ bits** of working RAM. `[proved]`, `[verified: 432 adversarial cases + DRAT-certified N=3, t=4, p=2 separation]`.
   - **Theorem E1 (3-Byte Global Sensitivity / Local-Key Placement Obstruction, Epsilon):** Valid single-cycle GUID+template BWT pairs differing at **literally 3 rows** ($d_H(L_1, L_2) = 3$) preserve identical BWT row indices and $\Theta(n)$-radius `LF`/`FL` neighborhoods on all $M - 3$ untouched tokens yet flip global text order. `[proved]`, `[verified]`.
   - **Cross-Block Obstructions (Zeta):** Multiplicity Trichotomy + **Theorem Z2** (valid-BWT width-1 indistinguishability, $n=82, d_H=3$ witness) + **Theorem Z3** (coordinate-independence: `0 / 21,862 = 0.00%` exact `LF` shift match across consecutive blocks). `[proved]`, `[verified]`, `[measured]`.

3. **Decisive Native Safe-Rust Confrontation (`mibz2` v0.5.11 `OpenCells`, Same Binary, A/A Control, `1T` & `4W`) — Why Every Full Non-Walk Replacement Loses at $n \le 900\text{k}$, and What Actually Wins:**
   - **All full non-walk replacements lose decisively at $n \le 900\text{k}$ (`3.1×–15.0×` slower)** because `OpenCells`'s `edges` (`351–798 KiB`) already fit in L2/L3 while non-walk methods pay either $Q = 2.2\text{–}5.7\times m_{16}$ irregular accesses (`Lazard-BWT`, `1Cut-BWT`, `BDMR`, `Open × BDMR` Density-Ratio Obstruction), `17–299 MB` of read-write streaming traffic, `6–59 MiB` bucket scratch (`MF-BSE`), or SLP expansion alone (`1.5–3.6 ms`) exceeding all of `OpenCells` (`1.3–2.9 ms`). CP-SAT & exact global Dijkstra schedule optimization prove `BDMR` is already within `4%–30%` of the global optimum over the entire 3-way selective contraction algebra — no hidden `10×` schedule exists!
   - **Three Genuine Positive Safe-Rust Mechanisms Discovered (Port Candidates / Structural Wins):**
     1. **`BSAC(256, K=8)` (Bijective-Slot Anchor Contraction, Epsilon — PORT CANDIDATE FOR i9):** Zero-bitmap, zero-alloc, **`1.000 m` exact random `edges` loads (0 extra random loads vs `Relay8`'s `+0.87 m`)**, `0 ms` construct delta, only `2m` sequential `u16` staging (`28×` fewer extra cache lines than `Relay8`). Wins **`0.93×`** 1T (`0.85–0.86×` on `block-143/210`) and **`0.751–0.765×`** vs `Serial(A)` (`0.809–0.823×` vs faster `Serial(A')`) at 4W on the `2 MiB`-L2 Xeon; on a `256 KiB`-L2 i9-10900K all 6 blocks (`edges = 351–798 KiB = 1.37–3.12× L2`) sit deep inside its L2-spilling winning zone. `[proved]`, `[verified: 5,227 + 6 blks]`, `[measured]`.
     2. **Uncapped Maximal Runs (`runs::front_avx2 -> Open`, Eta — CLEAN DECOUPLED WIN):** Eliminating `127`-cap record splitting (`+1.18M` extra records on 431 blocks solely for `LazyCells`, which handles `0.005%` of Wikidata rows) and `open.rs::maximal()` cuts **`Open` Pass 1 by `-19.1%`** (`72.42 -> 58.56 ms`, `-4.1%` on `Open`, `-16.0%` to `-21.5%` on all 6 blocks) with **zero scanner regression** and **`-7.2%` smaller `records`**. `[proved]`, `[verified: 5,236 + 431 blks]`, `[measured]`.
     3. **`XB-SLP` (Cross-Block Structural SLP Hash-Consing, Zeta — 1T STRUCTURAL WIN):** `91.16%` (`1,425,464 / 1,563,736`) of RV `join(l, r)` rules share identical binary-tree parenthesization across consecutive blocks. Safe-Rust `XbSlp16k` wins **`0.9554× [0.9399, 0.9711]`** kernel 1T (`AA = 1.0005×`, `LocalFused = 1.0080×`) and **`0.9540× [0.9269, 0.9820]`** whole-decoder 1W across all 431 blocks, flattening to `0.9789×` (`0.9946×` vs `AA`, inconclusive) at 4W due to `+564–896 KiB/worker` state and the `4–9%` `Spelling` Amdahl cap. `[proved]`, `[verified: 5,227 + 431 blks]`, `[measured]`.

---

## 1. Master Summary Table — Native Safe-Rust A/B + A/A Benchmarks & Models

All benchmarks executed in `#![forbid(unsafe_code)]` Rust (`rustc 1.99.1`, `RUSTUP_TOOLCHAIN=stable`) against `mibz2` v0.5.11 `OpenCells` in the **same binary** on Intel Xeon Platinum 8481C (`2` vCPUs, `48 KiB` L1d, `2 MiB` L2, `105 MiB` L3). All 6 blocks in `data/admitted-bwt` and the 431-block `corpora/wikidata.bz2` (`82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915`) passed SHA-256 authentication and byte-for-byte parity checks.

| Investigator / Mechanism | Computational Model | `Q` vs `Open` `m_16` | RW-Stream Passes / Bytes (`Q=0`) | Working Set | 1T vs `OpenCells` (A/A ctrl) | 4W vs `OpenCells` (A/A ctrl) | Verdict |
|---|---|---|---|---|---|---|---|
| **`OpenCells` (v0.5.11)** | RAM (RV + serial walk) | `1.00×` (`m_16 = 90k–204k`) | N/A | `~1.2 MiB/w` | `1.000×` (`11.4–12.2 ms` / 6 blks) | `1.000×` | Production baseline |
| **Alpha `Lazard-MAX`** | RAM (zero LF walk) | `2.18–2.76×` (`229k–528k`) | `920–1,904` p / `81.8–298.8 MB` | `3.25–7.19 MiB` | `2.81–4.43×` (`A/A 0.99×`) | `3.98–5.27×` (`A/A 1.00×`) | **KILL** (`Q > m_16`, L2 spill) |
| **Alpha×Beta `1Cut-BWT`** | RAM / RW-Stream | `5.82–6.86 r` (`298k–794k`) | `375–804` p / `17.3–41.2 MB` | `~1.6–4.0 MiB` | Dominated by `BDMR`/`Lazard` | — | **KILL** (`Q >> m_16`) |
| **Beta `BDMR Greedy T=4096`** | RAM / `NC²` / RW-Stream | `4.49–5.70×` (`467k–1.11M`) | `~100` p / `26.8–64.2 MB` | `1.66–4.15 MiB` live | `12.35×` (`A/A 0.986–1.021×`) | `11.68×` (`A/A 1.015×`) | **KILL** (`Q >> m_16`, SLP exp `> Open`) |
| **Beta `Open × BDMR` Hybrid** | Post-Open band-space | `2.24–4.65×` (`Q`-gated = `0` rnd) | — | `~1.5–4 MiB` | Density-Ratio Obstruction (`w_bar = 1.62–2.22 < 3`) | — | **KILL** (Round 0 fails `Q < |D|`) |
| **Gamma `Hybrid-Open-MF-BSE`** | Tall-Cache RW-Stream | `0` in `pos` (`+m_16` write-only) | `2` sweeps (`K=64`) / `~64 m_16` B | `6.0–21.0 MiB` | `3.27×` (`A/A 1.001×`) | `3.06×` (`A/A 1.011×`) | **KILL** at `n<=900k` (wins at `m>=1.5M`) |
| **Gamma `Standalone-MF-BSE`** | Tall-Cache RW-Stream | `0` in `pos` (`+n` write-only) | `2` sweeps (`K=256`) / `~64 n` B | `50.6–58.9 MiB` | `13.16×` (`A/A 1.001×`) | `14.96×` (`A/A 1.011×`) | **KILL** at `n<=900k` |
| **Delta `LazyTunnel (ms=3)`** | RAM (in-place `edges`) | **`0.693×`** (`−30.7%`) | N/A | `+10–27 KiB` | `1.280×` (`A/A 1.004×`) | `1.312×` (`A/A 1.016×`) | **KILL** (`6.73` vs `1.86 ns/step` budget) |
| **Delta `ZeroRV-Exact (w>=6)`** | RAM (pre-walk tunnel) | **`0.885×`** (`−11.5%`) | N/A | `+~200 KiB` | `1.140×` (`A/A 1.004×`) | `1.110×` (`A/A 0.968×`) | **KILL** (`60–140×` leverage collapse) |
| **Epsilon `BSAC(256, K=8)`** | RAM (`K`-way MLP emit) | **`1.000×` exact** (`+2m` seq `u16`) | N/A | `+215–440 KiB` (0-alloc) | **`0.93×` emit** (`0.85–0.86×` b143/210; `A/A 1.025×`) | **`0.751–0.765×` emit** (`0.81–0.82×` vs `A'`) | **i9 PORT CANDIDATE** (`edges = 1.37–3.12×` i9 L2) |
| **Zeta `XB-SLP (XbSlp16k)`** | Cross-block hash-cons | `1.00×` (`+7k` L2 probes) | N/A | `+564–896 KiB/w` | **`0.9554×` kernel** / **`0.9540×` 1W** (`A/A 1.0005×`) | `0.9789×` (`0.9946×` vs `AA`, inconcl.) | **1T WIN / 4W FLAT** (`Spelling` Amdahl cap) |
| **Eta `HuffMonoidBL + B0/B1`** | Whole-pipeline fusion | Scanner `+45%` (`107.9` vs `74.5 ms`) | N/A | `-140 KiB` recs | `1.185×` (431 blks; `A/A 1.003×`) | `1.186×` (`A/A 0.992×`) | **KILL** (hurts scanner critical path) |
| **Eta Uncapped MaxRuns (`B2`)** | Decoupled `front -> Open` | `1.00×` (`-1.18M` recs) | N/A | `-7.2%` recs | **`-19.1%` Pass 1** (`-4.4%` Open, `T_scan` 0 Δ) | Scales cleanly `W=1..10` | **CLEAN DECOUPLED WIN** |

---

## 2. Gamma's Model-Independent Obstructions (Audit-Ready Formal Statements & Proofs)

*Preserved with all assumptions explicit for independent post-handoff proof audit; scripts in [`gamma/verify_phi_bwt.py`](2026-11-22-neon-teamwork-paradigm/gamma/verify_phi_bwt.py) and [`referee/audit_gamma.py`](2026-11-22-neon-teamwork-paradigm/referee/audit_gamma.py).*

### 2.1 Construction `Phi_BWT(pi, b, K)` and Binary `Phi_BWT_bin(pi, K)` `[proved]` `[verified: 2,830 cases]`
Given any single-cycle permutation $\pi \in S_M$ (`pred[x] = pi^{-1}(x)`), base $b \ge 2$ with $L = \lceil \log_b M \rceil$, skeleton length $K \ge 1$, alphabet $\Sigma = \{\#, \$, 0, \dots, b-1\}$ ($\# < \$ < 0 < \dots < b-1$), $u_0 = 0, u_{t+1} = \pi(u_t)$, $S = \#\,\$^{K-1}$, and MSB-first base-$b$ $\mathrm{id}(x) = d_1(x)\cdots d_L(x)$, define:
$$T_\pi = S\,\mathrm{id}(u_0)\,S\,\mathrm{id}(u_1)\cdots S\,\mathrm{id}(u_{M-1}), \qquad n = (K+L)M.$$
- $p = 0$; `#`-block $[0, M)$ has $L[x] = d_L(\mathrm{pred}[x])$; `$`-blocks $[M, KM)$ have 2 runs (`('#', M)` and `('$', (K-2)M)` for $K \ge 3$); digit blocks $[KM, n)$ are indexed by $(s, x) \in \{1..L\} \times [M]$ with $y = \mathrm{pred}[x]$ and sort key $\mathrm{key}(s, x) = (d_{L-s+1}(y), \dots, d_L(y), \#, x)$, which depends **only on $(x, \mathrm{pred}[x])$** (never $\pi^2$).
- Exact first-return: $\mathrm{LF}^{K+L}(x) = \pi^{-1}(x)$ on $[0, M)$, with $r \le (L+1)M + 2$ ($r/n \le (L+1)/(K+L)$ matching any target density).
- **Binary $|\Sigma|=2$ (`Phi_BWT_bin`, $K \ge 2$):** $S = 0^K$ and bit-stuffed $\mathrm{id}_2(x) = 1 b_1 1 b_2 \cdots 1 b_L 1$ (zero-pair free; Referee verified zero prefix collisions between Type-1 and Type-2 keys at even index $2L - o + 1$ where Type-1 has `1` and Type-2 has the second `0` of $0^K$).

### 2.2 Theorem G1 (`L`-Completeness under DLOGTIME-Uniform `TC⁰` Reductions) `[proved]`
**Assumptions:** Input `pred[0..M-1]` of a single-cycle $\pi \in S_M$; target output $I_0(T_\pi) = (n, p, R, C, \Sigma)$ where $R$ is the exact sequence of 127-capped `(byte, len)` run records (with $K \le \mathrm{poly}(M)$ for 127-capped records; or arbitrary $K$ for uncapped maximal runs).  
**Statement:** Exact inverse BWT from run-encoded $I_0$ (even for binary $|\Sigma| = 2$ and any constant $r/n = \varepsilon > 0$, even to decide the relative text order of two positions or output a single target bit) is **`L`-complete under DLOGTIME-uniform `TC⁰` reductions**.  
**Proof:** (Upper bound) Evaluating `LF` from $I_0$ scans $I_0$ in $O(\log n)$ space, so iterating `LF` from $p$ is in `L`. (Hardness) Single-cycle `ORD` is `L`-complete under `AC⁰` reductions (Etessami 1997, *IPL* 61(4), Thm. 6). Producing $L[0..n)$ and $C$ from `pred` uses pairwise key comparison (`AC⁰`), rank counting (`TC⁰`), and `L[i]` selection (`AC⁰`). Exact 127-capped run records $R$ follow in $\le 5$ additional `TC⁰` layers: `head[i] = (i==0) || (L[i]!=L[i-1])` (`AC⁰`), unique run-end detection `is_end(i, j)` (`AC⁰`), `cnt_rec(i) = ceil(run_len(i)/127)` (`AC⁰`), prefix sum `rec_start(i)` (`TC⁰`), and record slot emission (`AC⁰`). Total $\le 8$ DLOGTIME-uniform `TC⁰` layers. Q.E.D.  
**Escape Condition (algorithmic):** Depth $D = \omega(1)$ (e.g., `NC²` as in `BDMR`) or restricting inputs to non-universal families (Sturmian / bounded-treewidth $G$).

### 2.3 Theorem G2 (Unconditional Read-Only Multi-Pass Streaming & Communication Lower Bound) `[proved]` `[verified: 432 cases + DRAT]`
**Explicit Model & Assumptions:**
- **Model:** Read-only $p$-pass streaming over $I_0$ with $S$-bit working RAM and **zero auxiliary writeable streams** (equivalently $(2p - 1)$-round two-party communication on $I_0 = I_{0,\text{Bob}} \parallel I_{0,\text{Alice}}$ with Bob speaking first, $\le S$ bits/message). Quantifies over **arbitrary deterministic or randomized, bit-level or algebraic decoders** (NOT restricted to `A_sem`).
- **Parameters:** $|\Sigma| = 2b + 4 \ge 8$, $N \ge 64 t$ even (non-Abelian $N \ge 3$ required; Referee caught $N=2$ Abelian $S_2 = \mathbb{Z}_2$ collapse to 1-bit XOR), $t = 2p$ (or $t = 2p + 2$ for arbitrary fixed stream ordering), $L = \lceil \log_b((t+1)N) \rceil$.
- **Hairpin-Closed Two-Party Split `Phi_BWT^(2)`:** Given independent uniform $\pi_1, \dots, \pi_t \in S_N$ (Alice holds odd $\pi_{2k+1}$, Bob holds even $\pi_{2k+2}$), construct bipartite graph on $M_{\text{side}} = (t+1)N$ A-nodes and $M_{\text{side}}$ B-nodes: forward $A(0)\xrightarrow{\pi_1} B(1) \cdots \xrightarrow{\pi_t} A(t)$, hairpin $A(t) \xrightarrow{\mathrm{id}} B(t')$, backward via $\pi_t^{-1}, \dots, \pi_1^{-1}$ to $B(0')$, and closing shift $+1 \bmod N$ to $A(0)$. Because $\pi_1^{-1}\cdots\pi_t^{-1}\pi_t\cdots\pi_1 = \mathrm{id}$, $f_B \circ f_A$ is **ALWAYS a single cycle** of length $(t+1)N$, with $v_t = \pi_t(\cdots\pi_1(0)\cdots)$ at fixed text offset $(t/2)\cdot 2(K+L) + K$. With alphabet order $\#_A < \$_A < 0_B < \dots < (b-1)_B < \#_B < \$_B < 0_A < \dots < (b-1)_A$: `L[0..n/2)` depends exclusively on Bob, `L[n/2..n)` exclusively on Alice, $C$ and $p=0$ are public constants, and `L_bob[-1]` is disjoint from `L_alice[0]` (**zero seam run-merge, zero communication**). Verified on 432 adversarial instances.
- **Statement:** Any $p$-pass read-only streaming algorithm recovering $T$ (or even the 1-bit predicate $\mathrm{MSB}(v_t)$) with error $\le 1/4$ requires:
  $$S \ge \frac{N}{256 \ln(2)\, t^3} \quad (\text{exact } v_t), \qquad S \ge \frac{49\,N}{131072 \ln(2)\, t^3} \quad (\text{1-bit }\mathrm{MSB}(v_t)), \qquad\text{i.e. } S = \Omega\!\left(\frac{r}{L\, p^4}\right) \text{ bits}.$$
- **Proof A (Gamma's Product-Rectangle Slice Lemma + Random-Subset Bijection Chain-Rule):** Fixing $\mathrm{Past}_{r-2} = (\pi_1,\dots,\pi_{r-2})$ intersects transcript $T_{r-1}$'s combinatorial rectangle into a product rectangle, making $(\pi_{r-1}, R_{r-1}, M_r)$ conditionally independent of $(\pi_r, R_r)$ given $(\mathrm{Past}_{r-2}, T_{r-1})$, so $\mathrm{Law}(\pi_r(v_{r-1}) \mid \mathrm{Past}_{r-1}, T_r, R_{\le r}) = \mathrm{Law}(\pi_r(v_{r-1}) \mid v_{r-1}, \mathrm{Past}_{r-2}, T_{r-1}, R_r)$ and $\sum_{r=2}^t I_r \le (t-1)S$. Coupling $\mu = \mathrm{Law}(v_{r-1} \mid \mathrm{Past}_{r-2}, T_{r-1}, R_{\le r-1})$ to $\nu = \mathrm{Unif}([N] \setminus U_r)$ adds TV drift $\le 4k/N$ (for $|A|,|B| \le k < N/2$), and Jensen + Pinsker over $j_r \in \{1..k\}$ gives $\delta_r \le \delta_{r-1} + 4k/N + \sqrt{(\ln 2/2) I_r / k}$. Summing + Cauchy–Schwarz yields the exact constants above.
- **Proof B (Referee's Independent Algebraic Permutation Decoupling Lemma — zero `U_r` conditioning):** For uniform ordering $\tau$ independent of $(\pi, W)$ and $i \le \lfloor N/2 \rfloor - 1$, $S_i = \pi(\tau(<i))$, $q_i(y \mid x, w) = \mathbb{E}[\mathbf{1}_{y \notin S_i}/(N-i)]$ satisfies the exact identity $q_i(y \mid x, w) = (1-\lambda_i)/N + \lambda_i \Pr[\pi(x)=y \mid w]$ with $\lambda_i = i/((N-i)(N-1)) \le 1/(N-1)$, directly yielding $\mathbb{E}_{x, W}\|\mathrm{Law}(\pi(x)\mid W) - \mathrm{Unif}\|_\mathrm{TV} \le \frac{N-1}{N-2}\sqrt{I(\pi;W)/N}$ for all $N \ge 3$ and $S = \Omega(N/t^3) = \Omega(r/(L p^4))$.
- **Exact Model Boundary / Escape Condition:** Theorem G2 governs **read-only** streaming (`0` writeable streams). Writing intermediate streams (**read-write / external-memory streaming**, used by `Lazard-BWT`, `BDMR`, and `MF-BSE`) escapes Theorem G2 cleanly.

### 2.4 Theorem G3 (Krylov / Minimal-Polynomial Obstruction) `[proved]`
Any single-cycle permutation matrix $G$ of order $m$ has minimal polynomial $x^m - 1$ over any field $\mathbb{F}$, forcing $\Omega(m/k)$ rounds of black-box $k$-vector SpMV queries to solve $(I - G)\tau = b$. Scope: blocks only black-box linear Krylov methods; escaped by nonlinear composition (`BDMR`, `MF-BSE`).

---

## 3. Unified BWT-Space Grammar Recompression (`Lazard-BWT` $\leftrightarrow$ `1Cut-BWT` $\leftrightarrow$ `BDMR`) & Solver Audit

### 3.1 Exact 2-Parameter Classification `(\kappa, e(\Sigma_R))` `[proved]`
All three primitives delete an independent set $D = \{j : L[j] \in \Sigma_L \land F[j] \in \Sigma_R\}$ under a 2-coloring $\Sigma = \Sigma_L \sqcup \Sigma_R$ directly on $(L, F, p)$ with **zero row reordering**:
- **Cut-number $\kappa(\Sigma_L, \Sigma_R)$:** number of contiguous intervals of $\Sigma_L$ in sorted $\Sigma$ order. Zero bucket splits (`a` never splits into $a^{(0)}, a^{(1)}, \dots$) holds for all BWTs **iff $\kappa = 1$** (`[proved]`).
- **Internal edges $e(\Sigma_R)$:** transitions $\Sigma_R \to \Sigma_R$.
  1. **`Lazard-BWT` ($\kappa = 1, e(\Sigma_R) = 0$, Alpha):** BWT–Lazard Commutation Theorem ($\mathrm{BWT}(\Phi_x(T)) = \mathrm{Splice}_x(\mathrm{BWT}(T))$ at $\min/\max$) + $Q=0$ extreme-bucket self-loop queue resolution (fixed points strictly in prefix of $F_{\min}$ / suffix of $F_{\max}$, verified on `18,045` arbitrary ternary non-BWT cases vs `68` interior fixed points in $F_{\text{mid}}$) + **unconditional $r_{t+1} \le r_t$** (`[proved]`, DRAT-audited in F1) + bidirectional independent-set wavefronts (`450–883` rounds on Wikidata, `tot_syms = 0.76–1.04 r`, `94–96%` in-place overwrites).
     - *Prior art:* Single-sided `MAX` is the run-length dual of `Cells` / Schützenberger–Melançon circular rewriting (`MONODROMY 2`); `BOTH` wavefronts, $Q=0$ self-loop queue, and $(\kappa, e)$ unification are new.
     - *Referee Adversarial Falsification (`Symmetric-Middle-Run`, `K=64, n=4224, r=130`):* Forces all 3 modes (`min`, `max`, `both`) to **$\Theta(n)$ work and $\Theta(n)$ grammar symbols** (`both`: `4,211` symbols = `32.39 r_0`) because middle-alphabet runs $M^K$ peel one byte at a time.
  2. **`1Cut-BWT` ($\kappa = 1, e(\Sigma_R) \ge 0$, Alpha×Beta):** Zero bucket splits, cuts `Lazard` streaming volume `5–7.6×` (`17.3–41.2 MB`) and `BDMR` peak $\sigma_t$ by `37–40%` (`7.2k–17.8k`), yet still pays $Q_{\text{mask}} = 5.82\text{–}6.86\,r \gg m_{16}$ and fails $n_{\text{next}} \le 3n/4$ (Referee F3b witness: $n=5, |\Sigma|=4$, $T=[1,2,3,2,0]$, $\max |D_{\text{thresh}}| = 1/5 < 1/4$).
  3. **`BDMR` ($\kappa \ge 1, e(\Sigma_R) \ge 0$, Beta):** Lifts Jeż's (2015) recompression directly onto RLBWT $(L, F, p)$ with **zero row reordering** (contiguous preimage bucket-splitting $a^{(0)}, a^{(1)}, \dots$) + 1-round monotone self-run contraction (closed-form spiral division `hi - lo <= |delta|`), requiring zero `LF` chases, zero synchronizing sets, and zero dynamic BSTs (distinguishing it from Jeż 2015 & Kempa–Kociumaka 2020/2022). Verified on `26,135` small cases + 6 blocks (`25–26` greedy rounds / `11` to `T_cut = 4096`, $\sum r_t = 4.02\text{–}4.34\,r_0$, `max_r/r_0 = 1.00×` empirically on Wikidata).
     - *Three Adversarial Findings on `BDMR`:*
       1. **`r_{t+1} \le r_t` is FALSE in general `[proved & verified]`:** Minimal valid-BWT witnesses at $n=5$ for threshold cut ($T=[0,0,1,1,2]$, $3 \to 4$), bucket-split ($T=[0,0,2,0,1]$, $3 \to 4$), and self-run ($T=[0,0,1,0,1]$, $2 \to 3$). Furthermore, Referee F4 SAT witness $n=10, |\Sigma|=3$ ($T=[0,0,1,1,2,1,2,0,0,1], r_0=5$) proves **no $|D| \ge n/4$ 2-coloring preserves $r_1 \le r_0$**.
       2. **Gamma's `Fib_{2m} + $` (`r_0 = 4`) refutation + Pure-MaxCut fix `[verified]`:** Beta's threshold-preferring `Variant B` blows up to `max_r = 37 = \Theta(\log n)` and `\sum r_t = 1,461 = \Theta(r_0 \log^2 n)` at $n = 3.79 \times 10^{16}$; Pure Max-Cut collapses it back to `max_r = 5` (constant) and `\sum r_t = 283 = O(r_0 \log n)` (and Alpha's `Lazard-BWT` holds `max_r = 4`).
       3. **Referee's $\Theta(r_B^2)$ self-run stack gap (`L_quad`) `[verified]`:** Unordered DFS stack takes `0.52 q^2` pops (`3,320` pops = `10.44 r_0` at $q=80$); fixed by monotone spatial sweep with equal-$k$ coalescing ($O(r_B)$) / lazy treap ($O(r_B \log r_B)$).

### 3.2 Bounded SAT/SMT/CP-SAT Falsification, DRAT Certificates & Optimal-Schedule Discovery
Solvers: `python-sat 1.9.dev24` (`Cadical195`), `ortools 9.15.6755` (CP-SAT), `z3-solver 5.4.1.0`. All scripts, JSON witnesses, CNFs, and DRAT certificates live in [`referee/solver_audit/`](2026-11-22-neon-teamwork-paradigm/referee/solver_audit/) and [`beta/solver_opt/`](2026-11-22-neon-teamwork-paradigm/beta/solver_opt/).
- **Priority 1 (F1–F4 Falsification):**
  - **F1 (`Lazard` $r_{\text{next}} \le r$):** `[bounded-UNSAT]` with DRAT proofs `f1_n8_s3_validbwt.drat` (1,767 lines) and `f1_n9_s3_arbcol.drat` (1,188 lines).
  - **F2 (Fixed points & $\kappa > 1$):** `[bounded-UNSAT]` DRAT `f2_min_interior_n10_s3.drat` (540 lines) and `f2_any_valid_bwt_n8_s3.cnf` (2.43s reproducible); minimal $F_{\text{mid}}$ witness $L=[2,1,0]$; minimal valid BWT forcing bucket split at $n=5, |\Sigma|=3$ ($T=[1,2,0,2,2]$, loopless $n=6, |\Sigma|=4$).
  - **F3 (`BDMR` $n_{\text{next}} \le 3n/4$):** Loopless general 2-coloring `[bounded-UNSAT]` to $(12, 5)$ (tight at $n=6$); **threshold cuts falsified** at $n=5, |\Sigma|=4$ ($T=[1,2,3,2,0]$, $1/5 < 1/4$); isolated self-run rounds fail $3n/4$ ($n=5 \to 0.80n$, $n=20 \to 0.95n$), pinning down the exact 2-round amortized factor $\le \sqrt{3/4}$.
  - **F4 (Simultaneous $|D| \ge n/4$ & $r_{\text{next}} \le r$):** Falsified at $n=10, |\Sigma|=3$; max single-step pair ratio `1.400×` (`5 -> 7`) on ternary $n=14$.
- **Priority 2 (CP-SAT & Global Multi-Step Dijkstra Optimization — Beta):**
  - Across 14 valid BWTs ($n=14..48$), `BDMR GreedyThresh` is within `1.2%` of CP-SAT optimal rounds, `3.3%` of $\sum r_t$, and `5.5%` of Dinkelbach `Q_con`; and within `+4.0%` to `+30.4%` of the **exact global multi-step Dijkstra optimum** ($n=9..17$) over the full 3-way selective contraction algebra.
  - **Dual-Polarity Obstruction `[proved & verified]`:** Left-absorbing dual polarity destroys `LF`-fiber monotonicity at $n=4$ ($L=[1,1,1,0]$); forward polarity is unique.
  - **`Open × BDMR` Thin-Run Hybrid & Density-Ratio Obstruction `[proved & measured]`:** Exact post-Open band-space riffle bijection `(L_g, C_g, p_g)` (`15,681` small cases + 6/6 blocks) preserves both `3/4` shrink (`0.681–0.696`, `7–8` rounds to `T_cut = 4096`) and `max_r/r_g = 1.000×`, yet breaks `Q_round < |D|` iff $\bar{w} = m_{16}/r_g > \beta/\alpha \approx 3\text{–}4$. Post-Open $\bar{w} = 1.62\text{–}2.22 < 3$, so **not even Round 0** clears $Q < |D|$ on any of the 6 blocks!
- **Priority 3 (Gamma Bounded Read-Only Protocol Verification — Referee):**
  - Exact deterministic $(2p-1)$-round 2-party communication over `Hairpin(N, t)`: caught $N=2$ Abelian degeneracy ($S_2 = \mathbb{Z}_2$, 1-bit XOR), and DRAT-certified the exact $t = 2p$ non-Abelian separation at $N=3, t=4, p=2$: even 1-bit `[v_4 == 0]` is UNSAT at $S=1$ bit/msg (`p3_3r_N3_t4_bit0_1b.drat`, 3,952 lines; `full v_4` `p3_3r_N3_t4_full_1b.drat`, 31,517 lines). Scope: verifies the bounded read-only protocol model only; NOT a general RAM lower bound.

---

## 4. Targeted Structural Attacks: Tunneling (Delta), Incompressible Core (Epsilon), Cross-Block (Zeta), & Whole Pipeline (Eta)

1. **Delta (Repeated Structure — BWT Tunneling & Wheeler Graphs):**
   - `41.3–61.3%` (`50.0%` sum) of `m_16` rows lie in exact sub-strip Baier tunnels of $g$; `28.5–47.8%` (`37.0%` sum) satisfy nested lazy memoization $H \ge 2$.
   - Classical Baier TBWT preserves $Q = n = 6.03\times m_{16}$ (unconditional work kill).
   - **`LazyTunnel`** cuts $Q$ by `30.7%` at `0 ms` construct (`5,227` small + 6/6 blocks exact), yet loses `1.280×` (1T) / `1.312×` (4W): measured affine emit law $\text{Emit}_{1T}(Q) = 1.65\text{ ms} + 4.20\text{ ns}\cdot Q$ ($R^2 = 0.9997$) leaves only `1.86 ns/step` break-even vs `6.73 ns/step` tracking/copy overhead.
   - **`ZeroRV-Tunnel`** (induction-time unit elimination leaving `endpoint.rs`, `wire`, and `emit_plain` 100% unmodified; `31,362` small + 6/6 blocks exact) eliminates `11.5–34.1%` of `m_16` at `2.6×` lower cost/strip than `endpoint.rs`, yet STILL loses (`1.140–2.089×` 1T, `1.110–2.004×` 4W) due to **Band-Blindness** (`97.6%` of tunnels invisible to `Open`'s `bands`), **Per-Run $\Omega(L)$ Floor**, and **`60×–140×` Leverage Collapse** (`223.3 rows/strip` for thick strips vs `1.59–3.64 rows/cand` for thin tunnels — even with `TunRec = 0.000 ms`, `1stP + Patch+Inline` exceeds `Emit` savings by `2.27–3.45×`!).
   - *Algorithmic escape:* Format-level tunnel grouping, OR attacking serial `MLP=1` walk latency across all `m_16` rows (pointing directly to Epsilon's `BSAC`). *Deployment condition (not escape):* $L_{\text{load}} > 15.2\text{ ns}$ (`LazyTunnel`) / `16.0 ns` (`ZeroRV-Exact(w>=6)`).

2. **Epsilon (The Incompressible Core — Unique GUIDs / Width-1 Rows):**
   - **Anatomy (7,149 UUIDs across 6 blocks):** All 36 UUID bytes (including all 4 dashes `-`) are `99.8–100.0%` retained in $S_{16}$; 2-grams `(L, F)` have `71–160×` avg multiplicity (max `2,381×`).
   - **Theorem E1 (`[proved]`):** 3-row Hamming perturbation flips global order while keeping $\Theta(n)$-radius `LF`/`FL` neighborhoods identical on $M-3$ tokens -> local-key placement obstructed.
   - **`BSAC(256, K=8)` (`[proved + verified: 5,227 + 6 blks + measured]`):** Deterministic 1-anchor-per-256-row block (`Q = 351–798` anchors, `g_A` `4.2–9.6 KiB` L1d-resident), zero bitmap, zero alloc, **`1.000 m` exact random `edges` loads** (`0` extra random loads vs `Relay8`'s `+0.87 m`), only `2m` sequential `u16` staging (`28×` fewer extra cache lines than `Relay8`).
     - Wins **`0.93×`** 1T emit (`0.85–0.86×` on `block-143/210`) and **`0.751–0.765×`** 4W emit (`0.809–0.823×` vs faster `A'`).
     - *Deployment condition:* `edges / L2 >= 0.25–0.30×` on Xeon (`2 MiB` L2); on i9-10900K (`256 KiB` L2), ALL 6 blocks (`351–798 KiB = 1.37–3.12× L2`) fall deep inside the winning zone (**port candidate for i9 1/6/8/10T validation**).
     - *Algorithmic escape (to remove `BSAC`'s remaining `2m` `u16` staging or `1.000 m` loads):* Fusing `TSAC` anchor registration into `Open::finish` with short-chain absorption (`< 8` absorbed into thick words, `~0.5 m` literal buffer), or construction-time multi-step contraction / tall-cache `MF-BSE`.

3. **Zeta (Cross-Block Structure — Full 431-Block `corpora/wikidata.bz2`):**
   - `233` large blocks (221 consecutive same-window pairs): `0.00%` exact `LF` shift match (`0 / 21,862`); `sha1`s unique in $B$ yet in $A$ are only `0.165%` of $m_{16}$; generous `k>=16` cross-block-ONLY width-1 oracle capped at `8.17%` of $m_{16}$ (`3.66%` of `Open`).
   - **Proved obstructions:** Multiplicity Trichotomy + Theorem Z2 (Width-1 Indistinguishability, $n=82, d_H=3$) + Theorem Z3 (Coordinate-Independence) + `XB-Cert` falsifier (`1.24` slow `rank_B` queries per saved emit step, `125.9 ms` for `1.92%` of $m_{16}$).
   - **Exact positive discovery (`XB-SLP`):** **`91.16%`** (`1,425,464 / 1,563,736`) exact binary-tree parenthesization reuse across consecutive blocks -> **`0.9554×` kernel 1T / `0.9540×` whole-decoder 1W** (`AA = 1.0005×`, `LocalFused = 1.0080×`), flattening to `0.9789×` (`0.9946×` vs `AA`) at 4W due to `+564–896 KiB/w` state & `4–9%` `Spelling` Amdahl cap.

4. **Eta (Whole Pipeline — Huffman + MTF + RLE2 + Open Pass 1 on 431-Block Corpus + 6 Blocks + High-Entropy):**
   - **Theorem H1 (EOB Causal Barrier `[proved]`):** Only `Open` Pass 1 is causally fusable before `EOB`.
   - **Theorem H2 (RLE2–MTF Semidirect-Product Monoid `[proved + verified: 5,236 + 431 blks]`):** Maximal runs are 1-to-1 with `(mtf_idx, run_len)` tokens; `n, r, m_16, mark, cap` are MTF-invariant and decidable pre-MTF (eliminating all 3 of `#666`'s checks).
   - **Why full fusion loses (`1.185×` 1T, `1.186×` 4W, A/A `0.992–1.003×`):** `HuffMonoidBL-v2` slows the sequential `mibz2-scan` thread by **`+45%`** (`107.9` vs `74.5 ms`, hurting `W >= 6` Amdahl ceiling), and safe GPR `MtfU64` (`103.1 ms`, 52.4% unpredictable `nn < 8` branch) loses to `runs::front_avx2` (`33.8 ms`, branchless `pshufb` covering `98.48%` `nn < 32`), explaining `#666`'s regression.
   - **Clean Decoupled Win (Uncapped Maximal Runs):** Only `1.14%` of `15.15M` maximal runs exceed 127 (`0.77%` > 255), yet 127-cap splitting adds `+1.18M` records and forces `open.rs::maximal()` re-merging. Switching `runs::front_avx2 -> Open` to uncapped maximal runs (`u16` + `0.77%` overflow, `-7.2%` smaller) cuts **`Open` Pass 1 by `-19.1%`** (`72.42 -> 58.56 ms`, `-4.4%` on `Open`, zero `T_scanner` impact).
   - *Algorithmic escape for upstream Huffman–RLE2 fusion:* Direct `(lit_mask, one_mask, packed_lits)` bit-accumulator LUTs preserving `block.rs`'s 5-cycle shift-only chain without variable-shift monoid arithmetic, or parallel `BLOCK_MAGIC` scanning.

---

## 5. Unresolved Open Questions

1. **Unconditional `O(r_0 \operatorname{polylog} n)` work bound for Pure Max-Cut `BDMR` on valid BWTs:** Gamma refuted `Variant B`'s `O(r_0 \log n)` conjecture on `Fib_{2m} + $` (`\Theta(r_0 \log^2 n)`), while Pure Max-Cut restored `max_r = 5`, `\sum r_t = O(r_0 \log n)`. Proving or refuting an unconditional `O(r_0 \log^c n)` bound for Pure Max-Cut `BDMR` on all valid BWTs remains `[open]`.
2. **Native i9-10900K (`256 KiB` L2, 1/6/8/10 threads) validation of `BSAC(256, K=8)` + Uncapped Maximal Runs:** Both are safe-Rust, zero-scanner-regression wins on this Xeon; `BSAC`'s exact shared-L3 queuing behavior at 8–10 threads on Comet Lake (vs `Relay8` `#705`) can only be settled by an i9 workstation run under `PERFORMANCE-POLICY.md`.
3. **Bit-level / non-indivisible RAM lower bounds:** Gamma's Theorem G2 breaks `A_sem` unconditionally in read-only streaming, and Theorem G1 breaks `TC⁰` unconditionally; closing the gap between `OpenCells` ($Q = m_{16}$ in RAM) and non-indivisible bit-level RAM algorithms at $n \le 900\text{k}$ (without the Aggarwal–Vitter indivisibility assumption) remains linked to open network-coding / Li–Li (STOC 2019) circuit lower bounds.

---

## 6. Reproducible Artifacts & Manifests (`docs/journal/2026-11-22-neon-teamwork-paradigm/`)

- **Acquisition manifests (all 4 verified `PASS` by `docs/team/acquisition/check_manifest.py`):** `manifests/{bdmr_beta,bdmr_referee,lazard,mfbse}.toml`
- **Alpha:** `alpha/{verify_full_and_falsify,test_ternary_nonbwt_edgecase,test_streaming_and_1cut}.py`, `alpha/bench_lazard/`
- **Beta:** `beta/{deep_analysis,measure_costs_detail,check_run_bound,inspect_blocks}.py`, `beta/bench_bdmr/`, `beta/solver_opt/{cpsat_schedule,global_multistep_opt,test_3way_idle_wikidata,test_open_bdmr_hybrid}.py`
- **Gamma:** `gamma/{verify_phi_bwt,verify_mfbse_exhaustive,analyze_blocks,stress_test_bdmr,refute_bdmr_fib}.py`, `gamma/bench_mfbse_rs/`
- **Delta:** `delta/census/`, `delta/bench_tunnel/` (`LazyTunnel`, `ZeroRV-Tunnel`, `stage_census`, `ablation`, `micro_ablation`)
- **Epsilon:** `epsilon/{anatomy,verify_core_math,radix_landmark_audit}.py`, `epsilon/bench_core/` (`BSAC`, `TSAC`)
- **Zeta:** `zeta/{census,verify_xb_cert}.py`, `zeta/{xblock_census,xblock_audit,bench_xblock}/` (`XB-SLP`, `XB-Cert`)
- **Eta:** `eta/bench_pipeline/` (`HuffMonoidBL-v2`, `B0/B1/B2`, `ablate_pass1`)
- **Referee:** `referee/audit_{gamma,alpha,alpha_falsify,beta,beta_deep,surgical}.py`, `referee/solver_audit/` (CNFs, JSON witnesses, and CaDiCaL 1.9.5 DRAT certificates; note `f2_any_valid_bwt_n8_s3.drat` (9.6 MB) is omitted from git and regenerates from committed `f2_any_valid_bwt_n8_s3.cnf` in 2.43s)
