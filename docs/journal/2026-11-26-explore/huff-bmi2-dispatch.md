# Runtime BMI2 (`shlx`/`shrx`) dispatch ablation for shipped `K = 2` Huffman decode: PASS

Probe for claim `huff-bmi2-dispatch` (26 November 2026). Standalone ablation harness in
`examples/huff_bmi2_ablation.rs` (zero `src/` edits; `#![deny(unsafe_op_in_unsafe_fn)]`, with the
only `unsafe` being runtime-checked calls to `#[target_feature]` wrappers for Rust 1.80 MSRV
compatibility) against `corpora/wikidata.bz2` (431 blocks: 233 full / 198 runts, 24,979,299
symbols, SHA-256 `82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915` verified) on
a 2-vCPU Haswell Xeon @ 2.30 GHz (32 KiB L1d / 256 KiB L2 / 45 MiB L3, `bmi1`/`bmi2`/`avx2`
detected, no vPMU).

**Verdict under the frozen stop test: PASS.** On a default baseline x86-64 release build over 7
interleaved repeats across all 431 blocks, isolated `+bmi2` (`bmi2_all`, with `shlx`/`shrx`
verified in `objdump -d`) cuts total A3 (`decode + tables`) by **−9.01%** (`75.508 → 68.702 ms`,
non-overlapping `[71.344..78.610]` vs `[64.228..70.856]`, Run 2) and **−9.51%** (`77.897 → 70.492
ms`, non-overlapping `[74.912..86.676]` vs `[70.366..71.950]`, Run 1), clearing the frozen ≥ 5.0%
non-overlapping gate nearly 2× over. Striated build claim `huff-bmi2-build` is opened at
`status = "idea"`.

## 1. Design and verification

- **Faithful shipped mirror:** `examples/huff_bmi2_ablation.rs` mirrors `src/bits.rs` (`Reg`),
  `src/huff.rs` (`Huff::build_into`, `decode_group_win`, `decode_group_slice`, `decode_one_win`,
  `decode_one_slice`, `decode_entry_slow_win`, `decode_entry_slow`, `decode_slow`), and
  `src/block.rs:220–262` (`decode_block_groups_inline`, including `Result<_, ErrorKind>`
  propagation). Headers, selectors, and code lengths are parsed once outside the clock.
- **Ten arms in a single baseline x86-64 binary** (all `#[inline(never)]` entry points for clean
  `objdump -d` attribution):
  - `baseline`: no `#[target_feature]` (matches shipped `src/huff.rs` + `src/block.rs`).
  - `bmi2_dec`: baseline `build_into` + block-level `#[target_feature(enable = "bmi2")]` decode.
  - `bmi2_all`: `#[target_feature(enable = "bmi2")]` on both `build_into` and block-level decode.
  - `bmi2_grp`: baseline `build_into` + **per-group** non-inlined `#[target_feature(enable = "bmi2")]`
    `decode_group_win`/`decode_group_slice` (tests dispatch granularity).
  - `bmi1_all`, `avx2_all`, `avx2_bmi1` (matches what `Kernels::detect().avx2` currently probes),
    `bmi1_bmi2`, `avx2_bmi1_bmi2` (full Haswell ISA without `-C target-cpu=native` scheduling).
  - `avx2_t_bmi2_d`: `avx2,bmi1,bmi2` on `build_into` + pure scalar `bmi2` on block-level decode
    (avoids AVX2/YMM `vzeroupper` transitions inside the decode loop).
- **Exactness:** All 10 arms verified 100% symbol-for-symbol and ending-bit-position identical
  against an independent canonical bit-by-bit decoder on all 431 blocks of `corpora/wikidata.bz2`
  (24,979,299 symbols) plus 4 synthetic edge cases:
  1. `win_pad300`: `alpha = 258`, `sym = 256` (len 3), `EOB = 257` (len 4), len-12 `SUB_PTR`,
     group boundaries `k = 48, 49, 50` through `decode_group_win`.
  2. `slice_pad0`: same bitstream with `start_bit = 5` and 0 B tail padding through
     `decode_group_slice` / `decode_one_slice` / `decode_entry_slow`.
  3. `len20`: Kraft-exact 31-symbol tree with two 10-bit prefixes reaching len 20 (first prefix
     fills `SUB_CAP = 1024` via `SUB_PTR`, second overflows `SUB_CAP` into `decode_slow`).
  4. `BAD`: incomplete length-2 code verifying `Err(ErrorKind::BadHuffmanCode)` parity.

## 2. Primary baseline x86-64 ablation (all 431 blocks, 7 interleaved repeats)

Single `Instant::now()` over all 431 blocks (`build_tables + decode_syms`) after 3 warmup passes,
7 interleaved repeats with rotated arm order per repeat, default release build
(`cfg(target_feature): bmi1=false bmi2=false avx2=false`).

### 2a. Primary run (final 10-arm harness, VM load average 1.00)

| arm | min ms | med ms | max ms | vs `baseline` | overlap? | ns/sym | `tbl_all` ms | `dec_all` ms | `dec` vs base |
|---|---:|---:|---:|---:|:---:|---:|---:|---:|---:|
| `baseline` | 71.344 | **75.508** | 78.610 | +0.00% | — | 3.023 | 5.459 | 68.511 | +0.00% |
| `bmi2_dec` | 65.594 | **68.345** | 71.578 | **−9.49%** | YES* | 2.736 | 5.592 | 63.531 | **−7.27%** |
| **`bmi2_all`** | 64.228 | **68.702** | 70.856 | **−9.01%** | **NO** | 2.750 | 5.188 | 63.092 | **−7.91%** |
| `bmi2_grp` | 73.100 | 75.639 | 77.606 | +0.17% | YES | 3.028 | 5.540 | 70.742 | **+3.26%** |
| `bmi1_all` | 72.121 | 74.980 | 76.660 | −0.70% | YES | 3.002 | 5.394 | 68.097 | −0.60% |
| `avx2_all` | 71.658 | 73.521 | 76.989 | −2.63% | YES | 2.943 | 5.532 | 68.455 | −0.08% |
| `avx2_bmi1` | 68.749 | 73.121 | 76.238 | −3.16% | YES | 2.927 | 5.213 | 67.295 | −1.77% |
| `bmi1_bmi2` | 65.710 | **68.166** | 71.680 | **−9.72%** | YES | 2.729 | 5.220 | 64.037 | −6.53% |
| `avx2_bmi1_bmi2` | 65.708 | **68.916** | 71.183 | **−8.73%** | **NO** | 2.759 | 5.242 | 63.975 | −6.62% |
| **`avx2_t_bmi2_d`** | 67.018 | **68.259** | 70.372 | **−9.60%** | **NO** | 2.733 | 5.265 | 63.029 | **−8.00%** |

*\*`bmi2_dec` was `< 71.344 ms` on 6 of 7 repeats (`65.594..69.512 ms`); its sole max (`71.578 ms`,
repeat 3) paired against `baseline` repeat 3 (`78.610 ms`, paired ratio −8.95%).*

### 2b. Earlier corroborating run (9-arm harness, VM load average 2.50)

| arm | min ms | med ms | max ms | vs `baseline` | overlap? | `tbl_all` ms | `dec_all` ms | `dec` vs base |
|---|---:|---:|---:|---:|:---:|---:|---:|---:|
| `baseline` | 74.912 | **77.897** | 86.676 | +0.00% | — | 5.540 | 70.511 | +0.00% |
| `bmi2_dec` | 69.183 | **71.646** | 115.154* | −8.02% | YES* | 5.580 | 65.242 | −7.47% |
| **`bmi2_all`** | 70.366 | **70.492** | 71.950 | **−9.51%** | **NO** | 5.288 | 65.343 | −7.33% |
| `bmi2_grp` | 77.325 | 78.504 | 80.318 | +0.78% | YES | 5.612 | 74.743 | **+6.00%** |
| `bmi1_all` | 75.072 | 76.340 | 77.312 | −2.00% | YES | 5.830 | 72.123 | +2.29% |
| `avx2_all` | 74.844 | 75.582 | 76.277 | −2.97% | YES | 5.275 | 70.342 | −0.24% |
| `avx2_bmi1` | 74.707 | 75.916 | 76.860 | −2.54% | YES | 5.056 | 70.532 | +0.03% |
| `avx2_bmi1_bmi2` | 70.028 | 71.572 | 91.409* | −8.12% | YES* | 4.675 | 66.587 | −5.57% |

*\*Single-repeat VM preemption spikes under parallel load; `bmi2_all` stayed non-overlapping in both runs.*

## 3. Disassembly check (`objdump -d`)

Every arm in the baseline x86-64 release binary was disassembled with `objdump -d -C`:

| function | insns | `shlx` | `shrx` | `sarx` | `shl %cl` | `shr %cl` | `bextr` | `andn` | `vmov` | `vpadd` | `vzeroupper` |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| `huff_dec_baseline` | 1,666 | 0 | 0 | 0 | **23** | **10** | 0 | 0 | 0 | 0 | 0 |
| **`huff_dec_bmi2`** | 1,694 | **23** | **10** | 0 | **0** | **0** | 0 | 0 | 0 | 0 | 0 |
| `huff_group_win_bmi2` | 1,020 | 11 | 5 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| `huff_dec_bmi1` | 1,666 | 0 | 0 | 0 | 23 | 10 | 0 | 0 | 0 | 0 | 0 |
| `huff_dec_avx2` | 1,693 | 0 | 0 | 0 | 23 | 10 | 0 | 0 | 4 | 0 | **38** |
| `huff_dec_avx2_bmi1` | 1,693 | 0 | 0 | 0 | 23 | 10 | 0 | 0 | 4 | 0 | **38** |
| `huff_dec_bmi1_bmi2` | 1,694 | 23 | 10 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| `huff_dec_avx2_bmi1_bmi2` | 1,723 | 23 | 10 | 0 | 0 | 0 | 0 | 0 | 4 | 0 | **38** |
| `huff_build_baseline` | 1,164 | 0 | 0 | 0 | 11 | 1 | 0 | 0 | 0 | 0 | 0 |
| `huff_build_bmi2` | 1,151 | 11 | 1 | 3 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| `huff_build_avx2` | 1,141 | 0 | 0 | 0 | 11 | 1 | 0 | 0 | 48 | 3 | 18 |
| `huff_build_avx2_bmi1_bmi2` | 1,127 | 11 | 1 | 3 | 0 | 0 | 0 | 1 | 48 | 3 | 18 |

Four structural findings directly from the emitted instructions:

1. **`bmi2` replaces 100% of variable shifts with zero AVX/YMM overhead.** `huff_dec_bmi2` replaces
   all 23 `shl %cl` (`skip_short_shl`) with `shlx` and all 10 `shr %cl` (`refill_win`/`refill`)
   with `shrx`, leaving `0 shl %cl`, `0 shr %cl`, `0 vmov`, and `0 vzeroupper`.
2. **`bmi1` is a no-op on decode.** `huff_dec_bmi1` is instruction-count-identical to
   `huff_dec_baseline` (`1,666` insns; `0 bextr`, `0 andn`, `0 tzcnt`), and `huff_dec_bmi1_bmi2` is
   identical to `huff_dec_bmi2` (`1,694` insns).
3. **`avx2` helps `build_into` (`48 vmov` + `3 vpaddd`), NOT decode — and pollutes decode with
   38 `vzeroupper`s.** In `huff_dec_avx2` / `huff_dec_avx2_bmi1_bmi2`, LLVM uses 4 × 256-bit
   `vmovups %ymm0` solely to zero-initialize `tail_buf = [0u16; 66]` at entry, which forces 38
   `vzeroupper` instructions across exits. Keeping decode pure `bmi2` (`avx2_t_bmi2_d`, −9.60%)
   avoids touching YMM registers during decode while optionally letting `build_into` vectorize
   `lut.fill` and Canonical Template-Clone `wrapping_add(delta)`.
4. **LLVM CSEs `and $0x3f` onto the critical path before `shlx` (and `shl %cl`).** Hot-step
   disassembly of `huff_dec_baseline` vs `huff_dec_bmi2`:
   ```asm
   ; baseline (160f0..1610e)                  ; bmi2 (28765..28785)
   mov    %r12,%rax                           mov    %r13,%rax
   shr    $0x36,%rax          ; +1 cyc        shr    $0x36,%rax          ; +1 cyc
   mov    (%r14,%rax,4),%edx  ; +5 cyc L1d    mov    (%r14,%rax,4),%edx  ; +5 cyc L1d
   mov    %edx,%ecx                           mov    %edx,%ecx
   and    $0x3f,%ecx          ; +1 cyc (!)    and    $0x3f,%ecx          ; +1 cyc (!)
   shl    %cl,%r12            ; +2 cyc, 3uop  shlx   %rcx,%r13,%r13      ; +1 cyc, 1uop
   sub    %ecx,%ebx                           sub    %ecx,%r15d
   ```
   Because `self.live -= e & 63` computes `e & 63` right beside `self.buf.wrapping_shl(e)`, LLVM
   reuses the masked `%ecx` as the shift count for `shlx` even though `shlx` ignores bits `6..63`
   in hardware. That leaves an extra 1-cycle `and $0x3f` on the dependent `buf` chain (a separate
   micro-optimization opportunity if `live` batching or unmasked shift count is ever explored).

## 4. Mechanism decomposition: 2×2 `RUSTFLAGS` factorial + `nostore` + dispatch granularity

### 4a. Separating BMI2 from `-C target-cpu=native` scheduling (`RUSTFLAGS` cross-checks)

| build (`RUSTFLAGS`) | `baseline` med (min) | `bmi2_all` med | `avx2_t_bmi2_d` med | `dec_all` base → `bmi2_all` | what it isolates |
|---|---:|---:|---:|---:|---|
| default baseline x86-64 | 75.508 (71.344) | **68.702** (−9.01%, NO) | **68.259** (−9.60%, NO) | 68.511 → 63.092 (−7.91%) | Primary frozen gate |
| `-C target-feature=+bmi2` | **69.899** (68.286) | 69.703 (−0.28%, YES) | 69.470 (−0.61%, YES) | 64.580 → 63.977 (−0.93%) | Once `+bmi2` is global, `bmi1`/`avx2` add ~0% |
| `-C target-cpu=native -C target-feature=-bmi2` | **75.326** (73.292) | 70.856 (−5.93%) | **69.366** (−7.91%, NO) | 70.999 → 66.040 (−6.98%) | Haswell tuning + AVX2 + BMI1 **without BMI2** stays at baseline (75.3 ms); re-enabling only `bmi2` recovers −6.0..−7.9% |
| `-C target-cpu=native` | 78.425* (min 70.589) | 81.383* (min 67.854) | 78.152* (min 69.369) | *(contended VM load)* | Uncontended mins (67.9–70.6 ms) match `bmi2_all` & `#986`'s native `k2_shipped` (67.7 ms) |

*\*Contended repeats under parallel VM load (load avg 2.50); uncontended mins match `feat-bmi2` and `bmi2_all`.*

### 4b. Latency vs uop/port-0/6 pressure (`nostore` diagnostic, 233 full blocks)

| arm (baseline build) | `nostore` med ms | vs base | full `dec` med ms | vs base | store+extract+uop share |
|---|---:|---:|---:|---:|---:|
| `baseline` | 61.261 (2.453 ns/s) | +0.00% | 68.511 (2.742 ns/s) | +0.00% | 7.25 ms |
| `bmi2` (`bmi2_all`) | 58.650 (2.349 ns/s) | **−4.26%** | 63.092 (2.525 ns/s) | **−7.91%** | 4.44 ms |
| `bmi1` | 60.869 (2.438 ns/s) | −0.64% | 68.097 (2.725 ns/s) | −0.60% | 7.23 ms |
| `avx2` | 61.078 (2.446 ns/s) | −0.30% | 68.455 (2.740 ns/s) | −0.08% | 7.38 ms |

*(In the `native-nobmi2` build, `nostore` dropped `64.424 → 60.674 ms`, −5.82%, non-overlapping.)*

`bmi2` cuts the pure `nostore` dependency chain by −2.61 ms (−4.26%), and cuts full decode by
−5.42 ms (−7.91%): roughly half the gain is the 1-cycle latency reduction (`shlx`/`shrx` 1 cyc vs
`shl`/`shr %cl` 2 cyc), and roughly half is eliminating 2 uops per shift plus `%rcx` register /
port-0/6 contention (~27.4M uops eliminated on `skip_short_shl` + ~5.5M uops on `refill_win`'s
`shrx` ≈ 33M uops removed across 431 blocks) against the 2-symbol extract-and-store instructions.

### 4c. Critical implementation constraint: block-level dispatch only (`bmi2_grp` fails)

Dispatching `#[target_feature(enable = "bmi2")]` at the per-group `decode_group_win` boundary
(`bmi2_grp`, 499,534 non-inlined calls across 431 blocks) **completely destroys the −7.9% BMI2
gain and regresses decode by +3.26% to +6.00%** (`+0.17%..+0.78%` total A3) due to per-group
call/prologue/register-reload overhead. The production build (`huff-bmi2-build`) **must** hoist
BMI2 dispatch to the **block-level group loop** (431 calls total, with `decode_group_win` and
`decode_group_slice` `#[inline(always)]` inside it).

## 5. Projected i9 impact & caveats

- **Projected i9 impact (derived):** On i9-10900K (Comet Lake, same 3-uop/2-cycle `shl %cl` vs
  1-uop/1-cycle `shlx` microarchitecture as Haswell), A3 is ~81% of the 177.5 ms serial scan
  (~144 ms). Applying the measured −7.9% to −9.6% A3 reduction projects **~11.4–13.8 ms** off the
  serial scan (`scan′ ≈ 163.7–166.1 ms`, ~6.4–7.8% of the 8-worker scan wall once worker-side
  improvements bring `S′(8) < 164 ms`), with zero table bloat, zero L1d spill, and zero algorithmic
  risk.
- **Caveats:** Measured on a 2-vCPU Haswell Xeon VM (no vPMU, shared host CPU), not the i9-10900K.
  Comet Lake has the same `shl %cl` vs `shlx` latency/uop ratio, but actual end-to-end scan and
  8-worker wall gains must be confirmed on the i9 under `huff-bmi2-build`'s frozen registration
  before default adoption.
