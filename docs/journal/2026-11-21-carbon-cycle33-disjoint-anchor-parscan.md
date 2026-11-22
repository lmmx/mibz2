# Carbon Cycle 33 — Disjoint 4-Byte Anchor Magic Finder & Parallel Huffman Decode (`0.629×` 10-Thread Critical Path)

**Baseline:** `origin/master` (`53f7ecf9`) and Cycle 32 serial champion (`ScannerC32a`).
**Candidate:** `docs/journal/2026-11-21-carbon-cycle33/c33_disjoint_anchor_parscan.rs` (`#![forbid(unsafe_code)]`).

## 1. Mathematical Construction: Disjoint 4-Byte Anchor Theorem

On `origin/master` (`53f7ecf9`) at `10` threads, per-worker BWT + front-end work averages `43.95 ms/worker` while the single serial `Scanner` thread takes `82.57 ms` (`1.88×` the worker budget). Even after Cycle 32 reduced serial `scan` to `70.9–74.3 ms`, the sequential scanner remained the hard Amdahl wall at `10` threads.

Prior `examples/par_scan.rs` used `unsafe` AVX2 nibble shuffles (or a 1-byte-stride scalar filter) to locate 48-bit magics. Under `#![forbid(unsafe_code)]`, we proved a 4-byte-stride disjoint anchor theorem:

> **Theorem (Disjoint 4-Byte Anchor Partition for 48-Bit Bit-Aligned Magics):**
> Any 48-bit magic $M \in \{\text{BLOCK\_MAGIC}, \text{END\_MAGIC}\}$ starting at bit $8k + s$ ($0 \le s < 8$) determines all $40$ bits of the $5$ whole bytes `data[k+1 ..= k+5]`. Writing $k = 4q + r'$ ($r' \in \{0,1,2,3\}$), the unique multiple of $4$ inside $\{k+1, k+2, k+3, k+4\}$ is $p = 4(q+1)$, and both $p$ and $p+1 \le k+5$ lie inside the deterministic 5-byte window. Across $2$ magics $\times 8$ shifts $\times 4$ offsets, at most $64$ of $65,536$ 16-bit values occur (`< 0.1%`). Moreover, each anchor $p = 4m$ ($m \ge 1$) uniquely and disjointly covers candidate starts $k \in [p-4, p)$ with zero overlap.

Probing an `8 KiB` compile-time `.rodata` bitmap (`[u64; 1024]`) at stride `4` (unrolled $4\times$ to `16` bytes/iteration in 100% safe Rust) scans all `11.85 MB` of `wikidata.bz2` in **`1.996 ms` (`5.94 GB/s` compressed = `105.1 GB/s` decoded equivalent)** with **`0` false positives** (`431` real blocks + `12` stream ends), moving `read_item_c32a` Huffman decoding onto the `10` parallel workers while the joiner enforces exact chain continuity `end_bit[i] == start_bit[i+1]`.

## 2. Native Safe-Rust A/B + A/A Results (`wikidata.bz2`, `W = 10`, 13 Rotated Rounds)

| Metric (`W = 10`) | `MasterA` (ms) | `MasterB` (ms) (`A/A`) | `C32a` Serial (ms) | **`C33 ParScan` (ms)** | **`A/B` (`C33/MasterA`)** | Verdict |
|---|---:|---:|---:|---:|---:|---|
| Serial FrontCPU (`Scan`/`Find`) | 82.57 | 82.26 (`0.9962×`) | 74.25 (`0.8992×`) | **3.62** | **`0.0438×` (`-95.62%`)** | **KEEP** |
| Per-Worker Avg CPU (`W/10`) | 43.95 | 44.31 (`1.0081×`) | 44.49 (`1.0123×`) | **51.93** | `1.1815×` (`+7.98 ms`) | Expected |
| **`10T CritPath max(S, W/10)`** | **82.57** | **82.26 (`0.9962×`)** | **74.25 (`0.8992×`)** | **51.93** | **`0.6289×` (`-37.11%`)** | **NEW 10T CHAMPION** |
| `10T TotalCPU` (all threads) | 523.18 | 527.21 (`1.0077×`) | 520.52 (`0.9949×`) | **525.66** | `1.0047×` (`+0.47%`) | Neutral (`+2 ms` finder) |
| `10T VM Wall` (2-vCPU) | 272.54 | 273.13 (`1.0022×`) | 267.65 (`0.9820×`) | **268.06** | **`0.9835×` (`-1.65%`)** | **KEEP** |

**Memory Reduction at `10` Threads:** Replacing `40` queued `Symbols { syms: Vec<u16> }` job slots (`40 × 1.8 MB = 72.0 MB` capacity) with `16`-byte `MagicCand` descriptors + `10` worker-local `syms` buffers cuts peak `syms` RSS by **`> 54 MB`** and keeps `syms` warm in each worker's local L2 cache entering `runs::front`.

**Parity:** Exhaustive `32/32` shift $\times$ alignment proof + `431/431` `wikidata.bz2` blocks + `W = 1, 4, 10` byte-exact + truncated/mutated/header error-exact.

## 3. Kill Decisions

- **Killed — Running `ParScan` at `1` thread:** Killed for `1T` because with a single thread (`W = 1`) there is no parallel worker overlap, so `FinderC33` (`+2.0 ms`) is pure overhead; `1T` retains direct `ScannerC32a` (`0.976×`).
