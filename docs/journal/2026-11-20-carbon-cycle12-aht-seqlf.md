# Carbon Cycle 12: Ostrowski / Agol–Hass–Thurston Self-Overlap Contraction (`AHT-FlatQ-BP4K`), the Meyer `32-bit` / L2 Phase Horizon, and Write-Streamed $\mathrm{LF}$ Column Inversion (`SeqLF-Col` — Gate G5 `PASS`)

*20 November 2026. Session: Carbon (`research/carbon-word-transport`). Mode: Reorientation (default, `METHOD.md` §3a).*

## 1. Cycle Orientation, Toy-to-Hard Synthesis, and Summary of Outcomes

At the Cycle 11 → 12 boundary (`e878e08`), read-only inspection of `origin/master` (`4ab98cd`), `src/bwt.rs`, `src/cells_lazy.rs`, `src/runs.rs`, `tests/common/mod.rs`, `docs/journal/2026-11-19-open-admission.md`, and `docs/site/blog/2026-11-20-research-by-obligation.md` connected two previously separate puzzles:

1. **Why did production `Open` fail on the Sturmian/periodic "toy" regime itself (`pattern`, `onebyte`, `r/n < 1/64`) AND on admitted RLE1-spiral blocks?**
   In `2026-11-20-research-by-obligation.md`, two-letter Sturmian and periodic rotations ("the toy") admit $O(\log n)$ Ostrowski numeration coordinates. Yet in `2026-11-19-open-admission.md`, shipped `Open<16>` **lost `1.55–1.87×` to `Segmented` on `pattern` and `onebyte`** (requiring the `64r >= n` sparse refusal gate in `src/cells_lazy.rs:29` and `src/open.rs:408`), and **also loses `>2×` inside its own admitted regime (`1/64 <= r/n < 0.15`) whenever long RLE1 escape runs (`"xxxx\xfb"`) induce rational spirals (`rle_spiral`, `r/n = 0.0204`)**.
   Tracing `R4-DGIM` and `Open::induce_thick` exposed the exact cause: additive Rauzy–Veech induction executes the **slow additive continued-fraction algorithm** ($\sum a_k = \Theta(n)$ steps: `128,579` steps and `128,560` linear `EpRule` nodes on `pattern` despite `H = 7` strips!), whereas self-overlapping intervals admit exact **multiplicative Euclidean / Agol–Hass–Thurston (AHT) quotient contraction** ($\sum 1 = O(H + \log n)$ steps).
2. **Why do the remaining non-Wikidata regimes in `2026-11-19-open-admission.md` (`runs`, `text`, `random`, `records`) pay `2.2–3.4 ms/block`?**
   - On `runs` (`r/n = 0.1825`), `Open<16>` lost `1.615×` (`4.07 ms` vs `2.18 ms`) due to 1-lane L2 overflow; under `AHT-FlatQ-BP4K` (`LIMIT = 24`), `m` stays inside the single-word `wide == 0` L2 horizon (`r/n < 0.20`), beating `Segmented` at **`1.313 ms` (`0.621×`)**.
   - On `text` (`r/n = 0.2982`), `m = 557,478` (`bits = 20`) crosses the **32-bit edge-packing horizon (`wide == 1`, doubling `edges` from `4 B` to `8 B` = `4.35 MiB > 2 MiB` L2)**.
   - On the **column route (`random` and `records`, `r/n > 0.50`, `296.1 ms` across 79 blocks in `2026-11-19-open-admission.md`)**, `Segmented::invert_in_place` (`src/bwt.rs:158`) still builds the **forward $\Psi$ table** (`build_one_pass`: `*t.add(s) = ll[s] as u32 | (i << 8)`), paying **$n$ random L3 RFO scatter writes across `3.52 MiB` + $n$ random `ll[s]` reads (`1.58–1.64 ms`, `48%` of total decode time)**, whereas conjugating to **1-pass write-streamed $\mathrm{LF}$ construction + AVX2 `vpshufb` reversed assembly (`SeqLF-Col`)** eliminates 100% of scatter RFO misses while preserving **true in-place `out == ll` overwrite**.

### Initial Cycle 12 Triage Kills (Recorded Before Deepening)

Before pivoting to the multi-regime synthesis, five candidates were tested and killed in Triage:
1. **`SWAR4-RunQ` (4-record guarded SWAR RunQ fast path):** `0.977–1.014×` Cycle 11 (`measured`; killed by Bernoulli branch-coalescing conservation $\frac{1-(1-\varepsilon)^4}{4} \approx \varepsilon$).
2. **`L1-SuperCode16` (`<= 1,023` CRC32-deduplicated `16 KiB` L1D dictionary + 4-bit inline edge length):** cut `Emit` by `4–7%` (`0.458 → 0.426 ms` on `block-0`, `0.819 → 0.779 ms` on `block-143`), but CRC32 hashing added `+0.015..0.035 ms` in `Con`, leaving net `Tot = 0.974–1.032×` (`< 10%` Gate G4).
3. **`BP4K-12L` / `BP4K-16L` (`K = 12, 16` lanes on `W24`):** `0.962–1.134×` (`16L` spills x86-64's 16 GPRs, losing `+11%` on `block-0`; confirms `K = 8` is optimal for L2-resident `edges[]`).
4. **Unstaged 32-lane `4 KiB` downward paging (`BPB-32L` without L1D `tile[32][256]` staging):** `1.25–1.58×` slower walk (`measured`) due to **12-way `48 KiB` L1D `4 KiB` set-aliasing** (`48 KiB / 12 ways = 4,096 B`, causing 32 concurrent lanes at matching page offsets to thrash a single 12-way L1D cache set) and the pigeonhole partial-page bound ($\text{pages} \le \lfloor n / P \rfloor + S$).
5. **Single-block KKP $\Psi^2 / \mathrm{LF}^2$ super-alphabet table on $n = 900,000$:** acquiring $\mathrm{LF}^2(i) = \mathrm{LF}(\mathrm{LF}(i))$ from $L[0..n)$ requires $n$ random lookups into a $\ge 20$-bit rank/successor table ($\ge 2.25\text{ MiB} > 2\text{ MiB}$ L2), paying the full $n$-load L3 walk cost during construction (`ACQUISITION-FIRST`).

---

## 2. Four Mandatory Candidate Fields (`METHOD.md` §3a)

### Candidate A: `AHT-FlatQ-BP4K` (Run Route, $0 < r/n < 0.20$ — Abolishing the Sparse Gate & Widening Density)
- **Mode:** Reorientation (replaces additive single-copy Rauzy–Veech subtraction on self-overlapping intervals with multiplicative Euclidean / Ostrowski quotient contraction + squaring-DAG `memcpy` doubling).
- **Displaced obligations:**
  1. $\Theta(n)$ additive Rauzy–Veech loop iterations and $\Theta(n)$-deep linear `EpRule` chains on periodic, quasi-periodic, Sturmian, and RLE1-escape spirals (`128,567` iterations removed on `pattern`; `164,693` removed on `rle_spiral`);
  2. Recursive stack-walking byte-by-byte rule expansion on repeated powers $w^q$;
  3. Production's `64r >= n` sparse refusal gate (`src/cells_lazy.rs:29`, `src/open.rs:408`) and `[0.15, 0.20)` fallback on `runs.bz2`.
- **Replacement principle:** Detect top-driven (`ra < rb && na == ts`) and bottom-driven (`ra > rb && nb == bs`) self-overlapping AHT spirals in registers, contract $q = \lfloor W / d \rfloor$ consecutive RV steps in $O(1)$ pointer updates, build $w^q$ via binary squaring ($\le 2\lfloor \log_2 q \rfloor$ `EpRule` nodes where squaring nodes carry the zero-bit structural invariant `r.left == r.right`), and expand squaring nodes in `r4_append_rule_fast` via $\lfloor \log_2 q \rfloor$ sequential `memcpy` doublings (Theorem 12.1).
- **New paid obligations:** Two register equality checks (`nb == bs` on `ra > rb`, `na == ts` on `ra < rb`) inside `R4-DGIM` (`7–12%` of total decode; measured overhead `0.974× – 1.017×` across `6/6` Wikidata blocks), plus integer `div`/`mod` and $\le 38$ `EpRule` nodes when a spiral fires.
- **Nearest prior art:** Agol–Hass–Thurston interval-pairing transmission (2006; examined on Wikidata in Carbon Cycle 1 where `aht_hits = 0..2`, whereas on `r/n < 0.03` sparse/RLE1 spirals it removes `99.8–99.99%` of RV steps) and Carbon Cycle 11's `FlatQ-BP4K` (`e878e08`).

### Candidate B: `SeqLF-Col` (Column Route, $r/n \ge 0.50$, True In-Place `out == ll`)
- **Mode:** Reorientation (conjugates column-route inversion from forward scattered $\Psi$ construction + forward walk to write-streamed 1-pass $\mathrm{LF}$ construction + backward room walk + AVX2 `vpshufb` reversed assembly).
- **Displaced obligations:** $n$ random L3 RFO cache-line writes into `tt[0..n)` (`3.52 MiB > 2 MiB` L2) and $n$ random `ll[s]` reads across `879 KiB` in `bwt::build_one_pass` (`src/bwt.rs:415`), plus byte-by-byte scalar reversal in `SegScratch::assemble_reversed` (`src/bwt.rs:777`).
- **Replacement principle:** Since $P[i] = L[i] \mid (\mathrm{LF}(i) \ll 8)$ is indexed by $L$-row $i$, scanning `ll[0..n)` once with L1D-resident `ftab[256]` streams `tt[0..n)` strictly sequentially (`seqlf_col_build`: 8x-unrolled scalar when $\max_c \mathrm{counts}[c] < n/8$, 16-byte GPR-SWAR + AVX2 16-wide run-fill otherwise), after which `ll` is never read again and `seg_assemble_reversed_avx2` (`_mm256_shuffle_epi8` + `_mm256_permute2x128_si256`) writes forward-ordered output directly over `ll` (`invert_in_place`, Theorem 12.3).
- **New paid obligations:** Reversed segment assembly (`vpshufb` + `vperm2i128` at 32 bytes/iter, `+0.04 ms` vs forward `memcpy`, outweighed `>25×` by `-1.00..1.05 ms` saved in `build`) and periodic `repeat_cycle` tail check.
- **Nearest prior art:** `src/runs.rs::fill_avx2` (`#324`, which applied backward $\mathrm{LF}$ table construction only to run-record inputs while leaving `src/bwt.rs::Segmented::{invert, invert_in_place}` on scattered $\Psi$ `build_one_pass`).

---

## 3. Foundational Theorems

### Theorem 12.1 — Exact Ostrowski / AHT Self-Overlap Contraction & Zero-Tag Squaring-DAG Doubling (`proved` & `verified`)

**Statement (`proved`):** In right-to-left pointer-free Rauzy–Veech elimination (`R4-DGIM`) with active top interval $(f_a, n_a, r_a, a_w, b_s)$ and active bottom interval $(f_b, n_b, t_s, r_b, b_w)$:
1. **Top-driven spiral (`ra < rb`, `na == ts`, $q = \lfloor r_b / r_a \rfloor \ge 2$, $r_{\text{rem}} = r_b \bmod r_a$, $p_t = \mathrm{tops}[t_s].\mathrm{prev}$):** Because the interval immediately preceding $f_a$ in top order IS $t_s$ itself, $q$ consecutive additive RV iterations peel $q$ copies of width $d = r_a$ from $t_s$, accumulating word $w_q = a_w \cdot (b_w)^q$ and reducing `remaining` by $q \cdot d$:
   - If $r_{\text{rem}} = 0$, the $q$-th step closes with equal widths ($d = d$), leaving $\mathrm{tops}[t_s] = (p_t, d, w_q, b_s)$, $\mathrm{bots}[b_s].\mathrm{top} = t_s$, $s_a \leftarrow t_s$, $s_b \leftarrow n_b$.
   - If $r_{\text{rem}} > 0$, $q$ steps leave active top $(f_a \leftarrow t_s,\; n_a \leftarrow f_a,\; r_a = d,\; a_w = w_q,\; b_s)$ and active bottom $(f_b,\; n_b,\; t_s \leftarrow f_a,\; r_b = r_{\text{rem}},\; b_w)$ with $\mathrm{tops}[f_a].\mathrm{prev} = p_t$, immediately entering Euclidean step $(d, r_{\text{rem}})$.
2. **Bottom-driven spiral (`ra > rb`, `nb == bs`, $q = \lfloor r_a / r_b \rfloor \ge 2$, $r_{\text{rem}} = r_a \bmod r_b$, $p_b = \mathrm{bots}[b_s].\mathrm{prev}$):** Symmetrically, $q$ consecutive additive RV iterations accumulate $w_q = (a_w)^q \cdot b_w$ and reduce `remaining` by $q \cdot r_b$:
   - If $r_{\text{rem}} = 0$, the $q$-th step closes with $\mathrm{tops}[t_s] = (\mathrm{tops}[t_s].\mathrm{prev}, r_b, w_q, b_s)$, $\mathrm{bots}[b_s] = (p_b, t_s)$, $s_a \leftarrow n_a$, $s_b \leftarrow b_s$.
   - If $r_{\text{rem}} > 0$, $q$ steps leave $(r_a = r_{\text{rem}}, a_w)$ paired against $(r_b, w_q)$ with $\mathrm{bots}[f_b].\mathrm{prev} = p_b$, $\mathrm{bots}[b_s] = (f_b, t_s)$, swapping $b_s \leftrightarrow f_b$ and setting $n_b \leftarrow b_s$.
3. **Exact Step-Conservation & Zero-Tag Squaring Identity:** Every contraction of quotient $q$ saves **exactly** $q - 1$ unaccelerated RV steps ($\mathrm{steps}_{\text{AHT}} + \sum (q - 1) = \mathrm{steps}_{\text{unacc}}$). Constructing $w^q$ by binary exponentiation emits $\le 2\lfloor \log_2 q \rfloor$ `EpRule` nodes in which every squaring node satisfies `r.left == r.right` (impossible for non-squaring RV joins of distinct words without self-concatenation harm, and semantically valid whenever `r.left == r.right` occurs!), allowing `r4_append_rule_fast` to expand $w^q$ in $\lfloor \log_2 q \rfloor$ sequential `memcpy` doublings with zero extra tag bits. $\square$

---

### Theorem 12.2 — Meyer Complementation `32-bit` Packing & L2 Horizon (`proved` & `verified`)

**Statement (`proved` & `verified`):** Packed 32-bit return-edge encoding `(code << bits) | next_thin` (`wide == 0`, `4m` bytes) holds if and only if
$$(255 + W_{\text{thick}} + S_{\text{sent}}) \cdot 2^{\lceil \log_2 \max(m, 2) \rceil} \;\le\; 2^{32}.$$
Across $n \approx 900,000$ blocks at `LIMIT = 24`:
- For all $0 < r/n < 0.20$ (`onebyte`, `pattern`, `rle_spiral`, `6/6` Wikidata blocks, and `runs` at $r/n = 0.1825$), $m \le 262,144$ ($\text{bits} \le 18$, $\text{max\_code} \le 16,383$), so `wide == 0` holds and `edges[0..m)` stays $\le 1.0\text{ MiB}$ inside `2 MiB` L2 while Theorem 12.1 eliminates sparse $\Theta(n)$ spirals.
- On prose/source text (`text0`, $r/n = 0.2982$), $m = 557,478 > 2^{19}$ forces $\text{bits} = 20$ and $\text{max\_code} > 4,095 \implies \text{wide} = 1$, doubling edge storage to `8m = 4.35 MiB > 2 MiB` L2 and making Meyer elimination strictly costlier than `runs::fill_avx2`. $\square$

---

### Theorem 12.3 — Write-Streamed Column Conjugacy & In-Place Safety (`proved` & `verified`)

**Statement (`proved`):** Given `ll[0..n)` and `counts[0..256)` summing to $n < 2^{23}$ with `orig_ptr < n`:
1. One sequential pass `tt[i] = ll[i] | (ftab[ll[i]]++ << 8)` constructs the exact backward table $P[i] = L[i] \mid (\mathrm{LF}(i) \ll 8)$ with zero random reads of `ll` and zero non-sequential writes to `tt`.
2. Walking $P$ from `orig_ptr` in segments (`room_segments` with `BACK = true`) reads only `tt` and writes only `seg.bufs`; subsequent AVX2 reversed assembly (`vpshufb` + `vperm2i128` + `repeat_cycle`) writes `out[0..n)` without reading `ll`, satisfying `InverseBwt::invert_in_place`'s exact postcondition when `out` aliases `ll`. $\square$

---

## 4. Executable Verification & Gate G5 Unanimous `PASS` (`verify_cycle12.py`, `g5_aht_seqlf.c`)

- **Manifest Validation (`check_manifest.py`):** Both `aht_seqlf.toml` and `seqlf_col.toml` **PASS**.
- **Standalone Python Verification (`verify_cycle12.py`):** **PASS** in `5.82s` across:
  - `1,538/1,538` exhaustive binary `(L, p)` pairs ($n = 1 \dots 7$, `limit = 1`), asserting both byte-exactness and `steps_aht + aht_saved == steps_unacc` + true in-place `SeqLF-Col`;
  - `3,500/3,500` adversarial Sturmian, quasi-periodic, continued-fraction, and RLE1-escape columns (`30,937` AHT steps saved, `100%` step-conservation identity);
  - `8/8` adversarial boundary controls + `200 KB` non-periodic `pattern` spiral (`12` RV steps, `28,570` additive steps saved, `52` doublings);
  - `6/6` authenticated Wikidata blocks (`wide = 0` on all 6; both `AHT-FlatQ` and in-place `SeqLF-Col` byte-exact).
- **Sanitizer Audit (`gcc -O1 -g -fsanitize=address,undefined -Wall -Wextra -Werror=sequence-point`):** **PASS** (`0` warnings, `0` errors, `0` leaks across `8/8` self-tests incl. `>65k` top/bottom spirals & true in-place `out == ll`, plus `13/13` full `900 KB` blocks across all `7` arms).
- **25-Round Paired Interleaved `-O3 -march=native` Confrontation (`g5_aht_seqlf.c`, `13` blocks $\times$ `7` arms; VM note: single-thread calibrated kernel timings on `Xeon Platinum 8481C` VM, not 20-thread native proof):**

| Block | `r/n` | Shipped Route (`0.5.7`) | Arm 0 `Open16` | Arm 1 `SegRuns` | Arm 2 `SegCol` | Arm 3 `C11-FQ` | Arm 4 `AHT-FQ` | Arm 5 `SeqLF` | Arm 6 `Unified-C12` (`Uni/Shp`, `AHT/C11`, Ovf) |
|---|---:|---|---:|---:|---:|---:|---:|---:|---:|
| `block-0.L` | `0.0627` | `Open16` (`1.501`) | `1.501` | `2.122` | `2.218` | `0.905` | **`0.882`** | `2.191` | **`0.897 ms` (`0.598×`, `0.974×`, `188 KiB`)** |
| `block-2.L` | `0.0661` | `Open16` (`1.867`) | `1.867` | `2.163` | `2.164` | `1.091` | **`1.086`** | `2.263` | **`1.102 ms` (`0.590×`, `0.996×`, `196 KiB`)** |
| `block-85.L` | `0.0941` | `Open16` (`1.914`) | `1.914` | `1.992` | `2.109` | `1.058` | **`1.032`** | `2.051` | **`1.050 ms` (`0.549×`, `0.976×`, `200 KiB`)** |
| `block-143.L` | `0.1248` | `Open16` (`3.398`) | `3.398` | `2.107` | `2.272` | `1.602` | **`1.585`** | `2.196` | **`1.581 ms` (`0.465×`, `0.990×`, `192 KiB`)** |
| `block-210.L` | `0.1480` | `Open16` (`2.905`) | `2.905` | `2.055` | `2.178` | `1.544` | **`1.520`** | `2.115` | **`1.537 ms` (`0.529×`, `0.985×`, `192 KiB`)** |
| `block-207.L` | `0.1016` | `Open16` (`2.232`) | `2.232` | `2.018` | `2.137` | `1.196` | **`1.217`** | `2.111` | **`1.225 ms` (`0.549×`, `1.017×`, `204 KiB`)** |
| `onebyte` | `0.0079` | `SegRuns` (`0.241`) | `0.026` | `0.241` | `0.849` | `0.030` | **`0.028`** | `0.292` | **`0.028 ms` (`0.116×`, `0.942×`, `0 KiB`)** |
| `pattern` | `0.0079` | `SegRuns` (`1.244`) | `1.829` | `1.244` | `1.284` | `1.571` | **`0.095`** | `1.318` | **`0.109 ms` (`0.088×`, `0.060×`, `16 KiB`)** |
| `rle_spiral` | `0.0204` | `Open16` (`2.543`) | `2.543` | `1.304` | `1.321` | `2.119` | **`0.203`** | `1.337` | **`0.205 ms` (`0.081×`, `0.096×`, `152 KiB`)** |
| `runs` | `0.1825` | `SegRuns` (`2.177`) | `4.065` | `2.177` | `2.377` | `1.339` | **`1.313`** | `2.292` | **`1.352 ms` (`0.621×`, `0.981×`, `216 KiB`)** |
| `text0` | `0.2982` | `SegRuns` (`2.312`) | `11.040` | `2.312` | `2.636` | `4.718` | `4.626` | `2.474` | **`2.300 ms` (`0.995×`, `0.980×`, `—`)** |
| `random` | `0.9922` | `SegCol` (`3.356`) | `24.888` | `3.178` | `3.356` | `8.488` | `8.424` | **`2.497`** | **`2.469 ms` (`0.736×`, `—`, `—`)** |
| `records` | `0.9969` | `SegCol` (`3.416`) | `26.098` | `3.130` | `3.416` | `9.388` | `9.099` | **`2.407`** | **`2.364 ms` (`0.692×`, `—`, `—`)** |

*Column-route stage attribution (`g5_aht_seqlf.c`):*
- `random`: `SegCol` Build `1.578 ms` → `SeqLF-Col` Build **`0.575 ms` (`0.364×`, `-63.6%`)**; Total `3.356 ms → 2.497 ms` (**`0.744×`**).
- `records`: `SegCol` Build `1.640 ms` → `SeqLF-Col` Build **`0.588 ms` (`0.359×`, `-64.1%`)**; Total `3.416 ms → 2.407 ms` (**`0.705×`**).

### Pre-Registered Gate G5 Verdict: **`PASS` (`9/9` sub-bars passed unanimously)**

1. **G5.2a (`{onebyte, pattern, rle_spiral}` `AHT-FQ <= 0.20× Shipped`):** **PASS (`3/3`: `0.116×, 0.076×, 0.080×`)**.
2. **G5.2b (`{pattern, rle_spiral}` `AHT-FQ <= 0.15× Cycle 11 FlatQ`):** **PASS (`2/2`: `0.060× [16.5× faster], 0.096× [10.4× faster]`)**.
3. **G5.3a (`6/6` Wikidata `AHT-FQ <= 0.60× Open-qi16`):** **PASS (`6/6`: `0.466× – 0.588×`)**.
4. **G5.3b (`6/6` Wikidata `AHT-FQ <= 1.03× Cycle 11 FlatQ`):** **PASS (`6/6`: `0.974× – 1.017×`)**.
5. **G5.3c (`9/9` Admitted+Sparse `BP4K` Overflow `<= 256 KiB`):** **PASS (`9/9`: `0 – 204 KiB`)**.
6. **G5.4a (`{random, records}` `SeqLF-Col Tot <= 0.80× Seg-Col Tot`):** **PASS (`2/2`: `0.744×, 0.705×`)**.
7. **G5.4b (`{random, records}` `SeqLF-Col Build <= 0.45× Seg-Col Build`):** **PASS (`2/2`: `0.364×, 0.359×`)**.
8. **G5.5a (`13/13` Multi-Regime `Unified-C12 <= 1.02× Shipped`, zero cliffs):** **PASS (`13/13`: max `0.995×` on `text0`)**.
9. **G5.5b (`>= 11/13` Multi-Regime `Unified-C12 <= 0.80× Shipped`):** **PASS (`12/13` blocks at `0.081× – 0.736×`)**.

---

## 5. Coordinator Audit Handoff Notes

Cycle 12 delivers three clean, orthogonal upgrades for Coordinator safe-Rust verification:
1. **`AHT-R4-DGIM` + Squaring-Doubling inside `Open` (`src/open/endpoint.rs` / `R4-DGIM`):** Adding the two self-overlap checks (`nb == bs` on `ra > rb`, `na == ts` on `ra < rb`) + `r.left == r.right` `memcpy` doubling removes the `64r >= n` sparse refusal gate (`src/cells_lazy.rs:29`, `src/open.rs:408`), raises the run-route admission line from `r/n < 0.15` to `r/n < 0.20` (`5 * recs.len() < n`), turns `Open`'s `1.55–1.87×` losses on `pattern`, `onebyte`, and `runs` into **`1.6×–13.1×` speedups over `Segmented`**, and eliminates `10×` spiral blowups inside `1/64 <= r/n < 0.15` (`rle_spiral`).
2. **`SeqLF-Col` + `assemble_reversed_avx2` inside `bwt::Segmented::{invert, invert_in_place}` (`src/bwt.rs:147-162, 767`):** Replacing `build` + `walk_segments_room` on high-entropy column-route blocks (`random`, `records`) with 1-pass write-streamed `LF` build + `walk_back_room` + 32-byte AVX2 `vpshufb` reversed assembly preserves `invert_in_place(ll)` while cutting `random`/`records` decode time by **`26–30%` (`0.70–0.74×`)**.
