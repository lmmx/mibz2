# S9 — Variable-Order / $k$-Mer De Bruijn Unitig Endpoint Merge on $L_{\min} = 16$

**Principle:** Contraction along composition (Principle 1: closing the exact open gap from `carbon-cycle1-critique.md` on Carbon C-4 — keeping `L_min = 16` cheap, replacing `qsort` + `38–93 h_1` thick probes with `Q = 0` ordered-frontier $k$-mer acquisition and sorted endpoint merge).  
**Script & Data:** [`s9_unitig_endpoint_merge.py`](s9_unitig_endpoint_merge.py), [`results_s9_unitig_endpoint_merge.json`](results_s9_unitig_endpoint_merge.json).

## 1. Exact Mathematical Definition

Rows $x \in M$ **move together along a de Bruijn unitig** when they form a maximal path of unique $k$-mers ($V_1(x) = 1$, meaning $\mathrm{name}_k(x) = T[\mathrm{sa}[x]..\mathrm{sa}[x]+k)$ occurs exactly once in $T$ and $x \neq p$).

**Zero-`Q`, Zero-Sort Ordered-Frontier Acquisition of $\mathrm{name}_k$ and Endpoint Merge:**
1. **Thick side (`L_min = 16`):** Genuine endpoint-only Rauzy–Veech strip contraction (`RetainedSystem`), eliminating Carbon C-4's `38–93 h_1` thick probes.
2. **$k$-mer names (`P = k - 1`, `Q = 0`, zero `qsort`):** Start from $\mathrm{name}_1[x] = F[x]$ (`P = 0`). At step $t \to t+1$, scan $x = 0 \dots n-1$ with 256 bucket cursors, emitting $(L[x] \ll 8t) \mid \mathrm{name}_t[x]$ into bucket $L[x]$. By the Ordered-Frontier Lemma, concatenating buckets $0 \dots 255$ produces $\mathrm{name}_{t+1}[y]$ in exact row order $y$ **and globally sorted lexicographic order** (Carbon C-4 Lemma U-1) with zero sorting.
3. **Uniqueness certificate $V_1(x)$ & sorted endpoint merge (`Q = 0`):** Because $\mathrm{name}_k$ is globally sorted, $V_1(x) = [\mathrm{name}_k[x] \neq \mathrm{name}_k[x-1] \land \mathrm{name}_k[x] \neq \mathrm{name}_k[x+1]]$ is a 1-pass sequential check, and bucketing $(\mathrm{name}_k[x] \gg 8)$ by $L[x]$ produces all destination $k$-mers in sorted order for a zero-`Q` linear endpoint merge.

## 2. Correctness & Theorem (`[proved]`, 8/8 byte-exact verified)

**Theorem S9.1 (The $k \ge \log_{|\Sigma|} n$ Pass-Wall & $L_{\min} = 16$ Branching Floor — proved & verified 8/8).**
1. *Exactness:* Every maximal $V_1$-chain in the retained permutation $R$ on $M$ has in-degree and out-degree $1$ on all interior nodes and never swallows `mark_rank`, so contracting each $V_1$-chain to a single jump from its head to its exit preserves the single cycle of length $m$. Verified `decoded == T_ref` byte-for-byte on **8/8 blocks** at $k \in \{2, 6, 8\}$.
2. *Pass-Wall on Novel Random Hex (corrected per `#26`):* On uniform random hex ($|\Sigma| = 16$), $\Pr[V_1(x) = 1] = (1 - 16^{-k})^{n-1} \approx \exp(-n/16^k)$. At $k = 2$ (the unique scale fitting within $\Pi$'s frozen $K = 2$ pass budget), $16^2 = 256 \ll n$, giving **$V_1 = 0.00\%$** on novel random hex. Reaching $V_1 > 50\%$ requires $k \ge \lceil \log_{16} n \rceil + 1$ ($k \ge 6$ on 900 KB blocks): linear step-by-step naming takes $k - 1 = 5$ bucket passes, while Manber–Myers prefix doubling names $k$-mers in $\lceil \log_2 k \rceil = 3$ passes (with sorting) — still exceeding $K = 2$.
3. *Spelling Obligation:* Even after $P_{\mathrm{names}} = k - 1$ passes identify the $u_k$ unitigs, spelling their bytes requires either walking the interior rows ($Q_{\mathrm{spell}} = \sum \ell_u \approx |V_1|$) or $O(\log \ell_{\max})$ further stream-contraction passes (S3/S4). $\square$

## 3. Prior Art

- **`research/carbon-kitchen-sink` Cycle 7 (`c9262be5`, Thm 7.1):** Burnside–BEST de Bruijn unitig condensation of the $(k+1)$-mer spectrum.
- **`research/carbon-new-basis` C-4 & `carbon-cycle1-critique.md`:** C-4 attached a row map at $k = 8, L = 1$ (paying `Q_tot = 38–93 h_1`, `P = 110–145`, uncharged `qsort`). The critique flagged resolving unitig endpoints by sorted merge at $L = 16$ as C-4's sole remaining live test.
- **What is new here:** Closes that exact open test: replaces `qsort` + thick probes with `Q = 0` ordered-frontier $k$-mer acquisition at $L_{\min} = 16$, sweeps $k \in \{2, 3, 4, 5, 6, 8\}$, and discovers BOTH the $k \ge \log_{16} n$ pass-wall AND that at $L_{\min} = 16$, non-unique thin runs (`w = 2..15`) keep $Q_{\mathrm{run}} = u_8 + m_{\mathrm{branch}} = 1.07–2.24\,h_1 > h_1$ on all 6/6 Wikidata blocks even at $k = 8$!

## 4. Measurements on Novel Random Hex & Six Admitted Blocks (8/8 Byte-Exact)

| Block | $L_{\min}$ | $k=2$ (`P=1`, fits `K=2`): $V_1(\%h_1)$ / $Q_{\mathrm{run}}/h_1$ | $k=6$ (`P=5`): $V_1(\%h_1)$ / $u_6$ / $Q_{\mathrm{run}}/h_1$ | $k=8$ (`P=7`): $V_1(\%h_1)$ / $u_8$ / $Q_{\mathrm{run}}/h_1$ | Byte-exact |
|---|---:|---:|---:|---:|:---:|
| `synth-hex-iid-65536` | 2 | **`0.00%`** / `1.000` | `99.6%` / `226` / `0.008` | `100.0%` / `3` / `0.000` | EXACT |
| `synth-tmpl-hex-1000x40` | 16 | **`0.00%`** / `1.178` | `85.1%` / `2,058` / `0.220` | `86.2%` / `2,001` / `0.206` | EXACT |
| `block-0.L` | 16 | `0.25%` / `2.513` | `74.2%` / `2,553` / `1.613` | `75.5%` / `2,424` / **`1.527`** | EXACT |
| `block-2.L` | 16 | `0.30%` / `3.196` | `73.4%` / `2,382` / `2.313` | `74.9%` / `2,288` / **`2.239`** | EXACT |
| `block-85.L` | 16 | `0.34%` / `2.794` | `73.4%` / `3,702` / `1.896` | `75.3%` / `3,311` / **`1.824`** | EXACT |
| `block-143.L` | 16 | `0.39%` / `2.664` | `72.9%` / `5,462` / `1.783` | `75.8%` / `5,171` / **`1.691`** | EXACT |
| `block-210.L` | 16 | `0.19%` / `2.081` | `77.0%` / `6,722` / `1.155` | `79.6%` / `5,400` / **`1.072`** | EXACT |
| `block-207.L` | 16 | `0.19%` / `2.156` | `73.5%` / `4,646` / `1.266` | `75.8%` / `4,494` / **`1.189`** | EXACT |

## 5. Referee Under $\Pi$

- **At `K = 2` (`k = 2`, `P_names = 1`):** `V_1 = 0.00%` on novel hex and `0.19–0.39%` on Wikidata — **no contraction.**
- **At `k = 8` (`P_names = 7 > K = 2`, `S >= 8n`):** Even before charging unitig spelling (`Q_spell` or `O(log ell)` extra passes) or thick strips (`Q_acq_strips`), keeping `L_min = 16` leaves `m_branch` (repeated width `2..15` thin rows) above `h_1` on all 6/6 Wikidata blocks (`Q_run = 1.072–2.239 h_1`), whereas lowering `L_min` to `1` triggers Carbon C-4's `38–93 h_1` thick-probe explosion.
- **Verdict:** Cleanly closes Carbon C-4's open test from `carbon-cycle1-critique.md` — de Bruijn unitigs hit a sharp two-sided wall ($P \ge \log_{|\Sigma|} n > K = 2$ on novel hex, and $m_{\mathrm{branch}} > h_1$ at $L_{\min} = 16$ on Wikidata).
