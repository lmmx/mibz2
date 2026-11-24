# C-1: Bipartite Boundary-Return Chords ($\partial\mathrm{Thin}_L = \mathrm{LF}(\mathrm{Thick}_L) \cap \mathrm{Thin}_L$)

## 1. Object and Input-Defined Cost Parameter

- **Motivation (working backwards from where intervals fail):**
  - `LF` walk acts on **points** ($n$ rows).
  - Rauzy–Veech (`Cells`) acts on **monochromatic intervals** ($r$ runs).
  - `Open` induces the first-return map on the **thin set** $\mathrm{Thin}_L = \{x : w_{\mathrm{run}}(x) \le L\} \cup \{p\}$, eliminating $\mathrm{Thick}_L = [0, n) \setminus \mathrm{Thin}_L$ via interval exchange, so its domain size remains $m_L = |\mathrm{Thin}_L| \ge h_1$.
  - **What comes after intervals (`C-1`):** induce one level further onto the **topological interface** between the repeated scaffold and novel content:
    $$B_L \;=\; \bigl(\mathrm{LF}(\mathrm{Thick}_L) \cap \mathrm{Thin}_L\bigr) \;\cup\; \bigl(\{p\} \text{ if } \mathrm{Thick}_L = \emptyset\bigr).$$
  - **Object ($\Phi_{\mathrm{chord}}$) `[proved]`:** each $b \in B_L$ carries a **Boundary-Return Chord**
    $$\omega_b \;=\; \bigl(\{b\},\; h_{\mathrm{thin}}(b) + h_{\mathrm{thick}}(b),\; (b',\; w_{\mathrm{thin}}(b),\; \mathrm{band}_{\mathrm{thick}}(b))\bigr),$$
    where $w_{\mathrm{thin}}(b) \in \Sigma^{h_{\mathrm{thin}}(b)}$ is the complete literal excursion from $b$ across scattered thin runs until first entering $y \in E_{\mathrm{out}} = \mathrm{LF}(\mathrm{Thin}_L) \cap \mathrm{Thick}_L$, and $\mathrm{band}_{\mathrm{thick}}(b)$ is the rigid scaffold return from $y$ to $b' \in B_L$.
  - **Satisfaction of `(1)` / `(N1)` `[proved]`:** $\omega_b$ is neither a single row ($h > 1$), nor an interval of rows (the rows traversed by $w_{\mathrm{thin}}(b)$ are scattered across disjoint width-1 runs), nor a positional copy plan (its literal word $w_{\mathrm{thin}}(b)$ occurs once in $T$ and carries no source text offset).
- **Input-defined cost parameter ($\pi(L) = \chi_L$) `[proved]`:**
  $$\chi_L(L_{\mathrm{col}}) \;=\; \bigl|\{y \in \mathrm{Thick}_L : \mathrm{LF}(y) \in \mathrm{Thin}_L\}\bigr|$$
  ($1$ when $\mathrm{Thick}_L = \emptyset$).
  - **Satisfaction of `(2)` / `(N2)` `[proved]` + `[measured]`:** inserting a novel random hex token of length $\ell$ (e.g., a 36-byte UUIDv4 or 40-byte SHA1 occurring once between repeated JSON delimiters in $\mathrm{Thick}_L$) increases $h_1$ by $\approx \frac{225}{256}\ell \approx 0.879\ell$ ($+32..35$) and $T^2 = m_{16}/2$ by $+\ell/2$ ($+18..20$), while increasing $\chi_L$ by **at most $+1$** (independent of $\ell$, provided no internal substring accidentally lands in a width-$>L$ run) `[proved]`.
  - On the six admitted blocks (`[measured]`):
    - $\chi_2 / h_1 = \mathbf{0.229\text{--}0.320}$ ($\mathbf{0.161\text{--}0.285 \times T^2}$, below $0.33\,h_1$ on **6/6 blocks**).
    - $\chi_4 / h_1 = \mathbf{0.260\text{--}0.424}$ ($\mathbf{0.212\text{--}0.298 \times T^2}$, below $0.43\,h_1$ on **6/6 blocks**).
    - $\chi_{16} / h_1 = \mathbf{0.367\text{--}0.652}$ ($\mathbf{0.279\text{--}0.453 \times T^2}$, below $0.46\,T^2$ on **6/6 blocks** and below $0.50\,h_1$ on **4/6 blocks**).

## 2. Declared Instruction Set ($\mathcal{I}_{\mathrm{chord}}$, Record 111 Cost Vector)

Under Record 111's word-RAM model (`(Q, D, P, S)` with sequential $\pm 1$ cursors at $Q = 0$):
1. `RUN_SCAN`: stream maximal run records $(c_i, w_i)$, accumulate histogram $C[0..256)$, classify $\mathrm{Thin}_L$ vs $\mathrm{Thick}_L$, and intersect sorted $\mathrm{LF}(\mathrm{Thick}_L)$ with $\mathrm{Thin}_L$ via two linear cursors ($Q = 0, D = 0, s \le 3$).
2. `THICK_PULL`: backward interval-exchange pull from $B_L$ across $\mathrm{Thick}_L$ strips (sorted binary search on strip targets, zero row-by-row `LF` walk; charges $Q_{\mathrm{acq,thick}}$ per interval binary-search probe).
3. `THIN_CURSOR_PULL` ($Q = 0, D = 0, s = 257$): one pass $x = 0..n-1$ reading successor state at $y = \mathrm{LF}(x)$ via $256$ per-byte $+1$ cursors, clamped at $\mathrm{Thick}_L$ (absorbing into $B_L$).
4. `THIN_DAG_COMPOSE` ($Q = 1$ per unresolved thin row, $D = 1$ per sweep): clamped pointer-jumping / Gauss–Seidel compose $J[x] \leftarrow J[J[x]]$ on thin rows not yet absorbed into $\mathrm{Thick}_L$.
5. `CHORD_EMIT` ($Q = 1, D = 1$ per consumed chord): emit $w_{\mathrm{thin}} \Vert w_{\mathrm{thick}}$ to $T$ and advance $b \leftarrow J[b]$.

## 3. Exact Construction & Instruction Counts on the 6 Admitted Blocks (`[measured]`)

Implemented in [`c1_boundary_chords.c`](src/c1_boundary_chords.c) and verified by [`run_c1.py`](scripts/run_c1.py). All **66 block-configurations + 15 edge-case unit tests** (including multi-cycle periodic `"bbaa"` and `baaaa`) decode **byte-exact** (`memcmp == 0` across all $5{,}399{,}692$ bytes).

### 3.1 Exact Converged $B_L$ Chords (`Mode C`: Consumer Payoff vs $h_1$ and $T^2 = m_{16}/2$)

| Block | $h_1$ | $T^2 (m_{16}/2)$ | $\chi_{16}$ ($Q_{\mathrm{run}}/h_1$, $/T^2$) | $\chi_4$ ($Q_{\mathrm{run}}/h_1$, $/T^2$) | $\chi_2$ ($Q_{\mathrm{run}}/h_1$, $/T^2$) | `bands` ($L=16 / 4 / 2$) |
|---|---:|---:|---:|---:|---:|---:|
| `block-0.L` | 35,116 | 44,937.0 | **15,278** (`0.435`, `0.340`) | **9,532** (`0.271`, `0.212`) | **8,052** (`0.229`, `0.179`) | 27.4k / 93.2k / 185.1k |
| `block-2.L` | 34,262 | 56,035.0 | **22,324** (`0.652`, `0.398`) | **14,538** (`0.424`, `0.259`) | **9,002** (`0.263`, `0.161`) | 24.1k / 117.9k / 189.8k |
| `block-85.L` | 55,504 | 78,202.0 | **21,786** (`0.393`, `0.279`) | **18,522** (`0.334`, `0.237`) | **13,463** (`0.243`, `0.172`) | 12.3k / 72.1k / 130.7k |
| `block-143.L` | 72,578 | 98,420.5 | **44,622** (`0.615`, `0.453`) | **26,558** (`0.366`, `0.270`) | **23,249** (`0.320`, `0.236`) | 32.0k / 149.7k / 222.7k |
| `block-210.L` | 97,054 | 102,140.5 | **35,571** (`0.367`, `0.348`) | **25,264** (`0.260`, `0.247`) | **24,330** (`0.251`, `0.238`) | 18.9k / 65.4k / 136.4k |
| `block-207.L` | 62,407 | 67,960.5 | **25,963** (`0.416`, `0.382`) | **20,224** (`0.324`, `0.298`) | **19,360** (`0.310`, `0.285`) | 21.3k / 96.2k / 177.3k |

### 3.2 $O(1)$-Pass Non-Walking Acquisition Attack on `(3)`

| Acquisition Variant | $Q_{\mathrm{acq,thin}}$ | $Q_{\mathrm{run}} / h_1$ (6-block range) | $Q_{\mathrm{run}} / T^2$ (6-block range) | $(Q_{\mathrm{acq,thin}} + Q_{\mathrm{run}}) / h_1$ |
|---|---:|---:|---:|---:|
| **`sweep_k1_L2`** (1 pure-cursor sweep, $L=2$, $s=257$) | **0** | **`0.793–0.911`** ($< h_1$ on **6/6**) | **`0.508–0.837`** ($< T^2$ on **6/6**) | **`0.793–0.911`** |
| **`push_d4`** (4 pure-cursor pull passes, $L=4$, $s=257$) | **0** | **`0.531–0.710`** ($< h_1$ on **6/6**) | **`0.434–0.544`** | **`0.531–0.710`** |
| **`sweep_k2_L2`** (1 cursor + 1 addressed DAG sweep, $L=2$) | `0.93–1.02 * h1` | **`0.356–0.439`** ($< 0.44\,h_1$ on **6/6**) | **`0.235–0.393`** | `1.341–1.460` |
| **`sweep_k3_L2`** (1 cursor + 2 addressed DAG sweeps, $L=2$) | `1.52–1.66 * h1` | **`0.268–0.357`** ($< 0.36\,h_1$ on **6/6**) | **`0.184–0.318`** | `1.881–1.956` |
| **`exact_L16`** (converged Thin-DAG, $L=16$) | `3.45–4.99 * h1` | **`0.367–0.652`** | **`0.279–0.453`** | `4.022–5.539` |

*(Under Record 108's strict scalar convention, Pass 1 touches every thin row record via cursors, so `strict_c1_acq` $= m_L \ge h_1$ on all variants `[proved]`.)*

## 4. Where the Cost Goes (Named Near Miss & Two-Sided Conservation Law)

1. **Thick-Band Fragmentation as $L$ Decreases (`[measured]` + `[argued]`):**
   Driving $\chi_L$ below $0.33\,h_1$ requires lowering $L$ from $16$ to $2$ or $4$ so short repeated punctuation runs ($w \in [3, 16]$) act as scaffold anchors. Moving $r_{3..16}$ runs into $\mathrm{Thick}_L$ injects $2 r_{3..16}$ cut endpoints into the interval exchange: resolved thick exit bands jump from $12.3\text{k}..32.0\text{k}$ ($0.22\text{--}0.78\,h_1$ at $L=16$) to $130.7\text{k}..222.7\text{k}$ ($1.35\text{--}5.54\,h_1$ at $L=2$), inflating $Q_{\mathrm{acq,thick}}$ (our uncontracted interval binary-search pull spends $0.13\text{M}..0.40\text{M}$ probes at $L=16$ and $1.84\text{M}..3.21\text{M}$ at $L=2$; even an optimal $O(\text{bands})$ Rauzy–Veech induction pays $\ge \text{bands} > h_1$ at $L=2$).
2. **Thin-DAG Compaction Cost (`[proved]` + `[measured]`):**
   Inside $\mathrm{Thin}_L$, `THIN_CURSOR_PULL` achieves $Q_{\mathrm{acq,thin}} = 0$ on Pass 1 because `LF` is order-preserving on each of the $256$ single-byte classes (`sweep_k1_L2` already pushes $Q_{\mathrm{run}}$ below $h_1$ on all 6 blocks at $0.793\text{--}0.911\,h_1$ with $Q_{\mathrm{acq,thin}} = 0$). However, after Pass 1, variable-length thin chords $J_1[x] = \mathrm{LF}^{\ell_1(x)}(x)$ cease to be monotone within the $256$ single-byte classes, so Sweep 2 pays one addressed read per unresolved thin row ($Q_{\mathrm{acq,thin}} \approx 0.98\,h_1$), keeping $Q_{\mathrm{acq,thin}} + Q_{\mathrm{run}} > 1.34\,h_1$.
3. **Direct Bridge to `C-2`:** To compact $\mathrm{Thin}_L$ *without* losing cursor monotonicity after $1$ byte, a single cursor pass must step $k \ge 2$ symbols at once on the novel hex subalphabet. That is `C-2`.
