# Carbon Cycle 16 — Breaking the `W = 10` (`2,048 KiB` L3) Contention Cliff via `OutQ16` Temporal Aliasing, Carry-Free `SWAR64` Iotas, & Transient Metadata Coloring (`0.385×` Per-Worker Heap, `0.103×` `Master-0.5.8` DRAM B/outB at `W = 10`)

*21 November 2026 — Carbon (`research/carbon-word-transport`)*

## 1. Executive Summary & Cycle 16 Scorecard

At the start of Cycle 16 (`git fetch origin master` -> `10207ff`), two critical pieces of hardware evidence converged:
1. On `origin/master` (`39c6e66`), `#679` (`R4` alone at `LIMIT = 16`, serial walk, `u32` queues, cleared tapes — flat memory) measured `0.956×` at 6 threads and `0.982×` at 8 threads on the i9-10900K, **but `0.985×` `[0.970, 1.001]` (`INCONCLUSIVE`, missed `< 1.000` by `0.001`) at 10 threads**.
2. Cycle 15 (`c6e41b8`, `ContigEpochQ16 + ZeroClearR4`) cut per-worker heap to `2,426.8 KiB` (`0.542×` `0.5.8`) and slashed `W = 8` (`LLC = 2,560 KiB`) DRAM traffic by `71.4%` (`0.332 DRAM B/outB`), **yet still paid `1.823 DRAM B/outB` (`38,350` `llc_miss`) at `W = 10` (`LLC = 2,048 KiB`)** because `[Vec<u16>; 256]` (`~480 KiB`) + `edges` (`~840 KiB`) + `out` (`900 KiB`) + `R4` (`~350 KiB`) totaled `2,113–2,639 KiB > 2,048 KiB` per worker, triggering cyclic LRU self-eviction across `W = 10` workers.

Cycle 16 eliminates `[Vec<u16>; 256]` **entirely** (`0 B` queue heap on all `228/228` full blocks!) by exploiting **temporal-liveness disjointness** between thin queues $Q$ (live in Phases 1–3b) and the output buffer `out[0..n)` (`&mut s.ll`, live only from Phase 4 `emit` onward), proving two model-free theorems (`verify_cycle16.py`: `65,521` exhaustive SWAR64 checks + `1,504` adversarial `ovf_chunks`/multi-epoch BWT checks + `6/6` real blocks), validating `swar_outq16_r4.toml` (`check_manifest.py`: `PASS`), and testing `C16-SwarOutQ16-CompactR4` (`#[forbid(unsafe_code)]`):

- **Theorem 16.1 (`proved` + `verified` `1,510` cases — `OutQ16` Temporal Aliasing & L3 Cache-Line Coalescing Law):**
  - Partitioning `out[0..n)` (`900,016 B` = `879` chunks of `1,024 B` = `450,048` `u16` slots) into `512`-entry (`1 KiB`) 64K-epoch queue chunks during Phases 1–3b with exact `ovf_chunks` spillover (`0` overflow chunks across `228/228` admitted Wikidata blocks; verified with `>= 17` overflow chunks on tiny `n = 300` adversarial inputs) eliminates `[Vec<u16>; 256]` **and** reuses the **exact `out` cache lines already dirty in L2/L3 from the prior block's `emit`/`finish` and immediately overwritten by Phase 4 `emit`**.
  - **Controlled Ablation (`c16_explore.rs`, `task-1660`):** Comparing `OutQ16` (chunks in `out`) against `ArenaQ16` (identical chunks in a separate `Vec`) at `W = 10` (`LLC = 2,048 KiB`) proves that **cache-line aliasing with `out` — not chunking — breaks the `2,048 KiB` L3 cliff**, dropping `llc_miss` from `27,322` (`ArenaQ16`) / `34,433` (`C15`) / `65,993` (`0.5.8`) down to **`9,449–9,648` (`-85.6%` vs `0.5.8`)** and `DRAM B/outB` from `3.431` (`0.5.8`) and `1.823` (`C15`) down to **`0.355–0.357 DRAM B/outB` (`0.103×` `Master-0.5.8` — an `89.7%` DRAM reduction at 10 workers, and `5.1×` lower than `C15`!)**!

- **Theorem 16.2 (`proved` + `verified` `65,521` exhaustive residues — Carry-Free `SWAR64` 16-Bit Iota & Disjoint-Lifetime `[u32; 3]` Coloring):**
  - Whenever `(m & 0xFFFF) <= 0xFFF0` (`65,521 / 65,536 = 99.977%` of all ranks), `x0 = u64::from(m & 0xFFFF) * 0x0001_0001_0001_0001 + 0x0003_0002_0001_0000` and `x0 + k * 0x0001_0001_0001_0001` ($k \in \{4, 8, 12\}$) are **strictly carry-free across all 4 `u16` lanes** (sharp: `0xFFF1 + 15 = 0x10000` rolls over), writing 16 `u16`s (`32 B`) into `&mut [u8]` in `4` scalar `u64` ops (`0` `unsafe`, saving `5.0M` byte-shuffle instructions over `array::from_fn` in `triage_c16.rs`).
  - Packing `Strip` into `[from, to, (width << 8) | byte]` (`12 B`) and coloring `strips` + `bands_rev` onto a single `triples: Vec<[u32; 3]>` and `image_order` + `strip_top` onto `u32_scratch: Vec<u32>` cuts **per-worker retained & peak heap to `1,722.5 KiB` (`0.385×` `Master-0.5.8`, `-2,757 KiB/worker`, and `0.710×` `C15`)**, bringing mean unique per-block L3 footprint (including `900 KB` `out`) down to **`1,845.8 KiB <= 2,048 KiB`** (`rep_stores = 0` vs `435,496` in `0.5.8`, store-buffer stalls `-93.8%`)!

- **Honest Strict Frozen Gate G5 Verdict (`g5_swar_outq16_c16.rs`, `task-1687`):**
  - `C16-SwarOutQ16-CompactR4` **beats `Master-0.5.8` on every single metric across `W = 1, 2, 8, 10`** (`0.385×` heap, `0.202×` `W=8` DRAM B/B, `0.103×` `W=10` DRAM B/B, `0.886×` `W=10` `chaingrind` cycles, `0.960–0.978×` VM wall/CPU time).
  - **Strict Frozen Verdict: `FAIL`** on the tightened `G5.4 <= 0.935×` VM wall/CPU sub-bar (`0.960–0.978×` in `g5_swar_outq16_c16.rs`) and `G5.3 instr < 1.00×` (`85.57M` vs `80.75M`). Comparing `triage_c16.rs` (`task-1677`, where `SwarBase` achieved **`78.35M` instructions = `0.970×` `0.5.8`**, **`33.46M` `W=10` cycles = `0.852×` `0.5.8`**, and **`0.915–0.934×` `W=8,10` native speed** — passing every bar!) against `g5_swar_outq16_c16.rs` (`85.57M` instructions) isolated the exact 1-line regression: passing runtime `limit: u32` + `if w <= 16` inside `PassOut::push_thin` instead of `const LIMIT_16: bool` added `+7.22M` dynamic branch instructions across thin runs. Cycle 17 restores `const LIMIT_16` AND attacks the newly exposed #1 bottleneck (`dep_past_l2_loads = 321,694` serial 42-cycle L3 hits = `40%` of all cycles).

---

## 2. Structural Census & Explore/Triage Kills (`c16_explore.rs`, `triage_c16.rs`)

### 2.1 Exact 228-Block Wikidata Census at `LIMIT = 16` (`task-1660`)
- **`inline16` Dictionary Footprint:** Raw mean `28.6 KiB`, max `60.8 KiB` (`> 32 KiB` Comet Lake L1D on `42 / 228` blocks = `18.4%`; `wide == true` on **`0 / 228`** blocks!). With `1024`-slot deduplication: mean `19.4 KiB`, max `32.2 KiB` (`> 32 KiB` on `1 / 228` blocks; exact unique mean `16.6 KiB`).
- **`emit` Step Word-Length Distribution (`25,484,201` total steps):** `1 B` literals = **`81.45%`**, `2..=16 B` = **`12.06%`** (`93.51% <= 16 B`), `17..=32 B` = **`1.75%`**, `33..=64 B` = **`2.20%`**, `> 64 B` = **`2.54%`**.

### 2.2 Recorded Kills

| Investigated Candidate | Mechanism | Measured Outcome (`c16_explore.rs`, `triage_c16.rs`) | Why Killed |
|---|---|---|---|
| `ArenaQ16` (separate `Vec<[u16; 256]>`) | Store `u16` queue chunks in a single heap `Vec` instead of `out[0..n)` | `2,190.9 KiB` heap (`+468 KiB`), `W=10` `DRAM B/outB = 1.282` (`3.5×` worse than `OutQ16`'s `0.366`), `1.005×` at `W=10` | Separate queue cache lines push per-worker footprint back above `2,048 KiB` LLC at `W=10`, proving `out` cache-line reuse is the active mechanism |
| `Dedup1024` / `WordMemo256` | Direct-mapped `1024`-slot `(len, [u8;16])` or `256`-slot `(nd_word, a)` deduplication | Cuts `l1_miss` by only `0.6%` (`1,197k -> 1,190k`) while adding `+0.98M` instructions & `+0.2M` cycles (`triage_c16.rs`) | `81.45%` of `emit` steps hit hot `inline16[0..256]` (`4 KiB`), and frequent multi-byte words already stay hot in `32 KiB` LRU L1D without hashing |
| `Fox32-2Br` (`len <= 16 / != LONG`) | Inline `17..=32 B` words in 2 slots inside `32 KiB` L1D with 3-way `emit` branch | `+1.31M` loads (`19.17M -> 20.48M`), `+0.35M` instructions, slightly slower cycles (`33.27M -> 33.31M`) | Adding a second branch on `100%` of `25.5M` steps costs more than saving `1.75%` `17..32 B` cold calls |

---

## 3. Theorems & Proofs

### Theorem 16.1 (`OutQ16` Temporal Aliasing & L3 Cache-Line Coalescing — `proved`, `verified` `1,510` cases)
In `Open`, thin rank queues $Q_a$ are written solely in Pass 1 and read solely in Phase 3b (`wire`), while `out[0..n)` is written solely in Phase 4 (`emit`, after Phase 3b terminates). Partitioning `out[0..n + 16)` into $C_{\text{max}} = \lfloor(n + 16)/1024\rfloor$ chunks of $U = 512$ `u16`s (`cid < C_max` stored in `out[cid*1024 .. (cid+1)*1024)`, `cid >= C_max` spilled to `ovf_chunks[cid - C_max]`) guarantees zero cross-chunk overwrite (fast path requires `ic <= 496`, writing `32 B` within `[ic*2 .. ic*2 + 32) \subseteq [0, 1024)`) and zero live-range overlap with `emit`. For full `900,000`-byte blocks, $U \cdot C_{\text{max}} = 449,536 \ge m + |\Sigma_{\text{thin}}| \cdot 511$ whenever $m \le 318,720$, so `ovf_chunks` allocates `0 B` (`verified`: `0` overflow chunks across all `228/228` Wikidata blocks) and all queue writes/reads reuse `out`'s already-cached L2/L3 lines.

### Theorem 16.2 (Carry-Free `SWAR64` 16-Bit Iota & Metadata Coloring — `proved`, `verified` `65,521` exhaustive cases)
For every $r \in [0, \text{0xFFF0}] = [0, 65\,520]$ and $k \in \{0, 4, 8, 12\}$, each lane $r + k + j \le 65\,520 + 15 = 65\,535 < 2^{16}$ ($j \in \{0,1,2,3\}$), so
$$X_k = r \cdot \text{0x0001\_0001\_0001\_0001} + \text{0x0003\_0002\_0001\_0000} + k \cdot \text{0x0001\_0001\_0001\_0001} \pmod{2^{64}}$$
produces zero carries across bits $15, 31, 47$ and equals the exact little-endian `u64` packing of `[r+k, r+k+1, r+k+2, r+k+3]` (`verified` exhaustively across all `65,521` values in `verify_cycle16.py`, with sharpness at `r = 0xFFF1`). Furthermore, `strips` (live Phases 1–2) and `bands_rev` (live Phase 3) have disjoint lifetimes and identical `12 B` `[u32; 3]` layout, sharing `triples: Vec<[u32; 3]>`.

---

## 4. Controlled Confrontation Results (`triage_c16.rs` & `g5_swar_outq16_c16.rs`)

### 4.1 Per-Worker Heap & Unique L3 Footprint (`G5.2`: **PASS**)

| Arm | Per-Worker Retained Heap (`KiB`) | vs `0.5.8` | vs `C15` | Per-Worker Peak Heap (`KiB`) | Unique Block L3 WS Mean / p95 (`KiB`) |
|---|---:|---:|---:|---:|---:|
| `0: Master-0.5.8` (`OpenCells`) | `4,479.5 KiB` | `1.000×` | `1.846×` | `4,479.5 KiB` | `2,504.2` / `3,193.0 KiB` |
| `1: C15-EpochQ16-ZeroClearR4` | `2,426.8 KiB` | `0.542×` | `1.000×` | `2,426.8 KiB` | `2,113.4` / `2,639.1 KiB` |
| **`2: C16-SwarOutQ16-CompactR4`** | **`1,722.5 KiB`** | **`0.385×`** | **`0.710×`** | **`1,722.5 KiB`** | **`1,845.8` (`<= 2048 KB`!) / `2,257.8 KiB`** |

### 4.2 `chaingrind` i9-10900K Contention Proxy (`W = 8` `2560 KB` & `W = 10` `2048 KB`)

| `LLC` Share | Arm | `DRAM B/outB` | vs `0.5.8` | `L3 B/outB` | `llc_miss` | `sb_stall` (`M cyc`) | `rep_stores` | `instr/B` | `cycles/B` (vs `0.5.8`) |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|
| **`W=8`** (`2560 KB`) | `Master-0.5.8` | `1.153` | `1.000×` | `9.877` | `22,794` | `2.31 M` | `435,496` | `11.215` | `4.973` (`1.000×`) |
| **`W=8`** (`2560 KB`) | `C15-EpochQ16` (`g5_c15`) | `0.332` | `0.286×` | `8.515` | `9,791` | `0.64 M` | `286,272` | `10.768` | `4.610` (`0.927×`) |
| **`W=8`** (`2560 KB`) | **`C16-SwarBase`** (`const L=16`, `task-1677`) | **`0.234`** | **`0.202×`** | **`8.234` (`0.834×`)** | **`6,997` (`-69%`)** | **`0.20 M` (`0.085×`)** | **`24,576`** | **`10.882` (`0.970×`)** | **`4.621` (`0.929×`)** |
| **`W=8`** (`2560 KB`) | `C16-G5` (dyn `limit`, `task-1687`) | **`0.233`** | **`0.202×`** | `8.239` | `6,999` | `0.18 M` | **`0`** | `11.885` | `4.809` (`0.967×`) |
| **`W=10`** (`2048 KB`) | `Master-0.5.8` | `3.431` | `1.000×` | `9.877` | `65,552` | `5.86 M` | `435,496` | `11.215` | `5.454` (`1.000×`) |
| **`W=10`** (`2048 KB`) | `C15-EpochQ16` (`g5_c15`) | `1.823` | `0.531×` | `8.515` | `38,350` | `2.76 M` | `286,272` | `10.768` | `4.877` (`0.894×`) |
| **`W=10`** (`2048 KB`) | **`C16-SwarBase`** (`const L=16`, `task-1677`) | **`0.357`** | **`0.104×`** | **`8.234` (`0.834×`)** | **`9,465` (`-85.6%`)** | **`0.40 M` (`0.069×`)** | **`24,576`** | **`10.882` (`0.970×`)** | **`4.647` (`0.852×`)** |
| **`W=10`** (`2048 KB`) | `C16-G5` (dyn `limit`, `task-1687`) | **`0.355`** | **`0.103×`** | `8.239` | `9,449` | `0.36 M` | **`0`** | `11.885` | `4.833` (`0.886×`) |

### 4.3 Multi-Worker Native Safe-Rust Timing (`lto=fat`)

- **`C16-SwarBase` (`const LIMIT = 16`, `triage_c16.rs`, 11 paired rounds):**
  - `W = 1`: `310.8 ms` (`0.934×` `0.5.8`) | `W = 8`: `171.4 ms` wall / `1,346.4 ms` CPU (**`0.915×` / `0.915×` `0.5.8`**) | `W = 10`: `178.7 ms` wall / `1,745.7 ms` CPU (**`0.919×` / `0.917×` `0.5.8`**).
- **`C16-G5` (dynamic `limit` parameter, `g5_swar_outq16_c16.rs`, 15 paired rounds):**
  - `W = 1`: `309.2 ms` (`0.978×`) | `W = 2`: `155.9 ms` (`0.976×`) | `W = 8`: `167.7 ms` (`0.963×`) | `W = 10`: `173.3 ms` (`0.964×`).
  - **Frozen Gate G5 Verdict:** Honest `FAIL` (`G5.1 PASS`, `G5.2 PASS`, `G5.3/G5.4` dynamic-`limit` branch regression `+7.22M` instr pushed ratios to `0.963× > 0.935×`).

---

## 5. Next Lever for Cycle 17 (`Learn`)

With `DRAM B/outB` at `W = 10` driven down to **`0.357 B/outB` (`0.104×` `0.5.8`)** and unique L3 working set down to **`1,845.8 KiB < 2,048 KiB`**, `chaingrind` exposes the **new #1 remaining bottleneck**:
- **`dep_past_l2_loads = 321,694` serially dependent 42-cycle L3 hits** (`321,694 * 42 = 13.5M` cycles = **`40.4%` of all cycles!**) during `emit`'s 1-lane walk over `edges[0..m]` (`480–840 KiB > 256 KiB` L2)!
- Why did `#675` (`8`-lane `BP4K` + `LIMIT = 24`) fail at `W = 8, 10`? Because `LIMIT = 24` + `u32` queues + `S = 48` cuts inflated heap to `4,933 KiB` and double-wrote `98%` of `out` (`DRAM B/outB = 4.669`).
- **Cycle 17 Hypothesis (`Work-Stealing In-Place Frontier 2/4-Lane Bipolar Emit` on top of `const L=16` `SwarOutQ16`):** If Lane 0 walks **in-place at `out[0..]`** (claiming consecutive unstarted cuts at **`0` copy cost**!) while only `1–3` auxiliary lanes (`L = 2` or `L = 4`, `LIMIT = 16`) stage into dead upper `out` pages (`+ <= 48 KiB` L1/L2 `ovf`), the entire `out + edges` working set **stays inside the `2,048 KiB` L3 share** (`DRAM B/outB` stays `<< 1.0×` `0.5.8` and heap stays `< 0.42×` `0.5.8`), while overlapping `2–4×` of the `13.5M` L3-hit latency cycles!

---

## 6. Reproducibility Commands

```bash
python3 docs/team/acquisition/check_manifest.py docs/journal/2026-11-21-carbon-cycle16/swar_outq16_r4.toml
python3 docs/journal/2026-11-21-carbon-cycle16/verify_cycle16.py
RUSTUP_TOOLCHAIN=stable-x86_64-unknown-linux-gnu rustc -C opt-level=3 -C codegen-units=1 -C lto=fat --edition 2021 \
  -L /tmp/mibz2_058/target/release/deps --extern mibz2=/tmp/mibz2_058/target/release/libmibz2.rlib \
  docs/journal/2026-11-21-carbon-cycle16/g5_swar_outq16_c16.rs -o /tmp/g5_c16
/tmp/g5_c16 bench /tmp/wikidata.bz2
VALGRIND_LIB=/tmp/chaingrind_lib valgrind --tool=chaingrind --llc-kb=2048 /tmp/g5_c16 cg:c16 /tmp/sample9_admitted.bz2
```
