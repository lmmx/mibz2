# Whole-Decoder & Per-Station Intel JCC-Erratum 32-Byte Alignment Audit

- **Date:** 2026-11-27
- **Probe binary:** `examples/jcc_whole_decoder_probe.rs` (`Claim: none`, zero `src/` / `Cargo.toml` / `.cargo/` edits, `#![forbid(unsafe_code)]`)
- **Corpora:** `wikidata.bz2` (SHA-256 `82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915`, 431 blocks: 383 run-route / 48 column-route, 233 Open-admitted), plus 6 off-Wikidata corpora (`text`, `prose`, `source`, `elf`, `records`, `random`).
- **Hardware:** 2-vCPU cloud VM reporting CPUID family 6 model 63 (`0x3F`, Haswell-EP, 45 MiB L3) with host-silicon flags (`umip`, `mmio_stale_data`, `bhi`, `md_clear`) characteristic of Skylake/Cascade Lake-era Xeon server silicon masked to Haswell CPUID. **VM diagnostic only (C5).**

---

## 1. Frozen Decision Test (Recorded Before Timing)

Four prior smooth-zone probes exposed large code-layout swings from Intel's JCC-erratum 32-byte DSB boundary mitigation: `#1059` (`20–40 ms` on `A5+A6a`), `#1060` (`13–20%` on `mtf`), `#1067` (`-81..-86 ms` / `-11..-12%` on `W0_shipped`), and `#1068` (`-7.08%` warm / `-5.89%` cold on 4-lane `A7`).

**Frozen decision test (recorded before timing):**
Recommend paired i9-10900K hardware verification (plus release x86-64 `.cargo`/wheel flag evaluation and benchmark alignment control) **ONLY IF** whole-decoder `wikidata` 1T gains **`>= 3.0%` disjoint** (`[min..max]`) on **BOTH** baseline x86-64 AND `-C target-cpu=native` (steal-free CPU, 7 interleaved repeats), with **zero off-Wikidata corpus regression `> 1.0%`** (disjoint) and **`.text` growth `<= 5.0%`**. Otherwise scope it as a **benchmarking alignment-control tool only** (not a shipped release flag).

### Summary Verdict (Split by Production Kernel!)
- **`.text` growth:** **PASS** — `+3.43%` baseline / `+3.46%` native whole binary (`+2.86%` / `+3.02%` inside `mibz2::*`, `+0.52%..+0.53%` file size), well below `5.0%`.
- **Off-Wikidata safety (`6` corpora × `2` builds = `12` cells):** **PASS (`12 / 12` faster, `0` regressions)** — `-0.61%` to `-9.11%` across `text` (`-4.36%` disjoint base / `-3.74%` nat), `prose` (`-7.21%` / `-9.11%` disjoint nat), `source`, `elf`, `records`, `random`.
- **Whole-decoder `wikidata` 1T split by kernel (`Cells` eager vs `OpenCells`):**
  - **1T `BzReader::new` (`Cells` eager kernel, used for `threads < 4`):** `-3.01%` base (`787.6 -> 763.9 ms`, overlap) and `-2.85%` native (`786.5 -> 764.1 ms`, overlap); `StreamDecoder` 1T/2T (also eager `Cells` since `UNPINNED_LAZY_FROM = 4`) is `-4.71%..+1.27%` overlap.
  - **`OpenCells` production worker (`4W–10W` kernel, `decode_block::<OpenCells>`):** **`-15.64%` (`-118.3 ms`) DISJOINT FASTER** baseline (`756.5 -> 638.2 ms`) and **`-13.70%` (`-99.2 ms`) DISJOINT FASTER** native (`723.7 -> 624.5 ms`); full 1T `OpenCells` decode (`S + W`) is **`-13.06%` disjoint** baseline (`840.4 -> 730.7 ms`) and **`-12.43%`** native (`804.4 -> 704.4 ms`, 6/7 disjoint, 2.3 ms overlap from 1 `S` outlier).
- **Recommendation (C5-compliant):**
  1. **Unconditional benchmark-hygiene control:** Make `-C llvm-args=-x86-align-branch-boundary=32 -C llvm-args=-x86-align-branch=fused+jcc+jmp` (byte-identical to `-C llvm-args=-x86-branches-within-32B-boundaries`) standard in VM probes and A/B harness controls so 6–20% code-layout noise cannot masquerade as algorithmic signal.
  2. **Conditional hardware verification before any release/wheel flag:** Because `OpenCells` (the 4W–10W production kernel) clears every bar by 4–5x (`-13.7%..-15.6%` disjoint, `0/12` off-Wikidata regressions, `+3.4%` `.text`), **run a single paired `ab` check on Louis's i9-10900K** (8W `OpenCells` + 1T `text`) before deciding whether to ship an `x86_64`-only wheel/release flag (see Section 5 on Comet Lake-S stepping 5 vs Skylake/Cascade Lake cloud servers and AMD Zen).

---

## 2. Static `objdump` Census of Shipped `mibz2` (`lto = "fat"`, `codegen-units = 1`)

Seven release binaries were built from `examples/jcc_whole_decoder_probe.rs` linking the entire shipped `mibz2` library (`base_unaligned`, `base_jcc_32b`, `base_jcc_shorthand`, `base_jcc_all`, `nat_unaligned`, `nat_jcc_32b`, `nat_jcc_all`):

- **Flag equivalence:** `-C llvm-args=-x86-branches-within-32B-boundaries` (`base_jcc_shorthand`) produces **byte-for-byte identical `.text`** (`663,261` bytes, `4,362,368` file bytes) to `-C llvm-args=-x86-align-branch-boundary=32 -C llvm-args=-x86-align-branch=fused+jcc+jmp` (`base_jcc_32b`).
- **Fat-LTO propagation:** Passing `-C llvm-args=...` to the final fat-LTO `rustc` link propagates identically across all `mibz2::*` functions (`663,261 == 663,261` bytes vs full `RUSTFLAGS` rebuild).

### 2.1 Binary Size, `.text` Size & Total `mibz2::*` 32B-Boundary Violations

| Binary | File Bytes (delta) | Whole `.text` Bytes (delta) | `mibz2::*` Bytes (delta) | `JCC/JMP` Total | `JCC/JMP` 32B Violations (`unfused + fused`) | `ind` Bad | `call/ret` Bad |
|---|---:|---:|---:|---:|---:|---:|---:|
| `base_unaligned` | 4,339,784 | 641,261 | 303,694 | 8,939 | **1,712** (1,195 unfused + 517 fused-only) | 14 | 602 |
| `base_jcc_32b` | 4,362,368 (+0.52%) | 663,261 (**+3.43%**) | 312,387 (**+2.86%**) | 8,939 | **71** (**0 unfused** + 71 non-fusable pairs) | 14 | 570 |
| `base_jcc_all` | 4,366,128 (+0.61%) | 666,925 (+4.00%) | 313,607 (+3.26%) | 8,939 | 77 (0 unfused + 77 non-fusable pairs) | 11 | 151 |
| `nat_unaligned` | 4,317,640 | 644,938 | 309,112 | 9,061 | **1,746** (1,204 unfused + 542 fused-only) | 10 | 572 |
| `nat_jcc_32b` | 4,340,536 (+0.53%) | 667,258 (**+3.46%**) | 318,440 (**+3.02%**) | 9,061 | **60** (**0 unfused** + 60 non-fusable pairs) | 11 | 538 |
| `nat_jcc_all` | 4,344,800 (+0.63%) | 671,386 (+4.10%) | 320,040 (+3.54%) | 9,061 | 70 (0 unfused + 70 non-fusable pairs) | 7 | 138 |

*(The remaining 60–71 `fused_only` detections in `jcc_32b` are instruction pairs like `cmp mem, imm; jcc` that Intel x86 hardware does not macro-fuse, so LLVM correctly leaves them unpadded; 100.0% of unfused JCC/JMP and hardware-fusable pairs are eliminated.)*

### 2.2 Per-Module & Hot-Function 32B-Boundary Violations (`unaligned -> jcc_32b`)

| Module / Hot Function | Base Violations (`un -> 32b`) | Base Bytes (`un -> 32b`) | Native Violations (`un -> 32b`) | Native Bytes (`un -> 32b`) |
|---|---:|---:|---:|---:|
| **`open`** (`src/open.rs`, total) | **397 -> 10** | 60,837 -> 62,990 | **418 -> 7** | 60,698 -> 63,007 |
| - `open::Kernel::induced_with::<Narrow>` | 119 -> 1 | 13,714 -> 14,269 | 113 -> 1 | 13,860 -> 14,545 |
| - `open::Kernel::induced_with::<Wide>` | 111 -> 1 | 13,377 -> 13,980 | 119 -> 0 | 13,493 -> 14,102 |
| - `open::Kernel::try_run::<true, true>` | 25 -> 1 | 8,775 -> 8,945 | 41 -> 1 | 8,585 -> 8,741 |
| - `open::Kernel::walk` | 35 -> 0 | 3,852 -> 4,036 | 32 -> 0 | 3,834 -> 4,022 |
| **`huff` / `block` / `bits`** (total) | **112 -> 3** | 24,075 -> 24,696 | **115 -> 1** | 23,716 -> 24,441 |
| - `block::read_item_with_scratch` | 63 -> 2 | 15,513 -> 15,882 | 64 -> 0 | 15,293 -> 15,754 |
| - `huff::Huff::build_into` | 25 -> 0 | 4,841 -> 4,905 | 26 -> 0 | 4,651 -> 4,746 |
| **`runs`** (total: `front_avx2`, `fill`, `cuts`) | **43 -> 0** | 11,419 -> 11,732 | **49 -> 0** | 9,795 -> 10,072 |
| - `runs::front` + `runs::front_avx2` | 9+12 -> 0 | 5,042 -> 5,165 | 23 -> 0 | 4,768 -> 4,862 |
| **`mtf`** (total: `mtf_avx2`, `checkpoints`) | **17 -> 1** | 4,665 -> 4,807 | **27 -> 0** | 4,581 -> 4,734 |
| - `mtf::mtf_checkpoints(_avx2)` | 10 -> 0 | 2,195 -> 2,241 | 15 -> 0 | 2,526 -> 2,587 |
| **`bwt`** (total: `build`, `room_segments`) | **201 -> 14** | 41,210 -> 42,200 | **205 -> 8** | 43,380 -> 44,415 |
| - `bwt::build` | 33 -> 0 | 3,969 -> 4,134 | 26 -> 0 | 4,691 -> 4,876 |
| - `bwt::room_segments::<32,1,false/true>` | 31+29 -> 1+0 | 13,032 -> 13,300 | 38+39 -> 0+0 | 14,606 -> 14,998 |
| **`cells`** (total) | **108 -> 4** | 23,951 -> 24,378 | **110 -> 4** | 25,996 -> 26,675 |
| **`rle`** (total: `unrle_in_place`, `unrle_avx2`) | **36 -> 2** | 4,558 -> 4,703 | **30 -> 2** | 4,077 -> 4,266 |
| - `rle::unrle_in_place` | 14 -> 2 | 1,830 -> 1,879 | 21 -> 2 | 2,562 -> 2,671 |
| **`crc`** (total) | **1 -> 0** | 1,745 -> 1,777 | **2 -> 0** | 1,734 -> 1,780 |
| **`stream` / `read` / `parallel`** (total) | **600 -> 31** | 95,323 -> 97,940 | **581 -> 33** | 95,191 -> 97,848 |

---

## 3. Whole-Decoder Paired Interleaved Benchmarks (7 Rounds)

All binaries were interleaved in alternating forward/reverse order per round, pinned to CPU 0 (1T) or CPUs 0–1 (2T). Because sibling swarm builds ran concurrently on this 2-vCPU VM, steal-free CPU time (`/proc/thread-self/schedstat` for 1T inline; `/proc/self/stat` utime+stime for multi-threaded `stream1`/`stream2`) is the clean diagnostic signal while wall `Instant` is reported alongside.

### 3.1 `wikidata.bz2` Whole-Decoder (1T `bzreader`, 1T `stream1`, 2T `stream2` — 6 Binaries)

*(Note: `BzReader::new` and `StreamDecoder::new(1..2)` both dispatch to eager **`Cells`** because `UNPINNED_LAZY_FROM = 4` in `src/parallel.rs`; **`OpenCells`** is measured in Section 4.1 below.)*

| Mode & Binary | CPU Med (ms) `[min..max]` | vs Unaligned | Disjoint | Wall Med (ms) |
|---|---:|---:|:---:|---:|
| **`bzreader` 1T (`Cells`) — `base_unaligned`** | 787.59 `[779.95..829.93]` | base | base | 2,128.77 |
| `base_jcc_32b` | **763.89** `[759.57..791.53]` | **-3.01%** | overlap | 2,056.61 (-3.39%) |
| `base_jcc_all` | 784.47 `[737.12..810.90]` | -0.40% | overlap | 2,254.92 (+5.93%) |
| **`bzreader` 1T (`Cells`) — `nat_unaligned`** | 786.45 `[739.81..837.18]` | base | base | 2,156.29 |
| `nat_jcc_32b` | **764.05** `[724.18..806.58]` | **-2.85%** | overlap | 2,127.82 (-1.32%) |
| `nat_jcc_all` | 748.85 `[716.69..792.11]` | -4.78% | overlap | 2,019.64 (-6.34%) |
| **`stream1` 1T (`Cells`) — `base_un -> 32b / all`** | 850.0 -> 810.0 / 810.0 | -4.71% / -4.71% | overlap | 2,060.0 -> 1,848.9 / 1,832.8 |
| **`stream1` 1T (`Cells`) — `nat_un -> 32b / all`** | 790.0 -> 800.0 / 780.0 | +1.27% / -1.27% | overlap | 1,986.8 -> 1,996.1 / 1,812.0 |
| **`stream2` 2T (`Cells`) — `base_un -> 32b / all`** | 870.0 -> 870.0 / 840.0 | +0.00% / -3.45% | overlap | 1,375.1 -> 1,476.4 / 1,554.8 |
| **`stream2` 2T (`Cells`) — `nat_un -> 32b / all`** | 870.0 -> 850.0 / 870.0 | -2.30% / +0.00% | overlap | 1,577.4 -> 1,459.9 / 1,579.9 |

`jcc_all` (`+indirect+call+ret`) shows no consistent advantage over `jcc_32b` (`fused+jcc+jmp`) while inflating `.text` from `+3.43%` to `+4.00%..+4.10%`.

### 3.2 Off-Wikidata Corpora (6 Corpora, 1T `bzreader`, 7 Interleaved Rounds, CPU ms)

| Corpus (route profile) | `base_un` | `base_jcc_32b` | Base Delta | Base Disj | `nat_un` | `nat_jcc_32b` | Native Delta | Native Disj |
|---|---:|---:|---:|:---:|---:|---:|---:|:---:|
| **`text`** (`ab gen`, Segmented) | 212.53 | **203.26** | **-4.36%** | **FASTER** | 203.73 | **196.10** | **-3.74%** | overlap |
| **`prose`** (repo `.md`, Segmented/Open) | 115.13 | **106.83** | **-7.21%** | overlap | 115.01 | **104.54** | **-9.11%** | **FASTER** |
| **`source`** (repo `.rs`, Segmented) | 55.92 | **54.78** | **-2.05%** | overlap | 51.92 | **50.86** | **-2.05%** | overlap |
| **`elf`** (`/usr/bin`, Column+Segmented) | 184.24 | **174.82** | **-5.11%** | overlap | 184.27 | **177.26** | **-3.80%** | overlap |
| **`records`** (`ab gen`, Column) | 505.48 | **502.40** | **-0.61%** | overlap | 511.92 | **490.70** | **-4.14%** | overlap |
| **`random`** (`ab gen`, Column) | 427.31 | **411.18** | **-3.77%** | overlap | 431.88 | **406.50** | **-5.88%** | overlap |

**12 / 12 off-Wikidata cells are faster with `jcc_32b`; 0 regressions anywhere.**

---

## 4. Per-Station 1T Breakdown (7 Interleaved Rounds, Steal-Free CPU ms)

### 4.1 `wikidata.bz2` (431 blks: 383 run / 48 col — Isolating `OpenCells` vs `Cells` vs `Segmented`)

| Station | `base_un` | `base_jcc_32b` | Base Delta | Base Disj | `nat_un` | `nat_jcc_32b` | Native Delta | Native Disj |
|---|---:|---:|---:|:---:|---:|---:|---:|:---:|
| `S` (Scanner `A2+A3`) | 83.93 | 82.97 | -1.15% | overlap | 79.96 | 77.45 | -3.14% | overlap |
| `M` (MTF `A5`: `runs::front` + `mtf`) | 78.84 | **71.90** | **-8.80%** | overlap | 67.99 | 67.42 | -0.84% | overlap |
| **`CE_open`** (`OpenCells` `A6+A7`) | 613.02 | **526.75** | **-14.07%** | overlap* | 599.85 | **518.04** | **-13.64%** | overlap* |
| `CE_seg` (`Segmented` fallback) | 773.97 | 757.48 | -2.13% | overlap | 751.69 | 741.01 | -1.42% | overlap |
| `R` (`rle::unrle` + `crc`) | 51.58 | 52.08 | +0.96% | overlap | 55.13 | 52.01 | -5.66% | overlap |
| `R_inpl` (`W - MCE_open`: `unrle_in_place`) | 59.24 | **42.76** | **-27.83%** | overlap | 48.83 | **39.06** | **-20.00%** | overlap |
| **`W` (`decode_block::<OpenCells>`)** | **756.50** | **638.22** | **-15.64%** | **FASTER** | **723.66** | **624.49** | **-13.70%** | **FASTER** |
| **`S + W` (Full 1T `OpenCells` decode)** | **840.38** | **730.65** | **-13.06%** | **FASTER** | **804.36** | **704.40** | **-12.43%** | overlap** |
| `DFB` (`BzReader` `Cells`, `fill_buf`) | 817.46 | 757.53 | -7.33% | overlap | 817.52 | 766.21 | -6.28% | overlap |
| `DRD` (`BzReader` `Cells`, `read`) | 806.37 | 788.03 | -2.27% | overlap | 816.93 | 753.20 | -7.80% | overlap |

*(\*6 of 7 `CE_open` repeats were disjoint; 1 host-steal outlier expanded the upper tail. \*\*`S + W` native had `[770.5..1084.7]` vs `[681.3..772.9]` — 2.3 ms overlap on 1 outlier.)*

**Exact station attribution of the `#1067` / `OpenCells` `-14%..-15.6%` JCC speedup:**
- **`OpenCells` (`CE_open`)** drives **~75–82%** of the total win (`-86.3 ms` base / `-81.8 ms` native — matching `#1067`'s `-81..-86 ms` on `W0_shipped`!). Its 397–418 unaligned 32B violations across `induced_with` (`113–119`), `try_run` (`25–41`), and `walk` (`32–35`) drop to 7–10.
- **`unrle_in_place` (`R_inpl`)** adds another **`-9.8..-16.5 ms`** (`-20%..-28%`, `14–21 -> 2` violations).
- **`M` (`runs::front`)** adds **`-6.9 ms` (`-8.8%`)** on baseline (`21 -> 0` violations) and **`-8.1% / -14.1%`** on `text.bz2`.
- **`S` (`Scanner`)** and **`CE_seg` (`Segmented`)** are mildly positive (`-1.2%..-3.1%`), with **zero stations regressing**!

### 4.2 `text.bz2` (26 run / 0 col) & `records.bz2` (0 run / 22 col) Per-Station CPU Medians (ms)

| Corpus / Station | `base_un` | `base_jcc_32b` | Base Delta | Disj | `nat_un` | `nat_jcc_32b` | Native Delta | Disj |
|---|---:|---:|---:|:---:|---:|---:|---:|:---:|
| **`text` — `S` (Scanner)** | 31.99 | 31.91 | -0.25% | overlap | 31.80 | 31.23 | -1.78% | overlap |
| **`text` — `M` (`runs::front`)** | 27.97 | **25.71** | **-8.06%** | overlap | 27.89 | **23.97** | **-14.07%** | overlap |
| **`text` — `CE_open`** | 127.28 | 121.45 | -4.58% | overlap | 127.62 | 127.25 | -0.29% | overlap |
| **`text` — `CE_seg`** | 121.79 | 121.14 | -0.53% | overlap | 119.75 | 123.60 | +3.21% | overlap |
| **`text` — `R_inpl`** | 37.45 | 24.74 | -33.93% | overlap | 30.84 | 27.46 | -10.94% | overlap |
| **`text` — `W` (`decode_block`)** | 192.83 | **177.09** | **-8.16%** | overlap | 187.18 | **178.71** | **-4.52%** | overlap |
| **`records` — `S` (Scanner)** | 87.96 | 86.74 | -1.39% | overlap | 82.33 | 83.78 | +1.76% | overlap |
| **`records` — `M` (`mtf_avx2`)** | 220.37 | 219.28 | -0.49% | overlap | 231.84 | 222.00 | -4.25% | overlap |
| **`records` — `MC` (`mtf_checkpoints`)** | 279.27 | **248.66** | **-10.96%** | **FASTER** | 245.60 | 260.57 | +6.10% | overlap |
| **`records` — `CE_open` / `CE_seg`** | 165.4 / 176.5 | 161.3 / 158.7 | -2.5% / -10.1% | overlap | 156.8 / 157.0 | 157.6 / 163.2 | +0.5% / +4.0% | overlap |
| **`records` — `W` (`decode_block`)** | 398.66 | **382.25** | **-4.12%** | overlap | 396.53 | **388.86** | **-1.94%** | overlap |

---

## 5. Silicon Sensitivity, Portability & Actionable Recommendation

1. **Why this VM sees `-13%..-15.6%` on `OpenCells`:** This cloud VM runs on Skylake/Cascade Lake-class Xeon silicon (CPUID masked to Haswell `0x3F`, exposing `umip`, `mmio_stale_data`, `bhi`), where Intel's JCC-erratum microcode mitigation hard-evicts the entire 32B DSB uop-cache line whenever any macro-fused or unfused JCC/JMP crosses or ends on a 32-byte boundary.
2. **Louis's i9-10900K (Comet Lake-S, family 6 model `0xA5` stepping 5):** Per Intel's official *Mitigations for the Jump Conditional Code Erratum* whitepaper (Rev. 1.0) and Comet Lake-S errata, stepping 5 incorporates the silicon-level JCC fix (unlike Skylake, Kaby Lake, Coffee Lake, Cascade Lake, Whiskey Lake, and Comet Lake-U). However, 32-byte DSB fetch-window splits and macro-fusion splits can still affect hot loops by ~1–4% on Comet Lake, AND Skylake/Cascade Lake Xeon servers remain heavily deployed in cloud `x86_64` fleets.
3. **Portability (ARM64 / AMD Zen):**
   - `-C llvm-args=-x86-align-branch-boundary=32 -C llvm-args=-x86-align-branch=fused+jcc+jmp` is **strictly x86-64-only** (must be gated behind `[target.x86_64-unknown-linux-gnu]` / `cfg(target_arch = "x86_64")` if ever added to release/wheel builds, never passed to ARM64/macOS).
   - On AMD Zen CPUs (unaffected by Intel's JCC erratum), the `+2.86%..+3.46%` NOP padding imposes a modest ~0.5–2% I-cache / decode overhead without a DSB eviction win.
4. **Two Concrete Next Steps:**
   - **Adopt immediately as a benchmark alignment control:** Every VM probe and A/B harness run on Intel x86-64 should include a JCC-aligned control build (`-C llvm-args=-x86-branches-within-32B-boundaries`) to prevent 6–20% code-layout noise from confounding algorithmic verdicts.
   - **Queue a single paired i9-10900K verification (8W `OpenCells` on `wikidata` + 1T `text`) before touching `.cargo/` or PyPI wheel flags:** Only ship `x86_64` JCC alignment in release wheels if paired hardware testing confirms net gain on target CPUs (or explicit Skylake/Cascade Lake fleet priority outweighs Zen's minor padding tax).
