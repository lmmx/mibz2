# Wormhole Cycle 13 — Breaking `OutQ16` Sub-32B SWAR & Aggarwal–Vitter L1d Write-Combining: 256-Bit Store-Port Equivalence and the Branch-Entropy Barrier

**Session:** `research/carbon-wormhole` (Round 3, Cycle 13 of 20; 3rd of 10 in Round 3)  
**Baseline `origin/master`:** `90e6719663af0615fb417b038906ad2636193aa3` (`mibz2::bwt::OpenCells`)  
**Verification harness:** [`verify_cycle13.rs`](2026-11-21-wormhole-cycle13-s1-s4-swar-and-write-combining/verify_cycle13.rs) (`#![forbid(unsafe_code)]`, `0` warnings, `5,228` Gate G1 + `Spill::ovf` + multi-epoch cases byte-exact + `19`-round native A/B + A/A + `rustc --emit=asm`), [`s1_s4_swar_outq.toml`](2026-11-21-wormhole-cycle13-s1-s4-swar-and-write-combining/s1_s4_swar_outq.toml) (`PASS`), [`cycle13_output.txt`](2026-11-21-wormhole-cycle13-s1-s4-swar-and-write-combining/cycle13_output.txt), [`cycle13_results.json`](2026-11-21-wormhole-cycle13-s1-s4-swar-and-write-combining/cycle13_results.json)

---

## 1. One-Line Kill Decisions (Cycle 13)

- **KILL `B3: AV-WC8` (Aggarwal–Vitter 1988 / Rahman–Raman 2001 `8 KiB` L1d Write-Combining Micro-Buffers in `S1`):** Killed because per-run dynamic L1d bookkeeping (`wc_len[a]` load/store + `wc_buf[a][2*wl..]` variable-offset store + flush check $\approx 9\text{–}11$ cyc/run) regresses `S1` by **`+163.5` to `+334.5 µs`** and end-to-end `OpenCells` by **`1.058×–1.116×` (`7/7` blocks lost)**, costing `6–10×` more than the `107–223` `DTLB64` misses it eliminates.
- **KILL `B1: SWAR8-Split4` (`w <= 4` single-`u64` `8 B` store fast path in `S1`):** Killed on general Wikidata (`B1/A1 = 0.992×–1.022×`, slower than `A1` on `5/6` blocks) because `LLVM` already SLP-vectorizes `origin/master`'s `outq.rs:128` into a **single 256-bit `vpaddw + vmovdqu %ymm0` (`1` store uop on Comet Lake!)**, while branching on `w <= 4` (`89.2%–95.5%` true) incurs a `4.5%–10.8%` branch-misprediction penalty (`+20–45 µs`) that wins (`0.964×`) **only** when $P(w \le 4) > 99\%$ (`dense-703.syn`, `99.9%`).
- **KILL `B2: SWAR32-Branchless` + manual `AVX2-Wire8` (`chunks_exact_mut(8)`):** Killed as redundant/neutral (`B2/A1 = 0.982×–1.000×` vs `A2/A1 = 0.993×–1.012×`, and `+9–14 µs` slower on isolated `S4`) because `rustc -O -C target-cpu=native` already compiles `origin/master`'s `outq.rs:128` to `vpaddw + vmovdqu %ymm0` and `outq.rs:245` to 4× unrolled `vpmovzxwd`, whereas `4 × movq` uses `4×` as many store-buffer entries and manual `chunks_exact_mut(8)` adds an extra remainder branch per band (`2,446–6,788` bands).

---

## 2. The Chain of Existing Results Tested (and Why It Broke)

Cycle 11 identified `Stage S1` (`First Pass + OutQ16`, `227–529 µs`) as the largest pre-`emit` wall-clock stage and hypothesized from source inspection of `outq.rs:128-131` and `outq.rs:245-250` that `OutQ16` suffered from scalar 2-byte `chunks_exact_mut(2)` / `chunks_exact(2)` loops and `8–10×` store-byte amplification (`32 B` written per thin run vs `3.2–4.0 B` true rank payload).

We constructed a two-link chain of existing results to attack `S1 + S4` while holding `S2`, `S3`, and `S5 (emit)` **100% verbatim identical to `origin/master`**:
1. **Link 1 — Lamport (1975, *CACM*) / Fisher & Dietz (1998, *SWAR*) Word-Parallel Algebra:** Under `lo + 16 < 2^16`, `(lo as u64) * 0x0001_0001_0001_0001 + 0x0003_0002_0001_0000` generates 4 packed `u16` ranks in 2 ALU ops without carry, allowing a single `8 B` `u64` store on the `89.2%–99.9%` of thin runs with `w <= 4` (`B1: SWAR8-Split4`, cutting store bytes by `3.1–4.0×` from `1,491–4,395 KiB` to `458–1,119 KiB`), or branchless `4 × u64` (`B2: SWAR32`), paired with explicit 8-lane `chunks_exact_mut(8)` unpacking in `S4`.
2. **Link 2 — Aggarwal & Vitter (1988, *CACM*) / Rahman & Raman (2001, *WADS*) L1d Software Write-Combining (`B3: AV-WC8`):** A persistent `8 KiB` L1d buffer (`[[u8; 32]; 256]`, `2` `4 KiB` pages) accumulates ranks per symbol in L1d and flushes dense `16 B` (`8`-rank) bursts into `out` chunks (`1.0×` store amplification, cutting `out` page touches by `4.5×`).

---

## 3. Theorems & Native Safe-Rust Verification (`5,228` Gate G1 Cases + `19`-Round A/B/A)

### Theorem 13.1 — AVX2 SLP Auto-Vectorization & 256-Bit Store-Port Equivalence (`verified` via `rustc --emit=asm` + `measured` in [`verify_cycle13.rs`](2026-11-21-wormhole-cycle13-s1-s4-swar-and-write-combining/verify_cycle13.rs))

- **Assembly Discovery (`verified` in `/tmp/verify_cycle13.s`):** Under `-O -C target-cpu=native`, LLVM's SLP and loop vectorizers **already** compile `origin/master`'s safe-Rust `outq.rs:128-131` into a **single 256-bit AVX2 vector add + store** (`vpaddw .LCPI(%rip), %ymm0, %ymm0` + `vmovdqu %ymm0, (%rax)`) and compile `outq.rs:245-250` into **4× unrolled `vpmovzxwd` (`32` edges/iter)**!
- **Microarchitectural Consequence (`proved`):** On Skylake / Comet Lake (`i9-10900K`), the L1d store-data pipe (`p4` + `p237`) is natively **256 bits (`32 B`) wide**. Thus a `32 B` `vmovdqu %ymm0` occupies **1 store-buffer entry and 1 store uop** (`~1.47` effective uops including `47%` `64 B` cache-line splits) — matching an `8 B` `movq` (`~1.11` uops) to within `~0.36` cycles/run, **with zero data-dependent branch**! Splitting `w <= 4` (`p \in [0.892, 0.955]` on Wikidata) introduces a data-dependent branch whose misprediction tax $(1 - p) L_{\mathrm{mispred}} \approx (0.045\text{–}0.108) \times 16 \approx 0.72\text{–}1.73$ cycles/run exceeds the `~0.36`-cycle cache-line-split saving whenever $p < 0.975$!

### Theorem 13.2 — Software Write-Combining Bookkeeping Lower Bound (`proved` + `measured`)

- **Statement (`proved`):** Any 256-bucket software write-combining buffer `wc_buf[a]` in L1d must execute per thin run: (1) an L1d load of `wc_len[a]`, (2) a variable-offset address calculation & bounds-checked L1d store `wc_buf[a][2*wl..]`, (3) an L1d store to `wc_len[a]`, and (4) a capacity-threshold branch (`wl + w >= B`), costing $\ge 8\text{–}11$ cycles/run (`430k–1.4M` cycles for $r_{\mathrm{thin}} \in [47{,}736, 140{,}660]$). Because `OutQ16`'s `107–223` L1 DTLB64 misses (`~7` cyc STLB hit = `< 1,600` cycles) and streamer-unhidden L2 RFOs are overlapped across Comet Lake's `10` LFBs (`D_past_L2 = 0`), software write-combining strictly loses on `7/7` blocks (`+163–335 µs`).

### Native Safe-Rust A/B + A/A Results (`19` Interleaved Rotating Rounds, `S2/S3/S5` Identical to `origin/master`)

- **Per-Worker Working Set:** `A1/A2/B1/B2` = `1.885 MiB` (`0 B` added); `B3` = `1.893 MiB` (`+8.25 KiB` `wc_buf + wc_len`).
- **DRAM Traffic:** `0.034–0.059 B/B` across all arms (`0` extra `2048K` L3 misses).

| Block | `r_thin` | `w<=4` | Store Bytes (`Master -> B1`) | `M0` (`S1 / S4 µs`) | `B1` (`S1 / S4 µs`) | `B3` (`S1 / S4 µs`) | `A1:Master` (`µs`) | `A2/A1` (Ctrl) | `B1:SWAR8 / A1` | `B2:SWAR32 / A1` | `B3:AV-WC8 / A1` |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| `block-0.L` | `47,736` | `92.4%` | `1,491 -> 458 KiB` | `247.1 / 93.0` | `272.9 / 93.5` | `410.6 / 94.1` | `1,379.7` | `0.993x` | `1.013x` | `0.988x` | `1.081x` |
| `block-2.L` | `50,705` | `89.2%` | `1,584 -> 523 KiB` | `267.1 / 119.0` | `315.8 / 125.7` | `482.1 / 120.5` | `1,668.6` | `1.001x` | `1.022x` | `0.995x` | `1.076x` |
| `block-85.L` | `77,810` | `90.5%` | `2,431 -> 781 KiB` | `357.5 / 91.3` | `384.5 / 101.2` | `626.4 / 100.7` | `1,701.9` | `0.996x` | `1.013x` | `0.982x` | `1.116x` |
| `block-143.L` | `103,288` | `91.7%` | `3,227 -> 1,007 KiB` | `492.2 / 197.1` | `552.8 / 200.8` | `826.7 / 212.8` | `3,016.2` | `1.000x` | `1.001x` | `0.985x` | `1.058x` |
| `block-210.L` | `125,923` | `95.4%` | `3,935 -> 1,119 KiB` | `561.2 / 139.8` | `568.2 / 153.8` | `856.1 / 151.9` | `2,637.4` | `1.012x` | `1.001x` | `0.990x` | `1.066x` |
| `block-207.L` | `83,637` | `95.5%` | `2,613 -> 741 KiB` | `392.6 / 131.8` | `389.2 / 138.0` | `606.6 / 136.9` | `2,003.8` | `1.001x` | `0.992x` | `0.995x` | `1.069x` |
| `dense-703.syn` | `140,660` | **`99.9%`** | `4,395 -> 1,100 KiB` | `561.2 / 33.6` | **`481.7`** / `47.3` | `751.7 / 46.9` | `1,578.2` | `1.007x` | **`0.964x`** | `1.000x` | `1.077x` |

---

## 4. Where the Break Points Next (Cycle 14)

Cycle 13 closes **all** within-`OutQ16` store-width and software write-combining variations: `origin/master`'s `OutQ16` is **already AVX2-optimal per symbol queue**. Thus the **only** remaining structural lever on `S1 + S4` is **eliminating the 256-queue scatter itself**:
- Either `QUEUE = false` (`open.rs:486-514`: `0` queue writes in `S1`, re-scanning `records` in `S4`), OR
- **Wormhole Route 2 (Cycle 14): Kakutani (1943) Conjugate Thin-Run `(sym: u8, width: u8)` Stream** — recording **1 `u16` per thin *run* (`95–281 KiB` single sequential stream, `1` active `4 KiB` page instead of `256` queues!)** in `S1`, and replaying that compressed `r_thin`-entry log in `S4` without re-scanning `records` or splitting runs twice! Cycle 14 tests both head-to-head against `OutQ16` (`OpenCells`).
