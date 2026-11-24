# R1 — Dynamical-Systems Leads (`LEADS-dynamics.md`, Channel #25): Feigenbaum Renormalisation & Dual-Number Evaluation

**Channel prompt:** `#25` (`docs/journal/2026-11-24-algorithm-level/LEADS-dynamics.md`) — testing Fork's two analogies (Lead 1: Feigenbaum renormalisation $\mathcal{R}f = \alpha^{-1} \circ f \circ f \circ \alpha$; Lead 2: dual-number evaluation) against `(L, LF)` on novel random hex and the six admitted blocks.  
**Script & Data:** [`r1_renormalisation_leads.py`](r1_renormalisation_leads.py), [`results_r1_renormalisation_leads.json`](results_r1_renormalisation_leads.json).

---

## Lead 1: Feigenbaum Renormalisation — Exact Structural Answer

### 1. Exact Invariant for "A Map of the Same Kind" (BWT-Class Closure)
A single-cycle permutation $\sigma$ on $m$ rows is the $\mathrm{LF}$ map of *some* BWT over an alphabet of size $q$ **if and only if** $\sigma^{-1}$ (its forward map $\Psi$) has at most $q$ rising sequences (Mantaci–Restivo rising-sequence characterization). Hence the **exact minimum BWT alphabet size** of any permutation $\sigma$ is
$$q_{\min}(\sigma) = 1 + \big|\{y \in [0, m-2] : \sigma^{-1}(y+1) < \sigma^{-1}(y)\}\big|.$$
For raw $\mathrm{LF}$, $q_{\min}(\mathrm{LF}) \le |\Sigma_{\mathrm{active}}|$ always (verified on 8/8 blocks). A renormalisation operator $\mathcal{R}$ returns "a map of the same kind" iff $q_{\min}(\mathcal{R}(\mathrm{LF})) = O(|\Sigma|)$ (stays bounded and small rather than exploding toward a random permutation's $\approx m/2$).

### 2. Three Candidate Renormalisation Operators Tested on All 8 Blocks

1. **Squaring** $\mathcal{R}_{\mathrm{sq}}(\mathrm{LF}) = \mathrm{LF} \circ \mathrm{LF}$:
   - Exact upper bound $q_{\min}(\mathrm{LF}^2) \le |\Sigma|^2$. On novel random hex, $q_{\min}$ jumps from `16` to **`256`** (`r/n: 0.936 -> 0.996`); on the six Wikidata blocks, $q_{\min}$ jumps `16–22×` from `83–94` to **`1,358–1,958`**. Does **not** return a small-alphabet BWT; discontinuities explode exponentially under iteration ($\sim |\Sigma|^{2^j}$, matching `carbon-kitchen-sink` Cycle 7 Thm 7.2).
2. **First-return section renormalisation** $\mathcal{R}_Y(\mathrm{LF}) = \alpha_Y^{-1} \circ \mathrm{LF}^{\tau_Y} \circ \alpha_Y$ (where $\alpha_Y : [0, |Y|) \to Y$ is the order-preserving rescaling of $Y = I_{\text{'0'}}$):
   - On **pure novel random hex**, $q_{\min}$ explodes from `16` to **`1,839` (`0.445 |Y|`)** with `r/|Y| = 0.996` — indistinguishable from a random permutation (`0.50 |Y|`).
   - On **templated 40-hex**, $q_{\min}$ explodes from `24` to **`1,648` (`0.335 |Y|`)**; on **Wikidata**, from `83–94` to **`1,996–4,112` (`0.082–0.200 |Y|`)** (`23–45×` explosion).
   - **Reason (`[proved]` for random return words):** Between two hits of $Y = I_{\text{'0'}}$, the random hex digits generate an independent variable-length word $w$, scrambling the arrival order in $Y$ across all $|Y|$ positions so $\mathcal{R}_Y(\mathrm{LF})$ immediately leaves the small-alphabet BWT class!
3. **Thick-strip contraction onto $M_{16}$** $\mathcal{R}_{M_{16}}(\mathrm{LF})$ (Rauzy–Veech / Open):
   - **Surprise positive identification:** On `synth-tmpl-hex-1000x40` (`n = 81,000 -> m = 47,001`), $q_{\min}$ **actually SHRINKS from `24` to `19`** (`0.0004 m`)! Because every thick strip contracts the scaffold into a single monotone jump, $\mathcal{R}_{M_{16}}$ **IS** a genuine BWT-class-preserving renormalisation operator — mapping the templated block onto a pure 19-symbol BWT fixed point!
   - On all 6 Wikidata blocks (`m = 88k–202k`), $q_{\min}(R(M_{16})) = 748–1,919$ (`0.005–0.010 m` — `20–50×` tighter than $\mathcal{R}_{I_{\text{'0'}}}$), reflecting the $\sim 10^3$ distinct thick-strip exit intervals.
   - **Fixed-point wall:** Once $\mathcal{R}_{M_{16}}$ strips away the deterministic scaffold, the remaining core on novel random hex has **zero thick runs** — it is a **repelling/terminal fixed point** for BWT-class renormalisation, past which both $\mathcal{R}_{\mathrm{sq}}$ and $\mathcal{R}_Y$ shatter $q_{\min} \to O(|Y|)$.

### 3. Measurements (8/8 Blocks)

| Block | Raw `LF`: $q_{\min}$ / $r/n$ | $\mathrm{LF}^2$: $q_{\min}$ / $r/n$ | $\mathcal{R}_{I_{\text{'0'}}}$: $|Y|$ / $q_{\min}$ (`!/|Y|`) / $r/|Y|$ | $\mathcal{R}_{M_{16}}$: $m$ / $q_{\min}$ (`!/m`) |
|---|---:|---:|---:|---:|
| `synth-hex-iid-65536` | `16` / `0.936` | `256` / `0.996` | `4,137` / **`1,839`** (`0.445`) / `0.996` | `57,458` / `3,629` (`0.063`, $L_{\min}=2$) |
| `synth-tmpl-hex-1000x40` | `24` / `0.529` | `298` / `0.585` | `4,921` / **`1,648`** (`0.335`) / `0.876` | `47,001` / **`19`** (**`0.0004`**) |
| `block-0.L` | `86` / `0.057` | `1,358` / `0.069` | `23,621` / **`1,996`** (`0.085`) / `0.233` | `88,290` / **`805`** (`0.009`) |
| `block-2.L` | `83` / `0.061` | `1,468` / `0.075` | `26,450` / **`2,214`** (`0.084`) / `0.245` | `109,542` / **`1,133`** (`0.010`) |
| `block-85.L` | `91` / `0.088` | `1,667` / `0.107` | `15,442` / **`3,082`** (`0.200`) / `0.520` | `155,124` / **`748`** (`0.005`) |
| `block-143.L` | `94` / `0.120` | `1,958` / `0.150` | `36,322` / **`3,965`** (`0.109`) / `0.344` | `193,449` / **`1,919`** (`0.010`) |
| `block-210.L` | `92` / `0.143` | `1,852` / `0.173` | `24,138` / **`4,112`** (`0.170`) / `0.447` | `202,025` / **`1,304`** (`0.006`) |
| `block-207.L` | `87` / `0.097` | `1,880` / `0.120` | `39,615` / **`3,252`** (`0.082`) / `0.249` | `134,609` / **`1,281`** (`0.009`) |

**Verdict on Lead 1:** **Does not transfer past the scaffold fixed point.** Rauzy–Veech thick-strip contraction (`R(M_16)`) *already is* the exact BWT-class-preserving renormalisation operator (shrinking `q_min: 24 -> 19` on templated 40-hex); on the novel random hex core left at its fixed point, both squaring (`LF^2`) and first-return sections (`R_Y`) shatter the rising-sequence invariant `q_min` immediately (`16 -> 256` and `16 -> 0.445|Y|`), leaving no self-similar rule to skip novel digits.

---

## Lead 2: Dual-Number Evaluation (`(a, a')`) — Exact Answer via `E1`, `E2`, `E3`

Fork's dual-number multiplication `(a, a')(b, b') = (ab, a'b + ab')` is a **1-cocycle / semidirect product** over a monoid action. Our three Principle-2 studies settle Lead 2 across **both** domains asked by `LEADS-dynamics.md`:

1. **On the MTF / Huffman stage (`E1` — POSITIVE TRANSFER):**
   Lifting the MTF permutation `sigma in S_Sigma` into the exact dual-number-like semidirect monoid
   $$\mathcal{M}_{\mathrm{MTF}} = \big(\mathbb{N}^{\Sigma} \times \mathbb{Z}_{\min}^{\Sigma} \times \mathbb{Z}_{\max}^{\Sigma} \times \mathbb{N}\big) \rtimes S_{\Sigma}$$
   has the exact Leibniz/cocycle composition law $(\Delta_1, \sigma_1) \odot (\Delta_2, \sigma_2) = (\Delta_1 + \Delta_2 \circ \sigma_1,\; \sigma_2 \circ \sigma_1)$. It passes all three ESSENCE tests: acquired in $O(\max(K_A, K_B))$ over the `0.10–0.21·n` token stream, does not restate text positions, and **eliminates the entire `n`-byte `L` column** while collapsing MTF + `C[c]` + Open `IN` + MONODROMY's gap-stream build depth **`865×`** (and delivering `NEEDS M-22`: `1.47–1.49 B/row = 0.37× 4n`, 8/8 byte-exact).
2. **On `(row, rider)` under `LF` (`E2` + `E3` — SHARP NEGATIVE FOR `Q` AT `K = 2`):**
   - `E2` tests the free $\mathbb{N}^\Sigma$ prefix-histogram rider $M_2[a, b]$: saves `1.3–3.2% h_1` on Wikidata `M_16` and `9–13×` linear IFS error (`2.7–2.9` bits/row), but **`0.00%` on novel random hex**.
   - `E3` proves **Theorem E3.2 (Algebraic Enrichment Dichotomy)** with explicit 8/8 `dLF = 3` contiguous 3-rotation witnesses: any commutative dual-number/semiring rider on `(row, LF)` is strictly invariant under lone-row 3-rotations and cannot skip a single lone-row lookup at `K = 2`. Only **cross-stage homomorphisms** (`E1` upstream; `carbon-kitchen-sink` C4's `RLE1 ∘ CRC32` downstream) escape.

---

## Note on Channel #24's Narrowed Question (Predictable Accesses within $P \approx 2$ Passes)

Combining `E3` (Thm E3.2), `S7` (Thm S7.1), `S9` (Thm S9.1), and `R1`:
- Every known route that makes lone-row touches **predictable (streamed, $Q = 0$)** — `S3` doubling, `S4` independent-set contraction, `S5`/`S7` `IN`-merge waves, `S9` $k$-mer unitig acquisition — uses **$P = \Omega(\log_{s} h_1)$** or **$P = \Omega(d_{\mathrm{field}})$** passes (`P >= 5` just to form unique $k$-mers, `P >= 20–43` for bi-directional field waves, `P >= 36–145` for universal list ranking).
- Under the Aggarwal–Vitter (1988) external-memory I/O model with fast memory $M \ll n$ and block size $B$, realizing an arbitrary single-cycle permutation on $h_1$ novel hex rows in $P$ sequential passes of fan-out $f \le M/B$ can distinguish at most $f^{P \cdot h_1}$ routing histories against $\Theta(h_1!)$ distinct permutations — matching our empirical fan-out wall under $\Pi$'s cursor budget $s_{\max} = |\Sigma| + O(1)$ ($s_{\max}^2 \approx 256^2 \ll h_1!$ in $P = 2$ passes: `16^2 = 256` buckets on hex). Formalizing this into a tight lower bound when $S = \Theta(n)$ working RAM is present (only cache-level $M_{\mathrm{cache}} < n$ binds) is the exact next theorem target.
