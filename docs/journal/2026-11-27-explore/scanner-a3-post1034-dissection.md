# Post-`#1034` `A3` (`decode_runs_bmi2` / `decode_runs_scalar`) Dissection & Redesign Probe

*Smooth `explore` session (27 November 2026) starting at `origin/master` `d2e5f463`. Zero `src/` changes. Harness: [`examples/scanner_a3_post1034_dissection.rs`](../../../examples/scanner_a3_post1034_dissection.rs).*

## 1. Context & Disassembly Audit of Shipped `#1034` `A3`

After `#1034` (`scanner-coalesced-u8-decode`), serial scan `S'` is ~`145 ms` on i9-10900K. With `#1069` (`scanner-a2-fast-build`, `−7..10 ms`) and `#1141` (`scanner-adaptive-seam-window`, `−10 ms`), **`A3` (`decode_runs_bmi2` / `decode_runs_scalar` + `Huff::decode_run`, ~`115 ms`, >85% of remaining `S'`)** is the #1 remaining serial bottleneck in the decoder.

Prior dead-ends audited (`frontier/dead-ends.tsv`) and not repeated:
- `#986` (`multi-symbol-huffman-table`): `B = 10`, `K = 3` writing 3× `u16` per lookup (`+31%` store/extract uops, uncoalesced group tails) and `K = 4` in 8B (`48 KiB > 32 KiB` L1d spill).
- `#1034` negative arms: inlining `SUB_PTR` (`+3.9..6.1 ms`), masked `refill_win` alone (`+0.5..1.0%`), `u16`-store `K = 3` even with `M <= 8` coalescing (`+5.7..6.0%` vs `K = 2`).

### Exact `objdump -d` Audit of Shipped `decode_runs_bmi2` (`d2e5f463`)
Inspecting `_RNvNtCsftzkLE3znye_5mibz25block16decode_runs_bmi2` in `target/release/examples/phases`:
- **Hot 5-step batch (`0x3e2b0..0x3e435`):** **102 instructions** for 5 lookups (**20.4 instructions/lookup**):
  - **Per step (15 instructions × 5 = 75 instructions):**
    - Critical chain (`7` cycles on Haswell/Comet Lake): `mov + shr $0x36` (1 cyc) -> `mov (%rdi,%rax,4), %edx` (5 cyc indexed L1d load) -> `shlx %rdx, %rbx, %rbx` (1 cyc).
    - Branch (`3` inst): `mov %edx, %eax; shr $0x1e, %eax; je slow` (22 total shifts + 9 branches per 5 lookups saturating Intel P0/P6).
    - Live tracking (`2` inst): `movzbl %dl, %ecx; sub %ecx, %r13d` on every single step.
    - Dual `u16` output stores (`5` inst, 2 stores, partial-register `%dh` read): `movzbl %dh, %ecx; mov %cx, (%r12,%r15,2); shr $0x10, %edx; movzbl %dl, %ecx; mov %cx, 2(%r12,%r15,2)`.
    - Counter (`1` inst): `add %rax, %r15`.
  - **Per-5-lookup batch/refill wrapper (27 instructions):**
    - `[u16; 18]` batch bounds check (`lea 0x12(%r14); cmp $0x413; jae panic`) + register shuffle (`mov %r12, %rsi` / `mov %rsi, %r12`).
    - Two `data.get(pos..pos+8)` bounds checks (`cmp $-9; ja` + `lea 8(%r12); cmp %r11; ja`) despite `decode_runs_inner` having already proven `pos + rem * 125 + 16 <= data_len`!
    - **Critical-path refill stall (+3 cycles every 5 lookups):** Step 4's `movzbl` (`T+1`) -> `sub %ecx, %r13d` (`T+2`) -> `shrx %rdx, %rax, %rbp` (`T+3`) -> `or %rbx, %rbp` (`T+4`) delays Step 5's `shr $0x36` by 3 extra cycles beyond Step 4's `shlx` (`T+1`).
    - **Spilled loop bound:** `cmp 0xa8(%rsp), %r14` reloads `stop_ck` from the stack every 5 lookups.
    - **JCC 32B boundary crossings:** 2 crossings inside `0x3e2b0..0x3e435` (`0x3e2bb..0x3e2c0 jae` and `0x3e37b..0x3e380 je`).
- **Struct & cache layout:** `ScanScratch` stores `[Huff; 6]` (`6 × 6,856 B = 41.1 KiB`), interleaving each hot `4 KiB` `lut` with `2.8 KiB` of cold `sub_lut`/`limit`/`base`/`perm`, plus `chunk: [u16; 1042]` (`2,084 B`).

---

## 2. Frozen Pre-Data Stop Test (C1a / C2 — Frozen Before Any Candidate Timing)

- **Primary corpus:** `wikidata.bz2` (431 blocks, 24,979,299 symbols, 11,854,933 B compressed, SHA-256 `82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915`).
- **Guard corpora (5):** deterministic `ab gen` corpora (`text`, `records`, `random`, `runs`, `onebyte`, level 9).
- **Timing protocol:** 7 interleaved rotated repeats per build on pinned CPU, reporting exact thread on-CPU time (`/proc/thread-self/schedstat` field 1) and wall time across:
  1. **Baseline x86-64** (default `--release`, runtime BMI2 dispatch matching shipped `src/block.rs`),
  2. **Baseline + JCC-32B** (`RUSTFLAGS="-C llvm-args=-x86-branches-within-32B-boundaries"`),
  3. **Native** (`RUSTFLAGS="-C target-cpu=native"`),
  4. **Native + JCC-32B** (`RUSTFLAGS="-C target-cpu=native -C llvm-args=-x86-branches-within-32B-boundaries"`).
- **Exactness requirement:** 100% symbol-for-symbol and ending-bit-position parity against shipped `mibz2::block::read_item_with_scratch` on all 431 `wikidata` blocks, all blocks of the 5 guard corpora, and synthetic `alpha = 258` edge cases (`sym == 256`, `EOB == 257`, `len == 12` `SubLut`, `len == 20` non-`SubLut`, run-boundary overshoot, and error parity).
- **Verdict rule (frozen before data):**
  - **PASS** iff at least one exact variant cuts **paid post-`#1034` `A3` (`A2d build_into + A3 decode_runs`)** thread on-CPU time on `wikidata.bz2` by **`>= 5.0%`** (disjoint 7-repeat `min..max` spread vs shipped `#1034` `decode_runs_bmi2` mirror, corresponding to `>= ~5.5–6.0 ms` off i9 serial `S'`) across **both** baseline x86-64 and JCC-32B-aligned baseline builds (and positive/consistent on native + native JCC-32B), **with `<= 1.02×` `ScanScratch` memory (`<= 56.1 KiB`)** and **no `> 2.0%` regression** on any of the 5 guard corpora (`text`, `records`, `random`, `runs`, `onebyte`).
  - **STOP** otherwise; record honest verdict and append scoped dead-end row(s) to `frontier/dead-ends.tsv`.

---

## 3. Six-Corpus Exact Census (`K = 2` vs `K = 3` + `K3_GATE_SEL = 1024`)

From `examples/scanner_a3_post1034_dissection.rs` (`SECTION 0`, exact replay of every block and symbol):

| Corpus | Blocks | Symbols | Blks `< 1024` sel | Syms `< 1024` sel | Shipped `B=10 K=2` lkps/sym | Gated `B=10 K=3` lkps/sym | `K=3` vs `K=2` | `c == 3` share |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| `wikidata` | 431 | 24,979,299 | 45.9% | 0.0% | 0.5469 | **0.4792** | **−12.39%** | 28.6% |
| `text` | 6 | 2,091,030 | 0.0% | 0.0% | 0.5593 | **0.4648** | **−16.88%** | 41.2% |
| `random` | 5 | 3,999,764 | 0.0% | 0.0% | 1.0000 | 1.0000 | +0.00% | 0.0% |
| `records` | 6 | 4,799,968 | 0.0% | 0.0% | 1.0000 | 1.0000 | +0.00% | 0.0% |
| `runs` | 1 | 31,249 | 100.0% | 100.0% | 0.7269 | 0.7269 | +0.00% | 0.0% |
| `onebyte` | 1 | 36 | 100.0% | 100.0% | 1.0000 | 1.0000 | +0.00% | 0.0% |

Key structural findings:
1. **`K3_GATE_SEL = 1024` is free on `wikidata` and `text` and protects tiny blocks:** On `wikidata`, 45.9% of blocks are small runts (`< 1024` selectors) holding `< 0.05%` (`0.0%` rounded) of symbols. Gating Pass-3 `K = 3` construction at `n_sel >= 1024` (keeping Pass-2 `K = 2` at `PAIR_GATE_SEL = 64`) skips Pass-3 on all 198 runts and on tiny 1-block `runs` (`n_sel = 625`, 31k symbols) while preserving 100.0% of the `−12.39%` `wikidata` and `−16.88%` `text` lookup reductions.
2. **Why `sym == 256` matters on `records`:** `wikidata`, `text`, and `runs` emit zero `sym == 256` symbols, whereas `records` emits `41,370` (`random` emits `6,887`). A `u8` chunk buffer that early-flushed on every `sym == 256` would regress `records`; recording rare `sym == 256` offsets in a 32-entry side list (`PatchBuf`, patched during the 1,000-byte bulk widen) keeps `records` fast (`−1.08%..+1.23%`).
3. **Why `B = 9` and `B = 11` fail structurally:** `B = 9` increases lookups on `wikidata` (`+7.8%`) and blows up slow-path frequency on `runs` (`6.98%`, `+20..25%` regression); `B = 11` doubles LUT working set to `48 KiB > 32 KiB` L1d, causing `+21..23%` L1d thrash on `records` and `+4..6%` on `random`.

---

## 4. Four-Build Benchmark Results & Exactness Verification

### Timing & Harness Note
- **Timer:** vDSO `clock_gettime(CLOCK_THREAD_CPUTIME_ID)` (sub-nanosecond thread on-CPU time, avoiding `/proc/thread-self/schedstat`'s 4 ms tick quantization and 862 file-open syscalls per pass). Tiny guard corpora (`runs`, `onebyte`) run `inner_iters = 16` / `1000` per repeat so each sample spans `>= 2 ms`, with medians normalized back to ms/pass.
- **Exactness:** All 9 decode arms (`R0_shipped`, `C1_pipe4_u16`, `C2_u8chunk_s5`, `C3_pipe4_u8`, `C6_k3hi_pipe4`, `C7_k3hi_s5`, `C8_b11_pipe4`, `C9_b9k3_pipe5`, `C10_split_k3_p4`) passed 100% symbol-for-symbol and ending-bit-position parity vs shipped `mibz2::block::read_item_with_scratch` across all 431 `wikidata` blocks, all 5 `ab gen` guard corpora, and synthetic `alpha = 258` (`sym == 256`, `eob == 257`).
- **`ScanScratch` memory:** Shipped `ScanScratch` (`d2e5f463`) is `61,232 B` (`59.8 KiB`: `6 × 6,856 B` tables + `18,002 B` selectors + `2,084 B` `u16` chunk; the `56.1 KiB` parenthetical in Section 2 was a rough pre-measurement estimate before checking `size_of`). `C6_k3hi_pipe4` adds `sym_len: [u8; 256]` per table (`+1,536 B`) and shrinks `chunk` from `[u16; 1042]` to `[u8; 1042]` (`−1,042 B`), netting **`61,720 B` (`60.3 KiB`, `+488 B` = `1.008×` shipped, well under `<= 1.02×`)**.

### Primary Corpus: `wikidata.bz2` (431 blocks, 24,979,299 symbols, 7 interleaved rotated repeats on CPU 0)

#### Build 1: Baseline x86-64 (`cargo build --release`, runtime BMI2 dispatch)

| Arm | CPU min / med / max (ms) | Paid vs ref | Overlap | ns/sym | `bld_med` (ms) | `dec_med` (ms) | `dec` vs ref | `ScanScratch` |
|---|---:|---:|:---:|---:|---:|---:|---:|---:|
| `R0_shipped` | `71.966 / 72.645 / 74.301` | `+0.00%` | — | 2.908 | 5.605 | 66.277 | `+0.00%` | 59.8 KiB |
| `D1_k2_nostore` (diag) | `65.577 / 65.858 / 66.375` | `−9.34%` | NO | 2.637 | 5.621 | 59.616 | `−10.05%` | 59.8 KiB |
| `D2_k3_nostore` (diag) | `61.056 / 62.209 / 63.521` | `−14.37%` | NO | 2.490 | 7.912 | 53.692 | `−18.99%` | 60.3 KiB |
| `C1_pipe4_u16` | `70.543 / 71.190 / 72.584` | `−2.00%` | YES | 2.850 | 5.539 | 64.846 | `−2.16%` | 59.8 KiB |
| `C2_u8chunk_s5` | `71.320 / 72.431 / 74.339` | `−0.29%` | YES | 2.900 | 5.524 | 65.989 | `−0.43%` | 58.8 KiB |
| `C3_pipe4_u8` | `69.639 / 69.819 / 70.533` | `−3.89%` | NO | 2.795 | 5.604 | 63.406 | `−4.33%` | 58.8 KiB |
| **`C6_k3hi_pipe4`** | **`68.079 / 68.545 / 69.202`** | **`−5.64%`** | **NO** | **2.744** | **7.613** | **60.112** | **`−9.30%`** | **60.3 KiB** |
| `C7_k3hi_s5` | `69.151 / 70.007 / 70.712` | `−3.63%` | NO | 2.803 | 7.560 | 61.493 | `−7.22%` | 60.3 KiB |
| `C8_b11_pipe4` | `74.732 / 75.582 / 77.103` | `+4.04%` | NO | 3.026 | 8.936 | 65.694 | `−0.88%` | 82.8 KiB |
| `C9_b9k3_pipe5` | `73.100 / 73.980 / 74.697` | `+1.84%` | YES | 2.962 | 6.024 | 67.135 | `+1.30%` | 48.3 KiB |
| `C10_split_k3_p4` | `69.892 / 70.711 / 71.526` | `−2.66%` | NO | 2.831 | 7.900 | 61.821 | `−6.72%` | 60.3 KiB |

#### Build 2: Baseline + JCC-32B (`RUSTFLAGS="-C llvm-args=-x86-branches-within-32B-boundaries"`)

| Arm | CPU min / med / max (ms) | Paid vs ref | Overlap | ns/sym | `bld_med` (ms) | `dec_med` (ms) | `dec` vs ref | `ScanScratch` |
|---|---:|---:|:---:|---:|---:|---:|---:|---:|
| `R0_shipped` | `70.456 / 71.008 / 81.199` | `+0.00%` | — | 2.843 | 5.428 | 64.814 | `+0.00%` | 59.8 KiB |
| `D1_k2_nostore` (diag) | `63.882 / 64.601 / 70.187` | `−9.02%` | NO | 2.586 | 5.447 | 58.349 | `−9.97%` | 59.8 KiB |
| `D2_k3_nostore` (diag) | `58.977 / 59.641 / 60.155` | `−16.01%` | NO | 2.388 | 7.054 | 51.853 | `−20.00%` | 60.3 KiB |
| `C1_pipe4_u16` | `68.768 / 69.149 / 71.187` | `−2.62%` | YES | 2.768 | 5.359 | 63.054 | `−2.72%` | 59.8 KiB |
| `C2_u8chunk_s5` | `69.815 / 70.659 / 71.295` | `−0.49%` | YES | 2.829 | 5.429 | 64.533 | `−0.43%` | 58.8 KiB |
| `C3_pipe4_u8` | `69.003 / 69.386 / 73.211` | `−2.29%` | YES | 2.778 | 5.551 | 63.233 | `−2.44%` | 58.8 KiB |
| **`C6_k3hi_pipe4`** | **`65.850 / 66.639 / 68.059`** | **`−6.15%`** | **NO** | **2.668** | **7.343** | **58.559** | **`−9.65%`** | **60.3 KiB** |
| `C7_k3hi_s5` | `66.341 / 67.772 / 68.326` | `−4.56%` | NO | 2.713 | 7.302 | 59.500 | `−8.20%` | 60.3 KiB |
| `C8_b11_pipe4` | `71.887 / 72.484 / 73.523` | `+2.08%` | YES | 2.902 | 7.372 | 64.137 | `−1.04%` | 82.8 KiB |
| `C9_b9k3_pipe5` | `70.522 / 71.613 / 74.538` | `+0.85%` | YES | 2.867 | 5.700 | 65.183 | `+0.57%` | 48.3 KiB |
| `C10_split_k3_p4` | `67.059 / 68.476 / 74.291` | `−3.57%` | YES | 2.741 | 7.691 | 59.856 | `−7.65%` | 60.3 KiB |

#### Build 3: Native (`RUSTFLAGS="-C target-cpu=native"`)

| Arm | CPU min / med / max (ms) | Paid vs ref | Overlap | ns/sym | `bld_med` (ms) | `dec_med` (ms) | `dec` vs ref |
|---|---:|---:|:---:|---:|---:|---:|---:|
| `R0_shipped` | `70.819 / 71.461 / 72.942` | `+0.00%` | — | 2.861 | 5.634 | 65.182 | `+0.00%` |
| `D1_k2_nostore` (diag) | `64.334 / 64.671 / 65.928` | `−9.50%` | NO | 2.589 | 5.479 | 58.540 | `−10.19%` |
| `D2_k3_nostore` (diag) | `59.676 / 60.284 / 60.944` | `−15.64%` | NO | 2.413 | 7.112 | 52.490 | `−19.47%` |
| `C1_pipe4_u16` | `68.845 / 69.523 / 70.283` | `−2.71%` | NO | 2.783 | 5.586 | 63.129 | `−3.15%` |
| `C2_u8chunk_s5` | `70.159 / 71.215 / 78.042` | `−0.34%` | YES | 2.851 | 5.618 | 64.814 | `−0.57%` |
| `C3_pipe4_u8` | `67.499 / 68.002 / 69.405` | `−4.84%` | NO | 2.722 | 5.548 | 61.786 | `−5.21%` |
| **`C6_k3hi_pipe4`** | **`65.746 / 65.992 / 67.772`** | **`−7.65%`** | **NO** | **2.642** | **7.140** | **58.045** | **`−10.95%`** |
| `C7_k3hi_s5` | `67.853 / 68.677 / 69.082` | `−3.90%` | NO | 2.749 | 7.211 | 60.699 | `−6.88%` |
| `C8_b11_pipe4` | `70.157 / 70.552 / 72.844` | `−1.27%` | YES | 2.824 | 7.251 | 62.723 | `−3.77%` |
| `C9_b9k3_pipe5` | `71.000 / 71.271 / 76.737` | `−0.27%` | YES | 2.853 | 5.490 | 65.048 | `−0.21%` |
| `C10_split_k3_p4` | `67.121 / 67.350 / 69.667` | `−5.75%` | NO | 2.696 | 7.406 | 59.383 | `−8.90%` |

#### Build 4: Native + JCC-32B (`RUSTFLAGS="-C target-cpu=native -C llvm-args=-x86-branches-within-32B-boundaries"`)

| Arm | CPU min / med / max (ms) | Paid vs ref | Overlap | ns/sym | `bld_med` (ms) | `dec_med` (ms) | `dec` vs ref |
|---|---:|---:|:---:|---:|---:|---:|---:|
| `R0_shipped` | `69.347 / 69.709 / 71.143` | `+0.00%` | — | 2.791 | 4.638 | 64.315 | `+0.00%` |
| `D1_k2_nostore` (diag) | `62.902 / 63.243 / 64.949` | `−9.27%` | NO | 2.532 | 4.638 | 57.908 | `−9.96%` |
| `D2_k3_nostore` (diag) | `57.978 / 58.477 / 60.290` | `−16.11%` | NO | 2.341 | 6.880 | 50.953 | `−20.78%` |
| `C1_pipe4_u16` | `67.068 / 67.885 / 69.343` | `−2.62%` | NO | 2.718 | 4.642 | 62.478 | `−2.86%` |
| `C2_u8chunk_s5` | `68.489 / 68.797 / 69.530` | `−1.31%` | YES | 2.754 | 4.621 | 63.430 | `−1.38%` |
| `C3_pipe4_u8` | `66.861 / 66.985 / 67.892` | `−3.91%` | NO | 2.682 | 4.646 | 61.697 | `−4.07%` |
| **`C6_k3hi_pipe4`** | **`64.270 / 64.497 / 64.839`** | **`−7.48%`** | **NO** | **2.582** | **6.602** | **57.025** | **`−11.33%`** |
| `C7_k3hi_s5` | `65.440 / 65.957 / 66.626` | `−5.38%` | NO | 2.640 | 6.658 | 58.539 | `−8.98%` |
| `C8_b11_pipe4` | `69.502 / 69.693 / 71.847` | `−0.02%` | YES | 2.790 | 6.479 | 62.372 | `−3.02%` |
| `C9_b9k3_pipe5` | `68.743 / 69.300 / 70.552` | `−0.59%` | YES | 2.774 | 5.029 | 63.469 | `−1.31%` |
| `C10_split_k3_p4` | `65.918 / 66.676 / 68.349` | `−4.35%` | NO | 2.669 | 6.853 | 58.903 | `−8.41%` |

### Five Guard Corpora Summary for `C6_k3hi_pipe4` vs `R0_shipped` (7 interleaved repeats, CPU 0)

| Build | `text` (2.09M syms) | `random` (4.00M syms) | `runs` (31k syms, ×16) | `records` (4.80M syms) | `onebyte` (36 syms, ×1000) |
|---|---:|---:|---:|---:|---:|
| **1. Baseline x86-64** | **−7.88%** (`dec −9.19%`) | **−1.35%** | **−0.21%** | `+0.93%` | **−2.33%** |
| **2. Baseline + JCC-32B** | **−11.61%** (NO ovlp) | **−3.54%** (NO ovlp) | **−5.22%** (NO ovlp) | `+1.23%` | `+3.28%`* (`0.004 vs 0.004 ms`, YES ovlp) |
| **3. Native** | **−11.33%** (NO ovlp) | **−0.20%** | **−6.28%** | **−1.08%** | **−2.56%** |
| **4. Native + JCC-32B** | **−11.40%** (NO ovlp) | **−1.78%** | **−6.43%** (NO ovlp) | **−0.58%** | `+0.24%` |

*\*Note on `onebyte`: 36 symbols taking `~4.3 µs/pass` where both `R0_shipped` and `C6_k3hi_pipe4` take the identical `K = 1` runt path and overlap at `0.004..0.005 ms`; across the four builds `onebyte` averages `−0.34%` (`−2.33% / +3.28% [~140 ns] / −2.56% / +0.24%`).*

---

## 5. `objdump -d` Proof & Mechanism Decomposition

Inspecting `run_c6_k3hi_pipe4_bmi2` (`0x3c450..0x3c573` in Build 2 `target-jcc`):
- **13 instructions, 1 store per lookup** (vs shipped `#1034`'s **15 instructions, 2 stores + partial-register `%dh` read**):
  - High-2-bit `cnt` in bits `30..31` -> single `shr $0x1e` + `je slow` (`cnt == 0` encodes `EOB_FAST` / `SYM256` / `SUB_PTR` / `BAD`).
  - `SYM1/SYM2/SYM3` packed contiguously in bits `6..29` -> **`shr $0x6, %edx` + ONE `mov %edx, (%rsi,%rbx,1)`** stores 1, 2, or 3 symbols into `chunk_u8: [u8; 1042]`, bulk-widened to `Vec<u16>` every 1,000 symbols.
  - `total_bits` in bits `0..3` with bits `4..5 == 0` -> `shlx %rdx, %r15, %r15` consumes `e & 63 == total_bits` directly with **ZERO `and $0x3f`**!
- **Pipelined 4-step overlapped refill (`0x3c4fe..0x3c547`):** Issues `mov 0x0(%r13,%rax,4), %r13d` (Step 3's 5-cycle L1d load), then immediately executes `r.refill(data)` (`mov (%r8,%r11,1), %rax; bswap; shrx; or %r15, %rax`) completely in parallel under Step 3's L1d latency. Because `r.live >= 46` at batch entry and Steps 0..2 consume `<= 30` bits, `r.live >= 16 >= 10` before Step 3, so `r.refill` leaves the top 10 bits of `r.buf` unchanged and delivers the refilled word the very cycle Step 3's `shlx` executes.
  - Isolating pipelined refill alone: `C1_pipe4_u16` saves `−2.00%..−2.71%` on `K = 2`, and `C6_k3hi_pipe4` vs `C7_k3hi_s5` adds **`−1.59%..−3.75%`** paid (`−1.45%..−4.07%` decode) on top of `K = 3`!
- **Why `C6_k3hi_pipe4` succeeds where `#986` (`multi-symbol-huffman-table`) stopped:**
  1. **Single 32-bit store into `u8` chunk** replaces `#986`'s 3× `u16` stores (`+31%` store/extract uops -> `−2` instructions/step vs even `K = 2`!).
  2. **Uncapped coalesced runs (`#1034`)** eliminate `−62.2%` of group-end tails that halved `#986`'s `K = 3` gain.
  3. **Pipelined 4-step overlapped refill** removes the `+3`-cycle `live -> shrx -> or` critical-path refill stall.
  4. **100% in-place zero-copy step-jumping `K = 3` build + `K3_GATE_SEL = 1024`** trims Pass-3 overhead from `+2.6 ms` to **`+1.7..+2.0 ms`** (and `0 ms` on runts / `runs`) — and will shrink further when composed with `#1069` (`scanner-a2-fast-build`).
- **Negative ablations proved (scoped dead-ends):**
  - `C9_b9k3_pipe5` (`B = 9`, `K = 3`): `+1.84% / +0.85% / −0.27% / −0.59%` on `wikidata` (`+7.8%` more lookups outweighs halved 12 KiB LUTs) and **`+19.9%..+24.9%` regression on `runs`** (6.98% slow-path rate).
  - `C8_b11_pipe4` (`B = 11`, `K = 2`, `48 KiB > 32 KiB` L1d): `+4.04% / +2.08% / −1.27% / −0.02%` on `wikidata`, **`+20.7%..+22.7%` on `records`**, **`+4.0%..+6.4%` on `random`**, `82.8 KiB` scratch (`1.38×`).
  - `C10_split_k3_p4` (split contiguous 24 KiB hot LUTs): consistently `1.9%..3.1%` slower than `C6_k3hi_pipe4` (`−2.66%..−5.75%` vs `−5.64%..−7.65%`) because the extra hot-LUT base pointer adds GPR pressure plus a 24 KiB LUT copy.
  - Deferred `sum_e` live accumulation across `PRE_STEPS` and runtime `max_k` batch sizing both regressed `+2.1..+3.5 ms` by stealing a 15th GPR inside the 14-GPR hot loop and causing register spills.

---

## 6. Verdict & Modeled i9-10900K Impact

- **Verdict: PASS** under the frozen pre-data stop test across **all four compiler builds**:
  - `C6_k3hi_pipe4` cuts paid post-`#1034` `A3` (`A2d build_into + A3 decode_runs`) on `wikidata.bz2` by **`−5.64%` Baseline x86-64 (NO overlap)**, **`−6.15%` Baseline + JCC-32B (NO overlap)**, **`−7.65%` Native (NO overlap)**, and **`−7.48%` Native + JCC-32B (NO overlap)** (decode-only `−9.30% / −9.65% / −10.95% / −11.33%`).
  - `ScanScratch` is `60.3 KiB` vs `59.8 KiB` shipped (`1.008× <= 1.02×`).
  - All 5 guard corpora pass cleanly (`text −7.88%..−11.61%`, `runs −0.21%..−6.43%`, `random −0.20%..−3.54%`, `records −1.08%..+1.23%`, `onebyte` flat within ~100 ns measurement floor).
- **Modeled i9-10900K impact:** On post-`#1034` `A3` (`~115 ms` of `S'`), `−5.6%..−7.6%` projects **`−6.5..−8.8 ms` directly off serial `S'`** (stacking orthogonally with `#1069` `scanner-a2-fast-build` and `#1141` `scanner-adaptive-seam-window`, pulling post-`#1034` `S'` from `~145 ms` toward `~118–122 ms`!), and **`−10..11%` off scanner-bound `text` at 8W**.
- **Follow-up claim opened:** [`frontier/claims/scanner-a3-k3-pipe-u8.toml`](../../../frontier/claims/scanner-a3-k3-pipe-u8.toml) (`status = "idea"`, `owned_paths = ["src/block.rs", "src/huff.rs", "src/bits.rs"]`, queued behind `#1069` `scanner-a2-fast-build`).

