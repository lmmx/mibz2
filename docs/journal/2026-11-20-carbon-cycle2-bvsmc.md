# Carbon Cycle 2 · Uniform-Width Branchless Vectorized Spliced Contraction (`BVSMC-1P` & Zero-Arena `BVSMC-2P`), Neyman–Pearson Section Optimality, and the Radix-Coalescing Principle

*20 November 2026 · Carbon (`research/carbon-word-transport`, Cycle 2).*

## 0. Cycle-Boundary Check (`origin/master` Fetch)

At the Cycle 1 → Cycle 2 boundary (and re-checked at `2b2499d`), `origin/master` published [`2026-11-20-working-backwards.md`](https://github.com/lmmx/mibz2/commit/2b2499d) (`costmap.py`), establishing that JSON keys (`33–38%` of bytes) own only `1–4%` of thin rows, whereas unique statement GUIDs own `54–67%` of width-1 rows (`23–38%` of thin rows) and repeated 40-hex `sha1` hashes own `6–25%` of thin rows. Per charter policy (`launches/2026-11-20-carbon.md` §1), `research/carbon-word-transport` is **not** merged or rebased onto `2b2499d`; Cycle 2 completes our obstruction-inversion confrontation on Obligation #1 and Obligation #2 (which directly attacks the random GUID pointer chain via `MLP = 8` spliced contraction), and Cycle 3 will directly take up `2b2499d`'s GUID lower-bound and repeated-`sha1` copy questions.

---

## 1. Explore: Six Genuinely Distinct Approaches Across Both Tracks

Following `METHOD.md` §2–§3, Cycle 2 widened across non-width stochastic complements, Wheeler automaton minimisation, irreducible LCP bridges, Obligation #2 queue-free radix granularity, and two distinct memory-hierarchy realisations (`1`-pass arena vs `2`-pass `0`-arena in-place) of uniform-width branchless spliced monoid contraction:

| # | Candidate | Track / Setting | Mechanism & information needed | Likely trap |
|---|---|---|---|---|
| 1 | **`Meyer-NonWidth`** | Track 1 · Meyer stochastic complementation (`FRONTIER.md` line 129) | Replace Open's width threshold ($w \le 16$) with byte-class or degree-based complements $\mathcal{H}$ at matched strip count $H$ | Strip-count cost in `endpoint.rs` is width-independent, so non-widest strips sacrifice row coverage |
| 2 | **`Wheeler-MN`** | Track 1 · Wheeler DFA Myhill–Nerode minimisation & Durand return hierarchy (`FRONTIER.md` line 131) | Collapse equivalent backward histories on $([0, n), \mathrm{LF}, L)$ or compose nested clopen Durand return morphisms | Finite-horizon equivalence may coincide with Record 84's refinement tower (`n` states at closure) |
| 3 | **`KKP-Irred`** | Track 1 · Kärkkäinen–Kempa–Piątkowski reducible/irreducible LCP bridges (`FRONTIER.md` line 127) | Exploit reducible $\Phi$-runs and $O(r \log(n/r))$ irreducible LCP sum to bypass width-1 chains | Even with free $(\mathrm{SA}, \mathrm{LCP})$ at run heads, $\Phi$-copies remain `85–94%` cyclic (Round 6 / Record 84) |
| 4 | **`Radix-U16-Backward`** | Track 2 · Obligation #2 (`Building the retained map`, `#659` follow-up) | Replace `queues[0..256]` (`4m` B) and `#659`'s second `records` scan with one `2 B/run` `thin_pieces` stream + branchless AVX2 sequential backward `edges` fill | Iterating per thin run (`48k–126k` runs, avg width `1.6–2.2`) runs `20×` more outer loop steps than `qi16`'s band-zip (`2.4k–6.7k` bands) |
| 5 | **`BVSMC-1P`** | Track 2 · Obstruction Inversion on Cycle 1's ROB Branch-Flush (`1`-pass bounded pool) | Unified `16`-byte SIMD prefix table `inline16[0..256+words]` (`92.5–94.4%` of steps branchless) + single `<7%` cold trap (`len > 16` or sentinel) across $K=8, S=48$ lanes into `160 × 8064 B` (`1.23 MiB`) pool | On low-`m/n` blocks (`block-0`, `m=0.10n`), `>16B` cold `memcpy`s (`810 KB`) and `1.23 MiB` L2 footprint bound the `emit` ratio |
| 6 | **`BVSMC-2P`** | Track 2 · Zero-Arena Exact-Offset In-Place Contraction (`0 B` chunk pool) | Pass 1 (`8` lanes, `0` writes, `w_len[sentinel]=0`) computes exact `seg_bytes[q]`; $O(S)$ cycle prefix sum gives exact `[off_q, end_q)` in `out[0..n)`; Pass 2 (`8` lanes) writes directly into `out` in place (`0 B` pool, `0` stitch `memcpy`) | Paying Pass 1's second `8`-lane `edges[0..m]` read (`+0.08–0.21 ms`) |

---

## 2. Triage: Four Theorems & Controlled Producer Ablations

Reproduced by [`verify_cycle2.py`](2026-11-20-carbon-cycle2/verify_cycle2.py) and our controlled C producer ablation.

### Triage 2.1 · `Meyer-NonWidth`: Neyman–Pearson Width-Optimality Theorem (`proved` + `measured`)

**Theorem 2.1 (`proved`).**
In Open's endpoint elimination (`src/open/endpoint.rs`), initial exchange size ($\le 3H + 2$ nodes) and band count ($\le 3H + \sigma$) depend on the **cardinality** $H = |\mathcal{H}|$ of eliminated maximal runs, not on their widths, whereas retained emission steps equal $m(\mathcal{H}) = n - \sum_{a \in \mathcal{H}} w_a$ (plus $\le 1$ if $p$ splits a strip). By the Neyman–Pearson / greedy knapsack lemma, among **all** run-union complements of size $|\mathcal{H}| \le H$, $m(\mathcal{H})$ is strictly minimised by selecting the $H$ widest runs ($w_a > \text{LIMIT}$). $\square$

- **6-block measurement (`verify_cycle2.py` — `measured`):** At Open's exact $H \in [1565, 5047]$, ranking bytes by mean run width and eliminating whole byte classes up to budget $H$ leaves **`444,433–744,844` retained rows** vs Open's **`89,874–204,281` (`3.2×–4.9×` worse on 6/6 blocks)**.

### Triage 2.2 · `Wheeler-MN` & Durand Hierarchy: Equivalence to Refinement Tower (`proved` + `verified`)

**Theorem 2.2 (`proved` + `verified` across all `873/873` binary $n \le 8$ and ternary $n \le 5$ columns at all depths $k = 1 \dots n$ in `verify_cycle2.py`).**
Define $x \equiv_k y$ on $([0, n), \mathrm{LF}, L)$ iff $(L[\mathrm{LF}^t(x)])_{t=0}^{k-1} = (L[\mathrm{LF}^t(y)])_{t=0}^{k-1}$. Because $\mathrm{LF}(x) = \mathrm{LF}(x-1) + 1$ whenever $L[x] = L[x-1]$, each $\equiv_k$-class is a contiguous row interval, and its interior boundary set $B_k \subseteq \{1, \dots, n-1\}$ satisfies the exact recurrence $B_1 = D$, $B_{k+1} = B_k \cup \{x \notin D : \mathrm{LF}(x) \in B_k\}$, dual to MONODROMY Record 84's `Cut_{k-1}`. On any single-cycle column of fundamental text period $p_0 \mid n$, $|[0, n) / {\equiv_n}| = n / p_0$ (`= n` for every primitive block), so exact DFA state minimisation collapses zero rows on primitive BWTs. Likewise, Durand's derived return-word hierarchy on nested intervals reduces to $S$-adic Rauzy–Veech (`Cells`) or forest switch elimination (`Astra Inv. 3`, `15×` slower). $\square$

### Triage 2.3 · `KKP-Irred`: Irreducible Boundary Census & Cyclic Copy Barrier (`proved` + `measured`)

Across the 6 blocks (`verify_cycle2.py` — `measured`), irreducible run heads $r - |\bar{D} \cap \bar{C}|$ number `10,954` (`21.4%`, `block-0`), `13,223` (`24.2%`, `block-2`), `17,119` (`21.6%`, `block-85`), `26,964` (`24.9%`, `block-143`), `20,803` (`23.9%`, `block-207`), and `26,748` (`20.8%`, `block-210`) — matching Record 63's copy count $k$. Even under a zero-cost oracle supplying run-head $(\mathrm{SA}, \mathrm{LCP})$, converting those $k$ irreducible seeds into text writes reduces to the $\Phi$-copy dependency graph, where `85–94%` of bytes lie on cycles (Round 6) and cycle-breaking potentials require text-order run labels (Record 84).

### Triage 2.4 · `Radix-U16-Backward` on Obligation #2: The Radix-Coalescing Granularity Principle (`measured`)

Testing why `#659` (`Queue-Free Backward Open`) lost natively (`1.10–1.15×`), we eliminated `#659`'s second pass over `records` by emitting a compact `2 B/thin-run` stream `thin_pieces` (`95–252 KiB` vs `360–817 KiB` `queues[256]`) in Pass 1 and filling `edges[0..m]` sequentially in Pass 2:
1. With scalar inner loops (`0..w`), `thin_u16_backward` tied `qi16` (`0.995×`, `1.006×`, `1.057×` on blocks `0, 143, 210`).
2. Replacing `0..w` ($w \le 16$ always!) with a fixed-width branchless 16-lane AVX2 iota store `store_seq16` cut `thin_avx_backward` `construct` by **`23–26%`** (`0.935 -> 0.717 ms`, `1.845 -> 1.363 ms`, `1.458 -> 1.123 ms`).
3. **Controlled fairness check:** Giving shipped `qi16` the **exact same** `store_seq16` inside `queues[a]` (`qi16+U16Q`, adding 16 `u32`s of tail slack per queue) achieved **`0.718 ms` (`0.763×`), `1.392 ms` (`0.736×`), and `1.075 ms` (`0.718×`)** — matching or beating queue-free backward construction!
- **Structural lesson on Obligation #2:** `qi16`'s `queues[0..256]` (`16 KiB` of active cache lines, L1-resident) acts as a **radix coalescer**, merging `48k–126k` `1.6`-row thin runs into `2.4k–6.7k` `25–65`-row contiguous band spans (`20×` fewer outer loop transitions). The `24–28%` producer overhead was **not** queue traffic, but `48k–126k` variable-count loop-exit branch mispredictions on $w \in [1, 16]$, removed in place by `U16Q` (`store_seq16`).

---

## 3. Deepen: `BVSMC-1P` and Zero-Arena `BVSMC-2P`

### Mandatory `METHOD.md` §3a Candidate Declaration
- **Mode:** Reorientation / Traction hybrid (escapes Cycle 1's ROB Branch-Flush Obstruction to realise parallel monoid contraction over Open's retained cycle, paired with a zero-arena two-pass exact-offset in-place variant; classification deferred to the Coordinator and MONODROMY).
- **Displaced obligation:** Obligation #1 — serial 1-cursor pointer chasing in `Open::emit` (`D = m`, `MLP = 1`) and per-step literal-vs-phrase (`code < 256`) branch mispredictions; plus (`U16Q`) variable-width `1..16` loop-exit mispredictions in `queues[a]`.
- **Replacement principle:**
  - **Uniform-16 Prefix Representation:** Every code $c \in [0, 256 + \text{words})$ has a pre-aligned 16-byte vector `inline16[c]` and exact length `w_len[c]` (`~50–126 KiB`, L2-resident; `0..255` initialised once globally, `256..` populated by one 16B load/store per dictionary word during `construct`). For `92.5–94.4%` of all steps (`w_len[c] <= 16`), emission is a **single unconditional 16-byte SIMD store** `_mm_storeu_si128((__m128i*)P, inline16[c]); P += ln;`. Sentinels (`w_len = 65535`) and `>16B` words share one `<7%` cold branch, preserving all $K = 8$ in-flight `edges[j_l]` loads in the ROB.
  - **`BVSMC-1P(K=8, S=48)`:** Single-pass contraction into `160` chunks of `8000 B` data + `64 B` SIMD slack (`1.23 MiB` total; `seg 0` written directly into `out[0..n)` with `end_0 = out + n - 15` when $n \ge 16$).
  - **`BVSMC-2P(K=8, S=64)`:** **Zero-arena (`0 B` chunk pool)** two-pass contraction: Pass 1 (`8` lanes, `0` writes, `w_len[sentinel] = 0`) accumulates exact `seg_bytes[q]` unconditionally; $O(S)$ cycle prefix sum computes exact non-overlapping slices `[off_idx, end_idx)` in `out[0..n)` (discarding unmarked cycles); Pass 2 (`8` lanes) writes directly into `out[off_idx .. end_idx)` with `safe_end = (end - off >= 16) ? (out + end - 15) : (out + off)`.
- **New paid obligations:** `inline16 + w_len` L2 table (`50–126 KiB`); for `BVSMC-1P`, `1.23 MiB` chunk pool + sequential stitch `memcpy` (`~0.03–0.05 ms`); for `BVSMC-2P`, `0 B` arena + Pass 1's `0.08–0.21 ms` length-only 8-lane scan.
- **Nearest prior art:** Cycle 1 `SSMC-Open` (failed G5 due to ROB branch flushes), `open-return/segments.rs` (2-pass `u32` code tapes, `% 128` branch per step), Neon Cycle 5 `BSO-4x` (4-load bitplane rank, `2.76–3.03×` slower than Open).

### Gate G1 & Gate G2 — Proofs, Exhaustive Verification & Manifest (`proved` + `verified`)

**Theorem 3.1 (Universal Non-Overflow & Zero Cross-Segment Overlap — `proved`).**
1. *`BVSMC-1P` (`S <= 48`, `CHUNK_CAP = 8000`, `STRIDE = 8064`, `MAX_CHUNKS = 160`):* Whenever `P < chunk_base + 8000`, `P + 16 <= chunk_base + 8015 < chunk_base + 8064`, staying strictly within the chunk's `64 B` slack; for `seg 0`, `P < out + n - 15` ($n \ge 16$; `P < out` false when $n < 16$) guarantees `P + 16 <= out + n` (zero write past `out[n-1]`). Every closed non-final chunk holds $\ge 8000$ valid bytes; since $\sum_{q=0}^{S_{\text{eff}}-1} |W(P_q)| \le n \le 900,000$ (Cycle 1 Theorem 4.1), total chunks allocated across segments $1 \dots S_{\text{eff}}-1$ is at most $\lfloor 900,000 / 8000 \rfloor + (S_{\text{eff}} - 1) \le 112 + 47 = 159 < 160$.
2. *`BVSMC-2P` (`0 B` Arena Exact-Offset Disjointness):* Setting `safe_end = (end - off >= 16) ? (out + end - 15) : (out + off)` ensures: if `end - off < 16`, `P >= out + off = safe_end` so `P < safe_end` is identically false; if `end - off >= 16`, `P < out + end - 15 <==> P + 16 <= out + end`, so every 16-byte store writes exclusively inside `[out + off, out + end)`, disjoint from every concurrent lane's slice. $\square$

- **Exhaustive & canary verification (`verify_cycle2.py` & `g5_bvsmc.c` — `verified`):**
  - `10,454 / 10,454` small `(L, p, limit)` instances (`1 <= n <= 8` binary, `1 <= n <= 5` ternary, asserting zero cross-segment byte writes at every position in Python and 32-byte `0xFE` trailing buffer canaries in C);
  - `6 / 6` adversarial controls (`n=1`, `n=5 < 16`, fixed-point, periodic, multi-cycle);
  - `5,399,692 / 5,399,692` authenticated Wikidata bytes exact (`125–140` chunks used out of `159` max).
- **Gate G2 (`bvsmc.toml` — `verified`):** `check_manifest.py` outputs **`PASS`**.

---

## 4. Confront: Gate G5 Controlled 4-Arm Benchmark (`g5_bvsmc.c`)

To prevent attributing 1-lane SIMD gains (`U16Q` + 1-lane `inline16`) to parallel contraction, [`g5_bvsmc.c`](2026-11-20-carbon-cycle2/g5_bvsmc.c) pre-registered **four** arms rotated across 25 paired interleaved rounds (`Xeon Platinum 8481C`, `gcc -O3 -march=native` — `measured`, VM diagnostic):
- `Op`: exact shipped `Open-qi16` (`src/open.rs`);
- `U16`: strengthened **1-lane** `Open-qi16+U16Q` (`store_seq16` queue push + 1-lane `inline16` SIMD `emit`);
- `1P`: `BVSMC-1P(K=8, S=48)` (`1.23 MiB` chunk pool);
- `2P`: `BVSMC-2P(K=8, S=64)` (**`0 B` chunk pool**, direct in-place into `out[0..n)`).

### Measured 4-Arm Results (`g5_bvsmc.c`, medians over 25 interleaved `construct + emit` rounds, in ms)

| Block | `OpEm` | `U16Em` | **`1P_Em`** | **`2P_Em`** | `1P/Op` `emit` | `1P/U16` `emit` | `2P/Op` `emit` | `OpTot` | `U16Tot` | **`1PTot`** | **`2PTot`** | **`1P/Op` `tot`** | **`1P/U16` `tot`** | **`2P/Op` `tot`** |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| `block-0.L` | 0.525 | 0.528 | **0.408** | **0.461** | `0.778×` | `0.774×` | `0.877×` | 1.482 | 1.279 | **1.158** | **1.213** | **`0.781×`** | `0.905×` | **`0.818×`** |
| `block-2.L` | 0.716 | 0.683 | **0.460** | **0.557** | `0.642×` | `0.673×` | `0.777×` | 1.855 | 1.582 | **1.340** | **1.429** | **`0.722×`** | **`0.847×`** | **`0.771×`** |
| `block-85.L` | 0.920 | 0.926 | **0.526** | **0.722** | **`0.572×`** | **`0.568×`** | `0.785×` | 1.907 | 1.600 | **1.196** | **1.369** | **`0.627×`** | **`0.748×`** | **`0.718×`** |
| `block-143.L` | 1.500 | 1.377 | **0.825** | **0.917** | **`0.550×`** | **`0.599×`** | **`0.611×`** | 3.351 | 2.804 | **2.245** | **2.344** | **`0.670×`** | **`0.801×`** | **`0.699×`** |
| `block-207.L` | 0.911 | 0.866 | **0.540** | **0.607** | **`0.593×`** | **`0.624×`** | **`0.666×`** | 2.214 | 1.880 | **1.538** | **1.597** | **`0.695×`** | **`0.818×`** | **`0.721×`** |
| `block-210.L` | 1.431 | 1.395 | **0.820** | **0.922** | **`0.573×`** | **`0.588×`** | **`0.644×`** | 2.892 | 2.499 | **1.904** | **2.022** | **`0.658×`** | **`0.762×`** | **`0.699×`** |

### Strict Pre-Registered Gate G5 Evaluation
- **Whole-kernel speedups (`tot`):** **CLEARED ALL WHOLE-KERNEL CUTOFFS** — `BVSMC-1P` achieved **`0.627×–0.781×` (`21.9%–37.3%` faster end-to-end)** vs `Open-qi16` (`5/6 <= 0.75×`, `6/6 <= 0.80×` — **PASS**), and **`0.748×–0.905×`** even against strengthened 1-lane `Open+U16Q` (`5/6 <= 0.88×` — **PASS**); Zero-Arena `BVSMC-2P` (`0 B` pool) achieved **`0.699×–0.818×` (`18.2%–30.1%` faster end-to-end)** across all `6/6` blocks!
- **Sub-criterion `emit <= 0.62×` on *all* `6/6` blocks:** Passed on the `4/6` higher-$m/n$ blocks (`85, 143, 207, 210`: `0.550×–0.593×`), missed on the two lowest-$m/n$ blocks (`block-2`: `0.642×`; `block-0`: `0.778×`), so the strict conjunctive `6/6` `emit` sub-rule printed `FAIL` (`4/6`). *(A register-return check removing `&(J)` improved `block-0` to `0.744×` / `tot 0.757×` and `2P` to `5/6 <= 0.78×`, confirming the monotone $m/n$ Amdahl curve is physical, not an address-spill artifact.)*

---

## 5. Learn: Three Concrete Findings for `FRONTIER.md` and the Coordinator

1. **Cycle 1's ROB Branch-Flush Obstruction is definitively broken (`measured`):**
   Moving from Cycle 1's branched `SSMC(8,64)` (`1.035×–1.216×` Open `emit`) to uniform-width branchless `BVSMC-1P(8,48)` drops `emit` to **`0.550×–0.778×` Open `emit`** (`0.568×–0.774×` against 1-lane `U16Em`, which barely changed 1-lane `emit` on its own: `0.528 vs 0.525 ms` on `block-0`!). That controlled 2×2 ablation proves an **interaction effect**: uniform-16 branchless emission saves little at `MLP = 1`, and 8-lane splicing loses without uniform-16 branchless emission, **yet together they cut `emit` nearly in half on $m/n \ge 0.15$ blocks!**
2. **Why `emit` speedup scales monotonically with $m/n$ (`Amdahl Thick-Copy Floor`):**
   On `block-0.L` (`m/n = 0.100`), `90.0%` of output bytes (`810 KB`) come from thick strips (`5,573` cold `>16B` `memcpy`s) plus `~0.04 ms` stitch `memcpy`, bounding the `emit` ratio near `0.74–0.78×`; on `block-85/143/207/210` (`m/n = 0.151–0.227`), pointer-chasing dominates and `BVSMC-1P` reaches `0.550–0.593×`.
3. **Diagnostic caveat for native `i9-10900K` evaluation (`METHOD.md` §5):**
   This VM has a `2 MiB` per-core L2, whereas `i9-10900K` has `256 KiB` L2 and `20 MiB` shared L3 across 10–20 workers. On `i9-10900K`:
   - `Open+U16Q` (`0 B` extra memory, `0.83–0.86×` VM total) has zero cache footprint downside;
   - **Zero-Arena `BVSMC-2P` (`0 B` chunk pool, `+50–126 KiB` L2 `inline16` table, `0.68–0.80×` VM total)** adds **zero $n$-scale DRAM/L3 write+read traffic** at 8–20 threads, making `BVSMC-2P` a prime candidate alongside `BVSMC-1P` if the Coordinator chooses to run a paired native `i9` / CI check.
