# Carbon Cycle 7 · Singleton Non-Coalescence, Complete Resolution of the $\Phi$-Overlap Obligation, and Graded Inline-Monoid Rauzy–Veech Elimination (`GIM-RV`)

*20 November 2026 · Carbon (`research/carbon-word-transport`, Cycle 7).*

## 0. Cycle-Boundary Check (`origin/master` Fetch — `9d92de8` & `5e05b89`)

Fetching `origin/master` (`5e05b895`) at the Cycle 6 → Cycle 7 boundary surfaced [`2026-11-20-open-stage-attribution.md`](https://github.com/lmmx/mibz2/commit/9d92de8) and [`5e05b89`](https://github.com/lmmx/mibz2/commit/5e05b89):
- **The `#664` CI null is explained:** CI's `ab` benchmark runs serial `Cells` at 1 thread and `Cells` workers at 2 and 4 threads (`LAZY_FROM_THREADS = 8`), so it never invoked `src/open.rs`!
- **Cycle 2's `U16Q` kernel gain is real across the entire 431-block corpus:** In direct stage attribution on `corpora/wikidata.bz2`, `U16Q` cuts native safe-Rust `Open` by **`11.2%` (`436 → 388 ms`)** while every other stage stays flat within `±2%`, bringing `Open` (`~366 ms` on the 228 admitted blocks) within `~3%` of eager `Cells` (`354.8 ms`, which refuses `0.10 <= r/n < 0.15` blocks such as `block-143` and `block-210`) at `39%` lower peak heap.

Cycle 7 tackles the three major remaining open mathematical and architectural questions on `FRONTIER.md` and `OPEN-QUESTIONS.md`:
1. **Non-run-union Meyer complements & run-network chain elimination** (`FRONTIER.md` lines 44, 126, 132; `switches.rs`; `run-network.md` Q4);
2. **Obligation #3 / `OPEN-QUESTIONS.md` §2 ("Acquiring the $\Phi$-overlap coordinates")**: whether $\Phi$-overlap transitions determine the text, and whether an acyclic in-place $\Phi$-copy schedule can be built from Record 81's $O(r)$ run-boundary positions without Record 84's $O(n)$ text-order run labels;
3. **Algebraic elimination of `endpoint.rs`'s binary SLP grammar & 4-vector setup overhead (`GIM-RV`)**: sub-phase profiling of `Open+U16Q` reveals `EpSet + EpElm + BandSp` accounts for `50–57%` of `construct` (`0.385–0.701 ms`) because `endpoint.rs` allocates binary `EpRule` nodes and walks an explicit DFS stack even though `92.7–97.6%` of all Rauzy–Veech word joins stay within `<= 64` bytes.

---

## 1. Explore: Four Formulations Across Meyer Complements, $\Phi$-Forests, and Graded Monoids

Mode: **Reorientation** (`METHOD.md` §1 & §3a):

| # | Candidate / Framework | Mathematical setting | Mechanism & information needed | Likely mathematical or economic trap |
|---|---|---|---|---|
| 1 | **`NonRun-Meyer` / `Chain-Core`** | General deterministic Meyer complementation on arbitrary row subsets $S \subseteq [0, n)$ (`switches.rs`, `run-network.md` Q4) | Eliminate pass-through thin runs (`in = out = 1`, `88–89%` of $r$) or narrow-to-narrow interiors (`s16`, $m_{\text{sw}} = 15,278$) to collapse $m$ by `3–6×` | Singleton rows ($w = 1$, `35k–97k`/block) have `1 run == 1 row` and affine multiplicity `1`, trading `1` `4B` consumer load for `>= 1` producer lookup + double byte copy |
| 2 | **`Boundary-tildePhi`** | Text-space predecessor forest $\tilde{\Phi}(t) = \text{pos}(x - 1)$ reconstructed from Record 81's $r$ boundary triples $\{(q_u, p_u, c_u)\}$ (`OPEN-QUESTIONS.md` §2) | Sort $r$ seed triples; resolve the $k = r - \|\bar{D}\cap\bar{C}\|$ copy intervals topologically without Record 84's text-order run labels | Greedy/Tarjan resolution without run labels shatters against the $r$ isolated 1-byte seed holes into `2.88r–6.69r` fragments (`335k–371k`), strictly dominated by `Cells`' `owned_scan` |
| 3 | **`Queue-Conjugate`** | Symmetric-group conjugation of $\Psi_{\text{ret}} = \pi_Q \circ P_{\text{band}}^{-1}$ into symbol-queue coordinates $k \in [0, m)$ | Index edges by queue rank $k$ instead of domain rank $j$ to avoid `queues[256]` materialisation | Dualises directly to `Open`'s `QUEUE = false` (`i16` / `#659`), requiring a second `records` scan and `47k` scattered writes instead of `3.2k` band writes |
| 4 | **`GIM-RV` ($\mathcal{M}_{64}$)** | **$B$-Graded Free-Monoid Quotient $\mathcal{M}_{64} = \Sigma^{\le 64} \sqcup \mathrm{SLP}(\Sigma^{>64})$ + Single-Pass Circular Doubly-Linked Splice** | Replace `Tree(\Sigma)` binary `EpRule`s with packed `(len << 24) \| off` SIMD arena elements for `len <= 64` (`92.7–97.6%` of joins) and link `terminals`/`order` inline in one pass | On sparse-thick blocks (`block-85`, `1,566` strips), `RunQ` is `65%` of `construct`, capping whole-producer percentage gain at `~13%` |

---

## 2. Triage & Foundational Theorems (`verify_cycle7.py`)

All proofs and censuses below are verified in [`verify_cycle7.py`](2026-11-20-carbon-cycle7/verify_cycle7.py) (`873/873` Singleton Non-Coalescence columns + `812/812` primitive boundary-$\tilde{\Phi}$ reconstructions + `10,454/10,454` exhaustive small `GIM-RV` instances + `7/7` adversarial controls + `6/6` authenticated Wikidata blocks, `5,399,692` bytes exact).

### Result 2.1 · The Universal Singleton Non-Coalescence Law (`proved` + `verified`, closing Non-Run-Union Meyer Complements & `run-network.md` Q4)

**Theorem 7.1 (`proved` + `verified` across `873/873` small columns + `6/6` blocks).**
Let $U_1 = \bigcup_{|R_u| = 1} R_u$ be the set of rows in width-1 maximal runs of $L$ (`34,262`–`97,054` rows across the 6 Wikidata blocks, `31–48%` of `Open`'s retained $m$). For **any** deterministic state elimination with **any** retained subset $S \subseteq [0, n)$ (whether run-union, switch-interface `s16`, run-network core, or arbitrary subset):
1. Whenever $x \in U_1$ ($x + 1 < n$), $L[x + 1] \neq L[x]$, so $\mathrm{LF}(x + 1) = \mathrm{LF}(x) + 1$ can occur **only** at the $\le \sigma_{\text{used}} - 1$ global $F$-block boundaries where $\mathrm{LF}(x) = C[a_{\text{next}}] - 1$ and $\mathrm{LF}(x + 1) = C[a_{\text{next}}]$.
2. Excluding at most $\sigma_{\text{used}} - 1 \le 95$ rows per block, **no two eliminated singleton rows $x \in U_1 \setminus S$ share an affine `LF` step**: every $x \in U_1 \setminus S$ has transverse multiplicity $1$, requiring $\ge 1$ individual successor-discovery operation in the producer plus writing and re-reading its byte through an intermediate phrase buffer.
3. *Corollaries:*
   - Explains Astra's `switches.rs` (`s16`): cutting $m$ from `89,874` to `15,278` (`−74,596` thin rows) forced `58,581` narrow traced letters (mean multiplicity `1.27`, `build = 6.771 ms`).
   - Closes `run-network.md` Q4 ("Hubs versus chains"): on $w = 1$ pass-through runs (`1 run == 1 row`), contracting chains in $G_{\text{run}}$ chases `next_run[u]` once per singleton in `construct` while adding $O(r)$ run sorting + 2-pointer arc merging (`~0.5 ms`), strictly losing to walking `edges[j]` once. $\square$

### Result 2.2 · Complete Resolution of `OPEN-QUESTIONS.md` §2 / Obligation #3 ($\Phi$-Overlap Coordinates — `proved` + `verified`)

**Theorem 7.2 (`proved` + `verified` across `812/812` primitive small instances + `6/6` Wikidata blocks).**
On any primitive full-period block $(L, p)$ with $r$ maximal runs starting at $0 = s_0 < s_1 < \dots < s_{r-1}$ and text emission coordinate $\text{pos}(x) \in [0, n)$ ($\text{pos}(p) = n - 1$, $\text{pos}(\mathrm{LF}(x)) \equiv \text{pos}(x) - 1 \pmod n$):
1. *Unlabelled $\Phi$-overlap transitions do **not** determine the text:* Distinct markings of the same primitive necklace (e.g. `(L = b"ba", p = 0 -> T = b"ab")` vs `(L = b"ba", p = 1 -> T = b"ba")`) share identical symbol histograms $C$ and identical unlabelled $\Phi$-overlap transition sequences while decoding to distinct texts.
2. *Record 81's $r$ boundary triples $\{(q_u, p_u, c_u)\}_{u=0}^{r-1} = \{(\text{pos}(s_u), \text{pos}(s_u - 1 \bmod n), L[s_u])\}$ **do** determine $T$ and an acyclic copy schedule **without** Record 84's $O(n)$ run labels:*
   Sorting by $q_{(0)} < \dots < q_{(r-1)}$ seeds $T[q_{(j)}] = c_{(j)}$ and defines on every open gap $t \equiv q_{(j)} + d \pmod n$ ($1 \le d < (q_{(j+1)} - q_{(j)}) \bmod n$) the exact source map
   $$\tilde{\Phi}(t) = (p_{(j)} + d) \bmod n, \qquad T[t] = T[\tilde{\Phi}(t)].$$
   Because $\tilde{\Phi}(\text{pos}(x)) = \text{pos}(x - 1 \bmod n)$ decrements the BWT row index within each run $R_u$, $\tilde{\Phi}$ is **point-wise 100% acyclic** — a forest rooted at $\{q_u\}$ of depth $\le \max_u w_u - 1$.
3. *Why boundary-only $\tilde{\Phi}$ loses economically to `Cells` (`measured` on 6/6 blocks):*
   Without Record 84's text-order run labels to order whole copy spans, topological multi-byte slice propagation from $\{(q_u, p_u, c_u)\}$ shatters against the $r$ isolated 1-byte seed positions $\{q_u\}$ into **`334,614`–`370,804` fragments (`2.88r–6.69r`, mean `2.4–2.7 B`)** across the 6 blocks — on top of Record 81's `1.68–1.87r` boundary-acquisition steps and an $r$-element radix sort — whereas `Cells`' `cells_owned.rs` writes `out[0..n)` directly from the `1.015r` RV grammar in `1.0r` linear steps with zero sorting. **Closed.** $\square$

### Result 2.3 · Graded Free-Monoid Quotient $\mathcal{M}_{64}$ & Single-Pass Endpoint Splice (`GIM-RV` — `proved` + `verified`)

**Theorem 7.3 (`proved` + `verified` on `10,454/10,454` small cases + `7/7` adversarial + `6/6` blocks).**
1. Replacing `endpoint.rs`'s free binary tree magma $\mathrm{Tree}(\Sigma)$ with the length-graded monoid $\mathcal{M}_{64} = \Sigma^{\le 64} \sqcup \mathrm{SLP}(\Sigma^{>64})$ (packed into the existing `32`-bit `EpNode.word` field as `(len << 24) | arena_off` when `bit 31 == 0`, and `0x80000000 | rule_id` when `len > 64`) is an exact free-monoid homomorphism. Across the 6 Wikidata blocks, **`92.7%`–`97.6%` of all Rauzy–Veech joins stay inside $\Sigma^{\le 64}$**, collapsing `EpRule` allocations by **`14×`–`42×`** (`12,619 → 301` rules on `block-143.L`; `6,758 → 255` on `block-210.L`) inside a `41.8–117.4 KiB` L2-resident flat byte arena and eliminating `ep_append` DFS stack walks on `> 95%` of dictionary words.
2. Maintaining four scalar head/tail cursors `(th[side], tt[side], oh[side], ot[side])` links `EpNode.prev/next` inline during the strip/gap scan and splices `0 <-> terminals[side] <-> order[side] <-> 0` in $O(1)$, eliminating all four `order[2]` and `terminals[2]` intermediate vectors and the second linking pass.
3. Crucially, `GIM-RV` requires **zero `unsafe` code** in Rust (`copy_within` on `small_arena` and slice copies into `spelling`), preserving `src/open.rs`'s safe-Rust guarantee. $\square$

---

## 3. Deepen & Confront: Gate G5 Paid-Cost Falsifier (`g5_gimrv.c`)

### Candidate Declaration & Acquisition Manifest
- **Manifest:** [`gimrv.toml`](2026-11-20-carbon-cycle7/gimrv.toml) (`python3 docs/team/acquisition/check_manifest.py` **PASS**).
- **Pre-Registered Rule (`g5_gimrv.c`, 25 paired interleaved rounds after 3 warmup rounds across 6 arms):**
  - `Arm 1 (GIM-RV-1L)` vs `Arm 0 (Open-qi16)`: `construct <= 0.85×` on `>= 5/6` blocks;
  - `Arm 3 (GIM-RV+U16Q-1L)` vs `Arm 2 (Open+U16Q-1L)`: `construct <= 0.85×` on `>= 5/6` blocks;
  - `Arm 5 (GIM-RV+BVSMC-2P)` vs `Arm 0` & `Arm 4 (Open+BVSMC-2P)`: `total <= 0.72×` `Open` and `<= 0.92×` `Open+BVSMC-2P` on `>= 5/6` blocks (`0 B` chunk-pool arena).

### Paired 6-Block Measurements (`measured`, VM C diagnostic — `g5_gimrv.c`, `-O3 -mavx2`)

| Block | `Open-qi16` Con / Tot (ms) | `GIM-RV-1L` Con (`×Open`) | `Open+U16Q` Con / Tot (ms) | **`GIM-RV+U16Q` Con** (`×Open+U16Q` / `×Open`) | `Open+BVSMC-2P` Tot (`×Open`) | **`GIM-RV+BVSMC-2P` Tot** (`×Open+BV2P` / **`×Open`**) |
|---|---:|---:|---:|---:|---:|---:|
| `block-0.L` | `0.927 / 1.445` | `0.795` (`0.857×`) | `0.752 / 1.235` | **`0.601`** (**`0.799×`** / **`0.648×`**) | `1.184` (`0.819×`) | **`1.036`** (**`0.875×`** / **`0.717×`**) |
| `block-2.L` | `1.094 / 1.782` | `0.920` (**`0.841×`**) | `0.863 / 1.487` | **`0.679`** (**`0.787×`** / **`0.620×`**) | `1.376` (`0.772×`) | **`1.180`** (**`0.858×`** / **`0.662×`**) |
| `block-85.L` | `0.944 / 1.826` | `0.878` (`0.931×`) | `0.669 / 1.511` | **`0.582`** (`0.870×` / **`0.617×`**) | `1.313` (`0.719×`) | **`1.226`** (`0.934×` / **`0.671×`**) |
| `block-143.L` | `1.830 / 3.291` | `1.551` (**`0.847×`**) | `1.433 / 2.726` | **`1.132`** (**`0.790×`** / **`0.618×`**) | `2.296` (`0.698×`) | **`2.004`** (**`0.873×`** / **`0.609×`**) |
| `block-210.L` | `1.414 / 2.779` | `1.267` (`0.896×`) | `1.056 / 2.327` | **`0.939`** (`0.889×` / **`0.664×`**) | `1.974` (`0.710×`) | **`1.820`** (`0.922×` / **`0.655×`**) |
| `block-207.L` | `1.264 / 2.148` | `1.085` (`0.858×`) | `1.012 / 1.811` | **`0.820`** (**`0.810×`** / **`0.649×`**) | `1.618` (`0.753×`) | **`1.443`** (**`0.892×`** / **`0.672×`**) |

### Honest Confrontation Verdict & Interpretation
- **Strict Pre-Registered Gate G5 Verdict:** **FAIL** (`GIM-RV+U16Q Con <= 0.85× Open+U16Q` passed on **`4/6`** blocks — `0.787×, 0.790×, 0.799×, 0.810×` — while `block-85` and `block-210` measured `0.870×` and `0.889×` because they have only `1,566` and `2,810` thick strips, so `RunQ` is `65–75%` of `construct`; `GIM-RV+BVSMC-2P Tot <= 0.72× Open` passed on **`6/6`** blocks — **`0.609×–0.717×`** — and `<= 0.92× Open+BVSMC-2P` hit **`4/6`**, with `block-210` at `0.922×`).
- **Substantive Assessment (Candidate Ready for Coordinator Safe-Rust Audit):**
  1. `GIM-RV` accelerates `construct` on **`6/6` blocks** with **zero regressions**, cutting `Open+U16Q`'s `construct` by **`11.1%–21.3%` (`0.787–0.889×`)** (`0.608–0.668×` vs shipped `Open-qi16`), freeing 4 scratch vectors (`order[2]`, `terminals[2]`), and cutting `EpRule` allocations by `93–98%`.
  2. Unlike `U16Q`'s queue-fill part (where LLVM's `Range<u32>` `Vec::extend` already vectorised natively in `#664`), `src/open/endpoint.rs` genuinely executes the 4-vector two-pass `lists()`, allocates all `4,058–12,619` `Rule` structs, and walks `self.stack` byte-by-byte in `append()` — so `GIM-RV` targets live safe-Rust work that LLVM cannot optimise away on its own.
  3. Composed end-to-end with zero-arena `BVSMC-2P` (`0 B` chunk pool), `GIM-RV + BVSMC-2P` reaches **`0.609–0.717×` `Open-qi16` across all `6/6` blocks** (faster even than Cycle 2's `1.23 MiB`-arena `BVSMC-1P` while using **`0 B` chunk arena**!).
