# Carbon Cycle 17: Shared-L3 Paged-Emit Crossover, Exact SWAR Epoch-Cut Guard, and Sentinel-Masked Safe Emit (`SentOutQ16` / `SentOutQ14`)

- **Branch:** `research/carbon-word-transport`
- **Baseline:** `origin/master` `mibz2 0.5.8` (`mibz2::bwt::OpenCells`, commit `bb98c99`; `src/` identical in `v0.5.9` `10207ff`)
- **Prior Internal Baselines:** Cycle 15 `EpochQ16-ZeroClearR4` (`c6e41b83`) and Cycle 16 `const LIMIT=16` `SwarOutQ16-CompactR4` (`d42b7fb9`)
- **Safety:** `#[forbid(unsafe_code)]` across all candidate modules
- **Gate G5 Verdict:** **STRICT `PASS` (`4/4` sub-gates passed on both `SentOutQ16` and `SentOutQ14`)**
- **Artifacts:**
  - Acquisition manifest: `docs/journal/2026-11-21-carbon-cycle17/sent_outq16_r4.toml` (`check_manifest.py`: `PASS`)
  - Exhaustive & adversarial verifier: `docs/journal/2026-11-21-carbon-cycle17/verify_cycle17.py` (`4,586,436` SWAR/epoch-cut cases + `1,378` adversarial cases + `6/6` real admitted Wikidata blocks: `PASS`)
  - Native Rust + `chaingrind` Gate G5 suite: `docs/journal/2026-11-21-carbon-cycle17/g5_sent_outq16_c17.rs`

---

## 1. Explore & Triage: Why Paged Multi-Lane `emit` Regresses Under Shared L3 (`Theorem 17.1`)

Cycle 16 eliminated cyclic `W = 10` (`2,048 KiB` LLC) queue eviction via `OutQ16` (`0.355 DRAM B/outB`), leaving `dep_past_l2_loads ≈ 324,000` serial 42-cycle L3 hits (`~40%` of kernel cycles) in Phase 4 `emit` as the largest remaining bottleneck.

In `Explore`, we built **Unified-Slice Work-Stealing Bipolar Multi-Lane `emit` (`WS-2L` at `S = 8` cuts and `WS-4L` at `S = 16` cuts, `#[forbid(unsafe_code)]`)** on top of both `SwarOutQ16` and `ContigQ16`:
- Lane `0` walks in-place from `out[0]` upward and steals untouched cuts `1, 2, ...` with zero copies;
- Auxiliary lanes claim cuts from `S/2` outward and allocate `4 KiB` pages downward from the top of `out` itself (`+6.0 KiB` retained heap: `1,729.4 KiB` = `0.386×` `Master-0.5.8`), assembling only the non-in-place suffix `[seg0_len .. n)` via Kahn topological `out.copy_within`.

Measuring all 7 arms in **both** the uncontended large-LLC cloud VM (`11` rounds) and `chaingrind` at i9-10900K `W = 8` (`LLC = 2,560 KiB`) and `W = 10` (`LLC = 2,048 KiB`) reproduced `#675`'s exact real-hardware reversal:

| Explore Arm | Heap (`KiB`) | VM `W=10` Wall | `chaingrind` `2048 KB` (`W=10`) `cyc/B` | `chaingrind` `2048 KB` (`W=10`) `DRAM B/B` | `dep_L3` (`2048 KB`) | `L2_miss` | `rep_stores` |
|---|---:|---:|---:|---:|---:|---:|---:|
| `Master-0.5.8` (`OpenCells`) | `4,479.5` (`1.000×`) | `1.000×` | `5.438` (`1.000×`) | `3.400` (`1.000×`) | `330,739` | `499,119` | `435,496` |
| `ContigQ16-1L` | `2,320.3` (`0.518×`) | `0.903×` | `4.766` (`0.877×`) | `1.415` (`0.416×`) | `323,745` | `473,329` | `0` |
| `SwarOutQ16-1L` | `1,723.4` (`0.385×`) | `0.938×` | `4.780` (`0.879×`) | **`0.423` (`0.124×`)** | `323,473` | `455,329` | `0` |
| `SwarOutQ16-WS2L` (`S=8`) | `1,729.4` (`0.386×`) | `0.911×` | `5.749` (`1.057×`) | `1.246` (`0.366×`) | `317,682` | `551,132` | `2,090,019` |
| `SwarOutQ16-WS4L` (`S=16`) | `1,729.4` (`0.386×`) | `0.880×` | `5.354` (`0.985×`) | `1.843` (`0.542×`) | `253,065` | `597,277` | `2,761,738` |
| `ContigQ16-WS2L` (`S=8`) | `2,326.3` (`0.519×`) | `0.906×` | `5.956` (`1.095×`) | `2.617` (`0.770×`) | `316,308` | `566,012` | `2,090,308` |
| `ContigQ16-WS4L` (`S=16`) | `2,326.3` (`0.519×`) | **`0.850×`** | **`5.589` (`1.028×` SLOWER)** | **`3.182` (`0.936×`)** | `252,808` | `612,722` | `2,762,027` |

### Theorem 17.1 (Shared-L3 Paged Multi-Lane Crossover — `proved` + `measured`)
Let an $L$-lane out-of-order emitter write fraction $\alpha \in (0, 1)$ of `out[0..n)` into `4 KiB` staging pages and reassemble $\alpha n$ bytes via topological `copy_within`.
1. **Cache-Line & `rep movsb` Tax:** Even when staging pages alias `out` (`0 B` separate arena), reassembling $\alpha n$ bytes issues $\alpha n$ `rep movsb` byte stores (`2.09M` at $L=2$, `2.76M` at $L=4$ across 8 blocks) and touches $2\alpha n$ non-sequential bytes across `out`, inflating `L2_miss` by `+96k..+157k` and `LLC_miss` at `2,048 KiB` by `+12k..+34k` (`DRAM B/outB` rises `3.0×–4.4×` on `OutQ16`, and `ContigQ16-WS4L` hits `3.182 DRAM B/B` and regresses to **`1.028×` slower** than `0.5.8`, matching `#675`'s `1.012–1.049×` i9 loss).
2. **Design Corollary:** Under `2,048–2,560 KiB`/worker shared L3, latency and instruction reductions in `Open` **must preserve single-pass sequential writes to `out[0..n)` (`\alpha = 0`)**.

### Killed in Triage (`triage_c17.rs`)
- `WS-2L` / `WS-4L` (`S = 8, 16`): killed by Theorem 17.1 (`3.0–4.4×` `W = 10` DRAM B/B inflation and `0.985–1.095×` `chaingrind` `cyc/B` regression).
- Manual 8-wide `[u8; 16] -> [u32; 8]` Phase-3b `SIMD8_WIRE`: killed (`78.69M` vs `78.34M` instructions; LLVM already auto-vectorizes `chunks_exact(2)` over `&out_bytes[..]`, and the manual `len >= 8` pre-loop adds a branch on short spans).
- `Po2Edges` (`1 << bits` `edges` slice mask): killed (`+129.5 KiB` heap to `1,852.0 KiB` and `+0.015 cyc/B` from sparse L3 set pressure for `-0.60M` instructions).
- `LIMIT = 20`: killed (`dep_past_l2_loads` jumps `326k -> 362k`, `chaingrind` `W = 8` `cyc/B` degrades `0.924× -> 0.975×` — explaining why `#679`'s `LIMIT = 24` lost most of its gain on the shared-L3 i9).
- `LIMIT = 8`: killed on heap/DRAM tradeoff (`heap` rises to `2,351.5 KiB = 0.525×` and `W = 10` `DRAM B/B` rises to `1.181` from larger `tops`/`bots`, though `dep_L3` drops to `251,559`).
- `2×` inner `budget` loop unroll (`UNROLL2`): neutral (`71.72M` vs `73.41M` instructions, `4.587` vs `4.586 cyc/B`, and `+0.3..0.9%` slower native wall due to larger uop footprint; kept `1×` loop).

---

## 2. Mathematical & Algorithmic Constructions (`Theorems 17.2–17.3`)

### Theorem 17.2 (Exact Carry-Free & Epoch-Cut-Preserving SWAR Bound — `proved` + `verified`)
Let `CAP_U16 = 4 * ceil(LIMIT / 4)` be the 64-bit SWAR write width in `u16` lanes (`16` for `LIMIT in {13..16}`, `12` for `LIMIT = 12`). In `PassOut::push_thin`, the fast path skips `push_slow` if and only if:
$$m_{\text{lo}} = (m \bmod 2^{16}) \le \text{M\_LO\_MAX} := \text{0xFFFF} - \text{CAP\_U16}.$$
*Proof & Sharpness:*
1. **Carry-freeness:** For every lane $k \in \{0, \dots, \text{CAP\_U16}-1\}$, $m_{\text{lo}} + k \le \text{0xFFFF} - 1 < 2^{16}$, so 4-lane `u64` multiplication `m_lo * 0x0001_0001_0001_0001 + 0x0003_0002_0001_0000 + k4 * 0x0004_0004_0004_0004` has zero inter-lane carries.
2. **Epoch-cut preservation:** For every thin run width $w \in \{1, \dots, \text{LIMIT}\} \subseteq \{1, \dots, \text{CAP\_U16}\}$, $m_{\text{lo}} + w \le (\text{0xFFFF} - \text{CAP\_U16}) + \text{CAP\_U16} = \text{0xFFFF} < 2^{16}$. Thus $((m + w) \gg 16) = (m \gg 16)$ AND $m + w < \text{ep\_end} = (\text{ep} + 1) \cdot 2^{16}$. Every exact epoch boundary $m + w = \text{ep\_end}$ (such as $m_{\text{lo}} = \text{0xFFF0}, w = 16$) is forced into `push_slow`, where `if cur_m == ep_end { ep_cuts[ep] = thin; }` snapshots the exact epoch cut. Replacing `0x10000 - CAP_U16` (`0xFFF0`, Cycle 16's bound) with `0xFFFF - CAP_U16` (`0xFFEF`) closes a rare `1-in-2^{20}` exact-boundary `ep_cuts` skip discovered during Cycle 17's `LIMIT = 20` sweep and exhaustively verified across `4,586,436` `(LIMIT, m_lo, w)` triples in `verify_cycle17.py`. $\square$

### Theorem 17.3 (Sentinel-Masked Budgeted Safe `emit` + Power-of-Two `n_codes` Cursor — `proved` + `verified`)
Suppose `wide == false` and `(n_codes as u32) <= (u32::MAX >> bits)`.
1. **Zero-Clear Power-of-Two Code Table (`n_codes` cursor):** Maintaining `inline16` and `lens` at power-of-two length `code_cap = 2^q` (`code_mask = code_cap - 1`) with a per-block cursor `n_codes` reset to `256` (without truncating or zeroing `[256..code_cap]`) eliminates `Vec::push` growth branches in `make_code` (`~8,000`/block), touches `0 B` beyond `0..n_codes`, and guarantees `((edge >> bits) as usize) & code_mask < code_cap` with zero bounds checks in `#[forbid(unsafe_code)]` Rust. Because every valid edge written in Phase 3b has `(edge >> bits) < n_codes < code_cap`, stale entries in `[n_codes + 1 .. code_cap)` are unreachable on valid graphs and bounded-length (`<= 16 B` or `LONG`) regardless.
2. **Single-Branch Inner Walk (`stop_code` + `budget = (n - at) >> 4`):** Allocating `stop_code = n_codes` with `lens[stop_code] = LONG` (`255 > 16`), emitting vertex `mark`'s initial word `code0` up front, and patching `edges[mark] = (stop_code << bits) | next0` folds cycle termination (`j == mark`) into the existing `ln <= 16` check (`stop_code` has `ln = 255 > 16`). Furthermore, while `n - at >= 64`, running inner loops of `budget = min((n - at) >> 4, steps_left)` steps guarantees that `at` advances by at most `16 * budget <= n - at` bytes if all `budget` steps satisfy `ln <= 16` (breaking out of the inner loop whenever `ln > 16` to recompute `budget`), removing both `j == mark` and `at >= n` from the per-step loop body and cutting total kernel instructions by **`-4.58M` vs `C16` and `-6.53M` (`-8.1%`) vs `Master-0.5.8`**. $\square$

### Shared-L3 `LIMIT` Co-Design (`SentOutQ16` vs `SentOutQ14`)
Lowering `LIMIT` from `16` to `14` shrinks `m` (`edges[0..m)` moves closer to the `256 KiB` per-core L2 cache), cutting `dep_past_l2_loads` from `331,176` (`0.5.8`) / `324,941` (`L=16`) down to **`300,359` (`-9.3%`)** and `L2_miss` from `499,670` down to **`440,872` (`-11.8%`)** with **zero page copies**, improving `chaingrind` `cycles/B` to **`0.900×` (`W = 8`) and `0.831×` (`W = 10`)** at only `+67.4 KiB` retained heap (`1,789.9 KiB = 0.400×` `0.5.8`). Both `SentOutQ16` (minimum heap & minimum `W = 10` DRAM traffic `0.104×`) and `SentOutQ14` (minimum `chaingrind` `cycles/B` `0.831×`) pass all 4 strict Gate G5 sub-gates.

---

## 3. Confront: Gate G5 Results (`g5_sent_outq16_c17.rs`)

### 3.1 Per-Worker Heap & Active Unique L3 Working Set (`228` Admitted Wikidata Blocks)

| Kernel | Per-Worker Retained Heap | vs `0.5.8` | Per-Worker Peak Heap | Mean Unique L3 WS (`incl. out`) | `p95` Unique L3 WS | `ovf_chunks` |
|---|---:|---:|---:|---:|---:|---:|
| `Master-0.5.8` (`OpenCells`) | `4,479.5 KiB` | `1.000×` | `4,479.5 KiB` | `> 2,500 KiB` | `> 3,000 KiB` | n/a |
| `C15-EpochQ16-ZeroClearR4` | `2,442.8 KiB` | `0.545×` | `2,442.8 KiB` | `~2,190 KiB` | `~2,520 KiB` | n/a |
| `C16-SwarOutQ16` (`const L=16`) | `1,722.5 KiB` | `0.385×` | `1,722.5 KiB` | `1,845.8 KiB` | `2,175.1 KiB` | `0` |
| **`C17-SentOutQ16` (`L=16`, Primary)** | **`1,722.5 KiB`** | **`0.385×`** | **`1,722.5 KiB`** | **`1,729.5 KiB`** | **`2,051.7 KiB`** | **`0`** |
| **`C17-SentOutQ14` (`L=14`, L2-Bal)** | **`1,789.9 KiB`** | **`0.400×`** | **`1,789.9 KiB`** | **`1,752.0 KiB`** | **`2,076.6 KiB`** | **`0`** |

### 3.2 `chaingrind` Shared-L3 Contention Proxy (`W = 8` `2,560 KiB` & `W = 10` `2,048 KiB`)

| `LLC` Slice | Kernel | Instructions (`vs 0.5.8`) | `cycles/B` (`vs 0.5.8`) | `DRAM B/outB` (`vs 0.5.8`) | `dep_past_l2` | `L2_miss` | `LLC_miss` |
|---|---|---:|---:|---:|---:|---:|---:|
| **`2,560 KiB` (`W=8`)** | `Master-0.5.8` | `80.75M` (`1.000×`) | `4.966` (`1.000×`) | `1.169` (`1.000×`) | `331,176` | `499,670` | `22,687` |
| `2,560 KiB` (`W=8`) | `C15-EpochQ16` | `81.37M` (`1.008×`) | `4.581` (`0.923×`) | `0.350` (`0.299×`) | `322,972` | `474,153` | `10,081` |
| `2,560 KiB` (`W=8`) | `C16-SwarOutQ16` | `78.80M` (`0.976×`) | `4.613` (`0.929×`) | `0.233` (`0.200×`) | `323,854` | `455,354` | `6,990` |
| **`2,560 KiB` (`W=8`)** | **`C17-SentOutQ16`** | **`74.22M` (`0.919×`)** | **`4.588` (`0.924×`)** | **`0.233` (`0.200×`)** | `324,941` | `455,761` | `6,998` |
| **`2,560 KiB` (`W=8`)** | **`C17-SentOutQ14`** | **`74.62M` (`0.924×`)** | **`4.471` (`0.900×`)** | **`0.244` (`0.208×`)** | **`300,359`** | **`440,872`** | `7,197` |
| **`2,048 KiB` (`W=10`)** | `Master-0.5.8` | `80.75M` (`1.000×`) | `5.448` (`1.000×`) | `3.439` (`1.000×`) | `330,829` | `499,670` | `65,355` |
| `2,048 KiB` (`W=10`) | `C15-EpochQ16` | `81.37M` (`1.008×`) | `4.773` (`0.876×`) | `1.676` (`0.487×`) | `322,855` | `474,153` | `35,886` |
| `2,048 KiB` (`W=10`) | `C16-SwarOutQ16` | `78.80M` (`0.976×`) | `4.638` (`0.851×`) | `0.353` (`0.103×`) | `323,674` | `455,354` | `9,391` |
| **`2,048 KiB` (`W=10`)** | **`C17-SentOutQ16`** | **`74.22M` (`0.919×`)** | **`4.612` (`0.847×`)** | **`0.359` (`0.104×`)** | `324,873` | `455,761` | `9,526` |
| **`2,048 KiB` (`W=10`)** | **`C17-SentOutQ14`** | **`74.62M` (`0.924×`)** | **`4.528` (`0.831×`)** | **`0.521` (`0.151×`)** | **`300,080`** | **`440,872`** | `12,610` |

### 3.3 Paired Native Rust Multi-Worker Execution (`15` Rotated Rounds, `lto=fat`, `228` Admitted Blocks)

| Workers | `Master-0.5.8` Wall (`CPU`) | `C15-EpochQ16` Wall (`CPU`) | `C16-SwarOutQ16` Wall (`CPU`) | **`C17-SentOutQ16` Wall (`CPU`)** | **`C17-SentOutQ14` Wall (`CPU`)** |
|---:|---:|---:|---:|---:|---:|
| `W = 1` | `333.1 ms` (`332.9 ms`) | `0.893×` (`0.893×`) | `0.939×` (`0.938×`) | **`306.1 ms` (`0.919×` / `0.919×`)** | **`308.4 ms` (`0.926×` / `0.926×`)** |
| `W = 2` | `168.1 ms` (`335.0 ms`) | `0.895×` (`0.895×`) | `0.937×` (`0.936×`) | **`154.8 ms` (`0.921×` / `0.920×`)** | **`155.7 ms` (`0.926×` / `0.926×`)** |
| **`W = 8`** | `180.5 ms` (`1415.6 ms`) | `0.898×` (`0.898×`) | `0.927×` (`0.926×`) | **`164.6 ms` (`0.912×` / `0.910×`)** | **`164.8 ms` (`0.913×` / `0.911×`)** |
| **`W = 10`** | `183.4 ms` (`1785.6 ms`) | `0.897×` (`0.892×`) | `0.929×` (`0.929×`) | **`166.4 ms` (`0.908×` / `0.906×`)** | **`166.9 ms` (`0.910×` / `0.909×`)** |

### 3.4 Pre-Registered Gate G5 Summary
- **`G5.1` Exactness & `#[forbid(unsafe_code)]`:** **`PASS`** (dirty-`out`, poisoned tapes, stale non-`LONG` code slots, `ovf_chunks` spill, exact `0xFFF0+16 == 0x10000` epoch-cut regression, `m == 1` self-loop, 228/228 admitted blocks, and 431-block `BzReader` parallel CRC decode at `W = 8, 10`).
- **`G5.2` Per-Worker Memory (`<= 0.405×` `0.5.8` & Unique L3 `<= 2,048 KiB`):** **`PASS`** (`0.385×` / `1,729.5 KiB` on `SentOutQ16`; `0.400×` / `1,752.0 KiB` on `SentOutQ14`).
- **`G5.3` `chaingrind` Contention Proxy (`DRAM <= 0.22×/0.16×`, `cyc <= 0.925×/0.860×`, `instr <= 0.925×`):** **`PASS`** (`dram8 = 0.200×`, `dram10 = 0.104×`, `cyc8 = 0.924×`, `cyc10 = 0.847×`, `instr = 0.919×` on `SentOutQ16`; `cyc8 = 0.900×`, `cyc10 = 0.831×` on `SentOutQ14`).
- **`G5.4` Multi-Worker Native Speed (`<= 0.920×` at `W = 8, 10`):** **`PASS`** (`0.912×` wall / `0.910×` CPU at `W = 8`; `0.908×` wall / `0.906×` CPU at `W = 10`).

---

## 4. Learn & Next Objective for Cycle 18 (`Whole-Corpus 431-Block Worker Footprint & Admission Gate`)

On the `228` admitted Wikidata blocks, `SentOutQ16` has cut `Open`'s per-worker heap from `4,479.5 KiB` to `1,722.5 KiB` (`-61.5%`) and `W = 10` DRAM traffic by `-89.6%` (`0.359` vs `3.439 B/outB`).

However, in a real `BzReader` parallel stream over all **`431` blocks of `wikidata.bz2`**, `OpenCells` (`Master-0.5.8`) currently refuses **`203` blocks**:
1. **`5` full `900 KB` blocks** with `0.15 <= r/n < 0.20`, and
2. **`198` tiny stream-tail blocks** (`n` of a few hundred bytes) with `64r < n` (refused by the `64r >= n` sparse gate!).

Whenever **any** worker encounters **even one** of those `5` refused `900 KB` blocks (or any sparse block), `OpenCells::lazy` (`LazyCells` -> `Segmented`) lazily allocates **`tt: 3.6 MiB + seg: 2.0 MiB = +5.6 MiB` per worker (`+45..56 MiB` across 8–10 workers!)**, dwarfing the `1.72 MiB` `SentOutQ16` footprint! Cycle 18 will census all `431` blocks of `wikidata.bz2`, port Safe-Rust **Ostrowski / AHT self-overlap & squaring contraction (`r.left == r.right`)** into `SentOutQ16` to safely abolish `64r >= n` and widen admission to `r/n < 0.20`, and measure the resulting **whole-corpus 431-block per-worker heap, `chaingrind` DRAM B/outB, and `W = 8, 10` `BzReader` scaling**.
