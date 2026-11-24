# E1 — Upstream Cross-Stage Enrichment: The MTF–Occurrence–Gap Semidirect Monoid $\mathcal{M}_{\mathrm{MTF}}$ (and `NEEDS M-22`)

**Principle:** Enrichment along composition (Principle 2, cross-stage track `CROSS-STAGE.md`).  
**Script & Data:** [`e1_cross_stage_mtf_monoid.py`](e1_cross_stage_mtf_monoid.py), [`results_e1_cross_stage_mtf_monoid.json`](results_e1_cross_stage_mtf_monoid.json).

## 1. Exact Mathematical Definition

Standard bzip2 decodes `Huffman tokens -> MTF+RLE2 -> n-byte column L -> histogram C[c] -> Open runs IN / inverse table tt`. We replace the serial MTF state (`S_Sigma`) with a single associative **semidirect monoid** serving MTF, occurrence prefix sums, Open run construction, and per-symbol gap streams simultaneously without ever materialising `L[0..n-1]`:
$$\mathcal{M}_{\mathrm{MTF}} = \big(\mathbb{N}^{\Sigma} \times \mathbb{Z}_{\min}^{\Sigma} \times \mathbb{Z}_{\max}^{\Sigma} \times \mathbb{N}\big) \rtimes S_{\Sigma}.$$

Each chunk $g$ of the RLE2+MTF token stream (aligned on non-`RUNA/RUNB` boundaries, evaluated independently starting from the identity slot order $[0, \dots, m-1]$) produces an element $(\Delta_g, f_g, \ell_g, L_g, \sigma_g)$ together with its **active support width** $K_g = 1 + \max\{\text{MTF depths touched in } g\}$:
- $\sigma_g[p]$: the initial slot occupying MTF position $p$ at chunk exit (fixed for all $p \ge K_g$);
- $\Delta_g[i]$: total emitted bytes of the symbol initially at slot $i$;
- $f_g[i] \in \mathbb{N} \cup \{+\infty\}$ and $\ell_g[i] \in \mathbb{N} \cup \{-\infty\}$: local first and last emitted row offsets of initial slot $i$;
- $L_g = \sum_i \Delta_g[i]$: total row span of chunk $g$.

The monoid product $(A \odot B)$ composes in $O(\max(K_A, K_B))$ operations:
$$\sigma_{AB}[p] = \sigma_A[\sigma_B[p]], \qquad L_{AB} = L_A + L_B,$$
and for each active slot $q < K_B$ mapped to $i = \sigma_A[q]$:
$$\Delta_{AB}[i] = \Delta_A[i] + \Delta_B[q], \qquad f_{AB}[i] = \min(f_A[i],\, L_A + f_B[q]), \qquad \ell_{AB}[i] = \max(\ell_A[i],\, L_A + \ell_B[q]).$$

## 2. Correctness (`[proved]`, 8/8 byte-exact verified)

**Theorem E1.1 (Strict Associativity & Zero-`L` Equivalence — proved).**
1. $\odot$ is strictly associative on $\mathcal{M}_{\mathrm{MTF}}$: $\sigma_{(AB)C} = \sigma_A \circ \sigma_B \circ \sigma_C = \sigma_{A(BC)}$, and right-action of $S_\Sigma$ on $(\Delta, f, \ell)$ distributes over $(\mathbb{N}, +)$, $(\mathbb{Z}_{\min}, \min)$ with translation $L_A$, and $(\mathbb{Z}_{\max}, \max)$ with translation $L_A$.
2. An exclusive prefix scan over $G$ chunks yields, at every chunk boundary $g$ in $O(\log G)$ parallel depth: the exact entry MTF permutation $\sigma_{<g}$, starting row $L_{<g}$, starting per-symbol occurrence counts $\Delta_{<g}$, and previous emitted row $\ell_{<g}[s]$ for every symbol $s \in \Sigma$.
3. Each chunk $g$ therefore emits, in parallel and with **zero bytes of `L` allocated, written, or read**, both (a) exact Open run records `IN = (start, width, sym, occ_rank)` and (b) MONODROMY 115's exact per-symbol gap-encoded $\Psi$ streams (absolute `u32` anchor every 16th occurrence + 1-byte gap $g \in [1, 254]$ or `255 + u32` escape).

*Verification:* Associativity asserted directly on sample chunk triples; zero-`L` `IN` runs asserted identical to `compute_counts_and_runs(L)` on all 8/8 blocks; forward-$\Psi$ inverse BWT decoded directly from the zero-`L` gap streams asserted `decoded == T_ref` byte-for-byte on **8/8 blocks**.

## 3. Prior Art

- **Sadakane (2000, 2003) & Grossi–Vitter (2000, 2005) Compressed Suffix Arrays (CSA):** Differenced, sampled $\Psi$ with bucket-rank lookup on $C[c]$.
- **`src/runs.rs` and `src/run_fed.rs`:** The production Open front end (`src/runs.rs`) already converts inverse MTF/RLE2 directly into run records and byte counts without a separate full-column pass; `src/run_fed.rs` emits serial `Run` records for the `T`-vector.
- **External reports 5–7 (30 Oct, `95b9cf9b`, `cc6c5831`):** Chunk-parallel MTF via $(O, P)$ permutation composition in $S_\Sigma$.
- **MONODROMY record 115 §3 & §5 (`NEEDS M-22`):** Stated the discrete-derivative identity for per-symbol gap streams and requested the six-block measurement.
- **Scope (corrected per `#26`):** E1 is a **memory trade** (2.7× smaller $\Psi$ table in exchange for inline gap decoding and two MTF-token scans) plus a parallel chunk-prefix measurement for `NEEDS M-22`.

## 4. Measurements on Novel Random Hex & Six Admitted Blocks (8/8 Byte-Exact)

| Block | $|\Sigma|$ | `tok/n` | $K_{50}$ med / p95 | $D_{\text{prefix}}$ (`1k`-chk) | `M-22` full (no dir) | `M-22` full (w/ dir, `u32` / varint) | `gap==1` / esc% | `M-22` $M_{16}$ (w/ dir, `u32` / varint, esc%) |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| `synth-hex-iid-65536` | 16 | 0.996 | 16 / 16 | 1,036 | `1.19 B/r` (`0.297× 4n`) | `1.44 / 1.44 B/r` (`0.360× 4n`) | `6.4%` / `0.00%` | `1.44 / 1.44 B/m` (`0.360× 4m`, `0.0%`) |
| `synth-tmpl-hex-1000x40` | 29 | 0.573 | 17 / 17 | 1,036 | `1.20 B/r` (`0.299× 4n`) | `1.45 / 1.44 B/r` (`0.361× 4n`) | `47.1%` / `0.20%` | `1.45 / 1.45 B/m` (`0.363× 4m`, `0.3%`) |
| `block-0.L` | 89 | 0.100 | 24 / 67 | 1,038 | `1.22 B/r` (`0.304× 4n`) | `1.47 / 1.45 B/r` (`0.367× 4n`) | `94.3%` / `0.77%` | `1.71 / 1.57 B/m` (`0.428× 4m`, `7.2%`) |
| `block-2.L` | 86 | 0.113 | 25 / 63 | 1,038 | `1.22 B/r` (`0.305× 4n`) | `1.47 / 1.45 B/r` (`0.368× 4n`) | `94.0%` / `0.89%` | `1.71 / 1.57 B/m` (`0.427× 4m`, `7.1%`) |
| `block-85.L` | 94 | 0.141 | 24 / 66 | 1,038 | `1.22 B/r` (`0.305× 4n`) | `1.47 / 1.45 B/r` (`0.368× 4n`) | `91.2%` / `0.87%` | `1.63 / 1.53 B/m` (`0.407× 4m`, `5.1%`) |
| `block-143.L` | 96 | 0.200 | 25 / 69 | 1,040 | `1.24 B/r` (`0.309× 4n`) | `1.49 / 1.46 B/r` (`0.371× 4n`) | `88.0%` / `1.25%` | `1.66 / 1.54 B/m` (`0.415× 4m`, `5.9%`) |
| `block-210.L` | 96 | 0.206 | 23 / 64 | 1,040 | `1.23 B/r` (`0.307× 4n`) | `1.48 / 1.46 B/r` (`0.370× 4n`) | `85.7%` / `1.09%` | `1.62 / 1.52 B/m` (`0.406× 4m`, `4.9%`) |
| `block-207.L` | 94 | 0.153 | 26 / 67 | 1,040 | `1.24 B/r` (`0.309× 4n`) | `1.49 / 1.46 B/r` (`0.371× 4n`) | `90.3%` / `1.27%` | `1.76 / 1.59 B/m` (`0.440× 4m`, `8.5%`) |

**Key findings:**
1. **Token stream & active support:** On Wikidata, the RLE2+MTF token stream is `0.100–0.206·n`, and 50-symbol groups touch a median of `K_50 = 23–26` MTF slots (`16` on pure hex).
2. **`NEEDS M-22` verdict:** On full `L`, gap streams take `1.22–1.24 B/row` (`0.304–0.309× 4n`) without a block-offset directory, or `1.47–1.49 B/row` (`0.367–0.371× 4n`) with an `O(1)` 16-entry block-offset directory (`0.77–1.27%` escapes on Wikidata; `0.00%` on novel hex). On Open's retained rows $M_{16}$, escapes rise to `4.9–8.5%`, giving `1.62–1.76 B/m` (`0.406–0.440× 4m`), while paying an inline prefix sum of mean `7.5` gap bytes per step.

## 5. Referee Under $\Pi$ (corrected per `#26`)

- **`Q`:** `Q_acq = 0`, `Q_run = n` (full gap walk) or `Q_run = m` (on $M_{16}$). **Does NOT cut `Q`.**
- **`D`:** `D_prefix = 1,038–1,040` is the **chunk-prefix scan depth alone** (determined by the 1,024-token chunk size, with Huffman decode and emit `D_run = n` excluded). Total decode depth remains `D = n`.
- **`P`, `S`:** Decodes the MTF token stream twice (`P_acq = 2` over `0.10–0.21·n` tokens) and scatters into per-symbol streams (`S = 1.33 MB` vs `3.60 MB` for `4n` `tt`).
- **Verdict:** A **CSA-style memory trade** (`2.7×` smaller $\Psi$ table for inline gap decoding) and exact `NEEDS M-22` measurement; does not lower decoder work or `Q`.
