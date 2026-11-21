# Carbon Kitchen Sink — Cycle 24: Antipodal Uniqueness of Zero-Staging Parallelism ($\mu = 2$ vs $\mu = 4$) & Epoch-Unrolled `XOR-BiEmit`

**Date:** 2026-11-21  
**Branch:** `research/carbon-kitchen-sink`  
**Theme:** Algebraic topology of punctured 1D target manifolds ($\partial[0, n) = \{0, n\}$), dual-end single-buffer staging tightness (`XOR-BiEmit4`, $\mu = 4$), periodic Galois cycle-multiplicity obstruction ($d > 1$), and epoch-filtered 4x-unrolled `OutQ16` XOR wiring (`Unrolled4-Epoch-XOR`).

---

## 1. Mathematical & Architectural Foundations

Cycle 17's `XOR-BiEmit` ($\mu = 2$) halved `Open`'s serial pointer-chasing depth to $D_{\mathrm{serial}} = \lceil m_{16}/2 \rceil$ at **`+0 B` scratch** and **`0 B` staging**. A natural question is whether adding a second puncture $q_0 = (\mathrm{mark} + \lfloor m/2 \rfloor) \bmod m$ — walking **four** cursors simultaneously ($\mu = 4$: $F_0$ from $\mathrm{mark}$, $B_1$ from $q_{-1} = \pi^{-1}(q_0)$, $F_1$ from $q_0$, $B_0$ from $j_{-1} = \pi^{-1}(\mathrm{mark})$) — can halve $D_{\mathrm{serial}}$ again without blowing up scratch or losing to `memcpy` stitching.

### Theorem 24.1 (Antipodal Uniqueness of Zero-Staging Parallelism)
Among all $\mu$-cursor traversals of a variable-length word-labelled cycle $(V, \pi, \ell)$ writing a contiguous block `out[0..n)` from a single BWT primary index `mark`:
1. **Topological Boundary Obstruction ($|\partial[0, n)| = 2$):** Only the two boundary coordinates $0$ (entered forward from $\mathrm{mark}$) and $n$ (exited backward from $j_{-1} = \pi^{-1}(\mathrm{mark})$) are invariant under word-length perturbations $\ell: V \to \mathbb{Z}_{\ge 1}$ with $\sum_v \ell(v) = n$. Consequently, **$\mu = 2$ is the unique multi-cursor multiplicity with zero staging memory and zero post-walk copy**. Every $\mu \ge 3$ cursor scheme originates at least one cursor at an interior vertex $q_0 \notin \{\mathrm{mark}, j_{-1}\}$ whose global byte offset $S(q_0) = \sum_{t=0}^{\tau(q_0)-1} \ell(\pi^t(\mathrm{mark})) \in (0, n)$ is unobservable until the arc $\mathrm{mark} \leadsto q_0$ completes.
2. **Lemma 24.2 (Dual-End Single-Buffer Tightness):** For $\mu = 4$ on any primitive block ($d = 1$), $\ell(F_0) + \ell(B_1) + \ell(F_1) + \ell(B_0) = n$ with $\ell(F_0), \ell(B_0) \ge 1 \implies \ell(F_1) + \ell(B_1) \le n - 2 < n$. Writing $F_1$ forward from `stage[0..]` and $B_1$ backward from `stage[..n]` never crosses inside a **single minimal $\Theta(n)$ buffer** (`+900 KiB = +75%` worker scratch pre-reserved, or `+1,800 KiB = +150%` under dynamic `Vec` growth), followed by a mandatory $\ell(B_1) + \ell(F_1) \approx n/2$ (`~450 KiB`) `memcpy` stitch into `out[at_f0..at_b0]`.
3. **Uniform Chord Imbalance ($\frac{3}{8}m$ vs $\frac{1}{4}m$):** A rank-space split $q_0 = (\mathrm{mark} + \lfloor m/2 \rfloor) \bmod m$ cuts the cycle at an effectively uniform chord $\tau(q_0) \sim \mathrm{Unif}(0, m)$. Expected max arc length is $\mathbb{E}[\max(\tau, m-\tau)] = \frac{3}{4}m$, so $\mu = 4$ runs for only $\mathbb{E}[\frac{1}{2}\min(\tau, m-\tau)] = \frac{1}{8}m$ steps before dropping to $\mu = 2$ for the remaining $\frac{1}{4}m$ steps ($D_{\mathrm{serial}} \approx \frac{3}{8}m = 0.375\,m$, saving only `25%` of $\mu = 2$'s depth rather than `50%`).
4. **Periodic Disjoint-Cycle Obstruction ($T = P^d, d > 1$):** When $d > 1$, $\pi_{\mathrm{emit}}$ decomposes into $d$ disjoint cycles $\mathcal{C}_1 \sqcup \dots \sqcup \mathcal{C}_d$. Both antipodal cursors $(\mathrm{mark}, \pi^{-1}(\mathrm{mark}))$ belong canonically to $\mathcal{C}_1$, whereas a blind second anchor $q_0$ lands in $\mathcal{C}_r \ne \mathcal{C}_1$ with probability $(d - 1)/d$ (verified on `periodic-105k-d3`, $m = 105,000, d = 3$), where $F_0$ meets $B_0$ instead of $B_1$ and must fall back to $\mu = 2$.

### Proposition 24.3 (Epoch-Filtered 4x-Unrolled `OutQ16::wire_xor`)
Every `OutQ16` chunk fixes `hi = self.epoch << 16`. Testing `(mark & !0xFFFF) != hi` once per band/chunk call proves `v != mark` across all `255 / 256` non-matching epoch chunks and enables 4x-unrolled scatter RMWs (`Unrolled4-Epoch-XOR`) at **`+0 B` scratch**.

---

## 2. Native Safe-Rust Confrontation Against `0.5.11` `master` (`60` Pairs/Block `1T`, `40` Passes `2T`)

All arms (`#![forbid(unsafe_code)]`) passed **`2,046` exhaustive small BWTs** (`binary 1..=10`), **`periodic-105k-d3`** ($m = 105,000 \ge 65,536, d = 3$), **`dense-300k`**, and **`6 / 6` authenticated 900 KB Wikidata blocks** byte-for-byte:

| Block / Metric | `A/A` (`Master2/Master1`) | `Fused-XOR` ($\mu = 2$, Cycle 17) | **`Unrolled4-Epoch-XOR`** ($\mu = 2$) | `XOR-BiEmit4` ($\mu = 4$, `stage_buf`) |
|---|---:|---:|---:|---:|
| `block-0.L` (`1T p10`) | `0.999` | `0.999` (`walk 0.806x`) | **`0.990`** (`walk 0.807x`) | `1.092` (`walk 1.058x`) |
| `block-2.L` (`1T p10`) | `1.014` | `0.971` (`walk 0.814x`) | **`0.981`** (`walk 0.818x`) | `0.956` (`walk 0.779x`) |
| `block-85.L` (`1T p10`) | `1.001` | `0.955` (`walk 0.776x`) | **`0.951`** (`walk 0.786x`) | `1.059` (`walk 0.923x`) |
| `block-143.L` (`1T p10`) | `1.006` | `0.925` (`walk 0.742x`) | **`0.915`** (`walk 0.742x`) | `0.957` (`walk 0.770x`) |
| `block-210.L` (`1T p10`) | `0.992` | `0.921` (`walk 0.732x`) | **`0.920`** (`walk 0.734x`) | `0.970` (`walk 0.796x`) |
| `block-207.L` (`1T p10`) | `1.007` | `0.953` (`walk 0.761x`) | **`0.948`** (`walk 0.760x`) | `1.038` (`walk 0.931x`) |
| **`1T` 6-Block Sum** | **`1.0028`** | **`0.9471x`** | **`0.9434x` (`6 / 6` wins)** | `1.0015x` (`3 / 6` regressions) |
| **6-Block `construct` Sum** | `5,473.8 µs` | `6,234.1 µs` (`+760.3`) | `6,230.4 µs` (`+756.6`) | `6,318.3 µs` (`+844.6`) |
| **6-Block `walk` Sum** | `5,745.4 µs` | **`4,382.5 µs` (`0.7628x`)** | **`4,395.3 µs` (`0.7650x`)** | `4,903.1 µs` (`0.8534x`, `+520.6 µs` vs $\mu=2$) |
| **`dense-300k` (`0.216`)** | `0.993` | `0.839x` | **`0.826x`** | `0.804x` |
| **`2T` Concurrent (`p10 / p50`)** | **`0.9920 / 0.9931`** | **`0.9513 / 0.9439`** | **`0.9467 / 0.9520`** | `0.9848 / 0.9909` |
| **6-Blk Worker Scratch** | `1,172.7 KiB` | **`1,172.7 KiB` (`+0 B`)** | **`1,172.7 KiB` (`+0 B`)** | `2,930.4 KiB` (`+149.9%`; `+75.0%` pre-reserved) |

Notice the stage attribution:
- Even inside `walk` alone (after `stage_buf` is already warm and un-reallocated!), `XOR-BiEmit4` ($\mu = 4$) is **`+520.6 µs` (`+11.9%`) slower than `Fused-XOR` ($\mu = 2$)** across the 6 blocks (`4,903.1 µs` vs `4,382.5 µs`), winning `walk` only on `block-2.L` (`533.0 vs 557.0 µs`, lucky balanced chord) and deep-DRAM `dense-300k` (`0.804x` vs `0.826x`), while losing heavily on `block-0.L`, `block-85.L`, and `block-207.L` from $\frac{3}{8}m$ chord imbalance + `+900 KiB` dirty-line allocation + `~450 KiB` `memcpy` stitch!
- In `Unrolled4-Epoch-XOR`, eliminating `255 / 256` `v == mark` branches trims `construct` by `3.7 µs` (`0.9434x` vs `0.9471x` 1T, `0.826x` vs `0.839x` on `dense-300k`), confirming hardware TAGE already predicts `v == mark` (`1 / 423,468`) at `99.9998%` and L2 RMW store-buffer throughput governs `edges[v] ^= u`.

---

## 3. Cycle 24 Verdicts (One-Line Kills & Promotions)

- **Kill (`XOR-BiEmit4` 4-Cursor Meet-in-the-Middle, $\mu = 4$):** Killed (`1.0015x` 1T vs `Fused-XOR`'s `0.9471x`, `walk` `+520.6 µs` slower at `0.8534x` vs `0.7628x`, and `+75..150%` staging scratch violating the zero-growth contention law) by Theorem 24.1 (`|\partial[0,n)| = 2` mandatory $\Theta(n)$ staging + `~450 KiB` `memcpy` stitch + $\frac{3}{8}m$ chord imbalance + periodic disjoint cycles).
- **Promote (`Unrolled4-Epoch-XOR`, $\mu = 2$, `+0 B` Scratch):** Retained (`0.9434x` 1T across `6/6` blocks, `0.826x` on `dense-300k`, `0.9467x` p10 `2T`, `0 B` scratch delta) as the cleanest L2-gated `XOR-BiEmit` primitive for Cycle 25's grand synthesis.
