# Carbon Cycle 8: The 4-Point Alternating Gauge Obstruction, Whole-Block vs Thick-Complement Monoid Separation, and Conjugate $F$-Space Bandless Schur Elimination (`CFSE`)

*20 November 2026. Session: Carbon (`research/carbon-word-transport`). Mode: Reorientation (default, `METHOD.md` §3a).*

## 1. Cycle Orientation & Summary of Outcomes

At the Cycle 7 → 8 boundary, read-only inspection of `origin/master` (`25e0097`) surfaced two decisive calibration facts (`2026-11-20-open-stage-attribution.md`, `9d92de8` / `5e05b89`):
1. Below `LAZY_FROM_THREADS = 8`, production runs eager `Cells` (`365–371 ms` across all 431 Wikidata blocks), whereas `Open` runs from 8 workers upward (`436 → 388 ms` with Cycle 2's `U16Q`, a confirmed `11.2%` whole-corpus kernel speedup).
2. Within Cycle 7's `GIM-RV + BVSMC-2P` (`1.00–2.01 ms` per 900 KB block), exact sub-phase profiling on the six authenticated blocks showed that thick Rauzy–Veech elimination (`RV-Elim = 0.086–0.281 ms`) is no longer dominant: **`RunQ + Wire` (`0.287–0.576 ms`, `52–71%` of `construct`)** and **2-pass `BVSMC-2P` emission (`0.441–0.864 ms`)** dominate residual decode time, while `queues[256]` + `nodes` + `bands` + `offsets` hold **`3.05–4.25 MiB`** of scratch per worker (`3.51–4.79 MiB` in `Open-qi16`).

Cycle 8 generated five distinct mechanisms across both tracks, triaged three quantitatively before Gate G5, and took **Theorem 8.1**, **Theorem 8.2**, and **Conjugate $F$-Space Bandless Schur Elimination (`CFSE`, Theorem 8.3)** through executable verification (`verify_cycle8.py`), manifest checking (`cfse.toml`), AddressSanitizer/UBSan verification, and a paired 25-round 6-arm C confrontation (`g5_cfse.c`):

1. **Theorem 8.1 — The 4-Point Alternating Gauge (Christie–Gates) Obstruction to Sub-Orbit Cut Potentials (`proved` & `verified`):**
   Proved that flipping $L[u..u\!+\!2] = ab \mapsto ba$ and $L[v..v\!+\!2] = ba \mapsto ab$ ($a \ne b$, $u + 1 < v$) leaves $\mathrm{LF}(x)$ identical on **all $n - 4$ rows** $x \notin \{u, u+1, v, v+1\}$, acts by the even 2-transposition $(u \;\; u\!+\!1)(v \;\; v\!+\!1) \in A_n$, and preserves single-$n$-cycle primitivity **if and only if** the two adjacent pairs interleave (alternate) around the cycle of $\mathrm{LF}$ (`verified` exhaustively across `1,086` switches on binary $n \le 8$ and ternary $n \le 6$, plus explicit true primitive BWT macro-swap witnesses at $n = 1,111$ with BWT Hamming distance `4`). This establishes an exact $\Omega(m)$ sensitivity lower bound on cut-point permutation $\sigma \in S_K$ and segment byte lengths $(B_0, \dots, B_{K-1})$, rigorously separating 2-pass zero-arena splicing (`BVSMC-2P`, $2m$ loads, $0\text{ B}$ pool) from 1-pass buffered splicing (`BVSMC-1P`, $1m$ loads, $\Theta(n)$ buffer).
2. **Theorem 8.2 — Word-Length Spectrum Separation Between Whole-Block `Cells` and Thick-Complement `Open` (`verified` & `measured`, killed in Triage):**
   Tested generalising Cycle 7's graded inline monoid $\mathcal{M}_K$ ($K \in \{16, 32, 48, 64\}$) + Bipolar Single-Tape allocation + wide `0.15` admission to whole-block eager `Cells` (`slots + induce_cells + write_owned`) across all six blocks (including `block-143` and `block-210`, where shipped `Cells` refuses and falls back to `Segmented`'s `12.1–15.4 ms` `T`-vector walk). While Bipolar Single-Tape + `0.15` admission rescues `block-143`/`block-210` to `2.40–2.85 ms`, raising $K$ from $16 \to 64$ leaves total time flat (`1.037 → 1.041 ms` on `block-0`, `2.404 → 2.372 ms` on `block-143`) because whole-block RV lacks thin-run absorption boundaries, inflating `arena` to `763–1,738 KiB` (`> L2`). Moreover, `GIM-RV + BVSMC-2P` (`1.006–2.006 ms`) **already beats eager `Cells` (`1.037–2.85 ms`) on 6/6 blocks even at 1 thread**.
3. **Theorem 8.3 & `CFSE` — Conjugate $F$-Space Bandless Schur Elimination (`verified` & `measured`, Gate G5 `FAIL` on strict speed bar, major architectural memory breakthrough for `OPEN-QUESTIONS.md` §3):**
   By conjugating `Open`'s backward-in-text return map $g_L = \beta \circ \alpha : [0, m)_L \to [0, m)_L$ by $\alpha = \mathrm{LF}_{\text{thin}}$ into **$F$-space** ($g_F = \alpha \circ \beta : [0, m)_F \to [0, m)_F$) and aliasing the sequential stream $\text{alpha\_raw}[0..m]$ **directly inside dead `out[0..n)`** (`4(m+16) \le n` holds on **6/6** blocks, up to `m = 204,281` / `817 KiB` on `block-210`), `CFSE` **eliminates six intermediate heap vectors** (`queues[256]`, `order[0..1]`, `terminals[0..1]`, `bands`, `offsets`) and cuts per-worker scratch memory in **half**: **`1,559–2,058 KiB` vs `3,510–4,793 KiB` (`Open-qi16`) and `3,046–4,251 KiB` (`GIM-RV+BVSMC-2P`)** (`-1.49 to -2.76 MiB/worker`!).
   - Unlike `#659` (`Backward Open`, `1.10–1.15×` slower than `Open-qi16`), `CFSE` requires **only one pass over `records`**, beating `Open-qi16` on **6/6 blocks** even in pure non-SIMD scalar mode (`0.943–0.978×`), beating `Open+U16Q-1L` on **6/6 blocks** in 1-lane SIMD mode (`0.869–0.943×` `Open+U16Q-1L`, `0.734–0.775×` `Open-qi16`, accelerating *both* `construct` and 1-lane `emit`), and reaching **`0.630–0.735×` `Open-qi16`** in zero-queue, zero-pool 8-lane `CFSE+BVSMC-2P`.
   - Pre-registered Gate G5 verdict is kept strictly as **`FAIL`** because `CFSE+BVSMC-2P` ran `2.6–12.0%` slower than `GIM-RV+BVSMC-2P` at 1 thread (`1.041–2.118 ms` vs `1.006–2.006 ms`), exposing the **Synchronous Scattered-Load vs Store-Buffer Absorption Asymmetry** (`edges_F` gathers `alpha_raw[dst]` synchronously across `817 KiB` L2, whereas `edges_L` streams sequential `queues[a]` and retires scattered stores `edges_L[dst]` asynchronously into the 56-entry store buffer).

---

## 2. Four Mandatory Candidate Fields (`METHOD.md` §3a)

- **Candidate:** Conjugate $F$-Space Bandless Schur Elimination (`CFSE`)
- **Mode:** Reorientation (changes the retained coordinate domain from $[0, m)_L$ to $[0, m)_F$, changes emission orientation to right-to-left over right-aligned XMM monoid words, and replaces 256-bucket `queues` + `bands` + `offsets` with `out`-aliased sequential `alpha_raw` + bandless in-flight wiring).
- **Displaced obligations:**
  1. `Open`'s 256-bucket `queues: Vec<Vec<u32>>` inversion scatter (`~600–850 KiB` across 256 heap blocks) AND `#659` (`Backward Open`)'s second pass over `records[0..R]`;
  2. `Open`'s `bands: Vec<Band>` intermediate buffer and two-pass `BandSpell → Wire` materialisation;
  3. `Open`'s `offsets: Vec<u32>` table and `spelling` duplication for all $\le 16\text{ B}$ words (`93%+` of bands);
  4. Cold RFO write misses on `out[0..4m]` during `emit` (`out` is pre-dirtied in `M`-state by `alpha_raw`).
- **Replacement principle:** Factor $g_L = \beta \circ \alpha$ ($\alpha = \mathrm{LF}_{\text{thin}} : [0, m)_L \to [0, m)_F$, $\beta = P : [0, m)_F \to [0, m)_L$) and conjugate by $\alpha$ to $g_F = \alpha \circ \beta : [0, m)_F \to [0, m)_F$ starting at $k_{\text{mark}} = \alpha(\text{mark}_L)$ (Theorem 8.3). Write $\text{alpha\_raw}[j] = (c \ll 20) \mid \text{rank}_c(j)$ sequentially into dead `out[0..n)` during the single `records` scan; bound non-literal word codes *a priori* from surviving non-zero $\mathcal{M}_{64}$ nodes ($W_{\max} \le N_{\neq 0} + 256$); build right-aligned $\le 16\text{ B}$ XMM words directly from `small_arena` via `vmovdqu` + `vpinsrb` (storing `(o0, ln)` inside `inline16_ra[code]` when $\ell > 16$); wire `edges_F[kc .. kc+w]` sequentially; and emit right-to-left via `_mm_storeu_si128(P - 16, inline16_ra[cd])` strictly inside `[out + off, out + end)`.
- **New paid obligations:**
  1. During bandless wiring (`kc = 0..m-1`), reading contiguous band slices `alpha_raw[dst .. dst+w]` across $360\text{--}817\text{ KiB}$ and adding `thin_start[v >> 20]` (`1 KiB` L1 table) per retained row;
  2. Right-to-left 8-lane segment writes (`P -= ln`) across `out[0..n)` instead of forward writes (`P += ln`).
- **Nearest prior art:** Neon Cycle 2's Queue-Free Backward Open (`#659`, `6ccab45`, which kept domain $[0, m)_L$ and paid a second full pass over `records[0..R]` with per-run band-splitting `while` loops, losing at `1.10–1.15× Open-qi16`), `Cells`' `lay.bout` dead-`out` bottom-slot aliasing (`src/cells.rs:281`), and Carbon Cycle 7's `GIM-RV` ($\mathcal{M}_{64}$ thick monoid, which still paid `queues[256]`, `bands[]`, `offsets[]`, and full `spelling[]`).

---

## 3. Foundational Theorems & Triage Falsifications

### Theorem 8.1 — The 4-Point Alternating Gauge (Christie–Gates) Obstruction (`proved` & `verified`)

**Statement (`proved`):** Let $L \in \Sigma^n$ be any column of length $n \ge 4$ containing disjoint adjacent pairs $L[u..u\!+\!2] = ab$ and $L[v..v\!+\!2] = ba$ with $a \ne b$ and $u + 1 < v$. Define $L'$ by flipping $L'[u..u\!+\!2] = ba$ and $L'[v..v\!+\!2] = ab$, with $L'[x] = L[x]$ elsewhere. Then:
1. **Strict $(n - 4)$-Row Locality:** The symbol histogram $C_{L'} = C_L$ is unchanged, and for every $x \notin \{u, u+1, v, v+1\}$, the prefix occurrences $\mathrm{occ}_{L'}(c, x) = \mathrm{occ}_L(c, x)$ are identical (for $u + 2 \le x < v$, $\{u, u+1\}$ contributes one $a$ and one $b$ in both $L$ and $L'$). Consequently,
   $$\mathrm{LF}_{L'}(x) = \mathrm{LF}_L(x) \quad \forall x \notin \{u, u+1, v, v+1\}, \qquad \mathrm{LF}_{L'} = \mathrm{LF}_L \circ (u \;\; u\!+\!1)(v \;\; v\!+\!1).$$
2. **Exact Single-Cycle Alternation Law:** Suppose $\mathrm{LF}_L$ is a single $n$-cycle with cyclic order $\tau : [0, n) \xrightarrow{\sim} \mathbb{Z}_n$ ($\tau(\mathrm{LF}_L(x)) \equiv \tau(x) + 1 \pmod n$). Composing $\mathrm{LF}_L$ with $(u \;\; u\!+\!1)$ splits the $n$-cycle into two disjoint cycles separating $u$ and $u+1$ into the complementary arcs $(\tau(u), \tau(u+1)]_{\mathbb{Z}_n}$ and $(\tau(u+1), \tau(u)]_{\mathbb{Z}_n}$. Composing further with $(v \;\; v\!+\!1)$ merges those two cycles back into a **single $n$-cycle** if and only if one of $\{\tau(v), \tau(v+1)\}$ lies in each arc — i.e., $\{u, u+1\}$ and $\{v, v+1\}$ **alternate (interleave)** on $\mathbb{Z}_n$, reversing/transposing two macro-segments of the orbit.
3. **True Primitive BWT & Cut-Potential Lower Bound:** Even restricted to true BWTs of primitive strings ($L = \mathrm{BWT}(T), L' = \mathrm{BWT}(T')$), Eulerian 2-opt exchanges across two repeated order-$k$ right-contexts $Z_1, Z_2$ ($T_a = \$ U \mathtt{1} Z_1 V \mathtt{1} Z_2 W \mathtt{2} Z_1 Q \mathtt{2} Z_2$ vs $T_b = \$ U \mathtt{1} Z_1 Q \mathtt{2} Z_2 W \mathtt{2} Z_1 V \mathtt{1} Z_2$) produce primitive strings of identical length whose BWTs differ in **exactly 4 rows** (`Hamming(La, Lb) = 4`), whose `Open` tables `edges[0..m]` differ in $\le 4$ entries, yet swap macro-segments $V$ and $Q$ of arbitrary lengths $\Theta(n)$. Hence any zero-error algorithm determining the cyclic order $\sigma \in S_K$ ($K \ge 3$) or exact byte lengths $(B_0, \dots, B_{K-1})$ between $K$ cut-points must inspect $m - O(1)$ entries of the retained map before placing the displaced segments in `out`. $\square$

**Executable Verification (`verified`, `verify_cycle8.py`):**
- Checked all `1,086` valid 4-point switches across binary $4 \le n \le 8$ and ternary $4 \le n \le 6$: `1,086/1,086` obey $(n-4)$-row invariance and the exact alternation law (`32` single-cycle-preserving macro-swaps, `104` cycle-splits).
- Verified explicit true primitive BWT pair $(T_a, T_b)$ at $n = 1,111$ (`|V| = 410`, `|Q| = 130`): `Hamming(BWT(T_a), BWT(T_b)) == 4`, `pa == pb`.

---

### Theorem 8.2 & Triage Kills (`GM-Cells` & `SIMD16-RunQ`)

Before Gate G5, two distinct candidates were triaged with executable C probes on all six blocks:

1. **Whole-Block Graded-Monoid Eager `Cells` (`GM-Cells(K)`, $K \in \{16, 32, 48, 64\}$, Bipolar Single-Tape, Wide `0.15` Admission):**
   - *Measured (`scratch/triage_cycle8.c`, 7 reps/block):*
     - Opening eager `Cells` to `0.15` with a single-tape bipolar buffer rescues `block-143` (`R/n = 0.125`) from `Segmented`'s `14.67 ms` cliff down to `2.40 ms` (`live = 3.61 MiB`).
     - However, sweeping $K = 16 \to 64$ cuts binary rules `nr` by only `1.89–2.53×` (`28,710 → 15,199` on `block-0`; `50,042 → 19,798` on `block-143`) while inflating `arena` from `315–714 KiB` to `763–1,738 KiB` (`> L2`), shifting time from `write_owned` (`0.242 → 0.151 ms`) into `induce_cells` (`0.566 → 0.665 ms`) for **zero net gain** (`1.037 → 1.041 ms` on `block-0`; `2.404 → 2.372 ms` on `block-143`).
     - Head-to-head on the same VM, `GIM-RV + BVSMC-2P` (`1.006–2.006 ms`, `live ~ 3.0–4.2 MiB`, or `1.56–2.06 MiB` in `CFSE`) **already beats eager `Cells` on 6/6 blocks at 1 thread** (`28%` faster on `block-85`, `17%` faster on `block-143`). *Killed at Triage.*
2. **16-Record AVX2 Prefix-Sum + Bitmask Partitioning on `RunQ` (`SIMD16-RunQ`):**
   - *Measured (`/tmp/probe_dars.c`, 25 reps/block):* Replacing scalar record coalescing and `width <= 16` branching with 16-lane AVX2 `exscan16_u16` + `tzcnt` dispatch yielded only `0.931–0.987×` `ScalarU16Q` (`0.483 → 0.450 ms` on `block-210`, `0.214 → 0.211 ms` on `block-0`). At `2.10 GHz`, `ScalarU16Q` already runs at `7.0–8.3 cycles/run`, bound by single-port unaligned store occupancy across 256 queues rather than front-end branching (`< 10%` G4 headroom alone). *Killed at Triage.*

---

### Theorem 8.3 — Conjugate $F$-Space Bandless Schur Elimination (`CFSE`, `proved` & `verified`)

**Statement (`proved`):** Let $\alpha = \mathrm{LF}_{\text{thin}} : [0, m)_L \xrightarrow{\sim} [0, m)_F$ map each retained $L$-row $j$ (of byte $c$) to $\text{thin\_start}[c] + \text{rank}_{\text{thin}, c}(j)$, and let $\beta = P : [0, m)_F \xrightarrow{\sim} [0, m)_L$ be the thick-complement first-return map with band words $W(k) \in \Sigma^+$ ($k \in [0, m)_F$). Then:
1. **Conjugate Orbit Equivalence:** The backward-in-text retained return map $g_L = \beta \circ \alpha : [0, m)_L \to [0, m)_L$ conjugates under $\alpha$ to
   $$g_F \;=\; \alpha \circ g_L \circ \alpha^{-1} \;=\; \alpha \circ \beta \;:\; [0, m)_F \xrightarrow{\sim} [0, m)_F, \qquad k_0 = \alpha(\text{mark}_L).$$
   Iterating $k_{t+1} = g_F(k_t)$ from $k_0$ until $k_t = k_0$ and placing each left-to-right word $W(k_t)$ **right-to-left** from the end of the period $[0, \text{clen})$ reproduces the exact cyclic period of $T$ through $p$ (`repeat_period` tiles $\text{clen} < n$ to $n$).
2. **Single-Pass Dead-`out` Aliasing & Bandless Wiring:**
   - Writing $\text{alpha\_raw}[j] = (c \ll 20) \mid \text{rank}_{\text{thin}, c}(j)$ ($m < 2^{20}$) sequentially for $j = 0 \dots m-1$ requires $4(m + 16)$ bytes (with 16-entry SIMD tail slack), which fits strictly inside dead `out[0..n)` whenever $4(m + 16) \le n$ (**verified on 6/6 blocks**, where $m/n \in [0.100, 0.227]$ so $4m/n \in [0.400, 0.908]$).
   - Counting surviving nodes with $\text{word} \ne 0$ ($N_{\neq 0}$) during the `bottom` scan bounds total band words *before* band generation by $W_{\max} \le N_{\neq 0} + 256$, determining `bits` and `wide` *a priori* so `edges_F[kc .. kc+w]` can be wired directly without materialising `bands[]`.
   - Seeding literals at `small_arena[32..288]` ensures every $\mathcal{M}_{64}$ word of length $\text{wl} \in [1, 15]$ at offset $\text{woff} \ge 32$ has $\ge 16$ readable bytes ending at $\text{woff} + \text{wl}$, so `vmovdqu(arena + woff + wl - 15)` + `vpinsrb(a, 15)` constructs right-aligned `inline16_ra[code]` in 2 XMM instructions with **zero `spelling[]` or `offsets[]` entries** (while $\ell > 16$ words store `{o0, ln}` directly in `inline16_ra[code]`).
   - During right-to-left segment emission over `[out + off, out + end)`, guarding `P >= out + off + 16` ensures `_mm_storeu_si128(P - 16, inline16_ra[cd])` writes `[P - 16, P) \subseteq [out + off, out + end)` with zero left padding and zero inter-segment clobbering. $\square$

**Executable Verification (`verified`, `verify_cycle8.py` & `cfse.toml`):**
- `check_manifest.py docs/journal/2026-11-20-carbon-cycle8/cfse.toml`: **PASS**.
- `verify_cycle8.py`: **PASS** across **`5,227/5,227` small `(L, p)` pairs $\times$ 2 limits $\times$ 2 consumers = `20,908` runs**, **`8/8` adversarial controls** (including `32k` all-width-16 `out`-scratch fallback and `50k` periodic shear $>64\text{ B}$ `GimRule` DAG), and **`6/6` authenticated Wikidata blocks** (`5,399,692` B, `5.30s`).

| Block | $m$ | $H$ | Words | `GimRule`s | `Open` Spell (KiB) | `CFSE` Long-Only Spell (KiB) | `out`-Aliased `alpha_raw` |
|---|---:|---:|---:|---:|---:|---:|---|
| `block-0.L` | 89,874 | 3,448 | 1,326 | 534 | 50.8 | 46.8 (`1.09×`) | YES (`351 KiB <= 879 KiB`) |
| `block-2.L` | 112,070 | 3,827 | 1,922 | 507 | 48.2 | 41.2 (`1.17×`) | YES (`438 KiB <= 879 KiB`) |
| `block-85.L` | 156,404 | 1,565 | 1,220 | 227 | 30.9 | 26.7 (`1.16×`) | YES (`611 KiB <= 879 KiB`) |
| `block-143.L` | 196,841 | 5,047 | 3,635 | 301 | 56.8 | 42.0 (`1.35×`) | YES (`769 KiB <= 879 KiB`) |
| `block-210.L` | 204,281 | 2,809 | 2,228 | 255 | 37.8 | 29.1 (`1.30×`) | YES (`798 KiB <= 879 KiB`) |
| `block-207.L` | 135,921 | 3,474 | 2,259 | 360 | 54.6 | 45.3 (`1.21×`) | YES (`531 KiB <= 879 KiB`) |

---

## 4. Gate G3 Cost Vector & Gate G5 Confrontation (`g5_cfse.c`)

### Producer–Consumer Cost Accounting (Gate G3)
- **`W` (Work):** $O(R + H + m + n)$, single record scan (`0` second record pass), `0` `EpRule` allocations on $\le 64\text{ B}$ joins, `0` `offsets` lookups.
- **`D` (Dependency depth):** $O(R)$ sequential `alpha_raw` scan + $O(H)$ RV steps + $O(m/8)$ 8-lane `BVSMC-2P` steps.
- **`B` (Bytes moved):** Eliminates writing + reading `queues[256]` (`2 × 4m`) in separate heap pages and `bands[]` (`12B × bands`); replaces with sequential `out[0..4m]` write + band-sliced `out[0..4m]` read (pre-dirtying `out` in cache).
- **`Q` (Irregular accesses):** `RunScan` drops from `256` concurrent write streams to **`1` monotonic stream**; `Wire` shifts from `B` scattered *writes* (`edges_L[dst]`) to `B` scattered *reads* (`alpha_raw[dst]`).
- **`M` (Peak live scratch memory):** **`1,559–2,058 KiB`** (`1.52–2.01 MiB`) per worker vs **`3,510–4,793 KiB`** (`Open-qi16`) and **`3,046–4,251 KiB`** (`GIM-RV+BVSMC-2P`) — eliminating `queues[256]`, `order[2]`, `terminals[2]`, `bands`, and `offsets` (`-1.49 to -2.76 MiB/worker`).

### Paired 25-Round 6-Arm Confrontation (`measured`, `g5_cfse.c`, Intel Xeon @ 2.10 GHz cloud VM)

*Note (`METHOD.md` §5): Single-thread VM C timings are calibrated kernel diagnostics, not proof of 20-thread native Rust throughput. Verified 100% clean under `-fsanitize=address,undefined` across all 8 adversarial self-tests and all 6 blocks.*

| Block | Arm | `Con` (ms) | `Emit` (ms) | `Tot` (ms) | `Con / Open` | `Tot / Open` | Scratch (KiB) |
|---|---|---:|---:|---:|---:|---:|---:|
| `block-0.L` | `Open-qi16` | 0.930 | 0.530 | 1.459 | 1.000× | 1.000× | 3,509.8 |
| | `CFSE-Scalar-1L` | 0.843 | 0.538 | 1.381 | 0.906× | 0.946× | **1,559.1** |
| | `Open+U16Q-1L` | 0.753 | 0.516 | 1.262 | 0.810× | 0.865× | 3,509.8 |
| | `CFSE-SIMD-1L` | 0.606 | **0.490** | **1.100** | 0.652× | **0.753×** | **1,559.1** |
| | `GIM-RV+BVSMC-2P` | 0.568 | 0.435 | 1.006 | 0.611× | 0.689× | 3,045.8 |
| | `CFSE+BVSMC-2P` | 0.599 | 0.434 | 1.041 | 0.644× | **0.714×** | **1,559.1** |
| `block-2.L` | `Open-qi16` | 1.112 | 0.716 | 1.825 | 1.000× | 1.000× | 3,733.1 |
| | `CFSE-Scalar-1L` | 1.015 | 0.726 | 1.736 | 0.913× | 0.951× | **1,654.5** |
| | `Open+U16Q-1L` | 0.889 | 0.660 | 1.554 | 0.800× | 0.851× | 3,733.1 |
| | `CFSE-SIMD-1L` | 0.716 | **0.633** | **1.351** | 0.643× | **0.740×** | **1,654.5** |
| | `GIM-RV+BVSMC-2P` | 0.663 | 0.534 | 1.198 | 0.596× | 0.657× | 3,230.8 |
| | `CFSE+BVSMC-2P` | 0.722 | 0.627 | 1.342 | 0.649× | **0.735×** | **1,654.5** |
| `block-85.L` | `Open-qi16` | 0.992 | 0.906 | 1.889 | 1.000× | 1.000× | 3,985.9 |
| | `CFSE-Scalar-1L` | 0.949 | 0.901 | 1.847 | 0.956× | 0.978× | **1,765.0** |
| | `Open+U16Q-1L` | 0.663 | 0.883 | 1.519 | 0.668× | 0.804× | 3,993.9 |
| | `CFSE-SIMD-1L` | 0.597 | **0.834** | **1.432** | 0.602× | **0.758×** | **1,765.0** |
| | `GIM-RV+BVSMC-2P` | 0.550 | 0.695 | 1.243 | 0.554× | 0.658× | 3,518.6 |
| | `CFSE+BVSMC-2P` | 0.589 | 0.721 | 1.313 | 0.594× | **0.695×** | **1,765.0** |
| `block-143.L` | `Open-qi16` | 1.849 | 1.504 | 3.363 | 1.000× | 1.000× | 4,722.8 |
| | `CFSE-Scalar-1L` | 1.731 | 1.520 | 3.236 | 0.936× | 0.962× | **2,057.5** |
| | `Open+U16Q-1L` | 1.423 | 1.348 | 2.770 | 0.769× | 0.824× | 4,722.8 |
| | `CFSE-SIMD-1L` | 1.177 | **1.288** | **2.468** | 0.637× | **0.734×** | **2,057.5** |
| | `GIM-RV+BVSMC-2P` | 1.119 | 0.890 | 2.006 | 0.605× | 0.596× | 4,222.0 |
| | `CFSE+BVSMC-2P` | 1.188 | 0.936 | 2.118 | 0.643× | **0.630×** | **2,057.5** |
| `block-210.L` | `Open-qi16` | 1.482 | 1.411 | 2.895 | 1.000× | 1.000× | 4,792.6 |
| | `CFSE-Scalar-1L` | 1.369 | 1.404 | 2.778 | 0.924× | 0.960× | **1,992.3** |
| | `Open+U16Q-1L` | 1.113 | 1.367 | 2.474 | 0.751× | 0.855× | 4,792.6 |
| | `CFSE-SIMD-1L` | 0.955 | **1.297** | **2.243** | 0.645× | **0.775×** | **1,992.3** |
| | `GIM-RV+BVSMC-2P` | 0.927 | 0.879 | 1.816 | 0.626× | 0.627× | 4,251.1 |
| | `CFSE+BVSMC-2P` | 0.959 | 0.903 | 1.864 | 0.647× | **0.644×** | **1,992.3** |
| `block-207.L` | `Open-qi16` | 1.288 | 0.912 | 2.203 | 1.000× | 1.000× | 4,512.5 |
| | `CFSE-Scalar-1L` | 1.174 | 0.906 | 2.078 | 0.912× | 0.943× | **1,754.4** |
| | `Open+U16Q-1L` | 1.019 | 0.844 | 1.853 | 0.791× | 0.841× | 4,512.5 |
| | `CFSE-SIMD-1L` | 0.840 | **0.794** | **1.632** | 0.652× | **0.741×** | **1,754.4** |
| | `GIM-RV+BVSMC-2P` | 0.806 | 0.594 | 1.389 | 0.626× | 0.631× | 4,018.4 |
| | `CFSE+BVSMC-2P` | 0.839 | 0.673 | 1.523 | 0.651× | **0.692×** | **1,754.4** |

---

## 5. What Cycle 8 Taught About the Obligations (`METHOD.md` §3a)

1. **Synchronous Scattered-Load vs Store-Buffer Absorption Obstruction (Obligation #2 — Building the retained map):**
   - **Witness:** `CFSE` (`g5_cfse.c` `Arm 5` vs `Arm 4`): `RunScan` saved `0.04–0.13 ms` over `U16Q` by replacing 256 scattered queue streams with 1 sequential `out`-aliased stream, yet bandless $F$-space wiring lost `+0.07–0.16 ms` (`Con = 0.599 vs 0.568 ms` on `block-0`; `0.959 vs 0.927 ms` on `block-210`).
   - **Scope:** Any queue-free conjugate retained-map constructor (`edges_F[kc] = pack | alpha(dst+s)`) on out-of-order CPUs with large store buffers and L2-resident queues. In $L$-space (`GIM-RV`), `Wire` reads sequential `queues[a]` and retires scattered writes `edges_L[dst+s]` asynchronously into the 56-entry store buffer (`8`-wide AVX2 `vpor`). In $F$-space (`CFSE`), `Wire` writes sequential `edges_F[kc+s]` but stalls synchronously on scattered L2 reads `alpha_raw[dst+s]` across `360–817 KiB` plus per-row `thin_start[v >> 20]` translation.
   - **Escape condition:** Either (a) multi-worker L3/DRAM contention (`8–20` threads, `OPEN-QUESTIONS.md` §3) where cutting per-worker scratch in **half** (`4.2 MiB → 1.9 MiB`, `-16 to -28 MiB` across workers, plus pre-dirtying `out` to eliminate cold RFO reads) outweighs `0.03–0.06 ms` of L2 read latency, OR (b) keeping $L$-space `GIM-RV` wiring while replacing `256` over-allocated `queues[256]` `Vec`s with a compact slab/arena inside dead `out[0..n)` (explored next!).
2. **1-Lane $F$-Space Gain vs 8-Lane Backward Streamer Asymmetry (Obligation #1 & #3):**
   - At 1 lane, `CFSE-SIMD-1L` beats `Open+U16Q-1L` on **6/6 blocks** (`0.869–0.943×` `Open+U16Q-1L`, `0.734–0.775×` `Open-qi16`), including `4–6%` faster 1-lane emission from `out` RFO pre-warming. At 8 lanes, backward writes (`P -= ln`) + symbol-clustered cut spacing `[0, m)_F` concede `0.00–0.09 ms` to forward $L$-space `BVSMC-2P`.
   - Crucially, **Bandless Direct-Arena Wiring + Dual-Purpose `inline16` (`0 B` `bands[]`, `0 B` `offsets[]`, long-only `spelling`)** transfers *directly* to forward $L$-space `GIM-RV` as well!
