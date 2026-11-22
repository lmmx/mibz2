# Carbon Cycle 31 — Huffman Multi-Symbol LUT Width (`10..12` Bits) and Arity (`Pair32` vs `Tri64`) Sweep

**Baseline:** `origin/master` (`53f7ecf9`) and Cycle 30 champion (`ScannerC30`, `LUT_BITS = 10` `u32` 2-symbol).
**Artifact:** `docs/journal/2026-11-21-carbon-cycle31/c31_lut_width_arity_sweep.rs` (`#![forbid(unsafe_code)]`).

## 1. Hypothesis

Because `wikidata.bz2` encodes `24.98M` Huffman symbols at an average of `3.79` bits/symbol (`82.23%` already 2-symbol pairs at `LUT_BITS = 10`), widening the direct LUT to `11` or `12` bits and/or packing up to `3` `u16` symbols into a single `u64` entry (`Tri64`, stored via a coalesced 8-byte write) might reduce the serial bit-buffer $\to$ LUT $\to$ shift dependency chain per decoded symbol.

## 2. Native Safe-Rust A/B + A/A Sweep (`wikidata.bz2`, Same Binary, 13 Rotated Rounds)

| Arm | LUT Entry / Table Size | `ScanOnly` Median (ms) | `A/B` vs `MasterA` | vs `C30` (`65.63 ms`) | Verdict |
|---|---|---:|---:|---:|---|
| `MasterA` (`53f7ecf9`) | `u32 × 1024` (`4 KiB`) | 75.29 | `1.0000×` | `+14.7%` | Baseline |
| `MasterB` (`A/A` ctrl) | `u32 × 1024` (`4 KiB`) | 74.64 | `0.9914×` | — | Control |
| **`C30` (`Lut10 Pair32`)** | **`u32 × 1024` (`4 KiB`)** | **65.63** | **`0.8717×` (`-12.83%`)** | **`1.0000×`** | **CHAMPION** |
| `C31a Pair32_Lut11` | `u32 × 2048` (`8 KiB`) | 67.33 | `0.8942×` (`-10.58%`) | `1.0259×` (`+1.70 ms`) | **KILL** |
| `C31b Tri64_Lut10_S5` | `u64 × 1024` (`8 KiB`) | 74.58 | `0.9906×` (`-0.94%`) | `1.1364×` (`+8.95 ms`) | **KILL** |
| `C31c Tri64_Lut11_S5` | `u64 × 2048` (`16 KiB`) | 80.22 | `1.0654×` (`+6.54%`) | `1.2223×` (`+14.59 ms`) | **KILL** |
| `C31d Tri64_Lut12_S4` | `u64 × 4096` (`32 KiB`) | 94.02 | `1.2488×` (`+24.88%`) | `1.4326×` (`+28.39 ms`) | **KILL** |

Full `1T` & `10T` check on the fastest `C31` arm (`C31a Pair32_Lut11`) confirmed the same ordering (`1T Cells` `0.9880×`, `10T CritPath` `0.9072×`, both behind `C30`'s `0.9823×` and `0.8948×`). All 4 variants passed `431/431` blocks symbol-exact + truncated/mutated error parity.

## 3. One-Line Kill Decisions

- **`C31a Pair32_Lut11` (`0.8942×` vs `0.8717×` `C30`):** Killed because doubling `Huff::lut` from `4 KiB` to `8 KiB` across `1,606` tables costs more in table construction and L1 cache footprint than the marginal pair-rate increase past `82.2%` saves.
- **`C31b Tri64_Lut10_S5` (`0.9906×` vs `0.8717×` `C30`):** Killed because 8-byte `u64` LUT entries double L1 footprint/build cost and force `k <= 35` early exit from the unrolled `step!(true)` loop (`28%` of symbols fall into `step!(false)`).
- **`C31c Tri64_Lut11_S5` (`1.0654×` vs `MasterA`):** Killed because `16 KiB/group` (`96 KiB` across 6 groups) overflows `48 KiB` L1d on 50-symbol selector switches.
- **`C31d Tri64_Lut12_S4` (`1.2488×` vs `MasterA`):** Killed because `32 KiB/group` (`192 KiB` working set) thrashes L1d and quadruples `Huff::build_into` overhead.
