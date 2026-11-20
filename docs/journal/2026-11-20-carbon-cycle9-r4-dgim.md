# Carbon Cycle 9: Bijective BWT Golomb–Dickman Ceiling, Terminal Prefix Absorbency, and Rule-R4 Direct-Arena Graded Inline-Monoid Elimination (`R4-DGIM`)

*20 November 2026. Session: Carbon (`research/carbon-word-transport`). Mode: Reorientation (default, `METHOD.md` §3a).*

## 1. Cycle Orientation & Summary of Outcomes

At the Cycle 8 → 9 boundary, read-only inspection of `origin/master` (`f164dd7` / `#666`) surfaced an essential native calibration datapoint across all 431 Wikidata blocks:
1. Replacing `Vec<u32>` queues with `Vec16` (`U16Q`) alone takes native whole-corpus `front+Open` down to **`384–398 ms`**, outperforming `front+Cells` (`403–419 ms`) even at 1 thread (`LAZY_FROM_THREADS = 1`), whereas fusing MTF/RLE2 into the front loop regressed by `+20–36 ms` (`405–434 ms`).
2. Tight, decoupled loops over clean data layouts systematically beat complex fused state machines on native cores. Meanwhile, `src/cells.rs` (`MONODROMY`) uses **Rule R4** (`16 B` `prev`-only `Top` + `8 B` `prev`-only `Bot` slots with `FAR = 8` sequential lookahead prefetching) on whole-block intervals, whereas `src/open/endpoint.rs` (and Cycle 7's `GIM-RV`) still uses `24 B` doubly-linked `EpNode`s (`prev[2]`, `next[2]`), `order[2]`/`terminals[2]` setup vectors, `bottom: Vec<u32>`, and `spelling: Vec<u8>` + `offsets: Vec<u32>` even for $\le 16\text{ B}$ words.

Cycle 9 explored three distinct mathematical directions (combinatorics on words / Gessel–Reutenauer bijective BWT, Meyer threshold shifting + `out`-aliased slab queues, and transferring MONODROMY's asymmetric `Rule R4` invariant onto `Open`'s thick-complement exchange), triaged two candidates quantitatively before Gate G5, and took **Theorems 9.1, 9.2, and 9.3 (`R4-DGIM`)** through manifest checking (`r4dgim.toml`), executable verification (`verify_cycle9.py`), full AddressSanitizer/LeakSanitizer/UBSan validation, and a paired 25-round 7-arm C confrontation (`g5_r4dgim.c`):

1. **Theorem 9.1 — Gessel–Reutenauer / MRRS Bijective BWT & The Golomb–Dickman Parallelism Ceiling (`proved`, `verified`, & `measured`):**
   Investigated whether switching from cyclic BWT (`1` $n$-cycle + primary index $p$) to the **Mantaci–Restivo–Rosone–Sciortino (2007) / Gessel–Reutenauer (1993) Bijective BWT (`BBWT`)** (which splits $\mathrm{LF}$ into $k$ natural disjoint cycles corresponding to the Chen–Fox–Lyndon factorization $T = w_1 \ge_{\text{lex}} \dots \ge_{\text{lex}} w_k$ without storing $p$) yields free multi-lane orbit parallelism (`verified` byte-exact without $p$ across `5,186/5,186` binary $1 \le n \le 11$ and ternary $1 \le n \le 6$ strings). Running Duval's $O(n)$ Lyndon factorization on all **6 authenticated Wikidata blocks** revealed $k \in [9, 19]$ factors (mean `14.8` vs $\ln n + \gamma = 14.3$), with the **single largest Lyndon factor occupying `34.6%–93.8%` of the entire 900 KB block** (`844,003 B` = **`93.79%`** on `block-143`; mean **`0.6900`** vs the Golomb–Dickman constant $\lambda \approx 0.62433$). Thus natural `BBWT` cycle decomposition caps multi-lane Amdahl speedup at **`1 / 0.690 = 1.45×`** (`1.07×` on `block-143`) while changing the wire format, whereas artificial `BVSMC` sentinel cutting splits any cyclic BWT into $S = 64$ balanced segments regardless of $\lambda$.
2. **Triage Kills — Marginal Equilibrium of `LIMIT` ($w^* = c_{\text{RV}}/c_{\text{row}}$) and `out`-Aliased `SlabQ` (`measured`, killed before G5):**
   - *Threshold sweep under `GIM-RV + U8Q + BVSMC-2P` (`/tmp/probe_gim_lim.c`):* Tested whether `GIM-RV`'s faster $\mathcal{M}_{64}$ elimination + `U8Q` (`LIMIT = 8`) shifts `Open`'s optimal thin/thick threshold below `16` (`#657`). Measured totals on `block-0` / `block-143`: `L=4` (`1.225 / 2.263 ms`), `L=8` (`1.031 / 1.981 ms`), `L=12` (`0.985 / 1.908 ms`), **`L=16` (`0.967 / 1.882 ms`, global minimum)**. Derived the exact marginal equilibrium law:
     $$w^* \;=\; \frac{c_{\text{RV\_strip}}}{c_{\text{thin\_row}}} \;\approx\; \frac{55\text{ cycles}}{4.0\text{ cycles}} \;\approx\; 14\text{--}16.$$
   - *`out`-Aliased 64-Entry `SlabQ` (`/tmp/probe_oaz.c`):* Eliminating `queues[256]` (`~800 KiB`) by bumping 64-entry (`256 B`) cache-line slabs inside dead `out[0..n)` regressed `RunQ` by **`2.51–2.81×`** (`0.556–1.220 ms` vs `0.214–0.483 ms`) because average thin width $\bar{w} \approx 4$ triggers slab-boundary slow-path splits on `~25%` of runs.
3. **Theorems 9.2 & 9.3 — Terminal Prefix Absorbency & Rule-R4 Direct-Arena Graded Inline-Monoid Elimination (`R4-DGIM`, `proved`, `verified`, & `measured`):**
   - Proved **Terminal Prefix Absorbency (Theorem 9.2)**: in `Open`'s thick-complement exchange $(P_0 \cup A_0,\; P_1 \cup A_1)$, $|P_0| = |P_1| = m$ and $|A_0| = |A_1| = n_{\text{rem}}$ hold at every step, the backward frontier $(sa, sb)$ never enters $P_0$ or $P_1$, and MONODROMY's asymmetric `Rule R4` overwrite (`bots[bs]` when $r_a > r_b$, `tops[ts]` when $r_a < r_b$) strictly obeys read-before-overwrite (`verified` across `10,454/10,454` small step-by-step audits).
   - Built **`R4-DGIM`** (`16 B` `R4Top` + `8 B` `R4Bot`, `100%` safe-Rust `u32` slice indexing with zero raw pointers, `FAR = 8` sequential lookahead prefetching, `0 B` `next[2]`, `0 B` `order[2]`/`terminals[2]`, `0 B` `bottom[]` via `tops[ts].other` reuse, `0 B` `offsets[]`, and direct `small_arena -> inline16[code]` XMM construction with long-only `long_spell` for $>16\text{ B}$ words).
   - Across 25 paired interleaved rounds (`g5_r4dgim.c`), **`R4-DGIM + BVSMC-2P` (`Arm 6`) strictly beat inline-linked `GIM-RV + BVSMC-2P` (`Arm 5`) on 6/6 blocks** (`0.971–0.996×` total, `0.964–0.987×` `construct`), reaching **`0.580–0.682×` shipped `Open-qi16`** (`<= 0.66×` on `5/6` blocks, **`0.580×` on `block-143`**) while cutting active scratch on **6/6 blocks** (`1,155–2,269 KiB` vs `1,279–2,504 KiB`). In 1-lane mode, **`R4-DGIM + U16Q - 1L` (`Arm 4`) beat `Open + U16Q - 1L` (`Arm 2`) by `9.7%–19.5%` end-to-end (`0.805–0.903×`)** and **accelerated 1-lane emission over `GIM-RV` by `4.2%–7.6%` (`0.924–0.958×`) on 6/6 blocks** by keeping `spelling[]` and `offsets[]` out of L2 cache.
   - Pre-registered strict Gate G5 verdict is **`FAIL`** (`0/6` on `<= 0.93×` Arm 3 `construct` and `0/6` on `<= 0.96×` Arm 5 total), isolating a sharp **L2-Resident Thick-Complement Regime Law**: once inline chain linking (`LINK_CHAIN`, zero `order/terminals`) is applied to both `GIM-RV` and `R4-DGIM`, `Open`'s thick complement ($H \in [1,565, 5,047]$, `75–243 KiB` active slots) already fits inside `1 MiB` L2 (`RV-Elim = 7–12%` of total time), bounding any further thick-RV representation speedup by `< 4%` end-to-end while establishing `R4-DGIM` as the cleanest, smallest, pointer-free safe-Rust synthesis of the `GIM-RV` family.

---

## 2. Four Mandatory Candidate Fields (`METHOD.md` §3a)

- **Candidate:** Rule-R4 Direct-Arena Graded Inline-Monoid Open Elimination (`R4-DGIM`)
- **Mode:** Reorientation (replaces `Open`'s doubly-linked `EpNode` exchange graph, `order`/`terminals` setup buffers, `bottom` coordinate pass, `offsets` table, and $\le 16\text{ B}$ `spelling` dictionary with an asymmetric `prev`-only `R4Top`/`R4Bot` sequential tape + `FAR = 8` lookahead prefetch + direct `small_arena -> inline16` XMM construction).
- **Displaced obligations:**
  1. `EpNode.next[0..1]` forward links (`8 B/node`), doubly-linked `ep_cut`/`ep_insert` pointer surgery (up to 8 memory writes/step), and `order[0..1]` + `terminals[0..1]` setup vectors (`4 Vec<u32>`);
  2. Separate `bottom: Vec<u32>` allocation and forward $P_1$ scan (replaced by storing `x_bot` directly inside dead `tops[ts].other` during a backward `prev` scan of $P_1$);
  3. `offsets: Vec<u32>` and `spelling: Vec<u8>` writes for all $\le 16\text{ B}$ words (`>93%` of bands), eliminating `~400–900 KiB` of L2 cache write pollution immediately before `Wire` and `Emit`.
- **Replacement principle:** Partition initial `R4Top` (`16 B`: `{prev, width, word, other}`) and `R4Bot` (`8 B`: `{prev, top}`) tapes into $(P_0 \cup A_0,\; P_1 \cup A_1)$ in decreasing elimination order so $(sa, sb)$ walk monotonically downward with `FAR = 8` software prefetching (`_mm_prefetch(&bots[tops[sa-8].other])`, `_mm_prefetch(&tops[bots[sb-8].top])`). By Terminal Prefix Absorbency (Theorem 9.2), $(sa, sb)$ exhaust $A_0, A_1$ simultaneously at `remaining == 0` without entering $P_0, P_1$, leaving surviving chains rooted at $(sa, sb)$ of total width $m$. Reuse `tops[ts].other` for $L$-coordinates, materialise $\le 16\text{ B}$ words directly from `small_arena` into `inline16[code]`, pack `{o0, ln}` into `inline16[code]` for $>16\text{ B}$ words, and wire `edges_L` sequentially per thin queue.
- **New paid obligations:**
  1. One `strip_top[0..H)` temporary `u32` map (`6–20 KiB` L1) during two-pass slot tape construction (`from`-sorted $A_0$ followed by `image_order`-sorted $P_0/B$);
  2. Reverse byte-bucket band scan (`a = 255..0`) matching the backward `prev`-linked surviving $P_0$ chain (`kc = m..0`).
- **Nearest prior art:** MONODROMY's whole-block `Cells` (`src/cells.rs`, which uses raw-pointer `Rule R4` across all $R$ runs with binary-DAG `rules` and `write_owned` single-cursor replay, refusing blocks with $R > 0.10n$ such as `block-143`/`block-210`) and Carbon Cycle 7's `GIM-RV` (`g5_gimrv.c`, which introduced $\mathcal{M}_{64}$ on `Open` but kept `24 B` doubly-linked `EpNode`s, `order[2]`/`terminals[2]`, `bottom[]`, `offsets[]`, and full `spelling[]`).

---

## 3. Foundational Theorems

### Theorem 9.1 — Gessel–Reutenauer / MRRS Bijective BWT & The Golomb–Dickman Parallelism Ceiling (`proved`, `verified`, & `measured`)

**Statement (`proved`):** Let $T = w_1 w_2 \cdots w_k \in \Sigma^n$ be the unique non-increasing Chen–Fox–Lyndon factorization ($w_1 \ge_{\text{lex}} w_2 \ge_{\text{lex}} \cdots \ge_{\text{lex}} w_k$, each $w_i$ a primitive Lyndon word). Sorting the $\sum_{i=1}^k |w_i| = n$ conjugates of all factors under the $\omega$-order ($u \prec_\omega v \iff u^\omega <_{\text{lex}} v^\omega$) produces $L_{\text{BBWT}}(T) \in \Sigma^n$ whose standard $\mathrm{LF}$ permutation decomposes into **$k$ disjoint cycles** $\mathcal{C}_1, \dots, \mathcal{C}_k$ of lengths $|w_1|, \dots, |w_k|$. In each cycle $\mathcal{C}$, the unique minimum row index $m(\mathcal{C}) = \min \mathcal{C}$ corresponds to the Lyndon conjugate $w_i$ itself, and sorting the cycle reconstructions in **decreasing** order of $m(\mathcal{C})$ recovers $T = w_1 \cdots w_k$ **uniquely without storing a primary index $p$**. However, for random/high-entropy strings of length $n$, the expected share of the longest cycle converges to the **Golomb–Dickman constant**
$$\lim_{n \to \infty} \frac{\mathbb{E}[\max_i |w_i|]}{n} \;=\; \lambda \;=\; \int_0^1 e^{\mathrm{li}(t)}\,dt \;\approx\; 0.6243299885,$$
bounding natural `BBWT` cycle-level multi-lane speedup by $1 / \lambda \approx 1.60\times$ in expectation (and $n / \max_i |w_i|$ per block). $\square$

**Executable Verification & Corpus Census (`verified` & `measured`, `verify_cycle9.py`):**
- Verified byte-exact `BBWT` forward transform + index-free cycle-minima inversion across **`5,186/5,186`** strings (all binary $1 \le n \le 11$ and ternary $1 \le n \le 6$).
- Measured exact Duval $O(n)$ Lyndon factorizations across all **6 authenticated Wikidata blocks**:

| Block | $n$ | Lyndon Factors $k$ | Max Factor $\max_i |w_i|$ (B) | Max Share $\max_i |w_i| / n$ | Natural `BBWT` Cycle Ceiling $n / \max_i |w_i|$ |
|---|---:|---:|---:|---:|---:|
| `block-0.L` | 899,931 | 16 | 311,300 | `0.3459` | `2.89×` |
| `block-2.L` | 900,000 | 19 | 428,982 | `0.4766` | `2.10×` |
| `block-85.L` | 899,935 | 17 | 774,522 | `0.8606` | `1.16×` |
| `block-143.L` | 899,869 | 12 | 844,003 | **`0.9379`** | **`1.07×`** |
| `block-210.L` | 899,959 | 16 | 682,812 | `0.7587` | `1.32×` |
| `block-207.L` | 899,998 | 9 | 684,206 | `0.7602` | `1.32×` |

Mean largest-factor share is **`0.6900`** (capping natural cycle-level parallelism at **`1.45×`**, and **`1.07×` on `block-143`**), rigorously proving why artificial `BVSMC` sentinel cutting ($S = 64$, uniform retained-orbit cuts) strictly dominates natural `BBWT` multi-cycle splitting.

---

### Theorem 9.2 — Terminal Prefix Absorbency & `Rule R4` Transfer to `Open` (`proved` & `verified`)

**Statement (`proved`):** Partition $[0, n)$ into $H$ thick strips $A_{\text{thick}}$ (total width $n_{\text{thick}} = n - m$) and complementary thin-run gap intervals (`word = 0`, total width $m$). Construct:
- Top chain $P_0 \cdot A_0$: prefix $P_0$ holds gap intervals of $[0, n) \setminus \mathrm{LF}(A_{\text{thick}})$ (total width $m$); active suffix $A_0$ holds $[0, n)$ partitioned into thin gaps + thick strips (total width $n$).
- Bottom chain $P_1 \cdot A_1$: prefix $P_1$ holds thin gap intervals of $[0, n) \setminus A_{\text{thick}}$ (total width $m$); active suffix $A_1$ holds $[0, n)$ partitioned into $\mathrm{LF}$-image gaps + thick strips (total width $n$).

Then at every right-to-left Rauzy–Veech step of `remaining` width from $n$ down to $0$:
1. **Prefix Width Invariance & Frontier Isolation:** Every splice removes $\min(r_a, r_b)$ from the rightmost intervals $(sa \in A_0,\; sb \in A_1)$ and inserts the spliced residual at its partner $(bs, ts)$, preserving total prefix widths $|P_0| = |P_1| = m$ and active suffix widths $|A_0| = |A_1| = \text{remaining}$. Hence $sa \notin P_0$ and $sb \notin P_1$ while $\text{remaining} > 0$, and at $\text{remaining} = 0$ both frontiers $(sa, sb)$ land simultaneously on the rightmost nodes of the surviving $m$-width terminal chains $(P_0', P_1')$.
2. **Read-Before-Overwrite Safety under Asymmetric `Rule R4`:** Storing only `prev` links (`R4Top = {prev, width, word, other}`, `R4Bot = {prev, top}`) with `sa`, `sb` starting at the rightmost indices of $A_0$, $A_1$:
   - When $r_a > r_b$, `bots[bs]` lies strictly left of $sb$ ($bs < sb$ or in $P_1$), `bots[fb].prev` is redirected to `bots[bs].prev`, and `bots[bs]` is overwritten with `{prev: fb, top: ts}` after `bots[bs].prev` has been read;
   - When $r_a < r_b$, `tops[ts]` lies strictly left of $sa$, `tops[fa].prev` is redirected to `tops[ts].prev`, and `tops[ts]` is overwritten with `{prev: fa, width: ra, word: wd, other: bs}` after `tops[ts]` has been read;
   - When $\text{remaining} = 0$, `tops[ts].other` is dead across all $ts \in P_0'$ and safely stores $x_{\text{bot}} \in [0, m)_L$ during a single backward scan of $P_1'$ from $sb$. $\square$

---

### Theorem 9.3 — Exactness of `R4-DGIM` (`verified`)

- `check_manifest.py docs/journal/2026-11-20-carbon-cycle9/r4dgim.toml`: **PASS**.
- `verify_cycle9.py`: **PASS** across **`10,454/10,454` exhaustive step-by-step invariant audits** (`5,227` small `(L, p)` pairs $\times$ 2 limits $\times$ 2 consumers = **`20,908` runs**), **`8/8` adversarial controls** (including `H = 0` all-thin `4 KiB` and `50k` periodic shear $>64\text{ B}$ `EpRule` DAG), and **`6/6` authenticated Wikidata blocks** (`5,399,692` B in `4.92s`).

---

## 4. Gate G3 Cost Vector & Gate G5 Confrontation (`g5_r4dgim.c`)

### Producer–Consumer Cost Accounting (Gate G3)

| Metric | Shipped `Open-qi16` (Arm 0) | Inline-Linked `GIM-RV` (Arms 3/5) | `R4-DGIM` (Arms 4/6) |
|---|---|---|---|
| Node / slot layout | `24 B` `EpNode` (`prev[2]`, `next[2]`, `w`, `word`) + `order[2]` + `terminals[2]` | `24 B` `EpNode` (`LINK_CHAIN`, zero `order/terminals`) | `16 B` `R4Top` (`4/line`) + `8 B` `R4Bot` (`8/line`), `prev`-only, `100%` `u32` safe-Rust slices |
| Lookahead prefetch | None (pointer-chased doubly-linked list) | None | `FAR = 8` sequential tape lookahead (`_mm_prefetch`) |
| `bottom[]` coordinate array | `4 B × N_nodes` heap vector + forward scan | `4 B × N_nodes` heap vector + forward scan | **`0 B`** (reuses dead `tops[ts].other` via backward $P_1$ scan) |
| `offsets[]` + $\le 16\text{ B}$ `spelling[]` | Full `offsets[]` + full `spelling[]` | Full `offsets[]` + full `spelling[]` + `inline16[]` | **`0 B` `offsets[]`**, **`0 B` `spelling[]` for $\le 16\text{ B}$ words** (`{o0, ln}` inside `inline16[code]` for $>16\text{ B}$) |
| Active working set (6 blocks) | `1,279.2 – 2,503.7 KiB` | `1,181.2 – 2,328.4 KiB` | **`1,155.0 – 2,269.0 KiB`** (`-19 to -59 KiB` vs `GIM-RV`, `-69 to -235 KiB` vs `Open`) |

### Sanitizer & Paired 25-Round 7-Arm Confrontation (`measured`, `g5_r4dgim.c`)

- **AddressSanitizer + LeakSanitizer + UndefinedBehaviorSanitizer (`-O1 -g -fsanitize=address,undefined`):** **PASS** (`0` warnings, `0` memory errors, `0` UB, `0` leaks across `8/8` adversarial self-tests with `32 B` left/right overrun canaries + `6/6` Wikidata blocks across all `7` arms).
- **25-Round Paired Interleaved Benchmark (`gcc -O3 -march=native`, rotating arm order `(step + r + 3) % 7` after 3 warmup rounds; VM note: single-thread calibrated kernel timings on `2.10 GHz` Xeon VM, not 20-thread native proof):**

| Block | Arm 0 `Open-qi16` (Con / Emit / Tot ms) | Arm 2 `Open+U16Q-1L` (Con / Emit / Tot ms) | Arm 3 `GIM-RV+U16Q-1L` (Con / Emit / Tot ms) | Arm 4 `R4-DGIM+U16Q-1L` (Con / Emit / Tot ms) | Arm 5 `GIM-RV+BV2P` (Con / Emit / Tot ms) | Arm 6 `R4-DGIM+BV2P` (Con / Emit / Tot ms, `/Open`) |
|---|---:|---:|---:|---:|---:|---:|
| `block-0.L` | `0.942 / 0.543 / 1.484` | `0.770 / 0.521 / 1.298` | `0.583 / 0.505 / 1.092` | **`0.565 / 0.484 / 1.049`** | `0.582 / 0.440 / 1.024` | **`0.565 / 0.445 / 1.011` (`0.682×`)** |
| `block-2.L` | `1.131 / 0.717 / 1.851` | `0.898 / 0.668 / 1.568` | `0.673 / 0.652 / 1.325` | **`0.652 / 0.619 / 1.263`** | `0.680 / 0.544 / 1.220` | **`0.657 / 0.544 / 1.193` (`0.644×`)** |
| `block-85.L` | `0.993 / 0.925 / 1.925` | `0.700 / 0.913 / 1.616` | `0.566 / 0.901 / 1.462` | **`0.565 / 0.849 / 1.412`** | `0.574 / 0.708 / 1.272` | **`0.562 / 0.706 / 1.267` (`0.658×`)** |
| `block-143.L` | `1.859 / 1.525 / 3.385` | `1.437 / 1.398 / 2.821` | `1.106 / 1.390 / 2.496` | **`1.084 / 1.285 / 2.363`** | `1.113 / 0.912 / 2.023` | **`1.073 / 0.891 / 1.964` (`0.580×`)** |
| `block-210.L` | `1.488 / 1.435 / 2.927` | `1.123 / 1.385 / 2.513` | `0.918 / 1.385 / 2.301` | **`0.946 / 1.319 / 2.268`** | `0.935 / 0.902 / 1.841` | **`0.923 / 0.902 / 1.819` (`0.621×`)** |
| `block-207.L` | `1.291 / 0.910 / 2.198` | `1.039 / 0.848 / 1.888` | `0.801 / 0.842 / 1.641` | **`0.789 / 0.799 / 1.588`** | `0.820 / 0.597 / 1.418` | **`0.797 / 0.591 / 1.392` (`0.633×`)** |

*Note: Arm 1 (`R4-DGIM-ScalQ-1L`, pure scalar non-`U16Q` producer swap) also beat `Open-qi16` on **6/6 blocks** (`0.770–0.851×` `construct`, `0.861–0.937×` total).*

### Pre-Registered Gate G5 Verdict & Exact Microarchitectural Diagnosis

Keeping the pre-registered criteria in `g5_r4dgim.c` strictly frozen:
- **(1) Exactness & Full Sanitizer Cleanliness:** **PASS** (`6/6` blocks, `8/8` adversarial self-tests, `0` ASan/LSan/UBSan issues).
- **(2) `Arm 4 Con <= 0.80× Arm 2 Con` (`Open+U16Q`):** `4/6` blocks (`0.734×, 0.726×, 0.807×, 0.754×, 0.842×, 0.759×`; needed `>= 5/6`).
- **(3) `Arm 4 Con <= 0.93× Arm 3 Con` (inline-linked `GIM-RV`):** `0/6` blocks (`0.969×, 0.969×, 0.998×, 0.980×, 1.031×, 0.985×`; Arm 6/Arm 5 `construct` = `0.964–0.987×` on `6/6` blocks).
- **(4a) `Arm 6 Tot <= 0.66× Arm 0 Tot` (`Open-qi16`):** **PASS (`5/6` blocks: `0.682×, 0.644×, 0.658×, 0.580×, 0.621×, 0.633×`)**.
- **(4b) `Arm 6 Tot < 1.00× Arm 5 Tot` (`GIM-RV+BVSMC-2P`):** **PASS (`6/6` blocks: `0.987×, 0.978×, 0.996×, 0.971×, 0.988×, 0.982×`)**.
- **(4c) `Arm 6 Tot <= 0.96× Arm 5 Tot`:** `0/6` blocks.
- **Overall Pre-Registered Verdict:** **`FAIL`** (missed strict relative jump over Arm 3/5).

**Why `R4-DGIM` won `6/6` blocks over `GIM-RV` by `1.4%–5.3%` instead of `>7%` (Two Exact Laws):**
1. **Inline Chain-Linking Uplift in Arm 3/5 (`LINK_CHAIN`):** In Cycle 7 (`g5_gimrv.c`), `GIM-RV` still materialised `src/open/endpoint.rs`'s four setup vectors (`order[0..1]`, `terminals[0..1]`) before linking `nodes[]`, running at `0.787–0.889× Open+U16Q`. Giving Arm 3/5 direct inline `LINK_CHAIN` already lifted `GIM-RV` itself to **`0.756–0.818× Open+U16Q`** (`0.566–1.106 ms`), capturing half of the setup win in the baseline arm.
2. **L2-Resident Thick-Complement Regime (`Open` $H \ll R$ vs `Cells` $R$):** In whole-block `Cells` (`LIMIT = 1`), $R \approx 180,000$ runs create `~360,000` slots (`4.3–8.6 MiB`, spilling `1 MiB` L2 into L3) and `RV-Elim` is `>65%` of total decode, so `Rule R4` + `FAR = 8` yields a `~20%` macro speedup. In `Open` (`LIMIT = 16`), thin admission removes `85–95%` of runs, leaving $H \in [1,565, 5,047]$ thick strips (`75–243 KiB` of active slots — **100% resident in L2 cache**) and shrinking `RV-Elim` to **`0.070–0.240 ms` (`7–12%` of total decode)**. By Amdahl's law, thick-RV slot improvements are strictly bounded by `< 4%` of total decode!
3. **Direct-Arena `inline16` L2 Cache Preservation (`-4.2% to -7.6%` 1-Lane Emit):** Eliminating `spelling[]` + `offsets[]` writes for $\le 16\text{ B}$ words sped up **1-lane emission (`Arm 4 Emit / Arm 3 Emit = 0.924–0.958×`) on all 6/6 blocks** (`1.390 → 1.285 ms` on `block-143`) because `400–900 KiB` of cold `spelling`/`offsets` stores no longer evict `edges[]` and `inline16[]` from L2 before `emit`. For safe-Rust implementation of Cycle 7's `GIM-RV` in `src/open/endpoint.rs`, `R4-DGIM` is strictly cleaner (`100%` `u32` slice indexing, zero raw pointers, zero `bottom[]`, zero `offsets[]`), smaller (`-19..59 KiB`), and faster on `6/6` blocks.

---

## 5. Next Frontier Implication for Cycle 10

With `RV-Elim` reduced to **`0.070–0.240 ms` (`7–12%` of total decode)** inside `R4-DGIM + BVSMC-2P` (`1.011–1.964 ms`), the residual wall-clock budget is now concentrated in two non-RV obligations:
1. **2-Pass vs 1-Pass Multi-Lane Emission (`0.445–0.902 ms`, `44–56%` of total):** `BVSMC-2P` walks `edges[0..m]` **twice** (`2m` random reads: Pass 1 length discovery + Pass 2 SIMD write) to avoid `BVSMC-1P`'s `1.23 MiB` segment pool. Can `R4-DGIM` compose with **single-pass `BVSMC-1P`** or **dead-`out` / sub-chunked 1-pass splicing** to drop from `2m` back to `1m` `edges[]` traversals?
2. **Retained Graph Degree & Run-Queue Scatter (`RunQ + Wire = 0.27–0.61 ms`, `27–31%` of total):** Can multi-step thin return composition or alphabet-clustered queue layout cut `m` or `RunQ` cache-line RFO pressure without incurring Cycle 4's `1.5m` extensional gather penalty?
