# Carbon Cycle 36 — Canonical Template-Clone Pair Construction & Linear `2 KiB` `SubLut` (`0.7962×` `ScanOnly`)

**Baseline:** `origin/master` (`53f7ecf9`) and Cycle 35 champion (`ScannerC35b`).
**Candidate:** `docs/journal/2026-11-21-carbon-cycle36/c36_tplclone_sub16_scan.rs` (`#![forbid(unsafe_code)]`).

## 1. Mathematical & Microarchitectural Construction (`C36a`)

1. **Canonical Template-Clone 2-Symbol Pair Theorem:**
   For any two non-EOB symbols $s_{1,0}, s_1$ sharing the same canonical code length $l_1 \le 10$, their $2^{10 - l_1}$ entries in `Huff::lut` see identical trailing $(10 - l_1)$-bit suffixes $u \ll l_1$ (`((p1 << (10 - l1)) + u) << l1 & 1023 == u << l1`), and therefore share identical `count`, `s2`, `len1`, and `total_bits` across every $u \in [0, \text{end\_at\_len}[10 - l_1] \gg l_1)$ (divisibility is exact because $2^{10 - l_2}$ is a multiple of $2^{l_1}$ whenever $l_2 \le 10 - l_1$). Only `s1 << SYM1` differs by a constant modular offset:
   $$\text{lut}[\text{off}_p + u] \equiv \text{lut}[\text{off}_0 + u] + ((s_1 - s_{1,0}) \ll \text{SYM1}) \pmod{2^{32}}.$$
   `C36a` builds 2-symbol pairs once per code length $l_1$ on its first non-EOB symbol and clones all remaining symbols of length $l_1$ via auto-vectorized `wrapping_add` (`paddd`).
2. **Linear `O(alpha)` `[u16; 1024]` (`2 KiB`) `SubLut` + `SUB_PTR`-First Ordering:**
   Compresses `sub_lut` from `[u32; 2048]` (`8 KiB`) to `[u16; 1024]` (`2 KiB`, `(sym << 6) | zn`, `0xffff` = `BAD`), keeping all 6 group tables (`36 KiB` total) inside `48 KiB` L1d, replaces `C35b`'s nested builder with a single-pass `O(alpha)` builder, and checks `SUB_PTR` (`99.75%` of slow calls) before `EOB_FAST` (`0.25%`).

We also tested `C36b` (inlining `SUB_PTR` directly inside the $5\times$ unrolled `step_win!()` loop).

## 2. Native Safe-Rust A/B + A/A Results (`wikidata.bz2`, Same Binary)

### 2a. `ScanOnly` 5-Arm Sweep (15 Rotated Rounds)

| Arm | `ScanOnly` Median (ms) | `A/B` vs `MasterA` | vs `C35b` (`63.75 ms`) | Verdict |
|---|---:|---:|---:|---|
| `MasterA` (`53f7ecf9`) | 76.68 | `1.0000×` | — | Baseline |
| `MasterB` (`A/A` ctrl) | 76.29 | `0.9950×` | — | Control |
| `C35b` (`FastWin + Sub32`) | 63.75 | `0.8314×` (`-16.86%`) | `1.0000×` | Prior best |
| **`C36a` (`TplClone + Sub16`)** | **61.05** | **`0.7962×` (`-20.38%`)** | **`0.9577×` (`-2.70 ms`)** | **NEW 1T CHAMPION** |
| `C36b` (`+ InlineSub`) | 61.50 | `0.8020×` (`-19.80%`) | `0.9647×` (`-2.25 ms`) | **KILL** |

### 2b. Full `1T` & `10T` Serial-Scanner A/B + A/A on `C36a`

| Benchmark | `MasterA` (ms) | `MasterB` (ms) | `A/A` (`B/A`) | `C36a` (ms) | `A/B` (`C36a/MasterA`) | Verdict |
|---|---:|---:|---:|---:|---:|---|
| **`1T E2E Full` (`Cells`, 9r)** | 479.47 | 479.47 | **`1.0000×`** | **462.88** | **`0.9654×` (`-3.46%`)** | **KEEP** |
| **`1T E2E Full` (`OpenCells`, 9r)** | 473.70 | 473.98 | `1.0006×` | **457.91** | **`0.9667×` (`-3.33%`)** | **KEEP** |
| `10T ScannerCPU` (serial, 11r) | 83.20 | 83.36 | `1.0019×` | **68.52** | **`0.8236×` (`-17.64%`)** | **KEEP** |
| `10T TotalCPU` (all thr, 11r) | 523.90 | 523.65 | `0.9995×` | **506.15** | **`0.9661×` (`-3.39%`)** | **KEEP** |
| `10T VM Wall` (2-vCPU, 11r) | 269.58 | 270.24 | `1.0025×` | **261.08** | **`0.9685×` (`-3.15%`)** | **KEEP** |

**Parity:** `431/431` `wikidata.bz2` blocks symbol-exact + all truncated/mutated error checks exact.

## 3. Kill Decisions

- **`C36b + InlineSub` (`0.8020×` vs `0.7962×` `C36a`):** Killed because expanding `SUB_PTR` inline $5\times$ inside the unrolled `step_win!()` loop increases L1i footprint and GPR pressure (`+0.45 ms`) relative to out-of-line `SUB_PTR`-first `decode_entry_slow_win`.
