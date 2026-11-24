# C-2: Subalphabet Radix-Fiber $k$-Cones ($\Sigma_R^k$ Order-Isomorphic $\mathrm{LF}^k$ Bundles)

## 1. Object, Theorem RF-1, and Input-Defined Cost Parameter

- **Hilbert's 24th Problem — What lies between Points and Rauzy–Veech Intervals?**
  - At full alphabet rank $\sigma = 256$, `LF` splits row space into $256$ symbol buckets (`k = 1` step per $|\Sigma|$-cursor pass: **Points**).
  - At monochromatic rank $\sigma = 1$, a run maps rigidly for arbitrary height $h$ (`k = \infty` on $1$ bucket: **Rauzy–Veech Intervals**).
  - Where runs fail on real corpora ($\sim 65\%$ of $h_1$: novel hex UUIDs, SHA1s, numeric IDs), the local effective alphabet has intermediate rank $1 < \sigma_R \ll |\Sigma|$ ($\sigma_R = 16$ for `[0-9a-f]`, $\sigma_R = 18$ for `[0-9a-f-"]`).
  - Within cursor budget $s_{\max}$, a regime of rank $\sigma_R$ supports stride
    $$k(\sigma_R, s_{\max}) \;=\; \lfloor \log_{\sigma_R} s_{\max} \rfloor$$
    simultaneous `LF` steps per pure-cursor pass ($k = 2$ at $\sigma_R = 16, s_{\mathrm{cone}} = 256 = |\Sigma|$; $k = 3$ at $\sigma_R = 16, s_{\mathrm{cone}} = 4{,}096$, fitting in $16\,\mathrm{KiB}$ L1d).

- **Theorem RF-1 ($\mathrm{LF}^k$ Order-Isomorphism & Zero-Walk Cone Synthesis) `[proved]`:**
  Let $u = c_1 c_2 \cdots c_k \in \Sigma^k$, $D_{u^R} = \{x \in [0, n) : L_1[x]=c_k, \dots, L_k[x]=c_1\}$ where $L_j[x] = L[\mathrm{LF}^{j-1}(x)]$, and $I_u = \{z \in [0, n) : F_1\cdots F_k[z] = u\}$.
  1. **Contiguity & Order-Isomorphism:** $I_u = [C_k(u), C_k(u) + |I_u|)$ is a contiguous BWT row interval, and $\mathrm{LF}^k\big|_{D_{u^R}} : D_{u^R} \to I_u$ is the unique strictly increasing bijection ($x_1 < x_2 \iff \mathrm{LF}^k(x_1) < \mathrm{LF}^k(x_2)$).
     *Proof:* For $z_i = \mathrm{LF}^k(x_i) \in I_u$, $\mathrm{suffix}(z_i) = u \cdot \mathrm{suffix}(x_i)$. Since $z_1, z_2$ share prefix $u$, lexicographic suffix order gives $z_1 < z_2 \iff u \cdot \mathrm{suffix}(x_1) < u \cdot \mathrm{suffix}(x_2) \iff \mathrm{suffix}(x_1) < \mathrm{suffix}(x_2) \iff x_1 < x_2$. $\square$
  2. **Pure $\pm 1$ Cursor Realisation ($Q = 0$):** As $x$ scans $0 \to n-1$ (or $n-1 \to 0$), a per-cone cursor initialized to $C_k(u)$ (or $C_k(u) + |I_u|$) visits $\mathrm{LF}^k(x)$ for every $x \in D_{u^R}$ by moving strictly by $+1$ (or $-1$), incurring **zero addressed memory accesses ($Q = 0$)**.
  3. **Zero-Walk Prefix Synthesis:** Because $z = \mathrm{LF}(w)$ satisfies $F_1[z] = L[w]$ and $F_j[z] = F_{j-1}[w]$,
     $$|I_{c_1 c_2 \cdots c_k}| \;=\; \#\bigl\{w \in [0, n) : L[w]=c_1,\; F_1[w]=c_2,\; \dots,\; F_{k-1}[w]=c_k\bigr\}.$$
     At $k = 2$, $|I_{c_1 c_2}|$ is folded directly from the run records in $O(r)$ with **zero extra passes**. At $k = 3$, $F_2[\mathrm{LF}(w)] = F_1[w]$ is synthesized in one $256$-cursor scatter pass ($Q = 0$).

- **Object ($\Phi_{\mathrm{cone}}$) & Cost Parameter ($\pi_{\mathrm{RF}}$):**
  - Object: a **Radix-Cone Chain** $\omega_x = (\{x\}, h, (x', w_{\mathrm{inline}}, \mathrm{band}_{\mathrm{opt}}))$ stored in a flat $64$-byte `ConeSlot[0..n)` array so that both $\mathrm{LF}^k(x)$ targets and inline words are read directly through the $\pm 1$ cone cursors ($Q_{\mathrm{acq,thin}} = 0$), clamped at $\mathrm{Thick}_{16}$.
  - Parameter: $\pi_{\mathrm{RF}}(L) = \chi_{16} + \sum_{\text{seg}} \frac{|\text{seg}|}{k(\sigma_{\text{seg}}) \cdot c_{\mathrm{arch}}}$, scaling inversely with $k = \lfloor \log_{\sigma_R} s_{\max} \rfloor$ on low-radix novel content (`(1)` and `(2)` met `[proved]`).

## 2. Declared Instruction Set ($\mathcal{I}_{\mathrm{cone}}$)

1. `CONE_HIST_SYNTH` ($Q = 0, D = 0, s \le 257$): fold $C_2$ (and $C_3$ via one $F_2$ scatter) plus lookahead byte streams $L_2, L_3$ via $256$ sequential cursors.
2. `THICK_OVERLAP_PULL` ($Q = Q_{\mathrm{acq,thick}}$): pre-merge `strips_from` and radix-sorted `strips_to` with two cursors ($Q = 0$), then pull exit bands across `ov_lo[f]..ov_hi[f]` ($1$ addressed probe per overlapping strip).
3. `CONE_SWEEP_ASC` / `CONE_SWEEP_DESC` ($Q_{\mathrm{acq,thin}} = 0, D = 0$): ascending ($+1$) and descending ($-1$) passes over `ConeSlot[0..n)` using $\sigma_R^k$ cone cursors + $256$ fallback cursors. Every composition reads `slots[z]` directly under its $\pm 1$ cursor.
4. `CONE_EMIT` ($Q = 1$ per slot step $+ 1$ per spilled thick band $> 52$ bytes, $D = 1$ per step).

## 3. Exact Construction & Instruction Counts on the 6 Admitted Blocks (`[measured]`)

Implemented in [`c2_radix_cones.c`](src/c2_radix_cones.c) and verified by [`run_c2.py`](scripts/run_c2.py). All **48 block-configurations + 40 unit tests** decode **byte-exact** (`memcmp == 0`), with **`q_acq_thin = 0` across all 48 runs** and Theorem RF-1 assertions (`target == slots[x].next`) verified on every row.

### 3.1 Consumer Count $Q_{\mathrm{run}}$ (at $Q_{\mathrm{acq,thin}} = 0$, $L = 16$) vs $h_1$ and $T^2 = m_{16}/2$

| Block | $h_1$ | $T^2$ | `k1_base` 1sw / 2sw ($/h_1$) | `RF2_hex16` 1sw / 2sw ($/h_1$, 2sw $/T^2$) | `RF2_uuid18` 2sw ($/h_1$, $/T^2$) | `RF3_hex16` 1sw / 2sw ($/h_1$, 2sw $/T^2$) |
|---|---:|---:|---:|---:|---:|---:|
| `block-0.L` | 35,116 | 44,937.0 | `1.674` / `1.142` | `1.305` / **`0.934`** (`0.730`) | **`0.922`** (`0.721`) | `1.204` / **`0.876`** (`0.685`) |
| `block-2.L` | 34,262 | 56,035.0 | `2.136` / `1.469` | `1.697` / `1.219` (`0.746`) | `1.204` (`0.736`) | `1.583` / `1.149` (`0.703`) |
| `block-85.L` | 55,504 | 78,202.0 | `1.692` / `1.123` | `1.304` / **`0.890`** (`0.631`) | **`0.877`** (`0.623`) | `1.199` / **`0.829`** (`0.588`) |
| `block-143.L` | 72,578 | 98,420.5 | `1.757` / `1.189` | `1.398` / **`0.994`** (`0.733`) | **`0.983`** (`0.725`) | `1.319` / **`0.955`** (`0.704`) |
| `block-210.L` | 97,054 | 102,140.5 | `1.317` / `0.877` | `1.085` / **`0.748`** (`0.711`) | **`0.736`** (`0.700`) | `1.021` / **`0.720`** (`0.684`) |
| `block-207.L` | 62,407 | 67,960.5 | `1.369` / `0.931` | `1.134` / **`0.802`** (`0.736`) | **`0.792`** (`0.727`) | `1.079` / **`0.775`** (`0.711`) |

**Key quantitative findings (`[measured]`):**
- **Beats $T^2 (m/2)$ on 6/6 blocks in ONE pure-cursor pass:** `RF3_hex16_1sw` reaches $Q_{\mathrm{run}} = \mathbf{0.851\text{--}0.990 \times T^2}$ in a single ascending sweep with $Q_{\mathrm{acq,thin}} = 0$.
- **Beats $h_1$ on 5/6 blocks and beats $T^2$ by $29\text{--}41\%$ on 6/6 blocks in TWO pure-cursor sweeps ($Q_{\mathrm{acq,thin}} = 0$):**
  - `RF2_hex16_2sw` ($16^2 = 256 = |\Sigma|$ cone cursors): $Q_{\mathrm{run}} = \mathbf{0.748\text{--}0.994\,h_1}$ on 5/6 blocks ($1.219\,h_1$ on `block-2.L` where $m_{16}/h_1 = 3.271$), and $\mathbf{0.631\text{--}0.746 \times T^2}$ on 6/6 blocks.
  - `RF2_uuid18_2sw` (`[0-9a-f-"]`, $324$ cone cursors): $Q_{\mathrm{run}} = \mathbf{0.736\text{--}0.983\,h_1}$ on 5/6 blocks ($\mathbf{0.623\text{--}0.736 \times T^2}$).
  - `RF3_hex16_2sw` ($4{,}096$ cone cursors): $Q_{\mathrm{run}} = \mathbf{0.720\text{--}0.955\,h_1}$ on 5/6 blocks ($\mathbf{0.588\text{--}0.711 \times T^2}$), achieving $2.85\text{--}3.40$ retained rows per link vs $2.00$ for $T^2$.

## 4. Where the Cost Goes (Named Near Miss & Relocation Ledger)

1. **Non-Hex Thin Dilution ($m_{16}/h_1 = 2.10\text{--}3.27$) & UUID Hyphen Punctuation (`[measured]`):**
   At $L = 16$, $52\text{--}69\%$ of retained rows $m_{16}$ lie in runs of width $2..16$ or non-hex bytes where $k = 1$ fallback applies; inside UUIDv4 (`8-4-4-4-12`), four hyphens interrupt $\Sigma_{\mathrm{hex}}$ every $4..12$ nibbles (`RF2_uuid18` recovers part of this by including `'-'` and `'"'`). Consequently $Q_{\mathrm{run}}/h_1$ saturates at $0.720\text{--}1.149$ rather than crossing $\delta = 1/2$ ($0.50\,h_1$).
2. **Thick-Interval Overlap Probes ($Q_{\mathrm{acq,thick}}$) (`[measured]`):**
   Pre-merged strip-overlap elimination eliminates all binary searches and cuts $Q_{\mathrm{acq,thick}}$ by $2.5\text{--}3.5\times$ vs `C-1` ($37{,}487..242{,}011 = 0.68\text{--}3.33\,h_1$), yet multi-step thick chains still contribute $> 0.68\,h_1$ to $Q_{\mathrm{total}}$ unless compressed by full Rauzy–Veech induction (`Open`'s `endpoint.rs`).
3. **Record 111 Relocation into $(P, s, S)$ (`[proved]` + `[measured]`):**
   Eliminating addressed thin gathers ($Q_{\mathrm{acq,thin}} = 0$) is paid explicitly in the other three coordinates of Record 111's vector: flat $64\,\mathrm{B}$ `ConeSlot[0..n)` uses $S \approx 57.6\,\mathrm{MB}$ ($64n$), $P = 131\text{--}138$ streamed bytes/$n$ (`2sw`), and $s_{\max} = 513$ (`RF2_hex16`) to $4{,}609$ (`RF3_hex16`). Under Record 108's strict convention, cursor passes over thin rows still count `strict_c1_acq` $\ge m_{16} > h_1$.
