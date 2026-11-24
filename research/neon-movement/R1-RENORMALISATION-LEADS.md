# R1 — Dynamical-Systems Leads (`LEADS-dynamics.md`, Channel #25): Feigenbaum Renormalisation & Dual-Number Evaluation

**Channel prompt:** `#25` (`docs/journal/2026-11-24-algorithm-level/LEADS-dynamics.md`) — testing Fork's two analogies (Lead 1: Feigenbaum renormalisation $\mathcal{R}f = \alpha^{-1} \circ f \circ f \circ \alpha$; Lead 2: dual-number evaluation) against `(L, LF)` on novel random hex and the six admitted blocks.  
**Script & Data:** [`r1_renormalisation_leads.py`](r1_renormalisation_leads.py), [`results_r1_renormalisation_leads.json`](results_r1_renormalisation_leads.json).

---

## Lead 1: Feigenbaum Renormalisation — Exact Structural Answer

### 1. Exact Invariant for "A Map of the Same Kind" (BWT-Class Closure)
A single-cycle permutation $\sigma$ on $m$ rows is a standard BWT $\mathrm{LF}$ permutation over an alphabet of size $q$ **if and only if** $\sigma^{-1}$ (its forward map $\Psi$) has at most $q$ rising sequences (Gessel–Reutenauer 1993; Crochemore–Désarménien–Perrin 2005). Hence the **exact minimum BWT alphabet size** of $\sigma$ is
$$q_{\min}(\sigma) = 1 + \big|\{y \in [0, m-2] : \sigma^{-1}(y+1) < \sigma^{-1}(y)\}\big|.$$
For raw $\mathrm{LF}$, $q_{\min}(\mathrm{LF}) \le |\Sigma_{\mathrm{active}}|$ always (verified on 8/8 blocks). A renormalisation operator $\mathcal{R}$ returns "a map of the same kind" iff $q_{\min}(\mathcal{R}(\mathrm{LF})) = O(|\Sigma|)$ (stays bounded and small rather than exploding toward a random permutation's $\approx m/2$).

### 2. Three Candidate Renormalisation Operators Tested on All 8 Blocks

1. **Squaring** $\mathcal{R}_{\mathrm{sq}}(\mathrm{LF}) = \mathrm{LF} \circ \mathrm{LF}$:
   - Exact upper bound $q_{\min}(\mathrm{LF}^2) \le |\Sigma|^2$. On novel random hex, $q_{\min}$ jumps from `16` to **`256`** (`r/n: 0.936 -> 0.996`); on the six Wikidata blocks, $q_{\min}$ jumps `16–22×` from `83–94` to **`1,358–1,958`**. Does **not** return a small-alphabet BWT; discontinuities explode exponentially under iteration ($\sim |\Sigma|^{2^j}$, matching `carbon-kitchen-sink` Cycle 7 Thm 7.2).
2. **First-return section renormalisation** $\mathcal{R}_Y(\mathrm{LF}) = \alpha_Y^{-1} \circ \mathrm{LF}^{\tau_Y} \circ \alpha_Y$ (where $\alpha_Y : [0, |Y|) \to Y$ is the order-preserving rescaling of $Y = I_{\text{'0'}}$):
   - On **pure novel random hex**, $q_{\min}$ explodes from `16` to **`1,839` (`0.445 |Y|`)** with `r/|Y| = 0.996` — indistinguishable from a random permutation (`0.50 |Y|`).
   - On **templated 40-hex**, $q_{\min}$ explodes from `24` to **`1,648` (`0.335 |Y|`)**; on **Wikidata**, from `83–94` to **`1,996–4,112` (`0.082–0.200 |Y|`)** (`23–45×` explosion).
   - **Reason (`[argued]` for random return words):** Between two hits of $Y = I_{\text{'0'}}$, the random hex digits generate an independent variable-length word $w$, scrambling the arrival order in $Y$ across all $|Y|$ positions so $\mathcal{R}_Y(\mathrm{LF})$ immediately leaves the small-alphabet BWT class.
3. **Thick-strip contraction onto $M_{16}$** $\mathcal{R}_{M_{16}}(\mathrm{LF})$ (Rauzy–Veech / Open, corrected per `#26`):
   - On the **synthetic template** `synth-tmpl-hex-1000x40` (`n = 81,000 -> m = 47,001`), $q_{\min}$ shrinks from `24` to `19` (`0.0004 m`).
   - **On real Wikidata blocks, this does NOT preserve a small alphabet:** $q_{\min}(R(M_{16}))$ rises **`8–21×`** from `83–94` to **`748–1,919`** (`0.005–0.010 m`), reflecting the $\sim 10^3$ distinct thick-strip exit intervals (much tighter than $\mathcal{R}_{I_{\text{'0'}}}$'s `1,996–4,112`, but not a fixed point). On pure hex, $\mathcal{R}_{M_{16}}$ is trivially the identity because no run has width $\ge 16$.

### 3. Measurements (8/8 Blocks)

| Block | Raw `LF`: $q_{\min}$ / $r/n$ | $\mathrm{LF}^2$: $q_{\min}$ / $r/n$ | $\mathcal{R}_{I_{\text{'0'}}}$: $|Y|$ / $q_{\min}$ (`!/|Y|`) / $r/|Y|$ | $\mathcal{R}_{M_{16}}$: $m$ / $q_{\min}$ (`!/m`) |
|---|---:|---:|---:|---:|
| `synth-hex-iid-65536` | `16` / `0.936` | `256` / `0.996` | `4,137` / **`1,839`** (`0.445`) / `0.996` | `57,458` / `3,629` (`0.063`, $L_{\min}=2$) |
| `synth-tmpl-hex-1000x40` | `24` / `0.529` | `298` / `0.585` | `4,921` / **`1,648`** (`0.335`) / `0.876` | `47,001` / `19` (`0.0004`) |
| `block-0.L` | `86` / `0.057` | `1,358` / `0.069` | `23,621` / **`1,996`** (`0.085`) / `0.233` | `88,290` / `805` (`0.009`) |
| `block-2.L` | `83` / `0.061` | `1,468` / `0.075` | `26,450` / **`2,214`** (`0.084`) / `0.245` | `109,542` / `1,133` (`0.010`) |
| `block-85.L` | `91` / `0.088` | `1,667` / `0.107` | `15,442` / **`3,082`** (`0.200`) / `0.520` | `155,124` / `748` (`0.005`) |
| `block-143.L` | `94` / `0.120` | `1,958` / `0.150` | `36,322` / **`3,965`** (`0.109`) / `0.344` | `193,449` / `1,919` (`0.010`) |
| `block-210.L` | `92` / `0.143` | `1,852` / `0.173` | `24,138` / **`4,112`** (`0.170`) / `0.447` | `202,025` / `1,304` (`0.006`) |
| `block-207.L` | `87` / `0.097` | `1,880` / `0.120` | `39,615` / **`3,252`** (`0.082`) / `0.249` | `134,609` / `1,281` (`0.009`) |

**Verdict on Lead 1:** **Does not transfer to novel random hex.** Both squaring (`LF^2`) and first-return sections (`R_Y`) leave the small-alphabet BWT class immediately on random content (`q_min` jumps `16 -> 256` and `16 -> 0.445|Y|`, matching `carbon-kitchen-sink` Cycle 7). Thick-strip contraction (`R(M_16)`) keeps `q_min` small only on the synthetic template (`24 -> 19`), while rising `8–21×` (`748–1,919`) on Wikidata.

---

## Lead 2: Dual-Number Evaluation (`(a, a')`) — Exact Answer via `E1`, `E2`, `E3` (corrected per `#26`)

1. **On the MTF stage (`E1`):** $\mathcal{M}_{\mathrm{MTF}}$ is an associative semidirect parallel-scan state (a memory trade for `NEEDS M-22`, not a derivative); it does not lower total decoder work or `Q`.
2. **On `(row, rider)` under `LF` (`E2` + `E3`):**
   - `E2` tests the free $\mathbb{N}^\Sigma$ prefix-histogram rider $M_2[a, b]$: `1.3–3.2% h_1` on Wikidata `M_16`, `0.00%` on novel random hex.
   - `E3` gives explicit 8/8 `dLF = 3` contiguous 3-rotation invariance witnesses leaving every commutative monoid and $M_2$ invariant.

---

## Note on Channel #24's Narrowed Question (corrected per `#26` & MONODROMY record 116)

Per MONODROMY record 116 §1, simple configuration counting (`s^{P h_1}` vs `h_1!`) **cannot** yield a `K = 2` pass lower bound because a single sweep with `s = 256` cursors carries more bits (`8 h_1`) than random hex digits hold (`4 h_1`). Any lower bound at `K = 2` must use the **structure** of what a sweep reads (as in MONODROMY record 116 Theorem O's necklace bound `Q >= n/(2(k+1))` in $\mathcal{S}_k$) rather than counting.
