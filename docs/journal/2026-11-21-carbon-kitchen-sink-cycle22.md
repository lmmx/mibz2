# Carbon Kitchen Sink — Cycle 22: Yang–Baxter / Huffman-Wavelet Serial-Load Lower Bounds & Fused vs Split `XOR-BiEmit` Wiring

**Date:** 2026-11-21  
**Branch:** `research/carbon-kitchen-sink`  
**Theme:** Yang–Baxter 0-1 bit-plane sorting networks, Shannon–Huffman wavelet trees, Lehmer factorial radix inversion, and native safe-Rust confrontation of two-pass non-aliasing `Split-XOR` vs single-pass `Fused-XOR` (`Cycle 17`).

---

## 1. Mathematical Exploration: Yang–Baxter Bit-Plane Networks & Lehmer Inversion Codes

### 1.1 Theorem 22.1 (Shannon–Huffman Wavelet Serial-Load Lower Bound vs `Open` & `XOR-BiEmit`)
By the radix-exchange / Yang–Baxter sorting-network isomorphism, stable bucket sorting over an active alphabet $\Sigma_0$ of size $\alpha = |\Sigma_0|$ factors into a binary tree $T_W$ of commuting 0-1 stable bit-splits (`bit-plane rank64 operations`), where traversing character $c \in \Sigma_0$ executes $\mathrm{depth}_{T_W}(c)$ serial dependent bitvector rank loads. Minimizing total serial loads $\sum_{c \in \Sigma_0} \mathrm{count}(c)\,\mathrm{depth}_{T_W}(c)$ over **all** binary tree shapes $T_W$ is solved uniquely by **Huffman's algorithm**, bounded below by $n \cdot H_0(L)$:
1. Across all 6 authenticated Wikidata blocks ($\alpha \in [86, 96]$), a balanced wavelet network requires **`7.00n` (`6.30M`)** dependent bitvector loads (`30.8x–70.1x` `Open<16>`'s $m$), and **even the globally optimal Huffman wavelet tree** requires **`5.11n–5.15n` (`4.60M–4.63M`)** serial dependent loads — **`22.7x–51.4x` deeper than `Open<16>`** and **`45.4x–102.7x` deeper than `XOR-BiEmit`'s $\lceil m / 2 \rceil$**, while its bitvectors still occupy $\approx 5.13 n\text{ bits} \approx 564\text{ KiB}$ (comparable to `edges`).
2. Dually, encoding $\pi_{\mathrm{ret}} \in S_m$ in the **Lehmer factorial radix** ($c_j \in [0, m - j)$) requires an order-statistic tree ($\lceil \log_2 m \rceil = 17\text{--}18$ dependent loads per step) to invert, `17x–36x` deeper than `Open` / `XOR-BiEmit`.

### 1.2 Fused vs Split-Phase `XOR-BiEmit` Wiring
In Cycle 17, `Fused-XOR` paid `+745 µs` across 6 blocks in `construct` (`edges[..m].fill(0)` + scalar aliased `edges[u] ^= pack | v; edges[v] ^= u`). `Split-XOR` tests eliminating `edges[..m].fill(0)` and restoring non-aliasing AVX2 `queue.wire` in Phase 1, followed by a second `OutQ16` pass performing only `edges[v] ^= u` (with `v == mark` checked only on `a == mark_byte`).

---

## 2. Wavelet Census & Native Safe-Rust Confrontation (`A/B + A/A`)

### 2.1 Exact Yang–Baxter / Huffman-Wavelet Serial-Load Census

| Block | Active $\alpha$ | `Open` $m$ | Balanced Wavelet Loads (`vs m`) | Optimal Huffman Wavelet Loads (`vs m` / `vs BiEmit D`) |
|---|---:|---:|---:|---:|
| `block-0.L` | `89` | `89,874` | `6,299,517` (`70.1x`) | `4,615,602` (`5.13n` = **`51.4x`** / **`102.7x`**) |
| `block-2.L` | `86` | `112,070` | `6,300,000` (`56.2x`) | `4,622,451` (`5.14n` = **`41.2x`** / **`82.5x`**) |
| `block-85.L` | `94` | `156,404` | `6,299,545` (`40.3x`) | `4,603,124` (`5.11n` = **`29.4x`** / **`58.9x`**) |
| `block-143.L` | `96` | `196,841` | `6,299,083` (`32.0x`) | `4,597,039` (`5.11n` = **`23.4x`** / **`46.7x`**) |
| `block-210.L` | `96` | `204,281` | `6,299,713` (`30.8x`) | `4,634,557` (`5.15n` = **`22.7x`** / **`45.4x`**) |
| `block-207.L` | `94` | `135,921` | `6,299,986` (`46.4x`) | `4,603,942` (`5.12n` = **`33.9x`** / **`67.7x`**) |

### 2.2 Stage Attribution & End-to-End `1T` (`60` Pairs) + `2T` (`40` Passes) Confrontation

| Block / Suite | `A/A` (`Master2/Master1`) | **`Fused-XOR` (Cycle 17) / `Master1`** | `Split-XOR` (Cycle 22) / `Master1` | `construct` (`Master` / `Fused` / `Split`) |
|---|---:|---:|---:|---:|
| `block-0.L` (`1T p10`) | `1.008` | **`0.991x`** (`1207.3 / 1217.8 µs`) | `1.003x` (`1243.9 / 1239.6 µs`) | `680.4 / 758.6 / 808.8 µs` |
| `block-2.L` (`1T p10`) | `1.006` | **`0.976x`** (`1462.7 / 1497.9 µs`) | `1.003x` (`1510.8 / 1506.8 µs`) | `767.2 / 843.8 / 976.5 µs` |
| `block-85.L` (`1T p10`) | `1.004` | **`0.967x`** (`1461.6 / 1511.7 µs`) | `0.972x` (`1464.4 / 1506.4 µs`) | `600.1 / 742.6 / 767.3 µs` |
| `block-143.L` (`1T p10`) | `0.996` | **`0.921x`** (`2504.1 / 2719.9 µs`) | `0.945x` (`2573.0 / 2721.6 µs`) | `1334.5 / 1490.0 / 1484.6 µs` |
| `block-210.L` (`1T p10`) | `1.007` | **`0.917x`** (`2174.1 / 2369.8 µs`) | `0.940x` (`2205.5 / 2346.6 µs`) | `1006.1 / 1191.3 / 1213.5 µs` |
| `block-207.L` (`1T p10`) | `1.003` | **`0.943x`** (`1733.2 / 1838.6 µs`) | `0.979x` (`1765.8 / 1803.3 µs`) | `936.1 / 1042.8 / 1095.1 µs` |
| **`1T` 6-Block Sum** | **`1.0034`** | **`0.9451x` (`−5.49%`)** | `0.9676x` (`−3.24%`) | `5324.3 / 6069.1 / 6345.9 µs` |
| **`2T` Concurrent (`p10`)** | **`0.9926`** | **`0.9501x` (`−4.99%`)** | `0.9550x` (`−4.50%`) | — |
| **`2T` Concurrent (`p50`)** | **`0.9957`** | **`0.9445x` (`−5.55%`)** | `0.9736x` (`−2.64%`) | — |

---

## 3. Cycle 22 Verdicts (One-Line Kills)

- **Kill (`Yang–Baxter / Huffman-Wavelet Bit-Plane Decomposition`):** Killed because even the globally optimal Huffman wavelet tree executes `5.11n–5.15n` serial dependent bitvector loads (`22.7x–51.4x` `Open`'s $m$ and `45.4x–102.7x` `XOR-BiEmit`'s $m/2$).
- **Kill (`Lehmer Factorial-Radix Inversion`):** Killed because order-statistic tree decoding requires $\lceil \log_2 m \rceil = 17\text{--}18$ serial dependent loads per retained vertex vs `Open`'s `1`.
- **Kill (`Split-XOR` Two-Pass Wire):** Killed in favour of Cycle 17's single-pass `Fused-XOR` (`0.9676x` vs **`0.9451x`** at `1T`, `0.9736x` vs **`0.9445x`** at `2T p50`) because reading `OutQ16` and `bands` twice costs `+277 µs` more than `Fused-XOR`'s single L2-streamed `edges[..m].fill(0)`.
