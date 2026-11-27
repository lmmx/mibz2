# Cold dissection of the serial scanner (`S`): selector coalescing, byte-aligned `u8` LUT, `syms` zero-fill, and BMI2

An `explore` session (26 November 2026), started cold from primary source code (`src/block.rs`, `src/huff.rs`, `src/bits.rs`, `src/parallel.rs`) and `corpora/wikidata.bz2` (431 blocks, 24,979,299 symbols, 11,854,933 compressed bytes, SHA-256 `82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915`). Zero changes to `src/`, zero `unsafe` (`#![forbid(unsafe_code)]`), no prior frozen gates touched.

**Harness:** [`examples/scanner_dissection_probe.rs`](../../../examples/scanner_dissection_probe.rs).  
**Hardware:** 2-vCPU Haswell Xeon @ 2.30 GHz KVM VM (32 KiB L1d / 256 KiB L2 / 45 MiB L3, no vPMU). Because other sessions share the VM, all Section 3 comparisons run 7 interleaved repeats with rotated arm order per repeat and report **both** exact thread on-CPU execution time (`/proc/thread-self/schedstat` field 1, immune to runqueue preemption) and wall-clock `Instant::now()`, compiled under both **baseline x86-64** (matching shipped wheels and workstation bench builds) and **`-C target-cpu=native`** (Haswell BMI2 `shlx`/`shrx`/`movbe` + AVX2).

---

## Executive summary

With `pooled-scan` killed as built and `multi-symbol-huffman-table` (`K = 3`/`K = 4`) at `STOP`, row `S` (serial Huffman scan, 170–179 ms on i9-10900K — the 8-worker bottleneck once `mtf-fold-build` and `forward-emission` land) had no active single-threaded lever on the frontier map. Cold dissection of the full `read_item_with_scratch` path exposes **four orthogonal, additive levers** inside the existing 24 KiB L1d footprint (`B = 10`, `K = 2`):

1. **Consecutive same-selector run coalescing (`M` groups -> `50 * M` symbols):** On the 233 full blocks, `62.4%` of selectors have MTF index `j = 0`, collapsing **499,534** 50-symbol groups into **188,898** uncapped same-table runs (`mean M = 2.64`, **−62.2%** group boundaries, tail loops, `k == 49` coin flips, and per-group refills). Alone, uncapped coalescing cuts A3 by **−7.2% to −10.0%** (`a3_coal_k2_direct`).
2. **Byte-aligned 8-bit `SYM1`/`SYM2` LUT + low-byte `skip_short_shl_u8` + `[u16; 18]` batch slices:** Packing `total_bits` in bits `0..7` (bits `4..7 == 0`), `SYM1` in `8..15`, `SYM2` in `16..23`, `LEN1` in `24..28`, and routing `sym == 256` to a `cnt == 0` slow tag (0 occurrences in `wikidata.bz2`; verified exact on synthetic `alpha = 258`) lets LLVM emit **`shl %cl` / `shlx %rdx` directly without `and $0x3f` on the critical shift chain**, replaces both `& 0x1ff` masks with `movzbl %ch` / `movzbl %cl`, and eliminates per-step bounds checks (`18 -> 14` instructions/lookup). Combined with uncapped coalescing in 100% safe Rust, A3 drops **−9.5% to −12.3%** and full end-to-end `S` drops **−10.8% (`79.49 -> 70.91 ms`, zero overlap)** on baseline x86-64.
3. **Eliminating `syms.resize(max_out, 0)` after runt truncations (`src/block.rs:221-223,268`):** Every runt block truncates its recycled `Vec<u16>` to ~22 symbols (`syms.truncate(n)`). The next full block to pop it zeroes `max_out * 2` bytes before overwriting them: **219/431** blocks (**37.41 MiB**, +78.5% extra writes) sequentially, and **235/431** blocks (**40.23 MiB**, +84.4% extra writes) under simulated 8-worker LIFO `free_syms` reuse. On a single L3-hot `Vec`, zeroing is mostly hidden, but on a 16-buffer rotating pool (`pool16_shipped_zero` vs `pool16_no_zero`), eliminating it saves **−3.6% to −5.0% of full `S`** (`3.0–4.0 ms` on VM).
4. **Runtime BMI2 dispatch (`is_x86_feature_detected!("bmi2")`, matching `src/runs.rs`):** Shipped binaries and workstation bench runs compile `src/huff.rs` as baseline x86-64 (`shl %cl` = 3 uops / 1–2 cyc). With BMI2 (`shlx`/`shrx`/`movbe`), the critical path per lookup drops from `shr(1) + load(4) + and(1) + shl_cl(2) = 8 cyc` to `shr(1) + load(4) + shlx(1) = 6 cyc` (**−25%**), taking full `S` from **`79.49 ms` (baseline shipped)** to **`67.72 ms` (native combined)** (**−14.8%**, or **−17.9%** vs `pool16_shipped_zero` `82.45 ms`).

**Four negative findings (ruled out cleanly):**
- **Coalescing does NOT rescue 4-byte packed `K = 3`:** Even with `M <= 8` coalescing removing 59.2% of group tails, `a3_coal_k3_win8` (`71.89 ms` base / `67.62 ms` nat) still loses to `a3_coal_k2_u8_dir` (`68.00 ms` / `63.79 ms`) by `+5.7–6.0%` (`full_combined_k3` loses to `full_combined_k2` by `+4.7–6.9%`). Reinforces `multi-symbol-huffman-table`'s STOP verdict.
- **Inlining `SUB_PTR` (`a3_coal_k2_inlinesub`) regresses `+3.9–6.1 ms`:** At 1.24% hit rate (`170k`), inlining into all 5 unrolled steps bloats the hot loop; keep `decode_entry_slow` `#[cold] #[inline(never)]`.
- **Branchless masked `refill_win` alone (`a3_mask_refill`) is null (`+0.50%` base / `+0.96%` nat):** The `win.get` branch is 100% predicted not-taken.
- **Batched `j == 0` selector MTF (`A2b_sel_batched_runs`) is neutral/slightly slower** (`2.85` vs `2.66 ms` base; `2.70` vs `2.39 ms` nat); `A2a+b` is already only ~3% of `S`.

Opened striated build claim [`scanner-coalesced-u8-decode`](../../../frontier/claims/scanner-coalesced-u8-decode.toml) at `status = "idea"` with a frozen stop test (≥ 8.0% cut to serial scan `S`, byte- and error-exact).

---

## 1. Exact corpus census (`corpora/wikidata.bz2`, 431 blocks)

| metric | all 431 blocks | 233 full blocks | 198 runts |
|---|---:|---:|---:|
| symbols | 24,979,299 | 24,971,004 | 8,295 |
| selectors (`n_sel`) | 501,448 | 500,366 | 1,082 |
| tables read / built (`used_groups`) | 1,796 / 1,606 | — / 1,398 | — / 208 |
| code lengths (`unchanged 0-bit`) | 133,280 (`91,363` = 68.5%; 50,863 delta steps, 0.38/len) | — | — |

- **Selector MTF index distribution (`501,448` total):** `j = 0`: `312,762` (**62.4%**), `j = 1`: `105,078` (21.0%), `j = 2`: `37,225` (7.4%), `j = 3`: `19,626` (3.9%), `j = 4`: `14,638` (2.9%), `j = 5`: `12,119` (2.4%).
- **Same-selector run coalescing (233 full blocks, `499,534` active groups):**
  - Uncapped runs: **`188,898`** (`mean M = 2.64`, **−62.2%** group boundaries).
  - Capped `M <= 8` runs: **`203,723`** (`mean M = 2.45`, **−59.2%** group boundaries).
  - Run-length histogram (`M = 1..15, >=16`): `M1=105,890`, `M2=33,472`, `M3=16,154`, `M4=9,471`, `M5=5,672`, `M6=3,687`, `M7=2,822`, `M8=2,327`, `M9=1,763`, `M10=1,350`, `M11=951`, `M12=705`, `M13=541`, `M14=456`, `M15=391`, `M16+=3,246`.
- **`syms.resize(max_out, 0)` zero-fill census (`src/block.rs:221-223,268`):**
  - Actual symbols written: `47.64 MiB` (`24,979,299 × 2 B`).
  - Sequential single-`Vec` reuse: **`219 / 431`** blocks trigger `resize(max_out, 0)`, zeroing **`19,611,527` `u16`s (`37.41 MiB`, +78.5% extra writes)**.
  - Simulated 8-worker LIFO `free_syms` pool (`src/parallel.rs:1131,1205`, where runts finish immediately and push their `len ~ 22` `Vec` onto the top of the LIFO stack): **`235 / 431`** blocks trigger `resize(max_out, 0)` (plus 12 initial allocations), zeroing **`21,090,724` `u16`s (`40.23 MiB`, +84.4% extra writes)** — `~161 MiB` on wikidata ×4.

**Exactness verification:** Every arm matched `mibz2::stream::Scanner` symbol-for-symbol and ending-bit-position-for-ending-bit-position across all 431 blocks (24,979,299 symbols), plus a synthetic `alpha = 258` edge case exercising `sym = 256` (short 4-bit `SYM256` tag), `len = 12` `SUB_PTR`, and `EOB = 257` at `target - 1`.

---

## 2. Full `read_item_with_scratch` stage breakdown (431 blocks)

Uncontended medians (7 interleaved repeats; native run was uncontended throughout Section 2; baseline figures from uncontended CPU-1 Section 2):

| stage | baseline x86-64 med (`min..max`) | `-C target-cpu=native` med (`min..max`) | share of `S` (approx) |
|---|---:|---:|---:|
| `A2a+b_hdr_sel_shipped` (header + `in_use16` + sel MTF) | 2.662 (`2.574..2.774`) ms | 2.393 (`2.246..2.533`) ms | 3.0% |
| `A2b_sel_batched_runs` (batched `j==0` run builder) | 2.854 (`2.632..3.014`) ms | 2.695 (`2.590..2.779`) ms | — |
| `A2c_codelens` (code-length delta decode) | 2.385 (`2.138..2.460`) ms | 2.262 (`2.086..2.314`) ms | 2.8% |
| `A2d_build_k2_all` (`Huff::build_into`, 1,606 tables) | 5.606 (`5.305..6.169`) ms | 5.558 (`5.311..7.110`) ms | 6.6% |
| `A2d_build_k2_gated` (`n_sel >= 64` skips runt pairs) | 5.210 (`4.996..5.551`) ms | 4.905 (`4.628..5.179`) ms | 6.0% |
| `A2d_build_k3_gated` (Pass-3 K=3, runt-gated) | 7.209 (`6.909..7.559`) ms | 6.313 (`5.977..6.936`) ms | — |
| `A2e_syms_zero_seq` (standalone `resize(max_out,0)`) | 3.342 (`3.231..3.819`) ms | 3.347 (`3.189..5.307`) ms | 3.9% |
| `A3_dec_shipped_only` (`decode_group_win`, prebuilt) | 73.796 (`72.331..76.092`) ms | 70.068 (`68.378..75.968`) ms | 83.7% |

*(Note: `A2` = header + selectors + code lengths + `Huff::build_into` + `syms.resize` accounts for ~16.3% of `S`; `A3` decode accounts for ~83.7%, matching the ~19%/81% i9 split.)*

---

## 3. Seven-repeat interleaved ablation (on-CPU `schedstat` and wall-clock)

Full-scanner arms compare against `shipped_full_zero` (faithful mirror of `read_item_with_scratch`); A3 arms (`build_into + decode`) compare against `a3_shipped`.

### 3a. Baseline x86-64 (default release build, matching shipped binary)

| arm | on-CPU min / **med** / max (ms) | vs ref (CPU) | overlap? | ns/sym | wall med (`min..max`) ms |
|---|---:|---:|:---:|---:|---:|
| `lib_scanner_seq` (`mibz2::stream::Scanner`) | 82.03 / **83.98** / 85.81 | +5.65% | NO | 3.362 | 86.89 (`82.84..96.25`) |
| `shipped_full_zero` *(full ref)* | 74.68 / **79.49** / 79.99 | +0.00% | — | 3.182 | 79.83 (`76.67..90.25`) |
| `full_no_zero` (1 hot `Vec`) | 75.74 / **79.89** / 80.81 | +0.51% | YES | 3.198 | 79.08 (`77.24..157.92`) |
| `pool16_shipped_zero` (16-buf LIFO pool) | 75.75 / **82.45** / 87.69 | +3.73% | YES | 3.301 | 86.67 (`78.98..174.94`) |
| `pool16_no_zero` (16-buf pool, no zero) | 76.96 / **79.45** / 79.83 | −0.05% | YES | 3.181 | 85.94 (`78.25..160.04`) |
| **`full_combined_k2`** (coalesced + `u8lut` + no-zero) | **67.65 / 70.91 / 71.99** | **−10.79%** | **NO** | **2.839** | **70.81 (`68.87..76.20`)** |
| `full_combined_k3` (coalesced `K=3` + no-zero) | 71.75 / **75.82** / 79.12 | −4.62% | YES | 3.035 | 75.08 (`73.86..119.68`) |
| `a3_shipped` *(A3 ref)* | 71.71 / **75.11** / 77.88 | +0.00% | — | 3.007 | 77.16 (`74.38..138.96`) |
| `a3_mask_refill` (`pos & 255` alone) | 71.79 / **75.49** / 75.96 | +0.50% | YES | 3.022 | 75.72 (`71.07..142.65`) |
| `a3_coal_k2_win8` (`M<=8`, 9-bit LUT) | 67.80 / **71.15** / 72.76 | −5.27% | YES | 2.848 | 72.45 (`67.70..105.25`) |
| `a3_coal_k2_direct` (uncapped `M`, 9-bit LUT) | 67.84 / **69.49** / 72.61 | −7.48% | YES | 2.782 | 73.28 (`67.56..117.76`) |
| **`a3_coal_k2_u8_win8`** (`M<=8` + `u8lut` + `[u16; 514]`) | **67.82 / 67.97 / 71.95** | **−9.51%** | YES | **2.721** | 69.94 (`66.38..136.37`) |
| **`a3_coal_k2_u8_dir`** (uncapped + `u8lut` + `[u16; 18]`) | **63.87 / 68.00 / 72.49** | **−9.48%** | YES | **2.722** | 76.30 (`64.67..100.64`) |
| `a3_coal_k2_inlinesub` (inline `SUB_PTR`) | 71.14 / **74.14** / 75.92 | −1.30% | YES | 2.968 | 77.64 (`73.28..135.97`) |
| `a3_coal_k3_win8` (`M<=8` + 4B `K=3`) | 67.97 / **71.89** / 74.90 | −4.30% | YES | 2.878 | 71.76 (`70.26..96.18`) |

*(In a separate pinned CPU-0 baseline run with tighter per-arm spread, `a3_coal_k2_u8_dir` beat `a3_shipped` by **−12.32%** with zero overlap and `a3_coal_k2_u8_win8` beat `a3_shipped` by **−11.01%** with zero overlap; `pool16_shipped_zero` vs `pool16_no_zero` showed −4.87%.)*

### 3b. Native build (`RUSTFLAGS="-C target-cpu=native"`, Haswell BMI2 `shlx`/`shrx`/`movbe`)

| arm | on-CPU min / **med** / max (ms) | vs native ref | overlap? | vs base ref | wall med (`min..max`) ms |
|---|---:|---:|:---:|---:|---:|
| `lib_scanner_seq` | 78.20 / **79.96** / 83.83 | +5.30% | NO | +0.60% | 81.01 (`79.88..84.06`) |
| `shipped_full_zero` *(nat full ref)* | 71.57 / **75.94** / 78.15 | +0.00% | — | −4.47% | 75.80 (`75.00..79.68`) |
| `full_no_zero` | 71.96 / **75.53** / 77.90 | −0.54% | YES | −4.98% | 76.05 (`73.19..79.18`) |
| `pool16_shipped_zero` | 75.93 / **79.92** / 82.14 | +5.24% | YES | +0.54% | 78.22 (`76.35..81.24`) |
| `pool16_no_zero` | 72.00 / **75.91** / 79.83 | −0.04% | YES | −4.50% | 76.46 (`74.39..78.12`) |
| **`full_combined_k2`** | **63.97 / 67.72 / 70.67** | **−10.82%** | **NO** | **−14.80%** | **67.84 (`66.33..69.74`)** |
| `full_combined_k3` | 67.66 / **70.90** / 77.32 | −6.63% | YES | −10.80% | 71.01 (`68.85..78.25`) |
| `a3_shipped` *(nat A3 ref)* | 67.69 / **71.10** / 72.27 | +0.00% | — | −5.34% | 71.00 (`69.32..77.07`) |
| `a3_mask_refill` | 67.94 / **71.79** / 75.52 | +0.96% | YES | −4.43% | 70.25 (`68.21..75.43`) |
| `a3_coal_k2_win8` | 67.85 / **67.97** / 71.84 | −4.40% | YES | −9.51% | 68.49 (`65.87..72.01`) |
| `a3_coal_k2_direct` | 63.65 / **63.99** / 67.96 | −10.00% | YES | −14.80% | 65.89 (`64.18..68.13`) |
| `a3_coal_k2_u8_win8` | 59.97 / **63.97** / 67.97 | −10.03% | YES | −14.84% | 64.92 (`62.10..66.03`) |
| **`a3_coal_k2_u8_dir`** | **63.45 / 63.79 / 67.51** | **−10.28%** | **NO** | **−15.07%** | **64.60 (`62.82..66.64`)** |
| `a3_coal_k2_inlinesub` | 63.93 / **67.89** / 67.99 | −4.53% | YES | −9.62% | 66.82 (`66.25..69.38`) |
| `a3_coal_k3_win8` | 63.91 / **67.62** / 69.73 | −4.89% | YES | −9.97% | 65.69 (`64.53..71.42`) |

*(Against `pool16_shipped_zero` baseline `82.45 ms`, native `full_combined_k2` `67.72 ms` is **−17.86%**; against `lib_scanner_seq` baseline `83.98 ms`, it is **−19.36%**.)*

---

## 4. Disassembly proof (`objdump -d`)

Comparing one unrolled lookup step of shipped `read_block_shipped` (baseline x86-64) against `read_block_combined_k2` (`decode_run_direct_u8`, native BMI2):

```asm
; SHIPPED baseline x86-64 (18 instructions, 8-cyc critical chain)
mov    %r14,%rax
shr    $0x36,%rax              ; 1 cyc
mov    0x0(%r13,%rax,4),%edx   ; 4 cyc L1d load
mov    %edx,%eax
shr    $0x1e,%eax
je     slow
mov    %edx,%ecx               ; extra copy
and    $0x3f,%ecx              ; +1 cyc ON CRITICAL PATH (CSE from live -= e & 63)
shl    %cl,%r14                ; 3 uops, 1-2 cyc (no BMI2)
sub    %ecx,%r12d
mov    %edx,%ecx
shr    $0xb,%ecx               ; SYM1 shift + mask
and    $0x1ff,%ecx
mov    %cx,(%rbx,%r15,2)
shr    $0x14,%edx              ; SYM2 shift + mask
and    $0x1ff,%edx
mov    %dx,0x2(%rbx,%r15,2)
add    %rax,%r15

; NEW decode_run_direct_u8 with BMI2 (14 instructions, 6-cyc critical chain)
mov    %rcx,%rdx
shr    $0x36,%rdx              ; 1 cyc
mov    (%r10,%rdx,4),%edx      ; 4 cyc L1d load
mov    %edx,%esi
shr    $0x1e,%esi
je     slow
shlx   %rdx,%rcx,%rcx          ; 1 uop, 1 cyc BMI2; ZERO `and $0x3f` on chain!
movzbl %dl,%edi                ; parallel with shlx (`live -= (e as u8) as u32`)
sub    %edi,%eax               ; parallel
movzbl %dh,%edi                ; SYM1: zero shift, zero mask
mov    %di,0x0(%r13,%r14,2)    ; zero per-step bounds checks (`[u16; 18]` batch)
shr    $0x10,%edx
movzbl %dl,%edx                ; SYM2: zero mask
mov    %dx,0x2(%r13,%r14,2)
add    %rsi,%r14               ; LLVM proves bk <= 8 < 16 -> zero `& 15`
```

Notice the subtle LLVM codegen discovery in `src/bits.rs:101-105` (`skip_short_shl`): even under `-C target-cpu=native`, writing `self.buf.wrapping_shl(e); self.live -= e & 63;` causes LLVM CSE to feed `and $0x3f` into `shlx` / `shl %cl`, adding **+1 cycle** to every single one of the 13.7M root lookups. Putting `total_bits` in bits `0..7` (`bits 4..7 == 0`) and writing `self.live -= (e as u8) as u32;` breaks that CSE link so `shl %cl` (on baseline, loading `e` straight into `%ecx`) and `shlx %rdx` (on BMI2) fire immediately when the L1d load arrives.

---

## 5. Premise audit on current top held claim (`bwt-digram-return-map`, P2)

Per `explore.toml`'s falsification mandate, we audited `bwt-digram-return-map`'s primary premises (`ideal-permutation-structures/results.md`, `synthesis/second-pass.md`) without touching its hold:

- **Not falsified:** `C_max = 719–799 ms` is cleanly anchored to `character-stage1` (`E_base = 490.3 ms`, `C_base = 577.7 ms`, `W_1w = 1,306 ms`), *not* to `SECMRD`'s cold-store inflation.
- **Two structural constraints for its probe:**
  1. **8-worker scanner coupling:** Once `mtf-fold-build` and `forward-emission` cut worker busy time from `185 -> 152–161 ms`, the 8-worker wall becomes 100% scanner-bound (`178 ms`). Thus `bwt-digram-return-map`'s `+43..+52 ms/w` scan-ideal 8W projection only translates to real 8W wall clock if serial scan `S` also drops (e.g., via `scanner-coalesced-u8-decode`, ~146–155 ms projected).
  2. **Unconditional `F`-bucket elimination vs `tau = 2` phrase reuse:** Unlike RePair (which only contracts pairs appearing `>= 2` times), eliminating an entire `F`-bucket `[C[b], C[b+1])` unconditionally merges adjacent eliminated bytes — including 1-use strings. Any BWT-space bucket/digram constructor must bound dictionary/band fan-out to preserve `F1a`/`F1b`'s L2-resident `edges + dict` footprint.

---

## 6. Caveats

- All timings are from a shared 2-vCPU Haswell Xeon VM (32 KiB L1d / 256 KiB L2, no vPMU), not the i9-10900K. Both Haswell and Comet Lake share the 32 KiB L1d / 256 KiB L2 geometry and the 3-uop `shl %cl` vs 1-uop `shlx` BMI2 split, so relative speedups are expected to transfer, but actual adoption requires the frozen workstation i9-10900K stop test in `frontier/claims/scanner-coalesced-u8-decode.toml`.
- `lib_scanner_seq` ran +5.3% to +5.7% slower than our `shipped_full_zero` mirror (including `Scanner::new` scratch setup and `Result`/overrun checks); all claimed percentages are conservatively anchored to `shipped_full_zero` and `a3_shipped`, not `lib_scanner_seq`.
- Runtime BMI2 dispatch in `src/huff.rs` would require a small, isolated `#[target_feature(enable = "bmi2")]` arch-gated wrapper (matching `src/runs.rs:219,435`), whereas levers 1–3 (`−10.8%` to `−14.0%` of `S`) are 100% safe Rust (`#![forbid(unsafe_code)]`).
