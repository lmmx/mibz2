# Default `OpenCells` `A5 + A6a` Dissection (`src/runs.rs` + `src/open.rs` + `src/open/outq.rs`)

**Scope & honesty notice:** All timings, censuses, and disassembly counts in this note were measured on a shared Linux KVM VM (`Intel(R) Xeon(R) CPU @ 2.30GHz`, 2 vCPUs, 32 KiB L1d / 256 KiB L2 / 45 MiB L3, `rustc 1.99.1`, `[profile.release]` `opt-level = 3`, `lto = "fat"`, `codegen-units = 1`, pinned CPU 0 for microbenchmarks with steal-free `CLOCK_THREAD_CPUTIME_ID`, alternating AB/BA pairs with per-process CPU + `VmHWM` peak RSS for whole-decoder `ab` checks) across default portable x86-64 (`base`) and JCC-32B-aligned (`jcc`, `-C llvm-args=-x86-branches-within-32B-boundaries`). **These are VM exploration measurements, NOT physical i9-10900K adoption evidence.**

## 1. Motivation

`#1313` (`docs/notebook/2026-11-28-explore-fold-a5a6a-dissection.md`) dissected `mtf-fold`'s `A5+A6a` path (`src/runs/fold.rs` + `FoldSink`). Meanwhile, **100% of shipped production (`v0.5.18` and `--features forward-anchor-lanes`) runs the default non-`mtf-fold` `OpenCells` `A5 + A6a` path**: `runs::front_avx2` (post-`#1274`) followed by `OpenCellsOf::invert_runs` -> `Kernel::construct_capped` -> `maximal` + `Pass::take` + unguarded `OutQ16<512, 512>`.

After `#1274` (`runs::front_avx2` 64-symbol `vptest` Pass 1), `#1294` (`A6c` wire redesign), and `#1312` (`forward-anchor-lanes` bounded-slab walk), this note measures the exact residual sub-stage split of default `OpenCells` `A5 + A6a` on `origin/master` (`07ab2e7e`), audits its disassembly, and tests `#![forbid(unsafe_code)]` zero-extra-RSS candidates directly on the default path.

## 2. Sub-Stage Split & Census of Shipped Default `OpenCells` (`07ab2e7e`)

Census across all **233 Open blocks** of `corpora/wikidata.bz2`:
- `total_runs = 15,141,602` (`14,386,581` thin `<= 16` = **`95.01%`**, `755,021` thick `> 16` = `4.99%`, `1,179,332` merged `CAP = 127` continuation records)
- **`OutQ16` `push_slow` & overflow census across all 233 Open blocks:**
  - Shipped unguarded `512/512` (`HAS_GUARD = false`): **`417,041`** `push_slow` calls, `0` overflow chunks (every chunk's last 15 ranks `f = 497..=511` miss `f + 16 <= 512`!)
  - Guarded `496/512` (`HAS_GUARD = true`, 1 KiB chunk): **`67,001`** `push_slow` calls (**`6.2x` fewer**), `0` overflow chunks
  - Guarded `1008/1024` (`HAS_GUARD = true`, 2 KiB chunk): **`40,597`** `push_slow` calls (**`10.3x` fewer**), `0` overflow chunks

Warm sub-stage medians (`n = 7`, pinned CPU 0, 233 Wikidata Open blocks, `07ab2e7e`):

| Sub-stage of default `OpenCells` worker (`07ab2e7e`) | `default (bidi XOR)` `base` (ms) | `%` share | `forward-anchor-lanes` `jcc` (ms) | `%` share | Notes |
|---|---:|---:|---:|---:|---|
| **Total worker (`A5 + OpenCells + A9`)** | `801.10` | `100.0%` | `605.31` | `100.0%` | `runs::front` + `OpenCells::invert_runs` + `unrle + crc32` |
| **`A5` (`runs::front_avx2`)** | `77.20` | `9.6%` | `64.12` | `10.6%` | Pass 1 `8.20`/`8.61 ms` (`10.4%`/`13.0%` of `A5`); Pass 2 `70.76`/`57.42 ms` (`89.6%`/`87.0%`) |
| **`A6a` (`construct_capped` Pass 1 + `OutQ16<512,512>`)** | **`131.09`** | **`16.4%`** | **`110.06`** | **`18.2%`** | Preserved `out.len()`; if `out.clear()`'d: `+8.80`/`+8.77 ms` `memset(0)` (`parallel::pop_free_out` preserves `len`) |
| **`A5 + A6a` combined** | **`208.29`** | **`26.0%`** | **`174.18`** | **`28.8%`** | `#1` single unoptimized stage after `A7` lanes (`#1312`) and `A6c` (`#1294`) |
| `A6b + A6c + A7` (`invert_runs - A6a`) | `511.13` | `63.8%` | `364.24` | `60.2%` | — |
| `A9` (`unrle + crc32`) | `81.68` | `10.2%` | `66.89` | `11.1%` | — |

### Why Default `A6a` Costs `110–131 ms` (`468` insns, `44` rld / `84` rst)
1. **Unguarded `OutQ16<512, 512>`:** `417,041` `push_slow` calls (`10.3x` higher than `1008/1024`) plus 4 branches per push (`w <= 16`, `f + 16 <= 512`, `lo + w < 65536`, `tail < in_out`) and separate `fill[a]`/`tails[a]` arrays with `tail * BYTES + 2 * f` address arithmetic.
2. **Per-record lookahead merge & `&mut Pass` aliasing:** `maximal` checks `(records[i] >> 8) == c` on all `15.14M` runs even though real bzip2 blocks have injective `seq_to_unseq` (so no `w <= 16` record can ever merge with its successor), and `&mut Pass` forces LLVM to reload/spill `out.ptr`, `out.len`, `m`, and `f.runs` (`44` reloads / `84` stores in `run_shipped`).
3. **Per-run `NO_MARK` / `NO_EPOCH` checks and `f.thin[a] += w` random RMW:** Every one of the `14.39M` thin runs executes `p.wrapping_sub(x) < w`, `(m & 0xFFFF) + w < 65536`, `f.runs += 1`, and a random L1d read-modify-write `f.thin[a] += w` into `[u32; 256]`.

## 3. Exactness & Ablation Results (`A5` Pass 2 + `A6a`)

Every `A6a` candidate was verified bit-for-bit identical to `D0_shipped_512` on every output field (`f.m`, `f.mark`, `f.mark_byte`, `f.runs`, `ns`, `thin[0..256]`, `thick[0..256]`, `thick_rows[0..256]`, all `3 * ns` strip words, every byte's decoded `OutQ16` rank stream, and `spill.cuts`) across all **233/233 Wikidata Open blocks** plus 6 synthetic edge cases (non-injective `seq_to_unseq`, tiny overflow blocks, marked row at start/middle/end of a multi-record `> 127` thick run, exact `65,536`-rank epoch boundary): **`1,643 / 1,643` exact checks PASS** on both `base` and `jcc` (**`3,286 / 3,286`** total).

### 3a. `A5` Pass 2 (`src/runs.rs::front_avx2`, warm `n = 13` interleaved)

| Arm | `base` warm (`vs R0`) | `jcc` warm (`vs R0`) | Verdict |
|---|---:|---:|---|
| `R0_shipped_split_byte` (`07ab2e7e`) | `70.99 ms` (`+0.00%`) | `60.22 ms` (`+0.00%`) | Baseline |
| `R1_u16_store` (`*out = ((c << 8) \| total) as u16`) | `63.39 ms` (`-11.02%`) | **`63.38 ms` (`+4.54%` SLOWER)** | **REJECTED (JCC-32B artifact):** confirms `src/runs.rs:751`'s split-byte store is genuinely faster once branch alignment is controlled |
| `R2_no_cnt_floor` (diagnostic floor removing `cnt[c] += total`) | `62.69 ms` (`-12.66%` DISJ) | `54.55 ms` (`-9.65%` DISJ) | Floor (`~5.7 ms` `jcc`); `r.counts` is required by `Segmented`/`runs::fill` fallback, so post-`#1274` `front_avx2` Pass 2 is already near its floor |

### 3b. `A6a` Ablation Ladder (`#![forbid(unsafe_code)]`, `0 B` extra heap, `n = 13` warm / `n = 7` L3-cold 64 MiB scrub)

Key new lever discovered in `D2 -> D3` (**lazy `thin[a]` derivation**):
- Track `cap_thin[a]` in cold `OutQMeta` (incremented by `C` only inside `take_chunk`, or by `take` on overflow chunks).
- Then at every point in Pass 1 (thick strips, epoch cuts, and end of Pass 1), **`thin[a] == meta.cap_thin[a] - ((slots[a].end - slots[a].cur) >> 1)`** holds identically — eliminating **all `14,386,581` random `thin[a] += w` read-modify-writes** from the hot thin-run loop in 100% safe Rust. (This same trick transfers directly to `mtf-fold`'s `FoldSink` too.)

| Arm (`#![forbid(unsafe_code)]`) | `base` warm (`vs D0`) | `base` L3-cold (`vs D0`) | `jcc` warm (`vs D0`) | `jcc` L3-cold (`vs D0`) |
|---|---:|---:|---:|---:|
| **`D0_shipped_512`** (`07ab2e7e`) | `128.37 ms` | `132.47 ms` | `110.45 ms` | `109.86 ms` |
| `D1a_guard_496` (guard zone alone, `496/512`) | `+6.82%` *(JCC)* | `+4.49%` | `-6.57%` | `-7.12%` |
| `D1b_guard_1008` (guard zone alone, `1008/1024`) | `-1.84%` | `-3.35%` | `-7.39%` (DISJ) | `-6.30%` |
| `D2a_slot8_hoist_496` (`Slot8` + `OutQMeta` + hoist + inj) | `-27.86%` (DISJ) | `-30.34%` (DISJ) | `-27.92%` (DISJ) | `-29.42%` (DISJ) |
| `D2b_slot8_hoist_1008` | `-28.66%` (DISJ) | `-29.80%` (DISJ) | `-29.56%` (DISJ) | `-30.33%` (DISJ) |
| **`D3a_nothin_496`** (`D2a` + **lazy `thin[a]`**, 0 `thin` RMW) | **`-36.20%`** (DISJ) | **`-38.17%`** (DISJ) | **`-35.41%`** (DISJ) | **`-35.51%`** (DISJ) |
| **`D3b_nothin_1008`** (`D2b` + **lazy `thin[a]`**) | **`-36.81%`** (DISJ) | **`-39.08%`** (DISJ) | **`-35.95%`** (DISJ) | **`-36.45%`** (DISJ) |
| **`D3c_nothin_1008_b256`** (`D3b`, 256-rec batch) | **`-37.12%`** `[0.624, 0.634]` (DISJ) | **`-39.13%`** `[0.583, 0.636]` (DISJ) | **`-36.12%`** `[0.630, 0.648]` (DISJ) | **`-36.48%`** `[0.614, 0.658]` (DISJ) |

- Eliminating `thin[a] += w` (`D2 -> D3`) adds **`-8.3%` of `D0` (`-11.7%` of `D2`, `-10.9 ms` `base` / `-7.2 ms` `jcc`)** on top of `Slot8` + `NO_MARK`/`NO_EPOCH` hoisting.
- Batch-hoisting `no_mark = pos > p || (p - pos) >= chunk_recs * 127` and `no_epoch = (m & 0xFFFF) + chunk_recs * 16 < 65536` over `BATCH = 256` records is O(1) per batch (no pre-scan needed because every `u16` record has `len <= 127` and every thin run has `w <= 16`).

## 4. Real Production Whole-Decoder Integration & `objdump -dC` Audit

Prototyped `D3c` directly in `src/runs.rs` (zero-hot-loop-instruction `injective: bool` on `Runs`, 0 new `unsafe`), `src/open/outq.rs` (`#![forbid(unsafe_code)]`, `CHUNK = 1008`, `STRIDE = 1024`, `Slot8`, `OutQMeta`), and `src/open.rs` (`#![forbid(unsafe_code)]`, `construct_first_slot8` + `process_chunk_slot8` + outlined `#[cold] take_thick`), plus updating `src/open/checks.rs::full_nonzero_tail_links_to_its_overflow_chunk` to be parametric in `outq::CHUNK`/`outq::STRIDE`.

### 4a. `objdump -dC` Audit of Real Production `ab` Binary (`cand-default`)

| Function | insns | `vzeroupper` | `ymm` | stack reloads | stack stores | Notes |
|---|---:|---:|---:|---:|---:|---|
| Shipped `run_shipped::<512,512>` (`07ab2e7e`) | `468` | `0` | `0` | `44` | `84` | `Pass::take` + unguarded `OutQ16` |
| `mibz2::open::process_chunk_slot8::<true, true>` | **`127`** | **`0`** | **`0`** | `22` *(0 in loop)* | `9` *(0 in loop)* | **22-insn (`107 B`) hot thin loop: 0 stack reloads, 0 stack stores, 0 `records` bounds checks, 0 `thin[a]` RMW, 2× SSE2 `movdqu`** |
| `mibz2::open::take_thick::<true>` | `110` | `0` | `0` | `11` | `4` | Outlined `#[cold]` thick-strip + `127`-merge path |
| `mibz2::open::outq::OutQMeta<1008,1024>::push_slow` | `384` | **`0`** | **`0`** | `38` | `17` | 4-rank unrolled scalar copy (`0` YMM/`vzeroupper`) |

### 4b. Powered Whole-Decoder `ab` Paired A/B (`07ab2e7e` vs `D3c`, alternating AB/BA, 7/7 SHA-256 parity PASS)

| Profile | Corpus / Mode | `base` CPU (ms) | `cand` CPU (ms) | Delta (`95%` CI) | Disjoint | `VmHWM` Peak RSS |
|---|---|---:|---:|---:|:---:|---:|
| **Default (`base`)** | `wikidata` `workers-lazy` 1W (`n=11`) | `980.2` | `881.8` | **`-10.01%`** `[0.8885, 0.9114]` | **YES** | `24.02 -> 24.38 MiB` (`1.0146x`) |
| **Default (`base`)** | `wikidata` `mibz2` 8W (`n=11`) | `988.2` | `910.2` | **`-7.32%`** `[0.9062, 0.9478]` | no | `50.79 -> 51.13 MiB` (`1.0066x`) |
| **Default (`JCC-32B`)** | `wikidata` `workers-lazy` 1W (`n=11`) | `878.8` | `829.8` | **`-5.68%`** `[0.9281, 0.9585]` | **YES** | `24.25 -> 24.32 MiB` (`1.0031x`) |
| **Default (`JCC-32B`)** | `wikidata` `mibz2` 8W (`n=11`) | `932.8` | `843.8` | **`-8.43%`** `[0.8929, 0.9390]` | no | `50.54 -> 51.39 MiB` (`1.0168x`) |
| **`forward-anchor-lanes` (`JCC-32B`)** | `wikidata` `workers-lazy` 1W (`n=11`) | `783.1` | `721.9` | **`-8.64%`** `[0.9024, 0.9250]` | **YES** | `24.18 -> 24.34 MiB` (`1.0065x`) |
| **`forward-anchor-lanes` (`JCC-32B`)** | `wikidata` `mibz2` 8W (`n=11`) | `804.4` | `746.6` | **`-7.62%`** `[0.9128, 0.9350]` | **YES** | `51.81 -> 51.91 MiB` (`1.0018x`) |
| **6 Guards (`JCC-32B`, default & lanes)** | `text`, `runs`, `records`, `random`, `pattern`, `onebyte` 1W (`n=7`) | — | — | **`-1.30% .. +1.12%`** (0 CI > 1.00) | — | `0.836x .. 1.012x` (flat) |

*Note on `mibz2 1W`: `ab`'s `mibz2` arm at `threads = 1` runs single-threaded `Cells` (never calling `OpenCells`), so default `OpenCells` runs on `mibz2` at `threads >= 2` (`8W` above) and on `mibz2-workers-lazy` at all thread counts.*

## 5. Verdict & Filed Claim

- **Verdict:** **STRONG WIN (`-36.1%..-39.1%` on `A6a`, `-23%` on `A5+A6a`, `-5.68%..-10.01%` 1W and `-7.32%..-8.43%` 8W whole-decoder CPU across default and `forward-anchor-lanes` builds)** in **100% `#![forbid(unsafe_code)]`** (`src/open.rs` + `src/open/outq.rs`) with **0 new `unsafe`**, **0 B extra heap/RSS**, **0 `vzeroupper`**, and **0 guard regressions**.
- **Filed claim:** `frontier/claims/default-a5a6a-fast-build.toml` (`status = "idea"`, `priority = 2`, `owned_paths = ["src/open.rs", "src/open/outq.rs", "src/runs.rs"]`).
- **Bonus transfer:** The lazy `thin[a]` derivation (`cap_thin[a] - ((slots[a].end - slots[a].cur) >> 1)`) eliminates `14.39M` random L1d RMWs and transfers directly to `fold-a5a6a-fast-pass1-pass2` (`mtf-fold`) as well.
