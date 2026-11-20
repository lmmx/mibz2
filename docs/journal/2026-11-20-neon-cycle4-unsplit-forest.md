# Neon Cycle 4 — Unsplit-Piece Functional In-Forest Contraction (`UFC`), Run-Graph Producer–Consumer Parity, and Thin-Excursion Spelling Conservation

*Date:* 2026-11-20  
*Branch:* `research/neon-bipolar-contraction`  
*Mode:* `reorientation` (default under `METHOD.md` §3a at `origin/master` `f129af32`)  
*Code & artifacts:* [`verify_ufc.py`](2026-11-20-neon-cycle4-unsplit-forest/verify_ufc.py), [`ufc.toml`](2026-11-20-neon-cycle4-unsplit-forest/ufc.toml), [`g5_ufc.c`](2026-11-20-neon-cycle4-unsplit-forest/g5_ufc.c), [`g5_ufc1_compact.c`](2026-11-20-neon-cycle4-unsplit-forest/g5_ufc1_compact.c)

---

## 1. Executive Summary

Cycle 4 attacks **Obligation #1 (Dependent traversal of retained rows)** and **Foundational Questions #1–#3** (*must the decoder follow an orbit at all?*, *is there a limit object of repeated cutting computed without doubly-linked Rauzy–Veech induction?*) from a fresh **representation import**: viewing a discrete interval exchange transformation (IET) $T$ through its **piece-overlap digraph** $G(T)$ and separating **unsplit pieces** ($\text{out\_deg}_{G(T)}(u) = 1$, whose image falls wholly inside a single domain piece) from **splitter pieces** ($\text{out\_deg}_{G(T)}(k) \ge 2$).

**Key discoveries:**
1. **Pigeonhole Unsplit-Piece Theorem (`proved`):** In *every* $r$-piece discrete IET ($r \ge 2$), at least $|D \cap I| + 1 \ge 1$ pieces are unsplit ($\text{out\_deg} = 1$). On all six authenticated Wikidata blocks (`measured` in [`verify_ufc.py`](2026-11-20-neon-cycle4-unsplit-forest/verify_ufc.py)), **$92.5\%–93.0\%$ of all $r$ BWT runs** ($47{,}395–119{,}691$ runs) are unsplit and form a 100% acyclic **functional in-forest** $U$ rooted at only **$|K| = 3{,}789–9{,}041$ splitter runs** ($7.0\%–7.5\%$ of $r$).
2. **F-Order Conjugation & Zero-Random-Access Forest Build (`proved`):** Indexing $U$ in $F$-order (`fi = 0..r-1`) instead of $L$-order builds the entire `4 B/run` in-forest (`200–515 KiB`) and all $A = 14{,}743–35{,}789$ splitter arcs in three streaming linear passes (zero random reads, zero sorting). Tracing all $A$ arcs through $U$ in 8-way parallel MLP produces a contracted exact IET $T_K$ with flat literal strings (`79–184 KiB`, mean `4.2–4.8 B`, zero SLP rules in Round 1).
3. **Two complete exact inverse-BWT architectures (`verified` byte-exact on `10,454 / 10,454` small cases, `7 / 7` adversarial cases, and all 6 Wikidata blocks = `5,399,692 / 5,399,692` bytes):**
   - **`UFC-Full`:** Flat Round 1 + `10–14` iterated L2-resident UFC rounds with **Quotient Self-Overlap Acceleration** (handling `Sturmian(1,99999)` in 3 rounds / 25 SLP rules and `Fibonacci(6765,10946)` in 20 rounds / 19 SLP rules) + direct-rotated first-occurrence SLP copy expansion. Zero Rauzy–Veech induction, zero doubly-linked lists, zero orbit walk; collapses $51\text{k}–129\text{k}$ runs to 1 piece in `0.78–1.00r` SLP rules (`9.0–19.3 B/copy`).
   - **`UFC-1 + Open`:** One round of Flat UFC followed by Open's endpoint elimination on only $H_1 = 592–2{,}046$ thick strips (`2.0–2.7×` fewer than Shipped Open) and an Open walk on only $m_1 = 26{,}124–70{,}082$ thin rows (`2.8–3.7×` smaller than Open's $m = 89{,}874–204{,}281$).
4. **Confrontation & Pre-Registered Gate G5 Verdict (`measured` in [`g5_ufc.c`](2026-11-20-neon-cycle4-unsplit-forest/g5_ufc.c) and [`g5_ufc1_compact.c`](2026-11-20-neon-cycle4-unsplit-forest/g5_ufc1_compact.c)):**
   - **Pre-registered G5 rule (all 6 blocks vs Shipped Open):** Byte-exact, `L2m <= 0.85x`, `DRAM B/B <= 0.90x`, `PeakMem <= 1.15x`, `VM time <= 0.90x`.
   - **`UFC-Full`: FAIL** (`L2m = 2.36–3.44×`, `DRAM = 1.85–3.34×`, `Peak = 0.89–1.39×`, `VM time = 1.70–2.58×`).
   - **`UFC-1 + Open` (initial [`g5_ufc.c`](2026-11-20-neon-cycle4-unsplit-forest/g5_ufc.c)): FAIL** (`L2m = 1.38–1.87×`, `DRAM = 1.44–2.30×`, `Peak = 0.58–0.96×` WIN, `Work = 0.80–0.96×` WIN, `VM time = 0.92–1.30×`, beating Open in VM time on `block-0.L` at `2.34 ms` vs `2.53 ms`).
   - **`UFC-1 + Open (Compact + Inline-5B u64)` ([`g5_ufc1_compact.c`](2026-11-20-neon-cycle4-unsplit-forest/g5_ufc1_compact.c), single justified incidental layout repair): FAIL** (`L2m = 1.37–1.72×`, `DRAM = 1.44–2.09×`, `Peak = 0.64–0.86×` WIN, `Work = 0.77–0.91×` WIN, `VM time = 1.10–1.49×`), even though **`emit` L2 misses fell to `0.49–0.67×` Open** and **`emit` VM time fell to `0.68–0.78×` Open** on all 6 blocks!
5. **Two New Structural Obstructions Uncovered:**
   - **Run-Graph Producer–Consumer Parity Obstruction:** After subtracting mandatory input-read and output-write lines, Shipped Open already decodes all 6 blocks at only **`1.08–1.49` L2 misses per BWT run $r$**. Because `62.8–75.4%` of all Wikidata runs have width $w = 1$ (and 100% of width-1 runs are trivially unsplit), *any* contraction that constructs and traverses a graph over all $r$ runs (`forest[0..r] = 200–515 KiB`, `|U| = 0.47–0.59m`, `hops = 0.63–0.75m`) spends `~1.0–1.2` L2 misses/run in the producer alone before `emit` starts—exceeding the `33–51%` L2-miss win it achieves in `emit`.
   - **Thin-Excursion Spelling Conservation Obstruction:** Attempting to bypass width-1 runs by lowering Open's `LIMIT` from `16` to `1–2` shrinks `edges` by `219–497 KiB` while inflating flattened `spelling + offsets` by `+500–670 KiB` (`32–58 KiB -> 495–623 KiB`) and increasing `ep` steps `7–10×` across all 6 blocks.

*Distinction of status:* Both `UFC-Full` and `UFC-1 + Open` are **possible** (`proved` + `verified` byte-exact) and **feasible** (Gate G2 `PASS`, zero-warning C implementations with Peak RAM `0.58–0.96×` Open and total Work `0.77–0.91×` Open); neither is **competitive** on L2/DRAM traffic (`G5 FAIL`).

---

## 2. §3a Mandatory Four-Field Statements (No Self-Classification)

### Candidate 4A: `UFC-Full` (Flat-R1 + Iterated Unsplit-Piece Functional In-Forest Contraction)
- **Mode:** `reorientation`.
- **Displaced obligation:** Eliminates *both* Obligation #1 (Open's $m$-step serial dependent orbit walk) *and* Rauzy–Veech doubly-linked endpoint induction (Cells / Open `ep`), replacing them with iterated piece-level functional in-forest contraction into a single straight-line program (SLP) of `0.78–1.00r` rules expanded by first-occurrence substring copies (`9.0–19.3 B/copy`).
- **Replacement principle:** **Pigeonhole Unsplit-Piece Theorem** (`proved`, §3.1) + **Quotient Self-Overlap Acceleration** (`proved`, §3.4): every discrete IET has $\ge |D \cap I| + 1$ unsplit pieces whose images sit inside a single domain piece; contracting the induced functional in-forest $U$ via first-return induces an exact smaller IET $T_K$ on the splitter subset $K$, converging to 1 piece in `11–15` rounds on Wikidata (and $O(\log n)$ rounds on Sturmian/Fibonacci inputs).
- **New paid obligations:** Round-1 `F`-order forest build (`3` linear scans over $r$ runs + `4 B/run` forest table) and 8-way parallel forest trace (`66k–153k` hops in `200–515 KiB`), followed by `10–14` smaller rounds of `UFC` + SLP rule construction (`46.7k–99.8k` rules) + memoized first-occurrence expansion copies from `out[0..n)`.
- **Nearest prior art:** Cells (`src/cells.rs`, Rauzy–Veech induction on doubly-linked interval endpoints) and Neon Cycle 1 Half-Bipolar contraction (which alternated forward/backward contraction across all $r$ pieces with `40–100 B/run` metadata). `UFC-Full` eliminates doubly-linked endpoints, eliminates per-piece sorting in Round 1 via $F$-order conjugation, keeps Round 1 flat (`4 B/run`, zero SLP rules), and contracts *only* unsplit pieces simultaneously across the whole in-forest.

### Candidate 4B: `UFC-1 + Open` (1-Round Flat UFC Prefix + Compact Inline-5B Open)
- **Mode:** `reorientation`.
- **Displaced obligation:** Displaces `64–73%` of Obligation #1 (shrinking Open's retained set from $m = 89{,}874–204{,}281$ to $m_1 = 26{,}124–70{,}082$, cutting `emit` L2 misses by `33–51%` and `emit` VM time by `22–32%`) and `50–63%` of thick-strip endpoint induction (`nstrips`: `1,565–5,047 -> 592–2,046`).
- **Replacement principle:** **Pigeonhole Unsplit-Piece Theorem** + **F-Order Conjugation** (`proved`, §§3.1–3.2): contracting the 1-round unsplit functional in-forest $U$ (with flat literal strings, `68.2–74.9%` of thin pieces $\le 5$ bytes) produces a strictly smaller IET $T_K$ whose thin rows are `2.8–3.7×` fewer than Open's $m$, and whose short literal words can be inlined inside `8 B` `u64` edges (`g5_ufc1_compact.c`).
- **New paid obligations:** Round-1 `F`-order forest build (`l_cw`, `f_of_l`, `f_w8`, `forest`, `raw_arcs`) + 8-way parallel forest trace (`66k–153k` hops in `200–515 KiB`) + 2-pass radix sort of `thins` by `to` (because Round-1 contraction breaks Open's free 256-symbol counting-sort queue ordering) + literal/excursion spelling tables.
- **Nearest prior art:** Shipped Open (`src/open.rs`, `LIMIT = 16` first-return on thick strips) and Astra investigation 3's multi-level forest cuts. Unlike Open's `LIMIT` reduction (which pushes `w = 2..16` into doubly-linked RV and explodes `spelling` to `495–623 KiB`), `UFC-1` eliminates $93\%$ of *all* runs (including `w = 1..16`) via a single acyclic in-forest before calling Open on $2.8–3.7\times$ fewer rows.

---

## 3. Mathematical Foundations (`proved`)

Let $T : [0, n) \to [0, n)$ be an $r$-piece discrete IET ($r \ge 2$) given by domain partition $0 = d_0 < d_1 < \dots < d_r = n$ into intervals $I_k = [d_k, d_{k+1})$ of width $w_k = d_{k+1} - d_k$, each translated onto an image interval $J_k = [t_k, t_k + w_k)$ with literal word $W_k \in \Sigma^+$, where $\{J_k\}_{k=0}^{r-1}$ partitions $[0, n)$ with endpoints $I = \{t_k : 0 \le k < r\} \setminus \{0\}$ and domain cutpoints $D = \{d_k : 1 \le k < r\}$. Define the **piece-overlap digraph** $G(T)$ on $\{0, \dots, r-1\}$ with directed arc $k \to v$ iff $J_k \cap I_v \neq \emptyset$. Call piece $u$ **unsplit** if $\text{out\_deg}_{G(T)}(u) = 1$, and a **splitter** if $\text{out\_deg}_{G(T)}(u) \ge 2$. Let $U = \{u : \text{out\_deg}(u) = 1\}$ and $K = \{k : \text{out\_deg}(k) \ge 2\}$.

### Theorem 3.1 (Pigeonhole Unsplit-Piece Theorem — `proved`)
For every $r$-piece discrete IET ($r \ge 2$),
$$|U| \ge |D \cap I| + 1 \ge 1, \qquad |K| \le r - 1 - |D \cap I|.$$
**Proof.** Each domain cutpoint $d \in D$ (there are $|D| = r - 1$ interior cutpoints) either coincides with an image boundary $d \in D \cap I$ (splitting zero image intervals) or lies strictly inside a unique image interval $J_k$ (incrementing $\text{out\_deg}(k)$ by $1$). Thus
$$\sum_{k=0}^{r-1} \bigl(\text{out\_deg}(k) - 1\bigr) = |D \setminus I| = (r - 1) - |D \cap I|.$$
Every splitter $k \in K$ contributes $\text{out\_deg}(k) - 1 \ge 1$ to the left-hand side, while every unsplit piece $u \in U$ contributes $0$. Hence $|K| \le r - 1 - |D \cap I|$, so $|U| = r - |K| \ge |D \cap I| + 1 \ge 1$. $\square$

*Remark (tightness & empirical collapse on Wikidata):* For a maximal-run BWT, $D$ consists of the $r - 1$ run boundaries in $L$, while $I$ consists of the $r - 1$ run-image boundaries in $F$ (which include all $|\Sigma| - 1$ symbol bucket boundaries!). Even more strongly, whenever an image interval $J_k$ contains $s_k \ge 2$ cutpoints of $D$, it absorbs $s_k - 1$ *extra* split points, forcing at least $s_k - 1$ additional pieces to be unsplit! Because Wikidata has thick runs whose images absorb dozens of thin-run cutpoints, $|K|$ collapses from $r = 51{,}184–128{,}732$ to only **$3{,}789–9{,}041$ ($7.0\%–7.5\%$ of $r$)**, leaving **$92.5\%–93.0\%$ of all runs in $U$**.

### Theorem 3.2 (F-Order Conjugation & Zero-Random-Access Round-1 Build — `proved`)
Let $T = \mathrm{LF}$ on the $r$ maximal runs of $L$, ordered in $L$-order ($k = 0, \dots, r-1$) and in $F$-order ($\mathrm{fi} = 0, \dots, r-1$, where $\mathrm{fi} = \text{f\_of\_l}[k]$ is the stable rank of $L$-run $k$ grouped by symbol $c \in [0, 256)$).
1. Every $F$-interval $J_{\mathrm{fi}}$ is contiguous in increasing order of $\mathrm{fi} = 0, \dots, r-1$, and every $L$-interval $I_v$ is contiguous in increasing order of $v = 0, \dots, r-1$.
2. Therefore a single **linear two-pointer merge** of $\mathrm{fi} \in [0, r)$ against $v \in [0, r)$ classifies every run $\mathrm{fi}$ as unsplit ($\text{succ}[\mathrm{fi}] = \text{f\_of\_l}[v]$, $\delta[\mathrm{fi}] = t_0(\mathrm{fi}) - d_v$) or splitter ($\mathrm{fi} \in K$, emitting all overlap arcs $(\mathrm{fi}, v)$) in $O(r + A)$ time with **zero random reads and zero sorting**.
3. For every $u \in U$, its functional step in $F$-order corresponds to applying $T$ once: $x \in I_u \mapsto T(x) = d_v + \delta[u] + (x - d_u) \in I_v$, accumulating the byte $L[I_u] = \text{f\_sym}[u]$ in backward LF order.

**Proof.** In $F$, all occurrences of symbol $c$ form a contiguous block in stable $L$-order, so the images $J_k$ of runs with symbol $c$ appear consecutively in $F$ in the exact order they occur in $L$. Thus the map $k \mapsto \mathrm{fi} = \text{f\_of\_l}[k]$ (computed in one sequential counting pass over $L$-runs) sorts $\{J_k\}$ in strictly increasing $F$-coordinate order. Merging the sorted partition $\{J_{\mathrm{fi}}\}$ against the sorted partition $\{I_v\}$ visits each interval boundary once ($O(r + A)$ total steps) and directly yields $\text{succ}[\mathrm{fi}] = \text{f\_of\_l}[v]$ whenever $J_{\mathrm{fi}} \subseteq I_v$. $\square$

### Theorem 3.3 (Width-Monotonicity Theorem — `proved`)
If $u \in U$ (unsplit) has width $w_u$ and $\text{succ}(u) = v$, then $w_v \ge w_u$. Consequently, for any threshold $W \ge 1$, the subset $V_{\ge W} = \{k : w_k \ge W\}$ is **forward-closed** under $\text{succ}$ on $U$.

**Proof.** Since $u \in U$, $J_u \subseteq I_v$, so $w_u = |J_u| \le |I_v| = w_v$. $\square$

### Theorem 3.4 (Exact First-Return Induction on $K$ & Quotient Self-Overlap Acceleration — `proved`)
1. **Functional in-forest contraction:** Since $G(T)|_U$ has out-degree $1$ on every vertex, each connected component of $U$ is either (a) a directed tree rooted at a splitter $r(u) \in K$, or (b) a pure functional cycle wholly inside $U$.
   - If the marked row $p$ lies in a piece $u \in U$ on a pure $U$-cycle $u_0 \to u_1 \to \dots \to u_{L-1} \to u_0$, then every $x \in I_{u_0}$ returns to $I_{u_0}$ at offset $x - d_{u_0} + \sum_i \delta[u_i]$. Because $T$ is a bijection and $I_{u_0}$ maps into itself rigidly, $\sum_i \delta[u_i] = 0$ and the exact marked cycle word is $\text{rev}(W_{u_0} W_{u_1} \cdots W_{u_{L-1}})$ immediately (`CLOSED`).
   - Otherwise, every splitter arc $(k, [a, b) \to I_v)$ follows the unique path $v = u_0 \to u_1 \to \dots \to u_t = r \in K$, shifting offset by $\Delta(v) = \sum_{j=0}^{t-1} \delta[u_j]$ and prepending $W_{u_{t-1}} \cdots W_{u_0}$ onto $W_k$. Compacting the union $\bigcup_{k \in K} I_k$ to $[0, n_K)$ ($n_K = \sum_{k \in K} w_k < n$) yields an exact first-return IET $T_K$ on $[0, n_K)$ whose marked orbit (rotated by `mark_shift`, the byte length from $p$ to its first entry into $K$) equals the exact marked orbit of $T$.
2. **Quotient Self-Overlap Acceleration:** If a piece $k$ satisfies $0 < |t_k - d_k| < w_k$ with $d = |t_k - d_k|$ and $w_k \ge 2d$, then writing $w_k = q d + \text{rem}$ ($q \ge 2$, $0 \le \text{rem} < d$), the interior interval of width $w_k - d$ (`[d_k + d, d_k + w_k)` if $t_k > d_k$, or `[d_k, d_k + w_k - d)` if $t_k < d_k$) is strictly contained in both $I_k$ and $J_k$, hence disjoint from all other pieces' domains and images. Eliminating this interior compresses $[0, n)$ by $w_k - d$ in $O(r + \log q)$ time, replacing piece $k$ by at most two exit pieces carrying $W_k^q$ and $W_k^{q+1}$ (via binary exponentiation in $O(\log q)$ SLP rules) and updating `mark_shift` by $\text{steps\_to\_exit} \cdot |W_k|$ if $p$ lay in the eliminated interior.

**Proof.** Standard first-return invariance of a bijection $T$ to a transversal $X$ (here $X = \bigcup_{k \in K} I_k$ or $X = [0, n) \setminus \text{interior}_k$), tracking exact backward-LF word concatenation (`left` = later LF step = earlier text bytes) and shifting the marked starting coordinate by the exact number of bytes emitted along the prefix from $p$ to its first entry in $X$. $\square$

---

## 4. Gate G1 & Gate G2 Verification (`verified`)

### 4.1 Gate G2 Acquisition Manifest (`PASS`)
Verified via `python3 docs/team/acquisition/check_manifest.py docs/journal/2026-11-20-neon-cycle4-unsplit-forest/ufc.toml`:
- **`PASS`** — every dependency traces cleanly from $I_0$ (`records`, `p`, `n`), with explicit $(W, D, B, Q, M)$ accounting on every pass.

### 4.2 Gate G1 Exhaustive, Adversarial, and 6-Block Verification (`PASS`)
Executed via `python3 docs/journal/2026-11-20-neon-cycle4-unsplit-forest/verify_ufc.py`:
- **Exhaustive small cases:** **`10,454 / 10,454` PASS** (`5,227` `UFC-Full` + `5,227` `UFC-1+Open` at `LIMIT = 2`; all binary $n \le 8$ and ternary $n \le 5$ across every $p \in [0, n)$, including `1,062` quotient-accelerated cases, multi-cycles, periodic blocks, and early $U$-cycle closures).
- **Adversarial cases (`7 / 7` PASS for both `UFC-Full` and `UFC-1+Open`):**
  - `Sturmian(1,99999)`: `3` rounds, `1` quotient accel, `25` SLP rules (`UFC1+Open`: `m1=2, strips=1`).
  - `Sturmian(99999,1)`: `1` round, `0` SLP rules.
  - `Sturmian(7,99993)`: `6` rounds, `2` quotient accels, `30` SLP rules (`UFC1+Open`: `m1=8, strips=2`).
  - `Fibonacci(6765,10946)`: `20` rounds, `1` quotient accel, `19` SLP rules (`UFC1+Open`: `m1=1, strips=3`).
  - `AllZero(10000)`, `Periodic(abc x 1000)`, `MultiCycle(010101)`: `1` round, `0` SLP rules.
- **Six authenticated Wikidata blocks (`5,399,692 / 5,399,692` bytes exact for both `UFC-Full` and `UFC-1+Open`):**

| Block | $r$ | $\|K\|$ (`%` of $r$) | R1 arcs $A$ | R1 hops $H_U$ | R1 flat lit | `UFC-Full` rds / SLP rules | `UFC-Full` mean copy | Open $m$ | `UFC-1+Open` $m_1$ (`strips`) |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| `block-0.L` | 51,184 | 3,789 (7.4%) | 14,743 | 66,429 | 79 KiB | 12 / 46,736 (`0.91r`) | 19.3 B | 89,874 | 26,124 (`1,302`) |
| `block-2.L` | 54,532 | 4,074 (7.5%) | 17,297 | 70,434 | 85 KiB | 13 / 54,791 (`1.00r`) | 16.4 B | 112,070 | 35,867 (`1,436`) |
| `block-85.L` | 79,375 | 5,526 (7.0%) | 22,645 | 100,237 | 120 KiB | 11 / 65,800 (`0.83r`) | 13.7 B | 156,404 | 41,946 (`592`) |
| `block-143.L` | 108,335 | 7,746 (7.2%) | 34,710 | 135,763 | 166 KiB | 15 / 93,826 (`0.87r`) | 9.6 B | 196,841 | 70,082 (`2,046`) |
| `block-207.L` | 87,111 | 6,233 (7.2%) | 27,036 | 107,156 | 131 KiB | 15 / 74,694 (`0.86r`) | 12.0 B | 135,921 | 45,058 (`1,372`) |
| `block-210.L` | 128,732 | 9,041 (7.0%) | 35,789 | 153,375 | 184 KiB | 11 / 99,811 (`0.78r`) | 9.0 B | 204,281 | 63,397 (`1,076`) |

*Additional measured forest statistics (`measured` via Python diagnostic on all 6 blocks):* `100%` of all unsplit nodes $|U| = r - |K|$ are visited during Round-1 arc tracing (`hops / |U| = 1.28–1.40×` unmemoized sharing factor); `74.4%–78.7%` of unsplit nodes have exact offset $\delta = 0$, while `6.2%–8.5%` have $\delta \ge 31$ (`10,402–22,332` hops hit $\delta \ge 31$).

---

## 5. Gate G5 Confrontation (`measured`)

**Pre-registered PASS/FAIL rule (fixed before running either C falsifier):** PASS iff on ALL 6 blocks vs Shipped Forward Open: (1) byte-exact, (2) `L2m <= 0.85x`, (3) `DRAM B/B <= 0.90x`, (4) `PeakMem <= 1.15x`, (5) uninstrumented `-O3` VM C time `<= 0.90x` (diagnostic only, not an i9 production proof).

### 5.1 Primary Falsifier: [`g5_ufc.c`](2026-11-20-neon-cycle4-unsplit-forest/g5_ufc.c) (`UFC-Full` & Initial `UFC-1+Open` vs Shipped Open)
Built with `cc -O3 -Wall -Wextra -Werror` (zero warnings) and `-DNO_CACHE_SIM` (median of 15 interleaved rounds). All 6 blocks `100%` byte-exact (`5,399,692/5,399,692` B).

| Block | Open L2m / DRAM / Peak / ms | `UFC-Full` L2m / DRAM / Peak / ms | `UFC-1+Open` L2m / DRAM / Peak / Work / ms |
|---|---:|---:|---:|
| `block-0.L` | 75.6k / 4.83 / 2.19 MB / 2.53 | **2.62×** / 1.85× / 0.89× / 4.32 (1.70×) | **1.48×** / 1.47× / **0.58×** / **0.87×** / **2.34 (0.92×)** |
| `block-2.L` | 97.6k / 5.42 / 2.44 MB / 2.15 | **2.36×** / 1.87× / 0.93× / 4.66 (2.17×) | **1.38×** / 1.44× / **0.58×** / **0.80×** / 2.42 (1.13×) |
| `block-85.L` | 103.4k / 5.60 / 2.58 MB / 2.40 | **2.97×** / 2.38× / 1.12× / 5.72 (2.38×) | **1.66×** / 1.71× / **0.76×** / **0.80×** / 2.88 (1.20×) |
| `block-143.L` | 199.1k / 7.91 / 3.44 MB / 4.05 | **2.83×** / 2.76× / 1.24× / 9.45 (2.33×) | **1.61×** / 1.90× / **0.82×** / **0.89×** / 4.80 (1.19×) |
| `block-207.L` | 118.4k / 5.92 / 2.69 MB / 2.60 | **3.21×** / 2.63× / 1.25× / 6.71 (2.58×) | **1.75×** / 1.88× / **0.83×** / **0.96×** / 3.39 (1.30×) |
| `block-210.L` | 166.9k / 7.13 / 3.22 MB / 4.48 | **3.44×** / 3.34× / 1.39× / 9.82 (2.19×) | **1.87×** / 2.30× / **0.96×** / **0.93×** / 4.53 (1.01×) |

**Verdict (`g5_ufc.c`):** `UFC-Full` = **FAIL** (Rounds 2..15 `R2+ = 87k–304k` L2m, `1.75–4.03 ms` + random SLP `out`-to-`out` expansion `1.21–3.13 ms`); `UFC-1+Open` = **FAIL** on L2m/DRAM despite winning Peak RAM (`0.58–0.96×`) and Total Work (`0.80–0.96×`) on all 6 blocks and beating Open in VM time on `block-0.L` (`0.92×`).

### 5.2 Single Justified Incidental Layout Repair: [`g5_ufc1_compact.c`](2026-11-20-neon-cycle4-unsplit-forest/g5_ufc1_compact.c)
Initial `g5_ufc.c` had two incidental prototype bloats: (i) `emit` split each step across 4 random arrays (`edges_nxt`, `edges_lit`, `lit_off`, `lit_spell`), causing `1.73` L2 misses/step on `block-143.L` even though `68.2–74.9%` of thin pieces have `lit_len <= 5` B; (ii) Round 1 allocated `20r` bytes (`l_from`, `f_of_l`, `f_w`, `forest`, `big_delta`). Under `METHOD.md` §2, we executed **one** targeted repair (`g5_ufc1_compact.c`): `2 B/run` `l_cw` + `1 B/run` `f_w8` + `big_d` overflow hash + `8 B` `u64` `edges` inlining `<= 5 B` literal words (`39.5%–47.7%` of all emit steps hit the pure inline fast path without any thick-strip excursion). Built with `cc -O3 -Wall -Wextra -Werror` (zero warnings), 100% byte-exact on all 6 blocks:

| Block | Open `emit` L2m (`ms`) | `UFC1-Cmp` `emit` L2m (`ms`) | `UFC1-Cmp` `R1` L2m (`13` + `4`) | `UFC1-Cmp` Total L2m | DRAM | Work | Peak | VM time |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| `block-0.L` | 44.9k (`0.82`) | **30.2k (`0.67×`) / `0.57` (`0.70×`)** | 46.9k (`25.6k + 21.3k`) | 105.4k (`1.46×`) | `1.46×` | **`0.83×`** | **`0.66×`** | 2.73 ms (`1.10×`) |
| `block-2.L` | 57.6k (`0.73`) | **37.2k (`0.65×`) / `0.56` (`0.77×`)** | 52.6k (`28.2k + 24.4k`) | 124.6k (`1.37×`) | `1.44×` | **`0.77×`** | **`0.64×`** | 2.63 ms (`1.31×`) |
| `block-85.L` | 65.6k (`0.94`) | **34.7k (`0.53×`) / `0.64` (`0.68×`)** | 80.6k (`40.1k + 40.5k`) | 160.8k (`1.56×`) | `1.67×` | **`0.77×`** | **`0.75×`** | 3.17 ms (`1.42×`) |
| `block-143.L` | 121.0k (`1.53`) | **75.6k (`0.62×`) / `1.20` (`0.78×`)** | 127.6k (`57.6k + 70.1k`) | 278.6k (`1.55×`) | `1.81×` | **`0.85×`** | **`0.75×`** | 5.14 ms (`1.34×`) |
| `block-207.L` | 72.5k (`0.94`) | **43.1k (`0.59×`) / `0.70` (`0.74×`)** | 94.3k (`45.9k + 48.4k`) | 192.1k (`1.72×`) | `1.82×` | **`0.91×`** | **`0.80×`** | 3.75 ms (`1.49×`) |
| `block-210.L` | 111.4k (`1.73`) | **54.3k (`0.49×`) / `1.17` (`0.68×`)** | 148.2k (`64.0k + 84.2k`) | 278.4k (`1.71×`) | `2.09×` | **`0.89×`** | **`0.86×`** | 5.59 ms (`1.31×`) |

*(Note: `g5_ufc1_compact.c`'s embedded Shipped Open baseline omitted a few `ep` auxiliary-array `sim_access` calls present in `g5_ufc.c`, lowering baseline `ep` L2m by `0.5k–19.7k` while `p1`/`wir`/`emit` match within `1%`; in both harnesses `UFC1-Cmp` fails L2m by `1.37–1.72×` and DRAM by `1.44–2.09×`.)*

**Verdict (`g5_ufc1_compact.c`):** **FAIL** — and the phase split reveals that the failure is **structural, not incidental**.

---

## 6. Two Structural Obstructions Discovered

### Obstruction 6.1: Run-Graph Producer–Consumer Parity Obstruction
- **Statement:** After subtracting mandatory input-read (`recs`, `~2–4k` lines) and output-write (`out[0..n)`, `~14.1k` lines) cache misses, Shipped Open completes the **entire** exact inverse-BWT reconstruction at only **`1.08–1.49` L2 misses per BWT run $r$** (`block-0`: `1.10`, `block-2`: `1.38`, `block-85`: `1.08`, `block-143`: `1.49`, `block-207`: `1.08`, `block-210`: `1.12`). On Wikidata, **`62.8%–75.4%` of all $r$ runs have width $w = 1$** (`34,262–97,054` runs; `100%` trivially unsplit). Consequently, any two-stage contraction that builds and traverses an $r$-vertex run graph (`forest[0..r]` = `200–515 KiB`, `|U| = 0.47–0.59m`, `hops = 0.63–0.75m`) spends **`0.92–1.18` L2 misses/run in the producer alone** (`R1`: `46.9k–148.2k` L2m, plus `wir` radix sort `16.4k–51.0k` L2m because contraction destroys Open's free 256-symbol queue order). On `block-210.L`, even the irreducible floor of `R1 Pass 1–3` (`~53k` streaming L2m) + `R1 Pass 4` (`~54k–69k` random L2m in `515 KiB`) + `emit` (`~45k` L2m) exceeds Shipped Open's entire `163k` L2m budget—swallowing the genuine `33–51%` `emit` L2m reduction (`57k` L2m saved on `block-210.L` at a producer cost of `+154k` L2m in `R1 + wir`).
- **Witness:** [`g5_ufc1_compact.c`](2026-11-20-neon-cycle4-unsplit-forest/g5_ufc1_compact.c) and [`g5_ufc.c`](2026-11-20-neon-cycle4-unsplit-forest/g5_ufc.c) on all 6 authenticated Wikidata blocks (`block-210.L`: `r = 128,732`, `w = 1` runs = `97,054` [`75.4%`], `R1` L2m = `148,169` vs `emit` savings = `57,075`).
- **Scope:** Any run-level pre-contraction or multi-stage permutation reduction (`UFC-1`, `UFC-Full`, `Bipolar`, `Half-Bipolar`) that indexes and randomly traverses all $r$ maximal runs (including width-1 runs) before emitting, whenever $r / m \gtrsim 0.45$ and `4r` approaches or exceeds L2 (`256 KiB`).
- **Escape condition:** Either (a) **never touch width-1 runs in a contraction graph** (operate *only* on structural runs $w \ge W$, which requires defeating Obstruction 6.2 below so `spelling` does not blow up), OR (b) eliminate Open's `emit` misses **in-place without a second $r$-scale random graph** (e.g., an L2-resident algebraic/output-sourced mechanism — Foundational Question #5 / `OPEN-QUESTIONS.md` §4 — or an orthogonal representation that beats `1.1` L2 misses per BWT run).

### Obstruction 6.2: Thin-Excursion Spelling Conservation Obstruction
- **Statement:** On all 6 Wikidata blocks, runs of width $w \in [2, 16]$ account for `52.5%–69.4%` of Open's thin rows $m$ despite being only `12,620–30,710` runs (`18%–32%` of $r$). By Theorem 3.3, $V_{\ge 2}$ and $V_{\ge 3}$ are forward-closed under `succ`. However, moving $w \in [2, 16]$ out of `edges` into Open's first-return `endpoint` (`LIMIT: 16 -> 1`) shrinks `edges` (`4m`) by `219–497 KiB` while **inflating flattened `spelling + offsets` by `+500–670 KiB`** (`32–58 KiB -> 495–623 KiB`, `bands: 2.4k–6.7k -> 12.5k–30.9k`) and increasing `ep` steps `7–10×` (`7k–21k -> 50k–118k`), **increasing** the total random-access `emit` working set on every block:

| Block | `LIMIT = 16`: `m` / `strips` / `ep_steps` / `spell` | `LIMIT = 2`: `m` / `strips` / `ep_steps` / `spell` | `LIMIT = 1`: `m` / `strips` / `ep_steps` / `spell` |
|---|---:|---:|---:|
| `block-0.L` | 89.9k / 3.4k / 11.2k / **52 KiB** | 46.8k / 10.2k / 32.6k / **343 KiB** | 35.1k / 16.1k / 50.3k / **561 KiB** |
| `block-2.L` | 112.1k / 3.8k / 13.7k / **49 KiB** | 45.2k / 14.8k / 42.9k / **346 KiB** | 34.3k / 20.3k / 60.3k / **583 KiB** |
| `block-85.L` | 156.4k / 1.6k / 7.1k / **32 KiB** | 69.5k / 16.9k / 46.8k / **228 KiB** | 55.5k / 23.9k / 73.7k / **495 KiB** |
| `block-143.L` | 196.8k / 5.0k / 21.3k / **58 KiB** | 102.4k / 20.8k / 78.3k / **371 KiB** | 72.6k / 35.8k / 118.3k / **623 KiB** |
| `block-207.L` | 135.9k / 3.5k / 13.9k / **56 KiB** | 88.6k / 11.6k / 47.7k / **309 KiB** | 62.4k / 24.7k / 81.9k / **539 KiB** |
| `block-210.L` | 204.3k / 2.8k / 12.1k / **39 KiB** | 130.8k / 14.8k / 59.4k / **254 KiB** | 97.1k / 31.7k / 107.4k / **507 KiB** |

- **Witness:** Exact `ep_stats` measurement across `LIMIT ∈ {16, 4, 2, 1}` on all 6 authenticated Wikidata blocks (matching `#657` from first principles).
- **Scope:** Any first-return elimination that absorbs low-multiplicity runs ($w \in [2, 16]$) into a flattened excursion-spelling table (`spelling + offsets`).
- **Escape condition:** Represent multi-run excursions **without materialising flattened `spelling` strings** (e.g., copying directly from already-emitted output `out[0..n)` — Foundational Question #5 / `OPEN-QUESTIONS.md` §4 — or compact grammar/back-reference emission whose random footprint stays $\ll 256\text{ KiB}$).

---

## 7. Cycle 4 Lessons & Redirection for Cycle 5

Cycle 4 proves a clean mathematical construction (`UFC` — eliminating Rauzy–Veech induction and orbit walks via the Pigeonhole Unsplit-Piece Theorem) and pinpoints **exact quantitative bounds** on Obligation #1:
- **Positive finding:** `UFC-1 + Open(cmp)` genuinely cuts `emit` L2 misses by `33–51%`, `emit` VM time by `22–32%`, `ep` time by `16–52%`, Peak RAM by `14–36%`, and Total Work by `9–23%` on all 6 blocks; and `UFC-Full` collapses entire 900 KB BWTs to a `0.78–1.00r` SLP in `11–15` rounds with zero RV induction and zero orbit walk.
- **Why it redirects Cycle 5:** Both Cycle 3 (`BSSE` -> LLC-Resident Streaming-Exchange Obstruction) and Cycle 4 (`UFC` -> Run-Graph Producer–Consumer Parity + Thin-Excursion Spelling Conservation) narrow the escape space sharply: any future candidate must **not** build an $r$-scale contraction graph over width-1 runs, must **not** move `> 40 B/row` in streaming passes, and must **not** inflate flattened `spelling` above L2. The cleanest unexplored escapes in `FRONTIER.md` point directly to:
  1. **Foundational Question #5 / `OPEN-QUESTIONS.md` §4 (Output-sourced / self-sourcing decoding from `out[0..n)` using information Open *already* holds, e.g., bands / LCP-chains / $\Phi$-adjacent rows!)** — replacing random lookups in `edges` / `spelling` by reusing bytes already written to `out`!
  2. **Foundational Question #1 / Stochastic Complementation / Parallel-Front Orbit Breaking (without extra $m$-scale queues or $r$-scale graphs)** — e.g., breaking Open's `emit` directly inside its existing `4m` table or exploiting unexplored mathematical structure (`LCP-chains` / Kärkkäinen–Kempa–Piątkowski, `Durand` return substitutions, or `Meyer` stochastic complements with non-width sections).
