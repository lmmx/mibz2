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

## Cycle 3 — The Closure Problem (`CL1–CL4`, Channel `#26–#27`, `chatgpt-closure.md` §5)

Following channel entries `#26–#27`, journal §42, `chatgpt-closure.md` (at `c1eee5da`), and MONODROMY record 116:
- **Channel `#26` corrections applied** (`c2f7e0b`): superseded Cycle-1 claims struck in place in `README.md`, `S4`, `S5`; Cycle-2 claims tightened in `E1`, `S7`, `S8`, `S9`, `R1`.
- **Coordinator corrections adopted (`#27`):** `G1` is not a proved machine lower bound; the supplied-codes `16%` figure (§30) is not a ceiling; cheap text-position computation is legitimate (only unpaid acquisition is ruled out).

**Headline verdict (Cycle 3 — corrected per `#28` / `reviews/neon-movement-cycle3-critique.md`):** **Closure is NOT achieved, and no route breaks `h_1`.**
1. **`CL1` (N-20 C4 unimodal arches on $R$) — no closure:** The earlier "44–133× less fragmented" compared Sweep 2's *read* schedule ($R$ itself) with M-17's *output* map. On ChatGPT's output-map metric, `CL1` has `54,438` exact `+1` segments vs M-17's `52,319` on `block-0.L`. Its `8m` binary DAG defers emit work (`m-1` internal DAG visits + `m` leaf loads = `4.25–6.63 h_1` when charged). Links of `~m/3` (`0.672–1.081 h_1`) are the random-permutation peak/valley statistic, not a floor.
2. **`CL2` (Bayer–Diaconis 1992 rising sequences):** $\mathrm{seg}_{+1}(T, \ell) \ge 1 + \mathrm{desc}(T^{-1})$ is the textbook rising-sequence count (not representation-independent across sweep orders/layouts, and `q_min(R_2) = 0.18 h_1` on `block-0.L` is not a wall once labelling is acquired). Its uniform depth-2 `(v, v_R)` schedule (`6,339–16,785` segments) is costed with payloads in Cycle 4.
3. **`CL3` (de Bruijn FAS $\kappa$ & $L_{\min}$ sweep):** Confirms MONODROMY 116 §4 on pure hex (`3.256` bytes/link, niche to 16-symbol hex); eager thin-run contraction ($L_{\min} \in \{2..17\}$) is a sound negative (`3.2–5.5×` more probes per saved row).
4. **`CL4` (endpoint schedule):** Eliminates per-row glue against `common.py`'s strawman, **but** its own binary-search wide-strip acquisition still costs `56.5k–163.7k` probes (`0.88–4.07 h_1`), bringing total `Q` to **`1.55–4.99 h_1`** (with sweeps and DAG uncharged).

---

## Cycle 4 — Targets T1/T2 (Acquisition & Depth-2 Payloads), T5 (Cross-Stage Huffman), and #31/#33 (`math-targets/`)

Following channel entries `#28–#33`, journal §§43–46 (`a3f7d9ed`), `math-targets/` (`targets.tsv`, `metrics.tsv`, `data/measurements-four-arm.tsv`), and `reviews/neon-movement-cycle3-critique.md`. Every headline is reported in the referee's terms (`Q_total / h_1` with acquisition and emission included, compared like for like), with deterministic TSV datasets in [`data/`](data/).

| Item | Target | Referee Headline (`Q_total / h_1`, 6 Wikidata blocks, like for like) | Prior Art | Script & TSVs | Report |
|---|---|---|---|---|---|
| **T1/T2** | Workers: acquisition & depth-2 `R⁴` | **No route breaks `h_1/2`.** Open IET wide-strip acq (`0.10–0.34 h_1`, `0` searches) fixes `CL4`'s `0.88–4.07 h_1` bsearch blowup. **`NEEDS M-27` CLOSED 6/6:** lazy `S*_1` leaves `m = 0.891–0.925 m_16 > 0.85` and explodes `B_P` `4.1–8.7×`, worsening every composed route `15–81%`. At `k=0` (`M_16`), `(v, v_R)` `R⁴` under L1 pair-cursors (`1.9–2.9k` pairs = `7.5–11.4 KB`) gives **`0.690–1.173 h_1`** links+build (`0.790–1.512` w/ IET; **`1.216–1.991 h_1`** w/ flat emit payload) vs Open walk `2.105–3.271 h_1` (`4.210–6.542` w/ dict), M-17 `0.799–1.289 h_1` (`1.561–2.486` flat), CL1 `0.709–1.172 h_1` (`1.382–2.253` flat / `4.246–6.634` DAG) — **but** pays `4n = 3.52 MB` flat words (4-code tuples pay `m` dict reads = `2.79–4.44 h_1`; strict $\Pi$ $s \le 256$ loses at `2.668–4.180 h_1`). All 8/8 byte-exact vs standalone LF oracle. | Nishimoto–Tabei 2021 move structure; MONODROMY 120 Prop A; Open `src/open/endpoint.rs`; `T²`/KKP `LF²` (killed 23 Nov). | [`t1_t2_lazy_acq_and_depth2.py`](t1_t2_lazy_acq_and_depth2.py) → [`t1_t2_lazy_thin_acquisition.tsv`](data/t1_t2_lazy_thin_acquisition.tsv), [`t1_t2_depth2_schedule_costed.tsv`](data/t1_t2_depth2_schedule_costed.tsv) | [`T1-T2`](T1-T2-LAZY-ACQ-AND-DEPTH2.md) |
| **T5** | Cross-stage: Huffman → workers | **$\Delta Q = 0$** (`Q_total / h_1` unchanged). **Thms T5.1–T5.2** (proved & witnessed 8/8): flipping `L[n-1]` shifts `15.4–50.0%` of exact `LF` targets (pre-EOB causality); reversing `seq_to_unseq` with bit-identical `syms` shifts `100%` of `C[L[x]]` (pre-MTF equivariance). Thus four-arm `26%` (`D/A=0.741`) is **pipeline masking of T4 (`0.806`) × T1 (`0.920`)**. Streamed/T3–T2 gains: RLE2 alone recovers 100% of `(n,r,h_1,m_16,n_thick,mark_rank)`, strips `30.5–46.5%` `RUNA/B` (`-30..46%` queue KB), exposes `4.0–5.3k` redundant `CAP=127` splits (fusible in worker at 0 scanner cost), and genuine libbzip2 50-sym selectors (`1.8–3.7 KB`) separate hex (`83–85%` lone, `0.03–0.25%` `nn≥32`) from thick templates (`91–98%` thick), `NMI = 0.190–0.338`. | MONODROMY 121–122; `src/runs.rs`; 22 Nov fused MTF+Huff kill (`1.185×`); Patel et al. 2012; pugz/rapidgzip. | [`t5_cross_stage_huffman_enrichment.py`](t5_cross_stage_huffman_enrichment.py) → [`t5_huffman_stage_accounting.tsv`](data/t5_huffman_stage_accounting.tsv), [`t5_selector_regime_map.tsv`](data/t5_selector_regime_map.tsv) | [`T5`](T5-CROSS-STAGE-HUFFMAN-ENRICHMENT.md) |
| **#31/#33** | Pipeline model (1–20) & metrics | `wall(k) = c + max(b, a/k)` is misspecified on 10c/20t i9: linear `(1,6)→10` already misses Arm C by **`+13.9%`** (`+23.6%` D) before SMT, ignores the `k = 9→10` SMT kink (`k+1=11` threads), and puts naive `k* = 6.51`. Use direct non-parametric $\Delta_k = S_{\text{busy}} - \overline{W}_{\text{busy}}$ / piecewise USL on `1..9` (`13–20` rotated rounds pins ±1w). #33: measure worker split (`T2,T3,T6`), Q-to-seconds & construct exchange rates on 232 census blocks, and `k=8..12` IPC/LLC; reason about exactness, referee screening, pipeline min-combination, and obstructions. | Denning & Buzen 1978; Gunther USL; tandem queueing; MONODROMY 123–124. | [`pipeline_model_audit.py`](pipeline_model_audit.py) → [`pipeline_model_four_arm_audit.tsv`](data/pipeline_model_four_arm_audit.tsv) | [`#31/#33`](PIPELINE-MODEL-AND-METRICS-31-33.md) |

---

## Channel #34 — Decoder Character Across 1–20 Workers

- **[`DECOMPOSITION.md`](DECOMPOSITION.md):** Answers the four questions of `briefs/decomposition.md` (`9174b294d`) and journal §§44 & 47: (1) code-grounded audit of `S0–S7`/`R1–R3` (splitting `S5` into `S5a/b/c`, `S3` into `S3a/b/c`, `S7` into `S7a` + serial `S8`, `R1` into `R1a/b/c`; adding `S9`, `R4`, `R5`; moving `R3` onset to `k = 8–9` for `k + 3` threads; dropping NUMA); (2) 4-tier linear design (~90 conditions, ~35–45 min) with `four-arm` noise proof showing why `δ = 5–10%` unpinned fails (`1,250–5,110` rounds) while pinned `δ = +0.25` buys `±0.05` in `10` rounds; (3) proved summation-theorem traps ($\sum \hat{C}_i^- \to 2$ at `k*`), L3-hit vs L3-spill DRAM bounds (`11.4` vs `1.6 GB/s`), and `READY_BYTES` head-of-line throttling; (4) coupled stage-roofline + exact trace-driven `src/parallel.rs` simulator + MCA elasticities/validation (with Coz's shared-resource blind spot).

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

# Cycle 3 (CL1-CL4)
python3 research/neon-movement/cl1_mirror_factored_closure.py
python3 research/neon-movement/cl2_closure_obstruction_anatomy.py
python3 research/neon-movement/cl3_debruijn_fas_and_thin_retention.py
python3 research/neon-movement/cl4_endpoint_schedule_acquisition.py

# Cycle 4 (T1/T2, T5, #31/#33) -> regenerates all TSVs in data/
python3 research/neon-movement/t1_t2_lazy_acq_and_depth2.py
python3 research/neon-movement/t5_cross_stage_huffman_enrichment.py
python3 research/neon-movement/pipeline_model_audit.py
```

Every script reads the six admitted blocks directly from `origin/data/admitted-bwt` (verifying SHA-256 against `manifest.json`) and asserts byte-exactness against an independent textbook `LF` oracle on every block.


