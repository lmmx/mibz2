# Wormhole Cycle 5 — Pigeonhole Envelopes, Direct-Linked `Node16` Morse Elimination, Full Chaitin Lifetime Coloring, and 5-Cycle Synthesis

**Date:** 2026-11-21  
**Branch:** `research/carbon-wormhole`  
**Base `origin/master`:** `a87b22d5446e1866f1ecd82452f0dc56d345d46d` (`mibz2 0.5.8`)  
**Admitted-BWT Corpus (`origin/data/admitted-bwt`):** `41c03a5b22d630adb171231f5225da348b8055af` (`6/6` blocks SHA-256 authenticated against `manifest.json`)  
**Cycle:** 5 of 5 (Final Cycle & Handoff Synthesis)

---

## 1. Where Cycle 4 Broke, and the Final Missing Link

Cycle 4's stage-attributed confrontation (`docs/journal/2026-11-21-wormhole-cycle4-dihedral-bidi-and-conjugate-stream.md`) overturned Known Obstruction `#659` (`Queue-Free Backward Open`, `1.10–1.15×` loss) via **Kakutani's (1943) Skyscraper Conjugacy** (`CSW-Open`), winning `6/6` blocks (`0.973–0.991×` `Open058`) at `0.573×` persistent scratch (`2,668.6 KiB` vs `4,656.5 KiB`) with `0 B` queues. Yet Cycle 4's stage split also exposed **Break 4**:

1. **Unaliased `Endpoint` bloat (`~1,300 KiB` persistent, `399–765 µs`):** [`Endpoint`](../../../src/open/endpoint.rs) allocates 24-byte `u32`-linked [`Node`](../../../src/open/endpoint.rs) structs plus four intermediate `Vec<u32>` staging buffers (`order[0..2]`, `terminals[0..2]`) only to stitch them once in [`Endpoint::lists`](../../../src/open/endpoint.rs) and never touch them again — and all of `Endpoint`'s transient state sits in separate heap allocations even though its live range ends *strictly before* `edges[0..m)` is first written.
2. **Untested `I_0` Pigeonhole capacity bound:** [`Runs`](../../../src/runs.rs) *already* maintains `counts: [u32; 256]` and `per: [u32; 256]` during Huffman/MTF/RLE2 decoding (`src/runs.rs:90-92`), raising the question whether Dirichlet's Pigeonhole Principle on `(counts, per)` can pre-partition a single flat queue arena in $O(1)$ without per-run reallocation checks — or whether bzip2's `CAP = 127` record splitting inflates `per[a]`.

Cycle 5 resolves both questions mathematically and combines the surviving links from Cycles 1–5 into two complete candidate chains — **`W1: Chaitin-Open`** (forward walk, band-span SIMD `wire`) and **`W2: CPC-Micro`** (conjugate `u16` thin-stream, `0 B` queues, full `Endpoint`-in-`edges` overlay) — tested under both single-thread and 4-worker concurrent barrier contention.

---

## 2. Cycle 5 Mathematical Links and Tagged Theorems

### Theorem 5.1 — $I_0$ Pigeonhole Queue Envelope & `CAP = 127` Inflation Law (`proved`; `verified` on `26,495` small/random cases + `6/6` blocks in `verify_cycle5.py`)

Let $C[a] = \text{counts}[a]$ be the total occurrences of byte $a \in \{0,\dots,255\}$ in $L$, $r_a$ be the number of *maximal* runs of byte $a$, and $\text{per}[a] \ge r_a$ be the number of `CAP = 127`-capped records of byte $a$ stored in [`Runs`](../../../src/runs.rs) (`src/runs.rs:91-92`). At threshold $\Lambda = 16$, the retained thin-row count $\mathrm{thin}[a]$ satisfies:

$$\mathrm{thin}[a] \;\le\; \min\!\big(C[a],\; 16\,r_a + \mathbf{1}_{L[p]=a}\big) \;\le\; \min\!\big(C[a],\; 16\,\mathrm{per}[a] + 1\big).$$

*Proof.* Every thin maximal run of byte $a$ contributes at most $\Lambda = 16$ retained rows, and every thick maximal run ($w > 16$) contributes $0$ retained rows unless it contains the unique primary index $p$, in which case [`split`](../../../src/open.rs) (`src/open.rs:115-123`) extracts the single retained row $\{p\}$ and classifies both flanking fragments as thick regardless of width. Summing over the $r_a \le \mathrm{per}[a]$ maximal runs of $a$ gives $\mathrm{thin}[a] \le 16 r_a + \mathbf{1}_{L[p]=a} \le 16\,\mathrm{per}[a] + 1$, while trivially $\mathrm{thin}[a] \le C[a]$. $\square$

- **`CAP = 127` Inflation Corollary (`proved` + `measured` in `verify_cycle5.py`):** Because [`Runs`](../../../src/runs.rs) splits runs of width $W > 127$ into $\lceil W / 127 \rceil$ records before `Open` runs, a thick run of width $W$ contributes $0$ to $\mathrm{thin}[a]$ yet inflates $16\,\mathrm{per}[a]$ by $16\lceil W/127\rceil \approx 0.126 W$. On the six authenticated blocks, $\sum_a \min(C[a], 16\,\mathrm{per}[a]+1)$ equals `397.9k–686.9k` entries (`1,570.2–2,698.7 KiB`), which is **`3.37–4.43×` larger** than true $m = \sum_a \mathrm{thin}[a] = 89.9\text{k–}204.3\text{k}$ (`351.1–798.0 KiB`). Pre-allocating a flat `u32` queue arena from `per[a]` therefore wastes `1.2–1.9 MiB` per worker and scatters writes across sparse `640 KiB` symbol strides. Replacing it with either compact `1.25×`-grown per-symbol queues (`W1: Chaitin-Open`, touching only the $4m$ live bytes) or Kakutani's `0 B`-queue `u16` thin-run stream (`W2: CPC-Micro`) avoids the inflation completely.

---

### Theorem 5.2 — Direct-Linked 16-Byte `u16` Veech–Forman–Sköldberg Elimination & Leaf-Band Bound (`proved`; `verified` on `26,495` small/random cases + `6/6` blocks in `verify_cycle5.py`)

For any block of length $n \le 900,000$ at threshold $\Lambda = 16$:

1. **Strict `u16` Node-Count Bound (`proved`):** Every thick maximal run has width $\ge 17$, and only the single maximal run containing $p$ can split into two thick fragments (`src/open.rs:115-123`). Hence the thick-strip count satisfies
   $$H \;\le\; \left\lfloor \frac{n - 1}{17} \right\rfloor + 2 \;=\; 52,943 \quad (n \le 900,000),$$
   and each node in [`Endpoint`](../../../src/open/endpoint.rs) is either the sentinel `0`, one of the $H$ thick-strip nodes, one of at most $H + 1$ domain-gap (`side = 0`) nodes, or one of at most $H + 1$ image-gap (`side = 1`) nodes, so total allocated nodes satisfy $N_{\mathrm{nodes}} \le 3H + 3$. Whenever $H \le 21,843$ (guaranteed whenever thick strips have average width $\ge 41.2$, or checked in $O(1)$ right after Pass 1 via `3 * nstrips + 8 < 65535`), every `prev[0..2]` and `next[0..2]` link fits in `u16`, packing `Node16 = { prev: [u16; 2], next: [u16; 2], width: u32, word: u32 }` into **16 bytes** (`4` nodes per 64-byte cache line vs `2.67` for 24-byte `Node32`). Across all 6 authenticated blocks, $H \in [1,565,\; 5,047]$ and $N_{\mathrm{nodes}} \in [4,131,\; 12,836] \ll 65,535$ (`cycle5_results.json`).
2. **Zero-Array Direct Chain Stitching (`proved`):** Because domain strips are visited in strictly increasing `from` order and image strips are visited in strictly increasing `to` order via `image_order`, appending gap and strip nodes directly onto four `(head, tail)` `u16` register pairs `(t0, o0, t1, o1)` constructs the exact circular doubly-linked lists of [`Endpoint::lists`](../../../src/open/endpoint.rs) in $O(H)$ register splices with **`0 B` allocated for `order[0]`, `order[1]`, `terminals[0]`, `terminals[1]`**.
3. **Exact Leaf-Band Bound (`proved`):** Let $L_{\mathrm{surv}}$ be the number of surviving leaf intervals counted during the `bottom` prefix-sum walk (`id = nd[0].next[1]`). In the subsequent band-emission loop over $a \in \{0,\dots,255\}$, every emitted `Band` either exhausts the current leaf (`id = nd[id].next[0]`, occurring at most $L_{\mathrm{surv}}$ times total because `next[0]` and `next[1]` link the exact same surviving leaves) or exhausts thin bucket $a$ (`k == end`, occurring at most $256$ times). Therefore
   $$N_{\mathrm{words}} \;\le\; N_{\mathrm{bands}} \;\le\; L_{\mathrm{surv}} + 256$$
   holds unconditionally for every input column $(L, p)$, tightly sizing `bands`, `offsets`, and fused `inline16` in one allocation with zero inner-loop reallocation checks.

---

### Theorem 5.3 — Full Chaitin Interval-Graph Lifetime Overlay of `Endpoint` Inside `edges` (`proved` via Chaitin *POPL* 1982; `verified` + `measured` on `6/6` blocks)

Construct the buffer interference graph $G_{\mathrm{live}}$ across the four phases of `Open`:
- **Phase 1 (Run classification):** writes `strips[0..H)` (`16H` bytes) and either `q_buf` (`W1`) or `thin_stream` (`W2`, `2 R_{\mathrm{rec}}$ bytes).
- **Phase 2 (`Endpoint` elimination & band fusion):** reads/consumes `strips[0..H)` and allocates transient `Node16[0..N_{\mathrm{nodes}})`, `Rule[0..N_{\mathrm{rules}})`, `bottom[0..N_{\mathrm{nodes}})`, `img_ord[0..H)`, and `stack`, producing live outputs `(bands, spelling, offsets, inline16, lens)` and **dying completely** upon exit from Phase 2.
- **Phase 3 (Wiring):** reads `(bands, q_buf / thin_stream)` and writes `edges[0..m)` (`4m` bytes) for the first time; `bands` and `q_buf / thin_stream` die upon exit from Phase 3.
- **Phase 4 (Critical walk):** reads `(edges, inline16, lens, spelling, offsets)` and writes `out[0..n)`.

Because interval graphs are chordal and perfect (Gavril 1972; Chaitin *ACM SIGPLAN POPL* 1982), $\chi(G_{\mathrm{live}}) = \omega(G_{\mathrm{live}})$, and **100% of Phase 1–2 transient structures (`strips + Node16 + Rule + bottom + img_ord + stack`) have strictly disjoint live range from Phase 3–4 `edges[0..m)`** and can share a single contiguous arena of size $\max(4m,\; B_{\mathrm{endpt}}(H))$.

On all **6/6 authenticated blocks** (`verify_cycle5.py`, `cycle5_results.json`), $B_{\mathrm{endpt}}(H) < 4m$ strictly:

| Block | $n$ | $r$ | $m$ | $H$ | $N_{\mathrm{nodes}}$ | `Endpoint` Transient $B_{\mathrm{endpt}}$ | `edges[0..m)` ($4m$) | Fits $100\%$ Inside `edges`? | `W2` Active Peak Scratch |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| `block-0.L` | `899,931` | `51,184` | `89,874` | `3,448` | `7,787` | `334.0 KiB` | `351.1 KiB` | **YES (`95.1%`)** | **`559.0 KiB`** |
| `block-2.L` | `900,000` | `54,532` | `112,070` | `3,827` | `9,257` | `386.1 KiB` | `437.8 KiB` | **YES (`88.2%`)** | **`671.0 KiB`** |
| `block-85.L` | `899,935` | `79,375` | `156,404` | `1,565` | `4,131` | `174.7 KiB` | `611.0 KiB` | **YES (`28.6%`)** | **`847.0 KiB`** |
| `block-143.L` | `899,869` | `108,335` | `196,841` | `5,047` | `12,836` | `546.4 KiB` | `769.0 KiB` | **YES (`71.1%`)** | **`1,171.6 KiB`** |
| `block-210.L` | `899,959` | `128,732` | `204,281` | `2,809` | `7,182` | `300.7 KiB` | `798.0 KiB` | **YES (`37.7%`)** | **`1,172.5 KiB`** |
| `block-207.L` | `899,998` | `87,111` | `135,921` | `3,474` | `8,975` | `369.7 KiB` | `531.0 KiB` | **YES (`69.6%`)** | **`844.6 KiB`** |

**Result:** `Endpoint` adds **literally `0 B`** to peak per-worker memory across all 6 blocks.

---

## 3. Verification and Gate G5 Single-Thread & 4-Worker Confrontation

### Gate G1 — Exactness (`verified`)
- **`verify_cycle5.py`:** **PASSES `26,495 / 26,495` small and multi-cycle `(L, p)` inputs** (`5,227` exhaustive binary $n \le 9$ and ternary $n \le 6$ columns across every mark $p$, plus `21,268` random & periodic multi-cycle stress columns) and **`6 / 6` authenticated 900 KB blocks** byte-for-byte.
- **`g5_cycle5_final.c`:** Clean under `-fsanitize=address,undefined` (`exit 0`) and unstrippable `CHECK(memcmp(out, ref, n) == 0)` + 16-byte tail-canary verification on all 3 arms across all 6 blocks.

### Gate G2 — Acquisition Manifests (`verified`)
- `python3 docs/team/acquisition/check_manifest.py cpc_arena.toml` $\to$ **`PASS`** (`W1: Chaitin-Open`).
- `python3 docs/team/acquisition/check_manifest.py cpc_micro.toml` $\to$ **`PASS`** (`W2: CPC-Micro`).

### Gate G5 — Paired Rotated 1-Thread & 4-Worker Barrier Confrontation (`measured`, `g5_cycle5_final.c`, `-O3 -march=native`, 25 rounds)

| Block | `C0: Open058` (1t) | `W1: Chaitin-Open` (1t) | `W2: CPC-Micro` (1t) | `C0: Open058` (4t/bar) | `W1: Chaitin-Open` (4t/bar) | `W2: CPC-Micro` (4t/bar) |
|---|---:|---:|---:|---:|---:|---:|
| `block-0.L` | `1,215.1 µs` | `1,196.7 µs` (**`0.985×`**) | **`1,193.4 µs` (`0.982×`)** | `2,475.3 µs` | `2,357.7 µs` (**`0.952×`**) | **`2,318.8 µs` (`0.937×`)** |
| `block-2.L` | `1,459.8 µs` | `1,412.8 µs` (**`0.968×`**) | **`1,404.4 µs` (`0.962×`)** | `3,083.5 µs` | **`2,921.5 µs` (`0.947×`)** | `2,975.9 µs` (**`0.965×`**) |
| `block-85.L` | `1,432.9 µs` | **`1,397.6 µs` (`0.975×`)** | `1,398.7 µs` (**`0.976×`**) | `3,129.2 µs` | `3,070.6 µs` (**`0.981×`**) | **`3,041.1 µs` (`0.972×`)** |
| `block-143.L` | `2,670.4 µs` | `2,616.4 µs` (**`0.980×`**) | **`2,529.2 µs` (`0.947×`)** | `5,728.0 µs` | `5,486.8 µs` (**`0.958×`**) | **`5,433.3 µs` (`0.949×`)** |
| `block-210.L` | `2,355.6 µs` | `2,277.9 µs` (**`0.967×`**) | **`2,245.7 µs` (`0.953×`)** | `5,148.7 µs` | `5,056.5 µs` (**`0.982×`**) | **`4,924.0 µs` (`0.956×`)** |
| `block-207.L` | `1,773.7 µs` | `1,732.7 µs` (**`0.977×`**) | **`1,698.5 µs` (`0.958×`)** | `3,741.7 µs` | `3,592.1 µs` (**`0.960×`**) | **`3,469.8 µs` (`0.927×`)** |

*Diagnostic note (`CONTEXT.md` §5a):* These C harness timings are diagnostic mechanism confrontations, not native production Rust speedups. Crucially, however, unlike the 8-lane `emit` candidate in the fidelity ledger (which won `13%` at 1 thread by bloating cache footprint and then *lost* `5%` at 8 threads on the i9-10900K), **`W2: CPC-Micro`'s advantage *widens* from `0.947–0.982×` at 1 thread to `0.927–0.972×` under 4-worker concurrent cache/bandwidth pressure** because it shrinks per-worker scratch by **`3.16×`**.

---

## 4. Mandatory Performance & Resource Ledger vs `mibz2 0.5.8` `Open` (Target: 6–10 Workers on i9-10900K)

From `bench/release-scaling/2026-11-21-intel-r-core-tm-i9-10900k-cpu-3-70ghz-0.5.8/env.json` and `docs/journal/2026-11-20-open-stage-attribution.md`, the maintainer's 10-core Intel Core i9-10900K has **`32 KiB` L1d/core, `256 KiB` L2/core, and `20 MiB` shared L3**, running **7 lazy workers** at 10–20 threads (up to 10 workers). Each worker also holds its `~879 KiB` (`900,000 B`) output buffer `out`.

| Metric | `Open 0.5.8` (`src/open.rs` baseline) | `W1: Chaitin-Open` (`cpc_arena.toml`) | `W2: CPC-Micro` (`cpc_micro.toml`, **Primary Candidate**) |
|---|---|---|---|
| **Active peak scratch / worker** (`verified`) | `~3,100–4,663 KiB` | `~1,050–2,220 KiB` | **`559.0–1,172.5 KiB`** (`0.25×` max) |
| **Persistent scratch / worker after all 6 blocks** (`measured`) | **`4,662.6 KiB`** (`1.000×`) | **`2,220.1 KiB`** (`0.476×`) | **`1,475.0 KiB`** (**`0.316×`** — **`-3.19 MiB`/worker**) |
| **Total 7-worker i9-10900K footprint (incl. `out`)** | **`37.9 MiB`** (`1.90×` L3 $\to$ spills `17.9 MiB`!) | **`21.2 MiB`** (`1.06×` L3) | **`16.1 MiB`** (**fits inside `20 MiB` i9 L3!**) |
| **Total 10-worker i9-10900K footprint (incl. `out`)** | **`54.1 MiB`** (`2.71×` L3 $\to$ spills `34.1 MiB`!) | **`30.3 MiB`** (`1.52×` L3) | **`23.0 MiB`** (`1.15×` L3) |
| **Modelled DRAM bytes / output byte (1t, L3-resident)** | `~1.12–1.28 B/B` (compulsory `records` read + `out` write/RFO) | `~1.12–1.28 B/B` | **`~1.12–1.28 B/B`** |
| **Modelled DRAM bytes / output byte (7–10t on `20 MiB` i9 L3)** | **`~3.6–6.8 B/B`** (`18–34 MiB` L3 capacity spill across `q_buf`, `Node32`, `wire` scatter & `edges[j]` walk) | **`~1.6–2.7 B/B`** (eliminates `1.3 MiB`/worker `Endpoint` spill) | **`~1.2–1.9 B/B`** (7-worker working set fits inside `20 MiB` L3; eliminates `q_buf` + `Endpoint` + scatter-RFO DRAM traffic) |
| **Acquisition cost from $I_0 = (\text{records}, C, p, n)$** | $O(R_{\mathrm{rec}} + m + H)$ (`256` `Vec`s + `Endpoint` lists + band scatter `wire`) | **$O(R_{\mathrm{rec}} + m + H)$** (`0` extra passes; sentinel record scan + direct `Node16` splices + fused `inline16`) | **$O(R_{\mathrm{rec}} + H + r_{\mathrm{thin}})$** (`0 B` queues; `79–147 KiB` L2-resident `u16` thin stream + `99.47–99.77%` sequential `Vec16` `wire`) |
| **Gate G5 diagnostic C ratio vs `Open058`** | `1.000×` (1t) / `1.000×` (4t) | `0.967–0.985×` (1t) / `0.947–0.982×` (4t) | **`0.947–0.982×` (1t) / `0.927–0.972×` (4t)** |

---

## 5. Master 5-Cycle Wormhole Synthesis: Every Chain, Every Break, Every Theorem

| Cycle | Mathematical Chain (Borrowed Infrastructure) | What Was Proved / Verified | Where It Broke & What Link That Forced Next |
|---|---|---|---|
| **Cycle 1** ([journal](../2026-11-21-wormhole-cycle1-fvs-macro-duality.md)) | Levy–Low (1988) Directed FVS + Tarjan (1981)/Lawson (2004) metric-DAG pushforward on $G_{\mathrm{ov}}$; Kärkkäinen–Kempa–Puglisi (2014) $\Phi_p$ + Storer–Szymanski (1982)/Kempa–Prezza (2018) $\sigma$-seed BMS + Kahn `memcpy` DAG on $G_\Phi$ | **Thm 1.1 (`proved`):** Kahn sub-interval pushforward produces `1.59–1.90r` metric pieces (refuting Kitchen Sink's $2^{35}\text{–}2^{63}$ phantom blowup). **Thm 1.2 (`proved`):** $\le 96$ byte seeds make punctured $\Phi_p$ 100% acyclic (`0` extra seeds on `6/6` blocks) + zero-split copy-DAG replay without run labels (`5,227` small + `6` blocks). | **Break 1 — Thm 1.3 (Hub–Spoke Cycle-Packing Conservation Law):** Indivisible run/phrase vertices force `80.7k–102.2k` rows + $O(r\log r)$ graph tax (`2.8–6.2 MiB` WS). *Forced Cycle 2:* sub-divide & eliminate intervals via **filtered discrete Morse / noncommutative Schur collapse**. |
| **Cycle 2** ([journal](../2026-11-21-wormhole-cycle2-morse-quasideterminant-cliff.md)) | Forman (1998) / Sköldberg (2006) Fox–Morse collapse $\mathcal{M}_\Lambda$ ($\Lambda \in \{1..16\}$) via Gel'fand–Retakh (1991) quasideterminants; Milnor (1966)–Fox (1953)–Perseus (Mischaikow–Nanda 2013) $\mathbb{Z}[t,t^{-1}]$ reductions | **Thm 2.1 (`proved`):** $\Lambda: 16\to 1$ shrinks `edges` to `134–379 KiB` (`4/6` blocks inside `256 KiB` L2!) and cuts `>16B` walk steps by `29–52%` (`26,135` small + `30` block decodes). **Thm 2.2 (`proved`):** `0 B`-dict $\mathbb{Z}[t,t^{-1}]$ coordinates, refuted as standalone decoder at $\Lambda=1$ by minimal $n=5$ twin `20011` vs `21100`. **Thm 2.3 (`measured`):** `0.04–0.07%` 64B index locality on `edges`. | **Break 2 — Eager SLP Flattening Cliff:** At $\Lambda \to 1$, `spelling+inline16` explodes to `633–821 KiB` (`61–76%` single-use `width=1` words). *Forced Cycle 3:* eliminate `spelling` via **Rytter–Gasieniec SLP output memoization** + **Zorich/AHT periodic quotienting**. |
| **Cycle 3** ([journal](../2026-11-21-wormhole-cycle3-slp-lz77-output-memoization.md)) | Zorich (1996) / Agol–Hass–Thurston (2006) periodic band quotienting + Rytter (2003) / Gasieniec et al. (2005) LZ77-in-`out` SLP memoization (`OM-Fox`) | **Thm 3.1 (`proved`):** Pre-BWT `RLE1` caps monochrome runs so Zorich/AHT saves `0–2` steps on Wikidata. **Thm 3.2 (`proved`):** `OM-Fox` eliminates `100%` of `self.spelling` (`0 B`) and cuts `inline16` to `17.1–47.0 KiB` (`26,495` small + `30` block decodes). | **Break 3 — Gate G5 Falsification:** `OM-Fox<16>` ran at `1.041–1.077×` `Open058` (L2-hot `50 KiB` flat `spelling` beats `900 KiB` `out` self-copies), and $\Lambda < 16$ ran at `1.15–2.07×` (`O(H_\Lambda)` `Endpoint` dominates). Isolated real bottleneck at $\Lambda=16$: **`4.66 MiB`/worker scratch (`> 20 MiB` i9 L3 at 7–10t)** + **`MLP=1` walk**. *Forced Cycle 4.* |
| **Cycle 4** ([journal](../2026-11-21-wormhole-cycle4-dihedral-bidi-and-conjugate-stream.md)) | Birkhoff (1915) / Pohl (1971) dihedral time-reversal meet-in-the-middle (`Bidi-Open`); Kakutani (1943) skyscraper return-map conjugacy + partial Chaitin overlay (`CSW-Open`) | **Thm 4.1 (`proved`):** `at_f == at_b` escapes Carbon Depth Thm 6's unknown-offset staging barrier at `0 B` staging (`emit` `-7..9%`, killed by `+101..318 µs` dual `eb[sj]` scatter). **Thm 4.2 (`proved`):** `CSW-Open` hits `99.47–99.77%` sequential `Vec16` `iota` wiring, **overturning `#659`** (`0.973–0.991×` `Open058` on `6/6` blocks, `0.573×` scratch `2.67 MiB`). | **Break 4 — Unaliased `Endpoint` (`1.3 MiB`, `399–765 µs`):** 24B `Node32` + 4 `ord/term` `Vec`s sat outside `edges`. *Forced Cycle 5:* $I_0$ Pigeonhole analysis + Direct-Linked 16B `Node16` + 100% Chaitin `Endpoint`-in-`edges` overlay. |
| **Cycle 5** *(this journal)* | Dirichlet Pigeonhole on `Runs` $I_0$ + Veech (1982)/Forman (1998)/Sköldberg (2006) Direct-Linked `u16` `Node16` + Kakutani (1943) conjugacy + Chaitin (*POPL* 1982) full interval-graph coloring (`W1: Chaitin-Open`, `W2: CPC-Micro`) | **Thm 5.1 (`proved`):** $I_0$ Pigeonhole bound + `CAP=127` `3.4–4.4×` inflation law. **Thm 5.2 (`proved`):** 16B `u16` `Node16` (`0 B` `ord/term`) + exact leaf-band bound $N_{\mathrm{bands}} \le L_{\mathrm{surv}} + 256$. **Thm 5.3 (`proved`):** `100%` of `Endpoint` (`175–546 KiB`) fits inside dead `edges` (`351–798 KiB`) on `6/6` blocks (`26,495` small + `6` blocks). | **Destination Reached (`W2: CPC-Micro` & `W1: Chaitin-Open`):** `W2` cuts persistent scratch to **`1,475.0 KiB` (`0.316×`)**, fits 7 i9 workers in `16.1 MiB <= 20 MiB` L3, and wins `6/6` blocks at 1t (`0.947–0.982×`) and 4t (`0.927–0.972×`). |

---

## 6. Clean Rust Handoff Notes for the Coordinator

1. **Zero `unsafe` required (`src/open.rs` safety invariant preserved):**
   - **Direct-Linked `Node16` (`src/open/endpoint.rs`):** Replace 24-byte `Node` (`prev/next: [u32; 2]`) with 16-byte `Node16` (`prev/next: [u16; 2], width: u32, word: u32`), guarded by a single entry check `3 * strips.len() + 8 < usize::from(u16::MAX)` (falling back to `LazyCells` or `Node32` only on synthetic adversarial blocks with `> 21,842` thick strips; real blocks have `1,565–5,047`). Delete `Endpoint::order: [Vec<u32>; 2]` and `Endpoint::terminals: [Vec<u32>; 2]` completely, splicing gap and strip nodes directly onto four `(head, tail)` `u16` pairs and counting `n_leaves` during the `bottom` walk to reserve `bands`, `offsets`, and fused `inline16` in one pass (`nbands <= n_leaves + 256`).
   - **Safe Rust Chaitin Lifetime Overlay:** In safe Rust without `unsafe` byte-casting, `Endpoint` (`Node16` = four `u32` words, `Rule` = four `u32` words, `Strip` = four `u32` words, `bottom` = `u32`, `img_ord` = `u32`) is **100% `u32`-homogeneous**! Every single transient array in Phases 1–2 can therefore be sub-sliced from `self.edges: Vec<u32>` via safe `split_at_mut` (`Node16`, `Rule`, and `Strip` packed/unpacked cleanly into `[u32; 4]` chunks or `[u32]` sub-slices with zero `unsafe`, zero heap `Vec`s in `Endpoint`, and 100% physical cache-line reuse when Phase 3 overwrites `self.edges[0..m)`!).
   - **`W2: CPC-Micro` vs `W1: Chaitin-Open` A/B on the i9-10900K:** Both candidates share the exact same `Node16` + Chaitin `self.edges` overlay and differ only in Pass 1 + Phase 3 + Walk direction (`W1`: compact `1.25×` `q_buf` + forward `U16` walk, `2.22 MiB` scratch; `W2`: `u16` `thin_stream` + `99.6%` `Vec16` sequential `wire` + downward right-aligned `U16` walk, `1.48 MiB` scratch, `16.1 MiB` across 7 workers `< 20 MiB` L3). Porting `Node16` + Chaitin overlay first gives an immediate safe modular step, and swapping Pass 1/Phase 3/Walk tests `W2`'s L3-fit win directly on the 10-core i9 scaling suite.

---

## 7. Reproducible Artefacts in This Directory

- `verify_cycle5.py` — Exact Gate G1 verifier (`26,495` small/random multi-cycle cases + `6` blocks) and exact byte-level Theorem 5.1/5.2/5.3 profiler.
- `cycle5_results.json` — Exact per-block verification and scratch ledger output of `verify_cycle5.py`.
- `cpc_arena.toml`, `cpc_micro.toml` — Validated Gate G2 acquisition manifests (`PASS` under `docs/team/acquisition/check_manifest.py`).
- `g5_cycle5_final.c` — Self-contained C confrontation (`C0: Open058`, `W1: Chaitin-Open`, `W2: CPC-Micro`), ASan/UBSan clean, with both 1-thread paired-rotated and 4-worker `pthread_barrier_t` concurrent throughput harnesses.
- `g5_cycle5_output.txt` — Recorded Gate G2 + 25-round 1-thread and 4-worker output.

---

## 8. Literature Cited Across the 5-Cycle Wormhole Line

1. **G. D. Birkhoff** (1915), *The restricted problem of three bodies*, Rend. Circ. Mat. Palermo **39**, 265–334 (time-reversal involution generating a dihedral symmetry of reversible orbits).
2. **S. Kakutani** (1943), *Induced measure preserving transformations*, Proc. Imp. Acad. Tokyo **19**, 635–641 (skyscraper / first-return map conjugacy between upward and downward return maps).
3. **R. H. Fox** (1953), *Free differential calculus. I. Derivation in the free group ring*, Ann. of Math. **57**(3), 547–560.
4. **J. Milnor** (1966), *Whitehead torsion*, Bull. Amer. Math. Soc. **72**(3), 358–426 (augmentation homomorphism $\mathbb{Z}[F_\Sigma] \to \mathbb{Z}[t,t^{-1}]$).
5. **I. Pohl** (1971), *Bi-directional search*, Machine Intelligence **6**, 127–140.
6. **F. Gavril** (1972), *Algorithms for minimum coloring, maximum clique, minimum covering by cliques, and maximum independent set of a chordal graph*, SIAM J. Comput. **1**(2), 180–187.
7. **R. E. Tarjan** (1981), *Fast algorithms for solving path problems*, J. ACM **28**(3), 594–614.
8. **G. J. Chaitin** (1982), *Register allocation & spilling via graph coloring*, Proc. ACM SIGPLAN POPL / SIGPLAN Notices **17**(6), 98–105.
9. **J. A. Storer & T. G. Szymanski** (1982), *Data compression via textual substitution*, J. ACM **29**(4), 928–951.
10. **W. A. Veech** (1982), *Gauss measures for transformations on the space of interval exchange maps*, Ann. of Math. **115**(2), 201–242.
11. **H. Levy & D. W. Low** (1988), *A contraction algorithm for finding small cycle cutsets*, J. Algorithms **9**(4), 470–493.
12. **I. M. Gel'fand & V. S. Retakh** (1991), *Determinants of matrices over noncommutative rings*, Funct. Anal. Appl. **25**(2), 91–102.
13. **A. Zorich** (1996), *Finite Gauss measure on the space of interval exchange transformations. Lyapunov exponents*, Ann. Inst. Fourier **46**(2), 325–370.
14. **R. Forman** (1998), *Morse theory for cell complexes*, Adv. Math. **134**(1), 90–145.
15. **W. Rytter** (2003), *Application of Lempel–Ziv factorization to the approximation of grammar-based compression*, Theor. Comput. Sci. **302**(1–3), 211–222.
16. **J. D. Lawson** (2004), *Idempotent analysis and continuous semilattices*, Theor. Comput. Sci. **316**, 75–87.
17. **L. Gasieniec, R. M. Kolpakov, I. Potapov & P. Sant** (2005), *Real-time traversal in grammar-based compressed files*, Proc. IEEE DCC 2005, 458.
18. **I. Agol, J. Hass & W. Thurston** (2006), *The computational complexity of knot genus and spanning area*, Trans. Amer. Math. Soc. **358**(9), 3821–3850.
19. **E. Sköldberg** (2006), *Morse theory from an algebraic viewpoint*, Trans. Amer. Math. Soc. **358**(1), 115–129.
20. **K. Mischaikow & V. Nanda** (2013), *Morse theory for filtrations and efficient computation of persistent homology*, Discrete Comput. Geom. **50**(2), 330–353.
21. **J. Kärkkäinen, D. Kempa & S. J. Puglisi** (2014), *Hybrid compression of bitvectors for the FM-index*, Proc. IEEE DCC 2014, 302–311.
22. **D. Kempa & N. Prezza** (2018), *At the roots of dictionary compression: string attractors*, Proc. ACM STOC 2018, 827–840.
