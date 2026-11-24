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

## Reproducing All Checks

```bash
python3 research/neon-movement/common.py
python3 research/neon-movement/s1_text_axis.py
python3 research/neon-movement/s2_relabelling.py
python3 research/neon-movement/s3_pell_recurrence.py
python3 research/neon-movement/s4_independent_set.py
python3 research/neon-movement/s5_frontier_wave.py
python3 research/neon-movement/s6_ifs_discrepancy.py
```

Every script reads the six admitted blocks directly from `origin/data/admitted-bwt` (verifying SHA-256 against `manifest.json`) and asserts `decoded == T_ref` byte-for-byte against the independent textbook `LF` oracle on every block.
