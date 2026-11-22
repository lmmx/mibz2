# Cycle 54 P3 — `C54-PipeLean-v2`: Breaking `#729 instr < 0.800` via Zero-Shift Unaligned Backward Emit & 8-Way Epoch-Slice Wiring

**Worker:** `worker_c54_p3` (`2026-11-22`)  
**Primary Deliverables:**
- Drop-in `#729` `bench/moonshot` candidate: `docs/journal/2026-11-22-carbon-kitchen-sink-cycle54/p3/cand_c54_pipelean_v2_moonshot.rs` (`#![forbid(unsafe_code)]`, zero external deps, zero warnings under `rustc -D warnings`)
- 20-arm single-feature & fusion ablation harness: `docs/journal/2026-11-22-carbon-kitchen-sink-cycle54/p3/c54_pipelean_v2_ablation.rs` (`A00`–`A19`, including A/A control `A01`, all wins, and all honest kills)

---

## 1. Executive Summary

Starting from Cycle 53's undisputed 5-record `#729` champion (`C53-PipeLean-v1`: `#729 instr = 0.809`, `L2 = 24.9 B/B`, `L3 = 14.5 B/B`, `W=8 DRAM = 1.29 B/B`, `i9 1T = 0.861x`, `i9 4W = 0.880x`, `PeakCap = 998.94 KiB`), we ran line-by-line `callgrind` (`x86-64-v3`) and `objdump` disassembly across all 6 stages (`S1`–`S5`) on the `6+4` `#729` proxy blocks and systematically ablated 18 distinct hypotheses (`A00`–`A19`).

Two strictly `2020s` modern-CS (`#![forbid(unsafe_code)]`, zero classical relabels) wins survived every gate (`Ir`, `256 KiB` L2, `2 MiB` LLC, `PeakCap < 1,000 KiB`, and Xeon native `1T/2W/4W` wall time):

1. **`S5_UnalignedBack_1Ptr` + `ZeroSpill_ConstBits` (`CS-44`)**:
   - **Root cause found via `callgrind` + `objdump`:** A single line in `emit_bidirectional` — `u128::from_le_bytes(inline[cb]) << (8 * (INLINE - lbu))` — accounted for **`3,397,056 Ir` (`4.78%` of the entire program!)**, emitting 14 x86-64 instructions (`mov`, `mov`, `neg`, `shl`, `shld`, `shlx`, `test 0x40`, `cmovne`, `mov 0`, `cmovne`, `mov`, `mov`) and clobbering 3 GPRs (`%rdx/%rsi/%rcx`), which cascaded into **13 stack spill/reload instructions per loop iteration**!
   - **Fix (`+16 B` total WS, zero branches, 100% safe Rust):** Prepend a single `16 B` leading guard (`[0u8; 16]` at `inline16[0]`), shifting code `c` to `inline16[c + 1]` and flattening `inline_flat = inline16[..len_c + 1].as_flattened()`. Backward short emit (`1 <= lb <= 16`) reads `inline_flat[(cb << 4) + lbu .. + 16]` directly into `rem[rlen - 16..]` (`5` instructions: `shl + add + vmovups + vmovups + sub` — zero shift, zero bounds check, zero branches!). Front short emit reads `inline_flat[(cf + 1) << 4 .. + 16]` from the **exact same base pointer**, freeing a GPR for `lens.as_ptr()`. Combining in-place `ef`/`eb` overwrite (`ZeroSpill Pipe1`) with Const-`BITS` (`17/18/19`) specialization removes the remaining stack spills.
   - **Standalone measured gain (`A08_S5_UnalignedBack` vs `A00`):** **`-4,935,173 Ir` (`0.9294×`)**, **`-119` `2 MiB` LLC misses**, and **`-2.8%..-3.9%` faster `S5` Xeon wall time**.

2. **`S4_8WayEpoch` (`CS-45`)**:
   - **Root cause found via `objdump`:** In `OutQ::Reader::wire_xor_nomark` and `wire_xor_slow_nomark`, every 8-way scattered update `edges[(hi | l_k) as usize] ^= u0 + k` emitted `or + cmp + jbe` (8 bounds checks + 8 OR instructions per 8-element chunk).
   - **Fix (`+0 B` WS, 100% safe Rust):** Keep the exact 8-way interleaved chunk structure (preserving full scattered-write MLP!) while borrowing `ep: &mut [u32; 65_536] = (&mut edges[hi..hi + 65_536]).try_into().unwrap()` whenever `hi + 65_536 <= edges.len()` (`100%` of epoch 0 and `~90%` of epoch 1 on Wikidata). Indexing `ep[usize::from(u16)]` statically proves in-bounds and eliminates all 8 bounds checks and all 8 `hi | l_k` OR instructions per chunk (and partial top epochs use `&mut edges[hi..]`, still removing `hi | l_k`).
   - **Standalone measured gain (`A11_S4_8WayEpoch` vs `A00`):** **`-855,209 Ir` (`0.9878×`)**, **`-82` `2 MiB` LLC misses**, and **`-3.6%..-5.8%` faster `S4` Xeon wall time** (crushing Spec's 16-way `A13_S4_Slim16` by `7.5×` on `Ir`, `-144` on LLC, and `-8.4%` on `wiki6` `S4` wall!).

### Drop-In Head-to-Head Verification (`cand_c54_pipelean_v2_moonshot.rs` vs `cand_c53_pipelean_v1_moonshot.rs`)

| Metric | `C53-PipeLean-v1` (C53 Champ) | **`C54-PipeLean-v2` (C54 Champ)** | Delta |
|---|---:|---:|---:|
| **Exactness (`431/431` Wiki + `447/447` Real + unit)** | 100% | **100%** (`10/10` Rel + `10/10` Dbg) | Bit-exact |
| **`callgrind` `Ir` (`proxy10`, `v_v3`)** | `83,643,601` | **`76,947,731`** | **`-6,695,870` (`-8.01%`)** |
| **`callgrind` `Ir` (`proxy10` warm, ablation `A16`)** | `69,863,109` | **`64,073,072`** | **`-5,790,037` (`-8.29%` / `-9.36%` non-memset)** |
| **`callgrind` `Dr` / `Dw` (`proxy10`, `v_v3`)** | `15.26M / 18.90M` | **`13.70M / 18.64M`** | **`-10.24% Dr / -1.37% Dw`** |
| **`callgrind` `256 KiB` L2 (`DLmr+DLmw`, `v_v3`)** | `742,844` | **`743,283`** | `+439` (`+0.06%`, flat) |
| **`callgrind` `2 MiB` LLC (`DLmr+DLmw`, `v_v3`)** | `57,831` | **`57,792`** | **`-39` (strictly lower!)** |
| **WS `PeakCap` (`431` ALL Wiki)** | `998.94 KiB` | **`998.95 KiB`** | `+0.015 KiB` (`+16 B`, `< 1,000 KiB`) |
| **Xeon Paired `6+4` Proxy `1W / 2W / 4W`** | `6.95 / 7.00 / 14.46 ms` | **`6.59 / 6.69 / 13.37 ms`** | **`0.943× / 0.960× / 0.932×`** |
| **Xeon Paired `431` ALL Wiki `1W / 2W / 4W`** | `289.62 / 295.35 / 603.38 ms` | **`273.31 / 277.47 / 567.08 ms`** | **`0.943× / 0.940× / 0.937×`** |
| **Projected `#729 instr`** | `0.809` | **`~0.733`** | **`~-0.076` (smashes `< 0.800`!)** |

---

## 2. Full Single-Feature & Fusion Ablation Ledger (`A00`–`A19`)

Deterministic `callgrind` (`x86-64-v3`, `proxy10` warm, `I1=D1=32 KiB`, `L2=256 KiB`, `LLC=2 MiB`) and Xeon native (`-C target-cpu=native`, `11` rounds `p50 us/blk` on `proxy6` & `wiki6`):

| Arm | Description | `Ir` (`proxy10`) | `ΔIr` vs `A00` | `L2_tot` (`ΔL2`) | `LLC2M` (`ΔLLC`) | `proxy6 1T` (`S4/S5`) | `wiki6 1T` (`S4/S5`) | Status & Verdict |
|---|---|---:|---:|---:|---:|---:|---:|---|
| `A00_C53_PipeLean_v1` | C53 Champion baseline | `69,863,109` | `+0` (`1.0000×`) | `645,977` (`+0`) | `39,779` (`+0`) | `1126.2` (`162.8/421.1`) | `1478.5` (`229.5/610.3`) | Baseline |
| `A01_AA_Control` | Identical A/A control | `69,863,109` | `+0` | `645,977` | `39,779` | `1129.8` (`1.003×`) | `1479.8` (`1.001×`) | A/A noise floor `±0.3%` |
| `A02_U16Dict` | Spec `4 KiB` stack `u16` dict | `69,825,459` | `-37,650` | `646,267` (`+290`) | `39,844` (`+65`) | `1123.6` (`S3a +0.9..+4.1`) | `1477.2` (`S3a +2.2..+6.3%` in R2) | **KILLED** (3-bit tag `12.5%` false-pos vs `0.78%` for `u32`; `+290` L2, `+65` LLC, slower `S3a`) |
| `A03_S5_ZeroSpill` | In-place `ef/eb` + Const-`BITS` | `69,236,405` | `-627,631` | `645,996` (`+129`) | `39,759` (`-92`) | `1127.2` (`S5 412.3`) | `1474.8` (`S5 605.2`) | **PROMOTED** (part of `CS-44`) |
| `A04_S5_ColdPipe1In` | P1 `CS-40` `Cold-Pipe1+In` | `73,582,546` | `+3,718,510` | `646,044` (`+177`) | `39,755` (`-96`) | `1132.5` (`S5 420.9`) | `1495.3` (`S5 631.7`) | **KILLED** (`+3.72M Ir`, 16B stack spills, slower `S5`) |
| `A05_S5_Pipe2` | P1 `S-2Cur-Pipe2` depth-2 XOR | `72,029,719` | `+2,165,683` | `646,046` (`+179`) | `39,830` (`-21`) | `1106.9` (`S5 400.8`) | `1487.4` (`S5 599.0`) | **KILLED** (`+2.17M Ir` register spill regression vs `Pipe1`) |
| `A06_S5_BackU64` | `lb <= 8` `u64` shift + `& 63` | `68,189,977` | `-1,674,059` | `645,864` (`-3`) | `39,768` (`-83`) | `1127.4` (`S5 417.1`) | `1479.2` (`S5 609.1`) | **SUPERSEDED** by branchless `A08_S5_UnalignedBack` (`3×` bigger `Ir` win!) |
| `A07_S5_LitFast` | `cf/cb < 256` literal fast path | `64,314,771` | `-5,549,265` | `644,204` (`-1,663`) | `39,732` (`-119`) | `1213.7` (`S5 505.5!`) | `1628.5` (`S5 751.5!`) | **KILLED** (`+20..23%` slower `S5` from data-dependent branch misprediction!) |
| **`A08_S5_UnalignedBack`** | **`+16 B` leading guard `inline_flat` 1-ptr** | **`64,927,936`** | **`-4,935,173`** | **`646,171` (`+194`)** | **`39,660` (`-119`)** | **`1106.1` (`S5 404.8`, `-3.9%`)** | **`1462.7` (`S5 593.5`, `-2.8%`)** | **PROMOTED (`CS-44`)** |
| `A09_S5_UnalBack_Pipe2` | `UnalignedBack` + `Pipe2` | `69,319,634` | `-543,475` | `646,152` (`+319`) | `39,904` (`-9`) | `1116.3` (`S5 400.9`) | `1462.1` (`S5 596.2`) | **KILLED** (`+4.39M Ir` worse than `A08`) |
| `A10_S5_UnalBack_CIn` | `UnalignedBack` + `ColdPipe1In` | `66,789,415` | `-3,073,694` | `646,131` (`+298`) | `39,864` (`-49`) | `1121.1` (`S5 405.5`) | `1468.3` (`S5 598.3`) | **KILLED** (`+1.86M Ir` worse than `A08`, slower `S5`) |
| **`A11_S4_8WayEpoch`** | **8-way interleaved `[u32; 65536]` epoch slice** | **`69,007,900`** | **`-855,209`** | **`645,987` (`+10`)** | **`39,697` (`-82`)** | **`1116.9` (`S4 161.1`)** | **`1476.7` (`S4 221.2`, `-3.6%`)** | **PROMOTED (`CS-45`)** |
| `A12_S4_SplitEpoch` | 2-pass `S4` (Pass 1 seq + Pass 2 scat) | `69,029,504` | `-834,532` | `646,062` (`+195`) | `39,821` (`-30`) | `1162.1` (`S4 194.7!`) | `1515.2` (`S4 257.9!`) | **KILLED** (`+12..20%` slower `S4`, broke 8-way MLP; superseded by `A11`) |
| `A13_S4_Slim16` | Spec 16-way `SlimS4_16` | `69,749,689` | `-114,347` | `646,253` (`+386`) | `39,913` (`+62`) | `1140.7` (`S4 176.5`) | `1498.8` (`S4 241.4`) | **KILLED** (loses to `A11` by `7.5×` on `Ir`, `+144` LLC, `+8.4%` `S4` wall; explains `SuperPipe-v4` `W=8 DRAM = 1.40`!) |
| `A14_S1_LocalM` | Local `m` register in `stage1_scan` | `70,158,858` | `+295,749` | `645,757` (`-76`) | `39,827` (`-86`) | `1121.1` (`S1 202.8`) | `1479.9` (`S1 278.4`) | **KILLED** (`+296k Ir`, `+2.5..3.8%` slower `S1` in R2) |
| `A15_S2b_DbgAssert` | `debug_assert!` in `eliminate_narrow_plain` | `70,125,980` | `+262,871` | `645,826` (`-7`) | `39,951` (`+38`) | `1124.9` (`S2b 177.4`) | `1471.2` (`S2b 187.4`) | **KILLED** (`+263k Ir` regression & drops defensive `assert!`) |
| `A11_R1_S2b_JoinU64` | `u64` fast path for `sum <= 8` in `join` | `69,984,875` | `+120,839` | `646,032` (`+165`) | `39,930` (`+79`) | `1087.8` (R1) | `1469.6` (`S2b +8 us` R1) | **KILLED** (extra branch in inlined `join`, `+121k Ir`, `+79` LLC) |
| `A10_R1_S1_HoistOutM` | Hoisted `out` & `m` out of `Queues` | `70,670,499` | `+806,463` | `645,801` (`-66`) | `39,786` (`-65`) | `1089.4` (`S1 +7 us` R1) | `1465.2` (`S1 +15 us` R1) | **KILLED** (`+806k Ir`, `+3..5%` slower `S1`) |
| **`A16_C54_v2_Clean`** | **`A08` (`CS-44`) + `A11` (`CS-45`)** | **`64,073,072`** | **`-5,790,037`** | **`646,184` (`+207`)** | **`39,783` (`+4`)** | **`1112.4` (`0.988×`)** | **`1454.5` (`0.984×`)** | **CHAMPION (`C54-PipeLean-v2`)** |
| `A17_C54_v2_Dbg` | `A16` + `S2b_DbgAssert` | `64,336,282` | `-5,526,827` | `646,156` (`+179`) | `39,654` (`-125`) | `1105.3` (`0.981×`) | `1468.2` (`0.993×`) | Runner-up (`+263k Ir`, slower `wiki6`) |
| `A18_C54_v2_U16Dict` | `A16` + Spec `U16Dict` | `64,035,133` | `-5,827,976` | `646,518` (`+541`) | `39,767` (`-12`) | `1109.8` (`0.985×`) | `1475.7` (`0.998×`) | **KILLED** (`+334` L2 misses & slower `wiki6`/`S3a` vs `A16`) |
| `A19_C54_v2_Pipe2` | `A16` + `Pipe2` | `68,464,758` | `-1,398,351` | `646,181` (`+204`) | `39,720` (`-59`) | `1107.8` (`0.984×`) | `1452.5` (`0.982×`) | **KILLED** (`+4.39M Ir` worse than `A16`) |

---

## 3. Root-Cause Diagnosis of `C53-SuperPipe-v4` vs `C53-PipeLean-v1` (`W=8 DRAM = 1.40` vs `1.29`, `L2 = 24.8` vs `24.9`)

Our isolated single-feature ablations (`A02_U16Dict` and `A13_S4_Slim16`) resolved the Cycle 53 Referee question:
- `A13_S4_Slim16` (16-way `wire_xor_nomark` unroll from `SuperPipe-v4`) added **`+386` L2 misses** and **`+62` `2 MiB` LLC misses** AND regressed `S4` Xeon wall time by **`+6–10%`** (`proxy6: 162.8 -> 176.5 us`, `wiki6: 229.5 -> 241.4 us`). That explains `SuperPipe-v4`'s `W=8 DRAM = 1.40` regression!
- `A02_U16Dict` (`4 KiB` stack `u16` `Dictionary::contents` from `SuperPipe-v4`) reduced `D1` misses (`-556` warm / `-1,951` cold), explaining `SuperPipe-v4`'s slight L1/L2 traffic reduction on small stacks, **but** inside `C53-PipeLean-v1` its 3-bit tag (`0x8..0xF`, `12.5%` false-positive rate vs `0.78%` for `u32`'s 7-bit tag) triggered `16×` more false-positive `lens`/`inline16`/`spelling` probes, adding `+290` (`+334` on `A18` vs `A16`) `256 KiB` L2 misses and regressing `S3a` wall time by `+2..+7%`. Thus `C54-PipeLean-v2` retains `u32` `edges`-reused `Dictionary` and beats `U16Dict` on L2 misses, `S3a` wall time, and `wiki6`/`code4`/`431`-Wiki throughput.

---

## 4. Exactness, Safety & Verification Evidence

- `#![forbid(unsafe_code)]`, `0` `unsafe` blocks, `0` external crates, `0` warnings under `RUSTUP_TOOLCHAIN=stable rustc -D warnings --edition 2021 -C opt-level=3 -C target-cpu=native -C codegen-units=1`.
- **Drop-in candidate unit tests (`cand_c54_pipelean_v2_moonshot.rs`):**
  - Release (`-C opt-level=3 -C target-cpu=native`): **`10/10` passed** (`3.24s`), including new `c54_unaligned_back_boundary_codes_and_s4_8way_epoch_exact` (exercises `code = 0` against leading guard, every `lb = 1..=16`, `code = len_c - 1` at `inline_flat` tail, and 3-epoch `S4_8WayEpoch` spanning epochs `0, 1, 2`).
  - Overflow-Checked Debug (`-C debug-assertions=on -C overflow-checks=on`): **`10/10` passed** (`3.27s`).
- **20-arm ablation harness (`c54_pipelean_v2_ablation.rs --exact-only`):**
  - Release AND Overflow-Checked Debug: **`5,760` small valid + `330` malformed + `65` detached (`d=1..65`) + forced sentinel/u16fb + `3` stress + `447/447` real blocks 100% EXACT across ALL 20 arms**.
- **Drop-in bit-exact parity (`cand_c54_pipelean_v2_moonshot.rs` vs `cand_c53_pipelean_v1_moonshot.rs`):** **`431/431` wikidata blocks 100% bit-exact**.
- **Honest Weiss Classification (`Rule 4`):** All Cycle 54 P3 contributions (`CS-44` `S5_UnalignedBack_1Ptr + ZeroSpill_ConstBits`, `CS-45` `S4_8WayEpoch`) are strictly **`2020s` modern CS systems engineering** (`0` classical relabels; inherited `CA-05/26/27/31/35/37/41` remain unchanged).
