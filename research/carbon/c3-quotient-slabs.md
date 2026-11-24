# C-3: Polychromatic Quotient Slabs ($\Pi_R$ Subalphabet-Merged Runs with SWAR Residuals)

## 1. Object and Input-Defined Cost Parameter

- **What comes after monochromatic intervals in row space?**
  - A monochromatic BWT run $(c, a, w)$ is an interval where $|\Sigma(S)| = 1$ ($0$ residual bits/row, $1$ rigid translation offset $\mathrm{dst}$).
  - Fix a subalphabet $\Sigma_R \subset \Sigma$ capturing the novel-content alphabet (`hex16` $= \texttt{[0-9a-f]}$, $\sigma_R = 16$, exact $4$ bits/nibble; or `hex23` $= \texttt{[0-9a-fA-F-]}$, $\sigma_R = 23$) and quotient map $\pi_R : \Sigma \to \{\square\} \sqcup (\Sigma \setminus \Sigma_R)$ sending $\Sigma_R \mapsto \square$.
  - **Object ($\Phi_{\mathrm{slab}}$):** a **Polychromatic Quotient Slab** is a maximal interval $[a, a+w)$ of constant $\pi_R(L[x])$:
    - if $c \notin \Sigma_R$ (or $w > 16$ under `thin16`), a monochromatic translation $([a, a+w), h=1, (c, \mathrm{dst}))$;
    - if $\pi_R = \square$, a **SWAR Wavelet Slab** $([a, a+w), h=1, (\mathrm{base}[0..\sigma_R), P_{\mathrm{swar}}, \mathrm{ckpt}_{64}))$ packing $16$ width-1 hex rows per `uint64_t` word and routing via branchless 64-bit SWAR nibble-rank:
      $$\mathrm{LF}(a + i) \;=\; \mathrm{base}[d] \;+\; \mathrm{ckpt}[i \gg 6][d] \;+\; \mathrm{swar\_rank}_{16}(P_{\mathrm{swar}}, i, d), \qquad d = P_{\mathrm{swar}}[i].$$
  - **Satisfaction of `(1)` and `(2)` (`[proved]` + `[measured]`):**
    - Input-defined parameter: $r_\pi(L) = \#\text{maximal }\pi_R\text{-runs in }L$ (and width-1 slab count $q_{w1}(L)$).
    - Where monochromatic runs shatter on novel UUIDs/SHA1s, consecutive hex/hyphen runs fuse into single slabs: under `hex23_pure`, **$r_\pi / h_1 = 0.307\text{--}0.443$** ($\mathbf{0.241\text{--}0.391 \times T^2}$, below $0.45\,h_1$ on **6/6 blocks**), and **$q_{w1} / h_1 = 0.117\text{--}0.181$** ($82\text{--}88\%$ of width-1 runs eliminated).

## 2. Declared Instruction Set ($\mathcal{I}_{\mathrm{slab}}$)

1. `SLAB_PACK_STREAM` ($Q_{\mathrm{acq}} = 0, D_{\mathrm{acq}} = 0, s_{\max} = 2$): one sequential pass over the $r$ run records packing $\Sigma_R$-runs into `uint64_t` SWAR words, recording $64$-row local rank checkpoints, and filling a $64$-row block directory `blk_slab[0..n/64]` ($56\,\mathrm{KiB}$).
2. `MONO_STEP` ($Q = 1$ slab lookup, $D = 1$): evaluate rigid translation $\mathrm{mono\_dst} + (x - a)$ and emit `mono_byte`.
3. `SWAR_SLAB_STEP` ($Q = 1$ slab/SWAR lookup, $D = 1$): extract $4$-bit nibble $d$, evaluate parallel `uint64_t` nibble-equality mask `(~(w ^ (0x1111...1ULL * d)))` + `__builtin_popcountll`, emit $\Sigma_R[d]$, and jump to $\mathrm{base}[d] + \mathrm{rnk}$.

## 3. Exact Construction & Instruction Counts on the 6 Admitted Blocks (`[measured]`)

Implemented in [`c3_quotient_slabs.c`](src/c3_quotient_slabs.c) and verified by [`run_c3.py`](scripts/run_c3.py). All **24 block-configurations + 16 unit tests** decode **byte-exact** (`memcmp == 0`), with **`Q_acq = 0`, `D_acq = 0`, `s_max = 2`** on every block.

| Block | $h_1$ | $T^2 (m_{16}/2)$ | `hex23` $r_\pi$ ($/h_1$, $/T^2$) | `hex23` $q_{w1}$ ($/h_1$) | `Acq`: $(Q_{\mathrm{acq}}, P, s, S)$ (`hex23_thin16`) | `Run` thin steps $Q_{\mathrm{run,thin}}$ ($/h_1$, $/T^2$) |
|---|---:|---:|---:|---:|---|---:|
| `block-0.L` | 35,116 | 44,937.0 | **10,901** (`0.310`, `0.243`) | **4,110** (`0.117`) | `(0, 1.06, 2, 1.21 MB)` | 89,874 (`2.56`, `2.000`) |
| `block-2.L` | 34,262 | 56,035.0 | **14,761** (`0.431`, `0.263`) | **5,007** (`0.146`) | `(0, 1.21, 2, 1.52 MB)` | 112,070 (`3.27`, `2.000`) |
| `block-85.L` | 55,504 | 78,202.0 | **18,871** (`0.340`, `0.241`) | **7,704** (`0.139`) | `(0, 1.64, 2, 1.73 MB)` | 156,404 (`2.82`, `2.000`) |
| `block-143.L` | 72,578 | 98,420.5 | **32,133** (`0.443`, `0.326`) | **12,164** (`0.168`) | `(0, 2.40, 2, 2.91 MB)` | 196,841 (`2.71`, `2.000`) |
| `block-210.L` | 97,054 | 102,140.5 | **29,829** (`0.307`, `0.292`) | **11,317** (`0.117`) | `(0, 2.59, 2, 2.76 MB)` | 204,281 (`2.10`, `2.000`) |
| `block-207.L` | 62,407 | 67,960.5 | **26,602** (`0.426`, `0.391`) | **11,292** (`0.181`) | `(0, 1.91, 2, 2.27 MB)` | 135,921 (`2.18`, `2.000`) |

*(Under `hex16` exact 4-bit `u64` SWAR: $r_\pi / h_1 = 0.672\text{--}0.892$, `swar_words` $= 18{,}965..22{,}869 = 0.24\text{--}0.60\,h_1$, $Q_{\mathrm{acq}} = 0$, $Q_{\mathrm{run,thin}} = m_{16}$.)*

## 4. Where the Cost Goes — Theorem QS-1 (Necessity of Height $h > 1$)

- **Why Acquisition Succeeds Completely (`[proved]` + `[measured]`):** `C-3` meets `(1)`, `(2)` on object count ($r_\pi = 0.31\text{--}0.44\,h_1$), and `(3)` strictly within budget ($Q_{\mathrm{acq}} = 0, D_{\mathrm{acq}} = 0, P \le 2.59, s = 2, S \le 2.91\,\mathrm{MB}$).
- **Theorem QS-1 (Height-1 Row-Space Aggregation Cannot Lower $Q_{\mathrm{run}}$) `[proved]`:**
  In Record 111's basis model $\omega = (S, h, \varphi)$ where object uses tile the $n$ output positions, every object of height $h = 1$ emits **one output byte per use**, regardless of $|S|$. Therefore any decoder whose thin-row objects all have $h = 1$ executes at least $m_{16} \ge h_1$ uses on $\mathrm{Thin}_{16}$; when those uses occur at data-dependent addresses along the orbit ($|S_{\mathrm{active}}| \gg s_{\max}$), $Q_{\mathrm{run,thin}} = m_{16} = 2.000 \times T^2 > h_1$ (`[proved]`, matching all 6 blocks to the exact integer).
- **Structural Lesson:** Row-space subalphabet merging (`C-3`, $h=1, |S| \gg 1$) eliminates **build cost and descriptor count** ($Q_{\mathrm{acq}} = 0, r_\pi \ll h_1$), whereas only **orbit-height $h > 1$** (`C-1`, `C-2`, `C-4`) can lower $Q_{\mathrm{run}}$ below $h_1$.
