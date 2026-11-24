# Carbon — Post-Interval Basis Campaign (`research/carbon-new-basis`)

Four distinct post-interval BWT basis candidates (`C-1`..`C-4`), de-conflicted against MONODROMY (`claude/sharp-meitner-cx0v78`) and Neon (`research/neon-main-theorem`), constructed in C from run records without walking `LF` row by row, and verified **100% byte-exact** across all six blocks in `data/admitted-bwt` (`174/174` block-configurations + `101` edge-case unit tests, `0` mismatches).

## Summary Matrix Across the 6 Admitted Blocks

| ID | Post-Interval Object | Input Parameter $\pi(L)$ (vs $h_1$, $T^2 = m_{16}/2$) | Best $Q_{\mathrm{acq,thin}}=0$ Run ($Q_{\mathrm{run}}/h_1$, $/T^2$) | Exact Object $Q_{\mathrm{run}}$ ($/h_1$, $/T^2$) | Where the Cost Goes (Named Bottleneck) |
|---|---|---|---|---|---|
| **[`C-1`](c1-boundary-chords.md)** | **Bipartite Boundary Chords** on $\partial\mathrm{Thin}_L = \mathrm{LF}(\mathrm{Thick}_L)\cap\mathrm{Thin}_L$ | $\chi_2 = \mathbf{0.229\text{--}0.320\,h_1}$ (`0.161–0.285 T²`); $\chi_4 = \mathbf{0.260\text{--}0.424\,h_1}$ | `push_d4`: **`0.531–0.710 h₁`** (`0.434–0.544 T²`); `sweep_k1_L2`: **`0.793–0.911 h₁`** | **`0.229–0.320 h₁`** (`0.161–0.285 T²`, 6/6) | Lowering $L: 16\to 2$ fragments thick bands ($27\text{k}\to 223\text{k}$); Sweep 2 adds $\sim 0.98\,h_1$ addressed thin reads |
| **[`C-2`](c2-radix-fiber-cones.md)** | **Subalphabet Radix-Fiber $k$-Cones** ($\Sigma_R^k$ order-isomorphic $\mathrm{LF}^k$ bundles, Thm RF-1) | Stride $k = \lfloor \log_{\sigma_R} s_{\max}\rfloor$ ($k=2$ at $s_{\mathrm{cone}}=256$, $k=3$ at $4{,}096$) | `RF3_hex16` 2sw: **`0.720–0.955 h₁` on 5/6** (`0.588–0.711 T²` on 6/6); `RF2_uuid18` 2sw: **`0.736–0.983 h₁` on 5/6** | Same ($Q_{\mathrm{acq,thin}} = 0$ exact) | $m_{16}/h_1 = 2.1\text{--}3.3$ non-hex width-$2..16$ dilution; UUID hyphens breaking $\Sigma_{\mathrm{hex}}^3$; $S = 64n$ relocation |
| **[`C-3`](c3-quotient-slabs.md)** | **Polychromatic Quotient Slabs** ($\Pi_R$ SWAR nibble-packed subalphabet slabs) | `hex23` $r_\pi = \mathbf{0.307\text{--}0.443\,h_1}$ (`0.241–0.391 T²`), $q_{w1} = \mathbf{0.117\text{--}0.181\,h_1}$ | `Q_acq = 0` total ($P \le 2.59, s=2, S \le 2.9\,\mathrm{MB}$); $Q_{\mathrm{run,thin}} = m_{16}$ (`2.000 T²`) | $m_{16} = 2.10\text{--}3.27\,h_1$ | **Theorem QS-1 [proved]:** height $h=1$ row-space aggregation cannot lower single-orbit $Q_{\mathrm{run}}$ below $h_1$; $h > 1$ is necessary |
| **[`C-4`](c4-debruijn-unitigs.md)** | **De Bruijn Unitig–Interval Duality** ($\mathcal{U}_8$ on $|I_{u_8}|=1$ $\sqcup$ intervals on $|I_{u_8}|\ge 2$, Lem U-1) | $u_8^{(16)} = \mathbf{0.066\text{--}0.097\,h_1}$ ($75\text{--}80\%$ of $h_1$); $u_8^{(1)} + h_{1,\mathrm{br}} = \mathbf{0.361\text{--}0.403\,h_1}$ | `u8_L1_cursor2sw`: **`0.522–0.555 h₁`** (`0.335–0.509 T²`, 6/6) | **`0.361–0.403 h₁`** (`0.243–0.370 T²`, 6/6) | **Name-to-Address Wall:** $u_8[\mathrm{LF}(x)]$ is free in registers ($Q=0$), while resolving $64$-bit names to row indices costs $7$ cursor passes or $0.58\,h_1$ $Q_{\mathrm{acq,u}}$; $L=1$ splits thick bands |

## Reproducing All 174 Byte-Exact Runs

```bash
python3 research/carbon/scripts/run_c1.py
python3 research/carbon/scripts/run_c2.py
python3 research/carbon/scripts/run_c3.py
python3 research/carbon/scripts/run_c4.py
```
