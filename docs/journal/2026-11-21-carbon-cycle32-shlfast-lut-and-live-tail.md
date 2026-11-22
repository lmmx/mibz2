# Carbon Cycle 32 — Hardware-Masked `wrapping_shl` LUT Packing & Zero-`step!(false)` Live Tail

**Baseline:** `origin/master` (`53f7ecf9`) and Cycle 30 champion (`ScannerC30`).
**Candidate:** `docs/journal/2026-11-21-carbon-cycle32/c32_shlfast_reg_scan.rs` (`#![forbid(unsafe_code)]`).

## 1. Mechanism (`C32a ShlFast_Reg`)

Within the Pareto-optimal `4 KiB` (`[u32; 1024]`) per-group LUT established in Cycle 31, we shortened the per-lookup critical chain and removed all conditional-refill steps:

1. **Zero-Mask Bit-Buffer Shift (`buf.wrapping_shl(e)`):** Placed `total_bits` (`1..=10`) in bits `0..5` with bits `4..5 == 0` (`LEN1 = 6`). Because `u64::wrapping_shl(rhs)` masks `rhs` by `& 63` in x86-64 hardware (`shl`/`shlx`), `self.buf = self.buf.wrapping_shl(e)` deletes the `and eax, 31` instruction directly from the serial `lut -> shift` dependency chain (`13.71M` lookups).
2. **Top-Bit Count (`e >> 30`):** Moved `count` (`0, 1, 2`) to bits `30..31`, replacing `(e >> 11) & 3` with a single `shr` (`0` mask instructions).
3. **Single-Mask Contiguous Pair Slice (`&mut g_out[idx..idx + 2]`):** Eliminates the second `(k + 1) & 63` index computation per step.
4. **Two-Stage `peek_live` Group Tail:** Replaces `while k <= GROUP - 2 { step!(false); }` with bounded `lr.refill` + `step_live!()` tail batches (`k > 40` has $\le 8$ steps = at most two `4`-step `56`-bit refills), eliminating `step!(false)` (`if live < 10`) completely.

We also tested `C32b ShlFast_BitPos` (replacing `{pos, buf, live}` with a stateless bit cursor `bp: usize` reloading `u64::from_be_bytes(data[bp>>3..]) << (bp & 7)` each batch).

## 2. Native Safe-Rust A/B + A/A Results (`wikidata.bz2`, Same Binary)

### 2a. `ScanOnly` 5-Arm Sweep (15 Rotated Rounds)

| Arm | `ScanOnly` Median (ms) | `A/B` vs `MasterA` | vs `C30` (`67.39 ms`) | Verdict |
|---|---:|---:|---:|---|
| `MasterA` (`53f7ecf9`) | 77.21 | `1.0000×` | — | Baseline |
| `MasterB` (`A/A` ctrl) | 77.60 | `1.0051×` | — | Control |
| `C30` (`Lut10 Pair32`) | 67.39 | `0.8729×` (`-12.71%`) | `1.0000×` | Prior best |
| **`C32a ShlFast_Reg`** | **65.73** | **`0.8514×` (`-14.86%`)** | **`0.9754×` (`-1.66 ms`)** | **NEW CHAMPION** |
| `C32b ShlFast_BitPos` | 72.23 | `0.9356×` (`-6.44%`) | `1.0718×` (`+4.84 ms`) | **KILL** |

### 2b. Full `1T` & `10T` A/B + A/A on `C32a ShlFast_Reg`

| Benchmark | `MasterA` (ms) | `MasterB` (ms) | `A/A` (`B/A`) | `C32a` (ms) | `A/B` (`C32a/MasterA`) | Verdict |
|---|---:|---:|---:|---:|---:|---|
| `1T E2E Full` (`Cells`, 9r) | 492.93 | 494.69 | `1.0036×` | **481.31** | **`0.9764×` (`-2.36%`)** | **KEEP** |
| `1T E2E Full` (`OpenCells`, 9r) | 468.75 | 470.45 | `1.0036×` | **457.28** | **`0.9755×` (`-2.45%`)** | **KEEP** |
| `10T ScannerCPU` (serial, 11r) | 82.41 | 82.00 | `0.9950×` | **70.86** | **`0.8598×` (`-14.02%`)** | **KEEP** |
| `10T CritPath max(S, W/10)` (11r) | 82.41 | 82.00 | `0.9950×` | **70.86** | **`0.8598×` (`-14.02%`)** | **KEEP** |
| `10T TotalCPU` (all thr, 11r) | 527.97 | 529.56 | `1.0030×` | **519.35** | **`0.9837×` (`-1.63%`)** | **KEEP** |
| `10T VM Wall` (2-vCPU, 11r) | 276.76 | 275.88 | `0.9968×` | **268.18** | **`0.9690×` (`-3.10%`)** | **KEEP** |

**Parity:** `431/431` `wikidata.bz2` blocks symbol-exact + all truncated/mutated error checks exact.

## 3. Kill Decisions

- **`C32b ShlFast_BitPos` (`0.9356×` `ScanOnly` vs `0.8514×` `C32a`):** Killed because stateless `refill_bp` places an unaligned load + `bswap` + variable `<< (bp & 7)` shift directly on the critical path before step 1 of every batch, whereas `Reg::refill` (`buf |= v >> live`) leaves existing high bits immediately consumable in parallel.
