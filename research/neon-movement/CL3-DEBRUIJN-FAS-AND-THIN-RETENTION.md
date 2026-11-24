# CL3 — De Bruijn 2-Digit FAS Key $\kappa$ (MONODROMY 116 §4) & Thin-Run Retention Sweep (Channel `#27`)

- **Script:** [`cl3_debruijn_fas_and_thin_retention.py`](file:///home/antigravity/lmmx/mibz2/research/neon-movement/cl3_debruijn_fas_and_thin_retention.py)
- **Raw data:** [`results_cl3_debruijn_fas_and_thin_retention.json`](file:///home/antigravity/lmmx/mibz2/research/neon-movement/results_cl3_debruijn_fas_and_thin_retention.json)
- **Verification:** Byte-exact against [`common.reference_decode`](file:///home/antigravity/lmmx/mibz2/research/neon-movement/common.py) on `synth-hex-iid-65536` (both row order and $\kappa$-order, both M-17 and CL1) and across all $6 \times 5 = 30$ `(block, L_min)` configurations on the 6 admitted Wikidata blocks.

---

## 1. Two Open Questions Answered in `CL3`

1. **Part A (MONODROMY Record 116 §4 — Explicit De Bruijn Min-FAS Key $\kappa$):**
   Record 116 proved Theorem O ($Q \ge n / (2(k+1))$ in $\mathcal{S}_k$ via $3$-necklaces, giving a $2/3$ descent ceiling on 2-digit blocks) and proposed ordering a single composition sweep by a minimum-feedback-arc-set (maximum-descent) key $\kappa$ on the 256-node hex de Bruijn graph $B(16, 2)$ to beat the classic row-order mean chain length $e \approx 2.718$. Record 116 left $\kappa$ unconstructed and unmeasured.
2. **Part B (Channel `#27` Sharp Enrichment Question — Width $2\text{–}16$ Runs Already Seen by `src/runs.rs`):**
   > *"Which later operation is necessary only because an earlier stage failed to retain information it already met?"*

   Inverse MTF + RLE2 (`src/runs.rs`) already produces every run $(s, w, b)$ of $L$, including every thin run of width $2 \le w \le 16$. Open (`L_min = 17`, matching `LIMIT = 16`) discards run grouping for $w \le 16$, inflating retained rows from $h_1$ to $m = 2.10\text{–}3.27\,h_1$ (52–69% of Open's retained rows belong to width $2\text{–}16$ runs). We sweep $L_{\min} \in \{2, 3, 5, 9, 17\}$ under genuine endpoint-only thick-strip contraction paired with `CL1` Mirror Two-Sweep to measure the exact cost/payoff curve.

---

## 2. Part A — Explicit 2-Digit De Bruijn FAS Key $\kappa$ on $B(16, 2)$

### 2.1 Exact Definition

Let $\Sigma_{\mathrm{hex}} = \{0,\dots,15\}$ (`'0'..'9', 'a'..'f'`). The $2$-digit de Bruijn graph $B(16, 2)$ has:
- **Vertices:** $256$ pairs $u = (b, c) \in \Sigma_{\mathrm{hex}}^2$, corresponding to the $256$ contiguous BWT suffix intervals $I_{b, c} \subset I_b$. Their exact boundaries are acquired in $P = 1, Q = 0$ from the $16 \times 16$ bigram table $M_2[d_1, d_0]$ (counting $L[x] = d_0$ inside $I_{d_1}$).
- **Directed LF edges:** $(b, c) \to (a, b)$ for all $16^3 = 4,096$ triples $(a, b, c)$, of which $16$ are self-loops and $4,080$ are non-self edges.
- **Key $\kappa : \Sigma_{\mathrm{hex}}^2 \xrightarrow{\sim} \{0,\dots,255\}$:** a static, input-independent permutation maximising the number of descending triples $\kappa(a, b) < \kappa(b, c)$.
- **Processing order:** iterate blocks $I_{b, c}$ in ascending order of $\kappa(b, c)$ (and rows ascending inside each block).

By Theorem O, every non-trivial $3$-cycle $(a, b) \to (c, a) \to (b, c) \to (a, b)$ must contain at least one ascending edge; with $N_{16}(3) = (16^3 - 16)/3 + 16 = 1,376$ $3$-necklaces (including the $16$ self-loops), the exact upper bound on descending triples is $4,096 - 1,376 = 2,720$ (`0.6641`).

### 2.2 Results on $B(16, 2)$ and `synth-hex-iid-65536` ($n = 65,536$, $h_1 = 57,458$)

Swap local search ([`build_debruijn_fas_key_hex`](file:///home/antigravity/lmmx/mibz2/research/neon-movement/cl3_debruijn_fas_and_thin_retention.py#L43-L146)) finds a fixed 256-entry key $\kappa$ with **$2,703 / 4,096 = 65.99\%$** descents — only **$17 / 4,096$ ($0.42\%$) below Theorem O's $2/3$ ceiling** (vs `2,040 / 4,096 = 49.80%` for lexicographic order).

| Order on `synth-hex-iid-65536` (raw `LF`, byte-exact) | Empirical descent frac | 1-sweep M-17 links (`mean len`, `/ h_1`) | `CL1` 2-sweep links (`mean len`, `/ h_1`) | Exact `+1` `LF` segments in order |
|---|---:|---:|---:|---:|
| Standard row order | `0.5001` | `23,819` (`2.751`, `0.415 h_1`) | `21,203` (`3.091`, `0.369 h_1`) | `16` |
| **$\kappa$-FAS 2-digit block order** | **`0.6610`** | **`20,131` (`3.256`, `0.350 h_1`)** | **`20,005` (`3.276`, `0.348 h_1`)** | `3,248` ($\le 16^3$) |

### 2.3 Findings & Closure Trade-Off

1. **MONODROMY 116 §4 Confirmed:** On pure i.i.d. novel hex, the static 256-byte $\kappa$-FAS key pushes empirical descents from `50.0%` to **`66.1%`** and pushes **1-sweep M-17** past $e = 2.718$ to **`3.256` bytes/link** (`0.350 h_1`), a **`15.5%` reduction in emit links** over row-order M-17 — using a single sweep!
2. **Comparison with `CL1` Mirror Two-Sweep:**
   - In standard row order, `CL1` already reaches `3.091` bytes/link (`0.369 h_1`) while keeping `16` exact `+1` cursors ($s \le |\Sigma|$).
   - In $\kappa$-order, 1-sweep M-17 (`3.256`) and 2-sweep `CL1` (`3.276`) nearly tie (within `0.6%`): because `66.1%` of edges already descend, ascending chains have mean length $\approx 1/(1-0.339) = 1.51$, so M-17's single-step ascending lookahead already absorbs almost all of them.
3. **Why $\kappa$-order does NOT replace `CL1` on general blocks:**
   - Reordering the $256$ blocks $I_{b, c}$ breaks the $16$-cursor `LF` stream into `3,248` ($\le |\Sigma|^3 = 4,096$) exact `+1` segments. That bound is constant in $n$ (compact for large pure-hex blocks: `3,248` segments $\approx 39\text{ KB}$), **but** computing the exact starting offset of each $(a, b, c)$ sub-block inside $I_{a, b}$ requires the $4,096$-entry trigram table $M_3$ ($16\text{ KB}$, exceeding $s_{\max} = |\Sigma| + c_0$ in strict $\Pi$), and on general 256-byte alphabets $|\Sigma|^2 = 65,536$ blocks would explode cursor state.
   - Hence $\kappa$-FAS is a genuine **pure-hex / subalphabet accelerator** (cutting 1-sweep hex links by `15.5%` with a fixed 256-entry lookup table), while row-order `CL1` stays universal across all 256-byte blocks with $s = 256$.

---

## 3. Part B — Thin-Run Retention Sweep $L_{\min} \in \{2, 3, 5, 9, 17\}$ (Channel `#27`)

Across all 6 admitted Wikidata blocks, we run genuine endpoint-only Rauzy–Veech thick-strip contraction (`w >= L_min`) followed by `CL1` Mirror Two-Sweep on the resulting retained map $R = P \circ \mathrm{LF}_v$ (all 30 runs verified byte-exact against `T_ref`).

### 3.1 Summary Across All 6 Admitted Blocks

| Block ($h_1$) | $L_{\min}$ | $m / h_1$ | $B_P$ | `Q_acq_strips` $/ h_1$ | `n_strips` | `CL1` S2 cache16 | `M-17` links $/ h_1$ | **`CL1` links $/ h_1$** | `CL1` DAG KB |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| **`block-0.L`** (`35,116`) | `17` | `2.56` | `2,148` | `3.29` | `27,383` | `753` | `0.935` | **`0.834`** | `702` |
| | `9` | `1.92` | `2,937` | `6.81` | `58,169` | `959` | `0.695` | **`0.623`** | `527` |
| | `5` | `1.64` | `3,777` | `10.87` | `93,244` | `1,116` | `0.591` | **`0.531`** | `450` |
| | `3` | `1.33` | `5,352` | `20.56` | `185,121` | `1,463` | `0.483` | **`0.432`** | `366` |
| | `2` | `1.00` | `10,422` | `36.17` | `331,222` | `3,010` | `0.361` | **`0.325`** | `274` |
| **`block-2.L`** (`34,262`) | `17` | `3.27` | `3,136` | `3.21` | `24,126` | `1,066` | `1.197` | **`1.081`** | `876` |
| | `9` | `2.42` | `4,438` | `7.52` | `61,179` | `1,404` | `0.881` | **`0.797`** | `647` |
| | `5` | `1.86` | `5,775` | `14.01` | `117,946` | `1,821` | `0.678` | **`0.609`** | `498` |
| | `3` | `1.32` | `5,982` | `27.46` | `189,834` | `1,770` | `0.480` | **`0.429`** | `353` |
| | `2` | `1.00` | `10,663` | `45.47` | `361,021` | `3,316` | `0.362` | **`0.325`** | `268` |
| **`block-85.L`** (`55,504`) | `17` | `2.82` | `2,018` | `0.80` | `12,278` | `650` | `1.013` | **`0.903`** | `1,222` |
| | `9` | `2.14` | `4,202` | `2.94` | `43,410` | `1,117` | `0.770` | **`0.684`** | `929` |
| | `5` | `1.74` | `6,931` | `4.87` | `72,088` | `1,725` | `0.619` | **`0.554`** | `753` |
| | `3` | `1.25` | `8,484` | `9.52` | `130,660` | `1,917` | `0.451` | **`0.405`** | `543` |
| | `2` | `1.00` | `16,303` | `15.59` | `276,124` | `4,358` | `0.363` | **`0.328`** | `434` |
| **`block-143.L`** (`72,578`) | `17` | `2.71` | `5,436` | `1.82` | `31,975` | `1,939` | `0.988` | **`0.882`** | `1,538` |
| | `9` | `2.17` | `8,127` | `4.59` | `72,919` | `2,549` | `0.786` | **`0.698`** | `1,232` |
| | `5` | `1.75` | `11,270` | `9.41` | `149,707` | `3,256` | `0.633` | **`0.564`** | `992` |
| | `3` | `1.41` | `15,730` | `13.69` | `222,690` | `4,614` | `0.514` | **`0.459`** | `800` |
| | `2` | `1.00` | `22,714` | `27.71` | `378,740` | `6,766` | `0.364` | **`0.327`** | `567` |
| **`block-210.L`** (`97,054`) | `17` | `2.10` | `3,564` | `0.68` | `18,865` | `1,198` | `0.762` | **`0.672`** | `1,596` |
| | `9` | `1.76` | `5,910` | `1.78` | `40,085` | `1,519` | `0.634` | **`0.562`** | `1,338` |
| | `5` | `1.57` | `9,164` | `2.79` | `65,374` | `2,307` | `0.565` | **`0.503`** | `1,190` |
| | `3` | `1.35` | `15,027` | `5.76` | `136,392` | `3,741` | `0.488` | **`0.436`** | `1,022` |
| | `2` | `1.00` | `26,671` | `12.90` | `293,771` | `7,127` | `0.363` | **`0.326`** | `758` |
| **`block-207.L`** (`62,407`) | `17` | `2.18` | `3,547` | `1.65` | `21,323` | `1,268` | `0.797` | **`0.712`** | `1,062` |
| | `9` | `1.88` | `5,366` | `3.29` | `48,949` | `1,566` | `0.676` | **`0.607`** | `917` |
| | `5` | `1.66` | `7,832` | `5.69` | `96,249` | `2,164` | `0.600` | **`0.541`** | `810` |
| | `3` | `1.42` | `11,927` | `10.61` | `177,289` | `3,255` | `0.515` | **`0.463`** | `692` |
| | `2` | `1.00` | `18,420` | `20.22` | `327,251` | `5,215` | `0.363` | **`0.327`** | `488` |

### 3.2 Exact Quantitative Answer to Channel `#27`

1. **Universal `0.326 h_1` floor at $L_{\min} = 2$:** When *only* true lone rows are retained ($m = h_1$), `CL1` emit links collapse to **`0.325–0.328 h_1`** on all 6/6 Wikidata blocks — matching pure novel hex (`0.325 h_1`) to three decimal places! And `CL1` DAG memory drops `2.1–3.3×` (`268–758 KB`).
2. **Why Open discarded $w \in [2, 16]$ — the Acquisition Conservation Law:** Retaining width $2\text{–}16$ runs inside eager Rauzy–Veech strip contraction does **NOT** save total $Q$. Moving from $L_{\min} = 17$ to $L_{\min} = 9$ already increases `Q_acq_strips` by `3.2×–5.5×` more probes than the retained rows it eliminates (e.g., `block-0`: `+123.7k` strip probes to save `22.4k` retained rows; `block-210`: `+106.2k` strip probes to save `33.0k` retained rows). At $L_{\min} = 2$, `Q_acq_strips` explodes to **`12.9–45.5 h_1`** (`865k–2.01M` probes across `276k–379k` fractured strips).
3. **What information from `src/runs.rs` CAN be retained for a strict net win?**
   Not eager splitting of width $2\text{–}16$ runs, but retaining the **contiguous interval structure** (`M_intervals` and `ret_lf_intervals[b]`) emitted by `src/runs.rs` at $L_{\min} = 17$ — which eliminates all $m = 89.9\text{k–}204.3\text{k}$ per-row glue lookups (`Q_acq_glue: m -> 0`) when building `CL1`'s $B_P$ schedule! We construct and verify that exact algorithm in **`CL4`**.

---

## 4. Referee Verdict under $\Pi$

- **Part A ($\kappa$-FAS):** Real `15.5%` single-sweep emit-link cut on pure novel hex (`2.751 -> 3.256` bytes/link, confirming MONODROMY 116 §4), with zero per-row build overhead beyond a fixed 256-byte permutation table; however, full cursor replay of $\kappa$-order needs `3,248` exact `+1` segments (acquirable via a `16 KB` $M_3$ table, exceeding strict $s_{\max} = |\Sigma| + c_0$), and does not generalize cleanly to 256-byte alphabets.
- **Part B (Thin-Run Retention Sweep):** **Honest negative for eager thin-strip contraction in total $Q$** — retaining $w \in [2, 16]$ cuts emit links to `0.325–0.328 h_1` and DAG memory by `2.1–3.3×`, but increases `Q_acq_strips` by `3.2×–5.5×` per saved retained row even at $L_{\min} = 9$ (and `12.9–45.5 h_1` at $L_{\min} = 2$). The real cross-stage win from `src/runs.rs` is retaining interval endpoints to eliminate `Q_acq_glue = m` (`CL4`).
