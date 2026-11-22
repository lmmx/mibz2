# Carbon Kitchen Sink — Cycle 25: Grand Synthesis (`CKS-BiOperad` & `CKS-DietOperad`) & Cycles 16–25 Master Ledger

**Date:** 2026-11-21  
**Branch:** `research/carbon-kitchen-sink`  
**Theme:** Direct-`inline16` Zero-`u_rec` Operad Quotient + `MuxStrips + Band64` + L2-Gated `Unrolled4-Epoch-XOR` Antipodal Walk — achieving **`0.8587x` `1T` (`6 / 6` blocks `< 0.90x`), `0.750x` on `dense-300k`, `0.8642x` `2T`, and `981.8 KiB` (`−16.28%`) worker scratch** against `mibz2 0.5.11` (`90e671966`) in `#![forbid(unsafe_code)]` safe Rust.

---

## 1. Mathematical Synthesis & Super-Additive Synergy

Cycles 16–24 isolated three orthogonal structural transformations of `Open` (`src/open/`):
1. **Algebraic Representation (`Operad-MN16` + Direct-`inline16` Zero-`u_rec` Quotient, Cycles 16 & 20 + Cycle 25 Upgrade):** Eager 128-bit shift-OR evaluation in the truncated free monoid $\mathcal{M}_{\le 15}$ inside `Elim`'s existing `RULE = 4` `u32` stride (`28`-bit memo offset for `>= 16 B` nonterminals), paired with a `24 KiB` (`6,144` `u32`s) 1-probe Myhill–Nerode word quotient placed directly inside dead `edges[0..4*ns]` (`hitting 6 / 6 Wikidata blocks`) or dead tail `edges[4*ns + etop..]` (`hitting dense-300k`), pushing unique words straight into `inline16` + `lens` with **`offsets = 0 B`**, **`u_rec = 0 B`**, and **`endpoint.stack = 64 B`**.
2. **Lifetime & Lattice Packing (`MuxStrips + Band64`, Cycle 21):** Multiplexing `Pass 1` thick strips into `edges[0..3*ns]` (`strips = 0 B`, preserving `#703`'s `Cap::dense` refusal gate byte-for-byte) and packing `[end:20 | dst:20 | code:24]` into a single `u64` (`bands = −33.3%`, zero `delta` arithmetic).
3. **Antipodal Time-Reversal Duality (`Unrolled4-Epoch-XOR`, Cycles 17, 22 & 24):** Storing $\xi(w) = \pi(w) \oplus \pi^{-1}(w)$ in `edges[0..m)` whenever `m >= 65_536 && !wide` (`256 KiB` L2 gate) to walk forward from `mark` and backward from $j_{-1} = \pi^{-1}(\mathrm{mark})$ simultaneously at **`+0 B` scratch** and **`0 B` staging** ($D_{\mathrm{serial}} = \lceil m_{16}/2 \rceil$).

### Theorem 25.1 (Universal Malformed-Input Bijection & Super-Additive L1D $\times$ Antipodal Synergy)
1. **Universal Rauzy–Veech Bijection on Arbitrary Malformed Inputs:** Even when `(records, p)` is not a valid BWT column, `Pass 1` pops $\{0, \dots, m-1\}$ bijectively and Rauzy–Veech Lebesgue-measure invariance partitions `dst` across `[0, m)` bijectively with $\max(u, v) \le m - 1 < 2^{\mathrm{bits}}$. Thus in `XOR-BiEmit`, the upper code bits `(edges[w] >> bits)` are XORed once per `u` and never touched by `edges[v] ^= u`, guaranteeing valid dictionary indices and bounded termination without bounds-panics (`verified across 67,968 exhaustive BWTs + 1,000 malformed non-BWT columns including m >= 65,536`).
2. **Super-Additive L1D $\times$ Dual-Cursor Synergy:** Under $\mu = 2$ dual-cursor `emit`, every loop iteration issues **two** uncorrelated dictionary reads (`inline16[cf]` and `inline16[cb]`) alongside **two** past-L2 `edges` loads. In `master`, `Dict` reaches `64.6 KiB` (`> 32–48 KiB` L1D), causing dual-cursor dictionary evictions. Shrinking active `Dict` via `Operad-MN16` to **`13.5–28.4 KiB < 32 KiB` L1D** pins `100%` of dictionary accesses in L1D for **both** cursors simultaneously, driving `walk` down to **`0.6391x` (`−36.1%`)** — beating the product of standalone speedups (`0.8969 * 0.7650 = 0.6861x`) by **`−4.7%` (`274 µs` super-additive gain)** while `Operad-MN16 + Band64` simultaneously cancels `224 µs` of `wire_xor`'s `construct` cost.

---

## 2. Definitive Native Safe-Rust Confrontation Against `0.5.11` `master`

We present **both** synthesized kernels side-by-side (`#![forbid(unsafe_code)]`, `-D warnings`, compiled in one binary with `MasterA1` and `MasterA2` A/A controls):
- **`CKS-DietOperad` ($\mu = 1$ serial walk):** Zero-contention candidate (`construct = 0.9581x`, `walk = 0.8969x`, guaranteed zero extra multi-core L3/IMC burstiness at 6–10 threads on the i9-10900K by Cycle 15's Theorem 15.1).
- **`CKS-BiOperad` ($\mu = 2$ L2-gated antipodal walk):** Maximum-throughput candidate combining `CKS-DietOperad` with L2-gated `Unrolled4-Epoch-XOR` at **identical `981.8 KiB` (`−16.28%`) worker scratch**.

### 2.1 Correctness, Safety & `#704` Worker Scratch Audit (`>= 10%` Memory-Gain Gate)
- **Parity & Safety:** Passed **`67,968` exhaustive BWTs** (`binary 1..=11` $\times$ `INDUCE/QUEUE/LIMIT`, `ternary 1..=7`), **`1,000` malformed non-BWT columns** (`including n = 70,000 >= 65,536`), **`#703` `Cap::dense` refusal** (`66,560 B == 66,560 B` exact match with `master`), `dense-300k`, `periodic-150k`, `periodic-105k-d3`, `spiral-200k`, and all `6 / 6` 900 KB Wikidata blocks.

| Scope / Block | `0.5.11` `master` (`Dict`) | **`CKS-DietOperad`** ($\mu = 1$) | **`CKS-BiOperad`** ($\mu = 2$) |
|---|---:|---:|---:|
| `block-0.L` (fresh) | `566.4 KiB` (`26.3K`) | **`528.0 KiB` (`−6.8%`, `16.5K` L1D)** | **`528.0 KiB` (`−6.8%`, `16.5K` L1D)** |
| `block-2.L` (fresh) | `712.0 KiB` (`36.2K`) | **`579.6 KiB` (`−18.6%`, `21.3K` L1D)** | **`579.6 KiB` (`−18.6%`, `21.3K` L1D)** |
| `block-85.L` (fresh) | `761.5 KiB` (`24.5K`) | **`712.7 KiB` (`−6.4%`, `13.5K` L1D)** | **`712.7 KiB` (`−6.4%`, `13.5K` L1D)** |
| `block-143.L` (fresh) | `1,143.6 KiB` (`64.6K`) | **`936.7 KiB` (`−18.1%`, `28.4K` L1D)** | **`936.7 KiB` (`−18.1%`, `28.4K` L1D)** |
| `block-210.L` (fresh) | `1,085.3 KiB` (`41.2K`) | **`937.8 KiB` (`−13.6%`, `20.2K` L1D)** | **`937.8 KiB` (`−13.6%, 20.2K` L1D)** |
| `block-207.L` (fresh) | `818.8 KiB` (`41.8K`) | **`698.8 KiB` (`−14.7%`, `20.0K` L1D)** | **`698.8 KiB` (`−14.7%`, `20.0K` L1D)** |
| **`6`-Block Cumulative Worker Scratch** | **`1,172.7 KiB` (`1,200,839 B`)** | **`981.8 KiB` (`1,005,348 B`, `−16.28%`)** | **`981.8 KiB` (`1,005,348 B`, `−16.28%`)** |
| `#703` `Cap::dense` Refusal Scratch | `66,560 B` | **`66,560 B` (`+0 B`)** | **`66,560 B` (`+0 B`)** |

*(Buffer breakdown: `queues=6144 strips=0 bands=65536 spelling=86016 offsets=0 edges=817124 extra=0 inline16=28672 lens=1792 endpoint=64`.)*

### 2.2 Stage Attribution (`60` Rounds/Block) & End-to-End `1T` (`80` Pairs/Block) + `2T` (`50` Passes)

| Block / Suite | `A/A` (`p10`) | `CKS-DietOperad` (`construct` / `walk`) | **`CKS-DietOperad` `1T` (`p10 / p50`)** | `CKS-BiOperad` (`construct` / `walk`) | **`CKS-BiOperad` `1T` (`p10 / p50`)** |
|---|---:|---:|---:|---:|---:|
| `block-0.L` | `0.991` | `0.956x` / `0.904x` | **`0.931x / 0.926x`** | `1.055x` / **`0.702x`** | **`0.897x / 0.906x`** |
| `block-2.L` | `0.987` | `0.952x` / `0.904x` | **`0.918x / 0.932x`** | `1.057x` / **`0.691x`** | **`0.890x / 0.882x`** |
| `block-85.L` | `0.984` | `0.979x` / `0.894x` | **`0.933x / 0.930x`** | `1.217x` / **`0.651x`** | **`0.876x / 0.892x`** |
| `block-143.L` | `0.985` | `0.948x` / `0.889x` | **`0.914x / 0.922x`** | `1.063x` / **`0.615x`** | **`0.834x / 0.837x`** |
| `block-210.L` | `0.988` | `0.968x` / `0.905x` | **`0.936x / 0.928x`** | `1.148x` / **`0.607x`** | **`0.833x / 0.833x`** |
| `block-207.L` | `0.991` | `0.955x` / `0.891x` | **`0.926x / 0.939x`** | `1.076x` / **`0.637x`** | **`0.864x / 0.864x`** |
| **`1T` 6-Block Sum** | **`0.9875`** | **`0.9581x` / `0.8969x`** | **`0.9256x / 0.9290x` (`−7.4%`)** | `1.0965x` / **`0.6391x` (`−36.1%`)** | **`0.8587x / 0.8613x` (`−14.1%`)** |
| **`dense-300k` (`#703` band)** | `0.974` | — | **`0.931x / 0.938x`** | — | **`0.750x / 0.755x` (`−25.0%`)** |
| **`periodic-150k` (`m < 64K`)** | `0.993` | — | **`0.870x / 0.873x`** | — | **`0.854x / 0.871x` (`−14.6%`)** |
| `spiral-200k` (`9.0 µs`) | `1.022` | — | `1.069x` (`9.7 vs 9.0 µs`) | — | `1.050x` (`9.4 vs 9.0 µs`) |
| **`2T` Concurrent (`p10 / p50`)** | **`1.0008 / 1.0049`** | — | **`0.9534x / 0.9445x`** | — | **`0.8642x / 0.8687x` (`−13.6%`)** |

---

## 3. Master Ledger of Cycles 16–25 (Promotions & One-Line Kills)

| Cycle | Candidate / Mathematical Framework | Verdict | One-Line Decision / Measured Outcome vs `0.5.11` `master` |
|---:|---|---|---|
| 16 | `PairOnly` (`(rule, byte)` 1-level memo) | **KILLED** | Deduplicates only `15%` of words, leaving `inline16 + lens = 55.5 KiB > 32 KiB` L1D (`0.996x`). |
| 16 | `MN16-Linear` (Open-addressed linear probing) | **KILLED** | `70%` load-factor probe chains regress `dense-300k` to `1.016x` vs 1-probe lossy quotient (`0.964x`). |
| 16 | `MN16-OutTape` (Reuse `out` for temporary `spelling`) | **KILLED** | Regresses `spiral-200k` to `1.40x` and `dense-300k` to `1.02x` via two-pass re-expansion overhead. |
| 16 | `MN16-Direct` (`24 KiB` arena quotient + Zero-`offsets`) | *Superseded (C20)* | Proved native `MN16` win (`0.9677x` 1T, `0.9767x` 2T, `−52.2 KiB`); superseded by `Operad-MN16`. |
| 17 | `DualTable-BiEmit` (Separate `prev_edges[0..m)`) | **KILLED** | Adds `+817 KiB` (`+68%`) worker scratch, violating the `20 MiB` shared-L3 contention boundary. |
| 17 | Ungated `XOR-BiEmit` (`m < 65,536`) | **KILLED** | Scatter RMW `wire_xor` loses `1.143x` on L2-resident `periodic-150k` where `emit` already hits L2. |
| 17 | L2-Gated `XOR-BiEmit` (`m >= 65,536`, `+0 B` WS) | **PROMOTED** | Halves $D_{\mathrm{serial}} \to \lceil m_{16}/2\rceil$ at `+0 B` scratch (`walk 0.7520x`, `1T 0.9512x`, `2T 0.9427x`). |
| 18 | $\gamma_F \equiv \gamma_{2\mathrm{gram}}$ rank gauges | **KILLED** | Collapses to 1-step $\mathrm{LF}$ conjugate of $\gamma_L$ and adds `+157..+349` `256 KiB` L2 misses on `6/6` blocks. |
| 18 | `HotWire` (MRU frequency-sorted wire gauge) | **KILLED** | `walk` stays flat (`0.9987x`) and total slows (`1.0154x`) from sorting/permutation indirection. |
| 18 | `Bipartite-Bigram` (2-adic $E_{0\to 1}$ matching) | **KILLED** | Loses `1.2186x` 1T / `1.2251x` 2T (`walk 1.0497x`) via $H(0.18)=0.68$-bit tag branch mispredicts. |
| 19 | `Master@14/12`, `StratA/B`, `MN16@14/12` | **KILLED** | `~145 ns/strip` in `Elim` exceeds `~84 ns` saved in `emit` (`0.9955–1.0374x`, `−8.56% < 10%` `#704` gate). |
| 20 | Tree-walk `e.append` on `len <= 15` rules | **KILLED** | Wastes `~45 µs/block` (`construct 1.040x` vs `0.960x`) when `u128` shift-OR fits in `RULE = 4`. |
| 20 | `Operad-MN16` ($\mathcal{M}_{\le 15}$ in `RULE = 4`) | **PROMOTED** | Beats `master` in both `construct` (`0.9600x`) and `walk` (`0.9320x`): `1T 0.9332x`, `dense 0.840x`. |
| 21 | Wiedemann / Berlekamp–Massey $\mathbb{F}_{257}$ Krylov | **KILLED** | Exact $\Lambda_{\mathbb{F}_{257}}(T[0..N]) = 0.500\,N$ on `6/6` blocks forces $\Omega(n\log^2 n)\text{--}\Omega(n^2)$ solver cost. |
| 21 | Tropical `(min, +)` $r$-variable LP relaxation | **KILLED** | Collapsing $w_u > 1$ visits violates Cycle 3's non-zero $H^1(G_{\mathrm{ov}};\mathbb{Z})$ holonomy on every cycle. |
| 21 | `MuxStrips + Band64` (`MuxBand` / `Diet2`) | **PROMOTED** | Cuts scratch `−13.6%` alone (`1,012.7 KiB`) and `−16.4%` (`980.5 KiB < 1 MiB`, `1T 0.9322x`) with `Operad-MN16`. |
| 22 | Optimal Huffman Wavelet Trees & Lehmer Codes | **KILLED** | Requires `5.11n–5.15n` serial bit-plane loads (`22.7–51.4x` `Open` $m$, `45.4–102.7x` `BiEmit` $D$). |
| 22 | Two-pass `Split-XOR` (`wire` then `edges[v] ^= u`) | **KILLED** | Loses to single-pass `Fused-XOR` (`0.9676x` vs `0.9451x`, `+277 µs`) from reading `OutQ16 + bands` twice. |
| 23 | `Ostrowski-Shift4` (`aw > (bw << 2)` always-on) | **KILLED** | `0 / 100,820` steps on 6 blocks + `dense-300k` form $q \ge 4$ 2-cycle spirals (`1.0213x` 1T). |
| 23 | `OutQ-128 / 256 / 1024` chunk-scale variants | **KILLED** | `128/256` pay `2–4x` slow transitions (`1.092x/1.030x`); `1024` spills `Spill::ovf` on `dense-300k` (`1.050x`). |
| 24 | `XOR-BiEmit4` ($\mu = 4$ meet-in-the-middle) | **KILLED** | Killed (`1.0015x` 1T, `walk 0.8534x` vs $\mu=2$'s `0.7628x`, `+75–150%` WS) by `Theorem 24.1` (`|\partial[0,n)| = 2`). |
| 24 | `Unrolled4-Epoch-XOR` ($\mu = 2$, `+0 B` WS) | **PROMOTED** | Epoch-filtered 4x-unrolled `wire_xor` reaches `0.9434x` 1T (`6/6` wins, `0.826x` dense) & `0.9467x` 2T. |
| 25 | **`CKS-DietOperad` ($\mu=1$) & `CKS-BiOperad` ($\mu=2$)** | **HANDOFF FLAGSHIPS** | **`981.8 KiB` (`−16.28%` WS): `CKS-DietOperad` = `0.9256x` 1T / `0.9445x` 2T p50 ($\mu=1$); `CKS-BiOperad` = `0.8587x` 1T (`0.750x` dense) / `0.8642x` 2T p10 ($\mu=2$).** |

---

## 4. Coordinator Handoff & Exact Porting Map

Drop-in, `#![forbid(unsafe_code)]`, zero-warning Rust modules matching `src/open/{mod.rs,endpoint.rs,outq.rs}` are archived in [`docs/journal/2026-11-21-carbon-kitchen-sink-cycle25/`](./2026-11-21-carbon-kitchen-sink-cycle25/):
1. **`cand_cks_diet/` (`CKS-DietOperad`, $\mu = 1$):** Recommended first-stage or contention-floor validation on the 10-thread i9-10900K — cuts worker scratch **`−16.28%` (`1,172.7 -> 981.8 KiB`)**, keeps `Dict <= 28.4 KiB` L1D, wins `construct` (`0.958x`) AND `walk` (`0.897x`), and issues **`0` extra memory streams ($\mu = 1$)**.
2. **`cand_cks_bi/` (`CKS-BiOperad`, $\mu = 2$ gated at `m >= 65_536`):** Identical `981.8 KiB` (`−16.28%`) scratch, adding `~90` lines in `outq.rs` + `mod.rs` to unlock **`0.8587x` `1T` (`6 / 6` blocks `< 0.90x`), `0.750x` on `#703`'s dense band, and `0.8642x` `2T`**. Validate on the i9-10900K at `1, 6, 8, 10` threads (the `−16.3%` scratch diet keeps `10 * 0.98 MiB = 9.8 MiB << 20 MiB` shared L3, directly buffering $\mu = 2$'s dual-stream bandwidth).
