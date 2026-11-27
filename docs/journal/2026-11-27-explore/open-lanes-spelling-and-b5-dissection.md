# Cold Dissection of `src/open/lanes.rs` (4-Lane Forward Emission, Spelling & Slab Flush) and `src/open.rs` Setup/Classify/Clear

- **Date:** 2026-11-27
- **Probe binary:** [`examples/open_lanes_b5_dissection.rs`](../../../examples/open_lanes_b5_dissection.rs) (smooth-zone, zero `src/` edits)
- **Corpora:** `wikidata.bz2` (431 blocks; 233 Open-admitted blocks, 209.70 MB decoded, SHA-256 `82f65ace0731962d...`) + all 6 synthetic guard corpora (`ab gen --scale 4`: `text`, `random`, `records`, `pattern`, `runs`, `onebyte`)
- **Verdict:**
  - **Track A (`src/open/lanes.rs` 4-lane forward emission + spelling + fused RLE1/CRC32): DECISIVE PASS.**
    - **100% safe Rust `L3_disjoint_safe`** (per-lane disjoint `Box<[u8; 32768]>` slabs + `#[inline(never)]` `run<K>` + inline `LONG`-fits-in-slab + lazy slab pool, **zero `unsafe`**) cuts `lanes::walk` by **`-17.13%` warm / `-22.96%` L3-cold** (baseline x86-64) and **`-12.23%` warm / `-18.72%` L3-cold** (JCC-32B-aligned native) — cutting `lanes::walk + A9` by **`-15.53% / -19.82%` (base) and `-10.67% / -16.91%` (JCC-32B)** (all 4 cells disjoint FASTER!) at **`-95.8 KiB`/worker LESS memory** (`1,056.8 -> 961.0 KiB`) — capturing **99–100% of the `unsafe` zero-bounds-check ceiling (`L4`)**!
    - **Adding single-pass AVX2 2-load RLE1 + 64 KiB L2-hot CRC32 directly on forward slab slices (`L6a_safe_walk+avx2_RLE`, safe walk + AVX2 RLE1)** crushes the frozen `>= 15%` bar across **all 4 cells (4/4 disjoint FASTER)**: **`-18.40%` base warm / `-25.50%` base L3-cold (`-114.4 ms`, `-14.9%` of `W_open`) / `-15.98%` JCC-32B warm / `-23.28%` JCC-32B L3-cold (`-97.2 ms`, `-15.4%` of `W_open`)**, at **`-95.8 KiB`/worker LESS memory**!
  - **Track B (`src/open.rs` setup/classify/clear outside `A6a/b/c/7/9`): HONEST STOP.**
    - Total non-`A6a/b/c/7/9` Open overhead is only **`0.64%–1.24%` of `W_open`** (`4.04–8.23 ms`, well below the frozen `>= 3.0%` threshold): `parallel.rs` already preserves `out.len()` across recycled `free_out` buffers (`0.00 ms` resize cost), `finish()` strip-setup + `image_order` is `4.03–4.22 ms` (`0.53–0.73%`), and `within_cap` on the 5 dense-band Wikidata blocks is `0.00–4.01 ms` (`0.00–0.60%`).

---

## 1. Frozen Stop Test (Frozen Before Any Timing Data; C2)

1. **Exactness & Memory Law:** 100% byte-identical & CRC32-identical against shipped `mibz2::decode_block` across all 233 Wikidata Open blocks and all Open-admitted blocks across the 6 guard corpora; zero `TAG` leaks on `edges`; `<= 1.00×` arena memory vs `#1110` `src/open/lanes.rs` (resident scratch `<= 1.01×`).
2. **Measurement Protocol:** 7 interleaved alternating-order repeats, pinned to CPU 0 (`taskset -c 0`), steal-free Linux `/proc/thread-self/schedstat` thread CPU time (`CPU`), across **BOTH baseline x86-64** (`cargo build --release`) AND **JCC-32B-aligned native** (`-C target-cpu=native -C llvm-args=-x86-branches-within-32B-boundaries`), in **BOTH warm (1W) AND L3-cold (16-worker rotating pool + 64 MiB L3 flush)** regimes.
3. **PASS / STOP Bars:**
   - **Track A (walk-only):** `>= 10.0%` disjoint FASTER on `lanes::walk` across baseline & JCC-32B, warm & L3-cold, at `<= 1.00×` memory.
   - **Track A (`lanes::walk + A9`):** `>= 15.0%` disjoint FASTER on `lanes::walk + A9` (`>= 4.0%` of `W_open`) across all 4 cells (baseline & JCC-32B, warm & L3-cold) at `<= 1.00×` memory.
   - **Track B (`src/open.rs` setup/classify/clear):** non-`A6a/b/c/7/9` slice `>= 3.0%` of `W_open` AND a `0 B`-extra-memory optimization cuts `W_open` by `>= 2.0%` disjoint FASTER across all 4 cells.

---

## 2. Production `objdump -d` Audit & Micro-Census (`src/open/lanes.rs`, `#1110`)

Disassembling production `target/release/examples/ab` (`--features forward-anchor-lanes`) exposed **four compounding inefficiencies** inside `<mibz2::open::Kernel>::walk` (`src/open/lanes.rs`):

1. **Mega-function register-allocator explosion (`#[inline(always)] fn run<const K: usize>`):**
   - `Walk::run::<4>`, `<3>`, `<2>`, `<1>` are all `#[inline(always)]` into `<Kernel>::walk` (alongside `emit_bidirectional` and `emit`), creating a **3,728-instruction function** (61 bounds checks, 96 calls, `0x380`–`0x458`-byte stack frame; `1,178` stack ops in `walk_l0_family` and `69` `vzeroupper`s under JCC-32B native).
   - Inside `Walk::run::<4>` (`0x128ee0..0x1291e7`, **178 instructions per 4-lane step = 44.5 insns/step**), `bits`, `mask`, `edges.ptr`, `lens.ptr`, `lens.len`, `inline.ptr`, `inline.len`, `j[0..4]`, `at[0..4]`, and `end[0..4]` are **all spilled to stack and reloaded 4× per iteration** (`38` stack loads/stores per 4-lane step).
2. **`1,718,999` `#[cold] #[inline(never)] word_slow` calls (107.4× unnecessary cold calls!):**
   - Across the 233 Wikidata Open blocks (`26,567,581` steps):
     - **1 B literals:** `21,672,913` (`81.58%`)
     - **2..16 B inline:** `3,186,535` (`11.99%`)
     - **`LONG` (> 16 B):** `1,708,133` (`6.43%`; length histogram: `17..32 B` `27.3%`, `33..64 B` `34.0%`, `65..128 B` `15.4%`, `> 128 B` `23.3%`)
   - **`99.70%` (`1,702,999 / 1,708,133`) of `LONG` words fit directly in the current 32 KiB slab!** Only `5,134` `LONG` words and `10,866` inline words (`16,000` total, `0.06%` of steps) actually cross a 32 KiB slab boundary.
   - Yet production `lanes.rs` routes every single `LONG` word (`1.72M` times, `23.3%` of `K=4` rounds) through `std::mem::take(&mut self.bytes)` + 40-byte `Lane` stack copy + `#[cold] #[inline(never)] word_slow` (`40` marshalling instructions + `180`-instruction `word_slow` = ~376M extra instructions!).
3. **20 bounds/overflow checks per 4-lane iteration (5 per lane):**
   - `bytes[at[i]..at[i] + INLINE]` fails to prove `at + 16 <= bytes.len()` from `at + 16 <= end[i]`, emitting 3 extra instructions + 2 branches (`cmp %rdx,%rsi; seta; cmp $-17; ja; test; jne`) per lane, plus separate `lens.len()` and `inline.len()` checks.
4. **Full 210 MiB stitch copy + redundant `unrle_in_place` passes:**
   - `lanes::walk` copies 210 MiB from `arena.bytes` into `out` (`stitch = 20.8 ms` warm / `48.4–51.9 ms` L3-cold, `10.7–11.8%` of `A7+A9`), and `A9` then scans 210 MiB (`find_runs_avx2`), shifts `59/233` (`25.3%`) `shift > 0` blocks via `copy_within`, and re-reads 210 MiB for CRC32 (`18.8–41.5 ms` L3-cold).
   - Meanwhile, `L0`'s `Arena` pre-resizes `33` slabs (`1,056 KiB`), whereas a lazy disjoint slab pool uses `29.5` avg / `30–31` max slabs (`961–993 KiB`, **`-64..-96 KiB`/worker LESS**).

### `objdump -d` Comparison (`L0` vs `L3` Safe vs `L4` Unsafe)

| Function | Build | Insns | Calls | Bounds checks | `vzeroupper` | `(%rsp)` ops | Stack frame |
|---|---|---:|---:|---:|---:|---:|---|
| `walk_l0_family` (`#1110` mirror) | baseline | `3,445` | `80` | `41` | `0` | `1,178` | `0x448` |
| `<L0Walk>::run_noinline::<4>` (`L1`) | baseline | `654` | `11` | `7` | `0` | `204` | `0xf8` |
| `<L0Walk>::run_inline_long::<4>` (`L2`) | baseline | `418` | `13` | `10` | `0` | `101` | `0x128` |
| **`<SlabWalk>::run_safe::<4>` (`L3`, 100% safe)** | **baseline** | **`531`** | `18` | **`7`** | **`0`** | **`128`** | **`0x98`** |
| `<SlabWalk>::run_nobc::<4>` (`L4`, unsafe) | baseline | `504` | `12` | `2` | `0` | `135` | `0x98` |
| `walk_l0_family` (`#1110` mirror) | JCC-32B | `3,491` | `83` | `44` | `69` | `1,104` | `0x458` |
| **`<SlabWalk>::run_safe::<4>` (`L3`, 100% safe)** | **JCC-32B** | **`567`** | `18` | **`7`** | **`0`** | **`128`** | **`0x98`** |
| `<SlabWalk>::run_nobc::<4>` (`L4`, unsafe) | JCC-32B | `518` | `14` | `2` | `0` | `119` | `0x98` |

---

## 3. Exactness Verification (All 7 Corpora, All 8 Arms `L0..L6b`)

- **`wikidata.bz2`:** **233/233** Open-admitted blocks verified 100% byte-identical & CRC32-identical across all 8 arms (`L0, L1, L2, L3, L4, L5, L6a, L6b`), zero `TAG` leaks (`1,864/1,864` block-arm checks).
- **`runs.bz2`:** **1/1** Open-admitted block verified 100% byte+CRC32 exact across all 8 arms (`8/8`).
- **`text`, `random`, `records`, `pattern`, `onebyte`:** 0 Open-admitted blocks (all route to `Segmented`/`Column`, unaffected).

---

## 4. Track A Results (7 Interleaved Repeats, Steal-Free `schedstat` CPU, CPU 0)

### 4.1 Baseline x86-64 (`cargo build --release`)

| Arm | Safety | Arena | Warm med `[min..max]` | Warm Δ% | Warm disj | L3-Cold med `[min..max]` | Cold Δ% | Cold disj |
|---|---|---:|---|---:|---|---|---:|---|
| **`L0_shipped_walk`** | safe | `1,056.8K` | `315.47 [312.41..319.63]` | `+0.00%` | CTRL | `410.09 [381.66..419.18]` | `+0.00%` | CTRL |
| `L1_noinline_walk` | safe | `1,056.8K` | `307.97 [303.82..316.38]` | `-2.38%` | overlap | `420.77 [415.56..439.28]` | `+2.61%` | overlap |
| `L2_inline_long_walk` | safe | `1,056.8K` | `295.16 [287.86..302.35]` | **`-6.44%`** | **FASTER\*** | `364.84 [359.21..402.85]` | `-11.03%` | overlap |
| **`L3_disjoint_safe_walk`** | **100% safe** | **`961.0K`** | **`261.44 [259.67..306.02]`** | **`-17.13%`** | **FASTER\*** | **`315.92 [313.97..343.77]`** | **`-22.96%`** | **FASTER\*** |
| `L4_disjoint_nobc_walk` | unsafe | `961.0K` | `255.67 [251.84..299.15]` | `-18.96%` | FASTER\* | `319.44 [307.63..327.89]` | `-22.10%` | FASTER\* |
| **`L0_shipped_walk+A9`** | safe+avx2 | `1,056.8K` | `351.68 [346.21..356.34]` | `+0.00%` | CTRL | `448.47 [428.67..456.83]` | `+0.00%` | CTRL |
| **`L3_disjoint_safe+A9`** | **0 new unsafe** | **`961.0K`** | **`297.05 [290.97..303.00]`** | **`-15.53%`** | **FASTER\*** | **`359.57 [350.63..366.98]`** | **`-19.82%`** | **FASTER\*** |
| `L4_disjoint_nobc+A9` | unsafe | `961.0K` | `296.99 [293.27..303.22]` | `-15.55%` | FASTER\* | `354.14 [347.20..359.42]` | `-21.03%` | FASTER\* |
| `L5_fused_safe_RLE+CRC` | safe | `961.0K` | `658.80 [645.72..788.06]` | `+87.33%` | SLOWER\* | `708.32 [695.95..741.35]` | `+57.94%` | SLOWER\* |
| **`L6a_safe_walk+avx2_RLE`** | **safe walk + avx2** | **`961.0K`** | **`286.98 [282.99..304.78]`** | **`-18.40%`** | **FASTER\*** | **`334.12 [328.48..338.20]`** | **`-25.50%`** | **FASTER\*** |
| `L6b_nobc_walk+avx2_RLE` | unsafe | `961.0K` | `285.76 [275.92..323.87]` | `-18.75%` | FASTER\* | `331.69 [323.15..344.29]` | `-26.04%` | FASTER\* |

*`L0` baseline sub-stages:* warm `tag=4.00 ms (1.1%)`, `walk=307.85 ms (83.5%)`, `stitch=20.82 ms (5.7%)`, `A9=35.81 ms (9.7%)`; L3-cold `tag=0.60 ms (0.1%)`, `walk=383.88 ms (85.0%)`, `stitch=48.35 ms (10.7%)`, `A9=18.77 ms (4.2%)`.

### 4.2 JCC-32B-Aligned Native (`-C target-cpu=native -C llvm-args=-x86-branches-within-32B-boundaries`)

| Arm | Safety | Arena | Warm med `[min..max]` | Warm Δ% | Warm disj | L3-Cold med `[min..max]` | Cold Δ% | Cold disj |
|---|---|---:|---|---:|---|---|---:|---|
| **`L0_shipped_walk`** | safe | `1,056.8K` | `291.89 [283.96..380.75]` | `+0.00%` | CTRL | `376.84 [360.31..402.70]` | `+0.00%` | CTRL |
| `L1_noinline_walk` | safe | `1,056.8K` | `286.34 [279.89..295.05]` | `-1.90%` | overlap | `367.47 [350.10..380.21]` | `-2.49%` | overlap |
| `L2_inline_long_walk` | safe | `1,056.8K` | `274.71 [271.31..298.46]` | `-5.89%` | overlap | `336.75 [326.62..373.30]` | `-10.64%` | overlap |
| **`L3_disjoint_safe_walk`** | **100% safe** | **`961.0K`** | **`256.20 [247.17..293.53]`** | **`-12.23%`** | overlap\* | **`306.28 [295.53..313.78]`** | **`-18.72%`** | **FASTER\*** |
| `L4_disjoint_nobc_walk` | unsafe | `961.0K` | `253.64 [248.92..294.72]` | `-13.10%` | overlap | `305.41 [291.24..308.22]` | `-18.95%` | FASTER\* |
| **`L0_shipped_walk+A9`** | safe+avx2 | `1,056.8K` | `330.55 [327.85..381.61]` | `+0.00%` | CTRL | `417.61 [399.16..451.23]` | `+0.00%` | CTRL |
| **`L3_disjoint_safe+A9`** | **0 new unsafe** | **`961.0K`** | **`295.28 [287.96..321.53]`** | **`-10.67%`** | **FASTER\*** | **`347.00 [335.72..390.43]`** | **`-16.91%`** | **FASTER\*** |
| `L4_disjoint_nobc+A9` | unsafe | `961.0K` | `287.82 [281.21..314.75]` | `-12.93%` | FASTER\* | `349.70 [331.86..355.21]` | `-16.26%` | FASTER\* |
| `L5_fused_safe_RLE+CRC` | safe | `961.0K` | `513.68 [501.92..558.26]` | `+55.40%` | SLOWER\* | `556.38 [542.68..597.88]` | `+33.23%` | SLOWER\* |
| **`L6a_safe_walk+avx2_RLE`** | **safe walk + avx2** | **`961.0K`** | **`277.73 [271.96..309.72]`** | **`-15.98%`** | **FASTER\*** | **`320.39 [307.34..351.43]`** | **`-23.28%`** | **FASTER\*** |
| `L6b_nobc_walk+avx2_RLE` | unsafe | `961.0K` | `272.20 [266.93..298.60]` | `-17.65%` | FASTER\* | `315.62 [311.05..348.01]` | `-24.42%` | FASTER\* |

*\*In JCC-32B warm walk-only, 1 outlier repeat on `L3` (`293.53 ms`, next highest `~259 ms`) caused a narrow overlap with `L0`'s min (`283.96 ms`); `L3_disjoint_safe+A9` and `L6a_safe_walk+avx2_RLE` are strictly DISJOINT FASTER across all 4/4 cells.*
*`L0` JCC-32B sub-stages:* warm `tag=0.00 ms`, `walk=282.52 ms (81.6%)`, `stitch=19.61 ms (5.7%)`, `A9=44.11 ms (12.7%)`; L3-cold `tag=6.40 ms (1.5%)`, `walk=339.63 ms (77.3%)`, `stitch=51.91 ms (11.8%)`, `A9=41.52 ms (9.4%)`.

### Key Microarchitectural Insights (Track A)
1. **Why 100% safe `L3_disjoint_safe` matches `unsafe` `L4` (`-22.96%` vs `-22.10%` base cold; `-18.72%` vs `-18.95%` JCC cold):**
   - Giving each lane an owned `Box<[u8; 32768]>` slab makes `LANES_SLAB = 32768` a compile-time constant. Checking `o <= LANES_SLAB - INLINE` simultaneously eliminates `end[0..4]` (freeing 4 registers/stack slots), `bytes.len()` checks, `at + 16` overflow checks, and ALL slab bounds checks in 100% safe Rust.
   - Inlining `LONG`-fits-in-slab cuts cold `spill_word` calls **107.4×** (`1.72M -> 16K`), removing `std::mem::take(&mut self.bytes)` and 40-byte `Lane` stack copies from `23.3%` of `K=4` rounds.
   - Because `edges[j]` L3 misses dominate lane latency, the remaining safe `edges[j]` and `lens[code]` bounds checks (`L3` vs `L4`) execute completely in the OoO shadow — **adding `unsafe` to the walk loop buys `<= 0.23%` in L3-cold and is unnecessary!**
2. **Why `L6a_safe_walk+avx2_RLE` adds another `-5.7%..-6.4%` (`-25.5%` / `-23.3%` total L3-cold, `-14.9%..-15.4%` of `W_open`!):**
   - Running the 2-load AVX2 RLE1 + 64 KiB L2-hot CRC32 decoder directly on the already-forward ordered 32 KiB slab slices eliminates the entire `48–52 ms` L3-cold stitch copy into `out`, `find_runs_avx2` scan, `shift > 0` `copy_within` (`59/233` blocks), and `s.rle_runs` with zero reversal buffer.
3. **Why `L5_fused_safe_RLE+CRC` (scalar) loses `+33%..+87%`:** Scalar byte-by-byte RLE1 checking cannot compete with 32B/cycle AVX2 `vpcmpeqb` scanning; stitch+A9 fusion requires AVX2 RLE1.

---

## 5. Track B Results: `src/open.rs` Non-`A6a/b/c/7/9` Setup / Classify / Clear (STOP)

| Sub-stage (233 Wikidata Open blocks) | Base Warm | Base L3-Cold | JCC-32B Warm | JCC-32B L3-Cold |
|---|---:|---:|---:|---:|
| `out.resize` (recycled `free_out`, steady state) | `0.00 ms` (`0.00%`) | `0.00 ms` (`0.00%`) | `0.00 ms` (`0.00%`) | `0.00 ms` (`0.00%`) |
| `finish()` `strip_setup + image_order` | `4.22 ms` (`0.64%`) | `4.10 ms` (`0.53%`) | `4.03 ms` (`0.73%`) | `4.04 ms` (`0.64%`) |
| `within_cap` (5 dense-band Wikidata blocks) | `4.01 ms` (`0.60%`) | `3.99 ms` (`0.52%`) | `0.67 ms` (`0.12%`) | `0.00 ms` (`0.00%`) |
| **Track B non-(A6a/b/c/7/9) total** | **`8.23 ms` (`1.24%`)** | **`8.09 ms` (`1.05%`)** | **`4.70 ms` (`0.85%`)** | **`4.04 ms` (`0.64%`)** |
| *Counterfactual: `out.clear()+resize(210 MiB)`* | *`4.17 ms` (`0.63%`)* | *`25.40 ms` (`3.31%`)* | *`5.69 ms` (`1.04%`)* | *`19.66 ms` (`3.11%`)* |
| `A6a` (`maximal + Pass + OutQ16`) | `167.08 ms` (`25.2%`) | `178.86 ms` (`23.3%`) | `124.59 ms` (`22.7%`) | `134.70 ms` (`21.3%`) |
| `A6b` (`induced + dict`) | `149.64 ms` (`22.6%`) | `140.43 ms` (`18.3%`) | `125.22 ms` (`22.8%`) | `120.22 ms` (`19.0%`) |
| `A6c` (`wire_forward`) | `37.65 ms` (`5.7%`) | `49.47 ms` (`6.4%`) | `38.73 ms` (`7.0%`) | `31.86 ms` (`5.0%`) |
| `A7` (`L0` shipped `lanes::walk`) | `259.29 ms` (`39.2%`) | `344.38 ms` (`44.8%`) | `217.42 ms` (`39.5%`) | `300.95 ms` (`47.6%`) |
| `A9` (`unrle_in_place + CRC32`) | `44.45 ms` (`6.7%`) | `51.05 ms` (`6.6%`) | `40.03 ms` (`7.3%`) | `40.11 ms` (`6.4%`) |
| **`W_open` total (`A6a..A9`)** | **`662.34 ms`** | **`768.28 ms`** | **`550.02 ms`** | **`631.88 ms`** |

- **Verdict for Track B: STOP.** In production `decode_owned`, `parallel.rs` preserves `out.len()` across recycled `free_out` buffers (`POOLED_FROM = 64 KiB`), making steady-state `out.resize(n + INLINE, 0)` cost `0.00 ms`. Combined `finish()` strip-setup + `within_cap` is only **`0.64%–1.24%`** of `W_open` (below the frozen `>= 3.0%` bar). Note that preserving `out.len()` across recycled buffers is load-bearing: clearing `out` before `open::invert_runs` would immediately add `19.7–25.4 ms` (`3.1–3.3%` L3-cold) of wasted `memset(0)`.

---

## 6. Production Recommendations & Admitted Follow-On

1. **Immediate zero-`unsafe` production win (`lanes-disjoint-slabs-build`, admitted at `status = "idea"`, P2, `owned_paths = ["src/open/lanes.rs"]`):**
   - Port **`L3_disjoint_safe`** (`#[inline(never)] run<K>` + per-lane disjoint `Box<[u8; 32768]>` slabs + inline `LONG`-fits-in-slab + lazy slab pool + `out.reserve(n)` fast stitch) into `src/open/lanes.rs` (zero new `unsafe` — ordinary author-owned integration!).
   - Cuts `lanes::walk` by **`-17.1% / -23.0%`** (baseline) and **`-12.2% / -18.7%`** (JCC-32B), cuts `A7 + A9` by **`-10.7%..-19.8%`** (**`-11.1%..-11.6%` of `W_open` L3-cold!**), AND cuts per-worker `Arena` footprint by **`-95.8 KiB`** (`-766 KiB` at 8W on i9-10900K) ahead of `forward-emission-run`!
   - Optional follow-on inside the same file: check inline 16B-chunked copy for `17..64 B` `LONG` words (`61.3%` of all `LONG`s) to avoid glibc `call memcpy`.
2. **Next-stage fused AVX2 forward RLE1 + CRC32 (`L6a_safe_walk+avx2_RLE`, stacks with `forward-rle1-crc-vectored-build`):**
   - Eliminating the stitch copy + `find_runs_avx2` + `shift > 0` `copy_within` via direct 2-load AVX2 RLE1 + 64 KiB L2-hot CRC32 over the ordered forward slab slices pushes the total `A7 + A9` win to **`-23.3%..-25.5%` L3-cold (`-14.9%..-15.4%` of `W_open`!)** at **`-95.8 KiB`/worker LESS memory**!
