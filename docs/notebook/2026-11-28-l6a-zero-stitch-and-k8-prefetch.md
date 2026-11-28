# Whole-Decoder Evaluation: Zero-Stitch Fused RLE1+CRC (`L6a-on-1312`) and `K=8 + edges[next]` Prefetch Synergy

**Scope & honesty notice:** All timings, RSS readings, and disassembly counts in this note were measured on a shared Linux KVM VM (`Intel(R) Xeon(R) CPU @ 2.30GHz`, 2 vCPUs, 45 MiB L3, `rustc 1.99.1`, `[profile.release]` `opt-level = 3`, `lto = "fat"`, `codegen-units = 1`). **These are VM whole-decoder (`examples/ab.rs`) measurements, NOT physical i9-10900K adoption evidence.**

## Executive Summary

We evaluated two stacked follow-ons on top of PR `#1312`'s bounded-slab four-lane forward walk (`0762c7ea`, `<= 80 KiB/worker` `Arena`, `#![forbid(unsafe_code)]` in `src/open/lanes.rs`) in a 4-arm paired whole-decoder `ab` matrix across both default baseline x86-64 (`base`) and Intel JCC-32B-aligned (`jcc`) builds, L3-warm and L3-cold (`64 MiB` LLC flush between runs), `1W` (`mibz2-workers-lazy`, pinned CPU 0) and `8W` (`mibz2`):

1. **`L6a-on-1312` (`050acb79`, arm `A2`, `K = 4`, `<= 80 KiB/worker`):** Fuses chunked RLE1 + 64 KiB tiled PCLMULQDQ CRC-32 directly onto `#1312`'s unordered `4 KiB` chunks and tails — eliminating the full-block chunk cycle permutation, backward `copy_within` tail stitch, `find_runs_avx2` pre-scan, and `unrle_in_place` staging shift while preserving `#![forbid(unsafe_code)]` in `src/open/lanes.rs` and `s.ll` length across blocks.
   - **Correctness & safety:** `cargo fmt`, 3-way `clippy`, `95/95` unit + integration tests, `4/4` Miri tests (`0 UB`), `0 vzeroupper / 0 ymm` in `Walk::run`, and **`140/140` `AB_SHA256=1` parity checks PASS** across all 7 corpora.
   - **Baseline (`base`) Wikidata:** Beats `#1312` (`A1`) by **`-3.22%` `[0.9508, 0.9850]`** (`1W` warm), **`-3.39%` `[0.9429, 0.9899]`** (`1W` cold), **`-1.93%` `[0.9239, 1.0409]`** (`8W` warm), and **`-4.34%` `[0.9216, 0.9930]`** (`8W` cold; Wall `-6.62% [0.8986, 0.9704]`) — reaching **`-12.34%` to `-16.62%` CPU vs default `A0`** at `0.991x..1.026x` RSS vs `A1` (`1.017x..1.027x` vs `A0`).
   - **JCC-32B control (`jcc`) Wikidata — crucial layout diagnostic:** Under `-x86-align-branch-boundary=32`, `A2` vs `A1` flattens to **`+0.26%` `[0.9861, 1.0193]`** (`1W` warm), **`+1.71%` `[0.9930, 1.0418]`** (`1W` cold), **`-0.30%` `[0.9744, 1.0201]`** (`8W` warm), and **`-0.06%` `[0.9822, 1.0169]`** (`8W` cold) — still **`-10.33%` to `-12.38%` CPU vs `A0`** at `0.996x..1.010x` RSS vs `A1`, with zero disjoint guard regressions across all 12 `text`/`runs`/`onebyte` cells. Disassembly (§4) reveals two concrete causes: (a) `A1_base` had `Walk::run<4>` at `0x12d030` (`mod 32 = 16`) while `A2_base` shifted it to `0x132d20` (`mod 32 = 0`), creating a ~`29 ms` Cascade Lake JCC layout difference on `base`; (b) in-place `decode_owned` still performs bounded free-list eviction copies on colliding `4 KiB` chunks inside a `10.5 KiB` `unrle_crc_chunks_in_place_avx2` body (`5.7x` larger than `unrle_in_place`).
   - **Disposition:** Admitted as follow-on striated build claim `lanes-zero-stitch-l6a-build` (`status = "idea"`, sequenced behind `#1324` `forward-emission-bounded-slab-rescue`) with a frozen `stop_test` explicitly guarding against `base`-only JCC artifacts and requiring genuine JCC-32B / i9 gains after tightening in-place eviction codegen.

2. **`L6a + K=8 + edges[next]` prefetch synergy (`A3_t0` / `A3_t1`, `<= 96 KiB/worker`):** **REJECTED (16/16 Wikidata cells slower than `A2`).**
   - Although the isolated `A7` micro-probe (`docs/notebook/2026-11-28-forward-lanes-mlp-and-prefetch.md`) measured `-5.5%..-18.5%` on `A7` for `K=8 + edges[next]` prefetch, inside the whole `ab` decoder `A3` is **slower than `A2` (`K=4`) in all 8 Wikidata cells (16/16 `T0`/`T1` pairs)** across both `base` (`+1.87%` to `+6.28%`, 3/8 disjoint slower) and `jcc` (`+0.55%` to `+3.38%`, 2/8 disjoint slower).
   - **Disposition:** Discarded cleanly; prototype worktree (`probe/l6a-k8-prefetch`) and temporary binaries deleted without pushing code.

---

## 1. Experimental Arms, Build Profiles & Verification

All 8 binaries (`4 arms × 2 profiles`) were verified to have 8 distinct SHA-256 digests and expected symbol tables (`unrle_crc_chunks_in_place_avx2` absent in `A1` and present in `A2`/`A3`; `Walk::run<8, 1/2>` with `8` `prefetcht0`/`prefetcht1` present in `A3`):

| Arm | Revision / Patch | Feature & Architecture | Bounded Per-Worker `Arena` |
|---|---|---|---|
| **`A0`** | `b28dd818` | Default shipped `OpenCells` (`default = []`) | baseline |
| **`A1`** | `b28dd818` | `--features forward-anchor-lanes` (`#1312`: `K=4`, 4 KiB chunk cycle perm + tail stitch + `unrle_in_place` + `crc32`) | `<= 80 KiB` (`4 × 4 KiB` slabs + `<= 64 KiB` tails) |
| **`A2`** | `050acb79` | `L6a-on-1312` (`--features forward-anchor-lanes`, `K=4`, zero-stitch fused RLE1 + 64 KiB tiled CRC-32; `decode_owned` free-list eviction + `decode_block` disjoint streaming) | `<= 80 KiB` (`4 × 4 KiB` slabs + `<= 64 KiB` tails) |
| **`A3_t0` / `A3_t1`** | `050acb79` + `K=8` + prefetch | `L6a-on-1312` + `K=8` lanes + 1-step `edges[next]` prefetch (`PREFETCHT0` / `PREFETCHT1` helper in `src/rle.rs`, `src/open/lanes.rs` `#![forbid(unsafe_code)]`) | `<= 96 KiB` (`8 × 4 KiB` slabs + `<= 64 KiB` tails) |

Build profiles:
- **`base`:** `cargo build --release --locked --example ab` (default x86-64 baseline).
- **`jcc`:** `RUSTFLAGS="-C llvm-args=-x86-align-branch-boundary=32 -C llvm-args=-x86-align-branch=fused+jcc+jmp" cargo build --release --locked --example ab`.

### Exactness, Miri & 140-Way SHA-256 Parity

- **Unit, integration & Miri:** Both `A2` (`050acb79`) and `A3` passed `4/4` Miri tests (`0 UB`), `0 vzeroupper / 0 ymm` across `Walk::run`, and all unit + integration tests (`95/95`).
- **`AB_SHA256=1` whole-decoder parity (`140/140` PASS):** Every combination of `7` corpora × `2` profiles (`base`, `jcc`) × `2` modes (`1W` `mibz2-workers-lazy`, `8W` `mibz2`) × `5` arms (`A0`, `A1`, `A2`, `A3_t0`, `A3_t1`) produced byte- and SHA-256-identical output:
  - `wikidata.bz2`: `209,700,196 B`, `3f075e30f4f8c73414150655afbf57caa2647c7927e743fc0635b4912d65cdaf` (`20/20`)
  - `text.bz2`: `28,706,026 B`, `2bb242855e9235c4d8acb48014957a7b954297c2cd3ad0c04ed93971c3b5270c` (`20/20`)
  - `runs.bz2`: `20,000,375 B`, `1f82334ee670c5a8c85cd7f0719a1a40b473514dc6375a9dfed92bb6488d0626` (`20/20`)
  - `onebyte.bz2`: `40,000,000 B`, `4a85e306aab98c44a6aba6476a263bd47310aadd05e5313ad28d6dff6aae3592` (`20/20`)
  - `records.bz2`: `19,200,000 B`, `93880bf79d4cb0c1567a7ba58874b0018daa6727c821d2597c8546b769c58cc3` (`20/20`)
  - `pattern.bz2`: `39,200,000 B`, `a1aeba031f968749292b7d098939867e5274c0586becf6df286449981bca87ae` (`20/20`)
  - `random.bz2`: `16,000,000 B`, `71b9e5abf7821af66f22af1d8897bd334f739643d8363836ccf7513cec6076b0` (`20/20`)

---

## 2. Wikidata Paired Matrix (`n = 10` Alternating Interleaved Rounds)

All ratios are paired geometric means with two-sided 95% Student-t confidence intervals (`df = 9`, `t = 2.2622`). Bold indicates a 95% CI excluding `1.0000` (`DISJOINT`).

### 2.1 Median CPU (ms) & Peak RSS (KiB)

| Cell (`wikidata.bz2`, `n=10`) | `A0` CPU / RSS | `A1` (`#1312`) CPU / RSS | `A2` (`050acb79`) CPU / RSS | `A3_t0` CPU / RSS | `A3_t1` CPU / RSS |
|---|---:|---:|---:|---:|---:|
| **`base` `1W` warm** | `948.00` / `24,560` | `836.45` / `25,158` | **`810.20`** / `25,044` | `825.10` / `25,132` | `835.90` / `25,078` |
| **`base` `1W` cold** | `933.90` / `24,566` | `832.95` / `25,116` | **`801.70`** / `25,090` | `835.60` / `25,182` | `837.85` / `25,112` |
| **`base` `8W` warm*** | `961.00` / `52,310` | `859.80` / `55,196` | **`839.80`** / `53,948` | `847.30` / `53,962` | `851.10` / `55,476` |
| **`base` `8W` cold** | `939.00` / `52,076` | `838.90` / `52,496` | **`796.90`** / `53,930` | `813.90` / `53,466` | `822.60` / `53,420` |
| **`jcc` `1W` warm** | `781.50` / `24,876` | **`686.30`** / `24,982` | `689.05` / `25,100` | `696.60` / `25,314` | `712.15` / `25,320` |
| **`jcc` `1W` cold** | `776.60` / `24,858` | **`692.45`** / `25,086` | `704.90` / `25,192` | `721.00` / `25,324` | `712.65` / `25,312` |
| **`jcc` `8W` warm** | `868.80` / `51,918` | `781.05` / `53,476` | **`778.45`** / `53,602` | `785.60` / `55,262` | `787.10` / `53,244` |
| **`jcc` `8W` cold** | `876.95` / `52,544` | `772.05` / `53,850` | **`765.20`** / `54,726` | `786.85` / `53,340` | `786.40` / `53,812` |

*\*Note: `base 8W warm` experienced a host CPU spike during round 4 (`A3_t0=1,476 ms`, `A3_t1=1,053 ms`), widening its paired CIs.*

### 2.2 Paired Ratios & 95% CIs (`CPU` & `RSS`)

| Cell (`wikidata.bz2`, `n=10`) | `A2 / A1` CPU [95% CI] | `A2 / A0` CPU [95% CI] | `A3_t0 / A2` CPU [95% CI] | `A3_t1 / A2` CPU [95% CI] | RSS `A2/A1` / `A2/A0` |
|---|---|---|---|---|---|
| **`base` `1W` warm** | **`-3.22%`** `[0.9508, 0.9850]` | **`-14.83%`** `[0.8299, 0.8740]` | `+1.87%` `[1.0000, 1.0378]` | **`+3.57%`** `[1.0134, 1.0585]` | `0.9949x` / `1.0188x` |
| **`base` `1W` cold** | **`-3.39%`** `[0.9429, 0.9899]` | **`-13.32%`** `[0.8537, 0.8801]` | **`+2.95%`** `[1.0122, 1.0471]` | **`+3.46%`** `[1.0184, 1.0511]` | `1.0036x` / `1.0241x` |
| **`base` `8W` warm** | `-1.93%` `[0.9239, 1.0409]` | **`-12.34%`** `[0.8567, 0.8969]` | `+6.28%` `[0.9439, 1.1967]` | `+1.96%` `[0.9549, 1.0887]` | `0.9911x` / `1.0274x` |
| **`base` `8W` cold** | **`-4.34%`** `[0.9216, 0.9930]` | **`-16.62%`** `[0.8061, 0.8624]` | `+2.29%` `[0.9951, 1.0513]` | `+2.95%` `[0.9966, 1.0634]` | `1.0257x` / `1.0168x` |
| **`jcc` `1W` warm** | `+0.26%` `[0.9861, 1.0193]` | **`-12.38%`** `[0.8583, 0.8944]` | `+1.73%` `[0.9938, 1.0414]` | **`+3.38%`** `[1.0084, 1.0598]` | `1.0033x` / `1.0081x` |
| **`jcc` `1W` cold** | `+1.71%` `[0.9930, 1.0418]` | **`-10.33%`** `[0.8693, 0.9251]` | `+1.07%` `[0.9888, 1.0331]` | **`+2.76%`** `[1.0006, 1.0554]` | `1.0047x` / `1.0147x` |
| **`jcc` `8W` warm** | `-0.30%` `[0.9744, 1.0201]` | **`-10.43%`** `[0.8707, 0.9215]` | `+0.55%` `[0.9741, 1.0378]` | `+2.15%` `[0.9955, 1.0481]` | `0.9955x` / `1.0298x` |
| **`jcc` `8W` cold** | `-0.06%` `[0.9822, 1.0169]` | **`-11.36%`** `[0.8558, 0.9181]` | `+1.77%` `[0.9964, 1.0396]` | `+2.56%` `[0.9854, 1.0673]` | `1.0104x` / `1.0453x` |

Wall-time highlights on `base` L3-cold: `A2 / A1` = **`-5.97%` `[0.9003, 0.9820]`** (`1W` cold) and **`-6.62%` `[0.8986, 0.9704]`** (`8W` cold); on `jcc` `8W` warm/cold: `A2 / A1` = `+0.64%` / `+0.73%`, `A2 / A0` = **`-4.72%` `[0.9127, 0.9947]`** / **`-6.61%` `[0.8894, 0.9806]`**.

---

## 3. Guard Corpora (`text`, `runs`, `onebyte`, `n = 7`, Warm)

`A2` (`050acb79`) shows **zero disjoint regressions** across all 12 guard cells (`3 corpora × 2 profiles × 2 thread modes`) against both `A1` and `A0`, with peak RSS `0.9487x..1.0289x`:

| Cell (`n = 7`, warm) | `A2` med CPU (ms) | `A2 / A1` CPU [95% CI] | `A2 / A0` CPU [95% CI] | `A3_t0 / A2` CPU [95% CI] | RSS `A2/A1` / `A2/A0` |
|---|---:|---|---|---|---|
| `base` `text` `1W` | `231.40` | `-0.12%` `[0.9667, 1.0320]` | `-1.25%` `[0.9379, 1.0398]` | `-0.83%` `[0.9716, 1.0123]` | `0.9930x` / `0.9487x` |
| `base` `text` `8W` | `328.10` | `-0.29%` `[0.9784, 1.0160]` | `+0.54%` `[0.9763, 1.0353]` | `-0.15%` `[0.9804, 1.0170]` | `1.0101x` / `1.0046x` |
| `jcc` `text` `1W` | `221.30` | `-1.85%` `[0.9200, 1.0471]` | `-1.54%` `[0.9160, 1.0583]` | `+1.89%` `[0.9711, 1.0690]` | `0.9566x` / `0.9653x` |
| `jcc` `text` `8W` | `321.90` | `+0.11%` `[0.9672, 1.0362]` | `+0.97%` `[0.9816, 1.0386]` | `-0.52%` `[0.9691, 1.0211]` | `0.9811x` / `0.9997x` |
| `base` `runs` `1W` | `48.60` | `-0.89%` `[0.9684, 1.0142]` | `-0.58%` `[0.9705, 1.0184]` | `+1.21%` `[0.9998, 1.0247]` | `1.0289x` / `1.0257x` |
| `base` `runs` `8W` | `47.30` | `-6.82%` `[0.7907, 1.0982]` | `-0.31%` `[0.9701, 1.0245]` | `+0.92%` `[0.9788, 1.0406]` | `1.0109x` / `0.9845x` |
| `jcc` `runs` `1W` | `48.10` | `+0.64%` `[0.9854, 1.0279]` | `+1.16%` `[0.9932, 1.0303]` | `-0.02%` `[0.9817, 1.0182]` | `1.0221x` / `1.0155x` |
| `jcc` `runs` `8W` | `49.00` | `+0.90%` `[0.9800, 1.0389]` | `-0.85%` `[0.9530, 1.0314]` | `-1.41%` `[0.9541, 1.0188]` | `1.0130x` / `1.0257x` |
| `base` `onebyte` `1W` | `76.60` | `+1.95%` `[0.9661, 1.0758]` | `+1.94%` `[0.9680, 1.0735]` | `-1.79%` `[0.9382, 1.0281]` | `1.0009x` / `0.9983x` |
| `base` `onebyte` `8W` | `72.00` | `+0.50%` `[0.9264, 1.0903]` | `+3.25%` `[0.9108, 1.1704]` | `+1.32%` `[0.8554, 1.2001]` | `1.0130x` / `1.0102x` |
| `jcc` `onebyte` `1W` | `71.60` | `+0.11%` `[0.9655, 1.0381]` | `-1.12%` `[0.9702, 1.0078]` | `+0.53%` `[0.9798, 1.0314]` | `0.9863x` / `0.9846x` |
| `jcc` `onebyte` `8W` | `73.90` | `+1.62%` `[0.9811, 1.0525]` | `+3.29%` `[0.9842, 1.0841]` | `+0.82%` `[0.9510, 1.0689]` | `0.9887x` / `0.9996x` |

Note: On `jcc onebyte 1W`, `A3_t1/A2` regressed `+4.10% [1.0066, 1.0766]` (disjoint slower), while on `jcc runs 8W`, `A3_t1/A2` was `-3.25% [0.9396, 0.9963]` on CPU but `+7.06% [1.0030, 1.1428]` slower on wall.

---

## 4. Disassembly & Mechanistic Root Causes

### 4.1 Why `A3` (`K=8 + edges[next]` prefetch) Lost 16/16 Whole-Decoder Cells

| Symbol (`objdump -d`) | `base` size / insns (`vzu/ymm`, `t0/t1`, `rld/rst`) | `jcc` size / insns (`vzu/ymm`, `t0/t1`, `rld/rst`) |
|---|---|---|
| `A1` / `A2` `Walk::run::<4>` | `2,111 B` / `490` (`0/0`, `0/0`, `81/33`) | `2,209 B` / `505` (`0/0`, `0/0`, `81/33`) |
| `A3` `Walk::run::<8, 0>` | `1,847 B` / `403` (`0/0`, `0/0`, `68/64`) | `1,928 B` / `416` (`0/0`, `0/0`, `68/64`) |
| `A3` `Walk::run::<8, 1>` (`T0`) | `1,877 B` / `400` (`0/0`, `8/0`, `62/63`) | `1,953 B` / `411` (`0/0`, `8/0`, `62/63`) |
| `A3` `Walk::run::<8, 2>` (`T1`) | `1,877 B` / `400` (`0/0`, `0/8`, `62/63`) | `1,953 B` / `411` (`0/0`, `0/8`, `62/63`) |

Four compounding effects turn the isolated `A7` micro-probe win into a universal whole-decoder loss:
1. **32 KiB L1d saturation:** `8 × 4 KiB = 32 KiB` active lane slabs occupies the entire 32 KiB L1d cache (versus `4 × 4 KiB = 16 KiB` for `K=4`), evicting `inline16` (`16.5 KiB` avg) and `lens` during the walk.
2. **Doubled stack stores & rolled loop:** `Walk::run<8, 1/2>` emits `63` stack stores (`rld/rst = 62/63`) vs `K=4`'s `33` (`81/33`) and rolls the `0..8` loop rather than fully unrolling `K=4`.
3. **Interleaved flush order & I-cache footprint:** `8` concurrent lanes interleave chunk flushes across 8 segments (increasing out-of-order collisions for `L6a`'s in-place free-list), and `run_dispatch::<PF>` monomorphizes 24 `Walk::run<1..8, 0..2>` bodies.
4. **Live `A6c -> A7` cache warmth:** In the real decoder, `A6c` builds `edges` (`445 KiB` avg) immediately before `A7` on the same core, leaving `edges` already warm in L2/L3, so extra software prefetch uops mostly add issue-width and L1d eviction overhead.

### 4.2 Why `A2` (`050acb79`) Beat `A1` (`#1312`) by `-3.2%..-4.3%` on `base` Yet Flattened (`-0.3%..+1.7%`) on `jcc`

`nm -nS` on `ab_a1_*` and `ab_a2_*` shows identical `Walk::run::<4>` machine code (`0x83f B` `base`, `0x8a1 B` `jcc`), but two important differences:
1. **JCC-32B layout shift on Cascade Lake `base`:** Inserting `unrle_crc_chunks_in_place_{scalar,avx2}` (`0xce9` + `0x27f1` = `13,530 B`) shifted `Walk::run::<4>` from `0x12d030` (`mod 32 = 16`, `A1_base`) to `0x132d20` (`mod 32 = 0`, `A2_base`). Under `-x86-align-branch-boundary=32`, `A1` dropped `836.45 -> 686.30 ms` (`-150.15 ms`) while `A2` dropped `810.20 -> 689.05 ms` (`-121.15 ms`), accounting for the `~29 ms` (`~3.4%`) `base` gap.
2. **In-place eviction cost & `10.5 KiB` fused loop size:** `ab`'s parallel worker path invokes `decode_owned` (in-place in `out`), not disjoint `decode_block`. When chunks in `out` are out of segment order, `unrle_crc_chunks_in_place_avx2` (`10,225 B` `base` / `10,513 B` `jcc`, vs `unrle_in_place`'s `1,829 B`) evicts colliding `4 KiB` chunks into bounded free-list slabs before reading them — moving roughly the same bytes as `#1312`'s in-place 4 KiB cycle permutation on this 2-vCPU VM where L3 bandwidth is lightly contended (`45 MiB` L3).

### 4.3 Implications for `lanes-zero-stitch-l6a-build`

`050acb79` proves the zero-stitch bounded-slab architecture is safe (`#![forbid(unsafe_code)]` in `src/open/lanes.rs`, `4/4` Miri clean, `140/140` SHA-256 exact, `<= 80 KiB/worker` `Arena`) and eliminates the full-block chunk permutation + tail stitch + `find_runs_avx2` pre-scan, **but `JCC-32B` caught that `050acb79` as-is does not yet beat `#1312` once JCC layout noise is removed on this VM.** Before spending a physical i9-10900K run under `lanes-zero-stitch-l6a-build` (sequenced after `#1324`), the implementation should:
- Shrink `unrle_crc_chunks_in_place_avx2` (`10.5 KiB` -> compact hot loop) and measure eviction-copy count / disjoint-delivery savings under `JCC-32B`;
- Require a genuine JCC-32B-verified VM win over `#1312` (not just a `base` layout shift) before queuing a paired i9-10900K adoption test where 8–10 real cores actually saturate the `20 MiB` L3.
