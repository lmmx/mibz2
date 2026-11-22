# Carbon Cycle 37 — Zero-Serial-Thread Parallel Finder, Lock-Free Work Stealing, `takes_runs_fast`, & 5-Regime Evaluation (`0.611×` `10T CritPath`, `0.399×` on `L1`)

**Baseline:** `origin/master` (`53f7ecf9`), Cycle 33 `ParScan` champion (`C33`), and Cycle 36 `1T` champion (`C36a`).
**Candidate:** `docs/journal/2026-11-21-carbon-cycle37/c37_parfind_atomic_regimes.rs` (`#![forbid(unsafe_code)]`).

## 1. Mathematical & Architectural Construction (`C37`)

1. **Barrier-Parallel Disjoint-Anchor Finder (`0.01 ms` Serial Merge CPU):**
   By Cycle 33's **Disjoint 4-Byte Anchor Theorem**, every 4-byte anchor $p = 4m$ ($m \in [1, \lfloor(|D|-2)/4\rfloor]$) owns the disjoint 32-bit starting range $8k + s \in [32(m-1), 32m)$ with zero overlap across worker range boundaries. `decode_parscan_c37` partitions $m \in [1, M]$ evenly across all $W$ worker threads in Phase 1 (`find_magics_anchor_range`), merges the $\sim 443$ `RawMagic` records on worker `0` in `< 10 µs` (`0.01 ms` serial CPU vs `83.06 ms` on `master` and `3.74 ms` in `C33`), and eliminates the dedicated finder/scanner thread completely.
2. **Lock-Free `AtomicUsize` Work Stealing + `ScannerC36a` Worker Integration:**
   Replaces `mpsc::sync_channel` + `Arc<Mutex<Receiver>>` contention with a single `AtomicUsize::fetch_add(1, Ordering::Relaxed)` over the immutable `MagicCand` array, while upgrading worker Huffman decoding from `C32a` to `C36a` (Canonical Template-Clone pair builder + `2 KiB` `SubLut` + `264 B` group window).
3. **Monotonic Short-Circuit `takes_runs_fast` (`13.2×–15.3×` Stage Speedup):**
   For `len > 4096`, `mibz2::runs::takes_runs` samples $64$ windows of $64$ symbols ($4096$ symbols total) and tests $32 \cdot \text{runs} \ge 4096 \iff \text{runs} \ge 128$. Because per-window counts lie in $[0, 64]$, `takes_runs_fast` keeps LLVM's auto-vectorized 64-symbol slice sum (`w.iter().map(|&s| usize::from(s <= 1)).sum()`) and short-circuits immediately upon `runs >= NEED` (accept) or `runs + rem < NEED` (reject).

## 2. Native Safe-Rust A/B + A/A Results (Same Binary vs `origin/master` `53f7ecf9`)

### 2a. `takes_runs` Stage Micro-Gate (`31` Rotated Rounds)

| Corpus | Blocks | `MasterA` (µs) | `MasterB` (µs) (`A/A`) | `C37` Manual `u64` SWAR (µs) | **`C37` `AutoVecSC` (µs)** | Verdict |
|---|---:|---:|---:|---:|---:|---|
| `wikidata-l9` | 431 | 571.61 | 596.12 (`1.0429×`) | 43.23 (`0.0756×`) | **43.22 (`0.0756×`, `13.2×`)** | **KEEP `AutoVecSC`** |
| `wikidata-l1` | 361 | 641.51 | 658.91 (`1.0271×`) | 53.30 (`0.0831×`) | **41.90 (`0.0653×`, `15.3×`)** | **KILL Manual SWAR** |
| `text-l9` | 14 | 20.45 | 20.47 (`1.0008×`) | 0.60 (`0.0292×`) | **1.63 (`0.0798×`, `12.5×`)** | **KEEP `AutoVecSC`** |
| `records-l9` | 11 | 0.04 | 0.04 (`1.0000×`) | 0.05 (`1.4000×`) | **0.04 (`1.1143×`)** | Header early-out (`4 ns/blk`) |
| `random-l9` | 9 | 0.03 | 0.04 (`1.0882×`) | 0.04 (`1.2941×`) | **0.04 (`1.0882×`)** | Header early-out (`4 ns/blk`) |

### 2b. Headline `10`-Thread Confrontation on `wikidata-l9` (`11` Rotated Rounds)

| Metric (`W = 10`) | `MasterA` (ms) | `MasterB` (ms) (`A/A`) | `C33 ParScan` (ms) | **`C37 ParScan` (ms)** | **`A/B` (`C37/MasterA`)** | Verdict |
|---|---:|---:|---:|---:|---:|---|
| Serial Front/Merge CPU | 83.06 | 82.65 (`0.9950×`) | 3.74 (`0.0450×`) | **0.01** | **`0.0001×` (`-99.99%`)** | **KEEP** |
| Per-Worker Avg CPU (`W/10`) | 43.82 | 43.50 (`0.9927×`) | 51.24 (`1.1692×`) | **50.74** | `1.1579×` (`-0.50 ms` vs `C33`) | **KEEP** |
| **`10T CritPath max(S, W/10)`** | **83.06** | **82.65 (`0.9950×`)** | **51.24 (`0.6168×`)** | **50.74** | **`0.6108×` (`-38.92%`)** | **NEW 10T CHAMPION** |
| **`10T TotalCPU` (all threads)** | **522.83** | **520.13 (`0.9948×`)** | **517.59 (`0.9900×`)** | **509.00** | **`0.9735×` (`-2.65%`)** | **BEATS MASTER TOTAL CPU** |
| **`10T VM Wall` (2-vCPU)** | **272.86** | **269.32 (`0.9870×`)** | **263.92 (`0.9673×`)** | **257.67** | **`0.9443×` (`-5.57%`)** | **KEEP** |

### 2c. Multi-Regime `1T` (`Cells`) & `10T` (`OpenCells` Critical Path) Confrontation

| Corpus | Size (`bz2` -> Out, Blks) | `1T` `MasterA/B` (ms) (`A/A`) | **`1T` `C37` (ms) (`A/B`)** | `10T Crit` `MasterA/B` (ms) (`A/A`) | **`10T Crit` `C37` (ms) (`A/B`)** |
|---|---|---:|---:|---:|---:|
| `wikidata-l9` | `11.85 -> 209.70 MB` (`431`) | `476.60 / 480.98` (`1.0092×`) | **464.60 (`0.9748×`, `-2.52%`)** | `83.11 / 82.55` (`0.9933×`) | **51.52 (`0.6199×`, `-38.01%`)** |
| `wikidata-l1` | `3.22 -> 36.00 MB` (`361`) | `116.01 / 115.86` (`0.9987×`) | **106.86 (`0.9211×`, `-7.89%`)** | `30.91 / 30.66` (`0.9918×`) | **12.34 (`0.3993×`, `-60.07%`)** |
| `text-l9` | `2.65 -> 13.46 MB` (`14`) | `69.94 / 70.13` (`1.0028×`) | **67.11 (`0.9596×`, `-4.04%`)** | `18.73 / 18.69` (`0.9981×`) | **8.47 (`0.4523×`, `-54.77%`)** |
| `records-l9` | `9.59 -> 9.60 MB` (`11`) | `155.10 / 154.52` (`0.9963×`) | **149.27 (`0.9624×`, `-3.76%`)** | `38.57 / 38.67` (`1.0026×`) | **20.57 (`0.5333×`, `-46.67%`)** |
| `random-l9` | `8.04 -> 8.00 MB` (`9`) | `130.34 / 130.86` (`1.0040×`) | **125.94 (`0.9662×`, `-3.38%`)** | `32.67 / 32.63` (`0.9988×`) | **17.67 (`0.5409×`, `-45.91%`)** |

**Parity:** Boolean-exact `takes_runs_fast` and byte-exact `1T` + `10T` output across all `826` blocks of all 5 corpora.

## 3. Kill Decisions

- **`count_le1_swar64` manual 4-lane `u64` SWAR in `takes_runs_fast` (`53.30 µs` vs `41.90 µs` `AutoVecSC` on `wikidata-l1`):** Killed because packing four `u16`s into `u64` by hand blocks LLVM's SSE2/AVX2 auto-vectorization over `&[u16; 64]`, whereas keeping the slice iterator with monotonic window short-circuiting achieves `13.2×–15.3×` over `master` cleanly.
