# C-4: De Bruijn Unitig–Interval Duality ($\mathcal{U}_k \sqcup \mathcal{I}_k^{\ge 2}$)

## 1. Object, Lemma U-1, and Input-Defined Cost Parameter

- **Complementary Suffix-Space Decomposition (Why Unitigs + Intervals Partition $L$):**
  - At order $k$ ($k = 8$, fitting each row's $8$-byte suffix prefix $F_1\cdots F_8[x]$ in a single `uint64_t` $u_8[x]$), `[0, n)` partitions into maximal $k$-mer suffix intervals $I_u$.
  - **Multiplicity $|I_u| \ge 2$ (Repeats):** Handled by monochromatic intervals (Rauzy–Veech / `Open`).
  - **Multiplicity $|I_u| = 1$ (Novel Content — where intervals fail):** On novel random hex tokens occurring once ($74.9\text{--}79.6\%$ of all width-1 rows $h_1$ at $k = 8$ `[measured]`), $I_u = \{x\}$ has **in-degree $\le 1$ and out-degree $1$** in the de Bruijn graph $\mathrm{dBG}_k(T)$. Maximal chains inside $V_1 = \{x : |I_{u_k[x]}| = 1\}$ contract uniquely into **Non-Branching Eulerian Unitigs** $\mathcal{U}_k$.
- **Object ($\Phi_{\mathrm{unitig}}$) & Parameter ($\pi_{\mathrm{dBG}, k}$) `[proved]`:**
  - Object: a maximal unique-$k$-mer unitig $\omega_\gamma = (\{x_0\}, |\gamma|, (x_{\mathrm{exit}}, w_\gamma))$ starting at $x_0 \in V_1 \setminus \mathrm{LF}(V_1)$ (`(1)` met: neither a row, nor a row-interval since its internal vertices are scattered singleton intervals $|I_{u_t}| = 1$, nor a copy plan since every constituent $k$-mer occurs **once** in $T$).
  - Input-defined parameter (paired with monochromatic $w \ge 2$ intervals):
    $$\pi_{\mathrm{dBG}, k}(L) \;=\; u_k^{(1)}(L) \;+\; h_{1, \mathrm{branch}, k}(L),$$
    where $u_k^{(1)}(L) = |\{y \notin V_1 \cap J_1 : \mathrm{LF}(y) \in V_1 \cap J_1\}|$ is the width-1 unitig count and $h_{1, \mathrm{branch}, k}(L) = |J_1 \setminus V_1|$ is the count of width-1 rows whose $k$-mer occurs $\ge 2$ times.
  - **Satisfaction of `(2)` (`[proved]` + `[measured]`):** An $\ell$-byte novel random hex token ($\ell \ge k$) whose $k$-mers are unique contributes **$+1$** to $u_k^{(1)}$ and **$0$** to $h_{1,\mathrm{branch},k}$ (vs $\approx 0.88\ell$ to $h_1$ and $0.5\ell$ to $T^2$). Across all 6 admitted blocks at $k = 8$:
    - Inside `Open` ($L = 16$), $74.9\text{--}79.6\%$ of $h_1$ collapses into **$u_8^{(16)} = 2{,}712..6{,}631 = 0.066\text{--}0.097 \times h_1$** ($14.7\text{--}16.1$ thin rows per unitig).
    - Paired with full $w \ge 2$ interval induction ($L = 1$), **$\pi_{\mathrm{dBG}, 8} / h_1 = 0.361\text{--}0.403$** ($\mathbf{0.243\text{--}0.370 \times T^2}$, below $0.41\,h_1$ on **6/6 blocks**).

- **Lemma U-1 (Zero-Sort Local $|I_u| = 1$ Certificate & Register-Local Eulerian Successor) `[proved]`:**
  1. Starting from $u_1[x] = F_1[x] \ll 8(k-1)$, $k - 1$ sequential $256$-cursor scatter passes
     $$u_{j+1}[\mathrm{LF}(x)] \;\leftarrow\; \bigl((\mathrm{uint64\_t})L[x] \ll 8(k-1)\bigr) \;\big|\; \bigl(u_j[x] \gg 8\bigr)$$
     synthesize $u_k[0..n)$ with **$Q = 0, D = 0, s = 257$**, and $u_k[0] \le u_k[1] \le \dots \le u_k[n-1]$ is **globally sorted**.
  2. Therefore $x \in V_1 \iff (x = 0 \lor u_k[x] > u_k[x-1]) \land (x = n-1 \lor u_k[x] < u_k[x+1])$ is certified in one linear scan with **zero sorting, zero hashing, and $Q = 0$**.
  3. At every row $x$, the $k$-mer name of $\mathrm{LF}(x)$ equals $((L[x] \ll 8(k-1)) \mid (u_k[x] \gg 8))$ in registers with **zero memory reads**.

## 2. Declared Instruction Set ($\mathcal{I}_{\mathrm{unitig}}$)

1. `KMER_CURSOR_SCATTER` ($Q = 0, D = 0, s = 257$): $k - 1$ sequential $256$-cursor scatter passes synthesizing sorted `uint64_t` $u_k[0..n)$, followed by $1$ linear scan marking $V_1$.
2. `THICK_OVERLAP_PULL` ($Q = Q_{\mathrm{acq,thick}}$): pre-merged strip-overlap pull eliminating monochromatic runs of width $> L$.
3. `UNITIG_CURSOR_COMPACT` ($Q_{\mathrm{acq,unitig}} = 0, D = 0, s = 257$): $1$ or $2$ ($\pm 1$) $256$-cursor sweeps compacting edges strictly inside $V_1 \cap \mathrm{Thin}_L$.
4. `UNITIG_CONVERGE_SWEEP` ($Q = 1$ per unresolved $V_1$ edge, $D = 1$ per sweep): pointer-jumping sweeps compacting $V_1 \cap \mathrm{Thin}_L$ to exact maximal unitigs $\mathcal{U}_k$.
5. `UNITIG_OR_BRANCH_EMIT` ($Q = 1, D = 1$ per step): emit unitig word or $1$-byte branching thin step.

## 3. Exact Construction & Instruction Counts on the 6 Admitted Blocks (`[measured]`)

Implemented in [`c4_debruijn_unitigs.c`](src/c4_debruijn_unitigs.c) and verified by [`run_c4.py`](scripts/run_c4.py). All **36 block-configurations + 30 unit tests** decode **byte-exact** (`memcmp == 0`), with Lemma U-1(1)–(3) asserted on every row ($5.4\text{M}$ rows, $0$ violations).

### 3.1 Exact Unitig–Interval Duality ($k = 8, L = 1$) & Zero-$Q_{\mathrm{acq,u}}$ Cursor Compaction

| Block | $h_1$ | $T^2$ | $v_{1,w1}/h_1$ | Exact $u_8^{(1)}$ ($/h_1$) | $h_{1,\mathrm{branch}}$ | **`u8_L1_exact` $Q_{\mathrm{run}}$** ($/h_1$, $/T^2$) | `u8_L1_exact` $Q_{\mathrm{acq,u}}$ ($/h_1$) | **`u8_L1_cursor2sw` $Q_{\mathrm{run}}$** ($Q_{\mathrm{acq,u}}=0$) ($/h_1$, $/T^2$) |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| `block-0.L` | 35,116 | 44,937.0 | `0.755` | **5,000** (`0.142`) | 8,604 | **13,604** (**`0.387`**, **`0.303`**) | 20,519 (`0.584`) | **19,046** (**`0.542`**, **`0.424`**) |
| `block-2.L` | 34,262 | 56,035.0 | `0.749` | **4,988** (`0.146`) | 8,615 | **13,603** (**`0.397`**, **`0.243`**) | 19,237 (`0.561`) | **18,795** (**`0.549`**, **`0.335`**) |
| `block-85.L` | 55,504 | 78,202.0 | `0.753` | **8,349** (`0.150`) | 13,694 | **22,043** (**`0.397`**, **`0.282`**) | 31,976 (`0.576`) | **30,582** (**`0.551`**, **`0.391`**) |
| `block-143.L` | 72,578 | 98,420.5 | `0.758` | **11,574** (`0.159`) | 17,600 | **29,174** (**`0.402`**, **`0.296`**) | 43,198 (`0.595`) | **40,259** (**`0.555`**, **`0.409`**) |
| `block-210.L` | 97,054 | 102,140.5 | `0.796` | **15,286** (`0.157`) | 19,767 | **35,053** (**`0.361`**, **`0.343`**) | 59,356 (`0.612`) | **50,636** (**`0.522`**, **`0.496`**) |
| `block-207.L` | 62,407 | 67,960.5 | `0.758` | **10,083** (`0.162`) | 15,081 | **25,164** (**`0.403`**, **`0.370`**) | 36,317 (`0.582`) | **34,589** (**`0.554`**, **`0.509`**) |

*(At $L = 16$ inside `Open`'s retained set, $u_8^{(16)} = 2{,}712..6{,}631 = \mathbf{0.066\text{--}0.097 \times h_1}$, while uncontracted width-$2..16$ branching runs leave $Q_{\mathrm{run,branch}} = 52{,}684..120{,}687$, giving $Q_{\mathrm{run}} = 1.104\text{--}2.321\,h_1$.)*

## 4. Where the Cost Goes — The Eulerian Name-to-Address Wall

1. **Name vs Address Separation (`[proved]` + `[measured]`):**
   By Lemma U-1(3), the successor's $64$-bit $8$-mer *name* $u_8[\mathrm{LF}(x)]$ is known locally at $x$ with **$Q = 0$**, and $|I_{u_8}| = 1$ guarantees that name uniquely identifies $\mathrm{LF}(x)$. Yet on a word RAM, arrays are indexed by integer *row addresses* $y \in [0, n)$, not $64$-bit content names. Resolving $u_8[\mathrm{LF}(x)] \mapsto y$ without row-by-row walking requires either:
   - **$|\Sigma|$-cursor sweeps (`u8_L1_cursor2sw`, $Q_{\mathrm{acq,u}} = 0$):** reaches $Q_{\mathrm{run}} = \mathbf{0.522\text{--}0.555\,h_1}$ ($\mathbf{0.335\text{--}0.509\,T^2}$), missing $0.50\,h_1$ by $2\text{--}5\%$, at a cost of $7 + 2 = 9$ cursor passes ($P \approx 144$ streamed bytes/$n$); or
   - **Addressed compaction (`u8_L1_exact`):** reaches $Q_{\mathrm{run}} = \mathbf{0.361\text{--}0.403\,h_1}$ on **6/6 blocks**, paying $Q_{\mathrm{acq,u}} = 0.561\text{--}0.612\,h_1$ ($Q_{\mathrm{acq,u}} + Q_{\mathrm{run}} = \mathbf{0.958\text{--}1.000\,h_1}$).
2. **Interval Side at $L = 1$ (`[measured]`):** As in `C-1`, eliminating all $w \ge 2$ runs ($L = 1$) fragments $\mathrm{Thick}_1$ into $250\text{k}+$ bands ($Q_{\mathrm{acq,thick}} = 2.9\text{M}..5.2\text{M}$), while keeping $L = 16$ leaves width-$2..16$ repeated $8$-mer runs uncontracted (`branch` $= 52.7\text{k}..120.7\text{k}$).
