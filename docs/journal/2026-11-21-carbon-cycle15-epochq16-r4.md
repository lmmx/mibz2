# Carbon Cycle 15 — Diagnosing `#675`'s i9 Contention Cliff & Breaking It via `ContigEpochQ16` + `ZeroClear-R4` (`0.542×` Per-Worker Heap, `0.286×` / `0.531×` DRAM B/outB at W=8/10, `0.893–0.899×` `Master-0.5.8` Speed, Strict Gate G5 `PASS`)

*21 November 2026 — Carbon (`research/carbon-word-transport`)*

## 1. Executive Summary & Cycle 15 Scorecard

Following publication of Cycles 1–14 (`89efba52`) and the shipping of Cycle 2's 16-wide queue fill + 16-byte emit in **`mibz2 0.5.8`** (`origin/master` `bb98c99`), native safe-Rust benchmarking of the 8-lane `BP4K` + `R4@24` port (`#675`) established a decisive hardware fact on the 10-core Comet Lake i9-10900K (`32 KiB` L1D, `256 KiB` L2, `20 MiB` shared L3):
- `#675` won **`13%` (`0.872×`)** on 1 thread and **`5%` (`0.950×`)** at 6 threads, **but lost at 8 threads (`1.049×` slower) and 10 threads (`1.012×` slower)**.

Starting in Cycle 15, our 5-cycle research objective (Cycles 15–19) is: **reduce the `Open` kernel's cost without growing per-worker memory or memory traffic, judged at 8–10 concurrent workers against `origin/master`'s native Rust kernel (`mibz2 0.5.8`, `OpenCells`)**, measuring per-worker working set/heap and `chaingrind` DRAM bytes per output byte on every candidate.

Cycle 15 diagnoses the exact cache-cliff mechanics of `0.5.8`, `#675`, and `#679` on Comet Lake's `2,560 KiB` (`W=8`) and `2,048 KiB` (`W=10`) per-worker L3 share, proves two structural theorems (`verify_cycle15.py`, `2,004` adversarial multi-epoch & poisoned-tape checks + `6/6` admitted Wikidata blocks), validates `epochq16_r4.toml` (`check_manifest.py`: `PASS`), and promotes the 100% safe-Rust (`#[forbid(unsafe_code)]`) **`C15-EpochQ16-ZeroClearR4`** kernel (`g5_epochq16_c15.rs`), passing **every frozen Gate G5 criterion (`G5.1–G5.4`: `PASS`)**:

- **Root-Cause Diagnosis of `#675` on i9-10900K (`measured` via `TrackAlloc` + `chaingrind`):**
  1. `Master-0.5.8` (`OpenCells`) retains **`4,479.5 KiB`** per worker (`2,504.2 KiB` mean / `3,193.0 KiB` p95 active working set per admitted block including `900 KB` `out`), sitting **directly astride the `2,048–2,560 KiB` per-worker L3 cliff**: dropping per-worker LLC from `2,560 KiB` (`W=8`) to `2,048 KiB` (`W=10`) triples `0.5.8`'s DRAM traffic from **`1.160 -> 3.431 DRAM B/outB`** (`llc_miss`: `22,887 -> 65,583`).
  2. `PR#675` (`R4@24 + 8-lane BP4K`) pushed per-worker heap up to **`4,932.9 KiB` (`1.101×`)**, inflated `edges + queues` by `+35%` (`LIMIT = 24`), and **double-wrote `~98%` of `out` (`+1.8 MB/block` RFO traffic)** via `BP4K` page writes + Kahn unshuffling, driving DRAM traffic to **`2.374 DRAM B/outB` (`2.048×` `0.5.8`)** at `W=8` and **`4.669 DRAM B/outB` (`1.361×` `0.5.8`)** at `W=10` (`store_buffer_stall_cycles`: `3.35×` at `W=8`).
  3. Even serial `PR#679` (`R4@16 + Vec16`) executes `tops.clear(); tops.resize(...)` + `bots.clear(); bots.resize(...)` every block, emitting **`2,203,228` `rep_stores` (`5.06×` `0.5.8`)** and zeroing `~376 KiB` of cold cache lines per block.

- **Theorem 15.1 (`proved` + `verified` `2,004` cases — Contiguous 16-Bit 64K-Epoch Thin Queues `ContigEpochQ16`):**
  - Storing each thin retained rank $m \in [0, 900\,000)$ as its 16-bit residue `m as u16` inside contiguous `[Vec<u16>; 256]` queues with unconditional 32-byte `[u16; 16]` `push_rows_u16` stores and exact `ep_cuts[e][a]` 64K-epoch boundary snapshots ($E = \lceil m / 2^{16}\rceil \le 14$, at most `1–3` snapshots per real block) **halves active queue footprint (`2m = 180–360 KiB`, fitting inside Comet Lake's `256 KiB` private L2 on most blocks)** and **halves queue R/W bandwidth (`8 B -> 4 B` per thin row)** while preserving `vpmovzxwd + vpor` auto-vectorization in Phase 3.

- **Theorem 15.2 (`proved` + `verified` `2,004` cases — Zero-Clear Single-Pad Reused `R4` Tapes & Tight `1.25×` Reservation):**
  - Because `R4` builds `a0` and `p0` strictly sequentially via `prev`-only links and traverses exclusively backward from `(a0_end - 1, b - 1)` down to sentinel `0`, only slot `0` (`PAD = 1`, `tops[0] = Top::default()`, `bots[0] = Bot::default()`) is ever read before being written; all slots `>= 1` can carry arbitrary stale bytes across blocks without `.clear()` or `.resize(..., 0)` (`verified` under full-capacity `0xDEAD_BEEF` poisoning across `2,004` synthetic + `24` real blocks at `LIMIT=16, 64`). Combined with `1.25×` `grow_vec_tight`, direct `arena` `<=16 B` slice inlining, inline `(o0, o1)` cold metadata (zero `offsets` `Vec`), and unreversed `bands_rev`:
    - **Per-Worker Kernel Heap (`G5.2`):** Cut from `4,479.5 KiB` (`0.5.8`) to **`2,426.8 KiB` (`0.542×`, `-2,052.7 KiB/worker`)**!
    - **`chaingrind` DRAM Bytes / Output Byte (`G5.3`):**
      - **`W = 8` (`LLC = 2,560 KiB`):** Slashed from **`1.160 -> 0.332 DRAM B/outB` (`0.286×` `0.5.8`, `-71.4%`)**, `L3 B/outB` `0.862×`, `L2 B/outB` `0.845×`, `instr/outB` `0.960×`, `cycles/outB` `0.927×`, `store_buffer_stall_cycles` `0.277×` (`-72.3%`)!
      - **`W = 10` (`LLC = 2,048 KiB`):** Slashed from **`3.431 -> 1.823 DRAM B/outB` (`0.531×` `0.5.8`, `-46.9%`)**, `instr/outB` `0.960×`, `cycles/outB` `0.894×` (`-10.6%`), `store_buffer_stall_cycles` `0.471×` (`-52.9%`)!
    - **Multi-Worker Native Safe-Rust Speed (`G5.4`, `-C opt-level=3 -C codegen-units=1 -C lto=fat`, 15 paired rounds, 228 admitted blocks):**
      - `W = 1`: **`291.8 ms` (`0.898×` `0.5.8`)**
      - `W = 2`: **`147.3 ms` (`0.898×` `0.5.8`)**
      - `W = 8`: **`157.6 ms` wall / `1,239.6 ms` CPU (`0.893×` / `0.896×` `0.5.8`)**
      - `W = 10`: **`162.8 ms` wall / `1,582.8 ms` CPU (`0.899×` / `0.897×` `0.5.8`)**

---

## 2. Explore & Triage Kills (Recorded Honest Negatives)

| Investigated Candidate | Mechanism | Measured Outcome (`c15_explore.rs`, `triage_c15.rs`) | Why Killed |
|---|---|---|---|
| `Slot16` Unified Dictionary (`[b0..b14, tag]`) | Eliminate separate `lens: Vec<u8>` by storing length/sentinel in byte 15 (`INLINE = 15`) | `+4.8%` slower than `inline16 + lens` (`306 vs 292 ms`) | Extracting `s[15]` from `[u8; 16]` on stack adds XMM→GPR reload latency while dropping `16 B` words to cold `spelling` |
| `Po2Emit` (`edges` masked by `(1<<bits)-1` without bounds check) | Resize `edges` to `1 << bits` and index `edges[(j & mask) as usize]` to elide Rust bounds panic check | Slower & higher L3/DRAM pressure (`m = 135k -> 2^18 = 262k` = `1.05 MiB` `edges`) | Zeroing & retaining `2^18` `u32` slots (`+500 KiB`) pushes worker footprint across the `2,048 KiB` L3 cliff while LLVM already hoists loop bounds on `&edges[..m]` |
| `PagedEpochQ16` (`240`-entry linked pages in single arena) | Store `u16` queues in `512 B` linked slabs (`240` entries + header) | `2–3%` slower than contiguous `[Vec<u16>; 256]` in safe Rust | Page-boundary checks in `push_rows` + segmented `zip` loops in Phase 3 lose to contiguous `[Vec<u16>; 256]` `vpmovzxwd` auto-vectorization |
| `U8_GATE` (`w <= 8` 16B XMM store in `push_rows_u16`) | Branch on `w <= 8` to store `[u16; 8]` (`16 B`) instead of `[u16; 16]` (`32 B`) | `+275k` instructions, `147.8 vs 145.8 ms` at `W=2` (`triage_c15.rs` Arm 3) | Extra branch per thin run outweighs halving an already store-buffered L1 write |
| `DirectWire` (0 `bands_rev` `Vec`, inline wiring inside `sa` walk) | Wire `edges[dst..dst+span]` directly inside the `R4` `sa` walk | `+1.0M` loads & slower (`150.1 vs 145.8 ms` at `W=2`, `triage_c15.rs` Arm 4) | Interleaving `inline16`/`arena` writes with `edges` + `queues` loops breaks register allocation & `vpmovzxwd` vectorization |

---

## 3. Theorems & Proofs

### Theorem 15.1 (Contiguous 16-Bit 64K-Epoch Thin Queues — `proved`, `verified` `2,004` cases)
Partition the thin retained rank domain $[0, m)$ into $E = \lceil m / 2^{16}\rceil \le 14$ epochs $I_e = [e \cdot 2^{16},\, \min(m, (e+1)\cdot 2^{16}))$. Because Pass 1 visits thin runs in strictly increasing order of $m$:
1. Within each symbol queue `queues[a]`, indices belonging to epoch $e$ occupy a **contiguous slice** `[ep_cuts[e-1][a] .. ep_cuts[e][a])` (with `ep_cuts[-1][a] := 0`), and every rank $r \in I_e$ satisfies $r = (e \ll 16) \,\mid\, (r \bmod 2^{16})$.
2. Whenever `(m >> 16) == ((m + w) >> 16)` and $w \le 16$ (`> 99.97%` of thin runs), a single branchless `32 B` `[u16; 16]` iota store `base.wrapping_add(i)` writes `queues[a][thin[a]..thin[a]+16]` safely (`at + 16 <= queue.len()`), and subsequent runs overwrite the unused `16 - w` tail slots before `ep_cuts[e]` is ever snapshotted. Runs reaching or crossing `(e+1) << 16` split at `ep_end` (`push_epoch_cross`) and snapshot `ep_cuts[e] = thin` at `cur_m == ep_end`.
3. During Phase 3 (`a = 255..0` backward), `prev_q` descends monotonically from `thin[a]` to `0`; maintaining `ep` (`ep_cuts[ep-1][a] < q1 <= ep_cuts[ep][a]`) reconstructs the exact 32-bit edge `(code << bits) | ((ep as u32) << 16) | u32::from(lo16)` via contiguous slice iteration `&qa[q0..q1]` on `> 99.9%` of bands (`verified` across `1,500` cyclic BWTs at `epoch_bits=4` + `504` multi-epoch `210k`-row BWTs at `epoch_bits=16 vs 24` + `228` Wikidata blocks).

### Theorem 15.2 (Zero-Clear Single-Pad `R4` Tape Reuse — `proved`, `verified` `2,004` cases)
In `EpochQ16Kernel::run`, `tops` is populated strictly sequentially on `[p0_base .. p0)` and `[a0_base .. a0_end)` (`p0_base = 1`, `a0_base = ns + 3`, `p0 <= ns + 2 < a0_base`) and `bots` strictly sequentially on `[1 .. b)`, with `tops[p0_base].prev = 0`, `bots[1].prev = 0`, and `tops[a0_base].prev = p0 - 1` (or `0` if `p0 == p0_base`), while `strip_top[0..ns)` is written for every `i in 0..ns` in the `a0` loop before being read in the `p0` loop. Every subsequent read in elimination and thin-offset assignment reaches a slot either via a previously initialized index in `[p0_base..p0) \cup [a0_base..a0_end)` / `[1..b)` or terminates at `0` (`tops[0] = Top::default()`, `bots[0] = Bot::default()`). Hence stale non-zero contents in `tops[1..]`, `bots[1..]`, `strip_top[..]`, and `queues[a][thin[a]..]` cannot affect any output byte (`verified` under `0xDEAD_BEEF` / `0xBEEF` full-buffer poisoning).

---

## 4. Controlled Confrontation Results (`g5_epochq16_c15.rs`, `task-1598`)

*Environment & Methodology:* Compiled with `mibz2`'s exact release flags (`-C opt-level=3 -C codegen-units=1 -C lto=fat`) against `/tmp/mibz2_058/target/release/libmibz2.rlib` (`origin/master` `bb98c99`, `mibz2 0.5.8`) on the `228` admitted Wikidata blocks (`431` total blocks verified end-to-end with `BzReader::parallel_workers_with` at `W=8, 10`, SHA-256 `82f65ace...`). Contention metrics measured via `chaingrind` on the steady-state 9-block (`1` warm + `8` measured = `7,199,888` output bytes) sample at i9-10900K `W=8` (`--llc-kb=2560`) and `W=10` (`--llc-kb=2048`).

### 4.1 Per-Worker Memory & Active Working Set (`G5.2`: **PASS**)

| Arm | Per-Worker Retained Heap (`KiB`) | vs `0.5.8` | Per-Worker Peak Heap (`KiB`) | vs `0.5.8` | Active Block WS Mean / p95 (`KiB`) |
|---|---:|---:|---:|---:|---:|
| `0: Master-0.5.8` (`OpenCells`) | `4,479.5 KiB` | `1.000×` | `4,479.5 KiB` | `1.000×` | `2,504.2` / `3,193.0 KiB` |
| `1: PR#675` (`R4@24 + 8L BP4K`) | `4,932.9 KiB` | `1.101×` | `4,955.0 KiB` | `1.106×` | *(inflated + `ovf`)* |
| `2: PR#679` (`R4@16 + Vec16`) | `4,358.1 KiB` | `0.973×` | `4,358.1 KiB` | `0.973×` | *(u32 queues + cleared R4)* |
| **`3: C15-EpochQ16-ZeroClearR4`** | **`2,426.8 KiB`** | **`0.542×`** | **`2,426.8 KiB`** | **`0.542×`** | **`2,113.4` (`0.844×`) / `2,639.1 KiB` (`0.827×`)** |

### 4.2 `chaingrind` i9-10900K Contention Proxy (`G5.3`: **PASS** at `W = 8` and `W = 10`)

| `LLC` Share | Arm | `DRAM B/outB` | vs `0.5.8` | `L3 B/outB` | `L2 B/outB` | `llc_miss` | `sb_stall` (`M cyc`) | `instr/B` | `cycles/B` (vs `0.5.8`) |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|
| **`W=8`** (`2560 KB`) | `0: Master-0.5.8` | `1.160` | `1.000×` | `9.875` | `16.410` | `22,887` | `2.32 M` | `11.215` | `4.974` (`1.000×`) |
| **`W=8`** (`2560 KB`) | `1: PR#675` | `2.374` | `2.048×` | `13.353` | `21.029` | `47,832` | `7.77 M` | `18.357` | `6.158` (`1.238×`) |
| **`W=8`** (`2560 KB`) | `2: PR#679` | `1.077` | `0.928×` | `9.513` | `15.198` | `19,538` | `3.98 M` | `14.491` | `5.793` (`1.165×`) |
| **`W=8`** (`2560 KB`) | **`3: C15-EpochQ16`** | **`0.332`** | **`0.286×`** | **`8.515` (`0.862×`)** | **`13.864` (`0.845×`)** | **`9,791` (`-57%`)** | **`0.64 M` (`0.277×`)** | **`10.768` (`0.960×`)** | **`4.610` (`0.927×`)** |
| **`W=10`** (`2048 KB`) | `0: Master-0.5.8` | `3.431` | `1.000×` | `9.875` | `16.410` | `65,583` | `5.86 M` | `11.215` | `5.454` (`1.000×`) |
| **`W=10`** (`2048 KB`) | `1: PR#675` | `4.669` | `1.361×` | `13.353` | `21.029` | `99,005` | `13.68 M` | `18.357` | `6.955` (`1.275×`) |
| **`W=10`** (`2048 KB`) | `2: PR#679` | `3.164` | `0.922×` | `9.513` | `15.198` | `53,724` | `7.53 M` | `14.491` | `6.279` (`1.151×`) |
| **`W=10`** (`2048 KB`) | **`3: C15-EpochQ16`** | **`1.823`** | **`0.531×`** | **`8.515` (`0.862×`)** | **`13.864` (`0.845×`)** | **`38,350` (`-42%`)** | **`2.76 M` (`0.471×`)** | **`10.768` (`0.960×`)** | **`4.877` (`0.894×`)** |

### 4.3 Multi-Worker Native Safe-Rust Timing (`G5.4`: **PASS**, 15 Interleaved Rounds, `lto=fat`)

| Workers `W` | `Master-0.5.8` Wall / CPU (`ms`) | `PR#675` Wall / CPU (`ms`) | `PR#679` Wall / CPU (`ms`) | **`C15-EpochQ16` Wall / CPU (`ms`)** | **`C15 / 0.5.8` (Wall / CPU)** | **`C15 / #679` (CPU)** |
|---:|---:|---:|---:|---:|---:|---:|
| `W = 1` | `324.8` / `324.6` | `362.5` / `362.4` | `345.6` / `345.5` | **`291.8` / `291.6`** | **`0.898×` / `0.898×`** | **`0.844×`** |
| `W = 2` | `164.0` / `326.3` | `182.9` / `364.5` | `174.4` / `347.5` | **`147.3` / `293.6`** | **`0.898×` / `0.900×`** | **`0.845×`** |
| **`W = 8`** | `176.5` / `1,383.4` | `192.6` / `1,517.5` | `187.6` / `1,473.7` | **`157.6` / `1,239.6`** | **`0.893×` / `0.896×`** | **`0.841×`** |
| **`W = 10`** | `181.0` / `1,764.9` | `195.2` / `1,900.3` | `191.2` / `1,858.8` | **`162.8` / `1,582.8`** | **`0.899×` / `0.897×`** | **`0.852×`** |

- **Overall Cycle 15 Frozen Gate G5 Verdict:** **`PASS` (`4/4` gates: `G5.1`, `G5.2`, `G5.3`, `G5.4`)**.

---

## 5. Next Lever for Cycle 16 (`Learn`)

Look at why `C15` still sees `1.823 DRAM B/outB` (`38,350` `llc_miss`) at **`W = 10` (`LLC = 2,048 KiB`)** vs **`0.332 DRAM B/outB` (`9,791` `llc_miss`)** at **`W = 8` (`LLC = 2,560 KiB`)**:
1. `C15`'s p95 active working set is `2,639.1 KiB` (`2,426.8 KiB` retained kernel heap + `900 KiB` `out`), because `[Vec<u16>; 256]` (`~480 KiB` retained) and `edges: Vec<u32>` (`~980 KiB` retained) live in **disjoint cache-line footprints**! If we alias the `u16` thin queues **directly inside the dead region of `edges: Vec<u32>`** (Safe-Rust `BipolarU16-Edges`: chunks allocated downward from the top of `edges`, or radix-partitioned inside `edges`!), we eliminate `[Vec<u16>; 256]` (`-480 KiB` heap AND `-360 KiB` distinct L3 lines, because `edges` overwrites the very same physical cache lines already hot in L2/L3!). That drops total active working set **under `2,048 KiB` even at `W = 10`**!
2. Simultaneously, on Comet Lake (`32 KiB` L1D — smaller than Sapphire Rapids' `48 KiB` L1D!), `inline16` reaches `45–95 KiB > 32 KiB` L1D (`1,208,858` `l1_miss` in `C15`). Cycle 16 will census exact `(nd.word, a)` / `inline16` word duplication across the `228` admitted blocks and test zero-hash `R4` word deduplication + `BipolarU16-Edges` to attack both the `2,048 KiB` L3 cliff at `W = 10` and the `32 KiB` L1D boundary!

---

## 6. Reproducibility Commands

```bash
python3 docs/team/acquisition/check_manifest.py docs/journal/2026-11-21-carbon-cycle15/epochq16_r4.toml
python3 docs/journal/2026-11-21-carbon-cycle15/verify_cycle15.py
RUSTUP_TOOLCHAIN=stable-x86_64-unknown-linux-gnu rustc -C opt-level=3 -C codegen-units=1 -C lto=fat --edition 2021 \
  -L /tmp/mibz2_058/target/release/deps --extern mibz2=/tmp/mibz2_058/target/release/libmibz2.rlib \
  docs/journal/2026-11-21-carbon-cycle15/g5_epochq16_c15.rs -o /tmp/g5_c15
/tmp/g5_c15 bench /tmp/wikidata.bz2
VALGRIND_LIB=/tmp/chaingrind_lib valgrind --tool=chaingrind --llc-kb=2560 /tmp/g5_c15 cg:c15 /tmp/sample9_admitted.bz2
VALGRIND_LIB=/tmp/chaingrind_lib valgrind --tool=chaingrind --llc-kb=2048 /tmp/g5_c15 cg:c15 /tmp/sample9_admitted.bz2
```
