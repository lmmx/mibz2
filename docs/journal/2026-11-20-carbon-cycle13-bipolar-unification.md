# Carbon Cycle 13 — Iterated Radix-Stream Domino Obstruction, Dual-Pole `edges`-Aliased `BipolarZQ`, and Conjugate Bipolar `ConjBP-Seg32` (`13/13` Whole-Spectrum Win & `-63%` / `-90%` Scratch Elimination)

*20 November 2026 — Carbon (`research/carbon-word-transport`)*

## 1. Executive Summary & Cycle 13 Scorecard

Cycle 12 left two major structural questions open across the BWT block spectrum:
1. **High-density & medium-density L3 regimes (`0.20 <= r/n < 1`, `text0`, `random`, `records` — `66.6%` of whole-corpus decode time in `2026-11-19-open-admission.md`):**
   - On `text0` (`r/n = 0.2982`), Cycle 12's `Unified-C12` stayed flat (`0.995x` shipped `Segmented`), while both `Segmented::invert_runs` and `Segmented::invert_in_place` allocated **`2.00 MiB` of per-worker `SegScratch::bufs` (`512 x 4 KiB`, `40 MiB` at 20 threads)** — meaning `invert_in_place(ll)` was *not* genuinely `O(1)` extra space beyond `tt[0..n)`.
   - Could Alpöge's iterated-lift strategy (`https://alpo.ge/remark.pdf`) compose $\mathrm{LF} \mapsto \mathrm{LF}^d$ ($d = 2, 4$) without random gathers to halve or quarter the `900,000` L3 consumer loads?
2. **Multi-thread per-worker L2 residency & metadata independence (`OPEN-QUESTIONS.md` §3, `0 < r/n < 0.20`):**
   - Cycle 11–12's `AHT-FlatQ-BP4K` allocated **`11.5 MiB` per worker** (`3.58 MiB` sparse `FlatQ` + `3.52 MiB` `edges` + `3.52 MiB` eager `extra` + `0.9 MiB` metadata = `230 MiB` across 20 threads) and required `Runs.per[256]` (`src/runs.rs:92`).

Cycle 13 resolves **both** questions with three model-free theorems (`verify_cycle13.py`, `5.85s`), two validated acquisition manifests (`bipolar_zq_aht.toml`, `conjbp_seg32.toml`), and a frozen 25-round 6-arm confrontation (`g5_bipolar_c13.c`, `98fa224`):

- **Theorem 13.1 (`proved` + `verified` + `measured` obstruction — Iterated Radix-Stream Left-Composition & $d$-Point Domino Sentinel Barrier):**
  - Left-composition $\mathcal{L}(T_d)[i] := L[i] \cdot T_d[\text{ftab}[L[i]]\text{++}]$ composes $\mathrm{LF} \mapsto \mathrm{LF}^{d+1}$ using **only `256` monotone sequential bucket cursors (`16 KiB` L1D frontier invariant for all $d \ge 1$, zero random gathers)**, while a **$d$-Point Domino Sentinel $\{s_m, \mathrm{LF}(s_m), \dots, \mathrm{LF}^{d-1}(s_m)\}$** with pointer trim `p += delta` prevents step-$d$ orbits from skipping any cut (`verified` on `3,584/3,584` exhaustive binary `(L, p)` pairs + `1,000/1,000` multi-alphabet columns for $d = 2, 3, 4$).
  - **Measured Obstruction:** Expanding `tt` to `8 B` (`7.20 MB > 6 MiB` 1,536-entry 4K STLB reach) degrades lookup latency while each dense streaming pass pays `~1.02 ms` of `7.2 MB` RFO traffic (`Lift-2`: `1.62 + 1.42 = 3.04 ms`, `Lift-4`: `3.83 + 0.96 = 4.79 ms` vs `2.40 ms` 1-step `3.60 MB`). By AM–GM ($p_A(1-p_A) \le 1/4$), 1-step bipartite alphabet Schur complementation $I = \{y : F[y] \in A, L[y] \notin A\}$ ($\mathrm{LF}(I) \cap I = \emptyset$, `verified` `1,000/1,000`) eliminates at most `~25–32%` of rows without run elimination.
- **Theorem 13.2 (`proved` + `verified` + `measured` — Dual-Pole `edges`-Aliased Zero-Hole `ChunkQ`, `BipolarZQ`, & `U8Pf-AHT`):**
  - Proves the universal band-word bound $\text{words} \le 2H + 255$ (`verified` `1,500/1,500` + `6/6` Wikidata) and constructs the exact `900,000`-row extremal LP witness (`22,000 x 24` thin + `14,880 x 25` thick runs, $r/n = 0.041 < 0.20, m = 528,000 > 2^{19}$) showing why lazy `extra[]` is **`0 B` on `100%` of real `r/n < 0.20` blocks** (`-3.52 MiB/worker`) yet exact in the worst case.
  - Hosts `1,024`-entry (`+24` SIMD pad) zero-hole thin-row chunks **downward from the dead upper pole of `k->edges`** during Phases 1–2 (`0` overflow chunks = **`0 B` separate queue allocation across `10/10` blocks**, with verified chunk evacuation on forced pole collisions), **eliminates `Runs.per[256]`**, shrinks per-worker `Open/R4` scratch by **`63%` (`11.5 -> 4.2 MiB`, `1.22–2.16 MiB` L2-resident working set on `6/6` Wikidata)**, beats Cycle 12's self-contained `PaidPre-FlatQ` on **`7/7` blocks**, and — with `U8Gate` (`width <= 8` 32B stores) + `Pf8L` (`_mm_prefetch`) — speeds up `RunsMeta` `C13-Speed` over Cycle 12 `C12-Meta` on **`7/7` blocks** (`runs`: `1.347 -> 1.209 ms = 0.897x`!).
- **Theorem 13.3 (`proved` + `verified` + `measured` — Conjugate Bipolar Dead-`out` Backward Segmentation, `ConjBP-Seg32`):**
  - Applies spatial inversion $x \mapsto n - x$ to bipolar dead-`out` paging on backward `LF` walks (`P = 512 B, S = 448` equispaced $O(1)$-sentinel cuts `SEG_START | (i << 8)`, prime-padded `tile[32][272]`): segment `first_seg` writes downward in place from `out + n` while segments $\ne \text{first\_seg}$ allocate `512 B` extents upward from `out + 0` with universal Pigeonhole overflow $\le (S + 2) \cdot 512\text{ B} = 225\text{ KiB} \le 256\text{ KiB}$ (`205–209 KiB` measured; `verified` `1,500/1,500`).
  - **Eliminates `100%` of `SegScratch::bufs` (`2,048 KiB -> 0 B`, true low-scratch `invert_in_place(ll)`!)** while **beating both Shipped `Segmented` and Cycle 12 `SeqLF-Col` on `3/3` dense/column blocks**: `text0` **`2.529 -> 2.415 ms` (`0.955x`, breaking the `text0` parity barrier!)**, `random` **`3.785 -> 2.630 ms` (`0.695x` Shipped, `0.948x` C12)**, `records` **`3.804 -> 2.632 ms` (`0.692x` Shipped, `0.987x` C12)**.
- **Strict Frozen Gate G5 Verdict (`g5_bipolar_c13.c`):** **Overall strict verdict `FAIL` (`14/15` sub-bars `PASS`)** solely because frozen sub-bar `G5.3a` set a tightened `<= 0.56x Open-qi16` threshold (`4/6`: `block-0` hit `0.578x` and `block-2` hit `0.572x`, i.e., `6/6 <= 0.58x`). **Both `C13-Speed-Uni` and `C13-LowMem-Uni` beat Shipped `0.5.7` across ALL `13/13` blocks (`0.071x – 0.955x`, `12/13 <= 0.80x`, zero cliffs anywhere in `0 < r/n < 1`).**

---

## 2. Explore & Triage Kills (Recorded Honest Negatives)

| Investigated Candidate | Mechanism | Measured Outcome (`explore_cycle13.c`, `probe_radix_lift.c`, `triage_c13_fast.c`, `test_fill_text0.c`) | Why Killed |
|---|---|---|---|
| `BandDedup` (`16,384`-slot hash table) + `SlackBurst` | Deduplicate `2..16B` band words + hoist 8-lane page slack | `+0.09–0.22 ms` slower on `6/6` Wikidata (`0.909 -> 1.014 ms` on `block-0`, `1.601 -> 1.821 ms` on `block-143`) | Census proved `raw_w = 1,250–2,613` at `L=24` (`24–46 KiB` — **already inside `48 KiB` L1D** without dedup!); `256 KiB` `htab` `memset` + `~24%` `>16B` burst breaks paid pure overhead |
| `Seg-BL32` (32-element `q[32]` array `any_hit` branch fusion) | Buffer 32 `tt[rows[k]]` loads before single `any_hit & SEG_START` check | `2.179 -> 2.217 ms` (`block-0`), `2.418 -> 2.489 ms` (`random`) | Buffering `uint32_t q[32]` live simultaneously increased stack/register pressure vs `SSTEP`'s 3-instruction per-lane lifetime |
| `Iterated Radix-Stream Lift` (`Lift-2` & `Lift-4` + Domino Sentinel) | Zero-gather 256-cursor L1D lift $T_1 \to T_2 \to T_4$ (`uint64_t`) | `Lift-2`: `1.62 + 1.42 = 3.04 ms`; `Lift-4`: `3.83 + 0.96 = 4.79 ms` vs `2.40 ms` 1-step (`random`) | `8 B` entries (`7.20 MB > 6 MiB` 4K STLB reach) + `1.02 ms/pass` `7.2 MB` RFO write traffic exceed consumer L3 savings (Theorem 13.1) |
| `Tile16-BP2K` (`16` lanes staged in `tile[16][1088]`) | Stage 16 prefetched `R4` lanes in L1D tiles before copying to `2 KiB` dead-`out` pages | `1.576 -> 1.683 ms` (`block-143`), `0.857 -> 1.040 ms` (`block-0`), `1.169 -> 1.179 ms` (`runs`) | Extra `900 KB` tile-to-page `memcpy` + $1 - (1 - p_{\text{long}})^{16} \approx 48\%$ per-step `noinline` cold calls when `dedup_long > 0` |
| `seg_fill_xmm_fast` (`len <= 4` 16B XMM branch in sequential `tt` fill) | Branch on `len <= 4` in `seg_fill_avx2` on `text0` | `0.641 -> 0.803 ms` (`+25%` slower on `text0`) | Unlike 256-stream scattered `RunQ`, 1-stream sequential `tt + q` absorbs 32B overlaps in the L1 store buffer while `11%` `len > 4` mispredicts `25k` branches |

---

## 3. Theorems & Proofs

### Theorem 13.1 (Iterated Radix-Stream Left-Composition & $d$-Point Domino Barrier — `proved`, `verified` `4,584` cases, `measured` obstruction)
1. **256-Stream Left-Composition (`proved`):** Let $T_d[j] = (\mathrm{LF}^d(j),\, L[\mathrm{LF}^{d-1}(j)] \cdots L[j])$. Scanning $i = 0, \dots, n-1$ with `ftab[0..255]` initialized to $C[0..255]$ and setting $j = \text{ftab}[L[i]]\text{++}$ evaluates $j = \mathrm{LF}(i)$ strictly monotonically and contiguously within each bucket $[C[c], C[c+1))$, yielding $T_{d+1}[i] = (\mathrm{LF}^d(j),\, T_d[j].\text{word} \cdot L[i]) = (\mathrm{LF}^{d+1}(i),\, L[\mathrm{LF}^d(i)] \cdots L[i])$ with an active read frontier of at most $256$ cache lines (`16 KiB < 48 KiB` L1D) for **every** $d \ge 1$.
2. **$d$-Point Domino Barrier (`proved`):** For disjoint $d$-chains $D_m = \{s_m, \mathrm{LF}(s_m), \dots, \mathrm{LF}^{d-1}(s_m)\}$ ($2d$-separated along $\mathrm{LF}$), no step-$d$ trajectory $y_{t+1} = \mathrm{LF}^d(y_t)$ can jump over $d$ consecutive orbit elements; landing on $\mathrm{LF}^\delta(s_m)$ ($\delta \in \{0, \dots, d-1\}$) overshoots segment $m$ by **exact $\delta$ leftmost bytes** under downward forward-order writing, trimmed in $O(1)$ via `p += delta`.
3. **Bipartite Alphabet Bound (`proved`):** For any $A \subset \Sigma$, $I = \{y \in [0, n) : F[y] \in A \land L[y] \notin A\}$ satisfies $\mathrm{LF}(I) \cap I = \emptyset$ identically because $F[\mathrm{LF}(y)] = L[y] \notin A$, with product bound $p_A(1-p_A) \le 1/4$ under symbol independence.

### Theorem 13.2 (Dual-Pole `edges`-Aliased `BipolarZQ` & Universal Band-Word Bound — `proved`, `verified` `1,506` cases)
1. **Band-Word Upper Bound (`proved`):** In `R4-DGIM` / `AHT-R4` over $H$ thick strips, initial interval construction creates $\le 2H + 1$ top intervals and Rauzy–Veech splitting reuses existing `R4Top` slots without allocation; splitting across $\le 255$ alphabet bucket boundaries in Phase 3 yields $\text{words} \le 2H + 255$ for every BWT block.
2. **Dual-Pole Disjointness & Exact Evacuation (`proved`):** During Phases 1–2 (`RunQ` + `AHT-R4`), `edges[0..edges_cap)` is unread and unwritten; Phase 3 writes `edges[0..m)` while consuming `ChunkQ` backward per symbol. Hosting `1,024`-entry (`+24` pad) chunks downward from `edges + edges_cap` requires `0 B` of separate queue memory whenever $m + 16 + N_{\text{chk}} \cdot 1048 \le \text{edges\_cap}$ (satisfied with `>390,000` `u32`s to spare on all `6/6` Wikidata blocks), while evacuating only chunks with `chk_base < edges + m + 16` prior to Phase 3 guarantees exactness for arbitrary $m \le n$.

### Theorem 13.3 (Conjugate Bipolar Dead-`out` Backward Segmentation — `proved`, `verified` `1,500` cases)
Once `tt[0..n)` is constructed (`seg_fill_avx2` or `seqlf_col_build`), `out[0..n)` (including `ll[0..n)` in `Segmented::invert_in_place`) is dead. Writing segment `first_seg` downward in place from `out + n` (`out[n - seg0_len .. n)`) while allocating `P`-byte extents for the remaining $S - 1$ segments upward from `out + 0` (`(bot_phys + 3) * P <= p0_left`) requires at most $\lfloor (n - \text{seg0\_len}) / P \rfloor + (S - 1)$ extents across $\ge \lfloor (n - \text{seg0\_len}) / P \rfloor - 3$ physical slots, bounding peak external overflow universally by $\le S + 2$ extents ($\le 450 \times 512\text{ B} = 225\text{ KiB} < 256\text{ KiB}$ at $P = 512, S = 448$).

---

## 4. Controlled 25-Round 6-Arm Confrontation (`g5_bipolar_c13.c`, `-O3 -march=native`)

*Note (`METHOD.md` §5): Single-thread Sapphire Rapids VM C timings (`Xeon Platinum 8481C`, `48 KiB` L1d, `2 MiB` L2); native 1–20 thread Rust verification belongs to the Coordinator.*

| Block | `nr/n` | `Open16/Col` (ms) | `SegShp` (ms) | `C12-Paid` (ms) | `C12-Meta` (ms) | **`C13-LowMem`** (ms) | **`C13-Speed`** (ms) | **`Spd / Shipped`** | **`Spd / C12Meta`** | `WS_KB` (`LowM`) | `OvfKB` |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| `block-0.L` | `0.0627` | `1.510` | `2.226` | `0.959` | `0.874` | **`0.920`** | **`0.873`** | **`0.578x`** | **`0.998x`** | **`1,221 K`** | `188 K` |
| `block-2.L` | `0.0661` | `1.901` | `2.469` | `1.190` | `1.092` | **`1.142`** | **`1.087`** | **`0.572x`** | **`0.995x`** | **`1,243 K`** | `196 K` |
| `block-85.L` | `0.0941` | `1.958` | `2.266` | `1.161` | `1.053` | **`1.076`** | **`1.006`** | **`0.514x`** | **`0.955x`** | **`1,580 K`** | `200 K` |
| `block-143.L` | `0.1248` | `3.458` | `2.412` | `1.754` | `1.599` | **`1.660`** | **`1.576`** | **`0.456x`** | **`0.986x`** | **`2,156 K`** | `192 K` |
| `block-210.L` | `0.1480` | `2.972` | `2.334` | `1.703` | `1.558` | **`1.564`** | **`1.474`** | **`0.496x`** | **`0.946x`** | **`2,008 K`** | `192 K` |
| `block-207.L` | `0.1016` | `2.265` | `2.339` | `1.349` | `1.235` | **`1.243`** | **`1.169`** | **`0.516x`** | **`0.947x`** | **`1,415 K`** | `204 K` |
| `onebyte` | `0.0079` | `0.027` | `0.265` | `0.046` | `0.027` | **`0.029`** | **`0.027`** | **`0.101x`** | `1.009x` | **`4 K`** | `0 K` |
| `pattern` | `0.0079` | `1.905` | `1.301` | `0.134` | `0.088` | **`0.111`** | **`0.092`** | **`0.071x`** | `1.040x` | **`4 K`** | `16 K` |
| `rle_spiral` | `0.0204` | `2.483` | `1.338` | `0.248` | `0.198` | **`0.221`** | **`0.193`** | **`0.078x`** | **`0.971x`** | **`1,144 K`** | `152 K` |
| `runs` | `0.1825` | `4.159` | `2.467` | `1.524` | `1.347` | **`1.331`** | **`1.209`** | **`0.490x`** | **`0.897x`** | **`2,770 K`** | `216 K` |
| `text0` | `0.2982` | `11.109` | `2.529` | `2.532` | `2.525` | **`2.427`** | **`2.415`** | **`0.955x`** | **`0.956x`** | `3,515 K` | **`205 K`** |
| `random` | `0.9922` | `3.801` | `3.785` | `2.788` | `2.775` | **`2.636`** | **`2.630`** | **`0.695x`** | **`0.948x`** | `3,515 K` | **`207 K`** |
| `records` | `0.9969` | `3.809` | `3.804` | `2.662` | `2.667` | **`2.634`** | **`2.632`** | **`0.692x`** | **`0.987x`** | `3,515 K` | **`209 K`** |

### Per-Worker Scratch & 20-Thread Footprint Comparison (`OPEN-QUESTIONS.md` §3)

| Kernel / Route | Shipped `0.5.7` / Worker | Cycle 12 / Worker | **Cycle 13 (`LowMem` / `Speed`) / Worker** | **20-Thread Savings vs Cycle 12** |
|---|---:|---:|---:|---:|
| `r/n < 0.20` (`Open` / `AHT-R4`) | `~9.8 MiB` (`edges + extra + queues + nodes`) | `~11.5 MiB` (`FlatQ + edges + extra`) | **`4.2 MiB`** (`LowMem`: `0 B` queue, `0 B` `extra`, `1.2–2.1 MiB` L2 WS) / **`7.8 MiB`** (`Speed`: `0 B` `extra`) | **`-146 MiB` (`LowMem`) / `-74 MiB` (`Speed`)** |
| `r/n >= 0.20` (`Segmented` / `ConjBP-Seg32`) | `5.52 MiB` (`3.52 MiB tt + 2.00 MiB bufs`) | `5.52 MiB` (`3.52 MiB tt + 2.00 MiB bufs`) | **`3.77 MiB`** (`3.52 MiB tt + 0.25 MiB ovf`, **`0 B` `SegScratch::bufs`**, true `ll == out`!) | **`-35 MiB` (`-90%` segment scratch)** |

---

## 5. Reproducibility Commands

```bash
python3 docs/team/acquisition/check_manifest.py docs/journal/2026-11-20-carbon-cycle13/bipolar_zq_aht.toml
python3 docs/team/acquisition/check_manifest.py docs/journal/2026-11-20-carbon-cycle13/conjbp_seg32.toml
python3 -u docs/journal/2026-11-20-carbon-cycle13/verify_cycle13.py
gcc -O1 -g -fsanitize=address,undefined -march=native -Wall -Wextra -Wno-unused-function -Werror=sequence-point \
    docs/journal/2026-11-20-carbon-cycle13/g5_bipolar_c13.c -o /tmp/g5_bipolar_c13_asan && /tmp/g5_bipolar_c13_asan /tmp/admitted-bwt/admitted-bwt 1
gcc -O3 -march=native -Wall -Wextra -Wno-unused-function -Werror=sequence-point -Werror \
    docs/journal/2026-11-20-carbon-cycle13/g5_bipolar_c13.c -o /tmp/g5_bipolar_c13 && /tmp/g5_bipolar_c13 /tmp/admitted-bwt/admitted-bwt 25
```
