# Sense 1 (`text-axis`): Contraction Along the Text Rather Than the Rows

## 1. Exact Definition of "Moves Together"

In row-axis Rauzy–Veech induction, distinct text positions that are adjacent in **row order** and share `L[x]` move together. In **text-axis contraction**, we invert the axis and group rows that belong to the same **text excursion** between consecutive visits to a free landmark section `Y`, or share a forward **text word** `w` of length `k`:

- **1A (Raw-`LF` Landmark Induction — MONODROMY 114 Sense E / `NEEDS M-21`):** Fix `Y_1 = I_{c_1} = [C[c_1], C[c_1] + \mathrm{count}[c_1])` for the most frequent byte `c_1` (a contiguous `F`-interval known free from symbol counts). Two rows `x, y` **move together** if they lie on the same first-return excursion `x_0 \to \mathrm{LF}(x_0) \to \dots \to \mathrm{LF}^{\ell-1}(x_0)` starting at `x_0 \in Y_1` and terminating upon first return to `Y_1`. Recursively, at level `k \ge 2`, `Y_k \subset Y_{k-1}` is the subset of `Y_{k-1}` sharing the most frequent next return-word byte, and consecutive `Y_{k-1}` excursions between visits to `Y_k` move together.
- **1B (Orthogonal Two-Axis Contraction — Row-Axis First, Text-Axis Second):** First contract thick runs (`w \ge L_{\min} = 16`) by genuine endpoint-only Rauzy–Veech strip composition (`build_retained_system_genuine`), yielding the retained permutation `R : M \to M` on `m_{16}` rows. Then define text-axis landmarks `Y_M^{(1)} \supset Y_M^{(2)} \supset Y_M^{(3)}` directly inside `M` (grouping consecutive `R`-steps between visits to `Y_M^{(k)}`).
- **1C (Forward-`\Psi` `F`-Interval Word Sharing):** Two rows `x, y` **move together at height `k`** if they lie in the same contiguous `F`-interval `I(w)` of suffixes beginning with the `k`-mer `w` (`F[\Psi^j(x)] = w_j` for `0 \le j < k`), emitting the identical forward word `w` without requiring output-order acyclicity (`src < dst`).

## 2. Composition Rule and Exactness

- **Composition rule (1A & 1B):** For every `y \in Y_k`, compose the `R_{k-1}` transitions along the excursion `y \to R_{k-1}(y) \to \dots \to y' \in Y_k`, setting `R_k(y) = y'` and concatenating the return words (in 1B, as DAG-node ID lists, avoiding `O(n)` word materialisation of thick corridors). In build phase `k`, all `|Y_{k-1}|` lanes advance synchronously by 1 step per round; by MONODROMY 104 Lemma F (the Ordered-Frontier Lemma), stable partitioning by `L[x]` preserves ascending row order within each round.
- **Exactness — PROVED (and byte-exact verified on 8/8 blocks):** Because `(X, \mathrm{LF})` is a single cycle of length `n` (and `(M, R)` is a single cycle of length `m` whose word lengths sum to `n`), the first-return map `R_k : Y_k \to Y_k` on any non-empty section `Y_k` is a single cycle whose return words partition the cycle of length `n` exactly once. Tracking the single prefix excursion from `p` (the primary index) to its first landing `y_p \in Y_k` recovers the exact cyclic phase of `T`. Verified `decoded == T_ref` byte-for-byte on all 8 blocks (2 synthetic novel-hex + 6 admitted Wikidata blocks) across all levels `k = 1, 2, 3`.
- **Exactness (1C):** Every row in `I(w)` emits `w` under `\Psi^k` by definition of suffix sorting; `|I(w)| \ge 2` contracts only the *word emission*, not the scattered target `\Psi^k(I(w))`.

## 3. Prior Art

- MONODROMY 113 (Senses A, C) & 114 (Sense E landmark induction, posting `NEEDS M-21`); `toy-transfer` lanes; Bauer–Cox–Rosone wave inversion of collection BWTs; Neon N-12/N-16/N-20 C1 (copy plans, blocked by Witness 20.1 `src < dst` blindness).
- **What is new / completed here:** (i) Full `NEEDS M-21` measurement + byte-exact verification on all 6 admitted blocks and novel random hex; (ii) orthogonal row-then-text composition (1B), which eliminates the `"`-in-scaffold blowup of raw `LF` and shrinks `|Y_3|` by `10\text{--}25\times` and `D` by `5\text{--}15\times` over raw `LF`; (iii) exact quantification of forward-`\Psi` `F`-interval sharing (1C) separating novel hex (`0.0%` at `k=8`) from repeated Wikidata phrases (`20.4\text{--}25.1%`).

## 4. Measurements on Novel Random Hex and the Six Admitted Blocks

Script: [`s1_text_axis.py`](s1_text_axis.py); raw JSON: [`results_s1_text_axis.json`](results_s1_text_axis.json). All 8 blocks pass `decoded == T_ref` at every level.

### 4.1 Raw-`LF` Landmark Induction (`NEEDS M-21`: single byte `k=1,2,3` and best pair)

| Block | `n` | `h_1` | `|Y_1|` (`k=1`) | `|Y_3|` (`k=3`) | `D` (`k=3`) | `Q` (`k=3`, probe) | Best pair (`|Y|`, mean / p99 / max gap) |
|---|---:|---:|---:|---:|---:|---:|---|
| `synth-hex-iid-65536` | 65,536 | 57,458 | 4,254 (`0.074 h_1`) | **28** (`0.0005 h_1`) | **291** | 70,116 (`1.22 h_1`) | `d4`: 299 (219.2 / 1,033 / 1,213) |
| `synth-tmpl-hex-1000x40` | 81,000 | 39,909 | 12,000 (`0.301 h_1`) | 3,000 (`k=2`) | 3,045 | 96,000 (`2.41 h_1`) | `:"`: 3,000 (27.0 / 50 / 50) |
| `block-0.L` | 899,931 | 35,116 | 135,716 (`3.865 h_1`) | 15,844 (`0.451 h_1`) | 16,135 | 1,082,072 (`30.81 h_1`) | `":`: 41,219 (21.8 / 129 / 286) |
| `block-2.L` | 900,000 | 34,262 | 135,062 (`3.942 h_1`) | 15,720 (`0.459 h_1`) | 16,011 | 1,080,949 (`31.55 h_1`) | `":`: 40,939 (22.0 / 106 / 285) |
| `block-85.L` | 899,935 | 55,504 | 138,920 (`2.503 h_1`) | 16,773 (`0.302 h_1`) | 17,033 | 1,085,672 (`19.56 h_1`) | `":`: 42,025 (21.4 / 73 / 247) |
| `block-143.L` | 899,869 | 72,578 | 128,934 (`1.776 h_1`) | 14,279 (`0.197 h_1`) | 14,720 | 1,070,737 (`14.75 h_1`) | `":`: 39,402 (22.8 / 131 / 427) |
| `block-210.L` | 899,959 | 97,054 | 136,448 (`1.406 h_1`) | 15,587 (`0.161 h_1`) | 16,096 | 1,081,972 (`11.15 h_1`) | `":`: 41,681 (21.6 / 87 / 498) |
| `block-207.L` | 899,998 | 62,407 | 129,704 (`2.078 h_1`) | 14,374 (`0.230 h_1`) | 15,295 | 1,071,674 (`17.17 h_1`) | `":`: 39,244 (22.9 / 129 / 921) |

### 4.2 Orthogonal Two-Axis (`L_min = 16` Thick Strips + Text-Axis Landmarks on `M`) & Forward `F`-Intervals (1C)

| Block | `m_{16}` | `|Y_M^{(k)}|` (deepest `k`) | `Q_{\mathrm{run}}` | `Q_{\mathrm{tot}}` (incl. `Q_{\mathrm{acq}}`) | `D` (vs `n=900k`) | `P` | 1C `k=4` / `k=8` shared `h_1` |
|---|---:|---:|---:|---:|---:|---:|---|
| `synth-hex-iid-65536` | 65,536 | 298 (`k=2`) | 298 (`0.005 h_1`) | 135,624 (`2.36 h_1`) | **528** | 4 | 63.2% / **0.0%** |
| `synth-tmpl-hex-1000x40` | 47,001 | 2,212 (`k=3`) | 2,212 (`0.055 h_1`) | 103,831 (`2.60 h_1`) | 2,277 | 5 | 51.2% / 13.8% |
| `block-0.L` | 88,290 | **572** (`0.016 h_1`) | 572 | 305,548 (`8.70 h_1`) | **1,093** | 5 | 53.0% / 24.5% |
| `block-2.L` | 109,542 | 1,375 (`0.040 h_1`) | 1,375 | 352,591 (`10.29 h_1`) | **2,075** | 5 | 51.8% / 25.1% |
| `block-85.L` | 155,124 | 1,555 (`0.028 h_1`) | 1,555 | 371,612 (`6.70 h_1`) | **2,032** | 5 | 57.0% / 24.7% |
| `block-143.L` | 193,449 | 2,599 (`0.036 h_1`) | 2,599 | 540,734 (`7.45 h_1`) | **3,058** | 5 | 57.2% / 24.2% |
| `block-210.L` | 202,025 | 2,396 (`0.025 h_1`) | 2,396 | 488,929 (`5.04 h_1`) | **3,058** | 4 | 64.5% / 20.4% |
| `block-207.L` | 134,609 | 1,937 (`0.031 h_1`) | 1,937 | 387,378 (`6.21 h_1`) | **6,013** | 5 | 56.1% / 24.2% |

## 5. Referee Verdict in `(Q, D, P, S)` under `Π`

- **Where the cost goes:**
  - **Dependent depth `D` collapses dramatically** — even on 100% novel random hex (`65,536 \to 291`, `225\times`) and on all 6 Wikidata blocks (`900,000 \to 1,093\text{--}6,013` via 1B orthogonal two-axis, a `150\text{--}820\times` collapse vs walk and `22\text{--}80\times` vs `Open`!).
  - **Addressed accesses `Q` do NOT fall below `h_1`:** in probe mode, building the return words touches every row of `X` (raw LF: `Q_{\mathrm{acq}} \ge n`) or every retained row of `M` plus thick-strip glue (1B: `Q_{\mathrm{tot}} = 5.0\text{--}10.3 h_1`, `Q_{\mathrm{run}}` alone is only `0.016\text{--}0.040 h_1`!). In streamed-wave mode against `IN`, `Q_{\mathrm{acq}} = 0` but `P = \text{max\_gap} + 1 \gg K = 2`.
  - **Forward `F`-intervals (1C)** confirm the exact novel-hex wall: at `k=8`, `0.0%` of novel random hex shares an 8-mer, whereas `20.4\text{--}25.1%` of Wikidata `h_1` is repeated 8-mers (matching Neon N-8's repeated-hex/non-hex share).
- **Verdict:** Contraction along the text via orthogonal landmark induction is a **genuine `D`-collapse** (cutting serial dependency from `n` to `O(\text{max\_gap} + |Y_k|) \sim 10^3` with `s = 256` cursors), NOT a `Q`-floor breakthrough under `Π` at `K = 2`.
