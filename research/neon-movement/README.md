# `research/neon-movement`: Six Senses of "Moving Together"

Following [`ESSENCE.md`](https://github.com/lmmx/mibz2/blob/b404cdf4f/docs/journal/2026-11-24-algorithm-level/ESSENCE.md), journal §§33–34, `council/COMPARISON.md`, and MONODROMY records 112–114, we treat **walk `\to` Rauzy–Veech** as one instance of **contraction along composition**, and construct **six distinct senses** in which width-1 lone rows (`h_1`) move together with something — three named starting senses (`S1–S3`) and three lateral mathematical senses (`S4–S6`), all pursued in parallel.

Every construction is:
- checked against prior art ([`PRIOR-ART.md`](PRIOR-ART.md));
- implemented in clean Python with **no hidden `LF` walks** and genuine endpoint-only thick-strip acquisition ([`common.py`](common.py));
- verified **100% byte-exact** against an independent textbook `LF` reference oracle on **8/8 blocks** (pure i.i.d. novel random hex `synth-hex-iid-65536`, templated novel 40-hex `synth-tmpl-hex-1000x40`, and all 6 admitted Wikidata blocks in `origin/data/admitted-bwt`);
- judged honestly under specification `Π` (MONODROMY 112) in `(Q, D, P, S)` with acquisition included.

**Headline verdict (Cycle 1, corrected per `#24`/`#26`):** **No breakthrough on `Q < h_1` within `Π`'s frozen `K = 2` sweep budget is claimed.**
- **External-memory list ranking (`S3`, `S4`) and stream wave (`S5`):** Literal `Q = 0, D = 0` holds only on raw `LF` at `P = 117\text{--}600 \gg K = 2` sweeps (classic CGGTVV / Wyllie list ranking); on `m_{16}`, thick-strip acquisition (`Q_{\mathrm{acq}} \approx 2.4\,m_{16}` on block-0) remains, and `S5`'s direct word append costs `Q_{\mathrm{scatter}} = m` unless replaced by radix-sort passes plus bitmap traffic.
- **Dependent depth `D`:** The strongest Cycle-1 find is `S1B` (Open thick-strip contraction + sparse ruling-set list ranking on `R`: total **`D = 1{,}093\text{--}6{,}013`** at `P = 4\text{--}5`, `Q = 5.0\text{--}10.3\,h_1`). For `S5`, `48\text{--}86` is wave depth `D_{\mathrm{wave}}` alone; total `D` including the `K_{\mathrm{exc}}` glue chain is **`14.9\text{k--}44.1\text{k}`**.
- **At `P = 2`:** `S2A` contracts `\le 2.8%` of `h_1` on Wikidata; `S2C` and `S6` are negative / state-compression results.

---

## Side-by-Side Comparison of Cycle-1 Senses (`S1–S6`, corrected per `#24`/`#26`)

| Sense | What "moves together" | Proof / Exactness | Novel random hex (`synth-hex-iid-65536`, `h_1=57{,}458`) | Six admitted Wikidata blocks (`block-0..207`, `n=900\text{k}`) | Referee `(Q, D, P, P_{\mathrm{bytes}}, S)` under `Π` (acquisition included) | Report |
|---|---|---|---|---|---|---|
| **S1 `text-axis`** | Sparse ruling-set list ranking on `LF` (`1A` = `M-21`) & on Open's retained map `R` (`1B`) | Proved (8/8 byte-exact) | `\|Y_3\|=28`, `D: 65\text{k}\to 291`; 1C `k=8` shared `h_1 = 0.0%` | 1A (`M-21`): `\|Y_3\|=14\text{--}17\text{k}`. **1B (Open + ruling set)**: `\|Y_3\|=572\text{--}2{,}599`, **`D=1{,}093\text{--}6{,}013`** | 1B probe: `Q_{\mathrm{tot}} = 5.0\text{--}10.3\,h_1` (`0.34\text{--}0.60\,n`), **`D = 1.1\text{--}6.0\text{k}`**, `P=4\text{--}5`. Real `D` find; `Q` unchanged. | [`S1-TEXT-AXIS.md`](S1-TEXT-AXIS.md) |
| **S2 `relabelling`** | (2A) Period-`p` residue progressions; (2B) ramps; (2C) Baker `S_\Sigma` shapes | Proved (8/8 byte-exact) | 2A: `0.82%` `h_1` UB; 2C `k=8`: `26.2\times` shape collapse (pigeonhole), med rank disc `33` | 2A: `1.7\text{--}2.8%` `h_1` UB; 2C `k=8`: `19.6\text{--}40.6\times` shape collapse, med rank disc `1{,}377\text{--}3{,}213` | 2A: `P=2`, negligible (`\le 2.8%`). 2C: transport fails (`LF` not `S_\Sigma`-equivariant). | [`S2-RELABELLING.md`](S2-RELABELLING.md) |
| **S3 `pell-recurrence`** | External-memory Wyllie doubling `R_{j+1}=R_j \circ R_j` via radix-sort-merge | Proved (8/8 byte-exact) | Raw `LF`: `Q=0, D=0`, `P=117`, `P_{\mathrm{bytes}}=1{,}525` | On `m_{16}`: `Q_{\mathrm{acq}}\approx 2.4\,m_{16}` + DAG expansion, `P=124\text{--}131`, `P_{\mathrm{bytes}}=156.8\text{--}381.2` | Classic EM Wyllie doubling; relocates `Q` into `P \gg K=2`. | [`S3-PELL-RECURRENCE.md`](S3-PELL-RECURRENCE.md) |
| **S4 `independent-set`** | External-memory Miller–Reif / CV list ranking (CGGTVV) | Proved (8/8 byte-exact) | `38` rnds, `2.95\,m` empirical touches (random-label `3.00\,m` density), `P_{\mathrm{bytes}}=596.8` | `36\text{--}46` rnds, `2.94\text{--}2.95\,m` empirical touches, `P_{\mathrm{bytes}}=59.1\text{--}134.3`, `P\approx 600` | Classic EM list ranking (`Q_{\mathrm{acq}}\ge m` on `m_{16}`; literal `Q=0` only on raw `LF` at `P\approx 600 \gg K=2`). | [`S4-INDEPENDENT-SET.md`](S4-INDEPENDENT-SET.md) |
| **S5 `frontier-wave`** | `M-21`-style stream wave from `\partial_{\mathrm{thick}}` merged against `IN` | Proved (8/8 byte-exact) | Pure hex (`L=2`): `K_{\mathrm{exc}}=0.123 h_1, D_{\mathrm{wave}}=78`; tmpl (`L=16`): `D_{\mathrm{wave}}=41` | `D_{\mathrm{wave}}=48\text{--}86`, **total `D=14.9\text{k--}44.1\text{k}`**, `K_{\mathrm{exc}}=0.365\text{--}0.634\,h_1`, wave `P_{\mathrm{bytes}}=19.6\text{--}75.0` (+ scatter/bitmap) | `Q_{\mathrm{scatter}}=m` unless `+4` radix passes; bitmap adds `0.125 n`/rnd; `P=51\text{--}89 > K=2`. | [`S5-FRONTIER-WAVE.md`](S5-FRONTIER-WAVE.md) |
| **S6 `ifs-discrepancy`** | Sampled rank / PGM piecewise-linear IFS residuals `\varepsilon_B(x)` | Proved (8/8 byte-exact) | `B=256`: `2.11` bits/target | `B=256`: `4.7\text{--}4.9` bits; `B=1024`: `3.2\text{--}3.4` bits | State compression only (`S` shrinks `4\text{--}6\times`); `Q=n, D=n` unchanged. | [`S6-IFS-DISCREPANCY.md`](S6-IFS-DISCREPANCY.md) |

---

## Cycle 1 Ranking (corrected per `#24`/`#26`)

1. **S1B (Open thick-strip contraction + sparse ruling-set list ranking on `R`):** Strongest Cycle-1 find — total `D = 1{,}093\text{--}6{,}013` at `P = 4\text{--}5` (further improved to `627\text{--}1{,}776` in Cycle 2 `S8`).
2. **S5 (`\partial_{\mathrm{thick}}` `IN`-Merge Wave, extended to bi-directional `S7` in Cycle 2):** Low wave depth (`D_{\mathrm{wave}} = 48\text{--}86`, halved to `24\text{--}43` in `S7`), but total `D = 14.9\text{k--}44.1\text{k}` and pointer-free operation pays `P \gg K = 2` plus scatter/word-sort and bitmap traffic.
3. **S3 / S4 (Classic external-memory list ranking):** Relocate `Q` into `P \approx 117\text{--}600 \gg K = 2` sweeps (`2.94\,m` empirical density in S4 is random-label minima density, not a worst-case CV bound).
4. **S2 & S6:** Negligible `P = 2` savings (`\le 2.8% h_1`) and target-state compression only.

---

## Cycle 2 — Enrichment Along Composition, Cross-Stage Lifting, Further Contraction Senses & Dynamical-Systems Leads (`E1–E3`, `S7–S9`, `R1`)

Following [`ESSENCE.md`](https://github.com/lmmx/mibz2/blob/efac4a340723e2b26ba66a91105ab6b4c14e1e22/docs/journal/2026-11-24-algorithm-level/ESSENCE.md) (Principle 2: Enrichment along composition), [`CROSS-STAGE.md`](https://github.com/lmmx/mibz2/blob/efac4a340723e2b26ba66a91105ab6b4c14e1e22/docs/journal/2026-11-24-algorithm-level/CROSS-STAGE.md), MONODROMY records 114 §6 & 115, and channel entries `#22–#25` (`LEADS-dynamics.md`, `neon-movement-critique.md`):

**Headline verdict (Cycle 2, corrected per `#26`):** **No `Q` breakthrough.**
- **Strongest find (`S8` — `NEEDS M-21b` STANDS `6/6`):** MONODROMY 114 §6's count-targeted landmark rule achieves **`D = 608–1,974` (`0.068–0.219%` of `n`)** on all 6/6 admitted blocks (`Q = 1.03–1.17 n`; reproduced independently). Composing with Open `M_16` + bi-directional `Y_k` (`ORTH-BIDIR`) yields **`D = 627–1,776`** at `Q = 0.34–0.60 n` (unrolling & label precompute caveats noted in `S8`).
- **CSA memory trade & `NEEDS M-22` (`E1`):** Parallel semidirect chunk scan (`(N^Σ × min × max × N) ⋊ S_Σ`, citing Sadakane / Grossi–Vitter CSA, 30 Oct `(O, P)` reports, and `src/runs.rs`) measures `NEEDS M-22`: full-`L` gap-encoded $\Psi$ takes **`1.47–1.49 B/row` (`0.367–0.371× 4n`, `1.22–1.24 B/row` without directory)** and retained $M_{16}$ takes **`1.62–1.76 B/m`**, 8/8 byte-exact (`D_prefix = 1,038–1,040` is chunk-prefix depth only, excluding Huffman and emit).
- **Invariance lemmas & structural negative results (`E2`, `E3`, `S7`, `S9`, `R1`):**
  1. **`E3`:** Dynamic monotone rank-envelope forces `33.31%` of novel-hex lone rows (`29.9–32.2%` Wikidata) information-theoretically (dynamic lookup still pays `Q >= 1`); 8/8 contiguous 3-rotation invariance witnesses (`dLF = 3`) leave `M_2` and all commutative aggregates invariant.
  2. **`S7`:** **Theorem S7.1** proves **invariance only** (`34/40` hex digits unconstrained by `pi` alone; does NOT prove `Q >= (d-2)m` at `K = 2` — **Relation C remains open**). Bi-directional `LF x Psi` wave halves `D_wave` to `24–43` (`D_tot = 14.9–44.0k`, `P_bytes = 14.8–54.8` + `3.2–7.2` word-sort).
  3. **`S9`:** Closes Carbon C-4's open test on `L_min = 16` — `V_1 = 0.00%` at `k = 2` (`K = 2`), while `k = 8` (`>= 3` passes via Manber–Myers prefix doubling) still leaves `Q_run = 1.07–2.24 h_1 > h_1` on 6/6 Wikidata blocks due to repeated width `2..15` thin runs.
  4. **`R1`:** Using Gessel–Reutenauer / Crochemore–Désarménien–Perrin's $q_{\min}(\sigma) = 1 + \mathrm{desc}(\sigma^{-1})$, squaring (`LF^2`) and first-return sections (`R(I_'0')`) immediately leave the small-alphabet BWT class on random content (`16 -> 256` and `16 -> 0.445|Y|`). `R(M_16)` shrinks `q_min` (`24 -> 19`) only on the synthetic template, rising `8–21×` (`748–1,919`) on Wikidata.

### Side-by-Side Cycle 2 Summary (All 8/8 Byte-Exact, corrected per `#26`)

| Item | Principle | Core Object / Theorem | Novel Random Hex (`65k` / `tmpl 40-hex`) | Six Admitted Wikidata Blocks (`900k`) | Referee `(Q, D, P, S)` under `Π` | Report |
|---|---|---|---|---|---|---|
| **E1** | P2 (CSA memory trade) | Semidirect MTF scan + `NEEDS M-22` (Sadakane / Grossi–Vitter CSA) | `K_50=16/17`; `M-22` full `1.44–1.45 B/r` (`0.0–0.2%` esc) | `tok/n=0.10–0.21`, `K_50=23–26`, `M-22` full `1.47–1.49 B/r` (`0.37× 4n`), $M_{16}$ `1.62–1.76 B/m` | Memory trade (`S=1.33 MB` vs `3.60 MB` for inline gap sum); `Q_run=n`, `D=n` unchanged | [`E1`](E1-CROSS-STAGE-MTF-MONOID.md) |
| **E2** | P2 (In-Stage $\mathbb{N}^\Sigma$) | Free `P=1, Q=0` 2-gram matrix $M_2[a,b]$ mono-run bypass + 2-gram IFS | `M_16` save `0.00%`; IFS med `16->4` / `349->8` (`-2.1/-4.1 b`) | Raw `Q/n=0.903–0.934`, genuine $M_{16}$ save `1.32–3.24% h_1`; IFS med `9–13×` lower (`-2.7/-2.9 b`) | `Q_acq=0, P=2, S_{M_2}<=37 KB`; `0%` on novel hex | [`E2`](E2-PREFIX-HISTOGRAM-M2.md) |
| **E3** | P2 (Invariance) | Dynamic 1-Lipschitz rank-envelope (`1/3` pin) & 3-rot invariance witnesses | `33.31%` / `31.93%` `h_1` forced; `dLF=3` witness `x=0` | `29.86–32.16%` `h_1` forced; 6/6 `dLF=3` single-cycle witnesses (`dT=10–79% n`) | Envelope check pays `Q>=1`; invariance witnesses only | [`E3`](E3-MONOTONE-ENVELOPE-DICHOTOMY.md) |
| **S7** | P1 (Contraction) | Pairing-invariance lemma (Relation C open) + Bi-Dir `LF x Psi` Wave | `34/40` hex digits `pi`-invariant; `D_wave`: `78->39`, `41->20` | `D_wave=24–43` (`2.0×` vs S5), `98.5–99.8%` by `r=21`, `P_bytes=14.8–54.8` (+ `3.2–7.2` sort) | Total `D=14.9–44.0k`; `Q_scatter=m` or `+4` radix passes; `P=26–45 > K=2` | [`S7`](S7-PAIRING-OBSTRUCTION-BIDIR-WAVE.md) |
| **S8** | P1 (Contraction) | Exact **`NEEDS M-21b`** count-targeted landmarks + `ORTH-BIDIR` | Raw `D=288` (`0.44%`), `185` (`0.23%`); `ORTH` `249`, `78` | **STANDS `6/6` `< 0.5%`**: raw **`D=608–1,974`** (`0.068–0.219% n`); `ORTH-BIDIR` **`D=627–1,776`** | Raw `Q=1.03–1.17 n`; `ORTH` `Q=0.34–0.60 n`, `P=4–6` (unrolling & label precompute caveats noted) | [`S8`](S8-COUNT-TARGETED-LANDMARKS.md) |
| **S9** | P1 (Contraction) | $k$-mer de Bruijn unitig endpoint merge on $L_{\min}=16$ | `k=2`: `V_1=0.00%`; `k=8`: `u_8=3` / `2,001` | `k=2`: `V_1<=0.39%`; `k=8`: `u_8=2.3–5.4k`, yet `Q_run=1.07–2.24 h_1 > h_1` on `6/6` | Nothing at `K=2` (`k=6` needs `>=3` passes via Manber–Myers) + `m_branch > h_1` at `L_min=16` | [`S9`](S9-UNITIG-ENDPOINT-MERGE.md) |
| **R1** | Ch. `#25` Leads | Renormalisation $q_{\min}(\sigma) = 1+\mathrm{desc}(\sigma^{-1})$ (Gessel–Reutenauer) | `LF^2`: `16->256`; `R(I_'0')`: `0.445|Y|`; `R(M_16)` tmpl: `24->19` | `LF^2`: `1.4–2.0k`; `R(I_'0')`: `2.0–4.1k`; `R(M_16)`: `748–1,919` (`8–21×` rise) | Lead 1 leaves small-alphabet class on random hex (`24->19` holds on synthetic template only) | [`R1`](R1-RENORMALISATION-LEADS.md) |

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

