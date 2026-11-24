# `research/neon-movement`: Six Senses of "Moving Together"

Following [`ESSENCE.md`](https://github.com/lmmx/mibz2/blob/b404cdf4f/docs/journal/2026-11-24-algorithm-level/ESSENCE.md), journal §§33–34, `council/COMPARISON.md`, and MONODROMY records 112–114, we treat **walk `\to` Rauzy–Veech** as one instance of **contraction along composition**, and construct **six distinct senses** in which width-1 lone rows (`h_1`) move together with something — three named starting senses (`S1–S3`) and three lateral mathematical senses (`S4–S6`), all pursued in parallel.

Every construction is:
- checked against prior art ([`PRIOR-ART.md`](PRIOR-ART.md));
- implemented in clean Python with **no hidden `LF` walks** and genuine endpoint-only thick-strip acquisition ([`common.py`](common.py));
- verified **100% byte-exact** against an independent textbook `LF` reference oracle on **8/8 blocks** (pure i.i.d. novel random hex `synth-hex-iid-65536`, templated novel 40-hex `synth-tmpl-hex-1000x40`, and all 6 admitted Wikidata blocks in `origin/data/admitted-bwt`);
- judged honestly under specification `Π` (MONODROMY 112) in `(Q, D, P, S)` with acquisition included.

**Headline verdict:** **No breakthrough on `Q < h_1` within `Π`'s frozen `K = 2` sweep budget is claimed.** Instead, the six senses map the exact `(Q, D, P, S)` escape geometry on novel random hex and Wikidata:
- **`Q` and `D` can BOTH be driven literally to `0`** on 100% novel random hex within `s = 256` cursors (`S3`, `S4`, `S5`), relocating cost entirely into `P > K = 2` sequential sweeps (`P_{\mathrm{bytes}}` dropping from `1{,}740` in raw `S3` to `59\text{--}134` in `S4` Cole–Vishkin and **`19.6\text{--}75.0`** in `S5` `\partial_{\mathrm{thick}}`-Wave!).
- **Dependent depth `D` collapses `150\text{--}10{,}000\times`** at small `P` (`P = 3\text{--}5`) via orthogonal row+text landmarks (`S1B`: `D = 1{,}093\text{--}6{,}013`) and `\partial_{\mathrm{thick}}` probe-wave (`S5`: `D_{\mathrm{wave}} = 48\text{--}86`, ascending-row probes!).
- At `P = 2`, **Period-`p` Strided Affine Rauzy–Veech (`S2A`)** is a genuine exact lone-row generalization of RV (`1.7\text{--}2.8%` of `h_1` on Wikidata), while **`S_\Sigma` relabelling (`S2C`)** and **IFS contraction (`S6`)** prove sharp structural obstructions explaining why neither alphabet symmetry nor `1/16`-metric contraction removes `Q` at `K = 2`.

---

## Side-by-Side Comparison of All Six Senses

| Sense | What "moves together" | Proof / Exactness | Novel random hex (`synth-hex-iid-65536`, `h_1=57{,}458`) | Six admitted Wikidata blocks (`block-0..207`, `n=900\text{k}`) | Referee `(Q, D, P, P_{\mathrm{bytes}}, S)` under `Π` (acquisition included) | Report |
|---|---|---|---|---|---|---|
| **S1 `text-axis`** | Consecutive text steps between sparse landmarks `Y_k` (raw `LF` `NEEDS M-21` + orthogonal on `m_{16}`) & forward-`\Psi` `F`-intervals `I(w)` | **Proved** (8/8 byte-exact) | `\|Y_3\|=28` (`0.0005 h_1`), `D: 65\text{k}\to 291`; 1C `k=8` shared `h_1 = 0.0%` | 1A (`NEEDS M-21`): `\|Y_3\|=14\text{--}17\text{k}`, best pair `":` (`39\text{--}42\text{k}`, mean `22`). **1B ortho**: `\|Y_3\|=572\text{--}2{,}599` (`0.016\text{--}0.040 h_1`), **`D=1{,}093\text{--}6{,}013`**; 1C `k=8` shared `20.4\text{--}25.1%` | 1B probe: `Q_{\mathrm{tot}} = 5.0\text{--}10.3\,h_1` (`Q_{\mathrm{run}}=0.02\text{--}0.04\,h_1`), **`D = 1.1\text{--}6.0\text{k}`**, `P=4\text{--}5`, `s=256`. Relocates `D \to Q_{\mathrm{acq}}`. | [`S1-TEXT-AXIS.md`](S1-TEXT-AXIS.md) |
| **S2 `relabelling`** | (2A) Period-`p` residue progressions (`a,b,a,b,\dots`) `\to` affine `\mathrm{LF}(s+r+tp)=\mathrm{LF}(s+r)+t\mu_r`; (2B) ramps; (2C) Baker `S_\Sigma` `W_k(y)=\phi(W_k(x))` | **Proved** Thms 2A & 2C (8/8 byte-exact) | 2A: `0.82%` `h_1` saved UB; 2C `k=8`: **`26.2\times` class collapse (`98.9%` shared)**, med rank disc `33` | 2A: **`4.3\text{--}6.5%` `h_1` in periodic (`1.7\text{--}2.8%` saved UB)**; 2B ramps `1.3\text{--}1.6%`; 2C `k=8`: **`19.6\text{--}40.6\times` collapse (`98.7\text{--}99.5%` shared)**, med rank disc `1{,}377\text{--}3{,}213` | 2A: `Q_{\mathrm{acq}}=0, P=2`, contracts `\le 2.8%` of `h_1`. 2C: word-shape collapses `99%`, **blocked from row transport** by proved `S_\Sigma`-Equivariance Obstruction. | [`S2-RELABELLING.md`](S2-RELABELLING.md) |
| **S3 `pell-recurrence`** | Join-key pairs `(x \to y, y \to z)` composed via pointer-free base-256 radix-sort-merge squaring `R_{j+1}=R_j \circ R_j` (plus 3B Zorich audit) | **Proved** (8/8 byte-exact; raw `LF` verified exact on 65k hex) | **Total contraction:** raw `LF` **`Q=0, D=0`**, `P=117`, `P_{\mathrm{bytes}}=1{,}525`, `S=1.57\text{ MB}` | Retained `m_{16}` (`17\text{--}18` rnds): `Q_{\mathrm{doub}}=0, D_{\mathrm{doub}}=0, Q_{\text{run,lone}}=0`, `P=124\text{--}131`, `P_{\mathrm{bytes}}=156.8\text{--}381.2` (`4.6\text{--}11.1\times` vs raw), `S=4.4\text{--}8.5\text{ MB}`. 3B Zorich: `0` lone rows. | Drives `Q, D \to 0` universally via `P = \Theta(\log m) \gg K=2` and `O(m \log m)` streamed volume. | [`S3-PELL-RECURRENCE.md`](S3-PELL-RECURRENCE.md) |
| **S4 `independent-set`** | Miller–Reif / **Cole–Vishkin** bit-difference independent set `I_k \subset V_k` spliced conflict-free (`p_u \to u \to s_u`), iterated geometrically `V_0 \supset V_1 \supset \dots \supset \{\mathrm{mark}\}` | **Proved** Thm 4.1 (8/8 byte-exact on BOTH Phase-1 DAG & Phase-2 reverse-unspool) | **Total & `O(m)`:** CV `38` rnds, `r_1=0.643`, **`2.95\,m` touches**, `P_{\mathrm{bytes}}=283.1` (1ph) / `596.8` (2ph) | **Universal Cole–Vishkin law:** `36\text{--}46` rnds (vs LM `79\text{--}270`), `r_1=0.643\text{--}0.645`, **`2.94\text{--}2.95\,m` touches** (and `2.97\,n` on raw `block-0.L`!), 2-phase **`P_{\mathrm{bytes}}=59.1\text{--}134.3`** (`2.6\text{--}5.7\times` below S3!) | `Q_{\mathrm{contract}}=0, D_{\mathrm{contract}}=0, Q_{\text{run,lone}}=0` (and literal `Q=0, D=0` on raw `LF`), relocated to `P \gg K=2` sweeps over geometrically shrinking arrays (`O(m)` total volume!). | [`S4-INDEPENDENT-SET.md`](S4-INDEPENDENT-SET.md) |
| **S5 `frontier-wave`** | All `K_{\mathrm{exc}}` thin excursions leaving Rauzy–Veech thick strips (`\partial_{\mathrm{thick}}`) move synchronously as ONE sorted frontier `F_t`, merged against `r`-sized `IN` | **Proved** Thm 5.1 (8/8 byte-exact; sortedness asserted every round) | Pure hex (`L=2`): `K_{\mathrm{exc}}=0.123 h_1, d_{\max}=78, P_{\mathrm{bytes}}=464.4`. **Templated 40-hex (`L=16`):** `d_{\max}=41`, **`Q_{\mathrm{tot}}=0.058\,h_1`**! | **`d_{\max}=48\text{--}86`** (`p99=41\text{--}43`), `K_{\mathrm{exc}}=0.365\text{--}0.634\,h_1`. At `P=32`: **`87.8\text{--}93.3%` done** (`P_{\mathrm{bytes}}=12.7\text{--}31.5`). Full stream: **`P_{\mathrm{bytes}}=19.6\text{--}75.0`**, `S=2.55\text{--}4.74\text{ MB}`! | Stream mode: `Q_{\mathrm{thin}}=0, D_{\mathrm{thin}}=0`, `Q_{\mathrm{tot}}=1.09\text{--}4.24\,h_1` (mostly thick strips), lowest `P_{\mathrm{bytes}}` (`19.6\text{--}75.0`), `P=51\text{--}89 > K=2`. Probe mode: `P=3`, **`D_{\mathrm{wave}}=48\text{--}86`** (ascending-row probes!). | [`S5-FRONTIER-WAVE.md`](S5-FRONTIER-WAVE.md) |
| **S6 `ifs-discrepancy`** | Rows in `(b, c)` move under affine contraction `f_{b,c}(x)` (piecewise-linear IFS) up to exact Brownian-bridge residual `\varepsilon_B(x)` | **Proved** Thm 6.1 (8/8 byte-exact at `B=256`) | `\Delta_0=32\text{k} \to 0.031` by `k=5`, bnd stays `16.3`; `B=256`: med `\|\varepsilon\|=1`, **`2.11` bits/target** (`26.6%` exact `0`) | `\Delta_0=450\text{k} \to \mathbf{0.0000}` by `k=5`, bnd stays `1{,}355\text{--}1{,}788`; `B=256`: med `8\text{--}10` (`4.7\text{--}4.9` bits); `B=1024`: med **`3`** (`3.2\text{--}3.4` bits, `12\text{--}14%` exact `0`) | Compresses target-address space `S` by `4\text{--}6\times` (`20 \to 3.4` bits) at `Q_{\mathrm{acq}}=0, P=2`, but `Q=n, D=n` unchanged by uncontracted final-step discrepancy over unsmoothed `L`. | [`S6-IFS-DISCREPANCY.md`](S6-IFS-DISCREPANCY.md) |

---

## Ranking by Which Sense Can Be Taken Furthest Next (Channel #21)

No single sense leads; ranked strictly by how far each can be pushed next:

1. **S5 (`\partial_{\mathrm{thick}}` `IN`-Merge Frontier Wave), optionally composed with S4 (Cole–Vishkin) on the `K_{\mathrm{exc}}` excursion graph:**
   - On Wikidata, S5 automatically discovers every 40-char SHA1 / GUID excursion from Rauzy–Veech's thick boundary with zero template oracle (`p99 = 41\text{--}43`, `d_{\max} = 48\text{--}86`).
   - In **probe mode**, it collapses thin-wave dependent depth `D` from `900{,}000` to **`48\text{--}86`** with strictly ascending-row accesses per round.
   - In **hybrid stream mode** (`P_{\mathrm{cap}} = 32` scans of `r`-sized `IN`), `88\text{--}93%` of all thin rows complete with `Q_{\mathrm{thin}} = 0` at `P_{\mathrm{bytes}} = 12.7\text{--}31.5` (`S \approx 2.6\text{--}4.7\text{ MB}`), and gluing the remaining `K_{\mathrm{exc}} = 0.365\text{--}0.634\,h_1` excursions with Cole–Vishkin (`S4`) needs only `2.94\,K_{\mathrm{exc}}` touches.
   - **Key referee question (matching MONODROMY 114 §5):** If real machine cost turns on *dependent* latency `D` (S5 probe / S1B) or *byte-volume* sequential scans of `IN` vs random DRAM misses (S5 stream), S5 is immediately testable.
2. **S1B (Orthogonal Row+Text Landmark Induction):**
   - Cleanest low-pass (`P = 4\text{--}5`) collapse of total dependent depth `D` (`1{,}093\text{--}6{,}013` on all 6 blocks), directly answering MONODROMY `NEEDS M-21` and showing `10\text{--}25\times` smaller sections than raw-`LF` landmarks.
3. **S4 (Cole–Vishkin Recursive Independent-Set Contraction):**
   - Best worst-case / unstructured-hex `Q = 0, D = 0` engine: the 2-adic ruler property of `cv(x)` guarantees a universal `2.94\text{--}2.97\,m` element-touch law on every input (even raw `LF` with million-row monotone runs), dominating S3 by `2.6\text{--}5.7\times` in streamed volume.
4. **S3 (Pell Sort-Merge Doubling):**
   - Clean proof-of-concept that literal `Q = 0, D = 0` is achievable on 100% novel random hex within `s = 256` cursors, dominated in total streamed volume by S4 (`O(m \log m)` vs `O(m)`).
5. **S2 (`relabelling`) & S6 (`ifs-discrepancy`):**
   - S2A provides an exact `1.7\text{--}2.8%` periodic-progression extension of Rauzy–Veech at `P = 2`; S2C and S6 are clean structural obstruction theorems bounding what alphabet symmetry and metric contraction can deliver at `K = 2` on novel random hex.

---

## Cycle 2 — Enrichment Along Composition, Cross-Stage Lifting, Further Contraction Senses & Dynamical-Systems Leads (`E1–E3`, `S7–S9`, `R1`)

Following [`ESSENCE.md`](https://github.com/lmmx/mibz2/blob/efac4a340723e2b26ba66a91105ab6b4c14e1e22/docs/journal/2026-11-24-algorithm-level/ESSENCE.md) (Principle 2: Enrichment along composition), [`CROSS-STAGE.md`](https://github.com/lmmx/mibz2/blob/efac4a340723e2b26ba66a91105ab6b4c14e1e22/docs/journal/2026-11-24-algorithm-level/CROSS-STAGE.md), MONODROMY records 114 §6 & 115, and channel entries `#22–#25` (`LEADS-dynamics.md`, `neon-movement-critique.md`):

**Headline verdict (Cycle 2):** **Still no breakthrough on `Q < h_1` within `Π`'s frozen `K = 2` sweep budget.** Cycle 2 establishes:
- **Positive Principle 2 / Cross-Stage & Lead 2 result (`E1`):** Lifting MTF into the associative semidirect monoid $\mathcal{M}_{\mathrm{MTF}} = (\mathbb{N}^\Sigma \times \mathbb{Z}_{\min}^\Sigma \times \mathbb{Z}_{\max}^\Sigma \times \mathbb{N}) \rtimes S_\Sigma$ (exact Leibniz/cocycle dual-number form) eliminates the entire `n`-byte `L` column, collapses build depth **`865×`** (`D_acq = 1,038–1,040`), reveals narrow active MTF support (`K_50` median `23–26` on `0.10–0.21·n` tokens), and delivers **`NEEDS M-22`**: full-`L` gap-encoded $\Psi$ takes **`1.47–1.49 B/row` (`0.367–0.371× 4n`, `0.77–1.27%` escapes; `1.22–1.24 B/row` without directory)** and retained $M_{16}$ takes **`1.62–1.76 B/m` (`0.406–0.440× 4m`)**, verified 8/8 byte-exact with zero `L`.
- **Positive Principle 1 `D` find (`S8` — `NEEDS M-21b` STANDS `6/6`):** MONODROMY 114 §6's count-targeted landmark rule crushes the frequency-rank `1.3–1.9%` figure by `10–20×`, achieving **`D = 608–1,974` (`0.068–0.219%` of `n`)** on all 6/6 admitted blocks (`Q = 1.03–1.17 n`). Composing with Open `M_16` + bi-directional `Y_k` (`ORTH-BIDIR`) yields **`D = 627–1,776` (`0.070–0.197%` of `n`, `507–1,435×` collapse)** at `2–3×` lower `Q` (`0.34–0.60 n`).
- **Four sharp negative/obstruction theorems closing open routes to `Q < h_1` at `K = 2`:**
  1. **`E3` (Thm E3.1 & E3.2):** Dynamic monotone rank-envelope forces **`33.31%`** of novel-hex lone rows (`29.9–32.2%` Wikidata) information-theoretically (yet dynamic lookup pays `Q >= 1`); explicit 8/8 contiguous 3-rotation witnesses (`dLF = 3`, `M_2` & all commutative aggregates invariant) prove no commutative monoid rider cuts `Q`.
  2. **`S7` (Thm S7.1 — Pairing-Oracle Obstruction):** Settles MONODROMY 113–115's #1 open candidate — even a zero-cost oracle for Relation C's pairing `pi` leaves **`34/40` hex digits (`136` bits/rec)** invariant (`pi' == pi` witnessed), leaving `Q >= (d - 2)m` unchanged. Meanwhile, S7's **Bi-Directional `LF x Psi` Sorted-Merge Wave** halves S5's wave depth (`D_wave = 24–43`, `98.5–99.8%` done by round `21`) and cuts `P_bytes` to `14.8–54.8` (+ `3.2–7.2` word-sort), with total `D = 14.9–44.0k` reported honestly.
  3. **`S9` (Thm S9.1 — Unitig Pass-Wall):** Closes Carbon C-4's open test from `carbon-cycle1-critique.md` — at `K = 2` (`k = 2`), `V_1 = 0.00%` on novel hex (`<= 0.39%` Wikidata); even at `k = 8` (`P = 7 > K = 2`) on `L_min = 16`, non-unique thin runs keep `Q_run = 1.07–2.24 h_1 > h_1` on 6/6 Wikidata blocks.
  4. **`R1` (Channel #25 Lead 1 — Renormalisation):** Using the exact BWT rising-sequence invariant $q_{\min}(\sigma) = 1 + \mathrm{desc}(\sigma^{-1})$, Rauzy–Veech `R(M_16)` *already is* the unique BWT-class-preserving renormalisation operator (`q_min: 24 -> 19` on templated 40-hex; `748–1,919` on Wikidata); past that scaffold fixed point, both `LF^2` (`q_min: 16 -> 256`) and `R(I_'0')` (`q_min: 16 -> 1,839 = 0.445|Y|`) immediately shatter out of the small-alphabet BWT class.

### Side-by-Side Cycle 2 Summary (All 8/8 Byte-Exact)

| Item | Principle | Core Object / Theorem | Novel Random Hex (`65k` / `tmpl 40-hex`) | Six Admitted Wikidata Blocks (`900k`) | Referee `(Q, D, P, S)` under `Π` | Report |
|---|---|---|---|---|---|---|
| **E1** | P2 (Cross-Stage) | $\mathcal{M}_{\mathrm{MTF}} = (\mathbb{N}^\Sigma \times \mathbb{Z}_{\min}^\Sigma \times \mathbb{Z}_{\max}^\Sigma \times \mathbb{N}) \rtimes S_\Sigma$ + `NEEDS M-22` | `K_50=16/17`; `M-22` full `1.44–1.45 B/r` (`0.0–0.2%` esc) | `tok/n=0.10–0.21`, `K_50=23–26`, `D_acq=1,038–1,040` (`865×`), `M-22` full **`1.47–1.49 B/r`** (`0.37× 4n`), $M_{16}$ `1.62–1.76 B/m` | `Q_acq=0, Q_run=n` (unchanged); **`0 B` `L`**, `P_acq=2` over `0.1–0.2n` tokens, `S=1.33 MB` (`2.7×` below `4n`) | [`E1`](E1-CROSS-STAGE-MTF-MONOID.md) |
| **E2** | P2 (In-Stage $\mathbb{N}^\Sigma$) | Free `P=1, Q=0` 2-gram matrix $M_2[a,b]$ mono-run bypass + 2-gram IFS | `M_16` save `0.00%`; IFS med `16->4` / `349->8` (`-2.1/-4.1 b`) | Raw `Q/n=0.903–0.934`, genuine $M_{16}$ save `1.32–3.24% h_1`; IFS med `9–13×` lower (`-2.7/-2.9 b`) | `Q_acq=0, P=2, S_{M_2}<=37 KB`; `0%` on novel hex | [`E2`](E2-PREFIX-HISTOGRAM-M2.md) |
| **E3** | P2 (Theory) | Dynamic 1-Lipschitz rank-envelope (`1/3` pin) & **Thm E3.2** 3-rot dichotomy | `33.31%` / `31.93%` `h_1` forced; `dLF=3` witness `x=0` | `29.86–32.16%` `h_1` forced (`89–96%` all `n`); 6/6 `dLF=3` single-cycle witnesses (`dT=10–79% n`) | Envelope check still pays `Q>=1`; proves no commutative rider cuts `Q` at `K=2` | [`E3`](E3-MONOTONE-ENVELOPE-DICHOTOMY.md) |
| **S7** | P1 (Contraction) | **Thm S7.1** Pairing-Oracle Obstruction + Bi-Dir `LF x Psi` Sorted-Merge Wave | `34/40` hex digits `pi`-invariant (`pi'==pi`); `D_wave`: `78->39`, `41->20` | **`D_wave=24–43`** (`2.0×` vs S5), **`98.5–99.8%` done by `r=21`**, `P_bytes=14.8–54.8` (+ `3.2–7.2` sort) | Total `D=14.9–44.0k` (incl. `K_exc` glue); `Q_scatter=m` charged or `+4` radix passes; `P=26–45 > K=2` | [`S7`](S7-PAIRING-OBSTRUCTION-BIDIR-WAVE.md) |
| **S8** | P1 (Contraction) | Exact **`NEEDS M-21b`** count-targeted landmarks + `ORTH-BIDIR` | Raw `D=288` (`0.44%`), `185` (`0.23%`); `ORTH` `249`, `78` | **STANDS `6/6` `< 0.5%`**: raw `D=608–1,974` (`0.068–0.219% n`); `ORTH-BIDIR` **`D=627–1,776`** (`507–1,435×`) | Raw `Q=1.03–1.17 n`; `ORTH` `Q=0.34–0.60 n` (`5.0–10.2 h_1`), `P=4–6`; major `D` find, not `Q` | [`S8`](S8-COUNT-TARGETED-LANDMARKS.md) |
| **S9** | P1 (Contraction) | Variable-order / $k$-mer de Bruijn unitig endpoint merge on $L_{\min}=16$ | `k=2`: `V_1=0.00%`; `k=8`: `u_8=3` (`0.000 h_1`) / `2,001` (`0.206`) | `k=2`: `V_1<=0.39%`; `k=8` (`P=7`): `u_8=2.3–5.4k`, yet `Q_run=1.07–2.24 h_1 > h_1` on `6/6` | Proves `k >= log_16 n` pass-wall (`P>=5 > K=2`) + `m_branch > h_1` at `L_min=16`; closes Carbon C-4 | [`S9`](S9-UNITIG-ENDPOINT-MERGE.md) |
| **R1** | Ch. `#25` Leads | Feigenbaum renormalisation $q_{\min}(\sigma) = 1+\mathrm{desc}(\sigma^{-1})$ + Lead 2 | `LF^2`: `16->256`; `R(I_'0')`: `0.445|Y|`; `R(M_16)` tmpl: **`24->19`** | `LF^2`: `1.4–2.0k`; `R(I_'0')`: `2.0–4.1k` (`0.08–0.20|Y|`); `R(M_16)`: `748–1,919` (`0.5–1.0% m`) | `R(M_16)` already *is* the BWT-preserving renorm fixed point; Lead 1 fails on novel hex; Lead 2 succeeds in `E1` | [`R1`](R1-RENORMALISATION-LEADS.md) |

### Cycle 1 Corrections Acknowledged (from Channel `#24` / `neon-movement-critique.md`)

1. **S5 total `D`:** `48–86` is thin-wave depth `D_wave` only; total `D` including the `K_exc` excursion-glue chain is **`14.9k–44.1k`** (same distinction enforced explicitly in S7 above).
2. **S5 thin-byte scatter:** `exc_words[eid].append(b)` is `m` addressed stores unless replaced by pointer-free radix sort of `(eid, t, b)` triples (`+4` passes), and the `is_thin` bitmap adds `0.125 n` bytes/round; both are charged explicitly in S7.
3. **S4 `2.94–2.97 m` density:** Reflects random-label local-minima density (`1/3 -> 3.00 m`) on these permutations, not a worst-case Cole–Vishkin guarantee (`1/(4 log m)` in 1 round).
4. **S1B terminology & accounting:** S1B is Open's thick-strip contraction followed by sparse ruling-set list ranking on `R` (`m_16 = 2.5–5.8 h_1`, not `m ≈ h_1`); raw-LF and `m_16` acquisition costs should always be compared on the same basis.

---

## Reproducing All Checks

```bash
# Cycle 1 (S1-S6)
python3 research/neon-movement/common.py
python3 research/neon-movement/s1_text_axis.py
python3 research/neon-movement/s2_relabelling.py
python3 research/neon-movement/s3_pell_recurrence.py
python3 research/neon-movement/s4_independent_set.py
python3 research/neon-movement/s5_frontier_wave.py
python3 research/neon-movement/s6_ifs_discrepancy.py

# Cycle 2 (E1-E3, S7-S9, R1)
python3 research/neon-movement/e1_cross_stage_mtf_monoid.py
python3 research/neon-movement/e2_prefix_histogram_m2.py
python3 research/neon-movement/e3_monotone_envelope_dichotomy.py
python3 research/neon-movement/s7_pairing_obstruction_bidir_wave.py
python3 research/neon-movement/s8_count_targeted_landmarks.py
python3 research/neon-movement/s9_unitig_endpoint_merge.py
python3 research/neon-movement/r1_renormalisation_leads.py
```

Every script reads the six admitted blocks directly from `origin/data/admitted-bwt` (verifying SHA-256 against `manifest.json`) and asserts `decoded == T_ref` byte-for-byte against the independent textbook `LF` oracle on every block.

