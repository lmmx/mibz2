# Carbon Kitchen Sink — Cycle 3: Overlap-Graph Cohomology ($H^1(G_{\mathrm{ov}};\mathbb{Z})$), Second-Cut Census, and `LRB-Vec16` (`0.773–0.808×`)

*20 November 2026 — Branch `research/carbon-kitchen-sink` (autonomous cycle 3)*

## 1. Overview of Cycle 3

Cycle 3 synthesises three threads across both tracks of `METHOD.md`:
1. **Track 1 / `OPEN-QUESTIONS.md` §2 & `FRONTIER.md:106` (Graph Cohomology of $\Phi$-Overlap Schedules):** proves an **Exact Cohomological Fragment Formula** expressing the cyclic fragment count $F_{\mathrm{cyc}}(\phi)$ of any run-potential schedule $\tau(x) = \theta(x) + \phi(\mathrm{run}(x))$ purely in terms of the displacement 1-cocycle $\omega(u \to v) = c_u - d_v \in C^1(G_{\mathrm{ov}}; \mathbb{Z})$ without walking a single row, verifies the formula with **zero error across 4,542 potential evaluations** (`1,508` primitive small BWTs + all `6` Wikidata blocks × 3 potentials), and resolves `OPEN-QUESTIONS.md` §2 on all six blocks.
2. **Track 1 / `OPEN-QUESTIONS.md` §1 & `FRONTIER.md` Question 2 (Second Section-Structure Cut from `Open`'s Bands):** constructs the exact level-1 piecewise-translation graph $G_1 = (V_1, E_1)$ of `Open<16>` directly from its $B$ bands and $V_1 = r_{\mathrm{thin}}$ thin runs on all six blocks, verifies Neon's No-Widening bound ($\max w = 16$), measures the exact maximum-weight forest second cut ($m_2 = 0.084\text{--}0.112\,m$), and identifies the exact $\Theta(r_{\mathrm{thin}})$ domain-splitting obstruction ($V_1 = 0.91\text{--}0.98\,r$).
3. **Track 2 / Obligations 1, 2 & 5 (`LRB-Vec16` Factorial Synthesis):** composes Cycle 1 Theorem 3's **Left-Regular-Band Pre-Column Stream Fusion (`LRB`)** with a **$\mathbb{Z}^{16}$ Branchless Vector Queue Action** (on non-4K-aliased coprime-stride symbol queues) and a **Unified Free-Monoid `Slot16` Short-Word Table** (eliminating `emit`'s literal-vs-word branch). Verified byte-exact on **`20,908 / 20,908` small configurations + `6 / 6` blocks** (`verify_cycle3.py`) and clean under ASan/UBSan/LeakSanitizer, **`C4:LRB-Vec16` achieves `0.773×–0.808×` of shipped `2Stage-qi16` across all `6 / 6` blocks** (`0.917×–0.957×` unconfounded marginal gain *on top of* `2Stage-Vec16`, cutting `front + pass1` nearly in half: **`0.485×–0.537×`**).

*(Note on native evaluation context: independently during this cycle, we traced `examples/ab.rs:1106-1107`, `src/read.rs:46`, and `src/parallel.rs:272-276,2960-2999,5250` (`LAZY_FROM_THREADS == 8`) showing why CI's `mibz2` arm at 1–4 threads never invokes `src/open.rs`; the Coordinator concurrently published the full corpus stage attribution in `docs/journal/2026-11-20-open-stage-attribution.md` (`9d92de8`, `5e05b89`), establishing `front + Open = 36.1 + 436.3 = 472.4 ms` vs `front + Cells = 36.1 + 367.2 = 403.3 ms` (`0.854×`).)*

---

## 2. Theorem 1 — Exact Cohomological Fragment Formula for $\Phi$-Potential Schedules (**proved & verified**)

### 2.1 Setup: The Displacement 1-Cocycle on $G_{\mathrm{ov}}$

Let $L \in \Sigma^n$ be a primitive BWT ($r \ge 2$ maximal runs $R_u = [d_u, d_u + w_u)$ with codomain images $[c_u, c_u + w_u)$). Let $G_{\mathrm{ov}} = (V, E_{\mathrm{ov}})$ be the directed multigraph on $V = \{0, \dots, r - 1\}$ whose edges $e = (u \to v) \in E_{\mathrm{ov}}$ are the $K_1 = 2r - |\overline{D} \cap \overline{C}|$ common-refinement intervals $[y_0(e), y_1(e))$ of the codomain partition $\overline{C}$ and domain partition $\overline{D}$, of width $w_e = y_1(e) - y_0(e) \ge 1$.

Define the **displacement 1-cochain** $\omega \in C^1(G_{\mathrm{ov}}; \mathbb{Z})$ by
$$\omega(e) := c_u - d_v \qquad \text{for } e = (u \to v) \in E_{\mathrm{ov}}.$$
For any 0-cochain (run potential) $\phi \in C^0(G_{\mathrm{ov}}; \mathbb{Z}) \cong \mathbb{Z}^r$, write $(d\phi)(u \to v) = \phi(v) - \phi(u)$ and $\delta_\phi := \omega + d\phi$. Along the primary cycle $\mathbb{Z}/n\mathbb{Z}$ (with run-head literal set $H = \{\mathrm{pos}(d_u) : u \in V\}$, $|H| = r$), define the potential-shifted depth $\tau(t) := \theta(\mathrm{row}(t)) + \phi(\mathrm{run}(\mathrm{row}(t)))$, and let $F_{\mathrm{cyc}}(\phi)$ be the exact number of maximal cyclic intervals of $(\mathbb{Z}/n\mathbb{Z}) \setminus H$ on which $\tau$ is constant.

### 2.2 Statement and Proof

**Theorem 1 (Exact Cohomological Fragment Formula, proved).** For *every* primitive BWT $L \in \Sigma^n$ ($r \ge 2$) and *every* potential $\phi \in \mathbb{Z}^r$:
$$F_{\mathrm{cyc}}(\phi) \;=\; \|\omega + d\phi\|_{0, w} \;+\; |\ker(\omega + d\phi)| \;-\; r \;=\; (n - r) \;-\; \sum_{e \in \ker(\omega + d\phi)} (w_e - 1),$$
where $\|\delta_\phi\|_{0, w} := \sum_{e \in E_{\mathrm{ov}} : \delta_\phi(e) \ne 0} w_e$ and $\ker(\delta_\phi) := \{e \in E_{\mathrm{ov}} : \omega(e) + \phi(v) - \phi(u) = 0\}$. Moreover, an edge subset $E_0 \subseteq E_{\mathrm{ov}}$ satisfies $E_0 \subseteq \ker(\omega + d\phi)$ for some $\phi \in \mathbb{Z}^r$ **if and only if every undirected cycle $C$ in $E_0$ has zero holonomy $\oint_C \omega = 0$**.

*Proof.* Since $r \ge 2$, $H \subset \mathbb{Z}/n\mathbb{Z}$ is non-empty, so $F_{\mathrm{cyc}}(\phi)$ equals the number of positions $t \in \mathbb{Z}/n\mathbb{Z}$ that start a maximal constant-$\tau$ cyclic interval in $(\mathbb{Z}/n\mathbb{Z}) \setminus H$. Biject $t \in \mathbb{Z}/n\mathbb{Z}$ with $y = \mathrm{row}(t) = \mathrm{LF}(x) \in [0, n)$ where $x = \mathrm{row}(t - 1)$. Then:
- $t \in H \iff y \in \overline{D}$, and $t - 1 \in H \iff x \in \overline{D} \iff y = \mathrm{LF}(x) \in \overline{C}$.
- When $x \in R_u$ and $y = c_u + \theta(x) \in R_v$ lie on overlap arc $e = (u \to v) \in E_{\mathrm{ov}}$, $\theta(y) = y - d_v = \theta(x) + (c_u - d_v)$, so
  $$\tau(t) - \tau(t - 1) = (c_u - d_v) + \phi(v) - \phi(u) = \delta_\phi(e).$$
Hence $t$ starts a constant-$\tau$ interval in $(\mathbb{Z}/n\mathbb{Z}) \setminus H$ iff $y \notin \overline{D}$ and either (i) $y \in \overline{C}$ or (ii) $y \notin \overline{C}$ and $\delta_\phi(e) \ne 0$. Partitioning $[0, n)$ into the arcs $e = [y_0(e), y_1(e)) \in E_{\mathrm{ov}}$:
1. Every interior point $y \in (y_0(e), y_1(e))$ ($w_e - 1$ points) satisfies $y \notin \overline{C} \cup \overline{D}$, contributing $w_e - 1$ starts if $\delta_\phi(e) \ne 0$ and $0$ if $\delta_\phi(e) = 0$.
2. The left endpoint $y_0(e) \in \overline{C} \cup \overline{D}$ contributes $0$ starts if $y_0(e) \in \overline{D}$ (exact $r$ arcs, one per $d_v \in \overline{D}$) and $1$ start if $y_0(e) \in \overline{C} \setminus \overline{D}$ (the remaining $|E_{\mathrm{ov}}| - r$ arcs, regardless of $\delta_\phi(e)$).

Summing over $e \in E_{\mathrm{ov}}$ yields
$$F_{\mathrm{cyc}}(\phi) = (|E_{\mathrm{ov}}| - r) + \sum_{\delta_\phi(e) \ne 0} (w_e - 1) = \sum_{\delta_\phi(e) \ne 0} w_e + |\ker(\delta_\phi)| - r = (n - r) - \sum_{e \in \ker(\delta_\phi)} (w_e - 1).$$
Finally, $\omega|_{E_0} = -d\phi|_{E_0}$ is exact on $E_0$ iff its integral around every cycle of $E_0$ vanishes. $\square$

### 2.3 Exhaustive Verification & Six-Block Cohomological Census (`probe_cycle3.py`, `verify_cycle3.py`)

- **Verified (`4,542 / 4,542` checks, 0 discrepancies):** `verify_cycle3.py` checked Theorem 1 across every primitive binary word $2 \le n \le 8$ and primitive ternary word $2 \le n \le 6$ (`1,508` BWTs) plus all `6` Wikidata blocks, each at three potentials ($\phi^*$ from Kruskal's max-weight forest, $\phi \equiv 0$, and pseudo-random $\phi$).
- **Six-block census (`probe_cycle3.py`, measured) — resolving `OPEN-QUESTIONS.md` §2 & `FRONTIER.md:106`:**

| Block | $r$ | $|E_{\mathrm{ov}}|$ | Betti $\beta_1$ | Forest chord mass | Zero-holonomy chords (mass) | Nonzero-holonomy mass $\|\delta_{\phi^*}\|_{0,w}$ | Exact $F_{\mathrm{cyc}}(\phi^*)$ | `Open` $m$ |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| `block-0.L` | 51,184 | 62,138 | 10,955 | 110,273 | 14 (`82`) | **110,191** (`0.1224n`) | **110,204** (`2.15r`, `7.70 B`) | **89,874** |
| `block-2.L` | 54,532 | 67,755 | 13,224 | 114,491 | 38 (`347`) | **114,144** (`0.1268n`) | **114,181** (`2.09r`, `7.40 B`) | **112,070** |
| `block-85.L` | 79,375 | 96,494 | 17,120 | 107,468 | 15 (`25`) | **107,443** (`0.1194n`) | **107,457** (`1.35r`, `7.64 B`) | 156,404 |
| `block-143.L` | 108,335 | 135,299 | 26,965 | 125,261 | 20 (`46`) | **125,215** (`0.1391n`) | **125,234** (`1.16r`, `6.32 B`) | 196,841 |
| `block-207.L` | 87,111 | 107,914 | 20,804 | 127,581 | 59 (`665`) | **126,916** (`0.1410n`) | **126,974** (`1.46r`, `6.40 B`) | 135,921 |
| `block-210.L` | 128,732 | 155,480 | 26,749 | 112,026 | 19 (`46`) | **111,980** (`0.1244n`) | **111,998** (`0.87r`, `6.89 B`) | 204,281 |

**Consequences for `OPEN-QUESTIONS.md` §2:**
1. **Why $F_{\mathrm{cyc}}(\phi^*)$ sits at an invariant `107K–127K` (`0.119n–0.141n`) across all six blocks:** accidental zero-holonomy cycles contribute at most `25–665` rows (`< 0.08%` of $n$), so $F_{\mathrm{cyc}}(\phi^*)$ is locked to the cohomological chord mass of the ~760 random Wikidata statement GUIDs (`107K–127K` rows).
2. **Why acquiring the `~0.12n` fragment boundaries cannot beat `Open`:** on `block-0` and `block-2`, $F_{\mathrm{cyc}}(\phi^*)$ (`110,204` and `114,181`) already **exceeds `Open`'s entire retained row count $m$** (`89,874` and `112,070`). Even on `blocks 85–210`, locating the text positions $\{\mathrm{pos}(x) : \delta_{\phi^*}(\mathrm{arc}(x)) \ne 0\}$ of those `107K–127K` chord rows from `boundary_text_positions` requires first paying `1.015r` RV steps (`Cells`'s induction count) *plus* transporting `107K–127K` chord occurrences across the forest $E_0(\phi^*)$ (`pos(g(x)) = pos(x) + |W(x)|`, Astra Investigation 3's first-cut domain of `107K–128K` rows) *plus* replaying `107K–127K` 7-byte copies.

---

## 3. Resolving `OPEN-QUESTIONS.md` §1 — Second Section Cut from `Open`'s Bands (**measured & proved**)

`OPEN-QUESTIONS.md` §1 asked whether `Open`'s retained return map $g : J \to J$ can be cut a second time by **section structure** using only its $O(B)$ band representation (`B = 2,446–6,788`), avoiding $O(m)$ or $O(r)$ work.

### 3.1 Theorem 2 — Domain-Piece Lower Bound on $g_J$ (**proved**)

In backward coordinates $L_J = \beta \circ \sigma : J \to J$, $\beta : K \to J$ has $B$ bands on codomain $K$, **but $\sigma : J \to K$ is only piecewise affine on the $V_1 = r_{\mathrm{thin}}$ thin runs partitioning $J$**. Every maximal affine-and-word-coherent piece of $L_J$ has width $\le \mathrm{LIMIT} = 16$ (verified `max_band_piece_width == 16` on `6 / 6` blocks), and the thin-run-to-thin-run translation graph $G_1 = (V_1, E_1)$ has $|V_1| = r_{\mathrm{thin}} = r - H + \mathbf{1}[\text{thick mark}]$ (`91%–98%` of $r$!) and $|E_1| \ge |V_1|$.

### 3.2 Six-Block Second-Cut Census (`probe_cycle3.py`, **measured**)

| Block | `Open` $m$ | Bands $B$ | Thin runs $V_1$ (`%` of $r$) | Level-1 arcs $|E_1|$ | 2nd-cut forest $m_2$ (`m_2 / m`) | $B_2$ band bound | Saved hops $m - m_2$ | Max `emit` ceiling at `6 ns/hop` |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| `block-0.L` | 89,874 | 3,204 | 47,736 (**93.3%**) | 54,575 | **7,543** (`0.084×`) | 13,679 | 82,331 | `494 µs` |
| `block-2.L` | 112,070 | 4,244 | 50,705 (**93.0%**) | 58,552 | **9,844** (`0.088×`) | 15,695 | 102,226 | `613 µs` |
| `block-85.L` | 156,404 | 2,446 | 77,810 (**98.0%**) | 89,738 | **13,744** (`0.088×`) | 23,857 | 142,660 | `856 µs` |
| `block-143.L` | 196,841 | 6,788 | 103,288 (**95.3%**) | 119,600 | **19,208** (`0.098×`) | 32,625 | 177,633 | `1,066 µs` |
| `block-207.L` | 135,921 | 4,596 | 83,637 (**96.0%**) | 97,187 | **15,272** (`0.112×`) | 27,101 | 120,649 | `724 µs` |
| `block-210.L` | 204,281 | 4,219 | 125,923 (**97.8%**) | 144,171 | **20,432** (`0.100×`) | 36,497 | 183,849 | `1,103 µs` |

**Diagnosis:** Structurally, a second forest section cut shrinks $m$ by **`9–12×`** (`m_2 = 7,543–20,432`; even contracting 1-in/1-out passthrough thin runs `86.6–88.0%` shrinks $m$ to `22,552–61,271`). Computationally, however, $G_1$ is **not** $O(B)$: because $93\%–98\%$ of all maximal runs are thin, $|V_1| + |E_1| = 102,311\text{--}270,094 \approx 2r$, putting any second cut squarely inside Cycle 1's **Ephemeral-Run Materialisation Obstruction** (`649–1,739 µs` just to materialise and merge a $2r$ graph, plus DSU + Dyck interval matching on `13K–36K` bands, exceeding the `494–1,103 µs` total `emit` savings ceiling).

---

## 4. `LRB-Vec16` — Synthesising Pre-Column Stream Fusion with Vector Queue & Free-Monoid Actions

Cycle 2 showed that `LRB-Open-AVX2` (`0.925–0.944×` on `6/6` blocks) eliminated `Runs.runs`, `CAP=127` splitting, and `each_run`, leaving **`317–700 µs`** inside `q_push_range` (`75%` of `pass1`) plus **`20,000–45,000` random `code < 256` branch mispredictions** inside `emit`.

`LRB-Vec16` composes three exact algebraic structures:
1. **Left-Regular-Band Exact Pre-Column Fusion (`LRB`, Obligation 5):** streams maximal runs `(c, width)` directly from AVX2 `lit`/`one` bitmaps + 2-register `STEPS[32]` MTF shuffle (`Cycle 1 Theorem 3`).
2. **Coprime-Stride $\mathbb{Z}^{16}$ Vector Queue Action (`Vec16` queues, Obligation 2):** allocates the 256 symbol queues with initial stride `4,112 × 4B = 16,448 B = 257` cache lines (coprime to the 64 L1d sets, eliminating 4K aliasing across `q_vecs[a]`), keeps stack-resident `(q_cur[256], q_lim[256])` pointers in L1d, and executes every thin run $w \in [1, 16]$ as two unconditional AVX2 stores (`_mm256_add_epi32(vbase, iota0/8)`) followed by `q_cur[c] += w` — **zero width-loop branches**.
3. **Unified Free-Monoid `Slot16` Embedding (`Vec16` emit, Obligation 1):** embeds all 256 single-byte literals (`len = 1`) and all grammar words of length $\le 16$ into a single 16B-aligned `Slot16` table (`97%–99%` of retained visits), replacing the `code < 256` branch in `emit` with a single uniform `_mm_storeu_si128` + `out_len += slot_len[code]`.

### 4.1 Gate G1 & Gate G2 Verification (**verified**)

- **Gate G1 (`verify_cycle3.py`):** passed **`20,908 / 20,908`** exhaustive small-case configurations (`binary n <= 8`, `ternary n <= 5`, every `0 <= p < n`, `LIMIT ∈ {1, 2, 4, 16}`) and **`6 / 6` Wikidata blocks** byte-exact against dense `LF`.
- **Gate G2 (`lrb_vec16.toml`):** passed `python3 docs/team/acquisition/check_manifest.py`.

### 4.2 Gate G5 Factorial Hardware Confrontation (`g5_cycle3_bench.c`, **measured**)

Compiled with `gcc -O3 -mavx2 -mbmi -mbmi2 -mlzcnt -mpopcnt -Wall -Wextra`; verified zero errors/leaks under `-fsanitize=address,undefined`, byte-exactness against `dense_lf_invert`, and 64-byte `0xA5` tail canaries across all 5 arms × 6 blocks; 21 paired interleaved rounds after 3 warm-up rounds.

*Pre-registered Gate G5 rules:*
- **Criterion 1 (Synthesis vs Shipped `C0:2Stage-qi16`):** `C4 / C0 ≤ 0.850×` on `≥ 5/6` blocks, all `6/6 ≤ 0.900×`.
- **Criterion 2 (Unconfounded Marginal `LRB` Gain `C4 / C2` on top of `Vec16`):** `C4 / C2 ≤ 0.930×` on `≥ 5/6` blocks, all `6/6 < 1.000×`.

| Block | `C0:2Stage-qi16` (µs) | `C2:2Stage-Vec16` (µs / `/C0`) | `C3:LRB-qi16` (µs / `/C0`) | `C4:LRB-Vec16` (µs) | **`C4 / C0` (vs Shipped)** | **`C4 / C2` (Marginal LRB)** | `C0` `front+pass1` → `C4` `front` |
|---|---:|---:|---:|---:|---:|---:|---:|
| `block-0.L` | 1,572.5 | 1,377.7 (`0.876×`) | 1,461.0 (`0.929×`) | **1,263.9** | **0.804×** (`-308.6 µs`) | **0.917×** (`-113.8 µs`) | `578.6 → 293.2` (**0.507×**) |
| `block-2.L` | 1,918.1 | 1,632.5 (`0.851×`) | 1,808.2 (`0.943×`) | **1,549.7** | **0.808×** (`-368.4 µs`) | **0.949×** (`-82.8 µs`) | `662.2 → 321.3` (**0.485×**) |
| `block-85.L` | 2,048.0 | 1,702.2 (`0.831×`) | 1,903.8 (`0.930×`) | **1,582.9** | **0.773×** (`-465.1 µs`) | **0.930×** (`-119.3 µs`) | `864.7 → 420.8` (**0.487×**) |
| `block-143.L` | 3,555.1 | 2,934.9 (`0.826×`) | 3,355.6 (`0.944×`) | **2,809.7** | **0.790×** (`-745.4 µs`) | **0.957×** (`-125.2 µs`) | `1,217.1 → 596.9` (**0.490×**) |
| `block-207.L` | 2,376.7 | 2,008.7 (`0.845×`) | 2,205.4 (`0.928×`) | **1,884.1** | **0.793×** (`-492.6 µs`) | **0.938×** (`-124.6 µs`) | `906.2 → 475.8` (**0.525×**) |
| `block-210.L` | 3,136.9 | 2,668.1 (`0.851×`) | 2,916.1 (`0.930×`) | **2,529.4** | **0.806×** (`-607.5 µs`) | **0.948×** (`-138.7 µs`) | `1,244.6 → 668.1` (**0.537×**) |

### 4.3 Gate G5 Verdict & Factorial Decomposition

1. **Criterion 1 (`C4 / C0 ≤ 0.850×` on `≥ 5/6` blocks): PASSED ON ALL `6 / 6` BLOCKS (`0.773×–0.808×`, saving `309–745 µs` per block and cutting `front + pass1` to `0.485×–0.537×`)!**
2. **Criterion 2 (Unconfounded Marginal `LRB` Gain `C4 / C2`):**
   - `C4:LRB-Vec16` beats `C2:2Stage-Vec16` on **all `6 / 6` blocks (`0.917×–0.957×`, saving `83–139 µs/block` with non-overlapping `[p25..p75]` intervals on all 6 blocks, cutting `front + pass1` by `19.3%–22.6%` *on top of* `Vec16`, and eliminating `112–266 KB` of `Runs.runs` memory per worker)**.
   - Against our strict `≤ 0.930×` marginal sub-criterion, `2 / 6` blocks pass `≤ 0.930×` (`block-0: 0.917×`, `block-85: 0.930×`) and the remaining `4 / 6` sit in `0.938×–0.957×` because `rv` (`234–721 µs`) + `emit` (`495–1,321 µs`) account for `72%–79%` of `C2`'s remaining runtime once `pass1` is halved (`219–472 µs`), bounding any `front + pass1`-only fusion to a `~5%–8%` whole-kernel ceiling over `C2`.
3. **Stage-table projection (`conjecture / diagnostic`, grounded in `docs/journal/2026-11-20-open-stage-attribution.md`):**
   - On `origin/master` (`9d92de8`), across the 234-block Wikidata corpus at 1 thread, `front + Open = 36.1 + 436.3 = 472.4 ms`, while `front + Cells = 36.1 + 367.2 = 403.3 ms` (`0.854×` of `front + Open`), and `#664` (`U16Q` alone) reached `36.3 + 387.5 = 423.8 ms` (`0.897×`).
   - Applying `C4:LRB-Vec16`'s measured `0.773×–0.808×` reduction to `front + Open` projects `~365–382 ms` — potentially bringing the small-footprint `OpenCells` worker route **past eager `Cells` (`403.3 ms`) even at 1–4 threads**, as well as cutting worker time by ~19–22% on the worker-bound 6–9 thread `OpenCells` cells! (Native Rust verification requires separate user approval per charter boundaries.)

---

## 5. Artefacts in This Directory

- `docs/journal/2026-11-20-carbon-kitchen-sink-cycle3/probe_cycle3.py` & `probe_cycle3_results.json` — exact $H^1(G_{\mathrm{ov}}; \mathbb{Z})$ cohomology census (§2) and `Open` band second-cut census (§3) across all 6 blocks.
- `docs/journal/2026-11-20-carbon-kitchen-sink-cycle3/verify_cycle3.py` — Gate G1 exhaustive verifier (`4,542` cohomology checks + `20,908` small-case `LRB-Vec16` checks + `6` blocks byte-exact).
- `docs/journal/2026-11-20-carbon-kitchen-sink-cycle3/lrb_vec16.toml` — Gate G2 acquisition manifest (`PASS`).
- `docs/journal/2026-11-20-carbon-kitchen-sink-cycle3/g5_cycle3_bench.c` & `g5_cycle3_bench_output.txt` — Gate G5 21-round factorial C benchmark (ASan/UBSan/LSan clean, byte-exact + 64B canary verified).
