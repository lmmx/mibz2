# Carbon Cycle 35 — `264`-Byte Group Input Window & `O(1)` 2-Level Canonical `SubLut` (`0.8315×` `ScanOnly`)

**Baseline:** `origin/master` (`53f7ecf9`) and Cycle 32 serial champion (`ScannerC32a`).
**Candidate:** `docs/journal/2026-11-21-carbon-cycle35/c35_fastwin_sublut_scan.rs` (`#![forbid(unsafe_code)]`).

## 1. Mechanisms (`C35a` and `C35b`)

1. **Bounds-Check-Free `264`-Byte Group Input Window (`g_in: &[u8; 264]`, `C35a`):**
   Every 50-symbol selector group consumes at most $50 \times 20 = 1{,}000$ bits $= 125$ bytes ($\le 255$ bytes including initial refill slack). Checking `lr.pos + 264 <= data.len()` once per 50-symbol group (`99.9%+` hit rate) lets `refill_win` index `&g_in[(self.pos & 255)..][..8]`, removing `data.get` bounds checks, `refill_slow` branches, and slice-length register pressure across all `3.25M` refills in 100% safe Rust.
2. **`O(1)` 2nd-Level Canonical Subtable (`SubLut`, `C35b`):**
   On `wikidata.bz2`, `1.243%` of steps (`170,400` symbols) exceed `LUT_BITS = 10` (`11..=20` bits). Both `origin/master` and `C32a` looped bit-by-bit from `zn = min_len` (`1..2`). `C35b` attaches compact 2nd-level canonical subtables (`SUB_PTR | (d_w << 16) | sub_off`) to `>10`-bit root prefixes, resolving `11..=20` bit codes in a single indexed read with zero `while` loop iterations.

## 2. Native Safe-Rust A/B + A/A Results (`wikidata.bz2`, Same Binary)

### 2a. `ScanOnly` 5-Arm Sweep (15 Rotated Rounds)

| Arm | `ScanOnly` Median (ms) | `A/B` vs `MasterA` | vs `C32a` (`66.51 ms`) | Verdict |
|---|---:|---:|---:|---|
| `MasterA` (`53f7ecf9`) | 76.79 | `1.0000×` | — | Baseline |
| `MasterB` (`A/A` ctrl) | 77.25 | `1.0060×` | — | Control |
| `C32a` (`ShlFast_Reg`) | 66.51 | `0.8661×` (`-13.39%`) | `1.0000×` | Prior best |
| `C35a` (`FastWin + z20`) | 65.40 | `0.8516×` (`-14.84%`) | `0.9833×` (`-1.11 ms`) | Superseded |
| **`C35b` (`FastWin + SubLut`)** | **63.86** | **`0.8315×` (`-16.85%`)** | **`0.9602×` (`-2.65 ms`)** | **NEW 1T CHAMPION** |

### 2b. Full `1T` & `10T` Serial-Scanner A/B + A/A on `C35b`

| Benchmark | `MasterA` (ms) | `MasterB` (ms) | `A/A` (`B/A`) | `C35b` (ms) | `A/B` (`C35b/MasterA`) | Verdict |
|---|---:|---:|---:|---:|---:|---|
| **`1T E2E Full` (`Cells`, 9r)** | 480.24 | 481.30 | `1.0022×` | **466.77** | **`0.9720×` (`-2.80%`)** | **KEEP** |
| **`1T E2E Full` (`OpenCells`, 9r)** | 470.86 | 472.25 | `1.0030×` | **459.60** | **`0.9761×` (`-2.39%`)** | **KEEP** |
| `10T ScannerCPU` (serial, 11r) | 84.33 | 83.85 | `0.9943×` | **72.21** | **`0.8563×` (`-14.37%`)** | **KEEP** |
| `10T VM Wall` (2-vCPU, 11r) | 282.06 | 282.82 | `1.0027×` | **275.22** | **`0.9757×` (`-2.43%`)** | **KEEP** |

**Parity:** `431/431` `wikidata.bz2` blocks symbol-exact + all truncated/mutated error checks exact.

## 3. Kill Decisions

- **`C35a` `z20`-loop without `SubLut` (`0.8516×` vs `0.8315×` `C35b`):** Superseded immediately because even starting at `zn = 11` with a single 20-bit peek still incurs data-dependent `while z20 > limit20[zn]` branch mispredictions across `170,400` `>10`-bit symbols (`+1.54 ms` slower than `O(1)` `SubLut`).
