# Carbon Kitchen Sink — Cycle 21: Wiedemann Krylov Saturation over $\mathbb{F}_{257}$ vs Bit-Lattice & Lifetime-Multiplexed Geometry (`MuxStrips + Band64`)

**Date:** 2026-11-21  
**Branch:** `research/carbon-kitchen-sink`  
**Theme:** Finite-field Wiedemann / Berlekamp–Massey Krylov linear complexity ($\Lambda_{\mathbb{F}_{257}}$), tropical difference-constraint holonomy, and 20-bit coordinate lattice packing + lifetime arena multiplexing (`MuxStrips + Band64`) taking per-worker `Kernel` scratch **below `1 MiB` (`980.5 KiB`, `−16.4%` vs `0.5.11` `master`)** with a **`6.8%` speed gain**.

---

## 1. Mathematical Exploration: Wiedemann Krylov Subspaces vs Lifetime Disjointness & Bit-Lattice Geometry

### 1.1 Theorem 21.1 (Wiedemann Krylov Equivalence & $\mathbb{F}_{257}$ Linear Complexity Saturation)
Define the byte-valuation vector $u = \sum_{x=0}^{n-1} (L[x] + 1) e_x \in \mathbb{F}_{257}^n$ (`piecewise constant on r runs`) and primary basis vector $e_p$. Then the reversed decoded text is identically Wiedemann's scalar Krylov sequence:
$$s_t \;=\; u^\top P_{\mathrm{LF}}^t e_p \;=\; T[n - 1 - t] + 1 \;\in\; \mathbb{F}_{257} \qquad (0 \le t < n).$$
Although $P_{\mathrm{LF}} w$ costs $O(r)$ operations on step vectors and $\operatorname{rank}[D, P_{\mathrm{LF}}] \le r - 1$, on any primitive BWT $m_{P_{\mathrm{LF}}}(x) = x^n - 1$. Exact Berlekamp–Massey evaluation on prefix windows $T[0..N)$ ($N \in \{1024, 4096\}$) across **all 6 / 6** authenticated Wikidata blocks yields:
$$\Lambda_{\mathbb{F}_{257}}\bigl(T[0..1024)\bigr) = 512 = 0.500\,N, \qquad \Lambda_{\mathbb{F}_{257}}\bigl(T[0..4096)\bigr) = 2048 = 0.500\,N$$
to the exact integer on every block. Extrapolating $s_t$ via Krylov minimal polynomials requires degree $\approx n/2$ ($\Omega(n^2)$ classical Berlekamp–Massey, $\Omega(n \log^2 n)$ superfast Half-GCD), strictly losing to `Open`'s $O(n)$ walk. Dually, in the tropical `(min, +)` semiring, relaxing $\tau(\mathrm{LF}(x)) - \tau(x) \le -1$ ($x \ne \Psi(p)$) to $r$ run-potentials $\bar{\phi}(u)$ collapses $w_u > 1$ distinct orbit times of diameter $\Theta(n)$ onto one variable, infeasible by Cycle 3's $H^1(G_{\mathrm{ov}};\mathbb{Z})$ holonomy.

### 1.2 Theorem 21.2 (`MuxStrips` Lifetime Disjointness & `Band64` Bit-Lattice Bijection)
After `#692` and `#703`, `strips` (`131,072 B`) and `bands` (`98,304 B`) account for **`229,376 B` (`19.1%`)** of `0.5.11` `master`'s `1,200,839 B` persistent scratch:
1. **`MuxStrips` (`strips = 0 B` via `Pass 1` $\to$ `Elim` Disjointness):** `self.strips` is written in `Pass 1`, updated in `finish`, read once in `induced_with` to seed `Elim`'s initial node widths/bytes, and is **100% dead before `Elim::eliminate` begins**. Meanwhile, `self.edges` is **100% unused during `Pass 1`**. Packing each strip into `3` `u32` words `[from, to, (width << 8) | byte]` inside `edges[0 .. 3 * ns]`, followed by `image_order` in `edges[3 * ns .. 4 * ns]` and `Elim` in `edges[4 * ns ..]`, eliminates `Kernel::strips: Vec<Strip>` (`−131,072 B`) while respecting `#703`'s `cap.strips` gate (`refused blocks allocate <= 3 * cap.strips` words, verified `<= 4 KiB` on tight cap). On `6 / 6` full blocks, `4 * ns + 8 * (3 * ns + 3) <= m` holds on `block-2..210` (`and is within 6.7K words on block-0`), adding **`0 B`** to cumulative `edges` (`817,124 B == 817,124 B`).
2. **`Band64` (`96 bits -> 64 bits` Direct Coordinate Lattice):** Because `OpenCells` enforces $n \le 900{,}000 < 2^{20}$, replacing `(end: u32, delta: u32, code: u32)` (`12 B`) with `dst` directly (`eliminating delta = dst - k` in `endpoint.rs` and `k + delta` in `wire`!) packs `[end: 20 | dst: 20 | code: 24]` bijectively into a single `u64` (`8 B`, cutting `bands` from `98,304 B` to `65,536 B`, `−32,768 B`).

---

## 2. Exhaustive Verification & Native Safe-Rust Confrontation (`A/B + A/A`)

- **`18,414` exhaustive checks** (`all binary BWTs of length 1..=10` across `QUEUE = true/false` and `LIMIT` $\in \{1, 4, 16\}$, plus `Cap` refusal scratch verification) + `6` authenticated 900 KB blocks + `dense-300k` + `spiral-200k`: **`100.000%` byte-for-byte parity**.

### 2.1 Persistent Worker Scratch (`diet_scratch`, Crossing `#704`'s `>= 10%` Memory-Gain Bar)

| Arm | Cumulative 6-Block Scratch | Delta vs `0.5.11` `master` | Breakdown |
|---|---:|---:|---|
| `master` (`0.5.11`) | `1,200,839 B` (`1,172.7 KiB`) | baseline | `strips=131072 bands=98304 spelling=65536 offsets=16384 inline16=62256` |
| `MuxBand` (`MuxStrips + Band64` alone) | **`1,036,999 B` (`1,012.7 KiB`)** | **`−160.0 KiB` (`−13.6%`)** | `strips=0 bands=65536 spelling=65536 offsets=16384 inline16=62256` |
| **`Diet2` (`Operad-MN16 + MuxBand`)** | **`1,003,988 B` (`980.5 KiB`)** | **`−192.2 KiB` (`−16.4%`)** | `strips=0 bands=65536 spelling=86016 offsets=0 inline16=27392 endpoint=64` |

*(Per-block fresh scratch drops by **`−8.5%` to `−19.8%`** across all 6 blocks and **`−16.2%`** on `dense-300k`.)*

### 2.2 End-to-End `1T` (`60` Pairs/Block) & `2T` (`40` Passes) Interleaved Ratios vs `0.5.11` `master`

| Block / Suite | `A/A` (`Master2/Master1`) | `MuxBand` Alone / `Master1` | **`Diet2` (`Operad-MN16 + MuxBand`) / `Master1`** |
|---|---:|---:|---:|
| `block-0.L` (`1T p10`) | `1.017` | `1.011` (`1263.2 / 1249.4 µs`) | **`0.957x`** (`1188.2 / 1241.4 µs`) |
| `block-2.L` (`1T p10`) | `0.994` | `1.001` (`1516.5 / 1515.1 µs`) | **`0.932x`** (`1405.0 / 1508.1 µs`) |
| `block-85.L` (`1T p10`) | `1.003` | `0.994` (`1512.9 / 1521.7 µs`) | **`0.943x`** (`1445.5 / 1532.9 µs`) |
| `block-143.L` (`1T p10`) | `0.982` | `0.992` (`2739.0 / 2761.3 µs`) | **`0.919x`** (`2520.5 / 2742.2 µs`) |
| `block-210.L` (`1T p10`) | `0.982` | `0.980` (`2396.6 / 2444.7 µs`) | **`0.927x`** (`2250.4 / 2427.1 µs`) |
| `block-207.L` (`1T p10`) | `0.990` | `0.995` (`1800.1 / 1808.3 µs`) | **`0.933x`** (`1691.8 / 1812.9 µs`) |
| **`1T` 6-Block Total** | **`0.9918`** | **`0.9936x`** (wall-neutral, **`−13.6%` scratch**) | **`0.9322x` (`−6.78%` wall, `−16.4%` scratch)** |
| `dense-300k` (`0.216`) | `0.993` | `0.996x` | **`0.880x` (`−12.0%` wall, `−16.2%` scratch)** |
| `spiral-200k` | `0.971` | `0.980x` | `1.048x` (`48.5 / 46.3 µs`) |
| **`2T` Concurrent (`p10`)** | **`1.0062`** | **`0.9837x`** | **`0.9438x` (`−5.62%`)** |
| **`2T` Concurrent (`p50`)** | **`0.9916`** | `1.0053x` | **`0.9383x` (`−6.17%`)** |

---

## 3. Cycle 21 Verdicts (Keep & One-Line Kills)

- **SURVIVOR (Advance to Cycle 25 Synthesis): `MuxStrips + Band64` (and `Diet2 = Operad-MN16 + MuxBand`)** — Standalone `MuxBand` cuts worker scratch by **`−13.6%` (`−160.0 KiB`)** at neutral speed (`0.9936x` vs A/A `0.9918x`, qualifying under `#704` on memory alone), while `Diet2` takes worker scratch **below `1 MiB` (`980.5 KiB`, `−16.4%`)** AND wins **`0.9322x` (`1T`) / `0.9438x` (`2T p10`) / `0.9383x` (`2T p50`)** across `6 / 6` blocks.
- **Kill (`Wiedemann / Berlekamp–Massey \mathbb{F}_{257} Krylov Solver`):** Killed because $\Lambda_{\mathbb{F}_{257}}(T[0..N]) = 0.500\,N$ (`512/1024`, `2048/4096`) identically on `6 / 6` blocks, forcing $\Omega(n \log^2 n)$–$\Omega(n^2)$ polynomial arithmetic vs `Open`'s $O(n)$.
- **Kill (`Tropical (min, +) Run-Potential LP Relaxation`):** Killed because collapsing $w_u > 1$ run visits to one tropical variable violates Cycle 3's non-zero $H^1(G_{\mathrm{ov}};\mathbb{Z})$ holonomy on every multi-visit cycle.
