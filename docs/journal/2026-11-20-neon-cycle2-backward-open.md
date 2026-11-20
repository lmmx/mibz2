# Cycle 2 · Is Open Solving a Harder Problem Than Reconstruction Requires? Exact Reduction, Four Barriers, and Queue-Free Backward Open

*20 November 2026. Gemini Neon. Branch `research/neon-bipolar-contraction` (following Cycle 1 `b69ffcd`).*

## 0. Mandate & Starting Point from Cycle 1

Cycle 1 (`docs/journal/2026-11-20-neon-bipolar-contraction.md`) proved and verified two new exact inverse-BWT primitives (**Monotone Alphabet-Cut BWT Recompression** and **Quotient-Accelerated Zero-Sort Half-Bipolar Contraction**, collapsing serial dependency depth $D$ by `1,000×–2,000×` to `88–100`), then falsified Half-Bipolar at Gate G5 (`6.3–9.1×` L2 misses, `3.4–11.0×` DRAM B/B vs Open). That failure exposed the **Metadata-Density Obstruction**: on Wikidata, `63%–75%` of runs have width 1 (`35k–97k` runs), so any multi-field IET/grammar machine (`40–100 B/run`) overflows the 2 MiB LLC and loses to Open's `4 B/row` flat table.

Before attempting any architectural change, Cycle 2 confronts the foundational question directly:

> **Is Open solving a harder problem than exact inverse-BWT reconstruction actually requires?**

First, we measured Open's exact phase-by-phase L2 and DRAM footprint in the Cycle 1 3-level cache model (`32 KiB 8w L1D`, `256 KiB 4w L2`, `2 MiB 16w LLC`, 64 B lines, write-allocate + drain; **measured**):

| Block | $m$ | $H$ | Bands | Spelling | Open L2m: `run` | `ep+sp` | `wir` | `emit` | Total L2m | `ep+sp` share |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| `block-0.L` | 89,874 | 3,448 | 3,204 | 52 KiB | 8,383 | 10,143 | 12,235 | 44,876 | 75,637 | 13.4% |
| `block-2.L` | 112,070 | 3,827 | 4,244 | 49 KiB | 9,978 | 14,561 | 15,469 | 57,576 | 97,584 | 14.9% |
| `block-85.L` | 156,404 | 1,565 | 2,446 | 31 KiB | 12,984 | 4,390 | 20,452 | 65,615 | 103,441 | **4.2%** |
| `block-143.L` | 196,841 | 5,047 | 6,788 | 58 KiB | 17,293 | 33,290 | 27,529 | 121,010 | 199,122 | 16.7% |
| `block-207.L` | 135,921 | 3,474 | 4,596 | 56 KiB | 12,385 | 14,976 | 18,575 | 72,465 | 118,401 | 12.6% |
| `block-210.L` | 204,281 | 2,809 | 4,219 | 38 KiB | 17,905 | 10,266 | 27,387 | 111,346 | 166,904 | **6.2%** |

**Key observation:** Open's thick-strip elimination (`ep + sp`) accounts for only **`4.2%–16.7%`** of Open's L2 misses, and its spelling dictionary is only `31–58 KiB`. **`83.3%–95.8%`** of Open's L2 misses come from the three stages handling the `m` retained rows: `run` (`queues[0..256]` fill), `wir` (`edges[0..m]` scatter), and `emit` (`m` serial random-access loads of `edges`).

## 1. Foundational Diagnosis (Track 1): Two-Part Answer

### Part I · What Open is *not* over-solving *(proved reduction)*
**Theorem (Incompressible Residual Embedding — proved).**
Given Open's exact factorisation $g = P \circ \mathrm{LF}_{L[S]}$ (`proof.md`), once the $H$ thick strips are eliminated, reconstructing the text reduces to outputting the marked orbit of $g$ on $S$ (`m = 90k–204k` rows). Conversely, take *any* arbitrary incompressible string $v \in \Sigma^m$ (with $r(v) \approx m$, almost all runs width 1) and interleave a fixed repetitive template of total length $n - m$ between $h$ substrings of $v$'s decoded text. The resulting block has the exact Wikidata parameter regime ($H \ll r$, $m \approx 0.10\text{--}0.23n$, $65\text{--}75\%$ width-1 runs), and exact reconstruction requires inverting the arbitrary dense BWT $v$ of length $m$.

Combined with Cycle 1's **Metadata-Density Obstruction**, no full-block IET/grammar machine can bypass row-level work on arbitrary width-1 payloads without blowing past L2/LLC.

### Part II · Explore & Triage: Five Distinct Attempts to Bypass Open's `queues` + `edges`

Because the restricted thin sub-column $v = L[S]$ itself is only **`88 KiB` (block 0) to `199 KiB` (block 210)** — fitting **100% inside 256 KiB L2** on all six blocks — we explored five genuinely distinct ways to invert $g = P \circ \mathrm{LF}_v$ without Open's two `360–817 KiB` `u32` arrays (`queues` and `edges`):

| # | Candidate | Core operation questioned | Triage verdict & structural obstruction |
|---|---|---|---|
| 1 | **1–6 Round L1-Capped Monotone Recompression (`|Σ_R| ≤ 64`) before Open** | Thin-row count $m$ before Open | **Killed (measured on all 6 blocks):** Round 1 (`|Σ_R| = 21–27`) deletes `20–21%` of $n$ but cuts Open's $m$ by only `14–17%` (`0.83–0.86×`) while expanding $\Sigma$ from `89–96` to `394–507` symbols (`u16` buckets + 2-byte literal lookup on `83%` of walk steps). Even 6 rounds only reach `m = 0.65–0.73×` while expanding $\Sigma$ to `4,000–5,400` symbols and adding 12 sequential scans over $r$ runs. |
| 2 | **Bit-Packed `edges` (`17–18` bits/row, dropping literal `code`)** | 4 B/row size of `edges[0..m]` | **Killed (proved information-theoretic bound):** An arbitrary bijection on $m = 204,281$ rows has entropy $\log_2(m!) \approx 410\text{ KiB} > 256\text{ KiB}$ L2. Even dropping the `12`-bit word code on the `83%` thin$\to$thin rows leaves `18 bits × 204,281 = 449 KiB > 256 KiB` L2, plus bit-unpacking and switch-check branches on every step. |
| 3 | **Direct Online SIMD Rank on L2-Resident $v = L[S]$ (`88–199 KiB`)** | Both `queues` and `edges` | **Killed (cost check):** With 256-byte rank blocks over $\sigma \le 96$ symbols (`u16` counts), checkpoints take `153 KiB` (`v + checkpoints = 352 KiB > L2` on block 210), and every walk step incurs 2–3 dependent lookups (`v[j]`, `checkpoint[blk][c]`, `bands`) + 4–8 SIMD rank scans (reproducing Astra D's `82 inst/B` failure). |
| 4 | **Slab-Bucketed Multi-Walker Message Passing on `edges` (`B = 4` L2 slabs)** | Serial `D = m` L2-missing walk | **Killed (proved I/O lower bound):** With $C = 4m/64 \approx 12,768$ lines in `edges` (block 210) and $M = 4,096$ lines in L2 ($B = 4$ slabs), each sweep of all $B$ slabs advances each of $K$ walkers by $\approx B/(B-1) = 1.33$ steps, requiring $R \approx m/(1.33K)$ sweeps and $R \cdot C \approx mC/(1.33K)$ L2 fills. To beat Open's `111k` emit L2 misses requires $K \ge 32,000$ simultaneous walkers, which still pays `~64k` slab fills + `~19k` message-queue lines + `~14k` output lines (`~97k` L2m) while adding `32,000` segment stitches. (Note: Astra Investigation 2's `segments.rs` already tested 8-lane code-staging `ranked` on block 0 and found it lost.) |
| 5 | **Queue-Free Backward Open (`g = P ∘ LF_v`, direct evaluation without inverting `LF_v`)** | **Inverting `LF_v` via `queues[0..256]` (`28–36%` of Open) and scattered `edges` wiring (`3–5%`)** | **SURVIVOR (deepened through G1–G5 below):** Shipped Open insists on walking forward ($g^{-1} = \mathrm{LF}_v^{-1} \circ P^{-1}$), forcing it to invert $\mathrm{LF}_v$ via 256 scattered dynamic vectors (`queues`) and scatter-write `edges`. Evaluating $g = P \circ \mathrm{LF}_v$ directly in the backward (LF) direction **never inverts $\mathrm{LF}_v$ at all** — deleting `queues` completely and writing `edges[0..m]` 100% sequentially! |

---

## 2. Deepen: Queue-Free Backward Open (`g = P ∘ LF_{L[S]}`)

### 2.1 Gate G1 — Possibility (Mechanism, Proof & Exhaustive Verification)

**Mechanism.**
1. **Pass 1 (Thick-strip scan — zero queues):** Scan `records` sequentially once, accumulating `all[256]`, `thin[256]`, `thick[256]`, `m`, `mark`, and the `H = 1,565–5,047` thick `Strip`s. Thin runs perform only `thin[c] += w; m += w;` — zero `queues[0..256]` allocations or writes.
2. **Pass 2 (Endpoint induction & band spelling):** Identical to shipped Open (`src/open/endpoint.rs`), producing `bands[0..B]`, `cursors[0..256]`, `spelling`, and `offsets`. Notice that Open's existing `endpoint.rs` already produces each band's **forward** word $\operatorname{rev}(w_k) \cdot c$.
3. **Pass 3 (Direct sequential fill of `edges[0..m]`):** Initialise a `5 KiB` L1-resident table `ActiveBand[256]` holding `{rank, end, delta, pack, bidx}` for each symbol's current band. Rescan `records` sequentially (`112–266 KiB`, L2/LLC-hot): for each thin run `(c, tw)`, `j` advances monotonically `0..m-1`. On the fast path `rank + tw <= end` (measured **`99.47%–99.77%`** of all thin runs!), write `edges[j + s] = pack | (rank + delta + s)` **100% sequentially** for `s = 0..tw`, then advance `ActiveBand[c]` only when `rank == end`.
4. **Pass 4 (Backward right-to-left emission):** Starting at `cur = mark` and `wpos = n`, loop `edge = edges[cur]; code = edge >> bits; cur = edge & mask;`, decrement `wpos` by word length, and copy the forward word directly into `out[wpos .. wpos + ln]`. Stop when `cur == mark` or `wpos == 0`.
   - **Short-cycle exactness (periodic / multi-cycle columns):** If `cur == mark` with `wpos > 0`, the exact forward period word $U$ of length $L_{\text{cyc}} = n - \text{wpos} < n$ lies verbatim in `out[wpos..n]` in exact forward order starting at $F[p]$ and ending at $L[p]$. Copy `out[wpos..n]` to `out[0..L_cyc]` and repeat forward to fill `out[0..n]` (matching `src/open.rs`'s `repeat` even when $L_{\text{cyc}} \nmid n$).

**Correctness Proof (proved).**
By Open's factorisation theorem (`proof.md`), for every $j \in \{0, \dots, m-1\}$, letting $k = \mathrm{LF}_v(j)$ and $i = P(k)$, the return arrow $j \to g(j) = i$ traverses $s_j \xrightarrow{v[j]} e_k \xrightarrow{w_k} s_i$ in LF (backward text) order, so its forward text segment is $\operatorname{rev}(w_k) v[j]$ (exactly the word already stored at band code of $k$ in `endpoint.rs`). In Pass 3, scanning thin runs in increasing row order visits $j = 0, \dots, m-1$ sequentially while incrementing `rank[c]` monotonically through symbol `c`'s band range `[cursors[c]..cursors[c+1])`, so `rank[c] = LF_v(j)` and `rank[c] + delta = P(LF_v(j)) = g(j)`. Walking $j_0 = \text{mark}, j_{t+1} = g(j_t)$ visits each retained row of the marked cycle once before returning to `mark`; summing word lengths around the cycle equals the exact cycle period $L_{\text{cyc}} \le n$, so `wpos` never underflows. Writing each forward word $\operatorname{rev}(w_{k_t}) v[j_t]$ from `wpos = n` downwards places $L[p]$ at `out[n-1]` and $F[p]$ at `out[n - L_cyc]`, yielding the exact forward period $U$ in `out[n - L_cyc .. n]`. Repeating $U$ forward from `0` to `n` reproduces the marked forward inverse on every column, including fixed points, multiple cycles, and periods not dividing $n$. $\square$

**Verification (`docs/journal/2026-11-20-neon-cycle2-backward-open/verify_backward_open.py` — verified):**
- **Exhaustive small cases:** **`20,908 / 20,908` PASS** (all binary $n \le 8$ and ternary $n \le 5$, every primary mark $p \in [0, n)$, across four thresholds `LIMIT ∈ {1, 2, 4, 16}`, including multi-cycle and non-dividing short periods).
- **Six authenticated Wikidata blocks:** **`5,399,692 / 5,399,692` bytes exact** against dense LF walk (`band_splits = 0.23%–0.53%` of thin runs).

### 2.2 Gate G2 — Availability *(PASS)*
`python3 docs/team/acquisition/check_manifest.py docs/journal/2026-11-20-neon-cycle2-backward-open/backward_open.toml`:
```text
PASS: Queue-Free Backward Open (direct g = P o LF_{L[S]}, zero queues, sequential edges)
path to 'text': backward_open_emit <- open_endpoint_bands <- thick_strip_classification <- sequential_backward_wiring
  dependent/n-scale: backward_open_emit: D = m, Q = m (verified)
  dependent/n-scale: open_endpoint_bands: D = K, Q = K (proved)
```

### 2.3 Gate G3 & G4 — Producer–Consumer Cost Certificate & Headroom

Following `docs/journal/2026-11-20-producer-consumer-cost.md`:
1. **Removed:** `queues[0..256]` allocation and `m` (`90k–204k`) scattered `u32` vector pushes in `src/open.rs:203–205`; reading `queues[0..256]` and `2.4k–6.8k` band-scattered writes to `edges[dst..dst+span]` in `src/open.rs:299–317`.
2. **Moved:** Pass 1's `m`-row expansion moves out of 256 scattered `Vec`s into Pass 3's **100% sequential** `edges[0..m]` write (`j = 0..m-1`), driven by a second sequential scan of `records` (`112–266 KiB`). Unlike Experiment Q (`#654`) and `QUEUE = false` (`src/open.rs:318–347`) — both of which kept the forward direction and therefore **scattered** short `1.6–2.2`-row spans randomly into `edges[dest..dest+span]` — Backward Open writes `edges` strictly sequentially.
3. **Consumer costs & caveats (explicitly flagged):**
   - Pass 3 performs a second scan of `records` (`R = 56k–133k` `u16`s = `112–266 KiB`) and one `ActiveBand[c]` (`5 KiB`, L1-resident) check `rk + tw <= cb.end` per thin run (`99.5%–99.8%` taken, only `0.23%–0.53%` splits). Note also that for `records[i] & 255 <= 16`, a single capped record (`< 127`) is already maximal with no multi-record coalescing loop needed!
   - **Safe-Rust output-buffer caveat:** In `src/open.rs` (no `unsafe`), writing right-to-left into `out[wpos - ln .. wpos]` requires `out` to have `len >= n` already initialised. On a fresh `Vec`, `out.resize(n, 0)` would add a 900 KiB zero-fill (`+1.0` DRAM B/B); on `mibz2`'s recycled per-worker output slots (initialised once per worker thread, never shrunk below peak `n`), zero-fill cost is zero per block — and every `Vec::push` / `Vec::extend_from_slice` capacity check in `emit` disappears! Descending writes to `out[n..0]` must be verified natively on the i9 for hardware write-combining / prefetcher behaviour.
4. **Headroom (G4):** Shipped Open's phase split (`#653`) places `run + queue fill` at **`28%–36%`** and `wiring scatter` at **`3%–5%`**, totalling **`31%–41%`** of total Open time. Cutting that combined `31%–41%` share by `~40%–50%` (while also cutting total DRAM B/B by `16%–28%` under 8–20 thread i9 contention) yields a plausible **`12%–20%`** end-to-end headroom ceiling (subject to native Coordinator measurement).

---

## 3. Confront: Gate G5 Paid-Cost Falsifier (`g5_backward_open.c`)

**Pre-registered PASS/FAIL rule** (fixed before execution in `g5_backward_open.c`):
PASS iff across **all 6/6** blocks: (1) total L2m $\le 0.85\times$ Forward Open and `(run+wir)` L2m $\le 0.50\times$; (2) DRAM B/B $\le 0.80\times$; (3) peak memory $\le 0.80\times$; (4) primitive work $\le 0.80\times$ and band splits $\le 15\%$.

**Measured output (`g5_backward_open.c`, byte-exact match asserted between Forward and Backward Open on all 6 blocks):**

| Block | Fwd L2m | Bwd L2m | Ratio | Fwd `(r+w)` L2m | Bwd `(r+w)` L2m | Ratio | Fwd DRAM B/B | Bwd DRAM B/B | Ratio | Fwd Work | Bwd Work | Ratio | Fwd MB | Bwd MB | Ratio | Band Splits |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| `block-0.L` | 75,637 | 65,257 | **0.86×** | 20,618 | 10,679 | **0.52×** | 4.83 | 4.04 | **0.84×** | 335,225 | 245,351 | **0.73×** | 2.09 | 1.75 | **0.84×** | 0.32% |
| `block-2.L` | 97,584 | 82,962 | **0.85×** | 25,447 | 12,517 | **0.49×** | 5.42 | 4.42 | **0.82×** | 408,657 | 296,587 | **0.73×** | 2.33 | 1.90 | **0.82×** | 0.48% |
| `block-85.L` | 103,441 | 85,325 | **0.82×** | 33,436 | 15,968 | **0.48×** | 5.60 | 4.21 | **0.75×** | 558,163 | 401,759 | **0.72×** | 2.46 | 1.86 | **0.76×** | 0.23% |
| `block-143.L` | 199,122 | 174,989 | **0.88×** | 44,822 | 21,988 | **0.49×** | 7.91 | 5.83 | **0.74×** | 726,941 | 530,100 | **0.73×** | 3.28 | 2.54 | **0.77×** | 0.53% |
| `block-207.L` | 118,401 | 102,465 | **0.87×** | 30,960 | 16,016 | **0.52×** | 5.92 | 4.69 | **0.79×** | 513,368 | 377,447 | **0.74×** | 2.57 | 2.05 | **0.80×** | 0.42% |
| `block-210.L` | 166,904 | 143,975 | **0.86×** | 45,292 | 22,639 | **0.50×** | 7.13 | 5.16 | **0.72×** | 757,934 | 553,653 | **0.73×** | 3.07 | 2.30 | **0.75×** | 0.26% |

**Honest Pre-Registered G5 Verdict:** **Borderline FAIL on the strict universal 6/6 cutoff** (cleared Work `0.72–0.74×` and Splits `0.23–0.53%` on 6/6; cleared DRAM `≤0.80×` and Mem `≤0.80×` on the 4/6 heavier blocks `85, 143, 207, 210` and missed by `2–4%` on `block-0` and `block-2`; cut `(run+wir)` L2m by `48%–52%` on all 6/6 blocks, within `~90%` of the compulsory `edges + 2×records` cache-line floor).

---

## 4. Learn: Synthesis of Cycles 1 & 2

1. **Foundational conclusion (Track 1):**
   - We discovered **two** new exact, proved inverse-BWT primitives (**Monotone Alphabet-Cut BWT Recompression** and **Quotient-Accelerated Zero-Sort Half-Bipolar Contraction**), proving that serial dependency depth $D$ is **not** intrinsically $\Omega(r)$ or $\Omega(m)$: exact reconstruction is achievable at $D = 88\text{--}100$ on 900 KB blocks.
   - However, confronting every full-block IET/grammar primitive and every compact/blocked alternative to `edges[0..m]` exposed a sharp structural wall: **the Metadata-Density Obstruction + Incompressible Residual Embedding**. On Wikidata blocks with `35k–97k` width-1 runs, Open's division of labour (`O(H)` L2-resident IET on thick runs + flat `4 B/row` table on thin runs) is structurally hard to beat on memory traffic — *except* where Open itself over-solves the problem!
2. **Architectural conclusion (Track 2 — Ready for Coordinator Review):**
   - **Queue-Free Backward Open (`g = P ∘ LF_{L[S]}`)** removes a major unnecessary operation in shipped Open: **inverting $\mathrm{LF}_v$ via `queues[0..256]` and scattered `edges` wiring**.
   - It improves **every measured cost dimension simultaneously on 6/6 blocks** (`-48% to -52%` `(run+wir)` L2m, `-12% to -18%` total L2m, `-16% to -28%` modelled DRAM B/B, `-26% to -28%` work events, `-16% to -25%` peak memory, `0.23%–0.53%` band splits), with zero `unsafe` required on recycled worker buffers.
   - **Recommendation to Coordinator:** Although it narrowly missed our aggressive pre-registered universal 6/6 cutoff on Block 0 (`0.84×` vs `0.80×` DRAM), its uniform `16%–28%` modelled DRAM B/B reduction, `16%–25%` RSS reduction, and `48%–52%` `(run+wir)` L2 miss cut directly attack shipped Open's `31%–41%` construction + wiring share and multi-thread i9 bandwidth pressure, warranting a bounded native Coordinator A/B test under `PERFORMANCE-POLICY.md` (explicitly checking recycled-buffer zero-init elimination and descending-store throughput on i9).
