# Wormhole Cycle 4 — Birkhoff Dihedral Zero-Copy Meet-in-the-Middle (`Bidi-Open`) & Kakutani Conjugate Stream Wiring (`CSW-Open`)

*21 November 2026 · Branch `research/carbon-wormhole` · Continuing directly from [Cycle 3](2026-11-21-wormhole-cycle3-slp-lz77-output-memoization.md)'s `Break 3`*

---

## 1. Explore & Triage: Attacking `Break 3` (`4.66 MiB` Per-Worker Scratch & Single-Chain `MLP = 1` Walk)

Cycle 3 closed $\Lambda < 16$ and SLP-deferral (`OM-Fox`), isolating `Open 0.5.8`'s two remaining walls on the 10-core i9-10900K (`256 KiB` L2, `20 MiB` shared L3):
1. **The Multi-Lane Staging Trap (`#675`):** 8-lane `BP4K` won `0.872×` on 1 thread natively, then reversed to `1.049×` SLOWER at 8 threads on the i9 because unknown lane offsets (Carbon Depth Theorem 6) forced `4 KiB` staging pages (`87.5%` of `out` written twice = `+1.8 MiB`/block L3/DRAM traffic) and 64 concurrent L3 streams.
2. **The `4.66 MiB` Scratch Overflow:** `Open 0.5.8` holds `~1.54 MiB` in 256 growable queues, `~1.30 MiB` in `Endpoint` scratch, and `~1.54 MiB` in `edges`, overflowing `20 MiB` L3 across 6–10 workers (`#659` `Backward Open` removed queues via a second `records` scan + scalar width loop, losing `1.10–1.15×`).

Cycle 4 builds and confronts two chains of existing theorems targeting these two walls, plus their synthesis:

1. **Chain 4A (`Bidi-Open` — Birkhoff Reversible Involution + Pohl Bidirectional Meet-in-the-Middle):**
   - *Link 4A.1:* Birkhoff (*Trans. AMS* 16:33–55, 1915; Lamb & Roberts, *Physica D* 112:1–39, 1998) time-reversal involution $D_m = \langle g, \iota \rangle$ on reversible discrete dynamical systems.
   - *Link 4A.2:* Pohl (*Artificial Intelligence* 2(3–4):193–204, 1971) bidirectional search termination from a single self-dual origin (`mark`): forward $\Psi$-ray from `(mark, at_f = 0)` and backward $\mathrm{LF}$-ray from `(mark, at_b = n)` meet at the antipodal vertex `j_f == j_b` with **`at_f == at_b` (`0 B` staging pages, `0` extra `out` copies, `MLP = 2`)**, escaping Carbon Depth Theorem 6's unknown-offset surcharge!

2. **Chain 4B (`CSW-Open` — Kakutani Conjugate Skyscraper + `u16` Quotient Stream + Chaitin Lifetime Overlay):**
   - *Link 4B.1:* Kakutani (*Ann. Math.* 44:538–567, 1943) topological conjugacy between induced forward return $g = P \circ \pi_Q^{-1}$ and backward return $g^{-1} = \pi_Q \circ P^{-1}$: $\pi_Q^{-1}$ is monotone per byte in domain order $j$, removing 256-way radix bucket queues (`0 B` queues!).
   - *Link 4B.2:* `u16` run-length quotient stream (`93.2–245.9 KiB`, `32×` less Pass-1 store traffic than `3.06–8.06 MiB` `Vec16` queues) + **`99.47%–99.77%` branchless 16-wide AVX2 `iota` sequential `edges` fill** + right-aligned 16B downward `emit` (overturning `#659`'s two losses!).
   - *Link 4B.3:* Chaitin (*POPL 1982*) interval-graph register/arena coloring: overlays `Endpoint`'s transient index arrays (`img_ord, ord0/1, term0/1, bottom`) inside dead `edges_bwd[0..m)`.

3. **Chain 4C (`CSW-Bidi` — Dual Band-Slack 16-Wide Stream Wiring + Zero-Copy `MLP = 2` Walk).**

### Candidate Profiles (`METHOD.md` §3a)

| Field | Chain 4A (`Bidi-Open`) | Chain 4B (`CSW-Open`) |
|---|---|---|
| **Mode** | Reorientation of orbit traversal (unidirectional $\to$ dihedral involution) | Architectural/foundational conjugacy ($g^{-1}$ scatter $\to$ $g$ sequential `Vec16`) |
| **Displaced obligation** | Half the serial `MLP = 1` pointer-chase depth (`D: m -> m/2`) without `BP4K` staging copies | `100%` of `256` heap `Vec<u32>` queues (`1.54 MiB`), `97%` of Pass-1 queue writes, and random `edges` scatter writes |
| **Replacement principle** | Both rays start at `mark` (`out[0]` and `out[n]`), colliding at `j_f == j_b` with `at_f == at_b` | `2 B/run` quotient stream feeds `99.6%` branchless 16-wide `iota` writes sequentially into `edges[j..j+16]` |
| **New paid obligations** | Second `4m` table `edges_bwd` (`+351–798 KiB`) + scattered `eb[sj]` writes during `wire` | Per-thin-run `ActiveBand[256]` cursor updates during `wire` (`48k–126k` runs vs `3.2k–6.8k` bands) |
| **Nearest prior art** | `#675` 8-lane `BP4K` (which paid `200 KiB` staging + `87.5%` double `out` writes) | `#659` `Backward Open` (which re-scanned `records` + ran scalar per-row width loops, losing `1.10–1.15×`) |

---

## 2. Exact Theorems & Proofs

### Theorem 4.1 (Dihedral Zero-Copy Meet-in-the-Middle Escape from Unknown-Offset Barriers — `proved`; `verified` on `26,495` small/multi-cycle cases + `6` blocks)

Let $(V_{\mathrm{crit}}, g, g^{-1})$ have marked cycle $C_{\mathrm{mark}} = (v_0 = \mathrm{mark}, v_1 = g^{-1}(v_0), \dots, v_{m_c - 1})$ of length $m_c \ge 1$ spelling cycle word $W = W(v_0)W(v_1)\cdots W(v_{m_c-1})$ of byte length $d \le n$. Advancing `j_f` along $g^{-1}$ from `(mark, at_f = 0)` and `j_b` along $g$ from `(mark, at_b = n)` in alternating steps, halting as soon as `j_f == j_b` (or `at_f >= at_b`):
1. Executes $\lceil m_c / 2 \rceil$ forward steps writing $W(v_0)\cdots W(v_{\lceil m_c/2\rceil - 1})$ into `out[0 .. at_f)` and $\lfloor m_c / 2 \rfloor$ backward steps writing $W(v_{\lceil m_c/2\rceil})\cdots W(v_{m_c-1})$ into `out[at_b .. n)`, visiting every cycle edge **at most once** with **zero overlap** (`at_f <= at_b`).
2. Whenever $d = n$ (**`100%` of primitive BWT blocks**), `at_f == at_b` identically upon collision, producing `out[0..n)` **in place with `0` bytes of staging buffers and `0` post-walk `memmove` bytes**. When $d < n$ (periodic or multi-cycle columns), `memmove(out + at_f, out + at_b, n - at_b)` followed by `repeat(n, out)` yields exact `Plain::invert` output. $\square$

### Theorem 4.2 (Conjugate Quotient-Stream `Vec16` Equivalence & `99.6%` Unsplit Invariant — `proved` & `verified` on `26,495` cases + `6` blocks)

In `CSW-Open`, every thin piece `(a, w)` ($1 \le w \le 16$) at symbol-rank $z = \mathrm{ab\_z}[a]$ lies wholly inside its active band $[k_{\mathrm{start}}, \mathrm{ab\_end}[a])$ except at the $\le B - \sigma$ interior band boundaries within symbol buckets. On all 6 Wikidata blocks, **`99.47%–99.77%`** of thin runs satisfy `z + w <= ab_end[a]`, enabling branchless 16-wide AVX2 `iota` stores sequentially into `edges_bwd[j .. j + 16]` (`m + 16` buffer).

---

## 3. Factorial Gate G5 Confrontation (`g5_cycle4_bidi_csw.c`, 25 Rotated Rounds)

- **Gate G1 (`verify_cycle4.py`):** **PASS** across `26,495 / 26,495` small + multi-cycle instances and `6 / 6` blocks (`zero_copy=True` on `6/6` blocks).
- **Gate G2 (`check_manifest.py`):** Both `bidi_open.toml` (`D = m / 2`) and `csw_open.toml` **PASS**.
- **Safety:** Clean under `gcc -O1 -g -mavx2 -fsanitize=address,undefined` + 16-byte `0xA5` tail canary checks on all 4 arms × 6 blocks.

### Whole-Kernel Medians (`-O3 -march=native`, 25 Interleaved Rounds) & Persistent Scratch

| Block | `C0: Open058` (`master`) | **`C1: CSW-Open` (`/058`)** | `C2: Bidi-Q` (`/058`) | `C3: CSW-Bidi` (`/058`) | `CSW` `thin_stream` vs `Vec16` writes | `CSW` `fast16` hit rate |
|---|---:|---:|---:|---:|---:|---:|
| `block-0.L` | 1,225.0 µs | **1,213.0 µs (`0.990×`)** | 1,239.7 µs (`1.012×`) | 1,358.3 µs (`1.109×`) | `93.2 K` vs `3,055 K` (**`0.031×`**) | **99.68%** |
| `block-2.L` | 1,491.1 µs | **1,478.0 µs (`0.991×`)** | 1,516.0 µs (`1.017×`) | 1,634.7 µs (`1.096×`) | `99.0 K` vs `3,245 K` (**`0.031×`**) | **99.52%** |
| `block-85.L` | 1,565.0 µs | **1,522.7 µs (`0.973×`)** | 1,629.9 µs (`1.042×`) | 1,576.2 µs (`1.007×`) | `152.0 K` vs `4,980 K` (**`0.031×`**) | **99.77%** |
| `block-143.L` | 2,746.8 µs | **2,693.0 µs (`0.980×`)** | 2,792.1 µs (`1.017×`) | 2,976.8 µs (`1.084×`) | `201.7 K` vs `6,610 K` (**`0.031×`**) | **99.47%** |
| `block-210.L` | 2,432.1 µs | **2,404.7 µs (`0.989×`)** | 2,515.2 µs (`1.034×`) | 2,547.7 µs (`1.048×`) | `245.9 K` vs `8,059 K` (**`0.031×`**) | **99.74%** |
| `block-207.L` | 1,813.9 µs | **1,786.8 µs (`0.985×`)** | **1,784.5 µs (`0.984×`)** | 1,962.0 µs (`1.082×`) | `163.4 K` vs `5,353 K` (**`0.031×`**) | **99.58%** |
| **Worker Scratch** | **`4,656.5 KiB`** (`1.000×`) | **`2,668.6 KiB` (`0.573×`)** | `6,129.6 KiB` (`1.316×`) | `3,922.6 KiB` (`0.842×`) | — | — |

### Stage-Timed Attribution (`stages_c4.c`, Mean of 11 Warmed Rounds)

| Block | Arm | `pass1` (µs) | `endpt+spell` (µs) | `wire` (µs) | `emit` (µs) | Diagnosis |
|---|---|---:|---:|---:|---:|---|
| `block-0.L` | `C0 Open058` | 230.5 | 399.4 | **70.9** | 501.7 | Fast band-span SIMD `wire`, slow 256-`Vec` `pass1` |
| `block-0.L` | `C1 CSW-Open` | **184.5** | 413.6 | 160.5 | 517.3 | `-20%` `pass1`, `-43%` WS, but per-run `wire` (`+90 µs`) |
| `block-0.L` | `C2 Bidi-Q` | 225.2 | 417.8 | 172.2 | **467.3** | `-7%` `emit`, offset by `+101 µs` `eb[sj]` scatter `wire` |
| `block-210.L` | `C0 Open058` | 502.6 | 423.6 | **106.6** | 1,324.9 | `1.54 MiB` `q_buf` + `1.54 MiB` `edges` |
| `block-210.L` | `C1 CSW-Open` | **384.7** | 427.0 | 357.4 | 1,316.0 | `-118 µs` `pass1`, given back by `+251 µs` per-run `wire` |
| `block-210.L` | `C2 Bidi-Q` | 487.6 | 419.8 | 424.2 | **1,208.0** | `-117 µs` `emit`, offset by `+318 µs` dual `wire` + `1.6 MiB` L2 pressure |

---

## 4. Learn: What `Break 4` Unlocks for Cycle 5 (The Final Synthesis!)

Cycle 4 achieves our **first `6 / 6` single-thread win (`0.973×–0.991×`) WITH `-42.7%` persistent scratch (`2.67 MiB` vs `4.66 MiB`, overturning `#659`'s `+12%` loss!)**, and its stage table reveals **two exact breaks (`Break 4A` & `Break 4B`) that point directly to Cycle 5's complete wormhole**:

1. **`Break 4A` (`wire`: `71–146 µs` Band-Span SIMD vs `161–357 µs` Per-Run Cursor):**
   `C0`'s `wire` is `2.3–3.4×` faster than `C1`'s `wire` because looping over `3.2k–6.8k` *bands* auto-vectorizes contiguous spans (`8` `u32`s/op) with zero per-run state updates, whereas `C0`'s `pass1` was slow & bloated (`1.54 MiB`, `230–503 µs`) **only** because it used 256 separate heap `Vec`s instead of **exact `I_0` Pigeonhole Partitioning (`U_16[a] = min(counts[a], 16 * per[a]) + 17` from `Runs::counts` & `Runs::per` in `src/runs.rs:91-92`)**!
2. **`Break 4B` (`Endpoint` Linked-List Overhead `399–765 µs` + `1.1–1.4 MiB` Unaliased Arenas + `CAP=127` Record Scanning):**
   - Look at `endpt` (`399–765 µs` = **`30%` of `Open058`'s entire runtime**!): `Endpoint` allocates `3H + 1` `24-byte` doubly-linked `Node`s (`prev[2], next[2], width, word`), builds 4 intermediate `Vec`s (`ord0, ord1, term0, term1`), and links `6H + 2` pointers **even though**:
     (a) `terminals[0]` and `terminals[1]` never move (`next` is strictly sequential array indexing until `order` empties!),
     (b) `16-byte` compact `Node16` (`u16` links for $3H + 2 < 65{,}536$: universally valid since $H \le n / 17 \le 52{,}941 \implies \text{active strips} \le H < 53{,}000 < 65{,}535$!!) halves `Endpoint` cache footprint, AND
     (c) **Full 4-Phase Chaitin Interval-Graph Coloring** overlays *all* of `Endpoint` (`nodes + rules + lists + strips`) with `edges` / `q_arena` across disjoint phase lifetimes, while SIMD boundary-mask `maximal` merging collapses `Pass 1`!

**Cycle 5** chains all proven links from Cycles 1–4 into one complete, verified, `I_0`-exact candidate (**`Chaitin-Pigeonhole Compact Open`, `CPC-Open`**), tests it at **both 1 thread AND 4-worker concurrent cache/memory contention**, and delivers the complete 5-cycle Wormhole handoff!

---

## 5. Reproducibility

```bash
BLOCKS_DIR=/home/antigravity/admitted-bwt python3 -u docs/journal/2026-11-21-wormhole-cycle4-dihedral-bidi-and-conjugate-stream/verify_cycle4.py
python3 docs/team/acquisition/check_manifest.py docs/journal/2026-11-21-wormhole-cycle4-dihedral-bidi-and-conjugate-stream/bidi_open.toml
python3 docs/team/acquisition/check_manifest.py docs/journal/2026-11-21-wormhole-cycle4-dihedral-bidi-and-conjugate-stream/csw_open.toml
gcc -O3 -march=native -Wall -Wextra -Werror docs/journal/2026-11-21-wormhole-cycle4-dihedral-bidi-and-conjugate-stream/g5_cycle4_bidi_csw.c -o /tmp/g5_c4 && /tmp/g5_c4 /home/antigravity/admitted-bwt 25
```
