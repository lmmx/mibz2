# Neon Cycle 5 — Bit-Sliced Implicit Backward Open (`BSO`), Width-1 Certificate Singularity, and the Implicit-Rank Serial-Latency Trade-Off

*Date:* 2026-11-20  
*Branch:* `research/neon-bipolar-contraction`  
*Mode:* `reorientation` (default under `METHOD.md` §3a at `origin/master` `e88830d`)  
*Code & artifacts:* [`verify_bso.py`](2026-11-20-neon-cycle5-bitsliced-open/verify_bso.py), [`bso.toml`](2026-11-20-neon-cycle5-bitsliced-open/bso.toml), [`g5_bso.c`](2026-11-20-neon-cycle5-bitsliced-open/g5_bso.c), [`g5_bso_par.c`](2026-11-20-neon-cycle5-bitsliced-open/g5_bso_par.c), [`g5_bso_branchless.c`](2026-11-20-neon-cycle5-bitsliced-open/g5_bso_branchless.c), [`g5_bso_flat.c`](2026-11-20-neon-cycle5-bitsliced-open/g5_bso_flat.c)

---

## 1. Executive Summary & Charter Corrections Incorporated

Cycle 5 attacks **Obligation #1 (Dependent traversal of retained rows)**, **Foundational Question #5 (`OPEN-QUESTIONS.md` §4: output-sourced self-copying)**, and **`OPEN-QUESTIONS.md` §3 (L2-resident working set / memory traffic)** by inverting two prior obstructions: (i) Astra C's output-sourced anchor section, and (ii) Cycle 2's Permutation-Entropy Barrier (`log2(m!) ≈ 410 KiB > 256 KiB` L2 for an *explicit* arbitrary bijection table on $m = 204\text{k}$ rows).

**Material calibration & audit corrections incorporated from `origin/master` (`e88830d`):**
- **Cache-model fidelity warning (`40b2906`):** Independent calibration on the reference pair (`i16` vs `qi16`) established that this C 3-level LRU cache simulator **mis-ranks** `i16` against `qi16` (predicting `i16` wins on L2 misses, DRAM B/B, and work while `i16` runs `1.10–1.16×` slower natively). Accordingly, **all cache-model counts below are treated strictly as diagnostic traffic indicators, never as predictions of native competitiveness.**
- **Backward Open native verdict (`6ccab45`, `#659`):** Claude's native audit confirmed Backward Open runs `1.10–1.15×` slower than shipped `qi16` on all 6 blocks because the second pass over capped run records offsets the queue savings (consistent with our Cycle 3 VM timing calibration of `1.02–1.09×`). In Cycle 5, `BSO` avoids a second scan over capped records by writing thin bytes sequentially to a single `88–200 KiB` buffer in Pass 1.

**Main findings of Cycle 5:**
1. **Candidate 5A — Dynamic Output-Sourced Anchor Sections (`OPEN-QUESTIONS.md` §4 / Foundational Question #5) & Width-1 Certificate Singularity (`proved` + `measured`):**
   - **Proved (Theorem 3.1):** In *any* online interval-certificate output-sourcing cursor (generalising Astra C to any stopping rule or anchor set $A$), passing through even one row in a width-1 run of $L$ collapses the certificate interval to a singleton $[s, s+1)$, yielding zero future reuse. Hence **100% of all width-1 runs** ($|R_1| = 34{,}262–97{,}054$, `30.6%–47.5%` of Open's $m$) must be traversed on misses, and at least one row per distinct thin run (`45.2%–61.6%` of $m$) must be missed.
   - **Measured** across the top 12 single-byte dynamic anchors (`cnt ∈ [8k, 90k]`) on all 6 Wikidata blocks: even the single best anchor byte on each block (`'e'`, `'a'`, `':'`, `','`, `'n'`) still traverses **`0.61×–0.86×` of Open's entire $m$ thin rows on misses** (`62.6k–167.8k` thin miss rows, plus `19.5k–57.2k` anchor hits; total probes `0.94×–1.33× m`). Killed cleanly at Triage.
2. **Candidate 5B — Bit-Sliced Implicit Backward Open (`BSO-512`) & Flat-Byte AVX2 Variants (`BSO-Flat128`, `BSO-Flat64`, `BSO-Par4/8`):**
   - **Obstruction Inversion (`proved` + `measured`):** Open's reduced bijection $g = P \circ \mathrm{LF}_v$ is *not* an arbitrary permutation on $m$ rows; it factors into an $O(B)$ piecewise translation $P$ (`2,372–6,712` bands, `23–58 KiB`) composed with the stable-sort LF of the thin column $v = L[S]$, whose 0th-order Shannon entropy is only **`H_0(v) = 5.14–5.37` bits/row (`56.4–132.2 KiB`)** and whose **top-63 symbols cover `99.1%–99.6%` of all thin rows** across all 6 Wikidata blocks (`m_esc = 374–1,270` escape rows = `1.5–5.0 KiB`).
   - **Construction (`proved`, Gate G1 `verified` on `28,418 / 28,418` small cases + `6 / 6` adversarial cases + `5,399,692 / 5,399,692` B on all 6 blocks; Gate G2 `PASS`):** `BSO-512` eliminates Open's `256` scattered `qdata` queues, explicit `360–817 KiB` `edges[0..m]` table, and `wir` scatter pass, replacing them with an aligned **`1.00 B/row` 6-bit-plane superblock structure `SB512`** (`512 B` per `512` rows = `88–200 KiB`, built in one L2-resident AVX2 `vpmovmskb` transpose pass) + 32-bit mega-block bases (`<= 7 KiB`, handling arbitrary counts $\ge 65{,}536$) + compact `Band8` (`8 B/band`) + direct `band_hint`.
   - **Pre-registered Gate G5 Verdict (`measured` in [`g5_bso.c`](2026-11-20-neon-cycle5-bitsliced-open/g5_bso.c), [`g5_bso_par.c`](2026-11-20-neon-cycle5-bitsliced-open/g5_bso_par.c), [`g5_bso_branchless.c`](2026-11-20-neon-cycle5-bitsliced-open/g5_bso_branchless.c), [`g5_bso_flat.c`](2026-11-20-neon-cycle5-bitsliced-open/g5_bso_flat.c)): FAIL overall (passes 4/5 criteria, fails single-thread VM time).**
     - **Diagnostic cache-model traffic (`BSO-512`, `g5_bso.c`, 6/6 blocks):** L2 misses **`0.419×–0.690×`** Open, LLC misses **`0.598×–0.751×`** Open, modelled DRAM traffic **`0.593×–0.745×`** Open (`3.47–4.80 B/B` vs `4.77–7.75 B/B`), Peak RAM **`0.360×–0.494×`** Open (`1.08–1.24 MB` vs `2.19–3.44 MB`), and producer time (`p1 + bsl` vs Open's `p1 + wir`) **`0.70×–0.77×`** Open.
     - **Single-thread uninstrumented `-O3 -mavx2 -mpopcnt` VM C time:** **`2.48–2.80×` slower** than Shipped Open (`BSO-512`), improving to **`2.02–2.35×` slower** with single-cache-line `BSO-Flat64` (`1.75 B/row`, `2` `vpcmpeqb` + `1` `popcnt`, which spills L2 on `block-143.L` at `1.074×` L2m).
     - **Zero-bitmap grid-splitter parallel walk (`z & 4095 == 0`, `BSO-Par4/8`):** Cuts per-pass `emit` time nearly in half (`3.16 -> 1.73 ms` on `block-0.L`) at **`+0.00` modelled DRAM B/B**, but requiring two passes (`4a` count + `4b` write) doubles total instructions and L1 traffic (`2.76–3.03×` Open overall).
3. **New Structural Obstruction (`Implicit-Rank Serial-Latency Obstruction`):** Any implicit rank representation of $g = P \circ \mathrm{LF}_v$ replaces Open's single dependent load (`edges[j]`) with **`4` serially chained dependent loads** (`v[z] -> chk[c] -> band_hint[y] -> bands[bidx]`, plus SIMD `popcnt`), costing `27.5–35.6 ns/step` vs Open's `6.2 ns/step` on an uncontended single thread.

*Distinction of status:* `BSO-512` and `BSO-Par` are **possible** (`proved` + `verified` byte-exact on `28,418` small + `6` adversarial + `6` Wikidata blocks) and **feasible** (Gate G2 `PASS`, `0.36–0.49×` Peak RAM, `0.70–0.77×` producer time); on single-thread VM time they are **not competitive** (`2.02–2.80×` Open). Whether the `25–41%` modelled DRAM reduction and `51–64%` Peak RAM reduction become advantageous on a bandwidth-saturated 10–20-worker native i9 (`OPEN-QUESTIONS.md` §3) remains **unresolved** without native multi-threaded measurement.

---

## 2. §3a Mandatory Four-Field Statements (No Self-Classification)

### Candidate 5A: Dynamic Output-Sourced Anchor Section (Generalised Astra C)
- **Mode:** `reorientation`.
- **Displaced obligation:** Eliminates Obligation #1 (explicit `edges[0..m]`), Open `ep` (thick-strip RV), `qdata[256]`, and flattened `spelling`, replacing them with an online self-sourcing cursor that copies repeated anchor-to-anchor phrases straight from already-emitted `out[0..pos)`.
- **Replacement principle:** Interval-certificate rigidity along forward $\Psi$ itineraries (`Astra C` Theorem) applied to a dynamically selected frequency-optimal anchor byte bucket $A = [C[c^*], C[c^*+1])$.
- **New paid obligations:** Online one-step $\Psi$ oracle on miss itineraries (`182k–438k` raw miss bytes, containing `62.6k–167.8k` thin rows = `0.61–0.86×` Open's entire `m`) plus an `8|A|`-byte section table (`156–488 KiB`) and certificate intersection/publication.
- **Nearest prior art:** Astra C (`2026-11-18-astra-c-output-first.md`, hardcoded `','` section against Cells). Cycle 5 generalises across the top 12 dynamic byte anchors on all 6 blocks, measures the exact thin-row miss fraction against Open, and proves the Width-1 Certificate Singularity lower bound.

### Candidate 5B: Bit-Sliced Implicit Backward Open (`BSO-512` / `BSO-Flat` / `BSO-Par`)
- **Mode:** `reorientation`.
- **Displaced obligation:** Eliminates Open's explicit `4m`-byte `edges[0..m]` table (`360–817 KiB`, > L2), Open's `256` scattered `qdata[c]` queues (`360–817 KiB`), and Open's `wir` band-scatter pass (Obligation #2 + the `> L2` footprint of Obligation #1).
- **Replacement principle:** **Low-Entropy Factorisation of $g = P \circ \mathrm{LF}_v$** (`proved`, §3.2): instead of storing an explicit permutation of entropy $\log_2(m!) \approx 410\text{ KiB}$, store the thin column $v = L[S]$ directly in a **`1.00 B/row` 6-bit-plane transposed superblock structure `SB512`** (`top-63` symbols cover `99.1–99.6%` of `v`, `< 5 KiB` escape table) and evaluate $\mathrm{LF}_v(z) = C[v[z]] + \text{rank}_{v[z]}(v, z)$ online via constant-time AVX2 XNOR-AND-`popcnt` followed by Open's $O(B)$ piecewise map $P$.
- **New paid obligations:** One sequential L2-resident pass building `v_raw` (`88–200 KiB`) and AVX2-transposing it into `SB512` (`88–200 KiB`) + `band_hint[y >> 4]` (`11–25 KiB`); during `emit`, each of the $m$ steps pays 4 serially chained L1/L2 loads (`planes/v` -> `chk` -> `band_hint` -> `Band8`) plus AVX2 rank arithmetic (`2.15–4.21×` Open's L1 misses).
- **Nearest prior art:** Cycle 2 online SIMD rank triage (which assumed 256-symbol checkpoints every 256 rows = `400 KiB > L2` and was killed without implementation), Astra D's `Forward Affine Tiles` (`260–579 KiB`, `3.44 B/B` DRAM on frozen, `900k` steps at `82` events/B), and Cycle 2 Queue-Free Backward Open (`g = P ∘ LF_v` with explicit `edges[0..m]`). `BSO-512` restricts implicit rank exclusively to Open's $m$ thin rows (`4.5–10×` fewer steps than Astra D), compresses 256 symbols into 6 bit-planes + `< 1%` escape (`1.00 B/row` including checkpoints), and achieves `O(1)` branchless AVX2 rank inside a `106–243 KiB` L2-resident hot set.

---

## 3. Mathematical Results (`proved`)

### Theorem 3.1 (Width-1 Certificate Singularity / Self-Sourcing Floor — `proved`)
Let $L \in \Sigma^n$ be any BWT column with forward step $\Psi = \mathrm{LF}^{-1}$, and let $R_1 = \{x \in [0, n) : \text{the maximal } L\text{-run containing } \Psi(x) \text{ has width } 1\}$. Consider *any* online interval-certificate output-sourcing cursor (intersecting translated maximal $\Psi$-run intervals $[r_{\text{lo}} - (x - s), r_{\text{hi}} - (x - s))$ along each itinerary $\gamma$ from $s$ to $s'$, regardless of stopping rule or anchor set).
1. If an itinerary $\gamma$ visits even a single row $x \in R_1$, then at that step $r_{\text{hi}} - r_{\text{lo}} = 1$, so the intersected certificate collapses to the singleton $[s, s+1)$ and can supply **zero** future hits.
2. Consequently, every row $x \in R_1$ must be traversed on a **miss** (zero width-1 rows can be skipped by certificate copying).
3. Furthermore, for every maximal run of width $w \ge 2$ visited by the marked orbit, at least one row of that run must be visited on a miss before any certificate covering rows of that run can be published. Thus total miss steps satisfy
$$\text{MissRows} \ge |R_1| + \#\{\text{distinct visited thin runs with } w \in [2, 16]\text{ not shadowed by wider runs}\} \ge |R_1|.$$

**Proof.** On a width-1 run of $L$, the corresponding $F$-interval $[r_{\text{lo}}, r_{\text{hi}})$ of $\Psi$ satisfies $r_{\text{hi}} - r_{\text{lo}} = 1$. Since $x \in [r_{\text{lo}}, r_{\text{hi}})$, intersecting any running interval $[\text{lo}, \text{hi})$ with $[r_{\text{lo}} - (x - s), r_{\text{hi}} - (x - s)) = [s, s+1)$ yields $[s, s+1)$, containing no row other than $s$. Hence publishing $[s, s+1)$ never produces a hit at any distinct row $s' \neq s$ on the marked orbit (and in a full-period block $s$ itself is never revisited). Claim 3 follows because no certificate can be published before its first traversal. $\square$

### Theorem 3.2 (Exact Bidirectional Bit-Sliced Rank with Padding Cancellation & Mega-Block Bases — `proved`)
Let $v \in \Sigma^m$ be the thin column $L[S]$ of Open's retained set $S$ (maximal run width $\le \text{LIMIT}$), mapped via $\phi: \Sigma \to \{0, \dots, 63\}$ sending the $63$ most frequent symbols in $v$ to $\{0, \dots, 62\}$ and all remaining symbols to escape code $63$. Pad $v$ with escape code $63$ to length $n_{\text{sb}} \cdot 512$ ($n_{\text{sb}} = \lceil m / 512 \rceil$), and partition into 512-row superblocks (`lo` = rows $0..255$, `hi` = rows $256..511$) grouped into mega-blocks of $64$ superblocks ($32{,}768$ rows). Store `mega_chk[mg][c]` (`u32`, cumulative padded count of code $c$ before mega-block `mg`) and `sbs[s].chk[c]` (`u16`, padded count of code $c$ in $[32768 \cdot \text{mg}, 512 s)$), plus a sentinel `sbs[n_sb].chk[c]`.
1. **Unconditional `u16` non-overflow:** Within any $32{,}768$-row mega-block, every code $c \in \{0,\dots,63\}$ occurs at most $32{,}768 < 65{,}536$ times, so `sbs[s].chk[c]` fits in `uint16_t` for *all* inputs of length $m < 2^{32}$.
2. **Exact rank for all $z \in [0, m)$ (including partial final superblocks):** Writing $s = \lfloor z / 512 \rfloor$, $h = \lfloor (z \bmod 512) / 256 \rfloor \in \{0, 1\}$, and $r = z \bmod 256$:
$$\text{rank}_{\phi(v[z])}(v, z) = \begin{cases} \text{base}(s, c) + \#\{u \in [0, r) : \text{lo}_s[u] = c\} & (h = 0), \\ \text{base}(s+1, c) - \#\{u \in [r, 256) : \text{hi}_s[u] = c\} & (h = 1), \end{cases}$$
where $\text{base}(s', c) = \text{mega\_chk}[\lfloor s'/64 \rfloor][c] + \text{sbs}[s'].\text{chk}[c]$. For the final superblock ($s = n_{\text{sb}} - 1$) when $h = 1$, every padding position $u \ge m - 512 s - 256$ has $u > r$ (since $z < m$), so every padding symbol contributes identically ($+1$) to both $\text{base}(n_{\text{sb}}, c)$ and $\#\{u \in [r, 256) : \text{hi}_s[u] = c\}$, cancelling identically.
3. **Exact `LF_v(z)`:** If $c = \phi(v[z]) < 63$, then $\mathrm{LF}_v(z) = C_{\text{thin}}[\phi^{-1}(c)] + \text{rank}_c(v, z)$. If $c = 63$, then $\text{rank}_{63}(v, z)$ is the exact sequential index of $z$ among escape rows, so storing `(esc_lf[i], esc_byte[i])` during the single construction pass yields exact $(\mathrm{LF}_v(z), v[z])$.
4. **Midpoint `uint8_t` bound (`BSO-Mid` / `BSO-Flat128` / `BSO-Flat64`):** Whenever $m > \text{LIMIT}$ (here `LIMIT = 16`), every maximal run of $v$ has length $\le 16$, so in any 256-row half-block (resp. 128-row half-block) no single heavy symbol $c < 63$ can occur more than $241 < 256$ (resp. $121 < 256$) times; hence `mid[c]` fits unconditionally in `uint8_t` (with $c = 63$ only padded at the final superblock when `m < 512*n_sb`; in `verify_bso.py` / general code, mega-block `u16` or `SB512` has no midpoint limit at all).

**Proof.** Immediate from telescoping cumulative sums over the padded array and bijectivity of $\phi$ on $\{0, \dots, 62\}$ paired with order-preserving indexing of code $63$. $\square$

### Theorem 3.3 (Zero-Bitmap Grid-Splitter Parallel Cycle Decomposition — `proved`)
Fix a power-of-two grid stride $G$ (e.g., $G = 4096$) and define splitter set $\mathcal{S} = \{\text{mark}\} \cup \{i G : 0 \le i < \lceil m / G \rceil\}$. Because $g = P \circ \mathrm{LF}_v$ is a bijection on $\{0, \dots, m-1\}$, every splitter $s \in \mathcal{S}$ has a unique first-return splitter $\sigma(s) \in \mathcal{S}$ reached in $t(s) \ge 1$ steps emitting $b(s)$ output bytes, and $\sigma$ is a bijection on $\mathcal{S}$. The cycle of $\sigma$ containing $\text{mark}$ partitions the exact marked cycle of $g$ into disjoint segments whose exact descending output write endpoints are determined by prefix-summing $b(s)$ along $\text{mark} \to \sigma(\text{mark}) \to \dots \to \text{mark}$, even when $g$ has multiple cycles (grid splitters on non-marked cycles are simply absent from the $\sigma$-cycle of $\text{mark}$). Membership $z \in \mathcal{S}$ requires zero memory: `((z & (G - 1)) == 0) || (z == mark)`. $\square$

---

## 4. Triage, Gate G1 & Gate G2 Evidence (`verified` & `measured`)

### 4.1 Candidate 5A Triage — Dynamic Anchor-Section Reuse Across All 6 Blocks (`measured`)
Measured across the top 12 single-byte anchors (`cnt ∈ [8k, 90k]`) on all 6 authenticated Wikidata blocks (best single anchor per block by `miss_thin`):

| Block | Open $m$ | Width-1 floor $\|R_1\|$ (`%` of $m$) | Best anchor $c^*$ (`f`, `8f` KiB) | Misses / Hits | Raw miss bytes (`%` of $n$) | **Thin rows on misses (`× m`)** | Copied from `out` |
|---|---:|---:|---|---:|---:|---:|---:|
| `block-0.L` | 89,874 | 35,116 (39.1%) | `'e'` (`62,527`, 488 KiB) | 5,370 / 57,157 | 182,081 (20.2%) | **62,554 (`0.70× m`)** | 79.8% |
| `block-2.L` | 112,070 | 34,262 (30.6%) | `'e'` (`60,579`, 473 KiB) | 6,826 / 53,753 | 205,321 (22.8%) | **68,533 (`0.61× m`)** | 77.2% |
| `block-85.L` | 156,404 | 55,504 (35.5%) | `'e'` (`56,909`, 444 KiB) | 8,772 / 48,137 | 266,688 (29.6%) | **111,351 (`0.71× m`)** | 70.4% |
| `block-143.L` | 196,841 | 72,578 (36.9%) | `'e'` (`56,548`, 441 KiB) | 14,246 / 42,302 | 370,220 (41.1%) | **142,969 (`0.73× m`)** | 58.9% |
| `block-207.L` | 135,921 | 62,407 (45.9%) | `'a'` (`53,708`, 419 KiB) | 10,229 / 43,479 | 311,402 (34.6%) | **115,555 (`0.85× m`)** | 65.4% |
| `block-210.L` | 204,281 | 97,054 (47.5%) | `'e'` (`54,522`, 425 KiB) | 13,179 / 41,343 | 430,203 (47.8%) | **167,820 (`0.82× m`)** | 52.2% |

Even with a free oracle for thick runs on misses, `miss_thin + hits` equals `0.94–1.33× m`, and Astra C's `','` anchor (`f = 28k–31k`, `219–243 KiB`) has `miss_thin = 0.74–0.89× m`. Thin runs are precisely the non-repetitive contexts where output-sourcing cannot reuse certificates. **Candidate 5A killed at Triage.**

### 4.2 Thin-Column Entropy & Working-Set Census (`measured`)

| Block | Open $m$ (`4m` KiB) | $|\Sigma_v|$ | $H_0(v)$ (`KiB`) | Top-63 cov (`esc63`) | `max(thin[c])` | `B` bands (`ne`, `%` rows) | `BSO-512` hot set (`v6+chk+esc+P`) | Total incl. `sp` |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| `block-0.L` | 89,874 (`351K`) | 87 | 5.14b (`56.4K`) | **99.6%** (`374`) | 5,462 | 3,158 (`1,321`, 17.0%) | **116.8 KiB** | 171.5 KiB |
| `block-2.L` | 112,070 (`438K`) | 85 | 5.24b (`71.7K`) | **99.6%** (`490`) | 6,986 | 4,191 (`1,921`, 19.9%) | **147.7 KiB** | 201.5 KiB |
| `block-85.L` | 156,404 (`611K`) | 94 | 5.22b (`99.7K`) | **99.4%** (`997`) | 9,959 | 2,372 (`1,219`, 13.9%) | **180.2 KiB** | 214.6 KiB |
| `block-143.L` | 196,841 (`769K`) | 96 | 5.28b (`126.9K`) | **99.4%** (`1,270`) | 12,125 | 6,712 (`3,634`, 22.7%) | **255.8 KiB** | 323.2 KiB |
| `block-207.L` | 135,921 (`531K`) | 94 | 5.37b (`89.2K`) | **99.1%** (`1,270`) | 8,461 | 4,523 (`2,259`, 19.1%) | **177.5 KiB** | 238.6 KiB |
| `block-210.L` | 204,281 (`798K`) | 96 | 5.30b (`132.2K`) | **99.4%** (`1,228`) | 12,448 | 4,145 (`2,228`, 17.4%) | **243.0 KiB** | 287.4 KiB |

### 4.3 Gate G2 Acquisition Manifest (`PASS`) & Gate G1 Exhaustive Verifier (`PASS`)
- **Gate G2 (`python3 docs/team/acquisition/check_manifest.py docs/journal/2026-11-20-neon-cycle5-bitsliced-open/bso.toml`):** **`PASS`** — all inputs trace cleanly from $I_0$ (`records`, `p`, `n`).
- **Gate G1 (`python3 docs/journal/2026-11-20-neon-cycle5-bitsliced-open/verify_bso.py`):**
  - **Exhaustive small cases:** **`28,418 / 28,418` PASS** (`14,209` `BSO` + `14,209` `BSO-Par`, covering all binary $n \le 9$ and all ternary $n \le 6$ at every primary mark $p$, including short periods and multi-cycles).
  - **Adversarial cases (`6 / 6` PASS for both `BSO-512` and `BSO-Par`):** `Uniform256(4000, 193 esc)`, `Alt140k(thin['a'] = 70,000 > u16)` (exercising `mega_chk`), `AllZero(10000, m=1)`, `Periodic(abc x 1000)`, `MultiCycle(010101)`, `Sturmian(1,9999)`.
  - **Six authenticated Wikidata blocks:** **`5,399,692 / 5,399,692` bytes exact** (`PASS`).

---

## 5. Gate G5 Confrontation (`measured`, Diagnostic Traffic + VM Timing)

**Pre-registered PASS/FAIL rule (fixed before running):** PASS iff on ALL 6 authenticated blocks vs Shipped Forward Open: (1) byte-exact, (2) `L2m <= 0.85x`, (3) `DRAM B/B <= 0.90x`, (4) `PeakMem <= 1.15x`, and (5) single-thread uninstrumented `-O3 -mavx2 -mpopcnt` VM C time `<= 0.90x`. *(Reminder per `40b2906`: the 3-level LRU cache model is diagnostic only and does not predict native ranking.)*

All four C harnesses compiled with `cc -O3 -mavx2 -mpopcnt -Wall -Wextra -Werror` (zero warnings) and passed `100%` byte-exact on all 6 blocks (`5,399,692/5,399,692` B).

### 5.1 Primary Confrontation: [`g5_bso.c`](2026-11-20-neon-cycle5-bitsliced-open/g5_bso.c) (`BSO-512` vs Shipped Forward Open)

| Block | Open L2m / LLC / DRAM / Peak / VM ms | `BSO-512` L2m / LLC / DRAM / Peak / VM ms | Ratios (L2m / LLC / DRAM / Peak / Prod ms / Total ms) |
|---|---:|---:|---:|
| `block-0.L` | 72.1k / 34.4k / 4.77 / 2.19M / 1.71 | **32.8k / 25.9k / 3.55 / 1.08M** / 4.24 | **0.454× / 0.751× / 0.745× / 0.494× / 0.75×** / `2.48×` |
| `block-2.L` | 91.2k / 38.4k / 5.33 / 2.44M / 2.07 | **38.2k / 27.7k / 3.81 / 1.11M** / 5.37 | **0.419× / 0.722× / 0.715× / 0.455× / 0.77×** / `2.59×` |
| `block-85.L` | 102.9k / 40.5k / 5.56 / 2.58M / 2.26 | **50.5k / 25.8k / 3.47 / 1.13M** / 6.33 | **0.491× / 0.636× / 0.624× / 0.437× / 0.76×** / `2.80×` |
| `block-143.L` | 179.4k / 58.3k / 7.75 / 3.44M / 3.89 | **123.8k / 35.5k / 4.80 / 1.24M** / 9.81 | **0.690× / 0.609× / 0.619× / 0.360× / 0.74×** / `2.52×` |
| `block-207.L` | 111.7k / 42.6k / 5.83 / 2.69M / 2.54 | **47.5k / 29.5k / 3.98 / 1.15M** / 6.43 | **0.425× / 0.692× / 0.683× / 0.428× / 0.75×** / `2.53×` |
| `block-210.L` | 163.1k / 52.6k / 7.04 / 3.22M / 3.45 | **78.8k / 31.4k / 4.17 / 1.20M** / 8.92 | **0.483× / 0.598× / 0.593× / 0.373× / 0.70×** / `2.59×` |

*Phase split on `block-0.L` (`1.71 ms` Open vs `4.24 ms` `BSO-512`):* Open `[p1=0.59, ep=0.42, wir=0.09, em=0.57]`; `BSO-512` `[p1=0.39, bsl=0.12, ep=0.47, em=3.20]`. `BSO-512` cuts producer time (`p1+bsl` vs `p1+wir`) by `23%–30%` on all 6 blocks, while single-thread `emit` rises from `0.57–1.59 ms` (`6.3–8.1 ns/step`) to `3.20–7.83 ms` (`35.6–39.8 ns/step`), with L1 misses `2.73–4.21×` Open.

### 5.2 Diagnostic Ablations of `emit` Latency: [`g5_bso_par.c`](2026-11-20-neon-cycle5-bitsliced-open/g5_bso_par.c), [`g5_bso_branchless.c`](2026-11-20-neon-cycle5-bitsliced-open/g5_bso_branchless.c), and [`g5_bso_flat.c`](2026-11-20-neon-cycle5-bitsliced-open/g5_bso_flat.c)

To separate incidental branch/layout overhead from structural latency, we tested three targeted ablations across all 6 blocks (all 100% byte-exact):

| Variant (`source`) | `LF_v` layout (`B/row`) | L2m vs Open (6 blocks) | DRAM vs Open (6 blocks) | Peak vs Open | L1m vs Open | 1-thread VM time vs Open | `block-0.L` `emit` (`ns/step`) |
|---|---|---:|---:|---:|---:|---:|---:|
| **`BSO-512`** (`g5_bso.c`) | 6-plane SB512 (`1.00`) | **0.419–0.690×** (6/6) | **0.593–0.745×** (6/6) | **0.360–0.494×** | 2.73–4.21× | 2.48–2.80× | 3.20 ms (`35.6`) |
| **`BSO-Mid`** (`g5_bso_par.c`) | 6-plane + `mid[64]` (`1.125`) | **0.442–0.710×** (6/6) | **0.600–0.750×** (6/6) | **0.367–0.499×** | 2.07–3.06× | 2.34–2.51× | 2.98 ms (`33.2`) |
| **`BSO-512-BL(1x)`** (`g5_bso_branchless.c`) | 100% branchless SB512 (`1.00`) | **0.423–0.760×** (6/6) | **0.596–0.747×** (6/6) | **0.363–0.496×** | — | 2.51–2.96× | 3.16 ms (`35.2`) |
| **`BSO-512-BL(4x)`** (`g5_bso_branchless.c`) | 4-lane zero-bitmap `z&4095==0` | 0.441–1.229× (5/6) | **0.596–0.747×** (+0.00!) | **0.363–0.496×** | — | 2.76–3.03× | 1.73 + 1.91 ms (`19.2`/pass) |
| **`BSO-Flat128`** (`g5_bso_flat.c`) | Flat byte 128 B AVX2 (`1.375`) | 0.505–1.065× (5/6) | **0.620–0.763×** (6/6) | **0.385–0.512×** | 2.63–3.85× | 2.16–2.55× | 2.67 ms (`29.7`) |
| **`BSO-Flat64`** (`g5_bso_flat.c`) | Flat byte 64 B (1 line!) (`1.75`) | 0.573–1.074× (4/6) | **0.645–0.779×** (6/6) | **0.406–0.527×** | 2.15–2.91× | 2.02–2.35× | 2.47 ms (`27.5`) |

**Pre-registered Gate G5 Verdict:** **FAIL** on criterion (5) (single-thread VM C time `2.02–2.80×` Open across all variants), despite `BSO-512` clearing criteria (1)–(4) by wide margins on all 6/6 blocks.

---

## 6. Structural Obstructions Discovered

### Obstruction 6.1: Width-1 Certificate Singularity Obstruction (Foundational Question #5 / `OPEN-QUESTIONS.md` §4)
- **Statement:** Every row in a width-1 run of $L$ ($30.6\%–47.5\%$ of Open's $m$ on Wikidata) collapses any BWT interval certificate to width 1, preventing future output-copy reuse (Theorem 3.1). Combined with first-visit misses on $w \in [2, 16]$ runs, even the best dynamic single-byte anchor per block still traverses `0.61–0.86×` of Open's entire $m$ thin rows on misses (`62.6k–167.8k` thin miss rows + `19.5k–57.2k` hits).
- **Witness:** Theorem 3.1 (`proved`) + dynamic top-12 single-byte anchor census across all 6 authenticated Wikidata blocks (§4.1).
- **Scope:** Any online output-sourcing cursor relying on BWT interval-certificate intersection (`Astra C` and multi-byte/dynamic-anchor variants).
- **Escape condition:** Copy from `out[0..pos)` using *approximate* or *LCP-bridged* matches across width-1 boundaries (not exact BWT cylinder certificates), or use a representation that resolves width-1 runs collectively rather than via online misses.

### Obstruction 6.2: Implicit-Rank Serial-Latency Obstruction (Obligation #1 / `OPEN-QUESTIONS.md` §3)
- **Statement:** Replacing Open's explicit `edges[0..m]` (`1` dependent load/step, `6.2 ns/step` in uncontended L3) with an L2-resident implicit rank oracle for $g = P \circ \mathrm{LF}_v$ (`BSO-512` / `BSO-Flat64`) converts each step into **`4` serially chained dependent loads** (`v[z]` -> `chk[c]` -> `band_hint[y]` -> `bands[bidx]`, plus SIMD `popcnt` ALU), costing **`27.5–35.6 ns/step`** on 1 thread even when 100% branchless and reading only 1 cache line (`BSO-Flat64`). Hiding that latency via 2-pass parallel walkers (`BSO-512-BL(4x)`) halves per-pass time (`35.2 -> 19.2 ns/step`) at `+0.00` modelled DRAM B/B, but paying 2 passes doubles total instructions/L1 traffic (`2.76–3.03×` Open on 1 thread).
- **Witness:** [`g5_bso.c`](2026-11-20-neon-cycle5-bitsliced-open/g5_bso.c), [`g5_bso_branchless.c`](2026-11-20-neon-cycle5-bitsliced-open/g5_bso_branchless.c), and [`g5_bso_flat.c`](2026-11-20-neon-cycle5-bitsliced-open/g5_bso_flat.c) across all 6 authenticated Wikidata blocks.
- **Scope:** Single-walker implicit-rank reconstructions of $g = P \circ \mathrm{LF}_v$ evaluated on uncontended single-thread latency, and 2-pass parallel walkers over `BSO` when `emit` is instruction-throughput-bound.
- **Escape condition:** Two distinct escape routes remain **unresolved** and are preserved for future work:
  1. **Multi-threaded bandwidth-bound regime (`OPEN-QUESTIONS.md` §3):** On a 10–20-worker i9 where shared LLC is contended and DRAM bandwidth saturates (`25–30 GB/s`), `BSO-512`'s `25–41%` modelled DRAM reduction (`1.22–2.95 B/B` saved) and `51–64%` Peak RAM reduction (`1.08–1.24 MB` vs `2.19–3.44 MB`) operate near Astra D's `3.36` events/B ten-worker break-even (`2–4×` better work-per-DRAM-byte than Astra D's `Forward Tiles`), though the fidelity ledger (`40b2906`) forbids predicting a native win without measuring native multi-threaded scaling of Open first.
  2. **1-pass `K`-walker consumer fusion + direct `P`-band folding:** Folding `P(y)` into direct wide-band entries (`90.4–96.2%` of $m$ lies in wide bands, cutting `4` chained loads to `3` on `~85%` of steps), and—if downstream RLE1+CRC can consume $K$ ordered chunk buffers directly—eliminating Pass 4A without an extra `n`-byte copy.

---

## 7. Unfinished Questions & Preserved Leads for Future Continuation

Per the Cycle 5 pause instruction, we stop here without beginning Cycle 6. The following unresolved questions and `origin/master` leads are preserved cleanly for future sessions:
1. **Native multi-thread bandwidth sensitivity (`OPEN-QUESTIONS.md` §3):** Does shipped Open (`qi16`) saturate DRAM bandwidth or suffer shared-LLC thrashing at 10–20 workers on the i9? Only if yes does `BSO-512` (`0.42–0.69×` L2m, `0.59–0.75×` DRAM, `0.36–0.49×` Peak RAM) merit a native multi-thread A/B test.
2. **Direct wide-band folding (`band_direct[y >> 5]`) & 1-pass `K`-walker RLE1 streaming:** Unmeasured in Cycle 5; could shorten `BSO`'s 4-load chain to 3 loads and remove Pass 4A if chunks stream into RLE1+CRC.
3. **New topological / combinatorial leads from `origin/master` (`0106c87`, `78f273a`, `eeaea59`):** Bestvina–Handel illegal-turn folding on a run-dynamics graph self-map, ribbon graphs / rotation systems, Dunfield–Thurston weighted train-track splitting carrying ordered values, Agol–Hass–Thurston interval-pairing reduction, and origami lift directions/transport (§6.2 of `2026-11-20-origami-lift.md`).
